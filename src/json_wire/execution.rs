// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: exec/health/scp/sftp/tunnel JSON DTOs (extracted from json_wire monolith).
#![forbid(unsafe_code)]
//! Typed JSON DTOs for one-shot SSH operation results.

use crate::ssh::ExecutionOutput;
use serde::{Deserialize, Serialize};

// `TargetSource`, `ExecTarget` and the process-wide resolved-target slot live in
// `json_wire/exec_target.rs`. They are re-exported from `json_wire` so call sites
// are unaffected by the split.
use super::exec_target::{ExecTarget, TargetEcho, TargetSource};

/// `exec` / `sudo-exec` / `su-exec` JSON stdout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionJson {
    /// Canonical target fields plus their `host_*` read aliases.
    #[serde(flatten)]
    pub target: TargetEcho,
    /// Whether the host was inherited from the active marker instead of argv.
    ///
    /// Redundant with [`TargetEcho::target_source`] on purpose: a boolean gate is what
    /// a shell pipeline can branch on without knowing the enum spelling.
    #[serde(default)]
    pub active_fallback: bool,
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

impl ExecutionJson {
    /// Builds the envelope with the target identity attached.
    ///
    /// The only constructor, deliberately. A `From<&ExecutionOutput>` conversion used
    /// to exist and filled the audit fields with empty values, because an
    /// `ExecutionOutput` knows what happened and not where. Removing it makes an
    /// envelope that disclaims its own target unrepresentable.
    #[must_use]
    pub fn with_target(o: &ExecutionOutput, target: &ExecTarget) -> Self {
        Self {
            target: TargetEcho::new(target),
            active_fallback: target.source.is_ambient(),
            stdout: o.stdout.clone(),
            stderr: o.stderr.clone(),
            exit_code: o.exit_code,
            truncated_stdout: o.truncated_stdout,
            truncated_stderr: o.truncated_stderr,
            duration_ms: o.duration_ms,
        }
    }
}

// `From<&ExecutionOutput> for ExecutionJson` used to live here and filled
// `host_resolved` with an empty string, because an `ExecutionOutput` knows what
// happened and not where. Measured before removal: its only caller was a unit test,
// so no production path ever emitted a host-less envelope — but the conversion was a
// loaded gun pointing at GAP-SSH-EXEC-ENVELOPE-002. The first future `.into()` would
// have produced an envelope that silently disclaims its own target, which is the
// exact condition that gap exists to remove. [`ExecutionJson::with_target`] is now
// the only constructor, so "envelope without a target" is unrepresentable rather
// than merely discouraged.

/// `health-check --json` stdout (single host).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthCheckJson {
    /// VPS name checked.
    pub name: String,
    /// Resolved target under the canonical `target_*` names and the `host_*` aliases.
    ///
    /// The marker is reachable only through an explicit `--use-active`; a nameless
    /// `health-check` is a usage error. Provenance is still reported, because the
    /// opt-in is a claim about intent and the field is what makes it checkable: it
    /// separates a host the caller typed from one `connect` last wrote, and a reader
    /// auditing a probe after the fact cannot recover that from the name alone.
    #[serde(flatten)]
    pub target: TargetEcho,
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
    /// How the host set was designated.
    ///
    /// [`TargetSource::Selector`] for the fleet paths, and [`TargetSource::Argv`] for
    /// single-host multi-file (G-PAR-37 / G-PAR-47), which reuses this envelope for a
    /// host the caller typed. Hardcoding `selector` would make that second case lie
    /// about its own designation.
    ///
    /// There is deliberately no `target_resolved` here. A fleet run resolves to a
    /// *set*, and every member is already named in `results[].name`; collapsing that
    /// set into one scalar would be an assertion the run cannot make.
    #[serde(default)]
    pub target_source: TargetSource,
    /// Compatibility alias of [`Self::target_source`] (0.5.5 spelling).
    #[serde(default)]
    pub host_source: TargetSource,
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
    /// Resolved target under the canonical `target_*` names and the `host_*` aliases.
    ///
    /// Explicit Target Designation: a transfer is a side effect, so the envelope has
    /// to name the machine it landed on and say how that machine was chosen. Without
    /// it a fleet transfer and a single-host transfer to the same name were the same
    /// bytes, and a misdirected upload could not be detected from stdout alone. `vps`
    /// carries the same string, but only [`TargetEcho::target_source`] states whether
    /// the caller typed it or a selector produced it.
    #[serde(flatten)]
    pub target: TargetEcho,
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
    /// Whether the remote modification time landed on the local file.
    ///
    /// G-SCP-R01: the CLI documented mtime preservation as a guarantee while treating
    /// it as best-effort in code, and the failure was discarded at two nesting levels.
    /// A build pipeline that decides whether to recompile by comparing mtime could act
    /// on a timestamp that was never applied, with the symptom surfacing far from the
    /// cause. Additive and defaulted to `true`, so events written before this field
    /// existed still deserialize and pre-existing consumers are unaffected.
    #[serde(default = "default_true")]
    pub mtime_preserved: bool,
    /// Whether the parent directory was fsynced after the atomic rename.
    ///
    /// G-SCP-R02: the rename is atomic but the directory entry is not durable until it
    /// is flushed. Success was reported unqualified, so an agent could not tell a
    /// durable write from one that a power loss would erase.
    #[serde(default = "default_true")]
    pub durable: bool,
}

