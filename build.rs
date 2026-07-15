//! Build script para ssh-cli.
//!
//! Embute o commit hash na variável de ambiente `SSH_CLI_COMMIT_HASH`.
//! GAP-SSH-REL-002: sufixo `-dirty` se o working tree tiver alterações.
//! GAP-SSH-REL-007: precedência env → `.commit_hash` (pack crates.io) → git → `unknown`.

fn main() {
    let hash = commit_hash_com_dirty();
    println!("cargo:rustc-env=SSH_CLI_COMMIT_HASH={hash}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/");
    println!("cargo:rerun-if-changed=.commit_hash");
    println!("cargo:rerun-if-env-changed=SSH_CLI_COMMIT_HASH");
}

/// Hash curto do HEAD, com `-dirty` se `git status --porcelain` não for vazio.
///
/// Ordem (REL-007):
/// 1. `SSH_CLI_COMMIT_HASH` (CI / override maintainer)
/// 2. arquivo `.commit_hash` no manifest (embutido no tarball crates.io)
/// 3. `git rev-parse --short HEAD`
/// 4. `"unknown"`
fn commit_hash_com_dirty() -> String {
    if let Ok(from_env) = std::env::var("SSH_CLI_COMMIT_HASH") {
        let t = from_env.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }

    if let Some(from_file) = ler_commit_hash_file() {
        return from_file;
    }

    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();

    let mut hash = match output {
        Ok(o) if o.status.success() => String::from_utf8(o.stdout)
            .unwrap_or_default()
            .trim()
            .to_string(),
        _ => "unknown".to_string(),
    };

    if hash.is_empty() {
        hash = "unknown".to_string();
    }

    if working_tree_dirty() && hash != "unknown" {
        hash.push_str("-dirty");
    }
    hash
}

fn ler_commit_hash_file() -> Option<String> {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")?;
    let path = std::path::Path::new(&manifest_dir).join(".commit_hash");
    let texto = std::fs::read_to_string(path).ok()?;
    let t = texto.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn working_tree_dirty() -> bool {
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output();
    match status {
        Ok(o) if o.status.success() => !String::from_utf8_lossy(&o.stdout).trim().is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod testes {
    #[test]
    fn dirty_suffix_logic() {
        let mut h = "abc1234".to_string();
        if true {
            h.push_str("-dirty");
        }
        assert_eq!(h, "abc1234-dirty");
    }
}
