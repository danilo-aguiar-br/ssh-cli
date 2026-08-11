// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit Target Designation for `scp` and `sftp`.
//!
//! # Why this suite exists
//!
//! `exec` was fixed first, in the 0.5.5 line, because that is where the incident
//! happened. The transfer surfaces kept two of the same defects.
//!
//! First, a token changed role when a non-positional flag entered the line.
//! `scp upload a b c` read `a` as the *host slot*; adding `--all` reread the same `a`
//! as a *source path*. A caller who took a working single-host line and appended
//! `--all` did not broaden that line — it silently became a fleet upload whose first
//! file was a host name. Because a host name is also a valid relative path, the
//! symptom was a file-not-found deep inside a fan-out rather than a usage error at
//! the door. That argv is now refused with `EX_USAGE`, and fleet multi-file travels
//! through the named `--src` / `--dest` slots where each path states its own role.
//!
//! Second, no transfer envelope named the machine it reached. A write to a remote
//! filesystem is the most expensive kind of side effect this CLI performs, and its
//! success payload could not be used to audit where the bytes landed.
//!
//! # Why the assertions never touch a real host
//!
//! Every case either fails during argv parsing — before a socket exists — or aims at
//! `127.0.0.1:1`, where the kernel refuses instantly with no listener. Nothing here
//! reaches the network, and the config directory is a fresh temp dir per test.

#![forbid(unsafe_code)]

use assert_cmd::Command;
use serial_test::serial;
use tempfile::TempDir;

/// Exit code for a usage error (`sysexits.h` `EX_USAGE`).
const EX_USAGE: i32 = 64;

/// Exit code `clap` itself returns when argv fails to parse.
///
/// Deliberately distinct from [`EX_USAGE`]: a missing *required* positional is caught
/// before any product code runs, so the process never reaches the layer that maps
/// errors onto `sysexits`.
const EX_CLAP_PARSE: i32 = 2;

/// The four transfer subcommands that share the slot parser.
const TRANSFER_SURFACES: [[&str; 2]; 4] = [
    ["scp", "upload"],
    ["scp", "download"],
    ["sftp", "upload"],
    ["sftp", "download"],
];

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

/// Registers a host pointed at a port that refuses instantly.
///
/// Port 1 has no listener, so the connection is refused in microseconds. That gives a
/// deterministic *post-resolution* failure: the registry lookup succeeds, the target
/// is published, and the run then fails on the wire — the exact window in which the
/// error envelope has to carry the host.
fn seed_dead(tmp: &TempDir, name: &str) {
    cmd(tmp)
        .args([
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "127.0.0.1",
            "--port",
            "1",
            "--user",
            "u",
            "--password-stdin",
        ])
        .write_stdin("secret")
        .assert()
        .success();
}

/// Points the active marker at `name`.
fn connect(tmp: &TempDir, name: &str) {
    cmd(tmp).args(["connect", name]).assert().success();
}

/// Creates a local file so upload path validation passes and the run reaches the wire.
fn local_file(tmp: &TempDir, name: &str) -> String {
    let p = tmp.path().join(name);
    std::fs::write(&p, b"payload").expect("write local fixture");
    p.display().to_string()
}

/// Runs argv and returns the exit code, refusing to guess when killed by a signal.
fn exit_code(tmp: &TempDir, args: &[&str]) -> i32 {
    let out = cmd(tmp).args(args).output().expect("spawn ssh-cli");
    out.status
        .code()
        .unwrap_or_else(|| panic!("ssh-cli terminated by signal for argv {args:?}"))
}

/// Runs argv and parses the JSON error envelope from stderr.
fn envelope(tmp: &TempDir, args: &[&str]) -> serde_json::Value {
    let out = cmd(tmp).args(args).output().expect("spawn ssh-cli");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let line = stderr
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| panic!("no JSON envelope for argv {args:?}; stderr was:\n{stderr}"));
    serde_json::from_str(line)
        .unwrap_or_else(|e| panic!("envelope is not JSON for argv {args:?}: {e}\n{line}"))
}

// ---------------------------------------------------------------------------
// Lacuna 1 — a flag may not remap a positional
// ---------------------------------------------------------------------------

/// The violating argv itself: a selector plus three or more path positionals.
///
/// Under the old parser the first token was a source path here and a host name
/// without the selector. Refusing is the only reading that keeps the token's role
/// independent of the flag set, and it must be a *usage* refusal — a run that
/// "helpfully" picked one interpretation would be the original defect wearing a
/// different coat.
#[test]
fn selector_with_three_positionals_is_a_usage_error() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "prod");
    let file = local_file(&tmp, "a.bin");

    for [group, action] in TRANSFER_SURFACES {
        for selector in [vec!["--all"], vec!["--hosts", "prod"]] {
            let mut args = vec![group, action];
            args.extend(selector.iter().copied());
            args.extend([file.as_str(), "b.bin", "/tmp/dir"]);
            assert_eq!(
                exit_code(&tmp, &args),
                EX_USAGE,
                "argv {args:?} is ambiguous and must be refused, never executed elsewhere"
            );
        }
    }
}

