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
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::Poll;

/// Joins a **remote** SCP directory with a basename using the SSH wire separator.
///
/// B6: `Path::join` uses the *host* separator, so on Windows it produces
/// `dir\file`. That string is handed to a POSIX shell on the far side, where `\`
/// is an escape character and not a directory boundary — the upload silently
/// lands in the wrong place (or creates a file with a backslash in its name).
/// Remote paths are always `/`-separated regardless of where the CLI runs.
pub(crate) fn join_remote_scp(remote_dir: &Path, base: &str) -> PathBuf {
    let dir = remote_dir.to_string_lossy().into_owned();
    if dir.is_empty() || dir == "." {
        return PathBuf::from(base);
    }
    if dir.ends_with('/') {
        PathBuf::from(format!("{dir}{base}"))
    } else {
        PathBuf::from(format!("{dir}/{base}"))
    }
}

/// Drives every future to completion concurrently, preserving input order.
///
/// B8: the previous window used `tokio::join!` on the first two entries and ran
/// the rest serially, so `--scp-file-concurrency 8` behaved like `2`. The futures
/// borrow the shared session (`&dyn SshClientTrait`), so `JoinSet` — which needs
/// `'static` — is not available; polling them in place is.
async fn join_window<T>(mut futures: Vec<Pin<Box<dyn Future<Output = T> + Send + '_>>>) -> Vec<T> {
    let mut slots: Vec<Option<T>> = (0..futures.len()).map(|_| None).collect();
    std::future::poll_fn(|cx| {
        let mut all_ready = true;
        for (slot, fut) in slots.iter_mut().zip(futures.iter_mut()) {
            if slot.is_some() {
                continue;
            }
            match fut.as_mut().poll(cx) {
                Poll::Ready(value) => *slot = Some(value),
                Poll::Pending => all_ready = false,
            }
        }
        if all_ready {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await;
    // Every slot was filled before `poll_fn` resolved.
    slots.into_iter().flatten().collect()
}

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
    let client: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
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
        tracing::debug!(
            window,
            files = sources.len(),
            "scp multi-file parallel window"
        );
    }
    let mut host_results = Vec::with_capacity(sources.len());
    // Process in windows of `window` concurrent uploads (same session, &self channels).
    let mut i = 0;
    while i < sources.len() {
        // G5: never return a short vector — fill cancelled remainder so
        // `results.len() == sources.len()` always holds.
        if crate::signals::should_stop() {
            push_cancelled_upload_remainder(&mut host_results, &sources[i..], name_prefix);
            break;
        }
        let end = (i + window).min(sources.len());
        let slice = &sources[i..end];
        if slice.is_empty() {
            break;
        }
        if slice.len() == 1 {
            host_results.push(upload_one(client, &slice[0], remote_dir, name_prefix).await);
        } else {
            // B8: the whole window runs concurrently, honouring the requested value.
            let futures = slice
                .iter()
                .map(|local| {
                    Box::pin(upload_one(client, local, remote_dir, name_prefix))
                        as Pin<Box<dyn Future<Output = HostScpResult> + Send>>
                })
                .collect();
            host_results.extend(join_window(futures).await);
        }
        i = end;
    }
    debug_assert_eq!(host_results.len(), sources.len());
    host_results
}

/// G5/G17: explicit cancelled rows for every unprocessed source (cardinal equality).
fn push_cancelled_upload_remainder(
    out: &mut Vec<HostScpResult>,
    remaining: &[PathBuf],
    name_prefix: Option<&str>,
) {
    for local in remaining {
        let label = match name_prefix {
            Some(h) => format!("{h}:{}", local.display()),
            None => local.display().to_string(),
        };
        out.push(cancelled_host_scp(label, Some(local.display().to_string())));
    }
}

/// Shared cancelled batch row (G5/G17 — machine-readable cancel, not a short vec).
pub(crate) fn cancelled_host_scp(name: String, local: Option<String>) -> HostScpResult {
    HostScpResult {
        name,
        ok: false,
        bytes: None,
        duration_ms: None,
        local,
        error: Some(i18n::t(Message::OperationCancelled)),
    }
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
        return cancelled_host_scp(label, Some(local.display().to_string()));
    }
    let base = local
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_owned());
    let remote = join_remote_scp(remote_dir, &base);
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
                "multi-file download destination must be a directory (not an existing file)".into(),
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
    let client: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
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
    for (idx, remote) in remotes.iter().enumerate() {
        let label = match name_prefix {
            Some(h) => format!("{h}:{}", remote.display()),
            None => remote.display().to_string(),
        };
        // G5/G17: fill every remaining remote as cancelled; never shorten the vec.
        if crate::signals::should_stop() {
            host_results.push(cancelled_host_scp(label, None));
            for remote in &remotes[idx + 1..] {
                let lab = match name_prefix {
                    Some(h) => format!("{h}:{}", remote.display()),
                    None => remote.display().to_string(),
                };
                host_results.push(cancelled_host_scp(lab, None));
            }
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
    debug_assert_eq!(host_results.len(), remotes.len());
    host_results
}

/// B4: rows for hosts `map_bounded` never admitted (`--fail-fast`, cancel).
const HOST_NOT_ATTEMPTED: &str = "not attempted (fan-out admission stopped)";

fn not_attempted_host_scp(name: String) -> HostScpResult {
    HostScpResult {
        name,
        ok: false,
        bytes: None,
        duration_ms: None,
        local: None,
        error: Some(HOST_NOT_ATTEMPTED.to_owned()),
    }
}

/// Batch op labels for [`crate::errors::finish_batch`] (needs `&'static str`).
fn scp_op(direction: &str, upload: &'static str, download: &'static str) -> &'static str {
    if direction == "upload" {
        upload
    } else {
        download
    }
}

