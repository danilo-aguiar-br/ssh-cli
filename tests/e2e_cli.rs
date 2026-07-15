// SPDX-License-Identifier: MIT OR Apache-2.0
//! Testes E2E da CLI via `assert_cmd`.
//!
//! TODOS os tests usam `--config-dir <TempDir>` para isolar completamente o
//! estado do sistema real. Testes que escrevem/leem env vars são marcados
//! com `#[serial]`.

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
    c.arg("--config-dir").arg(tmp.path());
    c
}

#[test]
#[serial]
fn testa_help() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("ssh-cli"))
        .stdout(predicate::str::contains("vps"))
        .stdout(predicate::str::contains("exec"));
}

#[test]
#[serial]
fn testa_version() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ssh-cli"));
}

#[test]
#[serial]
fn testa_vps_add_cria_registro() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "teste",
            "--host",
            "1.2.3.4",
            "--port",
            "22",
            "--user",
            "root",
            "--password",
            "senha-super-secreta-123",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("teste"));
}

#[test]
#[serial]
fn testa_vps_add_duplicado_retorna_erro() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "dupe",
            "--host",
            "1.2.3.4",
            "--user",
            "root",
            "--password",
            "senha-super-secreta-123",
        ])
        .assert()
        .success();

    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "dupe",
            "--host",
            "1.2.3.4",
            "--user",
            "root",
            "--password",
            "outra-senha-super-secreta",
        ])
        .assert()
        .failure();
}

#[test]
#[serial]
fn testa_vps_list_mascara_senhas() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "alfa",
            "--host",
            "a.example.com",
            "--user",
            "admin",
            "--password",
            "senha-muito-longa-para-mascarar-123",
        ])
        .assert()
        .success();

    cmd(&tmp)
        .args(["vps", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alfa"))
        // Senha NÃO pode aparecer inteira
        .stdout(predicate::str::contains("senha-muito-longa-para-mascarar-123").not())
        // GAP-SSH-SEC-002: mask agent-safe sempre "***"
        .stdout(predicate::str::contains("***"));
}

#[test]
#[serial]
fn testa_vps_list_json_funciona() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "beta",
            "--host",
            "b.example.com",
            "--user",
            "admin",
            "--password",
            "senha-muito-longa-para-mascarar-456",
        ])
        .assert()
        .success();

    cmd(&tmp)
        .args(["vps", "list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\""))
        .stdout(predicate::str::contains("\"beta\""));
}

#[test]
#[serial]
fn testa_vps_remove_existente() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "remover",
            "--host",
            "r.example.com",
            "--user",
            "root",
            "--password",
            "senha-muito-longa-para-mascarar",
        ])
        .assert()
        .success();

    cmd(&tmp)
        .args(["vps", "remove", "remover"])
        .assert()
        .success()
        .stdout(predicate::str::contains("remover"));
}

#[test]
#[serial]
fn testa_vps_remove_inexistente_retorna_erro() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["vps", "remove", "nao-existe"])
        .assert()
        .failure();
}

#[test]
#[serial]
fn testa_vps_edit_atualiza_campos() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "editar",
            "--host",
            "antigo.example.com",
            "--user",
            "root",
            "--password",
            "senha-original-muito-longa",
        ])
        .assert()
        .success();

    cmd(&tmp)
        .args([
            "vps",
            "edit",
            "editar",
            "--host",
            "novo.example.com",
            "--port",
            "2222",
        ])
        .assert()
        .success();

    cmd(&tmp)
        .args(["vps", "show", "editar"])
        .assert()
        .success()
        .stdout(predicate::str::contains("novo.example.com"))
        .stdout(predicate::str::contains("2222"));
}

#[test]
#[serial]
fn testa_vps_show_retorna_dados_mascarados() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "mostrar",
            "--host",
            "s.example.com",
            "--user",
            "admin",
            "--password",
            "senha-longa-para-mascaramento-total",
        ])
        .assert()
        .success();

    cmd(&tmp)
        .args(["vps", "show", "mostrar"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mostrar"))
        .stdout(predicate::str::contains("senha-longa-para-mascaramento-total").not())
        .stdout(predicate::str::contains("***"));
}

#[test]
#[serial]
fn testa_vps_show_json_mascara() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "jshow",
            "--host",
            "j.example.com",
            "--user",
            "admin",
            "--password",
            "senha-ultra-secreta-para-mascarar-json",
        ])
        .assert()
        .success();

    cmd(&tmp)
        .args(["vps", "show", "jshow", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("jshow"))
        .stdout(predicate::str::contains("senha-ultra-secreta-para-mascarar-json").not());
}

#[test]
#[serial]
fn testa_vps_path_retorna_caminho() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["vps", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("config.toml"));
}

#[test]
#[serial]
fn testa_connect_seleciona_vps() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "prod",
            "--host",
            "p.example.com",
            "--user",
            "admin",
            "--password",
            "senha-muito-longa-prod",
        ])
        .assert()
        .success();

    cmd(&tmp)
        .args(["connect", "prod"])
        .assert()
        .success()
        .stdout(predicate::str::contains("prod"));
}

#[test]
#[serial]
fn testa_connect_vps_inexistente_retorna_erro() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp).args(["connect", "fantasma"]).assert().failure();
}

#[test]
#[serial]
fn testa_list_vazio_mostra_mensagem() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp).args(["vps", "list"]).assert().success();
}

#[test]
#[serial]
fn secrets_init_force_reencrypts_hosts() {
    // GAP-AUD-SEC-001: --force must re-encrypt existing host secrets under the new key.
    let tmp = TempDir::new().unwrap();
    cmd(&tmp).args(["secrets", "init"]).assert().success();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "rotate-me",
            "--host",
            "1.2.3.4",
            "--user",
            "root",
            "--password",
            "rotate-secret-password-zzz-999",
        ])
        .assert()
        .success();

    let cfg = tmp.path().join("config.toml");
    let before = std::fs::read_to_string(&cfg).unwrap();
    assert!(before.contains("sshcli-enc:v1:"));

    cmd(&tmp)
        .args(["secrets", "init", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("primary-key ready").or(predicate::str::contains("primary-key pronta")));

    // Config remains readable (not stuck on wrong key).
    cmd(&tmp)
        .args(["vps", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rotate-me"))
        .stdout(predicate::str::contains("rotate-secret-password-zzz-999").not());

    let after = std::fs::read_to_string(&cfg).unwrap();
    assert!(after.contains("sshcli-enc:v1:"));
    // Ciphertext must change under new key (nonce+key differ).
    assert_ne!(before, after);

    // Backup of previous primary key is created.
    assert!(tmp.path().join("secrets.key.bak").is_file());
}

#[test]
#[serial]
fn vps_add_rejects_whitespace_name() {
    // GAP-AUD-VAL-001
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "a b",
            "--host",
            "1.2.3.4",
            "--user",
            "root",
            "--password",
            "long-enough-password-xyz",
        ])
        .assert()
        .failure()
        .code(64);
}

