// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! File path validation and normalization.
//!
//! Cross-platform guards against path traversal, Windows reserved device
//! names, forbidden characters, Unicode NFC drift (macOS NFD vs Linux NFC),
//! and legacy Windows `MAX_PATH` (260) limits without the `\\?\` prefix.

use crate::errors::{SshCliError, SshCliResult};
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

#[inline]
fn path_err(msg: impl Into<String>) -> SshCliError {
    SshCliError::InvalidArgument(msg.into())
}

/// Names reserved by the Windows file system (case-insensitive).
const WINDOWS_RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Characters forbidden in file names (Windows-illegal or shell-hostile on Unix).
const FORBIDDEN_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];

/// Legacy Windows `MAX_PATH` including the trailing NUL (Win32 default without long-path prefix).
pub const WINDOWS_MAX_PATH: usize = 260;

/// Maximum length of a single path component on Windows (excluding separators).
pub const WINDOWS_MAX_COMPONENT: usize = 255;

const _: () = assert!(!WINDOWS_RESERVED_NAMES.is_empty());
const _: () = assert!(!FORBIDDEN_CHARS.is_empty());
const _: () = assert!(WINDOWS_MAX_PATH > WINDOWS_MAX_COMPONENT);

/// Validates a file name (no path separators).
///
/// Rejects:
/// - Empty strings
/// - Names with `..` components (path traversal)
/// - Forbidden characters
/// - Windows reserved names (case-insensitive)
/// - Names ending with a dot or space (problematic on Windows)
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
///
/// # Errors
/// Returns [`SshCliError::InvalidArgument`] if the name is empty, contains
/// traversal/forbidden characters/whitespace, or is Windows-reserved.
pub fn validate_name(name: &str) -> SshCliResult<()> {
    if name.is_empty() {
        return Err(path_err("file name cannot be empty"));
    }

    if name.contains("..") {
        return Err(path_err(format!(
            "file name contains path traversal component: '{name}'"
        )));
    }

    for c in FORBIDDEN_CHARS {
        if name.contains(*c) {
            return Err(path_err(format!(
                "file name contains forbidden character '{}': '{name}'",
                c.escape_default()
            )));
        }
    }

    let name_upper = name.to_uppercase();
    // Also checks without extension (e.g. "NUL.txt" is forbidden on Windows)
    let root = name_upper.split('.').next().unwrap_or(&name_upper);
    if WINDOWS_RESERVED_NAMES.contains(&root) {
        return Err(path_err(format!(
            "file name uses a Windows reserved name: '{name}'"
        )));
    }

    if name.ends_with('.') || name.ends_with(' ') {
        return Err(path_err(format!(
            "file name cannot end with a dot or space: '{name}'"
        )));
    }

    // GAP-AUD-VAL-001: reject any internal whitespace (spaces/tabs) so VPS registry
    // keys stay shell/TOML/agent-safe single tokens.
    if name.chars().any(|c| c.is_whitespace()) {
        return Err(path_err(format!(
            "file name cannot contain whitespace: '{name}'"
        )));
    }

    Ok(())
}

/// Normalizes a file name to Unicode NFC form.
///
/// NFC normalization is required for consistent comparisons across OSes
/// (macOS often stores NFD; Linux typically uses NFC).
///
/// # Examples
///
/// ```
/// use ssh_cli::paths::normalize_nfc;
///
/// let nfc = normalize_nfc("cafe");
/// assert_eq!(nfc, "cafe");
/// assert_eq!(normalize_nfc(&nfc), nfc); // idempotent
/// ```
#[must_use]
pub fn normalize_nfc(name: &str) -> String {
    name.nfc().collect()
}

/// Validates and normalizes a file name in one operation.
///
/// Returns the NFC-normalized name if all validations pass.
///
/// # Examples
///
/// ```
/// use ssh_cli::paths::validate_and_normalize;
///
/// let name = validate_and_normalize("lab-01").unwrap();
/// assert_eq!(name.as_str(), "lab-01");
/// assert!(validate_and_normalize("../etc").is_err());
/// ```
///
/// # Errors
/// Returns [`SshCliError::Domain`] / [`SshCliError::InvalidArgument`] if
/// [`validate_name`] / [`crate::domain::VpsName::try_new`] fails.
pub fn validate_and_normalize(name: &str) -> SshCliResult<crate::domain::VpsName> {
    // G-TYPE-08: return refined type (proof not discarded as bare String).
    // DomainError maps via From → SshCliError::Domain (G-ERR-02).
    Ok(crate::domain::VpsName::try_new(name)?)
}

