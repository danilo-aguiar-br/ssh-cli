// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP-03: import/export extracted from `vps/mod` (SRP).
#![forbid(unsafe_code)]
//! VPS inventory import/export (TOML + agent JSON envelope).
//!
//! Workload: **local disk + optional secret materialization**. Sequential justified:
//! single atomic write path; concurrent import would race flock/rename.

use super::{
    load, save, validate_key_path_exists, write_atomic, ConfigFile,
};
use super::model;
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
            &crate::i18n::t(crate::i18n::Message::ExportCompleted {
                path: path_display,
            }),
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
pub fn parse_import_payload(text: &str) -> SshCliResult<ConfigFile> {
    // Rules JSON: strip UTF-8 BOM before format detection / parse.
    let text = crate::json_wire::strip_utf8_bom(text);
    let trimmed = text.trim_start();
    if trimmed.starts_with('{') {
        parse_import_json(trimmed)
    } else {
        crate::validation::from_toml_str(text)
    }
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
    let mut hosts = BTreeMap::new();
    for (key, entry) in envelope.hosts {
        let rec = entry
            .into_record(&key, defaults)
            .map_err(SshCliError::InvalidArgument)?;
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
    let text = crate::paths::read_text_capped(file, crate::paths::MAX_CONFIG_TOML_BYTES)
        .map_err(SshCliError::Io)?;
    let imported = parse_import_payload(&text)?;
    let mut current = load(path)?;
    let mut imported_count = 0usize;
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
        match v.validate() {
            Ok(()) => {
                current.hosts.insert(name.as_str().to_owned(), v);
                imported_count += 1;
            }
            Err(ref err) if allow_incomplete => {
                // GAP-SSH-IMP-001: incomplete skeleton allowed.
                tracing::warn!(host = %name, error = %err, "import incomplete allowed");
                current.hosts.insert(name.as_str().to_owned(), v);
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
    current.schema_version = model::CURRENT_SCHEMA_VERSION;
    save(path, &current)?;
    crate::output::emit_success(
        "vps-import",
        serde_json::json!({ "imported": imported_count }),
        &crate::i18n::t(crate::i18n::Message::ImportCompleted),
        format == OutputFormat::Json,
    )?;
    Ok(())
}
