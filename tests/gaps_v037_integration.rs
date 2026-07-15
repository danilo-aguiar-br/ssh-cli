// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regressão e2e dos 23 gaps resolvidos na 0.3.7.
//!
//! Usa apenas senhas/caminhos FALSOS. Nunca credenciais reais SSH.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use tempfile::TempDir;

fn cmd(tmp: &TempDir) -> Command {
    let llvm_profile_file = std::env::var_os("LLVM_PROFILE_FILE");
    let mut c = Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
    c.env_clear();
    c.env("PATH", std::env::var_os("PATH").unwrap_or_default());
    if let Some(valor) = llvm_profile_file {
        c.env("LLVM_PROFILE_FILE", valor);
    }
    c.env("HOME", tmp.path());
    c.env("XDG_CONFIG_HOME", tmp.path());
    c.env("SSH_CLI_ALLOW_PLAINTEXT_SECRETS", "1");
    c.env("SSH_CLI_FORCE_TEXT", "1");
    c.arg("--config-dir").arg(tmp.path());
    c
}

fn cmd_json(tmp: &TempDir) -> Command {
    let mut c = cmd(tmp);
    c.env_remove("SSH_CLI_FORCE_TEXT");
    c.arg("--output-format").arg("json");
    c
}

fn add_host(tmp: &TempDir, name: &str) {
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
        ])
        .assert()
        .success();
}

// --- VAL ---

#[test]
#[serial]
fn gap_val_001_rejeita_nome_path_traversal() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "../evil",
            "--host",
            "h",
            "--user",
            "u",
            "--password",
            "p",
        ])
        .assert()
        .failure()
        .code(64);
}

#[test]
#[serial]
fn gap_val_001_rejeita_nome_con() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "CON",
            "--host",
            "h",
            "--user",
            "u",
            "--password",
            "p",
        ])
        .assert()
        .failure()
        .code(64);
}

#[test]
#[serial]
fn gap_val_002_rejeita_porta_zero() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "p0",
            "--host",
            "h",
            "--user",
            "u",
            "--password",
            "p",
            "--port",
            "0",
        ])
        .assert()
        .failure()
        .code(64)
        .stderr(predicate::str::contains("porta"));
}

#[test]
#[serial]
fn gap_val_003_rejeita_key_inexistente() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "nokey",
            "--host",
            "h",
            "--user",
            "u",
            "--key",
            "/tmp/does-not-exist-key-xyz-ssh-cli",
        ])
        .assert()
        .failure()
        .code(predicate::in_iter([64i32, 66]));
}

// --- IO ---

#[test]
#[serial]
fn gap_io_001_output_format_json_list_vazio() {
    let tmp = TempDir::new().unwrap();
    cmd_json(&tmp)
        .args(["vps", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

#[test]
#[serial]
fn gap_io_002_health_check_aceita_json_local() {
    let tmp = TempDir::new().unwrap();
    // Sem VPS: falha de domínio, mas clap deve aceitar --json (não "unexpected argument").
    let assert = cmd(&tmp)
        .args(["health-check", "--json"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "health-check --json deve ser flag válida: {stderr}"
    );
}

#[test]
#[serial]
fn gap_io_003_erro_json_envelope() {
    let tmp = TempDir::new().unwrap();
    cmd_json(&tmp)
        .args(["vps", "show", "ghost"])
        .assert()
        .failure()
        .code(66)
        .stderr(predicate::str::contains("\"exit_code\""))
        .stderr(predicate::str::contains("\"message\""));
}

#[test]
#[serial]
fn gap_io_004_quiet_suprime_sucesso() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "--quiet",
            "vps",
            "add",
            "--name",
            "q1",
            "--host",
            "h",
            "--user",
            "u",
            "--password",
            "p",
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
#[serial]
fn gap_cli_002_password_e_stdin_conflitam() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["exec", "x", "true", "--password", "a", "--password-stdin"])
        .assert()
        .failure();
}

#[test]
#[serial]
fn gap_cli_001_su_exec_aceita_password_stdin() {
    let tmp = TempDir::new().unwrap();
    // Parser: --password-stdin existe em su-exec (falha depois por VPS inexistente).
    let assert = cmd(&tmp)
        .args(["su-exec", "x", "id", "--password-stdin"])
        .write_stdin("fake\n")
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "su-exec --password-stdin deve ser válido: {stderr}"
    );
}

#[test]
#[serial]
fn gap_state_001_remove_limpa_active() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "ativa");
    cmd(&tmp).args(["connect", "ativa"]).assert().success();
    assert!(tmp.path().join("active").exists());
    cmd(&tmp)
        .args(["vps", "remove", "ativa"])
        .assert()
        .success();
    assert!(!tmp.path().join("active").exists());
}

#[test]
#[serial]
fn gap_sec_002_mask_sempre_asteriscos() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "maskhost");
    cmd(&tmp)
        .args(["vps", "show", "maskhost", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"password\": \"***\""))
        .stdout(predicate::str::contains("fake-test-password").not());
}

#[test]
#[serial]
fn gap_imp_001_import_redacted_mensagem_clara() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "exp");
    let export = tmp.path().join("export.toml");
    cmd(&tmp)
        .args(["vps", "export", "--output", export.to_str().unwrap()])
        .assert()
        .success();
    let tmp2 = TempDir::new().unwrap();
    cmd(&tmp2)
        .args(["vps", "import", "--file", export.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("redacted")
                .or(predicate::str::contains("allow-incomplete"))
                .or(predicate::str::contains("password")),
        );
}

#[test]
#[serial]
fn gap_imp_001_allow_incomplete() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "exp2");
    let export = tmp.path().join("export2.toml");
    cmd(&tmp)
        .args(["vps", "export", "--output", export.to_str().unwrap()])
        .assert()
        .success();
    let tmp2 = TempDir::new().unwrap();
    cmd(&tmp2)
        .args([
            "vps",
            "import",
            "--file",
            export.to_str().unwrap(),
            "--allow-incomplete",
        ])
        .assert()
        .success();
}

#[test]
#[serial]
fn gap_scp_001_upload_arquivo_local_antes_connect() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "scp1");
    cmd(&tmp)
        .args([
            "scp",
            "upload",
            "scp1",
            "/tmp/ssh-cli-no-such-file-xyz",
            "/tmp/remote",
        ])
        .assert()
        .failure()
        .code(predicate::in_iter([66i32, 1, 74]))
        .stderr(
            predicate::str::contains("não encontrad")
                .or(predicate::str::contains("not found"))
                .or(predicate::str::contains("arquivo")),
        );
}
