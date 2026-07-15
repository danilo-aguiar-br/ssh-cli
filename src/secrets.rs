// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cifragem at-rest de segredos no `config.toml` (GAP-009 / R-SECRETS-DEFAULT).
//!
//! Ordem de resolução da chave mestra (32 bytes):
//! 1. `SSH_CLI_SECRETS_KEY` — 64 hex chars
//! 2. `SSH_CLI_SECRETS_KEY_FILE` — arquivo com 64 hex chars
//! 3. OS keyring (`service=ssh-cli`, `user=secrets-primary-key`) se `SSH_CLI_USE_KEYRING=1`
//! 4. Arquivo XDG `secrets.key` (ao lado do `config.toml`), criado automaticamente na 1ª gravação
//!
//! Opt-out (tests/debug): `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` — não auto-cria chave;
//! serialização permanece em texto se nenhuma fonte 1–3 estiver definida.
//!
//! Com chave: serialização grava `sshcli-enc:v1:<base64(nonce||ciphertext)>`.
//!
//! **Nunca** logar ou retornar a chave ou o plaintext em erros públicos.

use crate::erros::{SshCliError, SshCliResult};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use zeroize::Zeroize;

/// Prefixo de blobs cifrados no TOML.
pub const PREFIXO_ENC: &str = "sshcli-enc:v1:";

/// Nome do arquivo de chave mestra no diretório de config.
pub const KEY_FILE_NAME: &str = "secrets.key";

/// Override de diretório de config (ex.: `--config-dir`), para alinhar `secrets.key`.
static DIR_CONFIG_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Define o diretório de config para resolver `secrets.key` (one-shot; chamado no `dispatch`).
pub fn set_config_dir(dir: Option<PathBuf>) {
    if let Ok(mut g) = DIR_CONFIG_OVERRIDE.lock() {
        *g = dir;
    }
}

/// Fonte da chave mestra (sem expor material).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// No key source available (plaintext at-rest with opt-out or before first write).
    Absent,
    /// Environment variable `SSH_CLI_SECRETS_KEY`.
    Env,
    /// File from `SSH_CLI_SECRETS_KEY_FILE`.
    ConfigFile,
    /// OS keyring.
    Keyring,
    /// XDG / config-dir `secrets.key` file.
    XdgFile,
}

impl KeySource {
    /// Nome estável para JSON/doctor.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "none",
            Self::Env => "env",
            Self::ConfigFile => "file",
            Self::Keyring => "keyring",
            Self::XdgFile => "xdg_file",
        }
    }
}

/// Relatório de modo de segredos (sem material sensível).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretsStatus {
    /// Fonte da chave mestra.
    pub fonte: KeySource,
    /// Se true, serialização cifra secrets.
    pub cifragem_ativa: bool,
    /// Path do `secrets.key` (pode não existir ainda).
    pub key_file_path: PathBuf,
    /// Se true, opt-out de plaintext está ativo.
    pub plaintext_opt_out: bool,
}

/// True se `SSH_CLI_ALLOW_PLAINTEXT_SECRETS` pede plaintext.
#[must_use]
pub fn plaintext_permitido() -> bool {
    std::env::var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Diretório de config usado para `secrets.key` (override > SSH_CLI_HOME > XDG).
pub fn secrets_config_dir() -> SshCliResult<PathBuf> {
    if let Ok(g) = DIR_CONFIG_OVERRIDE.lock() {
        if let Some(ref d) = *g {
            return Ok(d.clone());
        }
    }
    if let Ok(home) = std::env::var("SSH_CLI_HOME") {
        if home.contains("..") {
            return Err(SshCliError::InvalidArgument(
                "SSH_CLI_HOME não pode conter '..'".to_string(),
            ));
        }
        return Ok(PathBuf::from(home));
    }
    let dirs = directories::ProjectDirs::from("", "", "ssh-cli").ok_or_else(|| {
        SshCliError::Generic("não foi possível resolver diretório de config".to_string())
    })?;
    Ok(dirs.config_dir().to_path_buf())
}

/// Path canônico do arquivo de chave mestra local.
pub fn secrets_key_path() -> SshCliResult<PathBuf> {
    Ok(secrets_config_dir()?.join(KEY_FILE_NAME))
}

/// Resolve chave mestra e origem (não auto-cria).
pub fn load_primary_key() -> SshCliResult<(Option<[u8; 32]>, KeySource)> {
    if let Ok(hex) = std::env::var("SSH_CLI_SECRETS_KEY") {
        let chave = parse_hex_key(hex.trim()).map_err(|e| {
            SshCliError::InvalidArgument(format!("SSH_CLI_SECRETS_KEY inválida: {e}"))
        })?;
        return Ok((Some(chave), KeySource::Env));
    }

    if let Ok(path) = std::env::var("SSH_CLI_SECRETS_KEY_FILE") {
        let texto = std::fs::read_to_string(&path).map_err(|e| {
            SshCliError::InvalidArgument(format!("falha lendo SSH_CLI_SECRETS_KEY_FILE: {e}"))
        })?;
        let chave = parse_hex_key(texto.trim()).map_err(|e| {
            SshCliError::InvalidArgument(format!("SSH_CLI_SECRETS_KEY_FILE inválida: {e}"))
        })?;
        return Ok((Some(chave), KeySource::ConfigFile));
    }

    if std::env::var("SSH_CLI_USE_KEYRING")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        match read_keyring() {
            Ok(Some(chave)) => return Ok((Some(chave), KeySource::Keyring)),
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(erro = %e, "keyring indisponível; tentando secrets.key");
            }
        }
    }

    let path = secrets_key_path()?;
    if path.is_file() {
        let texto = std::fs::read_to_string(&path)
            .map_err(|e| SshCliError::Generic(format!("falha lendo {}: {e}", path.display())))?;
        let chave = parse_hex_key(texto.trim())
            .map_err(|e| SshCliError::InvalidArgument(format!("secrets.key inválida: {e}")))?;
        return Ok((Some(chave), KeySource::XdgFile));
    }

    Ok((None, KeySource::Absent))
}

