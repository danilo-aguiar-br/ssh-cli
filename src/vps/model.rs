// SPDX-License-Identifier: MIT OR Apache-2.0
//! Data model for `VpsRecord` (schema v2).
//!
//! Passwords use `SecretString` for automatic zeroize via `Drop`. On-disk TOML is
//! plaintext (mode 0o600) or encrypted (`sshcli-enc:v1:`) when a primary key exists.
//! `Debug` is customized to NEVER expose sensitive values.
//!
//! Schema v2: password **or** key auth, max_command/max_output duality,
//! `disable_sudo`, and automatic migration from legacy `max_chars`.

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

/// Current schema version of the `config.toml` file.
pub const CURRENT_SCHEMA_VERSION: u32 = 2;

/// Default timeout in milliseconds (60s).
pub const DEFAULT_TIMEOUT_MS: u64 = 60_000;

/// Default character limit for the **command** (one-shot maxChars).
pub const DEFAULT_MAX_COMMAND_CHARS: usize = 1_000;

/// Default character limit for captured **output**.
pub const DEFAULT_MAX_OUTPUT_CHARS: usize = 100_000;

/// VPS host record in the configuration file.
#[derive(Clone, Serialize, Deserialize)]
pub struct VpsRecord {
    /// Logical unique VPS name.
    #[serde(rename = "nome")]
    pub name: String,
    /// Server hostname or IP.
    pub host: String,
    /// SSH port.
    #[serde(rename = "porta")]
    pub port: u16,
    /// SSH username.
    #[serde(rename = "usuario")]
    pub username: String,
    /// SSH password (empty when key-only auth).
    #[serde(default, rename = "senha", with = "secret_string_serde")]
    pub password: SecretString,
    /// Absolute or expandable OpenSSH private key path.
    #[serde(default)]
    pub key_path: Option<String>,
    /// Private key passphrase (optional).
    #[serde(default, with = "opcao_secret_string_serde")]
    pub key_passphrase: Option<SecretString>,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Command character limit (input). `0` = unlimited at runtime.
    #[serde(default = "default_max_command_chars")]
    pub max_command_chars: usize,
    /// Stdout/stderr character limit. Accepts legacy alias `max_chars`.
    #[serde(default = "default_max_output_chars", alias = "max_chars")]
    pub max_output_chars: usize,
    /// Password for `sudo` (optional).
    #[serde(default, rename = "senha_sudo", with = "opcao_secret_string_serde")]
    pub sudo_password: Option<SecretString>,
    /// Password for `su -` (optional).
    #[serde(default, rename = "senha_su", with = "opcao_secret_string_serde")]
    pub su_password: Option<SecretString>,
    /// If true, `sudo-exec` and `su-exec` are rejected for this host.
    #[serde(default)]
    pub disable_sudo: bool,
    /// Schema version for this record.
    pub schema_version: u32,
    /// RFC 3339 inclusion timestamp.
    #[serde(rename = "adicionado_em")]
    pub added_at: String,
}

fn default_max_command_chars() -> usize {
    DEFAULT_MAX_COMMAND_CHARS
}

fn default_max_output_chars() -> usize {
    DEFAULT_MAX_OUTPUT_CHARS
}

impl std::fmt::Debug for VpsRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VpsRecord")
            .field("name", &self.name)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("key_path", &self.key_path)
            .field(
                "key_passphrase",
                &self.key_passphrase.as_ref().map(|_| "<redacted>"),
            )
            .field("timeout_ms", &self.timeout_ms)
            .field("max_command_chars", &self.max_command_chars)
            .field("max_output_chars", &self.max_output_chars)
            .field(
                "senha_sudo",
                &self.sudo_password.as_ref().map(|_| "<redacted>"),
            )
            .field("su_password", &self.su_password.as_ref().map(|_| "<redacted>"))
            .field("disable_sudo", &self.disable_sudo)
            .field("schema_version", &self.schema_version)
            .field("added_at", &self.added_at)
            .finish()
    }
}

