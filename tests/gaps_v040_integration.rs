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
    // The bare `E10` operand subsumed both cased variants; keep one form.
    assert!(script.contains("E10"));
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
        src.contains("modo_p")
            || src.contains("-tp")
            || src.contains("remote_scp_command")
            || src.contains("mode_p"),
        "remote scp must request -p (OpenSSH source emits T only with -p)"
    );
    assert!(
        src.contains("apply_local_mode")
            || src.contains("aplicar_mode_local")
            || src.contains("set_permissions"),
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
    // Two independent halves of the same contract: the tunnel must *call* the
    // printer and `output` must *define* it. Asserting only one of the two lets a
    // dangling call or a dead printer survive.
    let src = tunnel_subsystem();
    assert!(
        src.contains("print_tunnel_listening_json"),
        "tunnel must emit structured listening JSON"
    );
    let out = output_subsystem();
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

/// Concatenates one subsystem under `src/` into a single searchable string.
///
/// A source assertion pinned to a single path is the wrong shape twice over: it
/// passes vacuously when code moves *in* to that file, and fails spuriously when
/// code moves *out* of it. `gaps_v057_sftp` and `gaps_v061_error_taxonomy` both hit
/// this, and so did the i18n reader below when C3 split the translation tables out.
/// Reading the whole subsystem makes the assertion about the contract, not the layout.
///
/// Test modules are never passed in as leaves: including them would let an assertion
/// be satisfied by the text of the test that makes it, which is exactly the tautology
/// `tests/test_quality.rs` exists to forbid.
fn concat_subsystem(entry: &str, leaves: &[&str]) -> String {
    let base = root().join("src");
    let mut out = std::fs::read_to_string(base.join(entry))
        .unwrap_or_else(|e| panic!("read src/{entry}: {e}"));
    for leaf in leaves {
        out.push('\n');
        out.push_str(
            &std::fs::read_to_string(base.join(leaf))
                .unwrap_or_else(|e| panic!("read src/{leaf}: {e}")),
        );
    }
    out
}

/// Reads the whole i18n subsystem: `en()` / `pt()` live in their own files since C3.
fn i18n_subsystem() -> String {
    concat_subsystem("i18n.rs", &["i18n/en.rs", "i18n/pt.rs"])
}

/// Reads the whole tunnel subsystem: the per-mode listeners live in `src/tunnel/`.
///
/// `tunnel/tests.rs` is deliberately absent — see [`concat_subsystem`].
fn tunnel_subsystem() -> String {
    concat_subsystem(
        "tunnel.rs",
        &[
            "tunnel/local.rs",
            "tunnel/reverse.rs",
            "tunnel/socks.rs",
            "tunnel/streamlocal.rs",
        ],
    )
}

/// Reads the whole output subsystem: the wire printers are split by concern.
fn output_subsystem() -> String {
    concat_subsystem(
        "output/mod.rs",
        &[
            "output/emit.rs",
            "output/json.rs",
            "output/text.rs",
            "output/batch.rs",
        ],
    )
}

#[test]
fn gap_scp_020_i18n_mensagens() {
    let src = i18n_subsystem();
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
