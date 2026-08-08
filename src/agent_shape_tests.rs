// SPDX-License-Identifier: MIT OR Apache-2.0
//! Unit tests for the agent-native shaping layer.

use super::*;
use serde_json::json;

fn envelope() -> Value {
    json!({
        "ok": true,
        "results": [
            {"name": "alpha", "port": 22,   "ok": true,  "detail": "connected fine"},
            {"name": "bravo", "port": 2222, "ok": false, "detail": "auth rejected"},
            {"name": "charlie", "port": 22, "ok": true,  "detail": "connected fine"},
        ]
    })
}

fn shaped(cfg: ShapeConfig) -> Value {
    let mut v = envelope();
    assert!(apply(&mut v, &cfg), "shaping must report a change");
    v
}

fn names(v: &Value) -> Vec<String> {
    v["results"]
        .as_array()
        .expect("results array")
        .iter()
        .map(|e| e["name"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn filter_equals_keeps_only_matching() {
    let cfg = ShapeConfig {
        filters: vec![Filter::parse("ok=true").expect("parse")],
        ..Default::default()
    };
    assert_eq!(names(&shaped(cfg)), vec!["alpha", "charlie"]);
}

#[test]
fn filter_not_equals_excludes_matching() {
    let cfg = ShapeConfig {
        filters: vec![Filter::parse("name!=bravo").expect("parse")],
        ..Default::default()
    };
    assert_eq!(names(&shaped(cfg)), vec!["alpha", "charlie"]);
}

#[test]
fn filter_contains_matches_substring() {
    let cfg = ShapeConfig {
        filters: vec![Filter::parse("detail~auth").expect("parse")],
        ..Default::default()
    };
    assert_eq!(names(&shaped(cfg)), vec!["bravo"]);
}

#[test]
fn filters_combine_with_and() {
    let cfg = ShapeConfig {
        filters: vec![
            Filter::parse("ok=true").expect("parse"),
            Filter::parse("port=22").expect("parse"),
        ],
        ..Default::default()
    };
    assert_eq!(names(&shaped(cfg)), vec!["alpha", "charlie"]);
}

#[test]
fn missing_field_never_matches_even_under_not_equals() {
    // A record lacking the key must not sneak through `!=`; absence is not difference.
    let cfg = ShapeConfig {
        filters: vec![Filter::parse("absent!=whatever").expect("parse")],
        ..Default::default()
    };
    let mut v = envelope();
    apply(&mut v, &cfg);
    assert!(names(&v).is_empty());
}

#[test]
fn malformed_filter_is_rejected_not_silently_empty() {
    // Failing loudly matters: a typo that matched nothing would look exactly like a
    // legitimately empty result.
    assert!(Filter::parse("no-operator-here").is_err());
    assert!(Filter::parse("=orphan").is_err());
}

#[test]
fn limit_caps_emitted_elements() {
    let cfg = ShapeConfig {
        limit: Some(2),
        ..Default::default()
    };
    assert_eq!(names(&shaped(cfg)).len(), 2);
}

#[test]
fn filter_runs_before_limit() {
    // Limiting first would return an arbitrary prefix and then filter it, yielding
    // fewer records than requested even when enough matches exist.
    let cfg = ShapeConfig {
        filters: vec![Filter::parse("ok=true").expect("parse")],
        limit: Some(2),
        ..Default::default()
    };
    assert_eq!(names(&shaped(cfg)), vec!["alpha", "charlie"]);
}

#[test]
fn sort_orders_numbers_numerically() {
    let cfg = ShapeConfig {
        sort: Some("port".to_string()),
        ..Default::default()
    };
    let v = shaped(cfg);
    let ports: Vec<i64> = v["results"]
        .as_array()
        .expect("array")
        .iter()
        .map(|e| e["port"].as_i64().unwrap_or_default())
        .collect();
    assert_eq!(ports, vec![22, 22, 2222], "2222 must not sort before 22");
}

#[test]
fn elements_missing_sort_key_go_last() {
    let mut v = json!({"results": [{"a": 1}, {"b": 2}, {"a": 0}]});
    let cfg = ShapeConfig {
        sort: Some("a".to_string()),
        ..Default::default()
    };
    apply(&mut v, &cfg);
    let arr = v["results"].as_array().expect("array");
    assert!(arr[2].get("a").is_none(), "missing key sinks to the end");
}

#[test]
fn dedupe_keeps_first_occurrence() {
    let cfg = ShapeConfig {
        dedupe_by: Some("detail".to_string()),
        ..Default::default()
    };
    assert_eq!(names(&shaped(cfg)), vec!["alpha", "bravo"]);
}

#[test]
fn dedupe_keeps_elements_without_the_key() {
    let mut v = json!({"results": [{"k": "x"}, {"other": 1}, {"other": 2}, {"k": "x"}]});
    let cfg = ShapeConfig {
        dedupe_by: Some("k".to_string()),
        ..Default::default()
    };
    apply(&mut v, &cfg);
    // Two keyless records survive: they are not duplicates, they merely lack the key.
    assert_eq!(v["results"].as_array().expect("array").len(), 3);
}

#[test]
fn select_projects_only_requested_paths() {
    let cfg = ShapeConfig {
        select: vec!["name".to_string()],
        ..Default::default()
    };
    let v = shaped(cfg);
    let first = &v["results"][0];
    assert!(first.get("name").is_some());
    assert!(first.get("port").is_none());
    assert!(first.get("detail").is_none());
}

#[test]
fn select_supports_dotted_paths_and_preserves_nesting() {
    let mut v = json!({"results": [{"host": {"addr": "1.2.3.4", "port": 22}, "extra": 9}]});
    let cfg = ShapeConfig {
        select: vec!["host.addr".to_string()],
        ..Default::default()
    };
    apply(&mut v, &cfg);
    assert_eq!(v["results"][0]["host"]["addr"], json!("1.2.3.4"));
    assert!(v["results"][0]["host"].get("port").is_none());
    assert!(v["results"][0].get("extra").is_none());
}

#[test]
fn select_skips_unresolved_path_instead_of_emitting_null() {
    // A `null` would be indistinguishable from a field that genuinely is null.
    let cfg = ShapeConfig {
        select: vec!["name".to_string(), "nope".to_string()],
        ..Default::default()
    };
    let v = shaped(cfg);
    assert!(v["results"][0].get("nope").is_none());
}

#[test]
fn count_only_replaces_collection_with_count() {
    let cfg = ShapeConfig {
        count_only: true,
        ..Default::default()
    };
    let v = shaped(cfg);
    assert_eq!(v["count"], json!(3));
    assert!(v.get("results").is_none());
}

#[test]
fn count_only_counts_after_filtering() {
    let cfg = ShapeConfig {
        filters: vec![Filter::parse("ok=true").expect("parse")],
        count_only: true,
        ..Default::default()
    };
    assert_eq!(shaped(cfg)["count"], json!(2));
}

#[test]
fn truncate_content_counts_characters_not_bytes() {
    // Cutting at a byte offset would split a multi-byte sequence and emit invalid UTF-8.
    let mut v = json!({"results": [{"s": "áéíóúçãõ"}]});
    let cfg = ShapeConfig {
        truncate_content: Some(3),
        ..Default::default()
    };
    apply(&mut v, &cfg);
    let s = v["results"][0]["s"].as_str().expect("string");
    assert_eq!(s.chars().count(), 3);
    assert_eq!(s, "áéí");
}

#[test]
fn truncate_reports_that_it_happened() {
    let mut v = json!({"results": [{"s": "0123456789"}]});
    let cfg = ShapeConfig {
        truncate_content: Some(4),
        ..Default::default()
    };
    apply(&mut v, &cfg);
    assert_eq!(v["agent_shape"]["content_truncated"], json!(true));
}

#[test]
fn max_output_bytes_drops_elements_and_keeps_valid_json() {
    let mut v = envelope();
    let cfg = ShapeConfig {
        max_output_bytes: Some(160),
        ..Default::default()
    };
    apply(&mut v, &cfg);
    let text = serde_json::to_string(&v).expect("serialize");
    assert!(
        serde_json::from_str::<Value>(&text).is_ok(),
        "envelope must stay parseable after byte capping"
    );
    assert_eq!(v["agent_shape"]["output_truncated"], json!(true));
}

#[test]
fn report_declares_input_output_and_dropped() {
    let cfg = ShapeConfig {
        limit: Some(1),
        ..Default::default()
    };
    let v = shaped(cfg);
    assert_eq!(v["agent_shape"]["input_count"], json!(3));
    assert_eq!(v["agent_shape"]["output_count"], json!(1));
    assert_eq!(v["agent_shape"]["dropped"], json!(2));
}

#[test]
fn envelope_without_collection_is_left_untouched() {
    // Single-record reads and ready events have nothing to reduce; rewriting them
    // would only risk breaking their published schema.
    let mut v = json!({"ok": true, "event": "tunnel_listening", "local_port": 8080});
    let before = v.clone();
    let cfg = ShapeConfig {
        limit: Some(1),
        ..Default::default()
    };
    assert!(!apply(&mut v, &cfg));
    assert_eq!(v, before);
}

#[test]
fn inactive_config_is_not_installed() {
    assert!(!ShapeConfig::default().is_active());
    let cfg = ShapeConfig {
        limit: Some(5),
        ..Default::default()
    };
    assert!(cfg.is_active());
}

#[test]
fn filter_parses_all_accepted_operators() {
    assert_eq!(
        Filter::parse("k=v").expect("eq"),
        Filter {
            path: "k".into(),
            op: FilterOp::Equals,
            value: "v".into()
        }
    );
    assert_eq!(Filter::parse("k==v").expect("eq2").op, FilterOp::Equals);
    assert_eq!(Filter::parse("k!=v").expect("ne").op, FilterOp::NotEquals);
    assert_eq!(Filter::parse("k~v").expect("has").op, FilterOp::Contains);
}

// --- bare top-level array (the shape `vps list` and `sftp ls` actually emit) ---

fn bare_array() -> Value {
    json!([
        {"name": "alpha", "port": 22,   "secret": "0123456789"},
        {"name": "bravo", "port": 2222, "secret": "0123456789"},
        {"name": "charlie", "port": 22, "secret": "0123456789"},
    ])
}

#[test]
fn bare_array_payload_is_shaped() {
    // Regression: `apply` originally handled only `Value::Object`, so shaping was
    // inert on `vps list` — the most frequently used structured command.
    let mut v = bare_array();
    let cfg = ShapeConfig {
        select: vec!["name".to_string()],
        limit: Some(2),
        ..Default::default()
    };
    assert!(apply(&mut v, &cfg));
    let arr = v.as_array().expect("must stay a top-level array");
    assert_eq!(arr.len(), 2);
    assert!(
        arr[0].get("port").is_none(),
        "select must drop other fields"
    );
    assert_eq!(arr[0]["name"], json!("alpha"));
}

#[test]
fn bare_array_stays_an_array_so_consumers_keep_indexing() {
    // Attaching a report object would change the payload's type and break every
    // consumer that indexes it; the accounting goes to stderr instead.
    let mut v = bare_array();
    let cfg = ShapeConfig {
        limit: Some(1),
        ..Default::default()
    };
    apply(&mut v, &cfg);
    assert!(v.is_array());
    assert!(v.get(REPORT_KEY).is_none());
}

#[test]
fn bare_array_filter_and_sort_apply() {
    let mut v = bare_array();
    let cfg = ShapeConfig {
        filters: vec![Filter::parse("port=22").expect("parse")],
        sort: Some("name".to_string()),
        ..Default::default()
    };
    apply(&mut v, &cfg);
    let names: Vec<&str> = v
        .as_array()
        .expect("array")
        .iter()
        .map(|e| e["name"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(names, vec!["alpha", "charlie"]);
}

#[test]
fn bare_array_count_only_becomes_count_object() {
    let mut v = bare_array();
    let cfg = ShapeConfig {
        filters: vec![Filter::parse("port=22").expect("parse")],
        count_only: true,
        ..Default::default()
    };
    apply(&mut v, &cfg);
    assert_eq!(v["count"], json!(2));
}

#[test]
fn bare_array_max_output_bytes_keeps_valid_json() {
    let mut v = bare_array();
    let cfg = ShapeConfig {
        max_output_bytes: Some(80),
        ..Default::default()
    };
    apply(&mut v, &cfg);
    let text = serde_json::to_string(&v).expect("serialize");
    assert!(serde_json::from_str::<Value>(&text).is_ok());
    assert!(text.len() <= 80 || v.as_array().is_some_and(std::vec::Vec::is_empty));
}

#[test]
fn bare_array_truncate_content_applies() {
    let mut v = bare_array();
    let cfg = ShapeConfig {
        truncate_content: Some(4),
        ..Default::default()
    };
    apply(&mut v, &cfg);
    assert_eq!(v[0]["secret"].as_str().expect("str").chars().count(), 4);
}

#[test]
fn scalar_payload_is_left_alone() {
    let mut v = json!("just a string");
    let cfg = ShapeConfig {
        limit: Some(1),
        ..Default::default()
    };
    assert!(!apply(&mut v, &cfg));
    assert_eq!(v, json!("just a string"));
}
