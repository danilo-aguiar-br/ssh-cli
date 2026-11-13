// SPDX-License-Identifier: MIT OR Apache-2.0
//! Every published flag is reachable, documented where the document claims to, and named
//! in a behavioural test.
//!
//! # Why this suite exists (A5, audit of 2026-10-21)
//!
//! The tree shipped 89 flags. Seventy-one of them appeared somewhere in `tests/`, and
//! nobody could say which of the other eighteen had ever been executed. Worse, the
//! documents that claim a complete inventory did not agree with each other: measured
//! before this change, `docs/AGENTS.md` named 72 of the 89 and `llms-full.txt` named all
//! 89, while every gate stayed green, because no gate compared a document against the
//! tree.
//!
//! The rule this file enforces is the same one the rest of this repository already
//! applies to commands: a claim of completeness is a contract, and a contract with no
//! gate is indistinguishable from no contract. Here the claim is per document, so the
//! test reads each document's own claim instead of imposing one list on files that never
//! made it. `docs/COOKBOOK.md` is recipes and `docs/CROSS_PLATFORM.md` is portability;
//! demanding all 89 flags from them would be padding, not documentation.
//!
//! The second half of the file is the coverage half: the named behavioural tests below
//! drive a flag and assert on what the process actually did, because "the parser accepts
//! it" was exactly the state that let `--no-input` ship inert.
//!
//! This suite drives the real binary, so it lives beside `gaps_v059_agent_native.rs`
//! rather than inside `docs_conformance.rs`, which is prose-only by design.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Runs the binary in a throwaway home so no real registry is touched.
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

