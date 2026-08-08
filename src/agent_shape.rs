// SPDX-License-Identifier: MIT OR Apache-2.0
// G-AGENT-01: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! Agent-native payload shaping applied at the single JSON serialization funnel.
//!
//! # Why this exists
//!
//! `ssh-cli` emits structured JSON for fan-out commands (`vps list`,
//! `health-check --all`, `exec --all`, the SCP/SFTP batches, `sftp ls`). Without a
//! reduction surface the agent receives the *whole* envelope and has to shell out to
//! `jaq` to cut it down — which means the large payload is built, serialized, written
//! and then thrown away. The tokens are already spent by then.
//!
//! Shaping therefore happens **before** serialization, inside
//! [`crate::json_wire::print_json_line`], so the oversized envelope never reaches
//! stdout in the first place.
//!
//! # Operation order is fixed
//!
//! `filter` → `sort` → `dedupe` → `limit` → `select` → `count-only` →
//! `truncate-content` → `max-output-bytes`
//!
//! The order matters: filtering before limiting keeps the first N *matching* records
//! rather than filtering an arbitrary prefix, and selecting after sorting lets a run
//! sort by a field it does not intend to emit.
//!
//! # Cost when unused
//!
//! [`crate::agent_shape::is_active`] short-circuits the whole path. When no shaping flag was passed the
//! caller serializes exactly as before, so the default path pays no
//! `serde_json::to_value` round-trip.
//!
//! # Truncation is never silent
//!
//! Any run that drops or shortens data reports it under the `agent_shape` key, so an
//! agent can tell "three hosts" apart from "three hosts shown out of ninety".

use serde_json::{Map, Value};
use std::sync::Mutex;

/// Envelope keys inspected, in order, when locating the shapeable array.
///
/// Fan-out payloads in this crate name their collection differently per command;
/// probing a fixed list keeps shaping generic without each call site opting in.
const ARRAY_KEYS: &[&str] = &[
    "results", "items", "hosts", "entries", "vps", "matches", "rows", "data", "steps",
];

/// Key holding the shaping report added to a reduced envelope.
const REPORT_KEY: &str = "agent_shape";

/// A single `--filter` predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    /// Dotted path into each element.
    pub path: String,
    /// Comparison to apply.
    pub op: FilterOp,
    /// Right-hand side, compared as a string.
    pub value: String,
}

/// Comparison used by a [`Filter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    /// `key=value` (also accepted as `key==value`).
    Equals,
    /// `key!=value`.
    NotEquals,
    /// `key~substring`.
    Contains,
}

impl Filter {
    /// Parses `key=value`, `key!=value` or `key~substring`.
    ///
    /// # Errors
    /// Returns a human-readable message when no operator is present or the key is
    /// empty. Failing loudly matters: a typo that silently matched nothing would look
    /// exactly like a legitimately empty result.
    pub fn parse(raw: &str) -> Result<Self, String> {
        if let Some((k, v)) = raw.split_once("!=") {
            return Self::build(k, FilterOp::NotEquals, v);
        }
        if let Some((k, v)) = raw.split_once("==") {
            return Self::build(k, FilterOp::Equals, v);
        }
        if let Some((k, v)) = raw.split_once('~') {
            return Self::build(k, FilterOp::Contains, v);
        }
        if let Some((k, v)) = raw.split_once('=') {
            return Self::build(k, FilterOp::Equals, v);
        }
        Err(format!(
            "invalid --filter `{raw}`: expected key=value, key!=value or key~substring"
        ))
    }

    fn build(key: &str, op: FilterOp, value: &str) -> Result<Self, String> {
        let key = key.trim();
        if key.is_empty() {
            return Err("invalid --filter: empty key".to_string());
        }
        Ok(Self {
            path: key.to_string(),
            op,
            value: value.to_string(),
        })
    }

    fn matches(&self, element: &Value) -> bool {
        let actual = lookup(element, &self.path).map(scalar_to_string);
        match (&self.op, actual) {
            // A missing field never satisfies a predicate, not even `!=`. Treating
            // absence as "different" would silently promote incomplete records.
            (_, None) => false,
            (FilterOp::Equals, Some(a)) => a == self.value,
            (FilterOp::NotEquals, Some(a)) => a != self.value,
            (FilterOp::Contains, Some(a)) => a.contains(&self.value),
        }
    }
}

/// Shaping options resolved from global CLI flags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShapeConfig {
    /// Dotted paths to keep in each element (`--select` / `--fields`).
    pub select: Vec<String>,
    /// Conjunctive predicates (`--filter`, repeatable).
    pub filters: Vec<Filter>,
    /// Max elements emitted (`--limit`).
    pub limit: Option<usize>,
    /// Dotted path to sort ascending by (`--sort`).
    pub sort: Option<String>,
    /// Dotted path to deduplicate by (`--dedupe-by`).
    pub dedupe_by: Option<String>,
    /// Replace the payload with `{"count": N}` (`--count-only`).
    pub count_only: bool,
    /// Shorten strings above this many characters (`--truncate-content`).
    pub truncate_content: Option<usize>,
    /// Drop trailing elements until the envelope fits (`--max-output-bytes`).
    pub max_output_bytes: Option<usize>,
}

