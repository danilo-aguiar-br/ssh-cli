// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Cliente SSH real via `russh` 0.62.2.
//!
//! One-shot connection: TCP + handshake + auth (password and/or key) + exec with
//! timeout, output truncation, and best-effort remote abort.
//! Host keys: TOFU em `known_hosts` XDG (ver [`super::known_hosts`]).
//!
//! # Workload classification (resource economy)
//!
//! - **Class:** I/O-bound (SSH/TCP + disk SCP). Not CPU-bound.
//! - **Runtime:** Tokio multi-thread (see `main.rs`) for russh crypto/IO + tunnel
//!   accept fan-out — not a substitute for CPU parallelism.
//! - **No Rayon / no process pool:** one-shot single session; coordination cost
//!   exceeds any local CPU fan-out on the agent path.
//! - **Capture RAM:** stdout/stderr bounded by `max_chars×4` bytes (UTF-8 worst
//!   case) and hard-capped at [`EXEC_CAPTURE_HARD_MAX_BYTES`] per stream.
//! - **SCP:** stream in 32 KiB chunks to/from disk (no full-file heap load);
//!   disk I/O uses `tokio::fs` so the async worker is not blocked on syscalls.
//! - **Latency:** RTT-bound; decode path reuses the capture `Vec` via
//!   [`take_utf8_capped`] when remote bytes are valid UTF-8.

use crate::errors::SshCliError;
use tokio::io::{AsyncRead, AsyncWrite};

// G-COMP-R: ConnectionConfig lives in `connection` (SRP).
pub use super::connection::ConnectionConfig;

/// Output of a remote SSH command execution.
#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    /// Stdout capturado (possivelmente truncated a `max_chars` codepoints).
    pub stdout: String,
    /// Stderr capturado (possivelmente truncated a `max_chars` codepoints).
    pub stderr: String,
    /// Exit code. `None` when the command was terminated by signal or timeout.
    pub exit_code: Option<i32>,
    /// `true` se `stdout` foi truncated em `max_chars`.
    pub truncated_stdout: bool,
    /// `true` se `stderr` foi truncated em `max_chars`.
    pub truncated_stderr: bool,
    /// Total execution duration in milliseconds.
    pub duration_ms: u64,
}

/// Result of an SCP file transfer operation.
#[derive(Debug, Clone)]
pub struct TransferResult {
    /// Number of bytes transferred.
    pub bytes_transferred: u64,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
}

/// Hard upper bound (bytes) retained **per stream** while capturing remote exec output.
///
/// Resource rule: even with `max_chars` unlimited (`usize::MAX`), never grow
/// unbounded from remote flood. Post-decode codepoint truncation still applies.
pub(crate) const EXEC_CAPTURE_HARD_MAX_BYTES: usize = 16 * 1024 * 1024;

/// Bytes retained per stream while capturing remote output.
///
/// Uses UTF-8 worst-case 4 bytes/codepoint (+4 slack for a trailing incomplete
/// sequence), then clamps to [`EXEC_CAPTURE_HARD_MAX_BYTES`].
#[must_use]
pub(crate) fn exec_capture_byte_cap(max_chars: usize) -> usize {
    if max_chars == 0 {
        return 0;
    }
    max_chars
        .saturating_mul(4)
        .saturating_add(4)
        .min(EXEC_CAPTURE_HARD_MAX_BYTES)
}

/// Appends `data` into `buf` without exceeding `cap`; sets `truncated` if any byte is dropped.
pub(crate) fn append_capped(buf: &mut Vec<u8>, data: &[u8], cap: usize, truncated: &mut bool) {
    if data.is_empty() {
        return;
    }
    if cap == 0 || buf.len() >= cap {
        *truncated = true;
        return;
    }
    let room = cap - buf.len();
    if data.len() <= room {
        buf.extend_from_slice(data);
    } else {
        buf.extend_from_slice(&data[..room]);
        *truncated = true;
    }
}

// G-TYPE-14: UTF-8 truncation lives in `session_io` (re-exported for callers).
pub use super::session_io::truncate_utf8;
pub(crate) use super::session_io::take_utf8_capped;

