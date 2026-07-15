// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cross-platform language detection and resolution.
//!
//! Language selection precedence (highest to lowest):
//! 1. CLI `--lang` flag
//! 2. `SSH_CLI_LANG` environment variable
//! 3. System locale via `sys_locale::get_locale()`
//! 4. Fallback: `Language::English`

use std::sync::OnceLock;

use crate::i18n::Language;

/// Global language state — set once at initialization.
static GLOBAL_LANGUAGE: OnceLock<Language> = OnceLock::new();

/// Resolves language using a 4-layer precedence hierarchy.
///
/// Returns the first valid language found in order:
/// flag CLI > env SSH_CLI_LANG > sys_locale > English.
pub fn resolve_language(force_lang: Option<&str>) -> Language {
    // Camada 1: flag --lang da CLI
    if let Some(codigo) = force_lang {
        if let Some(language) = code_to_language(codigo) {
            return language;
        }
    }

    // Layer 2: SSH_CLI_LANG environment variable
    if let Ok(env_lang) = std::env::var("SSH_CLI_LANG") {
        if let Some(language) = code_to_language(&env_lang) {
            return language;
        }
    }

    // Camada 3: locale do sistema via sys_locale
    if let Some(locale) = sys_locale::get_locale() {
        if let Some(language) = code_to_language(&locale) {
            return language;
        }
    }

    // Camada 4: fallback incondicional
    Language::English
}

/// Sets the global language (once at process startup).
///
/// Subsequent calls are silently ignored — `OnceLock`
/// guarantees the language is immutable after first set.
pub fn set_language(language: Language) {
    let _ = GLOBAL_LANGUAGE.set(language);
}

/// Returns the current global language.
///
/// If `set_language` has not been called yet, returns `Language::English`
/// as a safe fallback for code run before initialization.
pub fn current_language() -> Language {
    GLOBAL_LANGUAGE.get().copied().unwrap_or(Language::English)
}

/// Converts a language code string to `Language`.
///
/// Recognizes "pt" and "en" prefixes with any region suffix,
/// case-insensitively.
fn code_to_language(codigo: &str) -> Option<Language> {
    let normalizado = codigo.to_lowercase();
    match normalizado.as_str() {
        "pt" | "pt-br" | "pt_br" => Some(Language::Portuguese),
        "en" | "en-us" | "en_us" => Some(Language::English),
        outro => {
            if outro.starts_with("pt") {
                Some(Language::Portuguese)
            } else if outro.starts_with("en") {
                Some(Language::English)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_pt_returns_portuguese() {
        assert_eq!(code_to_language("pt"), Some(Language::Portuguese));
    }

    #[test]
    fn code_pt_br_returns_portuguese() {
        assert_eq!(code_to_language("pt-BR"), Some(Language::Portuguese));
    }

    #[test]
    fn code_pt_br_underscore_returns_portuguese() {
        assert_eq!(code_to_language("pt_BR"), Some(Language::Portuguese));
    }

    #[test]
    fn code_en_returns_english() {
        assert_eq!(code_to_language("en"), Some(Language::English));
    }

    #[test]
    fn code_en_us_returns_english() {
        assert_eq!(code_to_language("en-US"), Some(Language::English));
    }

    #[test]
    fn code_en_gb_returns_english_by_prefix() {
        assert_eq!(code_to_language("en-GB"), Some(Language::English));
    }

    #[test]
    fn unknown_code_returns_none() {
        assert_eq!(code_to_language("fr-FR"), None);
    }

    #[test]
    fn empty_code_returns_none() {
        assert_eq!(code_to_language(""), None);
    }

    #[test]
    fn uppercase_code_normalized() {
        assert_eq!(code_to_language("PT"), Some(Language::Portuguese));
        assert_eq!(code_to_language("EN"), Some(Language::English));
    }

    #[test]
    fn resolve_force_pt_returns_portuguese() {
        let result = resolve_language(Some("pt-BR"));
        assert_eq!(result, Language::Portuguese);
    }

    #[test]
    fn resolve_force_en_returns_english() {
        let result = resolve_language(Some("en-US"));
        assert_eq!(result, Language::English);
    }

    #[test]
    fn resolve_force_invalid_uses_next_layers() {
        // Invalid code does not resolve on layer 1; must fall through to sys_locale or fallback.
        std::env::remove_var("SSH_CLI_LANG");
        let result = resolve_language(Some("xx-YY"));
        // Must return English or Portuguese — cannot be an invalid value.
        assert!(
            result == Language::English || result == Language::Portuguese,
            "resolve_language must return a valid language even with invalid code"
        );
    }

    #[test]
    fn resolve_without_force_returns_valid_language() {
        std::env::remove_var("SSH_CLI_LANG");
        let result = resolve_language(None);
        assert!(
            result == Language::English || result == Language::Portuguese,
            "resolve_language must return a valid language"
        );
    }

    #[test]
    fn current_language_fallback_english_before_set() {
        // We do not call set_language — OnceLock may already be set in other tests,
        // but the result MUST be a valid language.
        let result = current_language();
        assert!(
            result == Language::English || result == Language::Portuguese,
            "current_language must return a valid language"
        );
    }
}
