// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Field application for `ssh-cli vps edit` / `vps rename`.
//!
//! # Why this lives apart from the dispatcher
//!
//! `crud.rs` carried a 186-line `Edit` arm that read stdin, locked the registry,
//! mutated the record, validated TLS and emitted the envelope in one block. The
//! arm could not be exercised without a filesystem, and the rename it now needs
//! would have doubled it. [`EditRequest`] names every input, [`apply_fields`] is
//! pure apart from the key-path existence check, and the locking/IO sequence
//! stays in the parent module where it can be read in order.
//!
//! # Order of application
//!
//! Clear flags run **before** value flags. `clap` already rejects a clear flag
//! paired with the value flag of the same field, so the order is not load
//! bearing for the same field; it is load bearing for `--clear-tags --tag a`,
//! which must end with `["a"]` rather than `[]`.
//!
//! # What `updated` means
//!
//! A field is listed only when its stored value actually changed, once, in the
//! order it was applied. Naming a flag whose value already matches the record is
//! a no-op and leaves the list empty, which is what lets the caller skip the
//! write. Secrets are compared through `expose_secret()` on both sides and never
//! copied: the comparison only detects a local change, there is no adversary
//! timing it, and the plaintext stays inside the `secrecy` wrappers.

use super::auth::apply_auth;
use crate::domain::{CharLimit, HostTag, SshHost, SshPort, SshUser, TimeoutMs};
use crate::errors::SshCliError;
use crate::vps::model::VpsRecord;
use secrecy::{ExposeSecret, SecretString};
use std::path::PathBuf;

/// Secrets already read from stdin.
///
/// Read before the config lock is taken: stdin drains once, and a blocking read
/// must never be performed while holding a `flock` that another one-shot waits on.
#[derive(Debug, Default)]
pub struct StdinSecrets {
    /// `--password-stdin`.
    pub password: Option<SecretString>,
    /// `--key-passphrase-stdin`.
    pub key_passphrase: Option<SecretString>,
    /// `--sudo-password-stdin`.
    pub sudo_password: Option<SecretString>,
    /// `--su-password-stdin`.
    pub su_password: Option<SecretString>,
}

/// Everything `vps edit` can change, named so a swapped pair cannot compile.
#[derive(Debug, Default)]
pub struct EditRequest {
    /// Current VPS name (the target designated on argv).
    pub name: String,
    /// New VPS name (`--name`), which renames the record.
    pub new_name: Option<String>,
    /// New hostname/IP.
    pub host: Option<String>,
    /// New SSH port.
    pub port: Option<u16>,
    /// New username.
    pub user: Option<String>,
    /// New password.
    pub password: Option<String>,
    /// Reads the password from stdin (stdin drains once per invocation).
    pub password_stdin: bool,
    /// New private key path.
    pub key: Option<PathBuf>,
    /// New key passphrase.
    pub key_passphrase: Option<String>,
    /// Reads the key passphrase from stdin.
    pub key_passphrase_stdin: bool,
    /// Switch primary auth to the SSH agent.
    pub use_agent: bool,
    /// Explicit agent socket (implies agent auth).
    pub agent_socket: Option<PathBuf>,
    /// New SSH timeout.
    pub timeout: Option<u64>,
    /// New command char limit.
    pub max_command_chars: Option<usize>,
    /// New output char limit.
    pub max_output_chars: Option<usize>,
    /// New sudo password.
    pub sudo_password: Option<String>,
    /// Reads the sudo password from stdin.
    pub sudo_password_stdin: bool,
    /// New su password.
    pub su_password: Option<String>,
    /// Reads the su password from stdin.
    pub su_password_stdin: bool,
    /// Disable sudo/su elevation.
    pub disable_sudo: bool,
    /// Re-enable sudo/su elevation.
    pub enable_sudo: bool,
    /// Enable SSH-over-TLS.
    pub tls: bool,
    /// Disable SSH-over-TLS.
    pub no_tls: bool,
    /// New TLS SNI.
    pub tls_sni: Option<String>,
    /// New mTLS client certificate path.
    pub tls_client_cert: Option<PathBuf>,
    /// New mTLS client key path.
    pub tls_client_key: Option<PathBuf>,
    /// Tags to append (deduplicated, order preserved).
    pub tags: Vec<String>,
    /// Clear the tag list before appending.
    pub clear_tags: bool,
    /// Remove the key path and its passphrase.
    pub no_key: bool,
    /// Remove the explicit agent socket.
    pub no_agent_socket: bool,
    /// Remove the sudo password.
    pub clear_sudo_password: bool,
    /// Remove the su password.
    pub clear_su_password: bool,
    /// Remove the TLS SNI override.
    pub no_tls_sni: bool,
    /// Remove the mTLS client certificate path.
    pub no_tls_client_cert: bool,
    /// Remove the mTLS client key path.
    pub no_tls_client_key: bool,
}

