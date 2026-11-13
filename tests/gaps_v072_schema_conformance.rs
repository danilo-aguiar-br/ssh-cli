// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real envelopes against the published JSON Schemas in `docs/schemas/`.
//!
//! # Why this file exists
//!
//! Before it, no test compared what the binary prints with what
//! `docs/schemas/*.schema.json` promises. The schemas drifted unnoticed: the
//! `vps-export` host was an opaque `{"type":"object"}`, `use_agent` and
//! `agent_socket` were missing from three of them, and two schemas claimed the
//! same event. A contract nobody checks is documentation, so every case here
//! spawns the real binary with a fresh `--config-dir`, takes the JSON line it
//! printed and validates it against the schema an agent would fetch.
//!
//! # Why a hand-written validator
//!
//! The `jsonschema` crate would pull a large dependency tree, and `deny.toml`
//! rejects duplicate versions, so a transitive clash would break the supply
//! chain gate for the sake of a test. The checker below covers exactly the
//! keywords these schemas use: `type` (single or array), `required`,
//! `properties`, `const`, `enum`, `items`, `additionalProperties` (boolean or
//! schema), `minimum`, `oneOf`, `anyOf` and `$ref`.
//!
//! # What the validator does NOT cover
//!
//! - It does not implement the whole of draft 2020-12.
//! - A schema keyword it does not know FAILS the test with the keyword's path
//!   (`format`, `allOf`, `not`, `if`, `$defs`, `maximum`, ...). A richer schema
//!   can therefore never pass without being checked for real: extend the
//!   validator first.
//! - `pattern` IS enforced, with the `regex` crate. It was the keyword whose
//!   absence let `vps-doctor` carry a regex nobody checked.
//! - `$ref` resolves a local `#/json/pointer` or a sibling `*.schema.json`
//!   file name only. There is no remote lookup and no `$id` base resolution.
//! - `format` and `description` are annotations and are not asserted.
//! - Numbers compare as `f64` for `minimum`, which is exact for every integer
//!   the CLI prints (`u16`, `u32`, `u64` below 2^53).
//!
//! # What is validated
//!
//! `vps rename`, `vps edit --name`, `vps edit` (changing and no-op), the
//! `--dry-run` plans of rename, edit and import, `vps import` success with the
//! `replaced` list, `vps export` (redacted stdout and the `--include-secrets`
//! file body), `vps show` and `vps list`, each with an agent-authenticated host.

use assert_cmd::Command;
use serde_json::{json, Value};
use serial_test::serial;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ── mini validator ─────────────────────────────────────────────────────────

/// Keywords that describe a schema without constraining an instance.
const ANNOTATIONS: &[&str] = &[
    "$schema",
    "$id",
    "$comment",
    "title",
    "description",
    "default",
    "examples",
];

/// Keywords the validator enforces. Anything outside both lists fails loudly.
const ENFORCED: &[&str] = &[
    "type",
    "required",
    "properties",
    "const",
    "enum",
    "items",
    "additionalProperties",
    "minimum",
    "oneOf",
    "anyOf",
    "$ref",
    "pattern",
];

fn schema_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("schemas")
}

/// Reads one schema leaf such as `vps-show.schema.json` from `docs/schemas/`.
fn read_schema_file(leaf: &str) -> Value {
    assert!(
        leaf.ends_with(".schema.json") && !leaf.contains(['/', '\\']),
        "unsupported $ref target `{leaf}`: only sibling *.schema.json leaves resolve"
    );
    let path = schema_dir().join(leaf);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{} is not JSON: {e}", path.display()))
}

fn load_schema(name: &str) -> Value {
    read_schema_file(&format!("{name}.schema.json"))
}

fn one_type(name: &str, v: &Value) -> bool {
    match name {
        "object" => v.is_object(),
        "array" => v.is_array(),
        "string" => v.is_string(),
        "boolean" => v.is_boolean(),
        "null" => v.is_null(),
        "integer" => v.is_i64() || v.is_u64(),
        "number" => v.is_number(),
        _ => false,
    }
}

