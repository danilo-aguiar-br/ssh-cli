// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! `ssh-cli vps rename` and `ssh-cli vps edit` (single locked write path).
//!
//! # Why rename lives here and not in the record model
//!
//! Renaming is not a property of a record; it is a registry operation. The map
//! key, the `name` field, the `active` marker, the duplicate check and the lock
//! all have to move together, and none of them belong to a value type.
//!
//! # Why a rename needs no extra cryptographic step
//!
//! [`crate::vps::model::VpsRecord`] serializes secrets through the **unbound**
//! `sshcli-enc:v2:` variants, so the AEAD associated data carries no host name.
//! Opening the blob therefore never needs the name it was stored under, and the
//! credential stays readable after the key moves. The token itself does change,
//! because `save` seals every secret again with a fresh nonce; that is ordinary
//! AEAD behavior and the plaintext is untouched throughout. If a future version
//! binds the ciphertext to the host, the blob would have to be re-sealed here —
//! and `tests/gaps_v071_vps_edit_rename.rs` asserts readability under the new
//! name, which is the test that fails first in that case.
//!
//! # Explicit Target Designation
//!
//! Both verbs take the host from argv and never consult the `active` marker.
//! The target is published to the error-envelope seam only **after** the lookup
//! succeeds, so `vps_not_found` keeps reporting no target at all.
//!
//! # Error precedence and no-op edits
//!
//! The order is: empty request (64), malformed new name (64, before the lock),
//! unknown host (66), identical new name (64), taken new name (64). The lookup
//! comes before the name comparison so `vps rename ghost ghost` is "not found".
//! An edit that changes no field and keeps the name exits 0 with `updated: []`
//! and never writes `config.toml`.
//!
//! # The `active` marker
//!
//! When a rename moves the marker it is written before the config, and restored
//! if the save fails (`commit_rename` in the private `commit` module).

mod apply;
#[cfg(test)]
mod apply_tests;
mod auth;
mod commit;

pub use apply::{apply_fields, validate_tls, EditRequest, StdinSecrets};

use crate::cli::OutputFormat;
use crate::domain::VpsName;
use crate::errors::{SshCliError, SshCliResult};
use crate::json_wire::{ExecTarget, TargetEcho, TargetSource};
use crate::vps::active::{read_active_at, write_active_vps};
use crate::vps::config_io::{load, lock_config, ConfigFile, ConfigGuard};
use crate::vps::read_secret_stdin;
use anyhow::Result;
use commit::commit_rename;
use secrecy::SecretString;
use std::path::Path;

/// Log target shared by every event of the edit/rename path.
const LOG_TARGET: &str = "ssh_cli::vps::edit";

/// What one validated edit did, as the emitters and the commit need it.
struct EditOutcome {
    /// Name before the edit.
    previous_name: String,
    /// Name after the edit (equal to `previous_name` unless renamed).
    final_name: String,
    /// Fields whose value really changed, in application order.
    updated: Vec<&'static str>,
    /// True when the record moved to a new key.
    renaming: bool,
    /// True when the `active` marker pointed at the old name and must follow.
    marker_rewrite: bool,
    /// Audit echo of the explicit target.
    echo: TargetEcho,
}

impl EditOutcome {
    /// An edit that changed no field and kept the name has nothing to write.
    fn is_noop(&self) -> bool {
        self.updated.is_empty() && !self.renaming
    }
}

/// Runs `vps rename <NAME> <NEW_NAME>`.
///
/// # Errors
/// Exit 66 for an unknown source (checked before the name comparison, so
/// `ghost ghost` is "not found"); exit 64 for an invalid or identical name or a
/// taken name; I/O and lock errors as reported by the config layer.
pub fn run_rename(name: &str, new_name: &str, path: &Path, format: OutputFormat) -> Result<()> {
    let request = EditRequest {
        name: name.to_string(),
        new_name: Some(new_name.to_string()),
        ..EditRequest::default()
    };
    run_edit(request, path, format)
}

