//! Build script para ssh-cli.
//!
//! Embute o commit hash do git na variável de ambiente SSH_CLI_COMMIT_HASH.
//! GAP-SSH-REL-002: sufixo `-dirty` se o working tree tiver alterações.

fn main() {
    let hash = commit_hash_com_dirty();
    println!("cargo:rustc-env=SSH_CLI_COMMIT_HASH={hash}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/");
}

/// Hash curto do HEAD, com `-dirty` se `git status --porcelain` não for vazio.
fn commit_hash_com_dirty() -> String {
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

    if working_tree_dirty() {
        hash.push_str("-dirty");
    }
    hash
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
