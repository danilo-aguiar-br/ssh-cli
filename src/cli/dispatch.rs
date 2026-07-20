// SPDX-License-Identifier: MIT OR Apache-2.0
//! Subcommand dispatch (G-COMP-06b) — kept separate from clap type definitions.
//!
//! Sequential local handlers (CRUD, locale, completions) are justified: work ≪ SSH RTT.
//! Multi-host I/O uses domain modules with [`crate::concurrency::map_bounded`].
#![forbid(unsafe_code)]

use super::{
    command_tree_json, effective_timeout_ms, generate_completions, parse_exec_target,
    parse_hosts_list, parse_remote_steps, read_stdin_if, warn_if_password_argv, CliArgs,
    Command, LocaleAction, OutputFormat, ScpAction, SftpAction,
};
use anyhow::Result;

/// Runs the requested subcommand.
///
/// Prefer [`crate::commands::run`] from new call sites; this remains the
/// shared implementation used by both layers.
pub async fn dispatch(args: CliArgs) -> Result<()> {
    dispatch_impl(args).await
}

/// Shared dispatch implementation (cli + commands layers).
pub async fn dispatch_impl(args: CliArgs) -> Result<()> {
    let config_override = args.config_dir.clone();
    // Aligns `secrets.key` with `--config-dir` / isolated tests.
    crate::secrets::set_config_dir(config_override.clone());
    crate::secrets::set_runtime_flags(
        args.allow_plaintext_secrets,
        args.secrets_key_file.clone(),
        args.use_keyring,
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

    match args.command {
        Command::Vps { action } => {
            // Sequential: local TOML CRUD (work ≪ SSH RTT; no multi-host I/O)
            // except `vps doctor --probe-ssh` which reuses health-check fan-out.
            crate::vps::run_vps_command(action, config_override, formato).await
        }
        Command::Connect { name } => {
            // Sequential: writes active marker only (no SSH fan-out).
            crate::vps::run_connect(&name, config_override, formato).await
        },
        Command::Exec {
            all,
            hosts,
            tags,
            target,
            steps,
            json,
            auth,
            timeout,
            description,
        } => {
            let (selection, command) = parse_exec_target(
                all,
                hosts,
                tags,
                target,
                crate::vps::read_active_vps(config_override.as_deref())?,
            )
            .map_err(|s| {
                if s.starts_with("no active VPS") {
                    crate::errors::SshCliError::NoActiveVps
                } else {
                    crate::errors::SshCliError::InvalidArgument(s)
                }
            })?;
            let key = auth.key_path_string();
            let password = read_stdin_if(auth.password_stdin, auth.password)?;
            let key_passphrase =
                read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
            let steps = parse_remote_steps(steps)
                .map_err(crate::errors::SshCliError::InvalidArgument)?;
            let opts = crate::vps::ExecOptions {
                password,
                key,
                key_passphrase,
                timeout: effective_timeout_ms(timeout, global_timeout)
                    .map_err(crate::errors::SshCliError::InvalidArgument)?,
                description,
                replace_host_key,
                disable_sudo,
                steps,
                use_agent: auth.use_agent,
                agent_socket: auth.agent_socket.as_ref().map(|p| p.to_string_lossy().into_owned()),
                ..Default::default()
            };
            crate::vps::run_exec(
                selection,
                &command,
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
            target,
            steps,
            json,
            auth,
            sudo_password,
            sudo_password_stdin,
            timeout,
            description,
        } => {
            let (selection, command) = parse_exec_target(
                all,
                hosts,
                tags,
                target,
                crate::vps::read_active_vps(config_override.as_deref())?,
            )
            .map_err(|s| {
                if s.starts_with("no active VPS") {
                    crate::errors::SshCliError::NoActiveVps
                } else {
                    crate::errors::SshCliError::InvalidArgument(s)
                }
            })?;
            let key = auth.key_path_string();
            let password = read_stdin_if(auth.password_stdin, auth.password)?;
            let sudo_password = read_stdin_if(sudo_password_stdin, sudo_password)?;
            let key_passphrase =
                read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
            let steps = parse_remote_steps(steps)
                .map_err(crate::errors::SshCliError::InvalidArgument)?;
            let opts = crate::vps::ExecOptions {
                password,
                sudo_password,
                key,
                key_passphrase,
                timeout: effective_timeout_ms(timeout, global_timeout)
                    .map_err(crate::errors::SshCliError::InvalidArgument)?,
                description,
                replace_host_key,
                disable_sudo,
                steps,
                use_agent: auth.use_agent,
                agent_socket: auth.agent_socket.as_ref().map(|p| p.to_string_lossy().into_owned()),
                ..Default::default()
            };
            crate::vps::run_sudo_exec(
                selection,
                &command,
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
            target,
            steps,
            json,
            auth,
            su_password,
            su_password_stdin,
            timeout,
            description,
        } => {
            let (selection, command) = parse_exec_target(
                all,
                hosts,
                tags,
                target,
                crate::vps::read_active_vps(config_override.as_deref())?,
            )
            .map_err(|s| {
                if s.starts_with("no active VPS") {
                    crate::errors::SshCliError::NoActiveVps
                } else {
                    crate::errors::SshCliError::InvalidArgument(s)
                }
            })?;
            let key = auth.key_path_string();
            let password = read_stdin_if(auth.password_stdin, auth.password)?;
            let su_password = read_stdin_if(su_password_stdin, su_password)?;
            let key_passphrase =
                read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
            let steps = parse_remote_steps(steps)
                .map_err(crate::errors::SshCliError::InvalidArgument)?;
            let opts = crate::vps::ExecOptions {
                password,
                su_password,
                key,
                key_passphrase,
                timeout: effective_timeout_ms(timeout, global_timeout)
                    .map_err(crate::errors::SshCliError::InvalidArgument)?,
                description,
                replace_host_key,
                disable_sudo,
                steps,
                use_agent: auth.use_agent,
                agent_socket: auth.agent_socket.as_ref().map(|p| p.to_string_lossy().into_owned()),
                ..Default::default()
            };
            crate::vps::run_su_exec(
                selection,
                &command,
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
            let key_passphrase =
                read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
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
            let key_passphrase =
                read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
            let json_efetivo = json_local || formato == OutputFormat::Json;
            if json_efetivo {
                crate::output::set_json_errors(true);
            }
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
            timeout_ms,
            auth,
            json,
            bind,
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
            let key_passphrase =
                read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
            crate::tunnel::run_tunnel(
                &vps_name,
                local_port,
                &remote_host,
                remote_port,
                config_override,
                password,
                key,
                key_passphrase,
                timeout_ms,
                replace_host_key,
                json_efetivo,
                &bind,
            )
            .await
        }
        Command::HealthCheck {
            vps_name,
            all,
            hosts,
            json,
            auth,
            timeout,
        } => {
            // GAP-SSH-CLI-006: auth parity with exec/scp (stdin + key + passphrase).
            let key = auth.key_path_string();
            let password = read_stdin_if(auth.password_stdin, auth.password)?;
            let key_passphrase =
                read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase)?;
            let selection = if all {
                crate::vps::HostSelection::All
            } else if let Some(h) = hosts {
                let names = parse_hosts_list(&h);
                if names.is_empty() {
                    return Err(crate::errors::SshCliError::InvalidArgument(
                        "--hosts requires at least one host name".into(),
                    )
                    .into());
                }
                let names = names
                    .into_iter()
                    .map(crate::domain::VpsName::try_new)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| crate::errors::SshCliError::InvalidArgument(e.to_string()))?;
                crate::vps::HostSelection::Named(names)
            } else {
                let name = match vps_name {
                    Some(n) => n,
                    None => {
                        // GAP-SSH-EXIT-002: typed → exit 66.
                        let active =
                            crate::vps::read_active_vps(config_override.as_deref())?;
                        active.ok_or(crate::errors::SshCliError::NoActiveVps)?
                    }
                };
                let name = crate::domain::VpsName::try_new(name)
                    .map_err(|e| crate::errors::SshCliError::InvalidArgument(e.to_string()))?;
                crate::vps::HostSelection::Single(name)
            };
            crate::vps::run_health_check(
                selection,
                config_override,
                formato,
                json,
                password,
                effective_timeout_ms(timeout, global_timeout)
                    .map_err(crate::errors::SshCliError::InvalidArgument)?,
                key,
                key_passphrase,
                replace_host_key,
            )
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
                crate::tls::commands::run_tls_command(
                    action,
                    config_override,
                    formato,
                    json,
                )
                .await
            }
            #[cfg(not(feature = "tls"))]
            {
                let _ = (action, json);
                Err(crate::errors::SshCliError::tls_msg(
                    "TLS feature disabled; rebuild with --features tls (default)".into(),
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

/// Implements `ssh-cli locale [show|set|clear]`.
pub(crate) fn run_locale_command(
    action: Option<LocaleAction>,
    config_override: Option<&std::path::Path>,
    formato: OutputFormat,
    force_lang: Option<&str>,
    json_flag: bool,
) -> Result<()> {
    use crate::i18n::{self, Message};
    use crate::locale::{
        clear_persisted_lang, current_language, lang_preference_path, negotiate_code,
        resolve_language_detailed, write_persisted_lang,
    };

    let action = action.unwrap_or(LocaleAction::Show);
    let override_ref = config_override;
    let wants_json = json_flag || formato == OutputFormat::Json;

    match action {
        LocaleAction::Show => {
            // Re-resolve for diagnostics (global already set at init; layers still useful).
            let detailed = resolve_language_detailed(force_lang, override_ref);
            let available: Vec<&str> = crate::i18n::Language::AVAILABLE
                .iter()
                .map(|l| l.bcp47())
                .collect();
            let pref_path = lang_preference_path(override_ref)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| String::from("(unavailable)"));

            if wants_json {
                let body = serde_json::json!({
                    "resolved": detailed.language.bcp47(),
                    "current": current_language().bcp47(),
                    "source": detailed.source.as_str(),
                    "available": available,
                    "system_raw": detailed.system_raw,
                    "persisted_raw": detailed.persisted_raw,
                    "preference_path": pref_path,
                    "direction": match detailed.language.direction() {
                        crate::i18n::TextDirection::Ltr => "ltr",
                        crate::i18n::TextDirection::Rtl => "rtl",
                    },
                    "script": detailed.language.script(),
                });
                crate::output::print_json_value(&body)?;
            } else {
                crate::output::write_line(&i18n::t(Message::LocaleStatusTitle))?;
                crate::output::write_line_fmt(format_args!(
                    "  resolved:   {} ({})",
                    detailed.language.bcp47(),
                    detailed.source.as_str()
                ))?;
                crate::output::write_line_fmt(format_args!(
                    "  current:    {}",
                    current_language().bcp47()
                ))?;
                crate::output::write_line_fmt(format_args!(
                    "  available:  {}",
                    available.join(", ")
                ))?;
                crate::output::write_line_fmt(format_args!(
                    "  system:     {}",
                    detailed.system_raw.as_deref().unwrap_or("(none)")
                ))?;
                crate::output::write_line_fmt(format_args!(
                    "  persisted:  {}",
                    detailed.persisted_raw.as_deref().unwrap_or("(none)")
                ))?;
                crate::output::write_line_fmt(format_args!("  pref_path:  {pref_path}"))?;
            }
            Ok(())
        }
        LocaleAction::Set { lang } => {
            let language = negotiate_code(&lang).ok_or_else(|| {
                anyhow::anyhow!("unsupported language after validation: {lang}")
            })?;
            let path = write_persisted_lang(language, override_ref)?;
            // Note: OnceLock already set for this process; preference applies next run
            // unless --lang/env override.
            if wants_json {
                crate::output::print_json_value(&serde_json::json!({
                    "ok": true,
                    "lang": language.bcp47(),
                    "path": path.display().to_string(),
                    "applies": "next_invocation_unless_overridden",
                }))?;
            } else {
                crate::output::print_success(&i18n::t(Message::LocalePreferenceSaved {
                    lang: language.bcp47().to_string(),
                    path: path.display().to_string(),
                }));
            }
            Ok(())
        }
        LocaleAction::Clear => {
            clear_persisted_lang(override_ref)?;
            if wants_json {
                crate::output::print_json_value(&serde_json::json!({
                    "ok": true,
                    "cleared": true,
                }))?;
            } else {
                crate::output::print_success(&i18n::t(Message::LocalePreferenceCleared));
            }
            Ok(())
        }
    }
}