fn type_matches(t: &Value, v: &Value) -> bool {
    match t {
        Value::String(s) => one_type(s, v),
        Value::Array(list) => list
            .iter()
            .any(|x| x.as_str().is_some_and(|s| one_type(s, v))),
        _ => false,
    }
}

/// Collects every violation of `schema` by `instance` into `out`.
///
/// `root` is the document `schema` belongs to, the target of `#/…` references.
fn check(root: &Value, schema: &Value, instance: &Value, at: &str, out: &mut Vec<String>) {
    let Some(obj) = schema.as_object() else {
        out.push(format!("{at}: schema node is not an object: {schema}"));
        return;
    };
    for key in obj.keys() {
        let k = key.as_str();
        if !ANNOTATIONS.contains(&k) && !ENFORCED.contains(&k) {
            out.push(format!(
                "{at}: unsupported schema keyword `{k}`; extend the validator before relying on it"
            ));
        }
    }

    if let Some(r) = obj.get("$ref") {
        match r.as_str() {
            Some(r) => check_ref(root, r, instance, at, out),
            None => out.push(format!("{at}: `$ref` is not a string: {r}")),
        }
    }

    if let Some(t) = obj.get("type") {
        if !type_matches(t, instance) {
            out.push(format!("{at}: expected type {t}, got {instance}"));
            // Every later check assumes the right type, so one error is enough.
            return;
        }
    }
    if let Some(c) = obj.get("const") {
        if c != instance {
            out.push(format!("{at}: expected const {c}, got {instance}"));
        }
    }
    match obj.get("enum") {
        Some(Value::Array(allowed)) if !allowed.contains(instance) => {
            out.push(format!("{at}: {instance} is not one of {allowed:?}"));
        }
        Some(Value::Array(_)) | None => {}
        Some(other) => out.push(format!("{at}: `enum` is not an array: {other}")),
    }
    if let (Some(pattern), Some(text)) = (
        obj.get("pattern").and_then(Value::as_str),
        instance.as_str(),
    ) {
        match regex::Regex::new(pattern) {
            Ok(re) => {
                if !re.is_match(text) {
                    out.push(format!("{at}: {text:?} does not match pattern {pattern}"));
                }
            }
            Err(e) => out.push(format!("{at}: `pattern` {pattern} is not a regex: {e}")),
        }
    }
    if let (Some(min), Some(n)) = (
        obj.get("minimum").and_then(Value::as_f64),
        instance.as_f64(),
    ) {
        if n < min {
            out.push(format!("{at}: {n} is below minimum {min}"));
        }
    }

    if let Value::Object(map) = instance {
        if let Some(Value::Array(required)) = obj.get("required") {
            for name in required.iter().filter_map(Value::as_str) {
                if !map.contains_key(name) {
                    out.push(format!("{at}: missing required property `{name}`"));
                }
            }
        }
        let props = obj.get("properties").and_then(Value::as_object);
        for (k, v) in map {
            let here = format!("{at}.{k}");
            if let Some(sub) = props.and_then(|p| p.get(k)) {
                check(root, sub, v, &here, out);
                continue;
            }
            match obj.get("additionalProperties") {
                Some(Value::Bool(false)) => {
                    out.push(format!("{here}: property not allowed by the schema"));
                }
                Some(Value::Bool(true)) | None => {}
                Some(sub) => check(root, sub, v, &here, out),
            }
        }
    }

    if let (Value::Array(items), Some(sub)) = (instance, obj.get("items")) {
        for (i, v) in items.iter().enumerate() {
            check(root, sub, v, &format!("{at}[{i}]"), out);
        }
    }

    for (keyword, exactly_one) in [("oneOf", true), ("anyOf", false)] {
        let Some(branches) = obj.get(keyword) else {
            continue;
        };
        let Some(branches) = branches.as_array() else {
            out.push(format!("{at}: `{keyword}` is not an array"));
            continue;
        };
        let passing = branches
            .iter()
            .filter(|b| {
                let mut scratch = Vec::new();
                check(root, b, instance, at, &mut scratch);
                scratch.is_empty()
            })
            .count();
        let satisfied = if exactly_one {
            passing == 1
        } else {
            passing >= 1
        };
        if !satisfied {
            out.push(format!(
                "{at}: {passing} of {} `{keyword}` branches matched {instance}",
                branches.len()
            ));
        }
    }
}

