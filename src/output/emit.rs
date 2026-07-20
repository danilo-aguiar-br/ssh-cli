// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: stdout/stderr emit primitives (extracted from output monólito).
#![forbid(unsafe_code)]
//! Quiet/JSON-error flags and LF writers for agent + human paths.

use crate::json_wire::{self, ErrorEnvelope, SuccessEnvelope};
use std::fmt;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

/// Global `--quiet` flag (suppresses human messages on stdout).
///
/// Concurrent access: process-wide flag only (no other data published with it).
/// `Ordering::Relaxed` is sufficient — no acquire/release of dependent state.
static QUIET: AtomicBool = AtomicBool::new(false);

/// When true, errors in `main` use a JSON envelope on stderr (IO-003).
///
/// Concurrent access: independent CLI mode bit; `Ordering::Relaxed` (no data fence).
static JSON_ERRORS: AtomicBool = AtomicBool::new(false);

/// Sets whether the CLI is in quiet mode (GAP-SSH-IO-004).
pub fn set_quiet(quiet: bool) {
    QUIET.store(quiet, Ordering::Relaxed);
}

/// Sets whether errors are emitted as a JSON envelope on stderr.
pub fn set_json_errors(json: bool) {
    JSON_ERRORS.store(json, Ordering::Relaxed);
}

/// Returns whether quiet mode is active.
#[must_use]
pub fn is_quiet() -> bool {
    QUIET.load(Ordering::Relaxed)
}

/// Returns whether errors should use a JSON envelope.
#[must_use]
pub fn wants_json_errors() -> bool {
    JSON_ERRORS.load(Ordering::Relaxed)
}

/// Writes a line to an arbitrary [`Write`] with pure LF, then flushes (G-IO-11).
///
/// Dependency-injection primitive: unit tests and alternate sinks pass a
/// `Cursor`/`Vec`/`File` instead of process stdout. Production paths call
/// [`write_line`] which locks real stdout.
///
/// Prefer [`write_line_to_fmt`] / [`write_line_fmt`] when the content is built
/// with `format_args!` so no intermediate `String` is allocated (G-MAC-01).
///
/// # Examples
///
/// ```
/// use ssh_cli::output::write_line_to;
/// use std::io::Cursor;
///
/// let mut buf = Cursor::new(Vec::new());
/// write_line_to(&mut buf, "hello").unwrap();
/// assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), "hello\n");
/// ```
///
/// # Errors
/// Propagates I/O errors from the underlying writer (including `BrokenPipe`).
pub fn write_line_to(out: &mut impl Write, content: &str) -> io::Result<()> {
    out.write_all(content.as_bytes())?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

/// Writes formatted content + pure LF via [`Write::write_fmt`] (G-MAC-01).
///
/// Call with `format_args!(...)` to avoid `format!` → temporary `String` →
/// `write_all` double work. Same LF + flush contract as [`write_line_to`].
///
/// # Examples
///
/// ```
/// use ssh_cli::output::write_line_to_fmt;
/// use std::io::Cursor;
///
/// let mut buf = Cursor::new(Vec::new());
/// let name = "lab";
/// write_line_to_fmt(&mut buf, format_args!("host={name}")).unwrap();
/// assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), "host=lab\n");
/// ```
///
/// # Errors
/// Propagates I/O errors from the underlying writer (including `BrokenPipe`).
pub fn write_line_to_fmt(out: &mut impl Write, args: fmt::Arguments<'_>) -> io::Result<()> {
    out.write_fmt(args)?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

/// Writes a line to stdout with pure LF (never CRLF), then flushes.
///
/// Uses `write_all` + explicit flush (rules: never rely on Drop alone).
/// BrokenPipe is propagated so callers / `main` can exit **141**.
///
/// # Errors
/// Returns an error if stdout I/O fails (including `BrokenPipe`).
pub fn write_line(content: &str) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_line_to(&mut handle, content)
}

/// Writes a formatted line to stdout without an intermediate `String` (G-MAC-01).
///
/// # Errors
/// Returns an error if stdout I/O fails (including `BrokenPipe`).
pub fn write_line_fmt(args: fmt::Arguments<'_>) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_line_to_fmt(&mut handle, args)
}

/// Writes many short lines under a single stdout lock (list/doctor/text dumps).
///
/// Batches under `BufWriter` then a single flush (not per-line flush).
/// Prefer direct `writeln!` into a locked `BufWriter` when building lines with
/// formatting (avoids a `Vec<String>` of `format!` results — G-MAC-02).
///
/// # Errors
/// Propagates I/O errors including `BrokenPipe`.
pub fn write_lines(lines: impl IntoIterator<Item = impl AsRef<str>>) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout.lock());
    for line in lines {
        handle.write_all(line.as_ref().as_bytes())?;
        handle.write_all(b"\n")?;
    }
    handle.flush()?;
    Ok(())
}

