// SPDX-License-Identifier: MIT OR Apache-2.0
//! Text output never carries a raw terminal control sequence.
//!
//! # The attack
//!
//! A host name, host, user or path that holds ESC, BEL or a right-to-left
//! override can clear the screen, retitle the terminal or reorder what the
//! operator reads, as soon as a text-mode command prints it. The JSON path was
//! already safe because `serde_json` escapes control characters; the text path
//! printed them raw.
//!
//! # What is asserted
//!
//! In `--output-format text` neither stdout nor stderr may hold byte 0x1b, byte
//! 0x07 or U+202E, and the escaped form `\u{1b}` must be there so the operator
//! still sees *that* something odd was typed. In `--json` the payload parses and
//! keeps the original value, because JSON consumers need the real string.
//!
//! The remote stdout and stderr of `exec` are deliberately not escaped; they are
//! the payload, and that is documented on `print_execution_output`.

use assert_cmd::Command;
use serial_test::serial;
use tempfile::TempDir;

/// Exit code for "named thing does not exist" (`sysexits.h` `EX_NOINPUT`).
const EX_NOINPUT: i32 = 66;
/// Exit code for a usage error (`sysexits.h` `EX_USAGE`).
const EX_USAGE: i32 = 64;

/// A name carrying ESC, BEL and a right-to-left override.
const HOSTILE: &str = "a\u{1b}[31m\u{7}\u{202e}b";

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

/// Runs argv in text mode, returning exit code plus both streams as bytes.
fn run_text(tmp: &TempDir, args: &[&str]) -> (i32, Vec<u8>, Vec<u8>) {
    let out = cmd(tmp)
        .args(["--output-format", "text"])
        .args(args)
        .output()
        .expect("spawn ssh-cli");
    (out.status.code().unwrap_or(-1), out.stdout, out.stderr)
}

/// Asserts that no byte of a terminal attack survived in `bytes`.
fn assert_no_raw_attack(label: &str, bytes: &[u8]) {
    assert!(
        !bytes.contains(&0x1b),
        "{label}: raw ESC reached the terminal"
    );
    assert!(
        !bytes.contains(&0x07),
        "{label}: raw BEL reached the terminal"
    );
    let text = String::from_utf8_lossy(bytes);
    assert!(
        !text.contains('\u{202e}'),
        "{label}: raw U+202E reached the terminal"
    );
}

/// Writes a `config.toml` with one host; `extra` is appended inside the table.
///
/// Values go in as TOML basic strings, so control characters are written as
/// `\uXXXX` escapes (the file itself never holds a raw control byte).
fn write_config(tmp: &TempDir, host: &str, extra: &str) {
    let doc = format!(
        "schema_version = 3\n\n[hosts.legacy]\nname = \"legacy\"\nhost = \"{host}\"\nport = 22\n\
         username = \"u\"\npassword = \"secret\"\nuse_agent = false\ntimeout_ms = 60000\n\
         max_command_chars = 1000\nmax_output_chars = 100000\ndisable_sudo = false\n\
         schema_version = 3\nadded_at = \"2026-10-19T19:48:55+00:00\"\ntags = []\ntls = false\n\
         {extra}"
    );
    std::fs::write(tmp.path().join("config.toml"), doc).expect("write config");
}

#[test]
#[serial]
fn not_found_errors_escape_the_typed_name() {
    let tmp = TempDir::new().expect("tempdir");
    for args in [
        vec!["vps", "show", HOSTILE],
        vec!["vps", "remove", HOSTILE],
        vec!["vps", "rename", HOSTILE, "ok-name"],
        vec!["vps", "edit", HOSTILE, "--host", "h"],
    ] {
        let label = args[1];
        let (code, stdout, stderr) = run_text(&tmp, &args);
        assert_eq!(
            code,
            EX_NOINPUT,
            "{label}: {}",
            String::from_utf8_lossy(&stderr)
        );
        assert_no_raw_attack(label, &stdout);
        assert_no_raw_attack(label, &stderr);
        let text = String::from_utf8_lossy(&stderr);
        assert!(
            text.contains("\\u{1b}"),
            "{label}: escaped form missing: {text}"
        );
        assert!(
            text.contains("\\u{202e}"),
            "{label}: escaped form missing: {text}"
        );
    }
}

#[test]
#[serial]
fn invalid_destination_errors_escape_the_typed_name() {
    let tmp = TempDir::new().expect("tempdir");
    let (code, _, _) = run_text(
        &tmp,
        &[
            "vps",
            "add",
            "--name",
            "orig",
            "--host",
            "203.0.113.10",
            "--user",
            "u",
            "--use-agent",
        ],
    );
    assert_eq!(code, 0, "seed failed");
    let (code, stdout, stderr) = run_text(&tmp, &["vps", "rename", "orig", "--", HOSTILE]);
    assert_eq!(code, EX_USAGE, "{}", String::from_utf8_lossy(&stderr));
    assert_no_raw_attack("rename", &stdout);
    assert_no_raw_attack("rename", &stderr);
    assert!(String::from_utf8_lossy(&stderr).contains("\\u{1b}"));
}

