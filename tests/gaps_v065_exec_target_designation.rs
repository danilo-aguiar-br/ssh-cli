// SPDX-License-Identifier: MIT OR Apache-2.0
//! GAP-SSH-EXEC-ARGC-001 / GAP-SSH-EXEC-ENVELOPE-002: explicit target designation.
//!
//! # Why this suite exists
//!
//! On 2026-08-09 a day of configuration landed on the wrong server. The argv was
//! `exec <HOST> --step <CMD>`. `--step` is `ArgAction::Append` and never counted as a
//! positional, so the parser saw one positional, read the *host name* as the command,
//! and resolved the host from the on-disk active marker. Step zero failed with
//! `command not found` — survivable — and every remaining step then ran happily on a
//! machine nobody had named. The aggregate exit was 0.
//!
//! Two properties made that possible and both are asserted here.
//!
//! First, argv arity carried meaning: the *same* token was a host with two
//! positionals and a command with one, and a non-positional flag decided which. That
//! is now impossible to express — the mode is declared by a flag.
//!
//! Second, the single-host envelope named no host, so the misdirection was undetectable by
//! reading stdout. `ExecHostJson` (the fleet path) always carried `name`; the
//! single-host path, the one that accepted the silent fallback, did not.
//!
//! # Why the assertions never touch a real host
//!
//! Every case fails during argv parsing or registry lookup, before any socket is
//! opened. The config directory is a fresh temp dir, so there is nothing to connect
//! to even if a case regressed into attempting a connection.

use assert_cmd::Command;
use serial_test::serial;
use tempfile::TempDir;

/// Exit code for a usage error (`sysexits.h` `EX_USAGE`).
const EX_USAGE: i32 = 64;
/// Exit code for "named thing does not exist" (`sysexits.h` `EX_NOINPUT`).
const EX_NOINPUT: i32 = 66;
/// Exit code `clap` itself returns for a declared argument conflict.
///
/// Deliberately distinct from [`EX_USAGE`]: a `conflicts_with_all` violation is
/// caught by the parser before any product code runs, so the process never reaches
/// the layer that maps errors onto `sysexits`. Asserting 64 here would be asserting
/// a behaviour the binary does not have.
const EX_CLAP_CONFLICT: i32 = 2;

/// The three surfaces that share the target-designation parser.
const EXEC_SURFACES: [&str; 3] = ["exec", "sudo-exec", "su-exec"];

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

/// Registers one host so the registry lookups have something real to match.
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

/// Points the active marker at `name`.
fn connect(tmp: &TempDir, name: &str) {
    cmd(tmp).args(["connect", name]).assert().success();
}

/// Registers a host pointed at a port that refuses instantly.
///
/// Port 1 has no listener, so the connection is refused in microseconds. That gives a
/// deterministic *post-resolution* failure: the registry lookup succeeds, the target
/// is published, and the run then fails on the wire — which is precisely the window
/// where the error envelope must carry the host.
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

/// Runs argv and parses the JSON error envelope from stderr.
///
/// The envelope goes to stderr by contract: stdout carries data and nothing else, so
/// a failing run must leave stdout parseable-empty rather than mixed.
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

/// Runs argv and returns the exit code, refusing to guess when the process was
/// killed by a signal.
fn exit_code(tmp: &TempDir, args: &[&str]) -> i32 {
    let out = cmd(tmp).args(args).output().expect("spawn ssh-cli");
    out.status
        .code()
        .unwrap_or_else(|| panic!("ssh-cli terminated by signal for argv {args:?}"))
}

/// The incident argv itself: a host name followed only by `--step`.
///
/// The active marker deliberately points somewhere *else*. Before the fix this run
/// succeeded against that other host; the whole point of the regression is that a
/// present marker must not rescue an incomplete designation.
#[test]
#[serial]
fn host_with_only_steps_is_rejected_on_every_surface() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "target-host");
    seed(&tmp, "other-host");
    connect(&tmp, "other-host");

    for verb in EXEC_SURFACES {
        let code = exit_code(
            &tmp,
            &[verb, "target-host", "--step", "hostname -f", "--json"],
        );
        assert_eq!(
            code, EX_USAGE,
            "{verb}: a host plus --step must be a usage error, got exit {code}"
        );
    }
}

