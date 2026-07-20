// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SFTP-06 / G-SFTP-16 / G-SFTP-R01: pure path safety for remote SFTP paths (no network).
#![forbid(unsafe_code)]
//! Remote SFTP path validation and join helpers.
//!
//! SFTP wire paths are POSIX-style strings (`/`). Local OS paths stay as
//! [`std::path::Path`]. Control characters and empty segments are rejected
//! fail-closed (agent/automation surface).
//!
//! Directory entry basenames from a remote listing are treated as **untrusted**
//! (malicious server) and must pass [`validate_entry_name`] before local join.

use crate::constants::SFTP_MAX_RECURSION_DEPTH;
use crate::errors::{SshCliError, SshCliResult};

/// Validates a remote SFTP path string for the CLI surface.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] when empty, contains ASCII controls, or has
/// empty path segments (other than a lone `/`).
pub fn validate_remote_path(path: &str) -> SshCliResult<()> {
    if path.is_empty() {
        return Err(SshCliError::InvalidArgument(
            "sftp remote path must not be empty".into(),
        ));
    }
    if path.chars().any(|c| c.is_control() || c == '\0') {
        return Err(SshCliError::InvalidArgument(
            "sftp remote path must not contain control characters".into(),
        ));
    }
    // Allow "." / ".." as segments (server resolves); reject empty `//a//b` middles
    // only when they create empty components after split (except leading root).
    let stripped = path.trim_start_matches('/');
    if stripped.is_empty() {
        // "/" is valid (root).
        return Ok(());
    }
    for seg in stripped.split('/') {
        if seg.is_empty() {
            return Err(SshCliError::InvalidArgument(format!(
                "sftp remote path has empty segment: {path}"
            )));
        }
    }
    Ok(())
}

/// Validates a single directory entry basename (local or remote listing).
///
/// Rejects empty, `.`, `..`, controls, and path separators (`/` or `\`) so that
/// `Path::join` cannot escape the intended destination tree (G-SFTP-R01).
///
/// # Errors
/// [`SshCliError::InvalidArgument`] when the name is unsafe.
pub fn validate_entry_name(name: &str) -> SshCliResult<()> {
    if name.is_empty() {
        return Err(SshCliError::InvalidArgument(
            "sftp entry name must not be empty".into(),
        ));
    }
    if name == "." || name == ".." {
        return Err(SshCliError::InvalidArgument(format!(
            "sftp entry name refuses relative segment: {name}"
        )));
    }
    if name.chars().any(|c| c.is_control() || c == '\0' || c == '/' || c == '\\') {
        return Err(SshCliError::InvalidArgument(format!(
            "sftp entry name must not contain controls or path separators: {name}"
        )));
    }
    Ok(())
}

/// Joins a remote directory and a file name with `/` (SFTP POSIX).
#[must_use]
pub fn join_remote(parent: &str, name: &str) -> String {
    if parent.is_empty() || parent == "." {
        return name.to_owned();
    }
    if parent.ends_with('/') {
        format!("{parent}{name}")
    } else {
        format!("{parent}/{name}")
    }
}

/// Ensures recursion depth stays within the product cap.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] when `depth` exceeds [`SFTP_MAX_RECURSION_DEPTH`].
pub fn check_depth(depth: u32) -> SshCliResult<()> {
    if depth > SFTP_MAX_RECURSION_DEPTH {
        return Err(SshCliError::InvalidArgument(format!(
            "sftp recursion depth {depth} exceeds max {SFTP_MAX_RECURSION_DEPTH}"
        )));
    }
    Ok(())
}

/// Returns true when `candidate` is under `root` after string prefix rules
/// (both must be absolute-ish SFTP paths from `canonicalize` when possible).
///
/// Reserved for optional follow-symlink-if-under-root policy; unit-tested.
#[must_use]
#[allow(dead_code)]
pub fn remote_is_under(root: &str, candidate: &str) -> bool {
    if root.is_empty() {
        return false;
    }
    if candidate == root {
        return true;
    }
    let root_slash = if root.ends_with('/') {
        root.to_owned()
    } else {
        format!("{root}/")
    };
    candidate.starts_with(&root_slash)
}

/// Ensures a local path stays under an intended destination root.
///
/// Component-wise comparison (does not require the path to exist yet).
/// Rejects any `..` (`ParentDir`) in `candidate` so joins cannot climb out.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] when the path escapes the root.
pub fn ensure_local_under(root: &std::path::Path, candidate: &std::path::Path) -> SshCliResult<()> {
    use std::path::Component;
    if candidate
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(SshCliError::InvalidArgument(format!(
            "sftp local path must not contain '..': {}",
            candidate.display()
        )));
    }
    let root_c = root.components().collect::<Vec<_>>();
    let cand_c = candidate.components().collect::<Vec<_>>();
    if cand_c.len() < root_c.len() {
        return Err(SshCliError::InvalidArgument(format!(
            "sftp local path escapes destination root: {}",
            candidate.display()
        )));
    }
    if cand_c[..root_c.len()] != root_c[..] {
        return Err(SshCliError::InvalidArgument(format!(
            "sftp local path escapes destination root: {}",
            candidate.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn validate_accepts_normal_paths() {
        validate_remote_path("/var/log/app.log").unwrap();
        validate_remote_path("relative/file").unwrap();
        validate_remote_path("/").unwrap();
        validate_remote_path(".").unwrap();
    }

    #[test]
    fn validate_rejects_empty_and_controls() {
        assert!(validate_remote_path("").is_err());
        assert!(validate_remote_path("a\0b").is_err());
        assert!(validate_remote_path("a\nb").is_err());
        assert!(validate_remote_path("/a//b").is_err());
    }

    #[test]
    fn entry_name_rejects_escape() {
        validate_entry_name("ok-file.txt").unwrap();
        assert!(validate_entry_name("").is_err());
        assert!(validate_entry_name(".").is_err());
        assert!(validate_entry_name("..").is_err());
        assert!(validate_entry_name("a/b").is_err());
        assert!(validate_entry_name("a\\b").is_err());
        assert!(validate_entry_name("a\0b").is_err());
    }

    #[test]
    fn join_remote_handles_slash() {
        assert_eq!(join_remote("/tmp", "x"), "/tmp/x");
        assert_eq!(join_remote("/tmp/", "x"), "/tmp/x");
        assert_eq!(join_remote(".", "x"), "x");
    }

    #[test]
    fn depth_cap() {
        check_depth(0).unwrap();
        check_depth(SFTP_MAX_RECURSION_DEPTH).unwrap();
        assert!(check_depth(SFTP_MAX_RECURSION_DEPTH + 1).is_err());
    }

    #[test]
    fn under_prefix() {
        assert!(remote_is_under("/home/u", "/home/u/a"));
        assert!(remote_is_under("/home/u", "/home/u"));
        assert!(!remote_is_under("/home/u", "/home/other"));
        assert!(!remote_is_under("/home/u", "/home/u2/x"));
    }

    #[test]
    fn local_under() {
        let root = Path::new("/tmp/dest");
        ensure_local_under(root, Path::new("/tmp/dest/a")).unwrap();
        assert!(ensure_local_under(root, Path::new("/tmp/other")).is_err());
        // `..` component escapes when joined under root then checked as absolute-ish paths:
        // component list for `/tmp/dest/../other` differs from `/tmp/dest` prefix length rules.
        assert!(ensure_local_under(root, Path::new("/tmp/dest/../other")).is_err());
    }
}
