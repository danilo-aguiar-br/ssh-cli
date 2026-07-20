// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! VPS record CRUD and persistence (XDG + atomic TOML + flock).
//!
//! No `.env` at runtime. Schema v3 English wire (dual-read PT legacy).

pub mod model;
/// Config path/load/save/permissions (SRP extract — G-UNSAFE-10).
mod config_io;
/// Multi-host selection resolution (SRP extract — G-COMP-01).
pub mod selection;
/// Local doctor + optional SSH probe (SRP extract — G-COMP-02).
mod doctor;
/// Inventory import/export (SRP extract — G-COMP-03).
mod import_export;
/// SSH health-check fan-out (SRP extract — G-COMP-04).
mod health;
/// Remote exec / sudo / su (SRP extract — G-COMP-05).
mod exec_ops;
/// VPS CRUD dispatcher (SRP extract — G-COMP-06).
mod crud;
/// Secrets primary-key commands (SRP extract — G-COMP-07).
mod secrets_cmd;

pub use config_io::{
    default_config_path, load, resolve_config_path, save, winning_layer, write_atomic, ConfigFile,
    ConfigLayer,
};
pub(crate) use config_io::validate_key_path_exists;
pub use selection::{dedupe_host_names, resolve_host_jobs, HostSelection};
pub use health::{run_health_check, HostHealthResult};
pub use exec_ops::{
    run_exec, run_exec_with_client, run_sudo_exec, run_sudo_exec_with_client, run_su_exec,
    ExecOptions, HostExecResult,
};
pub use crud::run_vps_command;
pub use secrets_cmd::run_secrets_command;
pub use import_export::parse_import_payload;

use crate::cli::OutputFormat;
use crate::errors::{SshCliError, SshCliResult};
use crate::ssh::client::ConnectionConfig;
use crate::ssh::known_hosts::KnownHosts;
use anyhow::Result;
use model::{effective_limit, VpsRecord};
use secrecy::SecretString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// JSON efetivo a partir de flag local e format global (IO-001/002).
#[must_use]
pub fn use_json(json_local: bool, format: OutputFormat) -> bool {
    json_local || format == OutputFormat::Json
}

/// Hard cap for secret payloads on stdin (agent hardening / DoS guard).
///
/// Passwords and key passphrases must not be multi-megabyte streams.
pub const MAX_SECRET_STDIN_BYTES: u64 = 64 * 1024;

const _: () = assert!(MAX_SECRET_STDIN_BYTES >= 1024);
const _: () = assert!(MAX_SECRET_STDIN_BYTES <= 1024 * 1024);

/// Reads a password line from stdin (no extra echo).
///
/// G-IO-06: rejects payloads larger than [`MAX_SECRET_STDIN_BYTES`] with
/// `EX_DATAERR` semantics via [`SshCliError::InvalidArgument`].
///
/// G-SECDEV-01: returns [`SecretString`] immediately (rules: never keep
/// credentials in bare `String` after the trust boundary). The read buffer is
/// [`zeroize::Zeroizing`] so leftover CR/LF bytes are scrubbed on drop.
pub fn read_secret_stdin() -> SshCliResult<SecretString> {
    use std::io::Read;
    use zeroize::Zeroizing;
    let mut limited = std::io::stdin().take(MAX_SECRET_STDIN_BYTES + 1);
    let mut buf = Zeroizing::new(String::new());
    limited.read_to_string(&mut buf)?;
    if buf.len() as u64 > MAX_SECRET_STDIN_BYTES {
        return Err(SshCliError::InvalidArgument(format!(
            "stdin secret exceeds max size of {MAX_SECRET_STDIN_BYTES} bytes"
        )));
    }
    let trimmed = buf.trim_end_matches(['\r', '\n']);
    Ok(SecretString::from(trimmed.to_owned()))
}

/// Applies runtime overrides onto a cloned `VpsRecord`.
///
/// Parameter order: password, sudo, su, timeout, key_path, key_passphrase, use_agent, agent_socket.
///
/// G-SECDEV-02: secret overrides are already [`SecretString`] (zeroize-on-drop);
/// never re-accept bare password `String` past the CLI boundary.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_overrides(
    vps: &mut VpsRecord,
    password_override: Option<SecretString>,
    sudo_password_override: Option<SecretString>,
    su_password_override: Option<SecretString>,
    timeout_override: Option<crate::domain::TimeoutMs>,
    key_path_override: Option<String>,
    key_passphrase_override: Option<SecretString>,
    use_agent: bool,
    agent_socket: Option<String>,
) {
    use crate::domain::KeyPath;
    if let Some(pwd) = password_override {
        vps.password = pwd;
    }
    if let Some(spwd) = sudo_password_override {
        vps.sudo_password = Some(spwd);
    }
    if let Some(sp) = su_password_override {
        vps.su_password = Some(sp);
    }
    // G-TYPE-18: timeout already refined at the CLI / options boundary.
    if let Some(t) = timeout_override {
        vps.timeout_ms = t;
    }
    if let Some(k) = key_path_override {
        if let Ok(kp) = KeyPath::try_new(k) {
            vps.key_path = Some(kp);
        }
    }
    if let Some(kp) = key_passphrase_override {
        vps.key_passphrase = Some(kp);
    }
    if use_agent {
        vps.use_agent = true;
    }
    if let Some(sock) = agent_socket {
        vps.agent_socket = Some(sock);
        vps.use_agent = true;
    }
}

