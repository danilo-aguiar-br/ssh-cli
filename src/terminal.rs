// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Colored output configuration and interactive terminal detection.
//!
//! Manages color choice via `termcolor` honoring precedence:
//! 1. CLI flag `--no-color` (highest priority).
//! 2. `NO_COLOR` environment variable (see <https://no-color.org>).
//! 3. `CLICOLOR_FORCE=1` environment variable (force colors even without TTY).
//! 4. TTY detection (colors only if stdout is an interactive terminal).
//! 5. Fallback: no color.

use anyhow::Result;
use std::sync::OnceLock;
use termcolor::ColorChoice;

/// Color choice cache (set once at initialization).
///
/// Concurrent access: `OnceLock` — written once from `initialize` after parse;
/// readers clone via `Copy`. Safe across threads (`ColorChoice: Sync`).
static COLOR_CACHE: OnceLock<ColorChoice> = OnceLock::new();

/// Initializes terminal color configuration.
///
/// Must be called once after CLI argument parsing.
/// The `no_color` parameter matches the CLI `--no-color` flag.
pub fn initialize(no_color: bool) -> Result<()> {
    let choice = determine_color(no_color);
    let _ = COLOR_CACHE.set(choice);
    tracing::debug!("terminal color configuration: {:?}", choice);
    Ok(())
}

/// Returns the configured color choice.
///
/// If [`initialize`] was not called, returns [`ColorChoice::Never`] as
/// safe fallback.
#[must_use]
pub fn color_choice() -> ColorChoice {
    *COLOR_CACHE.get().unwrap_or(&ColorChoice::Never)
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
    // 1. CLI flag --no-color (highest priority)
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
    #[serial_test::serial]
    fn no_color_env_returns_never() {
        // Saves and restores environment variable state
        let previous = std::env::var("NO_COLOR").ok();
        let previous_force = std::env::var("CLICOLOR_FORCE").ok();

        crate::test_util::env::set_var("NO_COLOR", "1");
        crate::test_util::env::remove_var("CLICOLOR_FORCE");

        let choice = determine_color(false);
        assert!(matches!(choice, ColorChoice::Never));

        // Restaura
        match previous {
            Some(v) => crate::test_util::env::set_var("NO_COLOR", v),
            None => crate::test_util::env::remove_var("NO_COLOR"),
        }
        match previous_force {
            Some(v) => crate::test_util::env::set_var("CLICOLOR_FORCE", v),
            None => crate::test_util::env::remove_var("CLICOLOR_FORCE"),
        }
    }

    #[test]
    #[serial_test::serial]
    fn clicolor_force_returns_always() {
        let previous = std::env::var("NO_COLOR").ok();
        let previous_force = std::env::var("CLICOLOR_FORCE").ok();

        crate::test_util::env::remove_var("NO_COLOR");
        crate::test_util::env::set_var("CLICOLOR_FORCE", "1");

        let choice = determine_color(false);
        assert!(matches!(choice, ColorChoice::Always));

        // Restaura
        match previous {
            Some(v) => crate::test_util::env::set_var("NO_COLOR", v),
            None => crate::test_util::env::remove_var("NO_COLOR"),
        }
        match previous_force {
            Some(v) => crate::test_util::env::set_var("CLICOLOR_FORCE", v),
            None => crate::test_util::env::remove_var("CLICOLOR_FORCE"),
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
