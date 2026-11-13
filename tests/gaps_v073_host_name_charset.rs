// SPDX-License-Identifier: MIT OR Apache-2.0
//! Host names: strict at creation, lenient at lookup, never silent.
//!
//! # Why two strictness levels
//!
//! `VpsName` used to reuse the file-name validator. That let a name carry ESC,
//! BEL, a right-to-left override, a zero-width space, a Cyrillic homoglyph or
//! shell metacharacters into the registry, where it was later printed raw and
//! pasted into shells. A name being *created* is now limited to ASCII
//! `[A-Za-z0-9._-]` starting with a letter or digit. A name already on disk, or
//! typed to look a host up, keeps the lenient rule so an older `config.toml`
//! never locks the operator out.
//!
//! Every case drives argv with a fresh `--config-dir`, because the guarantees
//! (exit code, empty stdout, byte-identical config) live at the process
//! boundary.

use assert_cmd::Command;
use serial_test::serial;
use tempfile::TempDir;

/// Exit code for a usage error (`sysexits.h` `EX_USAGE`).
const EX_USAGE: i32 = 64;
/// Longest accepted name, in bytes.
const MAX_NAME_BYTES: usize = 255;

/// Names that must never be accepted for a new host.
fn hostile_names() -> Vec<String> {
    vec![
        "bad\u{1b}[31m".into(),
        "bad\u{7}".into(),
        "bad\u{202e}gnp".into(),
        "bad\u{200b}name".into(),
        "bad\u{ad}name".into(),
        "bad\u{61c}name".into(),
        "bad\u{180e}name".into(),
        "bad\u{206a}name".into(),
        "bad\u{e0041}name".into(),
        "h\u{0430}st".into(),
        "a b".into(),
        "a;b".into(),
        "a$b".into(),
        "a`b".into(),
        "a&b".into(),
        "a'b".into(),
        "a(b".into(),
        "a)b".into(),
        "a!b".into(),
        "a#b".into(),
        ".hidden".into(),
        "-x".into(),
        "x".repeat(MAX_NAME_BYTES + 1),
        "CON".into(),
        "a..b".into(),
        "a.".into(),
        "servidor-produção".into(),
    ]
}

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

/// Runs argv, returning exit code plus both streams.
fn run(tmp: &TempDir, args: &[&str]) -> (i32, String, String) {
    let out = cmd(tmp).args(args).output().expect("spawn ssh-cli");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// `vps add` with the name passed as `--name=<name>` so a leading dash is data.
fn add(tmp: &TempDir, name: &str) -> (i32, String, String) {
    let out = cmd(tmp)
        .args(["vps", "add"])
        .arg(format!("--name={name}"))
        .args(["--host", "203.0.113.10", "--user", "u", "--password-stdin"])
        .write_stdin("secret")
        .output()
        .expect("spawn ssh-cli");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn seed(tmp: &TempDir, name: &str) {
    let (code, _, stderr) = add(tmp, name);
    assert_eq!(code, 0, "seed {name:?} failed: {stderr}");
}

/// Bytes of `config.toml`, or `None` when it does not exist yet.
fn config_bytes(tmp: &TempDir) -> Option<Vec<u8>> {
    std::fs::read(tmp.path().join("config.toml")).ok()
}

/// Writes a `config.toml` whose only host has the given table key and name.
///
/// `key` and `name` are inserted as TOML basic strings, so they must not hold
/// raw control characters; use `\\u001b`-style escapes for those.
fn write_legacy_config(tmp: &TempDir, key: &str, name: &str) {
    let doc = format!(
        "schema_version = 3\n\n[hosts.\"{key}\"]\nname = \"{name}\"\nhost = \"203.0.113.10\"\n\
         port = 22\nusername = \"u\"\npassword = \"secret\"\nuse_agent = false\n\
         timeout_ms = 60000\nmax_command_chars = 1000\nmax_output_chars = 100000\n\
         disable_sudo = false\nschema_version = 3\n\
         added_at = \"2026-10-19T19:48:55+00:00\"\ntags = []\ntls = false\n"
    );
    std::fs::write(tmp.path().join("config.toml"), doc).expect("write config");
}

#[test]
#[serial]
fn add_refuses_hostile_names_and_leaves_no_trace() {
    for name in hostile_names() {
        let tmp = TempDir::new().expect("tempdir");
        let (code, stdout, stderr) = add(&tmp, &name);
        let shown = name.escape_debug().to_string();
        assert_eq!(code, EX_USAGE, "{shown}: {stderr}");
        assert!(
            stdout.is_empty(),
            "{shown}: stdout must stay empty: {stdout}"
        );
        assert!(
            config_bytes(&tmp).is_none(),
            "{shown}: a refused add must not create config.toml"
        );
    }
}

#[test]
#[serial]
fn add_accepts_plain_host_names() {
    for name in [
        "ssh-danilo",
        "ssh-digitalocean01",
        "a.b_c-1",
        "host.example.com",
        "9lives",
    ] {
        let tmp = TempDir::new().expect("tempdir");
        let (code, _, stderr) = add(&tmp, name);
        assert_eq!(code, 0, "{name}: {stderr}");
    }
    let tmp = TempDir::new().expect("tempdir");
    let (code, _, stderr) = add(&tmp, &"x".repeat(MAX_NAME_BYTES));
    assert_eq!(
        code, 0,
        "a name of exactly {MAX_NAME_BYTES} bytes is valid: {stderr}"
    );
}

#[test]
#[serial]
fn rename_refuses_hostile_destinations_with_config_untouched() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "orig");
    let before = config_bytes(&tmp);
    for name in hostile_names() {
        let (code, stdout, stderr) = run(&tmp, &["vps", "rename", "orig", "--", &name]);
        let shown = name.escape_debug().to_string();
        assert_eq!(code, EX_USAGE, "{shown}: {stderr}");
        assert!(
            stdout.is_empty(),
            "{shown}: stdout must stay empty: {stdout}"
        );
        assert_eq!(config_bytes(&tmp), before, "{shown}: config.toml changed");
    }
}

