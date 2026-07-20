// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! TOFU persistence of host-key fingerprints under XDG.
//!
//! Line-oriented format (0o600):
//! `host:port <fingerprint_sha256>`
//!
//! # Concurrency (G-PAR-49)
//!
//! Multi-host fan-out can run N first-connect TOFU writes against the same file.
//! Every mutating path takes an exclusive flock on a sibling `*.lock` file,
//! reloads disk state, merges, then atomic-persists — same pattern as
//! [`crate::vps::save`] for `config.toml`.

use crate::errors::{SshCliError, SshCliResult};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Constant-time equality for fingerprint bytes (G-SEC-05).
///
/// Host-key fingerprints are not high-entropy passwords, but TOFU comparison
/// still prefers data-independent timing so a local co-tenant cannot learn
/// which prefix mismatched via timing. Length differences return `false`
/// immediately (fixed-format SHA-256 hex strings share a length in practice).
#[must_use]
fn fingerprints_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    // Prevent the compiler from short-circuit optimizing the loop away.
    std::hint::black_box(diff) == 0
}

/// Map of host:port → fingerprint.
#[derive(Debug, Default, Clone)]
pub struct KnownHosts {
    entries: BTreeMap<String, String>,
    path: PathBuf,
}

/// RAII exclusive lock on `known_hosts` sibling lock file (G-PAR-49).
struct HostsFileLock {
    file: File,
}

impl HostsFileLock {
    /// Lock path: `<known_hosts>.lock` (e.g. `known_hosts.lock`).
    fn acquire(kh_path: &Path) -> SshCliResult<Self> {
        let lock_path = {
            let mut os = kh_path.as_os_str().to_owned();
            os.push(".lock");
            PathBuf::from(os)
        };
        if let Some(parent_dir) = lock_path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(SshCliError::Io)?;
        // Best-effort secret mode on lock file.
        let _ = crate::fs_perm::set_secret_file_mode(&lock_path);
        fs2::FileExt::lock_exclusive(&file).map_err(SshCliError::Io)?;
        Ok(Self { file })
    }
}

impl Drop for HostsFileLock {
    fn drop(&mut self) {
        let _ = fs2::FileExt::unlock(&self.file);
    }
}

impl KnownHosts {
    /// Canonical key `host:port`.
    #[must_use]
    pub fn key(host: &str, port: u16) -> String {
        format!("{host}:{port}")
    }

    /// Loads the file (empty if missing). Does **not** take the flock.
    pub fn load(path: PathBuf) -> SshCliResult<Self> {
        let mut entries = BTreeMap::new();
        if path.exists() {
            let text = crate::paths::read_text_capped(
                &path,
                crate::paths::MAX_KNOWN_HOSTS_BYTES,
            )?;
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let mut parts = line.split_whitespace();
                if let (Some(k), Some(fp)) = (parts.next(), parts.next()) {
                    entries.insert(k.to_string(), fp.to_string());
                }
            }
        }
        Ok(Self { entries, path })
    }

    /// Default path `config_dir/known_hosts` next to `config.toml`.
    #[must_use]
    pub fn path_beside_config(config_toml: &Path) -> PathBuf {
        config_toml
            .parent()
            .map(|p| p.join(crate::constants::KNOWN_HOSTS_FILE_NAME))
            .unwrap_or_else(|| PathBuf::from(crate::constants::KNOWN_HOSTS_FILE_NAME))
    }

    /// Looks up a stored fingerprint (in-memory only).
    #[must_use]
    pub fn get(&self, host: &str, port: u16) -> Option<&str> {
        self.entries
            .get(&Self::key(host, port))
            .map(String::as_str)
    }

    /// Inserts or updates and persists under exclusive flock (G-PAR-49).
    ///
    /// Reloads disk state under the lock so concurrent multi-host first-connect
    /// writers merge instead of last-write-wins.
    pub fn store(&mut self, host: &str, port: u16, fingerprint: &str) -> SshCliResult<()> {
        let _lock = HostsFileLock::acquire(&self.path)?;
        self.reload_from_disk_unlocked()?;
        self.entries
            .insert(Self::key(host, port), fingerprint.to_string());
        self.persist_unlocked()
    }

    fn reload_from_disk_unlocked(&mut self) -> SshCliResult<()> {
        let fresh = Self::load(self.path.clone())?;
        self.entries = fresh.entries;
        Ok(())
    }

    fn persist_unlocked(&self) -> SshCliResult<()> {
        if let Some(parent_dir) = self.path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }
        let mut body = String::new();
        body.push_str("# ssh-cli known_hosts (TOFU)\n");
        for (k, v) in &self.entries {
            body.push_str(k);
            body.push(' ');
            body.push_str(v);
            body.push('\n');
        }

        let parent_dir = self
            .path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let mut tmp = tempfile::NamedTempFile::new_in(&parent_dir).map_err(SshCliError::Io)?;
        use std::io::Write;
        tmp.write_all(body.as_bytes())?;
        tmp.as_file().sync_data()?;
        tmp.persist(&self.path)
            .map_err(|e| SshCliError::Io(e.error))?;

        crate::fs_perm::set_secret_file_mode(&self.path)?;
        Ok(())
    }
}

