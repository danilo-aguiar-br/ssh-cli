// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP-04: health-check fan-out extracted from `vps/mod` (SRP + parallelism).
#![forbid(unsafe_code)]
//! SSH health-check (single host + multi-host bounded fan-out).
//!
//! Workload: **I/O-bound** connect probe. Multi-host uses
//! [`crate::concurrency::map_bounded`] (Semaphore + JoinSet). Doctor reuses
//! [`collect_health_check_batch`] for `--probe-ssh`.

use super::{
    apply_overrides, build_connection_config, load, resolve_config_path, resolve_host_jobs,
    use_json, HostSelection,
};
use crate::cli::OutputFormat;
use crate::errors::SshCliError;
use crate::output;
use crate::ssh::client::{SshClient, SshClientTrait};
use anyhow::Result;
use secrecy::SecretString;
use std::path::PathBuf;

/// Health-check SSH (single host or multi-host bounded fan-out).
///
/// Workload: **I/O-bound** connect probe. One-shot auth parity (GAP-SSH-CLI-006)
/// and TOFU (M1). Multi-host saturates sockets/auth — gated by concurrency budget.
/// Batch JSON when [`HostSelection::is_batch`] (G-PAR-36).
#[allow(clippy::too_many_arguments)]
pub async fn run_health_check(
    selection: HostSelection,
    config_override: Option<PathBuf>,
    format: OutputFormat,
    json_local: bool,
    password_override: Option<SecretString>,
    timeout_override: Option<crate::domain::TimeoutMs>,
    key_override: Option<String>,
    key_passphrase_override: Option<SecretString>,
    replace_host_key: bool,
) -> Result<()> {
    // M2: local --json or global format → JSON error envelope on failure.
    if json_local || format == OutputFormat::Json {
        crate::output::set_json_errors(true);
    }
    if crate::signals::should_stop() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Message::OperationCancelled
        )));
    }
    if selection.is_batch() {
        return run_health_check_all(
            &selection,
            config_override,
            format,
            json_local,
            password_override,
            timeout_override,
            key_override,
            key_passphrase_override,
            replace_host_key,
        )
        .await;
    }
    let HostSelection::Single(resolved_name) = selection else {
        // G-SEC-08: fail closed instead of panic on invariant slip.
        return Err(SshCliError::InvalidArgument(
            "internal: expected single-host selection for non-batch health-check".into(),
        )
        .into());
    };
    let resolved_key = resolved_name.as_str().to_owned();
    let path = resolve_config_path(config_override.as_deref())?;
    let mut file = load(&path)?;
    let mut vps = file
        .hosts
        .remove(&resolved_key)
        .ok_or_else(|| SshCliError::VpsNotFound(resolved_key.clone()))?;

    // GAP-SSH-CLI-004: --timeout; GAP-SSH-CLI-006: key + passphrase.
    // Ordem: password, sudo, su, timeout, key_path, key_passphrase.
    apply_overrides(
        &mut vps,
        password_override,
        None,
        None,
        timeout_override,
        key_override,
        key_passphrase_override,
        false,
        None,
    );
    // M1: honra --replace-host-key global (paridade exec/scp/tunnel).
    let cfg = build_connection_config(&vps, Some(&path), replace_host_key);
    let start = std::time::Instant::now();
    let client: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
    let latency_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    client.disconnect().await?;

    if use_json(json_local, format) {
        output::print_health_check_json(&resolved_key, latency_ms)?;
    } else {
        output::print_health_check(&resolved_key, latency_ms);
    }
    Ok(())
}

/// Collect multi-host health results without printing (doctor envelope + health-check).
pub(super) async fn collect_health_check_batch(
    selection: &HostSelection,
    config_override: Option<PathBuf>,
) -> Result<(Vec<HostHealthResult>, usize)> {
    collect_health_check_batch_with_opts(
        selection,
        config_override,
        None,
        None,
        None,
        None,
        false,
    )
    .await
}