pub(crate) fn finish_scp_results(
    direction: &'static str,
    host_results: Vec<HostScpResult>,
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    let failures = host_results.iter().filter(|h| !h.ok).count();
    let total = host_results.len();
    output::print_scp_batch(direction, &host_results, limit, json)?;
    crate::errors::finish_batch(
        failures,
        total,
        scp_op(
            direction,
            "multi-file scp upload",
            "multi-file scp download",
        ),
    )?;
    Ok(())
}

/// Places fan-out outcomes back on their requested host slot, in `names` order.
///
/// B4: `map_bounded` returns nothing for indices it never admitted, so the caller
/// must reconstruct them from the requested host list; `missing` builds the row.
fn slot_by_host<R>(
    results: Vec<crate::concurrency::IndexedResult<R>>,
    names: &[String],
    on_join_error: impl Fn(String, String) -> R,
    missing: impl Fn(String) -> R,
) -> Vec<R> {
    let mut slots: Vec<Option<R>> = (0..names.len()).map(|_| None).collect();
    // Out-of-range indices cannot happen, but appending beats dropping a real row.
    let mut surplus: Vec<R> = Vec::new();

    for r in results {
        let row = match r.outcome {
            Ok(value) => value,
            Err(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
            Err(e) => on_join_error(
                names
                    .get(r.index)
                    .cloned()
                    .unwrap_or_else(|| format!("task-{}", r.index)),
                e.to_string(),
            ),
        };
        match slots.get_mut(r.index) {
            Some(slot) => *slot = Some(row),
            None => surplus.push(row),
        }
    }

    let mut out = Vec::with_capacity(slots.len().saturating_add(surplus.len()));
    for (i, slot) in slots.into_iter().enumerate() {
        out.push(match slot {
            Some(row) => row,
            None => missing(names[i].clone()),
        });
    }
    out.extend(surplus);
    out
}

/// Flatten `map_bounded` of `Vec<HostScpResult>` per host (G-PAR-48).
pub(crate) fn finish_scp_nested_batch(
    direction: &'static str,
    results: Vec<crate::concurrency::IndexedResult<Vec<HostScpResult>>>,
    names: &[String],
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    let per_host = slot_by_host(
        results,
        names,
        |name, err| {
            vec![HostScpResult {
                name,
                ok: false,
                bytes: None,
                duration_ms: None,
                local: None,
                error: Some(err),
            }]
        },
        |name| vec![not_attempted_host_scp(name)],
    );
    let host_results: Vec<HostScpResult> = per_host.into_iter().flatten().collect();
    let failures = host_results.iter().filter(|h| !h.ok).count();
    let total = host_results.len();
    output::print_scp_batch(direction, &host_results, limit, json)?;
    crate::errors::finish_batch(
        failures,
        total,
        scp_op(
            direction,
            "multi-host multi-file scp upload",
            "multi-host multi-file scp download",
        ),
    )?;
    Ok(())
}

pub(crate) fn finish_scp_batch(
    direction: &'static str,
    results: Vec<crate::concurrency::IndexedResult<HostScpResult>>,
    names: &[String],
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    let host_results = slot_by_host(
        results,
        names,
        |name, err| HostScpResult {
            name,
            ok: false,
            bytes: None,
            duration_ms: None,
            local: None,
            error: Some(err),
        },
        not_attempted_host_scp,
    );
    let failures = host_results.iter().filter(|h| !h.ok).count();
    let total = host_results.len();
    output::print_scp_batch(direction, &host_results, limit, json)?;
    crate::errors::finish_batch(
        failures,
        total,
        scp_op(
            direction,
            "multi-host scp upload",
            "multi-host scp download",
        ),
    )?;
    Ok(())
}

#[cfg(test)]
mod remote_path_tests {
    use super::*;

    /// B6: the remote separator is `/` on every host OS. `Path::join` would emit
    /// `\` on Windows and the POSIX shell on the far side would read it as an
    /// escape, not a directory boundary.
    #[test]
    fn remote_join_always_uses_forward_slash() {
        let joined = join_remote_scp(Path::new("/srv/incoming"), "report.csv");
        assert_eq!(joined.to_string_lossy(), "/srv/incoming/report.csv");
        assert!(!joined.to_string_lossy().contains('\\'));
    }

    #[test]
    fn remote_join_does_not_double_separator() {
        assert_eq!(
            join_remote_scp(Path::new("/srv/incoming/"), "a.txt").to_string_lossy(),
            "/srv/incoming/a.txt"
        );
    }

    #[test]
    fn remote_join_relative_and_dot_dirs() {
        assert_eq!(
            join_remote_scp(Path::new("uploads"), "a.txt").to_string_lossy(),
            "uploads/a.txt"
        );
        assert_eq!(
            join_remote_scp(Path::new("."), "a.txt").to_string_lossy(),
            "a.txt"
        );
        assert_eq!(
            join_remote_scp(Path::new(""), "a.txt").to_string_lossy(),
            "a.txt"
        );
    }

    /// A Windows-shaped local dir string still crosses the wire as typed: we must
    /// not *introduce* a backslash, and we must not rewrite the caller's path.
    #[test]
    fn remote_join_preserves_caller_path_text() {
        assert_eq!(
            join_remote_scp(Path::new("/tmp/dir with space"), "b c.txt").to_string_lossy(),
            "/tmp/dir with space/b c.txt"
        );
    }
}
