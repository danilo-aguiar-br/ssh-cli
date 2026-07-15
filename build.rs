// SPDX-License-Identifier: MIT OR Apache-2.0
//! Build script for ssh-cli.
//!
//! Embeds the commit hash into the `SSH_CLI_COMMIT_HASH` environment variable.
//! GAP-SSH-REL-002: `-dirty` suffix when the working tree has local changes.
//! GAP-SSH-REL-007: precedence env → `.commit_hash` (crates.io pack) → git → `unknown`.

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
        return file_hash;
    }

    let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    else {
        return "unknown".to_string();
    };
    if !output.status.success() {
        return "unknown".to_string();
    }
    let mut hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.is_empty() {
        return "unknown".to_string();
    }

    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);
    if dirty {
        hash.push_str("-dirty");
    }
    hash
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
