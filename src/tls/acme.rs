// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! ACME (RFC 8555) via `instant-acme` — DNS-01, XDG storage, agent two-step flow.
//!
//! ## Agent-friendly flow
//!
//! 1. `tls acme issue --domain example.com --print-challenge`  
//!    Creates order, prints TXT challenge JSON, persists `order_url` under XDG.
//! 2. Operator sets `_acme-challenge.example.com` TXT to the printed value.
//! 3. `tls acme complete --domain example.com`  
//!    Resumes the **same** order via URL, `set_ready`, finalize, writes PEMs (0o600).

use std::path::Path;

use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt,
    NewAccount, NewOrder, OrderStatus, RetryPolicy,
};
use serde::{Deserialize, Serialize};

use super::paths::{
    acme_account_path, acme_domain_dir, cert_pem_path, ensure_dir, key_pem_path, order_json_path,
    write_secret_file,
};
use super::provider::ensure_provider;
use super::acme_error_map::map_acme_error;
use crate::domain::{AcmeOrderUrl, HttpsUrl, Rfc3339Utc};
use crate::errors::{SshCliError, SshCliResult};

/// ACME directory selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AcmeDirectory {
    /// Let's Encrypt production.
    #[default]
    Production,
    /// Let's Encrypt staging (no production rate-limits; untrusted chain).
    Staging,
}

impl AcmeDirectory {
    /// Validated HTTPS directory URL (from `instant-acme` constants).
    fn url(self) -> SshCliResult<HttpsUrl> {
        let raw = match self {
            Self::Production => LetsEncrypt::Production.url(),
            Self::Staging => LetsEncrypt::Staging.url(),
        };
        HttpsUrl::try_new(raw).map_err(|e| SshCliError::tls_msg(format!("ACME directory URL: {e}")))
    }

    fn label(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Staging => "staging",
        }
    }
}

/// Persisted pending order (challenge phase) — resume via `order_url`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOrder {
    /// Domain being ordered.
    pub domain: String,
    /// Directory label (`production` / `staging`).
    pub directory: String,
    /// ACME order URL (required to resume the same challenge token).
    pub order_url: AcmeOrderUrl,
    /// DNS TXT name (`_acme-challenge.<domain>`).
    pub dns_name: String,
    /// DNS TXT value for the challenge.
    pub dns_value: String,
    /// RFC 3339 UTC when the challenge was printed.
    pub created_at: Rfc3339Utc,
}

/// Account status for JSON/text output.
#[derive(Debug, Clone, Serialize)]
pub struct AccountStatus {
    /// Whether account credentials exist on disk.
    pub present: bool,
    /// Absolute path to account JSON.
    pub path: String,
    /// Directory used when account was created (if known).
    pub directory: Option<String>,
}

/// Certificate status for one domain.
#[derive(Debug, Clone, Serialize)]
pub struct DomainCertStatus {
    /// Domain leaf.
    pub domain: String,
    /// Cert PEM present.
    pub has_cert: bool,
    /// Key PEM present.
    pub has_key: bool,
    /// Pending order present.
    pub has_pending_order: bool,
    /// Absolute cert path when present.
    pub cert_path: Option<String>,
    /// Absolute key path when present.
    pub key_path: Option<String>,
}

