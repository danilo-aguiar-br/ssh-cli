// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: JSON VPS/exec emitters (extracted from output monólito).
#![forbid(unsafe_code)]
//! JSON-mode formatters for VPS inventory and one-shot results.

use super::emit::report_json_serialize_error;
use crate::json_wire::{
    self, ExecutionJson, ExportHostJson, HealthCheckJson, MaskedVpsJson, VpsExportJson,
};
use crate::ssh::ExecutionOutput;
use crate::vps::model::VpsRecord;
use std::collections::BTreeMap;
use std::io;

/// Prints VPS list as compact JSON array on stdout.
pub fn print_list_json(records: &[VpsRecord]) -> io::Result<()> {
    let list: Vec<MaskedVpsJson> = records.iter().map(MaskedVpsJson::from).collect();
    match json_wire::print_json_line(&list) {
        Ok(()) => Ok(()),
        Err(err) => {
            report_json_serialize_error(&err);
            Err(err)
        }
    }
}

/// Prints a single VPS record as masked text.
///
pub fn print_details_json(r: &VpsRecord) -> io::Result<()> {
    let v = MaskedVpsJson::from(r);
    match json_wire::print_json_line(&v) {
        Ok(()) => Ok(()),
        Err(err) => {
            report_json_serialize_error(&err);
            Err(err)
        }
    }
}

/// Builds a masked VPS JSON DTO (GAP-SSH-JSON-001).
#[must_use]
pub fn record_to_masked_json(r: &VpsRecord) -> MaskedVpsJson {
    MaskedVpsJson::from(r)
}

/// GAP-SSH-UX-001: hosts for `vps export --json`.
///
/// - Redacted (`include_secrets=false`): empty secrets / null optional, **never**
///   ciphertext `sshcli-enc:` (EXP-001 parity). Empty password → `""` skeleton.
/// - With secrets: plaintext only if `--include-secrets` (same risk as TOML).
#[must_use]
pub fn export_hosts_to_json(
    hosts: &BTreeMap<String, VpsRecord>,
    include_secrets: bool,
) -> BTreeMap<String, ExportHostJson> {
    let mut map = BTreeMap::new();
    for (name, r) in hosts {
        map.insert(
            name.clone(),
            ExportHostJson::from_record(r, include_secrets),
        );
    }
    map
}

/// Full `vps export --json` envelope (typed).
#[must_use]
pub fn export_envelope_json(
    hosts: &BTreeMap<String, VpsRecord>,
    schema_version: u32,
    include_secrets: bool,
) -> VpsExportJson {
    VpsExportJson {
        ok: true,
        event: "vps-export".into(),
        schema_version,
        include_secrets,
        hosts: export_hosts_to_json(hosts, include_secrets),
    }
}

/// Prints stdout/stderr from an SSH command execution.
///
/// Format:
/// ```text
/// --- stdout ---
/// <stdout>
/// --- stderr ---
/// <stderr>
/// --- exit code: <code> (<duration_ms>ms) ---
/// ```
///
/// Streams under one stdout lock with `writeln!` (G-MAC-02) — no cloned
pub fn print_execution_output_json(output: &ExecutionOutput) -> io::Result<()> {
    let v = ExecutionJson::from(output);
    match json_wire::print_json_line(&v) {
        Ok(()) => Ok(()),
        Err(e) => {
            report_json_serialize_error(&e);
            Err(e)
        }
    }
}

/// Prints a health-check result as JSON.
pub fn print_health_check_json(name: &str, latency_ms: u64) -> io::Result<()> {
    let v = HealthCheckJson {
        name: name.to_string(),
        status: "ok".into(),
        latency_ms,
    };
    match json_wire::print_json_line(&v) {
        Ok(()) => Ok(()),
        Err(e) => {
            report_json_serialize_error(&e);
            Err(e)
        }
    }
}
