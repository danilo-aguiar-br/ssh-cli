// SPDX-License-Identifier: MIT OR Apache-2.0
//! Storage hardening at the process boundary: lock and marker file modes.
//!
//! # Why argv and not the library
//!
//! The guarantee is about what an operator finds on disk after a real
//! invocation, so each case spawns the binary with a fresh `--config-dir`.
//! The lock wait itself (sliced sleep, signal latency, budget) is covered by
//! the unit tests next to `wait_for_lock` in `src/vps/config_io.rs`, where the
//! clock and the stop flag are injected instead of real signals.
//!
//! # Why the mode assertions are Unix-only
//!
//! Windows has no POSIX mode bits: `std` reports only a read-only flag there and
//! the crate does not apply a DACL (`fs_perm` reports `Unsupported`). Asserting
//! `0o600` on Windows would test the platform, not the product, so the cases
//! still check that both files exist but compare modes only under `cfg(unix)`.

use assert_cmd::Command;
use serial_test::serial;
use tempfile::TempDir;

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

/// Returns the permission bits (`& 0o777`) of `name` inside the config dir.
#[cfg(unix)]
fn mode_of(tmp: &TempDir, name: &str) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(tmp.path().join(name))
        .unwrap_or_else(|e| panic!("{name} missing: {e}"))
        .permissions()
        .mode()
        & 0o777
}

#[test]
#[serial]
fn vps_add_leaves_the_lock_file_private() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "alpha");
    assert!(tmp.path().join("config.toml.lock").exists());
    #[cfg(unix)]
    assert_eq!(mode_of(&tmp, "config.toml.lock"), 0o600);
}

#[test]
#[serial]
fn connect_leaves_the_active_marker_private() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "alpha");
    cmd(&tmp).args(["connect", "alpha"]).assert().success();
    let marker = std::fs::read_to_string(tmp.path().join("active")).expect("active marker");
    assert_eq!(marker.trim(), "alpha");
    #[cfg(unix)]
    assert_eq!(mode_of(&tmp, "active"), 0o600);
}

#[test]
#[serial]
fn connect_rewrites_the_marker_without_loosening_its_mode() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "alpha");
    seed(&tmp, "beta");
    cmd(&tmp).args(["connect", "alpha"]).assert().success();
    cmd(&tmp).args(["connect", "beta"]).assert().success();
    let marker = std::fs::read_to_string(tmp.path().join("active")).expect("active marker");
    assert_eq!(marker.trim(), "beta");
    #[cfg(unix)]
    assert_eq!(mode_of(&tmp, "active"), 0o600);
}

/// A pre-existing lock with a loose mode is tightened by the next mutation.
///
/// This exercises the second line of defense (`chmod` after open): `mode(0o600)`
/// on `OpenOptions` only applies when the file is created, so an old 0644 lock
/// left by a previous version must still be repaired.
#[cfg(unix)]
#[test]
#[serial]
fn a_loose_legacy_lock_is_tightened_by_the_next_write() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "alpha");
    let lock = tmp.path().join("config.toml.lock");
    std::fs::set_permissions(&lock, std::fs::Permissions::from_mode(0o644)).expect("loosen lock");
    seed(&tmp, "beta");
    assert_eq!(mode_of(&tmp, "config.toml.lock"), 0o600);
}
