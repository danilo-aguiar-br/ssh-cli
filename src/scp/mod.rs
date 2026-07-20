// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! File transfer via SCP over SSH (one-shot).
//!
//! Wrapper around [`SshClient`] `upload` and `download` methods.
//! Regular files only (no `-r` / no SFTP subsystem).
//!
//! # Workload classification
//!
//! **I/O-bound** (network + disk). Multi-host `--all` / `--hosts` uses
//! [`crate::concurrency::map_bounded`] via [`crate::vps::resolve_host_jobs`]
//! (one permit = one SSH **session**).
//!
//! **Multi-file (G-PAR-47):** one host, N files → **one** `connect`, serial
//! transfers on that session (auth RTT once; `&mut` client cannot safely fan-out
//! channels without redesign). Parallelism useful at **host** granularity.
//!
//! **Multi-host × multi-file (G-PAR-48):** outer `map_bounded` per host; inner
//! multi-file session reuse. Batch JSON even when `--hosts` has one name (G-PAR-36).

use crate::cli::ScpAction;
use crate::errors::SshCliError;
use crate::i18n::{self, Message};
use crate::output;
use crate::ssh::client::{SshClient, SshClientTrait};
use crate::vps;
use std::path::PathBuf;

mod batch;
mod multi_host;

use batch::{run_scp_multi_file_download, run_scp_multi_file_upload};
use multi_host::{
    run_scp_all_download, run_scp_all_upload, run_scp_multi_host_multi_file_download,
    run_scp_multi_host_multi_file_upload,
};

/// Runtime overrides for the `scp` subcommand (parity with exec).
///
/// G-SECDEV-02: secrets are [`secrecy::SecretString`] from the CLI boundary.
/// G-TYPE-18: `timeout` is refined [`TimeoutMs`].
#[derive(Debug, Default, Clone)]
pub struct ScpOptions {
    /// SSH password (already resolved from flag or stdin).
    pub password: Option<secrecy::SecretString>,
    /// Private key path.
    pub key: Option<String>,
    /// Key passphrase (already resolved).
    pub key_passphrase: Option<secrecy::SecretString>,
    /// Total connect+transfer timeout in ms (refined at CLI boundary).
    pub timeout: Option<crate::domain::TimeoutMs>,
    /// Replace divergent host key (global `--replace-host-key`).
    pub replace_host_key: bool,
    /// Emit success JSON (local flag or global format).
    pub json: bool,
    /// Use ssh-agent (G-SFTP-17 / G-SSH-04 parity). CLI/XDG only.
    pub use_agent: bool,
    /// Agent socket (Unix) or named pipe (Windows).
    pub agent_socket: Option<String>,
}

/// Per-host SCP outcome for multi-host batch output.
#[derive(Debug, Clone)]
pub struct HostScpResult {
    /// VPS name.
    pub name: String,
    /// Whether transfer succeeded.
    pub ok: bool,
    /// Bytes transferred when ok.
    pub bytes: Option<u64>,
    /// Duration ms when measured.
    pub duration_ms: Option<u64>,
    /// Effective local path (download may be host-suffixed).
    pub local: Option<String>,
    /// Error detail.
    pub error: Option<String>,
}