impl EditRequest {
    /// True when the invocation carries no field, rename, tag or clear flag.
    ///
    /// `--dry-run` is not counted: a preview on its own is a declared operation
    /// that reports the current state, while a real run with nothing to change
    /// would write the file and then claim an edit that never happened.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.new_name.is_none()
            && self.host.is_none()
            && self.port.is_none()
            && self.user.is_none()
            && self.password.is_none()
            && !self.password_stdin
            && self.key.is_none()
            && self.key_passphrase.is_none()
            && !self.key_passphrase_stdin
            && !self.use_agent
            && self.agent_socket.is_none()
            && self.timeout.is_none()
            && self.max_command_chars.is_none()
            && self.max_output_chars.is_none()
            && self.sudo_password.is_none()
            && !self.sudo_password_stdin
            && self.su_password.is_none()
            && !self.su_password_stdin
            && !self.disable_sudo
            && !self.enable_sudo
            && !self.tls
            && !self.no_tls
            && self.tls_sni.is_none()
            && self.tls_client_cert.is_none()
            && self.tls_client_key.is_none()
            && self.tags.is_empty()
            && !self.clear_tags
            && !self.no_key
            && !self.no_agent_socket
            && !self.clear_sudo_password
            && !self.clear_su_password
            && !self.no_tls_sni
            && !self.no_tls_client_cert
            && !self.no_tls_client_key
    }
}

/// Applies `request` to `record`, returning the field names that really changed.
///
/// The returned list feeds the `updated` array of the `vps-edited` envelope, so
/// an agent can tell an applied field from one it merely mentioned. A field
/// appears at most once, in the order it was applied, and only when its new value
/// differs from the stored one.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] for an unparsable host, port, user, timeout
/// or limit, a missing key file, or a clear that would leave the host with no
/// authentication method.
pub fn apply_fields(
    record: &mut VpsRecord,
    request: &EditRequest,
    secrets: &StdinSecrets,
) -> Result<Vec<&'static str>, SshCliError> {
    let mut updated: Vec<&'static str> = Vec::new();
    // Tags are compared as a whole at the end: `--clear-tags --tag a` on a host
    // that already carries exactly `["a"]` changes nothing.
    let tags_before = record.tags.clone();

    // Clear flags first: `--clear-tags --tag a` must end with `["a"]`.
    apply_clears(record, request, &mut updated);
    apply_identity(record, request, &mut updated)?;
    apply_auth(record, request, secrets, &mut updated)?;
    apply_agent_socket(record, request, &mut updated)?;
    apply_limits(record, request, &mut updated)?;
    apply_elevation(record, request, secrets, &mut updated);
    apply_tls(record, request, &mut updated);
    apply_tags(record, request, &tags_before, &mut updated)?;

    Ok(updated)
}

/// Adds `field` to `updated` when `changed` and it is not listed yet.
pub(super) fn mark_if_changed(updated: &mut Vec<&'static str>, field: &'static str, changed: bool) {
    if changed && !updated.contains(&field) {
        updated.push(field);
    }
}

/// Wraps a domain parse failure as the exit-64 argument error.
pub(super) fn invalid<E: std::fmt::Display>(e: E) -> SshCliError {
    SshCliError::InvalidArgument(e.to_string())
}

/// Stdin wins over argv for the same secret, matching the pre-existing order.
pub(super) fn pick_secret(
    from_stdin: &Option<SecretString>,
    from_argv: &Option<String>,
) -> Option<SecretString> {
    match (from_stdin, from_argv) {
        (Some(secret), _) => Some(secret.clone()),
        (None, Some(plain)) => Some(SecretString::from(plain.clone())),
        (None, None) => None,
    }
}

