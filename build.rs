// SPDX-License-Identifier: MIT OR Apache-2.0
//! Build script for ssh-cli.
//!
//! Embeds the commit hash into the `SSH_CLI_COMMIT_HASH` environment variable.
//! GAP-SSH-REL-002: `-dirty` suffix when the working tree has local changes.
//! GAP-SSH-REL-007: precedence env → `.commit_hash` (crates.io pack) → git → `unknown`.
//!
//! # External process policy (G-PROC)
//!
//! The only `std::process::Command` uses in this crate tree at **build** time are
//! optional `git` probes for the short HEAD hash. Runtime product code never
//! spawns local children (SSH is pure `russh`). Missing `git` is non-fatal.
//!
//! Each spawn sets `stdin`/`stdout`/`stderr` explicitly (rules: no implicit
//! inheritance). Arguments are static slices via `args` (no shell, no user input).

use std::process::{Command, Stdio};

fn main() {
    let hash = commit_hash_with_dirty();
    println!("cargo:rustc-env=SSH_CLI_COMMIT_HASH={hash}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.commit_hash");
    println!("cargo:rerun-if-env-changed=SSH_CLI_COMMIT_HASH");
}

/// Short HEAD hash, with `-dirty` when `git status --porcelain` is non-empty.
///
/// Precedence (REL-007):
/// 1. `SSH_CLI_COMMIT_HASH` env (CI/publish inject)
/// 2. `git rev-parse` when a checkout exists
/// 3. `.commit_hash` file in the package manifest (crates.io tarball, which has no
///    `.git`)
/// 4. `unknown`
///
/// # Why git outranks the file
///
/// The file used to win, on the assumption that it only exists inside a published
/// pack. It does not: `.commit_hash` is tracked in the repository, so every local
/// build of a checkout read a frozen value. Measured on 2026-08-10: the binary
/// reported `f3b98b7` while `HEAD` was `09590ba`, six commits later — provenance that
/// is confidently wrong is worse than provenance that is missing, because nobody
/// double-checks a hash that looks plausible.
///
/// Asking git first fixes the class rather than the instance. The file keeps its
/// stated purpose exactly: a crates.io tarball ships without `.git`, so the git probe
/// finds nothing there and the file answers, as designed.
fn commit_hash_with_dirty() -> String {
    if let Ok(env_hash) = std::env::var("SSH_CLI_COMMIT_HASH") {
        let t = env_hash.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }

    if let Some(hash) = git_head_hash() {
        return with_dirty_suffix(hash);
    }

    if let Some(file_hash) = read_commit_hash_file() {
        // G-E2E-06: reached when there is no checkout — the crates.io pack case. The
        // `-dirty` suffix is still applied because it is a no-op outside a checkout
        // and honest inside one (a source tree that vendored the file).
        return with_dirty_suffix(file_hash);
    }

    "unknown".to_string()
}

/// Short `HEAD` hash from git, or [`None`] when this is not a usable checkout.
///
/// Optional build-time dependency: `git` on PATH. Absent git, absent `.git`, or a
/// failed invocation all mean "no checkout here", which is a normal state for a
/// source build from a published tarball rather than an error.
fn git_head_hash() -> Option<String> {
    let output = git_command()
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.is_empty() {
        return None;
    }
    Some(hash)
}

/// Appends `-dirty` when a git checkout has local changes (G-E2E-06).
///
/// No-op when git is missing, not a checkout, or porcelain is empty. Never
/// strips an existing `-dirty` suffix.
fn with_dirty_suffix(mut hash: String) -> String {
    if hash.ends_with("-dirty") {
        return hash;
    }
    // Only probe dirty when `.git` exists (crates.io tarball has no git).
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR");
    let has_git = manifest
        .as_ref()
        .map(|d| std::path::Path::new(d).join(".git").exists())
        .unwrap_or(false);
    if !has_git {
        return hash;
    }
    let dirty = git_command()
        .args(["status", "--porcelain"])
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);
    if dirty {
        hash.push_str("-dirty");
    }
    hash
}

/// Builds a non-interactive `git` command with explicit stdio policy.
///
/// - `stdin`: null — never inherit the cargo parent stdin
/// - `stdout`/`stderr`: piped — captured by `.output()`; no terminal noise
///
/// No shell, no env mutation, no user-controlled arguments.
fn git_command() -> Command {
    let mut cmd = Command::new("git");
    // Explicit stdio (G-PROC-01): do not rely on spawn/status inheritance defaults.
    // `.output()` would pipe stdout/stderr and null stdin, but rules require the
    // policy to be visible at the call site for every external process.
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

fn read_commit_hash_file() -> Option<String> {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")?;
    let path = std::path::Path::new(&manifest_dir).join(".commit_hash");
    let raw = std::fs::read_to_string(path).ok()?;
    let t = raw.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}
