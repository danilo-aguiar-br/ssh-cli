//! Unit tests for the `model` module.
#![forbid(unsafe_code)]

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
    assert!(
        !s.contains("adicionado_em ="),
        "must not write PT adicionado_em: {s}"
    );
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