/// Best-effort human line on stdout; ignores BrokenPipe (consumer hung up).
pub(crate) fn write_line_human(content: &str) {
    match write_line(content) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {}
        Err(_) => {}
    }
}

/// Writes a diagnostic line to an arbitrary [`Write`] (G-IO-11 DI primitive).
///
/// Unlike [`write_line_to`], **BrokenPipe is treated as success** (downstream
/// closed) so human/error paths never panic the process on a closed pipe.
///
/// # Examples
///
/// ```
/// use ssh_cli::output::write_stderr_line_to;
/// use std::io::Cursor;
///
/// let mut buf = Cursor::new(Vec::new());
/// write_stderr_line_to(&mut buf, "warn").unwrap();
/// assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), "warn\n");
/// ```
///
/// # Errors
/// Non-pipe I/O failures from the underlying writer.
pub fn write_stderr_line_to(err: &mut impl Write, content: &str) -> io::Result<()> {
    match write_line_to(err, content) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e),
    }
}

/// Formatted stderr line via `write_fmt` (G-MAC-01); BrokenPipe → Ok.
///
/// # Errors
/// Non-pipe I/O failures from the underlying writer.
pub fn write_stderr_line_to_fmt(
    err: &mut impl Write,
    args: fmt::Arguments<'_>,
) -> io::Result<()> {
    match write_line_to_fmt(err, args) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e),
    }
}

/// Writes a line to stderr with flush (warnings / human errors).
///
/// # Errors
/// Returns I/O errors except BrokenPipe (treated as Ok — downstream closed).
pub fn write_stderr_line(content: &str) -> io::Result<()> {
    let stderr = io::stderr();
    let mut handle = stderr.lock();
    write_stderr_line_to(&mut handle, content)
}

/// Writes a formatted line to stderr without an intermediate `String` (G-MAC-01).
///
/// # Errors
/// Returns I/O errors except BrokenPipe (treated as Ok — downstream closed).
pub fn write_stderr_fmt(args: fmt::Arguments<'_>) -> io::Result<()> {
    let stderr = io::stderr();
    let mut handle = stderr.lock();
    write_stderr_line_to_fmt(&mut handle, args)
}

/// Shared stderr diagnostic when a typed JSON emit fails to serialize.
pub(crate) fn report_json_serialize_error(err: &impl fmt::Display) {
    let _ = write_stderr_fmt(format_args!("failed to serialize JSON: {err}"));
}

/// Prints a human success message (suppressed with `--quiet`).
pub fn print_success(message: &str) {
    if is_quiet() {
        return;
    }
    write_line_human(message);
}

/// Human success via `format_args!` — no intermediate `String` (G-MAC-01).
pub fn print_success_fmt(args: fmt::Arguments<'_>) {
    if is_quiet() {
        return;
    }
    match write_line_fmt(args) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {}
        Err(_) => {}
    }
}