/// Validates that a path has no traversal components.
///
/// Checks all path segments separated by `/` or `\`.
///
/// # Examples
///
/// ```
/// use ssh_cli::paths::validate_no_traversal;
///
/// assert!(validate_no_traversal("/tmp/file.bin").is_ok());
/// assert!(validate_no_traversal("a/../../etc/passwd").is_err());
/// assert!(validate_no_traversal("").is_err());
/// ```
///
/// # Errors
/// Empty path or any `..` segment → [`SshCliError::InvalidArgument`].
pub fn validate_no_traversal(path: &str) -> SshCliResult<()> {
    if path.is_empty() {
        return Err(path_err("path cannot be empty"));
    }

    let segments = path.split(['/', '\\']);
    for segment in segments {
        if segment == ".." {
            return Err(path_err(format!(
                "path contains path traversal component: '{path}'"
            )));
        }
    }

    Ok(())
}

/// Default cap for primary-key files (hex is 64 chars; allow whitespace/BOM).
pub const MAX_SECRETS_KEY_FILE_BYTES: u64 = 4_096;

/// Cap for `config.toml` (local agent registry — not a multi-tenant store).
pub const MAX_CONFIG_TOML_BYTES: u64 = 4 * 1024 * 1024;

/// Cap for TOFU `known_hosts` text file.
pub const MAX_KNOWN_HOSTS_BYTES: u64 = 1_024 * 1024;

const _: () = assert!(MAX_SECRETS_KEY_FILE_BYTES >= 64);
const _: () = assert!(MAX_CONFIG_TOML_BYTES >= 4_096);
const _: () = assert!(MAX_KNOWN_HOSTS_BYTES >= 256);

/// Resolves the XDG config directory for [`crate::constants::APP_NAME`].
///
/// Uses `directories::ProjectDirs` (Linux: `$XDG_CONFIG_HOME/ssh-cli` or
/// `~/.config/ssh-cli`). Does **not** honor `SSH_CLI_HOME` or `--config-dir`
/// — callers layer those overrides themselves.
///
/// # Errors
/// Returns [`SshCliError::XdgDirectory`] when the home/config root cannot be
/// determined (rare headless environments) — G-ERR-03.
pub fn xdg_config_dir() -> SshCliResult<std::path::PathBuf> {
    directories::ProjectDirs::from(
        crate::constants::PROJECT_QUALIFIER,
        crate::constants::PROJECT_ORGANIZATION,
        crate::constants::APP_NAME,
    )
    .map(|d| d.config_dir().to_path_buf())
    .ok_or(SshCliError::XdgDirectory)
}

