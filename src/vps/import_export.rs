// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP-03: import/export extracted from `vps/mod` (SRP).
#![forbid(unsafe_code)]
//! VPS inventory import/export (TOML + agent JSON envelope).
//!
//! Workload: **local disk + optional secret materialization**. Sequential justified:
//! single atomic write path; concurrent import would race flock/rename.

use super::config_io::lock_config;
use super::model;
use super::{load, validate_key_path_exists, write_atomic, ConfigFile};
use crate::cli::OutputFormat;
use crate::errors::{SshCliError, SshCliResult};
use anyhow::Result;
use secrecy::SecretString;
use std::collections::BTreeMap;
use std::path::Path;

/// Export hosts to TOML/JSON.
pub(super) fn run_export(
    path: &Path,
    include_secrets: bool,
    output: Option<&Path>,
    json: bool,
    i_understand_secrets_on_stdout: bool,
    format: OutputFormat,
) -> Result<()> {
    // GAP-AUD-011: refuse plaintext secrets on non-file stdout without explicit ack.
    if include_secrets && output.is_none() && !i_understand_secrets_on_stdout {
        let stdout_is_tty = std::io::IsTerminal::is_terminal(&std::io::stdout());
        if !stdout_is_tty {
            return Err(SshCliError::InvalidArgument(
                "refusing --include-secrets to a pipe/non-TTY stdout; \
                 use `--output <file>` (mode 0o600) or pass `--i-understand-secrets-on-stdout`"
                    .into(),
            )
            .into());
        }
    }

    let file = load(path)?;
    let mut export = ConfigFile {
        schema_version: model::CURRENT_SCHEMA_VERSION,
        hosts: BTreeMap::new(),
    };
    for (k, mut v) in file.hosts {
        if !include_secrets {
            // EXP-001 parity: redacted clears secrets (never sshcli-enc of empty).
            v.password = SecretString::from(String::new());
            v.sudo_password = None;
            v.su_password = None;
            v.key_passphrase = None;
        }
        v.schema_version = model::CURRENT_SCHEMA_VERSION;
        export.hosts.insert(k, v);
    }

    // G-AUD-03: JSON body when local `--json` OR global format is Json.
    // Agent wire: compact single-root JSON (Rules Rust JSON — no pretty-print).
    let wants_json = json || format == OutputFormat::Json;
    let bytes = if wants_json {
        let envelope = crate::output::export_envelope_json(
            &export.hosts,
            export.schema_version,
            include_secrets,
        );
        let text = serde_json::to_string(&envelope)?;
        text.into_bytes()
    } else {
        let text = toml::to_string_pretty(&export)?;
        text.into_bytes()
    };

    if let Some(out_path) = output {
        write_atomic(out_path, &bytes)?;
        let path_display = out_path.display().to_string();
        crate::output::emit_success(
            "vps-export",
            serde_json::json!({
                "path": path_display,
                "include_secrets": include_secrets,
                "format": if wants_json { "json" } else { "toml" },
            }),
            &crate::i18n::t(crate::i18n::Message::ExportCompleted { path: path_display }),
            format == OutputFormat::Json,
        )?;
    } else {
        // TOML/JSON body to stdout (agent-first: single payload).
        use std::io::Write;
        let mut out = std::io::stdout().lock();
        out.write_all(&bytes)?;
        if !bytes.ends_with(b"\n") {
            out.write_all(b"\n")?;
        }
    }
    Ok(())
}

/// Parses import source: TOML wire or JSON `vps-export` envelope / hosts map.
/// Parse import payload (TOML or JSON envelope). Public for fuzz targets (G-SERDE-12).
///
/// # Errors
/// Returns an error when the payload is neither valid TOML nor a valid JSON envelope or
/// hosts map.
pub fn parse_import_payload(text: &str) -> SshCliResult<ConfigFile> {
    // Rules JSON: strip UTF-8 BOM before format detection / parse.
    let text = crate::json_wire::strip_utf8_bom(text);
    let trimmed = text.trim_start();
    if trimmed.starts_with('{') {
        parse_import_json(trimmed)
    } else {
        let file: ConfigFile = crate::validation::from_toml_str(text)?;
        reject_invalid_names(file.hosts.keys())?;
        Ok(file)
    }
}

/// Fails once, naming every host key that is not a valid new VPS name.
///
/// An import is all or nothing, so the operator gets the whole list in one
/// message instead of fixing one name per run. Names are escaped: they come
/// from a file and may carry terminal control sequences.
fn reject_invalid_names<'a>(keys: impl Iterator<Item = &'a String>) -> SshCliResult<()> {
    let bad: Vec<String> = keys
        .filter(|k| crate::domain::VpsName::try_new(k).is_err())
        .map(|k| format!("'{}'", crate::output::sanitize::escape_untrusted(k)))
        .collect();
    if bad.is_empty() {
        return Ok(());
    }
    Err(SshCliError::InvalidArgument(format!(
        "invalid VPS names in import ({}): {}; a name is 1..={} ASCII letters, digits, '.', '_' \
         or '-' and starts with a letter or digit; nothing was imported",
        bad.len(),
        bad.join(", "),
        crate::validation::MAX_FIELD_LEN
    )))
}

