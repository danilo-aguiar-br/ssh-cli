// SPDX-License-Identifier: MIT OR Apache-2.0
//! Remote exec / sudo-exec / su-exec (SRP extract — G-COMP-05).
//!
//! Workload: **I/O-bound** SSH. Multi-host fan-out uses
//! [`crate::concurrency::map_bounded`]. Single-host is one-shot
//! connect → run → disconnect (rules one-shot).
//!
//! Secrets: [`secrecy::SecretString`] only; prefer `take` over clone (rules memory).
#![forbid(unsafe_code)]

use super::selection::{resolve_host_jobs, HostSelection};
use super::{
    apply_overrides, build_connection_config, load, resolve_config_path, validate_command_length,
};
use crate::cli::OutputFormat;
use crate::errors::{SshCliError, SshCliResult};
use crate::output;
use crate::ssh::client::{ExecutionOutput, SshClient, SshClientTrait};
use crate::ssh::packing::{append_description, pack_su, pack_sudo};
use crate::vps::model::{effective_limit, VpsRecord};
use anyhow::Result;
use secrecy::SecretString;
use std::path::PathBuf;

/// Common remote execution options.
///
/// G-SECDEV-02: password fields are [`SecretString`] so zeroize-on-drop applies
/// through multi-host clone/fan-out (secrecy 0.10 `SecretString: Clone`).
///
/// G-TYPE-18/19: `timeout` is [`TimeoutMs`]; `steps` are [`RemoteCommand`].
#[derive(Debug, Default, Clone)]
pub struct ExecOptions {
    /// Override password.
    pub password: Option<SecretString>,
    /// Override sudo.
    pub sudo_password: Option<SecretString>,
    /// Override su.
    pub su_password: Option<SecretString>,
    /// Override timeout (refined at CLI boundary).
    pub timeout: Option<crate::domain::TimeoutMs>,
    /// Override key path.
    pub key: Option<String>,
    /// Override key passphrase.
    pub key_passphrase: Option<SecretString>,
    /// Use ssh-agent (G-SSH-04).
    pub use_agent: bool,
    /// Agent socket path (CLI/XDG).
    pub agent_socket: Option<String>,
    /// Optional shell description comment.
    pub description: Option<String>,
    /// replace host key.
    pub replace_host_key: bool,
    /// disable sudo global.
    pub disable_sudo: bool,
    /// Extra commands on the same SSH session after the primary (G-O3 / G-TYPE-19).
    pub steps: Vec<crate::domain::RemoteCommand>,
}

/// Kind of remote elevation for multi-host exec fan-out.
#[derive(Clone, Copy)]
enum ExecKind {
    Plain,
    Sudo,
    Su,
}

/// Per-host result for multi-host exec JSON/text.
#[derive(Debug, Clone)]
pub struct HostExecResult {
    /// VPS name.
    pub name: String,
    /// Whether the remote command succeeded (exit 0).
    pub ok: bool,
    /// Remote exit code when available.
    pub exit_code: Option<i32>,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr or local error text.
    pub stderr: String,
    /// Wall duration in milliseconds.
    pub duration_ms: u64,
    /// Error summary when `ok` is false.
    pub error: Option<String>,
}

/// G-DRY-01: disconnect + print + map non-zero exit (single-host exec family).
///
/// One-shot: always disconnect before returning so the session does not linger.
async fn finish_execution_output(
    client: Box<dyn SshClientTrait>,
    result: SshCliResult<ExecutionOutput>,
    format: OutputFormat,
    json: bool,
) -> Result<()> {
    let _ = client.disconnect().await;
    let output = result?;
    if format == OutputFormat::Json || json {
        output::print_execution_output_json(&output)?;
    } else {
        output::print_execution_output(&output);
    }
    if let Some(code) = output.exit_code {
        if code != 0 {
            return Err(SshCliError::CommandFailed {
                exit_code: code,
                stderr: output.stderr,
            }
            .into());
        }
    }
    Ok(())
}

