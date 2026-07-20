// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for ssh-cli 0.5.1 gap closures (GAP-AUD-20260717-*).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin("ssh-cli").unwrap();
    c.env_clear();
    c.env("HOME", tmp.path());
    c.env("PATH", std::env::var("PATH").unwrap_or_default());
    c.env("XDG_CONFIG_HOME", tmp.path());
    c.args(["--config-dir", tmp.path().to_str().unwrap()]);
    c
}

fn seed_host(tmp: &TempDir, name: &str) {
    cmd(tmp)
        .args([
            "secrets",
            "init",
            "--json",
            "--allow-plaintext-secrets",
        ])
        .assert()
        .success();
    // force plaintext for simple roundtrips in this suite when needed
    cmd(tmp)
        .args([
            "--allow-plaintext-secrets",
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "127.0.0.1",
            "--user",
            "u",
            "--password",
            "pw-test-secret-not-real",
            "--timeout",
            "5000",
        ])
        .assert()
        .success();
}

#[test]
fn export_pipe_defaults_to_json_when_non_tty() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "e1");
    // G-AUD-03: non-TTY / global Json → JSON export envelope; force TOML with --output-format text.
    let out = cmd(&tmp)
        .args(["vps", "export"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.trim_start().starts_with('{') && s.contains("vps-export"),
        "export pipe must be JSON envelope under non-TTY: {s}"
    );
    let toml_out = cmd(&tmp)
        .args(["--output-format", "text", "vps", "export"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let t = String::from_utf8_lossy(&toml_out);
    assert!(
        t.contains("[hosts.") || t.contains("name =") || t.contains("schema_version"),
        "export with --output-format text must be TOML, got: {t}"
    );
}

#[test]
fn export_json_flag_envelope() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "e2");
    cmd(&tmp)
        .args(["vps", "export", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vps-export"))
        .stdout(predicate::str::contains("\"ok\""));
}

#[test]
fn export_import_toml_roundtrip() {
    let tmp = TempDir::new().unwrap();
    // Plaintext at-rest so export include-secrets is portable without copying secrets.key.
    cmd(&tmp)
        .args([
            "--allow-plaintext-secrets",
            "vps",
            "add",
            "--name",
            "rt",
            "--host",
            "127.0.0.1",
            "--user",
            "u",
            "--password",
            "pw-roundtrip-plain",
        ])
        .assert()
        .success();
    let export = tmp.path().join("exp.toml");
    cmd(&tmp)
        .args([
            "--allow-plaintext-secrets",
            "vps",
            "export",
            "--include-secrets",
            "--output",
            export.to_str().unwrap(),
        ])
        .assert()
        .success();
    let text = std::fs::read_to_string(&export).unwrap();
    assert!(
        !text.contains("sshcli-enc:"),
        "plaintext export expected: {text}"
    );
    let tmp2 = TempDir::new().unwrap();
    cmd(&tmp2)
        .args([
            "--allow-plaintext-secrets",
            "vps",
            "import",
            "--file",
            export.to_str().unwrap(),
        ])
        .assert()
        .success();
    cmd(&tmp2)
        .args(["vps", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rt"));
}

#[test]
fn import_english_fields() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["secrets", "init", "--allow-plaintext-secrets"])
        .assert()
        .success();
    let f = tmp.path().join("en.toml");
    fs::write(
        &f,
        r#"
schema_version = 3

[hosts.enhost]
name = "enhost"
host = "10.0.0.1"
port = 22
username = "admin"
password = "secret-en-only"
timeout_ms = 60000
schema_version = 3
"#,
    )
    .unwrap();
    cmd(&tmp)
        .args([
            "--allow-plaintext-secrets",
            "vps",
            "import",
            "--file",
            f.to_str().unwrap(),
        ])
        .assert()
        .success();
    cmd(&tmp)
        .args(["vps", "show", "enhost"])
        .assert()
        .success()
        .stdout(predicate::str::contains("10.0.0.1"));
}

#[test]
fn import_pt_without_added_at() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["secrets", "init", "--allow-plaintext-secrets"])
        .assert()
        .success();
    let f = tmp.path().join("pt.toml");
    fs::write(
        &f,
        r#"
schema_version = 2

[hosts.pthost]
nome = "pthost"
host = "10.0.0.2"
porta = 22
usuario = "root"
senha = "secret-pt"
timeout_ms = 60000
schema_version = 2
"#,
    )
    .unwrap();
    cmd(&tmp)
        .args([
            "--allow-plaintext-secrets",
            "vps",
            "import",
            "--file",
            f.to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[test]
fn import_bad_toml_exit_65() {
    let tmp = TempDir::new().unwrap();
    let f = tmp.path().join("bad.toml");
    fs::write(&f, "this is not = valid [toml").unwrap();
    cmd(&tmp)
        .args(["vps", "import", "--file", f.to_str().unwrap()])
        .assert()
        .failure()
        .code(65);
}

#[test]
fn secrets_init_json_envelope() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["secrets", "init", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secrets-init"))
        .stdout(predicate::str::contains("\"ok\""));
}

#[test]
fn empty_command_english() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "c1");
    // Will fail connect (no ssh) but empty command should fail before with invalid argument
    let out = cmd(&tmp)
        .args(["--lang", "en-US", "exec", "c1", "   "])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("empty command") || s.contains("invalid argument"),
        "expected EN empty command, got: {s}"
    );
    assert!(!s.contains("comando vazio"), "must not contain PT hardcode: {s}");
}

