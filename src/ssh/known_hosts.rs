// SPDX-License-Identifier: MIT OR Apache-2.0
//! TOFU persistence of host-key fingerprints under XDG.
//!
//! Line-oriented format (0o600):
//! `host:port <fingerprint_sha256>`

use crate::erros::{SshCliError, SshCliResult};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Map of host:port → fingerprint.
#[derive(Debug, Default, Clone)]
pub struct KnownHosts {
    entries: BTreeMap<String, String>,
    path: PathBuf,
}

impl KnownHosts {
    /// Canonical key `host:port`.
    #[must_use]
    pub fn key(host: &str, port: u16) -> String {
        format!("{host}:{port}")
    }

    /// Loads the file (empty if missing).
    pub fn load(path: PathBuf) -> SshCliResult<Self> {
        let mut entries = BTreeMap::new();
        if path.exists() {
            let text = std::fs::read_to_string(&path)?;
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
            .map(|p| p.join("known_hosts"))
            .unwrap_or_else(|| PathBuf::from("known_hosts"))
    }

    /// Looks up a stored fingerprint.
    #[must_use]
    pub fn get(&self, host: &str, port: u16) -> Option<&str> {
        self.entries
            .get(&Self::key(host, port))
            .map(String::as_str)
    }

    /// Inserts or updates and persists atomically.
    pub fn store(&mut self, host: &str, port: u16, fingerprint: &str) -> SshCliResult<()> {
        self.entries
            .insert(Self::key(host, port), fingerprint.to_string());
        self.persist()
    }

    fn persist(&self) -> SshCliResult<()> {
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

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&self.path, perms)?;
        }
        Ok(())
    }
}

/// Verifica fingerprint TOFU.
///
/// - Sem entrada: aceita e grava (TOFU).
/// - Com entrada igual: aceita.
/// - Com entrada diferente: recusa, a menos que `substituir` seja true.
pub fn verificar_tofu(
    kh: &mut KnownHosts,
    host: &str,
    port: u16,
    fingerprint: &str,
    substituir: bool,
) -> SshCliResult<bool> {
    match kh.get(host, port) {
        None => {
            kh.store(host, port, fingerprint)?;
            Ok(true)
        }
        Some(existente) if existente == fingerprint => Ok(true),
        Some(existente) if substituir => {
            tracing::warn!(
                host,
                port,
                antigo = %existente,
                novo = %fingerprint,
                "substituindo host key (--replace-host-key)"
            );
            kh.store(host, port, fingerprint)?;
            Ok(true)
        }
        Some(existente) => Err(SshCliError::HostKeyChanged {
            host: host.to_string(),
            port,
            expected: existente.to_string(),
            obtained: fingerprint.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn tofu_stores_and_accepts_same() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("known_hosts");
        let mut kh = KnownHosts::load(path).unwrap();
        assert!(verificar_tofu(&mut kh, "h", 22, "fp1", false).unwrap());
        assert!(verificar_tofu(&mut kh, "h", 22, "fp1", false).unwrap());
    }

    #[test]
    fn tofu_rejects_change() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("known_hosts");
        let mut kh = KnownHosts::load(path).unwrap();
        verificar_tofu(&mut kh, "h", 22, "fp1", false).unwrap();
        let err = verificar_tofu(&mut kh, "h", 22, "fp2", false).unwrap_err();
        assert!(matches!(err, SshCliError::HostKeyChanged { .. }));
    }

    #[test]
    fn tofu_replaces_with_flag() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("known_hosts");
        let mut kh = KnownHosts::load(path).unwrap();
        verificar_tofu(&mut kh, "h", 22, "fp1", false).unwrap();
        assert!(verificar_tofu(&mut kh, "h", 22, "fp2", true).unwrap());
        assert_eq!(kh.get("h", 22), Some("fp2"));
    }
}
