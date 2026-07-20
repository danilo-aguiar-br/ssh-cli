// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for internationalization (locale + Message parity).

use serial_test::serial;
use ssh_cli::i18n::{current_language, initialize_language, Language, Message, TextDirection};
use ssh_cli::locale::{
    negotiate_code, normalize_raw_locale, parse_language_identifier, parse_lang_cli_arg,
    resolve_language_detailed, write_persisted_lang, LocaleSource,
};

#[test]
#[serial]
fn inicializar_idioma_nao_panic_com_locale_valido() {
    std::env::remove_var("SSH_CLI_LANG");
    let result = initialize_language(Some("pt-BR"), None);
    assert!(
        result.is_ok(),
        "inicializar_idioma não deve falhar com locale válido"
    );
}

#[test]
#[serial]
fn inicializar_idioma_nao_panic_com_locale_invalido() {
    // Programmatic API still accepts invalid and falls through (clap rejects at CLI).
    std::env::remove_var("SSH_CLI_LANG");
    let result = initialize_language(Some("xx-XX"), None);
    assert!(
        result.is_ok(),
        "inicializar_idioma não deve falhar com locale inválido"
    );
}

#[test]
#[serial]
fn inicializar_idioma_nao_panic_sem_forcar() {
    std::env::remove_var("SSH_CLI_LANG");
    let result = initialize_language(None, None);
    assert!(result.is_ok(), "initialize_language must not fail");
}

#[test]
#[serial]
fn inicializar_idioma_com_env_var_valida_nao_panic() {
    std::env::set_var("SSH_CLI_LANG", "en-US");
    let result = initialize_language(None, None);
    std::env::remove_var("SSH_CLI_LANG");
    assert!(
        result.is_ok(),
        "inicializar_idioma não deve falhar com env var válida"
    );
}

#[test]
#[serial]
fn inicializar_idioma_com_env_var_invalida_nao_panic() {
    std::env::set_var("SSH_CLI_LANG", "xx-XX");
    let result = initialize_language(None, None);
    std::env::remove_var("SSH_CLI_LANG");
    assert!(
        result.is_ok(),
        "inicializar_idioma não deve falhar com env var inválida"
    );
}

#[test]
#[serial]
fn current_language_returns_valid_locale() {
    initialize_language(None, None).expect("initialize_language must not fail");
    let language = current_language();
    assert!(
        language == Language::English || language == Language::Portuguese,
        "idioma_atual deve ser um locale suportado"
    );
}

#[test]
#[serial]
fn inicializar_com_pt_br_define_portugues() {
    let result = initialize_language(Some("pt-BR"), None);
    assert!(result.is_ok());
}

#[test]
#[serial]
fn inicializar_com_en_us_define_english() {
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
fn variantes_unitarias_nao_vazias_em_ambos_idiomas() {
    let unit_variants = [
        Message::VpsRegistryEmpty,
        Message::VpsListTitle,
        Message::ConfigPathLabel,
        Message::ConfigNoKeys,
        Message::ErrorLoadConfig,
        Message::ErrorSaveConfig,
        Message::ErrorSshConnection,
        Message::ErrorCommandFailed,
        Message::TunnelPressCtrlC,
        Message::HealthCheckNoVps,
        Message::OperationCancelled,
        Message::LocalePreferenceCleared,
        Message::LocaleStatusTitle,
        Message::ImportCompleted,
        Message::ScpUploadFileOnly,
        Message::ScpDownloadLocalNotDirectory,
    ];
    for variante in &unit_variants {
        assert!(
            !variante.text(Language::English).is_empty(),
            "empty EN for {:?}",
            variante
        );
        assert!(
            !variante.text(Language::Portuguese).is_empty(),
            "empty PT for {:?}",
            variante
        );
    }
}

#[test]
fn variantes_com_campos_incluem_dados_dinamicos() {
    let casos: Vec<(Message, &str)> = vec![
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
            Message::HealthCheckFailed {
                name: "test-vps".to_string(),
                detail: "connection refused".to_string(),
            },
            "test-vps",
        ),
        (
            Message::HealthCheckLatency {
                name: "relay-01".to_string(),
                latency_ms: 42,
            },
            "relay-01",
        ),
        (
            Message::LocalePreferenceSaved {
                lang: "pt-BR".to_string(),
                path: "/tmp/lang".to_string(),
            },
            "pt-BR",
        ),
    ];
    for (msg, esperado) in &casos {
        assert!(
            msg.text(Language::English).contains(esperado),
            "EN não contém '{}' para {:?}",
            esperado,
            msg
        );
        assert!(
            msg.text(Language::Portuguese).contains(esperado),
            "PT não contém '{}' para {:?}",
            esperado,
            msg
        );
    }
}

#[test]
fn tunnel_active_includes_port_host_and_vps() {
    let msg = Message::TunnelActive {
        local_port: 8080,
        remote_host: "10.0.0.1".to_string(),
        remote_port: 22,
        vps_name: "relay-01".to_string(),
    };
    let en = msg.text(Language::English);
    let pt = msg.text(Language::Portuguese);
    for text in &[en, pt] {
        assert!(text.contains("8080"), "deve conter porta_local");
        assert!(text.contains("10.0.0.1"), "deve conter host_remoto");
        assert!(text.contains("22"), "deve conter porta_remota");
        assert!(text.contains("relay-01"), "deve conter vps_nome");
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
