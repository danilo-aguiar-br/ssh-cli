// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: human-readable VPS/exec output (extracted from output monolith).
#![forbid(unsafe_code)]
//! Text-mode formatters for VPS CRUD and one-shot execution.

use super::emit::{is_quiet, write_line};
use super::sanitize::escape_untrusted;
use crate::masking::mask;
use crate::ssh::ExecutionOutput;
use crate::vps::model::VpsRecord;
use secrecy::ExposeSecret;
use std::io::{self, Write};

/// Prints the doctor report as human text (GAP-SSH-IO-005).
///
/// # Errors
/// Any stdout write or flush failure, including `BrokenPipe` (exit 141 upstream).
// B3: kept deliberately. The doctor report is a flat, stable agent surface — the
// text renderer mirrors the JSON field set one-for-one. Wrapping it in a context
// struct would create a second definition of the same contract that could drift
// from the wire DTO, which is a worse failure than a long parameter list.
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
) -> io::Result<()> {
    if is_quiet() {
        return Ok(());
    }
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let opt_out = if plaintext_opt_out { "yes" } else { "no" };
    writeln!(out, "Winning layer:   {layer}")?;
    writeln!(out, "Config path:      {}", escape_untrusted(config_path))?;
    writeln!(out, "Exists:           {exists}")?;
    writeln!(out, "Permissions:      {perms}")?;
    writeln!(out, "Schema:           {schema_version}")?;
    writeln!(out, "Hosts:            {hosts}")?;
    writeln!(out, "known_hosts:      {}", escape_untrusted(known_hosts))?;
    writeln!(out, "active file:      {}", escape_untrusted(active_file))?;
    writeln!(
        out,
        "Secrets at-rest:  {secrets_at_rest} (key source: {secrets_key_source})"
    )?;
    writeln!(
        out,
        "Secrets key file: {}",
        escape_untrusted(secrets_key_file)
    )?;
    writeln!(out, "Plaintext opt-out: {opt_out}")?;
    writeln!(out, "Telemetry:        disabled")?;
    out.flush()
}

/// Prints the config-directory lines of the doctor report as human text.
///
/// Reads `config_dir_permissions` (omitted off Unix), `config_dir_exposed` and
/// `invalid_host_names` from the `local` section that the JSON report carries,
/// so the text and JSON surfaces cannot drift. Names are escaped again here;
/// that is a no-op for the already escaped list `doctor` builds.
///
/// # Errors
/// Any stdout write or flush failure, including `BrokenPipe` (exit 141 upstream).
pub fn print_doctor_dir_text(local: &serde_json::Value) -> io::Result<()> {
    if is_quiet() {
        return Ok(());
    }
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    if let Some(mode) = local["config_dir_permissions"].as_str() {
        writeln!(out, "Config dir mode:  {}", escape_untrusted(mode))?;
    }
    let exposed = local["config_dir_exposed"].as_bool().unwrap_or(false);
    writeln!(
        out,
        "Config dir exposed: {}",
        if exposed { "yes" } else { "no" }
    )?;
    let names: Vec<_> = local["invalid_host_names"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(escape_untrusted)
                .collect()
        })
        .unwrap_or_default();
    if names.is_empty() {
        writeln!(out, "Invalid host names: none")?;
    } else {
        writeln!(out, "Invalid host names: {}", names.join(", "))?;
    }
    out.flush()
}

