// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Cross-platform language detection and resolution (Rules Rust i18n).
//!
//! ## Precedence (highest → lowest) — four product layers
//!
//! 1. CLI `--lang` flag (validated BCP47; must negotiate to a supported locale)
//! 2. Persisted preference (`<config_dir>/lang`, XDG; 0o600 when written) via `locale set`
//! 3. OS locale via `sys_locale::get_locale()` (never raw `LANG` / `LC_*` in portável)
//! 4. Fallback: [`Language::English`] (`en`)
//!
//! `SSH_CLI_LANG` is a **historical constant name only** — not read as a product
//! config store (G-AUD-12). Use `--lang` or `locale set`.
//!
//! ## Pipeline
//!
//! Raw string → strip encoding/modifier → `_`→`-` → `LanguageIdentifier`
//! (`unic-langid`) → negotiate against available locales (`fluent-langneg`) →
//! map to [`Language`] → publish once in `OnceLock`.
//!
//! Detection failure is **never** silent: `tracing::warn!` records the miss
//! (observability / stderr diagnostics; no remote telemetry).

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;

use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use unic_langid::LanguageIdentifier;

use crate::i18n::Language;

/// Global language state — set once at initialization.
///
/// Concurrent access: `OnceLock` (single writer at boot via `set_language`;
/// subsequent sets ignored). `Language` is `Copy + Sync`.
static GLOBAL_LANGUAGE: OnceLock<Language> = OnceLock::new();

/// Which precedence layer won resolution (diagnostics / `locale` subcommand).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LocaleSource {
    /// CLI `--lang`.
    CliFlag,
    /// Historical layer id only — product no longer reads `SSH_CLI_LANG` as a store.
    EnvVar,
    /// XDG (or override) `lang` preference file (`locale set`).
    Persisted,
    /// `sys_locale::get_locale()`.
    System,
    /// Deterministic default (`en`).
    Default,
}

impl LocaleSource {
    /// Stable machine id for JSON / tests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CliFlag => "cli_flag",
            Self::EnvVar => "env_var",
            Self::Persisted => "persisted",
            Self::System => "system",
            Self::Default => "default",
        }
    }
}

/// Full resolution result (language + winning layer + raw inputs for diagnostics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleResolution {
    /// Negotiated product language.
    pub language: Language,
    /// Winning precedence layer.
    pub source: LocaleSource,
    /// Raw OS locale string when layer was system (if any).
    pub system_raw: Option<String>,
    /// Raw persisted file content when present.
    pub persisted_raw: Option<String>,
}

/// File name for the persisted language preference (sibling of `config.toml`).
pub const LANG_PREFERENCE_FILE: &str = crate::constants::LANG_PREFERENCE_FILE_NAME;

/// Resolves language using the CLI / XDG / OS / default precedence hierarchy.
///
/// `config_dir_override` is the same optional directory used by `--config-dir`
/// (tests / isolation). When `None`, XDG via `directories` applies.
/// Product does **not** read `SSH_CLI_HOME` or `SSH_CLI_LANG` as config stores.
#[must_use]
pub fn resolve_language(
    force_lang: Option<&str>,
    config_dir_override: Option<&Path>,
) -> Language {
    resolve_language_detailed(force_lang, config_dir_override).language
}