#[test]
fn crud_add_json_success_event() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["secrets", "init", "--allow-plaintext-secrets"])
        .assert()
        .success();
    cmd(&tmp)
        .args([
            "--output-format",
            "json",
            "--allow-plaintext-secrets",
            "vps",
            "add",
            "--name",
            "j1",
            "--host",
            "1.1.1.1",
            "--user",
            "u",
            "--password",
            "pw-json-add",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("vps-added").or(predicate::str::contains("secrets-key-auto-created")));
}

#[test]
fn include_secrets_pipe_refused_without_ack() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "sec");
    // Non-TTY by default in assert_cmd
    cmd(&tmp)
        .args(["vps", "export", "--include-secrets"])
        .assert()
        .failure()
        .code(64);
}

#[test]
fn wire_serialize_english_keys() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "w1");
    let cfg = fs::read_to_string(tmp.path().join("config.toml")).unwrap();
    assert!(cfg.contains("name =") || cfg.contains("[hosts.w1]"));
    assert!(!cfg.contains("nome ="), "must not write PT nome: {cfg}");
    assert!(!cfg.contains("porta ="), "must not write PT porta: {cfg}");
}

/// G-PAR-23: multi-host `--all` with empty registry fails closed (no fan-out spawn).
#[test]
fn health_check_all_empty_registry_exits_usage() {
    let tmp = TempDir::new().unwrap();
    // No hosts registered — fan-out path must reject before Semaphore work.
    cmd(&tmp)
        .args([
            "--max-concurrency",
            "4",
            "health-check",
            "--all",
            "--json",
        ])
        .assert()
        .failure()
        .code(64)
        // Agent error envelope on stderr when JSON mode is active.
        .stderr(predicate::str::contains("no hosts registered for --all"))
        .stderr(predicate::str::contains("\"exit_code\":64"));
}

/// G-PAR-23: global `--max-concurrency` is accepted with multi-host flags.
#[test]
fn max_concurrency_with_exec_all_empty_registry() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "--max-concurrency",
            "2",
            "exec",
            "--all",
            "true",
            "--json",
        ])
        .assert()
        .failure()
        .code(64);
}

