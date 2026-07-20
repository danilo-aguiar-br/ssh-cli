// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! Typed JSON wire DTOs for agent-facing stdout/stderr contracts.
//!
//! # Policy (Rules Rust — JSON / NDJSON)
//!
//! - **Format:** classic single-root JSON (object or array), RFC 8259. Not NDJSON.
//! - **Encoding:** UTF-8, no BOM on emit; BOM stripped on import parse.
//! - **Shape:** one complete document per CLI invocation on the data stream
//!   (stdout success or stderr error envelope), terminated by a single LF.
//! - **Pretty-print:** forbidden on the agent wire — compact `serde_json::to_string`
//!   only (machine interop; matches error envelope historical compact form).
//! - **Types:** known payloads use `Serialize`/`Deserialize` structs here; `Value`
//!   is reserved for genuinely dynamic trees (`meta command-tree`) and the
//!   flexible success-envelope field map at the emission boundary.
//! - **Must-Ignore:** import deserializers ignore unknown fields (default serde).
//! - **I-JSON:** integers are `u16`/`u32`/`u64`/`i32` within safe practical ranges
//!   (ports, exit codes, ms timeouts, byte counts); no `NaN`/`Infinity` floats.
//! - **Schemas:** hand-versioned under `docs/schemas/*.schema.json` (not generated
//!   at runtime). Agents validate offline; product does not embed a schema engine.
//! - **Config on disk:** TOML, not JSON5. JSON is only for CLI machine contracts
//!   and optional `vps export --json` / import of that envelope.

mod emit;
mod execution;
mod vps_export;

pub use emit::{
    print_json_line, print_json_line_stderr, strip_utf8_bom, write_json_line, ErrorEnvelope,
    SuccessEnvelope, UTF8_BOM,
};
pub use execution::{
    ExecBatchJson, ExecHostJson, ExecutionJson, HealthBatchJson, HealthCheckJson, HealthHostJson,
    ScpBatchJson, ScpHostJson, ScpTransferJson, SftpBatchJson, SftpFsOpJson, SftpListEntryJson,
    SftpListJson, SftpTransferJson, TunnelListeningJson,
};
pub use vps_export::{
    ExportHostJson, ImportDefaults, ImportEnvelope, ImportHostEntry, MaskedVpsJson, VpsExportJson,
};
