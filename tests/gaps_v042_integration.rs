// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regressão 1:1 dos gaps AUD-E2E fechados na v0.4.2
//! (TUN-003, IO-010, UX-001, REL-007, ENV-001, DOC-042, SCP-024, REL-008).

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
    if let Some(value) = llvm_profile_file {
        c.env("LLVM_PROFILE_FILE", value);
    }
    c.env("HOME", tmp.path());
    c.env("XDG_CONFIG_HOME", tmp.path());
    c.arg("--config-dir").arg(tmp.path());
    c
}

// --- version / packaging ---

#[test]
fn gap_version_042() {
    let v = env!("CARGO_PKG_VERSION");
    // Suite closed at 0.4.2; product line advanced to 0.5.0 (EN/API rename + force-init fix).
    assert!(
        v == "0.5.0" || v.starts_with("0.5.") || v == "0.4.2",
        "Cargo.toml product line must be 0.5.x (got {v})"
    );
}

#[test]
#[serial]
fn gap_version_cli_contem_042() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

// --- TUN-003 ---

#[test]
fn gap_tun_003_source_local_addr() {
    let src = std::fs::read_to_string(root().join("src/tunnel.rs")).expect("tunnel.rs");
    assert!(
        src.contains("local_addr()"),
        "tunnel must read local_addr() after bind (TUN-003)"
    );
    assert!(
        src.contains("effective_port"),
        "tunnel must expose effective_port in JSON event"
    );
}

#[test]
fn gap_tun_003_schema_min_1() {
    let schema =
        std::fs::read_to_string(root().join("docs/schemas/tunnel-listening.schema.json")).unwrap();
    assert!(
        schema.contains("\"minimum\": 1") || schema.contains("\"minimum\":1"),
        "tunnel-listening local_port minimum must be 1"
    );
    assert!(
        schema.contains("TUN-003") || schema.contains("ephemeral"),
        "schema should document TUN-003 / ephemeral port"
    );
}

// --- IO-010 ---

#[test]
fn gap_io_010_source_classificar() {
    let src = std::fs::read_to_string(root().join("src/ssh/client_real_scp.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/client_real_core.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/scp_wire.rs")).unwrap();
    assert!(
        src.contains("classificar_mensagem_scp")
            || src.contains("classify_scp_message")
            || src.contains("no such file"),
        "SCP client must classify remote missing messages"
    );
    assert!(
        src.contains("FileNotFound"),
        "SCP path must map to FileNotFound for missing remote"
    );
}

// --- UX-001 ---

#[test]
#[serial]
fn gap_ux_001_export_json_help_and_flag() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["vps", "export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--json"));
}

