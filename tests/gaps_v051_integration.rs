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
fn export_pipe_is_toml_not_json() {
    let tmp = TempDir::new().unwrap();
    seed_host(&tmp, "e1");
    let out = cmd(&tmp)
        .args(["vps", "export"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("[hosts.") || s.contains("name =") || s.contains("schema_version"),
        "export pipe must be TOML, got: {s}"
    );
    assert!(
        !s.trim_start().starts_with('{'),
        "export pipe must not be JSON without --json: {s}"
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