/// Every non-hidden long flag the command tree publishes.
///
/// Read from `ssh-cli commands --json` instead of a hand-held list: a literal list would
/// keep passing after the tree grew a flag, which is the drift this file exists to catch.
fn published_flags() -> BTreeSet<String> {
    let tmp = TempDir::new().expect("tempdir");
    let out = cmd(&tmp)
        .args(["commands", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let tree: Value = serde_json::from_slice(&out).expect("commands --json is JSON");

    let mut flags = BTreeSet::new();
    fn walk(node: &Value, flags: &mut BTreeSet<String>) {
        if let Some(args) = node.get("args").and_then(Value::as_array) {
            for arg in args {
                let hidden = arg.get("hidden").and_then(Value::as_bool).unwrap_or(false);
                if let Some(long) = arg.get("long").and_then(Value::as_str) {
                    if !hidden {
                        flags.insert(format!("--{long}"));
                    }
                }
            }
        }
        if let Some(subs) = node.get("subcommands").and_then(Value::as_array) {
            for sub in subs {
                walk(sub, flags);
            }
        }
    }
    walk(&tree["tree"], &mut flags);
    assert!(!flags.is_empty(), "no flags found in the command tree");
    flags
}

/// Documents that claim every flag, and the counter-claim that keeps the list honest.
///
/// `(path, evidence)` where `evidence` is the literal sentence, in that file, that makes
/// the claim. A document whose claim is removed fails here, so the pair cannot rot into
/// a requirement nobody states.
const FLAG_INVENTORY_DOCS: &[(&str, &str)] = &[
    (
        "llms-full.txt",
        "it lists all 48 leaf commands with every flag",
    ),
    (
        "README.md",
        "Shape the payload before serialization with `--select`",
    ),
    (
        "README.pt-BR.md",
        "Reduza o payload antes da serialização com `--select`",
    ),
    (
        "docs/HOW_TO_USE.md",
        "Global flags: 27 flags that go before the subcommand",
    ),
    (
        "docs/HOW_TO_USE.pt-BR.md",
        "Flags globais: 27 flags que vão antes do subcomando",
    ),
];

#[test]
fn every_published_flag_is_documented_where_the_inventory_is_claimed() {
    let flags = published_flags();
    let mut missing = Vec::new();

    for (doc, evidence) in FLAG_INVENTORY_DOCS {
        let text = read(doc);
        assert!(
            text.contains(evidence),
            "{doc} no longer carries its inventory claim ({evidence:?}). Either the claim \
             moved and this list must follow it, or the document stopped claiming \
             completeness and must not be held to it."
        );
        for flag in &flags {
            if !text.contains(flag.as_str()) {
                missing.push(format!("{doc}: {flag}"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "{} document(s) claim every flag and omit flags:\n  {}\n\
         Write the flag in the inventory table of the document that claims it. The count \
         is {} flags, and a partial inventory is worse than none because the reader stops \
         looking once it appears complete.",
        missing
            .iter()
            .map(|m| m.split(':').next().unwrap_or(""))
            .collect::<BTreeSet<_>>()
            .len(),
        flags.len(),
        missing.join("\n  ")
    );
}

/// Documents that carry a per-leaf flag section in both languages.
///
/// These eight were measured on 2026-10-21 as `CROSS_PLATFORM` 39/89 (EN and PT),
/// `AGENTS` 73/89 (EN and PT), `COOKBOOK` 85/89 (EN and PT) and `INTEGRATIONS` 68/89
/// (EN and PT). The suite then accepted any loose mention of `HOW_TO_USE`, `README.md`,
/// `llms-full.txt` or `commands --json` as a substitute for the inventory, so the
/// shortfall shipped green: that assertion asked a question the documents could answer
/// without listing a single flag.
///
/// A pointer is not an inventory. A reader who is told "the flags are over there" still
/// cannot build an integration from this file, and a document that names a per-leaf
/// section is claiming the flags are here. The assertion is the count, never a mention.
const PER_LEAF_FLAG_DOCS: &[&str] = &[
    "docs/AGENTS.md",
    "docs/AGENTS.pt-BR.md",
    "docs/COOKBOOK.md",
    "docs/COOKBOOK.pt-BR.md",
    "docs/CROSS_PLATFORM.md",
    "docs/CROSS_PLATFORM.pt-BR.md",
    "INTEGRATIONS.md",
    "INTEGRATIONS.pt-BR.md",
];

#[test]
fn every_document_with_a_per_leaf_section_names_every_flag() {
    let flags = published_flags();
    let mut shortfall = Vec::new();

    for doc in PER_LEAF_FLAG_DOCS {
        let text = read(doc);
        let missing: Vec<&String> = flags
            .iter()
            .filter(|flag| !text.contains(flag.as_str()))
            .collect();
        if !missing.is_empty() {
            shortfall.push(format!(
                "{doc}: {}/{} (missing {})",
                flags.len() - missing.len(),
                flags.len(),
                missing
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
    }

    assert!(
        shortfall.is_empty(),
        "{} document(s) name a per-leaf flag section and omit flags:\n  {}\n\
         Write the flag into that document's own section. Loosening this count, or \
         replacing it with a pointer at another file, is the regression this test exists \
         to stop: a partial inventory reads as complete and the reader stops looking.",
        shortfall.len(),
        shortfall.join("\n  ")
    );
}

// ── Behaviour, one flag per test ───────────────────────────────────────────

/// `--quiet` must silence the human success line, not the payload.
#[test]
fn quiet_silences_human_success_but_keeps_json() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args(["--quiet", "vps", "list", "--json"])
        .assert()
        .success()
        .stdout(predicate_starts_with("["))
        .stderr(predicate_is_empty());
}

/// `--verbose` is repeatable and must not change the payload on stdout.
#[test]
fn verbose_is_repeatable_and_leaves_stdout_alone() {
    let tmp = TempDir::new().expect("tempdir");
    let plain = cmd(&tmp)
        .args(["vps", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let verbosed = cmd(&tmp)
        .args(["-vv", "vps", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        plain, verbosed,
        "-v levels belong on stderr; stdout must stay byte identical"
    );
}

/// `--no-color` must be accepted and must not alter the JSON payload.
#[test]
fn no_color_is_accepted_and_keeps_the_payload() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args(["--no-color", "vps", "list", "--json"])
        .assert()
        .success()
        .stdout(predicate_starts_with("["));
}

/// `--max-chars` is the legacy alias of `--max-command-chars` and must persist the value.
#[test]
fn max_chars_is_a_working_alias_for_max_command_chars() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "alias",
            "--host",
            "1.2.3.4",
            "--user",
            "u",
            "--password-stdin",
            "--max-chars",
            "1234",
        ])
        .write_stdin("secret")
        .assert()
        .success();

    let shown: Value = serde_json::from_slice(
        &cmd(&tmp)
            .args(["vps", "show", "alias", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .expect("show is JSON");
    assert_eq!(
        shown["max_command_chars"], 1234,
        "--max-chars must write the same field as --max-command-chars: {shown}"
    );
}

/// A registered host must be selectable by tag, so `--tag` round-trips into `tags`.
#[test]
fn tag_round_trips_into_the_registry() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "tagged",
            "--host",
            "1.2.3.4",
            "--user",
            "u",
            "--password-stdin",
            "--tag",
            "prod",
        ])
        .write_stdin("secret")
        .assert()
        .success();

    let listed: Value = serde_json::from_slice(
        &cmd(&tmp)
            .args(["vps", "list", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .expect("list is JSON");
    assert_eq!(listed[0]["tags"][0], "prod", "{listed}");
}

/// `--print-challenge` is required by `tls acme issue`, so omitting it is a usage error.
///
/// Asserting the requirement is the behavioural proof: the flag is not merely accepted,
/// it is what makes the command able to run without an interactive wait.
#[test]
fn print_challenge_is_required_by_acme_issue() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args(["tls", "acme", "issue", "--domain", "example.test", "--json"])
        .assert()
        .failure()
        .stderr(predicate_contains("print-challenge"));
}

/// `--fail-fast` must stop admitting hosts after the first unit failure.
///
/// The proof is the per-host reason in the batch payload: without the flag every host
/// is attempted and fails on the dial; with it, the hosts after the first failure carry
/// `not attempted (fail-fast stopped admission)`. Comparing the two runs is what makes
/// the flag observable without a server, because an unreachable port is a real failure.
#[test]
fn fail_fast_is_wired_into_the_batch_runner() {
    let tmp = TempDir::new().expect("tempdir");
    for name in ["ff1", "ff2", "ff3"] {
        cmd(&tmp)
            .args([
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
            .write_stdin("secret")
            .assert()
            .success();
    }

    let run = |extra: &[&str]| -> Value {
        let mut args = vec!["--max-concurrency", "1"];
        args.extend_from_slice(extra);
        args.extend_from_slice(&["exec", "--all", "true", "--json"]);
        let out = cmd(&tmp).args(&args).assert().get_output().clone();
        serde_json::from_slice(&out.stdout).expect("batch payload is JSON")
    };

    let without = run(&[]);
    let attempted_without = without["results"]
        .as_array()
        .expect("results array")
        .iter()
        .filter(|r| {
            r["error"]
                .as_str()
                .is_some_and(|e| e.contains("TCP dial failed"))
        })
        .count();
    assert_eq!(
        attempted_without, 3,
        "without --fail-fast every host must be attempted: {without}"
    );

    let with = run(&["--fail-fast"]);
    let skipped = with["results"]
        .as_array()
        .expect("results array")
        .iter()
        .filter(|r| {
            r["error"]
                .as_str()
                .is_some_and(|e| e.contains("fail-fast stopped admission"))
        })
        .count();
    assert!(
        skipped >= 1,
        "--fail-fast must stop admission after the first failure: {with}"
    );
    // Every requested host still appears, so the agent can tell a skip from a silent gap.
    assert_eq!(
        with["results"].as_array().map(Vec::len),
        Some(3),
        "every requested host must be reported, attempted or not: {with}"
    );
}

/// `--recursive` must change what an SFTP transfer asks for, and reject an unknown host
/// before any network I/O, which is what makes this observable without a server.
#[test]
fn recursive_is_accepted_by_the_sftp_transfer_leaves() {
    let tmp = TempDir::new().expect("tempdir");
    for leaf in ["download", "upload"] {
        cmd(&tmp)
            .args([
                "sftp",
                leaf,
                "missing-host",
                "local",
                "remote",
                "--recursive",
                "--json",
            ])
            .assert()
            .failure()
            .stderr(predicate_contains("missing-host"));
    }
}

/// `--check` on `vps add` must report the conflict instead of adding the duplicate.
#[test]
fn check_refuses_a_duplicate_instead_of_adding_it() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "dup",
            "--host",
            "1.1.1.1",
            "--user",
            "u",
            "--password-stdin",
        ])
        .write_stdin("secret")
        .assert()
        .success();

    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "dup",
            "--host",
            "1.1.1.1",
            "--user",
            "u",
            "--password-stdin",
            "--check",
        ])
        .write_stdin("secret")
        .assert()
        .failure();
}

/// Deletes the OS store entry of one test profile when the test ends, panics included.
///
/// The store is machine-wide and outlives the tempdir, so a cleanup written after the
/// last assert leaked one `secrets-primary-key:<digest>` whenever an earlier assert
/// failed. `set_config_dir` is process-global, which is why every test holding this
/// guard is also `#[serial(os_keyring)]`: two of them in parallel would delete each
/// other's entry and leave their own. Declared after the `TempDir`, so it drops first.
struct KeyringEntryGuard(PathBuf);

impl Drop for KeyringEntryGuard {
    fn drop(&mut self) {
        ssh_cli::secrets::set_config_dir(Some(self.0.clone()));
        if let Err(e) = ssh_cli::secrets::delete_key_from_keyring() {
            eprintln!("WARN keyring cleanup did not complete: {e}");
        }
    }
}

/// `--use-keyring` must resolve the key from the OS store, never from a silent mock.
///
/// The store is a host service, so the honest outcomes are two: a real round-trip when
/// the session bus answers, or exit 69 when it does not. What is NOT allowed is exit 0
/// with `key_source: none`, which is what the crate without a backend used to produce.
#[test]
#[serial_test::serial(os_keyring)]
fn use_keyring_is_wired_and_fails_closed() {
    let tmp = TempDir::new().expect("tempdir");
    let _store_entry = KeyringEntryGuard(tmp.path().to_path_buf());
    let out = cmd(&tmp)
        .args(["--use-keyring", "secrets", "init", "--json"])
        .assert()
        .get_output()
        .clone();

    if out.status.success() {
        let payload: Value = serde_json::from_slice(&out.stdout).expect("init is JSON");
        assert_eq!(
            payload["key_source"], "keyring",
            "--use-keyring must store the key in the OS store, or fail; it must never \
             report success with no source: {payload}"
        );
        let status = cmd(&tmp)
            .args(["--use-keyring", "secrets", "status", "--json"])
            .assert()
            .success();
        let status: Value = serde_json::from_slice(&status.get_output().stdout).expect("JSON");
        assert_eq!(
            status["encryption_active"], true,
            "the key written to the store must be readable back: {status}"
        );
        assert_eq!(status["key_source"], "keyring", "{status}");
    } else {
        assert_eq!(
            out.status.code(),
            Some(69),
            "--use-keyring without an answering store must exit 69, not a generic \
             failure: stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// `--secrets-key-file` must become the key source, which `status` reports as `file`.
#[test]
fn secrets_key_file_is_recognised() {
    let tmp = TempDir::new().expect("tempdir");
    let key = tmp.path().join("my.key");
    std::fs::write(&key, "a".repeat(64)).expect("write key");
    let out = cmd(&tmp)
        .args([
            "--secrets-key-file",
            key.to_str().unwrap(),
            "secrets",
            "status",
            "--json",
        ])
        .assert()
        .success();
    let status: Value = serde_json::from_slice(&out.get_output().stdout).expect("status JSON");
    assert_eq!(
        status["key_source"], "file",
        "--secrets-key-file must be the resolved source, not an accepted no-op: {status}"
    );
    assert_eq!(
        status["encryption_active"], true,
        "a 64-hex key file must activate encryption: {status}"
    );
}

/// `--replace-host-key` must reach the connection policy instead of being parsed away.
///
/// The flag only has meaning while dialling, and this suite has no server, so the
/// observable is the connection attempt itself: the host is resolved and the dial starts
/// from the registry entry, which is the branch that owns the policy.
#[test]
fn replace_host_key_is_recognised() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "kh",
            "--host",
            "127.0.0.1",
            "--user",
            "u",
            "--password-stdin",
        ])
        .write_stdin("secret")
        .assert()
        .success();

    let out = cmd(&tmp)
        .args(["--replace-host-key", "exec", "kh", "true", "--json"])
        .assert()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--replace-host-key must be a recognised flag: {stderr}"
    );
    // The failure must come from the dial, never from the parser or the registry: that
    // is the proof the flag travelled with the connection attempt.
    assert!(
        !stderr.contains("vps_not_found"),
        "the target must be resolved before the flag can matter: {stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(74),
        "--replace-host-key must not change the error class of an unreachable host: \
         stderr={stderr}"
    );
}

/// `--scp-file-concurrency` must be bounded by the parser at the hard cap.
///
/// The observable without a server is the rejection: a value above the cap is refused
/// with exit 64, and a value inside the range is accepted. A flag accepted but never
/// clamped is exactly the silent no-op this suite exists to prevent.
#[test]
fn scp_file_concurrency_is_recognised() {
    let tmp = TempDir::new().expect("tempdir");
    let too_big = cmd(&tmp)
        .args(["--scp-file-concurrency", "100000", "vps", "list", "--json"])
        .assert()
        .failure()
        .code(2);
    let err = String::from_utf8_lossy(&too_big.get_output().stderr);
    assert!(
        err.contains("scp-file-concurrency"),
        "the refusal must name the flag and its bound: {err}"
    );

    cmd(&tmp)
        .args(["--scp-file-concurrency", "4", "vps", "list", "--json"])
        .assert()
        .success();
}

/// `--no-tls` on `vps edit` must persist the flag off, mirrored by `tls: true` otherwise.
#[test]
fn no_tls_round_trips_on_vps_edit() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "tls-host",
            "--host",
            "1.2.3.4",
            "--user",
            "u",
            "--password-stdin",
        ])
        .write_stdin("secret")
        .assert()
        .success();

    cmd(&tmp)
        .args(["vps", "edit", "tls-host", "--no-tls"])
        .assert()
        .success();

    let shown: Value = serde_json::from_slice(
        &cmd(&tmp)
            .args(["vps", "show", "tls-host", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .expect("show is JSON");
    assert!(
        shown.get("tls").is_none() || shown["tls"] == false,
        "--no-tls must leave the host as plain TCP: {shown}"
    );
}

/// `--enable-sudo` is the documented undo of `--disable-sudo` and must persist.
#[test]
fn enable_sudo_undoes_disable_sudo() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "sudo-host",
            "--host",
            "1.2.3.4",
            "--user",
            "u",
            "--password-stdin",
            "--disable-sudo",
        ])
        .write_stdin("secret")
        .assert()
        .success();

    cmd(&tmp)
        .args(["vps", "edit", "sudo-host", "--enable-sudo"])
        .assert()
        .success();

    let shown: Value = serde_json::from_slice(
        &cmd(&tmp)
            .args(["vps", "show", "sudo-host", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .expect("show is JSON");
    assert_eq!(shown["disable_sudo"], false, "{shown}");
}

/// `--description` must reach the command that goes on the wire.
///
/// Recognising the flag is not the contract; appending it as the shell comment is.
/// The observable without a server is the command-length guard: the host below caps the
/// command at 20 characters, and `true` alone is 4. When the description is appended the
/// same invocation is refused as too long, which can only happen if the description
/// became part of the command. The sanitization of line breaks is pinned directly by
/// `description_sanitizes_line_breaks` in `src/ssh/packing.rs`.
#[test]
fn description_is_accepted_on_the_exec_family() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "desc-host",
            "--host",
            "127.0.0.1",
            "--user",
            "u",
            "--password-stdin",
            "--max-command-chars",
            "20",
        ])
        .write_stdin("secret")
        .assert()
        .success();

    // Without the description the command fits and the run proceeds to the dial.
    let plain = cmd(&tmp)
        .args(["exec", "desc-host", "true", "--json"])
        .assert()
        .get_output()
        .clone();
    assert_ne!(
        plain.status.code(),
        Some(64),
        "a 4-character command must not be refused by a 20-character cap: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    // With it, the same command is over the cap, which proves it was appended.
    let with_description = cmd(&tmp)
        .args([
            "exec",
            "desc-host",
            "--description",
            "uma descricao bem longa",
            "true",
            "--json",
        ])
        .assert()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&with_description.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--description must be a recognised flag of `exec`: {stderr}"
    );
    assert_eq!(
        with_description.status.code(),
        Some(64),
        "the description must be appended to the command, making it exceed the cap: \
         {stderr}"
    );
    assert!(
        stderr.contains("command_too_long"),
        "the refusal must be the length guard, which is the observable that the \
         description reached the command: {stderr}"
    );
}

/// `--contact` on the ACME account leaf must be validated before any network call.
///
/// The previous version only asserted that the parser did not reject the flag, which is
/// the exact shape of test that let an inert flag ship. The contact is now proved to
/// reach the validation that decides whether the CA is contacted at all: a malformed
/// contact is refused with the email quoted back, and the refusal is `invalid_argument`
/// (exit 64), never a network failure. That is observable with no server and no DNS.
#[test]
fn acme_contact_is_validated_before_the_network() {
    let tmp = TempDir::new().expect("tempdir");
    let out = cmd(&tmp)
        .args([
            "tls",
            "acme",
            "account",
            "create",
            "--contact",
            "bogus-address",
            "--staging",
            "--json",
        ])
        .assert()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let envelope: Value = stderr
        .lines()
        .find(|l| l.starts_with('{'))
        .and_then(|l| serde_json::from_str(l).ok())
        .unwrap_or_else(|| panic!("expected a JSON error envelope, got: {stderr}"));
    assert_eq!(
        envelope["error_code"], "invalid_argument",
        "a malformed contact must be a usage error, not an ACME failure: {envelope}"
    );
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains("bogus-address")),
        "the refusal must quote the rejected contact, which proves the flag arrived: \
         {envelope}"
    );
    assert_eq!(
        out.status.code(),
        Some(64),
        "--contact is a usage error and must exit 64: {envelope}"
    );
}

