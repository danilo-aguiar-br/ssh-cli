// SPDX-License-Identifier: MIT OR Apache-2.0
//! `vps edit` switches the primary authentication method in one call.
//!
//! # The defect this closes
//!
//! `vps edit --key` on a password host, and `vps edit --password` on a key host,
//! used to exit 64 with "primary auth methods are mutually exclusive": the flags
//! set the new method but never cleared the old one, so the record failed its own
//! validation. The operator had to know a clear flag existed and run two commands
//! with a window between them where the host had no usable credential.
//!
//! # What each case pins
//!
//! - A switch is one command and `updated` lists the field that took the value
//!   plus every field it displaced, and nothing that was already empty.
//! - Two primary-auth flags in one call are refused by `clap` (exit 2) before any
//!   file is read, so the last flag can never win in silence.
//! - A key passphrase with no key anywhere is exit 64 and leaves `config.toml`
//!   byte for byte as it was.
//! - The target comes from argv. The `active` marker pointing at another host
//!   changes nothing, and `vps edit` has no `--use-active` flag at all.
//! - Failures leave stdout empty: the diagnostic goes to stderr.

use assert_cmd::Command;
use serde_json::Value;
use serial_test::serial;
use tempfile::TempDir;

/// Exit code for a usage error (`sysexits.h` `EX_USAGE`).
const EX_USAGE: i32 = 64;
/// Exit code `clap` returns for a malformed invocation.
const EX_CLAP: i32 = 2;

fn cmd(tmp: &TempDir) -> Command {
    cmd_with_dir(tmp, tmp.path())
}

/// Like [`cmd`], with the registry directory chosen by the caller.
fn cmd_with_dir(tmp: &TempDir, config_dir: &std::path::Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
    c.env_clear();
    c.env("PATH", std::env::var_os("PATH").unwrap_or_default());
    if let Some(value) = std::env::var_os("LLVM_PROFILE_FILE") {
        c.env("LLVM_PROFILE_FILE", value);
    }
    c.env("HOME", tmp.path());
    c.env("XDG_CONFIG_HOME", tmp.path());
    c.arg("--config-dir").arg(config_dir);
    c.arg("--allow-plaintext-secrets");
    c
}

/// Runs argv with stdin and returns the exit code plus both streams.
fn run_stdin(tmp: &TempDir, args: &[&str], stdin: &str) -> (i32, String, String) {
    let out = cmd(tmp)
        .args(args)
        .write_stdin(stdin.to_owned())
        .output()
        .expect("spawn ssh-cli");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Runs argv and returns stdout, asserting success.
fn ok_stdin(tmp: &TempDir, args: &[&str], stdin: &str) -> String {
    let (code, stdout, stderr) = run_stdin(tmp, args, stdin);
    assert_eq!(
        code, 0,
        "argv {args:?} failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    stdout
}

fn ok(tmp: &TempDir, args: &[&str]) -> String {
    ok_stdin(tmp, args, "")
}

/// Registers one password-authenticated host.
fn seed(tmp: &TempDir, name: &str) {
    ok_stdin(
        tmp,
        &[
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--password-stdin",
        ],
        "secret",
    );
}

/// Writes a real OpenSSH key fixture, or `None` when `ssh-keygen` is absent.
///
/// The registry validates key content when `ssh-real` is on, so a placeholder is
/// rejected. Test-only: the product never spawns `ssh-keygen`.
fn write_openssh_key(tmp: &TempDir) -> Option<String> {
    write_named_key(tmp, "id_ed25519_test")
}

/// Like [`write_openssh_key`], with the file name chosen by the caller.
fn write_named_key(tmp: &TempDir, file_name: &str) -> Option<String> {
    let key = tmp.path().join(file_name);
    let status = match std::process::Command::new("ssh-keygen")
        .args(["-t", "ed25519", "-N", "", "-q", "-f"])
        .arg(&key)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => panic!("ssh-keygen spawn failed: {e}"),
    };
    assert!(status.success(), "ssh-keygen failed: {status}");
    Some(key.to_str().expect("utf-8 path").to_owned())
}

/// Parses the single compact JSON object a `--json` success run writes to stdout.
fn envelope_of(stdout: &str) -> Value {
    let line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| panic!("no JSON envelope on stdout:\n{stdout}"));
    serde_json::from_str(line).unwrap_or_else(|e| panic!("envelope is not JSON: {e}\n{line}"))
}

