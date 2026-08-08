// SPDX-License-Identifier: MIT OR Apache-2.0
//! LOTE E: `tunnel` modes (`--reverse`, `--socks5`, `--remote-socket`) and the
//! `--dry-run` preview contract (C2).
//!
//! # Why this suite exists
//!
//! Both features are *argument-shaped*: what they do is decided by which flags
//! were combined, and a wrong combination points a tunnel somewhere the caller
//! never asked for or silently skips a preview. The v0.5.4 audit found exactly
//! that failure mode in `--no-input` — the flag parsed, was accepted, and did
//! nothing on the two commands most likely to run unattended, because nothing
//! ever executed the real binary with it. Every assertion here therefore drives
//! `target/*/ssh-cli` rather than a library function.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ssh-cli"))
}

/// Isolated registry + plaintext secrets, so no test touches the user's XDG dirs.
fn cmd(tmp: &TempDir, args: &[&str]) -> std::process::Output {
    let mut full = vec![
        "--config-dir",
        tmp.path().to_str().expect("utf-8 tempdir"),
        "--allow-plaintext-secrets",
    ];
    full.extend_from_slice(args);
    Command::new(bin())
        .args(&full)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .output()
        .expect("spawn ssh-cli")
}

/// Registers one host via stdin so no secret ever reaches argv.
fn seed(tmp: &TempDir, name: &str) {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(bin())
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "--allow-plaintext-secrets",
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "127.0.0.1",
            "--user",
            "u",
            "--password-stdin",
        ])
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn vps add");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"pw")
        .expect("write password");
    assert!(child.wait().expect("wait").success(), "seeding {name}");
}