/// Parallel health-check fan-out (I/O-bound, map_bounded); returns results + limit.
#[allow(clippy::too_many_arguments)]
async fn collect_health_check_batch_with_opts(
    selection: &HostSelection,
    config_override: Option<PathBuf>,
    password_override: Option<SecretString>,
    timeout_override: Option<crate::domain::TimeoutMs>,
    key_override: Option<String>,
    key_passphrase_override: Option<SecretString>,
    replace_host_key: bool,
) -> Result<(Vec<HostHealthResult>, usize)> {
    let path = resolve_config_path(config_override.as_deref())?;
    let file = load(&path)?;
    let jobs = resolve_host_jobs(selection, &file)?;
    let limit = crate::concurrency::effective_limit();
    let path_c = path.clone();

    tracing::info!(
        hosts = jobs.len(),
        max_concurrency = limit,
        "multi-host health-check fan-out"
    );

    let pw = password_override;
    let to = timeout_override;
    let key = key_override;
    let kp = key_passphrase_override;

    let results = crate::concurrency::map_bounded(jobs, limit, move |(name, mut vps)| {
        let path_c = path_c.clone();
        let pw = pw.clone();
        let key = key.clone();
        let kp = kp.clone();
        async move {
            if crate::signals::should_stop() {
                return HostHealthResult {
                    name,
                    ok: false,
                    latency_ms: None,
                    error: Some("operation cancelled by signal".into()),
                };
            }
            apply_overrides(&mut vps, pw, None, None, to, key, kp, false, None);
            let start = std::time::Instant::now();
            let cfg = build_connection_config(&vps, Some(&path_c), replace_host_key);
            match <SshClient as SshClientTrait>::connect(cfg).await {
                Ok(client) => {
                    let latency_ms =
                        u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                    let _ = client.disconnect().await;
                    HostHealthResult {
                        name,
                        ok: true,
                        latency_ms: Some(latency_ms),
                        error: None,
                    }
                }
                Err(e) => HostHealthResult {
                    name,
                    ok: false,
                    latency_ms: Some(
                        u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                    ),
                    error: Some(e.to_string()),
                },
            }
        }
    })
    .await;

    let mut host_results = Vec::with_capacity(results.len());
    for r in results {
        match r.outcome {
            Ok(h) => host_results.push(h),
            Err(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
            Err(e) => {
                host_results.push(HostHealthResult {
                    name: format!("task-{}", r.index),
                    ok: false,
                    latency_ms: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    Ok((host_results, limit))
}

/// Parallel health-check for `--all` / `--hosts` (I/O-bound, map_bounded).
#[allow(clippy::too_many_arguments)]
async fn run_health_check_all(
    selection: &HostSelection,
    config_override: Option<PathBuf>,
    format: OutputFormat,
    json_local: bool,
    password_override: Option<SecretString>,
    timeout_override: Option<crate::domain::TimeoutMs>,
    key_override: Option<String>,
    key_passphrase_override: Option<SecretString>,
    replace_host_key: bool,
) -> Result<()> {
    let (host_results, limit) = collect_health_check_batch_with_opts(
        selection,
        config_override,
        password_override,
        timeout_override,
        key_override,
        key_passphrase_override,
        replace_host_key,
    )
    .await?;

    let failures = host_results.iter().filter(|h| !h.ok).count();
    let as_json = use_json(json_local, format);
    output::print_health_batch(&host_results, limit, as_json)?;
    if failures > 0 {
        return Err(SshCliError::Config(format!(
            "{failures}/{} hosts failed health-check",
            host_results.len()
        ))
        .into());
    }
    Ok(())
}

/// Per-host health-check outcome for batch output.
#[derive(Debug, Clone)]
pub struct HostHealthResult {
    /// VPS name.
    pub name: String,
    /// Whether connect+auth succeeded.
    pub ok: bool,
    /// Latency when measured.
    pub latency_ms: Option<u64>,
    /// Error text when not ok.
    pub error: Option<String>,
}