fn show(tmp: &TempDir, name: &str) -> Value {
    envelope_of(&ok(tmp, &["vps", "show", name, "--json"]))
}

/// Exports every host with its secrets and returns the `hosts` object.
fn export_hosts(tmp: &TempDir) -> Value {
    let out = ok(
        tmp,
        &[
            "vps",
            "export",
            "--include-secrets",
            "--i-understand-secrets-on-stdout",
            "--json",
        ],
    );
    envelope_of(&out)["hosts"].clone()
}

/// The `updated` array of an envelope as plain strings.
fn updated_of(envelope: &Value) -> Vec<String> {
    envelope["updated"]
        .as_array()
        .unwrap_or_else(|| panic!("no `updated` array: {envelope}"))
        .iter()
        .map(|v| v.as_str().expect("string").to_owned())
        .collect()
}

fn config_bytes(tmp: &TempDir) -> Vec<u8> {
    std::fs::read(tmp.path().join("config.toml")).expect("config.toml")
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

// ── password -> key ─────────────────────────────────────────────────────────

#[test]
#[serial]
fn key_on_a_password_host_switches_the_method() {
    let tmp = TempDir::new().expect("tempdir");
    let Some(key) = write_openssh_key(&tmp) else {
        return;
    };
    seed(&tmp, "web");

    let v = envelope_of(&ok(&tmp, &["vps", "edit", "web", "--key", &key, "--json"]));
    assert_eq!(v["event"], "vps-edited", "{v}");
    assert_eq!(sorted(updated_of(&v)), ["key_path", "password"], "{v}");

    let shown = show(&tmp, "web");
    assert_eq!(shown["key_path"], key, "{shown}");
    assert!(
        shown["password"].is_null(),
        "the password must be gone after the switch: {shown}"
    );
}

#[test]
#[serial]
fn key_with_its_passphrase_in_one_call_lists_all_three_fields() {
    let tmp = TempDir::new().expect("tempdir");
    let Some(key) = write_openssh_key(&tmp) else {
        return;
    };
    seed(&tmp, "web");

    let out = ok_stdin(
        &tmp,
        &[
            "vps",
            "edit",
            "web",
            "--key",
            &key,
            "--key-passphrase-stdin",
            "--json",
        ],
        "key-pass",
    );
    assert_eq!(
        sorted(updated_of(&envelope_of(&out))),
        ["key_passphrase", "key_path", "password"]
    );
    let hosts = export_hosts(&tmp);
    assert_eq!(hosts["web"]["key_passphrase"], "key-pass", "{hosts}");
}

// ── key -> password ─────────────────────────────────────────────────────────

#[test]
#[serial]
fn password_stdin_on_a_key_host_clears_the_key_and_its_passphrase() {
    let tmp = TempDir::new().expect("tempdir");
    let Some(key) = write_openssh_key(&tmp) else {
        return;
    };
    seed(&tmp, "web");
    ok_stdin(
        &tmp,
        &[
            "vps",
            "edit",
            "web",
            "--key",
            &key,
            "--key-passphrase-stdin",
        ],
        "key-pass",
    );

    let out = ok_stdin(
        &tmp,
        &["vps", "edit", "web", "--password-stdin", "--json"],
        "fresh-pw",
    );
    let v = envelope_of(&out);
    assert_eq!(v["event"], "vps-edited", "{v}");
    assert_eq!(
        sorted(updated_of(&v)),
        ["key_passphrase", "key_path", "password"],
        "{v}"
    );

    let hosts = export_hosts(&tmp);
    let web = &hosts["web"];
    assert_eq!(web["password"], "fresh-pw", "{web}");
    assert!(web["key_path"].is_null(), "{web}");
    assert!(web["key_passphrase"].is_null(), "{web}");
}

// ── agent -> key / password ─────────────────────────────────────────────────

/// Registers an agent host with an explicit socket through the documented formula.
fn seed_agent_host(tmp: &TempDir, name: &str) {
    seed(tmp, name);
    let sock = tmp.path().join("agent.sock");
    ok(
        tmp,
        &[
            "vps",
            "edit",
            name,
            "--use-agent",
            "--agent-socket",
            sock.to_str().expect("utf-8 path"),
            "--json",
        ],
    );
    let hosts = export_hosts(tmp);
    assert_eq!(hosts[name]["use_agent"], true, "{hosts}");
    assert!(hosts[name]["agent_socket"].is_string(), "{hosts}");
}

#[test]
#[serial]
fn key_on_an_agent_host_clears_the_agent_flag_and_socket() {
    let tmp = TempDir::new().expect("tempdir");
    let Some(key) = write_openssh_key(&tmp) else {
        return;
    };
    seed_agent_host(&tmp, "web");

    let v = envelope_of(&ok(&tmp, &["vps", "edit", "web", "--key", &key, "--json"]));
    assert_eq!(
        sorted(updated_of(&v)),
        ["agent_socket", "key_path", "use_agent"],
        "{v}"
    );
    let hosts = export_hosts(&tmp);
    // The export omits `use_agent` when it is off.
    assert_ne!(hosts["web"]["use_agent"], true, "{hosts}");
    assert!(hosts["web"]["agent_socket"].is_null(), "{hosts}");
    assert_eq!(hosts["web"]["key_path"], key, "{hosts}");
}

#[test]
#[serial]
fn password_on_an_agent_host_clears_the_agent_flag_and_socket() {
    let tmp = TempDir::new().expect("tempdir");
    seed_agent_host(&tmp, "web");

    let out = ok_stdin(
        &tmp,
        &["vps", "edit", "web", "--password-stdin", "--json"],
        "pw-again",
    );
    assert_eq!(
        sorted(updated_of(&envelope_of(&out))),
        ["agent_socket", "password", "use_agent"]
    );
    let hosts = export_hosts(&tmp);
    // The export omits `use_agent` when it is off.
    assert_ne!(hosts["web"]["use_agent"], true, "{hosts}");
    assert!(hosts["web"]["agent_socket"].is_null(), "{hosts}");
    assert_eq!(hosts["web"]["password"], "pw-again", "{hosts}");
}

#[test]
#[serial]
fn use_agent_reports_the_key_material_it_cleared() {
    let tmp = TempDir::new().expect("tempdir");
    let Some(key) = write_openssh_key(&tmp) else {
        return;
    };
    seed(&tmp, "web");
    ok_stdin(
        &tmp,
        &[
            "vps",
            "edit",
            "web",
            "--key",
            &key,
            "--key-passphrase-stdin",
        ],
        "key-pass",
    );

    let v = envelope_of(&ok(&tmp, &["vps", "edit", "web", "--use-agent", "--json"]));
    assert_eq!(
        sorted(updated_of(&v)),
        ["key_passphrase", "key_path", "use_agent"],
        "{v}"
    );
}

// ── conflicts and the orphan passphrase ─────────────────────────────────────

#[test]
#[serial]
fn two_primary_auth_flags_in_one_call_exit_2_with_empty_stdout() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "web");
    let before = config_bytes(&tmp);

    let conflicting: [&[&str]; 3] = [
        &["--key", "/nonexistent/key", "--password", "x"],
        &["--key", "/nonexistent/key", "--password-stdin"],
        &["--agent-socket", "/run/a.sock", "--password", "x"],
    ];
    for flags in conflicting {
        let mut args = vec!["vps", "edit", "web"];
        args.extend_from_slice(flags);
        let (code, stdout, stderr) = run_stdin(&tmp, &args, "");
        assert_eq!(code, EX_CLAP, "{args:?}\nstderr:\n{stderr}");
        assert!(stdout.is_empty(), "{args:?}\nstdout:\n{stdout}");
    }
    assert_eq!(config_bytes(&tmp), before, "a refused call must not write");
}

