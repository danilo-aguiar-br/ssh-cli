//! Modelo de dados `VpsRegistro` (schema v2).
//!
//! Senhas usam `SecretString` para zeroize automático via `Drop`. O TOML
//! gravado em disco: texto claro (0o600) ou cifrado (`sshcli-enc:v1:`) se houver chave mestra.
//! `Debug` é customizado para NUNCA expor valores sensíveis.
//!
//! Schema v2: auth por senha **ou** chave, dualidade max_command/max_output,
//! `disable_sudo` e migração automática de `max_chars` legado.

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

/// Versão atual do schema do arquivo `config.toml`.
pub const SCHEMA_VERSION_ATUAL: u32 = 2;

/// Timeout padrão em milissegundos (default 60s).
pub const TIMEOUT_PADRAO_MS: u64 = 60_000;

/// Limite padrão de caracteres no **comando** (automação one-shot maxChars).
pub const MAX_COMMAND_CHARS_PADRAO: usize = 1_000;

/// Limite padrão de caracteres em **output** capturado.
pub const MAX_OUTPUT_CHARS_PADRAO: usize = 100_000;

/// Registro de uma VPS no arquivo de configuração.
#[derive(Clone, Serialize, Deserialize)]
pub struct VpsRegistro {
    /// Nome lógico único da VPS.
    pub nome: String,
    /// Hostname ou IP do servidor.
    pub host: String,
    /// Porta SSH.
    pub porta: u16,
    /// Usuário SSH.
    pub usuario: String,
    /// Senha SSH (vazia se auth só por chave).
    #[serde(default, with = "secret_string_serde")]
    pub senha: SecretString,
    /// Caminho absoluto ou expandível para chave privada OpenSSH.
    #[serde(default)]
    pub key_path: Option<String>,
    /// Passphrase da chave privada (opcional).
    #[serde(default, with = "opcao_secret_string_serde")]
    pub key_passphrase: Option<SecretString>,
    /// Timeout em milissegundos.
    pub timeout_ms: u64,
    /// Limite de caracteres do comando (entrada). `0` = ilimitado em runtime.
    #[serde(default = "default_max_command_chars")]
    pub max_command_chars: usize,
    /// Limite de caracteres de stdout/stderr. Aceita alias legado `max_chars`.
    #[serde(default = "default_max_output_chars", alias = "max_chars")]
    pub max_output_chars: usize,
    /// Senha para `sudo` (opcional).
    #[serde(default, with = "opcao_secret_string_serde")]
    pub senha_sudo: Option<SecretString>,
    /// Senha para `su -` (opcional).
    #[serde(default, with = "opcao_secret_string_serde")]
    pub senha_su: Option<SecretString>,
    /// Se true, `sudo-exec` e `su-exec` são recusados para este host.
    #[serde(default)]
    pub disable_sudo: bool,
    /// Versão do schema deste registro.
    pub schema_version: u32,
    /// Timestamp RFC 3339 de inclusão.
    pub adicionado_em: String,
}

fn default_max_command_chars() -> usize {
    MAX_COMMAND_CHARS_PADRAO
}

fn default_max_output_chars() -> usize {
    MAX_OUTPUT_CHARS_PADRAO
}

impl std::fmt::Debug for VpsRegistro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VpsRegistro")
            .field("nome", &self.nome)
            .field("host", &self.host)
            .field("porta", &self.porta)
            .field("usuario", &self.usuario)
            .field("senha", &"<redacted>")
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
                &self.senha_sudo.as_ref().map(|_| "<redacted>"),
            )
            .field("senha_su", &self.senha_su.as_ref().map(|_| "<redacted>"))
            .field("disable_sudo", &self.disable_sudo)
            .field("schema_version", &self.schema_version)
            .field("adicionado_em", &self.adicionado_em)
            .finish()
    }
}

