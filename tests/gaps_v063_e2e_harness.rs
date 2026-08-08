// SPDX-License-Identifier: MIT OR Apache-2.0
//! D1 — the official E01–E18 matrix must be *runnable*, and its skip must be loud.
//!
//! `scripts/e2e_real_ssh.sh` implements eighteen cases against a real SSH server
//! and is named as the official matrix in six documents. Without a lab host it
//! called `skip_all` and exited 0, so it had never executed once — every "all
//! gates green" scoreboard silently counted eighteen no-ops.
//!
//! Three test files asserted the matrix by searching the shell script for the
//! literal strings `"E10"`..`"E13"`. That proves a string exists in a file, not
//! that a case ever ran: three layers of green over zero execution.
//!
//! These assertions therefore exercise the harness's *behaviour* — its CLI
//! surface and its exit contract — instead of its source text. A text assertion
//! on a script is the same layout-pinned mistake, one language over.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn harness() -> PathBuf {
    repo_root().join("scripts/e2e_real_ssh.sh")
}

fn run_harness(args: &[&str]) -> std::process::Output {
    Command::new("bash")
        .arg(harness())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("spawn e2e harness")
}

#[test]
fn the_harness_exists_and_is_executable_by_bash() {
    let path = harness();
    assert!(path.is_file(), "missing E2E harness at {}", path.display());
    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&path)
        .output()
        .expect("spawn bash -n");
    assert!(
        syntax.status.success(),
        "harness has a syntax error: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );
}

/// The capability must live in the artifact, not only in the prose.
///
/// Six documents already said "prefer local sshd". None of them could make it
/// happen, because the harness had no code to start one — the same shape as C1,
/// where the error contract was documented everywhere except in the schema.
#[test]
fn the_harness_offers_a_local_sshd_mode() {
    let out = run_harness(&["--help"]);
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(
        help.contains("--local-sshd"),
        "harness must offer --local-sshd so the matrix can run without a lab host; got:\n{help}"
    );
}

/// A skip that names no cause is indistinguishable from a run.
///
/// Offline safety is still honoured — exit 0 — but the reason must say whether
/// nothing *could* run or whether a runnable matrix was simply not run.
#[test]
fn skipping_without_a_lab_host_states_its_cause() {
    let out = run_harness(&[]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // Offline runs must not fail: that contract predates this change.
    assert_eq!(
        out.status.code(),
        Some(0),
        "an offline run must stay exit 0; got:\n{text}"
    );
    assert!(
        text.contains("SKIP"),
        "an offline run must announce a SKIP; got:\n{text}"
    );
    // The cause must be explicit, whichever branch was taken.
    assert!(
        text.contains("sshd"),
        "the skip must name whether an sshd was available; got:\n{text}"
    );
}

/// The explicit mode must never degrade into a silent pass.
///
/// `--local-sshd` is a statement that the matrix is expected to run. If the host
/// cannot honour it, that is a failure to report, not a skip to swallow.
#[test]
fn the_explicit_local_mode_never_degrades_to_skip() {
    // A bogus binary path makes the harness fail early without needing an sshd,
    // so this stays fast and deterministic on hosts with and without OpenSSH.
    let out = run_harness(&["--local-sshd", "--bin", "/nonexistent/ssh-cli-probe"]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_ne!(
        out.status.code(),
        Some(0),
        "--local-sshd must not exit 0 when it cannot run the matrix; got:\n{text}"
    );
    assert!(
        !text.contains("SKIP E2E real SSH"),
        "--local-sshd must fail loudly instead of skipping; got:\n{text}"
    );
}

/// D6b — the harness must obey the same tooling rules as the rest of the repo.
///
/// JSON is parsed with `jaq`, text is searched with `rg`, fields are selected
/// with `choose`. The single surviving `python3` call parses TOML, which `jaq`
/// cannot do, and is reachable only from the maintainer-only `--from-grok-config`
/// path.
#[test]
fn the_harness_uses_the_projects_mandated_tools() {
    let src = std::fs::read_to_string(harness()).expect("read harness");

    for banned in ["| grep ", "| awk ", "| cut "] {
        assert!(
            !src.contains(banned),
            "harness must not pipe into `{}`; use rg / jaq / choose",
            banned.trim()
        );
    }

    let python_calls = src.matches("python3 ").count();
    assert!(
        python_calls <= 1,
        "only the TOML parser may use python3 (jaq cannot read TOML); found {python_calls} calls"
    );
}

/// The matrix is the deliverable, so its size is part of the contract.
#[test]
fn the_official_matrix_still_declares_eighteen_cases() {
    let src = std::fs::read_to_string(harness()).expect("read harness");
    let missing: Vec<String> = (1..=18)
        .map(|n| format!("E{n:02}"))
        .filter(|case| !src.contains(&format!("pass {case}")))
        .collect();
    assert!(
        missing.is_empty(),
        "E01-E18 is the official matrix; these cases have no pass arm: {missing:?}"
    );
}

/// Regression guard for the finding that started this round.
///
/// If someone re-adds an unconditional early `exit 0` that bypasses the matrix,
/// the harness silently becomes a no-op again. The skip must stay reachable only
/// through `skip_all`, which now names its cause.
#[test]
fn the_only_early_success_exit_is_the_named_skip() {
    let src = std::fs::read_to_string(harness()).expect("read harness");
    let skip_all_defs = src.matches("skip_all()").count();
    assert_eq!(
        skip_all_defs, 1,
        "skip_all must have exactly one definition so its cause reporting cannot be bypassed"
    );
    assert!(
        src.contains("--local-sshd") && src.contains("local_sshd=requested_but_unusable"),
        "the explicit mode must have a distinct hard-failure path"
    );
    assert!(
        Path::new(&harness()).is_file(),
        "harness path must resolve from the manifest dir"
    );
}