fn check_ref(root: &Value, reference: &str, instance: &Value, at: &str, out: &mut Vec<String>) {
    let (file, pointer) = reference.split_once('#').unwrap_or((reference, ""));
    let loaded;
    let target_root: &Value = if file.is_empty() {
        root
    } else {
        loaded = read_schema_file(file);
        &loaded
    };
    let target = if pointer.is_empty() {
        Some(target_root)
    } else {
        target_root.pointer(pointer)
    };
    match target {
        Some(t) => check(target_root, t, instance, at, out),
        None => out.push(format!("{at}: unresolved `$ref` {reference}")),
    }
}

/// Every violation of the named on-disk schema by `instance`; empty means valid.
fn violations(schema_name: &str, instance: &Value) -> Vec<String> {
    let schema = load_schema(schema_name);
    let mut out = Vec::new();
    check(&schema, &schema, instance, "$", &mut out);
    out
}

/// Panics with every violation and the offending envelope.
fn assert_conforms(schema_name: &str, instance: &Value) {
    let errors = violations(schema_name, instance);
    assert!(
        errors.is_empty(),
        "envelope violates {schema_name}.schema.json:\n  {}\nenvelope: {instance}",
        errors.join("\n  ")
    );
}

// ── process helpers (same shape as gaps_v071_vps_edit_rename.rs) ───────────

fn cmd(tmp: &TempDir) -> Command {
    let llvm_profile_file = std::env::var_os("LLVM_PROFILE_FILE");
    let mut c = Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
    c.env_clear();
    c.env("PATH", std::env::var_os("PATH").unwrap_or_default());
    if let Some(value) = llvm_profile_file {
        c.env("LLVM_PROFILE_FILE", value);
    }
    c.env("HOME", tmp.path());
    c.env("XDG_CONFIG_HOME", tmp.path());
    c.arg("--config-dir").arg(tmp.path());
    c.arg("--allow-plaintext-secrets");
    c
}

