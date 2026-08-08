// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! Validation for `tunnel --remote-socket` (G-TUN-R03).
//!
//! The forwarding itself reuses the local accept loop — a Unix socket target only
//! changes which SSH channel type is opened. What is specific to this mode is the
//! *precondition*: the path names a socket on the remote host, so it must be
//! judged by the remote's rules, not this machine's.
//!
//! # Platform note
//!
//! The **client** may run on Windows: it only ever speaks TCP locally and asks the
//! server to reach the socket. The gate that matters is the server's support for
//! the `direct-streamlocal@openssh.com` extension, which is detected on the wire
//! rather than guessed from the local platform.

use crate::errors::{SshCliError, SshCliResult};

/// Rejects a remote socket path that cannot be valid on any POSIX host.
///
/// Deliberately minimal: the socket lives on the *server*, so anything beyond
/// clearly-impossible input would be this machine second-guessing a filesystem it
/// cannot see. Checking `Path::exists` locally would be actively wrong — it would
/// pass or fail based on paths that have nothing to do with the remote host.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] (exit 64) for an empty path, a relative path,
/// or one containing a NUL byte.
pub fn validate_remote_socket(path: &str) -> SshCliResult<()> {
    if path.is_empty() {
        return Err(SshCliError::InvalidArgument(
            "--remote-socket requires a path".to_string(),
        ));
    }
    if !path.starts_with('/') {
        return Err(SshCliError::InvalidArgument(format!(
            "--remote-socket must be an absolute remote path, got `{path}`"
        )));
    }
    if path.contains('\0') {
        return Err(SshCliError::InvalidArgument(
            "--remote-socket must not contain a NUL byte".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_socket_path_is_accepted() {
        assert!(validate_remote_socket("/var/run/docker.sock").is_ok());
    }

    #[test]
    fn empty_path_is_rejected() {
        let err = validate_remote_socket("").expect_err("empty path must be rejected");
        assert_eq!(err.exit_code(), crate::errors::exit_codes::EX_USAGE);
    }

    #[test]
    fn relative_path_is_rejected() {
        // A relative path would be resolved against the server's cwd, which this
        // side cannot know — so it can never mean what the caller intended.
        let err =
            validate_remote_socket("run/docker.sock").expect_err("relative path must be rejected");
        assert!(matches!(err, SshCliError::InvalidArgument(_)));
    }

    #[test]
    fn nul_byte_is_rejected() {
        assert!(validate_remote_socket("/var/run/x\0y.sock").is_err());
    }

    #[test]
    fn windows_style_path_is_rejected_because_the_target_is_posix() {
        // The client may run on Windows, but the socket lives on the remote host.
        assert!(validate_remote_socket("C:\\pipe\\docker").is_err());
    }
}
