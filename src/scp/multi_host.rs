// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: multi-host SCP fan-out (extracted from scp/batch for line budget).
#![forbid(unsafe_code)]
//! Multi-host SCP paths using bounded concurrency.

use super::batch::{
    finish_scp_batch, finish_scp_nested_batch, multi_file_upload_on_session,
    validate_local_upload_sources,
};
use super::{apply_scp_options, HostScpResult, ScpOptions};
use crate::errors::SshCliError;
use crate::ssh::client::{SshClient, SshClientTrait};
use crate::vps;
use std::path::{Path, PathBuf};

pub(crate) async fn run_scp_multi_host_multi_file_upload(
    selection: &vps::HostSelection,
    sources: Vec<PathBuf>,
    remote_dir: &Path,
    config_override: Option<PathBuf>,
    opts: ScpOptions,
) -> anyhow::Result<()> {
    validate_local_upload_sources(&sources).await?;
    let path = vps::resolve_config_path(config_override.as_deref())?;
    let file = vps::load(&path)?;
    let jobs = vps::resolve_host_jobs(selection, &file)?;
    let limit = crate::concurrency::effective_limit();
    let remote_dir = remote_dir.to_path_buf();
    let path_c = path.clone();
    let replace = opts.replace_host_key;
    let sources_c = sources.clone();
    let json = opts.json;
    // G-MEM-SCP: share options across host tasks (Arc clone, not ScpOptions clone).
    let opts_arc = std::sync::Arc::new(opts);

    tracing::info!(
        hosts = jobs.len(),
        files = sources.len(),
        max_concurrency = limit,
        "multi-host multi-file scp upload (session reuse per host)"
    );

    let results = crate::concurrency::map_bounded(jobs, limit, move |(name, mut record)| {
        let opts = opts_arc.clone();
        let path_c = path_c.clone();
        let remote_dir = remote_dir.clone();
        let sources = sources_c.clone();
        async move {
            if crate::signals::should_stop() {
                return vec![HostScpResult {
                    name,
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: None,
                    error: Some("operation cancelled by signal".into()),
                }];
            }
            apply_scp_options(&mut record, opts.as_ref());
            let cfg = vps::build_connection_config(&record, Some(&path_c), replace);
            match <SshClient as SshClientTrait>::connect(cfg).await {
                Ok(client) => {
                    let out = multi_file_upload_on_session(
                        client.as_ref(),
                        &sources,
                        &remote_dir,
                        Some(&name),
                    )
                    .await;
                    let _ = client.disconnect().await;
                    out
                }
                Err(e) => {
                    vec![HostScpResult {
                        name,
                        ok: false,
                        bytes: None,
                        duration_ms: None,
                        local: None,
                        error: Some(e.to_string()),
                    }]
                }
            }
        }
    })
    .await;

    finish_scp_nested_batch("upload", results, limit, json)
}

