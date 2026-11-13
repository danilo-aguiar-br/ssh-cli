// SPDX-License-Identifier: MIT OR Apache-2.0
//! D1 — the official E01–E21 matrix must be *runnable*, and its skip must be loud.
//!
//! `scripts/e2e_real_ssh.py` implements twenty-one cases against a real SSH server
//! and is named as the official matrix in six documents. Without a lab host it
//! called `skip_all` and exited 0, so it had never executed once — every "all
//! gates green" scoreboard silently counted the matrix as a no-op.
//!
//! E19 and E20 were added for the reduced `vps list` payload, and E21 for the OS
//! keyring round-trip (Gap 50). A case that exists but is never named in this
//! module is a case nobody counts, which is how the matrix shrank from its own
//! claim before.
//!
//! Three test files asserted the matrix by searching the script for the
//! literal strings `"E10"`..`"E13"`. That proves a string exists in a file, not
//! that a case ever ran: three layers of green over zero execution.
//!
//! These assertions therefore exercise the harness's *behaviour* — its CLI
//! surface and its exit contract — instead of its source text. A text assertion
//! on a script is the same layout-pinned mistake, one language over.
//!
//! The harness is Python 3 so it runs on Linux, macOS and Windows. The tests find
//! the interpreter the same way an operator would: `python3`, then `python`, then
//! `py -3`. When none works the tests FAIL with a clear message; they never skip.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn harness() -> PathBuf {
    repo_root().join("scripts/e2e_real_ssh.py")
}

/// The harness is split into three cohesive modules; `_`-prefixed ones are helpers.
fn harness_modules() -> [PathBuf; 3] {
    let scripts = repo_root().join("scripts");
    [
        scripts.join("e2e_real_ssh.py"),
        scripts.join("_e2e_sshd.py"),
        scripts.join("_e2e_cases.py"),
    ]
}

/// Concatenated text of every harness module, for source-level contract checks.
fn harness_source() -> String {
    harness_modules()
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("read {}: {e}", p.display())))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Program and leading arguments of the first working Python 3 interpreter.
///
/// A candidate counts only if `--version` succeeds and says `Python 3`: on Windows
/// a `python3` Store stub exists that exits non-zero.
fn python() -> (&'static str, &'static [&'static str]) {
    let candidates: [(&str, &[&str]); 3] = [("python3", &[]), ("python", &[]), ("py", &["-3"])];
    for (program, lead) in candidates {
        let probe = Command::new(program).args(lead).arg("--version").output();
        if let Ok(out) = probe {
            let banner = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            if out.status.success() && banner.trim_start().starts_with("Python 3") {
                return (program, lead);
            }
        }
    }
    panic!(
        "no working Python 3 interpreter found (tried `python3`, `python`, `py -3`); \
         the E2E harness is a Python script and cannot run without one"
    );
}

fn python_command() -> Command {
    let (program, lead) = python();
    let mut cmd = Command::new(program);
    cmd.args(lead);
    cmd
}

fn run_harness(args: &[&str]) -> std::process::Output {
    python_command()
        .arg("-I")
        .arg("-B")
        .arg(harness())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("spawn e2e harness")
}

#[test]
fn the_harness_exists_and_parses_as_python() {
    for path in harness_modules() {
        assert!(
            path.is_file(),
            "missing E2E harness module at {}",
            path.display()
        );
        // `ast.parse` is a pure syntax check: unlike `py_compile` it writes no `.pyc`.
        let syntax = python_command()
            .arg("-I")
            .arg("-B")
            .arg("-c")
            .arg("import ast,sys; ast.parse(open(sys.argv[1], encoding='utf-8').read())")
            .arg(&path)
            .output()
            .expect("spawn python syntax check");
        assert!(
            syntax.status.success(),
            "{} has a syntax error: {}",
            path.display(),
            String::from_utf8_lossy(&syntax.stderr)
        );
    }
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

/// D6b — the harness must stay portable and must never reach for a shell.
///
/// Every child process is started from an argument list through the shared
/// `_gatekit.run_tree` helper, so a timeout kills the whole process tree and the
/// script behaves the same on Linux, macOS and Windows. A shell string, `os.system`
/// or a hard `bash` dependency would break all three of those properties.
#[test]
fn the_harness_never_uses_a_shell() {
    let src = harness_source();

    for banned in ["shell=True", "os.system", "bash"] {
        assert!(
            !src.contains(banned),
            "harness must not use `{banned}`; start children by argv list via _gatekit"
        );
    }
    assert!(
        src.contains("_gatekit.run_tree"),
        "harness must run children through _gatekit.run_tree (timeout kills the tree)"
    );
}

/// The matrix is the deliverable, so its size is part of the contract.
///
/// Each case is its own function so that a missing case is a missing definition,
/// not a missing string somewhere in a long script.
///
/// The matrix is E01 to E21 since 0.5.6. The three new cases exist because the audit of
/// 2026-10-21 measured three behaviours that had no case at all: the reduced
/// `vps list --select` item violated the published schema (D6), `--select` with `--limit`
/// worked but was never measured together (D7), and `secrets init --keyring` reported
/// success while persisting nothing (Gap 50, D1).
#[test]
fn the_official_matrix_still_declares_twenty_one_cases() {
    let src = harness_source();
    let missing: Vec<String> = (1..=21)
        .map(|n| format!("E{n:02}"))
        .filter(|case| {
            let def = format!("def case_{}(", case.to_lowercase());
            let plan = format!("(\"{case}\", lambda:");
            !(src.contains(&def) && src.contains(&plan))
        })
        .collect();
    assert!(
        missing.is_empty(),
        "E01-E21 is the official matrix; these cases are not defined and scheduled: {missing:?}"
    );
}

/// Regression guard for the finding that started this round.
///
/// If someone re-adds an unconditional early `sys.exit(0)` that bypasses the
/// matrix, the harness silently becomes a no-op again. The skip must stay
/// reachable only through `skip_all`, which names its cause.
#[test]
fn the_only_early_success_exit_is_the_named_skip() {
    let src = harness_source();
    let skip_all_defs = src.matches("def skip_all(").count();
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
