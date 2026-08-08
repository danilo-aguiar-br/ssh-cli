// SPDX-License-Identifier: MIT OR Apache-2.0
// G-ERR-12 / G10: stub client when feature `ssh-real` is disabled.
// Loaded as `mod stub` via `#[path]` from `client.rs` — types live at module root.
#![forbid(unsafe_code)]

use super::{ConnectionConfig, ExecutionOutput, TransferResult};
use crate::errors::SshCliError;
use crate::ssh::client::SshClientTrait;
use async_trait::async_trait;
use std::path::Path;

/// Stub when `ssh-real` is disabled: always returns [`SshCliError::ConnectionFailed`].
#[derive(Debug)]
pub struct SshClient;

#[async_trait]
impl SshClientTrait for SshClient {
    async fn connect(_cfg: ConnectionConfig) -> Result<Box<Self>, SshCliError> {
        Err(SshCliError::ConnectionFailed(
            "feature `ssh-real` is disabled; rebuild with --features ssh-real".to_string(),
        ))
    }

    async fn run_command(
        &mut self,
        _cmd: &str,
        _max_chars: usize,
        _stdin_data: Option<Vec<u8>>,
    ) -> Result<ExecutionOutput, SshCliError> {
        Err(SshCliError::channel_msg(
            "stub without russh: feature `ssh-real` disabled",
        ))
    }

    async fn upload(&self, _local: &Path, _remote: &Path) -> Result<TransferResult, SshCliError> {
        Err(SshCliError::channel_msg(
            "stub without russh: feature `ssh-real` disabled",
        ))
    }

    async fn download(&self, _remote: &Path, _local: &Path) -> Result<TransferResult, SshCliError> {
        Err(SshCliError::channel_msg(
            "stub without russh: feature `ssh-real` disabled",
        ))
    }

    async fn open_tunnel_channel(
        &self,
        _remote_host: &str,
        _remote_port: u16,
        _origin_address: &str,
        _origin_port: u16,
    ) -> Result<Box<dyn super::TunnelChannel>, SshCliError> {
        Err(SshCliError::channel_msg(
            "stub without russh: feature `ssh-real` disabled",
        ))
    }

    async fn disconnect(&self) -> Result<(), SshCliError> {
        Ok(())
    }
}