impl ShapeConfig {
    /// Whether any shaping was requested.
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.select.is_empty()
            || !self.filters.is_empty()
            || self.limit.is_some()
            || self.sort.is_some()
            || self.dedupe_by.is_some()
            || self.count_only
            || self.truncate_content.is_some()
            || self.max_output_bytes.is_some()
    }
}

static SHAPE: Mutex<Option<ShapeConfig>> = Mutex::new(None);

fn lock_shape() -> std::sync::MutexGuard<'static, Option<ShapeConfig>> {
    SHAPE.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("agent-shape mutex was poisoned; recovering (one-shot CLI)");
        poisoned.into_inner()
    })
}

/// Installs the process-wide shaping configuration (called once after argv parse).
pub fn set_shape(cfg: ShapeConfig) {
    *lock_shape() = if cfg.is_active() { Some(cfg) } else { None };
}

/// Whether shaping is active for this process.
#[must_use]
pub fn is_active() -> bool {
    lock_shape().is_some()
}

/// Returns a clone of the active configuration, if any.
#[must_use]
pub fn current() -> Option<ShapeConfig> {
    lock_shape().clone()
}

/// Resolves a dotted path such as `host.port` against a value.
fn lookup<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = value;
    for segment in path.split('.') {
        cur = cur.as_object()?.get(segment)?;
    }
    Some(cur)
}

/// Renders a scalar for comparison; containers never match a predicate.
fn scalar_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Orders two values, comparing numbers numerically and everything else as text.
fn compare(a: Option<&Value>, b: Option<&Value>) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        // Elements missing the sort key sink to the end instead of pretending to be
        // the smallest value.
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(x), Some(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(nx), Some(ny)) => nx.partial_cmp(&ny).unwrap_or(Ordering::Equal),
            _ => scalar_to_string(x).cmp(&scalar_to_string(y)),
        },
    }
}

/// Keeps only the selected dotted paths, preserving nesting.
fn project(element: &Value, paths: &[String]) -> Value {
    let mut out = Map::new();
    for path in paths {
        // A path that does not resolve is skipped rather than emitted as `null`;
        // a null would be indistinguishable from a field that genuinely is null.
        if let Some(found) = lookup(element, path) {
            insert_path(&mut out, path, found.clone());
        }
    }
    Value::Object(out)
}

fn insert_path(target: &mut Map<String, Value>, path: &str, value: Value) {
    let mut segments = path.split('.').peekable();
    let mut cursor = target;
    while let Some(seg) = segments.next() {
        if segments.peek().is_none() {
            cursor.insert(seg.to_string(), value);
            return;
        }
        let entry = cursor
            .entry(seg.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        match entry.as_object_mut() {
            Some(next) => cursor = next,
            None => return,
        }
    }
}

/// Shortens every string longer than `max` characters, on char boundaries.
fn truncate_strings(value: &mut Value, max: usize, changed: &mut bool) {
    match value {
        Value::String(s) => {
            // Count characters, never bytes: cutting at a byte offset would split a
            // multi-byte sequence and produce invalid UTF-8 in the envelope.
            if s.chars().count() > max {
                let cut: String = s.chars().take(max).collect();
                *s = cut;
                *changed = true;
            }
        }
        Value::Array(items) => {
            for item in items {
                truncate_strings(item, max, changed);
            }
        }
        Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                truncate_strings(v, max, changed);
            }
        }
        _ => {}
    }
}

/// Locates the shapeable array inside an envelope.
fn find_array_key(map: &Map<String, Value>) -> Option<String> {
    ARRAY_KEYS
        .iter()
        .find(|k| map.get(**k).is_some_and(Value::is_array))
        .map(|k| (*k).to_string())
}

/// Outcome of shaping a record collection.
#[derive(Debug, Clone, Copy, Default)]
struct ShapeReport {
    input_count: usize,
    output_count: usize,
    content_truncated: bool,
}

impl ShapeReport {
    fn dropped(&self) -> usize {
        self.input_count.saturating_sub(self.output_count)
    }

    fn changed_anything(&self) -> bool {
        self.dropped() > 0 || self.content_truncated
    }
}

