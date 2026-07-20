// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: VPS inventory JSON DTOs (extracted from json_wire monólito).
#![forbid(unsafe_code)]
//! Masked VPS / export / import wire types for agent inventory exchange.

use crate::domain::secret_nonempty;
use crate::masking::mask;
use crate::vps::model::VpsRecord;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Masked VPS record for `vps list|show --json` (secrets never raw).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaskedVpsJson {
    /// Logical name.
    pub name: String,
    /// Hostname or IP.
    pub host: String,
    /// SSH port.
    pub port: u16,
    /// SSH username (`user` on wire, not `username`).
    pub user: String,
    /// `null` when empty/key-only; `"***"` when present.
    pub password: Option<String>,
    /// Key path when set.
    pub key_path: Option<String>,
    /// Masked when present.
    pub key_passphrase: Option<String>,
    /// Masked when present.
    pub sudo_password: Option<String>,
    /// Masked when present.
    pub su_password: Option<String>,
    /// Timeout ms.
    pub timeout_ms: u64,
    /// Command char limit.
    pub max_command_chars: usize,
    /// Output char limit.
    pub max_output_chars: usize,
    /// Sudo/su disabled flag.
    pub disable_sudo: bool,
    /// Schema version.
    pub schema_version: u32,
    /// RFC 3339 added-at.
    pub added_at: String,
    /// Host tags (G-O2 / G-SERDE-06).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// SSH-over-TLS enabled.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tls: bool,
    /// TLS SNI override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_sni: Option<String>,
    /// mTLS client cert path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_client_cert: Option<String>,
    /// mTLS client key path (path only; never key material).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_client_key: Option<String>,
}

impl From<&VpsRecord> for MaskedVpsJson {
    fn from(r: &VpsRecord) -> Self {
        use crate::domain::secret_nonempty;
        let password = if secret_nonempty(&r.password) {
            Some(mask(r.password.expose_secret()).to_string())
        } else {
            None
        };
        Self {
            name: r.name.as_str().to_owned(),
            host: r.host.as_str().to_owned(),
            port: r.port.get(),
            user: r.username.as_str().to_owned(),
            password,
            key_path: r.key_path.as_ref().map(|k| k.to_string_lossy_owned()),
            key_passphrase: r
                .key_passphrase
                .as_ref()
                .map(|s| mask(s.expose_secret()).to_string()),
            sudo_password: r
                .sudo_password
                .as_ref()
                .map(|s| mask(s.expose_secret()).to_string()),
            su_password: r
                .su_password
                .as_ref()
                .map(|s| mask(s.expose_secret()).to_string()),
            timeout_ms: r.timeout_ms.get(),
            max_command_chars: r.max_command_chars.wire(),
            max_output_chars: r.max_output_chars.wire(),
            disable_sudo: r.disable_sudo,
            schema_version: r.schema_version,
            added_at: r.added_at.to_rfc3339(),
            tags: r.tags.iter().map(|t| t.as_str().to_owned()).collect(),
            tls: r.tls,
            tls_sni: r.tls_sni.clone(),
            tls_client_cert: r.tls_client_cert.clone(),
            tls_client_key: r.tls_client_key.clone(),
        }
    }
}

/// One host entry inside `vps export --json` (redacted or with secrets).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportHostJson {
    /// Logical name.
    pub name: String,
    /// Hostname or IP.
    pub host: String,
    /// SSH port.
    pub port: u16,
    /// Wire key `user` (export parity with list/show).
    pub user: String,
    /// Password plaintext when include_secrets; [`crate::masking::FIXED_MASK`] when redacted and non-empty; empty string when host has no password (G-E2E-10).
    pub password: String,
    /// Key path.
    pub key_path: Option<String>,
    /// Optional secrets (null when redacted).
    pub key_passphrase: Option<String>,
    /// Optional sudo password.
    pub sudo_password: Option<String>,
    /// Optional su password.
    pub su_password: Option<String>,
    /// Timeout ms.
    pub timeout_ms: u64,
    /// Command char limit.
    pub max_command_chars: usize,
    /// Output char limit.
    pub max_output_chars: usize,
    /// Sudo/su disabled.
    pub disable_sudo: bool,
    /// Schema version.
    pub schema_version: u32,
    /// RFC 3339.
    pub added_at: String,
    /// Host tags (G-O2 / G-SERDE-06).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// SSH-over-TLS.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tls: bool,
    /// TLS SNI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_sni: Option<String>,
    /// mTLS cert path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_client_cert: Option<String>,
    /// mTLS key path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_client_key: Option<String>,
}

