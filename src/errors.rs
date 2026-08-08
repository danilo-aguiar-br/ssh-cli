// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Structured error types for ssh-cli.
//!
//! Defines [`crate::errors::SshCliError`] with domain error categories used by the CLI.
//! Display strings are technical English (lowercase, no trailing period — G-ERR-01).
//! Localized UI copy belongs in `i18n`.
//!
//! # Errors contract (G-ERR)
//!
//! - Prefer typed variants over [`crate::errors::SshCliError::Generic`].
//! - Preserve [`std::error::Error::source`] via helpers [`crate::errors::SshCliError::tls_src`] /
//!   [`crate::errors::SshCliError::channel_src`] instead of embedding `{e}` in a flat string.
//! - Agents must read [`crate::errors::SshCliError::error_code`] + [`crate::errors::ErrorClass`], not parse Display.

use thiserror::Error;

/// Shared source box for layered errors (Send + Sync for fan-out tasks).
pub type ErrorSource = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Domain errors produced by ssh-cli operations.
///
/// `#[non_exhaustive]` (G-SEC-11): external crates must use wildcard arms so
/// new variants in minor releases do not break SemVer consumers.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SshCliError {
    /// Underlying I/O error.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML deserialization error.
    #[error("toml read error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// TOML serialization error.
    #[error("toml write error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// Domain newtype / parse construction failure (G-ERR-02).
    #[error("{0}")]
    Domain(#[from] crate::domain::DomainError),

    /// SSH connection-layer error.
    #[error("ssh connection error: {0}")]
    SshConnection(String),

    /// SSH authentication error with detail.
    #[error("ssh authentication error: {0}")]
    SshAuthentication(String),

    /// TCP/SSH connection failed before authentication.
    #[error("ssh connection failed: {0}")]
    ConnectionFailed(String),

    /// SSH authentication rejected by the server.
    #[error(
        "ssh authentication failed; try --password-stdin, --key PATH, --key-passphrase-stdin, or verify the user"
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

    /// Failed to open or operate an SSH channel (G-ERR-05: optional source chain).
    #[error("ssh channel failed: {message}")]
    ChannelFailed {
        /// Short context (no embedded cause Display).
        message: String,
        /// Root cause when available.
        #[source]
        source: Option<ErrorSource>,
    },

    /// SSH operation timed out.
    #[error("ssh timeout after {0}ms")]
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
    #[error("vps '{0}' not found in registry")]
    VpsNotFound(String),

    /// No active VPS (`active` sibling file missing) — GAP-SSH-EXIT-002.
    #[error("no active vps; run 'ssh-cli connect <NAME>' first")]
    NoActiveVps,

    /// Duplicate VPS name in the registry.
    #[error("vps '{0}' already exists in registry")]
    VpsDuplicate(String),

    /// Local or remote file not found.
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// Invalid CLI argument.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// TLS layer error (handshake, config, PEM, ACME, mTLS) — G-ERR-04.
    #[error("tls: {message}")]
    Tls {
        /// Short context (no embedded cause Display).
        message: String,
        /// Root cause when available.
        #[source]
        source: Option<ErrorSource>,
    },

    /// Cryptographic / secrets key material failure (no secret bytes in Display).
    #[error("crypto operation failed: {op}")]
    Crypto {
        /// Stable operation id (`encrypt`, `decrypt`, `keyring_get`, …).
        op: &'static str,
    },

    /// Configuration / registry serialization failure (non-TOML crate errors).
    #[error("configuration error: {0}")]
    Config(String),

    /// A host service the CLI depends on is not answering (exit 69).
    ///
    /// G-ERR-R01: `Config` was the default landing spot for every `map_err` without an
    /// obvious variant, so a locked OS keyring exited 65 — the same code as corrupt
    /// TOML — and inherited `retryable: false`. An agent therefore gave up permanently
    /// on the one failure in the list that a plain retry actually fixes.
    #[error("service unavailable: {service}")]
    Unavailable {
        /// Stable service id (`keyring`, …).
        service: &'static str,
    },

    /// Internal failure with no user-fixable input (exit 70).
    ///
    /// G-ERR-R01: reserved for conditions the caller cannot influence at all, such as
    /// a CSPRNG that refuses to produce bytes. Kept apart from [`Self::Unavailable`]
    /// because waiting and retrying is useless here, while there it is the remedy.
    #[error("internal failure: {op}")]
    Software {
        /// Stable operation id (`rng`, …).
        op: &'static str,
    },

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

    /// Multi-host fan-out where some targets failed and others succeeded.
    ///
    /// Partial success is the *normal* outcome of a fan-out, not malformed data. It
    /// previously reused [`Self::Config`], which meant an agent could not tell
    /// "1 of 10 hosts failed" apart from "the TOML is corrupt" — both exited 65.
    /// This variant exits [`exit_codes::EX_GENERAL`] and carries the counts so the
    /// agent can branch on scale of failure without parsing prose.
    #[error("{failed}/{total} hosts failed during {op}")]
    PartialFailure {
        /// Number of targets that failed.
        failed: usize,
        /// Total number of targets attempted.
        total: usize,
        /// Stable operation id (`exec`, `health-check`, `scp upload`, …).
        op: &'static str,
    },

    /// Uncategorized error — last resort (prefer typed variants).
    #[error("error: {0}")]
    Generic(String),
}