fn parse_import_json(text: &str) -> SshCliResult<ConfigFile> {
    // G-SERDE-08/14: path errors + warn on unknown fields (Must-Ignore).
    let envelope: crate::json_wire::ImportEnvelope =
        crate::validation::from_json_str_warn_unused(text)?;
    let defaults = crate::json_wire::ImportDefaults {
        timeout_ms: model::DEFAULT_TIMEOUT_MS,
        max_command_chars: model::DEFAULT_MAX_COMMAND_CHARS,
        max_output_chars: model::DEFAULT_MAX_OUTPUT_CHARS,
        schema_version: model::CURRENT_SCHEMA_VERSION,
    };
    reject_invalid_names(envelope.hosts.keys())?;
    let mut hosts = BTreeMap::new();
    for (key, entry) in envelope.hosts {
        let rec = entry.into_record(&key, defaults).map_err(|e| {
            SshCliError::InvalidArgument(format!("invalid host '{key}' in import: {e}"))
        })?;
        hosts.insert(key, rec);
    }
    Ok(ConfigFile {
        schema_version: envelope
            .schema_version
            .unwrap_or(model::CURRENT_SCHEMA_VERSION),
        hosts,
    })
}

/// Import hosts from TOML/JSON file.
pub(super) fn run_import(
    path: &Path,
    file: &Path,
    allow_incomplete: bool,
    format: OutputFormat,
) -> Result<()> {
    // Cap import file size (same ceiling as config.toml — OOM hygiene).
    // A missing file is permanent (exit 66), not a retryable I/O error: retrying the
    // same path cannot make it appear.
    let text =
        crate::paths::read_text_capped(file, crate::paths::MAX_CONFIG_TOML_BYTES).map_err(|e| {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    SshCliError::FileNotFound(file.display().to_string())
                }
                _ => SshCliError::Io(e),
            }
        })?;
    let imported = parse_import_payload(&text)?;
    // The lock spans load → merge → save. Loading first and saving later loses
    // an update whenever a peer commits between the two: the import wrote a map
    // it had read before the peer's host existed, and the peer's host vanished.
    let guard = lock_config(path)?;
    let mut current = load(path)?;
    let mut imported_count = 0usize;
    // C2: an import silently replaces same-named hosts, so the plan names them.
    // Reporting only a count would hide the one fact that decides whether the
    // import is safe to run.
    let mut planned: Vec<serde_json::Value> = Vec::new();
    // Names that already existed and were overwritten; the success envelope
    // reports them as `replaced` so a real run says what the dry-run planned.
    let mut replaced_names: Vec<String> = Vec::new();
    for (k, mut v) in imported.hosts {
        // VAL-001 on import (domain VpsName = path-safe NFC name).
        let name = crate::domain::VpsName::try_new(&k).map_err(|e| {
            SshCliError::InvalidArgument(format!("invalid VPS name in import '{k}': {e}"))
        })?;
        v.name = name.clone();
        v.normalize_schema();
        if let Some(ref key) = v.key_path {
            validate_key_path_exists(&key.to_string_lossy_owned())?;
        }
        // Same TLS rules `vps edit` enforces: an import must not smuggle in an
        // mTLS certificate without its key, which no connect could ever use.
        // `--allow-incomplete` covers missing auth only, never a broken TLS pair.
        super::edit::validate_tls(&v).map_err(|err| {
            SshCliError::InvalidArgument(format!(
                "host '{name}' has invalid TLS options in import: {err}"
            ))
        })?;
        match v.validate() {
            Ok(()) => {
                let replaced = current.hosts.insert(name.as_str().to_owned(), v).is_some();
                if replaced {
                    replaced_names.push(name.as_str().to_owned());
                }
                planned.push(serde_json::json!({
                    "name": name.as_str(),
                    "replaces_existing": replaced,
                    "incomplete": false,
                }));
                imported_count += 1;
            }
            Err(ref err) if allow_incomplete => {
                // GAP-SSH-IMP-001: incomplete skeleton allowed.
                tracing::warn!(host = %name, error = %err, "import incomplete allowed");
                let replaced = current.hosts.insert(name.as_str().to_owned(), v).is_some();
                if replaced {
                    replaced_names.push(name.as_str().to_owned());
                }
                planned.push(serde_json::json!({
                    "name": name.as_str(),
                    "replaces_existing": replaced,
                    "incomplete": true,
                }));
                imported_count += 1;
            }
            Err(err) => {
                // Detect redacted export.
                let redacted = !v.has_password() && !v.has_key();
                if redacted {
                    return Err(SshCliError::InvalidArgument(format!(
                        "host '{name}' looks like a redacted export (no password/key). \
                         Use `vps export --include-secrets`, complete with `vps edit`, \
                         or `vps import --allow-incomplete`. Detail: {err}"
                    ))
                    .into());
                }
                return Err(SshCliError::InvalidArgument(format!(
                    "host '{name}' invalid in import: {err}"
                ))
                .into());
            }
        }
    }
    // Every per-host validation above already ran, so a plan that reaches this
    // point is one the real run would also accept — the preview and the execution
    // share the same code path up to the single `save`.
    if crate::cli::dry_run_stop(
        "vps-import",
        &[
            ("source", serde_json::json!(file.display().to_string())),
            ("config_path", serde_json::json!(path.display().to_string())),
            ("imported", serde_json::json!(imported_count)),
            ("hosts", serde_json::Value::Array(planned)),
        ],
    )? {
        return Ok(());
    }
    current.schema_version = model::CURRENT_SCHEMA_VERSION;
    guard.save(path, &current)?;
    drop(guard);
    replaced_names.sort_unstable();
    crate::output::emit_success(
        "vps-import",
        serde_json::json!({ "imported": imported_count, "replaced": replaced_names }),
        &crate::i18n::t(crate::i18n::Message::ImportCompleted),
        format == OutputFormat::Json,
    )?;
    Ok(())
}
