// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SFTP: SFTP CLI surface (one-shot; stream transfers; no full-file heap).
#![forbid(unsafe_code)]
//! SFTP subsystem operations over SSH (upload/download/ls/mkdir/rm/stat/rename).
//!
//! Complements SCP (regular-file wire). SFTP adds directory trees and FS ops.
//! Multi-host fan-out uses [`crate::concurrency::map_bounded`]. Multi-file on one
//! host reuses **one** SFTP session (G-SFTP-19).

use crate::cli::SftpAction;
use crate::constants::SFTP_FALLBACK_BASENAME;
use crate::errors::SshCliError;
use crate::i18n::{self, Message};
use crate::output;
use crate::ssh::client::{SshClient, TransferResult};
use crate::ssh::sftp_path::{ensure_local_under, validate_entry_name};
use crate::ssh::sftp_session;
use crate::ssh::sftp_types::{SftpListEntry, SftpStat};
use crate::vps;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub(crate) mod batch;

/// Runtime overrides for the `sftp` subcommand (parity with scp + agent G-SFTP-18).
#[derive(Debug, Default, Clone)]
pub struct SftpOptions {
    /// SSH password (resolved).
    pub password: Option<secrecy::SecretString>,
    /// Private key path.
    pub key: Option<String>,
    /// Key passphrase (resolved).
    pub key_passphrase: Option<secrecy::SecretString>,
    /// Total connect+op timeout ms.
    pub timeout: Option<crate::domain::TimeoutMs>,
    /// Replace divergent host key.
    pub replace_host_key: bool,
    /// Emit JSON success envelopes.
    pub json: bool,
    /// Use ssh-agent (CLI/XDG only).
    pub use_agent: bool,
    /// Agent socket / named pipe path.
    pub agent_socket: Option<String>,
    /// Recursive tree transfer.
    pub recursive: bool,
}

/// Applies CLI overrides onto a VPS record (incl. agent — G-SFTP-18).
pub(crate) fn apply_sftp_options(record: &mut crate::vps::model::VpsRecord, opts: &SftpOptions) {
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
    if let Some(t) = opts.timeout {
        record.timeout_ms = t;
    }
    if opts.use_agent {
        record.use_agent = true;
    }
    if let Some(ref sock) = opts.agent_socket {
        record.agent_socket = Some(sock.clone());
        record.use_agent = true;
    }
}

async fn connect_client(
    vps_key: &str,
    config_override: Option<&std::path::Path>,
    opts: &SftpOptions,
) -> anyhow::Result<SshClient> {
    let mut record = vps::find_by_name(config_override, vps_key)?
        .ok_or_else(|| SshCliError::VpsNotFound(vps_key.to_owned()))?;
    apply_sftp_options(&mut record, opts);
    let path = vps::resolve_config_path(config_override)?;
    let cfg = vps::build_connection_config(&record, Some(&path), opts.replace_host_key);
    let client = SshClient::connect(cfg).await?;
    Ok(client)
}

