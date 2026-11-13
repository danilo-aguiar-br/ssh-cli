// SPDX-License-Identifier: MIT OR Apache-2.0
//! Configuration directory mode at the process boundary.
//!
//! # Why argv and not the library
//!
//! The guarantee is what an operator finds on disk after a real invocation: a
//! directory this binary creates is `0700`, an existing directory it does not own
//! the policy of (`--config-dir` pointing at a shared folder) is never `chmod`ed,
//! and `doctor` reports the exposure. The tightening branch itself, which depends on
//! the XDG default, is also unit-tested in `src/fs_perm.rs` with an injected default.
//!
//! # Why everything lives in a tempdir
//!
//! Each case clears the environment and points `HOME` and `XDG_CONFIG_HOME` at
//! tempdirs, so nothing here can touch the real configuration of the user.
//!
//! # Why the mode assertions are Unix-only
//!
//! Windows has no POSIX mode bits and the crate applies no DACL there, so the
//! assertions compare modes only under `cfg(unix)`.

#![cfg(unix)]

use assert_cmd::Command;
use serde_json::Value;
use serial_test::serial;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::TempDir;

/// Spawns the binary with an isolated `HOME` / `XDG_CONFIG_HOME` and no `--config-dir`.
fn bare(home: &TempDir, xdg: &TempDir) -> Command {
    let llvm_profile_file = std::env::var_os("LLVM_PROFILE_FILE");
    let mut c = Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
    c.env_clear();
    c.env("PATH", std::env::var_os("PATH").unwrap_or_default());
    if let Some(value) = llvm_profile_file {
        c.env("LLVM_PROFILE_FILE", value);
    }
    c.env("HOME", home.path());
    c.env("XDG_CONFIG_HOME", xdg.path());
    c.arg("--allow-plaintext-secrets");
    c
}

/// Same as [`bare`] plus `--config-dir <target>`.
fn with_config_dir(home: &TempDir, xdg: &TempDir, target: &Path) -> Command {
    let mut c = bare(home, xdg);
    c.arg("--config-dir").arg(target);
    c
}

fn mode_of(path: &Path) -> u32 {
    std::fs::metadata(path)
        .unwrap_or_else(|e| panic!("{} missing: {e}", path.display()))
        .permissions()
        .mode()
        & 0o777
}

fn set_mode(path: &Path, bits: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(bits)).expect("chmod");
}

fn add_host(mut c: Command, name: &str) {
    c.args([
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

/// Runs `doctor --json` and returns the `local` object.
fn doctor_local(c: &mut Command) -> Value {
    let out = c.args(["vps", "doctor", "--json"]).assert().success();
    let root: Value = serde_json::from_slice(&out.get_output().stdout).expect("doctor json");
    root.get("local").cloned().expect("local object")
}

#[test]
#[serial]
fn a_new_config_dir_behind_a_toml_path_is_born_private() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("fresh").join("sub");
    let toml = dir.join("config.toml");
    add_host(with_config_dir(&home, &xdg, &toml), "alpha");
    assert_eq!(mode_of(&dir), 0o700);
    assert_eq!(mode_of(&base.path().join("fresh")), 0o700);
    assert_eq!(mode_of(&toml), 0o600);
    assert_eq!(mode_of(&dir.join("config.toml.lock")), 0o600);
}

#[test]
#[serial]
fn a_new_config_dir_given_as_a_directory_is_born_private() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("plain").join("dir");
    add_host(with_config_dir(&home, &xdg, &dir), "alpha");
    assert_eq!(mode_of(&dir), 0o700);
    assert_eq!(mode_of(&dir.join("config.toml")), 0o600);
}

#[test]
#[serial]
fn connect_in_a_new_config_dir_keeps_the_marker_dir_private() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("marker");
    add_host(with_config_dir(&home, &xdg, &dir), "alpha");
    with_config_dir(&home, &xdg, &dir)
        .args(["connect", "alpha"])
        .assert()
        .success();
    assert_eq!(mode_of(&dir), 0o700);
    assert_eq!(mode_of(&dir.join("active")), 0o600);
}