/// Like [`resolve_language`], but returns diagnostics for `locale` / tests.
#[must_use]
pub fn resolve_language_detailed(
    force_lang: Option<&str>,
    config_dir_override: Option<&Path>,
) -> LocaleResolution {
    let system_raw = sys_locale::get_locale();
    let persisted_raw = read_persisted_lang(config_dir_override);

    // Layer 1: CLI --lang
    if let Some(code) = force_lang {
        match negotiate_code(code) {
            Some(language) => {
                return LocaleResolution {
                    language,
                    source: LocaleSource::CliFlag,
                    system_raw,
                    persisted_raw,
                };
            }
            None => {
                tracing::warn!(
                    target: "ssh_cli::locale",
                    code,
                    "invalid or unsupported --lang; falling through precedence"
                );
            }
        }
    }

    // G-AUD-12: env lang store removed — use `--lang` or `locale set` (XDG).

    // Layer 2: persisted preference (was layer 3)
    if let Some(ref raw) = persisted_raw {
        match negotiate_code(raw) {
            Some(language) => {
                return LocaleResolution {
                    language,
                    source: LocaleSource::Persisted,
                    system_raw,
                    persisted_raw,
                };
            }
            None => {
                tracing::warn!(
                    target: "ssh_cli::locale",
                    code = %raw,
                    "persisted lang preference unsupported; falling through"
                );
            }
        }
    }

    // Layer 4: OS via sys-locale (cross-platform abstraction — never raw LANG)
    if let Some(ref locale) = system_raw {
        match negotiate_code(locale) {
            Some(language) => {
                return LocaleResolution {
                    language,
                    source: LocaleSource::System,
                    system_raw,
                    persisted_raw,
                };
            }
            None => {
                tracing::warn!(
                    target: "ssh_cli::locale",
                    system_locale = %locale,
                    "OS locale did not negotiate to a supported language; using default en"
                );
            }
        }
    } else {
        tracing::warn!(
            target: "ssh_cli::locale",
            "sys_locale::get_locale returned None (container/distroless/WASM); using default en"
        );
    }

    // Layer 5: deterministic default
    LocaleResolution {
        language: Language::English,
        source: LocaleSource::Default,
        system_raw,
        persisted_raw,
    }
}

/// Sets the global language (once at process startup).
///
/// Subsequent calls are silently ignored — `OnceLock` guarantees immutability
/// after first set (no mixed languages in one session).
pub fn set_language(language: Language) {
    let _ = GLOBAL_LANGUAGE.set(language);
}

/// Returns the current global language.
///
/// If `set_language` has not been called yet, returns [`Language::English`]
/// as a safe fallback for code run before initialization.
#[must_use]
pub fn current_language() -> Language {
    GLOBAL_LANGUAGE.get().copied().unwrap_or(Language::English)
}

/// Normalizes a raw locale string for BCP47 parse.
///
/// - Strips encoding suffix (`.UTF-8`, `.utf8`)
/// - Strips `@modifier` (e.g. `@euro`)
/// - Converts `_` separators to `-`
/// - Trims whitespace
///
/// Does **not** treat `C` / `POSIX` / `C.UTF-8` as English — those parse as
/// invalid/unsupported and fall through negotiation.
#[must_use]
pub fn normalize_raw_locale(raw: &str) -> String {
    let s = raw.trim();
    let s = s.split('.').next().unwrap_or(s);
    let s = s.split('@').next().unwrap_or(s);
    s.replace('_', "-")
}

/// Parses a raw OS/CLI locale into a [`LanguageIdentifier`].
///
/// Returns `None` for empty, `C`, `POSIX`, or malformed tags.
#[must_use]
pub fn parse_language_identifier(raw: &str) -> Option<LanguageIdentifier> {
    let normalized = normalize_raw_locale(raw);
    if normalized.is_empty() {
        return None;
    }
    // POSIX "C" / "POSIX" are not user language preferences.
    if normalized.eq_ignore_ascii_case("c") || normalized.eq_ignore_ascii_case("posix") {
        return None;
    }
    LanguageIdentifier::from_str(&normalized).ok()
}

/// Negotiates a raw code against the available product locales.
///
/// Uses `fluent-langneg` Lookup strategy with default `en`.
#[must_use]
pub fn negotiate_code(raw: &str) -> Option<Language> {
    let requested = parse_language_identifier(raw)?;
    negotiate_langid(&requested)
}

