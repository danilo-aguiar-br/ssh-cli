// SPDX-License-Identifier: MIT OR Apache-2.0
//! Multi-host / multi-result batch emitters (G-COMP-06d).
//!
//! Keeps fan-out JSON/text formatting separate from single-record CRUD emitters.
#![forbid(unsafe_code)]

use super::{is_quiet, report_json_serialize_error};
use crate::domain::BatchRunId;
use crate::json_wire::{
    self, ExecBatchJson, ExecHostJson, HealthBatchJson, HealthHostJson, ScpBatchJson, ScpHostJson,
    ScpTransferJson, TunnelCloseReason, TunnelClosedJson, TunnelListeningJson,
};
#[cfg(feature = "ssh-real")]
use crate::json_wire::{
    SftpBatchJson, SftpFsOpJson, SftpListEntryJson, SftpListJson, SftpTransferJson,
};
// A6: the SFTP emitters below take types owned by the russh-backed subsystem, so
// they only exist when that stack is compiled in.
#[cfg(feature = "ssh-real")]
use crate::sftp::batch::HostSftpResult;
#[cfg(feature = "ssh-real")]
use crate::ssh::sftp_types::{SftpListEntry, SftpStat};
use crate::vps::{HostExecResult, HostHealthResult};
use std::io::{self, Write};

/// Prints multi-host health-check results (text or single-root JSON batch).
///
/// # Errors
/// Serialization or stdout I/O.
pub fn print_health_batch(
    results: &[HostHealthResult],
    max_concurrency: usize,
    json: bool,
) -> io::Result<()> {
    if json {
        // One v7 id per fan-out command (before/with emit; not per host).
        let batch_run_id = BatchRunId::new().to_string_canonical();
        let v = HealthBatchJson {
            event: "health-check-batch".into(),
            batch_run_id,
            max_concurrency: u32::try_from(max_concurrency).unwrap_or(u32::MAX),
            results: results
                .iter()
                .map(|h| HealthHostJson {
                    name: h.name.clone(),
                    status: if h.ok { "ok".into() } else { "error".into() },
                    latency_ms: h.latency_ms,
                    error: h.error.clone(),
                })
                .collect(),
        };
        return match json_wire::print_json_line(&v) {
            Ok(()) => Ok(()),
            Err(e) => {
                report_json_serialize_error(&e);
                Err(e)
            }
        };
    }
    if is_quiet() {
        return Ok(());
    }
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    writeln!(
        out,
        "health-check --all (max_concurrency={max_concurrency}, hosts={})",
        results.len()
    )?;
    for h in results {
        match (h.ok, h.latency_ms) {
            (true, Some(ms)) => writeln!(out, "  ok  {}  {ms}ms", h.name)?,
            (true, None) => writeln!(out, "  ok  {}", h.name)?,
            (false, _) => {
                let err = h.error.as_deref().unwrap_or("error");
                writeln!(out, "  ERR {}  {err}", h.name)?;
            }
        }
    }
    out.flush()
}

/// Prints multi-host exec results (text or single-root JSON batch).
///
/// # Errors
/// Serialization or stdout I/O.
pub fn print_exec_batch(
    results: &[HostExecResult],
    max_concurrency: usize,
    json: bool,
) -> io::Result<()> {
    if json {
        let batch_run_id = BatchRunId::new().to_string_canonical();
        let v = ExecBatchJson {
            event: "exec-batch".into(),
            batch_run_id,
            max_concurrency: u32::try_from(max_concurrency).unwrap_or(u32::MAX),
            results: results
                .iter()
                .map(|h| ExecHostJson {
                    name: h.name.clone(),
                    ok: h.ok,
                    exit_code: h.exit_code,
                    stdout: h.stdout.clone(),
                    stderr: h.stderr.clone(),
                    duration_ms: h.duration_ms,
                    error: h.error.clone(),
                })
                .collect(),
        };
        return match json_wire::print_json_line(&v) {
            Ok(()) => Ok(()),
            Err(e) => {
                report_json_serialize_error(&e);
                Err(e)
            }
        };
    }
    if is_quiet() {
        return Ok(());
    }
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    writeln!(
        out,
        "exec --all (max_concurrency={max_concurrency}, hosts={})",
        results.len()
    )?;
    for h in results {
        let status = if h.ok { "ok" } else { "ERR" };
        writeln!(
            out,
            "  {status}  {}  exit={:?}  {}ms",
            h.name, h.exit_code, h.duration_ms
        )?;
        if !h.stdout.is_empty() {
            for line in h.stdout.lines() {
                writeln!(out, "    | {line}")?;
            }
        }
        if !h.stderr.is_empty() {
            for line in h.stderr.lines() {
                writeln!(out, "    ! {line}")?;
            }
        }
    }
    out.flush()
}