fn cancelled_err() -> anyhow::Error {
    anyhow::anyhow!(crate::i18n::t(crate::i18n::Message::OperationCancelled))
}

fn expect_single(selection: HostSelection) -> Result<String> {
    match selection {
        HostSelection::Single(name) => Ok(name.into_inner()),
        _ => Err(SshCliError::InvalidArgument(
            "internal: expected single-host selection for non-batch exec".into(),
        )
        .into()),
    }
}

/// Runs a shell command on one VPS or a multi-host selection (bounded).
///
/// Workload: **I/O-bound** SSH. Multi-host (`All` / `Named`) uses
/// [`crate::concurrency::map_bounded`]. Batch JSON when [`HostSelection::is_batch`].
#[allow(clippy::too_many_arguments)]
pub async fn run_exec(
    selection: HostSelection,
    command: &str,
    config_override: Option<PathBuf>,
    format: OutputFormat,
    json: bool,
    opts: ExecOptions,
) -> Result<()> {
    if crate::signals::should_stop() {
        return Err(cancelled_err());
    }
    if selection.is_batch() {
        return run_exec_all(
            &selection,
            command,
            config_override,
            format,
            json,
            opts,
            ExecKind::Plain,
        )
        .await;
    }
    let vps_name = expect_single(selection)?;
    let path = resolve_config_path(config_override.as_deref())?;
    let mut file = load(&path)?;
    // Move the record out of the local map (file is discarded after connect setup).
    let mut vps = file
        .hosts
        .remove(&vps_name)
        .ok_or(SshCliError::VpsNotFound(vps_name))?;

    apply_overrides(
        &mut vps,
        opts.password,
        opts.sudo_password,
        opts.su_password,
        opts.timeout,
        opts.key,
        opts.key_passphrase,
        opts.use_agent,
        opts.agent_socket,
    );
    let cmd = append_description(command, opts.description.as_deref());
    validate_command_length(&cmd, vps.max_command_chars.wire())?;
    for s in &opts.steps {
        validate_command_length(s.as_str(), vps.max_command_chars.wire())?;
    }
    let cfg = build_connection_config(&vps, Some(&path), opts.replace_host_key);
    let client: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
    run_exec_with_client_steps(&vps, &cmd, &opts.steps, client, format, json).await
}

/// Testable version of run_exec.
pub async fn run_exec_with_client(
    vps: &VpsRecord,
    command: &str,
    client: Box<dyn SshClientTrait>,
    format: OutputFormat,
    json: bool,
) -> Result<()> {
    run_exec_with_client_steps(vps, command, &[], client, format, json).await
}

/// G-O3: one SSH session, primary command + optional extra `--step` commands.
pub async fn run_exec_with_client_steps(
    vps: &VpsRecord,
    command: &str,
    steps: &[crate::domain::RemoteCommand],
    mut client: Box<dyn SshClientTrait>,
    format: OutputFormat,
    json: bool,
) -> Result<()> {
    if crate::signals::should_stop() {
        return Err(cancelled_err());
    }
    let max_out = effective_limit(vps.max_output_chars.wire());
    let mut cmds: Vec<&str> = Vec::with_capacity(1 + steps.len());
    cmds.push(command);
    for s in steps {
        cmds.push(s.as_str());
    }
    let mut last_output = None;
    let mut failed: Option<(i32, String)> = None;
    for (i, cmd) in cmds.iter().enumerate() {
        if crate::signals::should_stop() {
            let _ = client.disconnect().await;
            return Err(cancelled_err());
        }
        tracing::debug!(step = i, "exec multi-cmd step");
        match client.run_command(cmd, max_out, None).await {
            Ok(output) => {
                if format == OutputFormat::Json || json {
                    // Multi-step: emit one JSON line per step with index.
                    let mut v = serde_json::to_value(crate::json_wire::ExecutionJson::from(&output))
                        .unwrap_or_else(|_| serde_json::json!({}));
                    if let Some(obj) = v.as_object_mut() {
                        obj.insert("step".into(), serde_json::json!(i));
                        obj.insert("command".into(), serde_json::json!(cmd));
                    }
                    crate::output::print_json_value(&v)?;
                } else if cmds.len() > 1 {
                    crate::output::write_line_fmt(format_args!("--- step {i}: {cmd} ---"))?;
                    crate::output::print_execution_output(&output);
                } else {
                    last_output = Some(output.clone());
                }
                if let Some(code) = output.exit_code {
                    if code != 0 && failed.is_none() {
                        failed = Some((code, output.stderr.clone()));
                    }
                }
                if cmds.len() == 1 {
                    last_output = Some(output);
                }
            }
            Err(e) => {
                let _ = client.disconnect().await;
                return Err(e.into());
            }
        }
    }
    let _ = client.disconnect().await;
    if let Some((code, stderr)) = failed {
        return Err(SshCliError::CommandFailed {
            exit_code: code,
            stderr,
        }
        .into());
    }
    if let Some(output) = last_output {
        if format == OutputFormat::Json || json {
            // already printed above for multi; single without json path:
            if cmds.len() == 1 {
                crate::output::print_execution_output_json(&output)?;
            }
        } else if cmds.len() == 1 {
            crate::output::print_execution_output(&output);
        }
    }
    Ok(())
}

