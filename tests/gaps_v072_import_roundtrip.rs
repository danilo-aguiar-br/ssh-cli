// SPDX-License-Identifier: MIT OR Apache-2.0
//! `vps export` / `vps import` round trip, TLS validation on import, the
//! `replaced` field and the closed-stdout exit code.
//!
//! # Why every case drives argv
//!
//! The guarantees live at the process boundary: which exit code appears, which
//! envelope reaches stdout and what a second registry holds after the import.
//! Every case spawns the binary with a fresh `--config-dir`.
//!
//! # What the round trip proves
//!
//! A host is filled through `vps edit`, exported with its secrets, imported into
//! an empty registry and compared field by field, once through the JSON envelope
//! and once through TOML. The comparison runs on the exported document, which
//! carries the secrets, and on `vps show --json`, which carries the masked view.
//! A field that export or import forgets shows up as a difference here.
//!
//! # Why the field count is pinned
//!
//! `VpsRecord` gained `use_agent` and `agent_socket` without export or import
//! learning them, and the loss was silent. The count test fails the moment the
//! struct grows, so the author has to revisit export, import and this file.

use assert_cmd::Command;
use serde_json::Value;
use serial_test::serial;
use tempfile::TempDir;

/// Exit code for a usage error (`sysexits.h` `EX_USAGE`).
const EX_USAGE: i32 = 64;

/// Number of `pub` fields `VpsRecord` has when export, import and this file agree.
const VPS_RECORD_FIELD_COUNT: usize = 22;

/// Builds the environment every spawn shares: empty env, fresh config dir.
fn base_env(tmp: &TempDir) -> Vec<(&'static str, std::ffi::OsString)> {
    let mut env = vec![
        ("PATH", std::env::var_os("PATH").unwrap_or_default()),
        ("HOME", tmp.path().as_os_str().to_owned()),
        ("XDG_CONFIG_HOME", tmp.path().as_os_str().to_owned()),
    ];
    if let Some(value) = std::env::var_os("LLVM_PROFILE_FILE") {
        env.push(("LLVM_PROFILE_FILE", value));
    }
    env
}

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
    c.env_clear();
    for (k, v) in base_env(tmp) {
        c.env(k, v);
    }
    c.arg("--config-dir").arg(tmp.path());
    c.arg("--allow-plaintext-secrets");
    c
}

