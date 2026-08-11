// SPDX-License-Identifier: MIT OR Apache-2.0
//! Single-host exec and the shared multi-step engine (A7 split).
#![forbid(unsafe_code)]
#![allow(unused_imports)]
use super::*;

/// Runs a shell command on one VPS or a multi-host selection (bounded).
///
/// Workload: **I/O-bound** SSH. Multi-host (`All` / `Named`) uses
/// [`crate::concurrency::map_bounded`]. Batch JSON when [`HostSelection::is_batch`].
pub async fn run_exec(
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
            ExecKind::Plain,
        )
        .await;
    }
    let vps_name = expect_single(selection)?;
    // GAP-SSH-EXEC-ENVELOPE-002: capture the identity before the name is consumed by
    // the registry lookup, so every emitted event can name its own target.
    let target = crate::json_wire::ExecTarget::new(vps_name.clone(), opts.target_source);
    let path = resolve_config_path(config_override.as_deref())?;
    let mut file = load(&path)?;
    // Move the record out of the local map (file is discarded after connect setup).
    let mut vps = file
        .hosts
        .remove(&vps_name)
        .ok_or(SshCliError::VpsNotFound(vps_name))?;
    // GAP-SSH-EXEC-ENVELOPE-002: publish the target only once the lookup succeeded,
    // so every failure from here on names its host. Publishing before the `?` above
    // would make a `VpsNotFound` envelope claim a host it never resolved.
    crate::json_wire::set_resolved_target(&target);

    apply_overrides(&mut vps, opts.take_auth_overrides());
    let cmd = append_description(command, opts.description.as_deref());
    validate_command_length(&cmd, vps.max_command_chars.wire())?;
    for s in &opts.steps {
        validate_command_length(s.as_str(), vps.max_command_chars.wire())?;
    }
    let cfg = build_connection_config(&vps, Some(&path), opts.replace_host_key);
    let client: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
    run_exec_with_client_steps(&vps, &cmd, &opts.steps, client, format, json, &target).await
}

/// Testable version of run_exec.
pub async fn run_exec_with_client(
    vps: &VpsRecord,
    command: &str,
    client: Box<dyn SshClientTrait>,
    format: OutputFormat,
    json: bool,
) -> Result<()> {
    let target =
        crate::json_wire::ExecTarget::new(vps.name.as_str(), crate::json_wire::TargetSource::Argv);
    run_exec_with_client_steps(vps, command, &[], client, format, json, &target).await
}

/// One remote step ready for the wire.
///
/// `label` is the raw command the caller typed; `packed` is what actually goes over
/// the channel (plain for `exec`, `sudo -S … sh -c` / `su - -c` for the elevated
/// paths, with the password on stdin). Keeping both apart lets every elevation kind
/// share one step loop while output still shows the command the user wrote.
pub(crate) struct PreparedStep {
    /// Raw command echoed in JSON (`command`) and in the text step header.
    label: String,
    /// Wire form plus optional stdin payload (zeroized on drop).
    packed: PackedCommand,
}

impl PreparedStep {
    /// Step with no elevation wrapper (`exec`).
    pub(crate) fn plain(command: &str) -> Self {
        Self {
            label: command.to_owned(),
            packed: PackedCommand {
                command: command.to_owned(),
                stdin: None,
            },
        }
    }

    /// Step from an already packed elevation command (`sudo-exec` / `su-exec`).
    pub(crate) fn packed(label: &str, packed: PackedCommand) -> Self {
        Self {
            label: label.to_owned(),
            packed,
        }
    }
}

/// Builds `primary + --step …` as raw command strings (description only on the primary).
pub(crate) fn step_labels(command: &str, steps: &[crate::domain::RemoteCommand]) -> Vec<String> {
    let mut out = Vec::with_capacity(1 + steps.len());
    out.push(command.to_owned());
    out.extend(steps.iter().map(|s| s.as_str().to_owned()));
    out
}