impl ExportHostJson {
    /// Builds export entry from a record.
    #[must_use]
    pub fn from_record(r: &VpsRecord, include_secrets: bool) -> Self {
        let tags: Vec<String> = r.tags.iter().map(|t| t.as_str().to_owned()).collect();
        let key_path = r.key_path.as_ref().map(|k| k.to_string_lossy_owned());
        let tls = r.tls;
        let tls_sni = r.tls_sni.clone();
        let tls_client_cert = r.tls_client_cert.clone();
        let tls_client_key = r.tls_client_key.clone();
        if include_secrets {
            Self {
                name: r.name.as_str().to_owned(),
                host: r.host.as_str().to_owned(),
                port: r.port.get(),
                user: r.username.as_str().to_owned(),
                password: r.password.expose_secret().to_string(),
                key_path,
                key_passphrase: r
                    .key_passphrase
                    .as_ref()
                    .map(|s| s.expose_secret().to_string()),
                sudo_password: r
                    .sudo_password
                    .as_ref()
                    .map(|s| s.expose_secret().to_string()),
                su_password: r
                    .su_password
                    .as_ref()
                    .map(|s| s.expose_secret().to_string()),
                timeout_ms: r.timeout_ms.get(),
                max_command_chars: r.max_command_chars.wire(),
                max_output_chars: r.max_output_chars.wire(),
                disable_sudo: r.disable_sudo,
                schema_version: r.schema_version,
                added_at: r.added_at.to_rfc3339(),
                tags,
                tls,
                tls_sni,
                tls_client_cert,
                tls_client_key,
            }
        } else {
            // G-E2E-10: redacted non-empty secrets use FIXED_MASK (`***`), not `""`.
            // Empty password stays `""` so agents distinguish key-only hosts.
            let password = if secret_nonempty(&r.password) {
                mask(r.password.expose_secret()).to_string()
            } else {
                String::new()
            };
            Self {
                name: r.name.as_str().to_owned(),
                host: r.host.as_str().to_owned(),
                port: r.port.get(),
                user: r.username.as_str().to_owned(),
                password,
                key_path,
                key_passphrase: r
                    .key_passphrase
                    .as_ref()
                    .filter(|s| secret_nonempty(s))
                    .map(|s| mask(s.expose_secret()).to_string()),
                sudo_password: r
                    .sudo_password
                    .as_ref()
                    .filter(|s| secret_nonempty(s))
                    .map(|s| mask(s.expose_secret()).to_string()),
                su_password: r
                    .su_password
                    .as_ref()
                    .filter(|s| secret_nonempty(s))
                    .map(|s| mask(s.expose_secret()).to_string()),
                timeout_ms: r.timeout_ms.get(),
                max_command_chars: r.max_command_chars.wire(),
                max_output_chars: r.max_output_chars.wire(),
                disable_sudo: r.disable_sudo,
                schema_version: r.schema_version,
                added_at: r.added_at.to_rfc3339(),
                tags,
                tls,
                tls_sni,
                tls_client_cert,
                tls_client_key,
            }
        }
    }
}

/// `vps export --json` envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpsExportJson {
    /// Always `true`.
    pub ok: bool,
    /// Discriminator: `"vps-export"`.
    pub event: String,
    /// File schema version.
    pub schema_version: u32,
    /// Whether secrets were included.
    pub include_secrets: bool,
    /// Host map (stable key order via `BTreeMap`).
    pub hosts: BTreeMap<String, ExportHostJson>,
}