/// Process exit codes aligned with sysexits.h and Unix signal conventions.
///
/// # Examples
///
/// ```
/// use ssh_cli::errors::exit_codes;
///
/// assert_eq!(exit_codes::EX_OK, 0);
/// assert_eq!(exit_codes::EX_PIPE, 141);
/// assert_eq!(exit_codes::EX_SIGINT, 130);
/// assert_eq!(exit_codes::EX_SIGTERM, 143);
/// assert_eq!(exit_codes::EX_NOPERM, 77);
/// ```
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
    /// A required host service is unavailable (OS keyring, secret service).
    ///
    /// G-ERR-R01: these failures used to exit [`EX_DATAERR`], which told an agent the
    /// *input* was malformed and that retrying was pointless. A locked keyring is the
    /// opposite: the argv is fine and the very same invocation succeeds once the
    /// service is up, so it is classified transient.
    pub const EX_UNAVAILABLE: i32 = 69;
    /// Internal software failure with no user-fixable input (CSPRNG unavailable).
    ///
    /// G-ERR-R01: distinct from [`EX_UNAVAILABLE`] because waiting does not help —
    /// nothing the caller can change makes the next attempt succeed.
    pub const EX_SOFTWARE: i32 = 70;
    /// Cannot create output.
    pub const EX_CANTCREAT: i32 = 73;
    /// I/O error.
    pub const EX_IOERR: i32 = 74;
    /// Permission denied.
    pub const EX_NOPERM: i32 = 77;
    /// Terminated by SIGINT (Ctrl+C).
    pub const EX_SIGINT: i32 = 130;
    /// Broken pipe on stdout/stderr (128 + SIGPIPE=13) — agent/pipe consumers closed early.
    pub const EX_PIPE: i32 = 141;
    /// Terminated by SIGTERM.
    pub const EX_SIGTERM: i32 = 143;

    // Compile-time invariants for sysexits-aligned codes.
    const _: () = assert!(EX_OK == 0);
    const _: () = assert!(EX_USAGE == 64);
    const _: () = assert!(EX_UNAVAILABLE == 69);
    const _: () = assert!(EX_SOFTWARE == 70);
    const _: () = assert!(EX_PIPE == 141);
    const _: () = assert!(EX_SIGINT == 130);
    const _: () = assert!(EX_SIGTERM == 143);
}

/// Returns true when `err` is a broken-pipe condition (EPIPE / SIGPIPE path).
///
/// # Examples
///
/// ```
/// use ssh_cli::errors::is_broken_pipe;
/// use std::io::{Error, ErrorKind};
///
/// assert!(is_broken_pipe(&Error::new(ErrorKind::BrokenPipe, "pipe")));
/// assert!(!is_broken_pipe(&Error::new(ErrorKind::Other, "x")));
/// ```
#[must_use]
pub fn is_broken_pipe(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::BrokenPipe
}

/// Walks `anyhow` / nested sources looking for a broken-pipe I/O error.
#[must_use]
pub fn anyhow_is_broken_pipe(err: &anyhow::Error) -> bool {
    if let Some(ioe) = err.downcast_ref::<std::io::Error>() {
        if is_broken_pipe(ioe) {
            return true;
        }
    }
    if let Some(SshCliError::Io(ioe)) = err.downcast_ref::<SshCliError>() {
        return is_broken_pipe(ioe);
    }
    // Nested chain (e.g. anyhow context wrappers).
    for cause in err.chain() {
        if let Some(ioe) = cause.downcast_ref::<std::io::Error>() {
            if is_broken_pipe(ioe) {
                return true;
            }
        }
        if let Some(SshCliError::Io(ioe)) = cause.downcast_ref::<SshCliError>() {
            if is_broken_pipe(ioe) {
                return true;
            }
        }
    }
    false
}

