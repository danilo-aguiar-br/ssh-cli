// SPDX-License-Identifier: MIT OR Apache-2.0
//! B1 gate: the Windows target must stay buildable, and the docs must not
//! outrun what a gate actually measures.
//!
//! # Why this suite exists
//!
//! `cargo check --target x86_64-pc-windows-msvc` failed with six errors while
//! `fmt`, `clippy`, 818 tests, `deny` and `doc` were all green. Every one of
//! those gates runs for the host triple, and `#[cfg(target_os = "windows")]`
//! code is discarded by cfg expansion *before* type-check on Linux. The Windows
//! sources were therefore invisible to the entire green board while
//! `docs/CROSS_PLATFORM.md` advertised the target as supported.
//!
//! A green gate only proves what the gate COMPILES.
//!
//! Five of the six errors came from `#![forbid(unsafe_code)]` on
//! `src/platform/mod.rs`: an inner attribute on a module file governs its
//! children, and `forbid` — unlike `deny` — cannot be lifted by an inner
//! `#[allow]`. So the module's own Win32 FFI child was forbidden from existing.
//! The sixth was `windows-sys` 0.61 redefining `HANDLE` from an integer to
//! `*mut c_void`, which broke a `handle == 0` comparison.
//!
//! The real fix is `scripts/check_cross_targets.sh`, which type-checks each
//! claimed target. These assertions guard the shape that made the breakage
//! possible in the first place, and keep the script wired into the docs.

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    fs::read_to_string(workspace_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// `src/platform/mod.rs` must `deny`, never `forbid`.
#[test]
fn platform_module_denies_unsafe_instead_of_forbidding_it() {
    let m = read("src/platform/mod.rs");

    assert!(
        !m.contains("forbid(unsafe_code)"),
        "src/platform/mod.rs must not `forbid(unsafe_code)`: the attribute also \
         governs the `windows` child, which is the product's only Win32 FFI \
         surface, and `forbid` cannot be lifted by an inner `#[allow]`. That is \
         what made the Windows target refuse to compile."
    );
    assert!(
        m.contains("deny(unsafe_code)"),
        "the prohibition must remain for every sibling — downgrade to `deny`, do \
         not delete it"
    );
}

/// The FFI child carries the file-scoped exception and keeps its SAFETY proofs.
#[test]
fn windows_ffi_module_carries_a_scoped_exception_with_proofs() {
    let w = read("src/platform/windows.rs");

    assert!(
        w.contains("allow(unsafe_code)"),
        "src/platform/windows.rs needs a file-scoped `#![allow(unsafe_code)]` to \
         override the `deny` inherited from `super`"
    );
    assert!(
        w.contains("// SAFETY:"),
        "the exception is only defensible while every block keeps its SAFETY proof"
    );

    // The G-UNSAFE allowlist must still name this file: the exception is audited,
    // not open-ended.
    let allowlist = read("tests/gaps_v055_unsafe_ffi.rs");
    assert!(
        allowlist.contains("platform/windows.rs"),
        "the unsafe allowlist must keep naming platform/windows.rs"
    );
}

/// `HANDLE` is a raw pointer in windows-sys 0.61 — comparing it to `0` is E0308.
#[test]
fn windows_handle_is_compared_as_a_pointer() {
    let w = read("src/platform/windows.rs");

    // Comment lines are skipped on purpose: the fix itself documents the old
    // spelling, and a naive substring search would flag its own explanation.
    let code_has_int_compare = w
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .any(|l| l.contains("handle == 0"));
    assert!(
        !code_has_int_compare,
        "windows-sys 0.61 changed HANDLE from an integer to `*mut c_void`, so an \
         integer comparison no longer type-checks. Use `handle.is_null()`."
    );
    assert!(
        w.contains("handle.is_null()"),
        "the null guard must survive the type change, not just the comparison"
    );
}

/// The cross-target script exists and covers every target the docs claim.
#[test]
fn cross_target_script_covers_the_advertised_targets() {
    let script = read("scripts/check_cross_targets.sh");

    for triple in [
        "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
        "x86_64-apple-darwin",
    ] {
        assert!(
            script.contains(triple),
            "scripts/check_cross_targets.sh must type-check {triple}"
        );
    }

    assert!(
        script.contains("--no-default-features"),
        "Windows must be checked without default features: the default TLS stack \
         pulls aws-lc-sys, which compiles C and needs a cross toolchain this host \
         does not ship (A8). Dropping default features still type-checks all of \
         the product's own cfg(windows) code."
    );
}

/// The gate must be documented where a contributor will actually meet it.
#[test]
fn the_cross_target_gate_is_declared_in_the_contributor_docs() {
    let contributing = read("CONTRIBUTING.md");
    assert!(
        contributing.contains("check_cross_targets.sh"),
        "CONTRIBUTING.md must list the cross-target gate: an undocumented script \
         is the same blind spot as no script at all"
    );

    let checklist = read("docs/RELEASE_CHECKLIST.md");
    assert!(
        checklist.contains("check_cross_targets.sh"),
        "the release checklist must run the cross-target gate before a build that \
         claims Windows support"
    );
}