/// G-PAR-48: multi-host × multi-file download — local paths host-suffixed under dest dir.
pub(crate) async fn run_scp_multi_host_multi_file_download(
    selection: &vps::HostSelection,
    remotes: Vec<PathBuf>,
    local_dir: &Path,
    config_override: Option<PathBuf>,
    opts: ScpOptions,
) -> anyhow::Result<()> {
    let dest_meta = tokio::fs::metadata(local_dir).await;
    match dest_meta {
        Ok(m) if m.is_file() => {
            return Err(SshCliError::InvalidArgument(
                "multi-file download destination must be a directory (not an existing file)"
                    .into(),
            )
            .into());
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tokio::fs::create_dir_all(local_dir)
                .await
                .map_err(SshCliError::Io)?;
        }
        Err(e) => return Err(SshCliError::Io(e).into()),
    }

    let path = vps::resolve_config_path(config_override.as_deref())?;
    let file = vps::load(&path)?;
    let jobs = vps::resolve_host_jobs(selection, &file)?;
    let limit = crate::concurrency::effective_limit();
    let local_dir = local_dir.to_path_buf();
    let path_c = path.clone();
    let replace = opts.replace_host_key;
    let remotes_c = remotes.clone();
    let json = opts.json;
    let opts_arc = std::sync::Arc::new(opts);

    tracing::info!(
        hosts = jobs.len(),
        files = remotes.len(),
        max_concurrency = limit,
        "multi-host multi-file scp download (session reuse per host)"
    );

    let results = crate::concurrency::map_bounded(jobs, limit, move |(name, mut record)| {
        let opts = opts_arc.clone();
        let path_c = path_c.clone();
        let local_dir = local_dir.clone();
        let remotes = remotes_c.clone();
        let host_safe = name.replace(['/', '\\'], "_");
        async move {
            if crate::signals::should_stop() {
                return vec![HostScpResult {
                    name,
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: None,
                    error: Some("operation cancelled by signal".into()),
                }];
            }
            apply_scp_options(&mut record, opts.as_ref());
            let cfg = vps::build_connection_config(&record, Some(&path_c), replace);
            let mut out = Vec::with_capacity(remotes.len());
            match <SshClient as SshClientTrait>::connect(cfg).await {
                Ok(client) => {
                    for remote in remotes {
                        let label = format!("{}:{}", name, remote.display());
                        if crate::signals::should_stop() {
                            out.push(HostScpResult {
                                name: label,
                                ok: false,
                                bytes: None,
                                duration_ms: None,
                                local: None,
                                error: Some("operation cancelled by signal".into()),
                            });
                            break;
                        }
                        let base = remote
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "file".into());
                        // Avoid collisions across hosts: dest/host_safe/basename
                        let host_subdir = local_dir.join(&host_safe);
                        if let Err(e) = tokio::fs::create_dir_all(&host_subdir).await {
                            out.push(HostScpResult {
                                name: label,
                                ok: false,
                                bytes: None,
                                duration_ms: None,
                                local: None,
                                error: Some(e.to_string()),
                            });
                            continue;
                        }
                        let local = host_subdir.join(&base);
                        match client.download(&remote, &local).await {
                            Ok(t) => out.push(HostScpResult {
                                name: label,
                                ok: true,
                                bytes: Some(t.bytes_transferred),
                                duration_ms: Some(t.duration_ms),
                                local: Some(local.display().to_string()),
                                error: None,
                            }),
                            Err(e) => out.push(HostScpResult {
                                name: label,
                                ok: false,
                                bytes: None,
                                duration_ms: None,
                                local: Some(local.display().to_string()),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    let _ = client.disconnect().await;
                }
                Err(e) => {
                    out.push(HostScpResult {
                        name,
                        ok: false,
                        bytes: None,
                        duration_ms: None,
                        local: None,
                        error: Some(e.to_string()),
                    });
                }
            }
            out
        }
    })
    .await;

    finish_scp_nested_batch("download", results, limit, json)
}

pub(crate) async fn run_scp_all_upload(
    selection: &vps::HostSelection,
    local: &Path,
    remote: &Path,
    config_override: Option<PathBuf>,
    opts: ScpOptions,
) -> anyhow::Result<()> {
    let path = vps::resolve_config_path(config_override.as_deref())?;
    let file = vps::load(&path)?;
    let jobs = vps::resolve_host_jobs(selection, &file)?;
    let limit = crate::concurrency::effective_limit();
    let local_owned = local.to_path_buf();
    let remote_owned = remote.to_path_buf();
    let path_c = path.clone();
    let replace = opts.replace_host_key;

    tracing::info!(
        hosts = jobs.len(),
        max_concurrency = limit,
        "multi-host scp upload fan-out"
    );

    let json = opts.json;
    let opts_arc = std::sync::Arc::new(opts);
    let results = crate::concurrency::map_bounded(jobs, limit, move |(name, mut record)| {
        let opts = opts_arc.clone();
        let local_owned = local_owned.clone();
        let remote_owned = remote_owned.clone();
        let path_c = path_c.clone();
        async move {
            if crate::signals::should_stop() {
                return HostScpResult {
                    name,
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: Some(local_owned.display().to_string()),
                    error: Some("operation cancelled by signal".into()),
                };
            }
            apply_scp_options(&mut record, opts.as_ref());
            let cfg = vps::build_connection_config(&record, Some(&path_c), replace);
            match <SshClient as SshClientTrait>::connect(cfg).await {
                Ok(client) => {
                    let result = client.upload(&local_owned, &remote_owned).await;
                    let _ = client.disconnect().await;
                    match result {
                        Ok(t) => HostScpResult {
                            name,
                            ok: true,
                            bytes: Some(t.bytes_transferred),
                            duration_ms: Some(t.duration_ms),
                            local: Some(local_owned.display().to_string()),
                            error: None,
                        },
                        Err(e) => HostScpResult {
                            name,
                            ok: false,
                            bytes: None,
                            duration_ms: None,
                            local: Some(local_owned.display().to_string()),
                            error: Some(e.to_string()),
                        },
                    }
                }
                Err(e) => HostScpResult {
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

    finish_scp_batch("upload", results, limit, json)
}

pub(crate) async fn run_scp_all_download(
    selection: &vps::HostSelection,
    remote: &Path,
    local_prefix: &Path,
    config_override: Option<PathBuf>,
    opts: ScpOptions,
) -> anyhow::Result<()> {
    let path = vps::resolve_config_path(config_override.as_deref())?;
    let file = vps::load(&path)?;
    let jobs = vps::resolve_host_jobs(selection, &file)?;
    let limit = crate::concurrency::effective_limit();
    let remote_owned = remote.to_path_buf();
    let local_prefix = local_prefix.to_path_buf();
    let path_c = path.clone();
    let replace = opts.replace_host_key;
    let json = opts.json;
    let opts_arc = std::sync::Arc::new(opts);

    tracing::info!(
        hosts = jobs.len(),
        max_concurrency = limit,
        "multi-host scp download fan-out"
    );

    let results = crate::concurrency::map_bounded(jobs, limit, move |(name, mut record)| {
        let opts = opts_arc.clone();
        let remote_owned = remote_owned.clone();
        let local_path = PathBuf::from(format!(
            "{}.{}",
            local_prefix.display(),
            name.replace(['/', '\\'], "_")
        ));
        let path_c = path_c.clone();
        async move {
            if crate::signals::should_stop() {
                return HostScpResult {
                    name,
                    ok: false,
                    bytes: None,
                    duration_ms: None,
                    local: Some(local_path.display().to_string()),
                    error: Some("operation cancelled by signal".into()),
                };
            }
            apply_scp_options(&mut record, opts.as_ref());
            let cfg = vps::build_connection_config(&record, Some(&path_c), replace);
            match <SshClient as SshClientTrait>::connect(cfg).await {
                Ok(client) => {
                    let result = client.download(&remote_owned, &local_path).await;
                    let _ = client.disconnect().await;
                    match result {
                        Ok(t) => HostScpResult {
                            name,
                            ok: true,
                            bytes: Some(t.bytes_transferred),
                            duration_ms: Some(t.duration_ms),
                            local: Some(local_path.display().to_string()),
                            error: None,
                        },
                        Err(e) => HostScpResult {
                            name,
                            ok: false,
                            bytes: None,
                            duration_ms: None,
                            local: Some(local_path.display().to_string()),
                            error: Some(e.to_string()),
                        },
                    }
                }
                Err(e) => HostScpResult {
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

    finish_scp_batch("download", results, limit, json)
}