impl VpsRegistro {
    /// Cria um novo registro aplicando defaults.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn novo(
        nome: String,
        host: String,
        porta: u16,
        usuario: String,
        senha: SecretString,
        key_path: Option<String>,
        key_passphrase: Option<SecretString>,
        timeout_ms: Option<u64>,
        max_command_chars: Option<usize>,
        max_output_chars: Option<usize>,
        senha_sudo: Option<SecretString>,
        senha_su: Option<SecretString>,
        disable_sudo: bool,
    ) -> Self {
        Self {
            nome,
            host,
            porta,
            usuario,
            senha,
            key_path,
            key_passphrase,
            timeout_ms: timeout_ms.unwrap_or(TIMEOUT_PADRAO_MS),
            max_command_chars: max_command_chars.unwrap_or(MAX_COMMAND_CHARS_PADRAO),
            max_output_chars: max_output_chars.unwrap_or(MAX_OUTPUT_CHARS_PADRAO),
            senha_sudo,
            senha_su,
            disable_sudo,
            schema_version: SCHEMA_VERSION_ATUAL,
            adicionado_em: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Retorna true se há senha não vazia.
    #[must_use]
    pub fn tem_senha(&self) -> bool {
        !self.senha.expose_secret().is_empty()
    }

    /// Retorna true se há caminho de chave privada.
    #[must_use]
    pub fn tem_chave(&self) -> bool {
        self.key_path.as_ref().is_some_and(|p| !p.trim().is_empty())
    }

    /// Valida que existe pelo menos um método de autenticação.
    pub fn validar_credenciais(&self) -> Result<(), String> {
        if !self.tem_senha() && !self.tem_chave() {
            return Err(
                "é obrigatório fornecer --password ou --key (auth password ou chave privada)"
                    .to_string(),
            );
        }
        Ok(())
    }

    /// Normaliza schema após deserialização (migração v1 → v2).
    pub fn normalizar_schema(&mut self) {
        if self.schema_version < SCHEMA_VERSION_ATUAL {
            self.schema_version = SCHEMA_VERSION_ATUAL;
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
pub fn parse_limite_chars(s: &str) -> usize {
    let t = s.trim();
    if t.eq_ignore_ascii_case("none") || t == "0" {
        0
    } else {
        t.parse().unwrap_or(MAX_OUTPUT_CHARS_PADRAO)
    }
}

/// Converte limite de config em valor efetivo para truncagem/validação.
///
/// `0` = sem limite (`usize::MAX` para comparação).
#[must_use]
pub fn limite_efetivo(configurado: usize) -> usize {
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
        let out = crate::secrets::serializar_segredo(plain).map_err(serde::ser::Error::custom)?;
        s.serialize_str(&out)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<SecretString, D::Error> {
        let s = String::deserialize(d)?;
        let plain = crate::secrets::deserializar_segredo(&s).map_err(serde::de::Error::custom)?;
        Ok(SecretString::from(plain))
    }
}

mod opcao_secret_string_serde {
    use super::{ExposeSecret, SecretString};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(valor: &Option<SecretString>, s: S) -> Result<S::Ok, S::Error> {
        match valor {
            Some(v) => {
                let out = crate::secrets::serializar_segredo(v.expose_secret())
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
                    crate::secrets::deserializar_segredo(&s).map_err(serde::de::Error::custom)?;
                Ok(Some(SecretString::from(plain)))
            }
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn novo_registro_aplica_defaults() {
        let r = VpsRegistro::novo(
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
        assert_eq!(r.timeout_ms, TIMEOUT_PADRAO_MS);
        assert_eq!(r.max_command_chars, MAX_COMMAND_CHARS_PADRAO);
        assert_eq!(r.max_output_chars, MAX_OUTPUT_CHARS_PADRAO);
        assert_eq!(r.schema_version, SCHEMA_VERSION_ATUAL);
        assert!(!r.adicionado_em.is_empty());
    }

    #[test]
    fn debug_nao_exibe_senha() {
        let r = VpsRegistro::novo(
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
    fn round_trip_toml_preserva_dados() {
        let r = VpsRegistro::novo(
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
        let r2: VpsRegistro = toml::from_str(&toml_str).expect("deserializar");
        assert_eq!(r2.nome, "producao");
        assert_eq!(r2.porta, 2222);
        assert_eq!(r2.senha.expose_secret(), "senha-do-admin-longa");
        assert_eq!(r2.key_path.as_deref(), Some("/home/u/.ssh/id_ed25519"));
        assert_eq!(r2.max_command_chars, 500);
        assert_eq!(r2.max_output_chars, 50_000);
        assert_eq!(
            r2.senha_sudo
                .as_ref()
                .map(|s| s.expose_secret().to_string()),
            Some("sudopass".to_string())
        );
        assert!(r2.senha_su.is_none());
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
        let r: VpsRegistro = toml::from_str(legado).expect("deserializar legado");
        assert_eq!(r.max_output_chars, 4242);
        assert_eq!(r.max_command_chars, MAX_COMMAND_CHARS_PADRAO);
    }

    #[test]
    fn validar_credenciais_exige_password_ou_key() {
        let mut r = VpsRegistro::novo(
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
        assert!(r.validar_credenciais().is_err());
        r.key_path = Some("/tmp/k".into());
        assert!(r.validar_credenciais().is_ok());
    }

    #[test]
    fn parse_limite_none_e_zero() {
        assert_eq!(parse_limite_chars("none"), 0);
        assert_eq!(parse_limite_chars("0"), 0);
        assert_eq!(parse_limite_chars("1000"), 1000);
        assert_eq!(limite_efetivo(0), usize::MAX);
        assert_eq!(limite_efetivo(10), 10);
    }
}
