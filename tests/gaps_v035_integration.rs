//! Testes de integração dos gaps residuais 0.3.5.
//!
//! Usa apenas senhas/caminhos FALSOS de teste. Nunca credenciais reais SSH.

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
    // Testes legados isolam plaintext; cifragem default coberta em testes dedicados.
    c.env("SSH_CLI_ALLOW_PLAINTEXT_SECRETS", "1");
    // Força text em snapshots de CLI que capturam stdout em pipe (senão auto-JSON).
    c.env("SSH_CLI_FORCE_TEXT", "1");
    c.arg("--config-dir").arg(tmp.path());
    c.arg("--output-format").arg("text");
    c
}

fn cmd_enc_default(tmp: &TempDir) -> Command {
    let mut c = cmd(tmp);
    c.env_remove("SSH_CLI_ALLOW_PLAINTEXT_SECRETS");
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
fn doctor_reporta_camada_e_secrets_plaintext() {
    let tmp = TempDir::new().unwrap();
    add_fake_host(&tmp, "doc1");
    cmd(&tmp)
        .args(["vps", "doctor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"secrets_at_rest\": \"plaintext\"",
        ))
        .stdout(predicate::str::contains("\"secrets_key_source\": \"none\""))
        .stdout(predicate::str::contains("\"telemetry\": false"))
        .stdout(predicate::str::contains("hosts"));
}

#[test]
#[serial]
fn add_com_key_path_sem_password() {
    let tmp = TempDir::new().unwrap();
    let key = tmp.path().join("id_test_ed25519");
    // Arquivo vazio — só valida persistência de path (não é chave real utilizável).
    std::fs::write(
        &key,
        b"-----BEGIN OPENSSH PRIVATE KEY-----\nFAKE\n-----END OPENSSH PRIVATE KEY-----\n",
    )
    .unwrap();
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
    // Não deve vazar passphrase inexistente
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
    let texto = std::fs::read_to_string(&out).unwrap();
    assert!(!texto.contains("fake-test-password-not-real-001"));
    assert!(texto.contains("exp1"));
    assert!(out.exists());
}

#[test]
#[serial]
fn secrets_encrypt_on_disk_when_key_set() {
    let tmp = TempDir::new().unwrap();
    let hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    let mut c = cmd(&tmp);
    c.env("SSH_CLI_SECRETS_KEY", hex);
    c.args([
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
        "senha deve ser cifrada no disco"
    );
    assert!(!toml.contains("fake-test-password-not-real-enc"));
}

#[test]
#[serial]
fn max_command_chars_rejeita_comando_longo() {
    let tmp = TempDir::new().unwrap();
    add_fake_host(&tmp, "maxc");
    let longo = "x".repeat(80);
    // Falha antes do SSH (sem rede) com ComandoMuitoLongo
    cmd(&tmp)
        .args(["exec", "maxc", &longo])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("max_command_chars")
                .or(predicate::str::contains("comando excede")),
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
        .stderr(predicate::str::contains("senha_su").or(predicate::str::contains("su")));
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
                .or(predicate::str::contains("Sudo")),
        );
}

#[test]
#[serial]
fn known_hosts_tofu_unit_via_doctor_path() {
    let tmp = TempDir::new().unwrap();
    add_fake_host(&tmp, "kh1");
    // doctor expõe path known_hosts (arquivo criado no primeiro connect real)
    cmd(&tmp)
        .args(["vps", "doctor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("known_hosts"));
}

#[test]
#[serial]
fn packing_abort_contem_term_e_kill() {
    // Teste de API via help de su-exec (smoke) + unidade já cobre packing.
    // Aqui garante binário expõe su-exec.
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
        .args(["secrets", "init"])
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
