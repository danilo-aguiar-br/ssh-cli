// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Unix secret file/dir modes — single source (G-AUD-24 / no hardcode drift).
//!
//! # Platform coverage (A9)
//!
//! Only Unix is implemented. A Windows DACL restricted to the current user needs
//! `windows-sys` with the `Win32_Security` / `Win32_Storage_FileSystem` features,
//! which this crate does not currently enable, so the code cannot be written
//! here without touching `Cargo.toml`. Until then, every entry point reports
//! [`crate::fs_perm::SecretProtection::Unsupported`] instead of returning a success that would
//! make the caller believe a secret file is locked down when it is not.

use std::path::Path;

use crate::constants::{SECRET_DIR_MODE_UNIX, SECRET_FILE_MODE_UNIX};
use crate::errors::{SshCliError, SshCliResult};

/// Whether a restrictive mode was actually applied to a path.
///
/// A9: the previous API returned `SshCliResult<()>` and answered `Ok(())` on
/// every non-Unix target, so `secrets.key`, `config.toml`, `known_hosts` and TLS
/// PEMs were left at whatever the platform default is while the caller believed
/// they were protected. Encoding "not applied" in the success value keeps the
/// caller honest: nothing here may claim a protection it did not perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretProtection {
    /// The restrictive mode was applied to the path.
    Applied,
    /// The platform has no supported implementation; the path is unprotected.
    Unsupported,
}

impl SecretProtection {
    /// True when the caller may consider the path protected.
    #[must_use]
    pub const fn is_applied(self) -> bool {
        matches!(self, Self::Applied)
    }

    /// Stable name for JSON / `doctor` reporting.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Unsupported => "unsupported",
        }
    }
}

/// True when this build can restrict access to secret files and directories.
///
/// A9: `false` means every secret written by this binary is readable by any
/// account that can reach the path; `doctor` is expected to surface that instead
/// of the process pretending the files are locked down.
#[must_use]
pub const fn secret_protection_supported() -> bool {
    cfg!(unix)
}

/// Sets secret-file mode (`0o600`) and reports whether it was applied.
///
/// Prefer this over [`set_secret_file_mode`] when the caller can report or act
/// on an unprotected file.
///
/// # Errors
/// Metadata read or permission change failure on a supported platform.
pub fn set_secret_file_mode_checked(path: &Path) -> SshCliResult<SecretProtection> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(SshCliError::Io)?
            .permissions();
        perms.set_mode(SECRET_FILE_MODE_UNIX);
        std::fs::set_permissions(path, perms).map_err(SshCliError::Io)?;
        Ok(SecretProtection::Applied)
    }
    #[cfg(not(unix))]
    {
        // A9: no Windows DACL implementation is reachable from this crate today
        // (see module note); report the gap rather than fake success.
        let _ = path;
        Ok(SecretProtection::Unsupported)
    }
}

/// Sets secret-dir mode (`0o700`) and reports whether it was applied.
///
/// # Errors
/// Metadata read or permission change failure on a supported platform.
pub fn set_secret_dir_mode_checked(path: &Path) -> SshCliResult<SecretProtection> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(SshCliError::Io)?
            .permissions();
        perms.set_mode(SECRET_DIR_MODE_UNIX);
        std::fs::set_permissions(path, perms).map_err(SshCliError::Io)?;
        Ok(SecretProtection::Applied)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(SecretProtection::Unsupported)
    }
}

/// Sets secret-file mode (`0o600`) on Unix; **warns** on unsupported targets.
///
/// Compatibility wrapper for call sites that cannot act on the outcome. It never
/// silently succeeds: an unsupported platform emits a warning naming the path,
/// so the gap is at least observable in logs. New code should call
/// [`set_secret_file_mode_checked`].
pub fn set_secret_file_mode(path: &Path) -> SshCliResult<()> {
    warn_if_unprotected(path, set_secret_file_mode_checked(path)?);
    Ok(())
}

/// Sets secret-dir mode (`0o700`) on Unix; **warns** on unsupported targets.
///
/// See [`set_secret_file_mode`] for why this does not fail closed.
pub fn set_secret_dir_mode(path: &Path) -> SshCliResult<()> {
    warn_if_unprotected(path, set_secret_dir_mode_checked(path)?);
    Ok(())
}

/// Emits a warning when a secret path was left at platform-default access.
fn warn_if_unprotected(path: &Path, outcome: SecretProtection) {
    if !outcome.is_applied() {
        tracing::warn!(
            path = %path.display(),
            "secret path left at platform-default permissions: this build cannot restrict access on this OS"
        );
    }
}

/// Writes secret bytes to `path` atomically, never exposing them at a wider mode.
///
/// A2: writing with [`std::fs::write`] creates the file at `0644` under the default
/// umask and only narrows it afterwards, leaving a window where a private key is
/// world-readable. [`tempfile::NamedTempFile`] creates at `0600` via `O_EXCL`, so the
/// content is never observable by other users, and the rename is atomic.
///
/// # Errors
/// Directory creation, temp-file creation, write, fsync, mode change or rename failure.
pub fn write_secret_file_atomic(path: &Path, data: &[u8]) -> SshCliResult<()> {
    use std::io::Write;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(SshCliError::Io)?;
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent).map_err(SshCliError::Io)?;
    tmp.write_all(data).map_err(SshCliError::Io)?;
    tmp.as_file().sync_all().map_err(SshCliError::Io)?;
    set_secret_file_mode(tmp.path())?;
    tmp.persist(path).map_err(|e| SshCliError::Io(e.error))?;
    // Re-apply after rename; the temp file already carried the restricted mode.
    set_secret_file_mode(path)?;
    Ok(())
}

/// Compile-time alias for call sites that need the raw secret-file mode integer.
#[must_use]
pub const fn secret_file_mode() -> u32 {
    SECRET_FILE_MODE_UNIX
}

/// Compile-time alias for call sites that need the raw secret-dir mode integer.
#[must_use]
pub const fn secret_dir_mode() -> u32 {
    SECRET_DIR_MODE_UNIX
}