/// Runs argv with optional stdin and returns stdout, asserting success.
fn ok_stdin(tmp: &TempDir, args: &[&str], stdin: &str) -> String {
    let out = cmd(tmp)
        .args(args)
        .write_stdin(stdin.to_owned())
        .output()
        .expect("spawn ssh-cli");
    assert!(
        out.status.success(),
        "argv {args:?} failed with {:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Runs argv and returns stdout as text, asserting success.
fn ok(tmp: &TempDir, args: &[&str]) -> String {
    ok_stdin(tmp, args, "")
}

/// Runs argv, returning exit code plus both streams.
fn run(tmp: &TempDir, args: &[&str]) -> (i32, String, String) {
    let out = cmd(tmp).args(args).output().expect("spawn ssh-cli");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Parses the single compact JSON object a `--json` success run writes to stdout.
fn envelope_of(stdout: &str) -> Value {
    let line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| panic!("no JSON envelope on stdout:\n{stdout}"));
    serde_json::from_str(line).unwrap_or_else(|e| panic!("envelope is not JSON: {e}\n{line}"))
}

/// Writes a real OpenSSH key fixture, or `None` when `ssh-keygen` is absent.
///
/// The registry validates key content when `ssh-real` is on, so a placeholder
/// byte string is rejected. Test-only: the product never spawns `ssh-keygen`.
fn write_openssh_key(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let key = dir.join("id_ed25519_test");
    let status = match std::process::Command::new("ssh-keygen")
        .args(["-t", "ed25519", "-N", "", "-q", "-f"])
        .arg(&key)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => panic!("ssh-keygen spawn failed: {e}"),
    };
    assert!(status.success(), "ssh-keygen failed: {status}");
    Some(key)
}

/// Registers up to three hosts and fills every exportable field through `vps edit`.
///
/// Returns whether the `keyed` host was created (it needs `ssh-keygen`).
fn populate(tmp: &TempDir, fixtures: &std::path::Path) -> bool {
    let cert = fixtures.join("client.pem");
    let client_key = fixtures.join("client.key");
    std::fs::write(
        &cert,
        "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n",
    )
    .expect("write cert");
    std::fs::write(
        &client_key,
        "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n",
    )
    .expect("write client key");
    let cert = cert.to_str().expect("utf-8 path").to_owned();
    let client_key = client_key.to_str().expect("utf-8 path").to_owned();

    ok_stdin(
        tmp,
        &[
            "vps",
            "add",
            "--name",
            "full",
            "--host",
            "203.0.113.10",
            "--port",
            "2222",
            "--user",
            "deploy",
            "--password-stdin",
        ],
        "pw-full",
    );
    ok(
        tmp,
        &[
            "vps",
            "edit",
            "full",
            "--tag",
            "prod",
            "--tag",
            "blue",
            "--tls",
            "--tls-sni",
            "sni.example",
            "--tls-client-cert",
            &cert,
            "--tls-client-key",
            &client_key,
            "--disable-sudo",
            "--timeout",
            "9000",
            "--max-command-chars",
            "1234",
            "--max-output-chars",
            "5678",
            "--json",
        ],
    );
    ok_stdin(
        tmp,
        &["vps", "edit", "full", "--sudo-password-stdin", "--json"],
        "sudo-secret",
    );
    ok_stdin(
        tmp,
        &["vps", "edit", "full", "--su-password-stdin", "--json"],
        "su-secret",
    );
    ok_stdin(
        tmp,
        &[
            "vps",
            "add",
            "--name",
            "agent",
            "--host",
            "203.0.113.11",
            "--user",
            "u",
            "--password-stdin",
        ],
        "pw-agent",
    );
    let sock = fixtures.join("agent.sock");
    let sock = sock.to_str().expect("utf-8 path").to_owned();
    ok(
        tmp,
        &[
            "vps",
            "edit",
            "agent",
            "--use-agent",
            "--agent-socket",
            &sock,
            "--json",
        ],
    );

    // A password host switched to key auth: `--key` replaces the password, so this
    // host carries the key and passphrase while `full` keeps its password.
    match write_openssh_key(fixtures) {
        Some(key) => {
            let key = key.to_str().expect("utf-8 path").to_owned();
            ok_stdin(
                tmp,
                &[
                    "vps",
                    "add",
                    "--name",
                    "keyed",
                    "--host",
                    "203.0.113.12",
                    "--user",
                    "u",
                    "--password-stdin",
                ],
                "pw-keyed",
            );
            ok_stdin(
                tmp,
                &[
                    "vps",
                    "edit",
                    "keyed",
                    "--key",
                    &key,
                    "--key-passphrase-stdin",
                    "--json",
                ],
                "key-pass",
            );
            true
        }
        None => false,
    }
}

/// Exports every host with its secrets as JSON and returns the `hosts` object.
fn export_hosts(tmp: &TempDir, file_name: &str) -> Value {
    let out = tmp.path().join(file_name);
    let out_arg = out.to_str().expect("utf-8 path");
    ok(
        tmp,
        &[
            "vps",
            "export",
            "--include-secrets",
            "--output",
            out_arg,
            "--json",
        ],
    );
    let text = std::fs::read_to_string(&out).expect("export file");
    let doc: Value = serde_json::from_str(&text).expect("export file is JSON");
    doc["hosts"].clone()
}

/// `vps show --json` for one host.
fn show(tmp: &TempDir, name: &str) -> Value {
    envelope_of(&ok(tmp, &["vps", "show", name, "--json"]))
}

/// Asserts the source registry really holds a value in every exportable field,
/// so an `edit` that silently dropped one cannot make the comparison vacuous.
fn assert_source_is_fully_populated(hosts: &Value, with_key: bool) {
    let full = &hosts["full"];
    assert_eq!(full["host"], "203.0.113.10", "{full}");
    assert_eq!(full["port"], 2222, "{full}");
    assert_eq!(full["user"], "deploy", "{full}");
    assert_eq!(full["password"], "pw-full", "{full}");
    assert_eq!(full["sudo_password"], "sudo-secret", "{full}");
    assert_eq!(full["su_password"], "su-secret", "{full}");
    assert_eq!(full["disable_sudo"], true, "{full}");
    assert_eq!(full["timeout_ms"], 9000, "{full}");
    assert_eq!(full["max_command_chars"], 1234, "{full}");
    assert_eq!(full["max_output_chars"], 5678, "{full}");
    assert_eq!(full["tags"], serde_json::json!(["prod", "blue"]), "{full}");
    assert_eq!(full["tls"], true, "{full}");
    assert_eq!(full["tls_sni"], "sni.example", "{full}");
    assert!(full["tls_client_cert"].is_string(), "{full}");
    assert!(full["tls_client_key"].is_string(), "{full}");
    if with_key {
        let keyed = &hosts["keyed"];
        assert!(keyed["key_path"].is_string(), "{keyed}");
        assert_eq!(keyed["key_passphrase"], "key-pass", "{keyed}");
        assert_eq!(
            keyed["password"], "",
            "`--key` must drop the password: {keyed}"
        );
    }
    let agent = &hosts["agent"];
    assert_eq!(agent["use_agent"], true, "{agent}");
    assert!(agent["agent_socket"].is_string(), "{agent}");
}

/// Compares both registries on the secret-bearing export and on the masked view.
fn assert_registries_match(source: &TempDir, dest: &TempDir) {
    let source_hosts = export_hosts(source, "source-export.json");
    let dest_hosts = export_hosts(dest, "dest-export.json");
    let names: Vec<&String> = source_hosts
        .as_object()
        .expect("hosts object")
        .keys()
        .collect();
    assert_eq!(
        names,
        dest_hosts
            .as_object()
            .expect("hosts object")
            .keys()
            .collect::<Vec<_>>(),
        "the imported registry must hold the same host names"
    );
    for name in names {
        let (src, dst) = (&source_hosts[name], &dest_hosts[name]);
        let fields = src.as_object().expect("host object");
        for (field, value) in fields {
            assert_eq!(
                value, &dst[field],
                "host '{name}' field '{field}' changed across export and import"
            );
        }
        assert_eq!(
            fields.len(),
            dst.as_object().expect("host object").len(),
            "host '{name}' gained or lost a field across export and import"
        );
        assert_eq!(
            show(source, name),
            show(dest, name),
            "`vps show` differs for host '{name}'"
        );
    }
}

// ── round trip ──────────────────────────────────────────────────────────────

#[test]
#[serial]
fn json_export_then_import_round_trips_every_field() {
    let source = TempDir::new().expect("tempdir");
    let fixtures = TempDir::new().expect("tempdir");
    let with_key = populate(&source, fixtures.path());
    let exported = export_hosts(&source, "hosts.json");
    assert_source_is_fully_populated(&exported, with_key);

    let dest = TempDir::new().expect("tempdir");
    let file = source.path().join("hosts.json");
    let v = envelope_of(&ok(
        &dest,
        &[
            "vps",
            "import",
            "--file",
            file.to_str().expect("utf-8 path"),
            "--json",
        ],
    ));
    assert_eq!(v["event"], "vps-import", "{v}");
    assert_eq!(v["imported"], if with_key { 3 } else { 2 }, "{v}");

    assert_registries_match(&source, &dest);
}

#[test]
#[serial]
fn toml_export_then_import_round_trips_every_field() {
    let source = TempDir::new().expect("tempdir");
    let fixtures = TempDir::new().expect("tempdir");
    let with_key = populate(&source, fixtures.path());
    assert_source_is_fully_populated(&export_hosts(&source, "reference.json"), with_key);

    // The export body is TOML only in text mode; a piped stdout defaults to JSON,
    // so the format is requested explicitly.
    let toml_file = source.path().join("hosts.toml");
    ok(
        &source,
        &[
            "--output-format",
            "text",
            "vps",
            "export",
            "--include-secrets",
            "--output",
            toml_file.to_str().expect("utf-8 path"),
        ],
    );
    let body = std::fs::read_to_string(&toml_file).expect("toml export");
    assert!(
        !body.trim_start().starts_with('{'),
        "the text-mode export must be TOML, not JSON:\n{body}"
    );

    let dest = TempDir::new().expect("tempdir");
    ok(
        &dest,
        &[
            "vps",
            "import",
            "--file",
            toml_file.to_str().expect("utf-8 path"),
            "--json",
        ],
    );

    assert_registries_match(&source, &dest);
}

#[test]
fn vps_record_field_count_is_pinned() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/vps/model.rs");
    let src = std::fs::read_to_string(path).expect("read src/vps/model.rs");
    let start = src
        .find("pub struct VpsRecord {")
        .expect("`pub struct VpsRecord {` moved or was renamed");
    let body = &src[start..];
    let end = body
        .find("\n}")
        .expect("`VpsRecord` struct has no closing brace");
    let count = body[..end]
        .lines()
        .filter(|l| l.starts_with("    pub "))
        .count();
    assert_eq!(
        count, VPS_RECORD_FIELD_COUNT,
        "`VpsRecord` now has {count} pub fields, this test pins {VPS_RECORD_FIELD_COUNT}: update \
         `ExportHostJson` (src/json_wire/vps_export.rs), `ImportHostEntry` \
         (src/json_wire/vps_import.rs), the TOML path and this test together, then change \
         VPS_RECORD_FIELD_COUNT"
    );
}

