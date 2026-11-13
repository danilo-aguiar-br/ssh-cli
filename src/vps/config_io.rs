// SPDX-License-Identifier: MIT OR Apache-2.0
// G-UNSAFE-10: config path/load/save/permissions extracted from monolith vps/mod (SRP).
#![forbid(unsafe_code)]
//! Atomic TOML config I/O under XDG (load/save/flock/0o600).

pub(crate) use super::config_load::load;
use super::model::VpsRecord;
use crate::errors::{SshCliError, SshCliResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Full configuration file.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    /// File schema version.
    #[serde(default)]
    pub schema_version: u32,
    /// Host map keyed by VPS name.
    #[serde(default)]
    pub hosts: BTreeMap<String, VpsRecord>,
}

/// Resolves the config file path from an optional override.
///
/// Takes `Option<&Path>` (not `Option<PathBuf>`) so callers can share one
/// override without cloning — ownership of the path stays with the caller.
///
/// # Errors
/// Returns the errors of [`default_config_path`] when no override is given.
pub fn resolve_config_path(override_path: Option<&Path>) -> SshCliResult<PathBuf> {
    match override_path {
        Some(p) => {
            if p.is_dir() {
                return Ok(p.join(crate::constants::CONFIG_FILE_NAME));
            }
            if p.extension().and_then(|e| e.to_str())
                == Some(crate::constants::CONFIG_FILE_EXTENSION)
            {
                return Ok(p.to_path_buf());
            }
            Ok(p.join(crate::constants::CONFIG_FILE_NAME))
        }
        None => default_config_path(),
    }
}

/// Returns the config file path under XDG (`--config-dir` wins at call sites).
///
/// G-AUD-12: no `SSH_CLI_HOME` env store — use `--config-dir` for overrides.
///
/// # Errors
/// Returns an error when no XDG configuration directory can be determined.
pub fn default_config_path() -> SshCliResult<PathBuf> {
    Ok(crate::paths::xdg_config_dir()?.join(crate::constants::CONFIG_FILE_NAME))
}

/// Winning configuration layer (doctor).
#[derive(Debug, Clone)]
pub struct ConfigLayer {
    /// Layer name.
    pub name: &'static str,
    /// Resolved path.
    pub path: PathBuf,
}

/// Resolves and describes the winning config layer.
///
/// # Errors
/// Returns the errors of [`resolve_config_path`] or [`default_config_path`].
pub fn winning_layer(override_path: Option<&Path>) -> SshCliResult<ConfigLayer> {
    if override_path.is_some() {
        return Ok(ConfigLayer {
            name: "--config-dir",
            path: resolve_config_path(override_path)?,
        });
    }
    Ok(ConfigLayer {
        name: "XDG ProjectDirs",
        path: default_config_path()?,
    })
}

/// Writes bytes to `path` atomically (tempfile + fsync + rename + 0o600).
///
/// `export -o` may target any user folder: only the levels created here are made `0o700`.
///
/// # Errors
/// Returns an I/O error when the parent directories, the temporary file, the rename or the
/// permission change fail.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> SshCliResult<()> {
    if let Some(parent_dir) = path.parent() {
        crate::fs_perm::create_missing_private_dirs(parent_dir)?;
    }
    let parent_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut tmp = tempfile::NamedTempFile::new_in(&parent_dir)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_data()?;
    tmp.persist(path).map_err(|e| SshCliError::Io(e.error))?;
    apply_permissions_600(path)?;
    sync_parent_dir(&parent_dir);
    Ok(())
}

/// Flushes the directory entry of a file that was just renamed into `parent_dir`.
///
/// G-SCP-R02 (same class): the rename is atomic, but the directory entry is not
/// durable until the parent is flushed. Kept best-effort — several filesystems
/// reject `sync_all` on a directory handle — yet no longer silent, so a lost
/// write after a power cut leaves a trace instead of a mystery. Shared by the
/// registry and the `active` marker; on non-Unix targets it does nothing.
pub(crate) fn sync_parent_dir(parent_dir: &Path) {
    #[cfg(unix)]
    {
        match std::fs::File::open(parent_dir).and_then(|dir| dir.sync_all()) {
            Ok(()) => {}
            Err(e) => tracing::warn!(
                err = %e,
                dir = %parent_dir.display(),
                "parent dir fsync failed; write may not survive a crash"
            ),
        }
    }
    #[cfg(not(unix))]
    {
        let _ = parent_dir;
    }
}

