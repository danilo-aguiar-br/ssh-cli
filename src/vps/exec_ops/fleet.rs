// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded multi-host exec fan-out (A7 split).
#![forbid(unsafe_code)]
#![allow(unused_imports)]
use super::*;

/// Multi-host exec/sudo/su with bounded concurrency (I/O-bound SSH).
///
/// Uses [`resolve_host_jobs`] so `--all` and `--hosts` share one gate (G-PAR-31).
pub(crate) async fn run_exec_all(
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
    // Kept outside the fan-out so hosts that are never admitted can still be reported
    // by name instead of by task index.
    let job_names: Vec<String> = jobs.iter().map(|(name, _)| name.clone()).collect();

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
                apply_overrides(&mut vps, opts.take_auth_overrides());
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
                // G-O3 parity with the single-host paths: run the primary command and
                // every `--step` on the same session instead of silently dropping them.
                let labels = step_labels(&cmd, &opts.steps);
                for extra in labels.iter().skip(1) {
                    if let Err(e) = validate_command_length(extra, vps.max_command_chars.wire()) {
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
                }
                let start = std::time::Instant::now();
                let run = async {
                    let cfg = build_connection_config(&vps, Some(&path_c), replace);
                    let mut client: Box<dyn SshClientTrait> =
                        <SshClient as SshClientTrait>::connect(cfg).await?;
                    let max_out = effective_limit(vps.max_output_chars.wire());
                    // `su` consumes the record secret once; each step re-packs from it.
                    let su_pw = match kind {
                        ExecKind::Su => Some(
                            vps.su_password
                                .take()
                                .ok_or(SshCliError::SuPasswordMissing)?,
                        ),
                        _ => None,
                    };
                    let multi = labels.len() > 1;
                    let mut stdout = String::new();
                    let mut stderr = String::new();
                    let mut exit_code: Option<i32> = None;
                    for (i, raw) in labels.iter().enumerate() {
                        let mut pack = match (kind, su_pw.as_ref()) {
                            (ExecKind::Plain, _) => PackedCommand {
                                command: raw.clone(),
                                stdin: None,
                            },
                            (ExecKind::Sudo, _) => pack_sudo(raw, vps.sudo_password.as_ref()),
                            (ExecKind::Su, Some(pw)) => pack_su(raw, pw),
                            // Unreachable: `su_pw` is Some for ExecKind::Su (set above).
                            (ExecKind::Su, None) => return Err(SshCliError::SuPasswordMissing),
                        };
                        let stdin = pack.take_stdin();
                        let output = client.run_command(&pack.command, max_out, stdin).await?;
                        if multi {
                            stdout.push_str(&format!("--- step {i}: {raw} ---\n"));
                        }
                        stdout.push_str(&output.stdout);
                        stderr.push_str(&output.stderr);
                        // First non-zero wins: it is the step the agent must inspect.
                        match (exit_code, output.exit_code) {
                            (None, code) => exit_code = code,
                            (Some(0), Some(code)) if code != 0 => exit_code = Some(code),
                            _ => {}
                        }
                    }
                    let _ = client.disconnect().await;
                    Ok::<_, SshCliError>((stdout, stderr, exit_code))
                }
                .await;
                let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                match run {
                    Ok((stdout, stderr, exit_code)) => {
                        let code_ok = exit_code.unwrap_or(0) == 0;
                        HostExecResult {
                            name,
                            ok: code_ok,
                            exit_code,
                            stdout,
                            stderr,
                            duration_ms,
                            error: if code_ok {
                                None
                            } else {
                                Some(format!("exit {}", exit_code.unwrap_or(-1)))
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
    let mut seen = std::collections::BTreeSet::new();
    for r in results {
        // A join error is still that host's outcome: report it under the real name so
        // the agent can act on it, not under an opaque task index.
        let name = job_names
            .get(r.index)
            .cloned()
            .unwrap_or_else(|| format!("task-{}", r.index));
        match r.outcome {
            Ok(h) => {
                seen.insert(r.index);
                host_results.push(h);
            }
            Err(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
            Err(e) => {
                seen.insert(r.index);
                host_results.push(HostExecResult {
                    name,
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
    // G-O1: every requested host must appear with an explicit state. Hosts that were
    // never admitted (fail-fast or cooperative cancel) are reported by their real name
    // as "not attempted", so the agent can tell them apart from hosts that ran and
    // failed — the old synthetic `skipped-{i}` entries hid which target was skipped.
    if host_results.len() < total_jobs {
        let reason = if crate::concurrency::fail_fast_enabled() {
            "not attempted (fail-fast stopped admission)"
        } else {
            "not attempted (fan-out stopped before admission)"
        };
        for (i, name) in job_names.iter().enumerate() {
            if !seen.contains(&i) {
                host_results.push(HostExecResult {
                    name: name.clone(),
                    ok: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: reason.into(),
                    duration_ms: 0,
                    error: Some(reason.into()),
                });
            }
        }
    }

    let as_json = format == OutputFormat::Json || json;
    output::print_exec_batch(&host_results, limit, as_json)?;
    // Denominator is every host asked for, including the ones never attempted.
    let failed = host_results.iter().filter(|h| !h.ok).count();
    finish_batch(failed, total_jobs, "exec")?;
    Ok(())
}