/// High-level error class for agent retry policy (Rules Rust — retry/backoff).
///
/// Serialized as snake_case in the JSON error envelope (`error_class`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass {
    /// Network / SSH transport may succeed on a fresh process re-invocation.
    Transient,
    /// Fix inputs, credentials, or remote state — do not blind-retry.
    Permanent,
    /// Signal or broken pipe — do not retry.
    Cancelled,
    /// Fan-out where some targets succeeded and others failed; inspect per-host detail.
    Partial,
}

/// Stack layer where the failure was observed (diagnostic only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorLayer {
    /// TCP connect / dial / DNS resolution failures.
    Network,
    /// SSH protocol / channel after TCP is up.
    Ssh,
    /// Authentication or host-key policy.
    Auth,
    /// Local CLI args, registry, schema, paths.
    Application,
    /// Local filesystem / std I/O (non-network).
    Io,
}

/// Detailed retry kind returned by [`SshCliError::retry_kind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryKind {
    /// Never retry with the same argv.
    NotRetryable,
    /// Transient network/connect failure (exit 74).
    TransientNetwork,
    /// Operation deadline exceeded (exit 74).
    TransientTimeout,
    /// SSH session/channel flaked after connect (exit 74).
    TransientSsh,
    /// Auth / host-key — change credentials first.
    PermanentAuth,
    /// Validation / not-found / schema — fix inputs.
    PermanentClient,
    /// Remote command non-zero — not a transport retry.
    PermanentRemoteCommand,
    /// SIGINT/SIGTERM/EPIPE.
    Cancelled,
}

impl SshCliError {
    /// TLS error with message only (no root cause).
    ///
    /// Accepts `&str` / `String` via [`AsRef<str>`] to avoid `Into` ambiguity
    /// with foreign `From<&str>` impls in the dependency graph.
    #[must_use]
    pub fn tls_msg(message: impl AsRef<str>) -> Self {
        Self::Tls {
            message: message.as_ref().to_owned(),
            source: None,
        }
    }

    /// TLS error preserving [`std::error::Error::source`] (G-ERR-04 / G-ERR-16 DRY).
    #[must_use]
    pub fn tls_src(
        message: impl AsRef<str>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Tls {
            message: message.as_ref().to_owned(),
            source: Some(Box::new(source)),
        }
    }

    /// Channel error with message only.
    #[must_use]
    pub fn channel_msg(message: impl AsRef<str>) -> Self {
        Self::ChannelFailed {
            message: message.as_ref().to_owned(),
            source: None,
        }
    }

    /// Channel error preserving source (G-ERR-05 / G-ERR-16 DRY).
    #[must_use]
    pub fn channel_src(
        message: impl AsRef<str>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ChannelFailed {
            message: message.as_ref().to_owned(),
            source: Some(Box::new(source)),
        }
    }

    /// Crypto / secrets failure without embedding secret material.
    #[must_use]
    pub fn crypto(op: &'static str) -> Self {
        Self::Crypto { op }
    }

    /// Builds an [`Self::Unavailable`] for a host service that is not answering.
    #[must_use]
    pub fn unavailable(service: &'static str) -> Self {
        Self::Unavailable { service }
    }

    /// Builds a [`Self::Software`] for an internal failure with no user-fixable input.
    #[must_use]
    pub fn software(op: &'static str) -> Self {
        Self::Software { op }
    }