/// Creates (or refuses if exists unless `force`) an ACME account under XDG.
pub async fn create_account(
    config_override: Option<&Path>,
    directory: AcmeDirectory,
    contact: &[String],
    force: bool,
) -> SshCliResult<AccountStatus> {
    ensure_provider()?;
    let path = acme_account_path(config_override)?;
    if path.exists() && !force {
        return Err(SshCliError::InvalidArgument(format!(
            "ACME account already exists at {}; use --force to replace",
            path.display()
        )));
    }
    // G-AUD-06/28: require mailto contact(s) before talking to the CA.
    if contact.is_empty() {
        return Err(SshCliError::InvalidArgument(
            "ACME account create requires at least one --contact mailto:address".into(),
        ));
    }
    for c in contact {
        let c = c.trim();
        if !c.to_ascii_lowercase().starts_with("mailto:")
            || c.len() <= "mailto:".len()
            || !c.contains('@')
        {
            return Err(SshCliError::InvalidArgument(format!(
                "invalid ACME contact '{c}'; expected mailto:user@example.com"
            )));
        }
    }
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    let contacts: Vec<&str> = contact.iter().map(String::as_str).collect();
    let directory_url = directory.url()?;
    let (_account, credentials) = Account::builder()
        .map_err(|e| map_acme_error("ACME account builder", e))?
        .create(
            &NewAccount {
                contact: &contacts,
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            directory_url.as_str().to_owned(),
            None,
        )
        .await
        .map_err(|e| map_acme_error("ACME create account", e))?;

    let json = serde_json::to_vec_pretty(&credentials)
        .map_err(|e| SshCliError::tls_msg(format!("serialize ACME credentials: {e}")))?;
    write_secret_file(&path, &json)?;

    let meta_path = path.with_extension("meta.json");
    let meta = serde_json::json!({
        "directory": directory.label(),
        "created_at": Rfc3339Utc::now().to_rfc3339(),
    });
    write_secret_file(
        &meta_path,
        serde_json::to_vec_pretty(&meta)
            .map_err(|e| SshCliError::tls_msg(format!("meta: {e}")))?
            .as_slice(),
    )?;

    Ok(AccountStatus {
        present: true,
        path: path.display().to_string(),
        directory: Some(directory.label().to_owned()),
    })
}

/// Loads account presence / path without contacting ACME.
pub fn load_account_status(config_override: Option<&Path>) -> SshCliResult<AccountStatus> {
    let path = acme_account_path(config_override)?;
    let directory = std::fs::read_to_string(path.with_extension("meta.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("directory")?.as_str().map(str::to_owned));
    Ok(AccountStatus {
        present: path.is_file(),
        path: path.display().to_string(),
        directory,
    })
}

async fn load_account(config_override: Option<&Path>) -> SshCliResult<Account> {
    ensure_provider()?;
    let path = acme_account_path(config_override)?;
    if !path.is_file() {
        return Err(SshCliError::FileNotFound(format!(
            "ACME account missing at {}; run `ssh-cli tls acme account create`",
            path.display()
        )));
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| SshCliError::tls_msg(format!("read account: {e}")))?;
    let credentials: AccountCredentials = serde_json::from_str(&data)
        .map_err(|e| SshCliError::tls_msg(format!("parse account credentials: {e}")))?;
    Account::builder()
        .map_err(|e| map_acme_error("ACME account builder", e))?
        .from_credentials(credentials)
        .await
        .map_err(|e| map_acme_error("restore ACME account", e))
}

/// Starts DNS-01 order and returns challenge details (does **not** wait for DNS).
pub async fn acme_issue_print_challenge(
    config_override: Option<&Path>,
    domain: &str,
    directory: AcmeDirectory,
) -> SshCliResult<PendingOrder> {
    let account = load_account(config_override).await?;
    let identifiers = vec![Identifier::Dns(domain.to_owned())];
    let mut order = account
        .new_order(&NewOrder::new(identifiers.as_slice()))
        .await
        .map_err(|e| map_acme_error("ACME new_order", e))?;

    let order_url = AcmeOrderUrl::try_new(order.url())
        .map_err(|e| SshCliError::tls_msg(format!("ACME order URL: {e}")))?;
    let mut dns_name = String::new();
    let mut dns_value = String::new();

    let mut authorizations = order.authorizations();
    while let Some(result) = authorizations.next().await {
        let mut authz = result.map_err(|e| map_acme_error("ACME authz", e))?;
        match authz.status {
            AuthorizationStatus::Pending => {}
            AuthorizationStatus::Valid => continue,
            other => {
                return Err(SshCliError::InvalidArgument(format!(
                    "unexpected ACME authorization status: {other:?}"
                )));
            }
        }
        let challenge = authz
            .challenge(ChallengeType::Dns01)
            .ok_or_else(|| SshCliError::InvalidArgument("no DNS-01 challenge on order".into()))?;
        dns_name = format!("_acme-challenge.{}", challenge.identifier());
        dns_value = challenge.key_authorization().dns_value();
    }

    if dns_value.is_empty() {
        return Err(SshCliError::InvalidArgument(
            "ACME order had no pending DNS-01 challenge".into(),
        ));
    }

    let pending = PendingOrder {
        domain: domain.to_owned(),
        directory: directory.label().to_owned(),
        order_url,
        dns_name,
        dns_value,
        created_at: Rfc3339Utc::now(),
    };

    let dir = acme_domain_dir(config_override, domain)?;
    ensure_dir(&dir)?;
    let order_path = order_json_path(&dir);
    let bytes = serde_json::to_vec_pretty(&pending)
        .map_err(|e| SshCliError::tls_msg(format!("serialize pending order: {e}")))?;
    write_secret_file(&order_path, &bytes)?;

    Ok(pending)
}

/// Completes ACME after DNS TXT is published: resume order → set_ready → finalize.
pub async fn acme_complete(
    config_override: Option<&Path>,
    domain: &str,
) -> SshCliResult<DomainCertStatus> {
    let dir = acme_domain_dir(config_override, domain)?;
    let pending_path = order_json_path(&dir);
    if !pending_path.is_file() {
        return Err(SshCliError::FileNotFound(format!(
            "no pending ACME order for {domain}; run `tls acme issue --domain {domain} --print-challenge` first"
        )));
    }

    let data = std::fs::read_to_string(&pending_path)
        .map_err(|e| SshCliError::tls_msg(format!("read pending order: {e}")))?;
    let pending: PendingOrder = serde_json::from_str(&data)
        .map_err(|e| SshCliError::tls_msg(format!("parse pending order: {e}")))?;

    let account = load_account(config_override).await?;
    let mut order = account
        .order(pending.order_url.to_string_owned())
        .await
        .map_err(|e| map_acme_error("ACME resume order", e))?;

    let mut authorizations = order.authorizations();
    while let Some(result) = authorizations.next().await {
        let mut authz = result.map_err(|e| map_acme_error("ACME authz", e))?;
        match authz.status {
            AuthorizationStatus::Pending => {}
            AuthorizationStatus::Valid => continue,
            other => {
                return Err(SshCliError::InvalidArgument(format!(
                    "unexpected ACME authorization status: {other:?}"
                )));
            }
        }
        let mut challenge = authz
            .challenge(ChallengeType::Dns01)
            .ok_or_else(|| SshCliError::InvalidArgument("no DNS-01 challenge".into()))?;
        challenge
            .set_ready()
            .await
            .map_err(|e| map_acme_error("ACME set_ready", e))?;
    }

    let status = order
        .poll_ready(&RetryPolicy::default())
        .await
        .map_err(|e| map_acme_error("ACME poll_ready", e))?;
    if status != OrderStatus::Ready {
        return Err(SshCliError::InvalidArgument(format!(
            "ACME order not ready (status={status:?}); verify DNS TXT {} = {}",
            pending.dns_name, pending.dns_value
        )));
    }

    let private_key_pem = order
        .finalize()
        .await
        .map_err(|e| map_acme_error("ACME finalize", e))?;
    let cert_chain_pem = order
        .poll_certificate(&RetryPolicy::default())
        .await
        .map_err(|e| map_acme_error("ACME poll_certificate", e))?;

    ensure_dir(&dir)?;
    let cert_path = cert_pem_path(&dir);
    let key_path = key_pem_path(&dir);
    write_secret_file(&cert_path, cert_chain_pem.as_bytes())?;
    write_secret_file(&key_path, private_key_pem.as_bytes())?;
    let _ = std::fs::remove_file(pending_path);

    Ok(DomainCertStatus {
        domain: domain.to_owned(),
        has_cert: true,
        has_key: true,
        has_pending_order: false,
        cert_path: Some(cert_path.display().to_string()),
        key_path: Some(key_path.display().to_string()),
    })
}

/// Status for one domain or all under ACME dir.
pub fn acme_status(
    config_override: Option<&Path>,
    domain: Option<&str>,
) -> SshCliResult<Vec<DomainCertStatus>> {
    if let Some(d) = domain {
        return Ok(vec![domain_status(config_override, d)?]);
    }
    acme_list(config_override)
}

/// Lists all domain directories under ACME storage.
pub fn acme_list(config_override: Option<&Path>) -> SshCliResult<Vec<DomainCertStatus>> {
    let root = super::paths::resolve_tls_root(config_override)?
        .join(crate::constants::TLS_ACME_DIR_NAME);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&root)
        .map_err(|e| SshCliError::tls_msg(format!("list acme: {e}")))?
    {
        let entry = entry.map_err(|e| SshCliError::tls_msg(format!("list acme entry: {e}")))?;
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            out.push(domain_status(config_override, name)?);
        }
    }
    out.sort_by(|a, b| a.domain.cmp(&b.domain));
    Ok(out)
}

fn domain_status(config_override: Option<&Path>, domain: &str) -> SshCliResult<DomainCertStatus> {
    let dir = acme_domain_dir(config_override, domain)?;
    let cert = cert_pem_path(&dir);
    let key = key_pem_path(&dir);
    let pending = order_json_path(&dir);
    Ok(DomainCertStatus {
        domain: domain.to_owned(),
        has_cert: cert.is_file(),
        has_key: key.is_file(),
        has_pending_order: pending.is_file(),
        cert_path: cert.is_file().then(|| cert.display().to_string()),
        key_path: key.is_file().then(|| key.display().to_string()),
    })
}