/// Runs a command with `sudo` (packed via `sh -c`).
///
/// Workload: **I/O-bound** SSH. Multi-host uses [`crate::concurrency::map_bounded`].
#[allow(clippy::too_many_arguments)]
pub async fn run_sudo_exec(
    selection: HostSelection,
    command: &str,
    config_override: Option<PathBuf>,
    format: OutputFormat,
    json: bool,
    opts: ExecOptions,
) -> Result<()> {
    if crate::signals::should_stop() {
        return Err(cancelled_err());
    }
    if selection.is_batch() {
        return run_exec_all(
            &selection,
            command,
            config_override,
            format,
            json,
            opts,
            ExecKind::Sudo,
        )
        .await;
    }
    let vps_name = expect_single(selection)?;
    let path = resolve_config_path(config_override.as_deref())?;
    let mut file = load(&path)?;
    let mut vps = file
        .hosts
        .remove(&vps_name)
        .ok_or(SshCliError::VpsNotFound(vps_name))?;

    apply_overrides(
        &mut vps,
        opts.password,
        opts.sudo_password,
        opts.su_password,
        opts.timeout,
        opts.key,
        opts.key_passphrase,
        opts.use_agent,
        opts.agent_socket,
    );
    if opts.disable_sudo || vps.disable_sudo {
        return Err(SshCliError::SudoDisabled.into());
    }
    let cmd = append_description(command, opts.description.as_deref());
    validate_command_length(&cmd, vps.max_command_chars.wire())?;
    let cfg = build_connection_config(&vps, Some(&path), opts.replace_host_key);
    let client: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
    run_sudo_exec_with_client(&vps, &cmd, client, format, json).await
}

/// Testable version of sudo-exec.
pub async fn run_sudo_exec_with_client(
    vps: &VpsRecord,
    command: &str,
    client: Box<dyn SshClientTrait>,
    format: OutputFormat,
    json: bool,
) -> Result<()> {
    if crate::signals::should_stop() {
        return Err(cancelled_err());
    }
    if vps.disable_sudo {
        return Err(SshCliError::SudoDisabled.into());
    }
    let mut pack = pack_sudo(command, vps.sudo_password.as_ref());
    let max_out = effective_limit(vps.max_output_chars.wire());
    let stdin = pack.take_stdin();
    let mut client = client;
    let result = client.run_command(&pack.command, max_out, stdin).await;
    finish_execution_output(client, result, format, json).await
}

