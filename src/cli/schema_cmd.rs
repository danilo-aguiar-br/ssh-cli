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
        "dry-run",
        "dry-run.schema.json",
        include_str!("../../docs/schemas/dry-run.schema.json"),
    ),
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
        "tunnel-closed",
        "tunnel-closed.schema.json",
        include_str!("../../docs/schemas/tunnel-closed.schema.json"),
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
        assert!(SCHEMAS.len() >= 22);
    }

    #[test]
    fn vps_list_present() {
        assert!(SCHEMAS
            .iter()
            .any(|(n, _, b)| *n == "vps-list" && b.contains("schema")));
    }

    /// Every schema on disk is reachable through `ssh-cli schema`, and vice versa.
    ///
    /// `docs/schemas/README.md` tells agents to discover the catalog at runtime, so a
    /// file that exists on disk but is missing from [`SCHEMAS`] is a published contract
    /// the consumer cannot fetch: `ssh-cli schema tunnel-closed` answered exit 64
    /// `unknown schema` while the README documented the very same name. Two schemas had
    /// drifted that way with no gate to notice, because the only assertions here were a
    /// lower bound on the count and one hand-picked name.
    ///
    /// The reverse direction matters too: a catalog entry without its file cannot exist
    /// (`include_str!` would fail the build), but a *renamed* file would leave the entry
    /// pointing at a stale leaf, so both sets are compared.
    #[test]
    fn catalog_and_disk_agree() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/schemas");

        let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
            .expect("read docs/schemas")
            .flatten()
            .filter_map(|e| e.file_name().to_str().map(str::to_owned))
            .filter(|f| f.ends_with(".schema.json"))
            .collect();
        on_disk.sort();

        let mut in_catalog: Vec<String> = SCHEMAS
            .iter()
            .map(|(_, file, _)| (*file).to_owned())
            .collect();
        in_catalog.sort();

        let missing: Vec<&String> = on_disk.iter().filter(|f| !in_catalog.contains(f)).collect();
        assert!(
            missing.is_empty(),
            "schemas published on disk but absent from the runtime catalog: {missing:?}"
        );

        let stale: Vec<&String> = in_catalog.iter().filter(|f| !on_disk.contains(f)).collect();
        assert!(
            stale.is_empty(),
            "catalog entries whose file no longer exists on disk: {stale:?}"
        );

        // Names are the lookup key an agent types; they must be the leaf minus the suffix.
        for (name, file, _) in SCHEMAS {
            assert_eq!(
                *file,
                format!("{name}.schema.json"),
                "catalog name '{name}' does not match its file leaf '{file}'"
            );
        }
    }
}