/// Runs argv with optional stdin, asserting success, and returns stdout.
fn run_ok(tmp: &TempDir, args: &[&str], stdin: Option<&str>) -> String {
    let mut c = cmd(tmp);
    c.args(args);
    if let Some(input) = stdin {
        c.write_stdin(input);
    }
    let out = c.output().expect("spawn ssh-cli");
    assert!(
        out.status.success(),
        "argv {args:?} failed with {:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn ok(tmp: &TempDir, args: &[&str]) -> String {
    run_ok(tmp, args, None)
}

/// First stdout line that starts a JSON document (`{` or `[`), parsed.
fn json_of(stdout: &str) -> Value {
    let line = stdout
        .lines()
        .find(|l| matches!(l.trim_start().chars().next(), Some('{' | '[')))
        .unwrap_or_else(|| panic!("no JSON on stdout:\n{stdout}"));
    serde_json::from_str(line).unwrap_or_else(|e| panic!("stdout is not JSON: {e}\n{line}"))
}

fn utf8(path: &Path) -> String {
    path.to_str().expect("utf-8 path").to_owned()
}

/// Registers one password-authenticated host.
fn seed_password(tmp: &TempDir, name: &str) {
    run_ok(
        tmp,
        &[
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--password-stdin",
        ],
        Some("secret"),
    );
}

/// Registers one host that authenticates through the SSH agent.
fn seed_agent(tmp: &TempDir, name: &str) {
    let socket = utf8(&tmp.path().join("agent.sock"));
    ok(
        tmp,
        &[
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "203.0.113.11",
            "--user",
            "u",
            "--use-agent",
            "--agent-socket",
            &socket,
        ],
    );
}

/// Registers a host with every conditional field set except the agent pair.
fn seed_full(tmp: &TempDir, fixtures: &Path, name: &str) {
    let cert = fixtures.join("client.crt");
    let key = fixtures.join("client.key");
    std::fs::write(
        &cert,
        "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n",
    )
    .expect("write cert");
    std::fs::write(
        &key,
        "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n",
    )
    .expect("write client key");
    run_ok(
        tmp,
        &[
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "203.0.113.12",
            "--port",
            "2222",
            "--user",
            "deploy",
            "--password-stdin",
        ],
        Some("pw-full"),
    );
    ok(
        tmp,
        &[
            "vps",
            "edit",
            name,
            "--tag",
            "prod",
            "--tag",
            "blue",
            "--tls",
            "--tls-sni",
            "sni.example",
            "--tls-client-cert",
            &utf8(&cert),
            "--tls-client-key",
            &utf8(&key),
            "--disable-sudo",
            "--timeout",
            "9000",
            "--json",
        ],
    );
    run_ok(
        tmp,
        &["vps", "edit", name, "--sudo-password-stdin", "--json"],
        Some("sudo-secret"),
    );
    run_ok(
        tmp,
        &["vps", "edit", name, "--su-password-stdin", "--json"],
        Some("su-secret"),
    );
}

// ── the validator must be able to fail ─────────────────────────────────────

#[test]
fn validator_rejects_what_a_schema_forbids() {
    let schema = json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["event", "n"],
        "properties": {
            "event": { "type": "string", "enum": ["a", "b"] },
            "n": { "type": ["integer", "null"], "minimum": 1 },
            "list": { "type": "array", "items": { "type": "string" } },
            "k": { "const": "fixed" }
        }
    });
    let run = |instance: Value| {
        let mut errors = Vec::new();
        check(&schema, &schema, &instance, "$", &mut errors);
        errors
    };

    assert!(run(json!({ "event": "a", "n": 3 })).is_empty());
    assert!(run(json!({ "event": "b", "n": null, "list": ["x"], "k": "fixed" })).is_empty());

    assert!(!run(json!({ "n": 3 })).is_empty(), "missing required");
    assert!(
        !run(json!({ "event": "c", "n": 3 })).is_empty(),
        "enum miss"
    );
    assert!(!run(json!({ "event": "a", "n": "3" })).is_empty(), "type");
    assert!(!run(json!({ "event": "a", "n": 0 })).is_empty(), "minimum");
    assert!(
        !run(json!({ "event": "a", "n": 1, "list": [1] })).is_empty(),
        "items"
    );
    assert!(
        !run(json!({ "event": "a", "n": 1, "k": "other" })).is_empty(),
        "const"
    );
    assert!(
        !run(json!({ "event": "a", "n": 1, "extra": true })).is_empty(),
        "additionalProperties false"
    );
}

#[test]
fn validator_refuses_keywords_it_cannot_check() {
    for keyword in ["format", "allOf", "not", "maximum", "$defs"] {
        let mut schema = serde_json::Map::new();
        schema.insert("type".into(), json!("string"));
        schema.insert(keyword.into(), json!("x"));
        let schema = Value::Object(schema);
        let mut errors = Vec::new();
        check(&schema, &schema, &json!("v"), "$", &mut errors);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("unsupported schema keyword") && e.contains(keyword)),
            "`{keyword}` must fail loudly, got {errors:?}"
        );
    }
}

