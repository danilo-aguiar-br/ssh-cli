// SPDX-License-Identifier: MIT OR Apache-2.0
//! Behavioural contracts for the 0.5.4 agent-native surface (C1/C2).
//!
//! # Why this suite exists
//!
//! `--no-input` shipped with **zero** test coverage and, as a direct result, shipped
//! broken: the refusal was implemented in `cli::read_stdin_if`, which only the
//! exec/scp/tunnel *override* path uses. `vps add` and `vps edit` call
//! `vps::read_secret_stdin` directly, so the flag was accepted and silently ignored on
//! the two commands most likely to run unattended — an agent asking for a declarative
//! refusal got a password read instead.
//!
//! The payload-shaping flags had the same exposure: they are installed once in
//! `print_json_line` and are easy to leave inert for any envelope shape that the
//! implementation did not anticipate (a bare JSON array at the root already caused
//! exactly that during development). Asserting on the real binary is the only honest
//! proof that the reduction actually happens.

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
    c.arg("--allow-plaintext-secrets");
    c
}

/// Registers two hosts so listing has something to shape.
fn seed(tmp: &TempDir) {
    for (name, host, tag) in [("h1", "1.1.1.1", "prod"), ("h2", "2.2.2.2", "dev")] {
        cmd(tmp)
            .args([
                "vps",
                "add",
                "--name",
                name,
                "--host",
                host,
                "--user",
                "u",
                "--password-stdin",
                "--tag",
                tag,
            ])
            .write_stdin("secret")
            .assert()
            .success();
    }
}

#[test]
#[serial]
fn no_input_refuses_stdin_on_vps_add() {
    let tmp = TempDir::new().unwrap();
    // Without the guard in `read_secret_stdin` this succeeds and creates the host.
    cmd(&tmp)
        .args([
            "--no-input",
            "vps",
            "add",
            "--name",
            "h9",
            "--host",
            "9.9.9.9",
            "--user",
            "u",
            "--password-stdin",
        ])
        .write_stdin("secret")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--no-input"));
}

#[test]
#[serial]
fn no_input_refuses_stdin_on_vps_edit() {
    let tmp = TempDir::new().unwrap();
    seed(&tmp);
    cmd(&tmp)
        .args(["--no-input", "vps", "edit", "h1", "--password-stdin"])
        .write_stdin("secret")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--no-input"));
}

#[test]
#[serial]
fn without_no_input_stdin_still_works() {
    // Guards that the refusal is conditional, not a blanket break of `--password-stdin`.
    let tmp = TempDir::new().unwrap();
    seed(&tmp);
    cmd(&tmp)
        .args(["vps", "list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("h1"));
}

#[test]
#[serial]
fn select_and_limit_shrink_the_envelope() {
    let tmp = TempDir::new().unwrap();
    seed(&tmp);
    let full = cmd(&tmp)
        .args(["vps", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let shaped = cmd(&tmp)
        .args(["--select", "name", "--limit", "1", "vps", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(
        shaped.len() < full.len(),
        "shaping must reduce the payload: full={} shaped={}",
        full.len(),
        shaped.len()
    );
    let text = String::from_utf8(shaped).unwrap();
    assert!(text.contains("h1"), "kept element must survive: {text}");
    assert!(
        !text.contains("2.2.2.2"),
        "--limit 1 must drop the second host: {text}"
    );
    assert!(
        !text.contains("\"host\""),
        "--select name must drop other keys: {text}"
    );
}

#[test]
#[serial]
fn filter_selects_by_field_value() {
    let tmp = TempDir::new().unwrap();
    seed(&tmp);
    let out = cmd(&tmp)
        .args([
            "--select", "name", "--filter", "name=h2", "vps", "list", "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("h2"), "filter must keep the match: {text}");
    assert!(!text.contains("h1"), "filter must drop non-matches: {text}");
}

#[test]
#[serial]
fn count_only_replaces_the_payload_with_a_count() {
    let tmp = TempDir::new().unwrap();
    seed(&tmp);
    cmd(&tmp)
        .args(["--count-only", "vps", "list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"count\":2"));
}

#[test]
#[serial]
fn non_loopback_bind_requires_explicit_acceptance() {
    // G-TUN-R13: the refusal must land before any network I/O, so no host needs to
    // be reachable for this to be a real contract test.
    let tmp = TempDir::new().unwrap();
    seed(&tmp);
    cmd(&tmp)
        .args([
            "tunnel",
            "h1",
            "18080",
            "127.0.0.1",
            "80",
            "--timeout-ms",
            "500",
            "--bind",
            "0.0.0.0",
            "--json",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--i-accept-network-exposure"));
}
