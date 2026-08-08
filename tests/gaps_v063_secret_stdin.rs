// SPDX-License-Identifier: MIT OR Apache-2.0
//! D13/D14 — secret input surface and argument-order contract.
//!
//! Both findings surfaced only when the official E01–E18 matrix ran against a
//! real sshd for the first time (D1). Three layers of green had asserted the
//! matrix *existed* — by searching the harness script for the literal string
//! `"E10"` — while the harness itself exited 0 without executing anything.
//!
//! These assertions exercise the real binary rather than reading source paths,
//! so they keep holding when the code moves.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ssh-cli"))
}

fn help_for(args: &[&str]) -> String {
    let out = Command::new(bin())
        .args(args)
        .arg("--help")
        .output()
        .expect("spawn ssh-cli");
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text
}

/// D13: every command that accepts a secret must accept it on stdin.
///
/// `exec`, `scp`, `health-check` and `tunnel` all offered
/// `--key-passphrase-stdin`. `vps add` and `vps edit` — the only two commands
/// that *persist* the passphrase — did not, so the one secret that reaches disk
/// was also the one forced through argv, where any local process reads it from
/// `ps`.
#[test]
fn every_persisting_command_accepts_the_key_passphrase_on_stdin() {
    let surfaces: &[&[&str]] = &[
        &["vps", "add"],
        &["vps", "edit"],
        &["exec"],
        &["scp", "upload"],
        &["scp", "download"],
        &["health-check"],
        &["tunnel"],
    ];

    for args in surfaces {
        let help = help_for(args);
        assert!(
            help.contains("--key-passphrase-stdin"),
            "{args:?} must accept --key-passphrase-stdin; argv is not a secret channel"
        );
    }
}

/// The whole point of a `--*-stdin` flag is that the secret never reaches argv.
#[test]
fn the_stdin_variants_exist_for_every_secret_on_vps_add() {
    let help = help_for(&["vps", "add"]);
    for flag in [
        "--password-stdin",
        "--key-passphrase-stdin",
        "--sudo-password-stdin",
        "--su-password-stdin",
    ] {
        assert!(help.contains(flag), "vps add must accept {flag}");
    }
}

/// D13: stdin drains once, so any two `--*-stdin` flags conflict.
///
/// The original guard enumerated pairs (`password && (sudo || su)`) and let
/// `--sudo-password-stdin --su-password-stdin` through: the first read consumed
/// stdin and the second silently stored an empty secret. Counting is the
/// invariant; enumerating pairs is not.
#[test]
fn two_stdin_secrets_are_refused_rather_than_silently_emptied() {
    let combos: &[(&str, &str)] = &[
        ("--password-stdin", "--sudo-password-stdin"),
        ("--password-stdin", "--key-passphrase-stdin"),
        ("--sudo-password-stdin", "--su-password-stdin"),
        ("--key-passphrase-stdin", "--su-password-stdin"),
    ];

    let tmp = std::env::temp_dir().join(format!("ssh-cli-d13-{}", std::process::id()));
    for (a, b) in combos {
        let out = Command::new(bin())
            .args(["--config-dir", &tmp.to_string_lossy()])
            .args(["vps", "add"])
            .args(["--name", "d13", "--host", "127.0.0.1", "--user", "u"])
            .arg(a)
            .arg(b)
            .output()
            .expect("spawn ssh-cli");
        assert!(
            !out.status.success(),
            "{a} + {b} must be refused; a second stdin read yields an empty secret"
        );
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

/// D14: the transfer commands collapse their paths into one variadic positional,
/// so a flag placed *between* positionals truncates the collection and clap
/// rejects the call with a usage error.
///
/// The harness had used `scp upload <host> --timeout N <local> <remote>` for its
/// entire existence and never learned it was invalid, because it never ran. This
/// pins the accepted order so the docs and the harness cannot drift back.
#[test]
fn transfer_commands_reject_a_flag_between_positionals() {
    let tmp = std::env::temp_dir().join(format!("ssh-cli-d14-{}", std::process::id()));
    let interleaved = Command::new(bin())
        .args(["--config-dir", &tmp.to_string_lossy()])
        .args([
            "scp",
            "upload",
            "somehost",
            "--timeout",
            "1000",
            "/tmp/a",
            "/tmp/b",
        ])
        .output()
        .expect("spawn ssh-cli");

    // Exit 2, not 64. `clap::Error::exit` documents "exits with a status of 2" for
    // parse failures; the sysexits codes in `src/errors.rs` (64 = `EX_USAGE`) only
    // apply to *product* errors raised after parsing succeeds. Asserting 64 here was
    // asserting the wrong layer, and the assertion never ran to say so.
    assert_eq!(
        interleaved.status.code(),
        Some(2),
        "a flag between positionals must fail loudly as a clap parse error, not be silently accepted"
    );

    // The supported form puts options ahead of the positional run. It must not
    // fail on *parsing* — it may still fail later on an unknown host.
    let leading = Command::new(bin())
        .args(["--config-dir", &tmp.to_string_lossy()])
        .args([
            "scp",
            "upload",
            "--timeout",
            "1000",
            "somehost",
            "/tmp/a",
            "/tmp/b",
        ])
        .output()
        .expect("spawn ssh-cli");
    // Excluding 2 is the assertion with teeth: 2 is the code clap actually produces
    // on a parse failure, so this is the value the supported order must never yield.
    // The old form excluded 64, which the parser never emits, so it passed vacuously.
    assert_ne!(
        leading.status.code(),
        Some(2),
        "options before the positional run must parse; got a clap parse error instead"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}
