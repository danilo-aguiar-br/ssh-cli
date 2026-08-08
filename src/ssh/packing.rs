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
            .field("stdin", &self.stdin.as_ref().map(|_| "<redacted bytes>"))
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

/// Random bytes behind a remote job marker.
///
/// 128 bits make an accidental collision between two concurrent invocations
/// impossible in practice, which is what keeps `pkill -f` from reaching a
/// process this invocation does not own.
const ABORT_MARKER_RANDOM_BYTES: usize = 16;

/// Fixed, greppable prefix of the remote job marker.
///
/// The prefix exists for humans reading `ps` output on the target host; the
/// uniqueness comes from the random suffix, never from the prefix.
const ABORT_MARKER_PREFIX: &str = "sshcli-job-";

/// Lowercase hex alphabet used to render the marker without a formatting machinery.
const HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";

/// Mints a per-invocation marker to be embedded in the remote command line.
///
/// A5: the previous abort path derived the `pkill -f` pattern from the *command
/// text*, which for elevated runs degraded to the literal `sudo -S -p`. That
/// pattern matches every `sudo` process on the target host, so a local timeout
/// killed unrelated sessions of other users and of concurrent `ssh-cli` runs.
/// A marker that only this process knows makes the kill self-scoped.
///
/// The marker is derived from the OS CSPRNG, never from the clock: two
/// invocations started in the same millisecond must still not collide, and a
/// skewed client clock must not be able to make one invocation adopt another's
/// identity.
///
/// Returns `None` when the CSPRNG is unavailable; callers must then skip the
/// remote abort entirely rather than fall back to a guessable identifier.
#[must_use]
pub fn new_remote_job_marker() -> Option<String> {
    let mut raw = [0u8; ABORT_MARKER_RANDOM_BYTES];
    if getrandom::fill(&mut raw).is_err() {
        return None;
    }
    let mut marker = String::with_capacity(ABORT_MARKER_PREFIX.len() + raw.len() * 2);
    marker.push_str(ABORT_MARKER_PREFIX);
    for byte in raw {
        marker.push(HEX_DIGITS[usize::from(byte >> 4)] as char);
        marker.push(HEX_DIGITS[usize::from(byte & 0x0f)] as char);
    }
    Some(marker)
}

/// True when `value` has the shape produced by [`new_remote_job_marker`].
///
/// Used as a guard before building a kill command: an abort pattern that is not
/// a marker would widen the blast radius back to arbitrary command text.
#[must_use]
pub fn is_remote_job_marker(value: &str) -> bool {
    value.strip_prefix(ABORT_MARKER_PREFIX).is_some_and(|hex| {
        hex.len() == ABORT_MARKER_RANDOM_BYTES * 2 && hex.bytes().all(|b| b.is_ascii_hexdigit())
    })
}

/// Wraps `command` so the remote process carries `marker` in its argv.
///
/// `sh -c '<command>' '<marker>'` runs the command unchanged and binds the
/// marker to `$0`, which is what lands in `/proc/<pid>/cmdline` and therefore
/// what `pkill -f` can match. The marker is not a secret and never carries one.
///
/// Caveat kept deliberately: only processes whose argv contains the marker are
/// reachable by the abort, i.e. the wrapper shell and anything that inherits the
/// text. Grandchildren that re-exec with a fresh argv survive. Leaking a stray
/// child is strictly safer than the previous behaviour of killing third-party
/// processes.
#[must_use]
pub fn wrap_with_abort_marker(command: &str, marker: &str) -> String {
    let cmd_esc = escape_shell_single_quotes(command);
    let marker_esc = escape_shell_single_quotes(marker);
    format!("sh -c {cmd_esc} {marker_esc}")
}

/// Sanitizes a command fragment for best-effort use with `pkill -f`.
///
/// Kept for callers that need the sanitizer itself. It is **no longer** used to
/// build remote aborts: a pattern taken from user command text matches foreign
/// processes (see [`new_remote_job_marker`]). Accepts alphanumerics and a
/// restricted symbol set; stops at the first dangerous metacharacter. Requires
/// at least 3 characters. Never embeds passwords (pattern only).
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
/// Does not embed secrets. `pattern` must be a marker from
/// [`new_remote_job_marker`]; anything else re-opens A5 by matching processes
/// this invocation does not own.
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

    #[test]
    fn job_markers_are_unique_per_invocation() {
        // A5: the abort of one invocation must not reach another invocation.
        let a = new_remote_job_marker().expect("csprng available");
        let b = new_remote_job_marker().expect("csprng available");
        assert_ne!(a, b);
        assert!(is_remote_job_marker(&a) && is_remote_job_marker(&b));

        let kill_a = pack_abort_pkill(&a);
        let kill_b = pack_abort_pkill(&b);
        // Neither kill command carries the other invocation's marker, so the
        // remote `pkill -f` cannot select the foreign process.
        assert!(!kill_a.contains(&b));
        assert!(!kill_b.contains(&a));

        let cmd_a = wrap_with_abort_marker("sleep 999", &a);
        let cmd_b = wrap_with_abort_marker("sleep 999", &b);
        assert!(cmd_a.contains(&a) && !cmd_a.contains(&b));
        assert!(cmd_b.contains(&b) && !cmd_b.contains(&a));
    }

    #[test]
    fn abort_marker_never_matches_third_party_sudo() {
        // A5 regression: the old pattern degraded to `sudo -S -p`, which
        // `pkill -f` matches on every sudo process of every user.
        let password = SecretString::from("s3cr3t".to_string());
        let pack = pack_sudo("systemctl restart nginx", Some(&password));
        let marker = new_remote_job_marker().expect("csprng available");
        let kill = pack_abort_pkill(&marker);
        assert!(!kill.contains("sudo"));
        assert!(kill.contains(&marker));

        // The wrapper keeps the secret off the remote command line.
        let wrapped = wrap_with_abort_marker(&pack.command, &marker);
        assert!(!wrapped.contains("s3cr3t"));
        assert!(wrapped.contains(&marker));
    }

    #[test]
    fn marker_shape_is_validated() {
        assert!(!is_remote_job_marker("sudo -S -p"));
        assert!(!is_remote_job_marker("sshcli-job-"));
        assert!(!is_remote_job_marker("sshcli-job-zz"));
    }
}
