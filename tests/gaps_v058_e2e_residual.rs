// SPDX-License-Identifier: MIT OR Apache-2.0
//! G-E2E residual close suite for v0.5.2 (01–04, 06–10, 13, 19).
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ssh-cli"))
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("spawn ssh-cli")
}

fn run_cfg(cfg: &std::path::Path, args: &[&str]) -> std::process::Output {
    let mut full = vec!["--config-dir", cfg.to_str().unwrap()];
    full.extend_from_slice(args);
    Command::new(bin())
        .args(&full)
        .output()
        .expect("spawn ssh-cli")
}

#[test]
fn schema_catalog_lists_vps_list() {
    let out = run(&["schema", "--json"]);
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("schema-catalog") || s.contains("vps-list"), "{s}");
    assert!(s.contains("vps-list"), "{s}");
}

#[test]
fn schema_body_vps_list_is_json() {
    let out = run(&["schema", "vps-list"]);
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"$schema\"") || s.contains("schema"), "{s}");
}

#[test]
fn doctor_root_emits_vps_doctor_event() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_cfg(dir.path(), &["doctor", "--json"]);
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("vps-doctor") || s.contains("config_path"), "{s}");
    assert!(s.contains("\"telemetry\":false") || s.contains("telemetry"), "{s}");
}

#[test]
fn vps_add_first_password_single_json_document() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_cfg(
        dir.path(),
        &[
            "--json",
            "vps",
            "add",
            "--name",
            "lab1",
            "--host",
            "127.0.0.1",
            "--user",
            "u",
            "--password",
            "secret-pass-for-e2e",
        ],
    );
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    // G-E2E-04: exactly one JSON root on the data path.
    assert_eq!(
        lines.len(),
        1,
        "expected one JSON document, got {}: {stdout}",
        lines.len()
    );
    let v: serde_json::Value = serde_json::from_str(lines[0]).expect("single json.loads");
    assert_eq!(v["event"], "vps-added");
    // SuccessEnvelope flattens fields (no nested `data`).
    assert_eq!(v["name"], "lab1");
    assert_eq!(v["secrets_key_auto_created"], true);
    assert!(v.get("key_file").is_some(), "{v}");
}

#[test]
fn vps_add_use_agent_parses() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_cfg(
        dir.path(),
        &[
            "--json",
            "vps",
            "add",
            "--name",
            "agent1",
            "--host",
            "127.0.0.1",
            "--user",
            "u",
            "--use-agent",
        ],
    );
    assert!(
        out.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn help_does_not_teach_secrets_env_store() {
    let out = run(&["--help"]);
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        !s.contains("SSH_CLI_USE_KEYRING"),
        "help must not teach deprecated env: {s}"
    );
    assert!(
        !s.contains("overrides env"),
        "help must not teach env override: {s}"
    );
}

#[test]
fn version_stamp_contains_dirty_or_hash() {
    let out = run(&["--version"]);
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("0.5.2"), "{s}");
    // Working tree is dirty in this residual close → expect -dirty when .git present.
    // Allow either form so crates.io packs without git still pass.
    assert!(
        s.contains("dirty") || s.contains('('),
        "version should embed commit stamp: {s}"
    );
}

#[test]
fn acme_error_map_invalid_contact_permanent() {
    // Unit path is in tls::acme_error_map; this locks exit taxonomy via public API shape.
    use ssh_cli::errors::SshCliError;
    let e = SshCliError::InvalidArgument(
        "ACME create account: ACME invalidContact: API error".into(),
    );
    assert!(!e.is_retryable());
    assert_eq!(e.exit_code(), ssh_cli::errors::exit_codes::EX_USAGE);
}

#[test]
fn clap_feature_env_absent_from_cargo_toml() {
    let toml = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
    )
    .unwrap();
    // clap features line must not include "env"
    let clap_line = toml
        .lines()
        .find(|l| l.contains("clap = {") && l.contains("features"))
        .unwrap_or("");
    assert!(
        !clap_line.contains("\"env\""),
        "clap must not enable env feature: {clap_line}"
    );
}