#[test]
fn validator_resolves_refs_and_branches() {
    let schema = json!({
        "properties": {
            "a": { "type": "string" },
            "b": { "$ref": "#/properties/a" },
            "c": { "oneOf": [{ "type": "integer" }, { "type": "array" }] },
            "d": { "anyOf": [{ "type": "integer" }, { "type": "null" }] }
        }
    });
    let mut errors = Vec::new();
    check(
        &schema,
        &schema,
        &json!({ "b": "x", "c": 1, "d": null }),
        "$",
        &mut errors,
    );
    assert!(errors.is_empty(), "{errors:?}");

    let mut errors = Vec::new();
    check(
        &schema,
        &schema,
        &json!({ "b": 1, "c": "s", "d": "s" }),
        "$",
        &mut errors,
    );
    assert!(errors.iter().any(|e| e.contains("$.b")), "{errors:?}");
    assert!(errors.iter().any(|e| e.contains("$.c")), "{errors:?}");
    assert!(errors.iter().any(|e| e.contains("$.d")), "{errors:?}");
}

#[test]
fn agent_fields_are_declared_by_show_and_export_schemas() {
    let show = load_schema("vps-show");
    for field in ["use_agent", "agent_socket"] {
        assert!(
            show.pointer(&format!("/properties/{field}")).is_some(),
            "vps-show.schema.json does not declare `{field}`"
        );
        assert!(
            load_schema("vps-export")
                .pointer(&format!(
                    "/properties/hosts/additionalProperties/properties/{field}"
                ))
                .is_some(),
            "vps-export.schema.json host does not declare `{field}`"
        );
    }
    // `vps list` must reach the same fields through the full-item branch of its `anyOf`.
    assert_eq!(
        load_schema("vps-list").pointer("/items/anyOf/0/$ref"),
        Some(&json!("vps-show.schema.json"))
    );
}

// ── rename and edit ────────────────────────────────────────────────────────

#[test]
#[serial]
fn rename_envelope_matches_rename_and_edit_schemas() {
    let tmp = TempDir::new().expect("tempdir");
    seed_password(&tmp, "antigo");

    let env = json_of(&ok(&tmp, &["vps", "rename", "antigo", "novo", "--json"]));
    assert_eq!(env["event"], "vps-renamed", "{env}");
    assert_eq!(env["previous_name"], "antigo", "{env}");
    assert_conforms("vps-rename", &env);
    assert_conforms("vps-edit", &env);
}

#[test]
#[serial]
fn edit_name_envelope_matches_rename_and_edit_schemas() {
    let tmp = TempDir::new().expect("tempdir");
    seed_password(&tmp, "antigo");

    let env = json_of(&ok(
        &tmp,
        &["vps", "edit", "antigo", "--name", "novo", "--json"],
    ));
    assert_eq!(env["event"], "vps-renamed", "{env}");
    assert_eq!(
        env["updated"],
        json!([]),
        "a pure rename lists no field: {env}"
    );
    assert_conforms("vps-rename", &env);
    assert_conforms("vps-edit", &env);
}

#[test]
#[serial]
fn edit_envelope_matches_edit_schema_for_change_and_noop() {
    let tmp = TempDir::new().expect("tempdir");
    seed_password(&tmp, "h");

    let changed = json_of(&ok(
        &tmp,
        &["vps", "edit", "h", "--timeout", "9000", "--json"],
    ));
    assert_eq!(changed["event"], "vps-edited", "{changed}");
    assert!(changed.get("previous_name").is_none(), "{changed}");
    assert_ne!(changed["updated"], json!([]), "{changed}");
    assert_conforms("vps-edit", &changed);

    let noop = json_of(&ok(
        &tmp,
        &["vps", "edit", "h", "--timeout", "9000", "--json"],
    ));
    assert_eq!(noop["event"], "vps-edited", "{noop}");
    assert_eq!(noop["updated"], json!([]), "{noop}");
    assert_conforms("vps-edit", &noop);
}