/// Garante chave para **escrita**: carrega existente ou auto-cria `secrets.key`
/// (salvo opt-out plaintext).
pub fn ensure_key_for_write() -> SshCliResult<(Option<[u8; 32]>, KeySource)> {
    let (existente, fonte) = load_primary_key()?;
    if existente.is_some() {
        return Ok((existente, fonte));
    }
    if plaintext_permitido() {
        return Ok((None, KeySource::Absent));
    }
    let path = secrets_key_path()?;
    let hex = generate_hex_key()?;
    write_key_file(&path, &hex, false)?;
    let chave = parse_hex_key(&hex)
        .map_err(|e| SshCliError::Generic(format!("chave gerada inválida: {e}")))?;
    Ok((Some(chave), KeySource::XdgFile))
}

/// Status atual (sem carregar material em logs).
pub fn secrets_status() -> SshCliResult<SecretsStatus> {
    let key_file_path = secrets_key_path()?;
    let (chave, fonte) = load_primary_key()?;
    let cifragem_ativa = chave.is_some();
    if let Some(mut k) = chave {
        k.zeroize();
    }
    Ok(SecretsStatus {
        fonte,
        cifragem_ativa,
        key_file_path,
        plaintext_opt_out: plaintext_permitido(),
    })
}

/// True se a string já é blob cifrado.
#[must_use]
pub fn eh_blob_cifrado(valor: &str) -> bool {
    valor.starts_with(PREFIXO_ENC)
}

/// Serializa um segredo para TOML: cifra se houver (ou auto-criar) chave; senão plaintext.
///
/// Segredo **vazio** nunca vira blob `sshcli-enc` (GAP-SSH-EXP-001): export redacted zera
/// senhas e deve gravar `""` legível, não ciphertext de string vazia (que engana import
/// em outra máquina sem a primary-key e finge "secret present").
pub fn serialize_secret(plaintext: &str) -> SshCliResult<String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    let (chave, _) = ensure_key_for_write()?;
    match chave {
        None => Ok(plaintext.to_string()),
        Some(mut key) => {
            let out = cifrar(&key, plaintext)?;
            key.zeroize();
            Ok(out)
        }
    }
}

/// Desserializa de TOML: decifra blobs `sshcli-enc:v1:`; senão devolve como está.
pub fn deserialize_secret(armazenado: &str) -> SshCliResult<String> {
    if !eh_blob_cifrado(armazenado) {
        return Ok(armazenado.to_string());
    }
    let (chave, _) = load_primary_key()?;
    let mut key = chave.ok_or_else(|| {
        SshCliError::InvalidArgument(
            "config contém secrets cifrados; defina SSH_CLI_SECRETS_KEY, SSH_CLI_SECRETS_KEY_FILE, SSH_CLI_USE_KEYRING=1 ou secrets.key (ssh-cli secrets init)"
                .to_string(),
        )
    })?;
    let plain = decifrar(&key, armazenado)?;
    key.zeroize();
    Ok(plain)
}

/// Gera 32 bytes aleatórios como hex 64.
pub fn generate_hex_key() -> SshCliResult<String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| SshCliError::Generic(format!("RNG falhou: {e}")))?;
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    bytes.zeroize();
    Ok(hex)
}

