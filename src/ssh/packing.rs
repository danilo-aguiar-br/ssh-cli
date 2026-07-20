// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Safe packing of `sudo`/`su` commands for one-shot multi-host LLM flows.
//!
//! Builds **remote** `sh -c` strings with shell-safe single-quote escaping for
//! compound commands sent over the SSH channel (`channel.exec`), **not** local
//! `std::process::Command` spawns.
//!
//! # External process boundary (G-PROC)
//!
//! - Local product code never invokes `sh`/`sudo`/`su` via `Command`.
//! - Remote packing is intentional: elevation must run on the target host shell.
//! - Secrets go on channel stdin (`sudo -S` / `su`), never in argv / command text.
//! - Callers must pass payloads already rejected for NUL (`validate_command_length`).

use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

/// Escapes a string for safe use inside shell single quotes.
///
/// Strategy: wrap in single quotes and escape inner single quotes
/// with the sequence `'\''` (close quote, backslash-quote, open quote).
#[must_use]
pub fn escape_shell_single_quotes(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 2);
    result.push('\'');
    for ch in value.chars() {
        if ch == '\'' {
            result.push_str("'\\''");
        } else {
            result.push(ch);
        }
    }
    result.push('\'');
    result
}

/// Appends `description` as a shell comment safely.
#[must_use]
pub fn append_description(command: &str, description: Option<&str>) -> String {
    match description {
        Some(d) if !d.trim().is_empty() => {
            let cleaned = d.replace(['\n', '\r'], " ");
            format!("{command} # {cleaned}")
        }
        _ => command.to_string(),
    }
}

/// Packing result: remote command **without** secret in argv + optional bytes
/// to send on the SSH channel stdin (GAP-SSH-SEC-001).
///
/// `stdin` may hold a password; [`Drop`] zeroizes it (memory / RAII rule).
/// Debug redacts stdin. Prefer moving `stdin` into `run_command` (which also
/// zeroizes after the channel write).
#[derive(Clone)]
pub struct PackedCommand {
    /// Remote command line (no embedded password).
    pub command: String,
    /// Bytes to write on channel stdin (e.g. password + `\n` for `sudo -S` / `su`).
    pub stdin: Option<Vec<u8>>,
}

impl std::fmt::Debug for PackedCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackedCommand")
            .field("command", &self.command)
            .field(
                "stdin",
                &self.stdin.as_ref().map(|_| "<redacted bytes>"),
            )
            .finish()
    }
}

impl Drop for PackedCommand {
    fn drop(&mut self) {
        if let Some(ref mut bytes) = self.stdin {
            bytes.zeroize();
        }
    }
}

impl PackedCommand {
    /// Moves stdin out for the channel write; remaining drop is a no-op.
    ///
    /// Prefer this over field access: `Drop` prevents partial moves of `stdin`.
    #[must_use]
    pub fn take_stdin(&mut self) -> Option<Vec<u8>> {
        self.stdin.take()
    }
}

/// Packs a command for `sudo` with `sh -c`.
///
/// - With password: `sudo -S -p '' sh -c 'cmd'` and password on the **channel stdin** (not argv).
/// - Without password: `sudo -n sh -c 'cmd'`.
#[must_use]
pub fn pack_sudo(command: &str, sudo_password: Option<&SecretString>) -> PackedCommand {
    let cmd_esc = escape_shell_single_quotes(command);
    match sudo_password {
        Some(password) => {
            let mut stdin = password.expose_secret().as_bytes().to_vec();
            stdin.push(b'\n');
            PackedCommand {
                command: format!("sudo -S -p '' sh -c {cmd_esc}"),
                stdin: Some(stdin),
            }
        }
        None => PackedCommand {
            command: format!("sudo -n sh -c {cmd_esc}"),
            stdin: None,
        },
    }
}

