// SPDX-License-Identifier: MIT OR Apache-2.0
//! SFTP subcommand dispatch (A7 split).
#![forbid(unsafe_code)]
#![allow(unused_imports)]

use super::*;

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
                    // B10: `--timeout` is a per-operation budget. Wrapping the whole
                    // loop made the deadline shrink with every extra file, so a
                    // healthy 40-file batch failed while each single file was fast.
                    let result = async {
                        let sftp =
                            sftp_session::under_timeout(timeout_ms, client.open_sftp()).await?;
                        let mut bytes = 0_u64;
                        let mut err: Option<SshCliError> = None;
                        for src in &sources {
                            let name = src
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| SFTP_FALLBACK_BASENAME.to_owned());
                            if let Err(e) = validate_entry_name(&name) {
                                err = Some(e);
                                break;
                            }
                            let remote = crate::ssh::sftp_path::join_remote(&dest, &name);
                            let one = sftp_session::under_timeout(
                                timeout_ms,
                                sftp_session::upload_file(&sftp, src, &remote),
                            )
                            .await;
                            match one {
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
                    }
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
                            ..Default::default()
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
                    // B10: per-file deadline (see the upload path above).
                    let result = async {
                        tokio::fs::create_dir_all(&local_dir)
                            .await
                            .map_err(SshCliError::Io)?;
                        let sftp =
                            sftp_session::under_timeout(timeout_ms, client.open_sftp()).await?;
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
                            let one = sftp_session::under_timeout(
                                timeout_ms,
                                sftp_session::download_file(&sftp, &remote, &local),
                            )
                            .await;
                            match one {
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
                    }
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
                            ..Default::default()
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
            let client = connect_client(&vps_name, config_override.as_deref(), &opts).await?;
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
            let client = connect_client(&vps_name, config_override.as_deref(), &opts).await?;
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
            // C2: emitted before the SSH handshake. Confirming the directory exists
            // would cost a full connect + auth, and a preview that opens an
            // authenticated session is no longer side-effect free — it shows up in
            // the server's auth log exactly like a real run.
            if crate::cli::dry_run_stop(
                "sftp-rmdir",
                &[
                    ("vps", serde_json::json!(vps_name)),
                    ("remote", serde_json::json!(remote)),
                ],
            )? {
                return Ok(());
            }
            let start = Instant::now();
            let client = connect_client(&vps_name, config_override.as_deref(), &opts).await?;
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
            // C2: see `sftp rmdir` — the preview stays offline on purpose.
            if crate::cli::dry_run_stop(
                "sftp-rm",
                &[
                    ("vps", serde_json::json!(vps_name)),
                    ("remote", serde_json::json!(remote)),
                ],
            )? {
                return Ok(());
            }
            let start = Instant::now();
            let client = connect_client(&vps_name, config_override.as_deref(), &opts).await?;
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
            let client = connect_client(&vps_name, config_override.as_deref(), &opts).await?;
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
            let client = connect_client(&vps_name, config_override.as_deref(), &opts).await?;
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
