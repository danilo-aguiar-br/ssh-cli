// SPDX-License-Identifier: MIT OR Apache-2.0
//! At-rest encryption of secrets in `config.toml` (GAP-009 / R-SECRETS-DEFAULT).
//!
//! Primary-key resolution order (32 bytes):
//! 1. `SSH_CLI_SECRETS_KEY` — 64 hex chars
//! 2. `SSH_CLI_SECRETS_KEY_FILE` — file with 64 hex chars
//! 3. OS keyring (`service=ssh-cli`, `user=secrets-primary-key`) if `SSH_CLI_USE_KEYRING=1`
//! 4. XDG `secrets.key` file (next to `config.toml`), auto-created on first write
//!
//! Opt-out (tests/debug): `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` — do not auto-create key;
//! serialization stays plaintext if none of sources 1–3 is defined.
//!
//! With a key: serialization writes `sshcli-enc:v1:<base64(nonce||ciphertext)>`.
//!
//! **Never** log or return the key or plaintext in public errors.

use crate::erros::{SshCliError, SshCliResult};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use zeroize::Zeroize;

/// Prefix for encrypted blobs in TOML.
pub const ENC_PREFIX: &str = "sshcli-enc:v1:";


/// File name of the primary key in the config directory.
pub const KEY_FILE_NAME: &str = "secrets.key";

/// Config directory override (e.g. `--config-dir`) to align `secrets.key`.
static DIR_CONFIG_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Sets the config directory used to resolve `secrets.key` (one-shot; called from `dispatch`).
pub fn set_config_dir(dir: Option<PathBuf>) {
    if let Ok(mut g) = DIR_CONFIG_OVERRIDE.lock() {
        *g = dir;
    }
}

/// Primary-key source (without exposing material).
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
    /// Stable name for JSON/doctor.
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

/// Secrets mode report (no sensitive material).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretsStatus {
    /// Primary-key source.
    pub source: KeySource,
    /// If true, serialization encrypts secrets.
    pub encryption_active: bool,
    /// Path of `secrets.key` (may not exist yet).
    pub key_file_path: PathBuf,
    /// If true, plaintext opt-out is active.
    pub plaintext_opt_out: bool,
}

/// True if `SSH_CLI_ALLOW_PLAINTEXT_SECRETS` requests plaintext.
#[must_use]
pub fn plaintext_allowed() -> bool {
    std::env::var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}


/// Config directory used for `secrets.key` (override > SSH_CLI_HOME > XDG).
pub fn secrets_config_dir() -> SshCliResult<PathBuf> {
    if let Ok(g) = DIR_CONFIG_OVERRIDE.lock() {
        if let Some(ref d) = *g {
            return Ok(d.clone());
        }
    }
    if let Ok(home) = std::env::var("SSH_CLI_HOME") {
        if home.contains("..") {
            return Err(SshCliError::InvalidArgument(
                "SSH_CLI_HOME must not contain '..'".to_string(),
            ));
        }
        return Ok(PathBuf::from(home));
    }
    let dirs = directories::ProjectDirs::from("", "", "ssh-cli").ok_or_else(|| {
        SshCliError::Generic("could not resolve config directory".to_string())
    })?;
    Ok(dirs.config_dir().to_path_buf())
}

/// Canonical path of the local primary-key file.
pub fn secrets_key_path() -> SshCliResult<PathBuf> {
    Ok(secrets_config_dir()?.join(KEY_FILE_NAME))
}

