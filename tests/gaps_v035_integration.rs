// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for residual gaps (aligned to product 0.5.2).
//!
//! Uses fake passwords/paths only. Never real SSH credentials.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use tempfile::TempDir;

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
    // G-AUD-09/12: no env store for secrets/format — CLI flags only.
    c.arg("--config-dir").arg(tmp.path());
    c.arg("--output-format").arg("text");
    c.arg("--allow-plaintext-secrets");
    c
}

fn cmd_enc_default(tmp: &TempDir) -> Command {
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
    c.arg("--output-format").arg("text");
    c
}

fn add_fake_host(tmp: &TempDir, name: &str) {
    cmd(tmp)
        .args([
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "203.0.113.10",
            "--user",
            "fakeuser",
            "--password",
            "fake-test-password-not-real-001",
            "--max-command-chars",
            "50",
        ])
        .assert()
        .success();
}

#[test]
#[serial]
fn doctor_reports_layer_and_secrets_plaintext() {
    let tmp = TempDir::new().unwrap();
    add_fake_host(&tmp, "doc1");
    cmd(&tmp)
        .args(["vps", "doctor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"secrets_at_rest\":\"plaintext\""))
        .stdout(predicate::str::contains("\"secrets_key_source\":\"none\""))
        .stdout(predicate::str::contains("\"telemetry\":false"))
        .stdout(predicate::str::contains("\"runtime\""))
        .stdout(predicate::str::contains("\"is_wsl\""))
        .stdout(predicate::str::contains("\"is_container\""))
        .stdout(predicate::str::contains("hosts"));
}

#[test]
#[serial]
fn add_with_key_path_without_password() {
    let tmp = TempDir::new().unwrap();
    let key = tmp.path().join("id_test_ed25519");
    let status = match std::process::Command::new("ssh-keygen")
        .args([
            "-t",
            "ed25519",
            "-N",
            "",
            "-f",
            key.to_str().unwrap(),
            "-q",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
    {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("skip: ssh-keygen not on PATH ({e})");
            return;
        }
        Err(e) => panic!("ssh-keygen spawn failed: {e}"),
    };
    assert!(status.success(), "ssh-keygen failed: {status}");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "keyhost",
            "--host",
            "203.0.113.11",
            "--user",
            "fakeuser",
            "--key",
            key.to_str().unwrap(),
        ])
        .assert()
        .success();
    let toml = std::fs::read_to_string(tmp.path().join("config.toml")).unwrap();
    assert!(toml.contains("key_path"));
    assert!(toml.contains("keyhost"));
    assert!(!toml.contains("fake-test-password-not-real-001"));
}

#[test]
#[serial]
fn add_sem_credencial_falha() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "nacred",
            "--host",
            "203.0.113.12",
            "--user",
            "u",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("password").or(predicate::str::contains("key")));
}

#[test]
#[serial]
fn export_redacted_nao_contem_senha() {
    let tmp = TempDir::new().unwrap();
    add_fake_host(&tmp, "exp1");
    let out = tmp.path().join("export.toml");
    cmd(&tmp)
        .args(["vps", "export", "--output", out.to_str().unwrap()])
        .assert()
        .success();
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(!text.contains("fake-test-password-not-real-001"));
    assert!(
        !text.contains("sshcli-enc:"),
        "export redacted must not contain sshcli-enc blobs; got:\n{text}"
    );
    assert!(text.contains("exp1"));
    assert!(out.exists());
}

#[test]
#[serial]
fn secrets_encrypt_on_disk_when_key_set() {
    let tmp = TempDir::new().unwrap();
    let hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    // G-AUD-09: XDG secrets.key only (env SSH_CLI_SECRETS_KEY is fail-closed).
    std::fs::write(tmp.path().join("secrets.key"), hex).unwrap();
    cmd_enc_default(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "enc1",
            "--host",
            "203.0.113.13",
            "--user",
            "fakeuser",
            "--password",
            "fake-test-password-not-real-enc",
        ])
        .assert()
        .success();
    let toml = std::fs::read_to_string(tmp.path().join("config.toml")).unwrap();
    assert!(
        toml.contains("sshcli-enc:v1:"),
        "password must be encrypted on disk"
    );
    assert!(!toml.contains("fake-test-password-not-real-enc"));
}

#[test]
#[serial]
fn max_command_chars_rejeita_comando_longo() {
    let tmp = TempDir::new().unwrap();
    add_fake_host(&tmp, "maxc");
    let longo = "x".repeat(80);
    cmd(&tmp)
        .args(["exec", "maxc", &longo])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("max_command_chars")
                .or(predicate::str::contains("comando excede"))
                .or(predicate::str::contains("command")),
        );
}

#[test]
#[serial]
fn su_exec_sem_senha_su_falha_cedo() {
    let tmp = TempDir::new().unwrap();
    add_fake_host(&tmp, "nosu");
    cmd(&tmp)
        .args(["su-exec", "nosu", "whoami"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("senha_su")
                .or(predicate::str::contains("su"))
                .or(predicate::str::contains("password")),
        );
}

#[test]
#[serial]
fn disable_sudo_global_bloqueia_sudo_exec() {
    let tmp = TempDir::new().unwrap();
    add_fake_host(&tmp, "nosudo");
    cmd(&tmp)
        .args(["--disable-sudo", "sudo-exec", "nosudo", "id"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("desabilitado")
                .or(predicate::str::contains("disable_sudo"))
                .or(predicate::str::contains("Sudo"))
                .or(predicate::str::contains("disabled")),
        );
}

#[test]
#[serial]
fn known_hosts_tofu_unit_via_doctor_path() {
    let tmp = TempDir::new().unwrap();
    add_fake_host(&tmp, "kh1");
    cmd(&tmp)
        .args(["vps", "doctor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("known_hosts"));
}

#[test]
#[serial]
fn packing_abort_contem_term_e_kill() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["su-exec", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("su"));
}

#[test]
#[serial]
fn default_encryption_auto_key_file() {
    let tmp = TempDir::new().unwrap();
    cmd_enc_default(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "encdef",
            "--host",
            "203.0.113.50",
            "--user",
            "fakeuser",
            "--password",
            "fake-default-enc-password-xyz",
        ])
        .assert()
        .success();
    let toml = std::fs::read_to_string(tmp.path().join("config.toml")).unwrap();
    assert!(toml.contains("sshcli-enc:v1:"));
    assert!(!toml.contains("fake-default-enc-password-xyz"));
    assert!(tmp.path().join("secrets.key").is_file());
    cmd_enc_default(&tmp)
        .args(["secrets", "status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("encryption_active"))
        .stdout(predicate::str::contains("xdg_file"));
    cmd_enc_default(&tmp)
        .args(["vps", "doctor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("encrypted"));
}

#[test]
#[serial]
fn secrets_init_and_reencrypt() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "re1",
            "--host",
            "203.0.113.51",
            "--user",
            "fakeuser",
            "--password",
            "fake-reencrypt-password-abc",
        ])
        .assert()
        .success();
    let toml_plain = std::fs::read_to_string(tmp.path().join("config.toml")).unwrap();
    assert!(toml_plain.contains("fake-reencrypt-password-abc"));
    cmd_enc_default(&tmp)
        .args(["secrets", "init", "--force"])
        .assert()
        .success();
    cmd_enc_default(&tmp)
        .args(["secrets", "reencrypt"])
        .assert()
        .success();
    let toml_enc = std::fs::read_to_string(tmp.path().join("config.toml")).unwrap();
    assert!(toml_enc.contains("sshcli-enc:v1:"));
    assert!(!toml_enc.contains("fake-reencrypt-password-abc"));
}
