// SPDX-License-Identifier: MIT OR Apache-2.0
//! Operating-system conditional abstractions.
//!
//! Platform initialization ([`initialize_platform`]) is the FIRST operation
//! executada no `main()`. Ela configura:
//!
//! - **Windows**: codepage UTF-8 (65001) via `SetConsoleOutputCP` e `SetConsoleCP`
//! - **Linux**: sandbox detection (Flatpak/Snap) and XDG paths
//! - **macOS**: config path resolution under `~/Library/Application Support`

use anyhow::Result;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Initializes the platform before any I/O.
///
/// MUST be called as the first operation in `main()`.
pub fn initialize_platform() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        windows::configure_utf8_codepage()?;
    }

    #[cfg(target_os = "linux")]
    {
        linux::detectar_sandbox();
    }

    #[cfg(target_os = "macos")]
    {
        macos::initialize();
    }

    Ok(())
}

/// Normalizes a stdin line by stripping trailing `\r` (CRLF → LF).
///
/// Required on Windows where pipes may emit `\r\n`.
#[must_use]
pub fn normalize_stdin_line(line: &str) -> &str {
    // Strip any trailing CR/LF combination.
    line.trim_end_matches(['\r', '\n'])
}

/// Returns `true` if stdout is connected to a terminal (TTY).
#[must_use]
pub fn e_tty() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_trailing_cr() {
        assert_eq!(normalize_stdin_line("teste\r"), "teste");
        assert_eq!(normalize_stdin_line("teste\r\n"), "teste");
        assert_eq!(normalize_stdin_line("teste\n"), "teste");
        assert_eq!(normalize_stdin_line("teste"), "teste");
    }

    #[test]
    fn normalize_empty_string() {
        assert_eq!(normalize_stdin_line(""), "");
    }

    #[test]
    fn normalize_newlines_only() {
        assert_eq!(normalize_stdin_line("\n\n\n"), "");
    }

    #[test]
    fn normalize_mixed_crlf_lf() {
        assert_eq!(
            normalize_stdin_line("linha1\r\nlinha2\r\nlinha3"),
            "linha1\r\nlinha2\r\nlinha3"
        );
    }

    #[test]
    fn normalize_with_spaces() {
        assert_eq!(
            normalize_stdin_line("texto com espacos  \r\n"),
            "texto com espacos  "
        );
    }

    #[test]
    fn is_tty_returns_bool() {
        let _ = e_tty();
    }
}