/// True when `stored` is absent or holds a different secret than `new`.
pub(super) fn secret_differs(stored: Option<&SecretString>, new: &SecretString) -> bool {
    !matches!(stored, Some(old) if old.expose_secret() == new.expose_secret())
}

/// Runs the clear flags; every one reports only when it removed something.
fn apply_clears(record: &mut VpsRecord, request: &EditRequest, updated: &mut Vec<&'static str>) {
    if request.clear_tags {
        // Reported by `apply_tags`, which compares against the original list.
        record.tags.clear();
    }
    if request.clear_sudo_password {
        mark_if_changed(updated, "sudo_password", record.sudo_password.is_some());
        record.sudo_password = None;
    }
    if request.clear_su_password {
        mark_if_changed(updated, "su_password", record.su_password.is_some());
        record.su_password = None;
    }
    if request.no_tls_sni {
        mark_if_changed(updated, "tls_sni", record.tls_sni.is_some());
        record.tls_sni = None;
    }
    if request.no_tls_client_cert {
        mark_if_changed(updated, "tls_client_cert", record.tls_client_cert.is_some());
        record.tls_client_cert = None;
    }
    if request.no_tls_client_key {
        mark_if_changed(updated, "tls_client_key", record.tls_client_key.is_some());
        record.tls_client_key = None;
    }
    if request.no_key {
        let had_key = record.key_path.is_some() || record.key_passphrase.is_some();
        mark_if_changed(updated, "key_path", had_key);
        record.key_path = None;
        record.key_passphrase = None;
    }
}

/// Applies `--host`, `--port` and `--user`.
fn apply_identity(
    record: &mut VpsRecord,
    request: &EditRequest,
    updated: &mut Vec<&'static str>,
) -> Result<(), SshCliError> {
    if let Some(h) = &request.host {
        let host = SshHost::try_new(h).map_err(invalid)?;
        mark_if_changed(updated, "host", record.host != host);
        record.host = host;
    }
    if let Some(p) = request.port {
        let port = SshPort::try_new(p).map_err(invalid)?;
        mark_if_changed(updated, "port", record.port != port);
        record.port = port;
    }
    if let Some(u) = &request.user {
        let user = SshUser::try_new(u).map_err(invalid)?;
        mark_if_changed(updated, "user", record.username != user);
        record.username = user;
    }
    Ok(())
}

/// Applies `--agent-socket` and `--no-agent-socket`.
fn apply_agent_socket(
    record: &mut VpsRecord,
    request: &EditRequest,
    updated: &mut Vec<&'static str>,
) -> Result<(), SshCliError> {
    if let Some(sock) = &request.agent_socket {
        let socket = sock.to_string_lossy().into_owned();
        let differs = record.agent_socket.as_deref() != Some(socket.as_str());
        mark_if_changed(updated, "agent_socket", differs);
        record.agent_socket = Some(socket);
    }
    if request.no_agent_socket {
        let had_socket = record.agent_socket.is_some();
        if had_socket
            && record.use_agent
            && !crate::ssh::connection::ConnectionConfig::platform_provides_default_agent_endpoint()
        {
            return Err(SshCliError::InvalidArgument(
                "--no-agent-socket would leave this host without an agent endpoint on this \
                 platform; pass --agent-socket PATH, switch to --key or --password, or drop \
                 --no-agent-socket"
                    .to_string(),
            ));
        }
        mark_if_changed(updated, "agent_socket", had_socket);
        record.agent_socket = None;
    }
    Ok(())
}

