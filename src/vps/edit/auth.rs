// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Primary-authentication switching for `ssh-cli vps edit`.
//!
//! # One method at a time
//!
//! A record carries exactly one primary method: password, key or agent
//! ([`crate::vps::model::VpsRecord::validate_credentials`]). Each flag here moves
//! the record to its own method and clears the other two, so a switch is one
//! command instead of a clear flag plus a value flag:
//!
//! - `--key` clears the password, the agent flag and the agent socket.
//! - `--password` / `--password-stdin` clear the key, its passphrase, the agent
//!   flag and the agent socket.
//! - `--use-agent` clears the password, the key and its passphrase. An explicit
//!   `--agent-socket` implies it.
//! - `--key` with a path different from the stored one also drops the stored
//!   passphrase unless the same call brings a new one.
//!
//! # What `updated` means
//!
//! A cleared field is listed only when it held a value before, so a flag that
//! clears nothing never inflates the list.
//!
//! # Orphan passphrase
//!
//! A key passphrase without a key would be a secret nothing reads. When the
//! request carries one and the record ends with no key, the edit fails with
//! [`SshCliError::InvalidArgument`] before anything is persisted: the edit works
//! on an in-memory record and the registry is written only after every check.

use super::apply::{
    invalid, mark_if_changed, pick_secret, secret_differs, EditRequest, StdinSecrets,
};
use crate::domain::KeyPath;
use crate::errors::SshCliError;
use crate::vps::model::VpsRecord;
use secrecy::{ExposeSecret, SecretString};

/// Applies the primary authentication flags (agent, password, key, passphrase).
///
/// # Errors
/// [`SshCliError::InvalidArgument`] for a missing key file, an unparsable key
/// path, or a key passphrase that would be left without a key.
///
/// # Examples
/// ```text
/// // `vps edit prod --key ~/.ssh/id_ed25519` on a password host lists both the
/// // new key and the password it replaced:
/// // updated == ["key_path", "password"]
/// ```
pub(super) fn apply_auth(
    record: &mut VpsRecord,
    request: &EditRequest,
    secrets: &StdinSecrets,
    updated: &mut Vec<&'static str>,
) -> Result<(), SshCliError> {
    // An explicit socket only makes sense for agent auth, so `--agent-socket`
    // alone switches the host to the agent exactly as `--use-agent` does.
    if request.use_agent || request.agent_socket.is_some() {
        apply_use_agent(record, updated);
    } else {
        apply_password(record, request, secrets, updated);
        apply_key(record, request, secrets, updated)?;
    }
    apply_passphrase(record, request, secrets, updated)
}

/// `--use-agent` moves the record to the agent and drops password and key.
fn apply_use_agent(record: &mut VpsRecord, updated: &mut Vec<&'static str>) {
    mark_if_changed(updated, "use_agent", !record.use_agent);
    record.use_agent = true;
    clear_password(record, updated);
    clear_key(record, updated);
}

/// `--password` / `--password-stdin` move the record to password auth.
fn apply_password(
    record: &mut VpsRecord,
    request: &EditRequest,
    secrets: &StdinSecrets,
    updated: &mut Vec<&'static str>,
) {
    let Some(password) = pick_secret(&secrets.password, &request.password) else {
        return;
    };
    let differs = password.expose_secret() != record.password.expose_secret();
    mark_if_changed(updated, "password", differs);
    record.password = password;
    clear_key(record, updated);
    clear_agent(record, updated);
}

/// `--key` moves the record to key auth.
fn apply_key(
    record: &mut VpsRecord,
    request: &EditRequest,
    secrets: &StdinSecrets,
    updated: &mut Vec<&'static str>,
) -> Result<(), SshCliError> {
    let Some(path) = &request.key else {
        return Ok(());
    };
    let path = path.to_string_lossy().into_owned();
    crate::vps::validate_key_path_exists(&path)?;
    let key = KeyPath::try_new(path).map_err(invalid)?;
    let new_key = record.key_path.as_ref() != Some(&key);
    mark_if_changed(updated, "key_path", new_key);
    record.key_path = Some(key);
    // A passphrase unlocks one key: a different key without a new passphrase in
    // the same call must not inherit the old one.
    if new_key && secrets.key_passphrase.is_none() && request.key_passphrase.is_none() {
        mark_if_changed(updated, "key_passphrase", record.key_passphrase.is_some());
        record.key_passphrase = None;
    }
    clear_password(record, updated);
    clear_agent(record, updated);
    Ok(())
}

/// Stores the key passphrase, refusing one that would have no key to unlock.
fn apply_passphrase(
    record: &mut VpsRecord,
    request: &EditRequest,
    secrets: &StdinSecrets,
    updated: &mut Vec<&'static str>,
) -> Result<(), SshCliError> {
    let Some(passphrase) = pick_secret(&secrets.key_passphrase, &request.key_passphrase) else {
        return Ok(());
    };
    if record.key_path.is_none() {
        return Err(SshCliError::InvalidArgument(
            "a key passphrase needs a private key: this host has no key; pass --key PATH \
             together with the passphrase"
                .to_string(),
        ));
    }
    let differs = secret_differs(record.key_passphrase.as_ref(), &passphrase);
    mark_if_changed(updated, "key_passphrase", differs);
    record.key_passphrase = Some(passphrase);
    Ok(())
}

/// Drops the password, reporting it only when one was stored.
fn clear_password(record: &mut VpsRecord, updated: &mut Vec<&'static str>) {
    mark_if_changed(updated, "password", record.has_password());
    record.password = SecretString::from(String::new());
}

/// Drops the key path and its passphrase, reporting each one that was stored.
fn clear_key(record: &mut VpsRecord, updated: &mut Vec<&'static str>) {
    mark_if_changed(updated, "key_path", record.key_path.is_some());
    mark_if_changed(updated, "key_passphrase", record.key_passphrase.is_some());
    record.key_path = None;
    record.key_passphrase = None;
}

/// Leaves agent mode and drops the explicit socket, reporting what was set.
fn clear_agent(record: &mut VpsRecord, updated: &mut Vec<&'static str>) {
    mark_if_changed(updated, "use_agent", record.use_agent);
    mark_if_changed(updated, "agent_socket", record.agent_socket.is_some());
    record.use_agent = false;
    record.agent_socket = None;
}
