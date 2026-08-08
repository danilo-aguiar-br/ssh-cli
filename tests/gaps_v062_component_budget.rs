// SPDX-License-Identifier: MIT OR Apache-2.0
//! A7 / B3 / B4 gates: file size, lint suppressions and deferred constants.
//!
//! # Why this suite exists
//!
//! A7 recorded three oversized modules and stayed `PARCIAL` across two rounds
//! because nothing measured it. B3 then found the reason the coupling never
//! surfaced: `#[allow(clippy::too_many_arguments)]` appeared sixteen times, and
//! twelve of those sat in exactly the files A7 flagged. The one lint that would
//! have reported the growth was silenced there, so `-D warnings` stayed green
//! over it.
//!
//! Six of those sixteen suppressions turned out to be dead attributes: the
//! functions had already dropped below the seven-argument threshold, and nobody
//! noticed because a stale `allow` costs nothing visible.
//!
//! These assertions turn all three into measurements that fail loudly.
//!
//! # Why the collector partitions product from test modules (D3)
//!
//! The first cut of this suite walked every `.rs` file under `src/` and called
//! the whole set "product sources". Nine of those files are test modules
//! (`tests.rs` / `*_tests.rs`) totalling roughly 3100 lines, and the same
//! conflation reached the lint counter: one of the three
//! `#[allow(clippy::too_many_arguments)]` sites sits under `#[cfg(test)]`, so
//! the product surface actually carries two.
//!
//! The ledger had already written the bug down. The entry for
//! `tunnel/tests.rs` justified itself with "test module, not product logic" —
//! prose admitting that the file did not belong to the set the constant above it
//! was measuring. A gate whose own exception text explains why the entry is
//! misclassified is not measuring what it claims to measure.
//!
//! Both populations are now measured, each against its own declared budget, and
//! a file is charged to exactly one of them.

#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Ceiling for any product source file that is not a declared exception.
///
/// # Why this is 600 and not "whatever passes today" (C3)
///
/// The first cut of this gate set a single ceiling of 830 lines, chosen so that
/// `src/ssh/sftp_session.rs` (810) would fit. Its doc comment named that one
/// file as "a deliberate exception" — but the constant silently exempted *nine*
/// others sitting between 601 and 810. The gate documented one exception and
/// granted ten, and its own failure message told the reader not to raise the
/// ceiling while the ceiling had already been raised to the status quo.
///
/// That is the B1 failure mode reappearing inside B1's own remediation: a gate
/// calibrated so that it passes proves only that it was calibrated.
///
/// The budget is now the target, and every file above it is named individually
/// in [`DECLARED_EXCEPTIONS`] with its own frozen cap.
const HARD_BUDGET: usize = 600;

/// Files above [`HARD_BUDGET`], each frozen at its measured size.
///
/// This is a debt ledger, not an exemption. An entry may never grow, and once a
/// file falls to [`HARD_BUDGET`] its entry must be deleted — the test fails on a
/// stale entry, so the list can only shrink. New code is held to the budget.
///
/// Format: (path relative to `src/`, frozen cap, why splitting is not obviously
/// better than leaving it whole).
const DECLARED_EXCEPTIONS: &[(&str, usize, &str)] = &[
    (
        "ssh/sftp_session.rs",
        810,
        "one cohesive SFTP protocol implementation; splitting moves the wire \
         sequence across files without reducing real coupling",
    ),
    (
        "errors.rs",
        754,
        "single error taxonomy; the enum and its exit-code/class/retryable/\
         suggestion tables must stay exhaustive in one place",
    ),
    (
        "secrets.rs",
        748,
        "at-rest encryption; splitting the key lifecycle widens the surface that \
         touches key material",
    ),
    (
        "cli/mod.rs",
        712,
        "clap derive definition; the argument surface is one declarative unit",
    ),
    (
        "vps/model.rs",
        699,
        "record model plus serde migration; wire schema v3 dual-read belongs with \
         the fields it reads",
    ),
    (
        "cli/dispatch.rs",
        651,
        "dispatch table; one match over the whole verb surface",
    ),
    (
        "json_wire/vps_export.rs",
        619,
        "export/import wire; EN and pt-BR alias handling stay together",
    ),
    // `tunnel.rs` was here at 601. It is gone because the file now measures 570: the
    // `TunnelStats` counters and their unit test moved to `src/tunnel/stats.rs`, which is
    // the split this ratchet asks for when a declared file grows. The ledger shrinking by
    // a whole entry is the outcome the contract is designed to produce; adding capacity
    // instead would have been the dishonest move it exists to block.
];

