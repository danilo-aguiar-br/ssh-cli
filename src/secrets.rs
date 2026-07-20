// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! At-rest encryption of secrets in `config.toml` (GAP-009 / R-SECRETS-DEFAULT).
//!
//! Primary-key resolution order (32 bytes), 0.5.1:
//! 1. CLI flags (`--secrets-key-file`, `--use-keyring`, `--allow-plaintext-secrets`)
//! 2. OS keyring when enabled (`service=ssh-cli`, `user=secrets-primary-key`; legacy read alias)
//! 3. XDG `secrets.key` file (next to `config.toml`), auto-created on first write
//!
//! **Env-as-store is forbidden (G-ERR-13 / G-UNSAFE):** if `SSH_CLI_SECRETS_KEY` or
//! `SSH_CLI_SECRETS_KEY_FILE` is present, load **fails closed** with a clear error
//! pointing to XDG `secrets.key` or `--secrets-key-file`.
//!
//! Plaintext at-rest opt-out: **only** CLI `--allow-plaintext-secrets` (no env store).
//!
//! With a key: serialization writes `sshcli-enc:v1:<base64(nonce||ciphertext)>`.
//!
//! **Never** log or return the key or plaintext in public errors.

use crate::constants::{
    AEAD_NONCE_LEN_BYTES, AEAD_TAG_LEN_BYTES, APP_NAME,
    ENV_SECRETS_KEY, ENV_SECRETS_KEY_FILE, KEYRING_SERVICE, KEYRING_USER_LEGACY,
    KEYRING_USER_PRIMARY, PRIMARY_KEY_HEX_LEN, PRIMARY_KEY_LEN_BYTES,
    SECRETS_KEY_FILE_NAME,
};
use crate::errors::{SshCliError, SshCliResult};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use zeroize::Zeroize;

/// Prefix for encrypted blobs in TOML.
pub const ENC_PREFIX: &str = "sshcli-enc:v1:";

/// File name of the primary key in the config directory (XDG sibling of `config.toml`).
pub const KEY_FILE_NAME: &str = SECRETS_KEY_FILE_NAME;

// Compile-time invariants (const/static rules).
const _: () = assert!(!ENC_PREFIX.is_empty());
const _: () = assert!(!KEY_FILE_NAME.is_empty());
const _: () = assert!(PRIMARY_KEY_LEN_BYTES == 32);

/// Locks a process-global `Mutex`, recovering from poison explicitly.
///
/// Poison means a previous holder panicked; the data is still usable for this
/// one-shot CLI, so we take `into_inner()` rather than silently skipping updates.
/// Recovery is **logged** (Rules Rust: never silence `PoisonError` without log).
///
/// Critical sections using this helper must stay short and **never** hold the
/// guard across `.await` or blocking I/O (clone/copy under lock, then release).
fn lock_global<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| {
        tracing::warn!(
            "secrets process-global mutex was poisoned; recovering via into_inner (one-shot CLI)"
        );
        poisoned.into_inner()
    })
}

/// Config directory override (e.g. `--config-dir`) to align `secrets.key`.
///
/// Concurrent access: `std::sync::Mutex` (const ctor) — single composite state
/// (`Option<PathBuf>`); not split into uncoordinated atomics. Poison recovered
/// via [`lock_global`]. Never held across await.
static DIR_CONFIG_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// CLI runtime overrides (flags). Env remains as deprecated fallback.
#[derive(Debug, Default, Clone)]
struct RuntimeSecretsFlags {
    allow_plaintext: bool,
    secrets_key_file: Option<PathBuf>,
    use_keyring: bool,
}

/// Process-wide secrets CLI flags (set once after parse).
///
/// Single `Mutex` keeps the three fields consistent (Rules: do not protect a
/// multi-field invariant with independent atomics). See [`lock_global`].
static RUNTIME_FLAGS: Mutex<RuntimeSecretsFlags> = Mutex::new(RuntimeSecretsFlags {
    allow_plaintext: false,
    secrets_key_file: None,
    use_keyring: false,
});

/// Set when `secrets.key` is auto-created during this process (GAP-AUD-007).
///
/// Concurrent access: independent status bit; `Ordering::Relaxed` (no dependent
/// data fence — isolated flag, not paired with other memory).
static AUTO_KEY_CREATED: AtomicBool = AtomicBool::new(false);

/// Sets the config directory used to resolve `secrets.key` (one-shot; called from `dispatch`).
pub fn set_config_dir(dir: Option<PathBuf>) {
    *lock_global(&DIR_CONFIG_OVERRIDE) = dir;
}

