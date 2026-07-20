// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP-02: doctor diagnostics extracted from `vps/mod` (SRP).
#![forbid(unsafe_code)]
//! Local XDG/schema doctor + optional multi-host SSH probe (G-PAR-29 / G-PAR-42).
//!
//! Workload: local disk/metadata by default; with `--probe-ssh` fans out via
//! [`super::collect_health_check_batch`] (I/O-bound, bounded concurrency).

use super::health::{collect_health_check_batch, HostHealthResult};
use super::{load, winning_layer, HostSelection};
use crate::errors::SshCliError;
use crate::ssh::known_hosts::KnownHosts;
use anyhow::Result;
use std::path::Path;

/// Local XDG/schema diagnostics as a JSON object (no print).
///
/// Workload: **local disk / metadata**. Sequential justified: no multi-host SSH
/// unless caller adds `--probe-ssh` (G-PAR-29) which fans out via health-check.
pub(super) fn collect_doctor_local(config_override: Option<&Path>) -> Result<serde_json::Value> {
    let layer = winning_layer(config_override)?;
    let path = layer.path.clone();
    let exists = path.exists();
    let file = load(&path)?;
    let kh = KnownHosts::path_beside_config(&path);
    let active = path
        .parent()
        .map(|p| p.join(crate::constants::ACTIVE_VPS_FILE_NAME))
        .unwrap_or_else(|| std::path::PathBuf::from(crate::constants::ACTIVE_VPS_FILE_NAME));
    let perms = if exists {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            format!(
                "{:o}",
                std::fs::metadata(&path)?.permissions().mode() & 0o777
            )
        }
        #[cfg(not(unix))]
        {
            "n/a".to_string()
        }
    } else {
        "missing".to_string()
    };

    let seg = crate::secrets::secrets_status()?;
    let runtime = crate::platform::detect_runtime();
    // Soft guard: warn when config path would break on Windows MAX_PATH (or is
    // already over limit on this host). Non-fatal — doctor must still report.
    if let Err(e) = crate::paths::validate_local_path_length(&path) {
        tracing::warn!(error = %e, path = %path.display(), "config path length warning");
    }
    Ok(serde_json::json!({
        "layer": layer.name,
        "config_path": path.display().to_string(),
        "exists": exists,
        "permissions": perms,
        "schema_version": file.schema_version,
        "hosts": file.hosts.len(),
        "known_hosts": kh.display().to_string(),
        "active_file": active.display().to_string(),
        "secrets_at_rest": if seg.encryption_active { "encrypted" } else { "plaintext" },
        "secrets_key_source": seg.source.as_str(),
        "secrets_key_file": seg.key_file_path.display().to_string(),
        "secrets_plaintext_opt_out": seg.plaintext_opt_out,
        "telemetry": false,
        "runtime": {
            "os": runtime.os,
            "arch": runtime.arch,
            "is_wsl": runtime.is_wsl,
            "is_container": runtime.is_container,
            "is_ci": runtime.is_ci,
            "is_termux": runtime.is_termux,
            "sandbox": runtime.sandbox,
        },
    }))
}

/// Doctor local report + optional bounded SSH probe in **one** stdout root (G-PAR-42).
///
/// When `probe_selection` is `Some`, runs health fan-out and embeds results under
/// `ssh_probe` (JSON) or prints a second human section (text). Never emits two
/// independent JSON roots.
pub(super) async fn run_doctor_with_optional_probe(
    config_override: Option<&Path>,
    json: bool,
    probe_ssh: bool,
    probe_selection: Option<HostSelection>,
) -> Result<()> {
    let local = collect_doctor_local(config_override)?;

    let probe_data: Option<(Vec<HostHealthResult>, usize)> = if probe_ssh {
        let selection = probe_selection.unwrap_or(HostSelection::All);
        let (results, limit) =
            collect_health_check_batch(&selection, config_override.map(Path::to_path_buf)).await?;
        Some((results, limit))
    } else {
        None
    };

    if json {
        let ssh_probe = match &probe_data {
            None => serde_json::Value::Null,
            Some((results, limit)) => {
                let failures = results.iter().filter(|h| !h.ok).count();
                serde_json::json!({
                    "event": "health-check-batch",
                    "max_concurrency": u32::try_from(*limit).unwrap_or(u32::MAX),
                    "hosts": results.len(),
                    "failures": failures,
                    "results": results.iter().map(|h| serde_json::json!({
                        "name": h.name,
                        "status": if h.ok { "ok" } else { "error" },
                        "latency_ms": h.latency_ms,
                        "error": h.error,
                    })).collect::<Vec<_>>(),
                })
            }
        };
        let probe_ok = probe_data
            .as_ref()
            .is_none_or(|(r, _)| r.iter().all(|h| h.ok));
        let envelope = serde_json::json!({
            "event": "vps-doctor",
            "ok": probe_ok,
            "local": local,
            "ssh_probe": ssh_probe,
        });
        crate::output::print_json_value(&envelope)?;
        if let Some((results, _)) = probe_data {
            let failures = results.iter().filter(|h| !h.ok).count();
            if failures > 0 {
                return Err(SshCliError::Config(format!(
                    "{failures}/{} hosts failed health-check",
                    results.len()
                ))
                .into());
            }
        }
        return Ok(());
    }

    // Text mode: local doctor fields + optional probe section.
    let layer = local
        .get("layer")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let config_path_s = local
        .get("config_path")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let exists = local
        .get("exists")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let perms = local
        .get("permissions")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // G-CLOSE-01: no truncating `as` on JSON numbers — TryFrom + saturate.
    let schema_version = local
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0);
    let hosts = local
        .get("hosts")
        .and_then(serde_json::Value::as_u64)
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(0);
    let kh_s = local
        .get("known_hosts")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let active_s = local
        .get("active_file")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let secrets_at_rest = local
        .get("secrets_at_rest")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let secrets_key_source = local
        .get("secrets_key_source")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let key_file_s = local
        .get("secrets_key_file")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let plaintext_opt_out = local
        .get("secrets_plaintext_opt_out")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    crate::output::print_doctor_text(
        layer,
        config_path_s,
        exists,
        perms,
        schema_version,
        hosts,
        kh_s,
        active_s,
        secrets_at_rest,
        secrets_key_source,
        key_file_s,
        plaintext_opt_out,
    );
    if let Some(rt) = local.get("runtime") {
        crate::output::write_stderr_fmt(format_args!(
            "runtime: os={} arch={} wsl={} container={} ci={} termux={} sandbox={}",
            rt.get("os").and_then(|v| v.as_str()).unwrap_or("?"),
            rt.get("arch").and_then(|v| v.as_str()).unwrap_or("?"),
            rt.get("is_wsl")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            rt.get("is_container")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            rt.get("is_ci")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            rt.get("is_termux")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            rt.get("sandbox")
                .and_then(|v| v.as_str())
                .unwrap_or("none"),
        ))?;
    }
    if let Some((results, limit)) = probe_data {
        crate::output::print_health_batch(&results, limit, false)?;
        let failures = results.iter().filter(|h| !h.ok).count();
        if failures > 0 {
            return Err(SshCliError::Config(format!(
                "{failures}/{} hosts failed health-check",
                results.len()
            ))
            .into());
        }
    }
    Ok(())
}
