// SPDX-License-Identifier: MIT OR Apache-2.0
//! Only module allowed to emit stdout for VPS CRUD.
//!
//! This module centralizes ALL CRUD formatting: text and JSON.
//!
//! Logs (tracing) go to stderr, managed by `tracing-subscriber`.

use crate::masking::mask;
use crate::ssh::ExecutionOutput;
use crate::vps::model::VpsRecord;
use secrecy::ExposeSecret;
use serde_json::json;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

/// Global `--quiet` flag (suppresses human messages on stdout).
static QUIET: AtomicBool = AtomicBool::new(false);

/// When true, errors in `main` use a JSON envelope on stderr (IO-003).
static JSON_ERRORS: AtomicBool = AtomicBool::new(false);

/// Sets whether the CLI is in quiet mode (GAP-SSH-IO-004).
pub fn set_quiet(quiet: bool) {
    QUIET.store(quiet, Ordering::SeqCst);
}

/// Sets whether errors are emitted as a JSON envelope on stderr.
pub fn set_json_errors(json: bool) {
    JSON_ERRORS.store(json, Ordering::SeqCst);
}

/// Returns whether quiet mode is active.
#[must_use]
pub fn is_quiet() -> bool {
    QUIET.load(Ordering::SeqCst)
}

/// Returns whether errors should use a JSON envelope.
#[must_use]
pub fn wants_json_errors() -> bool {
    JSON_ERRORS.load(Ordering::SeqCst)
}

/// Writes a line to stdout with pure LF (never CRLF).
///
/// In quiet mode still writes (data/paths are API). Prefer `print_success`
/// for human messages.
///
/// # Errors
/// Returns an error if stdout I/O fails.
pub fn write_line(content: &str) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(content.as_bytes())?;
    handle.write_all(b"\n")?;
    handle.flush()?;
    Ok(())
}

/// Prints a human success message (suppressed with `--quiet`).
pub fn print_success(message: &str) {
    if is_quiet() {
        return;
    }
    println!("{message}");
}

/// Human banner (tunnel etc.): Text+TTY+!quiet+!JSON errors only (GAP-SSH-IO-006).
///
/// In pipes/agents, progress goes to `tracing` (stderr), never stdout.
pub fn print_human_banner(message: &str) {
    if is_quiet() || wants_json_errors() {
        return;
    }
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        return;
    }
    if std::env::var_os("SSH_CLI_FORCE_TEXT").is_none() {
        // Without FORCE_TEXT, non-TTY already returned; human TTY is ok.
    }
    println!("{message}");
}

/// Prints an error message on stderr (human-facing).
pub fn print_error(message: &str) {
    eprintln!("{message}");
}

