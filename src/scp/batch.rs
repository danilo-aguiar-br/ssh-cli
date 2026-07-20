// SPDX-License-Identifier: MIT OR Apache-2.0
//! Multi-file and multi-host SCP batch paths (G-COMP-06c).
//!
//! Workload: **I/O-bound**. Multi-host uses [`crate::concurrency::map_bounded`]
//! (one permit = one SSH session). Multi-file on one host reuses one session
//! (G-PAR-47). Secrets stay in [`super::ScpOptions`] as [`secrecy::SecretString`].
#![forbid(unsafe_code)]

use super::{apply_scp_options, HostScpResult, ScpOptions};
use crate::errors::SshCliError;
use crate::i18n::{self, Message};
use crate::output;
use crate::ssh::client::{SshClient, SshClientTrait};
use crate::vps;
use std::path::{Path, PathBuf};

pub(crate) async fn validate_local_upload_sources(sources: &[PathBuf]) -> anyhow::Result<()> {
    for local in sources {
        let meta = tokio::fs::metadata(local).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SshCliError::FileNotFound(local.display().to_string())
            } else {
                SshCliError::Io(e)
            }
        })?;
        if meta.is_dir() {
            return Err(SshCliError::InvalidArgument(i18n::t(Message::ScpUploadFileOnly)).into());
        }
        if !meta.is_file() {
            return Err(SshCliError::FileNotFound(local.display().to_string()).into());
        }
    }
    Ok(())
}

/// G-PAR-37 + G-PAR-47: single-host multi-file upload — **one** SSH session, serial transfers.
pub(crate) async fn run_scp_multi_file_upload(
    vps_name: &str,
    sources: Vec<PathBuf>,
    remote_dir: &Path,
    config_override: Option<PathBuf>,
    opts: ScpOptions,
) -> anyhow::Result<()> {
    validate_local_upload_sources(&sources).await?;
    let mut record = vps::find_by_name(config_override.as_deref(), vps_name)?
        .ok_or_else(|| SshCliError::VpsNotFound(vps_name.to_string()))?;
    apply_scp_options(&mut record, &opts);
    let path = vps::resolve_config_path(config_override.as_deref())?;
    let replace = opts.replace_host_key;
    let json = opts.json;
    let limit = crate::concurrency::effective_limit();

    tracing::info!(
        files = sources.len(),
        vps = %vps_name,
        session_reuse = true,
        "multi-file scp upload (one session)"
    );

    let cfg = vps::build_connection_config(&record, Some(&path), replace);
    let client: Box<dyn SshClientTrait> =
        <SshClient as SshClientTrait>::connect(cfg).await?;
    // G-PAR-47 / G-O4: session reuse; optional parallel channels via scp_file_concurrency.
    let host_results =
        multi_file_upload_on_session(client.as_ref(), &sources, remote_dir, None).await;
    let _ = client.disconnect().await;
    finish_scp_results("upload", host_results, limit, json)
}

/// Multi-file upload on one session (G-PAR-47 serial default; G-O4 parallel windows).
///
/// `name_prefix`: when `Some(host)`, result `name` is `host:path` (multi-host batch).
pub(crate) async fn multi_file_upload_on_session(
    client: &dyn SshClientTrait,
    sources: &[PathBuf],
    remote_dir: &Path,
    name_prefix: Option<&str>,
) -> Vec<HostScpResult> {
    let window = crate::concurrency::scp_file_concurrency().max(1);
    if window > 1 && sources.len() > 1 {
        tracing::debug!(window, files = sources.len(), "scp multi-file parallel window");
    }
    let mut host_results = Vec::with_capacity(sources.len());
    // Process in windows of `window` concurrent uploads (same session, &self channels).
    let mut i = 0;
    while i < sources.len() {
        if crate::signals::should_stop() {
            break;
        }
        let end = (i + window).min(sources.len());
        let slice = &sources[i..end];
        match slice.len() {
            0 => break,
            1 => {
                host_results.push(upload_one(client, &slice[0], remote_dir, name_prefix).await);
            }
            2 => {
                let (a, b) = tokio::join!(
                    upload_one(client, &slice[0], remote_dir, name_prefix),
                    upload_one(client, &slice[1], remote_dir, name_prefix),
                );
                host_results.push(a);
                host_results.push(b);
            }
            _ => {
                // window >= 3: pair-wise concurrent then remainder
                let (a, b) = tokio::join!(
                    upload_one(client, &slice[0], remote_dir, name_prefix),
                    upload_one(client, &slice[1], remote_dir, name_prefix),
                );
                host_results.push(a);
                host_results.push(b);
                for local in &slice[2..] {
                    host_results.push(upload_one(client, local, remote_dir, name_prefix).await);
                }
            }
        }
        i = end;
    }
    host_results
}