impl VpsRecord {
    /// Creates a new record applying defaults.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        host: String,
        port: u16,
        username: String,
        password: SecretString,
        key_path: Option<String>,
        key_passphrase: Option<SecretString>,
        timeout_ms: Option<u64>,
        max_command_chars: Option<usize>,
        max_output_chars: Option<usize>,
        sudo_password: Option<SecretString>,
        su_password: Option<SecretString>,
        disable_sudo: bool,
    ) -> Self {
        Self {
            name,
            host,
            port,
            username,
            password,
            key_path,
            key_passphrase,
            timeout_ms: timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS),
            max_command_chars: max_command_chars.unwrap_or(DEFAULT_MAX_COMMAND_CHARS),
            max_output_chars: max_output_chars.unwrap_or(DEFAULT_MAX_OUTPUT_CHARS),
            sudo_password,
            su_password,
            disable_sudo,
            schema_version: CURRENT_SCHEMA_VERSION,
            added_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Retorna true se há password não vazia.
    #[must_use]
    pub fn has_password(&self) -> bool {
        !self.password.expose_secret().is_empty()
    }

    /// Retorna true se há path de chave privada.
    #[must_use]
    pub fn has_key(&self) -> bool {
        self.key_path.as_ref().is_some_and(|p| !p.trim().is_empty())
    }

    /// Valida que existe pelo menos um método de autenticação.
    pub fn validate_credentials(&self) -> Result<(), String> {
        if !self.has_password() && !self.has_key() {
            return Err(
                "é obrigatório fornecer --password ou --key (auth password ou chave privada)"
                    .to_string(),
            );
        }
        Ok(())
    }

    /// Validação completa do registro na fronteira de escrita (add/edit/import).
    ///
    /// Garante port ∈ 1..=65535, host/usuário não vazios e credenciais presentes.
    /// Não verifica existência de `key_path` no filesystem (isso fica no dispatcher).
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("porta SSH inválida: 0 (use 1..=65535)".to_string());
        }
        if self.host.trim().is_empty() {
            return Err("host não pode ser vazio".to_string());
        }
        if self.username.trim().is_empty() {
            return Err("usuário SSH não pode ser vazio".to_string());
        }
        self.validate_credentials()
    }

    /// Normaliza schema após deserialização (migração v1 → v2).
    pub fn normalize_schema(&mut self) {
        if self.schema_version < CURRENT_SCHEMA_VERSION {
            self.schema_version = CURRENT_SCHEMA_VERSION;
        }
        if self.max_command_chars == 0 && self.max_output_chars == 0 {
            // nada: 0 significa ilimitado na validação de runtime
        }
    }
}

/// Interpreta string de limite (`"none"`, `"0"` ou número).
///
/// `0`/`none` → `0` (ilimitado no runtime).
#[must_use]
pub fn parse_char_limit(s: &str) -> usize {
    let t = s.trim();
    if t.eq_ignore_ascii_case("none") || t == "0" {
        0
    } else {
        t.parse().unwrap_or(DEFAULT_MAX_OUTPUT_CHARS)
    }
}

/// Converte limite de config em valor efetivo para truncagem/validação.
///
/// `0` = sem limite (`usize::MAX` para comparação).
#[must_use]
pub fn effective_limit(configurado: usize) -> usize {
    if configurado == 0 {
        usize::MAX
    } else {
        configurado
    }
}