fn code(out: &std::process::Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

// ── G-TUN-R02: SOCKS5 argument contract ────────────────────────────────────

#[test]
fn socks5_rejects_a_destination_it_cannot_use() {
    // With SOCKS5 the destination arrives per connection, so accepting a
    // positional target would silently ignore it — the caller would believe the
    // proxy was pinned to one host when it is not.
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(
        &tmp,
        &[
            "--json",
            "tunnel",
            "h1",
            "1080",
            "db.internal",
            "5432",
            "--socks5",
            "--timeout-ms",
            "1000",
        ],
    );
    assert_eq!(code(&out), 64, "stderr={}", stderr(&out));
    assert!(
        stderr(&out).contains("--socks5"),
        "the refusal must name the offending flag: {}",
        stderr(&out)
    );
}

#[test]
fn socks5_and_remote_socket_are_mutually_exclusive() {
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(
        &tmp,
        &[
            "tunnel",
            "h1",
            "1080",
            "--socks5",
            "--remote-socket",
            "/var/run/docker.sock",
            "--timeout-ms",
            "1000",
        ],
    );
    // clap rejects conflicting flags at parse time (exit 2), before any I/O.
    assert_eq!(code(&out), 2, "stderr={}", stderr(&out));
}

// ── G-TUN-R03: remote Unix socket ──────────────────────────────────────────

#[test]
fn remote_socket_must_be_absolute() {
    // A relative path would resolve against the *server's* working directory,
    // which this side cannot know, so it can never mean what the caller intended.
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(
        &tmp,
        &[
            "--json",
            "tunnel",
            "h1",
            "0",
            "--remote-socket",
            "run/docker.sock",
            "--timeout-ms",
            "1000",
        ],
    );
    assert_eq!(code(&out), 64, "stderr={}", stderr(&out));
}

#[test]
fn remote_socket_rejects_a_positional_destination() {
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(
        &tmp,
        &[
            "--json",
            "tunnel",
            "h1",
            "1080",
            "db.internal",
            "5432",
            "--remote-socket",
            "/var/run/docker.sock",
            "--timeout-ms",
            "1000",
        ],
    );
    assert_eq!(code(&out), 64, "stderr={}", stderr(&out));
}

// ── G-TUN-R01: reverse forwarding ──────────────────────────────────────────

#[test]
fn reverse_requires_the_server_bind_arguments() {
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(
        &tmp,
        &[
            "--json",
            "tunnel",
            "h1",
            "8080",
            "--reverse",
            "--timeout-ms",
            "1000",
        ],
    );
    assert_eq!(code(&out), 64, "stderr={}", stderr(&out));
}

#[test]
fn reverse_bind_outside_remote_loopback_needs_acceptance() {
    // The exposed end is the *server's* listener. Guarding the local `--bind`
    // here would check the wrong side entirely and let the tunnel publish a
    // local service to the remote network unannounced.
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(
        &tmp,
        &[
            "--json",
            "tunnel",
            "h1",
            "8080",
            "0.0.0.0",
            "9090",
            "--reverse",
            "--timeout-ms",
            "1000",
        ],
    );
    assert_eq!(code(&out), 64, "stderr={}", stderr(&out));
    assert!(
        stderr(&out).contains("--i-accept-network-exposure"),
        "the refusal must name the escape hatch: {}",
        stderr(&out)
    );
}

#[test]
fn reverse_accepts_port_zero_because_the_server_allocates() {
    // Port 0 is meaningful only here. Reaching the *connection* attempt (not a
    // usage error) is the proof the argument was accepted; there is no server, so
    // the run cannot succeed.
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(
        &tmp,
        &[
            "--json",
            "tunnel",
            "h1",
            "8080",
            "127.0.0.1",
            "0",
            "--reverse",
            "--timeout-ms",
            "600",
        ],
    );
    assert_ne!(
        code(&out),
        64,
        "port 0 must not be a usage error under --reverse: {}",
        stderr(&out)
    );
    assert_ne!(code(&out), 2, "stderr={}", stderr(&out));
}

#[test]
fn local_forward_still_rejects_port_zero() {
    // The mirror of the case above: without a server to allocate, a local forward
    // to port 0 has nothing to connect to.
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(
        &tmp,
        &[
            "--json",
            "tunnel",
            "h1",
            "8080",
            "127.0.0.1",
            "0",
            "--timeout-ms",
            "600",
        ],
    );
    assert_eq!(code(&out), 64, "stderr={}", stderr(&out));
}

#[test]
fn plain_local_forward_arguments_are_unchanged() {
    // The positionals became optional in 0.5.4; the classic four-argument form
    // must keep parsing exactly as before, or every existing agent breaks.
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(
        &tmp,
        &[
            "--json",
            "tunnel",
            "h1",
            "0",
            "127.0.0.1",
            "5432",
            "--timeout-ms",
            "600",
        ],
    );
    assert_ne!(code(&out), 2, "stderr={}", stderr(&out));
    assert_ne!(code(&out), 64, "stderr={}", stderr(&out));
}

#[test]
fn tunnel_still_requires_a_deadline() {
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(&tmp, &["tunnel", "h1", "0", "--socks5"]);
    assert_eq!(
        code(&out),
        2,
        "missing --timeout-ms must fail at parse time"
    );
}

#[test]
fn tunnel_help_documents_every_mode() {
    let out = Command::new(bin())
        .args(["tunnel", "--help"])
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .output()
        .expect("spawn help");
    let text = stdout(&out);
    for flag in ["--socks5", "--remote-socket", "--reverse"] {
        assert!(text.contains(flag), "tunnel --help must document {flag}");
    }
}

// ── C2: `--dry-run` preview contract ───────────────────────────────────────

#[test]
fn dry_run_previews_vps_remove_without_removing() {
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");

    let plan = cmd(&tmp, &["--json", "--dry-run", "vps", "remove", "h1"]);
    assert_eq!(code(&plan), 0, "stderr={}", stderr(&plan));
    let v: serde_json::Value =
        serde_json::from_str(stdout(&plan).trim()).expect("plan must be one JSON document");
    assert_eq!(v["event"], "dry-run");
    assert_eq!(v["operation"], "vps-remove");
    assert_eq!(v["executed"], false);
    assert_eq!(v["name"], "h1");

    // The proof that matters: the host is still registered afterwards.
    let list = cmd(&tmp, &["--json", "--select", "name", "vps", "list"]);
    assert!(
        stdout(&list).contains("h1"),
        "--dry-run must not remove the host: {}",
        stdout(&list)
    );
}

#[test]
fn dry_run_on_a_missing_host_reports_the_real_failure() {
    // A preview that claims success for a host that does not exist is worse than
    // no preview: the agent then reads the real run's exit 66 as a regression.
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(&tmp, &["--json", "--dry-run", "vps", "remove", "absent"]);
    assert_eq!(code(&out), 66, "stderr={}", stderr(&out));
}

#[test]
fn dry_run_is_refused_where_it_is_not_implemented() {
    // This is the `--no-input` lesson encoded as a test: a global flag that is
    // accepted and silently ignored is indistinguishable from one that works.
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(&tmp, &["--json", "--dry-run", "vps", "list"]);
    assert_eq!(code(&out), 64, "stdout={}", stdout(&out));
    // And the refusal must be machine-readable when the caller asked for JSON.
    let v: serde_json::Value =
        serde_json::from_str(stderr(&out).trim()).expect("refusal must be a JSON envelope");
    assert_eq!(v["exit_code"], 64);
    assert_eq!(v["error_code"], "invalid_argument");
}

#[test]
fn dry_run_refusal_names_the_supported_commands() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(&tmp, &["--dry-run", "vps", "path"]);
    let text = stderr(&out);
    for supported in ["vps remove", "sftp rm", "secrets reencrypt"] {
        assert!(
            text.contains(supported),
            "the refusal must list {supported}: {text}"
        );
    }
}