/// Ceiling for a test module living under `src/` (`tests.rs`, `*_tests.rs`).
///
/// # Where this number comes from (and where it deliberately does not)
///
/// It is parity with [`HARD_BUDGET`], derived from a stated principle: a test
/// module is read, navigated and maintained like any other file in the tree, so
/// it earns no larger budget than the code it exercises.
///
/// It is emphatically *not* the largest test module measured today. That was
/// the C3 failure — a ceiling of 830 picked so the biggest file of the moment
/// would fit, which proves only that the author looked at the file. The largest
/// test module here is 754 lines, so this budget does **not** fit the status quo
/// on the day it was written; the overflow is named individually in
/// [`DECLARED_TEST_EXCEPTIONS`] and can only shrink.
const TEST_MODULE_BUDGET: usize = 600;

/// Test modules above [`TEST_MODULE_BUDGET`], each frozen at its measured size.
///
/// Same ratchet contract as [`DECLARED_EXCEPTIONS`]: an entry may never grow,
/// and an entry that is no longer needed fails the suite.
const DECLARED_TEST_EXCEPTIONS: &[(&str, usize, &str)] = &[(
    "tunnel/tests.rs",
    754,
    "mirrors the tunnel surface it exercises; forward, reverse and socks cases \
     share one fixture harness that splitting would have to duplicate",
)];

/// Suppressions of the coupling lint that remain justified after B3.
///
/// Counted on the product surface only. The third site in the tree,
/// `src/vps/model.rs`'s `test_new`, sits under `#[cfg(test)]` and mirrors
/// `try_new` by design; charging it to the product budget was the same
/// product/test conflation D3 found in the line-count collector.
///
/// Each survivor must carry a comment explaining why the flat shape is correct.
const MAX_TOO_MANY_ARGS_ALLOWS: usize = 2;

