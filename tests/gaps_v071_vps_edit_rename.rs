// SPDX-License-Identifier: MIT OR Apache-2.0
//! `vps rename`, `vps edit --name`, the eight clear flags and the bounded lock.
//!
//! # Why rename needed a native verb
//!
//! Before 0.5.6 the only way to change a host name was export, edit, import. The
//! export redacts secrets, so the round trip silently discarded the credential:
//! the operator had to retype a password the tool already had, and any host
//! authenticated by agent lost its auth entirely because the export carried no
//! `use_agent`. A name is a label on a record, not a reason to touch its secrets.
//!
//! # Why every case drives argv
//!
//! The guarantees here live at the process boundary: which exit code appears,
//! which envelope reaches stdout, whether the `active` marker moved and whether
//! the plaintext behind the encrypted blob is still readable *under the new
//! name*. Calling the functions directly would assert a library contract while
//! the CLI could still ship a different one. Every case therefore spawns the
//! binary with a fresh `--config-dir`.
//!
//! Note what is *not* asserted: the ciphertext token itself does not survive a
//! rename unchanged. Every save re-seals each secret with a fresh nonce, so the
//! token differs after any write, rename included. The property that matters is
//! that the secret is still decryptable under the new key — which is what the
//! relocation check in `secrets::deserialize_secret_in_context` would break, and
//! what the test measures by exporting with `--include-secrets`.
//!
//! # Why the marker cases matter most
//!
//! `vps rename` and `vps edit` take the subject from argv and never consult the
//! active marker. The two `*_never_retargets_from_the_marker` cases point the
//! marker at a *different* host and prove the verb still acted on the named one.
//! Without them the marker read could be reintroduced and every other test in
//! this file would still pass.

use assert_cmd::Command;
use serial_test::serial;
use tempfile::TempDir;

/// Exit code for a usage error (`sysexits.h` `EX_USAGE`).
const EX_USAGE: i32 = 64;
/// Exit code for "named thing does not exist" (`sysexits.h` `EX_NOINPUT`).
const EX_NOINPUT: i32 = 66;
/// Exit code for an unavailable dependency (`sysexits.h` `EX_UNAVAILABLE`).
const EX_UNAVAILABLE: i32 = 69;
/// Exit code `clap` returns for a malformed invocation.
const EX_CLAP: i32 = 2;

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

