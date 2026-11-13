//! Coverage contract for the Markdown formatting gate.
//!
//! Gaps 25 and 37 closed by declaring "the formatting linter" as their acceptance
//! criterion, and no such linter existed anywhere in the tree. `rg -i 'linter'` outside
//! `CHANGELOG*` and `gaps.md` returned zero, and `scripts/` held no Markdown checker, so
//! the criterion could not fail. A criterion that cannot fail is not a criterion: the
//! format was clean by tidiness, and any new document could have reverted it in silence.
//!
//! `scripts/check_markdown_format.py` is that missing artifact. These tests keep it from
//! becoming decorative, in three ways that a green scan alone cannot prove.
//!
//! The first is reachability. A checker that exists but no gate runs is the exact defect
//! `gaps_v064_gate_runner.rs` was written to close, one level down. The battery must
//! declare a gate that runs this script.
//!
//! The second is the self-test. Every rule is a regex, and a regex that matches nothing
//! reports a clean tree for the same reason a broken smoke alarm reports no fire. The
//! self-test executes one fixture per rule and requires each to fire, plus clean controls
//! that must not fire — proving each detector is wired to a real signal.
//!
//! The third is the live tree. The self-test runs in memory; only executing the scanner
//! against the real documents proves the scope globs find the published surface at all.
//! A scanner that walks zero files exits 0 and means nothing.

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// The gate script under contract.
const SCRIPT: &str = "check_markdown_format.py";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    fs::read_to_string(workspace_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// Python launchers to try, in order: `python3`, `python`, then the Windows `py -3`.
const PYTHON_LAUNCHERS: [(&str, &[&str]); 3] = [("python3", &[]), ("python", &[]), ("py", &["-3"])];

/// A working Python 3.9+ launcher. Panics rather than skipping: a skipped test here
/// would report a green board over a scanner nobody executed.
fn python3() -> (&'static str, &'static [&'static str]) {
    let probe = "import sys; sys.exit(0 if sys.version_info >= (3, 9) else 1)";
    for (program, prefix) in PYTHON_LAUNCHERS {
        let out = Command::new(program)
            .args(prefix)
            .args(["-c", probe])
            .output();
        if matches!(out, Ok(ref o) if o.status.success()) {
            return (program, prefix);
        }
    }
    panic!(
        "no Python 3.9+ interpreter found: tried `python3`, `python` and `py -3`. The gate \
         scripts are Python 3 (stdlib only); install it so this test can execute the scanner"
    );
}

/// Run the scanner with `args` and return `(exit code, stdout, stderr)`.
fn run_scanner(args: &[&str]) -> (Option<i32>, String, String) {
    let (program, prefix) = python3();
    let script = workspace_root().join("scripts").join(SCRIPT);
    let mut command = Command::new(program);
    command.args(prefix).arg(&script).args(args);
    let out = command
        .output()
        .unwrap_or_else(|e| panic!("run {program} {}: {e}", script.display()));
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The battery must declare a gate that runs the scanner.
///
/// This is the assertion whose absence made the original linter a claim instead of a
/// gate: the artifact existed in prose only, and nothing in the tree could notice.
#[test]
fn the_battery_declares_the_markdown_format_gate() {
    let runner = read("scripts/check_all_gates.py");
    assert!(
        runner.contains(SCRIPT),
        "scripts/check_all_gates.py must run {SCRIPT}: a formatting linter outside the \
         battery is the same blind spot as no linter, which is how Gaps 25 and 37 came to \
         name a criterion that could not fail"
    );
}

/// Every rule must fire on its own fixture.
///
/// `--selftest` carries one case per rule, so its exit code is a statement about the
/// detectors themselves rather than about the current tree.
#[test]
fn every_formatting_rule_fires_on_its_fixture() {
    let (code, stdout, stderr) = run_scanner(&["--selftest"]);
    assert_eq!(
        code,
        Some(0),
        "`{SCRIPT} --selftest` must exit 0: a rule whose fixture does not fire is a \
         detector that cannot report a violation. stderr: {stderr}"
    );
    assert!(
        stdout.contains("self-test: OK"),
        "`{SCRIPT} --selftest` must print its OK line on stdout, got: {stdout}"
    );
}

/// The live tree must scan clean, over a non-trivial number of documents.
///
/// The count assertion is the part that catches a scope glob which silently matches
/// nothing: zero files would make the scan vacuously green.
#[test]
fn the_published_documents_scan_clean() {
    let (code, stdout, stderr) = run_scanner(&[]);
    assert_eq!(
        code,
        Some(0),
        "the published Markdown surface must satisfy the formatting contract, got: \
         {stderr}"
    );
    let number = stdout
        .split('(')
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("cannot read the document count from: {stdout}"));
    assert!(
        number >= 30,
        "the scanner must cover the published surface (root, docs/ and skills/), found \
         only {number} documents: a scope glob that matches nothing reports a clean tree"
    );
}

/// The raw research corpora stay out of scope, and the exclusion stays declared.
///
/// `docs_rules/` and `docs-prd/` are imported source material, not prose this project
/// authors. Excluding them is correct; excluding them without saying so is not, because
/// an undeclared omission reads as full coverage.
#[test]
fn the_out_of_scope_directories_are_declared() {
    let script = read(&format!("scripts/{SCRIPT}"));
    for directory in ["docs_rules", "docs-prd"] {
        assert!(
            script.contains(directory),
            "scripts/{SCRIPT} must declare {directory} as out of scope: an omission \
             nobody wrote down is indistinguishable from an oversight"
        );
    }
    assert!(
        script.contains("OUT_OF_SCOPE"),
        "the exclusion must be a named constant, not a scattered string literal"
    );
}

/// The scanner must not require anything beyond the Python standard library.
///
/// Every gate script runs on three operating systems with no install step. An import of
/// a third-party package would make the gate vanish on a clean machine, which is worse
/// than a red gate because nothing reports it.
#[test]
fn the_scanner_imports_only_the_standard_library() {
    let script = read(&format!("scripts/{SCRIPT}"));
    for line in script.lines() {
        let trimmed = line.trim_start();
        let Some(module) = trimmed.strip_prefix("import ") else {
            continue;
        };
        let name = module.split_whitespace().next().unwrap_or("");
        let allowed = [
            "argparse",
            "re",
            "sys",
            "_gatekit",
            "pathlib",
            "typing",
            "__future__",
        ];
        assert!(
            allowed.contains(&name),
            "scripts/{SCRIPT} imports `{name}`, which is outside the stdlib allowlist: the \
             gate must run with no install step on Linux, macOS and Windows"
        );
    }
}
