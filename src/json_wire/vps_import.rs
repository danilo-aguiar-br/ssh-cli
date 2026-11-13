// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Import DTOs for `ssh-cli vps import` (dual-read EN wire + legacy PT aliases).
//!
//! # Why this lives apart from the export DTOs
//!
//! Export is a producer: it owns the shapes this CLI writes. Import is a
//! consumer: it must tolerate every shape a previous version or a human wrote,
//! so it carries aliases and defaults the export side must never have. They sat
//! in one file until the file hit its component budget, which is also why the
//! split is by direction rather than by field.
//!
//! # Must-ignore contract
//!
//! Every entry uses `serde(default)` and no `deny_unknown_fields`, so a document
//! from a newer version imports instead of failing. That is deliberate: an
//! import that rejects unknown fields cannot be used to migrate forward.

use crate::vps::model::VpsRecord;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Top-level import document: `vps-export` envelope or bare `{ "hosts": … }`.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportEnvelope {
    /// Optional schema version from export envelope.
    #[serde(default)]
    pub schema_version: Option<u32>,
    /// Host map (required).
    pub hosts: BTreeMap<String, ImportHostEntry>,
}

/// One host object under `hosts` for JSON import.
///
/// Accepts export wire (`user`) and domain/TOML keys (`username` / PT aliases).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ImportHostEntry {
    /// Name (falls back to map key when absent).
    #[serde(default, alias = "nome")]
    pub name: Option<String>,
    /// Host.
    #[serde(default)]
    pub host: Option<String>,
    /// Port (validated into `u16` after parse).
    #[serde(default, alias = "porta")]
    pub port: Option<u64>,
    /// Username — export uses `user`; TOML/domain uses `username`.
    #[serde(default, alias = "usuario", alias = "user")]
    pub username: Option<String>,
    /// Password.
    #[serde(default, alias = "senha")]
    pub password: Option<String>,
    /// Key path.
    #[serde(default)]
    pub key_path: Option<String>,
    /// Key passphrase.
    #[serde(default)]
    pub key_passphrase: Option<String>,
    /// Authenticate through the SSH agent (G-SSH-04).
    ///
    /// Absent means "not specified", which keeps the pre-0.5.6 behaviour of a
    /// password/key host for every document that never carried the field.
    #[serde(default)]
    pub use_agent: Option<bool>,
    /// Explicit agent socket or Windows named pipe path.
    #[serde(default)]
    pub agent_socket: Option<String>,
    /// Timeout ms.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Command limit.
    #[serde(default)]
    pub max_command_chars: Option<usize>,
    /// Output limit (legacy `max_chars`).
    #[serde(default, alias = "max_chars")]
    pub max_output_chars: Option<usize>,
    /// Sudo password.
    #[serde(default, alias = "senha_sudo")]
    pub sudo_password: Option<String>,
    /// Su password.
    #[serde(default, alias = "senha_su")]
    pub su_password: Option<String>,
    /// Disable sudo/su.
    #[serde(default)]
    pub disable_sudo: Option<bool>,
    /// Schema version.
    #[serde(default)]
    pub schema_version: Option<u32>,
    /// Added-at RFC 3339.
    #[serde(default, alias = "adicionado_em")]
    pub added_at: Option<String>,
    /// Host tags (G-O2 / G-SERDE-06).
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// SSH-over-TLS.
    #[serde(default)]
    pub tls: Option<bool>,
    /// TLS SNI.
    #[serde(default)]
    pub tls_sni: Option<String>,
    /// mTLS cert path.
    #[serde(default)]
    pub tls_client_cert: Option<String>,
    /// mTLS key path.
    #[serde(default)]
    pub tls_client_key: Option<String>,
}

impl ImportHostEntry {
    /// Converts to a domain [`VpsRecord`], validating port ∈ 1..=65535 when set.
    ///
    /// # Errors
    /// Returns a human-readable message when `port` is out of range, when the
    /// required `host`/`username` is missing, or when the domain rejects the
    /// record.
    pub fn into_record(self, map_key: &str, defaults: ImportDefaults) -> Result<VpsRecord, String> {
        // G-TYPE-10: domain try_new — empty host/user rejected (no silent defaults).
        let port_u64 = self.port.unwrap_or(22);
        let port = match u16::try_from(port_u64) {
            Ok(p) => p,
            Err(_) => {
                return Err(format!(
                    "invalid SSH port {port_u64} (use 1..=65535) for host '{map_key}'"
                ));
            }
        };
        let name = self.name.unwrap_or_else(|| map_key.to_string());
        let host = self
            .host
            .filter(|h| !h.trim().is_empty())
            .ok_or_else(|| format!("missing host for import key '{map_key}'"))?;
        let username = self
            .username
            .filter(|u| !u.trim().is_empty())
            .ok_or_else(|| format!("missing username for import key '{map_key}'"))?;
        let password = secrecy::SecretString::from(self.password.unwrap_or_default());
        let key_passphrase = self.key_passphrase.map(secrecy::SecretString::from);
        let sudo_password = self.sudo_password.map(secrecy::SecretString::from);
        let su_password = self.su_password.map(secrecy::SecretString::from);
        let mut record = VpsRecord::try_new(
            name,
            host,
            port,
            username,
            password,
            self.key_path.as_deref(),
            key_passphrase,
            Some(self.timeout_ms.unwrap_or(defaults.timeout_ms)),
            Some(self.max_command_chars.unwrap_or(defaults.max_command_chars)),
            Some(self.max_output_chars.unwrap_or(defaults.max_output_chars)),
            sudo_password,
            su_password,
            self.disable_sudo.unwrap_or(false),
        )?;
        record.schema_version = self.schema_version.unwrap_or(defaults.schema_version);
        if let Some(a) = self.added_at {
            record.added_at = crate::domain::Rfc3339Utc::try_new(a).map_err(|e| e.to_string())?;
        }
        if let Some(tags) = self.tags {
            record.set_tags_from_raw(tags).map_err(|e| e.to_string())?;
        }
        record.tls = self.tls.unwrap_or(false);
        record.tls_sni = self.tls_sni;
        record.tls_client_cert = self.tls_client_cert;
        record.tls_client_key = self.tls_client_key;
        // Agent auth round-trips: without these two lines an export from an
        // agent-authenticated host imported as a host with no working auth.
        record.use_agent = self.use_agent.unwrap_or(false);
        record.agent_socket = self.agent_socket;
        Ok(record)
    }
}

/// Default field values applied when import JSON omits them.
#[derive(Debug, Clone, Copy)]
pub struct ImportDefaults {
    /// Default timeout.
    pub timeout_ms: u64,
    /// Default command limit.
    pub max_command_chars: usize,
    /// Default output limit.
    pub max_output_chars: usize,
    /// Default schema version.
    pub schema_version: u32,
}