#[test]
#[serial]
fn a_passphrase_without_any_key_exits_64_and_leaves_the_file_untouched() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "web");
    let before = config_bytes(&tmp);

    let (code, stdout, stderr) = run_stdin(
        &tmp,
        &["vps", "edit", "web", "--key-passphrase-stdin", "--json"],
        "orphan",
    );
    assert_eq!(code, EX_USAGE, "stderr:\n{stderr}");
    assert!(stdout.is_empty(), "stdout must stay empty:\n{stdout}");
    assert!(
        stderr.contains("passphrase needs a private key"),
        "{stderr}"
    );
    assert_eq!(
        config_bytes(&tmp),
        before,
        "no partial state may be written"
    );
}

#[test]
#[serial]
fn a_passphrase_after_no_key_in_the_same_call_is_refused_without_a_write() {
    let tmp = TempDir::new().expect("tempdir");
    let Some(key) = write_openssh_key(&tmp) else {
        return;
    };
    seed(&tmp, "web");
    ok(&tmp, &["vps", "edit", "web", "--key", &key]);
    let before = config_bytes(&tmp);

    let (code, stdout, stderr) = run_stdin(
        &tmp,
        &[
            "vps",
            "edit",
            "web",
            "--no-key",
            "--key-passphrase-stdin",
            "--json",
        ],
        "orphan",
    );
    assert_eq!(code, EX_USAGE, "stderr:\n{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert_eq!(config_bytes(&tmp), before);
}

#[test]
#[serial]
fn a_missing_key_file_fails_with_empty_stdout_and_no_write() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "web");
    let before = config_bytes(&tmp);

    let (code, stdout, stderr) = run_stdin(
        &tmp,
        &[
            "vps",
            "edit",
            "web",
            "--key",
            "/nonexistent/id_ed25519",
            "--json",
        ],
        "",
    );
    assert_ne!(code, 0, "stderr:\n{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert_eq!(config_bytes(&tmp), before);
}

