// SPDX-License-Identifier: MIT OR Apache-2.0
// G-UNSAFE-01 / G-SECDEV-05: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! Data model for `VpsRecord` (schema v3) — domain newtypes (G-TYPE-*).
//!
//! Passwords use `SecretString` for automatic zeroize via `Drop`. On-disk TOML is
//! plaintext (mode 0o600) or encrypted (`sshcli-enc:v1:`) when a primary key exists.
//! `Debug` is customized to NEVER expose sensitive values.
//!
//! Schema v3: **English wire keys** on serialize (`name`, `port`, `username`, …).
//! Deserialize accepts both EN and legacy Portuguese aliases (`nome`, `porta`, …).
//! Field invariants are encoded in [`crate::domain`] newtypes (parse, don't validate).

use crate::domain::{
    secret_nonempty, try_tags, CharLimit, HostTag, KeyPath, Rfc3339Utc, SshHost, SshPort, SshUser,
    TimeoutMs, VpsName,
};
use crate::validation::MAX_TAGS;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

/// Current schema version of the `config.toml` file.
pub const CURRENT_SCHEMA_VERSION: u32 = 3;

/// Default timeout in milliseconds (60s).
pub const DEFAULT_TIMEOUT_MS: u64 = 60_000;

/// Default character limit for the **command** (one-shot maxChars).
pub const DEFAULT_MAX_COMMAND_CHARS: usize = 1_000;

/// Default character limit for captured **output**.
pub const DEFAULT_MAX_OUTPUT_CHARS: usize = 100_000;

// Compile-time invariants for schema defaults (const/static rules).
const _: () = assert!(CURRENT_SCHEMA_VERSION >= 1);
const _: () = assert!(DEFAULT_TIMEOUT_MS > 0);
const _: () = assert!(DEFAULT_MAX_COMMAND_CHARS > 0);
const _: () = assert!(DEFAULT_MAX_OUTPUT_CHARS >= DEFAULT_MAX_COMMAND_CHARS);

/// VPS host record in the configuration file.
///
/// Wire format (serialize): English field names. Legacy Portuguese keys remain
/// readable via `serde(alias = …)`.
///
/// Field types are domain newtypes — invalid host/port/user/name cannot be
/// constructed after a successful deserialize or [`Self::try_new`].
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VpsRecord {
    /// Logical unique VPS name.
    #[serde(alias = "nome")]
    pub name: VpsName,
    /// Server hostname or IP.
    pub host: SshHost,
    /// SSH port.
    #[serde(alias = "porta")]
    pub port: SshPort,
    /// SSH username.
    #[serde(alias = "usuario")]
    pub username: SshUser,
    /// SSH password (empty when key-only auth).
    #[serde(default, alias = "senha", with = "secret_string_serde")]
    pub password: SecretString,
    /// Absolute or expandable OpenSSH private key path.
    #[serde(default)]
    pub key_path: Option<KeyPath>,
    /// Private key passphrase (optional).
    #[serde(default, with = "opcao_secret_string_serde")]
    pub key_passphrase: Option<SecretString>,
    /// Use ssh-agent for publickey auth (G-SSH-04). Socket from `agent_socket` / CLI.
    #[serde(default)]
    pub use_agent: bool,
    /// Agent Unix socket or Windows named pipe path (XDG; never env-as-store).
    #[serde(default)]
    pub agent_socket: Option<String>,
    /// Timeout in milliseconds.
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: TimeoutMs,
    /// Command character limit (input). `0` = unlimited at runtime.
    #[serde(default = "default_max_command_chars")]
    pub max_command_chars: CharLimit,
    /// Stdout/stderr character limit. Accepts legacy alias `max_chars`.
    #[serde(default = "default_max_output_chars", alias = "max_chars")]
    pub max_output_chars: CharLimit,
    /// Password for `sudo` (optional).
    #[serde(default, alias = "senha_sudo", with = "opcao_secret_string_serde")]
    pub sudo_password: Option<SecretString>,
    /// Password for `su -` (optional).
    #[serde(default, alias = "senha_su", with = "opcao_secret_string_serde")]
    pub su_password: Option<SecretString>,
    /// If true, `sudo-exec` and `su-exec` are rejected for this host.
    #[serde(default)]
    pub disable_sudo: bool,
    /// Schema version for this record.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// RFC 3339 inclusion timestamp (`DateTime<Utc>` newtype).
    #[serde(default = "default_added_at", alias = "adicionado_em")]
    pub added_at: Rfc3339Utc,
    /// Optional host tags for fleet selection (G-O2). Empty on legacy records.
    #[serde(default)]
    pub tags: Vec<HostTag>,
    /// When true, wrap SSH in TLS (SSH-over-TLS via rustls) before the SSH handshake.
    #[serde(default)]
    pub tls: bool,
    /// SNI / certificate name for TLS (defaults to `host` when empty/`None`).
    #[serde(default)]
    pub tls_sni: Option<String>,
    /// Client certificate PEM path for mTLS (optional).
    #[serde(default)]
    pub tls_client_cert: Option<String>,
    /// Client private key PEM path for mTLS (optional; required with cert).
    #[serde(default)]
    pub tls_client_key: Option<String>,
}

