//! `ssh-cli locale [show|set|clear]` handler — local preference file and diagnostics.
#![forbid(unsafe_code)]

use super::{LocaleAction, OutputFormat};
use anyhow::Result;

/// Implements `ssh-cli locale [show|set|clear]`.
pub(crate) fn run_locale_command(
    action: Option<LocaleAction>,
    config_override: Option<&std::path::Path>,
    formato: OutputFormat,
    force_lang: Option<&str>,
    json_flag: bool,
) -> Result<()> {
    use crate::i18n::{self, Message};
    use crate::locale::{
        clear_persisted_lang, current_language, lang_preference_path, negotiate_code,
        resolve_language_detailed, write_persisted_lang,
    };

    let action = action.unwrap_or(LocaleAction::Show);
    let override_ref = config_override;
    let wants_json = json_flag || formato == OutputFormat::Json;

    match action {
        LocaleAction::Show => {
            // Re-resolve for diagnostics (global already set at init; layers still useful).
            let detailed = resolve_language_detailed(force_lang, override_ref);
            let available: Vec<&str> = crate::i18n::Language::AVAILABLE
                .iter()
                .map(|l| l.bcp47())
                .collect();
            let pref_path = lang_preference_path(override_ref)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| String::from("(unavailable)"));

            if wants_json {
                let body = serde_json::json!({
                    "resolved": detailed.language.bcp47(),
                    "current": current_language().bcp47(),
                    "source": detailed.source.as_str(),
                    "available": available,
                    "system_raw": detailed.system_raw,
                    "persisted_raw": detailed.persisted_raw,
                    "preference_path": pref_path,
                    "direction": match detailed.language.direction() {
                        crate::i18n::TextDirection::Ltr => "ltr",
                        crate::i18n::TextDirection::Rtl => "rtl",
                    },
                    "script": detailed.language.script(),
                });
                crate::output::print_json_value(&body)?;
            } else {
                crate::output::write_line(&i18n::t(Message::LocaleStatusTitle))?;
                crate::output::write_line_fmt(format_args!(
                    "  resolved:   {} ({})",
                    detailed.language.bcp47(),
                    detailed.source.as_str()
                ))?;
                crate::output::write_line_fmt(format_args!(
                    "  current:    {}",
                    current_language().bcp47()
                ))?;
                crate::output::write_line_fmt(format_args!(
                    "  available:  {}",
                    available.join(", ")
                ))?;
                crate::output::write_line_fmt(format_args!(
                    "  system:     {}",
                    detailed.system_raw.as_deref().unwrap_or("(none)")
                ))?;
                crate::output::write_line_fmt(format_args!(
                    "  persisted:  {}",
                    detailed.persisted_raw.as_deref().unwrap_or("(none)")
                ))?;
                crate::output::write_line_fmt(format_args!("  pref_path:  {pref_path}"))?;
            }
            Ok(())
        }
        LocaleAction::Set { lang } => {
            let language = negotiate_code(&lang)
                .ok_or_else(|| anyhow::anyhow!("unsupported language after validation: {lang}"))?;
            let path = write_persisted_lang(language, override_ref)?;
            // Note: OnceLock already set for this process; preference applies next run
            // unless --lang/env override.
            if wants_json {
                crate::output::print_json_value(&serde_json::json!({
                    "ok": true,
                    "lang": language.bcp47(),
                    "path": path.display().to_string(),
                    "applies": "next_invocation_unless_overridden",
                }))?;
            } else {
                crate::output::try_print_success(&i18n::t(Message::LocalePreferenceSaved {
                    lang: language.bcp47().to_string(),
                    path: path.display().to_string(),
                }))?;
            }
            Ok(())
        }
        LocaleAction::Clear => {
            clear_persisted_lang(override_ref)?;
            if wants_json {
                crate::output::print_json_value(&serde_json::json!({
                    "ok": true,
                    "cleared": true,
                }))?;
            } else {
                crate::output::try_print_success(&i18n::t(Message::LocalePreferenceCleared))?;
            }
            Ok(())
        }
    }
}