/// The refusal explains itself instead of leaving the caller to guess.
///
/// A bare `EX_USAGE` would stop the wrong transfer but teach nothing, and the caller's
/// next move would be to strip the selector — reintroducing the single-host reading
/// that was never what they meant.
#[test]
fn the_ambiguity_refusal_names_the_named_slots() {
    let tmp = TempDir::new().expect("tempdir");
    let out = cmd(&tmp)
        .args(["scp", "upload", "--all", "a", "b", "/tmp/dir"])
        .output()
        .expect("spawn ssh-cli");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--src") && stderr.contains("--dest"),
        "the refusal must point at the slots that express this intent; stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("ambiguous"),
        "the refusal must name the reason; stderr:\n{stderr}"
    );
}

/// The two-positional fleet form is *not* ambiguous and must keep working.
///
/// With one source there is no single-host reading to collide with: two positionals
/// were already a usage error without a selector, so no token changes role. Refusing
/// this too would be over-correction that breaks every documented fleet transfer.
#[test]
fn selector_with_two_positionals_still_parses() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "prod");
    let file = local_file(&tmp, "a.bin");

    let code = exit_code(
        &tmp,
        &["scp", "upload", "--hosts", "prod", &file, "/tmp/a.bin"],
    );
    assert_ne!(
        code, EX_USAGE,
        "one source under a selector is unambiguous and must reach the transfer layer"
    );
}

/// Named slots express fleet multi-file, the form the ambiguity refusal removed.
///
/// Without this the fix would have deleted a capability instead of relocating it.
#[test]
fn named_slots_express_fleet_multi_file() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "prod");
    let a = local_file(&tmp, "a.bin");
    let b = local_file(&tmp, "b.bin");

    let code = exit_code(
        &tmp,
        &[
            "scp", "upload", "--hosts", "prod", "--src", &a, "--src", &b, "--dest", "/tmp/dir",
        ],
    );
    assert_ne!(
        code, EX_USAGE,
        "the named slots are the sanctioned spelling of fleet multi-file"
    );
}

/// A half-filled slot pair is refused rather than guessed at.
#[test]
fn a_lone_named_slot_is_a_usage_error() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "prod");

    assert_eq!(
        exit_code(&tmp, &["scp", "upload", "prod", "--src", "a.bin"]),
        EX_USAGE,
        "--src without --dest has no destination and must not invent one"
    );
    assert_eq!(
        exit_code(&tmp, &["scp", "upload", "prod", "--dest", "/tmp/dir"]),
        EX_USAGE,
        "--dest without --src has nothing to move and must not invent one"
    );
}

/// Named slots plus a selector leave no slot for a positional to fill.
///
/// Accepting a stray positional here would resurrect the whole defect class: a token
/// with no declared role is a token whose role the parser has to invent.
#[test]
fn named_slots_plus_selector_refuse_stray_positionals() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "prod");

    assert_eq!(
        exit_code(
            &tmp,
            &["scp", "upload", "--all", "--src", "a.bin", "--dest", "/tmp/dir", "leftover",],
        ),
        EX_USAGE,
        "every slot is already named, so a positional has nowhere to go"
    );
}

// ---------------------------------------------------------------------------
// Ambient state — argv wins, and the marker is never an implicit target
// ---------------------------------------------------------------------------

/// With an active marker set, the host named in argv is the host that is reached.
///
/// This is the assertion that would have caught the original incident on the transfer
/// side. The marker points at `marker-host`; the argv names `argv-host`; the envelope
/// must say `argv-host` and must attribute it to argv, not to the marker.
#[test]
#[serial]
fn argv_beats_the_active_marker_on_transfers() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "marker-host");
    seed_dead(&tmp, "argv-host");
    connect(&tmp, "marker-host");
    let file = local_file(&tmp, "a.bin");

    let env = envelope(
        &tmp,
        &["--json", "scp", "upload", "argv-host", &file, "/tmp/a.bin"],
    );
    assert_eq!(
        env["target_resolved"], "argv-host",
        "argv designates the target; the marker is not consulted: {env}"
    );
    assert_eq!(env["target_source"], "argv", "{env}");
    assert_eq!(
        env["active_fallback"], false,
        "no part of this run came from the marker: {env}"
    );
}

