// SPDX-License-Identifier: MIT OR Apache-2.0
//! `sudo-exec` and `su-exec` entry points (A7 split).
#![forbid(unsafe_code)]
#![allow(unused_imports)]
use super::*;

/// Runs a command with `sudo` (packed via `sh -c`).
///
/// Workload: **I/O-bound** SSH. Multi-host uses [`crate::concurrency::map_bounded`].
pub async fn run_sudo_exec(
    selection: HostSelection,
    command: &str,
    config_override: Option<PathBuf>,
    format: OutputFormat,
    json: bool,
    mut opts: ExecOptions,
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

    apply_overrides(&mut vps, opts.take_auth_overrides());
    if opts.disable_sudo || vps.disable_sudo {
        return Err(SshCliError::SudoDisabled.into());
    }
    let cmd = append_description(command, opts.description.as_deref());
    validate_command_length(&cmd, vps.max_command_chars.wire())?;
    for s in &opts.steps {
        validate_command_length(s.as_str(), vps.max_command_chars.wire())?;
    }
    let cfg = build_connection_config(&vps, Some(&path), opts.replace_host_key);
    let client: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
    run_sudo_exec_with_client_steps(&vps, &cmd, &opts.steps, client, format, json).await
}

/// Testable version of sudo-exec.
pub async fn run_sudo_exec_with_client(
    vps: &VpsRecord,
    command: &str,
    client: Box<dyn SshClientTrait>,
    format: OutputFormat,
    json: bool,
) -> Result<()> {
    run_sudo_exec_with_client_steps(vps, command, &[], client, format, json).await
}

/// G-O3 parity: `sudo-exec` primary command plus `--step` commands on one session.
///
/// Each step is packed on its own because `sudo -S` consumes the password from the
/// channel stdin per command; reusing a single [`PackedCommand`] would leave the
/// later steps without credentials.
pub async fn run_sudo_exec_with_client_steps(
    vps: &VpsRecord,
    command: &str,
    steps: &[crate::domain::RemoteCommand],
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
    let prepared = step_labels(command, steps)
        .iter()
        .map(|c| PreparedStep::packed(c, pack_sudo(c, vps.sudo_password.as_ref())))
        .collect();
    run_prepared_steps(vps, prepared, client, format, json).await
}

/// Runs a command via `su -` one-shot (consumes `su_password`).
///
/// Workload: **I/O-bound** SSH. Multi-host uses [`crate::concurrency::map_bounded`].
pub async fn run_su_exec(
    selection: HostSelection,
    command: &str,
    config_override: Option<PathBuf>,
    format: OutputFormat,
    json: bool,
    mut opts: ExecOptions,
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

    apply_overrides(&mut vps, opts.take_auth_overrides());
    if opts.disable_sudo || vps.disable_sudo {
        return Err(SshCliError::SudoDisabled.into());
    }
    // `take` moves the secret out of the record (no clone of SecretString).
    let su_password = vps
        .su_password
        .take()
        .ok_or(SshCliError::SuPasswordMissing)?;
    let cmd = append_description(command, opts.description.as_deref());
    validate_command_length(&cmd, vps.max_command_chars.wire())?;
    for s in &opts.steps {
        validate_command_length(s.as_str(), vps.max_command_chars.wire())?;
    }
    // G-O3 parity: every step gets its own `su - -c` pack so each one receives the
    // password on stdin; previously `--step` was silently dropped on this path.
    let prepared = step_labels(&cmd, &opts.steps)
        .iter()
        .map(|c| PreparedStep::packed(c, pack_su(c, &su_password)))
        .collect();
    let cfg = build_connection_config(&vps, Some(&path), opts.replace_host_key);
    let client: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
    run_prepared_steps(&vps, prepared, client, format, json).await
}
