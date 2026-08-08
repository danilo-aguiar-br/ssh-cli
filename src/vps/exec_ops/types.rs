// SPDX-License-Identifier: MIT OR Apache-2.0
//! Option and result types shared by every exec path (A7 split).
#![forbid(unsafe_code)]
#![allow(unused_imports)]
use super::*;

/// Common remote execution options.
///
/// G-SECDEV-02: password fields are [`SecretString`] so zeroize-on-drop applies
/// through multi-host clone/fan-out (secrecy 0.10 `SecretString: Clone`).
///
/// G-TYPE-18/19: `timeout` is [`crate::domain::TimeoutMs`]; `steps` are
/// [`crate::domain::RemoteCommand`].
#[derive(Debug, Default, Clone)]
pub struct ExecOptions {
    /// Override password.
    pub password: Option<SecretString>,
    /// Override sudo.
    pub sudo_password: Option<SecretString>,
    /// Override su.
    pub su_password: Option<SecretString>,
    /// Override timeout (refined at CLI boundary).
    pub timeout: Option<crate::domain::TimeoutMs>,
    /// Override key path.
    pub key: Option<String>,
    /// Override key passphrase.
    pub key_passphrase: Option<SecretString>,
    /// Use ssh-agent (G-SSH-04).
    pub use_agent: bool,
    /// Agent socket path (CLI/XDG).
    pub agent_socket: Option<String>,
    /// Optional shell description comment.
    pub description: Option<String>,
    /// replace host key.
    pub replace_host_key: bool,
    /// disable sudo global.
    pub disable_sudo: bool,
    /// Extra commands on the same SSH session after the primary (G-O3 / G-TYPE-19).
    pub steps: Vec<crate::domain::RemoteCommand>,
}

impl ExecOptions {
    /// Moves the credential half of the options into an [`AuthOverrides`].
    ///
    /// B3: every exec entry point used to spell the same eight positional
    /// arguments into `apply_overrides`. Naming them once here removes four
    /// copies of that list and the transposition risk that came with it.
    pub(crate) fn take_auth_overrides(&mut self) -> crate::vps::AuthOverrides {
        crate::vps::AuthOverrides {
            password: self.password.take(),
            sudo_password: self.sudo_password.take(),
            su_password: self.su_password.take(),
            timeout: self.timeout,
            key_path: self.key.take(),
            key_passphrase: self.key_passphrase.take(),
            use_agent: self.use_agent,
            agent_socket: self.agent_socket.take(),
        }
    }
}

/// Kind of remote elevation for multi-host exec fan-out.
#[derive(Clone, Copy)]
pub(crate) enum ExecKind {
    Plain,
    Sudo,
    Su,
}

/// Per-host result for multi-host exec JSON/text.
#[derive(Debug, Clone)]
pub struct HostExecResult {
    /// VPS name.
    pub name: String,
    /// Whether the remote command succeeded (exit 0).
    pub ok: bool,
    /// Remote exit code when available.
    pub exit_code: Option<i32>,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr or local error text.
    pub stderr: String,
    /// Wall duration in milliseconds.
    pub duration_ms: u64,
    /// Error summary when `ok` is false.
    pub error: Option<String>,
}

pub(crate) fn cancelled_err() -> anyhow::Error {
    anyhow::anyhow!(crate::i18n::t(crate::i18n::Message::OperationCancelled))
}

pub(crate) fn expect_single(selection: HostSelection) -> Result<String> {
    match selection {
        HostSelection::Single(name) => Ok(name.into_inner()),
        _ => Err(SshCliError::InvalidArgument(
            "internal: expected single-host selection for non-batch exec".into(),
        )
        .into()),
    }
}