#[test]
#[serial]
fn dry_run_plans_match_the_dry_run_schema() {
    let tmp = TempDir::new().expect("tempdir");
    seed_password(&tmp, "antigo");

    let rename = json_of(&ok(
        &tmp,
        &["vps", "rename", "antigo", "novo", "--dry-run", "--json"],
    ));
    assert_eq!(rename["operation"], "vps-rename", "{rename}");
    assert_conforms("dry-run", &rename);

    let edit = json_of(&ok(
        &tmp,
        &[
            "vps",
            "edit",
            "antigo",
            "--timeout",
            "7000",
            "--dry-run",
            "--json",
        ],
    ));
    assert_eq!(edit["operation"], "vps-edit", "{edit}");
    assert_conforms("dry-run", &edit);
}

// ── import and export ──────────────────────────────────────────────────────

#[test]
#[serial]
fn import_envelopes_report_replaced_hosts() {
    let tmp = TempDir::new().expect("tempdir");
    let files = TempDir::new().expect("tempdir");
    seed_password(&tmp, "pw");
    seed_agent(&tmp, "agent");

    let file = files.path().join("hosts.json");
    ok(
        &tmp,
        &[
            "vps",
            "export",
            "--include-secrets",
            "--output",
            &utf8(&file),
            "--json",
        ],
    );
    let file_arg = utf8(&file);

    let plan = json_of(&ok(
        &tmp,
        &["vps", "import", "--file", &file_arg, "--dry-run", "--json"],
    ));
    assert_eq!(plan["operation"], "vps-import", "{plan}");
    assert_conforms("dry-run", &plan);
    let hosts = plan["hosts"].as_array().expect("hosts array");
    assert!(
        hosts.iter().all(|h| h["replaces_existing"] == true),
        "both hosts already exist: {plan}"
    );

    let overwritten = json_of(&ok(&tmp, &["vps", "import", "--file", &file_arg, "--json"]));
    assert_eq!(overwritten["event"], "vps-import", "{overwritten}");
    assert_eq!(overwritten["imported"], 2, "{overwritten}");
    assert_eq!(
        overwritten["replaced"],
        json!(["agent", "pw"]),
        "{overwritten}"
    );
    assert_conforms("vps-import", &overwritten);

    let fresh = TempDir::new().expect("tempdir");
    let created = json_of(&ok(
        &fresh,
        &["vps", "import", "--file", &file_arg, "--json"],
    ));
    assert_eq!(created["imported"], 2, "{created}");
    assert_eq!(
        created["replaced"],
        json!([]),
        "`replaced` is always present, empty when nothing was overwritten: {created}"
    );
    assert_conforms("vps-import", &created);
}

#[test]
#[serial]
fn export_matches_the_export_schema_with_an_agent_host() {
    let tmp = TempDir::new().expect("tempdir");
    let fixtures = TempDir::new().expect("tempdir");
    seed_password(&tmp, "pw");
    seed_agent(&tmp, "agent");
    seed_full(&tmp, fixtures.path(), "full");

    let redacted = json_of(&ok(&tmp, &["vps", "export", "--json"]));
    assert_eq!(redacted["event"], "vps-export", "{redacted}");
    assert_eq!(redacted["hosts"]["agent"]["use_agent"], true, "{redacted}");
    assert!(
        redacted["hosts"]["agent"]["agent_socket"].is_string(),
        "{redacted}"
    );
    assert_conforms("vps-export", &redacted);

    let file = fixtures.path().join("secrets.json");
    ok(
        &tmp,
        &[
            "vps",
            "export",
            "--include-secrets",
            "--output",
            &utf8(&file),
            "--json",
        ],
    );
    let body: Value =
        serde_json::from_str(&std::fs::read_to_string(&file).expect("export file")).expect("JSON");
    assert_eq!(body["include_secrets"], true, "{body}");
    assert_eq!(body["hosts"]["full"]["tls"], true, "{body}");
    assert_conforms("vps-export", &body);
}

// ── show and list ──────────────────────────────────────────────────────────

