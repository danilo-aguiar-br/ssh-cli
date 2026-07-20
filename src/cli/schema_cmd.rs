// SPDX-License-Identifier: MIT OR Apache-2.0
// G-E2E-02: runtime JSON Schema catalog for agent discovery.
#![forbid(unsafe_code)]
//! Embed and emit JSON Schemas from `docs/schemas/` (compile-time include).
//!
//! Workload: pure memory lookup (sequential; no fan-out). One-shot: list or body.

use crate::errors::{SshCliError, SshCliResult};

/// Embedded schema catalog: `(name, file leaf, body)`.
///
/// Names omit `.schema.json` and match `docs/schemas/README.md`.
const SCHEMAS: &[(&str, &str, &str)] = &[
    (
        "error-envelope",
        "error-envelope.schema.json",
        include_str!("../../docs/schemas/error-envelope.schema.json"),
    ),
    (
        "exec",
        "exec.schema.json",
        include_str!("../../docs/schemas/exec.schema.json"),
    ),
    (
        "exec-batch",
        "exec-batch.schema.json",
        include_str!("../../docs/schemas/exec-batch.schema.json"),
    ),
    (
        "health-check",
        "health-check.schema.json",
        include_str!("../../docs/schemas/health-check.schema.json"),
    ),
    (
        "health-check-batch",
        "health-check-batch.schema.json",
        include_str!("../../docs/schemas/health-check-batch.schema.json"),
    ),
    (
        "scp-batch",
        "scp-batch.schema.json",
        include_str!("../../docs/schemas/scp-batch.schema.json"),
    ),
    (
        "scp-transfer",
        "scp-transfer.schema.json",
        include_str!("../../docs/schemas/scp-transfer.schema.json"),
    ),
    (
        "secrets-init",
        "secrets-init.schema.json",
        include_str!("../../docs/schemas/secrets-init.schema.json"),
    ),
    (
        "secrets-reencrypt",
        "secrets-reencrypt.schema.json",
        include_str!("../../docs/schemas/secrets-reencrypt.schema.json"),
    ),
    (
        "sftp-batch",
        "sftp-batch.schema.json",
        include_str!("../../docs/schemas/sftp-batch.schema.json"),
    ),
    (
        "sftp-fs-op",
        "sftp-fs-op.schema.json",
        include_str!("../../docs/schemas/sftp-fs-op.schema.json"),
    ),
    (
        "sftp-list",
        "sftp-list.schema.json",
        include_str!("../../docs/schemas/sftp-list.schema.json"),
    ),
    (
        "sftp-transfer",
        "sftp-transfer.schema.json",
        include_str!("../../docs/schemas/sftp-transfer.schema.json"),
    ),
    (
        "su-exec",
        "su-exec.schema.json",
        include_str!("../../docs/schemas/su-exec.schema.json"),
    ),
    (
        "sudo-exec",
        "sudo-exec.schema.json",
        include_str!("../../docs/schemas/sudo-exec.schema.json"),
    ),
    (
        "tunnel-listening",
        "tunnel-listening.schema.json",
        include_str!("../../docs/schemas/tunnel-listening.schema.json"),
    ),
    (
        "vps-doctor",
        "vps-doctor.schema.json",
        include_str!("../../docs/schemas/vps-doctor.schema.json"),
    ),
    (
        "vps-export",
        "vps-export.schema.json",
        include_str!("../../docs/schemas/vps-export.schema.json"),
    ),
    (
        "vps-list",
        "vps-list.schema.json",
        include_str!("../../docs/schemas/vps-list.schema.json"),
    ),
    (
        "vps-show",
        "vps-show.schema.json",
        include_str!("../../docs/schemas/vps-show.schema.json"),
    ),
];

/// Runs `ssh-cli schema [NAME]`.
///
/// * No name → catalog JSON (`event: schema-catalog`)
/// * Name → raw JSON Schema document body
pub fn run_schema(name: Option<&str>, json: bool) -> SshCliResult<()> {
    match name {
        None => {
            let items: Vec<serde_json::Value> = SCHEMAS
                .iter()
                .map(|(n, file, _)| {
                    serde_json::json!({
                        "name": n,
                        "file": file,
                    })
                })
                .collect();
            if json {
                crate::output::print_json_value(&serde_json::json!({
                    "ok": true,
                    "event": "schema-catalog",
                    "schemas": items,
                }))?;
            } else {
                for (n, file, _) in SCHEMAS {
                    crate::output::write_line_fmt(format_args!("{n}\t{file}"))?;
                }
            }
            Ok(())
        }
        Some(n) => {
            let body = SCHEMAS
                .iter()
                .find(|(name, _, _)| *name == n)
                .map(|(_, _, body)| *body)
                .ok_or_else(|| {
                    SshCliError::InvalidArgument(format!(
                        "unknown schema '{n}'; run `ssh-cli schema` for the catalog"
                    ))
                })?;
            // Schema body is already JSON; emit raw on stdout (agent contract).
            crate::output::write_line(body.trim_end())?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_non_empty() {
        assert!(SCHEMAS.len() >= 18);
    }

    #[test]
    fn vps_list_present() {
        assert!(SCHEMAS.iter().any(|(n, _, b)| *n == "vps-list" && b.contains("schema")));
    }
}