/// A lone positional is a usage error even with a marker present.
#[test]
#[serial]
fn lone_positional_never_falls_back_to_the_marker() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "other-host");
    connect(&tmp, "other-host");

    for verb in EXEC_SURFACES {
        let code = exit_code(&tmp, &[verb, "uptime", "--json"]);
        assert_eq!(
            code, EX_USAGE,
            "{verb}: a lone positional must not reach the active marker, got exit {code}"
        );
    }
}

/// Two positionals still designate the host explicitly, and an unknown name is a
/// lookup miss rather than a usage error — proof the token was read as a *host*.
#[test]
#[serial]
fn two_positionals_designate_the_host() {
    let tmp = TempDir::new().expect("tempdir");
    for verb in EXEC_SURFACES {
        let code = exit_code(&tmp, &[verb, "absent-host", "hostname -f", "--json"]);
        assert_eq!(
            code, EX_NOINPUT,
            "{verb}: an unknown explicit host must be a lookup miss, got exit {code}"
        );
    }
}

/// The marker is reachable only through the opt-in, and its absence is a typed miss.
#[test]
#[serial]
fn use_active_without_marker_is_a_typed_miss() {
    let tmp = TempDir::new().expect("tempdir");
    for verb in EXEC_SURFACES {
        let code = exit_code(&tmp, &[verb, "--use-active", "uptime", "--json"]);
        assert_eq!(
            code, EX_NOINPUT,
            "{verb}: --use-active with no marker must be exit 66, got exit {code}"
        );
    }
}

/// The displaced-token guard: `--use-active <registered-name>` is the caller still
/// believing the first positional is the host.
#[test]
#[serial]
fn use_active_refuses_a_registered_name_as_command() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "target-host");
    connect(&tmp, "target-host");

    for verb in EXEC_SURFACES {
        let code = exit_code(&tmp, &[verb, "--use-active", "target-host", "--json"]);
        assert_eq!(
            code, EX_USAGE,
            "{verb}: a registered name is not a command, got exit {code}"
        );
    }
}

/// `--use-active` and the fleet selectors are mutually exclusive at the clap layer.
#[test]
#[serial]
fn use_active_conflicts_with_every_selector() {
    let tmp = TempDir::new().expect("tempdir");
    for verb in EXEC_SURFACES {
        for selector in ["--all", "--hosts=a", "--tags=prod"] {
            let code = exit_code(&tmp, &[verb, "--use-active", selector, "uptime", "--json"]);
            assert_eq!(
                code, EX_CLAP_CONFLICT,
                "{verb} {selector}: must conflict with --use-active, got exit {code}"
            );
        }
    }
}

/// Selectors still take exactly one positional; two is a usage error, never a host swap.
#[test]
#[serial]
fn selectors_reject_two_positionals() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "target-host");

    for verb in EXEC_SURFACES {
        for selector in ["--all", "--hosts=target-host", "--tags=prod"] {
            let code = exit_code(&tmp, &[verb, selector, "target-host", "uptime", "--json"]);
            assert_eq!(
                code, EX_USAGE,
                "{verb} {selector}: two positionals must fail, got exit {code}"
            );
        }
    }
}

/// `doctor` must report the *value* of the active marker, not only its path.
///
/// Before this, `local.active_file` gave the path and nothing gave the contents, so
/// no command could answer "which host would an ambient-target command hit?".
#[test]
#[serial]
fn doctor_reports_the_active_host_value() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "target-host");

    let before = cmd(&tmp)
        .args(["doctor", "--json"])
        .output()
        .expect("spawn");
    let before: serde_json::Value =
        serde_json::from_slice(&before.stdout).expect("doctor emits JSON");
    assert!(
        before["local"]["active_vps"].is_null(),
        "no marker must report null, got {}",
        before["local"]["active_vps"]
    );

    connect(&tmp, "target-host");
    let after = cmd(&tmp)
        .args(["doctor", "--json"])
        .output()
        .expect("spawn");
    let after: serde_json::Value =
        serde_json::from_slice(&after.stdout).expect("doctor emits JSON");
    assert_eq!(
        after["local"]["active_vps"], "target-host",
        "doctor must name the active host"
    );
    assert!(
        after["local"]["active_file"].is_string(),
        "the path must remain available alongside the value"
    );
}

/// Help text must teach the opt-in on every surface, or the breaking change becomes
/// a dead end for the caller who hits it.
#[test]
#[serial]
fn help_documents_the_opt_in() {
    let tmp = TempDir::new().expect("tempdir");
    for verb in EXEC_SURFACES {
        let out = cmd(&tmp).args([verb, "--help"]).output().expect("spawn");
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(
            text.contains("--use-active"),
            "{verb} --help must document --use-active"
        );
    }
}

