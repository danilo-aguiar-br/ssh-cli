//! Markdown tables must render with the column count they declare.
//!
//! A pipe inside a table cell ends that cell, even when the pipe sits inside a code
//! span. GitHub and crates.io both apply that rule before they apply inline code, so
//! ``| `scp upload|download` | text |`` is a three-column row inside a two-column
//! table: the reader sees ``download` `` as a column of its own and the description
//! shifted or dropped. Nothing errors. The document stays valid Markdown and renders
//! wrong, which is why this class survived every prose gate the repository had.
//!
//! Measured when this file was written: four such rows shipped in `README.md` and
//! `README.pt-BR.md`, on the `scp` and `secrets` lines — the landing page rendered on
//! crates.io. Neighbouring rows in the same tables escaped their pipes correctly, so
//! the defect was not a convention anyone had decided against; it was drift between
//! rows nobody could see by reading the source.
//!
//! The check is deliberately structural rather than lexical. It counts unescaped
//! pipes per row and compares against the delimiter row that opens each table, so it
//! catches a cell lost to any cause — a stray pipe in prose, a regex in a cell, a
//! copied shell alternation — not only the shape that was fixed here.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every Markdown document a reader or a registry renders.
const RENDERED_DOCS: &[&str] = &[
    "README.md",
    "README.pt-BR.md",
    "CONTRIBUTING.md",
    "CONTRIBUTING.pt-BR.md",
    "SECURITY.md",
    "SECURITY.pt-BR.md",
    "INTEGRATIONS.md",
    "INTEGRATIONS.pt-BR.md",
    "CODE_OF_CONDUCT.md",
    "CODE_OF_CONDUCT.pt-BR.md",
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
];

/// Counts pipes that actually separate cells, skipping `\|`.
fn unescaped_pipes(line: &str) -> usize {
    let bytes: Vec<char> = line.chars().collect();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == '\\' {
            // A backslash escapes the next character, whatever it is.
            i += 2;
            continue;
        }
        if bytes[i] == '|' {
            count += 1;
        }
        i += 1;
    }
    count
}

/// A delimiter row is the `|---|---|` line that fixes a table's column count.
fn is_delimiter_row(line: &str) -> bool {
    let body = line.trim();
    if !body.starts_with('|') {
        return false;
    }
    let inner: String = body.chars().filter(|c| *c != '|').collect();
    !inner.trim().is_empty()
        && inner
            .chars()
            .all(|c| c == '-' || c == ':' || c.is_whitespace())
}

fn scan(path: &Path, text: &str, out: &mut Vec<String>) {
    let mut expected: Option<usize> = None;
    let mut in_fence = false;

    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();

        // Fenced code blocks may contain anything, including tables that are examples.
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        if !trimmed.starts_with('|') {
            // Any non-table line closes the current table.
            expected = None;
            continue;
        }

        if is_delimiter_row(line) {
            expected = Some(unescaped_pipes(line));
            continue;
        }

        let Some(want) = expected else {
            // Header row: its own count becomes the reference only once the
            // delimiter row confirms it, so nothing to compare against yet.
            continue;
        };

        let got = unescaped_pipes(line);
        if got != want {
            out.push(format!(
                "{}:{} declares {want} cell separators but the row has {got}\n      {}",
                path.display(),
                idx + 1,
                line.trim().chars().take(120).collect::<String>()
            ));
        }
    }
}

#[test]
fn every_table_row_matches_its_declared_column_count() {
    let mut broken = Vec::new();

    for doc in RENDERED_DOCS {
        let path = root().join(doc);
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {doc}: {e}"));
        scan(Path::new(doc), &text, &mut broken);
    }

    assert!(
        broken.is_empty(),
        "these table rows will render with the wrong number of columns:\n  {}\n\n\
         A pipe inside a cell ends the cell even within a code span. Write `\\|` when \
         the pipe is content — for command alternation prefer separate spans \
         (`` `a` / `b` ``), which also lets a retriever match each command literally.",
        broken.join("\n  ")
    );
}

/// The escape must be the documented one, so the fix is not merely "delete the pipe".
#[test]
fn the_escape_and_the_counter_agree() {
    assert_eq!(unescaped_pipes("| a | b |"), 3, "plain row");
    assert_eq!(
        unescaped_pipes(r"| a\|b | c |"),
        3,
        "escaped pipe is content"
    );
    assert_eq!(
        unescaped_pipes("| a|b | c |"),
        4,
        "bare pipe splits the cell"
    );
    assert_eq!(
        unescaped_pipes(r"| a\\ | b |"),
        3,
        "escaped backslash is not an escape"
    );
    assert!(is_delimiter_row("|---|---|"));
    assert!(is_delimiter_row("| :--- | ---: |"));
    assert!(!is_delimiter_row("| Command | Purpose |"));
    assert!(
        !is_delimiter_row("||"),
        "a row with no dashes fixes no width"
    );
}
