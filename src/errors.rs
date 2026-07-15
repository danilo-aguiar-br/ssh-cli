// SPDX-License-Identifier: MIT OR Apache-2.0
//! Structured error types for ssh-cli.
//!
//! Defines [`SshCliError`] with domain error categories used by the CLI.
//! Display strings are technical English. Localized UI copy belongs in `i18n`.

use thiserror::Error;

/// Domain errors produced by ssh-cli operations.
#[derive(Debug, Error)]
pub enum SshCliError {
    /// Underlying I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML deserialization error.
    #[error("TOML read error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// TOML serialization error.
    #[error("TOML write error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// SSH connection-layer error.
    #[error("SSH connection error: {0}")]
    SshConnection(String),

    /// SSH authentication error with detail.
    #[error("SSH authentication error: {0}")]
    SshAuthentication(String),

    /// TCP/SSH connection failed before authentication.
    #[error("SSH connection failed: {0}")]
    ConnectionFailed(String),

    /// SSH authentication rejected by the server.
    #[error(
        "SSH authentication failed; try --password-stdin, --key PATH, --key-passphrase-stdin, or verify the user"
    )]
    AuthenticationFailed,

    /// Host key diverged from known_hosts (possible MITM).
    #[error(
        "host key changed for {host}:{port}: expected {expected}, got {obtained} (use --replace-host-key if legitimate)"
    )]
    HostKeyChanged {
        /// Host name.
        host: String,
        /// Port.
        port: u16,
        /// Expected fingerprint.
        expected: String,
        /// Observed fingerprint.
        obtained: String,
    },

    /// Command exceeds `max_command_chars`.
    #[error("command exceeds max_command_chars ({max}): {len} characters")]
    CommandTooLong {
        /// Configured limit.
        max: usize,
        /// Command length.
        len: usize,
    },

    /// Elevation disabled (`disable_sudo`).
    #[error("sudo/su disabled for this host (disable_sudo)")]
    SudoDisabled,

    /// Missing `su` password for `su-exec`.
    #[error("su_password not configured; use vps edit --su-password or --su-password-stdin")]
    SuPasswordMissing,

    /// Failed to open or operate an SSH channel.
    #[error("SSH channel failed: {0}")]
    ChannelFailed(String),

    /// SSH operation timed out.
    #[error("SSH timeout after {0}ms")]
    SshTimeout(u64),

    /// Remote command exited non-zero.
    #[error("command failed with exit code {exit_code}: {stderr}")]
    CommandFailed {
        /// Remote process exit code.
        exit_code: i32,
        /// Truncated stderr snippet.
        stderr: String,
    },

    /// VPS name not found in the registry.
    #[error("VPS '{0}' not found in registry")]
    VpsNotFound(String),

    /// No active VPS (`active` sibling file missing) — GAP-SSH-EXIT-002.
    #[error("No active VPS. Run 'ssh-cli connect <NAME>' first.")]
    NoActiveVps,

    /// Duplicate VPS name in the registry.
    #[error("VPS '{0}' already exists in registry")]
    VpsDuplicate(String),

    /// Local or remote file not found.
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// Invalid CLI argument.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Generic timeout.
    #[error("timeout exceeded after {0}ms")]
    Timeout(u64),

    /// XDG config directory unavailable.
    #[error("configuration directory unavailable")]
    XdgDirectory,

    /// Incompatible schema version.
    #[error("incompatible schema version: expected {expected}, found {found}")]
    SchemaIncompatible {
        /// Expected schema version.
        expected: u32,
        /// Found schema version.
        found: u32,
    },

    /// Uncategorized error.
    #[error("error: {0}")]
    Generic(String),
}

/// Process exit codes aligned with sysexits.h and Unix signal conventions.
pub mod exit_codes {
    /// Success.
    pub const EX_OK: i32 = 0;
    /// Generic domain failure.
    pub const EX_GENERAL: i32 = 1;
    /// Incorrect CLI usage.
    pub const EX_USAGE: i32 = 64;
    /// Invalid input data.
    pub const EX_DATAERR: i32 = 65;
    /// Input not found.
    pub const EX_NOINPUT: i32 = 66;
    /// Cannot create output.
    pub const EX_CANTCREAT: i32 = 73;
    /// I/O error.
    pub const EX_IOERR: i32 = 74;
    /// Permission denied.
    pub const EX_NOPERM: i32 = 77;
    /// Terminated by SIGINT (Ctrl+C).
    pub const EX_SIGINT: i32 = 130;
    /// Terminated by SIGTERM.
    pub const EX_SIGTERM: i32 = 143;
}

impl SshCliError {
    /// Returns the sysexits.h exit code for this error.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Io(_) => exit_codes::EX_IOERR,
            Self::Json(_) => exit_codes::EX_DATAERR,
            Self::TomlDe(_) => exit_codes::EX_DATAERR,
            Self::TomlSer(_) => exit_codes::EX_CANTCREAT,
            Self::SshConnection(_) => exit_codes::EX_IOERR,
            Self::SshAuthentication(_) => exit_codes::EX_IOERR,
            Self::ConnectionFailed(_) => exit_codes::EX_IOERR,
            Self::AuthenticationFailed => exit_codes::EX_NOPERM,
            Self::HostKeyChanged { .. } => exit_codes::EX_NOPERM,
            Self::CommandTooLong { .. } => exit_codes::EX_USAGE,
            Self::SudoDisabled => exit_codes::EX_NOPERM,
            Self::SuPasswordMissing => exit_codes::EX_USAGE,
            Self::ChannelFailed(_) => exit_codes::EX_IOERR,
            Self::SshTimeout(_) => exit_codes::EX_IOERR,
            Self::CommandFailed { .. } => exit_codes::EX_GENERAL,
            Self::VpsNotFound(_) => exit_codes::EX_NOINPUT,
            Self::NoActiveVps => exit_codes::EX_NOINPUT,
            Self::VpsDuplicate(_) => exit_codes::EX_USAGE,
            Self::FileNotFound(_) => exit_codes::EX_NOINPUT,
            Self::InvalidArgument(_) => exit_codes::EX_USAGE,
            Self::Timeout(_) => exit_codes::EX_IOERR,
            Self::XdgDirectory => exit_codes::EX_CANTCREAT,
            Self::SchemaIncompatible { .. } => exit_codes::EX_DATAERR,
            Self::Generic(_) => exit_codes::EX_GENERAL,
        }
    }
}

/// Result alias using [`SshCliError`].
pub type SshCliResult<T> = std::result::Result<T, SshCliError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vps_not_found_message_contains_name() {
        let err = SshCliError::VpsNotFound("prod".into());
        assert!(err.to_string().contains("prod"));
    }

    #[test]
    fn vps_duplicate_message_contains_name() {
        let err = SshCliError::VpsDuplicate("vps-1".into());
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn exit_code_auth_failed_is_noperm() {
        assert_eq!(
            SshCliError::AuthenticationFailed.exit_code(),
            exit_codes::EX_NOPERM
        );
    }

    #[test]
    fn exit_code_command_failed_is_general_not_remote() {
        let e = SshCliError::CommandFailed {
            exit_code: 64,
            stderr: "usage".into(),
        };
        assert_eq!(e.exit_code(), exit_codes::EX_GENERAL);
    }
}
