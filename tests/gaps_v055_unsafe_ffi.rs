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

/// Modules allowed to hold `unsafe` at all: OS FFI plus its test encapsulation.
///
/// Shared by both gates so an `allow(unsafe_code)` attribute can never open a
/// hole that the `unsafe {` scan does not already know about.
const UNSAFE_FFI_ALLOWLIST: &[&str] = &[
    "platform/windows.rs",
    "signals.rs",
    "test_util/env.rs", // test-only encapsulation
];

/// Product `unsafe {` may only appear in the OS FFI allowlist.
#[test]
fn product_unsafe_blocks_only_in_allowlist() {
    let root = workspace_root().join("src");
    let allow = UNSAFE_FFI_ALLOWLIST;
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
            if t.contains("unsafe {") || t.starts_with("unsafe fn") || t.contains("unsafe impl") {
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

/// Modules that carry no `unsafe_code` lint of their own, each with its reason.
///
/// Every entry must be justified here; an unjustified gap is the bug this gate
/// exists to catch.
const UNSAFE_LINT_EXEMPT: &[(&str, &str)] = &[
    // `include!` fragments: an inner attribute is illegal outside a module root,
    // so these inherit the `forbid` of `ssh/client_real.rs`.
    ("ssh/client_real_core.rs", "include! fragment"),
    ("ssh/client_real_scp.rs", "include! fragment"),
    ("ssh/client_real_sftp.rs", "include! fragment"),
    ("ssh/client_real_tests_body.rs", "include! fragment"),
    // `#[cfg(test)] mod tests;` bodies: they inherit the parent module's lint.
    ("agent_shape_tests.rs", "cfg(test) submodule"),
    ("cli/tests.rs", "cfg(test) submodule"),
    ("ssh/client_tests.rs", "cfg(test) submodule"),
    ("tunnel/tests.rs", "cfg(test) submodule"),
    // OS FFI holders: they need `unsafe`, and `product_unsafe_blocks_only_in_
    // allowlist` is the gate that bounds them.
    ("signals.rs", "OS signal FFI, in UNSAFE_FFI_ALLOWLIST"),
    (
        "test_util/env.rs",
        "test-only env FFI, in UNSAFE_FFI_ALLOWLIST",
    ),
];

/// Every product module states an explicit `unsafe_code` lint decision.
///
/// This used to be a literal inventory of eight paths, which made it blind to
/// whole subsystems (`src/secrets/`, `src/i18n/`, `src/sftp/`) and let it pass
/// in silence the moment a new module landed. Sweeping `src/` recursively
/// asserts the real invariant instead of a layout snapshot.
#[test]
fn every_product_module_states_an_unsafe_code_lint() {
    let root = workspace_root().join("src");
    let mut files = Vec::new();
    walk_rs(&root, &mut files);
    assert!(
        files.len() > 20,
        "the src/ walk is broken, not the product: found only {} files",
        files.len()
    );

    let mut offenders = Vec::new();
    for f in &files {
        let rel = f
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if UNSAFE_LINT_EXEMPT.iter().any(|(p, _)| rel == *p) {
            continue;
        }
        let text = fs::read_to_string(f).unwrap();
        let decided = text.contains("forbid(unsafe_code)")
            || text.contains("deny(unsafe_code)")
            || text.contains("allow(unsafe_code)");
        if !decided {
            offenders.push(rel);
        }
    }
    assert!(
        offenders.is_empty(),
        "these product modules state no `unsafe_code` lint and are not a declared \
         exception in UNSAFE_LINT_EXEMPT:\n  {}",
        offenders.join("\n  ")
    );

    // A file-scoped `allow(unsafe_code)` is the only way to widen the surface,
    // so it may never appear outside the FFI allowlist.
    let mut widened = Vec::new();
    for f in &files {
        let rel = f
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(f).unwrap();
        if text.contains("allow(unsafe_code)") && !UNSAFE_FFI_ALLOWLIST.contains(&rel.as_str()) {
            widened.push(rel);
        }
    }
    assert!(
        widened.is_empty(),
        "allow(unsafe_code) outside UNSAFE_FFI_ALLOWLIST:\n  {}",
        widened.join("\n  ")
    );

    // Keep the exemption list honest: a stale entry hides a real regression.
    let mut stale = Vec::new();
    for (rel, why) in UNSAFE_LINT_EXEMPT {
        if !root.join(rel).is_file() {
            stale.push(format!("{rel} ({why})"));
        }
    }
    assert!(
        stale.is_empty(),
        "UNSAFE_LINT_EXEMPT names files that no longer exist:\n  {}",
        stale.join("\n  ")
    );
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
