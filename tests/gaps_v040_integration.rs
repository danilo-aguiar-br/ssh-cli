// SPDX-License-Identifier: MIT OR Apache-2.0
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
    if let Some(value) = llvm_profile_file {
        c.env("LLVM_PROFILE_FILE", value);
    }
    c.env("HOME", tmp.path());
    c.env("XDG_CONFIG_HOME", tmp.path());
    c.arg("--config-dir").arg(tmp.path());
    c.arg("--json");
    c.arg("--allow-plaintext-secrets");
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
    // Suite histórica 0.4.0 SCP/IO; product line current is 0.5.x after EN/API rename.
    let v = env!("CARGO_PKG_VERSION");
    assert!(
        v.starts_with("0.5.") || v.starts_with("0.4."),
        "Cargo.toml product line must be 0.5.x (got {v})"
    );
}

#[test]
#[serial]
fn gap_version_cli_contem_040() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
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
        readme.contains("0.5.0") || readme.contains("0.4.0") || readme.contains("0.4.1"),
        "README must mention product line 0.5.x / 0.4.x history"
    );
    let lower = readme.to_lowercase();
    assert!(
        lower.contains("regular file")
            || lower.contains("file-only")
            || lower.contains("files only")
            || lower.contains("not directories"),
        "README must document file-only SCP limit"
    );
    assert!(
        readme.contains("scp-transfer") || readme.contains("tunnel_listening"),
        "README must surface scp-transfer and/or tunnel_listening for agents"
    );
    assert!(
        readme.contains(".ssh-cli.partial") || lower.contains("partial"),
        "README must document partial download path"
    );
}

#[test]
fn gap_doc_004_root_security_integrations_honest() {
    let sec = std::fs::read_to_string(root().join("SECURITY.md")).expect("SECURITY");
    assert!(
        (sec.contains("0.5.x") || sec.contains("0.5.0") || sec.contains("0.4.x"))
            && (sec.contains("current line") || sec.contains("current") || sec.contains("atual")),
        "SECURITY Supported Versions must brand current product line"
    );
    assert!(
        !sec.contains("| 0.3.x | Supported | Yes, current line |"),
        "SECURITY must not claim 0.3.x is the current product line"
    );
    let integ = std::fs::read_to_string(root().join("INTEGRATIONS.md")).expect("INTEGRATIONS");
    assert!(
        (integ.contains("0.5.0") || integ.contains("0.4.0") || integ.contains("0.4.1"))
            && (integ.contains("scp-transfer") || integ.contains("tunnel_listening")),
        "INTEGRATIONS 0.4.x must document real SCP/tunnel surface"
    );
    assert!(
        integ.contains("0.3.9"),
        "INTEGRATIONS must keep 0.3.9 residual facts under their own version bullet"
    );
    let llms_full = std::fs::read_to_string(root().join("llms-full.txt")).expect("llms-full");
    assert!(
        llms_full.contains("scp-transfer.schema.json"),
        "llms-full must index scp-transfer schema"
    );
    assert!(
        llms_full.contains("tunnel-listening.schema.json"),
        "llms-full must index tunnel-listening schema"
    );
    let contrib = std::fs::read_to_string(root().join("CONTRIBUTING.md")).expect("CONTRIBUTING");
    assert!(
        contrib.contains("gaps_v040"),
        "CONTRIBUTING must mention gaps_v040 regression suite"
    );
    assert!(
        contrib.contains("E10") || contrib.contains("E01–E14") || contrib.contains("E01-E14"),
        "CONTRIBUTING must mention official e2e SCP matrix E10+"
    );
}