/// Runs one validated edit under a single config lock.
///
/// An edit that changes nothing exits 0 with `updated: []` and does not write
/// `config.toml`, so a retry is idempotent and costs no fsync.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] for an empty edit or an invalid name,
/// [`SshCliError::VpsNotFound`] (66) for an unknown host,
/// [`SshCliError::VpsDuplicate`] (64) for a taken name, plus storage failures.
pub fn run_edit(request: EditRequest, path: &Path, format: OutputFormat) -> Result<()> {
    reject_empty(&request)?;
    require_registry(path, &request.name)?;
    // Validate the new name before the lock: a malformed name cannot be fixed by
    // any state on disk, so there is no reason to wait for a peer.
    let new_name = parse_new_name(&request)?;
    // Stdin drains once: read every secret before the lock covers the file.
    let secrets = read_stdin_secrets(&request)?;

    let guard = lock_config(path)?;
    let mut file = load(path)?;
    let outcome = apply_edit(&mut file, &request, new_name, &secrets, path)?;

    let operation = if outcome.renaming {
        "vps-rename"
    } else {
        "vps-edit"
    };
    // The plan echoes the same four audit fields as the real events, so a caller
    // reading a preview learns which host argv designated.
    let echo = echo_object(&outcome.echo);
    let mut plan: Vec<(&str, serde_json::Value)> = vec![
        ("name", serde_json::json!(outcome.final_name)),
        ("previous_name", serde_json::json!(outcome.previous_name)),
        ("config_path", serde_json::json!(path.display().to_string())),
        ("updated", serde_json::json!(outcome.updated)),
        (
            "active_rewritten",
            serde_json::json!(outcome.marker_rewrite),
        ),
    ];
    plan.extend(echo.iter().map(|(k, v)| (k.as_str(), v.clone())));
    if crate::cli::dry_run_stop(operation, &plan)? {
        return Ok(());
    }

    if outcome.is_noop() {
        tracing::debug!(
            target: LOG_TARGET,
            name = %outcome.final_name,
            "edit changed no field; config file left untouched"
        );
    } else {
        persist(&guard, path, &file, &outcome)?;
    }
    drop(guard);
    emit_outcome(outcome, format)
}

/// Fails with exit 66 when there is no registry file to look the host up in.
///
/// Locking would create the config directory and the lock file only to find an
/// empty registry, so a verb that needs an existing host checks the file first
/// and leaves the disk alone.
///
/// # Errors
/// [`SshCliError::VpsNotFound`] when `path` does not exist.
pub(crate) fn require_registry(path: &Path, name: &str) -> SshCliResult<()> {
    if path.exists() {
        return Ok(());
    }
    Err(SshCliError::VpsNotFound(name.to_owned()))
}

/// An invocation with no flag at all is a usage error (exit 64).
fn reject_empty(request: &EditRequest) -> SshCliResult<()> {
    if request.is_empty() {
        return Err(SshCliError::InvalidArgument(format!(
            "nothing to edit for '{}': pass at least one field, --name, --tag or a clear flag",
            request.name
        )));
    }
    Ok(())
}

/// Parses the syntax of `--name`. Comparison with the current name and the
/// duplicate check need the registry and run after the lookup.
fn parse_new_name(request: &EditRequest) -> SshCliResult<Option<VpsName>> {
    request
        .new_name
        .as_deref()
        .map(|raw| {
            VpsName::try_new(raw)
                .map_err(|e| SshCliError::InvalidArgument(format!("invalid VPS name '{raw}': {e}")))
        })
        .transpose()
}

/// Locates the record, applies the request in memory and returns what changed.
///
/// Error precedence is fixed: unknown host (66), then identical name (64), then
/// taken name (64). The target is published right after the lookup, so every
/// later failure can name where it happened while `vps_not_found` names nothing.
fn apply_edit(
    file: &mut ConfigFile,
    request: &EditRequest,
    new_name: Option<VpsName>,
    secrets: &StdinSecrets,
    path: &Path,
) -> SshCliResult<EditOutcome> {
    let not_found = || SshCliError::VpsNotFound(request.name.clone());
    if !file.hosts.contains_key(&request.name) {
        return Err(not_found());
    }
    crate::json_wire::set_resolved_target(&ExecTarget::new(
        request.name.clone(),
        TargetSource::Argv,
    ));
    check_new_name(file, request, new_name.as_ref())?;

    let mut record = file.hosts.remove(&request.name).ok_or_else(not_found)?;
    let updated = apply_fields(&mut record, request, secrets)?;
    validate_tls(&record)?;
    record.validate().map_err(SshCliError::from)?;

    let previous_name = request.name.clone();
    let final_name = match &new_name {
        Some(validated) => validated.as_str().to_owned(),
        None => previous_name.clone(),
    };
    if let Some(validated) = new_name {
        record.name = validated;
    }
    let renaming = final_name != previous_name;
    let marker_rewrite =
        renaming && read_active_at(path)?.as_deref() == Some(previous_name.as_str());
    file.hosts.insert(final_name.clone(), record);

    Ok(EditOutcome {
        echo: TargetEcho::from_parts(previous_name.clone(), TargetSource::Argv),
        previous_name,
        final_name,
        updated,
        renaming,
        marker_rewrite,
    })
}

