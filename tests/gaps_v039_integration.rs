//! Regressão e2e dos gaps residuais da auditoria pós-0.3.8 (v0.3.9).
//!
//! IDs: LOG-001, JSON-001, CLI-004, DOC-003 (version string), DENY-002 (policy),
//! REL-003 (tag/version), CHG-001 (docs). Usa apenas credenciais FALSAS.

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

fn add_host_password(tmp: &TempDir, name: &str) {
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

fn write_ed25519(tmp: &TempDir) -> std::path::PathBuf {
    let key = tmp.path().join("id_ed25519_test");
    let status = std::process::Command::new("ssh-keygen")
        .args([
            "-t",
            "ed25519",
            "-f",
            key.to_str().unwrap(),
            "-N",
            "",
            "-q",
        ])
        .status()
        .expect("ssh-keygen");
    assert!(status.success(), "ssh-keygen failed");
    key
}

// --- LOG-001 ---

#[test]
#[serial]
fn gap_log_001_tunnel_json_stderr_sem_info_prosa() {
    let tmp = TempDir::new().unwrap();
    add_host_password(&tmp, "tlog");
    let assert = cmd_json(&tmp)
        .args([
            "tunnel",
            "tlog",
            "19101",
            "127.0.0.1",
            "9",
            "--timeout-ms",
            "30",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        !stderr.contains("Tunnel SSH:"),
        "stderr não deve ter prosa INFO: {stderr}"
    );
    assert!(
        !stderr.contains("iniciando tunnel"),
        "stderr não deve ter INFO default: {stderr}"
    );
    assert!(
        stderr.contains("\"exit_code\""),
        "stderr deve conter envelope JSON: {stderr}"
    );
}

// --- JSON-001 ---

#[test]
#[serial]
fn gap_json_001_key_only_password_null() {
    let tmp = TempDir::new().unwrap();
    let key = write_ed25519(&tmp);
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "konly",
            "--host",
            "203.0.113.11",
            "--user",
            "root",
            "--key",
            key.to_str().unwrap(),
        ])
        .assert()
        .success();
    cmd_json(&tmp)
        .args(["vps", "show", "konly"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"password\": null"))
        .stdout(predicate::str::contains("\"password\": \"***\"").not());
}

#[test]
#[serial]
fn gap_json_001_com_password_continua_mascara() {
    let tmp = TempDir::new().unwrap();
    add_host_password(&tmp, "withpwd");
    cmd_json(&tmp)
        .args(["vps", "show", "withpwd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"password\": \"***\""))
        .stdout(predicate::str::contains("fake-test-password").not());
}

// --- CLI-004 ---

#[test]
#[serial]
fn gap_cli_004_health_check_aceita_timeout() {
    let tmp = TempDir::new().unwrap();
    // clap aceita a flag (falha de domínio sem VPS, não parse error)
    let assert = cmd(&tmp)
        .args(["health-check", "--json", "--timeout", "50"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "health-check deve aceitar --timeout: {stderr}"
    );
    assert!(
        stderr.contains("66") || stderr.contains("Nenhuma VPS") || stderr.contains("ativa"),
        "erro de domínio esperado: {stderr}"
    );
}

// --- DOC-003 / product line ---

#[test]
#[serial]
fn gap_doc_003_version_contem_039() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.3.9"));
}

// --- DENY-002 policy still yanked=deny ignore empty ---

#[test]
fn gap_deny_002_deny_toml_sem_ignore_cve() {
    let deny = std::fs::read_to_string("deny.toml").expect("deny.toml");
    assert!(deny.contains("yanked = \"deny\"") || deny.contains("yanked=\"deny\""));
    assert!(
        deny.contains("ignore = []") || deny.contains("ignore=[]"),
        "ignore deve permanecer vazio"
    );
    assert!(
        deny.contains("multiple-versions = \"warn\"")
            || deny.contains("GAP-SSH-DENY-002"),
        "política DENY-002 documentada"
    );
}

// --- CHG-001 / REL presence of changelog section ---

#[test]
fn gap_chg_001_changelog_tem_039() {
    let ch = std::fs::read_to_string("CHANGELOG.md").expect("CHANGELOG");
    assert!(ch.contains("## [0.3.9]"), "CHANGELOG deve ter seção 0.3.9");
    assert!(
        ch.contains("[0.3.9]:") || ch.contains("compare/v0.3.8"),
        "CHANGELOG deve ter âncora/link 0.3.9"
    );
}