pub(crate) fn validate_command_length(command: &str, max_command_chars: usize) -> SshCliResult<()> {
    let lim = effective_limit(max_command_chars);
    let len = command.chars().count();
    if len > lim {
        return Err(SshCliError::CommandTooLong {
            max: max_command_chars,
            len,
        });
    }
    if command.trim().is_empty() {
        return Err(SshCliError::InvalidArgument("empty command".to_string()));
    }
    // G-PROC-03: reject NUL in remote shell payloads. C-string / argv truncation
    // and opaque binary injection must not reach `channel.exec` / `sh -c` packing.
    // (CR/LF are allowed — multi-line remote scripts are intentional.)
    if command.as_bytes().contains(&0) {
        return Err(SshCliError::InvalidArgument(
            "command contains null byte".to_string(),
        ));
    }
    Ok(())
}

/// Sets the active VPS by writing its name to `<config_dir>/active` (sibling file).
///
/// Workload: **local marker file**. Sequential justified: single write; no host fan-out.
pub async fn run_connect(
    name: &str,
    config_override: Option<PathBuf>,
    format: OutputFormat,
) -> Result<()> {
    let path = resolve_config_path(config_override.as_deref())?;
    let file = load(&path)?;
    if !file.hosts.contains_key(name) {
        return Err(SshCliError::VpsNotFound(name.to_string()).into());
    }

    let active_file = path
        .parent()
        .map(|p| p.join(crate::constants::ACTIVE_VPS_FILE_NAME))
        .unwrap_or_else(|| PathBuf::from(crate::constants::ACTIVE_VPS_FILE_NAME));
    if let Some(parent_dir) = active_file.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }
    // atomic write of active marker
    let parent_dir = active_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut tmp = tempfile::NamedTempFile::new_in(&parent_dir)?;
    tmp.write_all(name.as_bytes())?;
    tmp.as_file().sync_data()?;
    tmp.persist(&active_file)
        .map_err(|e| SshCliError::Io(e.error))?;
    crate::output::emit_success(
        "vps-connected",
        serde_json::json!({ "name": name }),
        &crate::i18n::t(crate::i18n::Message::VpsActiveSelected {
            name: name.to_string(),
        }),
        format == OutputFormat::Json,
    )?;
    Ok(())
}

/// Looks up a VPS record by name.
///
/// Borrows the config override; returns an owned [`VpsRecord`] (cloned from the
/// on-disk map) so the caller can mutate without holding the file open.
pub fn find_by_name(
    config_override: Option<&Path>,
    name: &str,
) -> SshCliResult<Option<VpsRecord>> {
    let path = resolve_config_path(config_override)?;
    let file = load(&path)?;
    Ok(file.hosts.get(name).cloned())
}

/// Reads the active VPS name.
pub fn read_active_vps(config_override: Option<&Path>) -> SshCliResult<Option<String>> {
    let path = resolve_config_path(config_override)?;
    let active_file = path
        .parent()
        .map(|p| p.join(crate::constants::ACTIVE_VPS_FILE_NAME))
        .unwrap_or_else(|| PathBuf::from(crate::constants::ACTIVE_VPS_FILE_NAME));
    if !active_file.exists() {
        return Ok(None);
    }
    let name = std::fs::read_to_string(&active_file)?;
    Ok(Some(name.trim().to_string()))
}

/// Builds `ConnectionConfig` from a `VpsRecord`.
pub fn build_connection_config(
    vps: &VpsRecord,
    config_toml: Option<&Path>,
    replace_host_key: bool,
) -> ConnectionConfig {
    let known_hosts_path = config_toml.map(KnownHosts::path_beside_config);
    let tls = if vps.tls {
        let sni = vps
            .tls_sni
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| vps.host.as_str());
        let client_cert = vps
            .tls_client_cert
            .as_ref()
            .map(|p| std::path::PathBuf::from(p.as_str()));
        let client_key = vps
            .tls_client_key
            .as_ref()
            .map(|p| std::path::PathBuf::from(p.as_str()));
        match crate::tls::TlsConnectOptions::try_new(sni, client_cert, client_key) {
            Ok(o) => Some(o),
            Err(e) => {
                tracing::warn!(err = %e, "invalid TLS options on VPS record; plain SSH");
                None
            }
        }
    } else {
        None
    };
    ConnectionConfig {
        host: vps.host.clone(),
        port: vps.port,
        username: vps.username.clone(),
        password: vps.password.clone(),
        key_path: vps.key_path.clone(),
        key_passphrase: vps.key_passphrase.clone(),
        timeout_ms: vps.timeout_ms,
        known_hosts_path,
        replace_host_key,
        tls,
        use_agent: vps.use_agent,
        agent_socket: vps.agent_socket.as_ref().map(std::path::PathBuf::from),
    }
}

// Exec family: see `exec_ops` module (G-COMP-05 + G-DRY-01).

// Health-check: see `health` module (G-COMP-04).


#[cfg(test)]
#[path = "tests.rs"]
mod tests;