/// Applies `--timeout`, `--max-command-chars` and `--max-output-chars`.
fn apply_limits(
    record: &mut VpsRecord,
    request: &EditRequest,
    updated: &mut Vec<&'static str>,
) -> Result<(), SshCliError> {
    if let Some(t) = request.timeout {
        let timeout = TimeoutMs::try_new(t).map_err(invalid)?;
        mark_if_changed(updated, "timeout_ms", record.timeout_ms != timeout);
        record.timeout_ms = timeout;
    }
    if let Some(m) = request.max_command_chars {
        let limit = CharLimit::try_new(m).map_err(invalid)?;
        mark_if_changed(
            updated,
            "max_command_chars",
            record.max_command_chars != limit,
        );
        record.max_command_chars = limit;
    }
    if let Some(m) = request.max_output_chars {
        let limit = CharLimit::try_new(m).map_err(invalid)?;
        mark_if_changed(
            updated,
            "max_output_chars",
            record.max_output_chars != limit,
        );
        record.max_output_chars = limit;
    }
    Ok(())
}

/// Applies the sudo/su passwords and the sudo switch.
fn apply_elevation(
    record: &mut VpsRecord,
    request: &EditRequest,
    secrets: &StdinSecrets,
    updated: &mut Vec<&'static str>,
) {
    if let Some(sudo) = pick_secret(&secrets.sudo_password, &request.sudo_password) {
        let differs = secret_differs(record.sudo_password.as_ref(), &sudo);
        mark_if_changed(updated, "sudo_password", differs);
        record.sudo_password = Some(sudo);
    }
    if let Some(su) = pick_secret(&secrets.su_password, &request.su_password) {
        let differs = secret_differs(record.su_password.as_ref(), &su);
        mark_if_changed(updated, "su_password", differs);
        record.su_password = Some(su);
    }
    let disable = match (request.disable_sudo, request.enable_sudo) {
        (true, _) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
    };
    if let Some(disable) = disable {
        mark_if_changed(updated, "disable_sudo", record.disable_sudo != disable);
        record.disable_sudo = disable;
    }
}

/// Applies the TLS switch, SNI and the mTLS pair.
fn apply_tls(record: &mut VpsRecord, request: &EditRequest, updated: &mut Vec<&'static str>) {
    let tls = match (request.tls, request.no_tls) {
        (true, _) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
    };
    if let Some(tls) = tls {
        mark_if_changed(updated, "tls", record.tls != tls);
        record.tls = tls;
    }
    if let Some(sni) = &request.tls_sni {
        mark_if_changed(
            updated,
            "tls_sni",
            record.tls_sni.as_deref() != Some(sni.as_str()),
        );
        record.tls_sni = Some(sni.clone());
    }
    if let Some(cert) = &request.tls_client_cert {
        let cert = cert.to_string_lossy().into_owned();
        let differs = record.tls_client_cert.as_deref() != Some(cert.as_str());
        mark_if_changed(updated, "tls_client_cert", differs);
        record.tls_client_cert = Some(cert);
    }
    if let Some(key) = &request.tls_client_key {
        let key = key.to_string_lossy().into_owned();
        let differs = record.tls_client_key.as_deref() != Some(key.as_str());
        mark_if_changed(updated, "tls_client_key", differs);
        record.tls_client_key = Some(key);
    }
}

/// Appends `--tag` values and reports `tags` once when the list differs from
/// `before` (the list as it was prior to any clear flag).
fn apply_tags(
    record: &mut VpsRecord,
    request: &EditRequest,
    before: &[HostTag],
    updated: &mut Vec<&'static str>,
) -> Result<(), SshCliError> {
    if !request.tags.is_empty() {
        let mut merged: Vec<String> = record.tags.iter().map(|t| t.as_str().to_owned()).collect();
        merged.extend(request.tags.iter().cloned());
        let merged = crate::vps::dedupe_host_names(merged);
        record
            .set_tags_from_raw(merged)
            .map_err(SshCliError::from)?;
    }
    mark_if_changed(updated, "tags", record.tags.as_slice() != before);
    Ok(())
}

/// Validates TLS options exactly as a connect would build them.
///
/// # Errors
/// [`SshCliError::Tls`] when SNI is empty or only half of the mTLS pair is set.
pub fn validate_tls(record: &VpsRecord) -> Result<(), SshCliError> {
    if !record.tls {
        return Ok(());
    }
    let sni = record
        .tls_sni
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(record.host.as_str());
    let _ = crate::tls::TlsConnectOptions::try_new(
        sni,
        record.tls_client_cert.as_ref().map(PathBuf::from),
        record.tls_client_key.as_ref().map(PathBuf::from),
    )?;
    Ok(())
}
