// SPDX-License-Identifier: MIT OR Apache-2.0
//! Subcommand dispatch (G-COMP-06b) — kept separate from clap type definitions.
//!
//! Sequential local handlers (CRUD, locale, completions) are justified: work ≪ SSH RTT.
//! Multi-host I/O uses domain modules with [`crate::concurrency::map_bounded`].
#![forbid(unsafe_code)]

use super::locale_cmd::run_locale_command;
use super::targeting::{
    build_exec_options, resolve_exec_target, resolve_health_target, Elevation, ExecCommonArgs,
    ExecTargetArgs, HealthTargetArgs,
};
use super::{
    command_tree_json, effective_timeout_ms, generate_completions, read_stdin_if,
    warn_if_password_argv, CliArgs, Command, OutputFormat, ScpAction, SecretsAction, SftpAction,
};
use anyhow::Result;

/// Runs the requested subcommand; prefer [`crate::commands::run`] from new call sites.
///
/// # Errors
/// Returns the error of the dispatched subcommand.
pub async fn dispatch(args: CliArgs) -> Result<()> {
    dispatch_impl(args).await
}

/// Shared dispatch implementation (cli + commands layers).
///
/// # Errors
/// Returns the error of the dispatched subcommand.
pub async fn dispatch_impl(args: CliArgs) -> Result<()> {
    let config_override = args.config_dir.clone();
    // Aligns `secrets.key` with `--config-dir` / isolated tests.
    crate::secrets::set_config_dir(config_override.clone());
    // The leaf flag `secrets init --keyring` selects the same store as the global
    // `--use-keyring`, so it must reach the runtime state too. Without this the
    // command wrote to the keyring and then reported `key_source: none`, because
    // `secrets_status()` re-resolves the key from flags alone.
    let leaf_keyring = matches!(
        &args.command,
        Command::Secrets {
            action: SecretsAction::Init { keyring: true, .. }
        }
    );
    crate::secrets::set_runtime_flags(
        args.allow_plaintext_secrets,
        args.secrets_key_file.clone(),
        args.use_keyring || leaf_keyring,
    );
    // Bounded multi-host fan-out budget (Rules Rust — paralelismo).
    let limit = crate::concurrency::resolve_limit(args.max_concurrency.map(usize::from));
    crate::concurrency::install_process_limit(limit);
    crate::concurrency::install_fail_fast(args.fail_fast);
    if let Some(n) = args.scp_file_concurrency {
        crate::concurrency::install_scp_file_concurrency(usize::from(n));
    }
    // G-AUD-01: global `--json` forces JSON; conflicts with `--output-format text`.
    let formato = super::resolve_format_from_cli(args.json, args.output_format)?;
    // GAP-SSH-IO-003 / IO-004: centralized I/O policy.
    crate::output::set_quiet(args.quiet);
    crate::output::set_json_errors(formato == OutputFormat::Json);
    let disable_sudo = args.disable_sudo;
    let replace_host_key = args.replace_host_key;
    // G-OS-02: global `--timeout` fills missing local timeouts (local wins).
    let global_timeout = args.timeout;

    // GAP-AUD-010 / G-AUD-08: warn only when secrets appear on argv (visible in `ps`).
    warn_if_password_argv(&args);

    // C2: rejected before any registry read or socket, but *after* the error
    // channel knows whether the caller wants JSON — rejecting earlier produced a
    // prose refusal on stdout-for-agents, so the one command that fails this
    // check was also the one an agent could not parse.
    super::guard_dry_run_supported(&args.command)?;

    match args.command {
        Command::Vps { action } => {
            // Sequential: local TOML CRUD (work ≪ SSH RTT; no multi-host I/O)
            // except `vps doctor --probe-ssh` which reuses health-check fan-out.
            crate::vps::run_vps_command(action, config_override, formato).await
        }
        Command::Connect { name } => {
            // Sequential: writes active marker only (no SSH fan-out).
            crate::vps::run_connect(&name, config_override, formato).await
        }
        Command::Exec {
            all,
            hosts,
            tags,
            use_active,
            target,
            steps,
            json,
            auth,
            timeout,
            description,
        } => {
            let plan = resolve_exec_target(
                ExecTargetArgs {
                    all,
                    hosts,
                    tags,
                    use_active,
                    target,
                },
                config_override.as_deref(),
            )?;
            let opts = build_exec_options(ExecCommonArgs {
                steps,
                auth,
                elevation_password: None,
                elevation: Elevation::None,
                timeout,
                global_timeout,
                description,
                replace_host_key,
                disable_sudo,
                target_source: plan.source,
            })?;
            crate::vps::run_exec(
                plan.selection,
                &plan.command,
                config_override,
                formato,
                json,
                opts,
            )
            .await
        }
        Command::SudoExec {
            all,
            hosts,
            tags,
            use_active,
            target,
            steps,
            json,
            auth,
            sudo_password,
            sudo_password_stdin,
            timeout,
            description,
        } => {
            let plan = resolve_exec_target(
                ExecTargetArgs {
                    all,
                    hosts,
                    tags,
                    use_active,
                    target,
                },
                config_override.as_deref(),
            )?;
            let opts = build_exec_options(ExecCommonArgs {
                steps,
                auth,
                elevation_password: read_stdin_if(sudo_password_stdin, sudo_password)?,
                elevation: Elevation::Sudo,
                timeout,
                global_timeout,
                description,
                replace_host_key,
                disable_sudo,
                target_source: plan.source,
            })?;
            crate::vps::run_sudo_exec(
                plan.selection,
                &plan.command,
                config_override,
                formato,
                json,
                opts,
            )
            .await
        }
        Command::SuExec {
            all,
            hosts,
            tags,
            use_active,
            target,
            steps,
            json,
            auth,
            su_password,
            su_password_stdin,
            timeout,
            description,
        } => {
            let plan = resolve_exec_target(
                ExecTargetArgs {
                    all,
                    hosts,
                    tags,
                    use_active,
                    target,
                },
                config_override.as_deref(),
            )?;
            let opts = build_exec_options(ExecCommonArgs {
                steps,
                auth,
                elevation_password: read_stdin_if(su_password_stdin, su_password)?,
                elevation: Elevation::Su,
                timeout,
                global_timeout,
                description,
                replace_host_key,
                disable_sudo,
                target_source: plan.source,
            })?;
            crate::vps::run_su_exec(
                plan.selection,
                &plan.command,
                config_override,
                formato,
                json,
                opts,
            )
            .await
        }
        Command::Scp { action } => {
            let (auth, timeout, json_local) = match &action {
                ScpAction::Upload {
                    auth,
                    timeout,
                    json,
                    ..
                }
                | ScpAction::Download {
                    auth,
                    timeout,
                    json,
                    ..
                } => (auth.clone(), *timeout, *json),
            };
            let key = auth.key_path_string();
            let password = read_stdin_if(auth.password_stdin, auth.password)?;
            let key_passphrase = read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
            // GAP-SSH-IO-007b: local --json or global --format json → JSON error envelope.
            let json_efetivo = json_local || formato == OutputFormat::Json;
            if json_efetivo {
                crate::output::set_json_errors(true);
            }
            crate::scp::run_scp(
                action,
                config_override,
                crate::scp::ScpOptions {
                    password,
                    key,
                    key_passphrase,
                    timeout: effective_timeout_ms(timeout, global_timeout)
                        .map_err(crate::errors::SshCliError::InvalidArgument)?,
                    replace_host_key,
                    json: json_efetivo,
                    use_agent: auth.use_agent,
                    agent_socket: auth
                        .agent_socket
                        .as_ref()
                        .map(|p| p.to_string_lossy().into_owned()),
                },
            )
            .await
        }
        Command::Sftp { action } => {
            let (auth, timeout, json_local) = match &action {
                SftpAction::Upload {
                    auth,
                    timeout,
                    json,
                    ..
                }
                | SftpAction::Download {
                    auth,
                    timeout,
                    json,
                    ..
                }
                | SftpAction::Ls {
                    auth,
                    timeout,
                    json,
                    ..
                }
                | SftpAction::Mkdir {
                    auth,
                    timeout,
                    json,
                    ..
                }
                | SftpAction::Rmdir {
                    auth,
                    timeout,
                    json,
                    ..
                }
                | SftpAction::Rm {
                    auth,
                    timeout,
                    json,
                    ..
                }
                | SftpAction::Stat {
                    auth,
                    timeout,
                    json,
                    ..
                }
                | SftpAction::Rename {
                    auth,
                    timeout,
                    json,
                    ..
                } => (auth.clone(), *timeout, *json),
            };
            let key = auth.key_path_string();
            let password = read_stdin_if(auth.password_stdin, auth.password)?;
            let key_passphrase = read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
            let json_efetivo = json_local || formato == OutputFormat::Json;
            if json_efetivo {
                crate::output::set_json_errors(true);
            }
            // A6: without `ssh-real` there is no SFTP subsystem to dispatch into. Failing
            // with a typed error keeps the diagnostic build honest — the subcommand still
            // parses, and the caller is told the binary was built without the stack
            // instead of hitting a link error or a silent no-op.
            #[cfg(not(feature = "ssh-real"))]
            {
                let _ = (
                    action,
                    config_override,
                    password,
                    key,
                    key_passphrase,
                    timeout,
                );
                Err(crate::errors::SshCliError::InvalidArgument(
                    "this binary was built without the `ssh-real` feature; sftp is unavailable"
                        .to_string(),
                )
                .into())
            }
            #[cfg(feature = "ssh-real")]
            crate::sftp::run_sftp(
                action,
                config_override,
                crate::sftp::SftpOptions {
                    password,
                    key,
                    key_passphrase,
                    timeout: effective_timeout_ms(timeout, global_timeout)
                        .map_err(crate::errors::SshCliError::InvalidArgument)?,
                    replace_host_key,
                    json: json_efetivo,
                    use_agent: auth.use_agent,
                    agent_socket: auth
                        .agent_socket
                        .as_ref()
                        .map(|p| p.to_string_lossy().into_owned()),
                    recursive: false, // set from action in run_sftp for upload/download
                },
            )
            .await
        }
        Command::Tunnel {
            vps_name,
            local_port,
            remote_host,
            remote_port,
            socks5,
            remote_socket,
            reverse,
            timeout_ms,
            auth,
            json,
            bind,
            i_accept_network_exposure,
        } => {
            // GAP-SSH-IO-008: --json local or global format.
            let json_efetivo = json || formato == OutputFormat::Json;
            if json_efetivo {
                crate::output::set_json_errors(true);
            }
            // GAP-SSH-CLI-005: auth parity with exec/scp (stdin + passphrase).
            // Tunnel deadline remains explicit `--timeout-ms` (mandatory bound).
            // Forwards: JoinSet + Semaphore (concurrency::effective_limit).
            let key = auth.key_path_string();
            let password = read_stdin_if(auth.password_stdin, auth.password)?;
            let key_passphrase = read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
            // E3: `--use-agent` / `--agent-socket` were accepted by clap here and then
            // dropped on the floor, so a host registered for agent auth simply could
            // not open a tunnel. Forwarded now, at parity with exec/scp.
            let mode = super::resolve_tunnel_mode(
                socks5,
                remote_socket,
                reverse,
                remote_host,
                remote_port,
            )?;
            crate::tunnel::run_tunnel(crate::tunnel::TunnelRequest {
                vps_name,
                local_port,
                mode,
                config_override,
                auth: crate::tunnel::TunnelAuth {
                    password,
                    key,
                    key_passphrase,
                    use_agent: auth.use_agent,
                    agent_socket: auth
                        .agent_socket
                        .as_ref()
                        .map(|p| p.to_string_lossy().into_owned()),
                },
                timeout_ms,
                replace_host_key,
                json: json_efetivo,
                bind_addr: bind.to_string(),
                accept_network_exposure: i_accept_network_exposure,
            })
            .await
        }
        Command::HealthCheck {
            vps_name,
            all,
            hosts,
            use_active,
            json,
            auth,
            timeout,
        } => {
            // GAP-SSH-CLI-006: auth parity with exec/scp (stdin + key + passphrase).
            let key = auth.key_path_string();
            let password = read_stdin_if(auth.password_stdin, auth.password)?;
            let key_passphrase = read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
            // Target resolution lives beside the exec-family resolver: deciding
            // *where* a command lands is one responsibility, and it had drifted into
            // two copies with different rules — which is how the read surface kept an
            // ambient shortcut the write surface had already removed.
            let (selection, host_source) = resolve_health_target(
                HealthTargetArgs {
                    all,
                    hosts,
                    use_active,
                    vps_name,
                },
                config_override.as_deref(),
            )?;
            crate::vps::run_health_check(crate::vps::HealthCheckRequest {
                selection,
                config_override,
                format: formato,
                json_local: json,
                password_override: password,
                timeout_override: effective_timeout_ms(timeout, global_timeout)
                    .map_err(crate::errors::SshCliError::InvalidArgument)?,
                key_override: key,
                key_passphrase_override: key_passphrase,
                replace_host_key,
                host_source,
            })
            .await
        }
        Command::Secrets { action } => {
            // Sequential: local crypto / key file (work ≪ SSH RTT).
            crate::vps::run_secrets_command(action, config_override, formato).await
        }
        Command::Completions { shell } => {
            // Sequential: emit shell script metadata only.
            generate_completions(shell)
        }
        Command::Commands { json: _ } => {
            // Sequential: clap tree walk (agent discovery; no I/O fan-out).
            // Always JSON: agent discovery surface (rules checklist `mycli commands`).
            crate::output::print_json_value(&command_tree_json())?;
            Ok(())
        }
        Command::Schema { name, json } => {
            // Sequential: embedded catalog lookup (G-E2E-02).
            let wants_json = json || formato == OutputFormat::Json;
            crate::cli::run_schema(name.as_deref(), wants_json).map_err(Into::into)
        }
        Command::Doctor {
            json,
            probe_ssh,
            hosts,
        } => {
            // G-E2E-03: root alias → same handler as `vps doctor`.
            crate::vps::run_vps_command(
                crate::cli::VpsAction::Doctor {
                    json,
                    probe_ssh,
                    hosts,
                },
                config_override,
                formato,
            )
            .await
        }
        Command::Tls { json, action } => {
            #[cfg(feature = "tls")]
            {
                crate::tls::commands::run_tls_command(action, config_override, formato, json).await
            }
            #[cfg(not(feature = "tls"))]
            {
                let _ = (action, json);
                Err(crate::errors::SshCliError::tls_msg(
                    "TLS feature disabled; rebuild with --features tls (default)",
                )
                .into())
            }
        }
        Command::Locale { json, action } => {
            // Sequential: locale preference file / diagnostics (local only).
            run_locale_command(
                action,
                config_override.as_deref(),
                formato,
                args.lang.as_deref(),
                json,
            )
        }
    }
}
