// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: secrets command dispatcher extracted from vps/mod (SRP; line budget).
#![forbid(unsafe_code)]
//! One-shot `secrets status|init|reencrypt` local crypto workload.

use super::config_io::{load, resolve_config_path, save};
use crate::cli::{OutputFormat, SecretsAction};
use crate::errors::SshCliError;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Dispatcher one-shot de `secrets status|init|reencrypt`.
/// Secrets primary-key status/init/reencrypt.
///
/// Workload: **local crypto + disk** (not multi-host SSH). Sequential justified:
/// single key file / single config rewrite; Mutex in secrets layer.
pub async fn run_secrets_command(
    action: SecretsAction,
    config_override: Option<PathBuf>,
    format: OutputFormat,
) -> Result<()> {
    // Keep secrets.key aligned with --config-dir (owned copy for process static).
    crate::secrets::set_config_dir(config_override.clone());
    match action {
        SecretsAction::Status { json } => {
            let seg = crate::secrets::secrets_status()?;
            let use_json = json || format == OutputFormat::Json;
            if use_json {
                let v = serde_json::json!({
                    "encryption_active": seg.encryption_active,
                    "key_source": seg.source.as_str(),
                    "key_file": seg.key_file_path.display().to_string(),
                    "plaintext_opt_out": seg.plaintext_opt_out,
                    "at_rest": if seg.encryption_active { "encrypted" } else { "plaintext" },
                });
                crate::output::print_json_value(&v)?;
            } else {
                let at_rest = if seg.encryption_active {
                    "encrypted"
                } else {
                    "plaintext"
                };
                crate::output::print_success_fmt(format_args!(
                    "at-rest: {at_rest} | source: {} | key_file: {} | plaintext_opt_out: {}",
                    seg.source.as_str(),
                    seg.key_file_path.display(),
                    seg.plaintext_opt_out
                ));
            }
            Ok(())
        }
        SecretsAction::Init {
            keyring,
            force,
            json,
        } => {
            // GAP-AUD-SEC-001: rotating the primary key without re-encrypting hosts
            // permanently loses at-rest secrets. Load/decrypt BEFORE overwriting the key,
            // then save under the new key after rotation.
            let path = resolve_config_path(config_override.as_deref())?;
            let hosts_to_reencrypt = if force && path.is_file() {
                Some(load(&path)?)
            } else {
                None
            };

            let seg = crate::secrets::init_primary_key(keyring, force)?;
            let mut reencrypted_hosts = 0usize;

            if let Some(file) = hosts_to_reencrypt {
                reencrypted_hosts = file.hosts.len();
                save(&path, &file).map_err(|e| {
                    SshCliError::Config(format!(
                        "primary key was rotated but re-encrypting config failed: {e}; \
                         restore secrets.key.bak if present and re-run `secrets reencrypt`"
                    ))
                })?;
            }

            let use_json = json || format == OutputFormat::Json;
            crate::output::emit_success(
                "secrets-init",
                serde_json::json!({
                    "key_source": seg.source.as_str(),
                    "key_file": seg.key_file_path.display().to_string(),
                    "reencrypted_hosts": reencrypted_hosts,
                    "force": force,
                }),
                &crate::i18n::t(crate::i18n::Message::PrimaryKeyReady {
                    source: seg.source.as_str().to_string(),
                    key_file: seg.key_file_path.display().to_string(),
                }),
                use_json,
            )?;
            Ok(())
        }
        SecretsAction::Reencrypt { json } => {
            let path = resolve_config_path(config_override.as_deref())?;
            run_reencrypt(&path, json || format == OutputFormat::Json)?;
            Ok(())
        }
    }
}

/// Reloads and rewrites config, re-encrypting secrets with the current key.
/// Re-encrypt all secrets in `config.toml` with the current primary key.
///
/// Workload: **local AEAD** over one file. Sequential justified: single atomic
/// save; host count is small vs coordination overhead.
fn run_reencrypt(path: &Path, json: bool) -> Result<()> {
    let (key, _source) = crate::secrets::ensure_key_for_write()?;
    if key.is_none() {
        return Err(SshCliError::InvalidArgument(
            "no primary-key; run `ssh-cli secrets init` or pass --allow-plaintext-secrets"
                .to_string(),
        )
        .into());
    }
    if let Some(mut k) = key {
        use zeroize::Zeroize;
        k.zeroize();
    }
    let file = load(path)?;
    let hosts = file.hosts.len();
    save(path, &file)?;
    crate::output::emit_success(
        "secrets-reencrypt",
        serde_json::json!({ "hosts": hosts }),
        &crate::i18n::t(crate::i18n::Message::ReencryptCompleted { hosts }),
        json,
    )?;
    Ok(())
}

/// Side-effect metadata when `secrets.key` was auto-created in this process.
///
/// G-E2E-04: callers **fold** this into the primary success event (one JSON
/// document per one-shot). Do **not** emit a second stdout root.
#[derive(Debug, Clone)]
pub(crate) struct AutoKeyMeta {
    /// Absolute path to the primary-key file.
    pub key_file: String,
    /// Always `xdg_file` for auto-create.
    pub key_source: &'static str,
}

/// Consumes the auto-key-created flag without writing to stdout (G-E2E-04).
#[must_use]
pub(crate) fn take_auto_key_meta() -> Option<AutoKeyMeta> {
    if crate::secrets::take_auto_key_created() {
        let path = crate::secrets::secrets_key_path().unwrap_or_default();
        Some(AutoKeyMeta {
            key_file: path.display().to_string(),
            key_source: "xdg_file",
        })
    } else {
        None
    }
}

