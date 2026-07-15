// SPDX-License-Identifier: MIT OR Apache-2.0
//! Empacotamento seguro de comandos sudo/su (one-shot multi-host para LLMs).
//!
//! Usa `sh -c` com escape de aspas para comandos compostos seguros.

use secrecy::{ExposeSecret, SecretString};

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
            let limpo = d.replace(['\n', '\r'], " ");
            format!("{command} # {limpo}")
        }
        _ => command.to_string(),
    }
}

/// Packing result: remote command **without** secret in argv + optional bytes
/// to send on the SSH channel stdin (GAP-SSH-SEC-001).
#[derive(Debug, Clone)]
pub struct PackedCommand {
    /// Remote command line (no embedded password).
    pub command: String,
    /// Bytes to write on channel stdin (e.g. password + `\n` for `sudo -S` / `su`).
    pub stdin: Option<Vec<u8>>,
}

/// Packs a command for `sudo` with `sh -c`.
///
/// - With password: `sudo -S -p '' sh -c 'cmd'` and password on the **channel stdin** (not argv).
/// - Sem password: `sudo -n sh -c 'cmd'`.
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

/// Sanitiza trecho de command para uso best-effort em `pkill -f`.
///
/// Accepts alphanumerics and restricted symbols; stops at the first metacharacter
/// dangerous. Requires at least 3 chars. Never embeds passwords (only the command pattern).
#[must_use]
pub fn remote_abort_pattern(command: &str) -> Option<String> {
    let mut limpo = String::with_capacity(command.len().min(128));
    for ch in command.chars().take(128) {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ' ' | ':' | '=') {
            limpo.push(ch);
        } else {
            break;
        }
    }
    let t = limpo.trim();
    if t.len() < 3 {
        None
    } else {
        Some(t.to_string())
    }
}

/// Monta command de abort best-effort remoto (TERM depois KILL).
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
        let stdin = pack.stdin.expect("stdin com senha");
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