    /// Stable machine-oriented code for JSON envelopes (G-ERR-08).
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Io(e) if is_broken_pipe(e) => "broken_pipe",
            Self::Io(_) => "io",
            Self::Json(_) => "json",
            Self::TomlDe(_) => "toml_read",
            Self::TomlSer(_) => "toml_write",
            Self::Domain(_) => "domain_validation",
            Self::SshConnection(_) => "ssh_connection",
            Self::SshAuthentication(_) => "ssh_authentication",
            Self::ConnectionFailed(_) => "connection_failed",
            Self::AuthenticationFailed => "authentication_failed",
            Self::HostKeyChanged { .. } => "host_key_changed",
            Self::CommandTooLong { .. } => "command_too_long",
            Self::SudoDisabled => "sudo_disabled",
            Self::SuPasswordMissing => "su_password_missing",
            Self::ChannelFailed { .. } => "channel_failed",
            Self::SshTimeout(_) => "ssh_timeout",
            Self::CommandFailed { .. } => "command_failed",
            Self::VpsNotFound(_) => "vps_not_found",
            Self::NoActiveVps => "no_active_vps",
            Self::VpsDuplicate(_) => "vps_duplicate",
            Self::FileNotFound(_) => "file_not_found",
            Self::InvalidArgument(_) => "invalid_argument",
            Self::Tls { .. } => "tls",
            Self::Crypto { .. } => "crypto",
            Self::Config(_) => "config",
            Self::Unavailable { .. } => "unavailable",
            Self::Software { .. } => "software",
            Self::Timeout(_) => "timeout",
            Self::XdgDirectory => "xdg_directory",
            Self::SchemaIncompatible { .. } => "schema_incompatible",
            Self::PartialFailure { .. } => "partial_failure",
            Self::Generic(_) => "generic",
        }
    }

    /// Returns the sysexits.h exit code for this error.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            // G-IO-01: EPIPE → 141 (never report as generic I/O 74).
            Self::Io(e) if is_broken_pipe(e) => exit_codes::EX_PIPE,
            Self::Io(_) => exit_codes::EX_IOERR,
            Self::Json(_) => exit_codes::EX_DATAERR,
            Self::TomlDe(_) => exit_codes::EX_DATAERR,
            Self::TomlSer(_) => exit_codes::EX_CANTCREAT,
            Self::Domain(_) => exit_codes::EX_USAGE,
            Self::SshConnection(_) => exit_codes::EX_IOERR,
            // GAP-AUD-020: authentication failures use EX_NOPERM (77); connect/IO stay 74.
            Self::SshAuthentication(_) => exit_codes::EX_NOPERM,
            Self::ConnectionFailed(_) => exit_codes::EX_IOERR,
            Self::AuthenticationFailed => exit_codes::EX_NOPERM,
            Self::HostKeyChanged { .. } => exit_codes::EX_NOPERM,
            Self::CommandTooLong { .. } => exit_codes::EX_USAGE,
            Self::SudoDisabled => exit_codes::EX_NOPERM,
            Self::SuPasswordMissing => exit_codes::EX_USAGE,
            Self::ChannelFailed { .. } => exit_codes::EX_IOERR,
            Self::SshTimeout(_) => exit_codes::EX_IOERR,
            Self::CommandFailed { .. } => exit_codes::EX_GENERAL,
            Self::VpsNotFound(_) => exit_codes::EX_NOINPUT,
            Self::NoActiveVps => exit_codes::EX_NOINPUT,
            Self::VpsDuplicate(_) => exit_codes::EX_USAGE,
            Self::FileNotFound(_) => exit_codes::EX_NOINPUT,
            Self::InvalidArgument(_) => exit_codes::EX_USAGE,
            Self::Tls { .. } => exit_codes::EX_IOERR,
            Self::Crypto { .. } => exit_codes::EX_NOPERM,
            Self::Config(_) => exit_codes::EX_DATAERR,
            Self::Unavailable { .. } => exit_codes::EX_UNAVAILABLE,
            Self::Software { .. } => exit_codes::EX_SOFTWARE,
            Self::Timeout(_) => exit_codes::EX_IOERR,
            Self::XdgDirectory => exit_codes::EX_CANTCREAT,
            Self::SchemaIncompatible { .. } => exit_codes::EX_DATAERR,
            // Partial fan-out is not a data error: EX_DATAERR (65) is reserved for
            // malformed input. Per-host detail travels in the JSON envelope.
            Self::PartialFailure { .. } => exit_codes::EX_GENERAL,
            Self::Generic(_) => exit_codes::EX_GENERAL,
        }
    }

    /// Typed retry kind (no string matching on `Display`).
    #[must_use]
    pub fn retry_kind(&self) -> RetryKind {
        match self {
            Self::Io(e) if is_broken_pipe(e) => RetryKind::Cancelled,
            // Non-pipe I/O maps to exit 74 — agent may re-invoke (same as transport).
            Self::Io(_) => RetryKind::TransientNetwork,
            Self::Json(_) | Self::TomlDe(_) | Self::TomlSer(_) | Self::Config(_) => {
                RetryKind::PermanentClient
            }
            Self::Domain(_) => RetryKind::PermanentClient,
            Self::SshConnection(_) => RetryKind::TransientSsh,
            Self::SshAuthentication(_)
            | Self::AuthenticationFailed
            | Self::HostKeyChanged { .. } => RetryKind::PermanentAuth,
            Self::ConnectionFailed(_) | Self::Tls { .. } => RetryKind::TransientNetwork,
            // G-ERR-R01: a keyring that is locked, not yet started, or momentarily
            // busy answers the *same* argv successfully a moment later. This is the
            // one case in the old `Config` bucket where retrying is the fix, and
            // classifying it permanent is what made agents give up on it.
            Self::Unavailable { .. } => RetryKind::TransientTimeout,
            // No amount of waiting repairs a CSPRNG, so this stays permanent even
            // though it is not the caller's fault.
            Self::Software { .. } => RetryKind::PermanentClient,
            Self::CommandTooLong { .. }
            | Self::SudoDisabled
            | Self::SuPasswordMissing
            | Self::VpsNotFound(_)
            | Self::NoActiveVps
            | Self::VpsDuplicate(_)
            | Self::FileNotFound(_)
            | Self::InvalidArgument(_)
            | Self::Crypto { .. }
            | Self::XdgDirectory
            | Self::SchemaIncompatible { .. }
            | Self::Generic(_) => RetryKind::PermanentClient,
            // Never blind-retry a batch: the hosts that succeeded would run twice.
            Self::PartialFailure { .. } => RetryKind::NotRetryable,
            Self::ChannelFailed { .. } => RetryKind::TransientSsh,
            Self::SshTimeout(_) | Self::Timeout(_) => RetryKind::TransientTimeout,
            Self::CommandFailed { .. } => RetryKind::PermanentRemoteCommand,
        }
    }

    /// Whether an agent may re-invoke the CLI with the same argv after backoff.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.retry_kind(),
            RetryKind::TransientNetwork | RetryKind::TransientTimeout | RetryKind::TransientSsh
        )
    }

    /// Explicit complement of [`Self::is_retryable`].
    #[must_use]
    pub fn is_permanent(&self) -> bool {
        !self.is_retryable() && !matches!(self.retry_kind(), RetryKind::Cancelled)
    }

    /// High-level class for the JSON envelope `error_class` field.
    #[must_use]
    pub fn classify(&self) -> ErrorClass {
        // Partial fan-out is its own class: not transient (retrying the whole batch
        // re-runs the hosts that already succeeded) and not plain permanent (part of
        // the work did land). The agent must read per-host detail to decide.
        if matches!(self, Self::PartialFailure { .. }) {
            return ErrorClass::Partial;
        }
        match self.retry_kind() {
            RetryKind::TransientNetwork | RetryKind::TransientTimeout | RetryKind::TransientSsh => {
                ErrorClass::Transient
            }
            RetryKind::Cancelled => ErrorClass::Cancelled,
            RetryKind::NotRetryable
            | RetryKind::PermanentAuth
            | RetryKind::PermanentClient
            | RetryKind::PermanentRemoteCommand => ErrorClass::Permanent,
        }
    }

    /// Stack layer for diagnostics.
    #[must_use]
    pub fn layer(&self) -> ErrorLayer {
        match self {
            Self::Io(e) if is_broken_pipe(e) => ErrorLayer::Io,
            Self::Io(e) if io_error_is_transient_network(e) => ErrorLayer::Network,
            Self::Io(_) => ErrorLayer::Io,
            Self::ConnectionFailed(_) | Self::Timeout(_) | Self::Tls { .. } => ErrorLayer::Network,
            Self::SshConnection(_) | Self::ChannelFailed { .. } | Self::SshTimeout(_) => {
                ErrorLayer::Ssh
            }
            Self::SshAuthentication(_)
            | Self::AuthenticationFailed
            | Self::HostKeyChanged { .. }
            | Self::SudoDisabled
            | Self::Crypto { .. }
            // Keyring failures are credential-material failures, not application ones.
            | Self::Unavailable { .. } => ErrorLayer::Auth,
            Self::Json(_)
            | Self::TomlDe(_)
            | Self::TomlSer(_)
            | Self::Domain(_)
            | Self::CommandTooLong { .. }
            | Self::SuPasswordMissing
            | Self::CommandFailed { .. }
            | Self::VpsNotFound(_)
            | Self::NoActiveVps
            | Self::VpsDuplicate(_)
            | Self::FileNotFound(_)
            | Self::InvalidArgument(_)
            | Self::Config(_)
            | Self::Software { .. }
            | Self::XdgDirectory
            | Self::SchemaIncompatible { .. }
            | Self::PartialFailure { .. }
            | Self::Generic(_) => ErrorLayer::Application,
        }
    }

    /// Optional cool-down hint (SSH has no HTTP Retry-After; always `None`).
    #[must_use]
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        let _ = self;
        None
    }

    /// Short agent-facing suggestion for the JSON envelope.
    #[must_use]
    pub fn suggestion(&self) -> Option<&'static str> {
        // G-ERR-R01: these two share a `RetryKind` with unrelated failures, and the
        // generic hint derived from it would be actively misleading — telling an
        // operator to raise `--timeout` when the OS keyring is locked sends them to
        // the wrong knob entirely. Matching the variant first keeps the advice true.
        match self {
            Self::Unavailable { .. } => {
                return Some(
                    "unlock or start the OS keyring, or fall back to XDG `secrets.key` \
                     (`--secrets-key-file`); the same argv succeeds once it answers (exit 69)",
                );
            }
            Self::Software { .. } => {
                return Some(
                    "internal failure with no user-fixable input; report it — retrying \
                     unchanged will not help (exit 70)",
                );
            }
            _ => {}
        }
        match self.retry_kind() {
            RetryKind::TransientNetwork | RetryKind::TransientSsh => {
                Some("retry at most twice with exponential full-jitter backoff (exit 74)")
            }
            RetryKind::TransientTimeout => {
                Some("increase --timeout / --timeout-ms, then retry at most twice with backoff")
            }
            RetryKind::PermanentAuth => {
                Some("change credentials (--key / --password-stdin) or host-key policy; do not blind-retry")
            }
            RetryKind::PermanentRemoteCommand => {
                Some("inspect remote stderr; fix remote command — transport retry will not help")
            }
            RetryKind::PermanentClient => {
                Some("fix CLI arguments, registry state, or schema; do not retry unchanged")
            }
            RetryKind::Cancelled => Some("do not retry after signal or broken pipe"),
            RetryKind::NotRetryable => None,
        }
    }
}