#[test]
#[serial]
fn locale_set_creates_a_new_directory_private() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("loc").join("sub");
    with_config_dir(&home, &xdg, &dir)
        .args(["locale", "set", "en"])
        .assert()
        .success();
    assert_eq!(mode_of(&dir), 0o700);
    assert_eq!(mode_of(&dir.join("lang")), 0o600);
}

#[test]
#[serial]
fn locale_set_behind_a_toml_path_creates_the_parent_not_a_directory_named_toml() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("locfile");
    let toml = dir.join("config.toml");
    with_config_dir(&home, &xdg, &toml)
        .args(["locale", "set", "en"])
        .assert()
        .success();
    assert_eq!(mode_of(&dir), 0o700);
    assert!(dir.join("lang").is_file());
    assert!(!toml.is_dir(), "config.toml must not become a directory");
}

#[test]
#[serial]
fn an_existing_non_default_directory_is_never_chmodded_and_doctor_reports_it() {
    let (home, xdg, shared) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    set_mode(shared.path(), 0o755);
    add_host(with_config_dir(&home, &xdg, shared.path()), "alpha");
    assert_eq!(
        mode_of(shared.path()),
        0o755,
        "foreign directory must stay as is"
    );
    assert_eq!(mode_of(&shared.path().join("config.toml")), 0o600);

    let local = doctor_local(&mut with_config_dir(&home, &xdg, shared.path()));
    assert_eq!(local["config_dir_exposed"], Value::Bool(true));
    assert_eq!(local["config_dir_permissions"], Value::from("0755"));
    assert_eq!(mode_of(shared.path()), 0o755, "doctor is read-only");
}

#[test]
#[serial]
fn an_existing_default_xdg_directory_is_tightened() {
    let (home, xdg) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let default_dir = xdg.path().join(ssh_cli::constants::APP_NAME);
    std::fs::create_dir(&default_dir).expect("mkdir default");
    set_mode(&default_dir, 0o755);
    add_host(bare(&home, &xdg), "alpha");
    assert_eq!(mode_of(&default_dir), 0o700);
    assert_eq!(mode_of(&default_dir.join("config.toml")), 0o600);
}

#[test]
#[serial]
fn a_default_xdg_symlink_is_not_followed() {
    let (home, xdg, real) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    set_mode(real.path(), 0o755);
    let link = xdg.path().join(ssh_cli::constants::APP_NAME);
    std::os::unix::fs::symlink(real.path(), &link).expect("symlink");
    add_host(bare(&home, &xdg), "alpha");
    assert_eq!(
        mode_of(real.path()),
        0o755,
        "link target must not be chmodded"
    );
}

#[test]
#[serial]
fn doctor_on_a_directory_created_by_the_cli_is_not_exposed() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("clean");
    add_host(with_config_dir(&home, &xdg, &dir), "alpha");
    let local = doctor_local(&mut with_config_dir(&home, &xdg, &dir));
    assert_eq!(local["config_dir_exposed"], Value::Bool(false));
    assert_eq!(local["config_dir_permissions"], Value::from("0700"));
    assert_eq!(local["invalid_host_names"], Value::Array(Vec::new()));
}

#[test]
#[serial]
fn doctor_lists_legacy_host_names_that_fail_the_strict_charset() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("legacy");
    std::fs::create_dir(&dir).expect("mkdir");
    set_mode(&dir, 0o700);
    std::fs::write(
        dir.join("config.toml"),
        r#"schema_version = 3

[hosts."my host"]
name = "my host"
host = "10.0.0.1"
port = 22
username = "admin"
password = "x"
timeout_ms = 60000
schema_version = 3

[hosts.good]
name = "good"
host = "10.0.0.2"
port = 22
username = "admin"
password = "x"
timeout_ms = 60000
schema_version = 3
"#,
    )
    .expect("write legacy config");
    set_mode(&dir.join("config.toml"), 0o600);

    let local = doctor_local(&mut with_config_dir(&home, &xdg, &dir));
    assert_eq!(local["invalid_host_names"], serde_json::json!(["my host"]));
}

