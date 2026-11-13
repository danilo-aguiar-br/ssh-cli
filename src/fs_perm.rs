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

use crate::constants::{GROUP_OTHER_MODE_MASK_UNIX, SECRET_DIR_MODE_UNIX, SECRET_FILE_MODE_UNIX};
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
///
/// # Errors
/// Returns the I/O error of [`set_secret_file_mode_checked`] when metadata cannot be read or
/// the mode cannot be changed on a supported platform.
pub fn set_secret_file_mode(path: &Path) -> SshCliResult<()> {
    warn_if_unprotected(path, set_secret_file_mode_checked(path)?);
    Ok(())
}

/// Sets secret-dir mode (`0o700`) on Unix; **warns** on unsupported targets.
///
/// See [`set_secret_file_mode`] for why this does not fail closed.
///
/// # Errors
/// Returns the I/O error of [`set_secret_dir_mode_checked`] when metadata cannot be read or
/// the mode cannot be changed on a supported platform.
pub fn set_secret_dir_mode(path: &Path) -> SshCliResult<()> {
    warn_if_unprotected(path, set_secret_dir_mode_checked(path)?);
    Ok(())
}

/// Creates the configuration directory at `0o700`, or hardens the default one.
///
/// Every module that writes under the config directory (registry, lock, active
/// marker, `secrets.key`, `known_hosts`, `lang`, TLS) goes through this single
/// helper, so a directory created by this binary is never left at the umask
/// default (`0o755`) with `0o600` files inside it.
///
/// - Missing directory: created recursively, every new level at `0o700`.
/// - Existing directory: tightened to `0o700` ONLY when it is the XDG default
///   directory, is not a symlink and is owned by the current user. Any other
///   existing directory (`--config-dir` pointing at `$HOME`, `/tmp`, a shared
///   folder) never gets a `chmod`: a structured `tracing::warn!` is emitted once
///   per path and `doctor` reports `config_dir_exposed`.
/// - Non-Unix targets: plain `create_dir_all` (no POSIX mode bits; ACLs stay with
///   the user profile).
///
/// # Errors
/// [`SshCliError::Io`] when the directory cannot be inspected, created or tightened.
///
/// # Examples
/// ```no_run
/// let dir = std::path::Path::new("/tmp/ssh-cli-demo/config");
/// ssh_cli::fs_perm::ensure_config_dir(dir)?;
/// # Ok::<(), ssh_cli::errors::SshCliError>(())
/// ```
pub fn ensure_config_dir(dir: &Path) -> SshCliResult<()> {
    let default_dir = crate::paths::xdg_config_dir().ok();
    ensure_config_dir_with_default(dir, default_dir.as_deref())
}

/// Creates the missing levels of `dir` at `0o700` and leaves an existing directory alone.
///
/// For output folders chosen by the user (`vps export -o`): directories this binary
/// creates may end up holding secrets, so they are born private, while a folder that
/// already exists is never inspected, warned about or changed.
///
/// # Errors
/// [`SshCliError::Io`] when the directory cannot be created, or exists as a non-directory.
pub fn create_missing_private_dirs(dir: &Path) -> SshCliResult<()> {
    if dir.is_dir() {
        return Ok(());
    }
    ensure_config_dir_with_default(dir, None)
}

/// Same as [`ensure_config_dir`] with the "default directory" injected.
///
/// Lets unit tests exercise the tightening branch without touching process-wide
/// environment variables. `None` means the default is unknown, so nothing that
/// already exists is ever tightened.
///
/// # Errors
/// See [`ensure_config_dir`].
pub(crate) fn ensure_config_dir_with_default(
    dir: &Path,
    default_dir: Option<&Path>,
) -> SshCliResult<()> {
    #[cfg(unix)]
    {
        ensure_config_dir_unix(dir, default_dir)
    }
    #[cfg(not(unix))]
    {
        let _ = default_dir;
        std::fs::create_dir_all(dir).map_err(SshCliError::Io)
    }
}

#[cfg(unix)]
fn ensure_config_dir_unix(dir: &Path, default_dir: Option<&Path>) -> SshCliResult<()> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt};

    if dir.as_os_str().is_empty() {
        return Ok(());
    }
    let meta = match std::fs::symlink_metadata(dir) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return std::fs::DirBuilder::new()
                .recursive(true)
                .mode(SECRET_DIR_MODE_UNIX)
                .create(dir)
                .map_err(SshCliError::Io);
        }
        Err(e) => return Err(SshCliError::Io(e)),
    };
    if meta.is_symlink() || !meta.is_dir() {
        // Validate the target exactly as `create_dir_all` always did, but never
        // chmod through a link or a non-directory.
        std::fs::create_dir_all(dir).map_err(SshCliError::Io)?;
        if let Some(mode) = path_mode_bits(dir).filter(|m| mode_is_exposed(*m)) {
            warn_exposed_once(dir, mode, "symlink");
        }
        return Ok(());
    }
    let mode = meta.mode() & 0o777;
    if !mode_is_exposed(mode) {
        return Ok(());
    }
    if !default_dir.is_some_and(|default| same_directory(dir, default)) {
        warn_exposed_once(dir, mode, "not the default config directory");
    } else if meta.uid() != rustix::process::getuid().as_raw() {
        warn_exposed_once(dir, mode, "owned by another user");
    } else {
        set_secret_dir_mode_checked(dir)?;
    }
    Ok(())
}

