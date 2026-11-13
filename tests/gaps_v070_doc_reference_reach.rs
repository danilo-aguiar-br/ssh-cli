//! Every repository path a document names must exist on disk.
//!
//! Coverage audits are usually run in one direction: walk the tree, then check that
//! each file found is mentioned somewhere. That direction proves nothing about the
//! opposite failure, where a document names an artefact the repository never had.
//! Nothing errors. The prose reads as a promise of proof, the reader goes looking,
//! and the trail ends.
//!
//! Measured when this file was written: `docs/TESTING.md` and `docs/TESTING.pt-BR.md`
//! both catalogued `tests/gaps_v063_gate_falsification.rs`, asserting that every check
//! of `scripts/check_en_identifiers.py` had a standing falsifiability proof. The file
//! did not exist and never had. The forward sweep — is every suite in `tests/` listed
//! in TESTING? — returned a clean zero, because the phantom is invisible from that
//! side. Only the reverse sweep, document to disk, finds it.
//!
//! The check reads paths out of inline code spans, which is where this repository
//! writes canonical paths, and skips globs and placeholders because those name a set
//! rather than a file.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Documents whose path references are load-bearing claims about this repository.
const DOCS_WITH_PATH_CLAIMS: &[&str] = &[
    "README.md",
    "README.pt-BR.md",
    "CONTRIBUTING.md",
    "CONTRIBUTING.pt-BR.md",
    "SECURITY.md",
    "SECURITY.pt-BR.md",
    "INTEGRATIONS.md",
    "INTEGRATIONS.pt-BR.md",
    "docs/AGENTS.md",
    "docs/AGENTS.pt-BR.md",
    "docs/COOKBOOK.md",
    "docs/COOKBOOK.pt-BR.md",
    "docs/HOW_TO_USE.md",
    "docs/HOW_TO_USE.pt-BR.md",
    "docs/CROSS_PLATFORM.md",
    "docs/CROSS_PLATFORM.pt-BR.md",
    "docs/MIGRATION.md",
    "docs/MIGRATION.pt-BR.md",
    "docs/TESTING.md",
    "docs/TESTING.pt-BR.md",
    "docs/RELEASE_CHECKLIST.md",
    "docs/RELEASE_CHECKLIST.pt-BR.md",
    "docs/schemas/README.md",
    "llms.txt",
    "llms-full.txt",
    "llms.pt-BR.txt",
];

/// Directories whose contents are committed, so a named file there must be present.
const TRACKED_ROOTS: &[&str] = &[
    "tests/",
    "scripts/",
    "src/",
    "docs/",
    "benches/",
    "examples/",
];

/// Extracts the contents of every inline code span on one line.
///
/// A fence marker is not a span opener, so a line that starts one is skipped whole;
/// otherwise the triple backtick would be read as an unterminated span and swallow
/// the rest of the file's meaning on that line.
fn code_spans(line: &str) -> Vec<String> {
    if line.trim_start().starts_with("```") {
        return Vec::new();
    }
    let chars: Vec<char> = line.chars().collect();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '`' {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < chars.len() && chars[end] != '`' {
            end += 1;
        }
        if end >= chars.len() {
            break;
        }
        spans.push(chars[start..end].iter().collect());
        i = end + 1;
    }
    spans
}

/// True when the span names one concrete file this repository should carry.
///
/// A glob names a set and a placeholder names a variable, so neither is a claim that
/// a particular file exists. Treating them as claims would make the gate noisy enough
/// to be switched off, which is the usual way a true gate dies.
fn is_concrete_repo_path(span: &str) -> bool {
    if !TRACKED_ROOTS.iter().any(|r| span.starts_with(r)) {
        return false;
    }
    if span.ends_with('/') {
        return false;
    }
    if span.contains('*') || span.contains('<') || span.contains('{') || span.contains(' ') {
        return false;
    }
    // A path with no extension is usually a module reference, not a file on disk.
    Path::new(span).extension().is_some()
}

#[test]
fn every_repository_path_a_document_names_exists() {
    let root = root();
    let mut missing: Vec<String> = Vec::new();

    for doc in DOCS_WITH_PATH_CLAIMS {
        let path = root.join(doc);
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => panic!("{doc} is catalogued here but unreadable: {e}"),
        };

        for (idx, line) in text.lines().enumerate() {
            for span in code_spans(line) {
                if !is_concrete_repo_path(&span) {
                    continue;
                }
                if !root.join(&span).exists() {
                    missing.push(format!("{doc}:{} names `{span}`", idx + 1));
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "documents name repository files that do not exist:\n{}",
        missing.join("\n")
    );
}

#[test]
fn every_suite_on_disk_is_catalogued_in_testing() {
    let root = root();
    let mut on_disk: BTreeSet<String> = BTreeSet::new();

    let entries = std::fs::read_dir(root.join("tests")).expect("tests/ must be readable");
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".rs") {
            on_disk.insert(format!("tests/{name}"));
        }
    }

    for doc in ["docs/TESTING.md", "docs/TESTING.pt-BR.md"] {
        let text = std::fs::read_to_string(root.join(doc)).expect("TESTING must be readable");
        let uncatalogued: Vec<&String> = on_disk.iter().filter(|s| !text.contains(*s)).collect();
        assert!(
            uncatalogued.is_empty(),
            "{doc} omits suites that exist on disk:\n{}",
            uncatalogued
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn the_extractor_reads_spans_and_ignores_sets() {
    // A span is only what sits between two backticks on the same line.
    assert_eq!(
        code_spans("see `tests/a.rs` and `scripts/b.sh` now"),
        vec!["tests/a.rs".to_string(), "scripts/b.sh".to_string()]
    );
    // A fence marker opens a block, so the line carries no spans of its own.
    assert!(code_spans("```bash").is_empty());
    // An unterminated span is not a claim about anything.
    assert!(code_spans("a `dangling span").is_empty());

    // Concrete files are claims; sets and variables are not.
    assert!(is_concrete_repo_path(
        "tests/gaps_v070_doc_reference_reach.rs"
    ));
    assert!(is_concrete_repo_path("docs/schemas/exec.schema.json"));
    assert!(!is_concrete_repo_path("tests/*.rs"));
    assert!(!is_concrete_repo_path("docs/<NAME>.md"));
    assert!(!is_concrete_repo_path("src/"));
    assert!(!is_concrete_repo_path("ssh-cli exec"));
    // A path outside the tracked roots is somebody else's tree, not a claim on ours.
    assert!(!is_concrete_repo_path("/tmp/hosts.toml"));
}