fn default_max_command_chars() -> CharLimit {
    CharLimit::try_new(DEFAULT_MAX_COMMAND_CHARS).expect("default command limit in range")
}

fn default_max_output_chars() -> CharLimit {
    CharLimit::try_new(DEFAULT_MAX_OUTPUT_CHARS).expect("default output limit in range")
}

fn default_timeout_ms() -> TimeoutMs {
    TimeoutMs::try_new(DEFAULT_TIMEOUT_MS).expect("default timeout in range")
}

fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

fn default_added_at() -> Rfc3339Utc {
    Rfc3339Utc::now()
}

impl std::fmt::Debug for VpsRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VpsRecord")
            .field("name", &self.name.as_str())
            .field("host", &self.host.as_str())
            .field("port", &self.port.get())
            .field("username", &self.username.as_str())
            .field("password", &"<redacted>")
            .field("key_path", &self.key_path.as_ref().map(|k| k.as_path()))
            .field(
                "key_passphrase",
                &self.key_passphrase.as_ref().map(|_| "<redacted>"),
            )
            .field("timeout_ms", &self.timeout_ms.get())
            .field("max_command_chars", &self.max_command_chars.wire())
            .field("max_output_chars", &self.max_output_chars.wire())
            .field(
                "sudo_password",
                &self.sudo_password.as_ref().map(|_| "<redacted>"),
            )
            .field("su_password", &self.su_password.as_ref().map(|_| "<redacted>"))
            .field("disable_sudo", &self.disable_sudo)
            .field("schema_version", &self.schema_version)
            .field("added_at", &self.added_at.to_rfc3339())
            .field("tags", &self.tags)
            .field("tls", &self.tls)
            .field("use_agent", &self.use_agent)
            .field("agent_socket", &self.agent_socket)
            .field("tls_sni", &self.tls_sni)
            .field("tls_client_cert", &self.tls_client_cert)
            .field(
                "tls_client_key",
                &self.tls_client_key.as_ref().map(|_| "<redacted-path>"),
            )
            .finish()
    }
}

