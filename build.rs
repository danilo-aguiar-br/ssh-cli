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
/// 2. `.commit_hash` file in the package manifest (embedded in crates.io tarball)
/// 3. `git rev-parse` when a checkout exists
/// 4. `unknown`
fn commit_hash_with_dirty() -> String {
    if let Ok(env_hash) = std::env::var("SSH_CLI_COMMIT_HASH") {
        let t = env_hash.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }

    if let Some(file_hash) = read_commit_hash_file() {
        // G-E2E-06: `.commit_hash` is for crates.io packs; local dirty trees must
        // still surface `-dirty` so provenance matches the working tree.
        return with_dirty_suffix(file_hash);
    }

    // Optional build-time dependency: `git` on PATH. Not required for crates.io
    // source builds (`.commit_hash` or env take precedence when present).
    let Ok(output) = git_command()
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    else {
        return "unknown".to_string();
    };
    if !output.status.success() {
        return "unknown".to_string();
    }
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.is_empty() {
        return "unknown".to_string();
    }
    with_dirty_suffix(hash)
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