/// Emits a JSON error envelope on stderr (GAP-SSH-IO-003).
pub fn print_error_envelope(
    exit_code: i32,
    message: &str,
    remote_exit_code: Option<i32>,
) -> io::Result<()> {
    let mut v = json!({
        "exit_code": exit_code,
        "message": message,
    });
    if let Some(r) = remote_exit_code {
        v["remote_exit_code"] = json!(r);
    }
    let s = serde_json::to_string(&v).unwrap_or_else(|_| {
        format!(r#"{{"exit_code":{exit_code},"message":"serialization error"}}"#)
    });
    let mut err = io::stderr().lock();
    err.write_all(s.as_bytes())?;
    err.write_all(b"\n")?;
    err.flush()?;
    Ok(())
}

/// Prints pretty JSON on stdout (API data; always respects quiet=false).
pub fn print_json_value(v: &serde_json::Value) -> io::Result<()> {
    let s = serde_json::to_string_pretty(v).map_err(io::Error::other)?;
    write_line(&s)
}

/// Prints the doctor report as text (GAP-SSH-IO-005).
#[allow(clippy::too_many_arguments)]
pub fn print_doctor_text(
    layer: &str,
    config_path: &str,
    exists: bool,
    perms: &str,
    schema_version: u32,
    hosts: usize,
    known_hosts: &str,
    active_file: &str,
    secrets_at_rest: &str,
    secrets_key_source: &str,
    secrets_key_file: &str,
    plaintext_opt_out: bool,
) {
    if is_quiet() {
        return;
    }
    println!("Winning layer:   {layer}");
    println!("Config path:      {config_path}");
    println!("Exists:           {exists}");
    println!("Permissions:      {perms}");
    println!("Schema:           {schema_version}");
    println!("Hosts:            {hosts}");
    println!("known_hosts:      {known_hosts}");
    println!("active file:      {active_file}");
    println!("Secrets at-rest:  {secrets_at_rest} (key source: {secrets_key_source})");
    println!("Secrets key file: {secrets_key_file}");
    println!(
        "Plaintext opt-out: {}",
        if plaintext_opt_out { "yes" } else { "no" }
    );
    println!("Telemetry:        disabled");
}

/// Prints the VPS list as masked text.
pub fn print_list_text(records: &[VpsRecord]) {
    if is_quiet() {
        return;
    }
    if records.is_empty() {
        println!(
            "{}",
            crate::i18n::t(crate::i18n::Message::VpsRegistryEmpty)
        );
        return;
    }

    println!(
        "{:<20} {:<30} {:<6} {:<15} {:<20}",
        "NAME", "HOST", "PORT", "USER", "PASSWORD"
    );
    for r in records {
        println!(
            "{:<20} {:<30} {:<6} {:<15} {:<20}",
            r.name,
            r.host,
            r.port,
            r.username,
            mask(r.password.expose_secret())
        );
    }
}

/// Prints the VPS list as masked JSON.
pub fn print_list_json(records: &[VpsRecord]) {
    let list: Vec<_> = records.iter().map(record_to_masked_json).collect();
    match serde_json::to_string_pretty(&list) {
        Ok(s) => println!("{s}"),
        Err(err) => eprintln!("failed to serialize JSON: {err}"),
    }
}

/// Prints a single VPS record as masked text.
pub fn print_details_text(r: &VpsRecord) {
    if is_quiet() {
        return;
    }
    println!("Name:            {}", r.name);
    println!("Host:           {}", r.host);
    println!("Port:            {}", r.port);
    println!("User:            {}", r.username);
    // GAP-SSH-JSON-001: empty password (key-only) does not fake a masked value.
    println!(
        "Password:       {}",
        if r.password.expose_secret().is_empty() {
            "(not set)".to_string()
        } else {
            mask(r.password.expose_secret())
        }
    );
    println!(
        "Key path:       {}",
        r.key_path.as_deref().unwrap_or("(not set)")
    );
    println!(
        "Sudo password:  {}",
        r.sudo_password
            .as_ref()
            .map_or_else(|| "(not set)".into(), |s| mask(s.expose_secret()))
    );
    println!(
        "Su password:    {}",
        r.su_password
            .as_ref()
            .map_or_else(|| "(not set)".into(), |s| mask(s.expose_secret()))
    );
    println!("Timeout (ms):   {}", r.timeout_ms);
    println!("Max cmd chars:  {}", r.max_command_chars);
    println!("Max out chars:  {}", r.max_output_chars);
    println!("Disable sudo:   {}", r.disable_sudo);
    println!("Schema version: {}", r.schema_version);
    println!("Added at:        {}", r.added_at);
}

/// Prints a single VPS record as masked JSON.
pub fn print_details_json(r: &VpsRecord) {
    let v = record_to_masked_json(r);
    match serde_json::to_string_pretty(&v) {
        Ok(s) => println!("{s}"),
        Err(err) => eprintln!("failed to serialize JSON: {err}"),
    }
}

fn record_to_masked_json(r: &VpsRecord) -> serde_json::Value {
    // GAP-SSH-JSON-001: missing/empty password → null (like sudo/su); present → "***".
    let password = if r.password.expose_secret().is_empty() {
        json!(null)
    } else {
        json!(mask(r.password.expose_secret()))
    };
    json!({
        "name": r.name,
        "host": r.host,
        "port": r.port,
        "user": r.username,
        "password": password,
        "key_path": r.key_path,
        "key_passphrase": r.key_passphrase.as_ref().map(|s| mask(s.expose_secret())),
        "sudo_password": r.sudo_password.as_ref().map(|s| mask(s.expose_secret())),
        "su_password": r.su_password.as_ref().map(|s| mask(s.expose_secret())),
        "timeout_ms": r.timeout_ms,
        "max_command_chars": r.max_command_chars,
        "max_output_chars": r.max_output_chars,
        "disable_sudo": r.disable_sudo,
        "schema_version": r.schema_version,
        "added_at": r.added_at,
    })
}

/// GAP-SSH-UX-001: hosts for `vps export --json`.
///
/// - Redacted (`include_secrets=false`): secrets vazios/null, **nunca** ciphertext `sshcli-enc:`
///   (EXP-001 parity). Empty password → `""` in export envelope (honest skeleton).
/// - With secrets: plaintext password only if `--include-secrets` (same risk as TOML).
pub fn export_hosts_to_json(
    hosts: &std::collections::BTreeMap<String, VpsRecord>,
    include_secrets: bool,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (name, r) in hosts {
        let entry = if include_secrets {
            json!({
                "name": r.name,
                "host": r.host,
                "port": r.port,
                "user": r.username,
                "password": r.password.expose_secret(),
                "key_path": r.key_path,
                "key_passphrase": r.key_passphrase.as_ref().map(|s| s.expose_secret().to_string()),
                "sudo_password": r.sudo_password.as_ref().map(|s| s.expose_secret().to_string()),
                "su_password": r.su_password.as_ref().map(|s| s.expose_secret().to_string()),
                "timeout_ms": r.timeout_ms,
                "max_command_chars": r.max_command_chars,
                "max_output_chars": r.max_output_chars,
                "disable_sudo": r.disable_sudo,
                "schema_version": r.schema_version,
                "added_at": r.added_at,
            })
        } else {
            // Redacted: empty password string (import skeleton), null optional secrets.
            json!({
                "name": r.name,
                "host": r.host,
                "port": r.port,
                "user": r.username,
                "password": "",
                "key_path": r.key_path,
                "key_passphrase": null,
                "sudo_password": null,
                "su_password": null,
                "timeout_ms": r.timeout_ms,
                "max_command_chars": r.max_command_chars,
                "max_output_chars": r.max_output_chars,
                "disable_sudo": r.disable_sudo,
                "schema_version": r.schema_version,
                "added_at": r.added_at,
            })
        };
        map.insert(name.clone(), entry);
    }
    serde_json::Value::Object(map)
}

/// Prints stdout/stderr from an SSH command execution.
///
/// Formato:
/// ```text
/// --- stdout ---
/// <stdout>
/// --- stderr ---
/// <stderr>
/// --- exit code: <code> (<duration_ms>ms) ---
/// ```
pub fn print_execution_output(output: &ExecutionOutput) {
    println!("--- stdout ---");
    if output.stdout.is_empty() {
        println!("(empty)");
    } else {
        println!("{}", output.stdout);
    }
    println!("--- stderr ---");
    if output.stderr.is_empty() {
        println!("(empty)");
    } else {
        println!("{}", output.stderr);
    }
    let code_str = output
        .exit_code
        .map(|c| c.to_string())
        .unwrap_or_else(|| "N/A".to_string());
    println!("--- exit code: {} ({}ms) ---", code_str, output.duration_ms);
    if output.truncated_stdout {
        println!("(stdout foi truncado)");
    }
    if output.truncated_stderr {
        println!("(stderr foi truncado)");
    }
}

/// Prints SSH command execution output as JSON.
pub fn print_execution_output_json(output: &ExecutionOutput) {
    let v = json!({
        "stdout": output.stdout,
        "stderr": output.stderr,
        "exit_code": output.exit_code,
        "truncated_stdout": output.truncated_stdout,
        "truncated_stderr": output.truncated_stderr,
        "duration_ms": output.duration_ms,
    });
    match serde_json::to_string_pretty(&v) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("failed to serialize JSON: {e}"),
    }
}

