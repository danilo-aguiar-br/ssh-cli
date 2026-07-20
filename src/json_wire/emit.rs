// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: compact JSON emit primitives + envelopes (extracted from json_wire monólito).
#![forbid(unsafe_code)]
//! Compact JSON + LF writers and agent error/success envelopes.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{self, Write};

/// UTF-8 BOM character; stripped before parsing external JSON.
pub const UTF8_BOM: char = '\u{feff}';

/// Strips a leading UTF-8 BOM if present (Rules: remove BOM before parse).
///
/// # Examples
///
/// ```
/// use ssh_cli::json_wire::{strip_utf8_bom, UTF8_BOM};
///
/// assert_eq!(strip_utf8_bom("{\"ok\":true}"), "{\"ok\":true}");
/// let with_bom = format!("{UTF8_BOM}{{\"ok\":true}}");
/// assert_eq!(strip_utf8_bom(&with_bom), "{\"ok\":true}");
/// ```
#[must_use]
pub fn strip_utf8_bom(s: &str) -> &str {
    s.strip_prefix(UTF8_BOM).unwrap_or(s)
}

/// Serializes `value` as **compact** JSON + trailing LF on the given writer.
///
/// DI primitive (G-IO-11): pass a `Cursor`/`Vec` in tests; production uses
/// process stdout/stderr via [`print_json_line`] / [`print_json_line_stderr`].
///
/// # Examples
///
/// ```
/// use ssh_cli::json_wire::write_json_line;
/// use serde_json::json;
/// use std::io::Cursor;
///
/// let mut buf = Cursor::new(Vec::new());
/// write_json_line(&mut buf, &json!({"ok": true})).unwrap();
/// let s = String::from_utf8(buf.into_inner()).unwrap();
/// assert_eq!(s, "{\"ok\":true}\n");
/// assert!(!s.contains('\r'));
/// ```
///
/// # Errors
/// Serialization failure or I/O (including `BrokenPipe`).
pub fn write_json_line<W: Write, T: Serialize>(mut w: W, value: &T) -> io::Result<()> {
    let s = serde_json::to_string(value).map_err(io::Error::other)?;
    w.write_all(s.as_bytes())?;
    w.write_all(b"\n")?;
    w.flush()?;
    Ok(())
}

/// Compact JSON + LF on stdout (agent success / data path).
///
/// # Errors
/// Serialization or stdout I/O (including `BrokenPipe` → exit 141).
pub fn print_json_line<T: Serialize>(value: &T) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_json_line(&mut handle, value)
}

/// Compact JSON + LF on stderr; BrokenPipe is ignored (downstream closed).
///
/// # Errors
/// Non-pipe stderr write failures.
pub fn print_json_line_stderr<T: Serialize>(value: &T) -> io::Result<()> {
    let stderr = io::stderr();
    let mut handle = stderr.lock();
    match write_json_line(&mut handle, value) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e),
    }
}

// ---------------------------------------------------------------------------
// Error envelope (stderr)
// ---------------------------------------------------------------------------

/// Stderr failure envelope when JSON errors mode is active.
///
/// Agents must read `retryable` / `error_class` before re-invoking (Rules Rust —
/// retry/backoff). Historical clients that only inspect `exit_code` remain valid
/// (`additionalProperties` / unknown-field ignore on the consumer side).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorEnvelope {
    /// Process exit code (sysexits-inspired).
    pub exit_code: i32,
    /// Stable machine code (`vps_not_found`, `tls`, …) — G-ERR-08.
    #[serde(default)]
    pub error_code: String,
    /// Human-readable message (may be localized).
    pub message: String,
    /// Optional remote shell exit when process exit is general failure.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub remote_exit_code: Option<i32>,
    /// High-level class (`transient` | `permanent` | `cancelled`).
    pub error_class: crate::errors::ErrorClass,
    /// Whether an agent may re-invoke with the same argv after backoff.
    pub retryable: bool,
    /// Optional short remediation hint for agents.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub suggestion: Option<String>,
}

// ---------------------------------------------------------------------------
// Success envelope (stdout)
// ---------------------------------------------------------------------------

/// Agent-first success envelope: `{ "ok": true, "event": …, …fields }`.
///
/// Extra fields are merged from a map so callers can attach event-specific keys
/// without proliferating one struct per CRUD event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessEnvelope {
    /// Always `true` for success envelopes.
    pub ok: bool,
    /// Event discriminator (`vps-added`, `scp-transfer`, …).
    pub event: String,
    /// Additional event fields (flattened at serialize time via map merge).
    #[serde(flatten)]
    pub fields: BTreeMap<String, serde_json::Value>,
}

impl SuccessEnvelope {
    /// Builds a success envelope from an event name and extra fields.
    #[must_use]
    pub fn new(event: impl Into<String>, fields: BTreeMap<String, serde_json::Value>) -> Self {
        Self {
            ok: true,
            event: event.into(),
            fields,
        }
    }

    /// Builds from a `serde_json::Value` object (or wraps non-objects under `data`).
    #[must_use]
    pub fn from_value(event: &str, fields: serde_json::Value) -> Self {
        let mut map = BTreeMap::new();
        match fields {
            serde_json::Value::Object(obj) => {
                for (k, v) in obj {
                    map.insert(k, v);
                }
            }
            other => {
                map.insert("data".into(), other);
            }
        }
        Self::new(event, map)
    }
}
