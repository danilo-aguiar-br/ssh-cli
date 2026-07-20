// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! CLI handlers for `ssh-cli tls …` (feature `tls`).

use std::path::PathBuf;

use crate::cli::{
    OutputFormat, TlsAcmeAccountAction, TlsAcmeAction, TlsAction, TlsMtlsAction,
};
use crate::errors::SshCliError;
use crate::output;
use anyhow::Result;

/// Dispatches `tls` subcommands.
pub async fn run_tls_command(
    action: TlsAction,
    config_override: Option<PathBuf>,
    format: OutputFormat,
    json_flag: bool,
) -> Result<()> {
    let json = format == OutputFormat::Json || json_flag;
    let cfg = config_override.as_deref();

    match action {
        TlsAction::Provider => {
            let installed = super::provider_is_installed();
            let name = super::provider_name();
            let payload = serde_json::json!({
                "provider": name,
                "installed": installed,
                "min_rustls": "0.23.18",
                "stack": "rustls+aws_lc_rs",
            });
            output::emit_success(
                "tls-provider",
                payload,
                &format!("TLS provider: {name} (installed={installed})"),
                json,
            )?;
        }
        TlsAction::Paths => {
            let root = super::tls_root_dir(cfg)?;
            let account = super::acme_account_path(cfg)?;
            let payload = serde_json::json!({
                "tls_root": root.display().to_string(),
                "mtls_dir": root.join(crate::constants::TLS_MTLS_DIR_NAME).display().to_string(),
                "acme_dir": root.join(crate::constants::TLS_ACME_DIR_NAME).display().to_string(),
                "acme_account": account.display().to_string(),
            });
            output::emit_success(
                "tls-paths",
                payload,
                &format!("TLS root: {}", root.display()),
                json,
            )?;
        }
        TlsAction::Mtls { action } => match action {
            TlsMtlsAction::List => {
                let names = super::mtls_list(cfg)?;
                let payload = serde_json::json!({ "identities": names });
                let msg = if names.is_empty() {
                    "no mTLS identities".to_string()
                } else {
                    format!("mTLS identities: {}", names.join(", "))
                };
                output::emit_success("tls-mtls-list", payload, &msg, json)?;
            }
            TlsMtlsAction::Import { name, cert, key } => {
                let id = super::mtls_import(cfg, &name, &cert, &key)?;
                let payload = serde_json::json!({
                    "name": id.name,
                    "cert_path": id.cert_path.display().to_string(),
                    "key_path": id.key_path.display().to_string(),
                });
                output::emit_success(
                    "tls-mtls-import",
                    payload,
                    &format!("imported mTLS identity '{}'", id.name),
                    json,
                )?;
            }
            TlsMtlsAction::Show { name } => {
                let id = super::mtls_show(cfg, &name)?;
                let payload = serde_json::json!({
                    "name": id.name,
                    "cert_path": id.cert_path.display().to_string(),
                    "key_path": id.key_path.display().to_string(),
                });
                output::emit_success(
                    "tls-mtls-show",
                    payload,
                    &format!(
                        "mTLS '{}': cert={} key={}",
                        id.name,
                        id.cert_path.display(),
                        id.key_path.display()
                    ),
                    json,
                )?;
            }
            TlsMtlsAction::Remove { name } => {
                super::mtls_remove(cfg, &name)?;
                output::emit_success(
                    "tls-mtls-remove",
                    serde_json::json!({ "name": name }),
                    &format!("removed mTLS identity '{name}'"),
                    json,
                )?;
            }
        },
        TlsAction::Acme { action } => match action {
            TlsAcmeAction::Account { action } => match action {
                TlsAcmeAccountAction::Create {
                    staging,
                    contact,
                    force,
                } => {
                    let dir = if staging {
                        super::AcmeDirectory::Staging
                    } else {
                        super::AcmeDirectory::Production
                    };
                    let st = super::create_account(cfg, dir, &contact, force).await?;
                    let payload = serde_json::to_value(&st)
                        .map_err(|e| SshCliError::tls_msg(format!("json: {e}")))?;
                    output::emit_success(
                        "tls-acme-account-create",
                        payload,
                        &format!("ACME account at {}", st.path),
                        json,
                    )?;
                }
                TlsAcmeAccountAction::Show => {
                    let st = super::load_account_status(cfg)?;
                    let payload = serde_json::to_value(&st)
                        .map_err(|e| SshCliError::tls_msg(format!("json: {e}")))?;
                    let msg = if st.present {
                        format!("ACME account present at {}", st.path)
                    } else {
                        format!("no ACME account at {}", st.path)
                    };
                    output::emit_success("tls-acme-account-show", payload, &msg, json)?;
                }
            },
            TlsAcmeAction::Issue {
                domain,
                staging,
                print_challenge,
            } => {
                if !print_challenge {
                    return Err(SshCliError::InvalidArgument(
                        "pass --print-challenge (agent two-step DNS-01; no interactive wait)"
                            .into(),
                    )
                    .into());
                }
                let dir = if staging {
                    super::AcmeDirectory::Staging
                } else {
                    super::AcmeDirectory::Production
                };
                let pending = super::acme_issue_print_challenge(cfg, &domain, dir).await?;
                let payload = serde_json::to_value(&pending)
                    .map_err(|e| SshCliError::tls_msg(format!("json: {e}")))?;
                let msg = format!(
                    "set DNS TXT {} = {} then run: ssh-cli tls acme complete --domain {}",
                    pending.dns_name, pending.dns_value, pending.domain
                );
                output::emit_success("tls-acme-challenge", payload, &msg, json)?;
            }
            TlsAcmeAction::Complete { domain } => {
                let st = super::acme_complete(cfg, &domain).await?;
                let payload = serde_json::to_value(&st)
                    .map_err(|e| SshCliError::tls_msg(format!("json: {e}")))?;
                output::emit_success(
                    "tls-acme-complete",
                    payload,
                    &format!("certificate issued for {domain}"),
                    json,
                )?;
            }
            TlsAcmeAction::Status { domain } => {
                let list = super::acme_status(cfg, domain.as_deref())?;
                let payload = serde_json::json!({ "domains": list });
                output::emit_success(
                    "tls-acme-status",
                    payload,
                    &format!("{} ACME domain(s)", list.len()),
                    json,
                )?;
            }
            TlsAcmeAction::List => {
                let list = super::acme_list(cfg)?;
                let payload = serde_json::json!({ "domains": list });
                output::emit_success(
                    "tls-acme-list",
                    payload,
                    &format!("{} ACME domain(s)", list.len()),
                    json,
                )?;
            }
        },
    }
    Ok(())
}
