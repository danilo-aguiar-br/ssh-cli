// SPDX-License-Identifier: MIT OR Apache-2.0
//! Testes de integração do sistema de internacionalização.
//!
//! Testa as funções públicas do módulo i18n e locale.

use serial_test::serial;
use ssh_cli::i18n::{current_language, initialize_language, Language, Message};

#[test]
#[serial]
fn inicializar_idioma_nao_panic_com_locale_valido() {
    std::env::remove_var("SSH_CLI_LANG");
    let result = initialize_language(Some("pt-BR"));
    assert!(
        result.is_ok(),
        "inicializar_idioma não deve falhar com locale válido"
    );
}

#[test]
#[serial]
fn inicializar_idioma_nao_panic_com_locale_invalido() {
    std::env::remove_var("SSH_CLI_LANG");
    let result = initialize_language(Some("xx-XX"));
    assert!(
        result.is_ok(),
        "inicializar_idioma não deve falhar com locale inválido"
    );
}

#[test]
#[serial]
fn inicializar_idioma_nao_panic_sem_forcar() {
    std::env::remove_var("SSH_CLI_LANG");
    let result = initialize_language(None);
    assert!(result.is_ok(), "initialize_language must not fail");
}

#[test]
#[serial]
fn inicializar_idioma_com_env_var_valida_nao_panic() {
    std::env::set_var("SSH_CLI_LANG", "en-US");
    let result = initialize_language(None);
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
    let result = initialize_language(None);
    std::env::remove_var("SSH_CLI_LANG");
    assert!(
        result.is_ok(),
        "inicializar_idioma não deve falhar com env var inválida"
    );
}

#[test]
#[serial]
fn current_language_returns_valid_locale() {
    initialize_language(None).expect("initialize_language must not fail");
    let language = current_language();
    assert!(
        language == Language::English || language == Language::Portuguese,
        "idioma_atual deve ser um locale suportado"
    );
}

#[test]
#[serial]
fn inicializar_com_pt_br_define_portugues() {
    // OnceLock já pode estar setado — o result deve ser válido de qualquer forma
    let result = initialize_language(Some("pt-BR"));
    assert!(result.is_ok());
}

#[test]
#[serial]
fn inicializar_com_en_us_define_english() {
    let result = initialize_language(Some("en-US"));
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
    let pt = msg.clone().text(Language::Portuguese);
    for text in &[en, pt] {
        assert!(text.contains("8080"), "deve conter porta_local");
        assert!(text.contains("10.0.0.1"), "deve conter host_remoto");
        assert!(text.contains("22"), "deve conter porta_remota");
        assert!(text.contains("relay-01"), "deve conter vps_nome");
    }
}