/// Classifies `std::io::Error` kinds that are typically transient on the network path.
#[must_use]
pub fn io_error_is_transient_network(err: &std::io::Error) -> bool {
    use std::io::ErrorKind;
    matches!(
        err.kind(),
        ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected
            | ErrorKind::AddrNotAvailable
            | ErrorKind::TimedOut
            | ErrorKind::Interrupted
            | ErrorKind::WouldBlock
            | ErrorKind::UnexpectedEof
            | ErrorKind::NetworkUnreachable
            | ErrorKind::HostUnreachable
    )
}

/// Result alias using [`SshCliError`].
pub type SshCliResult<T> = std::result::Result<T, SshCliError>;

/// Turns a multi-host fan-out tally into the batch outcome.
///
/// Single source for the "N of M failed" verdict. The same eight-line block used to
/// be copy-pasted across the SCP, SFTP, exec and health-check batch paths, each one
/// building a [`SshCliError::Config`] (exit 65) by hand — so a partial fan-out was
/// indistinguishable from corrupt TOML. Callers now report the tally and let this
/// decide.
///
/// `total` must count **every** target the caller was asked to reach, including ones
/// skipped by `--fail-fast`; otherwise the ratio understates the work not done.
///
/// # Errors
/// [`SshCliError::PartialFailure`] when `failed > 0`.
///
/// # Examples
///
/// ```
/// use ssh_cli::errors::{finish_batch, SshCliError};
///
/// assert!(finish_batch(0, 10, "exec").is_ok());
///
/// let err = finish_batch(3, 10, "exec").unwrap_err();
/// assert_eq!(err.exit_code(), ssh_cli::errors::exit_codes::EX_GENERAL);
/// assert_eq!(err.error_code(), "partial_failure");
/// assert!(matches!(err, SshCliError::PartialFailure { failed: 3, total: 10, .. }));
/// ```
pub fn finish_batch(failed: usize, total: usize, op: &'static str) -> SshCliResult<()> {
    if failed == 0 {
        return Ok(());
    }
    Err(SshCliError::PartialFailure { failed, total, op })
}

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;