/// Runs argv and returns stdout as text, asserting success.
fn ok(tmp: &TempDir, args: &[&str]) -> String {
    let out = cmd(tmp).args(args).output().expect("spawn ssh-cli");
    assert!(
        out.status.success(),
        "argv {args:?} failed with {:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Runs argv, returning exit code plus both streams.
fn run(tmp: &TempDir, args: &[&str]) -> (i32, String, String) {
    let out = cmd(tmp).args(args).output().expect("spawn ssh-cli");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
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

/// Writes a real OpenSSH key fixture, or `None` when `ssh-keygen` is absent.
///
/// The registry validates key *content* when `ssh-real` is on, so a placeholder
/// byte string is rejected at `vps add` before the case under test runs.
/// This is test-only: the product never spawns `ssh-keygen` (G-PROC-02).
fn write_openssh_key(tmp: &TempDir) -> Option<std::path::PathBuf> {
    let key = tmp.path().join("id_ed25519_test");
    let status = match std::process::Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519")
        .arg("-f")
        .arg(&key)
        .arg("-N")
        .arg("")
        .arg("-q")
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
    Some(key)
}

/// Points the active marker at `name`.
fn connect(tmp: &TempDir, name: &str) {
    cmd(tmp).args(["connect", name]).assert().success();
}

/// Extracts the `sshcli-enc:v2:…` token from a TOML document.
///
/// Used to prove the secret is *sealed* rather than to compare tokens across
/// writes: every save re-seals with a fresh nonce, so the token changes on any
/// write by design.
fn ciphertext(doc: &str) -> String {
    for line in doc.lines() {
        if let Some(pos) = line.find("sshcli-enc:v2:") {
            let rest = &line[pos..];
            let end = rest
                .find(|c: char| c == '"' || c.is_whitespace())
                .unwrap_or(rest.len());
            return rest[..end].to_string();
        }
    }
    panic!("no sshcli-enc:v2 token in:\n{doc}");
}

/// Reads the sibling `active` marker, if any.
fn active_marker(tmp: &TempDir) -> Option<String> {
    std::fs::read_to_string(tmp.path().join("active"))
        .ok()
        .map(|s| s.trim().to_string())
}

/// Parses the single compact JSON object a `--json` success run writes to stdout.
fn envelope_of(stdout: &str) -> serde_json::Value {
    let line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| panic!("no JSON envelope on stdout:\n{stdout}"));
    serde_json::from_str(line).unwrap_or_else(|e| panic!("envelope is not JSON: {e}\n{line}"))
}

// ── rename and the secrets it must not touch ────────────────────────────────

#[test]
#[serial]
fn rename_preserves_encrypted_secrets() {
    let tmp = TempDir::new().expect("tempdir");
    ok(&tmp, &["secrets", "init"]);
    seed(&tmp, "antigo");
    let before = std::fs::read_to_string(tmp.path().join("config.toml")).expect("config");
    assert!(
        before.contains("sshcli-enc:v2:"),
        "the fixture must exercise an encrypted secret:\n{before}"
    );
    let blob_before = ciphertext(&before);

    ok(&tmp, &["vps", "rename", "antigo", "novo", "--json"]);

    let after = std::fs::read_to_string(tmp.path().join("config.toml")).expect("config");
    assert!(after.contains("novo"));
    assert!(
        after.contains("sshcli-enc:v2:"),
        "the secret must still be sealed, never downgraded to plaintext:\n{after}"
    );
    assert!(
        !after.contains("secret"),
        "the plaintext must never reach the file:\n{after}"
    );

    // Every save re-seals with a fresh nonce, so the token differs by design.
    // What must hold is that the secret is still readable under the new name:
    // a bound (A7) blob would fail tag verification here.
    let exported = ok(
        &tmp,
        &[
            "vps",
            "export",
            "--include-secrets",
            "--i-understand-secrets-on-stdout",
            "--json",
        ],
    );
    let v = envelope_of(&exported);
    assert_eq!(
        v["hosts"]["novo"]["password"], "secret",
        "the credential must survive the rename readable under the new key"
    );
    assert!(
        v["hosts"]["novo"]["password"] != serde_json::json!(blob_before),
        "the exported password is plaintext, never the sealed token"
    );

    let shown = envelope_of(&ok(&tmp, &["vps", "show", "novo", "--json"]));
    assert_eq!(shown["password"], "***");
}

#[test]
#[serial]
fn rename_updates_the_active_marker() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "antigo");
    connect(&tmp, "antigo");
    assert_eq!(active_marker(&tmp).as_deref(), Some("antigo"));

    let out = ok(&tmp, &["vps", "rename", "antigo", "novo", "--json"]);
    assert_eq!(envelope_of(&out)["active_rewritten"], true);
    assert_eq!(active_marker(&tmp).as_deref(), Some("novo"));
}

#[test]
#[serial]
fn rename_leaves_another_active_host_untouched() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "antigo");
    seed(&tmp, "outro");
    connect(&tmp, "outro");

    let out = ok(&tmp, &["vps", "rename", "antigo", "novo", "--json"]);
    assert_eq!(envelope_of(&out)["active_rewritten"], false);
    assert_eq!(
        active_marker(&tmp).as_deref(),
        Some("outro"),
        "renaming one host must never clear or move another host's selection"
    );
}

#[test]
#[serial]
fn rename_never_retargets_from_the_marker() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "alvo");
    seed(&tmp, "outro");
    // The marker names a host that is NOT the subject, which is the exact ambient
    // state a silent fallback would read.
    connect(&tmp, "outro");

    ok(&tmp, &["vps", "rename", "alvo", "alvo2", "--json"]);

    let shown = ok(&tmp, &["vps", "show", "outro", "--json"]);
    assert_eq!(envelope_of(&shown)["name"], "outro");
    let renamed = ok(&tmp, &["vps", "show", "alvo2", "--json"]);
    assert_eq!(envelope_of(&renamed)["name"], "alvo2");
}