// ── TLS validation on import ────────────────────────────────────────────────

/// Writes an import document with a single host and returns its path.
fn write_import(tmp: &TempDir, name: &str, host: Value) -> std::path::PathBuf {
    let file = tmp.path().join(name);
    let doc = serde_json::json!({ "hosts": { "h": host } });
    std::fs::write(&file, doc.to_string()).expect("write import file");
    file
}

#[test]
#[serial]
fn import_refuses_a_client_cert_without_its_key() {
    let tmp = TempDir::new().expect("tempdir");
    let file = write_import(
        &tmp,
        "half.json",
        serde_json::json!({
            "host": "203.0.113.10",
            "username": "u",
            "password": "pw",
            "tls": true,
            "tls_client_cert": "/etc/certs/client.pem",
        }),
    );
    let (code, stdout, stderr) = run(
        &tmp,
        &[
            "vps",
            "import",
            "--file",
            file.to_str().expect("utf-8 path"),
            "--json",
        ],
    );
    assert_eq!(code, EX_USAGE, "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stderr.contains("'h'"),
        "the error must name the host that was refused:\n{stderr}"
    );
    let (missing, _, _) = run(&tmp, &["vps", "show", "h", "--json"]);
    assert_ne!(missing, 0, "a refused import must not register the host");
}