/// G-PAR-32: `--hosts` with empty registry fails closed (no fan-out spawn).
#[test]
fn exec_hosts_empty_registry_exits_usage() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "--max-concurrency",
            "4",
            "exec",
            "--hosts",
            "a,b",
            "true",
            "--json",
        ])
        .assert()
        .failure()
        .code(64)
        .stderr(predicate::str::contains("no hosts registered for --hosts"));
}

/// G-PAR-32: unknown host in `--hosts` fails closed.
#[test]
fn health_check_hosts_unknown_exits_usage() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "real");
    cmd(&tmp)
        .args(["health-check", "--hosts", "ghost", "--json"])
        .assert()
        .failure()
        .code(64)
        .stderr(predicate::str::contains("unknown host(s) for --hosts"));
}

/// G-PAR-32: clap rejects `--all` combined with `--hosts`.
#[test]
fn exec_all_hosts_conflict() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["exec", "--all", "--hosts", "a", "true"])
        .assert()
        .failure();
}

/// G-PAR-42: doctor without probe is a single JSON root with event vps-doctor.
#[test]
fn doctor_json_single_root_envelope() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "d1");
    let out = cmd(&tmp)
        .args(["vps", "doctor", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8_lossy(&out);
    // Single root: one JSON object, envelope fields present (no dual health-check root).
    assert!(
        s.trim_start().starts_with('{'),
        "doctor JSON must start with object: {s}"
    );
    assert!(
        s.contains("\"event\":\"vps-doctor\"") || s.contains("\"event\": \"vps-doctor\""),
        "event vps-doctor missing: {s}"
    );
    assert!(s.contains("\"local\""), "local nested missing: {s}");
    assert!(
        s.contains("\"ssh_probe\":null") || s.contains("\"ssh_probe\": null"),
        "ssh_probe null missing: {s}"
    );
    // Must not emit a second top-level health-check-batch event.
    assert!(
        !s.contains("health-check-batch"),
        "dual root / nested batch without probe unexpected: {s}"
    );
}

/// G-PAR-38: --hosts on doctor without --probe-ssh is rejected.
#[test]
fn doctor_hosts_requires_probe_ssh() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "d2");
    cmd(&tmp)
        .args(["vps", "doctor", "--hosts", "d2", "--json"])
        .assert()
        .failure()
        .code(64)
        .stderr(predicate::str::contains("--probe-ssh"));
}

/// G-PAR-48: multi-file scp with --all is accepted (cartesian); fails on SSH, not parse.
#[test]
fn scp_multi_file_with_all_parses_and_attempts_transfer() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "s1");
    let a = tmp.path().join("a.bin");
    let b = tmp.path().join("b.bin");
    fs::write(&a, b"aa").unwrap();
    fs::write(&b, b"bb").unwrap();
    // Connection refused expected (no sshd) — must emit scp-batch, not clap usage error.
    cmd(&tmp)
        .args([
            "scp",
            "upload",
            "--all",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "/tmp/out",
        ])
        .assert()
        .failure()
        .code(predicate::ne(64))
        .stdout(predicate::str::contains("scp-batch"));
}

/// G-PAR-37: clap accepts multi-file positionals (fails later on SSH, not on parse).
#[test]
fn scp_multi_file_positionals_parsed() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "s2");
    let a = tmp.path().join("a.bin");
    let b = tmp.path().join("b.bin");
    fs::write(&a, b"aa").unwrap();
    fs::write(&b, b"bb").unwrap();
    // Will fail on connect to 127.0.0.1 without sshd — but must not be clap usage error.
    let assert = cmd(&tmp)
        .args([
            "--max-concurrency",
            "2",
            "scp",
            "upload",
            "s2",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "/tmp",
            "--json",
        ])
        .assert()
        .failure();
    let code = assert.get_output().status.code();
    // Not clap usage (2) for multi-file parse — domain/network failure is fine.
    assert_ne!(code, Some(2), "clap must accept multi-file positionals");
}
