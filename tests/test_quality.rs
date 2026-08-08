// SPDX-License-Identifier: MIT OR Apache-2.0
//! Guards against tests that pass by construction (G-QA-R01).
//!
//! # Why
//!
//! `src/tunnel.rs` shipped `assert_eq!(0_u64, 0)` under the name
//! `timeout_zero_conceptually_rejected`. It was green forever, exercised no product
//! code, and its name told any reviewer that the `timeout_ms == 0` guard was covered.
//! Deleting that guard — and with it the one-shot bound that keeps `tunnel` from
//! becoming a daemon — would not have failed the suite.
//!
//! With 470+ tests nobody audits each one by hand; the aggregate green count is what
//! gets trusted, so a single tautology quietly inflates it. This suite makes the
//! pattern impossible to reintroduce silently.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Recursively collects `.rs` files under a directory.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Strips `//` line comments so commented-out examples do not trip the scan.
fn without_line_comments(line: &str) -> &str {
    match line.find("//") {
        Some(idx) => &line[..idx],
        None => line,
    }
}

/// True when the text is an integer, float, bool or char literal.
///
/// Handles suffixed numeric literals (`0_u64`, `42u8`, `1.5f64`): the numeric core is
/// taken first and whatever follows must be a known primitive suffix. Naively trimming
/// trailing alphabetic characters does not work — `0_u64` ends in a digit.
fn is_literal(token: &str) -> bool {
    let t = token.trim();
    if t == "true" || t == "false" {
        return true;
    }
    if t.starts_with('\'') && t.ends_with('\'') && t.len() >= 3 {
        return true;
    }
    let core: String = t
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if core.is_empty() {
        return false;
    }
    let rest = t[core.len()..].trim_start_matches('_');
    rest.is_empty()
        || matches!(
            rest,
            "u8" | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "f32"
                | "f64"
        )
}

#[test]
fn no_assert_between_two_literals() {
    let mut files = Vec::new();
    rust_files(&root().join("src"), &mut files);
    rust_files(&root().join("tests"), &mut files);

    let mut offenders = Vec::new();
    for file in &files {
        // This very file describes the pattern in prose and in its own helpers.
        if file.ends_with("test_quality.rs") {
            continue;
        }
        let Ok(body) = std::fs::read_to_string(file) else {
            continue;
        };
        for (idx, raw) in body.lines().enumerate() {
            let line = without_line_comments(raw);
            let Some(start) = line
                .find("assert_eq!(")
                .or_else(|| line.find("assert_ne!("))
            else {
                continue;
            };
            let after = &line[start..];
            let Some(open) = after.find('(') else {
                continue;
            };
            let Some(close) = after.rfind(')') else {
                continue;
            };
            if close <= open + 1 {
                continue;
            }
            let args = &after[open + 1..close];
            // Only the simple two-argument form; anything with nesting is not a
            // literal-vs-literal comparison.
            if args.contains('(') || args.contains('{') || args.contains('[') {
                continue;
            }
            let parts: Vec<&str> = args.split(',').collect();
            if parts.len() != 2 {
                continue;
            }
            if is_literal(parts[0]) && is_literal(parts[1]) {
                offenders.push(format!(
                    "{}:{}: {}",
                    file.strip_prefix(root()).unwrap_or(file).display(),
                    idx + 1,
                    raw.trim()
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "assertions comparing two literals are true by construction and exercise no \
         product code, while inflating the green count and implying coverage that does \
         not exist. Assert on real behaviour instead.\nOffenders:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn no_assert_true_literal() {
    let mut files = Vec::new();
    rust_files(&root().join("src"), &mut files);
    rust_files(&root().join("tests"), &mut files);

    let mut offenders = Vec::new();
    for file in &files {
        if file.ends_with("test_quality.rs") {
            continue;
        }
        let Ok(body) = std::fs::read_to_string(file) else {
            continue;
        };
        for (idx, raw) in body.lines().enumerate() {
            let line = without_line_comments(raw).replace(' ', "");
            if line.contains("assert!(true)") || line.contains("assert!(true,") {
                offenders.push(format!(
                    "{}:{}: {}",
                    file.strip_prefix(root()).unwrap_or(file).display(),
                    idx + 1,
                    raw.trim()
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "`assert!(true)` can never fail.\nOffenders:\n{}",
        offenders.join("\n")
    );
}

/// The scan must actually be able to catch the pattern it exists to forbid.
///
/// A detector that silently matches nothing is its own kind of tautology, so the
/// matcher is exercised against known-bad and known-good shapes.
#[test]
fn literal_detector_recognises_the_pattern_it_forbids() {
    assert!(is_literal("0"));
    assert!(is_literal("0_u64"));
    assert!(is_literal(" 42 "));
    assert!(is_literal("true"));
    assert!(is_literal("1.5"));

    assert!(!is_literal("port"));
    assert!(!is_literal("v.len()"));
    assert!(!is_literal("EX_OK"));
    assert!(!is_literal(""));
}

#[test]
fn line_comment_stripper_ignores_commented_examples() {
    assert_eq!(
        without_line_comments("let x = 1; // assert_eq!(0, 0)").trim(),
        "let x = 1;"
    );
}