#[test]
#[serial]
fn import_accepts_a_client_cert_with_its_key() {
    let tmp = TempDir::new().expect("tempdir");
    let file = write_import(
        &tmp,
        "pair.json",
        serde_json::json!({
            "host": "203.0.113.10",
            "username": "u",
            "password": "pw",
            "tls": true,
            "tls_client_cert": "/etc/certs/client.pem",
            "tls_client_key": "/etc/certs/client.key",
        }),
    );
    let v = envelope_of(&ok(
        &tmp,
        &[
            "vps",
            "import",
            "--file",
            file.to_str().expect("utf-8 path"),
            "--json",
        ],
    ));
    assert_eq!(v["imported"], 1, "{v}");
}

// ── replaced ────────────────────────────────────────────────────────────────

#[test]
#[serial]
fn import_reports_the_hosts_it_replaced() {
    let tmp = TempDir::new().expect("tempdir");
    ok_stdin(
        &tmp,
        &[
            "vps",
            "add",
            "--name",
            "dup",
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--password-stdin",
        ],
        "old",
    );
    let file = tmp.path().join("two.json");
    let doc = serde_json::json!({ "hosts": {
        "dup": { "host": "203.0.113.20", "username": "u", "password": "new" },
        "fresh": { "host": "203.0.113.21", "username": "u", "password": "pw" },
    }});
    std::fs::write(&file, doc.to_string()).expect("write import file");
    let file_arg = file.to_str().expect("utf-8 path");

    let plan = envelope_of(&ok(
        &tmp,
        &["vps", "import", "--file", file_arg, "--dry-run", "--json"],
    ));
    assert_eq!(plan["event"], "dry-run", "{plan}");
    let planned = plan["hosts"].as_array().expect("planned hosts");
    let dup = planned
        .iter()
        .find(|h| h["name"] == "dup")
        .expect("dup in plan");
    assert_eq!(dup["replaces_existing"], true, "{plan}");

    let v = envelope_of(&ok(&tmp, &["vps", "import", "--file", file_arg, "--json"]));
    assert_eq!(v["event"], "vps-import", "{v}");
    assert_eq!(v["imported"], 2, "{v}");
    assert_eq!(v["replaced"], serde_json::json!(["dup"]), "{v}");

    // The second run replaces both and lists them alphabetically.
    let again = envelope_of(&ok(&tmp, &["vps", "import", "--file", file_arg, "--json"]));
    assert_eq!(
        again["replaced"],
        serde_json::json!(["dup", "fresh"]),
        "{again}"
    );
}