/// G-O3: runs every prepared step on **one** SSH session, then disconnects.
///
/// Shared by `exec`, `sudo-exec` and `su-exec`: the elevated paths used to ignore
/// `--step` entirely and still exit 0, which reported a partial run as a full one.
/// Every step is executed here, the first non-zero exit is remembered, and the
/// remaining steps still run so their output is not lost.
///
/// One exception, and it is a security one: exit **127 on step zero** aborts the
/// batch immediately (GAP-SSH-EXEC-ARGC-001 rule 6). That code on the first step is
/// the fingerprint of a host name having been promoted to a command, so continuing
/// would run the remaining steps against a machine the caller never designated.
/// Every other non-zero exit, at any index, keeps the run-everything contract.
pub(crate) async fn run_prepared_steps(
    vps: &VpsRecord,
    steps: Vec<PreparedStep>,
    mut client: Box<dyn SshClientTrait>,
    format: OutputFormat,
    json: bool,
    target: &crate::json_wire::ExecTarget,
) -> Result<()> {
    if crate::signals::should_stop() {
        return Err(cancelled_err());
    }
    let max_out = effective_limit(vps.max_output_chars.wire());
    let as_json = format == OutputFormat::Json || json;
    let multi = steps.len() > 1;
    let mut last_output: Option<ExecutionOutput> = None;
    let mut failed: Option<(i32, String)> = None;
    for (i, mut step) in steps.into_iter().enumerate() {
        if crate::signals::should_stop() {
            let _ = client.disconnect().await;
            return Err(cancelled_err());
        }
        tracing::debug!(step = i, "exec multi-cmd step");
        // Move stdin out so the password is written once and zeroized by `run_command`.
        let stdin = step.packed.take_stdin();
        match client
            .run_command(&step.packed.command, max_out, stdin)
            .await
        {
            Ok(output) => {
                if let Some(code) = output.exit_code {
                    if code != 0 && failed.is_none() {
                        failed = Some((code, output.stderr.clone()));
                    }
                    // GAP-SSH-EXEC-ARGC-001 rule 6: 127 on step *zero* is the
                    // signature of the misdirection itself. When a host name is
                    // promoted to a command, the shell answers `command not found`
                    // and every later `--step` then runs on a machine nobody named,
                    // which is exactly how one incident aggregated to exit 0.
                    //
                    // Scoped to index zero on purpose. A later step returning 127 is
                    // an ordinary missing binary and keeps the existing contract:
                    // remaining steps run so their output is not lost. Index zero is
                    // different because nothing has been established yet — there is
                    // no evidence the session is aimed where the caller believes.
                    if i == 0 && code == 127 {
                        let _ = client.disconnect().await;
                        return Err(SshCliError::CommandFailed {
                            exit_code: code,
                            stderr: output.stderr,
                        }
                        .into());
                    }
                }
                // G8: multi-step emits one document per step (JSON and text alike);
                // single-step must emit exactly one object, printed after the loop.
                if multi && as_json {
                    let mut v = serde_json::to_value(crate::json_wire::ExecutionJson::with_target(
                        &output, target,
                    ))
                    .unwrap_or_else(|_| serde_json::json!({}));
                    if let Some(obj) = v.as_object_mut() {
                        obj.insert("step".into(), serde_json::json!(i));
                        obj.insert("command".into(), serde_json::json!(step.label));
                    }
                    crate::output::print_json_value(&v)?;
                } else if multi {
                    crate::output::write_line_fmt(format_args!(
                        "--- step {i}: {} ---",
                        step.label
                    ))?;
                    crate::output::print_execution_output(&output);
                } else {
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
    // Print before failing: a non-zero remote exit still produced output the caller
    // needs (parity with the previous single-command elevated path).
    if let Some(output) = last_output {
        if as_json {
            crate::output::print_execution_output_json(&output, target)?;
        } else {
            crate::output::print_execution_output(&output);
        }
    }
    if let Some((code, stderr)) = failed {
        return Err(SshCliError::CommandFailed {
            exit_code: code,
            stderr,
        }
        .into());
    }
    Ok(())
}

/// G-O3: one SSH session, primary command + optional extra `--step` commands.
pub async fn run_exec_with_client_steps(
    vps: &VpsRecord,
    command: &str,
    steps: &[crate::domain::RemoteCommand],
    client: Box<dyn SshClientTrait>,
    format: OutputFormat,
    json: bool,
    target: &crate::json_wire::ExecTarget,
) -> Result<()> {
    let prepared = step_labels(command, steps)
        .iter()
        .map(|c| PreparedStep::plain(c))
        .collect();
    run_prepared_steps(vps, prepared, client, format, json, target).await
}