/// Exclusive hold on the config file for a whole read-modify-write cycle.
///
/// `flock` around the write alone was not enough: two one-shot invocations could both
/// [`load`], mutate their own copy and write in turn, so the second one silently
/// dropped the first one's host. Callers that mutate must take this guard **before**
/// `load` and keep it until [`ConfigGuard::save`] returns.
///
/// The guard must never be held across network I/O — `flock` is process-wide and a
/// blocked peer would wait for an SSH round trip. Drop it (or scope it) before any
/// connect. Re-locking while holding it would deadlock: the second `flock` on a new
/// file descriptor blocks on the first.
#[derive(Debug)]
pub struct ConfigGuard {
    /// Sibling lock file; `flock` is released on drop with the descriptor.
    lock_file: std::fs::File,
}

impl ConfigGuard {
    /// Writes the file while still holding the lock (see [`save`] for the standalone form).
    ///
    /// # Errors
    /// Returns an error if serialization, atomic write, or permission hardening fails.
    pub fn save(&self, path: &Path, file: &ConfigFile) -> SshCliResult<()> {
        save_locked(path, file)
    }
}

impl Drop for ConfigGuard {
    fn drop(&mut self) {
        let _ = fs2::FileExt::unlock(&self.lock_file);
    }
}

/// Takes the exclusive config lock for a read-modify-write cycle.
///
/// Waits up to [`lock_wait_ms`] using full-jitter backoff instead of blocking
/// forever: a peer holding the lock must not turn a one-shot into a hang.
///
/// # Errors
/// Returns an error if the sibling lock file cannot be created, hardened or
/// locked; [`SshCliError::Unavailable`] (exit 69, retryable) when the deadline
/// expires. A received signal wins over the deadline and exits 130/143.
pub fn lock_config(path: &Path) -> SshCliResult<ConfigGuard> {
    if let Some(parent_dir) = path.parent() {
        crate::fs_perm::ensure_config_dir(parent_dir)?;
    }
    // Sibling lock file to serialize concurrent mutations (N one-shots).
    let lock_path = path.with_extension("toml.lock");
    let mut lock_options = std::fs::OpenOptions::new();
    lock_options
        .create(true)
        .truncate(false)
        .read(true)
        .write(true);
    // X3: on Unix the lock is born 0o600, so no umask-0644 window exists between
    // creation and the chmod below (which stays as the second line of defense).
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        lock_options.mode(crate::constants::SECRET_FILE_MODE_UNIX);
    }
    let lock_file = lock_options.open(&lock_path)?;
    // GAP-SSH-PERM-001: lock with 0o600 (not umask 0644).
    apply_permissions_600(&lock_path)?;
    acquire_with_deadline(&lock_file)?;
    Ok(ConfigGuard { lock_file })
}

/// Wait budget for the config lock, resolved from the `--lock-wait-ms` override.
#[must_use]
fn lock_wait_ms() -> u64 {
    crate::cli::lock_wait_ms()
}

/// Tries the exclusive lock until the deadline, reporting contention as 69.
fn acquire_with_deadline(lock_file: &std::fs::File) -> SshCliResult<()> {
    use std::time::Duration;
    let outcome = wait_for_lock(
        || fs2::FileExt::try_lock_exclusive(lock_file),
        || crate::signals::signal_exit_code().is_some(),
        Duration::from_millis(lock_wait_ms()),
        Duration::from_millis(crate::constants::LOCK_WAIT_SLICE_MS),
    )?;
    match outcome {
        LockWait::Acquired => Ok(()),
        // Ctrl-C during contention must exit 130/143, never 69: the operator
        // asked to stop, and a retryable code would invite a retry.
        LockWait::Interrupted => {
            let code = crate::signals::signal_exit_code().unwrap_or_default();
            Err(SshCliError::Io(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                format!("config lock wait interrupted (signal exit {code})"),
            )))
        }
        LockWait::DeadlineExpired => Err(SshCliError::unavailable(
            crate::errors::UnavailableService::ConfigLock,
        )),
    }
}

/// How a bounded wait for the config lock ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LockWait {
    /// The lock is held.
    Acquired,
    /// The wait budget ran out while a peer still held the lock.
    DeadlineExpired,
    /// The stop signal fired before the lock was acquired.
    Interrupted,
}

