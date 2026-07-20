// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: VPS CRUD dispatcher extracted from vps/mod (SRP; line budget).
#![forbid(unsafe_code)]
//! Dispatcher for `ssh-cli vps …` subcommands.

use super::config_io::{load, resolve_config_path, save, validate_key_path_exists};
use super::doctor::run_doctor_with_optional_probe;
use super::import_export::{run_export, run_import};
use super::health::run_health_check;
use super::model::{self, VpsRecord};
use super::secrets_cmd::take_auto_key_meta;
use super::selection::HostSelection;
use super::{read_secret_stdin, use_json};
use crate::cli::{OutputFormat, VpsAction};
use crate::errors::SshCliError;
use anyhow::Result;
use secrecy::SecretString;
use std::path::{Path, PathBuf};

/// Dispatcher dos subcomandos `vps`.
pub async fn run_vps_command(
    action: VpsAction,
    config_override: Option<PathBuf>,
    format: OutputFormat,
) -> Result<()> {
    let path = resolve_config_path(config_override.as_deref())?;

    match action {
        VpsAction::Add {
            name,
            host,
            port,
            user,
            password,
            password_stdin,
            key,
            key_passphrase,
            use_agent,
            agent_socket,
            timeout,
            max_command_chars,
            max_output_chars,
            max_chars,
            sudo_password,
            sudo_password_stdin,
            su_password,
            su_password_stdin,
            disable_sudo,
            tags,
            tls,
            tls_sni,
            tls_client_cert,
            tls_client_key,
            check,
        } => {
            // GAP-SSH-VAL-001: validate na fronteira de escrita.
            let name = crate::paths::validate_and_normalize(&name)
                .map_err(|e| SshCliError::InvalidArgument(format!("invalid VPS name: {e}")))?;
            let name_key = name.as_str().to_owned();
            let mut file = load(&path)?;
            if file.hosts.contains_key(&name_key) {
                return Err(SshCliError::VpsDuplicate(name_key).into());
            }
            if password_stdin && (sudo_password_stdin || su_password_stdin) {
                return Err(SshCliError::InvalidArgument(
                    "only one --*-stdin per one-shot invocation; use vps edit for sudo/su".into(),
                )
                .into());
            }
            let password = if password_stdin {
                read_secret_stdin()?
            } else {
                SecretString::from(password.unwrap_or_default())
            };
            let sudo_s = if sudo_password_stdin {
                Some(read_secret_stdin()?)
            } else {
                sudo_password.map(SecretString::from)
            };
            let su_s = if su_password_stdin {
                Some(read_secret_stdin()?)
            } else {
                su_password.map(SecretString::from)
            };
            let key = key.map(|p| p.to_string_lossy().into_owned());
            if let Some(ref k) = key {
                validate_key_path_exists(k)?;
            }
            // legacy max_chars → command if max_command was not set explicitly
            // (clap already parses `none`/`0`/decimal via parse_cli_char_limit)
            let max_cmd = max_command_chars
                .or(max_chars)
                .unwrap_or(model::DEFAULT_MAX_COMMAND_CHARS);
            let max_out = max_output_chars.unwrap_or(model::DEFAULT_MAX_OUTPUT_CHARS);
            // GAP-AUD-009: timeout is milliseconds; warn agents that use "5" meaning seconds.
            if timeout > 0 && timeout < 1000 {
                crate::output::print_warning_fmt(format_args!(
                    "--timeout {timeout} is only {timeout}ms (< 1s); did you mean seconds? Use e.g. --timeout 5000 for 5s"
                ));
            }
            let mut record = VpsRecord::try_new(
                name.as_str(),
                host,
                port,
                user,
                password,
                key,
                key_passphrase.map(SecretString::from),
                Some(timeout),
                Some(max_cmd),
                Some(max_out),
                sudo_s,
                su_s,
                disable_sudo,
            )
            .map_err(SshCliError::InvalidArgument)?;
            // G-E2E-19: registry auth triplo (password | key | agent).
            if use_agent {
                record.use_agent = true;
                record.password = SecretString::from(String::new());
                record.key_path = None;
                record.key_passphrase = None;
                record.agent_socket = agent_socket.map(|p| p.to_string_lossy().into_owned());
            }
            // G-O2: tags for fleet selection (dedupe preserve order).
            let tag_list = crate::vps::selection::dedupe_host_names(tags);
            record
                .set_tags_from_raw(tag_list).map_err(SshCliError::from)?;
            record.tls = tls;
            record.tls_sni = tls_sni;
            record.tls_client_cert = tls_client_cert
                .map(|p| p.to_string_lossy().into_owned());
            record.tls_client_key = tls_client_key
                .map(|p| p.to_string_lossy().into_owned());
            if record.tls {
                // Validate options early (SNI empty / partial mTLS).
                let sni = record
                    .tls_sni
                    .as_deref()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or(record.host.as_str());
                let _ = crate::tls::TlsConnectOptions::try_new(
                    sni,
                    record.tls_client_cert.as_ref().map(std::path::PathBuf::from),
                    record.tls_client_key.as_ref().map(std::path::PathBuf::from),
                )?;
            }
            // GAP-SSH-VAL-002 / VAL-003: full domain validation on the write-path.
            record.validate().map_err(SshCliError::from)?;
            file.hosts.insert(name_key.clone(), record);
            file.schema_version = model::CURRENT_SCHEMA_VERSION;
            save(&path, &file)?;
            // G-E2E-04 / one-shot: single stdout document (fold auto-key into vps-added).
            // Workload: local single-file CRUD — sequential justified (≪ SSH RTT).
            let auto_key = take_auto_key_meta();
            let mut data = serde_json::json!({ "name": name_key });
            if let Some(ref meta) = auto_key {
                data["secrets_key_auto_created"] = serde_json::Value::Bool(true);
                data["key_file"] = serde_json::Value::String(meta.key_file.clone());
                data["key_source"] = serde_json::Value::String(meta.key_source.to_owned());
            } else {
                data["secrets_key_auto_created"] = serde_json::Value::Bool(false);
            }
            let msg = if let Some(ref meta) = auto_key {
                format!(
                    "{}; primary-key auto-created at {}",
                    crate::i18n::t(crate::i18n::Message::VpsAdded {
                        name: name_key.clone(),
                    }),
                    meta.key_file
                )
            } else {
                crate::i18n::t(crate::i18n::Message::VpsAdded {
                    name: name_key.clone(),
                })
            };
            crate::output::emit_success(
                "vps-added",
                data,
                &msg,
                format == OutputFormat::Json,
            )?;
            if check {
                run_health_check(
                    HostSelection::Single(name.clone()),
                    config_override,
                    format,
                    false,
                    None,
                    None,
                    None,
                    None,
                    false,
                )
                .await?;
            }
        }
        VpsAction::List { json, tags } => {
            let file = load(&path)?;
            let records: Vec<_> = if tags.is_empty() {
                file.hosts.values().cloned().collect()
            } else {
                {
                    let wanted = crate::domain::try_tags(&tags)
                        .map_err(|e| SshCliError::InvalidArgument(e.to_string()))?;
                    file.hosts
                        .values()
                        .filter(|r| r.has_any_tag(&wanted))
                        .cloned()
                        .collect()
                }
            };
            // GAP-SSH-IO-001: respeitar format global.
            if use_json(json, format) {
                crate::output::print_list_json(&records)?;
            } else {
                crate::output::print_list_text(&records);
            }
        }
        VpsAction::Remove { name } => {
            let mut file = load(&path)?;
            if file.hosts.remove(&name).is_none() {
                return Err(SshCliError::VpsNotFound(name).into());
            }
            save(&path, &file)?;
            // GAP-SSH-STATE-001: clear orphan active marker.
            clear_active_if_name(&path, &name)?;
            crate::output::emit_success(
                "vps-removed",
                serde_json::json!({ "name": name }),
                &crate::i18n::t(crate::i18n::Message::VpsRemoved { name: name.clone() }),
                format == OutputFormat::Json,
            )?;
        }
        VpsAction::Edit {
            name,
            host,
            port,
            user,
            password,
            password_stdin,
            key,
            key_passphrase,
            use_agent,
            agent_socket,
            timeout,
            max_command_chars,
            max_output_chars,
            max_chars,
            sudo_password,
            sudo_password_stdin,
            su_password,
            su_password_stdin,
            disable_sudo,
            enable_sudo,
            tls,
            no_tls,
            tls_sni,
            tls_client_cert,
            tls_client_key,
        } => {
            let mut file = load(&path)?;
            let record = file
                .hosts
                .get_mut(&name)
                .ok_or(SshCliError::VpsNotFound(name.clone()))?;
            use crate::domain::{CharLimit, KeyPath, SshHost, SshPort, SshUser, TimeoutMs};
            if let Some(h) = host {
                record.host = SshHost::try_new(h).map_err(|e| SshCliError::InvalidArgument(e.to_string()))?;
            }
            if let Some(p) = port {
                record.port = SshPort::try_new(p).map_err(|e| SshCliError::InvalidArgument(e.to_string()))?;
            }
            if let Some(u) = user {
                record.username = SshUser::try_new(u).map_err(|e| SshCliError::InvalidArgument(e.to_string()))?;
            }
            if use_agent {
                record.use_agent = true;
                record.password = SecretString::from(String::new());
                record.key_path = None;
                record.key_passphrase = None;
                if let Some(s) = agent_socket {
                    record.agent_socket = Some(s.to_string_lossy().into_owned());
                }
            } else {
                if password_stdin {
                    record.password = read_secret_stdin()?;
                    record.use_agent = false;
                } else if let Some(pw) = password {
                    record.password = SecretString::from(pw);
                    record.use_agent = false;
                }
                if let Some(k) = key {
                    let k = k.to_string_lossy().into_owned();
                    validate_key_path_exists(&k)?;
                    record.key_path = Some(
                        KeyPath::try_new(k).map_err(|e| SshCliError::InvalidArgument(e.to_string()))?,
                    );
                    record.use_agent = false;
                }
                if let Some(kp) = key_passphrase {
                    record.key_passphrase = Some(SecretString::from(kp));
                }
                if let Some(s) = agent_socket {
                    record.agent_socket = Some(s.to_string_lossy().into_owned());
                }
            }
            if let Some(t) = timeout {
                record.timeout_ms =
                    TimeoutMs::try_new(t).map_err(|e| SshCliError::InvalidArgument(e.to_string()))?;
            }
            if let Some(m) = max_command_chars.or(max_chars) {
                record.max_command_chars =
                    CharLimit::try_new(m).map_err(|e| SshCliError::InvalidArgument(e.to_string()))?;
            }
            if let Some(m) = max_output_chars {
                record.max_output_chars =
                    CharLimit::try_new(m).map_err(|e| SshCliError::InvalidArgument(e.to_string()))?;
            }
            if sudo_password_stdin {
                record.sudo_password = Some(read_secret_stdin()?);
            } else if let Some(sp) = sudo_password {
                record.sudo_password = Some(SecretString::from(sp));
            }
            if su_password_stdin {
                record.su_password = Some(read_secret_stdin()?);
            } else if let Some(sp) = su_password {
                record.su_password = Some(SecretString::from(sp));
            }
            // G-10: tri-state edit without Option<bool> — exclusive SetTrue flags.
            if disable_sudo {
                record.disable_sudo = true;
            } else if enable_sudo {
                record.disable_sudo = false;
            }
            if tls {
                record.tls = true;
            } else if no_tls {
                record.tls = false;
            }
            if let Some(sni) = tls_sni {
                record.tls_sni = Some(sni);
            }
            if let Some(c) = tls_client_cert {
                record.tls_client_cert = Some(c.to_string_lossy().into_owned());
            }
            if let Some(k) = tls_client_key {
                record.tls_client_key = Some(k.to_string_lossy().into_owned());
            }
            if record.tls {
                let sni = record
                    .tls_sni
                    .as_deref()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or(record.host.as_str());
                let _ = crate::tls::TlsConnectOptions::try_new(
                    sni,
                    record.tls_client_cert.as_ref().map(std::path::PathBuf::from),
                    record.tls_client_key.as_ref().map(std::path::PathBuf::from),
                )?;
            }
            record.validate().map_err(SshCliError::from)?;
            save(&path, &file)?;
            crate::output::emit_success(
                "vps-edited",
                serde_json::json!({ "name": name }),
                &crate::i18n::t(crate::i18n::Message::VpsEdited { name: name.clone() }),
                format == OutputFormat::Json,
            )?;
        }
        VpsAction::Show { name, json } => {
            let file = load(&path)?;
            let record = file
                .hosts
                .get(&name)
                .ok_or(SshCliError::VpsNotFound(name.clone()))?;
            if use_json(json, format) {
                crate::output::print_details_json(record)?;
            } else {
                crate::output::print_details_text(record);
            }
        }
        VpsAction::Path => {
            // G-AUD-02: JSON envelope when format is Json; plain path in Text.
            if use_json(false, format) {
                let path_s = path.display().to_string();
                crate::output::emit_success(
                    "vps-path",
                    serde_json::json!({ "path": path_s }),
                    &path_s,
                    true,
                )?;
            } else {
                // G-MAC-01: format_args + write_fmt — no intermediate String for Display.
                crate::output::write_line_fmt(format_args!("{}", path.display()))?;
            }
        }
        VpsAction::Doctor {
            json,
            probe_ssh,
            hosts,
        } => {
            // G-PAR-38/42: single envelope (local + optional ssh_probe); no dual JSON roots.
            let as_json = use_json(json, format);
            if hosts.is_some() && !probe_ssh {
                return Err(SshCliError::InvalidArgument(
                    "--hosts on vps doctor requires --probe-ssh".into(),
                )
                .into());
            }
            let selection = if probe_ssh {
                match hosts {
                    None => HostSelection::All,
                    Some(raw) => {
                        let names = crate::cli::parse_hosts_list(&raw);
                        if names.is_empty() {
                            return Err(SshCliError::InvalidArgument(
                                "--hosts requires at least one host name".into(),
                            )
                            .into());
                        }
                        let names = names
                            .into_iter()
                            .map(crate::domain::VpsName::try_new)
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|e| SshCliError::InvalidArgument(e.to_string()))?;
                        HostSelection::Named(names)
                    }
                }
            } else {
                // unused when !probe_ssh
                HostSelection::All
            };
            run_doctor_with_optional_probe(
                config_override.as_deref(),
                as_json,
                probe_ssh,
                if probe_ssh { Some(selection) } else { None },
            )
            .await?;
        }
        VpsAction::Export {
            include_secrets,
            output,
            json,
            i_understand_secrets_on_stdout,
        } => {
            // G-AUD-03: export body JSON when local --json or global format Json.
            run_export(
                &path,
                include_secrets,
                output.as_deref(),
                json,
                i_understand_secrets_on_stdout,
                format,
            )?;
        }
        VpsAction::Import {
            file,
            allow_incomplete,
        } => {
            run_import(&path, &file, allow_incomplete, format)?;
        }
    }
    Ok(())
}

/// Removes the `active` file if its content matches the removed name (STATE-001).
fn clear_active_if_name(config_path: &Path, name: &str) -> Result<()> {
    let active = config_path
        .parent()
        .map(|p| p.join(crate::constants::ACTIVE_VPS_FILE_NAME))
        .unwrap_or_else(|| PathBuf::from(crate::constants::ACTIVE_VPS_FILE_NAME));
    if !active.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&active).unwrap_or_default();
    if content.trim() == name {
        let _ = std::fs::remove_file(&active);
    }
    Ok(())
}