/// GAP-SSH-EXEC-ENVELOPE-002 step 3: the error envelope names the host too.
///
/// The success envelope carried `host_resolved` from 0.5.5, but the error path did
/// not — and the error path is where the information matters most, because a failing
/// step is exactly when a caller needs to know *where* it failed. Measured before the
/// fix: a failure reached through argv and one reached through the marker produced
/// byte-identical envelopes, so the misdirection stayed invisible in the one channel an
/// agent actually reads.
///
/// Both runs below fail for the same reason (the host is unreachable), so any
/// difference between the two envelopes is provenance and nothing else.
///
/// Proven in a separate process on purpose: the resolved target is process state, and
/// `serial_test` only serializes serial tests against each other — a non-serial test
/// still runs concurrently and could observe another test's write.
#[test]
#[serial]
fn the_error_envelope_distinguishes_argv_from_the_marker() {
    let tmp = TempDir::new().expect("tempdir");
    seed_dead(&tmp, "argv-host");
    seed_dead(&tmp, "marker-host");
    connect(&tmp, "marker-host");

    let by_argv = envelope(&tmp, &["exec", "argv-host", "true", "--json"]);
    let by_marker = envelope(&tmp, &["exec", "--use-active", "true", "--json"]);

    assert_eq!(
        by_argv["host_resolved"], "argv-host",
        "the envelope must name the host reached through argv: {by_argv}"
    );
    assert_eq!(by_argv["host_source"], "argv", "{by_argv}");
    assert_eq!(by_argv["active_fallback"], false, "{by_argv}");

    assert_eq!(
        by_marker["host_resolved"], "marker-host",
        "the envelope must name the inherited host: {by_marker}"
    );
    assert_eq!(by_marker["host_source"], "active_marker", "{by_marker}");
    assert_eq!(
        by_marker["active_fallback"], true,
        "an inherited host must admit it: {by_marker}"
    );

    assert_ne!(
        by_argv, by_marker,
        "identical envelopes are what made the original misdirection undetectable"
    );
}

/// A failure that happens *before* a host is resolved must not claim one.
///
/// `vps_not_found` is the case: nothing was resolved, so `host_resolved` is absent
/// rather than empty. An empty string would assert that a host was chosen and has no
/// name, which is a different claim and a false one.
#[test]
#[serial]
fn an_unresolved_failure_reports_no_host_at_all() {
    let tmp = TempDir::new().expect("tempdir");
    let env = envelope(&tmp, &["exec", "absent-host", "true", "--json"]);

    assert_eq!(env["error_code"], "vps_not_found", "{env}");
    assert!(
        env.get("host_resolved").is_none(),
        "a failure with no target must omit the field, not empty it: {env}"
    );
    assert!(env.get("active_fallback").is_none(), "{env}");
}

/// GAP-SSH-EXEC-ARGC-001, applied to the read surface: `health-check` needs a target.
///
/// It used to inherit the marker with no opt-in, defended as safe because a probe is
/// an idempotent read. The probe is safe; the habit is not. This is also the branch
/// the ETD rule says must never ship without a test that injects a *present* marker —
/// and it had none, which is the same blind spot that let the exec defect survive.
#[test]
#[serial]
fn health_check_demands_an_explicit_target() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "probe-host");
    connect(&tmp, "probe-host");

    // A present marker must NOT rescue an undesignated probe.
    assert_eq!(
        exit_code(&tmp, &["health-check", "--json"]),
        EX_USAGE,
        "an undesignated health-check must be a usage error even with a marker set"
    );

    // The opt-in reaches the marker, and fails only on the connection itself.
    let code = exit_code(&tmp, &["health-check", "--use-active", "--json"]);
    assert_ne!(
        code, EX_USAGE,
        "--use-active is the supported way to reach the marker"
    );
}

/// Without a marker, the opt-in is a typed miss rather than a usage error.
///
/// This is the same 64-versus-66 split the exec family uses: 64 means no target was
/// designated, 66 means the designated target does not exist.
#[test]
#[serial]
fn health_check_opt_in_without_a_marker_is_a_typed_miss() {
    let tmp = TempDir::new().expect("tempdir");
    assert_eq!(
        exit_code(&tmp, &["health-check", "--use-active", "--json"]),
        EX_NOINPUT,
        "--use-active with no marker must be exit 66"
    );
}
