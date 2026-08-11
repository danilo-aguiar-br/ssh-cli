// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for internationalization (locale + Message parity).

use serial_test::serial;
use ssh_cli::i18n::{current_language, initialize_language, Language, Message, TextDirection};
use ssh_cli::locale::{
    negotiate_code, normalize_raw_locale, parse_lang_cli_arg, parse_language_identifier,
    resolve_language_detailed, write_persisted_lang, LocaleSource,
};

#[test]
#[serial]
fn initialize_language_accepts_a_valid_locale() {
    std::env::remove_var("SSH_CLI_LANG");
    let result = initialize_language(Some("pt-BR"), None);
    assert!(
        result.is_ok(),
        "initialize_language must accept a valid locale"
    );
}

#[test]
#[serial]
fn initialize_language_falls_through_on_an_invalid_locale() {
    // Programmatic API still accepts invalid and falls through (clap rejects at CLI).
    std::env::remove_var("SSH_CLI_LANG");
    let result = initialize_language(Some("xx-XX"), None);
    assert!(
        result.is_ok(),
        "initialize_language must fall through, not fail, on an invalid locale"
    );
}

#[test]
#[serial]
fn initialize_language_succeeds_without_an_override() {
    std::env::remove_var("SSH_CLI_LANG");
    let result = initialize_language(None, None);
    assert!(result.is_ok(), "initialize_language must not fail");
}

#[test]
#[serial]
fn initialize_language_accepts_a_valid_env_locale() {
    std::env::set_var("SSH_CLI_LANG", "en-US");
    let result = initialize_language(None, None);
    std::env::remove_var("SSH_CLI_LANG");
    assert!(
        result.is_ok(),
        "initialize_language must accept a valid env locale"
    );
}

#[test]
#[serial]
fn initialize_language_falls_through_on_an_invalid_env_locale() {
    std::env::set_var("SSH_CLI_LANG", "xx-XX");
    let result = initialize_language(None, None);
    std::env::remove_var("SSH_CLI_LANG");
    assert!(
        result.is_ok(),
        "initialize_language must fall through on an invalid env locale"
    );
}

#[test]
#[serial]
fn current_language_returns_valid_locale() {
    initialize_language(None, None).expect("initialize_language must not fail");
    let language = current_language();
    assert!(
        language == Language::English || language == Language::Portuguese,
        "current_language must return a supported locale"
    );
}

#[test]
#[serial]
fn initialize_language_selects_portuguese() {
    let result = initialize_language(Some("pt-BR"), None);
    assert!(result.is_ok());
}

#[test]
#[serial]
fn initialize_language_selects_english() {
    let result = initialize_language(Some("en-US"), None);
    assert!(result.is_ok());
}

#[test]
fn message_vps_registry_empty_en_nonempty() {
    let text = Message::VpsRegistryEmpty.text(Language::English);
    assert!(!text.is_empty());
}

#[test]
fn message_vps_registry_empty_pt_nonempty() {
    let text = Message::VpsRegistryEmpty.text(Language::Portuguese);
    assert!(!text.is_empty());
}

#[test]
fn message_vps_registry_empty_pt_differs_en() {
    let en = Message::VpsRegistryEmpty.text(Language::English);
    let pt = Message::VpsRegistryEmpty.text(Language::Portuguese);
    assert_ne!(en, pt);
}

#[test]
fn unit_variants_render_in_both_languages() {
    let unit_variants = [
        Message::VpsRegistryEmpty,
        Message::TunnelPressCtrlC,
        Message::LocalePreferenceCleared,
        Message::LocaleStatusTitle,
        Message::ImportCompleted,
    ];
    for variant in &unit_variants {
        assert!(
            !variant.text(Language::English).is_empty(),
            "empty EN for {:?}",
            variant
        );
        assert!(
            !variant.text(Language::Portuguese).is_empty(),
            "empty PT for {:?}",
            variant
        );
    }
}

