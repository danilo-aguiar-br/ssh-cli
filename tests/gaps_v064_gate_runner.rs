//! Coverage contract for the mandatory gate battery.
//!
//! `scripts/check_all_gates.sh` was written to close this project's dominant defect
//! class: a gate that nobody executes is not a gate. The audit that followed found
//! the class had recurred inside its own remediation — the runner was referenced by
//! zero files in the repository, while its siblings appeared in six to eleven
//! documents each. Nothing failed if the file was deleted, and no document told a
//! maintainer to run it.
//!
//! Two distinct holes are closed here.
//!
//! The first is coverage. Every `scripts/check_*.sh` happened to be wired into the
//! runner, so coverage was complete *by convention* — but nothing asserted it, and a
//! future `scripts/check_foo.sh` would have slipped in unnoticed. A battery that
//! presents itself as complete while omitting a script in silence reads as total
//! coverage, which is precisely the failure mode of a green board that measures
//! nothing. The runner now declares its deliberate exclusions, and the assertions
//! below require every script to be either a gate or a named exclusion.
//!
//! The second is discoverability. A gate documented nowhere is the same blind spot
//! as no gate at all, so the runner must be declared where a maintainer meets it —
//! in both languages, since a pt-BR-only reader following pt-BR-only docs would
//! otherwise never learn the battery exists.
//!
//! Directory order is not part of the contract: `std::fs::read_dir` documents that
//! the order entries come back in is platform- and filesystem-dependent and may
//! change between calls, so every collection here is sorted before it is asserted
//! on or reported. Without that, a failure message would read differently on two
//! hosts looking at the same tree.

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

/// The runner itself: a gate cannot be its own gate.
const RUNNER: &str = "check_all_gates.sh";

/// Marker that opens the declared-exclusions block in the runner header.
const EXCLUSIONS_OPEN: &str = "# Declared non-gates";

/// Marker that closes it. Anything between the two is a declaration, not a gate.
const EXCLUSIONS_CLOSE: &str = "# Sequential by design";