#[test]
fn gap_doc_004c_docs_folder_scp_tunnel_honest() {
    let agents = std::fs::read_to_string(root().join("docs/AGENTS.md")).expect("AGENTS");
    assert!(
        agents.contains("scp-transfer") && agents.contains("tunnel_listening"),
        "docs/AGENTS.md must document scp-transfer and tunnel_listening contracts"
    );
    assert!(
        agents.to_lowercase().contains("regular files only")
            || agents.contains("file-only")
            || agents.contains("no directories"),
        "docs/AGENTS.md must document SCP file-only"
    );
    let howto = std::fs::read_to_string(root().join("docs/HOW_TO_USE.md")).expect("HOW_TO_USE");
    assert!(
        howto.contains("0.3.9") && howto.contains(".ssh-cli.partial"),
        "docs/HOW_TO_USE.md must warn 0.3.9 and document partial downloads"
    );
    let cook = std::fs::read_to_string(root().join("docs/COOKBOOK.md")).expect("COOKBOOK");
    assert!(
        cook.contains("tunnel_listening") && cook.contains("scp-transfer"),
        "docs/COOKBOOK.md must include tunnel_listening and scp-transfer recipes"
    );
    let mig = std::fs::read_to_string(root().join("docs/MIGRATION.md")).expect("MIGRATION");
    assert!(
        mig.contains("tunnel_listening")
            && mig.contains(".ssh-cli.partial")
            && mig.contains("32 KiB"),
        "docs/MIGRATION.md 0.4.0 section must cover tunnel JSON, partial, stream"
    );
    let testing = std::fs::read_to_string(root().join("docs/TESTING.md")).expect("TESTING");
    assert!(
        testing.contains("gaps_v040")
            && (testing.contains("E10")
                || testing.contains("E01–E14")
                || testing.contains("E01-E14")),
        "docs/TESTING.md must list gaps_v040 and e2e E10+"
    );
    let release =
        std::fs::read_to_string(root().join("docs/RELEASE_CHECKLIST.md")).expect("RELEASE");
    assert!(
        release.contains("gaps_v040") && release.contains("DOC-004"),
        "docs/RELEASE_CHECKLIST.md must gate gaps_v040 and DOC-004"
    );
    let cross = std::fs::read_to_string(root().join("docs/CROSS_PLATFORM.md")).expect("CROSS");
    let cross_l = cross.to_lowercase();
    assert!(
        cross.contains(".ssh-cli.partial")
            && (cross_l.contains("regular files only")
                || cross.contains("file-only")
                || cross_l.contains("regular files")),
        "docs/CROSS_PLATFORM.md must document SCP portability"
    );
    let schema_idx =
        std::fs::read_to_string(root().join("docs/schemas/README.md")).expect("schemas README");
    assert!(
        schema_idx.contains("scp-transfer.schema.json")
            && schema_idx.contains("tunnel-listening.schema.json"),
        "docs/schemas/README.md must index scp-transfer and tunnel-listening"
    );
    assert!(
        root()
            .join("docs/schemas/tunnel-listening.schema.json")
            .is_file(),
        "missing tunnel-listening.schema.json"
    );
}