#[test]
#[serial]
fn show_and_list_match_their_schemas_with_an_agent_host() {
    let tmp = TempDir::new().expect("tempdir");
    let fixtures = TempDir::new().expect("tempdir");
    seed_password(&tmp, "pw");
    seed_agent(&tmp, "agent");
    seed_full(&tmp, fixtures.path(), "full");

    for name in ["pw", "agent", "full"] {
        let shown = json_of(&ok(&tmp, &["vps", "show", name, "--json"]));
        assert_conforms("vps-show", &shown);
        if name == "agent" {
            assert_eq!(shown["use_agent"], true, "{shown}");
            assert!(shown["agent_socket"].is_string(), "{shown}");
        }
    }

    let listed = json_of(&ok(&tmp, &["vps", "list", "--json"]));
    let items = listed.as_array().expect("list is an array");
    assert_eq!(items.len(), 3, "{listed}");
    assert!(
        items
            .iter()
            .any(|h| h["name"] == "agent" && h["use_agent"] == true),
        "list must carry `use_agent`: {listed}"
    );
    assert_conforms("vps-list", &listed);
}

/// D6 (auditoria 2026-10-21): `vps list --select` must satisfy the list schema.
///
/// Measured before this change: `vps list --select name --json` returned
/// `[{"name":"a"}]`, which violates `vps-list` because the item inherits
/// `vps-show.schema.json` and that document requires `schema_version` plus `added_at`,
/// or an `agent_shape`. A bare array has no room for `agent_shape` — its reduction
/// report goes to stderr — so the only honest fix is to publish the reduced item as a
/// second shape instead of pretending the full one covers it. The old test validated
/// only the unshaped list, which is exactly why the deviation was never measured.
#[test]
#[serial]
fn vps_list_reduced_items_match_the_published_second_shape() {
    let tmp = TempDir::new().expect("tempdir");
    seed_password(&tmp, "a");
    seed_agent(&tmp, "b");

    let full = json_of(&ok(&tmp, &["vps", "list", "--json"]));
    assert_conforms("vps-list", &full);

    let reduced = json_of(&ok(
        &tmp,
        &[
            "vps", "list", "--json", "--select", "name", "--select", "host",
        ],
    ));
    assert_conforms("vps-list", &reduced);
    let items = reduced.as_array().expect("list is an array");
    assert_eq!(items.len(), 2, "{reduced}");
    for item in items {
        assert!(
            item.get("schema_version").is_none() && item.get("agent_shape").is_none(),
            "the reduced item must keep only the projection: {item}"
        );
        assert!(item["name"].is_string(), "{item}");
        assert!(item["host"].is_string(), "{item}");
        assert!(
            item.get("port").is_none(),
            "an unselected key must not survive: {item}"
        );
    }

    // The reduction of a bare array is reported on stderr, never inside an item, and
    // an item that carried `agent_shape` would be the third shape no schema publishes.
    let out = cmd(&tmp)
        .args(["vps", "list", "--json", "--select", "name", "--limit", "1"])
        .output()
        .expect("spawn ssh-cli");
    assert!(out.status.success(), "exit {:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let array: Value = serde_json::from_str(stdout.trim()).expect("stdout stays a bare array");
    assert_eq!(array.as_array().map(Vec::len), Some(1), "{stdout}");
    assert!(array[0].get("agent_shape").is_none(), "{array}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("\"event\":\"agent-shape\"") && stderr.contains("\"dropped\":1"),
        "the reduction must be reported on stderr: {stderr}"
    );
}