// ---------------------------------------------------------------------------
// Import DTOs (Must-Ignore unknown fields; dual-read EN + legacy PT)
// ---------------------------------------------------------------------------

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
    /// Returns a human-readable message when `port` is out of range.
    pub fn into_record(
        self,
        map_key: &str,
        defaults: ImportDefaults,
    ) -> Result<VpsRecord, String> {
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
            Some(
                self.max_command_chars
                    .unwrap_or(defaults.max_command_chars),
            ),
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
            record
                .set_tags_from_raw(tags)
                .map_err(|e| e.to_string())?;
        }
        record.tls = self.tls.unwrap_or(false);
        record.tls_sni = self.tls_sni;
        record.tls_client_cert = self.tls_client_cert;
        record.tls_client_key = self.tls_client_key;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_wire::{
        strip_utf8_bom, ErrorEnvelope, ExecutionJson, SuccessEnvelope, UTF8_BOM,
    };
    use crate::ssh::ExecutionOutput;
    use crate::vps::model::{
        CURRENT_SCHEMA_VERSION, DEFAULT_MAX_COMMAND_CHARS, DEFAULT_MAX_OUTPUT_CHARS,
        DEFAULT_TIMEOUT_MS,
    };
    use secrecy::SecretString;

    fn sample_record() -> VpsRecord {
        VpsRecord::test_new(
            "vps-teste",
            "1.2.3.4",
            22,
            "root",
            SecretString::from("senha-super-secreta".to_string()),
            None,
            None,
            Some(5000),
            Some(1000),
            Some(1000),
            Some(SecretString::from("sudo-password-longa-aqui".to_string())),
            None,
            false,
        )
    }

    #[test]
    fn strip_bom_removes_leading_feff() {
        let with = format!("{UTF8_BOM}{{\"ok\":true}}");
        assert_eq!(strip_utf8_bom(&with), "{\"ok\":true}");
        assert_eq!(strip_utf8_bom("{\"ok\":true}"), "{\"ok\":true}");
    }

    #[test]
    fn compact_json_is_single_line() {
        let env = ErrorEnvelope {
            exit_code: 65,
            error_code: "invalid_argument".into(),
            message: "bad".into(),
            remote_exit_code: None,
            error_class: crate::errors::ErrorClass::Permanent,
            retryable: false,
            suggestion: None,
        };
        let s = serde_json::to_string(&env).unwrap();
        assert!(!s.contains('\n'), "agent wire must be compact: {s}");
        assert!(s.starts_with('{'));
        assert!(s.contains("\"exit_code\":65"));
    }

    #[test]
    fn masked_vps_roundtrip_value_eq() {
        let r = sample_record();
        let m = MaskedVpsJson::from(&r);
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(v["name"], "vps-teste");
        assert_eq!(v["user"], "root");
        assert_eq!(v["password"], "***");
        assert_eq!(v["sudo_password"], "***");
        assert!(v["su_password"].is_null());
        let back: MaskedVpsJson = serde_json::from_value(v).unwrap();
        assert_eq!(back.name, m.name);
        assert_eq!(back.password.as_deref(), Some("***"));
    }

    #[test]
    fn execution_json_from_output() {
        let o = ExecutionOutput {
            stdout: "out".into(),
            stderr: "err".into(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: true,
            duration_ms: 42,
        };
        let j = ExecutionJson::from(&o);
        let s = serde_json::to_string(&j).unwrap();
        assert!(s.contains("\"duration_ms\":42"));
        assert!(s.contains("\"truncated_stderr\":true"));
    }

    #[test]
    fn import_envelope_accepts_user_alias_and_unknown_fields() {
        let raw = r#"{
            "event": "vps-export",
            "extra_future": 1,
            "hosts": {
                "h1": {
                    "host": "10.0.0.1",
                    "user": "admin",
                    "port": 2222,
                    "password": "p",
                    "future_field": true
                }
            }
        }"#;
        let env: ImportEnvelope = serde_json::from_str(raw).unwrap();
        let rec = env.hosts["h1"]
            .clone()
            .into_record(
                "h1",
                ImportDefaults {
                    timeout_ms: DEFAULT_TIMEOUT_MS,
                    max_command_chars: DEFAULT_MAX_COMMAND_CHARS,
                    max_output_chars: DEFAULT_MAX_OUTPUT_CHARS,
                    schema_version: CURRENT_SCHEMA_VERSION,
                },
            )
            .unwrap();
        assert_eq!(rec.username.as_str(), "admin");
        assert_eq!(rec.port.get(), 2222);
        assert_eq!(rec.host.as_str(), "10.0.0.1");
    }

    #[test]
    fn import_rejects_port_out_of_range() {
        let entry = ImportHostEntry {
            host: Some("h".into()),
            port: Some(70000),
            username: Some("u".into()),
            password: Some("p".into()),
            ..Default::default()
        };
        let err = entry
            .into_record(
                "bad",
                ImportDefaults {
                    timeout_ms: DEFAULT_TIMEOUT_MS,
                    max_command_chars: DEFAULT_MAX_COMMAND_CHARS,
                    max_output_chars: DEFAULT_MAX_OUTPUT_CHARS,
                    schema_version: CURRENT_SCHEMA_VERSION,
                },
            )
            .unwrap_err();
        assert!(err.contains("70000"), "{err}");
    }

    #[test]
    fn export_envelope_stable_keys_btree() {
        let mut hosts = BTreeMap::new();
        let r = sample_record();
        hosts.insert("b".into(), ExportHostJson::from_record(&r, false));
        hosts.insert("a".into(), ExportHostJson::from_record(&r, false));
        let env = VpsExportJson {
            ok: true,
            event: "vps-export".into(),
            schema_version: CURRENT_SCHEMA_VERSION,
            include_secrets: false,
            hosts,
        };
        let s = serde_json::to_string(&env).unwrap();
        let pos_a = s.find("\"a\":").unwrap();
        let pos_b = s.find("\"b\":").unwrap();
        assert!(pos_a < pos_b, "BTreeMap must emit sorted keys: {s}");
        assert!(!s.contains('\n'));
    }

    #[test]
    fn success_envelope_flattens_fields() {
        let mut fields = BTreeMap::new();
        fields.insert("name".into(), serde_json::json!("x"));
        let e = SuccessEnvelope::new("vps-added", fields);
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["event"], "vps-added");
        assert_eq!(v["name"], "x");
    }

    #[test]
    fn tags_roundtrip_export_import_json() {
        let mut r = sample_record();
        r.set_tags_from_raw(["prod", "web"]).unwrap();
        let export = ExportHostJson::from_record(&r, true);
        assert_eq!(export.tags, vec!["prod", "web"]);
        let masked = MaskedVpsJson::from(&r);
        assert_eq!(masked.tags, vec!["prod", "web"]);
        let entry = ImportHostEntry {
            name: Some(r.name.as_str().to_owned()),
            host: Some(r.host.as_str().to_owned()),
            port: Some(u64::from(r.port.get())),
            username: Some(r.username.as_str().to_owned()),
            password: Some("secret".into()),
            timeout_ms: Some(r.timeout_ms.get()),
            max_command_chars: Some(r.max_command_chars.wire()),
            max_output_chars: Some(r.max_output_chars.wire()),
            disable_sudo: Some(false),
            schema_version: Some(CURRENT_SCHEMA_VERSION),
            added_at: Some(r.added_at.to_rfc3339()),
            tags: Some(vec!["prod".into(), "web".into()]),
            ..Default::default()
        };
        let defaults = ImportDefaults {
            timeout_ms: DEFAULT_TIMEOUT_MS,
            max_command_chars: DEFAULT_MAX_COMMAND_CHARS,
            max_output_chars: DEFAULT_MAX_OUTPUT_CHARS,
            schema_version: CURRENT_SCHEMA_VERSION,
        };
        let back = entry.into_record("k", defaults).unwrap();
        assert_eq!(
            back.tags.iter().map(|t| t.as_str()).collect::<Vec<_>>(),
            vec!["prod", "web"]
        );
    }
}