fn walk_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            walk_rs(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// Every `.rs` file under `src/`, product and test module alike.
fn all_sources() -> Vec<(String, String)> {
    let root = workspace_root().join("src");
    let mut files = Vec::new();
    walk_rs(&root, &mut files);
    files
        .into_iter()
        .filter_map(|p| {
            let rel = p
                .strip_prefix(&root)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");
            Some((rel, fs::read_to_string(&p).ok()?))
        })
        .collect()
}

/// A test module is a file whose whole reason to exist is `#[cfg(test)]`.
///
/// The predicate is the naming convention this tree already follows without
/// exception: `tests.rs` for a module directory, `<unit>_tests.rs` beside the
/// unit it covers. Both spellings end in `tests.rs`.
fn is_test_module(rel: &str) -> bool {
    rel.ends_with("tests.rs")
}

fn product_sources() -> Vec<(String, String)> {
    all_sources()
        .into_iter()
        .filter(|(rel, _)| !is_test_module(rel))
        .collect()
}

fn test_module_sources() -> Vec<(String, String)> {
    all_sources()
        .into_iter()
        .filter(|(rel, _)| is_test_module(rel))
        .collect()
}

fn declared_cap(ledger: &[(&str, usize, &str)], rel: &str) -> Option<usize> {
    ledger
        .iter()
        .find(|(p, _, _)| *p == rel)
        .map(|(_, cap, _)| *cap)
}

/// Line index where a file's inline `#[cfg(test)] mod ...` block starts.
///
/// Everything from that line down is test surface even in a product file. The
/// convention here puts that block last, so a single index is enough.
///
/// The `mod` lookahead matters: `#[cfg(test)]` also guards individual test-only
/// items (`src/vps/model.rs`'s `test_new`), and treating the first such
/// attribute as a module boundary would silently exempt every product item
/// below it.
fn inline_test_module_start(lines: &[&str]) -> Option<usize> {
    lines.iter().enumerate().find_map(|(i, l)| {
        if !l.trim().starts_with("#[cfg(test)]") {
            return None;
        }
        let declares_mod = lines[i + 1..]
            .iter()
            .map(|next| next.trim())
            .find(|next| !next.is_empty() && !next.starts_with("#[") && !next.starts_with("//"))
            .is_some_and(|next| next.starts_with("mod ") || next.starts_with("pub mod "));
        declares_mod.then_some(i)
    })
}

/// Whether the item at `idx` carries `#[cfg(test)]` in its own attribute block.
///
/// Walks up over the contiguous run of attributes and comments that belong to
/// the same item; anything else ends the block.
fn item_is_cfg_test(lines: &[&str], idx: usize) -> bool {
    let mut i = idx;
    while i > 0 {
        i -= 1;
        let t = lines[i].trim();
        if t.starts_with("#[cfg(test)]") {
            return true;
        }
        if t.starts_with("#[") || t.starts_with("//") {
            continue;
        }
        return false;
    }
    false
}

/// An undeclared file must fit the budget; a declared one may not grow.
#[test]
fn no_product_file_exceeds_its_budget() {
    let mut undeclared = Vec::new();
    let mut grown = Vec::new();

    for (rel, text) in product_sources() {
        let lines = text.lines().count();
        match declared_cap(DECLARED_EXCEPTIONS, &rel) {
            Some(cap) if lines > cap => {
                grown.push(format!("{rel}: {lines} lines, frozen at {cap}"));
            }
            Some(_) => {}
            None if lines > HARD_BUDGET => {
                undeclared.push(format!("{rel}: {lines} lines"));
            }
            None => {}
        }
    }

    assert!(
        undeclared.is_empty(),
        "these files cross the {HARD_BUDGET}-line budget and are not declared \
         exceptions:\n  {}\n\
         Split them by responsibility (see `src/vps/exec_ops/` and `src/sftp/` \
         for the pattern). Adding an entry to DECLARED_EXCEPTIONS instead is a \
         decision that must be argued in the reason field, not a formality.",
        undeclared.join("\n  ")
    );

    assert!(
        grown.is_empty(),
        "these declared exceptions grew past their frozen cap:\n  {}\n\
         The ledger ratchets down only. Reduce the file, or split it.",
        grown.join("\n  ")
    );
}

/// Test modules are held to their own declared budget, not to the product one.
///
/// Charging them to [`HARD_BUDGET`] was the D3 bug: nine files, roughly 3100
/// lines, counted as product. Exempting them entirely would be the opposite
/// error — an unmeasured population always grows.
#[test]
fn no_test_module_exceeds_its_budget() {
    let mut undeclared = Vec::new();
    let mut grown = Vec::new();

    for (rel, text) in test_module_sources() {
        let lines = text.lines().count();
        match declared_cap(DECLARED_TEST_EXCEPTIONS, &rel) {
            Some(cap) if lines > cap => {
                grown.push(format!("{rel}: {lines} lines, frozen at {cap}"));
            }
            Some(_) => {}
            None if lines > TEST_MODULE_BUDGET => {
                undeclared.push(format!("{rel}: {lines} lines"));
            }
            None => {}
        }
    }

    assert!(
        undeclared.is_empty(),
        "these test modules cross the {TEST_MODULE_BUDGET}-line budget and are \
         not declared exceptions:\n  {}\n\
         Split them by the behaviour they cover. A test module gets the same \
         budget as product code because it is read the same way.",
        undeclared.join("\n  ")
    );

    assert!(
        grown.is_empty(),
        "these declared test-module exceptions grew past their frozen cap:\n  {}\n\
         The ledger ratchets down only. Reduce the module, or split it.",
        grown.join("\n  ")
    );
}

/// The debt ledger may not outlive the debt.
///
/// Without this, a file that gets split below the budget keeps its entry, and
/// the entry silently re-authorizes the next 200 lines of growth. Every stale
/// row is a permit nobody meant to issue.
fn assert_ledger_is_current(
    label: &str,
    ledger: &[(&str, usize, &str)],
    budget: usize,
    sources: &[(String, String)],
) {
    let mut missing = Vec::new();
    let mut no_longer_needed = Vec::new();

    for (path, cap, reason) in ledger {
        assert!(
            !reason.trim().is_empty(),
            "{path} is declared without a reason"
        );
        assert!(
            *cap > budget,
            "{path} is capped at {cap}, at or below the {budget}-line \
             budget — it does not need an exception at all"
        );

        match sources.iter().find(|(rel, _)| rel == path) {
            None => missing.push((*path).to_string()),
            Some((_, text)) => {
                let lines = text.lines().count();
                if lines <= budget {
                    no_longer_needed.push(format!("{path}: now {lines} lines"));
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "these entries in {label} name files that are not in the population it \
         measures (deleted, renamed, or charged to the other ledger):\n  {}",
        missing.join("\n  ")
    );

    assert!(
        no_longer_needed.is_empty(),
        "these files now fit the {budget}-line budget — delete their entries \
         from {label}:\n  {}",
        no_longer_needed.join("\n  ")
    );
}

#[test]
fn the_exception_ledger_contains_no_stale_entries() {
    assert_ledger_is_current(
        "DECLARED_EXCEPTIONS",
        DECLARED_EXCEPTIONS,
        HARD_BUDGET,
        &product_sources(),
    );
}

/// Also catches a test module wrongly filed in the product ledger, which is how
/// `tunnel/tests.rs` sat there for two rounds with its own reason field saying
/// "test module, not product logic".
#[test]
fn the_test_module_ledger_contains_no_stale_entries() {
    assert_ledger_is_current(
        "DECLARED_TEST_EXCEPTIONS",
        DECLARED_TEST_EXCEPTIONS,
        TEST_MODULE_BUDGET,
        &test_module_sources(),
    );
}

#[test]
fn coupling_lint_suppressions_stay_within_budget_and_are_justified() {
    let mut sites = Vec::new();
    let mut unjustified = Vec::new();

    for (rel, text) in product_sources() {
        let lines: Vec<&str> = text.lines().collect();
        let inline_tests = inline_test_module_start(&lines);
        for (i, line) in lines.iter().enumerate() {
            if !line.contains("allow(clippy::too_many_arguments)") {
                continue;
            }
            // D3: a suppression on a `#[cfg(test)]` item is not product
            // coupling. `src/vps/model.rs`'s `test_new` was charged to the
            // product budget purely because it lives in a product file.
            if inline_tests.is_some_and(|start| i >= start) || item_is_cfg_test(&lines, i) {
                continue;
            }
            sites.push(format!("{rel}:{}", i + 1));

            // A justification is a comment on the attribute's own line or in the
            // block immediately above it. Without one, the suppression carries no
            // record of *why* the flat shape is correct — which is how twelve of
            // them silently outlived their reason.
            let inline = line.contains("//");
            let above = i
                .checked_sub(1)
                .and_then(|j| lines.get(j))
                .is_some_and(|prev| prev.trim_start().starts_with("//"));
            if !inline && !above {
                unjustified.push(format!("{rel}:{}", i + 1));
            }
        }
    }

    assert!(
        sites.len() <= MAX_TOO_MANY_ARGS_ALLOWS,
        "{} suppressions of `too_many_arguments` (budget {MAX_TOO_MANY_ARGS_ALLOWS}):\n  {}\n\
         Group the parameters into a named context struct instead — see \
         `crate::vps::AuthOverrides` and `crate::tunnel::ServeContext`.",
        sites.len(),
        sites.join("\n  ")
    );

    assert!(
        unjustified.is_empty(),
        "these suppressions carry no comment explaining why the flat shape is \
         correct:\n  {}",
        unjustified.join("\n  ")
    );
}

/// B4: constants must live in `crate::constants`, not as deferred TODOs.
///
/// `SCP_IO_CHUNK = 32_768` was declared twice inside `client_real_scp.rs`, each
/// copy carrying a marker admitting it belonged in `constants.rs`. The stated
/// reason was an edit allowlist from a round that had already closed, so the
/// justification expired while the duplication stayed.
#[test]
fn no_deferred_constant_markers_remain() {
    // Scans the whole tree, product and test modules alike: a deferred constant
    // is debt wherever it is parked.
    let offenders: Vec<String> = all_sources()
        .into_iter()
        .flat_map(|(rel, text)| {
            text.lines()
                .enumerate()
                .filter(|(_, l)| l.contains("TODO(constants)"))
                .map(|(i, l)| format!("{rel}:{}: {}", i + 1, l.trim()))
                .collect::<Vec<_>>()
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "deferred constant markers found — move the value into \
         `src/constants.rs`:\n  {}",
        offenders.join("\n  ")
    );
}

/// The SCP stream window and header cap are named constants, used from both
/// transfer directions.
#[test]
fn scp_constants_are_centralized_and_used() {
    let constants =
        fs::read_to_string(workspace_root().join("src/constants.rs")).expect("read constants.rs");
    assert!(
        constants.contains("SCP_IO_CHUNK") && constants.contains("SCP_HEADER_MAX_BYTES"),
        "both SCP constants must be declared in crate::constants"
    );

    for rel in ["src/ssh/client_real_scp.rs", "src/ssh/scp_wire.rs"] {
        let text = fs::read_to_string(workspace_root().join(rel)).expect("read scp source");
        assert!(
            !text.contains("const SCP_IO_CHUNK") && !text.contains("const SCP_HEADER_MAX_BYTES"),
            "{rel} must not redeclare an SCP constant locally"
        );
    }
}