#[test]
#[serial]
fn lookup_by_exec_escapes_the_typed_name() {
    let tmp = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = run_text(&tmp, &["exec", HOSTILE, "true"]);
    assert_ne!(code, 0);
    assert_no_raw_attack("exec", &stdout);
    assert_no_raw_attack("exec", &stderr);
}

#[test]
#[serial]
fn list_and_show_escape_stored_fields_in_text_mode() {
    let tmp = TempDir::new().expect("tempdir");
    // Host and TLS fields accept U+202E and ESC from an older file; the text
    // renderer is the last line of defence.
    write_config(
        &tmp,
        "evil\\u202ehost",
        "tls_sni = \"sni\\u001b[2Jx\"\nagent_socket = \"sock\\u0007et\"\n",
    );
    for args in [vec!["vps", "list"], vec!["vps", "show", "legacy"]] {
        let label = args[1];
        let (code, stdout, stderr) = run_text(&tmp, &args);
        assert_eq!(code, 0, "{label}: {}", String::from_utf8_lossy(&stderr));
        assert_no_raw_attack(label, &stdout);
        assert_no_raw_attack(label, &stderr);
        let text = String::from_utf8_lossy(&stdout);
        assert!(text.contains("evil\\u{202e}host"), "{label}: {text}");
    }
    let (_, stdout, _) = run_text(&tmp, &["vps", "show", "legacy"]);
    let text = String::from_utf8_lossy(&stdout);
    assert!(text.contains("sni\\u{1b}[2Jx"), "{text}");
}

/// A key-only host has no password, so the text table must not print a masked value for it.
#[test]
#[serial]
fn list_in_text_mode_does_not_fake_a_password_for_a_key_only_host() {
    let tmp = TempDir::new().expect("tempdir");
    let doc =
        "schema_version = 3\n\n[hosts.keyed]\nname = \"keyed\"\nhost = \"h.example\"\nport = 22\n\
         username = \"u\"\npassword = \"\"\nkey_path = \"/nonexistent/key\"\nuse_agent = false\n\
         timeout_ms = 60000\nmax_command_chars = 1000\nmax_output_chars = 100000\n\
         disable_sudo = false\nschema_version = 3\nadded_at = \"2026-10-19T19:48:55+00:00\"\n\
         tags = []\ntls = false\n";
    std::fs::write(tmp.path().join("config.toml"), doc).expect("write config");
    let (code, stdout, stderr) = run_text(&tmp, &["vps", "list"]);
    assert_eq!(code, 0, "{}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8_lossy(&stdout);
    assert!(text.contains("(not set)"), "{text}");
    assert!(!text.contains("***"), "{text}");
}

/// Invisible characters that must be escaped in text: soft hyphen, ALM, Mongolian
/// vowel separator, the deprecated format controls and a tag character.
const INVISIBLE: [char; 5] = ['\u{ad}', '\u{61c}', '\u{180e}', '\u{206a}', '\u{e0041}'];

#[test]
#[serial]
fn invisible_formatting_characters_are_escaped_in_text_output() {
    for c in INVISIBLE {
        let tmp = TempDir::new().expect("tempdir");
        // A TOML basic string takes `\uXXXX` or `\UXXXXXXXX`.
        write_config(&tmp, &format!("evil\\U{:08x}host", c as u32), "");
        let (code, stdout, stderr) = run_text(&tmp, &["vps", "list"]);
        let shown = c.escape_unicode().to_string();
        assert_eq!(code, 0, "{shown}: {}", String::from_utf8_lossy(&stderr));
        let text = String::from_utf8_lossy(&stdout);
        assert!(
            !text.contains(c),
            "{shown}: raw character reached the terminal"
        );
        assert!(
            text.contains(&format!("evil{shown}host")),
            "{shown}: {text}"
        );
    }
}

#[test]
#[serial]
fn json_keeps_the_original_value() {
    let tmp = TempDir::new().expect("tempdir");
    write_config(&tmp, "evil\\u202ehost", "");
    let out = cmd(&tmp)
        .args(["vps", "show", "legacy", "--json"])
        .output()
        .expect("spawn ssh-cli");
    assert_eq!(out.status.code(), Some(0));
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout is one JSON document");
    assert_eq!(v["host"], "evil\u{202e}host", "{v}");
}

#[test]
#[serial]
fn json_error_envelope_parses_and_keeps_the_typed_name() {
    let tmp = TempDir::new().expect("tempdir");
    let out = cmd(&tmp)
        .args(["vps", "show", HOSTILE, "--json"])
        .output()
        .expect("spawn ssh-cli");
    assert_eq!(out.status.code(), Some(EX_NOINPUT));
    assert!(out.stdout.is_empty());
    // The wire is JSON, so ESC and BEL appear as `\u001b` / `\u0007`, never as a byte.
    // U+202E is not a JSON control character, so serde leaves it as is; JSON
    // consumers parse it, they do not render it.
    assert!(!out.stderr.contains(&0x1b) && !out.stderr.contains(&0x07));
    let line = String::from_utf8_lossy(&out.stderr);
    let env: serde_json::Value =
        serde_json::from_str(line.trim()).expect("one JSON line on stderr");
    assert_eq!(env["error_code"], "vps_not_found", "{env}");
    assert!(
        env["message"]
            .as_str()
            .is_some_and(|m| m.contains('\u{1b}')),
        "the envelope must carry the real value: {env}"
    );
}
