// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: human-readable VPS/exec output (extracted from output monólito).
#![forbid(unsafe_code)]
//! Text-mode formatters for VPS CRUD and one-shot execution.

use super::emit::{is_quiet, write_line_human};
use crate::masking::mask;
use crate::ssh::ExecutionOutput;
use crate::vps::model::VpsRecord;
use secrecy::ExposeSecret;
use std::io::{self, Write};

/// Prints the doctor report as human text (GAP-SSH-IO-005).
#[allow(clippy::too_many_arguments)] // doctor report is a flat field set (stable agent surface)
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
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let opt_out = if plaintext_opt_out { "yes" } else { "no" };
    let _ = (|| -> io::Result<()> {
        writeln!(out, "Winning layer:   {layer}")?;
        writeln!(out, "Config path:      {config_path}")?;
        writeln!(out, "Exists:           {exists}")?;
        writeln!(out, "Permissions:      {perms}")?;
        writeln!(out, "Schema:           {schema_version}")?;
        writeln!(out, "Hosts:            {hosts}")?;
        writeln!(out, "known_hosts:      {known_hosts}")?;
        writeln!(out, "active file:      {active_file}")?;
        writeln!(
            out,
            "Secrets at-rest:  {secrets_at_rest} (key source: {secrets_key_source})"
        )?;
        writeln!(out, "Secrets key file: {secrets_key_file}")?;
        writeln!(out, "Plaintext opt-out: {opt_out}")?;
        writeln!(out, "Telemetry:        disabled")?;
        out.flush()
    })();
}

/// Prints the VPS list as masked text.
///
/// Streams rows with `writeln!` under one stdout lock (G-MAC-02).
pub fn print_list_text(records: &[VpsRecord]) {
    if is_quiet() {
        return;
    }
    if records.is_empty() {
        write_line_human(&crate::i18n::t(crate::i18n::Message::VpsRegistryEmpty));
        return;
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let _ = (|| -> io::Result<()> {
        writeln!(
            out,
            "{:<20} {:<30} {:<6} {:<15} {:<20}",
            "NAME", "HOST", "PORT", "USER", "PASSWORD"
        )?;
        for r in records {
            writeln!(
                out,
                "{:<20} {:<30} {:<6} {:<15} {:<20}",
                r.name,
                r.host,
                r.port,
                r.username,
                mask(r.password.expose_secret())
            )?;
        }
        out.flush()
    })();
}

/// Prints the VPS list as masked JSON.
///
/// # Errors
pub fn print_details_text(r: &VpsRecord) {
    if is_quiet() {
        return;
    }
    // GAP-SSH-JSON-001: empty password (key-only) does not fake a masked value.
    // mask() is &'static str (zero-alloc); keep both branches as &str.
    let password = if r.password.expose_secret().is_empty() {
        "(not set)"
    } else {
        mask(r.password.expose_secret())
    };
    let key_path_owned = r
        .key_path
        .as_ref()
        .map(|k| k.to_string_lossy_owned());
    let key_path = key_path_owned.as_deref().unwrap_or("(not set)");
    let sudo = r
        .sudo_password
        .as_ref()
        .map_or("(not set)", |s| mask(s.expose_secret()));
    let su = r
        .su_password
        .as_ref()
        .map_or("(not set)", |s| mask(s.expose_secret()));

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let _ = (|| -> io::Result<()> {
        writeln!(out, "Name:            {}", r.name)?;
        writeln!(out, "Host:           {}", r.host)?;
        writeln!(out, "Port:            {}", r.port)?;
        writeln!(out, "User:            {}", r.username)?;
        writeln!(out, "Password:       {password}")?;
        writeln!(out, "Key path:       {key_path}")?;
        writeln!(out, "Sudo password:  {sudo}")?;
        writeln!(out, "Su password:    {su}")?;
        writeln!(out, "Timeout (ms):   {}", r.timeout_ms)?;
        writeln!(out, "Max cmd chars:  {}", r.max_command_chars.wire())?;
        writeln!(out, "Max out chars:  {}", r.max_output_chars.wire())?;
        writeln!(out, "Disable sudo:   {}", r.disable_sudo)?;
        writeln!(out, "Schema version: {}", r.schema_version)?;
        writeln!(out, "Added at:        {}", r.added_at)?;
        out.flush()
    })();
}

/// Prints a single VPS record as masked JSON.
///
/// # Errors
pub fn print_execution_output(output: &ExecutionOutput) {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let _ = (|| -> io::Result<()> {
        writeln!(out, "--- stdout ---")?;
        if output.stdout.is_empty() {
            writeln!(out, "(empty)")?;
        } else {
            writeln!(out, "{}", output.stdout)?;
        }
        writeln!(out, "--- stderr ---")?;
        if output.stderr.is_empty() {
            writeln!(out, "(empty)")?;
        } else {
            writeln!(out, "{}", output.stderr)?;
        }
        match output.exit_code {
            Some(code) => writeln!(
                out,
                "--- exit code: {} ({}ms) ---",
                code, output.duration_ms
            )?,
            None => writeln!(
                out,
                "--- exit code: N/A ({}ms) ---",
                output.duration_ms
            )?,
        }
        // G-IO-04: technical English only on stdout (agent contract).
        if output.truncated_stdout {
            writeln!(out, "(stdout was truncated)")?;
        }
        if output.truncated_stderr {
            writeln!(out, "(stderr was truncated)")?;
        }
        out.flush()
    })();
}

/// Prints SSH command execution output as JSON.
///
/// # Errors
pub fn print_health_check(name: &str, latency_ms: u64) {
    if is_quiet() {
        return;
    }
    let msg = crate::i18n::t(crate::i18n::Message::HealthCheckOk {
        name: name.to_string(),
    });
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let _ = (|| -> io::Result<()> {
        writeln!(out, "{msg}")?;
        writeln!(out, "  latency: {latency_ms}ms")?;
        out.flush()
    })();
}

