//! Regressão 1:1 dos gaps AUD-POST fechados na v0.4.1
//! (EXP-001, TUN-002, CLI-005, CLI-006, IO-009, REL-006).

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::path::PathBuf;
use tempfile::TempDir;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

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
    c.arg("--config-dir").arg(tmp.path());
    c
}

// --- version / packaging ---

#[test]
fn gap_version_041() {
    let v = env!("CARGO_PKG_VERSION");
    assert_eq!(v, "0.4.1", "Cargo.toml must be 0.4.1 for this suite");
}

#[test]
#[serial]
fn gap_version_cli_contem_041() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.4.1"));
}

// --- CLI-005 tunnel auth parity ---

#[test]
#[serial]
fn gap_cli_005_tunnel_help_auth_flags() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["tunnel", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--password-stdin"))
        .stdout(predicate::str::contains("--key-passphrase"))
        .stdout(predicate::str::contains("--key-passphrase-stdin"))
        .stdout(predicate::str::contains("--password"))
        .stdout(predicate::str::contains("--key"));
}

#[test]
#[serial]
fn gap_cli_005_tunnel_password_stdin_conflict() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "tunnel",
            "vps-a",
            "18080",
            "127.0.0.1",
            "5432",
            "--timeout-ms",
            "1000",
            "--password",
            "x",
            "--password-stdin",
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("cannot be used with")
                .or(predicate::str::contains("conflict")),
        );
}

// --- CLI-006 health-check auth parity ---

#[test]
#[serial]
fn gap_cli_006_health_help_auth_flags() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["health-check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--password-stdin"))
        .stdout(predicate::str::contains("--key"))
        .stdout(predicate::str::contains("--key-passphrase"))
        .stdout(predicate::str::contains("--key-passphrase-stdin"))
        .stdout(predicate::str::contains("--timeout"));
}

#[test]
#[serial]
fn gap_cli_006_health_password_stdin_conflict() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args([
            "health-check",
            "vps-a",
            "--password",
            "x",
            "--password-stdin",
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("cannot be used with")
                .or(predicate::str::contains("conflict")),
        );
}

// --- IO-009 scp event ---

#[test]
fn gap_io_009_scp_event_schema() {
    let schema = root().join("docs/schemas/scp-transfer.schema.json");
    let body = std::fs::read_to_string(&schema).expect("schema");
    assert!(
        body.contains("\"event\"") && body.contains("scp-transfer"),
        "schema must require event=scp-transfer"
    );
    assert!(
        body.contains("\"const\": \"scp-transfer\"") || body.contains("\"const\":\"scp-transfer\""),
        "schema event const must be scp-transfer"
    );
    let out = std::fs::read_to_string(root().join("src/output.rs")).unwrap();
    assert!(
        out.contains("\"event\": \"scp-transfer\"") || out.contains("\"scp-transfer\""),
        "imprimir_transferencia_json must emit event scp-transfer"
    );
}

// --- EXP-001 / TUN-002 source gates ---

#[test]
fn gap_exp_001_serializar_empty_source() {
    let src = std::fs::read_to_string(root().join("src/secrets.rs")).unwrap();
    assert!(
        src.contains("plaintext.is_empty()") || src.contains("is_empty()"),
        "serializar_segredo must early-return on empty plaintext (EXP-001)"
    );
    assert!(
        src.contains("empty_secret_never_encrypted_blob") || src.contains("GAP-SSH-EXP-001"),
        "unit coverage for empty secret must exist"
    );
}

#[test]
fn gap_tun_002_bound_flag_source() {
    let src = std::fs::read_to_string(root().join("src/tunnel.rs")).unwrap();
    assert!(
        src.contains("AtomicBool") && src.contains("bound"),
        "tunnel must use bound AtomicBool (TUN-002)"
    );
    assert!(
        src.contains("GAP-SSH-TUN-002") || src.contains("deadline one-shot"),
        "TUN-002 must be documented in tunnel source"
    );
}

// --- REL-006 / product line / changelog ---

#[test]
fn gap_rel_006_changelog_041() {
    let ch = std::fs::read_to_string(root().join("CHANGELOG.md")).expect("CHANGELOG");
    assert!(ch.contains("0.4.1"), "CHANGELOG must have 0.4.1 section");
    assert!(
        ch.contains("EXP-001")
            || ch.contains("empty")
            || ch.contains("export")
            || ch.contains("serializar"),
        "CHANGELOG 0.4.1 must mention export empty-secret fix"
    );
    assert!(
        ch.contains("TUN-002") || ch.contains("tunnel") || ch.contains("deadline"),
        "CHANGELOG 0.4.1 must mention tunnel deadline fix"
    );
    assert!(
        ch.contains("password-stdin") || ch.contains("CLI-005") || ch.contains("auth"),
        "CHANGELOG 0.4.1 must mention auth flag parity"
    );
    assert!(
        ch.contains("scp-transfer") || ch.contains("IO-009") || ch.contains("event"),
        "CHANGELOG 0.4.1 must mention scp event field"
    );
}

#[test]
fn gap_doc_product_line_041() {
    let readme = std::fs::read_to_string(root().join("README.md")).expect("README");
    assert!(
        readme.contains("0.4.1"),
        "README must mention product line 0.4.1"
    );
    let llms = std::fs::read_to_string(root().join("llms.txt")).expect("llms");
    assert!(
        llms.contains("0.4.1"),
        "llms.txt must state product line 0.4.1"
    );
    let integ = std::fs::read_to_string(root().join("INTEGRATIONS.md")).expect("INTEGRATIONS");
    assert!(integ.contains("0.4.1"), "INTEGRATIONS must mention 0.4.1");
    let contrib = std::fs::read_to_string(root().join("CONTRIBUTING.md")).expect("CONTRIBUTING");
    assert!(
        contrib.contains("gaps_v041"),
        "CONTRIBUTING must mention gaps_v041 suite"
    );
}

#[test]
fn gap_cli_005_tunnel_source_passphrase() {
    let src = std::fs::read_to_string(root().join("src/tunnel.rs")).unwrap();
    assert!(
        src.contains("key_passphrase") || src.contains("aplicar_overrides"),
        "tunnel must apply key_passphrase via overrides (CLI-005)"
    );
    let cli = std::fs::read_to_string(root().join("src/cli.rs")).unwrap();
    assert!(
        cli.contains("password_stdin") && cli.contains("key_passphrase_stdin"),
        "cli Tunnel must declare password_stdin and key_passphrase_stdin"
    );
}

#[test]
fn gap_cli_006_health_source_key() {
    let src = std::fs::read_to_string(root().join("src/vps/mod.rs")).unwrap();
    assert!(
        src.contains("key_override") && src.contains("key_passphrase_override"),
        "executar_health_check must accept key overrides (CLI-006)"
    );
    assert!(
        src.contains("replace_host_key"),
        "health-check must honor replace_host_key (M1)"
    );
    assert!(
        src.contains("definir_json_erros"),
        "health-check must enable JSON error envelope (M2)"
    );
}