mod secret_string_serde {
    use super::{ExposeSecret, SecretString};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(valor: &SecretString, s: S) -> Result<S::Ok, S::Error> {
        let plain = valor.expose_secret();
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

    pub fn serialize<S: Serializer>(valor: &Option<SecretString>, s: S) -> Result<S::Ok, S::Error> {
        match valor {
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
    fn novo_registro_aplica_defaults() {
        let r = VpsRecord::new(
            "teste".into(),
            "1.2.3.4".into(),
            22,
            "root".into(),
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
        assert_eq!(r.timeout_ms, DEFAULT_TIMEOUT_MS);
        assert_eq!(r.max_command_chars, DEFAULT_MAX_COMMAND_CHARS);
        assert_eq!(r.max_output_chars, DEFAULT_MAX_OUTPUT_CHARS);
        assert_eq!(r.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(!r.added_at.is_empty());
    }

    #[test]
    fn debug_nao_exibe_senha() {
        let r = VpsRecord::new(
            "t".into(),
            "h".into(),
            22,
            "u".into(),
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
    fn round_trip_toml_preserva_dados() {
        // Isola cifragem at-rest de outros tests (primary-key global).
        let tmp = tempfile::TempDir::new().unwrap();
        crate::secrets::set_config_dir(Some(tmp.path().to_path_buf()));
        // SAFETY:
        // 1. Contract: temporary mutation of process environment for a serial test/setup path.
        // 2. Invariant: no concurrent threads in this process mutate the same env keys.
        // 3. Caller guarantees serial_test::serial (or single-threaded test) around this block.
        // 4. See std::env::set_var / remove_var safety notes for multi-threaded processes.
        unsafe {

            std::env::set_var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS", "1");
        }
        let r = VpsRecord::new(
            "producao".into(),
            "srv.exemplo.com".into(),
            2222,
            "admin".into(),
            SecretString::from("senha-do-admin-longa".to_string()),
            Some("/home/u/.ssh/id_ed25519".into()),
            None,
            Some(5000),
            Some(500),
            Some(50_000),
            Some(SecretString::from("sudopass".to_string())),
            None,
            false,
        );
        let toml_str = toml::to_string(&r).expect("serializar");
        let r2: VpsRecord = toml::from_str(&toml_str).expect("deserializar");
        assert_eq!(r2.name, "producao");
        assert_eq!(r2.port, 2222);
        assert_eq!(r2.password.expose_secret(), "senha-do-admin-longa");
        assert_eq!(r2.key_path.as_deref(), Some("/home/u/.ssh/id_ed25519"));
        assert_eq!(r2.max_command_chars, 500);
        assert_eq!(r2.max_output_chars, 50_000);
        assert_eq!(
            r2.sudo_password
                .as_ref()
                .map(|s| s.expose_secret().to_string()),
            Some("sudopass".to_string())
        );
        assert!(r2.su_password.is_none());
        // SAFETY:

        // 1. Contract: temporary mutation of process environment for a serial test/setup path.

        // 2. Invariant: no concurrent threads in this process mutate the same env keys.

        // 3. Caller guarantees serial_test::serial (or single-threaded test) around this block.

        // 4. See std::env::set_var / remove_var safety notes for multi-threaded processes.

        unsafe {
            std::env::remove_var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS");
        }
        crate::secrets::set_config_dir(None);
    }

    #[test]
    fn migra_max_chars_legado() {
        let legado = r#"
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
        let r: VpsRecord = toml::from_str(legado).expect("deserializar legado");
        assert_eq!(r.max_output_chars, 4242);
        assert_eq!(r.max_command_chars, DEFAULT_MAX_COMMAND_CHARS);
    }

    #[test]
    fn validar_credenciais_exige_password_ou_key() {
        let mut r = VpsRecord::new(
            "t".into(),
            "h".into(),
            22,
            "u".into(),
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
        r.key_path = Some("/tmp/k".into());
        assert!(r.validate_credentials().is_ok());
    }

    #[test]
    fn parse_limite_none_e_zero() {
        assert_eq!(parse_char_limit("none"), 0);
        assert_eq!(parse_char_limit("0"), 0);
        assert_eq!(parse_char_limit("1000"), 1000);
        assert_eq!(effective_limit(0), usize::MAX);
        assert_eq!(effective_limit(10), 10);
    }
}