#[test]
#[serial]
fn edit_never_retargets_from_the_marker() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "alvo");
    seed(&tmp, "outro");
    connect(&tmp, "outro");

    ok(
        &tmp,
        &["vps", "edit", "alvo", "--timeout", "9000", "--json"],
    );

    let alvo = envelope_of(&ok(&tmp, &["vps", "show", "alvo", "--json"]));
    assert_eq!(alvo["timeout_ms"], 9000);
    let outro = envelope_of(&ok(&tmp, &["vps", "show", "outro", "--json"]));
    assert_eq!(
        outro["timeout_ms"], 60000,
        "the marker host must be untouched; 60000 ms is the registry default"
    );
}

#[test]
#[serial]
fn rename_to_existing_name_is_vps_duplicate() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "a");
    seed(&tmp, "b");
    let (code, stdout, stderr) = run(&tmp, &["vps", "rename", "a", "b", "--json"]);
    assert_eq!(code, EX_USAGE, "{stderr}");
    assert!(
        stdout.trim().is_empty(),
        "stdout must stay empty on failure"
    );
    assert!(stderr.contains("vps_duplicate"), "{stderr}");
}

#[test]
#[serial]
fn rename_to_invalid_name_is_usage_error() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "a");
    let (code, _, stderr) = run(&tmp, &["vps", "rename", "a", "../x", "--json"]);
    assert_eq!(code, EX_USAGE, "{stderr}");
}

#[test]
#[serial]
fn rename_to_the_same_name_is_refused() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "a");
    let (code, _, stderr) = run(&tmp, &["vps", "rename", "a", "a", "--json"]);
    assert_eq!(
        code, EX_USAGE,
        "a rename that changes nothing would claim a change it never made: {stderr}"
    );
}

#[test]
#[serial]
fn rename_missing_source_is_not_found() {
    let tmp = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = run(&tmp, &["vps", "rename", "fantasma", "novo", "--json"]);
    assert_eq!(code, EX_NOINPUT, "{stderr}");
    assert!(stdout.trim().is_empty());
    assert!(stderr.contains("vps_not_found"), "{stderr}");
    assert!(
        !stderr.contains("target_resolved"),
        "the target is published only after a successful lookup: {stderr}"
    );
}

#[test]
#[serial]
fn edit_name_matches_rename() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "antigo");
    let out = ok(&tmp, &["vps", "edit", "antigo", "--name", "novo", "--json"]);
    let v = envelope_of(&out);
    // `vps edit --name` reaches the same code path as `vps rename`, so it emits
    // the same event. One operation, one event name.
    assert_eq!(v["event"], "vps-renamed");
    assert_eq!(v["previous_name"], "antigo");
    assert_eq!(v["name"], "novo");
    assert_eq!(v["updated"], serde_json::json!([]));
    let shown = envelope_of(&ok(&tmp, &["vps", "show", "novo", "--json"]));
    assert_eq!(shown["name"], "novo");
    let (code, _, _) = run(&tmp, &["vps", "show", "antigo", "--json"]);
    assert_eq!(code, EX_NOINPUT, "the old name must stop resolving");
}

#[test]
#[serial]
fn edit_keeps_the_vps_edited_event_when_it_only_changes_fields() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let v = envelope_of(&ok(
        &tmp,
        &["vps", "edit", "h", "--timeout", "9000", "--json"],
    ));
    assert_eq!(v["event"], "vps-edited");
    assert_eq!(v["updated"], serde_json::json!(["timeout_ms"]));
    assert_eq!(v["name"], "h");
    assert!(
        v.get("previous_name").is_none(),
        "no rename happened, so there is no previous name to report: {v}"
    );
}

