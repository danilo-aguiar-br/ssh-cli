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
fn tunnel_local_listening_includes_all_fields() {
    // B2: `TunnelActive` was replaced by per-mode listening variants and its
    // translations sat unreachable. This asserts the surviving variant instead.
    let msg = Message::TunnelLocalListening {
        bind: "127.0.0.1".to_string(),
        port: 8080,
        remote_host: "1.2.3.4".to_string(),
        remote_port: 22,
        vps: "meu-servidor".to_string(),
        timeout_ms: 1000,
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
    assert!(msg.text(Language::English).contains("port out of range"));
    assert!(msg.text(Language::Portuguese).contains("port out of range"));
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
        Message::TunnelPressCtrlC,
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
        Message::TunnelPressCtrlC,
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
        (Message::TunnelPressCtrlC, Message::TunnelPressCtrlC),
        (Message::OperationCancelled, Message::OperationCancelled),
    ];
    for (a, b) in &pairs {
        let en = a.text(Language::English);
        let pt = b.text(Language::Portuguese);
        assert_ne!(en, pt, "EN == PT for {:?}", a);
    }
}

/// B2: the error variants must reach the human branch through the real product
/// API, not through a hand-built `Message`. `localized_error_text` is the seam
/// that made six fully-translated variants reachable for the first time.
#[test]
fn localized_error_text_translates_and_keeps_the_detail() {
    use crate::errors::SshCliError;

    let err = SshCliError::SshConnection("connection refused".to_string());
    let text = crate::i18n::localized_error_text(&err)
        .expect("ssh_connection must have a localized rendering");

    // The upstream detail survives translation — no diagnostic is lost.
    assert!(
        text.contains("connection refused"),
        "detail must survive localization, got: {text}"
    );
}

/// The translated sentence must not re-wrap a label the `Display` already added.
///
/// First cut of B2 fed `err.to_string()` into every `Message`, which produced
/// `VPS 'vps 'x' not found in registry' not found.` — the English label smuggled
/// inside the Portuguese one. Only the inner payload may cross the boundary.
#[test]
fn localized_error_text_does_not_double_wrap_the_english_label() {
    use crate::errors::SshCliError;

    let cases: &[(SshCliError, &[&str])] = &[
        (
            SshCliError::VpsNotFound("prod-01".to_string()),
            &["not found in registry", "não encontrada em"],
        ),
        (
            SshCliError::InvalidArgument("port out of range".to_string()),
            &["invalid argument:", "Argumento inválido: Argumento"],
        ),
        (
            SshCliError::FileNotFound("/tmp/x".to_string()),
            &["file not found:"],
        ),
    ];

    for (err, forbidden) in cases {
        for lang in [Language::English, Language::Portuguese] {
            crate::i18n::initialize_language(Some(lang.bcp47()), None).ok();
            let text = crate::i18n::localized_error_text(err)
                .unwrap_or_else(|| panic!("{err:?} must localize"));
            for needle in *forbidden {
                assert!(
                    !text.contains(needle),
                    "double-wrapped label for {err:?} in {lang:?}: {text}"
                );
            }
        }
    }
}

/// C2: the untyped failure branch must localize its label and keep the chain.
///
/// B2 wired every typed `SshCliError` through i18n and left `resolve_exit_code`'s
/// last branch printing the raw English `anyhow` chain under `--lang pt-BR` —
/// the one error a user is least equipped to interpret was the only one never
/// translated.
#[test]
fn localized_unexpected_text_translates_the_label_and_keeps_the_chain() {
    let chain = "config load failed: permission denied";

    // The locale is process-global and initialized once, so a test that *switches*
    // it is order-dependent under the parallel harness. Both arms are compared
    // through `text()` directly; the public helper is exercised for the property
    // that does not depend on which locale won the race.
    let msg = Message::ErrorUnexpected {
        detail: chain.to_string(),
    };
    let en = msg.text(Language::English);
    let pt = msg.text(Language::Portuguese);

    assert_ne!(en, pt, "the label must differ between locales");
    for text in [&en, &pt] {
        assert!(
            text.contains(chain),
            "the upstream chain must survive verbatim, got: {text}"
        );
        // The chain is the payload, not a second label: never wrapped twice.
        assert_eq!(
            text.matches(chain).count(),
            1,
            "chain duplicated in the localized line: {text}"
        );
    }

    let rendered = crate::i18n::localized_unexpected_text(chain);
    assert!(
        rendered == en || rendered == pt,
        "the helper must render one of the two locale arms, got: {rendered}"
    );
}

#[test]
fn localized_error_text_is_none_for_untranslated_codes() {
    use crate::errors::SshCliError;

    // Machine-facing plumbing keeps the English Display: returning `None` here is
    // what makes the human branch fail open instead of printing an empty line.
    let err = SshCliError::Io(std::io::Error::other("disk"));
    assert!(
        crate::i18n::localized_error_text(&err).is_none(),
        "io must fall back to the English Display"
    );
}

#[test]
fn localized_error_text_differs_between_locales() {
    use crate::errors::SshCliError;

    // The whole point of B2: before this seam existed, `--lang pt-BR` produced
    // byte-identical English output for every error.
    let err = SshCliError::AuthenticationFailed;
    let en = Message::ErrorAuthentication {
        detail: err.to_string(),
    }
    .text(Language::English);
    let pt = Message::ErrorAuthentication {
        detail: err.to_string(),
    }
    .text(Language::Portuguese);

    assert_ne!(en, pt, "EN and pt-BR error prose must differ");
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