/// Wire default for the additive SCP durability flags.
///
/// `true` rather than `false`: an event that predates the fields was emitted by a
/// build that never reported a failure, so assuming loss would invent one.
fn default_true() -> bool {
    true
}

/// `sftp upload|download --json` success stdout (G-SFTP-09).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SftpTransferJson {
    /// Always `true`.
    pub ok: bool,
    /// Discriminator: `"sftp-transfer"`.
    pub event: String,
    /// Resolved target under the canonical `target_*` names and the `host_*` aliases.
    ///
    /// Same reason as [`ScpTransferJson::target`]: a write to a remote filesystem must
    /// declare which filesystem it reached.
    #[serde(flatten)]
    pub target: TargetEcho,
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
    /// Provenance of the host set (see [`ScpBatchJson::target_source`]).
    #[serde(default)]
    pub target_source: TargetSource,
    /// Compatibility alias of [`Self::target_source`] (0.5.5 spelling).
    #[serde(default)]
    pub host_source: TargetSource,
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
    /// Effective local bind address.
    ///
    /// G-TUN-R06: without this an agent could not tell a `127.0.0.1` bind from a
    /// `0.0.0.0` one, so it had no way to audit — from the structured contract alone —
    /// whether it had just published a remote database to the local network. Additive
    /// field: existing consumers are unaffected.
    pub bind: String,
    /// Which tunnel mode is serving: `local`, `socks5`, `streamlocal` or `reverse`.
    ///
    /// The other fields are read differently per mode — `local_port` is a local
    /// listener for three of them and the *server's* port for `reverse`, and
    /// `remote_host` is a concrete target except under SOCKS5, where the
    /// destination is chosen per connection. Without this discriminator an agent
    /// would have to infer the mode from the flags it passed, which stops working
    /// the moment anything else launches the tunnel.
    #[serde(default = "default_tunnel_mode")]
    pub mode: String,
}

/// Wire default for [`TunnelListeningJson::mode`] / [`TunnelClosedJson::mode`].
///
/// Events written before the mode field existed can only have been plain local
/// forwards, so deserializing them as `local` is a statement of fact rather than
/// a guess.
fn default_tunnel_mode() -> String {
    "local".to_string()
}

/// Reason a tunnel stopped serving, reported by [`TunnelClosedJson`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelCloseReason {
    /// The `--timeout-ms` deadline elapsed after a successful bind (exit 0).
    Deadline,
    /// SIGINT / SIGTERM arrived.
    Signal,
    /// The accept loop hit a fatal error and stopped early.
    AcceptError,
}

/// `tunnel --json` shutdown event.
///
/// G-TUN-R07: the tunnel used to emit `tunnel_listening` and then fall silent until
/// death, so three very different endings shared exit 0 — deadline reached, signal
/// received, and a fatal accept error that broke the loop early. The last case was
/// the worst: because `bound` was already true, the timeout wrapper returned `Ok(())`
/// and the process reported success even though it had stopped accepting connections
/// seconds into a five-minute deadline. Those are now distinguishable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TunnelClosedJson {
    /// `true` when the tunnel served its full lifetime.
    pub ok: bool,
    /// Discriminator: `"tunnel_closed"`.
    pub event: String,
    /// VPS name.
    pub vps: String,
    /// Why the tunnel stopped.
    pub reason: TunnelCloseReason,
    /// Effective local bind address.
    pub bind: String,
    /// Local listen port that was served.
    pub local_port: u16,
    /// Connections accepted and forwarded during the tunnel's lifetime.
    ///
    /// G-TUN-R11: answers the most basic diagnostic question — did anything ever
    /// connect? A tunnel that bound correctly but was never used produced output
    /// identical to one that served five hundred connections.
    pub forwards_served: u64,
    /// Times a new connection had to wait for a concurrency permit.
    ///
    /// G-TUN-R12: saturation was previously invisible, so the symptom was rising
    /// latency with no stated cause and the operator could hunt the network instead.
    pub capacity_waits: u64,
    /// Wall-clock lifetime in milliseconds.
    pub duration_ms: u64,
    /// Which tunnel mode was serving (mirrors [`TunnelListeningJson::mode`]).
    #[serde(default = "default_tunnel_mode")]
    pub mode: String,
}

#[cfg(test)]
#[path = "execution_tests.rs"]
mod tests;