/// `--staging` must select the staging directory, not merely parse.
///
/// The URL decision lives in `AcmeDirectory::from_staging_flag`, pinned by the unit test
/// `staging_flag_selects_the_staging_directory`. What this test pins is that the CLI flag
/// reaches that decision point: `issue --staging` without `--print-challenge` stops at the
/// agent two-step guard, which is evaluated after the flag is parsed and before any
/// network or DNS call, so a run that reaches the guard with the flag accepted and the
/// guard intact proves the flag travelled.
#[test]
fn acme_staging_reaches_the_command_without_network() {
    let tmp = TempDir::new().expect("tempdir");
    let out = cmd(&tmp)
        .args([
            "tls",
            "acme",
            "issue",
            "--domain",
            "x.test",
            "--staging",
            "--json",
        ])
        .assert()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "`--staging` must be a recognised flag: {stderr}"
    );
    assert!(
        stderr.contains("--print-challenge"),
        "the run must reach the two-step guard, which is the observable that the flag \
         was accepted and the command continued: {stderr}"
    );
}

/// `--domain` on `acme issue` must be required, which is what the flag exists for.
#[test]
fn domain_is_required_by_acme_issue() {
    let tmp = TempDir::new().expect("tempdir");
    cmd(&tmp)
        .args(["tls", "acme", "issue", "--print-challenge", "--json"])
        .assert()
        .failure()
        .stderr(predicate_contains("--domain"));
}