#[test]
fn dry_run_previews_sftp_rm_without_connecting() {
    // Emitted before the SSH handshake on purpose: confirming the file exists
    // would cost a full connect + auth, and a preview that authenticates is no
    // longer side-effect free — it appears in the server's auth log like a real
    // run. The host here is unreachable, so a connect would surface as exit 74.
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    let out = cmd(
        &tmp,
        &["--json", "--dry-run", "sftp", "rm", "h1", "/tmp/whatever"],
    );
    assert_eq!(code(&out), 0, "stderr={}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).expect("plan JSON");
    assert_eq!(v["operation"], "sftp-rm");
    assert_eq!(v["remote"], "/tmp/whatever");
    assert_eq!(v["executed"], false);
}

#[test]
fn dry_run_previews_secrets_reencrypt() {
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");
    seed(&tmp, "h2");
    let out = cmd(&tmp, &["--json", "--dry-run", "secrets", "reencrypt"]);
    assert_eq!(code(&out), 0, "stderr={}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).expect("plan JSON");
    assert_eq!(v["operation"], "secrets-reencrypt");
    assert_eq!(
        v["hosts"], 2,
        "the plan must state how many hosts are at risk"
    );
}

#[test]
fn dry_run_previews_vps_import_naming_the_replacements() {
    // A count alone would hide the fact that decides whether the import is safe:
    // which existing hosts it overwrites.
    let tmp = TempDir::new().unwrap();
    seed(&tmp, "h1");

    // `--output` rather than a pipe: exporting secrets to stdout is refused with
    // exit 64 unless the caller acknowledges it, and that guard is correct — the
    // test must satisfy it, not work around it.
    let file = tmp.path().join("hosts.toml");
    let exported = cmd(
        &tmp,
        &[
            "vps",
            "export",
            "--include-secrets",
            "--output",
            file.to_str().unwrap(),
        ],
    );
    assert_eq!(code(&exported), 0, "stderr={}", stderr(&exported));

    let out = cmd(
        &tmp,
        &[
            "--json",
            "--dry-run",
            "vps",
            "import",
            "--file",
            file.to_str().unwrap(),
        ],
    );
    assert_eq!(code(&out), 0, "stderr={}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).expect("plan JSON");
    assert_eq!(v["operation"], "vps-import");
    let hosts = v["hosts"].as_array().expect("hosts array");
    assert_eq!(hosts.len(), 1);
    assert_eq!(hosts[0]["name"], "h1");
    assert_eq!(
        hosts[0]["replaces_existing"], true,
        "re-importing an existing host must be flagged as a replacement"
    );
}

// ── Wire contract ──────────────────────────────────────────────────────────

#[test]
fn tunnel_schemas_declare_every_mode() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for name in ["tunnel-listening", "tunnel-closed"] {
        let text = std::fs::read_to_string(root.join(format!("docs/schemas/{name}.schema.json")))
            .unwrap_or_else(|_| panic!("{name} schema"));
        let schema: serde_json::Value = serde_json::from_str(&text).expect("valid JSON schema");
        let modes = schema["properties"]["mode"]["enum"]
            .as_array()
            .unwrap_or_else(|| panic!("{name} must declare mode"));
        for expected in ["local", "socks5", "streamlocal", "reverse"] {
            assert!(
                modes.iter().any(|m| m == expected),
                "{name} schema must list mode {expected}"
            );
        }
    }
}

#[test]
fn dry_run_schema_is_indexed_and_valid() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(root.join("docs/schemas/dry-run.schema.json"))
        .expect("dry-run schema");
    let schema: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(schema["properties"]["event"]["const"], "dry-run");
    let ops = schema["properties"]["operation"]["enum"]
        .as_array()
        .expect("operation enum");
    assert_eq!(
        ops.len(),
        6,
        "the schema must list exactly the six commands that implement --dry-run"
    );

    let readme = std::fs::read_to_string(root.join("docs/schemas/README.md")).expect("README");
    assert!(
        readme.contains("dry-run.schema.json"),
        "every schema must be indexed in docs/schemas/README.md"
    );
}