/// Prints the VPS list as masked text.
///
/// Column widths come from the rows themselves: the widest cell (or header) of each
/// column, counted in `char`s after escaping. The previous fixed widths (20/30/6/15)
/// pushed every later column right as soon as one value was longer, which a real
/// 17-character user name did. The last column is not padded, so no line ends in
/// spaces. Rows are escaped once and borrowed (`Cow`) for both passes.
///
/// # Errors
/// Any stdout write or flush failure, including `BrokenPipe` (exit 141 upstream).
pub fn print_list_text(records: &[VpsRecord]) -> io::Result<()> {
    if is_quiet() {
        return Ok(());
    }
    if records.is_empty() {
        return write_line(&crate::i18n::t(crate::i18n::Message::VpsRegistryEmpty));
    }

    const HEADER: [&str; 5] = ["NAME", "HOST", "PORT", "USER", "PASSWORD"];
    let rows: Vec<[std::borrow::Cow<'_, str>; 5]> = records
        .iter()
        .map(|r| {
            // A key-only or agent host has no password: do not fake a masked value.
            let password = if r.password.expose_secret().is_empty() {
                "(not set)"
            } else {
                mask(r.password.expose_secret())
            };
            [
                escape_untrusted(r.name.as_str()),
                escape_untrusted(r.host.as_str()),
                r.port.to_string().into(),
                escape_untrusted(r.username.as_str()),
                password.into(),
            ]
        })
        .collect();

    let mut widths = HEADER.map(|h| h.chars().count());
    for row in &rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    write_list_row(&mut out, &HEADER, &widths)?;
    for row in &rows {
        write_list_row(&mut out, row, &widths)?;
    }
    out.flush()
}

/// Writes one `vps list` line: every cell but the last padded to its column width.
fn write_list_row<W: Write, S: AsRef<str>>(
    out: &mut W,
    cells: &[S; 5],
    widths: &[usize; 5],
) -> io::Result<()> {
    let last = cells.len() - 1;
    for (i, (cell, width)) in cells.iter().zip(widths).enumerate() {
        if i == last {
            writeln!(out, "{}", cell.as_ref())?;
        } else {
            write!(out, "{:<width$} ", cell.as_ref())?;
        }
    }
    Ok(())
}

/// Prints one VPS record as masked text.
///
/// # Errors
/// Any stdout write or flush failure, including `BrokenPipe` (exit 141 upstream).
pub fn print_details_text(r: &VpsRecord) -> io::Result<()> {
    if is_quiet() {
        return Ok(());
    }
    // GAP-SSH-JSON-001: empty password (key-only) does not fake a masked value.
    // mask() is &'static str (zero-alloc); keep both branches as &str.
    let password = if r.password.expose_secret().is_empty() {
        "(not set)"
    } else {
        mask(r.password.expose_secret())
    };
    let key_path_owned = r.key_path.as_ref().map(|k| k.to_string_lossy_owned());
    let key_path = escape_untrusted(key_path_owned.as_deref().unwrap_or("(not set)"));
    let sudo = r
        .sudo_password
        .as_ref()
        .map_or("(not set)", |s| mask(s.expose_secret()));
    let su = r
        .su_password
        .as_ref()
        .map_or("(not set)", |s| mask(s.expose_secret()));
    let key_passphrase = r
        .key_passphrase
        .as_ref()
        .map_or("(not set)", |s| mask(s.expose_secret()));
    let agent_socket = escape_untrusted(r.agent_socket.as_deref().unwrap_or("(not set)"));
    // Tags and TLS were stored but never printed, so a human reviewing a host
    // could not see the fleet grouping or the transport without asking SQL.
    let tags = if r.tags.is_empty() {
        "(none)".to_string()
    } else {
        r.tags
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let tags = escape_untrusted(&tags);
    let tls_sni = escape_untrusted(r.tls_sni.as_deref().unwrap_or("(host name)"));
    let tls_client_cert = escape_untrusted(r.tls_client_cert.as_deref().unwrap_or("(not set)"));
    let tls_client_key = escape_untrusted(r.tls_client_key.as_deref().unwrap_or("(not set)"));

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    writeln!(out, "Name:           {}", escape_untrusted(r.name.as_str()))?;
    writeln!(out, "Host:           {}", escape_untrusted(r.host.as_str()))?;
    writeln!(out, "Port:           {}", r.port)?;
    writeln!(
        out,
        "User:           {}",
        escape_untrusted(r.username.as_str())
    )?;
    writeln!(out, "Password:       {password}")?;
    writeln!(out, "Key path:       {key_path}")?;
    writeln!(out, "Key passphrase: {key_passphrase}")?;
    writeln!(out, "Agent:          {}", r.use_agent)?;
    writeln!(out, "Agent socket:   {agent_socket}")?;
    writeln!(out, "Tags:           {tags}")?;
    writeln!(out, "TLS:            {}", r.tls)?;
    writeln!(out, "TLS SNI:        {tls_sni}")?;
    writeln!(out, "TLS cli cert:   {tls_client_cert}")?;
    writeln!(out, "TLS client key: {tls_client_key}")?;
    writeln!(out, "Sudo password:  {sudo}")?;
    writeln!(out, "Su password:    {su}")?;
    writeln!(out, "Timeout (ms):   {}", r.timeout_ms)?;
    writeln!(out, "Max cmd chars:  {}", r.max_command_chars.wire())?;
    writeln!(out, "Max out chars:  {}", r.max_output_chars.wire())?;
    writeln!(out, "Disable sudo:   {}", r.disable_sudo)?;
    writeln!(out, "Schema version: {}", r.schema_version)?;
    writeln!(out, "Added at:       {}", r.added_at)?;
    out.flush()
}

/// Prints SSH command execution output as text.
///
/// The remote stdout and stderr are written verbatim, on purpose: they are the
/// payload the caller asked for, and escaping them would corrupt it. Only the
/// local labels around them are ours.
///
/// # Errors
/// Any stdout write or flush failure, including `BrokenPipe` (exit 141 upstream).
pub fn print_execution_output(output: &ExecutionOutput) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
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
        None => writeln!(out, "--- exit code: N/A ({}ms) ---", output.duration_ms)?,
    }
    // G-IO-04: technical English only on stdout (agent contract).
    if output.truncated_stdout {
        writeln!(out, "(stdout was truncated)")?;
    }
    if output.truncated_stderr {
        writeln!(out, "(stderr was truncated)")?;
    }
    out.flush()
}

/// Prints the health-check result as text.
///
/// # Errors
/// Any stdout write or flush failure, including `BrokenPipe` (exit 141 upstream).
pub fn print_health_check(name: &str, latency_ms: u64) -> io::Result<()> {
    if is_quiet() {
        return Ok(());
    }
    let msg = crate::i18n::t(crate::i18n::Message::HealthCheckOk {
        name: escape_untrusted(name).into_owned(),
    });
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    writeln!(out, "{msg}")?;
    writeln!(out, "  latency: {latency_ms}ms")?;
    out.flush()
}