/// Resolves primary key and source (does not auto-create).
///
/// # Errors
/// Returns an error if a configured key source exists but cannot be read or parsed.
pub fn load_primary_key() -> SshCliResult<(Option<[u8; 32]>, KeySource)> {
    if let Ok(hex) = std::env::var("SSH_CLI_SECRETS_KEY") {
        let key = parse_hex_key(hex.trim()).map_err(|e| {
            SshCliError::InvalidArgument(format!("invalid SSH_CLI_SECRETS_KEY: {e}"))
        })?;
        return Ok((Some(key), KeySource::Env));
    }

    if let Ok(path) = std::env::var("SSH_CLI_SECRETS_KEY_FILE") {
        let text = std::fs::read_to_string(&path).map_err(|e| {
            SshCliError::InvalidArgument(format!("failed reading SSH_CLI_SECRETS_KEY_FILE: {e}"))
        })?;
        let key = parse_hex_key(text.trim()).map_err(|e| {
            SshCliError::InvalidArgument(format!("invalid SSH_CLI_SECRETS_KEY_FILE: {e}"))
        })?;
        return Ok((Some(key), KeySource::ConfigFile));
    }

    if std::env::var("SSH_CLI_USE_KEYRING")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        match read_keyring() {
            Ok(Some(key)) => return Ok((Some(key), KeySource::Keyring)),
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(err = %e, "keyring unavailable; trying secrets.key");
            }
        }
    }

    let path = secrets_key_path()?;
    if path.is_file() {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| SshCliError::Generic(format!("failed reading {}: {e}", path.display())))?;
        let key = parse_hex_key(text.trim())
            .map_err(|e| SshCliError::InvalidArgument(format!("invalid secrets.key: {e}")))?;
        return Ok((Some(key), KeySource::XdgFile));
    }

    Ok((None, KeySource::Absent))
}

/// Ensures a key for **write**: loads existing or auto-creates `secrets.key`
/// (unless plaintext opt-out).
///
/// # Errors
/// Returns an error if auto-creating `secrets.key` fails when encryption is required.
pub fn ensure_key_for_write() -> SshCliResult<(Option<[u8; 32]>, KeySource)> {
    let (existing, source) = load_primary_key()?;
    if existing.is_some() {
        return Ok((existing, source));
    }
    if plaintext_allowed() {
        return Ok((None, KeySource::Absent));
    }
    let path = secrets_key_path()?;
    let hex = generate_hex_key()?;
    write_key_file(&path, &hex, false)?;
    let key = parse_hex_key(&hex)
        .map_err(|e| SshCliError::Generic(format!("invalid generated key: {e}")))?;
    Ok((Some(key), KeySource::XdgFile))
}

/// Current status (without loading material into logs).
pub fn secrets_status() -> SshCliResult<SecretsStatus> {
    let key_file_path = secrets_key_path()?;
    let (key, source) = load_primary_key()?;
    let encryption_active = key.is_some();
    if let Some(mut k) = key {
        k.zeroize();
    }
    Ok(SecretsStatus {
        source,
        encryption_active,
        key_file_path,
        plaintext_opt_out: plaintext_allowed(),
    })
}

/// True if the string is already an encrypted blob.
#[must_use]
pub fn is_encrypted_blob(value: &str) -> bool {
    value.starts_with(ENC_PREFIX)
}

/// Serializes a secret for TOML: encrypts if a key exists (or is auto-created); otherwise plaintext.
///
/// Empty secret never becomes a blob `sshcli-enc` (GAP-SSH-EXP-001): export redacted zera
/// passwords and must store readable `""`, not ciphertext of empty string (which fools import
/// on another machine without the primary-key and fakes "secret present").
///
/// # Errors
/// Returns an error if key resolution, RNG, or AEAD encryption fails.
pub fn serialize_secret(plaintext: &str) -> SshCliResult<String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    let (key, _) = ensure_key_for_write()?;
    match key {
        None => Ok(plaintext.to_string()),
        Some(mut key) => {
            let out = encrypt_secret(&key, plaintext)?;
            key.zeroize();
            Ok(out)
        }
    }
}

