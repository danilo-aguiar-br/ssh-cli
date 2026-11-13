// SPDX-License-Identifier: MIT OR Apache-2.0
//! `vps edit` reports only what changed, and an edit that changes nothing is a no-op.
//!
//! # Why every case drives argv
//!
//! The promises live at the process boundary: which fields the `updated` array
//! lists, whether `config.toml` was rewritten, and which exit code a rename onto
//! a missing or identical name returns. Every case spawns the binary with a fresh
//! `--config-dir`, so none of them can pass by exercising a helper the CLI does
//! not use.
//!
//! # What the cases pin
//!
//! - Naming a flag whose value already matches the record leaves `updated` empty.
//! - A no-op exits 0 and leaves `config.toml` byte-identical (no write, no fsync).
//! - `updated` never disappears from the envelope, even when empty.
//! - Error precedence in a rename is: unknown host (66), identical name (64),
//!   taken name (64), and the identical-name error carries the resolved target.

use assert_cmd::Command;
use serial_test::serial;
use tempfile::TempDir;

/// Exit code for a usage error (`sysexits.h` `EX_USAGE`).
const EX_USAGE: i32 = 64;
/// Exit code for "named thing does not exist" (`sysexits.h` `EX_NOINPUT`).
const EX_NOINPUT: i32 = 66;

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
    c.arg("--allow-plaintext-secrets");
    c
}

/// Runs argv and returns stdout as text, asserting success.
fn ok(tmp: &TempDir, args: &[&str]) -> String {
    let out = cmd(tmp).args(args).output().expect("spawn ssh-cli");
    assert!(
        out.status.success(),
        "argv {args:?} failed with {:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Runs argv, returning exit code plus both streams.
fn run(tmp: &TempDir, args: &[&str]) -> (i32, String, String) {
    let out = cmd(tmp).args(args).output().expect("spawn ssh-cli");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Registers one password-authenticated host on the default port (22).
fn seed(tmp: &TempDir, name: &str) {
    cmd(tmp)
        .args([
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--password-stdin",
        ])
        .write_stdin("secret")
        .assert()
        .success();
}

/// Parses the single compact JSON object a `--json` success run writes to stdout.
fn envelope_of(stdout: &str) -> serde_json::Value {
    let line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| panic!("no JSON envelope on stdout:\n{stdout}"));
    serde_json::from_str(line).unwrap_or_else(|e| panic!("envelope is not JSON: {e}\n{line}"))
}

/// Runs `vps edit <name> <args> --json` and returns its envelope.
fn edit(tmp: &TempDir, name: &str, args: &[&str]) -> serde_json::Value {
    let mut argv = vec!["vps", "edit", name];
    argv.extend_from_slice(args);
    argv.push("--json");
    envelope_of(&ok(tmp, &argv))
}

fn config_bytes(tmp: &TempDir) -> Vec<u8> {
    std::fs::read(tmp.path().join("config.toml")).expect("config.toml")
}

#[test]
#[serial]
fn same_port_is_not_reported() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let v = edit(&tmp, "h", &["--port", "22"]);
    assert_eq!(v["event"], "vps-edited");
    assert_eq!(v["updated"], serde_json::json!([]));
}

#[test]
#[serial]
fn a_different_port_is_reported() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let v = edit(&tmp, "h", &["--port", "2222"]);
    assert_eq!(v["updated"], serde_json::json!(["port"]));
}

#[test]
#[serial]
fn clear_tags_on_an_empty_list_is_not_reported() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let v = edit(&tmp, "h", &["--clear-tags"]);
    assert_eq!(v["updated"], serde_json::json!([]));
}

#[test]
#[serial]
fn clear_tags_with_a_new_tag_reports_tags_once() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    edit(&tmp, "h", &["--tag", "old"]);
    let v = edit(&tmp, "h", &["--clear-tags", "--tag", "z"]);
    assert_eq!(v["updated"], serde_json::json!(["tags"]));
}

#[test]
#[serial]
fn clear_sudo_password_without_one_is_not_reported() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let v = edit(&tmp, "h", &["--clear-sudo-password"]);
    assert_eq!(v["updated"], serde_json::json!([]));
}

#[test]
#[serial]
fn no_key_without_a_key_is_not_reported() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let v = edit(&tmp, "h", &["--no-key"]);
    assert_eq!(v["updated"], serde_json::json!([]));
}

#[test]
#[serial]
fn a_no_op_edit_leaves_the_config_byte_identical() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let before = config_bytes(&tmp);
    let v = edit(&tmp, "h", &["--port", "22", "--clear-tags", "--no-key"]);
    assert_eq!(v["event"], "vps-edited");
    assert_eq!(v["updated"], serde_json::json!([]));
    assert_eq!(
        before,
        config_bytes(&tmp),
        "a no-op must not rewrite config.toml"
    );
}

#[test]
#[serial]
fn a_real_edit_still_writes_the_config() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let before = config_bytes(&tmp);
    let v = edit(&tmp, "h", &["--timeout", "9000"]);
    assert_eq!(v["updated"], serde_json::json!(["timeout_ms"]));
    assert_ne!(before, config_bytes(&tmp));
}

#[test]
#[serial]
fn updated_is_present_in_every_envelope() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let noop = edit(&tmp, "h", &["--port", "22"]);
    let changed = edit(&tmp, "h", &["--timeout", "9000"]);
    let renamed = envelope_of(&ok(&tmp, &["vps", "rename", "h", "k", "--json"]));
    for (label, v) in [("noop", noop), ("changed", changed), ("renamed", renamed)] {
        assert!(
            v.get("updated").is_some_and(serde_json::Value::is_array),
            "{label}: `updated` must always be an array: {v}"
        );
    }
}

#[test]
#[serial]
fn renaming_a_missing_host_onto_itself_is_not_found() {
    let tmp = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = run(&tmp, &["vps", "rename", "ghost", "ghost", "--json"]);
    assert_eq!(code, EX_NOINPUT, "{stderr}");
    assert!(stdout.trim().is_empty(), "{stdout}");
    assert!(stderr.contains("vps_not_found"), "{stderr}");
}

#[test]
#[serial]
fn renaming_onto_a_taken_name_is_a_usage_error() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "a");
    seed(&tmp, "b");
    let (code, _, stderr) = run(&tmp, &["vps", "rename", "a", "b", "--json"]);
    assert_eq!(code, EX_USAGE, "{stderr}");
    assert!(stderr.contains("vps_duplicate"), "{stderr}");
}

#[test]
#[serial]
fn renaming_an_existing_host_onto_itself_names_the_target() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let before = config_bytes(&tmp);
    let (code, _, stderr) = run(&tmp, &["vps", "rename", "h", "h", "--json"]);
    assert_eq!(code, EX_USAGE, "{stderr}");
    assert!(
        stderr.contains("target_resolved"),
        "the identical-name error must carry the resolved target: {stderr}"
    );
    assert_eq!(before, config_bytes(&tmp));
}
