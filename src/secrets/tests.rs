//! Unit tests for the `secrets` module.
#![forbid(unsafe_code)]

use super::*;
// Crypto types live in `secrets::aead` now; the legacy-v1 fixture below still
// needs to seal a blob by hand, so it imports them locally instead of forcing
// the production module to keep an import it no longer uses.
use crate::constants::AEAD_NONCE_LEN_BYTES;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
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
#[serial]
fn v2_blob_rejects_foreign_host_and_field() {
    // A7: an operator editing config.toml must not be able to move a blob
    // between hosts or between fields and still have it decrypt.
    let _tmp = sandbox();
    init_primary_key(false, false).expect("init key");
    let plain = "fake-bound-secret";
    let host_a = SecretContext::new("host-a", "password");
    let enc = serialize_secret_in_context(host_a, plain).unwrap();
    assert!(enc.starts_with(ENC_PREFIX_V2), "v2 must be written");

    assert_eq!(deserialize_secret_in_context(host_a, &enc).unwrap(), plain);

    let other_host = SecretContext::new("host-b", "password");
    assert!(
        deserialize_secret_in_context(other_host, &enc).is_err(),
        "blob bound to host-a must not open as host-b"
    );

    let other_field = SecretContext::new("host-a", "su_password");
    assert!(
        deserialize_secret_in_context(other_field, &enc).is_err(),
        "blob bound to password must not open as su_password"
    );
    clear_key_env();
}

#[test]
#[serial]
fn unbound_blob_stays_readable_under_any_context() {
    // Migration: call sites not yet passing host/field write unbound blobs;
    // wiring a context later must not lock the user out of their config.
    let _tmp = sandbox();
    init_primary_key(false, false).expect("init key");
    let plain = "fake-unbound-secret";
    let enc = serialize_secret(plain).unwrap();
    assert!(enc.starts_with(ENC_PREFIX_V2));
    let bound = SecretContext::new("host-a", "password");
    assert_eq!(deserialize_secret_in_context(bound, &enc).unwrap(), plain);
    clear_key_env();
}

#[test]
#[serial]
fn legacy_v1_blob_still_decrypts() {
    // Existing configs hold v1 blobs sealed without associated data.
    let _tmp = sandbox();
    init_primary_key(false, false).expect("init key");
    let (key, _) = load_primary_key().unwrap();
    let key = key.expect("key present");
    let plain = "fake-legacy-v1-secret";

    // Rebuild a v1 blob exactly as the previous version wrote it.
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_slice()).unwrap();
    let mut nonce_bytes = [0u8; AEAD_NONCE_LEN_BYTES];
    getrandom::fill(&mut nonce_bytes).unwrap();
    let ct = cipher
        .encrypt(&Nonce::from(nonce_bytes), plain.as_bytes())
        .unwrap();
    let mut packed = nonce_bytes.to_vec();
    packed.extend_from_slice(&ct);
    let blob = format!(
        "{ENC_PREFIX}{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &packed)
    );

    assert!(is_encrypted_blob(&blob));
    assert_eq!(deserialize_secret(&blob).unwrap(), plain);
    assert_eq!(
        deserialize_secret_in_context(SecretContext::new("host-a", "password"), &blob).unwrap(),
        plain
    );
    clear_key_env();
}

#[test]
fn aad_encoding_is_unambiguous() {
    // A host literally named "a:b" must not collide with the pair (a, b).
    assert_ne!(
        SecretContext::new("a:b", "password").aad(),
        SecretContext::new("a", "b:password").aad()
    );
    assert!(SecretContext::unbound().is_unbound());
    assert!(!SecretContext::new("host-a", "password").is_unbound());
}

#[test]
fn parse_hex_tamanho() {
    assert!(parse_hex_key("aa").is_err());
    assert!(
        parse_hex_key("00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff").is_ok()
    );
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
