// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: unit tests extracted for line budget.
#![forbid(unsafe_code)]

use super::*;

    #[test]
    fn language_enum_is_copy() {
        let a = Language::English;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn message_is_not_copy_but_is_clone() {
        let m = Message::VpsAdded {
            name: "vps-01".to_string(),
        };
        let m2 = m.clone();
        assert_eq!(m, m2);
    }

    #[test]
    fn vps_registry_empty_en() {
        assert_eq!(
            Message::VpsRegistryEmpty.text(Language::English),
            "No VPS registered."
        );
    }

    #[test]
    fn vps_registry_empty_pt() {
        assert_eq!(
            Message::VpsRegistryEmpty.text(Language::Portuguese),
            "Nenhum VPS cadastrado."
        );
    }

    #[test]
    fn vps_added_includes_name_en() {
        let msg = Message::VpsAdded {
            name: "prod-01".to_string(),
        };
        assert_eq!(
            msg.text(Language::English),
            "VPS 'prod-01' added successfully."
        );
    }

    #[test]
    fn vps_added_includes_name_pt() {
        let msg = Message::VpsAdded {
            name: "prod-01".to_string(),
        };
        assert_eq!(
            msg.text(Language::Portuguese),
            "VPS 'prod-01' adicionada com sucesso."
        );
    }

    #[test]
    fn vps_removed_includes_name() {
        let msg = Message::VpsRemoved {
            name: "dev-01".to_string(),
        };
        assert!(msg.text(Language::English).contains("dev-01"));
        assert!(msg.text(Language::Portuguese).contains("dev-01"));
    }

    #[test]
    fn vps_duplicate_includes_name() {
        let msg = Message::VpsDuplicate {
            name: "staging".to_string(),
        };
        assert!(msg.text(Language::English).contains("staging"));
        assert!(msg.text(Language::Portuguese).contains("staging"));
    }

    #[test]
    fn vps_not_found_includes_name() {
        let msg = Message::VpsNotFound {
            name: "inexistente".to_string(),
        };
        assert!(msg.text(Language::English).contains("inexistente"));
        assert!(msg.text(Language::Portuguese).contains("inexistente"));
    }

    #[test]
    fn tunnel_active_includes_all_fields() {
        let msg = Message::TunnelActive {
            local_port: 8080,
            remote_host: "1.2.3.4".to_string(),
            remote_port: 22,
            vps_name: "meu-servidor".to_string(),
        };
        let en = msg.text(Language::English);
        assert!(en.contains("8080"));
        assert!(en.contains("1.2.3.4"));
        assert!(en.contains("22"));
        assert!(en.contains("meu-servidor"));
    }

    #[test]
    fn error_invalid_argument_includes_detail() {
        let msg = Message::ErrorInvalidArgument {
            detail: "port out of range".to_string(),
        };
        assert!(msg
            .text(Language::English)
            .contains("port out of range"));
        assert!(msg
            .text(Language::Portuguese)
            .contains("port out of range"));
    }

    #[test]
    fn health_check_ok_includes_name() {
        let msg = Message::HealthCheckOk {
            name: "prod-01".to_string(),
        };
        assert!(msg.text(Language::English).contains("prod-01"));
        assert!(msg.text(Language::Portuguese).contains("prod-01"));
    }

    #[test]
    fn all_unit_variants_en_nonempty() {
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
            Message::ImportCompleted,
            Message::ScpUploadFileOnly,
            Message::ScpDownloadLocalNotDirectory,
            Message::LocalePreferenceCleared,
            Message::LocaleStatusTitle,
        ];
        for v in &unit_variants {
            let text = v.text(Language::English);
            assert!(!text.is_empty(), "empty EN for {:?}", v);
        }
    }

    #[test]
    fn all_unit_variants_pt_nonempty() {
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
            Message::ImportCompleted,
            Message::ScpUploadFileOnly,
            Message::ScpDownloadLocalNotDirectory,
            Message::LocalePreferenceCleared,
            Message::LocaleStatusTitle,
        ];
        for v in &unit_variants {
            let text = v.text(Language::Portuguese);
            assert!(!text.is_empty(), "empty PT for {:?}", v);
        }
    }

    #[test]
    fn language_bcp47_and_direction() {
        assert_eq!(Language::English.bcp47(), "en");
        assert_eq!(Language::Portuguese.bcp47(), "pt-BR");
        assert_eq!(Language::English.direction(), TextDirection::Ltr);
        assert_eq!(Language::Portuguese.script(), "Latn");
        assert_eq!(Language::Portuguese.fallback(), Language::English);
        assert_eq!(Language::AVAILABLE.len(), 2);
        let id = Language::Portuguese.language_identifier();
        assert_eq!(Language::from_langid(&id), Some(Language::Portuguese));
    }

    #[test]
    fn en_pt_parity_unit_variants_differ() {
        // Parity: both non-empty and (for pure UI units) not identical.
        for v in [
            Message::VpsRegistryEmpty,
            Message::LocaleStatusTitle,
            Message::LocalePreferenceCleared,
        ] {
            let en = v.text(Language::English);
            let pt = v.text(Language::Portuguese);
            assert!(!en.is_empty());
            assert!(!pt.is_empty());
            assert_ne!(en, pt, "EN/PT must differ for {:?}", v);
        }
    }

    #[test]
    fn pt_translations_differ_from_en_for_units() {
        let pairs = [
            (Message::VpsRegistryEmpty, Message::VpsRegistryEmpty),
            (Message::ErrorSshConnection, Message::ErrorSshConnection),
            (Message::HealthCheckNoVps, Message::HealthCheckNoVps),
            (Message::OperationCancelled, Message::OperationCancelled),
        ];
        for (a, b) in &pairs {
            let en = a.text(Language::English);
            let pt = b.text(Language::Portuguese);
            assert_ne!(en, pt, "EN == PT for {:?}", a);
        }
    }

    #[test]
    fn health_check_failed_includes_name_and_detail() {
        let msg = Message::HealthCheckFailed {
            name: "prod-01".to_string(),
            detail: "timeout".to_string(),
        };
        assert!(msg.text(Language::English).contains("prod-01"));
        assert!(msg.text(Language::English).contains("timeout"));
        assert!(msg.text(Language::Portuguese).contains("prod-01"));
        assert!(msg.text(Language::Portuguese).contains("timeout"));
    }

    #[test]
    fn health_check_latency_includes_name_and_ms() {
        let msg = Message::HealthCheckLatency {
            name: "relay-01".to_string(),
            latency_ms: 42,
        };
        assert!(msg.text(Language::English).contains("relay-01"));
        assert!(msg.text(Language::English).contains("42"));
        assert!(msg.text(Language::Portuguese).contains("relay-01"));
        assert!(msg.text(Language::Portuguese).contains("42"));
    }

    #[test]
    fn initialize_language_without_force_no_panic() {
        let result = initialize_language(None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn initialize_language_with_pt_br_works() {
        let result = initialize_language(Some("pt-BR"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn current_language_returns_valid_value() {
        let language = current_language();
        assert!(language == Language::English || language == Language::Portuguese);
    }