/// A transfer with no host slot is a usage error, never a marker fallback.
///
/// `scp upload <LOCAL> <REMOTE>` is two positionals and no selector. The tempting
/// reading — "use whatever `connect` last selected" — is exactly the ambient
/// resolution the law forbids for a side-effecting verb.
#[test]
#[serial]
fn a_transfer_without_a_host_slot_never_falls_back_to_the_marker() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "marker-host");
    connect(&tmp, "marker-host");
    let file = local_file(&tmp, "a.bin");

    for [group, action] in TRANSFER_SURFACES {
        let args = [group, action, file.as_str(), "/tmp/a.bin"];
        assert_eq!(
            exit_code(&tmp, &args),
            EX_USAGE,
            "argv {args:?} names no host; with a marker set this must still refuse"
        );
    }
}

/// `sftp` filesystem verbs take the host positionally and never read the marker.
///
/// They are side-effecting (`rm`, `rmdir`, `rename`) or cheap reads (`ls`, `stat`)
/// that share the same connect path, so the same rule has to hold for all of them.
///
/// The refusal is `EX_CLAP_PARSE`, not [`EX_USAGE`]: these verbs declare two required
/// positionals, so a missing host is caught by the parser before any product code
/// runs. Asserting 64 here would assert a behaviour the binary does not have. What
/// matters for the law is identical either way — the run stops without a target
/// instead of borrowing the marker's.
#[test]
#[serial]
fn sftp_filesystem_verbs_require_a_named_host() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "marker-host");
    connect(&tmp, "marker-host");

    for verb in ["ls", "mkdir", "rmdir", "rm", "stat"] {
        let args = ["sftp", verb, "/tmp/x"];
        assert_eq!(
            exit_code(&tmp, &args),
            EX_CLAP_PARSE,
            "argv {args:?} omits the host slot and must not inherit one"
        );
    }
}

// ---------------------------------------------------------------------------
// Lacuna 2 / 3 — the envelope declares the target it reached
// ---------------------------------------------------------------------------

/// Every transfer surface names its target on the failure path.
///
/// The failure path matters most: it is the moment a caller most needs to know which
/// machine was involved, and until now a refused `scp` and a refused `sftp` produced
/// envelopes that were silent about the host entirely.
#[test]
#[serial]
fn transfer_failures_name_the_target_they_reached() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "dead-host");
    let file = local_file(&tmp, "a.bin");

    let cases: [Vec<&str>; 4] = [
        vec!["--json", "scp", "upload", "dead-host", &file, "/tmp/a.bin"],
        vec![
            "--json",
            "scp",
            "download",
            "dead-host",
            "/tmp/a.bin",
            &file,
        ],
        vec!["--json", "sftp", "upload", "dead-host", &file, "/tmp/a.bin"],
        vec!["--json", "sftp", "ls", "dead-host", "/tmp"],
    ];

    for args in cases {
        let env = envelope(&tmp, &args);
        assert_eq!(
            env["target_resolved"], "dead-host",
            "argv {args:?} must name the host it aimed at: {env}"
        );
        assert_eq!(env["target_source"], "argv", "argv {args:?}: {env}");
    }
}

/// The 0.5.5 spelling keeps working, so no existing consumer breaks.
///
/// `host_resolved` / `host_source` are read aliases of the canonical names, written
/// from the same value. A consumer pinned to either spelling must see the same host.
#[test]
#[serial]
fn the_legacy_host_spelling_is_still_emitted() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "dead-host");
    let file = local_file(&tmp, "a.bin");

    let env = envelope(
        &tmp,
        &["--json", "scp", "upload", "dead-host", &file, "/tmp/a.bin"],
    );
    assert_eq!(env["host_resolved"], "dead-host", "{env}");
    assert_eq!(env["host_source"], "argv", "{env}");
    assert_eq!(
        env["host_resolved"], env["target_resolved"],
        "the alias is one value under two names, never a second source of truth: {env}"
    );
    assert_eq!(env["host_source"], env["target_source"], "{env}");
}

/// A failure before the host is resolved must not claim a target.
///
/// Absent is the honest answer: `""` would assert that a host was chosen and is
/// nameless, and only one of those two statements is true.
#[test]
#[serial]
fn a_pre_resolution_failure_claims_no_target() {
    let tmp = TempDir::new().expect("tempdir");
    let file = local_file(&tmp, "a.bin");

    let env = envelope(
        &tmp,
        &[
            "--json",
            "scp",
            "upload",
            "never-registered",
            &file,
            "/tmp/a",
        ],
    );
    assert!(
        env.get("target_resolved").is_none(),
        "the lookup failed, so nothing was resolved: {env}"
    );
    assert!(env.get("host_resolved").is_none(), "{env}");
}