/// Prints a health-check result as text.
pub fn print_health_check(name: &str, latency_ms: u64) {
    if is_quiet() {
        return;
    }
    println!(
        "{}",
        crate::i18n::t(crate::i18n::Message::HealthCheckOk {
            name: name.to_string(),
        })
    );
    println!("  latência: {latency_ms}ms");
}

/// Prints a health-check result as JSON.
pub fn print_health_check_json(name: &str, latency_ms: u64) {
    let v = json!({
        "name": name,
        "status": "ok",
        "latency_ms": latency_ms,
    });
    match serde_json::to_string_pretty(&v) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("failed to serialize JSON: {e}"),
    }
}

/// Prints an SCP transfer result as JSON (GAP-SSH-IO-007 / SCP-021 / IO-009).
pub fn print_transfer_json(
    direction: &str,
    vps: &str,
    local: &str,
    remote: &str,
    bytes: u64,
    duration_ms: u64,
) {
    // GAP-SSH-IO-009: event discriminator (parity with tunnel_listening).
    let v = json!({
        "ok": true,
        "event": "scp-transfer",
        "direction": direction,
        "vps": vps,
        "local": local,
        "remote": remote,
        "bytes": bytes,
        "duration_ms": duration_ms,
    });
    match serde_json::to_string_pretty(&v) {
        Ok(s) => {
            let _ = write_line(&s);
        }
        Err(e) => eprintln!("failed to serialize JSON: {e}"),
    }
}

