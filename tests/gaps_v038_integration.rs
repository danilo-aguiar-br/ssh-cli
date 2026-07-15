// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regressão e2e 1:1 dos gaps produto 0.3.7 + residuais 0.3.8 (TEST-004).
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
    if let Some(value) = llvm_profile_file {
        c.env("LLVM_PROFILE_FILE", value);
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
fn gap_val_001_rejects_path_traversal_name() {
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
fn gap_val_001_rejects_con_name() {
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
fn gap_val_002_rejects_port_zero() {
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
        .stderr(predicate::str::contains("port").or(predicate::str::contains("porta")));
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

#[test]
#[serial]
fn gap_val_004_key_lixo_rejeitada() {
    let tmp = TempDir::new().unwrap();
    let key = tmp.path().join("not-a-key");
    std::fs::write(&key, b"this is not an openssh private key\n").unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "badkey",
            "--host",
            "h",
            "--user",
            "u",
            "--key",
            key.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(64)
        .stderr(
            predicate::str::contains("inválid")
                .or(predicate::str::contains("invalid"))
                .or(predicate::str::contains("OpenSSH")),
        );
}

// --- IO ---

#[test]
#[serial]
fn gap_io_001_output_format_json_empty_list() {
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
fn gap_io_003_json_error_envelope() {
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
fn gap_io_005_doctor_json_sem_println_cru() {
    let tmp = TempDir::new().unwrap();
    cmd_json(&tmp)
        .args(["vps", "doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

#[test]
#[serial]
fn gap_io_006_tunnel_no_banner_nontty() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "tun1");
    let assert = cmd_json(&tmp)
        .args([
            "tunnel",
            "tun1",
            "18765",
            "127.0.0.1",
            "9",
            "--timeout-ms",
            "80",
        ])
        .assert()
        .failure();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains("Tunnel SSH:"),
        "banner vazou em stdout agent: {stdout}"
    );
    assert!(
        !stdout.contains("Pressione Ctrl+C"),
        "banner Ctrl+C vazou: {stdout}"
    );
}

// --- TUN / SCP / STATE ---

#[test]
#[serial]
fn gap_tun_001_timeout_curto_nao_hang() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "tun2");
    let start = std::time::Instant::now();
    cmd(&tmp)
        .args([
            "tunnel",
            "tun2",
            "18766",
            "203.0.113.50",
            "22",
            "--timeout-ms",
            "100",
        ])
        .assert()
        .failure();
    assert!(
        start.elapsed().as_secs() < 15,
        "tunnel com timeout 100ms não deve hang >15s"
    );
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
fn gap_perm_001_lock_mode_via_save() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "perm1");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for entry in std::fs::read_dir(tmp.path()).unwrap() {
            let p = entry.unwrap().path();
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.ends_with(".toml") {
                let mode = std::fs::metadata(&p).unwrap().permissions().mode() & 0o777;
                assert_eq!(mode, 0o600, "toml perms {name}: {mode:o}");
            }
        }
    }
}

// --- CLI ---

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
fn gap_cli_003_completions_bash_escreve() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ssh-cli").or(predicate::str::contains("_ssh")));
}

// --- SEC / EXIT / IMP ---

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
fn gap_sec_001_packing_unit_via_lib() {
    use secrecy::SecretString;
    use ssh_cli::ssh::packing::{pack_su, pack_sudo};
    let password = SecretString::from("never-in-argv-sec001".to_string());
    let s = pack_sudo("id", Some(&password));
    assert!(!s.command.contains("never-in-argv-sec001"));
    let u = pack_su("id", &password);
    assert!(!u.command.contains("never-in-argv-sec001"));
}

#[test]
#[serial]
fn gap_exit_002_sem_vps_ativa_66() {
    let tmp = TempDir::new().unwrap();
    cmd_json(&tmp)
        .args(["health-check"])
        .assert()
        .failure()
        .code(66)
        .stderr(predicate::str::contains("\"exit_code\":66"));
}

#[test]
#[serial]
fn gap_exit_001_envelope_schema() {
    let schema = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/schemas/error-envelope.schema.json");
    assert!(schema.is_file());
    let body = std::fs::read_to_string(schema).unwrap();
    assert!(body.contains("exit_code"));
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

// --- DEP / TEST meta ---

#[test]
#[serial]
fn gap_dep_002_russh_patched_no_lock() {
    let lock = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock");
    let body = std::fs::read_to_string(lock).unwrap();
    let mut found = false;
    let mut lines = body.lines();
    while let Some(line) = lines.next() {
        if line == "name = \"russh\"" {
            if let Some(v) = lines.next() {
                assert!(
                    v.contains("0.62.") || v.contains("0.61.") || v.contains("0.60.3"),
                    "russh must be patched: {v}"
                );
                assert!(!v.contains("\"0.60.0\""));
                found = true;
                break;
            }
        }
    }
    assert!(found, "russh package missing in Cargo.lock");
    let deny =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("deny.toml"))
            .unwrap();
    assert!(!deny.contains("RUSTSEC-2026-0153"));
    assert!(!deny.contains("RUSTSEC-2026-0154"));
    assert!(deny.contains("yanked = \"deny\"") || deny.contains("yanked=\"deny\""));
}

#[test]
#[serial]
fn gap_test_001_signals_api() {
    let _ = ssh_cli::signals::is_cancelled();
    let _ = ssh_cli::signals::is_terminated();
}

#[test]
#[serial]
fn gap_test_002_version_contem_semver() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"ssh-cli 0\.\d+\.\d+ \(").unwrap());
}

#[test]
#[serial]
fn gap_test_003_padrao_abort_nao_tautologico() {
    use ssh_cli::ssh::packing::remote_abort_pattern;
    assert!(remote_abort_pattern("$(rm -rf)").is_none());
    assert_eq!(remote_abort_pattern("sleep 9"), Some("sleep 9".into()));
}
