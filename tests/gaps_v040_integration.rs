//! Regressão 1:1 dos gaps AUD-SCP fechados na v0.4.0.
//!
//! IDs: SCP-010..023, REL-004, DOC-004, TEST-004, IO-007.
//! Credenciais FALSAS apenas; sem rede real.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::path::Path;
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
            "fake-test-password-not-real-040",
        ])
        .assert()
        .success();
}

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

// --- version / packaging ---

#[test]
fn gap_version_040() {
    let v = env!("CARGO_PKG_VERSION");
    assert_eq!(v, "0.4.0", "Cargo.toml must be 0.4.0 for this suite");
}

#[test]
#[serial]
fn gap_version_cli_contem_040() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.4.0"));
}

// --- SCP-017 flags ---

#[test]
#[serial]
fn gap_scp_017_help_contem_flags() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["scp", "upload", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--timeout"))
        .stdout(predicate::str::contains("--password-stdin"))
        .stdout(predicate::str::contains("--key"))
        .stdout(predicate::str::contains("--key-passphrase"))
        .stdout(predicate::str::contains("--json"));
}

#[test]
#[serial]
fn gap_scp_017_password_stdin_conflict() {
    let tmp = TempDir::new().unwrap();
    let f = tmp.path().join("x.bin");
    std::fs::write(&f, b"x").unwrap();
    cmd(&tmp)
        .args([
            "scp",
            "upload",
            "any",
            f.to_str().unwrap(),
            "/tmp/x",
            "--password",
            "a",
            "--password-stdin",
        ])
        .assert()
        .failure();
}

// --- SCP-019 dir reject ---

#[test]
#[serial]
fn gap_scp_019_upload_diretorio_rejeita() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "scpdir");
    cmd(&tmp)
        .args([
            "scp",
            "upload",
            "scpdir",
            tmp.path().to_str().unwrap(),
            "/tmp/x",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("regular files").or(predicate::str::contains("arquivo")));
}

// --- SCP-001 still holds ---

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
            tmp.path().join("missing-040.bin").to_str().unwrap(),
            "/tmp/x",
        ])
        .assert()
        .failure();
}

// --- IO-007 JSON surface ---

#[test]
#[serial]
fn gap_io_007_scp_json_flag_na_help() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["scp", "download", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--json"));
}

// --- DOC-004 / product line ---

#[test]
fn gap_doc_004_product_line_040_e_file_only() {
    let readme = std::fs::read_to_string(root().join("README.md")).expect("README");
    assert!(
        readme.contains("0.4.0"),
        "README must mention product line 0.4.0"
    );
    let lower = readme.to_lowercase();
    assert!(
        lower.contains("regular file")
            || lower.contains("file-only")
            || lower.contains("files only")
            || lower.contains("not directories"),
        "README must document file-only SCP limit"
    );
}

#[test]
fn gap_rel_004_changelog_039_scp_broken_e_040() {
    let ch = std::fs::read_to_string(root().join("CHANGELOG.md")).expect("CHANGELOG");
    assert!(ch.contains("0.4.0"), "CHANGELOG must have 0.4.0 section");
    let lower = ch.to_lowercase();
    assert!(
        lower.contains("0.3.9")
            && (lower.contains("broken") || lower.contains("inoperant") || lower.contains("wire")),
        "CHANGELOG must honestly mention 0.3.9 SCP wire issue"
    );
}

#[test]
fn gap_scp_021_schema_scp_transfer() {
    let schema = root().join("docs/schemas/scp-transfer.schema.json");
    assert!(schema.is_file(), "missing {}", schema.display());
    let body = std::fs::read_to_string(&schema).unwrap();
    assert!(body.contains("direction"));
    assert!(body.contains("bytes"));
    assert!(body.contains("duration_ms"));
}

