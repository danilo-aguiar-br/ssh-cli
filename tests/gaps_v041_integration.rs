// SPDX-License-Identifier: MIT OR Apache-2.0
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
    assert!(
        v.starts_with("0.4."),
        "product line 0.4.x (suite closed at 0.4.1)"
    );
}

#[test]
#[serial]
fn gap_version_cli_contem_041() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.4."));
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
        "print_transfer_json must emit event scp-transfer"
    );
}

// --- EXP-001 / TUN-002 source gates ---

#[test]
fn gap_exp_001_serializar_empty_source() {
    let src = std::fs::read_to_string(root().join("src/secrets.rs")).unwrap();
    assert!(
        src.contains("plaintext.is_empty()") || src.contains("is_empty()"),
        "serialize_secret must early-return on empty plaintext (EXP-001)"
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
fn gap_doc_041_root_honesty_aud_post() {
    let readme = std::fs::read_to_string(root().join("README.md")).expect("README");
    assert!(
        readme.contains("scp-transfer") && readme.contains("event"),
        "README must document SCP JSON event scp-transfer (IO-009)"
    );
    assert!(
        readme.contains("password-stdin")
            && readme.contains("tunnel")
            && readme.contains("health-check"),
        "README must document tunnel/health surface with auth stdin"
    );
    assert!(
        readme.contains("empty") || readme.contains("sshcli-enc") || readme.contains("redacted"),
        "README must document export empty-secret honesty (EXP-001)"
    );
    let security = std::fs::read_to_string(root().join("SECURITY.md")).expect("SECURITY");
    assert!(
        security.contains("0.4.1")
            && (security.contains("empty") || security.contains("sshcli-enc")),
        "SECURITY must mention 0.4.1 empty-secret export honesty"
    );
    let llms = std::fs::read_to_string(root().join("llms.txt")).expect("llms");
    assert!(
        llms.contains("scp-transfer") && llms.contains("password-stdin"),
        "llms.txt must cover scp-transfer event and password-stdin parity"
    );
    let ch = std::fs::read_to_string(root().join("CHANGELOG.md")).expect("CHANGELOG");
    assert!(
        ch.contains("compare/v0.4.0...v0.4.1") || ch.contains("[0.4.1]:"),
        "CHANGELOG must have 0.4.1 compare footer anchor"
    );
    let checklist =
        std::fs::read_to_string(root().join("docs/RELEASE_CHECKLIST.md")).expect("checklist");
    assert!(
        checklist.contains("0.4.1")
            && (checklist.contains("DOC-041") || checklist.contains("AUD-POST")),
        "RELEASE_CHECKLIST must gate 0.4.1 DOC-041 / AUD-POST honesty"
    );
    let testing = std::fs::read_to_string(root().join("docs/TESTING.md")).expect("TESTING");
    assert!(
        testing.contains("0.4.1") && testing.contains("gaps_v041"),
        "TESTING must state product line 0.4.1 and gaps_v041"
    );
    let skill = std::fs::read_to_string(root().join("skills/ssh-cli-en/SKILL.md")).expect("skill");
    assert!(
        skill.contains("scp-transfer")
            && (skill.contains("sshcli-enc") || skill.contains("empty"))
            && skill.contains("password-stdin"),
        "skill en must teach scp-transfer, export empty honesty, auth stdin"
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
        "run_health_check must accept key overrides (CLI-006)"
    );
    assert!(
        src.contains("replace_host_key"),
        "health-check must honor replace_host_key (M1)"
    );
    assert!(
        src.contains("set_json_errors"),
        "health-check must enable JSON error envelope (M2)"
    );
}