#[test]
#[serial]
fn doctor_does_not_panic_on_an_unreadable_stored_name() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("hostile");
    std::fs::create_dir(&dir).expect("mkdir");
    set_mode(&dir, 0o700);
    std::fs::write(
        dir.join("config.toml"),
        "schema_version = 3\n\n[hosts.\"a\\u001bb\"]\nname = \"a\\u001bb\"\nhost = \"10.0.0.1\"\nport = 22\nusername = \"u\"\npassword = \"x\"\ntimeout_ms = 60000\nschema_version = 3\n",
    )
    .expect("write hostile config");
    let out = with_config_dir(&home, &xdg, &dir)
        .args(["vps", "doctor", "--json"])
        .output()
        .expect("spawn");
    // A panic exits 101; a clean refusal exits with a project code.
    assert_ne!(out.status.code(), Some(101), "doctor must not panic");
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains('\u{1b}'),
        "the raw escape byte must never reach the terminal"
    );
}

/// Files allowed to mention `create_dir_all`, with the exact number of mentions.
///
/// Derived with `rg -c create_dir_all src`. Every entry creates a directory that is
/// NOT the configuration directory, so it keeps the plain umask-default behaviour:
/// - `scp/batch.rs`, `scp/multi_host.rs`, `sftp/batch.rs`, `sftp/dispatch.rs`,
///   `ssh/sftp_session.rs`, `ssh/client_real_scp.rs`: local destination folders of
///   downloads, chosen by the user on the command line.
///
/// `fs_perm.rs` is the one place that may create config directories and is exempt
/// from the count (its docs and branches mention the call several times).
const CREATE_DIR_ALL_ALLOWLIST: &[(&str, usize)] = &[
    ("scp/batch.rs", 1),
    ("scp/multi_host.rs", 2),
    ("sftp/batch.rs", 1),
    ("sftp/dispatch.rs", 1),
    ("ssh/client_real_scp.rs", 1),
    ("ssh/sftp_session.rs", 2),
];

/// The single module allowed to create configuration directories.
const CONFIG_DIR_OWNER: &str = "fs_perm.rs";

fn collect_rs(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read_dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_module_creates_a_config_directory_outside_the_helper() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs(&src, &mut files);
    assert!(files.len() > 50, "source scan found too few files");

    let mut offenders = Vec::new();
    let mut seen_allowed = Vec::new();
    for file in &files {
        let rel = file
            .strip_prefix(&src)
            .expect("under src")
            .to_string_lossy()
            .replace('\\', "/");
        let count = std::fs::read_to_string(file)
            .expect("read source")
            .matches("create_dir_all")
            .count();
        if count == 0 || rel == CONFIG_DIR_OWNER {
            continue;
        }
        match CREATE_DIR_ALL_ALLOWLIST.iter().find(|(p, _)| *p == rel) {
            Some((_, expected)) if *expected == count => seen_allowed.push(rel),
            _ => offenders.push(format!("{rel}: {count} mention(s)")),
        }
    }
    assert!(
        offenders.is_empty(),
        "create_dir_all outside fs_perm::ensure_config_dir (add to the allowlist only if the \
         directory is not config): {offenders:?}"
    );
    for (path, _) in CREATE_DIR_ALL_ALLOWLIST {
        assert!(
            seen_allowed.iter().any(|s| s == path),
            "stale allowlist entry: {path}"
        );
    }
}

/// Like [`with_config_dir`] but WITHOUT `--allow-plaintext-secrets`: the password is
/// encrypted, so `secrets.key` is created next to the registry.
fn encrypted(home: &TempDir, xdg: &TempDir, target: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
    c.env_clear();
    c.env("PATH", std::env::var_os("PATH").unwrap_or_default());
    if let Some(value) = std::env::var_os("LLVM_PROFILE_FILE") {
        c.env("LLVM_PROFILE_FILE", value);
    }
    c.env("HOME", home.path());
    c.env("XDG_CONFIG_HOME", xdg.path());
    c.arg("--config-dir").arg(target);
    c
}

fn no_entry_named_toml_is_a_directory(dir: &Path) {
    for entry in std::fs::read_dir(dir).expect("read_dir") {
        let path = entry.expect("entry").path();
        let named_toml = path.extension().and_then(|e| e.to_str()) == Some("toml");
        assert!(
            !(named_toml && path.is_dir()),
            "{} is a directory",
            path.display()
        );
    }
}