fn remote_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// Runs the `sftp` subcommand.
pub async fn run_sftp(
    action: SftpAction,
    config_override: Option<PathBuf>,
    opts: SftpOptions,
) -> anyhow::Result<()> {
    if crate::signals::should_stop() {
        return Err(anyhow::anyhow!(i18n::t(Message::OperationCancelled)));
    }

    match action {
        SftpAction::Upload {
            all,
            hosts,
            target,
            recursive,
            ..
        } => {
            let mut opts = opts;
            opts.recursive = recursive;
            let plan = crate::cli::parse_scp_target(all, hosts, target)
                .map_err(SshCliError::InvalidArgument)?;
            match plan {
                crate::cli::ScpPathPlan::Single {
                    selection,
                    path_a: local,
                    path_b: remote,
                } => {
                    if selection.is_batch() {
                        return batch::run_sftp_all_upload(
                            &selection,
                            &local,
                            &remote_str(&remote),
                            config_override,
                            opts,
                        )
                        .await;
                    }
                    let vps::HostSelection::Single(vps_name) = selection else {
                        return Err(SshCliError::InvalidArgument(
                            "internal: expected single-host sftp upload".into(),
                        )
                        .into());
                    };
                    let client =
                        connect_client(vps_name.as_str(), config_override.as_deref(), &opts)
                            .await?;
                    let remote = remote_str(&remote);
                    let result = if opts.recursive {
                        client.sftp_upload_tree(&local, &remote).await
                    } else {
                        client.sftp_upload(&local, &remote).await
                    };
                    let _ = client.disconnect().await;
                    emit_transfer(
                        "upload",
                        vps_name.as_str(),
                        &local.display().to_string(),
                        &remote,
                        result?,
                        opts.json,
                        opts.recursive,
                    )?;
                }
                crate::cli::ScpPathPlan::MultiFile {
                    vps,
                    sources,
                    dest_dir,
                } => {
                    if opts.recursive {
                        return Err(SshCliError::InvalidArgument(
                            "sftp multi-file upload does not combine with --recursive".into(),
                        )
                        .into());
                    }
                    let client =
                        connect_client(vps.as_str(), config_override.as_deref(), &opts).await?;
                    let dest = remote_str(&dest_dir);
                    let local_label = sources
                        .first()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default();
                    let start = Instant::now();
                    let timeout_ms = client.timeout_ms();
                    let result = sftp_session::under_timeout(timeout_ms, async {
                        let sftp = client.open_sftp().await?;
                        let mut bytes = 0_u64;
                        let mut err: Option<SshCliError> = None;
                        for src in &sources {
                            let name = src
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| SFTP_FALLBACK_BASENAME.to_owned());
                            validate_entry_name(&name)?;
                            let remote = crate::ssh::sftp_path::join_remote(&dest, &name);
                            match sftp_session::upload_file(&sftp, src, &remote).await {
                                Ok(r) => bytes = bytes.saturating_add(r.bytes_transferred),
                                Err(e) => {
                                    err = Some(e);
                                    break;
                                }
                            }
                        }
                        sftp_session::close_sftp(&sftp).await;
                        if let Some(e) = err {
                            return Err(e);
                        }
                        Ok(bytes)
                    })
                    .await;
                    let _ = client.disconnect().await;
                    let bytes = result?;
                    emit_transfer(
                        "upload",
                        vps.as_str(),
                        &local_label,
                        &dest,
                        TransferResult {
                            bytes_transferred: bytes,
                            duration_ms: u64::try_from(start.elapsed().as_millis())
                                .unwrap_or(u64::MAX),
                        },
                        opts.json,
                        false,
                    )?;
                }
                crate::cli::ScpPathPlan::MultiHostMultiFile {
                    selection,
                    sources,
                    dest_dir,
                } => {
                    return batch::run_sftp_multi_host_multi_file_upload(
                        &selection,
                        sources,
                        &remote_str(&dest_dir),
                        config_override,
                        opts,
                    )
                    .await;
                }
            }
        }
        SftpAction::Download {
            all,
            hosts,
            target,
            recursive,
            ..
        } => {
            let mut opts = opts;
            opts.recursive = recursive;
            let plan = crate::cli::parse_scp_target(all, hosts, target)
                .map_err(SshCliError::InvalidArgument)?;
            match plan {
                crate::cli::ScpPathPlan::Single {
                    selection,
                    path_a: remote,
                    path_b: local,
                } => {
                    if selection.is_batch() {
                        return batch::run_sftp_all_download(
                            &selection,
                            &remote_str(&remote),
                            &local,
                            config_override,
                            opts,
                        )
                        .await;
                    }
                    let vps::HostSelection::Single(vps_name) = selection else {
                        return Err(SshCliError::InvalidArgument(
                            "internal: expected single-host sftp download".into(),
                        )
                        .into());
                    };
                    let client =
                        connect_client(vps_name.as_str(), config_override.as_deref(), &opts)
                            .await?;
                    let remote = remote_str(&remote);
                    let result = if opts.recursive {
                        client.sftp_download_tree(&remote, &local).await
                    } else {
                        client.sftp_download(&remote, &local).await
                    };
                    let _ = client.disconnect().await;
                    emit_transfer(
                        "download",
                        vps_name.as_str(),
                        &local.display().to_string(),
                        &remote,
                        result?,
                        opts.json,
                        opts.recursive,
                    )?;
                }
                crate::cli::ScpPathPlan::MultiFile {
                    vps,
                    sources: remotes,
                    dest_dir: local_dir,
                } => {
                    if opts.recursive {
                        return Err(SshCliError::InvalidArgument(
                            "sftp multi-file download does not combine with --recursive".into(),
                        )
                        .into());
                    }
                    let client =
                        connect_client(vps.as_str(), config_override.as_deref(), &opts).await?;
                    let local_label = local_dir.display().to_string();
                    let remote_label = remotes
                        .first()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default();
                    let start = Instant::now();
                    let timeout_ms = client.timeout_ms();
                    let local_root = local_dir.clone();
                    let result = sftp_session::under_timeout(timeout_ms, async {
                        tokio::fs::create_dir_all(&local_dir)
                            .await
                            .map_err(SshCliError::Io)?;
                        let sftp = client.open_sftp().await?;
                        let mut bytes = 0_u64;
                        let mut err: Option<SshCliError> = None;
                        for remote_p in &remotes {
                            let remote = remote_str(remote_p);
                            let name = remote_p
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| SFTP_FALLBACK_BASENAME.to_owned());
                            if let Err(e) = validate_entry_name(&name) {
                                err = Some(e);
                                break;
                            }
                            let local = local_dir.join(&name);
                            if let Err(e) = ensure_local_under(&local_root, &local) {
                                err = Some(e);
                                break;
                            }
                            match sftp_session::download_file(&sftp, &remote, &local).await {
                                Ok(r) => bytes = bytes.saturating_add(r.bytes_transferred),
                                Err(e) => {
                                    err = Some(e);
                                    break;
                                }
                            }
                        }
                        sftp_session::close_sftp(&sftp).await;
                        if let Some(e) = err {
                            return Err(e);
                        }
                        Ok(bytes)
                    })
                    .await;
                    let _ = client.disconnect().await;
                    let bytes = result?;
                    emit_transfer(
                        "download",
                        vps.as_str(),
                        &local_label,
                        &remote_label,
                        TransferResult {
                            bytes_transferred: bytes,
                            duration_ms: u64::try_from(start.elapsed().as_millis())
                                .unwrap_or(u64::MAX),
                        },
                        opts.json,
                        false,
                    )?;
                }
                crate::cli::ScpPathPlan::MultiHostMultiFile {
                    selection,
                    sources: remotes,
                    dest_dir: local_dir,
                } => {
                    return batch::run_sftp_multi_host_multi_file_download(
                        &selection,
                        remotes,
                        &local_dir,
                        config_override,
                        opts,
                    )
                    .await;
                }
            }
        }
        SftpAction::Ls {
            vps_name,
            remote,
            json: json_local,
            ..
        } => {
            let json = opts.json || json_local;
            let client =
                connect_client(&vps_name, config_override.as_deref(), &opts).await?;
            let timeout_ms = client.timeout_ms();
            let entries = sftp_session::under_timeout(timeout_ms, async {
                let sftp = client.open_sftp().await?;
                let entries = sftp_session::list_dir(&sftp, &remote).await;
                sftp_session::close_sftp(&sftp).await;
                entries
            })
            .await;
            let _ = client.disconnect().await;
            emit_list(&vps_name, &remote, &entries?, json)?;
        }
        SftpAction::Mkdir {
            vps_name,
            remote,
            json: json_local,
            ..
        } => {
            let json = opts.json || json_local;
            let start = Instant::now();
            let client =
                connect_client(&vps_name, config_override.as_deref(), &opts).await?;
            let timeout_ms = client.timeout_ms();
            let result = sftp_session::under_timeout(timeout_ms, async {
                let sftp = client.open_sftp().await?;
                let result = sftp_session::mkdir(&sftp, &remote).await;
                sftp_session::close_sftp(&sftp).await;
                result
            })
            .await;
            let _ = client.disconnect().await;
            result?;
            emit_fs_op(
                "mkdir",
                &vps_name,
                &remote,
                None,
                u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                json,
            )?;
        }
        SftpAction::Rmdir {
            vps_name,
            remote,
            json: json_local,
            ..
        } => {
            let json = opts.json || json_local;
            let start = Instant::now();
            let client =
                connect_client(&vps_name, config_override.as_deref(), &opts).await?;
            let timeout_ms = client.timeout_ms();
            let result = sftp_session::under_timeout(timeout_ms, async {
                let sftp = client.open_sftp().await?;
                let result = sftp_session::rmdir(&sftp, &remote).await;
                sftp_session::close_sftp(&sftp).await;
                result
            })
            .await;
            let _ = client.disconnect().await;
            result?;
            emit_fs_op(
                "rmdir",
                &vps_name,
                &remote,
                None,
                u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                json,
            )?;
        }
        SftpAction::Rm {
            vps_name,
            remote,
            json: json_local,
            ..
        } => {
            let json = opts.json || json_local;
            let start = Instant::now();
            let client =
                connect_client(&vps_name, config_override.as_deref(), &opts).await?;
            let timeout_ms = client.timeout_ms();
            let result = sftp_session::under_timeout(timeout_ms, async {
                let sftp = client.open_sftp().await?;
                let result = sftp_session::rm(&sftp, &remote).await;
                sftp_session::close_sftp(&sftp).await;
                result
            })
            .await;
            let _ = client.disconnect().await;
            result?;
            emit_fs_op(
                "rm",
                &vps_name,
                &remote,
                None,
                u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                json,
            )?;
        }
        SftpAction::Stat {
            vps_name,
            remote,
            json: json_local,
            ..
        } => {
            let json = opts.json || json_local;
            let client =
                connect_client(&vps_name, config_override.as_deref(), &opts).await?;
            let timeout_ms = client.timeout_ms();
            let st = sftp_session::under_timeout(timeout_ms, async {
                let sftp = client.open_sftp().await?;
                let st = sftp_session::stat(&sftp, &remote).await;
                sftp_session::close_sftp(&sftp).await;
                st
            })
            .await;
            let _ = client.disconnect().await;
            emit_stat(&vps_name, &st?, json)?;
        }
        SftpAction::Rename {
            vps_name,
            from,
            to,
            json: json_local,
            ..
        } => {
            let json = opts.json || json_local;
            let start = Instant::now();
            let client =
                connect_client(&vps_name, config_override.as_deref(), &opts).await?;
            let timeout_ms = client.timeout_ms();
            let result = sftp_session::under_timeout(timeout_ms, async {
                let sftp = client.open_sftp().await?;
                let result = sftp_session::rename(&sftp, &from, &to).await;
                sftp_session::close_sftp(&sftp).await;
                result
            })
            .await;
            let _ = client.disconnect().await;
            result?;
            emit_fs_op(
                "rename",
                &vps_name,
                &from,
                Some(to.as_str()),
                u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                json,
            )?;
        }
    }
    Ok(())
}