/// Prints multi-host SCP batch results.
///
/// # Errors
/// Serialization or stdout I/O.
pub fn print_scp_batch(
    direction: &str,
    results: &[crate::scp::HostScpResult],
    max_concurrency: usize,
    json: bool,
) -> io::Result<()> {
    if json {
        let batch_run_id = BatchRunId::new().to_string_canonical();
        let v = ScpBatchJson {
            event: "scp-batch".into(),
            batch_run_id,
            direction: direction.into(),
            max_concurrency: u32::try_from(max_concurrency).unwrap_or(u32::MAX),
            results: results
                .iter()
                .map(|h| ScpHostJson {
                    name: h.name.clone(),
                    ok: h.ok,
                    bytes: h.bytes,
                    duration_ms: h.duration_ms,
                    local: h.local.clone(),
                    error: h.error.clone(),
                })
                .collect(),
        };
        return match json_wire::print_json_line(&v) {
            Ok(()) => Ok(()),
            Err(e) => {
                report_json_serialize_error(&e);
                Err(e)
            }
        };
    }
    if is_quiet() {
        return Ok(());
    }
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    writeln!(
        out,
        "scp {direction} --all (max_concurrency={max_concurrency}, hosts={})",
        results.len()
    )?;
    for h in results {
        if h.ok {
            writeln!(
                out,
                "  ok  {}  bytes={:?}  {:?}ms",
                h.name, h.bytes, h.duration_ms
            )?;
        } else {
            let err = h.error.as_deref().unwrap_or("error");
            writeln!(out, "  ERR {}  {err}", h.name)?;
        }
    }
    out.flush()
}

/// Prints an SCP transfer result as JSON (GAP-SSH-IO-007 / SCP-021 / IO-009).
///
/// # Errors
/// Serialization or stdout I/O (including BrokenPipe).
pub fn print_transfer_json(
    direction: &str,
    vps: &str,
    local: &str,
    remote: &str,
    result: &crate::ssh::client::TransferResult,
) -> io::Result<()> {
    // GAP-SSH-IO-009: event discriminator (parity with tunnel_listening).
    let v = ScpTransferJson {
        ok: true,
        event: "scp-transfer".into(),
        direction: direction.to_string(),
        vps: vps.to_string(),
        local: local.to_string(),
        remote: remote.to_string(),
        bytes: result.bytes_transferred,
        duration_ms: result.duration_ms,
        mtime_preserved: result.mtime_preserved,
        durable: result.durable,
    };
    match json_wire::print_json_line(&v) {
        Ok(()) => Ok(()),
        Err(e) => {
            report_json_serialize_error(&e);
            Err(e)
        }
    }
}

/// Prints an SFTP transfer result as JSON (G-SFTP-09).
///
/// # Errors
/// Serialization or stdout I/O.
#[cfg(feature = "ssh-real")]
pub fn print_sftp_transfer_json(
    direction: &str,
    vps: &str,
    local: &str,
    remote: &str,
    bytes: u64,
    duration_ms: u64,
    recursive: bool,
) -> io::Result<()> {
    let v = SftpTransferJson {
        ok: true,
        event: "sftp-transfer".into(),
        direction: direction.to_string(),
        vps: vps.to_string(),
        local: local.to_string(),
        remote: remote.to_string(),
        bytes,
        duration_ms,
        recursive,
    };
    match json_wire::print_json_line(&v) {
        Ok(()) => Ok(()),
        Err(e) => {
            report_json_serialize_error(&e);
            Err(e)
        }
    }
}