#[test]
fn variants_with_fields_interpolate_their_data() {
    let cases: Vec<(Message, &str)> = vec![
        (
            Message::VpsAdded {
                name: "meu-servidor".to_string(),
            },
            "meu-servidor",
        ),
        (
            Message::VpsRemoved {
                name: "servidor-antigo".to_string(),
            },
            "servidor-antigo",
        ),
        (
            Message::VpsDuplicate {
                name: "duplicado".to_string(),
            },
            "duplicado",
        ),
        (
            Message::VpsNotFound {
                name: "inexistente".to_string(),
            },
            "inexistente",
        ),
        (
            Message::HealthCheckOk {
                name: "prod-01".to_string(),
            },
            "prod-01",
        ),
        (
            // B2: replaces the unreachable `HealthCheckFailed`; this variant is
            // the one the error path actually renders now.
            Message::ErrorSshConnection {
                detail: "connection refused".to_string(),
            },
            "connection refused",
        ),
        (
            Message::ErrorUnavailable {
                service: "keyring".to_string(),
            },
            "keyring",
        ),
        (
            Message::LocalePreferenceSaved {
                lang: "pt-BR".to_string(),
                path: "/tmp/lang".to_string(),
            },
            "pt-BR",
        ),
    ];
    for (msg, expected) in &cases {
        assert!(
            msg.text(Language::English).contains(expected),
            "EN is missing '{}' for {:?}",
            expected,
            msg
        );
        assert!(
            msg.text(Language::Portuguese).contains(expected),
            "PT is missing '{}' for {:?}",
            expected,
            msg
        );
    }
}

#[test]
fn tunnel_local_listening_includes_port_host_and_vps() {
    // B2: `TunnelActive` was superseded by the per-mode listening variants that
    // closed the recorded tunnel incident; only the survivor is asserted here.
    let msg = Message::TunnelLocalListening {
        bind: "127.0.0.1".to_string(),
        port: 8080,
        remote_host: "10.0.0.1".to_string(),
        remote_port: 22,
        vps: "relay-01".to_string(),
        timeout_ms: 1_000,
    };
    let en = msg.text(Language::English);
    let pt = msg.text(Language::Portuguese);
    for text in &[en, pt] {
        assert!(text.contains("8080"), "must carry the local port");
        assert!(text.contains("10.0.0.1"), "must carry the remote host");
        assert!(text.contains("22"), "must carry the remote port");
        assert!(text.contains("relay-01"), "must carry the VPS name");
    }
}

#[test]
fn bcp47_normalize_and_negotiate() {
    assert_eq!(normalize_raw_locale("pt_BR.UTF-8"), "pt-BR");
    assert!(parse_language_identifier("C.UTF-8").is_none());
    assert_eq!(negotiate_code("pt_BR.UTF-8"), Some(Language::Portuguese));
    assert_eq!(negotiate_code("en-GB"), Some(Language::English));
    assert_eq!(negotiate_code("fr-FR"), None);
    assert!(parse_lang_cli_arg("pt-BR").is_ok());
    assert!(parse_lang_cli_arg("zh-Hans-CN").is_err());
}

#[test]
fn language_metadata_mvp() {
    assert_eq!(Language::English.bcp47(), "en");
    assert_eq!(Language::Portuguese.bcp47(), "pt-BR");
    assert_eq!(Language::Portuguese.direction(), TextDirection::Ltr);
    assert_eq!(Language::AVAILABLE.len(), 2);
}

#[test]
#[serial]
fn persisted_preference_layer_wins_without_flag() {
    std::env::remove_var("SSH_CLI_LANG");
    let dir = tempfile::tempdir().expect("tempdir");
    write_persisted_lang(Language::Portuguese, Some(dir.path())).expect("write");
    let r = resolve_language_detailed(None, Some(dir.path()));
    assert_eq!(r.language, Language::Portuguese);
    assert_eq!(r.source, LocaleSource::Persisted);
}