// =========================================================================
// SshClientTrait enables real or mock SSH clients in tests.
// =========================================================================

use async_trait::async_trait;
use std::path::Path;

/// Bidirectional stream used for SSH tunnel (direct-tcpip).
pub trait TunnelChannel: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T> TunnelChannel for T where T: AsyncRead + AsyncWrite + Unpin + Send {}

/// SSH client trait allowing a real (russh) or mock implementation for tests.
///
/// Abstracts SSH connection operations so unit tests can run without a real network.
#[async_trait]
pub trait SshClientTrait: Send + Sync + 'static {
    /// Connects to an SSH server and authenticates with the provided credentials.
    async fn connect(cfg: ConnectionConfig) -> Result<Box<Self>, SshCliError>
    where
        Self: Sized;

    /// Runs a remote shell command and returns the captured output.
    ///
    /// `stdin_data`, if present, is written to the channel after `exec` and before the loop
    /// read loop (GAP-SSH-SEC-001: sudo/su password stays off the remote argv).
    async fn run_command(
        &mut self,
        cmd: &str,
        max_chars: usize,
        stdin_data: Option<Vec<u8>>,
    ) -> Result<ExecutionOutput, SshCliError>;

    /// Uploads a local file to the remote server via SCP.
    async fn upload(
        &self,
        local: &Path,
        remote: &Path,
    ) -> Result<TransferResult, SshCliError>;

    /// Downloads a remote file to the local filesystem via SCP.
    async fn download(
        &self,
        remote: &Path,
        local: &Path,
    ) -> Result<TransferResult, SshCliError>;

    /// Opens a `direct-tcpip` channel for tunnel forwarding.
    async fn open_tunnel_channel(
        &self,
        remote_host: &str,
        remote_port: u16,
        origin_addr: &str,
        origin_port: u16,
    ) -> Result<Box<dyn TunnelChannel>, SshCliError>;

    /// Cleanly closes the SSH connection.
    async fn disconnect(&self) -> Result<(), SshCliError>;
}

#[cfg(test)]
/// SSH client mocks used in unit tests.
pub mod mocks {
    use super::*;
    use mockall::mock;

    mock! {
        pub SshClient {}

    #[async_trait]
    impl crate::ssh::client::SshClientTrait for SshClient {
            async fn connect(cfg: ConnectionConfig) -> Result<Box<Self>, SshCliError>;
            async fn run_command(&mut self, cmd: &str, max_chars: usize, stdin_data: Option<Vec<u8>>) -> Result<ExecutionOutput, SshCliError>;
            async fn upload(&self, local: &Path, remote: &Path) -> Result<TransferResult, SshCliError>;
            async fn download(&self, remote: &Path, local: &Path) -> Result<TransferResult, SshCliError>;
            async fn open_tunnel_channel(
                &self,
                remote_host: &str,
                remote_port: u16,
                origin_addr: &str,
                origin_port: u16,
            ) -> Result<Box<dyn TunnelChannel>, SshCliError>;
            async fn disconnect(&self) -> Result<(), SshCliError>;
        }
    }
}

// =========================================================================
// REAL SSH implementation (`ssh-real` feature).
// =========================================================================


#[cfg(feature = "ssh-real")]
#[path = "client_real.rs"]
mod real;

/// Real SSH client backed by `russh` (default feature `ssh-real`).
#[cfg(feature = "ssh-real")]
#[cfg_attr(docsrs, doc(cfg(feature = "ssh-real")))]
pub use real::{SshClient, ClientHandler};

#[cfg(not(feature = "ssh-real"))]
#[path = "client_stub.rs"]
mod stub;

/// Stub client when `ssh-real` is disabled.
#[cfg(not(feature = "ssh-real"))]
#[cfg_attr(docsrs, doc(cfg(not(feature = "ssh-real"))))]
pub use stub::SshClient;

// =========================================================================
// Unit tests (no network, no feature gate).
// =========================================================================

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