/// Verify TOFU fingerprint under exclusive flock (G-PAR-49 / G-TLS-10).
///
/// - Sem entrada: aceita e grava (TOFU).
/// - Com entrada igual: aceita.
/// - Com entrada diferente: recusa, a menos que `replace` seja true.
///
/// Reloads from disk under the lock so concurrent multi-host first-connect
/// sees peers' writes before deciding.
///
/// # Errors
/// Returns an error if the host key changed and replacement was not allowed, or if persistence fails.
pub fn verify_tofu(
    kh: &mut KnownHosts,
    host: &str,
    port: u16,
    fingerprint: &str,
    replace: bool,
) -> SshCliResult<bool> {
    let _lock = HostsFileLock::acquire(&kh.path)?;
    kh.reload_from_disk_unlocked()?;
    match kh.get(host, port).map(str::to_string) {
        None => {
            kh.entries
                .insert(KnownHosts::key(host, port), fingerprint.to_string());
            kh.persist_unlocked()?;
            Ok(true)
        }
        Some(existing) if fingerprints_eq(&existing, fingerprint) => Ok(true),
        Some(existing) if replace => {
            tracing::warn!(
                host,
                port,
                old = %existing,
                novo = %fingerprint,
                "replacing host key (--replace-host-key)"
            );
            kh.entries
                .insert(KnownHosts::key(host, port), fingerprint.to_string());
            kh.persist_unlocked()?;
            Ok(true)
        }
        Some(existing) => Err(SshCliError::HostKeyChanged {
            host: host.to_string(),
            port,
            expected: existing,
            obtained: fingerprint.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use tempfile::TempDir;

    #[test]
    fn fingerprints_eq_matches_and_rejects() {
        assert!(fingerprints_eq("abc", "abc"));
        assert!(!fingerprints_eq("abc", "abd"));
        assert!(!fingerprints_eq("abc", "ab"));
        assert!(!fingerprints_eq("ab", "abc"));
    }

    #[test]
    fn tofu_stores_and_accepts_same() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("known_hosts");
        let mut kh = KnownHosts::load(path).unwrap();
        assert!(verify_tofu(&mut kh, "h", 22, "fp1", false).unwrap());
        assert!(verify_tofu(&mut kh, "h", 22, "fp1", false).unwrap());
    }

    #[test]
    fn tofu_rejects_change() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("known_hosts");
        let mut kh = KnownHosts::load(path).unwrap();
        verify_tofu(&mut kh, "h", 22, "fp1", false).unwrap();
        let err = verify_tofu(&mut kh, "h", 22, "fp2", false).unwrap_err();
        assert!(matches!(err, SshCliError::HostKeyChanged { .. }));
    }

    #[test]
    fn tofu_replaces_with_flag() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("known_hosts");
        let mut kh = KnownHosts::load(path).unwrap();
        verify_tofu(&mut kh, "h", 22, "fp1", false).unwrap();
        assert!(verify_tofu(&mut kh, "h", 22, "fp2", true).unwrap());
        assert_eq!(kh.get("h", 22), Some("fp2"));
    }

    /// G-PAR-49 / G-PAR-54: concurrent first-connect TOFU must not drop entries.
    #[test]
    fn concurrent_store_merges_both_hosts() {
        let tmp = TempDir::new().unwrap();
        let path = Arc::new(tmp.path().join("known_hosts"));
        let barrier = Arc::new(Barrier::new(2));
        let p1 = Arc::clone(&path);
        let p2 = Arc::clone(&path);
        let b1 = Arc::clone(&barrier);
        let b2 = Arc::clone(&barrier);

        let t1 = thread::spawn(move || {
            let mut kh = KnownHosts::load((*p1).clone()).unwrap();
            b1.wait();
            verify_tofu(&mut kh, "alpha.example", 22, "fp-alpha", false).unwrap();
        });
        let t2 = thread::spawn(move || {
            let mut kh = KnownHosts::load((*p2).clone()).unwrap();
            b2.wait();
            verify_tofu(&mut kh, "beta.example", 22, "fp-beta", false).unwrap();
        });
        t1.join().unwrap();
        t2.join().unwrap();

        let final_kh = KnownHosts::load((*path).clone()).unwrap();
        assert_eq!(final_kh.get("alpha.example", 22), Some("fp-alpha"));
        assert_eq!(final_kh.get("beta.example", 22), Some("fp-beta"));
        assert_eq!(final_kh.entries.len(), 2);
    }
}