#[test]
#[serial]
fn import_into_an_empty_registry_reports_an_empty_replaced_list() {
    let tmp = TempDir::new().expect("tempdir");
    let file = write_import(
        &tmp,
        "one.json",
        serde_json::json!({ "host": "203.0.113.10", "username": "u", "password": "pw" }),
    );
    let v = envelope_of(&ok(
        &tmp,
        &[
            "vps",
            "import",
            "--file",
            file.to_str().expect("utf-8 path"),
            "--json",
        ],
    ));
    assert_eq!(
        v["replaced"],
        serde_json::json!([]),
        "`replaced` is always present: {v}"
    );
}

// ── closed stdout ───────────────────────────────────────────────────────────

/// Runs argv in text mode on a stdout whose reader is gone; returns exit code and stderr.
///
/// Unix only: the test needs a pipe whose read end is closed, and std offers no
/// portable way to build one before Rust 1.87 (the crate's MSRV is 1.88). A
/// `cat` child supplies the pipe: its stdin is the write end, and once the
/// child is killed and reaped the read end is closed before `ssh-cli` starts, so
/// the first write fails with `EPIPE` every time instead of racing the reader.
#[cfg(unix)]
fn run_with_closed_stdout(tmp: &TempDir, args: &[&str]) -> (Option<i32>, String) {
    use std::process::Stdio;

    let mut reader = std::process::Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn cat");
    let write_end = reader.stdin.take().expect("cat stdin");
    reader.kill().expect("kill cat");
    reader.wait().expect("reap cat");

    let mut c = std::process::Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
    c.env_clear();
    for (k, v) in base_env(tmp) {
        c.env(k, v);
    }
    let out = c
        .arg("--config-dir")
        .arg(tmp.path())
        .arg("--allow-plaintext-secrets")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(write_end))
        .stderr(Stdio::piped())
        .output()
        .expect("spawn ssh-cli");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The success line of `vps export` (no `--json`) goes through `emit_success`.
#[cfg(unix)]
#[test]
#[serial]
fn text_success_line_on_a_closed_stdout_exits_141_not_0() {
    let tmp = TempDir::new().expect("tempdir");
    let out_file = tmp.path().join("hosts.toml");
    let (code, stderr) = run_with_closed_stdout(
        &tmp,
        &[
            "vps",
            "export",
            "--output",
            out_file.to_str().expect("utf-8 path"),
        ],
    );
    assert_eq!(
        code,
        Some(141),
        "a closed stdout must end in 141 (BrokenPipe), stderr:\n{stderr}"
    );
}

/// `vps list` in text mode streams a table through `print_list_text`.
#[cfg(unix)]
#[test]
#[serial]
fn text_list_on_a_closed_stdout_exits_141_not_0() {
    let tmp = TempDir::new().expect("tempdir");
    ok_stdin(
        &tmp,
        &[
            "vps",
            "add",
            "--name",
            "h",
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--password-stdin",
        ],
        "pw",
    );
    let (code, stderr) = run_with_closed_stdout(&tmp, &["vps", "list"]);
    assert_eq!(
        code,
        Some(141),
        "a closed stdout must end in 141 (BrokenPipe), stderr:\n{stderr}"
    );
}

#[test]
#[serial]
fn import_of_a_missing_file_is_a_permanent_not_found() {
    let tmp = TempDir::new().expect("tempdir");
    let missing = tmp.path().join("absent.toml");
    let (code, stdout, stderr) = run(
        &tmp,
        &[
            "vps",
            "import",
            "--file",
            missing.to_str().expect("utf-8 path"),
            "--json",
        ],
    );
    assert_eq!(code, 66, "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stdout.is_empty(),
        "a failed import must keep stdout clean:\n{stdout}"
    );
    let line = stderr
        .lines()
        .find(|l| l.starts_with('{'))
        .unwrap_or_else(|| panic!("no JSON envelope on stderr:\n{stderr}"));
    let envelope: Value = serde_json::from_str(line).expect("error envelope is JSON");
    assert_eq!(envelope["error_code"], "file_not_found");
    assert_eq!(envelope["error_class"], "permanent");
    assert_eq!(envelope["retryable"], false);
}