// ── explicit target designation ─────────────────────────────────────────────

#[test]
#[serial]
fn a_switch_changes_only_the_host_named_on_argv() {
    let tmp = TempDir::new().expect("tempdir");
    let Some(key) = write_openssh_key(&tmp) else {
        return;
    };
    seed(&tmp, "x");
    seed(&tmp, "y");
    ok(&tmp, &["connect", "y"]);
    let y_before = show(&tmp, "y");

    let v = envelope_of(&ok(&tmp, &["vps", "edit", "x", "--key", &key, "--json"]));
    assert_eq!(v["target_resolved"], "x", "{v}");
    assert_eq!(v["target_source"], "argv", "{v}");
    assert_eq!(show(&tmp, "x")["key_path"], key);
    assert_eq!(
        show(&tmp, "y"),
        y_before,
        "the host the marker points at must be untouched"
    );

    let w = envelope_of(&ok_stdin(
        &tmp,
        &["vps", "edit", "x", "--password-stdin", "--json"],
        "back-to-pw",
    ));
    assert_eq!(w["target_resolved"], "x", "{w}");
    assert_eq!(w["target_source"], "argv", "{w}");
    assert_eq!(show(&tmp, "y"), y_before);
}

#[test]
#[serial]
fn edit_has_no_use_active_flag() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "x");
    ok(&tmp, &["connect", "x"]);
    let before = config_bytes(&tmp);

    let (code, stdout, _) = run_stdin(
        &tmp,
        &["vps", "edit", "--use-active", "--timeout", "9000"],
        "",
    );
    assert_eq!(code, EX_CLAP);
    assert!(stdout.is_empty(), "{stdout}");
    assert_eq!(config_bytes(&tmp), before);
}

// ── event names ─────────────────────────────────────────────────────────────

#[test]
#[serial]
fn the_event_is_vps_edited_without_a_rename_and_vps_renamed_with_one() {
    let tmp = TempDir::new().expect("tempdir");
    let Some(key) = write_openssh_key(&tmp) else {
        return;
    };
    seed(&tmp, "old");

    let edited = envelope_of(&ok(&tmp, &["vps", "edit", "old", "--key", &key, "--json"]));
    assert_eq!(edited["event"], "vps-edited", "{edited}");
    assert!(edited.get("previous_name").is_none(), "{edited}");
    assert_eq!(sorted(updated_of(&edited)), ["key_path", "password"]);

    let renamed = envelope_of(&ok_stdin(
        &tmp,
        &[
            "vps",
            "edit",
            "old",
            "--name",
            "new",
            "--password-stdin",
            "--json",
        ],
        "pw-after-rename",
    ));
    assert_eq!(renamed["event"], "vps-renamed", "{renamed}");
    assert_eq!(renamed["previous_name"], "old", "{renamed}");
    assert_eq!(renamed["name"], "new", "{renamed}");
    assert_eq!(renamed["target_resolved"], "old", "{renamed}");
    assert_eq!(renamed["target_source"], "argv", "{renamed}");
    assert_eq!(
        sorted(updated_of(&renamed)),
        ["key_path", "password"],
        "{renamed}"
    );
}

// ── key replacement and the stored passphrase ───────────────────────────────