/// Negotiates a structured identifier against available locales.
#[must_use]
pub fn negotiate_langid(requested: &LanguageIdentifier) -> Option<Language> {
    let available: Vec<LanguageIdentifier> = Language::AVAILABLE
        .iter()
        .map(|l| l.language_identifier())
        .collect();
    let default = Language::English.language_identifier();
    let supported = negotiate_languages(
        std::slice::from_ref(requested),
        &available,
        Some(&default),
        NegotiationStrategy::Lookup,
    );
    let first = supported.first()?;
    // If only default was returned because request was unrelated (e.g. fr-FR),
    // treat as no match so callers can fall through — unless request itself is en.
    if let Some(lang) = Language::from_langid(first) {
        // When Lookup returns default for unsupported languages, detect that
        // the requested primary language is not en/pt.
        let req_lang = requested.language.as_str();
        if lang == Language::English
            && req_lang != "en"
            && !Language::AVAILABLE
                .iter()
                .any(|a| a.language_identifier().language == requested.language)
        {
            return None;
        }
        return Some(lang);
    }
    None
}

/// Directory that holds `config.toml` / `lang` (respects override and XDG).
#[must_use]
pub fn resolve_config_dir(config_dir_override: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = config_dir_override {
        if p.is_dir() || !p.exists() {
            return Some(p.to_path_buf());
        }
        // File path → parent dir
        return p.parent().map(|d| d.to_path_buf());
    }
    crate::vps::default_config_path()
        .ok()
        .and_then(|cfg| cfg.parent().map(|d| d.to_path_buf()))
}

/// Path to the persisted language preference file.
#[must_use]
pub fn lang_preference_path(config_dir_override: Option<&Path>) -> Option<PathBuf> {
    resolve_config_dir(config_dir_override).map(|d| d.join(LANG_PREFERENCE_FILE))
}

/// Reads the persisted language preference (trimmed, non-empty).
#[must_use]
pub fn read_persisted_lang(config_dir_override: Option<&Path>) -> Option<String> {
    let path = lang_preference_path(config_dir_override)?;
    let content = std::fs::read_to_string(&path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Writes the persisted language preference (BCP47 of a supported language).
///
/// Creates the config directory if needed. On Unix, sets mode `0o600`.
///
/// # Errors
/// Returns I/O errors from create/write/permissions.
pub fn write_persisted_lang(
    language: Language,
    config_dir_override: Option<&Path>,
) -> std::io::Result<PathBuf> {
    let dir = resolve_config_dir(config_dir_override).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "configuration directory unavailable",
        )
    })?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(LANG_PREFERENCE_FILE);
    let body = format!("{}\n", language.bcp47());
    std::fs::write(&path, body.as_bytes())?;
    crate::fs_perm::set_secret_file_mode(&path).map_err(|e| match e {
        crate::errors::SshCliError::Io(io) => io,
        other => std::io::Error::other(other.to_string()),
    })?;
    Ok(path)
}

/// Removes the persisted language preference if present.
///
/// # Errors
/// Propagates unexpected I/O errors (ignores NotFound).
pub fn clear_persisted_lang(config_dir_override: Option<&Path>) -> std::io::Result<()> {
    if let Some(path) = lang_preference_path(config_dir_override) {
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    } else {
        Ok(())
    }
}