/// Agent-first success emitter (GAP-AUD-003/008).
///
/// When `json` is true, writes a single **compact** stdout envelope:
/// `{ "ok": true, "event": <event>, …fields }`.
/// Otherwise prints the human `message` (respecting `--quiet`).
///
/// # Errors
/// Returns I/O errors from writing stdout (including BrokenPipe for JSON path).
pub fn emit_success(
    event: &str,
    fields: serde_json::Value,
    human: &str,
    json: bool,
) -> io::Result<()> {
    if json {
        let envelope = SuccessEnvelope::from_value(event, fields);
        json_wire::print_json_line(&envelope)?;
    } else {
        print_success(human);
    }
    Ok(())
}

/// Like [`emit_success`], but the human line is built with `format_args!` (G-MAC-01).
///
/// Prefer this when the human text is dynamic and the JSON path does not need
/// the formatted string (avoids allocating when `json` is true *and* when false).
///
/// # Errors
/// Returns I/O errors from writing stdout (including BrokenPipe for JSON path).
pub fn emit_success_fmt(
    event: &str,
    fields: serde_json::Value,
    human: fmt::Arguments<'_>,
    json: bool,
) -> io::Result<()> {
    if json {
        let envelope = SuccessEnvelope::from_value(event, fields);
        json_wire::print_json_line(&envelope)?;
    } else {
        print_success_fmt(human);
    }
    Ok(())
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
    // G-AUD-12: no FORCE_TEXT env — human banner only on TTY text path above.
    write_line_human(message);
}

/// Prints an error message on stderr (human-facing).
///
/// # Errors
/// Propagates non-pipe stderr write failures.
pub fn print_error(message: &str) -> io::Result<()> {
    write_stderr_line(message)
}

/// Stderr error via `format_args!` — no intermediate `String` (G-MAC-01).
///
/// # Errors
/// Propagates non-pipe stderr write failures.
pub fn print_error_fmt(args: fmt::Arguments<'_>) -> io::Result<()> {
    write_stderr_fmt(args)
}

/// Prints a warning on stderr (agent-visible, never stdout).
pub fn print_warning(message: &str) {
    // G-MAC-01: `format_args!` + `write_fmt` — no temporary `String`.
    let _ = write_stderr_fmt(format_args!("warning: {message}"));
}

/// Warning with dynamic body via `format_args!` (G-MAC-01 residual close).
///
/// `fmt::Arguments` implements [`Display`], so the `"warning: "` prefix composes
/// without a second allocation.
pub fn print_warning_fmt(args: fmt::Arguments<'_>) {
    let _ = write_stderr_fmt(format_args!("warning: {args}"));
}

/// Emits a JSON error envelope on stderr (GAP-SSH-IO-003 / G-RETRY / G-ERR-08).
pub fn print_error_envelope(
    exit_code: i32,
    error_code: &str,
    message: &str,
    remote_exit_code: Option<i32>,
    error_class: crate::errors::ErrorClass,
    retryable: bool,
    suggestion: Option<&str>,
) -> io::Result<()> {
    let env = ErrorEnvelope {
        exit_code,
        error_code: error_code.to_string(),
        message: message.to_string(),
        remote_exit_code,
        error_class,
        retryable,
        suggestion: suggestion.map(str::to_string),
    };
    // Fallback if serialization ever fails (should not for this plain struct).
    match json_wire::print_json_line_stderr(&env) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::Other => write_stderr_fmt(format_args!(
            r#"{{"exit_code":{exit_code},"message":"serialization error"}}"#
        )),
        Err(e) => Err(e as io::Error),
    }
}

/// Prints **compact** JSON on stdout (agent wire; always respects quiet=false).
///
/// Prefer typed DTOs in [`crate::json_wire`] for known payloads. This helper
/// remains for dynamic documents (`meta command-tree`, doctor ad-hoc maps).
///
/// # Errors
/// Serialization or stdout I/O (including BrokenPipe → exit 141).
pub fn print_json_value(v: &serde_json::Value) -> io::Result<()> {
    json_wire::print_json_line(v)
}