/// Runs a command via `su -` one-shot (consumes `su_password`).
///
/// Workload: **I/O-bound** SSH. Multi-host uses [`crate::concurrency::map_bounded`].
#[allow(clippy::too_many_arguments)]
pub async fn run_su_exec(
    selection: HostSelection,
    command: &str,
    config_override: Option<PathBuf>,
    format: OutputFormat,
    json: bool,
    opts: ExecOptions,
) -> Result<()> {
    if crate::signals::should_stop() {
        return Err(cancelled_err());
    }
    if selection.is_batch() {
        return run_exec_all(
            &selection,
            command,
            config_override,
            format,
            json,
            opts,
            ExecKind::Su,
        )
        .await;
    }
    let vps_name = expect_single(selection)?;
    let path = resolve_config_path(config_override.as_deref())?;
    let mut file = load(&path)?;
    let mut vps = file
        .hosts
        .remove(&vps_name)
        .ok_or(SshCliError::VpsNotFound(vps_name))?;

    apply_overrides(
        &mut vps,
        opts.password,
        opts.sudo_password,
        opts.su_password,
        opts.timeout,
        opts.key,
        opts.key_passphrase,
        opts.use_agent,
        opts.agent_socket,
    );
    if opts.disable_sudo || vps.disable_sudo {
        return Err(SshCliError::SudoDisabled.into());
    }
    // `take` moves the secret out of the record (no clone of SecretString).
    let su_password = vps.su_password.take().ok_or(SshCliError::SuPasswordMissing)?;
    let cmd = append_description(command, opts.description.as_deref());
    validate_command_length(&cmd, vps.max_command_chars.wire())?;
    let mut pack = pack_su(&cmd, &su_password);
    let cfg = build_connection_config(&vps, Some(&path), opts.replace_host_key);
    let mut client: Box<dyn SshClientTrait> =
        <SshClient as SshClientTrait>::connect(cfg).await?;
    let max_out = effective_limit(vps.max_output_chars.wire());
    let stdin = pack.take_stdin();
    let result = client.run_command(&pack.command, max_out, stdin).await;
    finish_execution_output(client, result, format, json).await
}