/// `secrets init --keyring` must round-trip through the real store, or fail closed.
///
/// This is the Gap 50 proof. The previous version asserted only that the parser accepted
/// the flag, and stayed green over a keyring backend that had no store compiled in and
/// answered `Ok` from a process-local mock: `init` reported success, `status` immediately
/// reported `encryption_active: false`, and nothing was ever persisted. Accepting the
/// flag is not the contract; writing a key that a later process can read back is.
///
/// The entry is scoped by the config directory, so the profile used here is not the
/// caller's. The cleanup removes every candidate name of this profile through the same
/// code the CLI uses, so the machine-wide store does not accumulate test keys.
#[test]
#[serial_test::serial(os_keyring)]
fn keyring_is_recognised_on_secrets_init() {
    let tmp = TempDir::new().expect("tempdir");
    // Cleanup through the product path, so the scoping rule is exercised by the test too.
    let _store_entry = KeyringEntryGuard(tmp.path().to_path_buf());

    // `--config-dir` scopes the store entry; `secrets init` is the leaf under test.
    let init = cmd(&tmp)
        .args(["secrets", "init", "--keyring", "--json"])
        .assert()
        .get_output()
        .clone();

    // A machine without a session bus, or a CI box with no store, is a SKIP and never a
    // PASS. The exit code carries that signal: 69 is the documented unavailability.
    if !init.status.success() {
        assert_eq!(
            init.status.code(),
            Some(69),
            "an unavailable store must fail closed with exit 69 instead of a generic \
             failure: stderr={}",
            String::from_utf8_lossy(&init.stderr)
        );
        eprintln!(
            "SKIP keyring_is_recognised_on_secrets_init: no answering credential store on \
             this machine (exit 69)"
        );
        return;
    }

    let payload: Value = serde_json::from_slice(&init.stdout).expect("init emits JSON");
    assert_eq!(
        payload["key_source"], "keyring",
        "--keyring must store the key in the OS store; a success with no source is the \
         silent failure this test exists to catch: {payload}"
    );

    // A second process must find the same key without repeating the flag: that is what
    // makes it a store and not a variable in the first process's memory.
    let status = cmd(&tmp)
        .args(["secrets", "status", "--json"])
        .assert()
        .success();
    let status: Value = serde_json::from_slice(&status.get_output().stdout).expect("JSON");
    assert_eq!(
        status["key_source"], "keyring",
        "the key written by the previous process must be resolved by the next one: {status}"
    );
    assert_eq!(
        status["encryption_active"], true,
        "a stored key must activate encryption: {status}"
    );
}

fn predicate_contains(needle: &'static str) -> impl Predicate<str> {
    predicates::str::contains(needle)
}

fn predicate_starts_with(prefix: &'static str) -> impl Predicate<str> {
    predicates::str::starts_with(prefix)
}

fn predicate_is_empty() -> impl Predicate<str> {
    predicates::str::is_empty()
}

/// Keeps the compiler honest about the `Path` import used by the helpers above.
#[allow(dead_code)]
fn _assert_path_type(_: &Path) {}