impl VpsRecord {
    /// Creates a new record applying defaults (validated — G-TYPE-06).
    ///
    /// # Errors
    /// Returns a message when any field fails domain parse.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        name: impl AsRef<str>,
        host: impl AsRef<str>,
        port: u16,
        username: impl AsRef<str>,
        password: SecretString,
        key_path: Option<impl AsRef<str>>,
        key_passphrase: Option<SecretString>,
        timeout_ms: Option<u64>,
        max_command_chars: Option<usize>,
        max_output_chars: Option<usize>,
        sudo_password: Option<SecretString>,
        su_password: Option<SecretString>,
        disable_sudo: bool,
    ) -> Result<Self, String> {
        let key_path = KeyPath::try_from_optional(key_path).map_err(|e| e.to_string())?;
        Ok(Self {
            name: VpsName::try_new(name).map_err(|e| e.to_string())?,
            host: SshHost::try_new(host).map_err(|e| e.to_string())?,
            port: SshPort::try_new(port).map_err(|e| e.to_string())?,
            username: SshUser::try_new(username).map_err(|e| e.to_string())?,
            password,
            key_path,
            key_passphrase,
            use_agent: false,
            agent_socket: None,
            timeout_ms: TimeoutMs::try_new(timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS))
                .map_err(|e| e.to_string())?,
            max_command_chars: CharLimit::try_new(
                max_command_chars.unwrap_or(DEFAULT_MAX_COMMAND_CHARS),
            )
            .map_err(|e| e.to_string())?,
            max_output_chars: CharLimit::try_new(
                max_output_chars.unwrap_or(DEFAULT_MAX_OUTPUT_CHARS),
            )
            .map_err(|e| e.to_string())?,
            sudo_password,
            su_password,
            disable_sudo,
            schema_version: CURRENT_SCHEMA_VERSION,
            added_at: Rfc3339Utc::now(),
            tags: Vec::new(),
            tls: false,
            tls_sni: None,
            tls_client_cert: None,
            tls_client_key: None,
        })
    }

    /// Test/helper constructor that panics on invalid input (not public API).
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn test_new(
        name: impl AsRef<str>,
        host: impl AsRef<str>,
        port: u16,
        username: impl AsRef<str>,
        password: SecretString,
        key_path: Option<&str>,
        key_passphrase: Option<SecretString>,
        timeout_ms: Option<u64>,
        max_command_chars: Option<usize>,
        max_output_chars: Option<usize>,
        sudo_password: Option<SecretString>,
        su_password: Option<SecretString>,
        disable_sudo: bool,
    ) -> Self {
        Self::try_new(
            name,
            host,
            port,
            username,
            password,
            key_path,
            key_passphrase,
            timeout_ms,
            max_command_chars,
            max_output_chars,
            sudo_password,
            su_password,
            disable_sudo,
        )
        .expect("test_new requires valid domain fields")
    }

    /// Returns true if this host has **any** of the requested tags (OR match, G-O2).
    #[must_use]
    pub fn has_any_tag(&self, wanted: &[HostTag]) -> bool {
        if wanted.is_empty() {
            return true;
        }
        wanted
            .iter()
            .any(|w| self.tags.iter().any(|t| t.as_str() == w.as_str()))
    }

    /// Returns true if there is a non-empty password (G-TYPE-12).
    #[must_use]
    pub fn has_password(&self) -> bool {
        secret_nonempty(&self.password)
    }

    /// Returns true if there is a private key path.
    #[must_use]
    pub fn has_key(&self) -> bool {
        self.key_path.is_some()
    }

    /// Validates primary authentication: **exactly one** of password / key / agent (G-AUD-07).
    ///
    /// # Errors
    /// [`DomainError`] when zero or more than one primary method is set.
    pub fn validate_credentials(&self) -> Result<(), crate::domain::DomainError> {
        let n = u8::from(self.has_password()) + u8::from(self.has_key()) + u8::from(self.use_agent);
        if n == 0 {
            return Err(crate::domain::DomainError::new(
                "vps_auth",
                "must provide exactly one of --password, --key, or --use-agent",
            ));
        }
        if n > 1 {
            return Err(crate::domain::DomainError::new(
                "vps_auth",
                "primary auth methods are mutually exclusive: use only one of --password, --key, or --use-agent",
            ));
        }
        Ok(())
    }

    /// Structural validation (G-SERDE-04 / G-TYPE): tags cardinality + field proofs already in types.
    ///
    /// # Errors
    /// [`DomainError`] when tag cardinality exceeds the limit.
    pub fn validate_structure(&self) -> Result<(), crate::domain::DomainError> {
        if self.tags.len() > MAX_TAGS {
            return Err(crate::domain::DomainError::new(
                "tags",
                format!("at most {MAX_TAGS} tags allowed"),
            ));
        }
        // `added_at` proof lives in `Rfc3339Utc` (RFC 3339 parse on deserialize).
        Ok(())
    }

    /// Full record validation at the write boundary (add/edit/import).
    ///
    /// # Errors
    /// Propagates [`DomainError`] from structure or credentials checks.
    pub fn validate(&self) -> Result<(), crate::domain::DomainError> {
        self.validate_structure()?;
        self.validate_credentials()
    }

    /// Normalizes schema after deserialization (v1 → v2 migration).
    pub fn normalize_schema(&mut self) {
        if self.schema_version < CURRENT_SCHEMA_VERSION {
            self.schema_version = CURRENT_SCHEMA_VERSION;
        }
    }

    /// Sets tags from raw strings (validated).
    pub fn set_tags_from_raw(
        &mut self,
        raw: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<(), crate::domain::DomainError> {
        self.tags = try_tags(raw)?;
        Ok(())
    }
}

/// Parses a limit string (`"none"`, `"0"`, or a number) into wire `usize`.
///
/// `0`/`none` → `0` (unlimited at runtime).
#[must_use]
pub fn parse_char_limit(s: &str) -> usize {
    let t = s.trim();
    if t.eq_ignore_ascii_case("none") || t == "0" {
        0
    } else {
        t.parse().unwrap_or(DEFAULT_MAX_OUTPUT_CHARS)
    }
}