/// Deserializes from TOML: decrypts `sshcli-enc:v1:` blobs; otherwise returns as-is.
pub fn deserialize_secret(stored: &str) -> SshCliResult<String> {
    if !is_encrypted_blob(stored) {
        return Ok(stored.to_string());
    }
    let (key, _) = load_primary_key()?;
    let mut key = key.ok_or_else(|| {
        SshCliError::InvalidArgument(
            "config contains encrypted secrets; set SSH_CLI_SECRETS_KEY, SSH_CLI_SECRETS_KEY_FILE, SSH_CLI_USE_KEYRING=1, or secrets.key (ssh-cli secrets init)"
                .to_string(),
        )
    })?;
    let plain = decrypt_secret(&key, stored)?;
    key.zeroize();
    Ok(plain)
}

/// Generates 32 random bytes as 64 hex chars.
pub fn generate_hex_key() -> SshCliResult<String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| SshCliError::Generic(format!("RNG failed: {e}")))?;
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    bytes.zeroize();
    Ok(hex)
}

/// Writes hex key to file with 0o600 (when supported).
///
/// # Errors
/// Returns an error if the key is invalid, the file exists without force, or I/O fails.
pub fn write_key_file(path: &Path, hex64: &str, force: bool) -> SshCliResult<()> {
    let _ = parse_hex_key(hex64)
        .map_err(|e| SshCliError::InvalidArgument(format!("invalid key: {e}")))?;
    if path.exists() && !force {
        return Err(SshCliError::InvalidArgument(format!(
            "{} already exists; use --force to overwrite",
            path.display()
        )));
    }
    // GAP-AUD-SEC-001: backup previous key before force-overwrite.
    if path.exists() && force {
        let bak = path.with_file_name(format!(
            "{}.bak",
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(KEY_FILE_NAME)
        ));
        if let Err(e) = std::fs::copy(path, &bak) {
            tracing::warn!(
                err = %e,
                path = %bak.display(),
                "failed to backup secrets key before --force"
            );
        }
    }
    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }
    let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent_dir)
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