#[test]
#[serial]
fn rename_envelope_reports_argv_provenance() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "antigo");
    let v = envelope_of(&ok(&tmp, &["vps", "rename", "antigo", "novo", "--json"]));
    assert_eq!(v["target_source"], "argv");
    assert_eq!(v["target_resolved"], "antigo");
    assert_eq!(v["host_source"], "argv");
    assert_eq!(v["host_resolved"], "antigo");
}

#[test]
#[serial]
fn renamed_host_is_not_reachable_by_the_old_name() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "antigo");
    ok(&tmp, &["vps", "rename", "antigo", "novo", "--json"]);
    let (code, _, stderr) = run(&tmp, &["exec", "antigo", "true", "--json"]);
    assert_eq!(
        code, EX_NOINPUT,
        "no lookup path may survive the rename: {stderr}"
    );
}

#[test]
#[serial]
fn rename_arity_is_fixed() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "antigo");
    let (one, _, _) = run(&tmp, &["vps", "rename", "antigo"]);
    assert_eq!(
        one, EX_CLAP,
        "one positional must never be read as a rename"
    );
    let (three, _, _) = run(&tmp, &["vps", "rename", "antigo", "novo", "extra"]);
    assert_eq!(three, EX_CLAP, "a third positional must never be ignored");
}

// ── the eight clear flags ───────────────────────────────────────────────────

#[test]
#[serial]
fn each_clear_flag_empties_its_field() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "h",
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--password-stdin",
            "--sudo-password",
            "s1",
            "--su-password",
            "s2",
            "--tag",
            "prod",
            "--tls",
            "--tls-sni",
            "sni.example",
            "--tls-client-cert",
            "/etc/certs/client.pem",
            "--tls-client-key",
            "/etc/certs/client.key",
        ])
        .write_stdin("pw")
        .assert()
        .success();

    ok(
        &tmp,
        &[
            "vps",
            "edit",
            "h",
            "--clear-tags",
            "--clear-sudo-password",
            "--clear-su-password",
            "--no-tls-sni",
            "--no-tls-client-cert",
            "--no-tls-client-key",
            "--json",
        ],
    );

    let v = envelope_of(&ok(&tmp, &["vps", "show", "h", "--json"]));
    assert!(v["tags"].is_null() || v["tags"].as_array().is_some_and(Vec::is_empty));
    assert!(v["sudo_password"].is_null(), "{v}");
    assert!(v["su_password"].is_null(), "{v}");
    assert!(v["tls_sni"].is_null(), "{v}");
    assert!(v["tls_client_cert"].is_null(), "{v}");
    assert!(v["tls_client_key"].is_null(), "{v}");
}

#[test]
#[serial]
fn clear_key_without_alternate_auth_fails_closed() {
    let tmp = TempDir::new().expect("tempdir");
    let Some(key) = write_openssh_key(&tmp) else {
        eprintln!("skip: ssh-keygen not on PATH; the key path is content-validated");
        return;
    };
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "h",
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--key",
        ])
        .arg(&key)
        .assert()
        .success();

    let (code, _, stderr) = run(&tmp, &["vps", "edit", "h", "--no-key", "--json"]);
    assert_eq!(
        code, EX_USAGE,
        "a host with no auth method at all must be refused here: {stderr}"
    );
}

#[test]
#[serial]
fn clear_agent_socket_on_unix_agent_host_fails_closed() {
    let tmp = TempDir::new().expect("tempdir");
    let sock = tmp.path().join("agent.sock");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "h",
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--use-agent",
            "--agent-socket",
        ])
        .arg(&sock)
        .assert()
        .success();

    let (code, _, stderr) = run(&tmp, &["vps", "edit", "h", "--no-agent-socket", "--json"]);
    if cfg!(windows) {
        // Windows does provide a default agent endpoint, so clearing is allowed.
        assert_eq!(code, 0, "{stderr}");
    } else {
        assert_eq!(
            code, EX_USAGE,
            "on Unix an agent host with no socket stops authenticating: {stderr}"
        );
    }
}