/// Prints `sftp ls` JSON.
///
/// # Errors
/// Serialization or stdout I/O.
#[cfg(feature = "ssh-real")]
pub fn print_sftp_list_json(vps: &str, path: &str, entries: &[SftpListEntry]) -> io::Result<()> {
    let v = SftpListJson {
        ok: true,
        event: "sftp-list".into(),
        vps: vps.to_string(),
        path: path.to_string(),
        entries: entries
            .iter()
            .map(|e| SftpListEntryJson {
                name: e.name.clone(),
                path: e.path.clone(),
                kind: e.kind.clone(),
                size: e.size,
                mode: e.mode,
            })
            .collect(),
    };
    match json_wire::print_json_line(&v) {
        Ok(()) => Ok(()),
        Err(e) => {
            report_json_serialize_error(&e);
            Err(e)
        }
    }
}

/// Prints `sftp` fs-op JSON (mkdir/rmdir/rm/rename).
///
/// # Errors
/// Serialization or stdout I/O.
#[cfg(feature = "ssh-real")]
pub fn print_sftp_fs_op_json(
    op: &str,
    vps: &str,
    path: &str,
    to: Option<&str>,
    duration_ms: u64,
) -> io::Result<()> {
    let v = SftpFsOpJson {
        ok: true,
        event: "sftp-fs-op".into(),
        op: op.to_string(),
        vps: vps.to_string(),
        path: path.to_string(),
        to: to.map(str::to_owned),
        duration_ms,
        kind: None,
        size: None,
        mode: None,
        mtime: None,
    };
    match json_wire::print_json_line(&v) {
        Ok(()) => Ok(()),
        Err(e) => {
            report_json_serialize_error(&e);
            Err(e)
        }
    }
}

/// Prints `sftp stat` JSON.
///
/// # Errors
/// Serialization or stdout I/O.
#[cfg(feature = "ssh-real")]
pub fn print_sftp_stat_json(vps: &str, st: &SftpStat) -> io::Result<()> {
    let v = SftpFsOpJson {
        ok: true,
        event: "sftp-fs-op".into(),
        op: "stat".into(),
        vps: vps.to_string(),
        path: st.path.clone(),
        to: None,
        duration_ms: 0,
        kind: Some(st.kind.clone()),
        size: st.size,
        mode: st.mode,
        mtime: st.mtime,
    };
    match json_wire::print_json_line(&v) {
        Ok(()) => Ok(()),
        Err(e) => {
            report_json_serialize_error(&e);
            Err(e)
        }
    }
}

/// Prints multi-host SFTP batch results.
///
/// # Errors
/// Serialization or stdout I/O.
#[cfg(feature = "ssh-real")]
pub fn print_sftp_batch(
    direction: &str,
    results: &[HostSftpResult],
    max_concurrency: usize,
    json: bool,
) -> io::Result<()> {
    if json {
        let batch_run_id = BatchRunId::new().to_string_canonical();
        let v = SftpBatchJson {
            event: "sftp-batch".into(),
            batch_run_id,
            direction: direction.to_string(),
            max_concurrency: u32::try_from(max_concurrency).unwrap_or(u32::MAX),
            results: results
                .iter()
                .map(|h| ScpHostJson {
                    name: h.name.clone(),
                    ok: h.ok,
                    bytes: h.bytes,
                    duration_ms: h.duration_ms,
                    local: h.local.clone(),
                    error: h.error.clone(),
                })
                .collect(),
        };
        return match json_wire::print_json_line(&v) {
            Ok(()) => Ok(()),
            Err(e) => {
                report_json_serialize_error(&e);
                Err(e)
            }
        };
    }
    if is_quiet() {
        return Ok(());
    }
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    writeln!(
        out,
        "sftp {direction} --all (max_concurrency={max_concurrency}, hosts={})",
        results.len()
    )?;
    for h in results {
        if h.ok {
            writeln!(
                out,
                "  ok  {}  bytes={:?}  ms={:?}",
                h.name, h.bytes, h.duration_ms
            )?;
        } else {
            let err = h.error.as_deref().unwrap_or("error");
            writeln!(out, "  ERR {}  {err}", h.name)?;
        }
    }
    out.flush()
}