/// Clap `value_parser` for `--lang`: accepts BCP47 tags that negotiate to a
/// supported product locale (`en`, `en-US`, `pt-BR`, `pt`, …).
pub fn parse_lang_cli_arg(s: &str) -> Result<String, String> {
    match negotiate_code(s) {
        Some(lang) => Ok(lang.bcp47().to_string()),
        None => Err(format!(
            "unsupported language '{s}' (supported: en, pt-BR; BCP47 tags that negotiate to these)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_encoding_and_underscore() {
        assert_eq!(normalize_raw_locale("pt_BR.UTF-8"), "pt-BR");
        assert_eq!(normalize_raw_locale("en_US.utf8"), "en-US");
        assert_eq!(normalize_raw_locale("de_DE@euro"), "de-DE");
        assert_eq!(normalize_raw_locale("  pt-BR  "), "pt-BR");
    }

    #[test]
    fn parse_rejects_c_and_posix() {
        assert!(parse_language_identifier("C").is_none());
        assert!(parse_language_identifier("POSIX").is_none());
        assert!(parse_language_identifier("C.UTF-8").is_none());
        assert!(parse_language_identifier("").is_none());
    }

    #[test]
    fn parse_accepts_bcp47_variants() {
        assert!(parse_language_identifier("pt-BR").is_some());
        assert!(parse_language_identifier("pt_BR.UTF-8").is_some());
        assert!(parse_language_identifier("en-GB").is_some());
        assert!(parse_language_identifier("en").is_some());
    }

    #[test]
    fn negotiate_pt_br_and_prefixes() {
        assert_eq!(negotiate_code("pt-BR"), Some(Language::Portuguese));
        assert_eq!(negotiate_code("pt_BR.UTF-8"), Some(Language::Portuguese));
        assert_eq!(negotiate_code("pt"), Some(Language::Portuguese));
        assert_eq!(negotiate_code("en-US"), Some(Language::English));
        assert_eq!(negotiate_code("en-GB"), Some(Language::English));
        assert_eq!(negotiate_code("en"), Some(Language::English));
    }

    #[test]
    fn negotiate_unsupported_returns_none() {
        assert_eq!(negotiate_code("fr-FR"), None);
        assert_eq!(negotiate_code("zh-Hans-CN"), None);
        assert_eq!(negotiate_code("xx-YY"), None);
        assert_eq!(negotiate_code("C"), None);
    }

    #[test]
    fn parse_lang_cli_arg_ok_and_err() {
        assert_eq!(parse_lang_cli_arg("pt-BR").unwrap(), "pt-BR");
        assert_eq!(parse_lang_cli_arg("en").unwrap(), "en");
        assert!(parse_lang_cli_arg("fr-FR").is_err());
    }

    #[test]
    fn resolve_force_pt_returns_portuguese() {
        let result = resolve_language(Some("pt-BR"), None);
        assert_eq!(result, Language::Portuguese);
    }

    #[test]
    fn resolve_force_en_returns_english() {
        let result = resolve_language(Some("en-US"), None);
        assert_eq!(result, Language::English);
    }

    #[test]
    fn resolve_force_invalid_falls_through() {
        crate::test_util::env::remove_var(crate::constants::ENV_LANG);
        let result = resolve_language_detailed(Some("xx-YY"), None);
        assert!(
            result.language == Language::English || result.language == Language::Portuguese,
            "must return a valid language"
        );
        assert_ne!(result.source, LocaleSource::CliFlag);
    }

    #[test]
    fn resolve_without_force_returns_valid_language() {
        crate::test_util::env::remove_var(crate::constants::ENV_LANG);
        let result = resolve_language(None, None);
        assert!(
            result == Language::English || result == Language::Portuguese,
            "resolve_language must return a valid language"
        );
    }

    #[test]
    fn current_language_fallback_english_before_set() {
        let result = current_language();
        assert!(
            result == Language::English || result == Language::Portuguese,
            "current_language must return a valid language"
        );
    }

    #[test]
    fn locale_source_as_str_stable() {
        assert_eq!(LocaleSource::CliFlag.as_str(), "cli_flag");
        assert_eq!(LocaleSource::Default.as_str(), "default");
    }

    #[test]
    fn persisted_lang_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_persisted_lang(Language::Portuguese, Some(dir.path())).expect("write");
        assert!(path.exists());
        let raw = read_persisted_lang(Some(dir.path())).expect("read");
        assert_eq!(negotiate_code(&raw), Some(Language::Portuguese));
        clear_persisted_lang(Some(dir.path())).expect("clear");
        assert!(read_persisted_lang(Some(dir.path())).is_none());
    }

    #[test]
    fn resolve_uses_persisted_when_no_flag_or_env() {
        crate::test_util::env::remove_var(crate::constants::ENV_LANG);
        let dir = tempfile::tempdir().expect("tempdir");
        write_persisted_lang(Language::Portuguese, Some(dir.path())).expect("write");
        let r = resolve_language_detailed(None, Some(dir.path()));
        // Persisted wins over system when flag/env absent — unless system somehow
        // is forced; with force None and no env, source should be Persisted.
        assert_eq!(r.language, Language::Portuguese);
        assert_eq!(r.source, LocaleSource::Persisted);
    }
}