fn emit_transfer(
    direction: &str,
    vps: &str,
    local: &str,
    remote: &str,
    result: TransferResult,
    json: bool,
    recursive: bool,
) -> anyhow::Result<()> {
    if json {
        output::print_sftp_transfer_json(
            direction,
            vps,
            local,
            remote,
            result.bytes_transferred,
            result.duration_ms,
            recursive,
        )?;
    } else {
        let msg = if direction == "upload" {
            Message::SftpUploadCompleted {
                bytes: result.bytes_transferred,
                ms: result.duration_ms,
            }
        } else {
            Message::SftpDownloadCompleted {
                bytes: result.bytes_transferred,
                ms: result.duration_ms,
            }
        };
        output::print_success(&i18n::t(msg));
    }
    Ok(())
}

fn emit_list(vps: &str, path: &str, entries: &[SftpListEntry], json: bool) -> anyhow::Result<()> {
    if json {
        output::print_sftp_list_json(vps, path, entries)?;
    } else {
        for e in entries {
            println!(
                "{}\t{}\t{}",
                e.kind,
                e.size.map(|s| s.to_string()).unwrap_or_else(|| "-".into()),
                e.path
            );
        }
    }
    Ok(())
}

fn emit_stat(vps: &str, st: &SftpStat, json: bool) -> anyhow::Result<()> {
    if json {
        output::print_sftp_stat_json(vps, st)?;
    } else {
        println!(
            "path={} kind={} size={} mode={:?} mtime={:?}",
            st.path,
            st.kind,
            st.size.map(|s| s.to_string()).unwrap_or_else(|| "-".into()),
            st.mode,
            st.mtime
        );
    }
    Ok(())
}

fn emit_fs_op(
    op: &str,
    vps: &str,
    path: &str,
    to: Option<&str>,
    duration_ms: u64,
    json: bool,
) -> anyhow::Result<()> {
    if json {
        output::print_sftp_fs_op_json(op, vps, path, to, duration_ms)?;
    } else {
        match to {
            Some(t) => output::print_success(&format!("sftp {op} ok: {path} -> {t} ({duration_ms}ms)")),
            None => output::print_success(&format!("sftp {op} ok: {path} ({duration_ms}ms)")),
        }
    }
    Ok(())
}