/// Packs a command for `su - -c` one-shot; password goes on the channel stdin.
#[must_use]
pub fn pack_su(command: &str, su_password: &SecretString) -> PackedCommand {
    let cmd_esc = escape_shell_single_quotes(command);
    let mut stdin = su_password.expose_secret().as_bytes().to_vec();
    stdin.push(b'\n');
    PackedCommand {
        command: format!("su - -c {cmd_esc}"),
        stdin: Some(stdin),
    }
}

/// Sanitizes a command fragment for best-effort use with `pkill -f`.
///
/// Accepts alphanumerics and a restricted symbol set; stops at the first dangerous
/// metacharacter. Requires at least 3 characters. Never embeds passwords (pattern only).
#[must_use]
pub fn remote_abort_pattern(command: &str) -> Option<String> {
    let mut cleaned = String::with_capacity(command.len().min(128));
    for ch in command.chars().take(128) {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ' ' | ':' | '=') {
            cleaned.push(ch);
        } else {
            break;
        }
    }
    // Avoid a second heap string when trim does not shrink `cleaned`.
    let trimmed = cleaned.trim();
    if trimmed.len() < 3 {
        None
    } else if trimmed.len() == cleaned.len() {
        Some(cleaned)
    } else {
        Some(trimmed.to_string())
    }
}

/// Builds a best-effort remote abort command (TERM, then KILL).
///
/// Does not embed secrets; uses only the sanitized command pattern.
#[must_use]
pub fn pack_abort_pkill(pattern: &str) -> String {
    let esc = escape_shell_single_quotes(pattern);
    format!(
        "(pkill -TERM -f {esc} 2>/dev/null || true); sleep 0.2; (pkill -KILL -f {esc} 2>/dev/null || true)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_single_quote() {
        assert_eq!(escape_shell_single_quotes("ab'cd"), "'ab'\\''cd'");
        assert_eq!(escape_shell_single_quotes("abc"), "'abc'");
    }

    #[test]
    fn sudo_with_password_uses_sh_c_no_secret_in_argv() {
        let password = SecretString::from("s3cr3t".to_string());
        let pack = pack_sudo("echo hi | tee /tmp/x", Some(&password));
        assert!(pack.command.contains("sudo -S -p '' sh -c"));
        assert!(!pack.command.contains("s3cr3t"));
        assert!(!pack.command.contains("printf"));
        let mut pack = pack;
        let stdin = pack.take_stdin().expect("stdin with password");
        assert_eq!(stdin, b"s3cr3t\n");
    }

    #[test]
    fn sudo_without_password_uses_n() {
        let pack = pack_sudo("id", None);
        assert_eq!(pack.command, "sudo -n sh -c 'id'");
        assert!(pack.stdin.is_none());
    }

    #[test]
    fn su_pack_no_secret_in_argv() {
        let password = SecretString::from("rootpw".to_string());
        let pack = pack_su("whoami", &password);
        assert!(pack.command.contains("su - -c"));
        assert!(!pack.command.contains("rootpw"));
        assert_eq!(pack.stdin.as_deref(), Some(b"rootpw\n".as_slice()));
    }

    #[test]
    fn description_appends_comment() {
        assert_eq!(
            append_description("ls", Some("lista arquivos")),
            "ls # lista arquivos"
        );
        assert_eq!(append_description("ls", None), "ls");
    }

    #[test]
    fn debug_redacts_stdin() {
        let password = SecretString::from("s3cr3t".to_string());
        let pack = pack_sudo("id", Some(&password));
        let dbg = format!("{pack:?}");
        assert!(!dbg.contains("s3cr3t"));
        assert!(dbg.contains("<redacted bytes>"));
    }

    #[test]
    fn abort_pattern_sanitizes() {
        assert_eq!(
            remote_abort_pattern("sleep 999"),
            Some("sleep 999".to_string())
        );
        // GAP-SSH-TEST-003: dangerous metacharacter → reject (not a tautology).
        assert_eq!(remote_abort_pattern("$(rm -rf)"), None);
        assert!(remote_abort_pattern("ab").is_none());
    }
}