#[test]
#[serial]
fn encrypted_secrets_behind_a_toml_path_land_in_the_parent_directory() {
    for file_name in ["config.toml", "c.toml"] {
        let (home, xdg, base) = (
            TempDir::new().unwrap(),
            TempDir::new().unwrap(),
            TempDir::new().unwrap(),
        );
        let dir = base.path().join("sub");
        let toml = dir.join(file_name);
        add_host(encrypted(&home, &xdg, &toml), "alpha");
        assert_eq!(mode_of(&dir), 0o700, "{file_name}: parent mode");
        assert_eq!(
            mode_of(&dir.join("secrets.key")),
            0o600,
            "{file_name}: key mode"
        );
        assert!(toml.is_file(), "{file_name} must be the registry file");
        no_entry_named_toml_is_a_directory(&dir);
        no_entry_named_toml_is_a_directory(base.path());
    }
}

#[test]
#[serial]
fn export_creates_missing_output_directories_private_and_spares_existing_ones() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cfg = base.path().join("cfg");
    add_host(with_config_dir(&home, &xdg, &cfg), "alpha");
    set_mode(base.path(), 0o755);
    let out = base.path().join("new").join("deep").join("e.json");
    with_config_dir(&home, &xdg, &cfg)
        .args(["vps", "export", "--include-secrets", "-o"])
        .arg(&out)
        .assert()
        .success();
    assert_eq!(mode_of(&base.path().join("new")), 0o700);
    assert_eq!(mode_of(&base.path().join("new").join("deep")), 0o700);
    assert_eq!(mode_of(&out), 0o600);
    assert_eq!(
        mode_of(base.path()),
        0o755,
        "existing output folder is untouched"
    );
}

/// Writes a legacy registry whose tables are keyed differently from the record names.
fn write_registry(dir: &Path, body: &str) {
    std::fs::create_dir_all(dir).expect("mkdir");
    set_mode(dir, 0o700);
    std::fs::write(dir.join("config.toml"), body).expect("write registry");
    set_mode(&dir.join("config.toml"), 0o600);
}

fn table(key: &str, name: &str) -> String {
    format!(
        "[hosts.\"{key}\"]\nname = \"{name}\"\nhost = \"10.0.0.1\"\nport = 22\nusername = \"u\"\npassword = \"x\"\ntimeout_ms = 60000\nschema_version = 3\n\n"
    )
}

#[test]
#[serial]
fn a_decomposed_unicode_key_is_addressable_by_the_normalized_name() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("nfd");
    write_registry(
        &dir,
        &format!(
            "schema_version = 3\n\n{}",
            table("cafe\u{301}", "cafe\u{301}")
        ),
    );
    let listed = with_config_dir(&home, &xdg, &dir)
        .args(["vps", "list", "--json"])
        .assert()
        .success();
    assert!(String::from_utf8_lossy(&listed.get_output().stdout).contains("caf\u{e9}"));
    with_config_dir(&home, &xdg, &dir)
        .args(["vps", "show", "caf\u{e9}"])
        .assert()
        .success();
}

#[test]
#[serial]
fn a_key_with_a_control_byte_and_a_valid_name_is_addressable_by_the_name() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("esc");
    write_registry(
        &dir,
        &format!("schema_version = 3\n\n{}", table("a\\u001bb", "ab")),
    );
    with_config_dir(&home, &xdg, &dir)
        .args(["vps", "show", "ab"])
        .assert()
        .success();
}

#[test]
#[serial]
fn two_tables_that_normalize_to_one_name_fail_with_a_clear_data_error() {
    let (home, xdg, base) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dir = base.path().join("clash");
    let body = format!(
        "schema_version = 3\n\n{}{}",
        table("caf\u{e9}", "caf\u{e9}"),
        table("cafe\u{301}", "cafe\u{301}")
    );
    write_registry(&dir, &body);
    let out = with_config_dir(&home, &xdg, &dir)
        .args(["vps", "list"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(65), "data error exit");
    assert!(out.stdout.is_empty(), "stdout must stay empty on failure");
    assert!(String::from_utf8_lossy(&out.stderr).contains("two hosts named"));
}
