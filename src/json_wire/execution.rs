// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: exec/health/scp/sftp/tunnel JSON DTOs (extracted from json_wire monólito).
#![forbid(unsafe_code)]
//! Typed JSON DTOs for one-shot SSH operation results.

use crate::ssh::ExecutionOutput;
use serde::{Deserialize, Serialize};

/// `exec` / `sudo-exec` / `su-exec` JSON stdout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionJson {
    /// Captured remote stdout.
    pub stdout: String,
    /// Captured remote stderr.
    pub stderr: String,
    /// Remote exit code when available.
    pub exit_code: Option<i32>,
    /// Whether stdout was truncated by max_output_chars.
    pub truncated_stdout: bool,
    /// Whether stderr was truncated by max_output_chars.
    pub truncated_stderr: bool,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
}

impl From<&ExecutionOutput> for ExecutionJson {
    fn from(o: &ExecutionOutput) -> Self {
        Self {
            stdout: o.stdout.clone(),
            stderr: o.stderr.clone(),
            exit_code: o.exit_code,
            truncated_stdout: o.truncated_stdout,
            truncated_stderr: o.truncated_stderr,
            duration_ms: o.duration_ms,
        }
    }
}

/// `health-check --json` stdout (single host).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthCheckJson {
    /// VPS name checked.
    pub name: String,
    /// Always `"ok"` on success path.
    pub status: String,
    /// Round-trip latency in milliseconds.
    pub latency_ms: u64,
}

/// One entry in `health-check --all --json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthHostJson {
    /// VPS name.
    pub name: String,
    /// `"ok"` or `"error"`.
    pub status: String,
    /// Latency when measured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    /// Error detail when status is error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// `health-check --all --json` batch envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthBatchJson {
    /// Discriminator.
    pub event: String,
    /// Time-ordered UUID v7 correlating this multi-host run (G-DOM-05).
    pub batch_run_id: String,
    /// Concurrency budget used for the fan-out.
    pub max_concurrency: u32,
    /// Per-host results (stable name order when possible).
    pub results: Vec<HealthHostJson>,
}

/// One entry in multi-host `exec --all --json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecHostJson {
    /// VPS name.
    pub name: String,
    /// Whether remote exit was 0.
    pub ok: bool,
    /// Remote exit code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Error summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Multi-host exec batch envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecBatchJson {
    /// Discriminator.
    pub event: String,
    /// Time-ordered UUID v7 correlating this multi-host run (G-DOM-05).
    pub batch_run_id: String,
    /// Concurrency budget used.
    pub max_concurrency: u32,
    /// Per-host results.
    pub results: Vec<ExecHostJson>,
}

/// One entry in multi-host `scp --all --json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScpHostJson {
    /// VPS name.
    pub name: String,
    /// Whether transfer succeeded.
    pub ok: bool,
    /// Bytes transferred when ok.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    /// Duration ms when measured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Local path used (download may be host-suffixed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<String>,
    /// Error detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Multi-host SCP batch envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScpBatchJson {
    /// Discriminator.
    pub event: String,
    /// Time-ordered UUID v7 correlating this multi-host run (G-DOM-05).
    pub batch_run_id: String,
    /// `"upload"` or `"download"`.
    pub direction: String,
    /// Concurrency budget used.
    pub max_concurrency: u32,
    /// Per-host results.
    pub results: Vec<ScpHostJson>,
}

/// `scp upload|download --json` success stdout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScpTransferJson {
    /// Always `true`.
    pub ok: bool,
    /// Discriminator: `"scp-transfer"`.
    pub event: String,
    /// `"upload"` or `"download"`.
    pub direction: String,
    /// VPS name.
    pub vps: String,
    /// Local path.
    pub local: String,
    /// Remote path.
    pub remote: String,
    /// Bytes transferred.
    pub bytes: u64,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

/// `sftp upload|download --json` success stdout (G-SFTP-09).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SftpTransferJson {
    /// Always `true`.
    pub ok: bool,
    /// Discriminator: `"sftp-transfer"`.
    pub event: String,
    /// `"upload"` or `"download"`.
    pub direction: String,
    /// VPS name.
    pub vps: String,
    /// Local path.
    pub local: String,
    /// Remote path.
    pub remote: String,
    /// Bytes transferred.
    pub bytes: u64,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the transfer was recursive.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub recursive: bool,
}

/// One entry in `sftp ls --json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SftpListEntryJson {
    /// Base name.
    pub name: String,
    /// Full remote path.
    pub path: String,
    /// `file` | `dir` | `symlink` | `other`.
    pub kind: String,
    /// Size when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Mode bits when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
}

/// `sftp ls --json` success stdout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SftpListJson {
    /// Always `true`.
    pub ok: bool,
    /// Discriminator: `"sftp-list"`.
    pub event: String,
    /// VPS name.
    pub vps: String,
    /// Directory path listed.
    pub path: String,
    /// Entries.
    pub entries: Vec<SftpListEntryJson>,
}

/// `sftp mkdir|rmdir|rm|rename|stat --json` (stat uses size/mode fields).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SftpFsOpJson {
    /// Always `true`.
    pub ok: bool,
    /// Discriminator: `"sftp-fs-op"`.
    pub event: String,
    /// Operation name.
    pub op: String,
    /// VPS name.
    pub vps: String,
    /// Primary path.
    pub path: String,
    /// Rename target when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Duration ms.
    pub duration_ms: u64,
    /// Stat kind when op=stat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Stat size when op=stat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Stat mode when op=stat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
    /// Stat mtime when op=stat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime: Option<u32>,
}

/// Multi-host SFTP batch (reuses scp-host shape).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SftpBatchJson {
    /// Discriminator: `"sftp-batch"`.
    pub event: String,
    /// UUID v7 batch id.
    pub batch_run_id: String,
    /// `"upload"` or `"download"`.
    pub direction: String,
    /// Concurrency budget.
    pub max_concurrency: u32,
    /// Per-host results.
    pub results: Vec<ScpHostJson>,
}

/// `tunnel --json` post-bind event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TunnelListeningJson {
    /// Always `true`.
    pub ok: bool,
    /// Discriminator: `"tunnel_listening"`.
    pub event: String,
    /// VPS name.
    pub vps: String,
    /// Local listen port.
    pub local_port: u16,
    /// Remote target host.
    pub remote_host: String,
    /// Remote target port.
    pub remote_port: u16,
    /// One-shot timeout in milliseconds.
    pub timeout_ms: u64,
}