/// Multi-host exec/sudo/su with bounded concurrency (I/O-bound SSH).
///
/// Uses [`resolve_host_jobs`] so `--all` and `--hosts` share one gate (G-PAR-31).
#[allow(clippy::too_many_arguments)]
async fn run_exec_all(
    selection: &HostSelection,
    command: &str,
    config_override: Option<PathBuf>,
    format: OutputFormat,
    json: bool,
    opts: ExecOptions,
    kind: ExecKind,
) -> Result<()> {
    let path = resolve_config_path(config_override.as_deref())?;
    let file = load(&path)?;
    let jobs = resolve_host_jobs(selection, &file)?;
    let limit = crate::concurrency::effective_limit();
    let cmd_base = command.to_string();
    let path_c = path.clone();
    // G-O6: Arc options — clone Arc per task, not full SecretString bundle by accident.
    let opts_c = std::sync::Arc::new(opts);
    let replace = opts_c.replace_host_key;
    let total_jobs = jobs.len();

    tracing::info!(
        hosts = jobs.len(),
        max_concurrency = limit,
        fail_fast = crate::concurrency::fail_fast_enabled(),
        kind = ?match kind {
            ExecKind::Plain => "exec",
            ExecKind::Sudo => "sudo-exec",
            ExecKind::Su => "su-exec",
        },
        "multi-host exec fan-out"
    );

    let results = crate::concurrency::map_bounded_with(
        jobs,
        limit,
        move |(name, mut vps)| {
        let cmd_base = cmd_base.clone();
        let path_c = path_c.clone();
        let opts_arc = std::sync::Arc::clone(&opts_c);
        async move {
            let mut opts = (*opts_arc).clone();

            if crate::signals::should_stop() {
                return HostExecResult {
                    name,
                    ok: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: "cancelled".into(),
                    duration_ms: 0,
                    error: Some("operation cancelled by signal".into()),
                };
            }
            apply_overrides(
                &mut vps,
                opts.password.take(),
                opts.sudo_password.take(),
                opts.su_password.take(),
                opts.timeout,
                opts.key.take(),
                opts.key_passphrase.take(),
                opts.use_agent,
                opts.agent_socket.take(),
            );
            let cmd = append_description(&cmd_base, opts.description.as_deref());
            if let Err(e) = validate_command_length(&cmd, vps.max_command_chars.wire()) {
                return HostExecResult {
                    name,
                    ok: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: e.to_string(),
                    duration_ms: 0,
                    error: Some(e.to_string()),
                };
            }
            match kind {
                ExecKind::Sudo | ExecKind::Su if opts.disable_sudo || vps.disable_sudo => {
                    return HostExecResult {
                        name,
                        ok: false,
                        exit_code: None,
                        stdout: String::new(),
                        stderr: "sudo/su disabled".into(),
                        duration_ms: 0,
                        error: Some("sudo/su disabled".into()),
                    };
                }
                _ => {}
            }
            let start = std::time::Instant::now();
            let run = async {
                let cfg = build_connection_config(&vps, Some(&path_c), replace);
                let mut client: Box<dyn SshClientTrait> =
                    <SshClient as SshClientTrait>::connect(cfg).await?;
                let max_out = effective_limit(vps.max_output_chars.wire());
                let output = match kind {
                    ExecKind::Plain => client.run_command(&cmd, max_out, None).await?,
                    ExecKind::Sudo => {
                        let mut pack = pack_sudo(&cmd, vps.sudo_password.as_ref());
                        let stdin = pack.take_stdin();
                        client.run_command(&pack.command, max_out, stdin).await?
                    }
                    ExecKind::Su => {
                        let su_pw = vps
                            .su_password
                            .take()
                            .ok_or(SshCliError::SuPasswordMissing)?;
                        let mut pack = pack_su(&cmd, &su_pw);
                        let stdin = pack.take_stdin();
                        client.run_command(&pack.command, max_out, stdin).await?
                    }
                };
                let _ = client.disconnect().await;
                Ok::<_, SshCliError>(output)
            }
            .await;
            let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            match run {
                Ok(output) => {
                    let code_ok = output.exit_code.unwrap_or(0) == 0;
                    HostExecResult {
                        name,
                        ok: code_ok,
                        exit_code: output.exit_code,
                        stdout: output.stdout,
                        stderr: output.stderr,
                        duration_ms,
                        error: if code_ok {
                            None
                        } else {
                            Some(format!(
                                "exit {}",
                                output.exit_code.unwrap_or(-1)
                            ))
                        },
                    }
                }
                Err(e) => HostExecResult {
                    name,
                    ok: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: e.to_string(),
                    duration_ms,
                    error: Some(e.to_string()),
                },
            }
        }
    },
        |h: &HostExecResult| !h.ok,
    )
    .await;

    let mut host_results = Vec::with_capacity(total_jobs.max(results.len()));
    let mut failures = 0usize;
    let mut seen = std::collections::BTreeSet::new();
    for r in results {
        match r.outcome {
            Ok(h) => {
                if !h.ok {
                    failures += 1;
                }
                seen.insert(r.index);
                host_results.push(h);
            }
            Err(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
            Err(e) => {
                failures += 1;
                seen.insert(r.index);
                host_results.push(HostExecResult {
                    name: format!("task-{}", r.index),
                    ok: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: e.to_string(),
                    duration_ms: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    // G-O1: pad skipped hosts when fail-fast stopped admission mid-fleet.
    if crate::concurrency::fail_fast_enabled() && host_results.len() < total_jobs {
        let skipped = total_jobs - host_results.len();
        for i in 0..total_jobs {
            if !seen.contains(&i) {
                host_results.push(HostExecResult {
                    name: format!("skipped-{i}"),
                    ok: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: "skipped (fail-fast)".into(),
                    duration_ms: 0,
                    error: Some("skipped (fail-fast)".into()),
                });
            }
        }
        let _ = skipped;
    }

    let as_json = format == OutputFormat::Json || json;
    output::print_exec_batch(&host_results, limit, as_json)?;
    if failures > 0 || host_results.iter().any(|h| !h.ok) {
        let failed = host_results.iter().filter(|h| !h.ok).count();
        return Err(SshCliError::Config(format!(
            "{failed}/{} hosts failed multi-host exec",
            host_results.len()
        ))
        .into());
    }
    Ok(())
}
