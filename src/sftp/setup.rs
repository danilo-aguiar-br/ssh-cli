// SPDX-License-Identifier: MIT OR Apache-2.0
//! SFTP option plumbing and session bootstrap (A7 split).
#![forbid(unsafe_code)]
#![allow(unused_imports)]

use super::*;

#[derive(Debug, Default, Clone)]
/// Per-invocation SFTP options resolved at the CLI boundary.
pub struct SftpOptions {
    /// SSH password (resolved).
    pub password: Option<secrecy::SecretString>,
    /// Private key path.
    pub key: Option<String>,
    /// Key passphrase (resolved).
    pub key_passphrase: Option<secrecy::SecretString>,
    /// Total connect+op timeout ms.
    pub timeout: Option<crate::domain::TimeoutMs>,
    /// Replace divergent host key.
    pub replace_host_key: bool,
    /// Emit JSON success envelopes.
    pub json: bool,
    /// Use ssh-agent (CLI/XDG only).
    pub use_agent: bool,
    /// Agent socket / named pipe path.
    pub agent_socket: Option<String>,
    /// Recursive tree transfer.
    pub recursive: bool,
}

/// Applies CLI overrides onto a VPS record (incl. agent — G-SFTP-18).
pub(crate) fn apply_sftp_options(record: &mut crate::vps::model::VpsRecord, opts: &SftpOptions) {
    if let Some(ref pwd) = opts.password {
        record.password = pwd.clone();
    }
    if let Some(ref k) = opts.key {
        if let Ok(kp) = crate::domain::KeyPath::try_new(k.as_str()) {
            record.key_path = Some(kp);
        }
    }
    if let Some(ref kp) = opts.key_passphrase {
        record.key_passphrase = Some(kp.clone());
    }
    if let Some(t) = opts.timeout {
        record.timeout_ms = t;
    }
    if opts.use_agent {
        record.use_agent = true;
    }
    if let Some(ref sock) = opts.agent_socket {
        record.agent_socket = Some(sock.clone());
        record.use_agent = true;
    }
}

pub(crate) async fn connect_client(
    vps_key: &str,
    config_override: Option<&std::path::Path>,
    opts: &SftpOptions,
) -> anyhow::Result<SshClient> {
    let mut record = vps::find_by_name(config_override, vps_key)?
        .ok_or_else(|| SshCliError::VpsNotFound(vps_key.to_owned()))?;
    apply_sftp_options(&mut record, opts);
    let path = vps::resolve_config_path(config_override)?;
    let cfg = vps::build_connection_config(&record, Some(&path), opts.replace_host_key);
    let client = SshClient::connect(cfg).await?;
    Ok(client)
}

pub(crate) fn remote_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}