#[test]
#[serial]
fn rename_accepts_a_plain_destination() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "orig");
    let (code, _, stderr) = run(&tmp, &["vps", "rename", "orig", "host.example.com"]);
    assert_eq!(code, 0, "{stderr}");
}

#[test]
#[serial]
fn rename_with_escape_in_destination_is_usage_and_publishes_no_target() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "x");
    let before = config_bytes(&tmp);
    let (code, stdout, stderr) = run(&tmp, &["vps", "rename", "x", "bad\u{1b}", "--json"]);
    assert_eq!(code, EX_USAGE, "{stderr}");
    assert!(stdout.trim().is_empty(), "{stdout}");
    assert!(
        !stderr.contains("target_resolved"),
        "a rejected destination must not publish a resolved target: {stderr}"
    );
    assert_eq!(config_bytes(&tmp), before, "config.toml changed");
}

#[test]
#[serial]
fn edit_name_refuses_hostile_destinations() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "orig");
    let before = config_bytes(&tmp);
    let (code, _, stderr) = run(&tmp, &["vps", "edit", "orig", "--name=bad name"]);
    assert_eq!(code, EX_USAGE, "{stderr}");
    assert_eq!(config_bytes(&tmp), before, "config.toml changed");
}

#[test]
#[serial]
fn legacy_names_with_space_or_accents_still_load_and_resolve() {
    let tmp = TempDir::new().expect("tempdir");
    write_legacy_config(&tmp, "servidor produção", "servidor produção");

    let (code, stdout, stderr) = run(&tmp, &["vps", "list", "--json"]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("servidor produção"), "{stdout}");

    let (code, stdout, stderr) = run(&tmp, &["vps", "show", "servidor produção", "--json"]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("servidor produção"), "{stdout}");

    let (code, _, stderr) = run(&tmp, &["vps", "show", "servidor produção"]);
    assert_eq!(
        code, 0,
        "lookup of a legacy name must not need the strict charset: {stderr}"
    );
}

#[test]
#[serial]
fn legacy_name_can_be_renamed_to_a_strict_one() {
    let tmp = TempDir::new().expect("tempdir");
    write_legacy_config(&tmp, "servidor produção", "servidor produção");
    let (code, _, stderr) = run(
        &tmp,
        &["vps", "rename", "servidor produção", "servidor-prod"],
    );
    assert_eq!(code, 0, "{stderr}");
    let (code, _, stderr) = run(&tmp, &["vps", "show", "servidor-prod"]);
    assert_eq!(code, 0, "{stderr}");
}

/// Config key and name as TOML escapes: ESC, a right-to-left override and a zero-width space.
const LEGACY_HOSTILE_ESCAPED: &str = "a\\u001b\\u202e\\u200bb";
/// The same name as the real characters, the way argv carries it.
const LEGACY_HOSTILE: &str = "a\u{1b}\u{202e}\u{200b}b";

/// Asserts that no raw terminal-attack character is in `text`.
fn assert_no_raw_controls(label: &str, text: &str) {
    assert!(
        !text.contains(['\u{1b}', '\u{7}', '\u{202e}', '\u{200b}']),
        "{label}: raw control or bidi character in {text:?}"
    );
}