/// Where the shell array of gates begins. Everything after it is the gate region:
/// splitting here keeps the header's prose — which legitimately names excluded
/// scripts and the runner's own usage line — from being mistaken for a gate entry.
const GATES_OPEN: &str = "GATES=(";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    fs::read_to_string(workspace_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// Every `*.sh` under `scripts/`, sorted for a deterministic failure message.
fn shell_scripts() -> Vec<String> {
    let dir = workspace_root().join("scripts");
    let entries = fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir scripts: {e}"));

    // `size_hint` on ReadDir is uninformative, so seed from the current inventory
    // rather than growing from empty on every push.
    let mut names = Vec::with_capacity(16);
    for entry in entries {
        let name = entry.expect("dir entry").file_name();
        let name = name.to_string_lossy().into_owned();
        if name.ends_with(".sh") {
            names.push(name);
        }
    }
    names.sort_unstable();
    names
}

/// Split the runner into the header (prose, usage, declared exclusions) and the gate
/// region (the array and the driver that consumes it).
fn split_runner(runner: &str) -> (&str, &str) {
    let at = runner
        .find(GATES_OPEN)
        .unwrap_or_else(|| panic!("runner must declare its gates in a `{GATES_OPEN}` array"));
    runner.split_at(at)
}

/// The declared-exclusions block, or a panic naming the missing marker.
fn exclusions_block(header: &str) -> &str {
    let open = header.find(EXCLUSIONS_OPEN).unwrap_or_else(|| {
        panic!(
            "the runner must carry a `{EXCLUSIONS_OPEN}` block: a battery that omits \
             scripts in silence reads as total coverage"
        )
    });
    let rest = &header[open..];
    let close = rest.find(EXCLUSIONS_CLOSE).unwrap_or_else(|| {
        panic!("the `{EXCLUSIONS_OPEN}` block must be terminated by `{EXCLUSIONS_CLOSE}`")
    });
    &rest[..close]
}

/// Every `check_*.sh` in the repository must be a gate in the battery.
///
/// This is the assertion whose absence let the runner's coverage be true only by
/// coincidence of naming.
#[test]
fn every_check_script_is_a_gate_in_the_runner() {
    let runner = read("scripts/check_all_gates.sh");
    let (_, gate_region) = split_runner(&runner);

    let mut missing = Vec::new();
    for name in shell_scripts() {
        if !name.starts_with("check_") || name == RUNNER {
            continue;
        }
        if !gate_region.contains(&name) {
            missing.push(name);
        }
    }

    assert!(
        missing.is_empty(),
        "these check scripts exist but no gate in scripts/{RUNNER} runs them: {missing:?} — \
         a gate outside the battery is a gate nobody executes"
    );
}

/// Every shell script is either a gate or an explicitly declared non-gate.
///
/// The stronger of the two coverage assertions: a new `scripts/*.sh` cannot enter the
/// tree without either joining the battery or being named — with a reason — in the
/// exclusions block.
#[test]
fn every_script_is_either_a_gate_or_a_declared_non_gate() {
    let runner = read("scripts/check_all_gates.sh");
    let (header, gate_region) = split_runner(&runner);
    let declared = exclusions_block(header);

    let mut unaccounted = Vec::new();
    let mut ambiguous = Vec::new();
    for name in shell_scripts() {
        if name == RUNNER {
            continue;
        }
        let is_gate = gate_region.contains(&name);
        let is_excluded = declared.contains(&name);
        match (is_gate, is_excluded) {
            (false, false) => unaccounted.push(name),
            (true, true) => ambiguous.push(name),
            _ => {}
        }
    }

    assert!(
        unaccounted.is_empty(),
        "these scripts are neither a gate in scripts/{RUNNER} nor a declared non-gate: \
         {unaccounted:?} — add them to the battery, or name them in the \
         `{EXCLUSIONS_OPEN}` block with the reason they are excluded"
    );
    assert!(
        ambiguous.is_empty(),
        "these scripts are both a gate and a declared exclusion, so the declaration \
         contradicts the battery: {ambiguous:?}"
    );
}

/// The exclusions block must name every script it excludes, not gesture at a category.
#[test]
fn the_declared_exclusions_name_each_script() {
    let runner = read("scripts/check_all_gates.sh");
    let (header, _) = split_runner(&runner);
    let declared = exclusions_block(header);

    for name in [
        "dist_multiarch.sh",
        "generate_sbom.sh",
        "release_attest.sh",
        "e2e_real_ssh.sh",
    ] {
        assert!(
            declared.contains(name),
            "the declared-exclusions block must name {name}: an unnamed omission is \
             indistinguishable from an oversight"
        );
    }
}

/// The test gate must keep `--locked`.
///
/// Dropping it would let the suite resolve a different dependency graph than the one
/// `Cargo.lock` pins, so the battery would be measuring a tree nobody ships.
#[test]
fn the_test_gate_stays_locked() {
    let runner = read("scripts/check_all_gates.sh");
    let (_, gate_region) = split_runner(&runner);

    let test_gate = gate_region
        .lines()
        .find(|line| line.trim_start().starts_with("\"test|"))
        .expect("the battery must contain a `test` gate");

    assert!(
        test_gate.contains("--locked"),
        "the test gate must pass --locked so the battery measures the pinned graph: {test_gate}"
    );
}

/// A partial run must say so, so it can never be reported as a full board.
#[test]
fn the_runner_reports_what_it_skipped() {
    let runner = read("scripts/check_all_gates.sh");
    assert!(
        runner.contains("skipped"),
        "the summary record must report skipped gates: --only would otherwise produce \
         a green board indistinguishable from a full run"
    );
}

/// The battery must be declared in both language versions of every document a
/// maintainer actually reads.
///
/// Asserting only the English file is how `CONTRIBUTING.pt-BR.md` came to omit the
/// cross-target gate entirely without any gate noticing.
#[test]
fn the_battery_is_declared_in_both_languages_of_the_maintainer_docs() {
    for doc in [
        "CONTRIBUTING.md",
        "CONTRIBUTING.pt-BR.md",
        "docs/TESTING.md",
        "docs/TESTING.pt-BR.md",
        "docs/RELEASE_CHECKLIST.md",
        "docs/RELEASE_CHECKLIST.pt-BR.md",
    ] {
        let text = read(doc);
        assert!(
            text.contains("check_all_gates.sh"),
            "{doc} must tell the reader to run the gate battery: an undocumented \
             runner is the same blind spot as no runner at all"
        );
    }
}

/// The advisory freshness gate has no caller other than the battery, so documenting
/// the battery is the only thing that makes it reachable by a human.
#[test]
fn the_advisory_freshness_gate_has_a_documented_home() {
    for doc in [
        "docs/TESTING.md",
        "docs/TESTING.pt-BR.md",
        "docs/RELEASE_CHECKLIST.md",
        "docs/RELEASE_CHECKLIST.pt-BR.md",
    ] {
        let text = read(doc);
        assert!(
            text.contains("check_advisory_freshness.sh"),
            "{doc} must name the advisory freshness gate: the battery is its only \
             consumer, so an undocumented battery hides it completely"
        );
    }
}
