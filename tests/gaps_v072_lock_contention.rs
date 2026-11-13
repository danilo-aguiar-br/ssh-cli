// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exit 69 on registry lock contention carries the lock remedy, not the keyring one.
//!
//! # Why this drives the binary
//!
//! `suggestion()` used to answer with the OS keyring text for every
//! `Unavailable`, so an agent that lost a race for `config.toml.lock` was told to
//! unlock a keyring that was never involved. The unit tests pin the text; this one
//! proves the text actually reaches the envelope on stderr when a real second
//! process holds the lock, which is the only place an agent reads it.
//!
//! The lock is held with `fs2`, the same advisory-lock crate the product uses, so
//! the contention is real on Linux, macOS and Windows alike.

use assert_cmd::Command;
use serial_test::serial;
use tempfile::TempDir;

/// Exit code for an unavailable dependency (`sysexits.h` `EX_UNAVAILABLE`).
const EX_UNAVAILABLE: i32 = 69;

fn cmd(tmp: &TempDir) -> Command {
    let llvm_profile_file = std::env::var_os("LLVM_PROFILE_FILE");
    let mut c = Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
    c.env_clear();
    c.env("PATH", std::env::var_os("PATH").unwrap_or_default());
    if let Some(value) = llvm_profile_file {
        c.env("LLVM_PROFILE_FILE", value);
    }
    c.env("HOME", tmp.path());
    c.env("XDG_CONFIG_HOME", tmp.path());
    c.arg("--config-dir").arg(tmp.path());
    c.arg("--allow-plaintext-secrets");
    c
}

/// Registers one password-authenticated host.
fn seed(tmp: &TempDir, name: &str) {
    cmd(tmp)
        .args([
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--password-stdin",
        ])
        .write_stdin("secret")
        .assert()
        .success();
}

#[test]
#[serial]
fn rename_under_a_held_lock_exits_69_with_the_lock_remedy() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "a");

    // The registry lives at `<dir>/config.toml`, so its sibling lock is
    // `<dir>/config.toml.lock`.
    let lock_path = tmp.path().join("config.toml.lock");
    let holder = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .expect("lock file");
    fs2::FileExt::lock_exclusive(&holder).expect("hold lock");

    let out = cmd(&tmp)
        .args(["vps", "rename", "a", "b", "--lock-wait-ms", "300", "--json"])
        .output()
        .expect("spawn ssh-cli");
    fs2::FileExt::unlock(&holder).expect("release lock");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(EX_UNAVAILABLE),
        "contention must fail with 69, never hang: {stderr}"
    );
    assert!(stderr.contains("unavailable"), "{stderr}");
    assert!(
        stderr.contains("--lock-wait-ms"),
        "the envelope must point at the lock knob: {stderr}"
    );
    assert!(
        !stderr.contains("keyring"),
        "the lock failure must not inherit the keyring advice: {stderr}"
    );
}
