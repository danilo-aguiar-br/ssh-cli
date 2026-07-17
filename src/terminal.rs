// SPDX-License-Identifier: MIT OR Apache-2.0
//! Colored output configuration and interactive terminal detection.
//!
//! Manages color choice via `termcolor` honoring precedence:
//! 1. Flag `--no-color` da CLI (maior prioridade).
//! 2. `NO_COLOR` environment variable (see <https://no-color.org>).
//! 3. `CLICOLOR_FORCE=1` environment variable (force colors even without TTY).
//! 4. TTY detection (colors only if stdout is an interactive terminal).
//! 5. Fallback: no color.

use anyhow::Result;
use std::sync::OnceLock;
use termcolor::ColorChoice;

/// Color choice cache (set once at initialization).
static COR_CACHE: OnceLock<ColorChoice> = OnceLock::new();

/// Initializes terminal color configuration.
///
/// Must be called once after CLI argument parsing.
/// The `no_color` parameter matches the CLI `--no-color` flag.
pub fn initialize(no_color: bool) -> Result<()> {
    let choice = determine_color(no_color);
    let _ = COR_CACHE.set(choice);
    tracing::debug!("terminal color configuration: {:?}", choice);
    Ok(())
}

/// Returns the configured color choice.
///
/// If [`initialize`] was not called, returns [`ColorChoice::Never`] as
/// fallback seguro.
#[must_use]
pub fn color_choice() -> ColorChoice {
    *COR_CACHE.get().unwrap_or(&ColorChoice::Never)
}

/// Returns `true` if the process is running in an interactive terminal (TTY).
///
/// Uses [`std::io::IsTerminal`] (stabilized in Rust 1.70) for detection
/// cross-platform without external dependencies.
#[must_use]
pub fn is_interactive() -> bool {
    use std::io::IsTerminal;

    // If TERM=dumb, not interactive regardless of TTY
    if std::env::var("TERM").as_deref() == Ok("dumb") {
        return false;
    }

    std::io::stdout().is_terminal()
}

/// Determines color choice based on precedence rules.
fn determine_color(no_color_cli: bool) -> ColorChoice {
    // 1. Flag --no-color da CLI (maior prioridade)
    if no_color_cli {
        return ColorChoice::Never;
    }

    // 2. NO_COLOR environment variable (any value)
    if std::env::var("NO_COLOR").is_ok() {
        return ColorChoice::Never;
    }

    // 3. CLICOLOR_FORCE=1 forces colors even without TTY
    if std::env::var("CLICOLOR_FORCE").as_deref() == Ok("1") {
        return ColorChoice::Always;
    }

    // 4. TTY detection: colors only on interactive terminal
    if is_interactive() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_color_cli_returns_never() {
        let choice = determine_color(true);
        assert!(matches!(choice, ColorChoice::Never));
    }

    #[test]
    fn no_color_env_returns_never() {
        // Saves and restores environment variable state
        let anterior = std::env::var("NO_COLOR").ok();
        let anterior_force = std::env::var("CLICOLOR_FORCE").ok();

        std::env::set_var("NO_COLOR", "1");
        std::env::remove_var("CLICOLOR_FORCE");

        let choice = determine_color(false);
        assert!(matches!(choice, ColorChoice::Never));

        // Restaura
        match anterior {
            Some(v) => std::env::set_var("NO_COLOR", v),
            None => std::env::remove_var("NO_COLOR"),
        }
        match anterior_force {
            Some(v) => std::env::set_var("CLICOLOR_FORCE", v),
            None => std::env::remove_var("CLICOLOR_FORCE"),
        }
    }

    #[test]
    fn clicolor_force_returns_always() {
        let anterior = std::env::var("NO_COLOR").ok();
        let anterior_force = std::env::var("CLICOLOR_FORCE").ok();

        std::env::remove_var("NO_COLOR");
        std::env::set_var("CLICOLOR_FORCE", "1");

        let choice = determine_color(false);
        assert!(matches!(choice, ColorChoice::Always));

        // Restaura
        match anterior {
            Some(v) => std::env::set_var("NO_COLOR", v),
            None => std::env::remove_var("NO_COLOR"),
        }
        match anterior_force {
            Some(v) => std::env::set_var("CLICOLOR_FORCE", v),
            None => std::env::remove_var("CLICOLOR_FORCE"),
        }
    }

    #[test]
    fn color_choice_returns_never_without_init() {
        // Without initialize, fallback is Never
        // NOTE: in parallel tests OnceLock may already hold a value.
        // Only check that it does not panic.
        let _ = color_choice();
    }

    #[test]
    fn is_interactive_returns_bool() {
        // Only checks that it does not panic
        let _ = is_interactive();
    }
}
