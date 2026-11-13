// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Active-VPS marker (sibling `active` file next to `config.toml`).
//!
//! The marker remembers which host `connect` selected, so `--use-active` can
//! reach it deliberately. It is *ambient* state: nothing here may decide a
//! target on the caller's behalf. Explicit Target Designation lives in
//! `crate::cli::targeting` and this module only stores and reads the value.
//!
//! # Why the path helper exists
//!
//! `<config_dir>/active` was spelled inline in `crud`, `doctor` and `mod`, and
//! three copies of a path rule are three chances to disagree about the parent
//! directory. [`active_path`] is the single spelling.

use crate::constants::ACTIVE_VPS_FILE_NAME;
use crate::errors::{SshCliError, SshCliResult};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Returns `<config_dir>/active` for the given `config.toml` path.
///
/// The marker is a sibling of the configuration file, so a `--config-dir`
/// override moves both together and tests never touch the real XDG directory.
#[must_use]
pub fn active_path(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .map(|p| p.join(ACTIVE_VPS_FILE_NAME))
        .unwrap_or_else(|| PathBuf::from(ACTIVE_VPS_FILE_NAME))
}

/// Reads the active marker through the resolved config path.
///
/// # Errors
/// Configuration-path resolution or a read failure on an existing marker.
pub fn read_active_vps(config_override: Option<&Path>) -> SshCliResult<Option<String>> {
    let path = crate::vps::resolve_config_path(config_override)?;
    read_active_at(&path)
}

/// Reads the marker beside `config_path`.
///
/// # Errors
/// A read failure on an existing marker.
pub fn read_active_at(config_path: &Path) -> SshCliResult<Option<String>> {
    let active = active_path(config_path);
    if !active.exists() {
        return Ok(None);
    }
    let name = std::fs::read_to_string(&active)?;
    Ok(Some(name.trim().to_string()))
}

/// Writes the marker atomically (temp file, `sync_data`, `persist`, parent fsync).
///
/// `persist` replaces the destination atomically on Linux, macOS and Windows,
/// which matters because a half-written marker would make `--use-active`
/// resolve a truncated name. The parent directory is then flushed best-effort
/// (warning, never an error), like `config.toml`, so the new entry survives a
/// crash. The temp file is created `0o600` on Unix and keeps that mode.
///
/// # Errors
/// Directory creation, write, fsync or rename failure.
pub fn write_active_vps(config_path: &Path, name: &str) -> SshCliResult<()> {
    let active_file = active_path(config_path);
    let parent_dir = active_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    crate::fs_perm::ensure_config_dir(&parent_dir)?;
    let mut tmp = tempfile::NamedTempFile::new_in(&parent_dir)?;
    tmp.write_all(name.as_bytes())?;
    tmp.as_file().sync_data()?;
    tmp.persist(&active_file)
        .map_err(|e| SshCliError::Io(e.error))?;
    super::config_io::sync_parent_dir(&parent_dir);
    Ok(())
}

/// Removes the marker when it points at `name` (STATE-001).
///
/// A marker pointing at another host is left untouched: removing a host must
/// never silently clear a different selection.
///
/// A missing marker or a failed removal is not an error: a stale marker is
/// harmless, because the target lookup reports the missing host.
///
/// # Errors
/// Currently always returns `Ok`: a missing marker or a failed removal is deliberately
/// ignored.
pub fn clear_active_if_name(config_path: &Path, name: &str) -> SshCliResult<()> {
    let active = active_path(config_path);
    if !active.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&active).unwrap_or_default();
    if content.trim() == name {
        let _ = std::fs::remove_file(&active);
    }
    Ok(())
}