#[test]
#[serial]
fn gap_ux_001_export_json_envelope() {
    let tmp = TempDir::new().unwrap();
    // empty registry still exports valid envelope
    let assert = cmd(&tmp)
        .args(["vps", "export", "--json"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        out.contains("\"event\"") && out.contains("vps-export"),
        "export --json must emit event vps-export, got: {out}"
    );
    assert!(
        out.contains("\"ok\"") && out.contains("true"),
        "export --json must have ok:true"
    );
    assert!(
        !out.contains("sshcli-enc:"),
        "redacted export JSON must never contain sshcli-enc ciphertext"
    );
}

// --- REL-007 ---

#[test]
fn gap_rel_007_build_rs_precedence() {
    let src = std::fs::read_to_string(root().join("build.rs")).unwrap();
    assert!(
        src.contains("SSH_CLI_COMMIT_HASH"),
        "build.rs must honor SSH_CLI_COMMIT_HASH env"
    );
    assert!(
        src.contains(".commit_hash"),
        "build.rs must read .commit_hash pack file"
    );
    assert!(
        src.contains("rerun-if-changed=.commit_hash")
            || src.contains("rerun-if-changed=.commit_hash"),
        "build.rs must rerun on .commit_hash"
    );
    let ch = root().join(".commit_hash");
    assert!(
        ch.is_file(),
        ".commit_hash must exist for crates.io pack (REL-007)"
    );
    let hash = std::fs::read_to_string(&ch).unwrap();
    assert!(
        !hash.trim().is_empty() && hash.trim() != "unknown",
        ".commit_hash must be non-empty real hash"
    );
}

// --- ENV-001 ---

#[test]
fn gap_env_001_e2e_script_auth_policy() {
    let script = std::fs::read_to_string(root().join("scripts/e2e_real_ssh.sh")).unwrap();
    assert!(
        script.contains("ENV-001") || script.contains("fail2ban"),
        "e2e script must document fail2ban / ENV-001 policy"
    );
    assert!(
        script.contains("PROIBIDO") || script.contains("no máximo 1") || script.contains("max 1"),
        "e2e must forbid mass auth-fail loops"
    );
    assert!(
        script.contains("E13_EC") || script.contains("eq 66"),
        "E13 must assert exit 66 (IO-010)"
    );
    assert!(
        script.contains("E15") && script.contains("local_port"),
        "E15 tunnel port 0 case required"
    );
    assert!(
        script.contains("E16") && script.contains("symlink"),
        "E16 symlink case required"
    );
}

// --- DOC-042 ---

#[test]
fn gap_doc_042_tunnel_positional_skills() {
    let en = std::fs::read_to_string(root().join("skills/ssh-cli-en/SKILL.md")).unwrap();
    let pt = std::fs::read_to_string(root().join("skills/ssh-cli-pt/SKILL.md")).unwrap();
    // positional form in examples
    assert!(
        en.contains("tunnel prod ") || en.contains("tunnel <"),
        "EN skill must document positional tunnel args"
    );
    // Honesty notes may say "NEVER invent --local-port"; forbid documenting it as a real flag usage.
    for (label, body) in [("EN", &en), ("PT", &pt)] {
        let forbidden = body.lines().any(|l| {
            let t = l.trim();
            (t.contains("--local-port") || t.contains("—local-port"))
                && !(t.contains("NEVER")
                    || t.contains("NUNCA")
                    || t.contains("invent")
                    || t.contains("inventar")
                    || t.contains("DOC-042"))
        });
        assert!(
            !forbidden,
            "{label} skill must not document --local-port as a real CLI flag"
        );
    }
}

// --- REL-008 / product line ---

#[test]
fn gap_rel_008_changelog_042() {
    let ch = std::fs::read_to_string(root().join("CHANGELOG.md")).unwrap();
    assert!(
        ch.contains("0.4.2") || ch.contains("[0.4.2]"),
        "CHANGELOG must have 0.4.2 section"
    );
    assert!(
        ch.contains("TUN-003") || ch.contains("local_port") || ch.contains("ephemeral"),
        "CHANGELOG 0.4.2 must mention TUN-003 / ephemeral port"
    );
    assert!(
        ch.contains("IO-010") || ch.contains("No such file") || ch.contains("exit 66"),
        "CHANGELOG 0.4.2 must mention IO-010 / exit 66"
    );
}

#[test]
fn gap_doc_product_line_042() {
    let readme = std::fs::read_to_string(root().join("README.md")).unwrap();
    let llms = std::fs::read_to_string(root().join("llms.txt")).unwrap();
    assert!(
        readme.contains("0.5.0") || readme.contains("0.4.2"),
        "README must mention product line 0.5.0 / 0.4.2 history"
    );
    assert!(
        llms.contains("0.5.0") || llms.contains("0.4.2"),
        "llms.txt must state product line 0.5.0 / 0.4.2 history"
    );
}

#[test]
fn gap_telemetry_false_doctor_source() {
    let src = std::fs::read_to_string(root().join("src/vps/doctor.rs")).unwrap();
    assert!(
        src.contains("\"telemetry\": false") || src.contains("telemetry\": false"),
        "doctor JSON must hardcode telemetry false"
    );
}

#[test]
fn gap_vps_export_schema_exists() {
    let p = root().join("docs/schemas/vps-export.schema.json");
    assert!(p.is_file(), "vps-export.schema.json must exist");
    let s = std::fs::read_to_string(p).unwrap();
    assert!(s.contains("vps-export"));
}