/// Applies one-shot CLI flags for secrets resolution (GAP-AUD-006).
pub fn set_runtime_flags(
    allow_plaintext: bool,
    secrets_key_file: Option<PathBuf>,
    use_keyring: bool,
) {
    {
        let mut g = lock_global(&RUNTIME_FLAGS);
        g.allow_plaintext = allow_plaintext;
        g.secrets_key_file = secrets_key_file;
        g.use_keyring = use_keyring;
    }
    AUTO_KEY_CREATED.store(false, Ordering::Relaxed);
}

/// Returns true once if a key was auto-created since the last flag reset (consume).
#[must_use]
pub fn take_auto_key_created() -> bool {
    // RMW on an independent flag — Relaxed is enough (no data publish).
    AUTO_KEY_CREATED.swap(false, Ordering::Relaxed)
}

/// Returns true if a key was auto-created (non-consuming).
#[must_use]
pub fn auto_key_created() -> bool {
    AUTO_KEY_CREATED.load(Ordering::Relaxed)
}

/// Primary-key source (without exposing material).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// No key source available (plaintext at-rest with opt-out or before first write).
    Absent,
    /// Reserved: env key material is **rejected** (fail-closed); never a success source.
    Env,
    /// File from CLI `--secrets-key-file`.
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

/// True if plaintext opt-out is active (CLI flag only — G-ERR-13, no env store).
#[must_use]
pub fn plaintext_allowed() -> bool {
    lock_global(&RUNTIME_FLAGS).allow_plaintext
}


/// Config directory used for `secrets.key` (CLI/test override > XDG).
///
/// # Errors
/// [`SshCliError::XdgDirectory`] when XDG cannot be resolved and no override is set.
pub fn secrets_config_dir() -> SshCliResult<PathBuf> {
    if let Some(d) = lock_global(&DIR_CONFIG_OVERRIDE).clone() {
        return Ok(d);
    }
    crate::paths::xdg_config_dir()
}

/// Canonical path of the local primary-key file.
pub fn secrets_key_path() -> SshCliResult<PathBuf> {
    Ok(secrets_config_dir()?.join(KEY_FILE_NAME))
}

/// Resolves primary key and source (does not auto-create).
///
/// # Errors
/// Returns an error if a configured key source exists but cannot be read or parsed.
pub fn load_primary_key() -> SshCliResult<(Option<[u8; PRIMARY_KEY_LEN_BYTES]>, KeySource)> {
    // CLI flag: --secrets-key-file
    let secrets_key_file = lock_global(&RUNTIME_FLAGS).secrets_key_file.clone();
    if let Some(path) = secrets_key_file {
        let mut text = crate::paths::read_text_capped(
            &path,
            crate::paths::MAX_SECRETS_KEY_FILE_BYTES,
        )
        .map_err(|e| {
            SshCliError::InvalidArgument(format!(
                "failed reading --secrets-key-file {}: {e}",
                path.display()
            ))
        })?;
        let key = parse_hex_key(text.trim()).map_err(|e| {
            SshCliError::InvalidArgument(format!("invalid --secrets-key-file: {e}"))
        });
        text.zeroize();
        return Ok((Some(key?), KeySource::ConfigFile));
    }

    // G-ERR-13: env-as-store for key material is forbidden (fail closed).
    if std::env::var_os(ENV_SECRETS_KEY).is_some()
        || std::env::var_os(ENV_SECRETS_KEY_FILE).is_some()
    {
        return Err(SshCliError::InvalidArgument(format!(
            "{ENV_SECRETS_KEY} / {ENV_SECRETS_KEY_FILE} are not supported; use XDG `{KEY_FILE_NAME}` \
             (`{APP_NAME} secrets init`) or --secrets-key-file"
        )));
    }

    let use_keyring_flag = lock_global(&RUNTIME_FLAGS).use_keyring;
    if use_keyring_flag {
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
        let mut text = crate::paths::read_text_capped(
            &path,
            crate::paths::MAX_SECRETS_KEY_FILE_BYTES,
        )
        .map_err(|e| {
            SshCliError::Config(format!("failed reading {}: {e}", path.display()))
        })?;
        let key = parse_hex_key(text.trim())
            .map_err(|e| SshCliError::InvalidArgument(format!("invalid {KEY_FILE_NAME}: {e}")));
        text.zeroize();
        return Ok((Some(key?), KeySource::XdgFile));
    }

    Ok((None, KeySource::Absent))
}

