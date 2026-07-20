// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP-R / G-TYPE-11/15 / G-SSH: ConnectionConfig uses domain newtypes (SRP — connect params).
#![forbid(unsafe_code)]
//! SSH connection configuration (auth + timeout + TOFU paths).
//!
//! # Auth policy (G-SSH-15)
//!
//! - **Public key file** (`key_path`) is preferred when present.
//! - **ssh-agent** when `use_agent` is true (socket from CLI/XDG — never env-as-store).
//! - **Password** is an opt-in fallback: a non-empty `password` SecretString in the
//!   XDG inventory (or CLI override) enables password auth after key/agent failure.
//!   An empty password does **not** enable password auth.

use crate::domain::{secret_nonempty, KeyPath, SshHost, SshPort, SshUser, TimeoutMs};
use crate::errors::{SshCliError, SshCliResult};
use crate::tls::TlsConnectOptions;
use secrecy::SecretString;
use std::path::PathBuf;

/// SSH connection configuration.
///
/// Built from a [`crate::vps::model::VpsRecord`] at the time
/// of the call. Auth: private key (preferred), optional agent, and/or password.
///
/// Host, port, user, and timeout are domain-proven — empty host / port 0 are
/// unrepresentable (G-TYPE-11: no redundant field checks).
///
/// When [`Self::tls`] is `Some`, the client dials TCP then completes a rustls
/// handshake (optional mTLS) before the SSH protocol (SSH-over-TLS).
#[derive(Clone)]
pub struct ConnectionConfig {
    /// SSH server hostname or IP.
    pub host: SshHost,
    /// SSH server TCP port (always ≥ 1).
    pub port: SshPort,
    /// SSH username.
    pub username: SshUser,
    /// SSH password (`SecretString` for automatic zeroize); may be empty for key-only.
    pub password: SecretString,
    /// OpenSSH private key path (optional).
    pub key_path: Option<KeyPath>,
    /// Key passphrase (optional).
    pub key_passphrase: Option<SecretString>,
    /// Total timeout for connect + handshake + authentication + exec, in ms.
    pub timeout_ms: TimeoutMs,
    /// Path to known_hosts file (TOFU). `None` = test-only always-trust / product fail-closed.
    pub known_hosts_path: Option<PathBuf>,
    /// When true, allow replacing a divergent host key fingerprint.
    pub replace_host_key: bool,
    /// Optional TLS wrapper (SSH-over-TLS / mTLS). `None` = plain TCP SSH.
    pub tls: Option<TlsConnectOptions>,
    /// Attempt ssh-agent publickey auth (G-SSH-04). Socket from [`Self::agent_socket`]
    /// or platform default (Windows named pipe only — Unix requires explicit path).
    pub use_agent: bool,
    /// Agent socket / named-pipe path (CLI or XDG). Never read from env store.
    pub agent_socket: Option<PathBuf>,
}

impl std::fmt::Debug for ConnectionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionConfig")
            .field("host", &self.host.as_str())
            .field("port", &self.port.get())
            .field("username", &self.username.as_str())
            .field("password", &"<redacted>")
            .field("key_path", &self.key_path.as_ref().map(|k| k.as_path()))
            .field(
                "key_passphrase",
                &self.key_passphrase.as_ref().map(|_| "<redacted>"),
            )
            .field("timeout_ms", &self.timeout_ms.get())
            .field("known_hosts_path", &self.known_hosts_path)
            .field("replace_host_key", &self.replace_host_key)
            .field("tls", &self.tls)
            .field("use_agent", &self.use_agent)
            .field("agent_socket", &self.agent_socket)
            .finish()
    }
}

impl ConnectionConfig {
    /// Validates authentication material only (host/port/user proven by types).
    ///
    /// Accepts password, key path, and/or agent (G-SSH-17).
    pub fn validate(&self) -> SshCliResult<()> {
        let has_password = secret_nonempty(&self.password);
        let has_key = self.key_path.is_some();
        let has_agent = self.use_agent;
        if !has_password && !has_key && !has_agent {
            return Err(SshCliError::InvalidArgument(
                "auth requires password, key_path, or --use-agent".to_string(),
            ));
        }
        if has_agent {
            // Fail closed early when Unix agent has no socket path.
            #[cfg(unix)]
            if self.agent_socket.is_none() {
                return Err(SshCliError::InvalidArgument(
                    "use_agent requires --agent-socket PATH (or agent_socket in XDG VPS record); \
                     env SSH_AUTH_SOCK is not used as product store"
                        .into(),
                ));
            }
        }
        Ok(())
    }

    /// Resolved agent endpoint path (CLI/XDG or Windows platform default).
    #[must_use]
    pub fn resolved_agent_socket(&self) -> Option<PathBuf> {
        if !self.use_agent {
            return None;
        }
        if let Some(p) = &self.agent_socket {
            return Some(p.clone());
        }
        #[cfg(windows)]
        {
            return Some(PathBuf::from(crate::constants::WINDOWS_SSH_AGENT_PIPE));
        }
        #[cfg(not(windows))]
        {
            None
        }
    }
}