/// A reduced `vps show` must satisfy the second shape published for that payload.
///
/// The schema used to hold one shape only and `additionalProperties: false`, so
/// `--select` could not add its `agent_shape` report without failing the gate. The
/// `anyOf` states both shapes instead of loosening the strictness, and this test proves
/// a reduced run lands on the second one. `anyOf` and not `oneOf` because only
/// `--select` removes fields: `--truncate-content` and the collection-only flags keep
/// the record whole, so with those the first variant matches too.
#[test]
#[serial]
fn vps_show_reduced_body_matches_the_published_second_shape() {
    let tmp = TempDir::new().expect("tempdir");
    seed_password(&tmp, "pw");

    let reduced = json_of(&ok(
        &tmp,
        &[
            "vps", "show", "pw", "--json", "--select", "name", "--select", "host",
        ],
    ));
    assert_conforms("vps-show", &reduced);
    assert_eq!(reduced["name"], "pw", "{reduced}");
    assert!(
        reduced.get("port").is_none(),
        "the projection must drop unselected fields: {reduced}"
    );
    assert_eq!(
        reduced["agent_shape"]["selected"],
        serde_json::json!(["name", "host"]),
        "a reduction must say which paths it kept: {reduced}"
    );

    // The full body keeps the first shape, so the previous contract is untouched.
    let full = json_of(&ok(&tmp, &["vps", "show", "pw", "--json"]));
    assert_conforms("vps-show", &full);
    assert!(
        full.get("agent_shape").is_none(),
        "a run without shaping flags must not grow a report: {full}"
    );
}

/// A2 (auditoria 2026-10-21): `doctor` publishes a reduced shape too.
///
/// The full envelope requires `event`, `ok`, `local` and `ssh_probe`, so a `--select`
/// that keeps only one dotted path cannot satisfy it. Without the second variant the
/// reduced body would violate the document the CLI itself ships.
#[test]
fn doctor_reduced_body_matches_the_published_second_shape() {
    let tmp = TempDir::new().expect("tempdir");
    seed_password(&tmp, "pw");

    let reduced = json_of(&ok(
        &tmp,
        &["doctor", "--json", "--select", "local.config_dir_exposed"],
    ));
    assert_conforms("vps-doctor", &reduced);
    assert!(
        reduced["local"]["config_dir_exposed"].is_boolean(),
        "the projection must keep the requested path: {reduced}"
    );
    assert!(
        reduced.get("event").is_none(),
        "the projection must drop the unselected fields: {reduced}"
    );
    assert_eq!(
        reduced["agent_shape"]["selected"],
        serde_json::json!(["local.config_dir_exposed"]),
        "a reduction must say which paths it kept: {reduced}"
    );

    // The full envelope keeps the first shape, so the previous contract is untouched.
    let full = json_of(&ok(&tmp, &["doctor", "--json"]));
    assert_conforms("vps-doctor", &full);
    assert!(
        full.get("agent_shape").is_none(),
        "a run without shaping flags must not grow a report: {full}"
    );
}

/// A single record names the collection operations it could not apply.
#[test]
fn doctor_reduced_body_names_the_ignored_collection_flags() {
    let tmp = TempDir::new().expect("tempdir");
    seed_password(&tmp, "pw");

    let reduced = json_of(&ok(
        &tmp,
        &[
            "doctor",
            "--json",
            "--select",
            "local.config_dir_exposed",
            "--limit",
            "2",
        ],
    ));
    assert_conforms("vps-doctor", &reduced);
    assert_eq!(
        reduced["agent_shape"]["ignored"],
        serde_json::json!(["--limit"]),
        "--limit cannot apply to a single record and must be named: {reduced}"
    );
}

/// The `--limit` alone is enough: the name cannot need a reducing flag beside it.
///
/// The test above passed `--select` as well, and the early return in
/// `shape_single_object` only fired when neither `--select` nor `--truncate-content`
/// was present. That combination kept every case green while the promised name never
/// reached `ssh-cli doctor --limit 2`.
#[test]
fn a_collection_flag_alone_on_the_doctor_names_itself() {
    let tmp = TempDir::new().expect("tempdir");
    seed_password(&tmp, "pw");

    let reduced = json_of(&ok(&tmp, &["doctor", "--json", "--limit", "2"]));
    assert_conforms("vps-doctor", &reduced);
    assert_eq!(
        reduced["agent_shape"]["ignored"],
        serde_json::json!(["--limit"]),
        "a collection flag on a single record must be named on its own: {reduced}"
    );
    assert!(
        reduced.get("local").is_some(),
        "naming the ignored flag must not drop the record itself: {reduced}"
    );
}