async fn upload_one(
    client: &dyn SshClientTrait,
    local: &Path,
    remote_dir: &Path,
    name_prefix: Option<&str>,
) -> HostScpResult {
    let label = match name_prefix {
        Some(h) => format!("{h}:{}", local.display()),
        None => local.display().to_string(),
    };
    if crate::signals::should_stop() {
        return HostScpResult {
            name: label,
            ok: false,
            bytes: None,
            duration_ms: None,
            local: Some(local.display().to_string()),
            error: Some("operation cancelled by signal".into()),
        };
    }
    let base = local
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("file"));
    let remote = remote_dir.join(base);
    match client.upload(local, &remote).await {
        Ok(t) => HostScpResult {
            name: label,
            ok: true,
            bytes: Some(t.bytes_transferred),
            duration_ms: Some(t.duration_ms),
            local: Some(local.display().to_string()),
            error: None,
        },
        Err(e) => HostScpResult {
            name: label,
            ok: false,
            bytes: None,
            duration_ms: None,
            local: Some(local.display().to_string()),
            error: Some(e.to_string()),
        },
    }
}

#[allow(dead_code)]
/// G-PAR-37 + G-PAR-47: single-host multi-file download — **one** SSH session.
pub(crate) async fn run_scp_multi_file_download(
    vps_name: &str,
    remotes: Vec<PathBuf>,
    local_dir: &Path,
    config_override: Option<PathBuf>,
    opts: ScpOptions,
) -> anyhow::Result<()> {
    // Destination must be a directory (or non-existent path we treat as dir name).
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

    let mut record = vps::find_by_name(config_override.as_deref(), vps_name)?
        .ok_or_else(|| SshCliError::VpsNotFound(vps_name.to_string()))?;
    apply_scp_options(&mut record, &opts);
    let path = vps::resolve_config_path(config_override.as_deref())?;
    let replace = opts.replace_host_key;
    let json = opts.json;
    let limit = crate::concurrency::effective_limit();

    tracing::info!(
        files = remotes.len(),
        vps = %vps_name,
        session_reuse = true,
        "multi-file scp download (one session)"
    );

    let cfg = vps::build_connection_config(&record, Some(&path), replace);
    let client: Box<dyn SshClientTrait> =
        <SshClient as SshClientTrait>::connect(cfg).await?;
    let host_results =
        multi_file_download_on_session(client.as_ref(), &remotes, local_dir, None).await;
    let _ = client.disconnect().await;
    finish_scp_results("download", host_results, limit, json)
}

/// Serial multi-file download on an already-open session (G-PAR-47 testable).
pub(crate) async fn multi_file_download_on_session(
    client: &dyn SshClientTrait,
    remotes: &[PathBuf],
    local_dir: &Path,
    name_prefix: Option<&str>,
) -> Vec<HostScpResult> {
    let mut host_results = Vec::with_capacity(remotes.len());
    for remote in remotes {
        let label = match name_prefix {
            Some(h) => format!("{h}:{}", remote.display()),
            None => remote.display().to_string(),
        };
        if crate::signals::should_stop() {
            host_results.push(HostScpResult {
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
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("file"));
        let local = local_dir.join(base);
        match client.download(remote, &local).await {
            Ok(t) => host_results.push(HostScpResult {
                name: label,
                ok: true,
                bytes: Some(t.bytes_transferred),
                duration_ms: Some(t.duration_ms),
                local: Some(local.display().to_string()),
                error: None,
            }),
            Err(e) => host_results.push(HostScpResult {
                name: label,
                ok: false,
                bytes: None,
                duration_ms: None,
                local: Some(local.display().to_string()),
                error: Some(e.to_string()),
            }),
        }
    }
    host_results
}


pub(crate) fn finish_scp_results(
    direction: &str,
    host_results: Vec<HostScpResult>,
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    let failures = host_results.iter().filter(|h| !h.ok).count();
    output::print_scp_batch(direction, &host_results, limit, json)?;
    if failures > 0 {
        return Err(SshCliError::Config(format!(
            "{failures}/{} transfers failed multi-file scp {direction}",
            host_results.len()
        ))
        .into());
    }
    Ok(())
}

/// Flatten `map_bounded` of `Vec<HostScpResult>` per host (G-PAR-48).
pub(crate) fn finish_scp_nested_batch(
    direction: &str,
    results: Vec<crate::concurrency::IndexedResult<Vec<HostScpResult>>>,
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    let mut host_results = Vec::new();
    let mut failures = 0usize;
    for r in results {
        match r.outcome {
            Ok(batch) => {
                for h in batch {
                    if !h.ok {
                        failures += 1;
                    }
                    host_results.push(h);
                }
            }
            Err(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
            Err(e) => {
                failures += 1;
                host_results.push(HostScpResult {
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
    output::print_scp_batch(direction, &host_results, limit, json)?;
    if failures > 0 {
        return Err(SshCliError::Config(format!(
            "{failures}/{} transfers failed multi-host multi-file scp {direction}",
            host_results.len()
        ))
        .into());
    }
    Ok(())
}

pub(crate) fn finish_scp_batch(
    direction: &str,
    results: Vec<crate::concurrency::IndexedResult<HostScpResult>>,
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
                host_results.push(HostScpResult {
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
    output::print_scp_batch(direction, &host_results, limit, json)?;
    if failures > 0 {
        return Err(SshCliError::Config(format!(
            "{failures}/{} hosts failed multi-host scp {direction}",
            host_results.len()
        ))
        .into());
    }
    Ok(())
}