/// Retry loop behind [`acquire_with_deadline`], with every effect injected.
///
/// Full-jitter backoff decides how long to wait between attempts, but the wait
/// is spent in slices of at most `slice` and `should_stop` is consulted before
/// each one, so a signal turns into an exit within one slice instead of one
/// whole backoff delay. No sleep ever crosses the remaining `budget`, and the
/// deadline is checked before sleeping. A stop request wins over the deadline.
///
/// # Errors
/// Returns the I/O error from `try_lock` when it is not plain contention.
fn wait_for_lock(
    mut try_lock: impl FnMut() -> std::io::Result<()>,
    should_stop: impl Fn() -> bool,
    budget: std::time::Duration,
    slice: std::time::Duration,
) -> SshCliResult<LockWait> {
    use std::time::Instant;
    let start = Instant::now();
    let mut attempt: u32 = 0;
    loop {
        if should_stop() {
            return Ok(LockWait::Interrupted);
        }
        match try_lock() {
            Ok(()) => return Ok(LockWait::Acquired),
            Err(e) => {
                let would_block = e.kind() == std::io::ErrorKind::WouldBlock
                    || e.raw_os_error() == Some(11)
                    || e.raw_os_error() == Some(35);
                if !would_block {
                    return Err(SshCliError::Io(e));
                }
            }
        }
        let elapsed = start.elapsed();
        if elapsed >= budget {
            // The service id is a stable slug, not a sentence: the wait time is
            // carried by this log line, so the error code's identity never
            // depends on a number.
            tracing::debug!(
                target: "ssh_cli::vps::config_io",
                waited_ms = elapsed.as_millis(),
                "config lock deadline expired"
            );
            return Ok(LockWait::DeadlineExpired);
        }
        let delay = crate::retry::backoff_full_jitter(
            crate::constants::AGENT_RETRY_BASE_MS,
            attempt,
            crate::constants::AGENT_RETRY_MAX_DELAY_MS,
        );
        attempt = attempt.saturating_add(1);
        let sleep_end = Instant::now() + delay.min(budget.saturating_sub(elapsed));
        loop {
            if should_stop() {
                return Ok(LockWait::Interrupted);
            }
            let left = sleep_end.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            std::thread::sleep(left.min(slice));
        }
    }
}

/// Serializes and writes atomically. Caller must already hold the config lock.
fn save_locked(path: &Path, file: &ConfigFile) -> SshCliResult<()> {
    if let Some(parent_dir) = path.parent() {
        crate::fs_perm::ensure_config_dir(parent_dir)?;
    }
    let text = toml::to_string_pretty(file).map_err(|e| {
        // serde flattens a typed key-resolution failure into text. Asking the resolver
        // again recovers its real class (usage, exit 64) instead of reporting it as a
        // configuration error (exit 65); the success path is untouched.
        match crate::secrets::load_primary_key() {
            Err(typed) => typed,
            Ok(_) => SshCliError::Config(format!("failed to serialize TOML: {e}")),
        }
    })?;
    write_atomic(path, text.as_bytes())
}

/// Saves the configuration file atomically with flock and 0o600.
///
/// Takes the lock for the write only. Callers that read, mutate and write back must
/// use `lock_config` instead, or they race with a concurrent invocation.
///
/// # Errors
/// Returns an error if serialization, atomic write, or permission hardening fails.
pub fn save(path: &Path, file: &ConfigFile) -> SshCliResult<()> {
    let guard = lock_config(path)?;
    guard.save(path, file)
}

/// Expands leading `~` in a path (user home).
pub(crate) fn expand_tilde(path: &str) -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home {
            return home.join(rest);
        }
    }
    if path == "~" {
        if let Some(home) = home {
            return home;
        }
    }
    PathBuf::from(path)
}

/// Validates that `key_path` points to an existing local file (VAL-003)
/// and, with `ssh-real`, that the content is a parseable OpenSSH key (VAL-004).
pub(crate) fn validate_key_path_exists(key_path: &str) -> Result<(), SshCliError> {
    validate_key_path_exists_with_passphrase(key_path, None)
}

/// Like [`validate_key_path_exists`], with optional passphrase from add/edit.
pub(crate) fn validate_key_path_exists_with_passphrase(
    key_path: &str,
    passphrase: Option<&str>,
) -> Result<(), SshCliError> {
    let p = expand_tilde(key_path);
    if !p.is_file() {
        return Err(SshCliError::FileNotFound(format!(
            "private key not found: {}",
            p.display()
        )));
    }
    #[cfg(feature = "ssh-real")]
    {
        match russh::keys::load_secret_key(&p, passphrase) {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string().to_lowercase();
                // Valid encrypted key without passphrase on the write-path.
                if msg.contains("password")
                    || msg.contains("passphrase")
                    || msg.contains("encrypted")
                    || msg.contains("decrypt")
                {
                    return Ok(());
                }
                Err(SshCliError::InvalidArgument(format!(
                    "invalid OpenSSH private key at {}: {e}",
                    p.display()
                )))
            }
        }
    }
    #[cfg(not(feature = "ssh-real"))]
    {
        let _ = passphrase;
        Ok(())
    }
}

fn apply_permissions_600(path: &Path) -> SshCliResult<()> {
    crate::fs_perm::set_secret_file_mode(path)
}

#[cfg(test)]
mod tests;