#[test]
#[serial]
fn a_different_key_drops_the_old_passphrase_unless_a_new_one_comes_along() {
    let tmp = TempDir::new().expect("tempdir");
    let (Some(first), Some(second)) = (
        write_named_key(&tmp, "id_first"),
        write_named_key(&tmp, "id_second"),
    ) else {
        return;
    };
    seed(&tmp, "web");
    ok_stdin(
        &tmp,
        &[
            "vps",
            "edit",
            "web",
            "--key",
            &first,
            "--key-passphrase-stdin",
        ],
        "old-pass",
    );

    // Same key again: nothing changes and the passphrase survives.
    let same = envelope_of(&ok(
        &tmp,
        &["vps", "edit", "web", "--key", &first, "--json"],
    ));
    assert!(updated_of(&same).is_empty(), "{same}");
    assert_eq!(export_hosts(&tmp)["web"]["key_passphrase"], "old-pass");

    // A different key without a passphrase must not inherit the old one.
    let swapped = envelope_of(&ok(
        &tmp,
        &["vps", "edit", "web", "--key", &second, "--json"],
    ));
    assert_eq!(
        sorted(updated_of(&swapped)),
        ["key_passphrase", "key_path"],
        "{swapped}"
    );
    let web = export_hosts(&tmp)["web"].clone();
    assert_eq!(web["key_path"], second, "{web}");
    assert!(web["key_passphrase"].is_null(), "{web}");

    // A different key with its own passphrase keeps the new one.
    ok_stdin(
        &tmp,
        &[
            "vps",
            "edit",
            "web",
            "--key",
            &first,
            "--key-passphrase-stdin",
        ],
        "new-pass",
    );
    assert_eq!(export_hosts(&tmp)["web"]["key_passphrase"], "new-pass");
}

// ── --agent-socket implies --use-agent ──────────────────────────────────────

#[test]
#[serial]
fn an_agent_socket_alone_switches_a_password_host_to_the_agent() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "web");
    let sock = tmp.path().join("agent.sock");
    let sock = sock.to_str().expect("utf-8 path");

    let v = envelope_of(&ok(
        &tmp,
        &["vps", "edit", "web", "--agent-socket", sock, "--json"],
    ));
    assert_eq!(
        sorted(updated_of(&v)),
        ["agent_socket", "password", "use_agent"],
        "{v}"
    );
    let hosts = export_hosts(&tmp);
    assert_eq!(hosts["web"]["use_agent"], true, "{hosts}");
    assert_eq!(hosts["web"]["agent_socket"], sock, "{hosts}");
    assert_eq!(hosts["web"]["password"], "", "{hosts}");
}

// ── dry-run keeps the target echo and creates nothing ───────────────────────

#[test]
#[serial]
fn dry_run_plans_echo_the_target_from_argv() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "x");
    seed(&tmp, "y");
    ok(&tmp, &["connect", "y"]);
    let before = config_bytes(&tmp);

    let edit = envelope_of(&ok(
        &tmp,
        &[
            "vps",
            "edit",
            "x",
            "--timeout",
            "9000",
            "--dry-run",
            "--json",
        ],
    ));
    assert_eq!(edit["event"], "dry-run", "{edit}");
    assert_eq!(edit["target_resolved"], "x", "{edit}");
    assert_eq!(edit["target_source"], "argv", "{edit}");

    let rename = envelope_of(&ok(
        &tmp,
        &["vps", "rename", "x", "z", "--dry-run", "--json"],
    ));
    assert_eq!(rename["operation"], "vps-rename", "{rename}");
    assert_eq!(rename["target_resolved"], "x", "{rename}");
    assert_eq!(rename["target_source"], "argv", "{rename}");
    assert_eq!(config_bytes(&tmp), before, "a preview must not write");
}

#[test]
#[serial]
fn dry_run_remove_echoes_the_target_from_argv() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "x");
    ok(&tmp, &["connect", "x"]);
    seed(&tmp, "y");
    let before = config_bytes(&tmp);

    let plan = envelope_of(&ok(&tmp, &["vps", "remove", "y", "--dry-run", "--json"]));
    assert_eq!(plan["event"], "dry-run", "{plan}");
    assert_eq!(plan["operation"], "vps-remove", "{plan}");
    assert_eq!(plan["name"], "y", "{plan}");
    assert_eq!(plan["target_resolved"], "y", "{plan}");
    assert_eq!(plan["target_source"], "argv", "{plan}");
    assert_eq!(plan["host_resolved"], "y", "{plan}");
    assert_eq!(plan["host_source"], "argv", "{plan}");
    assert_eq!(config_bytes(&tmp), before, "a preview must not write");
}

