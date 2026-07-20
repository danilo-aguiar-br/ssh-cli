// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SFTP-08: multi-host SFTP fan-out (map_bounded; 1 session per host).
#![forbid(unsafe_code)]

use super::{apply_sftp_options, SftpOptions};
use crate::constants::SFTP_FALLBACK_BASENAME;
use crate::errors::SshCliError;
use crate::output;
use crate::ssh::client::SshClient;
use crate::ssh::sftp_path::{ensure_local_under, validate_entry_name};
use crate::ssh::sftp_session;
use crate::vps;
use std::path::{Path, PathBuf};

/// Per-host SFTP transfer result (batch JSON).
#[derive(Debug, Clone)]
pub struct HostSftpResult {
    /// VPS name (or host:path label).
    pub name: String,
    /// Success flag.
    pub ok: bool,
    /// Bytes when ok.
    pub bytes: Option<u64>,
    /// Duration when measured.
    pub duration_ms: Option<u64>,
    /// Local path.
    pub local: Option<String>,
    /// Error detail.
    pub error: Option<String>,
}

fn finish_batch(
    direction: &str,
    results: Vec<crate::concurrency::IndexedResult<HostSftpResult>>,
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    let mut host_results = Vec::with_capacity(results.len());
    let mut failures = 0usize;
    for r in results {
        match r.outcome {
            Ok(h) => {
                if !h.ok {
                    failures += 1;
                }
                host_results.push(h);
            }
            Err(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
            Err(e) => {
                failures += 1;
                host_results.push(HostSftpResult {
                    name: format!("task-{}", r.index),
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    output::print_sftp_batch(direction, &host_results, limit, json)?;
    if failures > 0 {
        return Err(SshCliError::Config(format!(
            "{failures}/{} transfers failed multi-host sftp {direction}",
            host_results.len()
        ))
        .into());
    }
    Ok(())
}

pub(crate) async fn run_sftp_all_upload(
    selection: &vps::HostSelection,
    local: &Path,
    remote: &str,
    config_override: Option<PathBuf>,
    opts: SftpOptions,
) -> anyhow::Result<()> {
    let path = vps::resolve_config_path(config_override.as_deref())?;
    let file = vps::load(&path)?;
    let jobs = vps::resolve_host_jobs(selection, &file)?;
    let limit = crate::concurrency::effective_limit();
    let local_owned = local.to_path_buf();
    let remote_owned = remote.to_owned();
    let path_c = path.clone();
    let replace = opts.replace_host_key;
    let recursive = opts.recursive;
    let json = opts.json;
    let opts_arc = std::sync::Arc::new(opts);

    let results = crate::concurrency::map_bounded(jobs, limit, move |(name, mut record)| {
        let opts = opts_arc.clone();
        let local_owned = local_owned.clone();
        let remote_owned = remote_owned.clone();
        let path_c = path_c.clone();
        async move {
            if crate::signals::should_stop() {
                return HostSftpResult {
                    name,
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: Some(local_owned.display().to_string()),
                    error: Some("operation cancelled by signal".into()),
                };
            }
            apply_sftp_options(&mut record, opts.as_ref());
            let cfg = vps::build_connection_config(&record, Some(&path_c), replace);
            match SshClient::connect(cfg).await {
                Ok(client) => {
                    let result = if recursive {
                        client.sftp_upload_tree(&local_owned, &remote_owned).await
                    } else {
                        client.sftp_upload(&local_owned, &remote_owned).await
                    };
                    let _ = client.disconnect().await;
                    match result {
                        Ok(t) => HostSftpResult {
                            name,
                            ok: true,
                            bytes: Some(t.bytes_transferred),
                            duration_ms: Some(t.duration_ms),
                            local: Some(local_owned.display().to_string()),
                            error: None,
                        },
                        Err(e) => HostSftpResult {
                            name,
                            ok: false,
                            bytes: None,
                            duration_ms: None,
                            local: Some(local_owned.display().to_string()),
                            error: Some(e.to_string()),
                        },
                    }
                }
                Err(e) => HostSftpResult {
                    name,
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: Some(local_owned.display().to_string()),
                    error: Some(e.to_string()),
                },
            }
        }
    })
    .await;

    finish_batch("upload", results, limit, json)
}

pub(crate) async fn run_sftp_all_download(
    selection: &vps::HostSelection,
    remote: &str,
    local: &Path,
    config_override: Option<PathBuf>,
    opts: SftpOptions,
) -> anyhow::Result<()> {
    let path = vps::resolve_config_path(config_override.as_deref())?;
    let file = vps::load(&path)?;
    let jobs = vps::resolve_host_jobs(selection, &file)?;
    let limit = crate::concurrency::effective_limit();
    let remote_owned = remote.to_owned();
    let local_owned = local.to_path_buf();
    let path_c = path.clone();
    let replace = opts.replace_host_key;
    let recursive = opts.recursive;
    let json = opts.json;
    let opts_arc = std::sync::Arc::new(opts);

    let results = crate::concurrency::map_bounded(jobs, limit, move |(name, mut record)| {
        let opts = opts_arc.clone();
        let remote_owned = remote_owned.clone();
        let local_base = local_owned.clone();
        let path_c = path_c.clone();
        async move {
            // Per-host local path to avoid collisions.
            let local_path = if recursive || local_base.is_dir() {
                local_base.join(&name)
            } else {
                let stem = local_base
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "download".into());
                let ext = local_base
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default();
                local_base.with_file_name(format!("{stem}.{name}{ext}"))
            };
            if crate::signals::should_stop() {
                return HostSftpResult {
                    name,
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: Some(local_path.display().to_string()),
                    error: Some("operation cancelled by signal".into()),
                };
            }
            apply_sftp_options(&mut record, opts.as_ref());
            let cfg = vps::build_connection_config(&record, Some(&path_c), replace);
            match SshClient::connect(cfg).await {
                Ok(client) => {
                    let result = if recursive {
                        client
                            .sftp_download_tree(&remote_owned, &local_path)
                            .await
                    } else {
                        client.sftp_download(&remote_owned, &local_path).await
                    };
                    let _ = client.disconnect().await;
                    match result {
                        Ok(t) => HostSftpResult {
                            name,
                            ok: true,
                            bytes: Some(t.bytes_transferred),
                            duration_ms: Some(t.duration_ms),
                            local: Some(local_path.display().to_string()),
                            error: None,
                        },
                        Err(e) => HostSftpResult {
                            name,
                            ok: false,
                            bytes: None,
                            duration_ms: None,
                            local: Some(local_path.display().to_string()),
                            error: Some(e.to_string()),
                        },
                    }
                }
                Err(e) => HostSftpResult {
                    name,
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: Some(local_path.display().to_string()),
                    error: Some(e.to_string()),
                },
            }
        }
    })
    .await;

    finish_batch("download", results, limit, json)
}

pub(crate) async fn run_sftp_multi_host_multi_file_upload(
    selection: &vps::HostSelection,
    sources: Vec<PathBuf>,
    dest_dir: &str,
    config_override: Option<PathBuf>,
    opts: SftpOptions,
) -> anyhow::Result<()> {
    let path = vps::resolve_config_path(config_override.as_deref())?;
    let file = vps::load(&path)?;
    let jobs = vps::resolve_host_jobs(selection, &file)?;
    let limit = crate::concurrency::effective_limit();
    let dest = dest_dir.to_owned();
    let path_c = path.clone();
    let replace = opts.replace_host_key;
    let json = opts.json;
    let opts_arc = std::sync::Arc::new(opts);
    let sources_arc = std::sync::Arc::new(sources);

    let results = crate::concurrency::map_bounded(jobs, limit, move |(name, mut record)| {
        let opts = opts_arc.clone();
        let sources = sources_arc.clone();
        let dest = dest.clone();
        let path_c = path_c.clone();
        async move {
            apply_sftp_options(&mut record, opts.as_ref());
            let cfg = vps::build_connection_config(&record, Some(&path_c), replace);
            match SshClient::connect(cfg).await {
                Ok(client) => {
                    let start = std::time::Instant::now();
                    let timeout_ms = client.timeout_ms();
                    let outcome = sftp_session::under_timeout(timeout_ms, async {
                        let sftp = client.open_sftp().await?;
                        let mut bytes = 0_u64;
                        for src in sources.iter() {
                            let name = src
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| SFTP_FALLBACK_BASENAME.to_owned());
                            validate_entry_name(&name)?;
                            let remote = crate::ssh::sftp_path::join_remote(&dest, &name);
                            let r = sftp_session::upload_file(&sftp, src, &remote).await?;
                            bytes = bytes.saturating_add(r.bytes_transferred);
                        }
                        sftp_session::close_sftp(&sftp).await;
                        Ok::<_, SshCliError>(bytes)
                    })
                    .await;
                    let _ = client.disconnect().await;
                    match outcome {
                        Ok(bytes) => HostSftpResult {
                            name,
                            ok: true,
                            bytes: Some(bytes),
                            duration_ms: Some(
                                u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                            ),
                            local: None,
                            error: None,
                        },
                        Err(e) => HostSftpResult {
                            name,
                            ok: false,
                            bytes: None,
                            duration_ms: None,
                            local: None,
                            error: Some(e.to_string()),
                        },
                    }
                }
                Err(e) => HostSftpResult {
                    name,
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: None,
                    error: Some(e.to_string()),
                },
            }
        }
    })
    .await;

    finish_batch("upload", results, limit, json)
}

pub(crate) async fn run_sftp_multi_host_multi_file_download(
    selection: &vps::HostSelection,
    remotes: Vec<PathBuf>,
    local_dir: &Path,
    config_override: Option<PathBuf>,
    opts: SftpOptions,
) -> anyhow::Result<()> {
    let path = vps::resolve_config_path(config_override.as_deref())?;
    let file = vps::load(&path)?;
    let jobs = vps::resolve_host_jobs(selection, &file)?;
    let limit = crate::concurrency::effective_limit();
    let local_base = local_dir.to_path_buf();
    let path_c = path.clone();
    let replace = opts.replace_host_key;
    let json = opts.json;
    let opts_arc = std::sync::Arc::new(opts);
    let remotes_arc = std::sync::Arc::new(remotes);

    let results = crate::concurrency::map_bounded(jobs, limit, move |(name, mut record)| {
        let opts = opts_arc.clone();
        let remotes = remotes_arc.clone();
        let host_dir = local_base.join(&name);
        let path_c = path_c.clone();
        async move {
            apply_sftp_options(&mut record, opts.as_ref());
            let cfg = vps::build_connection_config(&record, Some(&path_c), replace);
            match SshClient::connect(cfg).await {
                Ok(client) => {
                    let start = std::time::Instant::now();
                    let timeout_ms = client.timeout_ms();
                    let host_root = host_dir.clone();
                    let outcome = sftp_session::under_timeout(timeout_ms, async {
                        tokio::fs::create_dir_all(&host_dir)
                            .await
                            .map_err(SshCliError::Io)?;
                        let sftp = client.open_sftp().await?;
                        let mut bytes = 0_u64;
                        for remote_p in remotes.iter() {
                            let remote = remote_p.to_string_lossy().into_owned();
                            let fname = remote_p
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| SFTP_FALLBACK_BASENAME.to_owned());
                            validate_entry_name(&fname)?;
                            let local = host_dir.join(&fname);
                            ensure_local_under(&host_root, &local)?;
                            let r =
                                sftp_session::download_file(&sftp, &remote, &local).await?;
                            bytes = bytes.saturating_add(r.bytes_transferred);
                        }
                        sftp_session::close_sftp(&sftp).await;
                        Ok::<_, SshCliError>(bytes)
                    })
                    .await;
                    let _ = client.disconnect().await;
                    match outcome {
                        Ok(bytes) => HostSftpResult {
                            name,
                            ok: true,
                            bytes: Some(bytes),
                            duration_ms: Some(
                                u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                            ),
                            local: Some(host_dir.display().to_string()),
                            error: None,
                        },
                        Err(e) => HostSftpResult {
                            name,
                            ok: false,
                            bytes: None,
                            duration_ms: None,
                            local: Some(host_dir.display().to_string()),
                            error: Some(e.to_string()),
                        },
                    }
                }
                Err(e) => HostSftpResult {
                    name,
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: Some(host_dir.display().to_string()),
                    error: Some(e.to_string()),
                },
            }
        }
    })
    .await;

    finish_batch("download", results, limit, json)
}