/// Rejects a rename onto the current name or onto another registered host.
fn check_new_name(
    file: &ConfigFile,
    request: &EditRequest,
    new_name: Option<&VpsName>,
) -> SshCliResult<()> {
    let Some(validated) = new_name else {
        return Ok(());
    };
    if validated.as_str() == request.name {
        return Err(SshCliError::InvalidArgument(format!(
            "new name '{}' is identical to the current name; a rename must change it",
            validated.as_str()
        )));
    }
    if file.hosts.contains_key(validated.as_str()) {
        return Err(SshCliError::VpsDuplicate(validated.as_str().to_owned()));
    }
    Ok(())
}

/// Writes the registry, moving the `active` marker first when it must follow.
fn persist(
    guard: &ConfigGuard,
    path: &Path,
    file: &ConfigFile,
    outcome: &EditOutcome,
) -> SshCliResult<()> {
    commit_rename(
        outcome.marker_rewrite,
        || guard.save(path, file),
        || write_active_vps(path, &outcome.final_name),
        || write_active_vps(path, &outcome.previous_name),
    )
}

/// Emits `vps-renamed` or `vps-edited`; `updated` is always present, even empty.
fn emit_outcome(outcome: EditOutcome, format: OutputFormat) -> Result<()> {
    // One event name for one operation. `vps edit --name` reaches this same
    // path, so it emits `vps-renamed` too: `previous_name` is present exactly
    // when a rename happened, and `docs/schemas/vps-edit.schema.json` documents
    // it as optional for that reason. Two names for one outcome would have made
    // the caller branch on a string to learn what it already asked for.
    let mut fields = serde_json::json!({
        "name": outcome.final_name,
        "updated": outcome.updated,
    });
    merge_echo(&mut fields, &outcome.echo);
    let json = format == OutputFormat::Json;
    if !outcome.renaming {
        crate::output::emit_success_fmt(
            "vps-edited",
            fields,
            format_args!(
                "{}",
                crate::i18n::t(crate::i18n::Message::VpsEdited {
                    name: outcome.final_name,
                })
            ),
            json,
        )?;
        return Ok(());
    }
    if let Some(map) = fields.as_object_mut() {
        map.insert(
            "previous_name".to_string(),
            serde_json::json!(outcome.previous_name),
        );
        map.insert(
            "active_rewritten".to_string(),
            serde_json::json!(outcome.marker_rewrite),
        );
    }
    // The human line formats `format_args!` directly: `format` is already
    // known to be non-JSON here, so allocating the translated String first
    // would only keep a buffer alive for nothing.
    crate::output::emit_success_fmt(
        "vps-renamed",
        fields,
        format_args!(
            "{}",
            crate::i18n::t(crate::i18n::Message::VpsRenamed {
                from: outcome.previous_name,
                to: outcome.final_name,
            })
        ),
        json,
    )?;
    Ok(())
}

/// Serializes the target echo into its four audit fields.
///
/// `emit_success` takes a `serde_json::Value`, and the echo is the single
/// definition of the four audit fields; rebuilding them here by hand would make
/// this module a second source of truth for a contract defined in
/// [`crate::json_wire::exec_target`]. The fields are serialized rather than
/// hand-copied for exactly that reason.
pub(crate) fn echo_object(echo: &TargetEcho) -> serde_json::Map<String, serde_json::Value> {
    match serde_json::to_value(echo) {
        Ok(serde_json::Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    }
}

/// Flattens the target echo into an event object.
fn merge_echo(fields: &mut serde_json::Value, echo: &TargetEcho) {
    let Some(map) = fields.as_object_mut() else {
        return;
    };
    map.extend(echo_object(echo));
}

/// Reads at most one secret from stdin, refusing two reads in one invocation.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] when more than one `--*-stdin` flag is set,
/// or when `--no-input` forbids the read (see [`read_secret_stdin`]).
fn read_stdin_secrets(request: &EditRequest) -> SshCliResult<StdinSecrets> {
    let flags = usize::from(request.password_stdin)
        + usize::from(request.key_passphrase_stdin)
        + usize::from(request.sudo_password_stdin)
        + usize::from(request.su_password_stdin);
    if flags > 1 {
        return Err(SshCliError::InvalidArgument(
            "only one --*-stdin per one-shot invocation (stdin is drained once); \
             run vps edit again for the remaining secrets"
                .to_string(),
        ));
    }
    Ok(StdinSecrets {
        password: read_optional(request.password_stdin)?,
        key_passphrase: read_optional(request.key_passphrase_stdin)?,
        sudo_password: read_optional(request.sudo_password_stdin)?,
        su_password: read_optional(request.su_password_stdin)?,
    })
}

fn read_optional(flag: bool) -> SshCliResult<Option<SecretString>> {
    if flag {
        read_secret_stdin().map(Some)
    } else {
        Ok(None)
    }
}