#[test]
#[serial]
fn edit_rename_and_remove_on_a_missing_registry_exit_66_and_create_nothing() {
    let tmp = TempDir::new().expect("tempdir");
    let missing = tmp.path().join("never-created");
    let verbs: [&[&str]; 3] = [
        &["vps", "edit", "ghost", "--timeout", "9000"],
        &["vps", "rename", "ghost", "other"],
        &["vps", "remove", "ghost"],
    ];
    for verb in verbs {
        for dry_run in [false, true] {
            let mut args = verb.to_vec();
            if dry_run {
                args.push("--dry-run");
            }
            args.push("--json");
            let out = cmd_with_dir(&tmp, &missing)
                .args(&args)
                .output()
                .expect("spawn ssh-cli");
            assert_eq!(out.status.code(), Some(66), "{args:?}");
            assert!(out.stdout.is_empty(), "{args:?}");
            assert!(
                !missing.exists(),
                "{args:?} must not create the config directory"
            );
        }
    }
}

/// A secret-key environment variable is refused as a usage error on a write that needs
/// the key, the same class `secrets status` reports, and never as a config error.
#[test]
#[serial]
fn secrets_key_env_variable_on_a_write_is_a_usage_error_not_a_config_error() {
    for variable in ["SSH_CLI_SECRETS_KEY", "SSH_CLI_SECRETS_KEY_FILE"] {
        let tmp = TempDir::new().expect("tempdir");
        let mut c = Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
        c.env_clear();
        c.env("PATH", std::env::var_os("PATH").unwrap_or_default());
        c.env("HOME", tmp.path());
        c.env("XDG_CONFIG_HOME", tmp.path());
        c.env(variable, "x");
        let out = c
            .arg("--config-dir")
            .arg(tmp.path())
            .args([
                "vps",
                "add",
                "--name",
                "h",
                "--host",
                "h.example",
                "--user",
                "u",
                "--password-stdin",
            ])
            .write_stdin("pw")
            .output()
            .expect("spawn ssh-cli");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(EX_USAGE), "{variable}: {stderr}");
        assert!(out.stdout.is_empty(), "{variable}: stdout must stay empty");
        assert!(stderr.contains("not supported"), "{variable}: {stderr}");
    }
}

/// Reading an encrypted registry with a secret-key environment variable set is a usage
/// error, the same class `secrets status` reports, and not a complaint about the TOML.
#[test]
#[serial]
fn secrets_key_env_variable_on_a_read_is_a_usage_error_not_a_config_error() {
    let tmp = TempDir::new().expect("tempdir");
    let base = |extra: Option<(&str, &str)>| {
        let mut c = Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
        c.env_clear();
        c.env("PATH", std::env::var_os("PATH").unwrap_or_default());
        c.env("HOME", tmp.path());
        c.env("XDG_CONFIG_HOME", tmp.path());
        if let Some((k, v)) = extra {
            c.env(k, v);
        }
        c.arg("--config-dir").arg(tmp.path());
        c
    };
    let init = base(None)
        .args(["secrets", "init"])
        .output()
        .expect("spawn ssh-cli");
    assert!(init.status.success(), "secrets init failed");
    let add = base(None)
        .args([
            "vps",
            "add",
            "--name",
            "h",
            "--host",
            "h.example",
            "--user",
            "u",
            "--password-stdin",
        ])
        .write_stdin("pw")
        .output()
        .expect("spawn ssh-cli");
    assert!(add.status.success(), "vps add failed");
    for variable in ["SSH_CLI_SECRETS_KEY", "SSH_CLI_SECRETS_KEY_FILE"] {
        for verb in ["list", "show"] {
            let mut args = vec!["vps", verb];
            if verb == "show" {
                args.push("h");
            }
            let out = base(Some((variable, "x")))
                .args(&args)
                .output()
                .expect("spawn ssh-cli");
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert_eq!(
                out.status.code(),
                Some(EX_USAGE),
                "{variable} {verb}: {stderr}"
            );
            assert!(
                stderr.contains("not supported"),
                "{variable} {verb}: {stderr}"
            );
        }
    }
}