/// Converts a config wire limit into the effective value for truncation/validation.
///
/// `0` = unlimited (`usize::MAX` for comparison).
#[must_use]
pub fn effective_limit(configured: usize) -> usize {
    CharLimit::try_new(configured)
        .map(|c| c.effective())
        .unwrap_or(usize::MAX)
}

/// Effective limit from a [`CharLimit`].
#[must_use]
pub fn effective_char_limit(limit: CharLimit) -> usize {
    limit.effective()
}

mod secret_string_serde {
    use super::{ExposeSecret, SecretString};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &SecretString, s: S) -> Result<S::Ok, S::Error> {
        let plain = value.expose_secret();
        let out = crate::secrets::serialize_secret(plain).map_err(serde::ser::Error::custom)?;
        s.serialize_str(&out)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<SecretString, D::Error> {
        let s = String::deserialize(d)?;
        let plain = crate::secrets::deserialize_secret(&s).map_err(serde::de::Error::custom)?;
        Ok(SecretString::from(plain))
    }
}

mod opcao_secret_string_serde {
    use super::{ExposeSecret, SecretString};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &Option<SecretString>, s: S) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => {
                let out = crate::secrets::serialize_secret(v.expose_secret())
                    .map_err(serde::ser::Error::custom)?;
                s.serialize_some(&out)
            }
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<SecretString>, D::Error> {
        let opt = Option::<String>::deserialize(d)?;
        match opt {
            None => Ok(None),
            Some(s) => {
                let plain =
                    crate::secrets::deserialize_secret(&s).map_err(serde::de::Error::custom)?;
                Ok(Some(SecretString::from(plain)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_applies_defaults() {
        let r = VpsRecord::test_new(
            "teste",
            "1.2.3.4",
            22,
            "root",
            SecretString::from("senha".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        );
        assert_eq!(r.timeout_ms.get(), DEFAULT_TIMEOUT_MS);
        assert_eq!(r.max_command_chars.wire(), DEFAULT_MAX_COMMAND_CHARS);
        assert_eq!(r.max_output_chars.wire(), DEFAULT_MAX_OUTPUT_CHARS);
        assert_eq!(r.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(!r.added_at.to_rfc3339().is_empty());
    }

    #[test]
    fn try_new_rejects_port_zero() {
        let err = VpsRecord::try_new(
            "t",
            "h",
            0,
            "u",
            SecretString::from("p".to_string()),
            None::<&str>,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        )
        .unwrap_err();
        assert!(err.contains("port") || err.contains("0"), "{err}");
    }

    #[test]
    fn debug_does_not_show_password() {
        let r = VpsRecord::test_new(
            "t",
            "h",
            22,
            "u",
            SecretString::from("senha-super-secreta".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        );
        let dbg = format!("{r:?}");
        assert!(!dbg.contains("senha-super-secreta"));
        assert!(dbg.contains("redacted"));
    }

    #[test]
    #[serial_test::serial]
    fn round_trip_toml_preserves_data() {
        let tmp = tempfile::TempDir::new().unwrap();
        crate::secrets::set_config_dir(Some(tmp.path().to_path_buf()));
        // G-UNSAFE-02: plaintext opt-out is CLI/runtime flag only (no env store).
        crate::secrets::set_runtime_flags(true, None, false);
        let r = VpsRecord::test_new(
            "producao",
            "srv.exemplo.com",
            2222,
            "admin",
            SecretString::from("senha-do-admin-longa".to_string()),
            Some("/home/u/.ssh/id_ed25519"),
            None,
            Some(5000),
            Some(500),
            Some(50_000),
            Some(SecretString::from("sudopass".to_string())),
            None,
            false,
        );
        let toml_str = toml::to_string(&r).expect("serialize");
        let r2: VpsRecord = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(r2.name.as_str(), "producao");
        assert_eq!(r2.port.get(), 2222);
        assert_eq!(r2.password.expose_secret(), "senha-do-admin-longa");
        assert_eq!(
            r2.key_path.as_ref().map(|k| k.to_string_lossy_owned()),
            Some("/home/u/.ssh/id_ed25519".into())
        );
        assert_eq!(r2.max_command_chars.wire(), 500);
        assert_eq!(r2.max_output_chars.wire(), 50_000);
        assert_eq!(
            r2.sudo_password
                .as_ref()
                .map(|s| s.expose_secret().to_string()),
            Some("sudopass".to_string())
        );
        assert!(r2.su_password.is_none());
        crate::secrets::set_runtime_flags(false, None, false);
        crate::secrets::set_config_dir(None);
    }

    #[test]
    fn migrates_legacy_max_chars() {
        let legacy = r#"
nome = "x"
host = "h"
porta = 22
usuario = "u"
senha = "s"
timeout_ms = 30000
max_chars = 4242
schema_version = 1
adicionado_em = "2020-01-01T00:00:00Z"
"#;
        let r: VpsRecord = toml::from_str(legacy).expect("deserialize legacy PT wire");
        assert_eq!(r.max_output_chars.wire(), 4242);
        assert_eq!(r.max_command_chars.wire(), DEFAULT_MAX_COMMAND_CHARS);
        assert_eq!(r.name.as_str(), "x");
        assert_eq!(r.port.get(), 22);
        assert_eq!(r.username.as_str(), "u");
    }

    #[test]
    fn deserializes_english_wire_keys() {
        let en = r#"
name = "prod"
host = "h.example"
port = 2222
username = "admin"
password = "secret"
timeout_ms = 5000
schema_version = 3
"#;
        let r: VpsRecord = toml::from_str(en).expect("deserialize EN wire");
        assert_eq!(r.name.as_str(), "prod");
        assert_eq!(r.port.get(), 2222);
        assert_eq!(r.username.as_str(), "admin");
        assert!(!r.added_at.to_rfc3339().is_empty());
    }

    #[test]
    #[serial_test::serial]
    fn serializes_english_wire_keys() {
        let tmp = tempfile::TempDir::new().unwrap();
        crate::secrets::set_config_dir(Some(tmp.path().to_path_buf()));
        crate::secrets::set_runtime_flags(true, None, false);
        let r = VpsRecord::test_new(
            "prod",
            "h",
            22,
            "u",
            SecretString::from("p".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        );
        let s = toml::to_string(&r).expect("serialize");
        assert!(s.contains("name ="), "expected EN key name: {s}");
        assert!(s.contains("port ="), "expected EN key port: {s}");
        assert!(s.contains("username ="), "expected EN key username: {s}");
        assert!(s.contains("password ="), "expected EN key password: {s}");
        assert!(s.contains("added_at ="), "expected EN key added_at: {s}");
        assert!(!s.contains("nome ="), "must not write PT key nome: {s}");
        assert!(!s.contains("porta ="), "must not write PT key porta: {s}");
        assert!(!s.contains("adicionado_em ="), "must not write PT adicionado_em: {s}");
        crate::secrets::set_runtime_flags(false, None, false);
        crate::secrets::set_config_dir(None);
    }

    #[test]
    fn deserializes_without_added_at() {
        let bare = r#"
nome = "x"
host = "h"
porta = 22
usuario = "u"
senha = "s"
schema_version = 2
"#;
        let r: VpsRecord = toml::from_str(bare).expect("default added_at");
        assert!(!r.added_at.to_rfc3339().is_empty());
    }

    #[test]
    fn validate_credentials_requires_password_or_key() {
        let mut r = VpsRecord::test_new(
            "t",
            "h",
            22,
            "u",
            SecretString::from(String::new()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        );
        assert!(r.validate_credentials().is_err());
        r.key_path = Some(KeyPath::try_new("/tmp/k").unwrap());
        assert!(r.validate_credentials().is_ok());
    }

    #[test]
    fn deny_unknown_fields_on_vps_record() {
        let bad = r#"
name = "x"
host = "1.2.3.4"
port = 22
username = "root"
password = "p"
timeuot_ms = 1
"#;
        let err = toml::from_str::<VpsRecord>(bad).unwrap_err();
        let s = err.to_string();
        assert!(
            s.contains("unknown") || s.contains("timeuot") || s.contains("did not expect"),
            "expected deny_unknown, got: {s}"
        );
    }

    #[test]
    fn deserialize_rejects_empty_host() {
        let bad = r#"
name = "n"
host = "  "
port = 22
username = "root"
password = "p"
"#;
        assert!(toml::from_str::<VpsRecord>(bad).is_err());
    }

    #[test]
    fn parse_limit_none_and_zero() {
        assert_eq!(parse_char_limit("none"), 0);
        assert_eq!(parse_char_limit("0"), 0);
        assert_eq!(parse_char_limit("1000"), 1000);
        assert_eq!(effective_limit(0), usize::MAX);
        assert_eq!(effective_limit(10), 10);
    }
}