#[test]
#[serial]
fn clear_one_mtls_half_is_rejected() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "h",
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--password-stdin",
            "--tls",
            "--tls-client-cert",
            "/etc/certs/client.pem",
            "--tls-client-key",
            "/etc/certs/client.key",
        ])
        .write_stdin("pw")
        .assert()
        .success();

    let (code, _, stderr) = run(
        &tmp,
        &["vps", "edit", "h", "--no-tls-client-cert", "--json"],
    );
    assert_eq!(code, EX_USAGE, "mTLS needs cert and key together: {stderr}");
}

#[test]
#[serial]
fn edit_tags_append_dedupe_and_clear() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "h",
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--password-stdin",
            "--tag",
            "a",
            "--tag",
            "b",
        ])
        .write_stdin("pw")
        .assert()
        .success();

    ok(
        &tmp,
        &["vps", "edit", "h", "--tag", "b", "--tag", "c", "--json"],
    );
    let v = envelope_of(&ok(&tmp, &["vps", "show", "h", "--json"]));
    let tags: Vec<String> = v["tags"]
        .as_array()
        .expect("tags")
        .iter()
        .map(|t| t.as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(tags, vec!["a", "b", "c"], "append must dedupe in order");

    ok(
        &tmp,
        &["vps", "edit", "h", "--clear-tags", "--tag", "z", "--json"],
    );
    let v = envelope_of(&ok(&tmp, &["vps", "show", "h", "--json"]));
    let tags: Vec<String> = v["tags"]
        .as_array()
        .expect("tags")
        .iter()
        .map(|t| t.as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(tags, vec!["z"], "clear runs before append");
}

#[test]
#[serial]
fn edit_without_any_field_is_refused() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let (code, stdout, stderr) = run(&tmp, &["vps", "edit", "h", "--json"]);
    assert_eq!(code, EX_USAGE, "{stderr}");
    assert!(stdout.trim().is_empty());
}

// ── dry-run ────────────────────────────────────────────────────────────────

#[test]
#[serial]
fn dry_run_previews_edit_without_writing() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");
    let before = std::fs::read_to_string(tmp.path().join("config.toml")).expect("config");

    let v = envelope_of(&ok(
        &tmp,
        &[
            "vps",
            "edit",
            "h",
            "--timeout",
            "9000",
            "--dry-run",
            "--json",
        ],
    ));
    assert_eq!(v["event"], "dry-run");
    assert_eq!(v["operation"], "vps-edit");
    assert_eq!(v["dry_run"], true);
    assert_eq!(v["executed"], false);
    assert_eq!(v["updated"][0], "timeout_ms");

    let after = std::fs::read_to_string(tmp.path().join("config.toml")).expect("config");
    assert_eq!(
        before, after,
        "a preview must leave the file byte-identical"
    );
}

#[test]
#[serial]
fn dry_run_previews_rename_without_writing() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "antigo");
    connect(&tmp, "antigo");
    let before = std::fs::read_to_string(tmp.path().join("config.toml")).expect("config");

    let v = envelope_of(&ok(
        &tmp,
        &["vps", "rename", "antigo", "novo", "--dry-run", "--json"],
    ));
    assert_eq!(v["operation"], "vps-rename");
    assert_eq!(v["previous_name"], "antigo");
    assert_eq!(v["name"], "novo");
    assert_eq!(v["active_rewritten"], true);
    assert_eq!(v["executed"], false);

    let after = std::fs::read_to_string(tmp.path().join("config.toml")).expect("config");
    assert_eq!(before, after);
    assert_eq!(
        active_marker(&tmp).as_deref(),
        Some("antigo"),
        "the preview must not rewrite the marker either"
    );
}

#[test]
#[serial]
fn dry_run_on_missing_host_still_exits_66() {
    let tmp = TempDir::new().expect("tempdir");
    let (code, _, stderr) = run(
        &tmp,
        &["vps", "rename", "fantasma", "novo", "--dry-run", "--json"],
    );
    assert_eq!(
        code, EX_NOINPUT,
        "the precondition outranks the preview: {stderr}"
    );
}

