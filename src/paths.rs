// SPDX-License-Identifier: MIT OR Apache-2.0
//! File path validation and normalization.
//!
//! Provides functions to validate file names safely and
//! cross-platform, prevenindo path traversal, nomes reservados do Windows
//! and forbidden characters.

use anyhow::{bail, Result};
use unicode_normalization::UnicodeNormalization;

/// Names reserved by the Windows file system (case-insensitive).
const NOMES_RESERVADOS_WINDOWS: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Characters forbidden in file names (forbidden on Windows or problematic
/// em sistemas Unix).
const CHARS_PROIBIDOS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];

/// Validates a file name (no path separators).
///
/// Rejeita:
/// - Strings vazias.
/// - Names with `..` components (path traversal).
/// - Forbidden characters.
/// - Windows reserved names (case-insensitive).
/// - Names ending with a dot or space (problematic on Windows).
///
/// # Examples
///
/// ```
/// use ssh_cli::paths::validate_name;
///
/// assert!(validate_name("meu-servidor").is_ok());
/// assert!(validate_name("../etc/passwd").is_err());
/// assert!(validate_name("CON").is_err());
/// ```
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("nome de file não pode ser vazio");
    }

    if name.contains("..") {
        bail!("nome de file contém componente de path traversal: '{name}'");
    }

    for c in CHARS_PROIBIDOS {
        if name.contains(*c) {
            bail!(
                "nome de file contém caractere proibido '{}': '{name}'",
                c.escape_default()
            );
        }
    }

    let name_upper = name.to_uppercase();
    // Also checks without extension (e.g. "NUL.txt" is forbidden on Windows)
    let raiz = name_upper.split('.').next().unwrap_or(&name_upper);
    if NOMES_RESERVADOS_WINDOWS.contains(&raiz) {
        bail!("nome de file usa nome reservado do Windows: '{name}'");
    }

    if name.ends_with('.') || name.ends_with(' ') {
        bail!("nome de file não pode terminar com ponto ou espaço: '{name}'");
    }

    Ok(())
}

/// Normalizes a file name to Unicode NFC form.
///
/// NFC normalization is required for consistent comparisons
/// entre diferentes sistemas operacionais (macOS usa NFD, Linux usa NFC).
#[must_use]
pub fn normalize_nfc(name: &str) -> String {
    name.nfc().collect()
}

/// Validates and normalizes a file name in one operation.
///
/// Returns the NFC-normalized name if all validations pass.
pub fn validate_and_normalize(name: &str) -> Result<String> {
    validate_name(name)?;
    Ok(normalize_nfc(name))
}

/// Validates that a path has no traversal components.
///
/// Checks all path segments separated by `/` or `\`.
pub fn validate_no_traversal(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("caminho não pode ser vazio");
    }

    let segmentos = path.split(['/', '\\']);
    for segmento in segmentos {
        if segmento == ".." {
            bail!("caminho contém componente de path traversal: '{path}'");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_valid_name_passes() {
        assert!(validate_name("meu-servidor").is_ok());
        assert!(validate_name("vps_01").is_ok());
        assert!(validate_name("servidor.produção").is_ok());
    }

    #[test]
    fn empty_name_rejected() {
        assert!(validate_name("").is_err());
    }

    #[test]
    fn path_traversal_rejected() {
        assert!(validate_name("..").is_err());
        assert!(validate_name("../etc/passwd").is_err());
        assert!(validate_name("foo/../bar").is_err());
    }

    #[test]
    fn forbidden_chars_rejected() {
        assert!(validate_name("foo/bar").is_err());
        assert!(validate_name("foo\\bar").is_err());
        assert!(validate_name("foo:bar").is_err());
        assert!(validate_name("foo*bar").is_err());
        assert!(validate_name("foo?bar").is_err());
    }

    #[test]
    fn windows_reserved_names_rejected() {
        assert!(validate_name("CON").is_err());
        assert!(validate_name("con").is_err());
        assert!(validate_name("NUL.txt").is_err());
        assert!(validate_name("COM1").is_err());
        assert!(validate_name("LPT9").is_err());
    }

    #[test]
    fn name_ending_with_dot_rejected() {
        assert!(validate_name("file.").is_err());
    }

    #[test]
    fn name_ending_with_space_rejected() {
        assert!(validate_name("file ").is_err());
    }

    #[test]
    fn normalize_nfc_returns_string() {
        let result = normalize_nfc("servidor");
        assert_eq!(result, "servidor");
    }

    #[test]
    fn validate_and_normalize_returns_valid_string() {
        let result = validate_and_normalize("meu-servidor").unwrap();
        assert_eq!(result, "meu-servidor");
    }

    #[test]
    fn validate_no_traversal_accepts_normal_path() {
        assert!(validate_no_traversal("/home/usuario/file.txt").is_ok());
        assert!(validate_no_traversal("relative/path/file.txt").is_ok());
    }

    #[test]
    fn validate_no_traversal_rejects_traversal() {
        assert!(validate_no_traversal("/home/../etc/passwd").is_err());
        assert!(validate_no_traversal("../secreto").is_err());
    }

    #[test]
    fn validate_no_traversal_rejects_empty() {
        assert!(validate_no_traversal("").is_err());
    }

    #[test]
    fn name_with_brazilian_accents_valid() {
        assert!(validate_name("produção").is_ok());
        assert!(validate_name("ação-configuração").is_ok());
    }

    #[test]
    fn name_with_cjk_unicode_valid() {
        assert!(validate_name("server-\u{4e16}\u{754c}").is_ok());
    }

    #[test]
    fn name_with_emoji_valid() {
        assert!(validate_name("server-\u{1f680}").is_ok());
    }

    #[test]
    fn windows_reserved_mixed_case_rejected() {
        assert!(validate_name("cOn").is_err());
        assert!(validate_name("Nul").is_err());
        assert!(validate_name("lPt1").is_err());
    }

    #[test]
    fn normalize_nfc_converts_nfd_to_nfc() {
        let nfd = "e\u{0301}"; // e + combining acute
        let nfc = "\u{00e9}"; // é precomposed
        assert_eq!(normalize_nfc(nfd), nfc);
    }

    #[test]
    fn normalize_nfc_preserves_nfc() {
        let nfc = "\u{00e9}";
        assert_eq!(normalize_nfc(nfc), nfc);
    }

    #[test]
    fn normalize_nfc_idempotent() {
        let input = "cafe\u{0301}";
        let once = normalize_nfc(input);
        let twice = normalize_nfc(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn validate_and_normalize_converts_nfd() {
        let result = validate_and_normalize("cafe\u{0301}").unwrap();
        assert_eq!(result, "caf\u{00e9}");
    }

    #[test]
    fn validate_no_traversal_rejects_backslash() {
        assert!(validate_no_traversal("foo\\..\\bar").is_err());
    }

    #[test]
    fn validate_no_traversal_accepts_dot_alone() {
        assert!(validate_no_traversal("./file").is_ok());
    }
}
