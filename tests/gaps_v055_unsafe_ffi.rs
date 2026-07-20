// SPDX-License-Identifier: MIT OR Apache-2.0
//! G-UNSAFE gates: product unsafe allowlist, forbid inventory, no plaintext env store.
#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn walk_rs(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            if p.file_name().and_then(|s| s.to_str()) == Some("target") {
                continue;
            }
            walk_rs(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// Product `unsafe {` may only appear in the OS FFI allowlist.
#[test]
fn product_unsafe_blocks_only_in_allowlist() {
    let root = workspace_root().join("src");
    let allow = [
        "platform/windows.rs",
        "signals.rs",
        "test_util/env.rs", // test-only encapsulation
    ];
    let mut files = Vec::new();
    walk_rs(&root, &mut files);
    let mut offenders = Vec::new();
    for f in files {
        let rel = f
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if allow.iter().any(|a| rel == *a) {
            continue;
        }
        let text = fs::read_to_string(&f).unwrap();
        for (i, line) in text.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            if t.contains("unsafe {") || t.starts_with("unsafe fn") || t.contains("unsafe impl")
            {
                offenders.push(format!("{rel}:{}: {t}", i + 1));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "product unsafe outside allowlist:\n{}",
        offenders.join("\n")
    );
}

/// Pure modules that must keep `forbid(unsafe_code)`.
#[test]
fn pure_modules_forbid_unsafe_code() {
    let root = workspace_root().join("src");
    let must = [
        "ssh/mod.rs",
        "ssh/client.rs",
        "vps/model.rs",
        "vps/mod.rs",
        "vps/config_io.rs",
        "secrets.rs",
        "concurrency.rs",
        "main.rs",
    ];
    for rel in must {
        let path = root.join(rel);
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        assert!(
            text.contains("forbid(unsafe_code)"),
            "{rel} must contain #![forbid(unsafe_code)]"
        );
    }
}

/// G-UNSAFE-02/08: no plaintext secrets env mutation in sources.
#[test]
fn no_allow_plaintext_env_set_var() {
    let root = workspace_root().join("src");
    let mut files = Vec::new();
    walk_rs(&root, &mut files);
    let mut hits = Vec::new();
    for f in files {
        let text = fs::read_to_string(&f).unwrap();
        if text.contains("SSH_CLI_ALLOW_PLAINTEXT_SECRETS") {
            let rel = f.strip_prefix(&root).unwrap().display();
            for (i, line) in text.lines().enumerate() {
                if line.contains("SSH_CLI_ALLOW_PLAINTEXT")
                    && (line.contains("set_var") || line.contains("remove_var"))
                {
                    hits.push(format!("{rel}:{}", i + 1));
                }
            }
        }
    }
    assert!(
        hits.is_empty(),
        "forbidden plaintext env mutation:\n{}",
        hits.join("\n")
    );
}

/// G-UNSAFE-13: binary registers signals before Tokio multi_thread.
#[test]
fn main_registers_signals_before_multi_thread_runtime() {
    let main = fs::read_to_string(workspace_root().join("src/main.rs")).unwrap();
    let reg = main
        .find("signals::register_handler")
        .expect("main must call signals::register_handler");
    let rt = main
        .find("new_multi_thread")
        .expect("main must build multi_thread runtime");
    assert!(
        reg < rt,
        "G-UNSAFE-13: register_handler must appear before new_multi_thread in main.rs"
    );
}

/// Test env mutation is encapsulated in test_util.
#[test]
fn test_util_env_module_exists() {
    let p = workspace_root().join("src/test_util/env.rs");
    let text = fs::read_to_string(&p).expect("test_util/env.rs");
    assert!(text.contains("SAFETY:"));
    assert!(text.contains("std::env::set_var"));
    assert!(text.contains("std::env::remove_var"));
}