// ── export, import and agent auth ──────────────────────────────────────────

#[test]
#[serial]
fn agent_auth_survives_export_import_round_trip() {
    let tmp = TempDir::new().expect("tempdir");
    let other = TempDir::new().expect("tempdir");
    let sock = tmp.path().join("agent.sock");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "agente",
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--use-agent",
            "--agent-socket",
        ])
        .arg(&sock)
        .assert()
        .success();

    let exported = ok(&tmp, &["vps", "export", "--json"]);
    let v = envelope_of(&exported);
    assert_eq!(v["hosts"]["agente"]["use_agent"], true);
    assert_eq!(
        v["hosts"]["agente"]["agent_socket"],
        sock.to_string_lossy().as_ref()
    );

    let file = tmp.path().join("export.json");
    std::fs::write(&file, exported.as_bytes()).expect("write export");
    // `vps import` takes the source as `--file`, not as a positional.
    ok(
        &other,
        &[
            "vps",
            "import",
            "--file",
            file.to_string_lossy().as_ref(),
            "--json",
        ],
    );

    let shown = envelope_of(&ok(&other, &["vps", "show", "agente", "--json"]));
    assert_eq!(
        shown["use_agent"], true,
        "an agent host must not lose its auth through export and import"
    );
    assert_eq!(shown["agent_socket"], sock.to_string_lossy().as_ref());
}

// ── bounded lock ───────────────────────────────────────────────────────────

#[test]
#[serial]
fn config_lock_times_out_with_unavailable() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "h");

    // The registry lives at `<dir>/config.toml`, so its sibling lock is
    // `<dir>/config.toml.lock` (`with_extension` would not produce this name).
    let lock_path = tmp.path().join("config.toml.lock");
    let holder = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .expect("lock file");
    fs2::FileExt::lock_exclusive(&holder).expect("hold lock");

    let (code, _, stderr) = run(
        &tmp,
        &[
            "vps",
            "edit",
            "h",
            "--timeout",
            "9000",
            "--lock-wait-ms",
            "60",
            "--json",
        ],
    );
    fs2::FileExt::unlock(&holder).expect("release lock");

    assert_eq!(
        code, EX_UNAVAILABLE,
        "contention must fail fast and retryably, never hang: {stderr}"
    );
    assert!(stderr.contains("unavailable"), "{stderr}");
}

#[test]
#[serial]
fn discovery_publishes_the_new_flags() {
    let tmp = TempDir::new().expect("tempdir");
    let v = envelope_of(&ok(&tmp, &["commands", "--json"]));
    let tree = &v["tree"];
    let vps = tree["subcommands"]
        .as_array()
        .expect("subcommands")
        .iter()
        .find(|c| c["name"] == "vps")
        .expect("vps node");
    let rename = vps["subcommands"]
        .as_array()
        .expect("vps subcommands")
        .iter()
        .find(|c| c["name"] == "rename")
        .expect("vps rename must appear in discovery");
    let positionals: Vec<&serde_json::Value> = rename["args"]
        .as_array()
        .expect("args")
        .iter()
        .filter(|a| a["long"].is_null() && a["id"] != "help")
        .collect();
    assert_eq!(positionals.len(), 2, "two named positionals, never a Vec");
    assert!(
        positionals.iter().all(|a| a["required"] == true),
        "both positionals are required: {rename}"
    );

    let edit = vps["subcommands"]
        .as_array()
        .expect("vps subcommands")
        .iter()
        .find(|c| c["name"] == "edit")
        .expect("vps edit");
    let longs: Vec<&str> = edit["args"]
        .as_array()
        .expect("args")
        .iter()
        .filter_map(|a| a["long"].as_str())
        .collect();
    for flag in ["no-key", "clear-tags", "no-agent-socket", "name"] {
        assert!(
            longs.contains(&flag),
            "flag --{flag} must be discoverable, not only present in --help: {longs:?}"
        );
    }
}