#[test]
fn gap_e2e_script_e10_e12() {
    let script = std::fs::read_to_string(root().join("scripts/e2e_real_ssh.sh")).unwrap();
    assert!(script.contains("pass E10") || script.contains("PASS E10") || script.contains("E10"));
    assert!(script.contains("E11"));
    assert!(script.contains("E12"));
    assert!(script.contains("E13"));
    assert!(
        script.contains("E14") && script.contains("preserve"),
        "e2e must cover SCP-023 mode/mtime preserve (E14)"
    );
    assert!(script.contains("scp upload") || script.contains("scp download"));
}

#[test]
fn gap_scp_023_comando_remoto_usa_p() {
    let src = std::fs::read_to_string(root().join("src/ssh/cliente.rs")).unwrap();
    assert!(
        src.contains("modo_p") || src.contains("-tp") || src.contains("format!(\"{modo}p\")"),
        "remote scp must request -p (OpenSSH source emits T only with -p)"
    );
    assert!(
        src.contains("aplicar_mode_local") || src.contains("set_permissions"),
        "download must apply remote mode from C-header"
    );
}

#[test]
fn gap_io_008_tunnel_json_flag() {
    let help = Command::new(env!("CARGO_BIN_EXE_ssh-cli"))
        .args(["tunnel", "--help"])
        .output()
        .expect("tunnel --help");
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(
        stdout.contains("--json"),
        "tunnel must expose --json (GAP-SSH-IO-008): {stdout}"
    );
    let src = std::fs::read_to_string(root().join("src/tunnel.rs")).unwrap();
    assert!(
        src.contains("imprimir_tunnel_listening_json") || src.contains("tunnel_listening"),
        "tunnel must emit structured listening JSON"
    );
    let out = std::fs::read_to_string(root().join("src/output.rs")).unwrap();
    assert!(
        out.contains("imprimir_tunnel_listening_json"),
        "output must define tunnel listening JSON printer"
    );
}

#[test]
fn gap_scp_022_partial_suffix_na_fonte() {
    let src = std::fs::read_to_string(root().join("src/ssh/cliente.rs")).unwrap();
    assert!(
        src.contains("ssh-cli.partial") || src.contains("SCP_PARTIAL_SUFFIX"),
        "download must use partial file path for atomic write"
    );
    assert!(
        !src.contains("std::fs::read(local)"),
        "upload must not load entire file with fs::read"
    );
    // SCP-022b: mode/times no partial antes do rename (sem residual pós-rename).
    assert!(
        src.contains("aplicar_mode_local(&partial") || src.contains("aplicar_mode_local(&partial,"),
        "mode must be applied on partial before rename"
    );
}

#[test]
fn gap_scp_020_i18n_mensagens() {
    let src = std::fs::read_to_string(root().join("src/i18n.rs")).unwrap();
    assert!(src.contains("ScpUploadConcluido"));
    assert!(src.contains("ScpDownloadConcluido"));
    assert!(src.contains("ScpUploadSomenteArquivo"));
    assert!(src.contains("ScpDownloadLocalNaoDiretorio"));
    assert!(src.contains("Upload completed"));
    assert!(src.contains("Upload concluído") || src.contains("Upload concluido"));
}

/// IO-007b: `scp --json` local promove envelope de erro JSON (paridade tunnel).
#[test]
#[serial]
fn gap_io_007b_scp_json_local_envelope_erro() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "jsonscp");
    cmd(&tmp)
        .args([
            "scp",
            "upload",
            "jsonscp",
            "--json",
            tmp.path().to_str().unwrap(),
            "/tmp/x",
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("\"exit_code\"").and(predicate::str::contains("\"message\"")),
        );
}

#[test]
fn gap_scp_010_header_unit_source() {
    let src = std::fs::read_to_string(root().join("src/ssh/cliente.rs")).unwrap();
    assert!(src.contains("formatar_header_upload_scp"));
    assert!(src.contains("formatar_linha_t_scp"));
    assert!(src.contains("SCP_OK"));
}
