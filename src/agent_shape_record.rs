// SPDX-License-Identifier: MIT OR Apache-2.0
// G-AGENT-01: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! Shaping of payloads that carry no record collection.
//!
//! # Why this is a sibling module
//!
//! [`crate::agent_shape`] owns the collection pipeline (filter, sort, dedupe, limit,
//! select, truncate). The paths here answer a different question: what does a shaping
//! flag mean when there is no collection at all, and where does the accounting go when
//! the payload has no room for it. Keeping both in one file pushed it past the
//! component budget, so the record paths live here and the collection pipeline stays
//! where it was.
//!
//! # Two payloads, two carriers
//!
//! A bare top-level array cannot gain a field without changing its type, and consumers
//! index it. Its reduction is therefore reported on **stderr** as one JSON line tagged
//! `"event":"agent-shape"`. A single-record object can gain `agent_shape` inline, and
//! that key is also what marks the payload as the reduced shape.

use serde_json::{Map, Value};

use crate::agent_shape::{project, truncate_strings, ShapeConfig, ShapeReport};

/// Key holding the shaping report added to a reduced payload.
pub(crate) const REPORT_KEY: &str = "agent_shape";

/// Discriminator carried by the stderr report of a bare-array reduction.
///
/// It differs from `error-envelope` on purpose: a consumer that reads stderr must be
/// able to tell a successful reduction from a failure without inspecting the exit code
/// first.
const REPORT_EVENT: &str = "agent-shape";

/// Key naming the event in the stderr report.
const EVENT_KEY: &str = "event";

/// Emits the bare-array reduction report on stderr as one JSON line.
///
/// A previous version only wrote this to `tracing::info!`, and the default filter is
/// `error`, so the count was invisible on every successful run: the module promised
/// "truncation is never silent" while the default path was exactly that. stderr is the
/// right carrier because stdout must keep the array's type, and `--quiet` is the
/// documented way to ask for a silent stream.
pub(crate) fn emit_array_report(report: &ShapeReport, byte_capped: bool) {
    tracing::info!(
        input_count = report.input_count,
        output_count = report.output_count,
        dropped = report.dropped(),
        content_truncated = report.content_truncated,
        output_truncated = byte_capped,
        "agent-shape reduced the payload"
    );

    if crate::output::is_quiet() {
        return;
    }

    let mut out = Map::new();
    out.insert(EVENT_KEY.to_string(), Value::from(REPORT_EVENT));
    out.insert("input_count".to_string(), Value::from(report.input_count));
    out.insert("output_count".to_string(), Value::from(report.output_count));
    out.insert("dropped".to_string(), Value::from(report.dropped()));
    if report.content_truncated {
        out.insert("content_truncated".to_string(), Value::Bool(true));
    }
    if byte_capped {
        out.insert("output_truncated".to_string(), Value::Bool(true));
    }

    // Losing the report is not worth failing a successful command over: stderr may be
    // closed by the consumer, and the payload on stdout is already correct.
    if let Err(err) = crate::json_wire::print_json_line_stderr(&Value::Object(out)) {
        tracing::warn!(error = %err, "could not write the agent-shape report to stderr");
    }
}

/// Computes the operations in `cfg` that cannot apply to a single record.
///
/// Kept as its own function so the message can name exactly the flags the caller
/// passed. Reporting "some flags were ignored" would leave the caller guessing which
/// part of the request went unanswered.
fn inapplicable_operations(cfg: &ShapeConfig) -> Vec<&'static str> {
    let mut ignored = Vec::new();
    if !cfg.filters.is_empty() {
        ignored.push("--filter");
    }
    if cfg.limit.is_some() {
        ignored.push("--limit");
    }
    if cfg.sort.is_some() {
        ignored.push("--sort");
    }
    if cfg.dedupe_by.is_some() {
        ignored.push("--dedupe-by");
    }
    if cfg.count_only {
        ignored.push("--count-only");
    }
    if cfg.max_output_bytes.is_some() {
        ignored.push("--max-output-bytes");
    }
    ignored
}

/// Whether any collection-only flag was passed, so there is something to name.
///
/// Kept as a predicate instead of `inapplicable_operations(cfg).is_empty()` so the
/// early return in `shape_single_object` does not build a `Vec` just to ask a yes/no
/// question on every single-record payload.
fn has_inapplicable_operations(cfg: &ShapeConfig) -> bool {
    !cfg.filters.is_empty()
        || cfg.limit.is_some()
        || cfg.sort.is_some()
        || cfg.dedupe_by.is_some()
        || cfg.count_only
        || cfg.max_output_bytes.is_some()
}

/// Reduces a single-record envelope by projection and content truncation.
///
/// Returns `false` when the caller asked for nothing that a single record can honour
/// and nothing that it cannot, so an unshaped run stays byte for byte identical. A
/// collection flag alone still returns `true`, because naming it is the answer.
pub(crate) fn shape_single_object(map: &mut Map<String, Value>, cfg: &ShapeConfig) -> bool {
    // A collection flag passed on a single record cannot reduce anything, and the
    // published contract says it is named in `agent_shape.ignored` instead of being
    // dropped in silence. Returning early whenever neither `--select` nor
    // `--truncate-content` was passed left `ssh-cli doctor --limit 2` byte identical to
    // an unshaped run, so the very flag the contract promised to name vanished.
    if cfg.select.is_empty() && cfg.truncate_content.is_none() && !has_inapplicable_operations(cfg)
    {
        return false;
    }

    let mut content_truncated = false;

    // Truncate the whole record first, then project: reversing the order would leave a
    // long string outside the selection untouched while one inside it got cut after
    // the projection had already copied it.
    if let Some(max) = cfg.truncate_content {
        let mut holder = Value::Object(std::mem::take(map));
        truncate_strings(&mut holder, max, &mut content_truncated);
        // `truncate_strings` never replaces an object, so the match is total by
        // construction; the fallback keeps the payload instead of dropping it.
        if let Value::Object(restored) = holder {
            *map = restored;
        }
    }

    if !cfg.select.is_empty() {
        let snapshot = Value::Object(map.clone());
        if let Value::Object(selected) = project(&snapshot, &cfg.select) {
            // The report is appended after projection, so a `--select` that does not
            // name it still receives the accounting. Without this an agent could ask
            // for one field, get it, and have no way to learn that its selection was
            // the reason the payload shrank.
            *map = selected;
        }
    }

    let mut report = Map::new();
    // Always present on a reduced record, even when the only thing that happened was
    // a projection: this key is also what tells a consumer which of the two published
    // shapes it is holding, so omitting it would make the reduced body indistinguishable
    // from a full one that happened to lose fields.
    if !cfg.select.is_empty() {
        report.insert(
            "selected".to_string(),
            Value::Array(cfg.select.iter().map(|p| Value::from(p.as_str())).collect()),
        );
    }
    if content_truncated {
        report.insert("content_truncated".to_string(), Value::Bool(true));
    }

    let ignored = inapplicable_operations(cfg);
    if !ignored.is_empty() {
        report.insert(
            "ignored".to_string(),
            Value::Array(ignored.iter().map(|f| Value::from(*f)).collect()),
        );
    }

    map.insert(REPORT_KEY.to_string(), Value::Object(report));

    true
}