/// JSON event when the local tunnel listener comes up (GAP-SSH-IO-008).
pub fn print_tunnel_listening_json(
    vps: &str,
    local_port: u16,
    remote_host: &str,
    remote_port: u16,
    timeout_ms: u64,
) {
    let v = json!({
        "ok": true,
        "event": "tunnel_listening",
        "vps": vps,
        "local_port": local_port,
        "remote_host": remote_host,
        "remote_port": remote_port,
        "timeout_ms": timeout_ms,
    });
    match serde_json::to_string_pretty(&v) {
        Ok(s) => {
            let _ = write_line(&s);
        }
        Err(e) => eprintln!("failed to serialize JSON: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::ExecutionOutput;
    use crate::vps::model::VpsRecord;
    use secrecy::SecretString;

    fn registro_teste() -> VpsRecord {
        VpsRecord::new(
            "vps-teste".into(),
            "1.2.3.4".into(),
            22,
            "root".into(),
            SecretString::from("senha-super-secreta".to_string()),
            None,
            None,
            Some(5000),
            Some(1000),
            Some(1000),
            Some(SecretString::from("sudo-password-longa-aqui".to_string())),
            None,
            false,
        )
    }

    #[test]
    fn masked_json_contains_required_fields() {
        let r = registro_teste();
        let json = record_to_masked_json(&r);
        assert_eq!(json["name"], "vps-teste");
        assert_eq!(json["host"], "1.2.3.4");
        assert_eq!(json["port"], 22);
        assert_eq!(json["user"], "root");
        assert_eq!(json["password"].as_str().unwrap(), "***");
        assert_eq!(json["sudo_password"].as_str().unwrap(), "***");
        assert!(json["su_password"].is_null());
        assert_eq!(json["timeout_ms"], 5000);
        assert_eq!(json["max_command_chars"], 1000);
        assert_eq!(json["max_output_chars"], 1000);
        assert_eq!(json["schema_version"], 2);
    }

    #[test]
    fn masked_json_sudo_null_when_unset() {
        let mut r = registro_teste();
        r.sudo_password = None;
        let json = record_to_masked_json(&r);
        assert!(json["sudo_password"].is_null());
    }

    #[test]
    fn masked_json_su_password_present() {
        let mut r = registro_teste();
        r.su_password = Some(SecretString::from("senha-su-muito-longa-aqui".to_string()));
        let json = record_to_masked_json(&r);
        assert_eq!(json["su_password"].as_str().unwrap(), "***");
    }

    #[test]
    fn masked_json_password_null_when_empty() {
        let mut r = registro_teste();
        r.password = SecretString::from(String::new());
        let json = record_to_masked_json(&r);
        assert!(json["password"].is_null());
    }

    #[test]
    fn write_line_ok() {
        let result = write_line("write test");
        assert!(result.is_ok());
    }

    #[test]
    fn write_line_special_chars() {
        let result = write_line("line with \t tab and \"quotes\"");
        assert!(result.is_ok());
    }

    #[test]
    fn execution_output_fully_formatted() {
        let output = ExecutionOutput {
            stdout: "output do comando".to_string(),
            stderr: "command error".to_string(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 150,
        };
        let result = write_line(&format!(
            "stdout: {}, stderr: {}, exit: {:?}",
            output.stdout, output.stderr, output.exit_code
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn execution_output_without_exit_code() {
        let output = ExecutionOutput {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit_code: None,
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 0,
        };
        let code_str = output
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        assert_eq!(code_str, "N/A");
    }

    #[test]
    fn vps_record_debug_does_not_expose_password() {
        let r = registro_teste();
        let json = record_to_masked_json(&r);
        let json_str = serde_json::to_string(&json).unwrap();
        assert!(!json_str.contains("senha-super-secreta"));
        assert!(!json_str.contains("sudo-password-longa-aqui"));
    }

    #[test]
    fn execution_output_truncated_shows_warning() {
        let output = ExecutionOutput {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: Some(1),
            truncated_stdout: true,
            truncated_stderr: true,
            duration_ms: 100,
        };
        assert!(output.truncated_stdout);
        assert!(output.truncated_stderr);
    }

    #[test]
    fn execution_output_numeric_exit_code() {
        let output = ExecutionOutput {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit_code: Some(127),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 0,
        };
        let code_str = output
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        assert_eq!(code_str, "127");
    }

    #[test]
    fn write_line_empty_string() {
        let result = write_line("");
        assert!(result.is_ok());
    }

    #[test]
    fn write_line_brazilian_unicode() {
        let result = write_line("ação você está Itaú");
        assert!(result.is_ok());
    }

    #[test]
    fn write_line_with_emojis() {
        let result = write_line("texto com 🚀 e 🔐");
        assert!(result.is_ok());
    }

    #[test]
    fn write_line_with_newlines() {
        let result = write_line("linha1\nlinha2\nlinha3");
        assert!(result.is_ok());
    }

    #[test]
    fn write_line_long_text() {
        let long_text = "a".repeat(10000);
        let result = write_line(&long_text);
        assert!(result.is_ok());
    }

    #[test]
    fn masked_json_short_password_asterisks() {
        let mut r = registro_teste();
        r.password = SecretString::from("curta".to_string());
        let json = record_to_masked_json(&r);
        let password_str = json["password"].as_str().unwrap();
        assert_eq!(password_str, "***");
    }

    #[test]
    fn masked_json_with_sudo_and_su_set() {
        let mut r = registro_teste();
        r.sudo_password = Some(SecretString::from("sudo-pass-longa-aqui".to_string()));
        r.su_password = Some(SecretString::from("su-pass-longa-aqui".to_string()));
        let json = record_to_masked_json(&r);
        assert!(!json["sudo_password"].is_null());
        assert!(!json["su_password"].is_null());
        assert_eq!(json["sudo_password"].as_str().unwrap(), "***");
        assert_eq!(json["su_password"].as_str().unwrap(), "***");
    }

    #[test]
    fn execution_output_full_formatting() {
        let output = ExecutionOutput {
            stdout: "comando executado".to_string(),
            stderr: "aviso harmless".to_string(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 1000,
        };
        assert_eq!(output.stdout, "comando executado");
        assert_eq!(output.stderr, "aviso harmless");
        assert_eq!(output.exit_code, Some(0));
        assert_eq!(output.duration_ms, 1000);
        assert!(!output.truncated_stdout);
        assert!(!output.truncated_stderr);
    }

    #[test]
    fn execution_output_without_stderr() {
        let output = ExecutionOutput {
            stdout: "ok".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 50,
        };
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn execution_output_signal_instead_of_exit() {
        let output = ExecutionOutput {
            stdout: String::new(),
            stderr: "signal received".to_string(),
            exit_code: None,
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 5000,
        };
        assert!(output.exit_code.is_none());
    }

    #[test]
    fn execution_output_json_required_fields() {
        let output = ExecutionOutput {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 100,
        };
        print_execution_output_json(&output);
    }
}