/// Returns `true` when `path` uses the Windows extended-length prefix (`\\?\` or `//?/`).
#[must_use]
pub fn has_windows_long_path_prefix(path: &Path) -> bool {
    let s = path.as_os_str().to_string_lossy();
    s.starts_with(r"\\?\") || s.starts_with("//?/")
}

/// Validates a local filesystem path against Windows legacy length limits.
///
/// On **all** platforms this checks component length (≤255) so config written
/// on Linux remains openable if copied to Windows. On **Windows** (or when
/// `force_windows_rules` is true in tests), rejects total path length ≥
/// [`WINDOWS_MAX_PATH`] unless the path already uses the `\\?\` long-path
/// prefix.
///
/// Remote SCP paths are **not** validated here (remote FS is Unix-like).
///
/// # Errors
/// Returns [`SshCliError::InvalidArgument`] when a component or the full path
/// exceeds platform limits.
pub fn validate_local_path_length(path: &Path) -> SshCliResult<()> {
    validate_local_path_length_inner(path, cfg!(windows))
}

fn validate_local_path_length_inner(path: &Path, enforce_windows_total: bool) -> SshCliResult<()> {
    // Split on both separators so Windows-style paths are validated even when
    // this code runs on Unix hosts (CI, cross-compile checks, agent sandboxes).
    let raw = path.as_os_str().to_string_lossy();
    let stripped = raw
        .strip_prefix(r"\\?\")
        .or_else(|| raw.strip_prefix("//?/"))
        .unwrap_or(raw.as_ref());
    for segment in stripped.split(['/', '\\']) {
        if segment.is_empty() || segment == "." || segment == ".." {
            continue;
        }
        // Drive letter ("C:") is not subject to the 255-byte file-name limit.
        if segment.len() == 2 && segment.as_bytes()[1] == b':' {
            continue;
        }
        if segment.len() > WINDOWS_MAX_COMPONENT {
            return Err(path_err(format!(
                "path component exceeds {WINDOWS_MAX_COMPONENT} bytes (Windows limit): '{segment}'"
            )));
        }
    }

    if enforce_windows_total && !has_windows_long_path_prefix(path) {
        // Lossy UTF-8 length as a conservative Win32 MAX_PATH estimate.
        let encoded_len = raw.len();
        // Win32 MAX_PATH counts the trailing NUL; reject at 259 visible chars.
        if encoded_len >= WINDOWS_MAX_PATH - 1 {
            return Err(path_err(format!(
                "local path length {encoded_len} approaches Windows MAX_PATH ({WINDOWS_MAX_PATH}); \
                 use a shorter path or the \\\\?\\ extended-length prefix"
            )));
        }
    }

    Ok(())
}

/// Reads a UTF-8 text file with a hard byte cap (memory / OOM hygiene).
///
/// Checks `metadata().len()` first, then reads with `Take(max+1)` so a TOCTOU
/// grow cannot allocate unbounded heap. Rejects files larger than `max_bytes`.
///
/// # Errors
/// Returns [`std::io::Error`] on I/O failure or when the file exceeds `max_bytes`.
pub fn read_text_capped(path: &std::path::Path, max_bytes: u64) -> std::io::Result<String> {
    use std::io::Read;

    let meta = std::fs::metadata(path)?;
    if meta.len() > max_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "file {} exceeds max size of {max_bytes} bytes",
                path.display()
            ),
        ));
    }

    let file = std::fs::File::open(path)?;
    let mut limited = file.take(max_bytes.saturating_add(1));
    let mut buf = String::new();
    limited.read_to_string(&mut buf)?;
    if (buf.len() as u64) > max_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "file {} exceeds max size of {max_bytes} bytes",
                path.display()
            ),
        ));
    }
    Ok(buf)
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
    fn name_with_internal_space_rejected() {
        assert!(validate_name("a b").is_err());
        assert!(validate_name("a\tb").is_err());
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
        assert_eq!(result.as_str(), "meu-servidor");
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
        assert_eq!(result.as_str(), "caf\u{00e9}");
    }

    #[test]
    fn validate_no_traversal_rejects_backslash() {
        assert!(validate_no_traversal("foo\\..\\bar").is_err());
    }

    #[test]
    fn validate_no_traversal_accepts_dot_alone() {
        assert!(validate_no_traversal("./file").is_ok());
    }

    #[test]
    fn long_path_prefix_detected() {
        assert!(has_windows_long_path_prefix(Path::new(r"\\?\C:\very\long")));
        assert!(has_windows_long_path_prefix(Path::new("//?/C:/very/long")));
        assert!(!has_windows_long_path_prefix(Path::new(r"C:\short")));
    }

    #[test]
    fn component_over_255_rejected() {
        let long = "a".repeat(WINDOWS_MAX_COMPONENT + 1);
        let p = Path::new(&long);
        let err = validate_local_path_length_inner(p, false).unwrap_err();
        assert!(err.to_string().contains("255"));
    }

    #[test]
    fn windows_total_path_limit_enforced() {
        // Build a path whose lossy length is >= 259 without long-path prefix.
        let mut s = String::from("C:");
        while s.len() < WINDOWS_MAX_PATH - 1 {
            s.push_str("\\seg");
        }
        let p = Path::new(&s);
        assert!(validate_local_path_length_inner(p, true).is_err());
        let extended = format!(r"\\?\{s}");
        assert!(validate_local_path_length_inner(Path::new(&extended), true).is_ok());
    }

    #[test]
    fn short_local_path_ok() {
        assert!(validate_local_path_length(Path::new("/home/user/.config/ssh-cli/config.toml")).is_ok());
    }

    #[test]
    fn read_text_capped_accepts_small_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("k.txt");
        std::fs::write(&p, "abc").unwrap();
        let s = read_text_capped(&p, 64).unwrap();
        assert_eq!(s, "abc");
    }

    #[test]
    fn read_text_capped_rejects_oversize() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("big.txt");
        std::fs::write(&p, "0123456789").unwrap();
        let err = read_text_capped(&p, 4).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

}
