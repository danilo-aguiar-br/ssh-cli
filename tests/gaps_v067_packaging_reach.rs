// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gate: every source file must survive `cargo package`.
//!
//! # The measured incident
//!
//! Round 0.5.5 renamed `src/cli/exec_target/` to `src/cli/target/`. That name is
//! ordinary Rust and the compiler was happy: `cargo build`, `cargo clippy`,
//! `cargo test --all-features` and all eleven gate scripts stayed green.
//!
//! The published crate did not compile.
//!
//! `.gitignore` carried a bare `target/`. A gitignore pattern with no leading
//! slash matches at *every* depth, so `src/cli/target/` was ignored by git.
//! Inside a checkout `cargo package` builds its file list from git, so both files
//! of that module were dropped from the tarball, and the verify step failed with
//! `E0583: file not found for module `target``. The only command in the whole
//! battery that touched the tarball was `cargo publish --dry-run`.
//!
//! # Why a gate rather than a fix
//!
//! The fix was two lines — anchor the rule to `/target/`, rename the module to
//! `targeting`. Neither line stops the *class*. The ignore file grows: it already
//! carries `logs/`, `coverage/`, `uploads/`, `.claude/` and a block rewritten by
//! an external hygiene tool that this repository does not own. Any of those, if a
//! source directory ever takes the same name, deletes source from the published
//! artefact in silence.
//!
//! What makes the class dangerous is not the pattern, it is the *silence*: the
//! defect is invisible to every local build because a local build reads the disk,
//! not the index. So the assertion has to be about reachability, not about any
//! particular forbidden name. A denylist of names like `target` would be
//! superstition once the causal rule is anchored, and would miss the next one.
//!
//! # Two arms, and why the exact one is not the only one
//!
//! - [`no_source_path_is_swallowed_by_a_bare_ignore_pattern`] parses `.gitignore`
//!   itself. It is a literal-only approximation of git's matcher, but it never
//!   skips, needs no subprocess, and catches the whole measured family.
//! - [`git_agrees_that_every_source_file_is_tracked_material`] asks `git` through
//!   `check-ignore`, which is the exact matcher `cargo package` ends up consulting
//!   — globs, negations, precedence and the managed block included. It skips when
//!   git is absent, and says so out loud rather than passing quietly.
//!
//! Keeping both is deliberate. The exact arm can go silent in an environment
//! without git; the approximate arm cannot go silent at all.

#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Directories whose contents ship inside the crate and must stay reachable.
///
/// `tests/` is here because `cargo package` carries the integration suite, and a
/// gate file that vanishes from the tarball is a gate that stops existing exactly
/// where nobody looks — the same silence this suite was written to end.
const PACKAGED_SOURCE_DIRS: &[&str] = &["src", "tests", "benches", "examples"];

/// Collects every file under the packaged source directories, as repo-relative paths.
fn packaged_source_files(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    for dir in PACKAGED_SOURCE_DIRS {
        let path = root.join(dir);
        if path.is_dir() {
            walk(&path, root, &mut out);
        }
    }
    out.sort();
    out
}

/// Literal, unanchored, single-segment ignore patterns paired with their line number.
///
/// Only literals are collected. A pattern carrying `*` or `?` is a glob, and
/// reimplementing git's globbing here would trade a small honest approximation for
/// a large dishonest one — that case is what the `git check-ignore` arm is for.
/// A leading `/` anchors the pattern to the repository root, which is precisely
/// the shape that cannot reach into `src/`, so those are skipped too.
fn bare_literal_ignore_patterns(gitignore: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (idx, raw) in gitignore.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        if line.starts_with('/') {
            continue;
        }
        let body = line.strip_prefix("**/").unwrap_or(line);
        let name = body.strip_suffix('/').unwrap_or(body);
        if name.is_empty() || name.contains('/') {
            continue;
        }
        if name.contains('*') || name.contains('?') || name.contains('[') {
            continue;
        }
        out.push((idx + 1, name.to_string()));
    }
    out
}

#[test]
fn no_source_path_is_swallowed_by_a_bare_ignore_pattern() {
    let root = workspace_root();
    let gitignore_path = root.join(".gitignore");
    let Ok(gitignore) = fs::read_to_string(&gitignore_path) else {
        // No ignore file means nothing can be swallowed by one.
        return;
    };

    let patterns = bare_literal_ignore_patterns(&gitignore);
    assert!(
        !patterns.is_empty(),
        "parsed zero bare literal patterns out of `.gitignore`; the parser has \
         drifted away from the file it is supposed to read, so this gate is \
         measuring nothing"
    );

    let mut offenders = Vec::new();
    for rel in packaged_source_files(&root) {
        for segment in rel.split('/') {
            for (line, name) in &patterns {
                if segment == name {
                    offenders.push(format!(
                        "{rel} — segment `{segment}` matches `.gitignore:{line}` (`{name}`)"
                    ));
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "these source paths are ignored by git, so `cargo package` will drop them \
         and the published crate will not compile — a failure no local build can \
         show you:\n  {}\n\nFix by renaming the source path, or by anchoring the \
         `.gitignore` rule with a leading slash when it was only ever meant to \
         name a root-level artefact directory.",
        offenders.join("\n  ")
    );
}

#[test]
fn git_agrees_that_every_source_file_is_tracked_material() {
    let root = workspace_root();
    let files = packaged_source_files(&root);
    assert!(
        !files.is_empty(),
        "walked the packaged source directories and found no files at all; the \
         walk is broken, not the tree"
    );

    let probe = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();
    let inside_work_tree = matches!(&probe, Ok(out) if out.status.success());
    if !inside_work_tree {
        // Announced rather than swallowed: a skip that prints nothing is how a
        // gate stops mattering without anyone deciding that it should.
        eprintln!(
            "gaps_v067: skipping the exact arm — `git` is unavailable or this is \
             not a work tree. The literal arm still ran."
        );
        return;
    }

    // `check-ignore` exits 0 when at least one path is ignored and 1 when none is,
    // so the exit code alone is the answer; stdout names the offenders.
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["check-ignore", "--stdin"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write as _;
            let joined = files.join("\n");
            child
                .stdin
                .take()
                .expect("stdin was piped")
                .write_all(joined.as_bytes())?;
            child.wait_with_output()
        })
        .expect("spawning `git check-ignore` inside a work tree must not fail");

    let ignored = String::from_utf8_lossy(&output.stdout);
    let offenders: Vec<&str> = ignored.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(
        offenders.is_empty(),
        "`git check-ignore` reports these source files as ignored, which is the \
         exact matcher `cargo package` consults when building the tarball inside a \
         checkout:\n  {}\n\nThey exist on disk, they compile locally, and they will \
         be missing from the published crate.",
        offenders.join("\n  ")
    );
}