#[test]
fn gap_doc_004d_skills_scp_tunnel_honest() {
    // Skills must teach agents the 0.4.0 scp/tunnel contracts without version-story prose.
    for rel in ["skills/ssh-cli-en/SKILL.md", "skills/ssh-cli-pt/SKILL.md"] {
        let body = std::fs::read_to_string(root().join(rel)).expect(rel);
        let lower = body.to_ascii_lowercase();
        assert!(
            body.contains("tunnel_listening"),
            "{rel} must document tunnel_listening ready event"
        );
        assert!(
            body.contains(".ssh-cli.partial"),
            "{rel} must document partial download path"
        );
        assert!(
            body.contains("32 KiB") || body.contains("32KiB"),
            "{rel} must document 32 KiB upload stream"
        );
        assert!(
            lower.contains("files-only")
                || lower.contains("file-only")
                || lower.contains("regular-file")
                || lower.contains("regular file")
                || body.contains("somente-arquivo")
                || body.contains("só-arquivo")
                || body.contains("arquivo regular"),
            "{rel} must document scp regular-files-only"
        );
        assert!(
            body.contains("ok")
                && body.contains("direction")
                && body.contains("bytes")
                && body.contains("duration_ms"),
            "{rel} must document scp-transfer success fields"
        );
        assert!(
            body.contains("local_port")
                && body.contains("remote_host")
                && body.contains("remote_port")
                && body.contains("timeout_ms"),
            "{rel} must document tunnel_listening fields"
        );
        assert!(
            body.contains("scp upload")
                && body.contains("--json")
                && body.contains("tunnel")
                && body.contains("--timeout-ms"),
            "{rel} must include scp --json and tunnel --timeout-ms formulas"
        );
        assert!(
            !body.contains("0.4.0 did")
                && !body.contains("0.3.9 did")
                && !body.contains("in version 0.3.9")
                && !body.contains("versão 0.3.9")
                && !body.contains("na versão 0.3.9"),
            "{rel} must stay consolidated without version-story prose"
        );
        let fm = body
            .strip_prefix("---\n")
            .and_then(|s| s.split_once("\n---"))
            .map(|(a, _)| a)
            .expect("frontmatter");
        let desc = fm
            .lines()
            .find(|l| l.starts_with("description:"))
            .expect("description")
            .trim_start_matches("description:")
            .trim();
        assert!(
            desc.chars().count() < 1024,
            "{rel} description must be < 1024 chars (got {})",
            desc.chars().count()
        );
        assert_eq!(
            desc.matches(':').count(),
            0,
            "{rel} description must not contain ':' in content"
        );
        assert!(
            desc.contains("tunnel_listening")
                && (desc.contains("files-only")
                    || desc.contains("só-arquivo")
                    || desc.contains("file-only")
                    || desc.contains("regular")),
            "{rel} description must surface scp file-only + tunnel_listening for auto-activation"
        );
    }
    for rel in [
        "skills/ssh-cli-en/evals/queries.json",
        "skills/ssh-cli-pt/evals/queries.json",
    ] {
        let q = std::fs::read_to_string(root().join(rel)).expect(rel);
        assert!(
            q.contains("tunnel_listening")
                && (q.contains(".ssh-cli.partial") || q.contains("ssh-cli.partial"))
                && (q.contains("files only")
                    || q.contains("regular files")
                    || q.contains("arquivos regulares")
                    || q.contains("somente arquivo")
                    || q.contains("directory")
                    || q.contains("diretorio")),
            "{rel} evals must cover tunnel_listening + partial + file-only surface"
        );
    }
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
fn gap_io_008_tunnel_schema_listening() {
    let schema = root().join("docs/schemas/tunnel-listening.schema.json");
    assert!(schema.is_file(), "missing {}", schema.display());
    let body = std::fs::read_to_string(&schema).unwrap();
    assert!(body.contains("tunnel_listening"));
    assert!(body.contains("local_port"));
    assert!(body.contains("timeout_ms"));
}

#[test]
fn gap_scp_021_schema_scp_transfer() {
    let schema = root().join("docs/schemas/scp-transfer.schema.json");
    assert!(schema.is_file(), "missing {}", schema.display());
    let body = std::fs::read_to_string(&schema).unwrap();
    assert!(body.contains("direction"));
    assert!(body.contains("bytes"));
    assert!(body.contains("duration_ms"));
    // GAP-SSH-IO-009 (0.4.1): event discriminator required.
    assert!(
        body.contains("scp-transfer") && body.contains("\"event\""),
        "scp-transfer schema must require event field (IO-009)"
    );
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
    let src = std::fs::read_to_string(root().join("src/ssh/client_real.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/client_real_scp.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/client_real_core.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/client_real_tests_body.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/scp_wire.rs")).unwrap();
    assert!(
        src.contains("modo_p") || src.contains("-tp") || src.contains("remote_scp_command") || src.contains("mode_p"),
        "remote scp must request -p (OpenSSH source emits T only with -p)"
    );
    assert!(
        src.contains("apply_local_mode") || src.contains("aplicar_mode_local") || src.contains("set_permissions"),
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
        src.contains("print_tunnel_listening_json") || src.contains("tunnel_listening"),
        "tunnel must emit structured listening JSON"
    );
    let out = std::fs::read_to_string(root().join("src/output/mod.rs")).unwrap();
    assert!(
        out.contains("print_tunnel_listening_json"),
        "output must define tunnel listening JSON printer"
    );
}

#[test]
fn gap_scp_022_partial_suffix_na_fonte() {
    let src = std::fs::read_to_string(root().join("src/ssh/client_real.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/client_real_scp.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/client_real_core.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/scp_wire.rs")).unwrap();
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
        src.contains("apply_local_mode(&partial") || src.contains("aplicar_mode_local(&partial"),
        "mode must be applied on partial before rename"
    );
}

#[test]
fn gap_scp_020_i18n_mensagens() {
    let src = std::fs::read_to_string(root().join("src/i18n.rs")).unwrap();
    assert!(src.contains("ScpUploadCompleted"));
    assert!(src.contains("ScpDownloadCompleted"));
    assert!(src.contains("ScpUploadFileOnly"));
    assert!(src.contains("ScpDownloadLocalNotDirectory"));
    assert!(src.contains("Upload completed"));
    assert!(src.contains("Upload concluído") || src.contains("Upload concluido"));
}

/// IO-007b: `scp --json` local promove envelope de err JSON (paridade tunnel).
#[test]
#[serial]
fn gap_io_007b_scp_json_local_error_envelope() {
    let tmp = TempDir::new().unwrap();
    add_host(&tmp, "jsonscp");
    cmd(&tmp)
        .args([
            "scp",
            "upload",
            "jsonscp",
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
    let src = std::fs::read_to_string(root().join("src/ssh/client_real.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/client_real_scp.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/client_real_core.rs")).unwrap()
        + &std::fs::read_to_string(root().join("src/ssh/scp_wire.rs")).unwrap();
    assert!(src.contains("format_scp_upload_header") || src.contains("formatar_header_upload_scp"));
    assert!(src.contains("format_scp_t_line") || src.contains("formatar_linha_t_scp"));
    assert!(src.contains("SCP_OK"));
}