/// Builds the `tunnel_listening` payload without writing it anywhere.
///
/// Split out from the printer so the document can be asserted on. Every field of this
/// event was previously reachable only by driving a real listener and reading stdout,
/// which is why `bind` — added by G-TUN-R06 precisely so an agent could audit whether a
/// service had been published beyond loopback — shipped covered by nothing but a schema
/// file and a mention in prose.
#[must_use]
pub fn build_tunnel_listening(
    vps: &str,
    local_port: u16,
    remote_host: &str,
    remote_port: u16,
    timeout_ms: u64,
    bind: &str,
    mode: &str,
) -> TunnelListeningJson {
    TunnelListeningJson {
        ok: true,
        event: "tunnel_listening".into(),
        vps: vps.to_string(),
        local_port,
        remote_host: remote_host.to_string(),
        remote_port,
        timeout_ms,
        bind: bind.to_string(),
        mode: mode.to_string(),
    }
}

/// JSON event when the local tunnel listener comes up (GAP-SSH-IO-008).
///
/// # Errors
/// Serialization or stdout I/O (including BrokenPipe).
pub fn print_tunnel_listening_json(
    vps: &str,
    local_port: u16,
    remote_host: &str,
    remote_port: u16,
    timeout_ms: u64,
    bind: &str,
    mode: &str,
) -> io::Result<()> {
    let v = build_tunnel_listening(
        vps,
        local_port,
        remote_host,
        remote_port,
        timeout_ms,
        bind,
        mode,
    );
    match json_wire::print_json_line(&v) {
        Ok(()) => Ok(()),
        Err(e) => {
            report_json_serialize_error(&e);
            Err(e)
        }
    }
}

/// Builds the `tunnel_closed` payload without writing it anywhere.
///
/// The whole event — `reason`, `forwards_served`, `capacity_waits`, `ok` — used to be
/// constructed inside the printer, so nothing could inspect it. The 0.5.4 audit found
/// the three field names appearing in exactly one place outside the emitter: a
/// documentation test asserting the CHANGELOG mentions them. Deleting the emission
/// would not have turned the suite red, which is the same failure mode G-QA-R01 was
/// written to stop.
#[must_use]
pub fn build_tunnel_closed(input: TunnelClosedInput<'_>) -> TunnelClosedJson {
    TunnelClosedJson {
        // An accept-error shutdown is not a clean lifetime, even though the process
        // still exits 0 for having bound successfully.
        ok: !matches!(input.reason, TunnelCloseReason::AcceptError),
        event: "tunnel_closed".into(),
        vps: input.vps.to_string(),
        reason: input.reason,
        bind: input.bind.to_string(),
        local_port: input.local_port,
        forwards_served: input.forwards_served,
        capacity_waits: input.capacity_waits,
        duration_ms: input.duration_ms,
        mode: input.mode.to_string(),
    }
}

/// Inputs for [`build_tunnel_closed`].
///
/// B3: three of the eight fields are bare `u64` counters and one is a `u16`
/// port. Passed positionally, swapping `forwards_served` with `capacity_waits`
/// compiles and produces a plausible-looking event that misreports the tunnel's
/// lifetime — the exact class of silent wrongness the suppressed
/// `too_many_arguments` lint was pointing at.
pub struct TunnelClosedInput<'a> {
    /// Registry name of the relay host.
    pub vps: &'a str,
    /// Why the tunnel stopped.
    pub reason: TunnelCloseReason,
    /// Effective local bind address.
    pub bind: &'a str,
    /// Effective local port (OS-assigned when `0` was requested).
    pub local_port: u16,
    /// Connections accepted over the tunnel's lifetime.
    pub forwards_served: u64,
    /// Times an accept waited on the concurrency semaphore.
    pub capacity_waits: u64,
    /// Wall lifetime in milliseconds.
    pub duration_ms: u64,
    /// Tunnel mode label (`local`, `reverse`, `socks5`, `streamlocal`).
    pub mode: &'a str,
}

/// Emits the `tunnel_closed` shutdown event (G-TUN-R07).
///
/// Always emitted, including on the happy deadline path, so an agent can tell the
/// three endings apart instead of inferring them from a shared exit 0.
///
/// # Errors
/// Serialization or stdout I/O (including BrokenPipe).
pub fn print_tunnel_closed_json(event: &TunnelClosedJson) -> io::Result<()> {
    match json_wire::print_json_line(event) {
        Ok(()) => Ok(()),
        Err(e) => {
            report_json_serialize_error(&e);
            Err(e)
        }
    }
}
