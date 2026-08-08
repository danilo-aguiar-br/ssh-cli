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

/// G5/G17: explicit cancelled batch row (cardinal honesty for multi-host SFTP).
fn cancelled_host_sftp(name: String, local: Option<String>) -> HostSftpResult {
    HostSftpResult {
        name,
        ok: false,
        bytes: None,
        duration_ms: None,
        local,
        error: Some(crate::i18n::t(crate::i18n::Message::OperationCancelled)),
    }
}

/// B4: a host that `map_bounded` never admitted (`--fail-fast`, cancel) produces no
/// `IndexedResult` at all. Emitting nothing for it would silently shrink the batch and
/// make "3/3 ok" mean "3 of the 5 you asked for". Every requested host gets a row.
const HOST_NOT_ATTEMPTED: &str = "not attempted (fan-out admission stopped)";

fn not_attempted_host_sftp(name: String) -> HostSftpResult {
    HostSftpResult {
        name,
        ok: false,
        bytes: None,
        duration_ms: None,
        local: None,
        error: Some(HOST_NOT_ATTEMPTED.to_owned()),
    }
}

/// Batch op label for [`crate::errors::finish_batch`] (needs `&'static str`).
fn sftp_batch_op(direction: &str) -> &'static str {
    if direction == "upload" {
        "multi-host sftp upload"
    } else {
        "multi-host sftp download"
    }
}

/// Collapses fan-out outcomes into one row per **requested** host, in `names` order.
///
/// `names` is the host list handed to `map_bounded`, so index `i` of a result maps
/// back to `names[i]`.
fn finish_batch(
    direction: &'static str,
    results: Vec<crate::concurrency::IndexedResult<HostSftpResult>>,
    names: &[String],
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    let mut slots: Vec<Option<HostSftpResult>> = (0..names.len()).map(|_| None).collect();
    // Indices outside `names` cannot happen, but dropping a row would be worse than
    // appending one, so keep any surplus instead of discarding it.
    let mut surplus: Vec<HostSftpResult> = Vec::new();

    for r in results {
        let row = match r.outcome {
            Ok(h) => h,
            Err(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
            Err(e) => HostSftpResult {
                name: names
                    .get(r.index)
                    .cloned()
                    .unwrap_or_else(|| format!("task-{}", r.index)),
                ok: false,
                bytes: None,
                duration_ms: None,
                local: None,
                error: Some(e.to_string()),
            },
        };
        match slots.get_mut(r.index) {
            Some(slot) => *slot = Some(row),
            None => surplus.push(row),
        }
    }

    let mut host_results = Vec::with_capacity(slots.len().saturating_add(surplus.len()));
    for (i, slot) in slots.into_iter().enumerate() {
        host_results.push(match slot {
            Some(row) => row,
            None => not_attempted_host_sftp(names[i].clone()),
        });
    }
    host_results.extend(surplus);

    let failures = host_results.iter().filter(|h| !h.ok).count();
    let total = host_results.len();
    output::print_sftp_batch(direction, &host_results, limit, json)?;
    crate::errors::finish_batch(failures, total, sftp_batch_op(direction))?;
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
    let names: Vec<String> = jobs.iter().map(|(n, _)| n.clone()).collect();
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
                return cancelled_host_sftp(name, Some(local_owned.display().to_string()));
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

    finish_batch("upload", results, &names, limit, json)
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
    let names: Vec<String> = jobs.iter().map(|(n, _)| n.clone()).collect();
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
                return cancelled_host_sftp(name, Some(local_path.display().to_string()));
            }
            apply_sftp_options(&mut record, opts.as_ref());
            let cfg = vps::build_connection_config(&record, Some(&path_c), replace);
            match SshClient::connect(cfg).await {
                Ok(client) => {
                    let result = if recursive {
                        client.sftp_download_tree(&remote_owned, &local_path).await
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

    finish_batch("download", results, &names, limit, json)
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
    let names: Vec<String> = jobs.iter().map(|(n, _)| n.clone()).collect();
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
                        // B5: `?` inside this block would jump over `close_sftp` and
                        // leak the subsystem channel until the SSH session is torn
                        // down. Capture the first error, close, then return it.
                        let mut err: Option<SshCliError> = None;
                        for src in sources.iter() {
                            match upload_one_file(&sftp, src, &dest).await {
                                Ok(n) => bytes = bytes.saturating_add(n),
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

    finish_batch("upload", results, &names, limit, json)
}

/// One file of a multi-file upload: basename validation + remote join + transfer.
///
/// Extracted so the caller's loop can use `match` and still reach `close_sftp`.
async fn upload_one_file(
    sftp: &russh_sftp::client::SftpSession,
    src: &Path,
    dest_dir: &str,
) -> Result<u64, SshCliError> {
    let name = src
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| SFTP_FALLBACK_BASENAME.to_owned());
    validate_entry_name(&name)?;
    // Remote paths are always `/`-separated: `Path::join` would emit `\` on Windows.
    let remote = crate::ssh::sftp_path::join_remote(dest_dir, &name);
    let r = sftp_session::upload_file(sftp, src, &remote).await?;
    Ok(r.bytes_transferred)
}

/// One file of a multi-file download: basename validation + jail check + transfer.
///
/// Extracted so the caller's loop can use `match` and still reach `close_sftp`.
async fn download_one_file(
    sftp: &russh_sftp::client::SftpSession,
    remote_p: &Path,
    host_dir: &Path,
    host_root: &Path,
) -> Result<u64, SshCliError> {
    let remote = remote_p.to_string_lossy().into_owned();
    let fname = remote_p
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| SFTP_FALLBACK_BASENAME.to_owned());
    validate_entry_name(&fname)?;
    let local = host_dir.join(&fname);
    ensure_local_under(host_root, &local)?;
    let r = sftp_session::download_file(sftp, &remote, &local).await?;
    Ok(r.bytes_transferred)
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
    let names: Vec<String> = jobs.iter().map(|(n, _)| n.clone()).collect();
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
                        // B5: same leak as the upload path — never `?` past `close_sftp`.
                        let mut err: Option<SshCliError> = None;
                        for remote_p in remotes.iter() {
                            match download_one_file(&sftp, remote_p, &host_dir, &host_root).await {
                                Ok(n) => bytes = bytes.saturating_add(n),
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

    finish_batch("download", results, &names, limit, json)
}