/// Ensures a key for **write**: loads existing or auto-creates `secrets.key`
/// (unless plaintext opt-out).
///
/// # Errors
/// Returns an error if auto-creating `secrets.key` fails when encryption is required.
pub fn ensure_key_for_write() -> SshCliResult<(Option<[u8; PRIMARY_KEY_LEN_BYTES]>, KeySource)> {
    let (existing, source) = load_primary_key()?;
    if existing.is_some() {
        return Ok((existing, source));
    }
    if plaintext_allowed() {
        return Ok((None, KeySource::Absent));
    }
    let path = secrets_key_path()?;
    let mut hex = generate_hex_key()?;
    write_key_file(&path, &hex, false)?;
    AUTO_KEY_CREATED.store(true, Ordering::Relaxed);
    tracing::info!(
        path = %path.display(),
        "secrets.key auto-created (event secrets-key-auto-created)"
    );
    let key = parse_hex_key(&hex)
        .map_err(|e| SshCliError::Config(format!("invalid generated key: {e}")));
    hex.zeroize();
    Ok((Some(key?), KeySource::XdgFile))
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
        SshCliError::InvalidArgument(format!(
            "config contains encrypted secrets; run `{APP_NAME} secrets init` (XDG `{KEY_FILE_NAME}`) or pass `--secrets-key-file PATH` / `--use-keyring` (env key material is not supported)"
        ))
    })?;
    let plain = decrypt_secret(&key, stored)?;
    key.zeroize();
    Ok(plain)
}

/// Generates [`PRIMARY_KEY_LEN_BYTES`] random bytes as [`PRIMARY_KEY_HEX_LEN`] hex chars.
pub fn generate_hex_key() -> SshCliResult<String> {
    let mut bytes = [0u8; PRIMARY_KEY_LEN_BYTES];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| SshCliError::Config(format!("RNG failed: {e}")))?;
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
        .map_err(|e| SshCliError::Config(format!("tempfile secrets.key: {e}")))?;
    use std::io::Write;
    tmp.write_all(hex64.trim().as_bytes())
        .map_err(|e| SshCliError::Config(format!("write secrets.key: {e}")))?;
    tmp.write_all(b"\n")
        .map_err(|e| SshCliError::Config(format!("write secrets.key: {e}")))?;
    tmp.as_file()
        .sync_all()
        .map_err(|e| SshCliError::Config(format!("fsync secrets.key: {e}")))?;
    crate::fs_perm::set_secret_file_mode(tmp.path())
        .map_err(|e| SshCliError::Config(format!("chmod secrets.key: {e}")))?;
    tmp.persist(path)
        .map_err(|e| SshCliError::Config(format!("persist secrets.key: {e}")))?;
    // Best-effort re-apply after rename (matches prior ignore-on-error chmod).
    let _ = crate::fs_perm::set_secret_file_mode(path);
    Ok(())
}

/// Initializes primary-key in XDG file or keyring. **Never** prints the key.
///
/// # Errors
/// Returns an error if the key already exists without `--force`, RNG fails, or keyring/file I/O fails.
pub fn init_primary_key(use_keyring: bool, force: bool) -> SshCliResult<SecretsStatus> {
    let mut hex = generate_hex_key()?;
    if use_keyring {
        if !force {
            match read_keyring() {
                Ok(Some(_)) => {
                    hex.zeroize();
                    return Err(SshCliError::InvalidArgument(
                        "keyring already has a primary-key; use --force".to_string(),
                    ));
                }
                Ok(None) => {}
                Err(e) => {
                    hex.zeroize();
                    return Err(e);
                }
            }
        }
        let result = write_key_to_keyring(&hex);
        hex.zeroize();
        result?;
        return secrets_status();
    }
    let path = secrets_key_path()?;
    let result = write_key_file(&path, &hex, force);
    hex.zeroize();
    result?;
    secrets_status()
}

/// Stores primary-key (hex) in the OS keyring. Does not print the key.
pub fn write_key_to_keyring(hex64: &str) -> SshCliResult<()> {
    let _ = parse_hex_key(hex64)
        .map_err(|e| SshCliError::InvalidArgument(format!("invalid key: {e}")))?;
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER_PRIMARY)
        .map_err(|e| SshCliError::Config(format!("keyring Entry::new failed: {e}")))?;
    entry
        .set_password(hex64.trim())
        .map_err(|e| SshCliError::Config(format!("keyring set failed: {e}")))?;
    Ok(())
}

fn parse_hex_key(hex: &str) -> Result<[u8; PRIMARY_KEY_LEN_BYTES], String> {
    let h = hex.trim();
    if h.len() != PRIMARY_KEY_HEX_LEN {
        return Err(format!(
            "expected {PRIMARY_KEY_HEX_LEN} hex characters ({PRIMARY_KEY_LEN_BYTES} bytes)"
        ));
    }
    let mut out = [0u8; PRIMARY_KEY_LEN_BYTES];
    for i in 0..PRIMARY_KEY_LEN_BYTES {
        let byte =
            u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).map_err(|_| "invalid hex".to_string())?;
        out[i] = byte;
    }
    Ok(out)
}