#[test]
#[serial]
fn legacy_name_with_control_characters_still_loads_and_prints_escaped() {
    // Decision: an older config.toml must never lock the CLI out. The stored name
    // loads as is; every text surface escapes it, and the lookup by argv (which
    // would have to type the control characters) stays available for repair verbs.
    let tmp = TempDir::new().expect("tempdir");
    write_legacy_config(&tmp, LEGACY_HOSTILE_ESCAPED, LEGACY_HOSTILE_ESCAPED);
    for args in [
        vec!["--output-format", "text", "vps", "list"],
        vec!["--output-format", "text", "vps", "show", LEGACY_HOSTILE],
    ] {
        let (code, stdout, stderr) = run(&tmp, &args);
        assert_eq!(code, 0, "{args:?}: {stderr}");
        assert_no_raw_controls("stdout", &stdout);
        assert_no_raw_controls("stderr", &stderr);
        assert!(
            stdout.contains("a\\u{1b}\\u{202e}\\u{200b}b"),
            "{args:?}: escaped form missing: {stdout}"
        );
    }
    let (code, _, stderr) = run(&tmp, &["vps", "list", "--json"]);
    assert_eq!(code, 0, "{stderr}");
}

#[test]
#[serial]
fn doctor_lists_a_legacy_name_with_control_characters_escaped() {
    let tmp = TempDir::new().expect("tempdir");
    write_legacy_config(&tmp, LEGACY_HOSTILE_ESCAPED, LEGACY_HOSTILE_ESCAPED);
    let (code, stdout, stderr) = run(&tmp, &["doctor", "--json"]);
    assert_eq!(code, 0, "{stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("one JSON document");
    assert_eq!(
        v["local"]["invalid_host_names"],
        serde_json::json!(["a\\u{1b}\\u{202e}\\u{200b}b"]),
        "{v}"
    );
    let (code, stdout, stderr) = run(&tmp, &["--output-format", "text", "doctor"]);
    assert_eq!(code, 0, "{stderr}");
    assert_no_raw_controls("doctor text", &stdout);
    assert!(stdout.contains("a\\u{1b}\\u{202e}\\u{200b}b"), "{stdout}");
}

#[test]
#[serial]
fn rename_repairs_a_legacy_name_with_control_characters() {
    let tmp = TempDir::new().expect("tempdir");
    write_legacy_config(&tmp, LEGACY_HOSTILE_ESCAPED, LEGACY_HOSTILE_ESCAPED);
    let (code, _, stderr) = run(&tmp, &["vps", "rename", LEGACY_HOSTILE, "fixed"]);
    assert_eq!(code, 0, "{stderr}");
    let (code, stdout, stderr) = run(&tmp, &["doctor", "--json"]);
    assert_eq!(code, 0, "{stderr}");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("one JSON document");
    assert_eq!(
        v["local"]["invalid_host_names"],
        serde_json::json!([]),
        "{v}"
    );
    let (code, _, stderr) = run(&tmp, &["vps", "show", "fixed"]);
    assert_eq!(code, 0, "{stderr}");
}

#[test]
#[serial]
fn lookup_that_becomes_a_target_refuses_control_characters_with_exit_64() {
    // `connect` selects the host, as `exec` does, so both refuse the typed name.
    // A stored-name bad host is repaired with `vps rename`, which only reads it.
    let tmp = TempDir::new().expect("tempdir");
    write_legacy_config(&tmp, LEGACY_HOSTILE_ESCAPED, LEGACY_HOSTILE_ESCAPED);
    for args in [
        vec!["connect", LEGACY_HOSTILE],
        vec!["exec", LEGACY_HOSTILE, "true"],
        vec!["health-check", LEGACY_HOSTILE],
    ] {
        let (code, stdout, stderr) = run(&tmp, &args);
        assert_eq!(code, EX_USAGE, "{args:?}: {stderr}");
        assert!(stdout.is_empty(), "{args:?}: {stdout}");
    }
    assert!(
        !tmp.path().join("active").exists(),
        "a refused connect must not write the active marker"
    );
}

#[test]
#[serial]
fn lookup_with_a_hostile_name_is_not_found_not_a_crash() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "orig");
    let (code, stdout, stderr) = run(&tmp, &["vps", "show", "bad\u{1b}[31m", "--json"]);
    assert_eq!(code, 66, "{stderr}");
    assert!(stdout.trim().is_empty(), "{stdout}");
}

/// Names an import must refuse, as they appear in the file.
const MIXED_IMPORT_BAD: [&str; 4] = ["bad\u{1b}name", "ação", "a b", "-x"];

fn mixed_import_json() -> String {
    let record = |name: &str| {
        serde_json::json!({
            "name": name, "host": "203.0.113.20", "port": 22, "username": "u",
            "password": "p"
        })
    };
    let mut hosts = serde_json::Map::new();
    for name in std::iter::once("ok1").chain(MIXED_IMPORT_BAD) {
        hosts.insert(name.to_owned(), record(name));
    }
    serde_json::json!({ "schema_version": 3, "hosts": hosts }).to_string()
}