/// True when both paths name the same directory (lexically or after resolving links).
#[cfg(unix)]
fn same_directory(a: &Path, b: &Path) -> bool {
    a == b
        || matches!(
            (std::fs::canonicalize(a), std::fs::canonicalize(b)),
            (Ok(ca), Ok(cb)) if ca == cb
        )
}

/// Logs that a directory stays readable by group/other, once per path per process.
#[cfg(unix)]
fn warn_exposed_once(dir: &Path, mode: u32, reason: &'static str) {
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock, PoisonError};

    static SEEN: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    let first = SEEN
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(dir.to_path_buf());
    if first {
        tracing::warn!(
            path = %dir.display(),
            mode = %format_args!("{mode:04o}"),
            reason,
            "config directory is accessible to group or other users and was left unchanged"
        );
    }
}

/// Permission bits (`& 0o777`) of `path`, or `None` when the path cannot be read
/// or the platform has no POSIX mode bits.
#[must_use]
pub fn path_mode_bits(path: &Path) -> Option<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .ok()
            .map(|m| m.permissions().mode() & 0o777)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

/// True when `mode` grants any access to the group or to other users.
#[must_use]
pub const fn mode_is_exposed(mode: u32) -> bool {
    mode & GROUP_OTHER_MODE_MASK_UNIX != 0
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
        ensure_config_dir(parent)?;
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777
    }

    fn set_mode(path: &Path, bits: u32) {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(bits)).expect("chmod");
    }

    #[test]
    fn missing_directory_is_created_private_at_every_new_level() {
        let tmp = TempDir::new().expect("tempdir");
        let leaf = tmp.path().join("a").join("b").join("c");
        ensure_config_dir_with_default(&leaf, None).expect("create");
        assert_eq!(mode(&leaf), SECRET_DIR_MODE_UNIX);
        assert_eq!(mode(&tmp.path().join("a").join("b")), SECRET_DIR_MODE_UNIX);
        assert_eq!(mode(&tmp.path().join("a")), SECRET_DIR_MODE_UNIX);
    }

    #[test]
    fn default_directory_owned_by_us_is_tightened() {
        let tmp = TempDir::new().expect("tempdir");
        set_mode(tmp.path(), 0o755);
        ensure_config_dir_with_default(tmp.path(), Some(tmp.path())).expect("ensure");
        assert_eq!(mode(tmp.path()), SECRET_DIR_MODE_UNIX);
    }

    #[test]
    fn default_directory_is_matched_through_canonical_paths() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join("cfg");
        std::fs::create_dir(&dir).expect("mkdir");
        set_mode(&dir, 0o750);
        let roundabout = dir.join("..").join("cfg");
        ensure_config_dir_with_default(&dir, Some(&roundabout)).expect("ensure");
        assert_eq!(mode(&dir), SECRET_DIR_MODE_UNIX);
    }

    #[test]
    fn non_default_existing_directory_is_never_chmodded() {
        let tmp = TempDir::new().expect("tempdir");
        let other = TempDir::new().expect("tempdir");
        set_mode(tmp.path(), 0o755);
        ensure_config_dir_with_default(tmp.path(), Some(other.path())).expect("ensure");
        assert_eq!(mode(tmp.path()), 0o755);
    }

    #[test]
    fn unknown_default_never_tightens() {
        let tmp = TempDir::new().expect("tempdir");
        set_mode(tmp.path(), 0o755);
        ensure_config_dir_with_default(tmp.path(), None).expect("ensure");
        assert_eq!(mode(tmp.path()), 0o755);
    }

    #[test]
    fn already_private_directory_is_left_as_is() {
        let tmp = TempDir::new().expect("tempdir");
        set_mode(tmp.path(), 0o500);
        ensure_config_dir_with_default(tmp.path(), Some(tmp.path())).expect("ensure");
        assert_eq!(mode(tmp.path()), 0o500);
        set_mode(tmp.path(), 0o700);
    }

    #[test]
    fn symlink_is_not_followed_nor_chmodded() {
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("real");
        std::fs::create_dir(&target).expect("mkdir");
        set_mode(&target, 0o755);
        let link = tmp.path().join("link");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        ensure_config_dir_with_default(&link, Some(&link)).expect("ensure");
        assert_eq!(mode(&target), 0o755);
        assert!(std::fs::symlink_metadata(&link)
            .expect("link metadata")
            .is_symlink());
    }

    #[test]
    fn a_regular_file_in_place_of_the_directory_is_an_error() {
        let tmp = TempDir::new().expect("tempdir");
        let file = tmp.path().join("not-a-dir");
        std::fs::write(&file, b"x").expect("write");
        assert!(ensure_config_dir_with_default(&file, None).is_err());
    }

    #[test]
    fn empty_path_is_a_no_op() {
        ensure_config_dir_with_default(Path::new(""), None).expect("empty path");
    }

    #[test]
    fn exposure_follows_group_and_other_bits_only() {
        assert!(!mode_is_exposed(0o700));
        assert!(!mode_is_exposed(0o500));
        assert!(mode_is_exposed(0o750));
        assert!(mode_is_exposed(0o701));
    }

    #[test]
    fn path_mode_bits_reads_the_directory_mode() {
        let tmp = TempDir::new().expect("tempdir");
        set_mode(tmp.path(), 0o750);
        assert_eq!(path_mode_bits(tmp.path()), Some(0o750));
        assert_eq!(path_mode_bits(&tmp.path().join("missing")), None);
    }
}