fn encrypt_secret(key: &[u8; PRIMARY_KEY_LEN_BYTES], plaintext: &str) -> SshCliResult<String> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| SshCliError::crypto("aead_key"))?;
    let mut nonce_bytes = [0u8; AEAD_NONCE_LEN_BYTES];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| SshCliError::Config(format!("RNG failed: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| SshCliError::crypto("encrypt"))?;
    let mut packed = Vec::with_capacity(AEAD_NONCE_LEN_BYTES + ciphertext.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);
    Ok(format!(
        "{ENC_PREFIX}{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &packed)
    ))
}

fn decrypt_secret(key: &[u8; PRIMARY_KEY_LEN_BYTES], blob: &str) -> SshCliResult<String> {
    let b64 = blob
        .strip_prefix(ENC_PREFIX)
        .ok_or_else(|| SshCliError::crypto("blob_parse"))?;
    let packed = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
        .map_err(|_| SshCliError::crypto("blob_b64"))?;
    if packed.len() < AEAD_NONCE_LEN_BYTES + AEAD_TAG_LEN_BYTES {
        return Err(SshCliError::Config(
            "encrypted blob too short".to_string(),
        ));
    }
    let (nonce_bytes, ct) = packed.split_at(AEAD_NONCE_LEN_BYTES);
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| SshCliError::crypto("aead_key"))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher.decrypt(nonce, ct).map_err(|_| {
        SshCliError::crypto("decrypt")
    })?;
    match String::from_utf8(plain) {
        Ok(s) => Ok(s),
        Err(e) => {
            // from_utf8 failure keeps bytes in the error — scrub before drop.
            let mut bad = e.into_bytes();
            bad.zeroize();
            Err(SshCliError::Config(
                "decrypted secret is not valid UTF-8".to_string(),
            ))
        }
    }
}

fn read_keyring() -> SshCliResult<Option<[u8; PRIMARY_KEY_LEN_BYTES]>> {
    // Prefer inclusive primary-key id; fall back to legacy master-key user for migration.
    for user in [KEYRING_USER_PRIMARY, KEYRING_USER_LEGACY] {
        let entry = match keyring::Entry::new(KEYRING_SERVICE, user) {
            Ok(e) => e,
            Err(e) => {
                if user == "secrets-master-key" {
                    return Err(SshCliError::Config(format!("keyring Entry::new failed: {e}")));
                }
                continue;
            }
        };
        match entry.get_password() {
            Ok(mut s) => {
                let key = parse_hex_key(&s).map_err(|e| {
                    SshCliError::InvalidArgument(format!("invalid keyring primary-key: {e}"))
                });
                s.zeroize();
                return Ok(Some(key?));
            }
            Err(keyring::Error::NoEntry) => continue,
            Err(e) => {
                if user == "secrets-master-key" {
                    return Err(SshCliError::Config(format!("keyring get failed: {e}")));
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
        // Fail-closed path reads these keys; clear so serial tests start clean.
        crate::test_util::env::remove_var(ENV_SECRETS_KEY);
        crate::test_util::env::remove_var(ENV_SECRETS_KEY_FILE);
        crate::test_util::env::remove_var(crate::constants::ENV_USE_KEYRING);
        set_runtime_flags(false, None, false);
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
    fn roundtrip_with_xdg_key() {
        let _tmp = sandbox();
        init_primary_key(false, false).expect("init key");
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
        set_runtime_flags(true, None, false);
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
        init_primary_key(false, false).expect("init");
        let enc = serialize_secret("fake-secret").unwrap();
        // Drop key material from sandbox; allow plaintext so deserialize path
        // still requires a key for encrypted blobs.
        clear_key_env();
        set_config_dir(Some(tmp.path().to_path_buf()));
        let _ = std::fs::remove_file(tmp.path().join(KEY_FILE_NAME));
        set_runtime_flags(true, None, false);
        let err = deserialize_secret(&enc).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("encrypted") || msg.contains("secrets") || msg.contains("key"),
            "msg={msg}"
        );
        clear_key_env();
    }

    #[test]
    #[serial]
    fn empty_secret_never_encrypted_blob() {
        // GAP-SSH-EXP-001
        let _tmp = sandbox();
        init_primary_key(false, false).expect("init");
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

    #[test]
    fn lock_global_recovers_from_poison_with_usable_data() {
        let m = Mutex::new(42_u32);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _g = m.lock().unwrap();
            panic!("intentional poison for lock_global test");
        }));
        assert!(m.is_poisoned());
        let g = lock_global(&m);
        assert_eq!(*g, 42);
    }
}