fn mixed_import_toml() -> String {
    let mut doc = String::from("schema_version = 3\n");
    for name in std::iter::once("ok1").chain(MIXED_IMPORT_BAD) {
        let key: String = name
            .chars()
            .map(|c| {
                if c.is_control() {
                    format!("\\u{:04x}", c as u32)
                } else {
                    c.to_string()
                }
            })
            .collect();
        doc.push_str(&format!(
            "[hosts.\"{key}\"]\nname = \"{key}\"\nhost = \"203.0.113.20\"\nport = 22\n\
             username = \"u\"\npassword = \"p\"\n"
        ));
    }
    doc
}

#[test]
#[serial]
fn import_lists_every_invalid_name_at_once_and_imports_nothing() {
    let tmp = TempDir::new().expect("tempdir");
    seed(&tmp, "keep");
    let before = config_bytes(&tmp);

    for (label, body) in [
        ("import.json", mixed_import_json()),
        ("import.toml", mixed_import_toml()),
    ] {
        let file = tmp.path().join(label);
        std::fs::write(&file, body).expect("write import");
        let file = file.to_string_lossy().into_owned();
        for dry in [true, false] {
            let mut args = vec!["vps", "import", "--file", &file, "--json"];
            if dry {
                args.push("--dry-run");
            }
            let (code, stdout, stderr) = run(&tmp, &args);
            let ctx = format!("{label} dry_run={dry}: {stderr}");
            assert_eq!(code, EX_USAGE, "{ctx}");
            assert!(stdout.trim().is_empty(), "{ctx}");
            let envelope: serde_json::Value =
                serde_json::from_str(stderr.trim()).expect("one JSON envelope on stderr");
            let message = envelope["message"].as_str().expect("message");
            assert!(message.contains("invalid VPS names in import (4)"), "{ctx}");
            for escaped in ["'bad\\u{1b}name'", "'ação'", "'a b'", "'-x'"] {
                assert!(message.contains(escaped), "missing {escaped}: {ctx}");
            }
            assert!(
                !message.contains("'ok1'"),
                "the valid name is not reported: {ctx}"
            );
            assert!(
                !message.contains('\u{1b}'),
                "raw ESC in the message: {ctx:?}"
            );
            assert_eq!(config_bytes(&tmp), before, "{ctx}: config.toml changed");
        }
    }
}

/// Writes the two PEM files `tls mtls import` reads, returning their paths.
fn write_pem_fixtures(tmp: &TempDir) -> (String, String) {
    let cert = tmp.path().join("client.pem");
    let key = tmp.path().join("client.key");
    std::fs::write(
        &cert,
        "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n",
    )
    .expect("write cert");
    std::fs::write(
        &key,
        "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n",
    )
    .expect("write key");
    (
        cert.to_string_lossy().into_owned(),
        key.to_string_lossy().into_owned(),
    )
}

#[test]
#[serial]
fn mtls_import_refuses_control_and_bidi_names_and_creates_no_directory() {
    let tmp = TempDir::new().expect("tempdir");
    let (cert, key) = write_pem_fixtures(&tmp);
    for name in [
        "id\u{1b}x",
        "id\u{7}x",
        "id\u{7f}x",
        "id\u{200b}x",
        "id\u{202e}x",
        "id\u{ad}x",
        "id\u{61c}x",
        "id\u{180e}x",
        "id\u{206a}x",
        "id\u{e0041}x",
    ] {
        let (code, stdout, stderr) = run(
            &tmp,
            &[
                "tls",
                "mtls",
                "import",
                &format!("--name={name}"),
                "--cert",
                &cert,
                "--key",
                &key,
            ],
        );
        let shown = name.escape_debug().to_string();
        assert_eq!(code, EX_USAGE, "{shown}: {stderr}");
        assert!(stdout.is_empty(), "{shown}: {stdout}");
        assert!(
            !tmp.path().join("tls").join("mtls").exists(),
            "{shown}: a refused name must not create tls/mtls"
        );
    }
}

#[test]
#[serial]
fn mtls_import_still_accepts_an_accented_identity_name() {
    let tmp = TempDir::new().expect("tempdir");
    let (cert, key) = write_pem_fixtures(&tmp);
    let (code, _, stderr) = run(
        &tmp,
        &[
            "tls",
            "mtls",
            "import",
            "--name=serviço-1",
            "--cert",
            &cert,
            "--key",
            &key,
        ],
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(tmp
        .path()
        .join("tls")
        .join("mtls")
        .join("serviço-1")
        .is_dir());
}