/// Applies filter → sort → dedupe → limit → select → truncate to a record list.
fn shape_items(items: &mut Vec<Value>, cfg: &ShapeConfig) -> ShapeReport {
    let input_count = items.len();
    let mut content_truncated = false;

    if !cfg.filters.is_empty() {
        items.retain(|item| cfg.filters.iter().all(|f| f.matches(item)));
    }

    if let Some(path) = &cfg.sort {
        items.sort_by(|a, b| compare(lookup(a, path), lookup(b, path)));
    }

    if let Some(path) = &cfg.dedupe_by {
        let mut seen = std::collections::HashSet::new();
        items.retain(|item| match lookup(item, path) {
            // Elements without the key are always kept: dropping them would delete
            // records for lacking a field rather than for being duplicates.
            None => true,
            Some(v) => seen.insert(scalar_to_string(v)),
        });
    }

    if let Some(limit) = cfg.limit {
        items.truncate(limit);
    }

    if !cfg.select.is_empty() {
        for item in items.iter_mut() {
            *item = project(item, &cfg.select);
        }
    }

    if let Some(max) = cfg.truncate_content {
        for item in items.iter_mut() {
            truncate_strings(item, max, &mut content_truncated);
        }
    }

    ShapeReport {
        input_count,
        output_count: items.len(),
        content_truncated,
    }
}

/// Applies the active shaping to a serialized payload, in the documented order.
///
/// Handles both payload shapes this CLI emits: a bare top-level array (`vps list`,
/// `sftp ls`) and an envelope object wrapping a named collection (the batch events).
/// Missing the bare-array case would have left shaping inert on the most frequently
/// used commands.
///
/// Returns `true` when the payload was modified.
pub fn apply(root: &mut Value, cfg: &ShapeConfig) -> bool {
    match root {
        Value::Array(items) => {
            let report = shape_items(items, cfg);
            if cfg.count_only {
                *root = Value::Object({
                    let mut m = Map::new();
                    m.insert("count".to_string(), Value::from(report.output_count));
                    m
                });
                return true;
            }
            let byte_capped = cap_output_bytes(root, cfg);
            // A bare array has nowhere to carry a report object without changing its
            // type and breaking every consumer that indexes it. stdout stays a pure
            // array and the accounting goes to stderr, where diagnostics belong.
            if report.changed_anything() || byte_capped {
                tracing::info!(
                    input_count = report.input_count,
                    output_count = report.output_count,
                    dropped = report.dropped(),
                    content_truncated = report.content_truncated,
                    output_truncated = byte_capped,
                    "agent-shape reduced the payload"
                );
            }
            true
        }
        Value::Object(_) => apply_to_envelope(root, cfg),
        _ => false,
    }
}

/// Shapes the collection inside an envelope object and attaches the report inline.
fn apply_to_envelope(root: &mut Value, cfg: &ShapeConfig) -> bool {
    let Some(map) = root.as_object_mut() else {
        return false;
    };
    let Some(key) = find_array_key(map) else {
        // Envelopes without a collection (single-record reads, ready events) are left
        // untouched; there is nothing to reduce and rewriting them would only risk
        // breaking their schema.
        return false;
    };
    let Some(Value::Array(items)) = map.get_mut(&key) else {
        return false;
    };

    let report = shape_items(items, cfg);
    let mut output_count = report.output_count;

    let mut byte_capped = false;
    if cfg.count_only {
        // Counted after every other stage, so the number describes what *would* have
        // been emitted rather than the raw input. The collection itself is dropped.
        map.remove(&key);
        map.insert("count".to_string(), Value::from(output_count));
    } else if let Some(max_bytes) = cfg.max_output_bytes {
        // Drop whole trailing elements rather than slicing the serialized text: a
        // byte-sliced envelope would not parse as JSON at all.
        loop {
            let too_big = serde_json::to_string(&Value::Object(map.clone()))
                .map(|s| s.len() > max_bytes)
                .unwrap_or(false);
            if !too_big {
                break;
            }
            let Some(Value::Array(items)) = map.get_mut(&key) else {
                break;
            };
            if items.pop().is_none() {
                break;
            }
            byte_capped = true;
            output_count = items.len();
        }
    }

    let mut out = Map::new();
    out.insert("input_count".to_string(), Value::from(report.input_count));
    out.insert("output_count".to_string(), Value::from(output_count));
    out.insert(
        "dropped".to_string(),
        Value::from(report.input_count.saturating_sub(output_count)),
    );
    if report.content_truncated {
        out.insert("content_truncated".to_string(), Value::Bool(true));
    }
    if byte_capped {
        out.insert("output_truncated".to_string(), Value::Bool(true));
    }
    map.insert(REPORT_KEY.to_string(), Value::Object(out));
    true
}

/// Drops trailing elements of a top-level array until it fits `max_output_bytes`.
fn cap_output_bytes(root: &mut Value, cfg: &ShapeConfig) -> bool {
    let Some(max_bytes) = cfg.max_output_bytes else {
        return false;
    };
    let mut capped = false;
    loop {
        let too_big = serde_json::to_string(&*root)
            .map(|s| s.len() > max_bytes)
            .unwrap_or(false);
        if !too_big {
            return capped;
        }
        let Some(items) = root.as_array_mut() else {
            return capped;
        };
        if items.pop().is_none() {
            return capped;
        }
        capped = true;
    }
}

#[cfg(test)]
#[path = "agent_shape_tests.rs"]
mod tests;