/// Initializes primary-key in XDG file or keyring. **Never** prints the key.
///
/// # Errors
/// Returns an error if the key already exists without `--force`, RNG fails, or keyring/file I/O fails.
pub fn init_primary_key(use_keyring: bool, force: bool) -> SshCliResult<SecretsStatus> {
    let hex = generate_hex_key()?;
    if use_keyring {
        if !force {
            match read_keyring() {
                Ok(Some(_)) => {
                    return Err(SshCliError::InvalidArgument(
                        "keyring already has a primary-key; use --force".to_string(),
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

/// Stores primary-key (hex) in the OS keyring. Does not print the key.
pub fn write_key_to_keyring(hex64: &str) -> SshCliResult<()> {
    let _ = parse_hex_key(hex64)
        .map_err(|e| SshCliError::InvalidArgument(format!("invalid key: {e}")))?;
    let entry = keyring::Entry::new("ssh-cli", "secrets-primary-key")
        .map_err(|e| SshCliError::Generic(format!("keyring Entry::new failed: {e}")))?;
    entry
        .set_password(hex64.trim())
        .map_err(|e| SshCliError::Generic(format!("keyring set failed: {e}")))?;
    Ok(())
}

fn parse_hex_key(hex: &str) -> Result<[u8; 32], String> {
    let h = hex.trim();
    if h.len() != 64 {
        return Err("expected 64 hex characters (32 bytes)".to_string());
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        let byte =
            u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).map_err(|_| "invalid hex".to_string())?;
        out[i] = byte;
    }
    Ok(out)
}

fn encrypt_secret(key: &[u8; 32], plaintext: &str) -> SshCliResult<String> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| SshCliError::Generic("invalid AEAD key".to_string()))?;
    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| SshCliError::Generic(format!("RNG failed: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| SshCliError::Generic("failed to encrypt secret".to_string()))?;
    let mut packed = Vec::with_capacity(12 + ciphertext.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);
    Ok(format!(
        "{ENC_PREFIX}{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &packed)
    ))
}

fn decrypt_secret(key: &[u8; 32], blob: &str) -> SshCliResult<String> {
    let b64 = blob
        .strip_prefix(ENC_PREFIX)
        .ok_or_else(|| SshCliError::Generic("malformed encrypted blob".to_string()))?;
    let packed = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
        .map_err(|_| SshCliError::Generic("invalid encrypted blob base64".to_string()))?;
    if packed.len() < 12 + 16 {
        return Err(SshCliError::Generic(
            "encrypted blob too short".to_string(),
        ));
    }
    let (nonce_bytes, ct) = packed.split_at(12);
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| SshCliError::Generic("invalid AEAD key".to_string()))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher.decrypt(nonce, ct).map_err(|_| {
        SshCliError::Generic("failed to decrypt secret (wrong key?)".to_string())
    })?;
    String::from_utf8(plain)
        .map_err(|_| SshCliError::Generic("decrypted secret is not valid UTF-8".to_string()))
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

    fn clear_key_env() {
        std::env::remove_var("SSH_CLI_SECRETS_KEY");
        std::env::remove_var("SSH_CLI_SECRETS_KEY_FILE");
        std::env::remove_var("SSH_CLI_USE_KEYRING");
        std::env::remove_var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS");
        std::env::remove_var("SSH_CLI_HOME");
        set_config_dir(None);
    }

    /// Isolates tests from real XDG (never pollute user config).
    fn sandbox() -> TempDir {
        clear_key_env();
        let tmp = TempDir::new().unwrap();
        set_config_dir(Some(tmp.path().to_path_buf()));
        tmp
    }

    #[test]
    #[serial]
    fn roundtrip_with_env_key() {
        let _tmp = sandbox();
        let hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        std::env::set_var("SSH_CLI_SECRETS_KEY", hex);
        let plain = "fake-test-password-not-real";
        let enc = serialize_secret(plain).unwrap();
        assert!(is_encrypted_blob(&enc));
        assert!(!enc.contains(plain));
        let back = deserialize_secret(&enc).unwrap();
        assert_eq!(back, plain);
        clear_key_env();
    }

    #[test]
    #[serial]
    fn opt_out_keeps_plaintext() {
        let _tmp = sandbox();
        std::env::set_var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS", "1");
        let plain = "fake-plaintext-only-for-unit-test";
        let out = serialize_secret(plain).unwrap();
        assert_eq!(out, plain);
        assert!(!is_encrypted_blob(&out));
        clear_key_env();
    }

    #[test]
    #[serial]
    fn default_auto_creates_secrets_key() {
        let tmp = sandbox();
        let plain = "fake-auto-enc-password";
        let enc = serialize_secret(plain).unwrap();
        assert!(is_encrypted_blob(&enc));
        assert!(!enc.contains(plain));
        assert!(tmp.path().join(KEY_FILE_NAME).is_file());
        let back = deserialize_secret(&enc).unwrap();
        assert_eq!(back, plain);
        clear_key_env();
    }

    #[test]
    #[serial]
    fn blob_without_key_fails() {
        let tmp = sandbox();
        let hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        std::env::set_var("SSH_CLI_SECRETS_KEY", hex);
        let enc = serialize_secret("fake-secret").unwrap();
        // Remove env e qualquer secrets.key do sandbox
        clear_key_env();
        set_config_dir(Some(tmp.path().to_path_buf()));
        let _ = std::fs::remove_file(tmp.path().join(KEY_FILE_NAME));
        std::env::set_var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS", "1");
        let err = deserialize_secret(&enc).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("encrypted") || msg.contains("SSH_CLI") || msg.contains("secrets"),
            "msg={msg}"
        );
        clear_key_env();
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
        assert!(!is_encrypted_blob(&out));
        clear_key_env();
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
    fn init_creates_file() {
        clear_key_env();
        let tmp = TempDir::new().unwrap();
        set_config_dir(Some(tmp.path().to_path_buf()));
        let st = init_primary_key(false, false).unwrap();
        assert!(st.encryption_active);
        assert_eq!(st.source, KeySource::XdgFile);
        assert!(st.key_file_path.is_file());
        clear_key_env();
    }
}