/// Runs the SCP subcommand (upload/download), single host, multi-file, or multi-host.
pub async fn run_scp(
    action: ScpAction,
    config_override: Option<PathBuf>,
    opts: ScpOptions,
) -> anyhow::Result<()> {
    if crate::signals::should_stop() {
        return Err(anyhow::anyhow!(i18n::t(Message::OperationCancelled)));
    }

    match action {
        ScpAction::Upload {
            all,
            hosts,
            target,
            ..
        } => {
            let plan = crate::cli::parse_scp_target(all, hosts, target)
                .map_err(SshCliError::InvalidArgument)?;
            match plan {
                crate::cli::ScpPathPlan::MultiFile {
                    vps,
                    sources,
                    dest_dir,
                } => {
                    return run_scp_multi_file_upload(
                        &vps,
                        sources,
                        &dest_dir,
                        config_override,
                        opts,
                    )
                    .await;
                }
                crate::cli::ScpPathPlan::MultiHostMultiFile {
                    selection,
                    sources,
                    dest_dir,
                } => {
                    return run_scp_multi_host_multi_file_upload(
                        &selection,
                        sources,
                        &dest_dir,
                        config_override,
                        opts,
                    )
                    .await;
                }
                crate::cli::ScpPathPlan::Single {
                    selection,
                    path_a: local,
                    path_b: remote,
                } => {
                    // GAP-SSH-SCP-001 / SCP-019: validate file local antes do connect.
                    if local.is_dir() {
                        return Err(SshCliError::InvalidArgument(i18n::t(
                            Message::ScpUploadFileOnly,
                        ))
                        .into());
                    }
                    if !local.is_file() {
                        return Err(
                            SshCliError::FileNotFound(local.display().to_string()).into()
                        );
                    }

                    if selection.is_batch() {
                        return run_scp_all_upload(
                            &selection,
                            &local,
                            &remote,
                            config_override,
                            opts,
                        )
                        .await;
                    }
                    let vps::HostSelection::Single(vps_name) = selection else {
                        // G-SEC-08: fail closed instead of panic on invariant slip.
                        return Err(SshCliError::InvalidArgument(
                            "internal: expected single-host selection for non-batch SCP".into(),
                        )
                        .into());
                    };
                    let vps_key = vps_name.as_str();

                    let mut record = vps::find_by_name(config_override.as_deref(), vps_key)?
                        .ok_or_else(|| SshCliError::VpsNotFound(vps_key.to_owned()))?;

                    apply_scp_options(&mut record, &opts);

                    let path = crate::vps::resolve_config_path(config_override.as_deref())?;
                    let cfg = crate::vps::build_connection_config(
                        &record,
                        Some(&path),
                        opts.replace_host_key,
                    );

                    let client: Box<dyn SshClientTrait> =
                        <SshClient as SshClientTrait>::connect(cfg).await?;
                    run_scp_upload_with_client(vps_key, &local, &remote, client, opts.json)
                        .await?;
                }
            }
        }
        ScpAction::Download {
            all,
            hosts,
            target,
            ..
        } => {
            let plan = crate::cli::parse_scp_target(all, hosts, target)
                .map_err(SshCliError::InvalidArgument)?;
            match plan {
                crate::cli::ScpPathPlan::MultiFile {
                    vps,
                    sources: remotes,
                    dest_dir: local_dir,
                } => {
                    return run_scp_multi_file_download(
                        &vps,
                        remotes,
                        &local_dir,
                        config_override,
                        opts,
                    )
                    .await;
                }
                crate::cli::ScpPathPlan::MultiHostMultiFile {
                    selection,
                    sources: remotes,
                    dest_dir: local_dir,
                } => {
                    return run_scp_multi_host_multi_file_download(
                        &selection,
                        remotes,
                        &local_dir,
                        config_override,
                        opts,
                    )
                    .await;
                }
                crate::cli::ScpPathPlan::Single {
                    selection,
                    path_a: remote,
                    path_b: local,
                } => {
                    if selection.is_batch() {
                        return run_scp_all_download(
                            &selection,
                            &remote,
                            &local,
                            config_override,
                            opts,
                        )
                        .await;
                    }
                    if local.is_dir() {
                        return Err(SshCliError::InvalidArgument(i18n::t(
                            Message::ScpDownloadLocalNotDirectory,
                        ))
                        .into());
                    }
                    let vps::HostSelection::Single(vps_name) = selection else {
                        // G-SEC-08: fail closed instead of panic on invariant slip.
                        return Err(SshCliError::InvalidArgument(
                            "internal: expected single-host selection for non-batch SCP".into(),
                        )
                        .into());
                    };
                    let vps_key = vps_name.as_str();

                    let mut record = vps::find_by_name(config_override.as_deref(), vps_key)?
                        .ok_or_else(|| SshCliError::VpsNotFound(vps_key.to_owned()))?;

                    apply_scp_options(&mut record, &opts);

                    let path = crate::vps::resolve_config_path(config_override.as_deref())?;
                    let cfg = crate::vps::build_connection_config(
                        &record,
                        Some(&path),
                        opts.replace_host_key,
                    );

                    let client: Box<dyn SshClientTrait> =
                        <SshClient as SshClientTrait>::connect(cfg).await?;
                    run_scp_download_with_client(
                        vps_key, &remote, &local, client, opts.json,
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}

/// G-PAR-51: reject directories / missing files via `tokio::fs` (async path).
pub(crate) fn apply_scp_options(record: &mut crate::vps::model::VpsRecord, opts: &ScpOptions) {
    // G-MEM-SCP: borrow opts (often behind Arc) and clone secrets into the record.
    // Prefer Arc fan-out over cloning ScpOptions per host.
    if let Some(ref pwd) = opts.password {
        record.password = pwd.clone();
    }
    if let Some(ref k) = opts.key {
        if let Ok(kp) = crate::domain::KeyPath::try_new(k.as_str()) {
            record.key_path = Some(kp);
        }
    }
    if let Some(ref kp) = opts.key_passphrase {
        record.key_passphrase = Some(kp.clone());
    }
    // G-TYPE-18: timeout already TimeoutMs at the options boundary.
    if let Some(t) = opts.timeout {
        record.timeout_ms = t;
    }
    // G-SFTP-17: agent parity with exec/sftp (CLI/XDG — not env store).
    if opts.use_agent {
        record.use_agent = true;
    }
    if let Some(ref sock) = opts.agent_socket {
        record.agent_socket = Some(sock.clone());
        record.use_agent = true;
    }
}

/// Testable SCP upload that accepts the client as a parameter.
pub async fn run_scp_upload_with_client(
    vps_name: &str,
    local: &std::path::Path,
    remote: &std::path::Path,
    client: Box<dyn SshClientTrait>,
    json: bool,
) -> anyhow::Result<()> {
    let result = client.upload(local, remote).await;
    let _ = client.disconnect().await;
    let result = result?;
    if json {
        output::print_transfer_json(
            "upload",
            vps_name,
            &local.display().to_string(),
            &remote.display().to_string(),
            result.bytes_transferred,
            result.duration_ms,
        )?;
    } else {
        output::print_success(&i18n::t(Message::ScpUploadCompleted {
            bytes: result.bytes_transferred,
            ms: result.duration_ms,
        }));
    }
    Ok(())
}

/// Testable SCP download that accepts the client as a parameter.
pub async fn run_scp_download_with_client(
    vps_name: &str,
    remote: &std::path::Path,
    local: &std::path::Path,
    client: Box<dyn SshClientTrait>,
    json: bool,
) -> anyhow::Result<()> {
    let result = client.download(remote, local).await;
    let _ = client.disconnect().await;
    let result = result?;
    if json {
        output::print_transfer_json(
            "download",
            vps_name,
            &local.display().to_string(),
            &remote.display().to_string(),
            result.bytes_transferred,
            result.duration_ms,
        )?;
    } else {
        output::print_success(&i18n::t(Message::ScpDownloadCompleted {
            bytes: result.bytes_transferred,
            ms: result.duration_ms,
        }));
    }
    Ok(())
}


#[cfg(test)]
#[path = "tests.rs"]
mod tests;