/// Grava chave hex em arquivo com 0o600 (quando suportado).
pub fn write_key_file(path: &Path, hex64: &str, force: bool) -> SshCliResult<()> {
    let _ = parse_hex_key(hex64)
        .map_err(|e| SshCliError::InvalidArgument(format!("chave inválida: {e}")))?;
    if path.exists() && !force {
        return Err(SshCliError::InvalidArgument(format!(
            "{} já existe; use --force para sobrescrever",
            path.display()
        )));
    }
    if let Some(pai) = path.parent() {
        std::fs::create_dir_all(pai)?;
    }
    let pai = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(pai)
        .map_err(|e| SshCliError::Generic(format!("tempfile secrets.key: {e}")))?;
    use std::io::Write;
    tmp.write_all(hex64.trim().as_bytes())
        .map_err(|e| SshCliError::Generic(format!("write secrets.key: {e}")))?;
    tmp.write_all(b"\n")
        .map_err(|e| SshCliError::Generic(format!("write secrets.key: {e}")))?;
    tmp.as_file()
        .sync_all()
        .map_err(|e| SshCliError::Generic(format!("fsync secrets.key: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        tmp.as_file()
            .set_permissions(perms)
            .map_err(|e| SshCliError::Generic(format!("chmod secrets.key: {e}")))?;
    }
    tmp.persist(path)
        .map_err(|e| SshCliError::Generic(format!("persist secrets.key: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// Inicializa primary-key em arquivo XDG ou keyring. **Nunca** imprime a chave.
pub fn init_primary_key(use_keyring: bool, force: bool) -> SshCliResult<SecretsStatus> {
    let hex = generate_hex_key()?;
    if use_keyring {
        if !force {
            match read_keyring() {
                Ok(Some(_)) => {
                    return Err(SshCliError::InvalidArgument(
                        "keyring já contém primary-key; use --force".to_string(),
                    ));
                }
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        }
        write_key_to_keyring(&hex)?;
        drop(hex);
        return secrets_status();
    }
    let path = secrets_key_path()?;
    write_key_file(&path, &hex, force)?;
    drop(hex);
    secrets_status()
}

/// Grava chave mestra (hex) no OS keyring. Não imprime a chave.
pub fn write_key_to_keyring(hex64: &str) -> SshCliResult<()> {
    let _ = parse_hex_key(hex64)
        .map_err(|e| SshCliError::InvalidArgument(format!("chave inválida: {e}")))?;
    let entry = keyring::Entry::new("ssh-cli", "secrets-primary-key")
        .map_err(|e| SshCliError::Generic(format!("keyring Entry::new falhou: {e}")))?;
    entry
        .set_password(hex64.trim())
        .map_err(|e| SshCliError::Generic(format!("keyring set falhou: {e}")))?;
    Ok(())
}

fn parse_hex_key(hex: &str) -> Result<[u8; 32], String> {
    let h = hex.trim();
    if h.len() != 64 {
        return Err("espere 64 caracteres hex (32 bytes)".to_string());
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        let byte =
            u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).map_err(|_| "hex inválido".to_string())?;
        out[i] = byte;
    }
    Ok(out)
}

fn cifrar(key: &[u8; 32], plaintext: &str) -> SshCliResult<String> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| SshCliError::Generic("chave AEAD inválida".to_string()))?;
    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| SshCliError::Generic(format!("RNG falhou: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| SshCliError::Generic("falha ao cifrar segredo".to_string()))?;
    let mut packed = Vec::with_capacity(12 + ciphertext.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);
    Ok(format!(
        "{PREFIXO_ENC}{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &packed)
    ))
}

fn decifrar(key: &[u8; 32], blob: &str) -> SshCliResult<String> {
    let b64 = blob
        .strip_prefix(PREFIXO_ENC)
        .ok_or_else(|| SshCliError::Generic("blob cifrado malformado".to_string()))?;
    let packed = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
        .map_err(|_| SshCliError::Generic("blob cifrado base64 inválido".to_string()))?;
    if packed.len() < 12 + 16 {
        return Err(SshCliError::Generic(
            "blob cifrado demasiado curto".to_string(),
        ));
    }
    let (nonce_bytes, ct) = packed.split_at(12);
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| SshCliError::Generic("chave AEAD inválida".to_string()))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher.decrypt(nonce, ct).map_err(|_| {
        SshCliError::Generic("falha ao decifrar segredo (chave errada?)".to_string())
    })?;
    String::from_utf8(plain)
        .map_err(|_| SshCliError::Generic("segredo decifrado não é UTF-8".to_string()))
}

fn read_keyring() -> SshCliResult<Option<[u8; 32]>> {
    // Prefer inclusive primary-key id; fall back to legacy master-key user for migration.
    for user in ["secrets-primary-key", "secrets-master-key"] {
        let entry = match keyring::Entry::new("ssh-cli", user) {
            Ok(e) => e,
            Err(e) => {
                if user == "secrets-master-key" {
                    return Err(SshCliError::Generic(format!("keyring Entry::new failed: {e}")));
                }
                continue;
            }
        };
        match entry.get_password() {
            Ok(s) => {
                let key = parse_hex_key(&s).map_err(|e| {
                    SshCliError::InvalidArgument(format!("invalid keyring primary-key: {e}"))
                })?;
                return Ok(Some(key));
            }
            Err(keyring::Error::NoEntry) => continue,
            Err(e) => {
                if user == "secrets-master-key" {
                    return Err(SshCliError::Generic(format!("keyring get failed: {e}")));
                }
                continue;
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    fn limpar_env_chave() {
        std::env::remove_var("SSH_CLI_SECRETS_KEY");
        std::env::remove_var("SSH_CLI_SECRETS_KEY_FILE");
        std::env::remove_var("SSH_CLI_USE_KEYRING");
        std::env::remove_var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS");
        std::env::remove_var("SSH_CLI_HOME");
        set_config_dir(None);
    }

    /// Isola tests do XDG real (nunca poluir config do usuário).
    fn sandbox() -> TempDir {
        limpar_env_chave();
        let tmp = TempDir::new().unwrap();
        set_config_dir(Some(tmp.path().to_path_buf()));
        tmp
    }

    #[test]
    #[serial]
    fn roundtrip_com_chave_env() {
        let _tmp = sandbox();
        let hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        std::env::set_var("SSH_CLI_SECRETS_KEY", hex);
        let plain = "fake-test-password-not-real";
        let enc = serialize_secret(plain).unwrap();
        assert!(eh_blob_cifrado(&enc));
        assert!(!enc.contains(plain));
        let back = deserialize_secret(&enc).unwrap();
        assert_eq!(back, plain);
        limpar_env_chave();
    }

    #[test]
    #[serial]
    fn opt_out_mantem_plaintext() {
        let _tmp = sandbox();
        std::env::set_var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS", "1");
        let plain = "fake-plaintext-only-for-unit-test";
        let out = serialize_secret(plain).unwrap();
        assert_eq!(out, plain);
        assert!(!eh_blob_cifrado(&out));
        limpar_env_chave();
    }

    #[test]
    #[serial]
    fn default_auto_cria_secrets_key() {
        let tmp = sandbox();
        let plain = "fake-auto-enc-password";
        let enc = serialize_secret(plain).unwrap();
        assert!(eh_blob_cifrado(&enc));
        assert!(!enc.contains(plain));
        assert!(tmp.path().join(KEY_FILE_NAME).is_file());
        let back = deserialize_secret(&enc).unwrap();
        assert_eq!(back, plain);
        limpar_env_chave();
    }

    #[test]
    #[serial]
    fn blob_sem_chave_falha() {
        let tmp = sandbox();
        let hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        std::env::set_var("SSH_CLI_SECRETS_KEY", hex);
        let enc = serialize_secret("fake-secret").unwrap();
        // Remove env e qualquer secrets.key do sandbox
        limpar_env_chave();
        set_config_dir(Some(tmp.path().to_path_buf()));
        let _ = std::fs::remove_file(tmp.path().join(KEY_FILE_NAME));
        std::env::set_var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS", "1");
        let err = deserialize_secret(&enc).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("cifrados") || msg.contains("SSH_CLI") || msg.contains("secrets"),
            "msg={msg}"
        );
        limpar_env_chave();
    }

    #[test]
    #[serial]
    fn empty_secret_never_encrypted_blob() {
        // GAP-SSH-EXP-001
        let _tmp = sandbox();
        let hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        std::env::set_var("SSH_CLI_SECRETS_KEY", hex);
        let out = serialize_secret("").unwrap();
        assert_eq!(out, "");
        assert!(!eh_blob_cifrado(&out));
        limpar_env_chave();
    }

    #[test]
    fn parse_hex_tamanho() {
        assert!(parse_hex_key("aa").is_err());
        assert!(parse_hex_key(
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
        )
        .is_ok());
    }

    #[test]
    #[serial]
    fn init_cria_arquivo() {
        limpar_env_chave();
        let tmp = TempDir::new().unwrap();
        set_config_dir(Some(tmp.path().to_path_buf()));
        let st = init_primary_key(false, false).unwrap();
        assert!(st.cifragem_ativa);
        assert_eq!(st.fonte, KeySource::XdgFile);
        assert!(st.key_file_path.is_file());
        limpar_env_chave();
    }
}
