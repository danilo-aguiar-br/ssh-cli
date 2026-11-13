// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end regression for the residual gaps from the post-0.3.8 audit (0.3.9).
//!
//! IDs: LOG-001, JSON-001, CLI-004, DOC-003 (version string), DENY-002 (policy),
//! REL-003 (tag/version), CHG-001 (docs), SEC-001..003 (exposure hygiene).
//! Usa apenas credenciais FALSAS.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use tempfile::TempDir;

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
    c.arg("--output-format").arg("text");
    c.arg("--allow-plaintext-secrets");
    c
}

fn cmd_json(tmp: &TempDir) -> Command {
    let mut c = cmd(tmp);
    c.env_remove("SSH_CLI_FORCE_TEXT");
    c.arg("--json");
    c
}

fn add_host_password(tmp: &TempDir, name: &str) {
    cmd(tmp)
        .args([
            "vps",
            "add",
            "--name",
            name,
            "--host",
            "203.0.113.10",
            "--user",
            "fakeuser",
            "--password",
            "fake-test-password-not-real-001",
        ])
        .assert()
        .success();
}

fn write_ed25519(tmp: &TempDir) -> Option<std::path::PathBuf> {
    // G-PROC-02: test-only OpenSSH key fixture. Explicit stdio; direct argv; no shell.
    // Runtime product never spawns ssh-keygen — keys are files parsed by russh.
    let key = tmp.path().join("id_ed25519_test");
    let key_path = key.to_str().expect("temp path is UTF-8");
    let status = match std::process::Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519")
        .arg("-f")
        .arg(key_path)
        .arg("-N")
        .arg("")
        .arg("-q")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
    {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("skip: ssh-keygen not on PATH ({e})");
            return None;
        }
        Err(e) => panic!("ssh-keygen spawn failed: {e}"),
    };
    assert!(status.success(), "ssh-keygen failed: {status}");
    Some(key)
}

// --- LOG-001 ---

#[test]
#[serial]
fn gap_log_001_tunnel_json_stderr_has_no_info_prose() {
    let tmp = TempDir::new().unwrap();
    add_host_password(&tmp, "tlog");
    let assert = cmd_json(&tmp)
        .args([
            "tunnel",
            "tlog",
            "19101",
            "127.0.0.1",
            "9",
            "--timeout-ms",
            "30",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        !stderr.contains("Tunnel SSH:"),
        "stderr must carry no INFO prose: {stderr}"
    );
    assert!(
        !stderr.contains("iniciando tunnel"),
        "stderr must not default to INFO: {stderr}"
    );
    assert!(
        stderr.contains("\"exit_code\""),
        "stderr must carry the JSON envelope: {stderr}"
    );
}

// --- JSON-001 ---

#[test]
#[serial]
fn gap_json_001_key_only_password_null() {
    let tmp = TempDir::new().unwrap();
    let Some(key) = write_ed25519(&tmp) else {
        return;
    };
    cmd(&tmp)
        .args([
            "vps",
            "add",
            "--name",
            "konly",
            "--host",
            "203.0.113.11",
            "--user",
            "root",
            "--key",
            key.to_str().unwrap(),
        ])
        .assert()
        .success();
    cmd_json(&tmp)
        .args(["vps", "show", "konly"])
        .assert()
        .success()
        // Compact JSON wire (no spaces after `:`); key-only hosts use JSON null.
        .stdout(predicate::str::contains("\"password\":null"))
        .stdout(predicate::str::contains("\"password\":\"***\"").not());
}

#[test]
#[serial]
fn gap_json_001_password_stays_masked() {
    let tmp = TempDir::new().unwrap();
    add_host_password(&tmp, "withpwd");
    cmd_json(&tmp)
        .args(["vps", "show", "withpwd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"password\":\"***\""))
        .stdout(predicate::str::contains("fake-test-password").not());
}

// --- CLI-004 ---

#[test]
#[serial]
fn gap_cli_004_health_check_aceita_timeout() {
    let tmp = TempDir::new().unwrap();
    // clap accepts the flag; without a VPS this is a domain failure, not a parse error.
    // `--use-active` designates the target explicitly (GAP-SSH-EXEC-ARGC-001); without
    // it the run would fail at argv parsing and never reach the flag under test.
    let assert = cmd(&tmp)
        .args(["health-check", "--use-active", "--json", "--timeout", "50"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "health-check must accept --timeout: {stderr}"
    );
    assert!(
        stderr.contains("66") || stderr.contains("Nenhuma VPS") || stderr.contains("ativa"),
        "expected domain error: {stderr}"
    );
}

// --- DOC-003 / product line ---

#[test]
#[serial]
fn gap_doc_003_version_mentions_039() {
    // Historical 0.3.9 suite; the product line is 0.4.0+ and this keeps the LOG/JSON/CLI behaviour regression.
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// Product-line public docs must state current package version (not stale 0.3.6-as-current).
#[test]
fn gap_doc_003_product_line_docs_mention_039() {
    let current = env!("CARGO_PKG_VERSION");
    const FILES: &[&str] = &[
        "README.md",
        "README.pt-BR.md",
        "llms.txt",
        "llms.pt-BR.txt",
        "llms-full.txt",
        "INTEGRATIONS.md",
        "INTEGRATIONS.pt-BR.md",
        "docs/AGENTS.md",
        "docs/AGENTS.pt-BR.md",
        "docs/HOW_TO_USE.md",
        "docs/HOW_TO_USE.pt-BR.md",
        "docs/COOKBOOK.md",
        "docs/COOKBOOK.pt-BR.md",
        "docs/MIGRATION.md",
        "docs/MIGRATION.pt-BR.md",
        "docs/TESTING.md",
        "docs/TESTING.pt-BR.md",
        "docs/CROSS_PLATFORM.md",
        "docs/CROSS_PLATFORM.pt-BR.md",
        "docs/schemas/README.md",
        "docs/RELEASE_CHECKLIST.md",
        "docs/RELEASE_CHECKLIST.pt-BR.md",
    ];
    for path in FILES {
        let body = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("ler {path}: {e}"));
        assert!(
            body.contains(current),
            "{path} must mention product line {current}"
        );
        // HOW_TO_USE/COOKBOOK/TESTING/CROSS_PLATFORM must not claim current line is only 0.3.6
        if path.contains("HOW_TO_USE")
            || path.contains("COOKBOOK")
            || path.contains("TESTING")
            || path.contains("CROSS_PLATFORM")
            || path.contains("schemas/README")
        {
            assert!(
                !body.contains("Product line: **0.3.6**")
                    && !body.contains("Linha de produto: **0.3.6**")
                    && !body.contains("product line documented here: **0.3.6**")
                    && !body.contains("Linha de produto documentada aqui: **0.3.6**")
                    && !body.contains("payloads (**0.3.6**)"),
                "{path} ainda declara product line 0.3.6 como atual"
            );
        }
    }
}

/// Residual audit behaviors must appear in agent-facing docs (LOG/JSON/CLI).
/// Schema JSON-001: vps-show.schema.json must allow password null.
/// Skills must stay consolidated operational formulas (no version stories).
#[test]
fn gap_doc_003_residual_behaviors_documentados() {
    let agents = std::fs::read_to_string("docs/AGENTS.md").expect("AGENTS");
    let readme = std::fs::read_to_string("README.md").expect("README");
    let skill_en = std::fs::read_to_string("skills/ssh-cli-en/SKILL.md").expect("skill en");
    let skill_pt = std::fs::read_to_string("skills/ssh-cli-pt/SKILL.md").expect("skill pt");
    for (label, body) in [("AGENTS", agents.as_str()), ("README", readme.as_str())] {
        assert!(
            body.to_ascii_lowercase().contains("error")
                && (body.contains("RUST_LOG") || body.contains("tracing")),
            "{label} must document the default tracing error level and RUST_LOG"
        );
        assert!(
            body.contains("null"),
            "{label} must document the JSON null password"
        );
        assert!(
            body.contains("health-check") && body.contains("--timeout"),
            "{label} must document health-check --timeout"
        );
    }
    for (label, skill) in [("en", skill_en.as_str()), ("pt", skill_pt.as_str())] {
        assert!(
            skill.contains("null")
                && skill.contains("--timeout")
                && skill.contains("error")
                && skill.contains("truncated_stdout")
                && skill.contains("remote_exit_code")
                && skill.contains("--quiet")
                && skill.contains("key-passphrase-stdin")
                && skill.contains("--port")
                && !skill.contains("0.4.0 did") && !skill.contains("0.3.9 did")
                && !skill.contains("in version 0.3.9")
                && !skill.contains("versão 0.3.9")
                && !skill.contains("na versão 0.3.9"),
            "skill {label} must consolidate null/timeout/error/envelope/quiet without a per-version changelog"
        );
        // Frontmatter description constraints (GraphRAG skill rules).
        let fm = skill
            .strip_prefix("---\n")
            .and_then(|s| s.split_once("\n---"))
            .map(|(a, _)| a)
            .expect("skill frontmatter");
        let desc_line = fm
            .lines()
            .find(|l| l.starts_with("description:"))
            .expect("description field");
        let desc = desc_line.trim_start_matches("description:").trim();
        assert!(
            desc.chars().count() < 1024,
            "skill {label} description must be under 1024 chars (got {})",
            desc.chars().count()
        );
        assert_eq!(
            desc.matches(':').count(),
            0,
            "skill {label} description MUST NOT contain ':' in its body"
        );
        assert!(
            desc.starts_with("This skill MUST") || desc.starts_with("Esta skill DEVE"),
            "skill {label} description MUST be third person and declare auto-activation"
        );
        assert!(
            desc.contains("auto-activate") || desc.contains("auto-ativar"),
            "skill {label} description MUST declare auto-activation"
        );
    }

    // JSON-001 schema contract: password type includes null (not string-only).
    let schema_text =
        std::fs::read_to_string("docs/schemas/vps-show.schema.json").expect("vps-show.schema.json");
    let schema: serde_json::Value =
        serde_json::from_str(&schema_text).expect("vps-show.schema.json must be valid JSON");
    assert_eq!(
        schema["properties"]["password"]["type"],
        serde_json::json!(["string", "null"]),
        "vps-show.schema.json properties.password.type must be [\"string\", \"null\"] (JSON-001)"
    );
}

// --- DENY-002 policy still yanked=deny ignore empty ---

#[test]
fn gap_deny_002_deny_toml_ignores_no_cve() {
    let deny = std::fs::read_to_string("deny.toml").expect("deny.toml");
    assert!(deny.contains("yanked = \"deny\"") || deny.contains("yanked=\"deny\""));
    // DENY-002 said "ignore must stay empty". That was written when the only reason to
    // ignore anything was silencing a CVE, so emptiness stood in for "no waiver". The
    // keyring backend brought three `unmaintained` advisories whose crates have no safe
    // upgrade at all, which is a different fact from a vulnerability waiver.
    //
    // The assertion is now the rule itself instead of a proxy for it: every ignored id
    // is one of these three, and every one of them carries a written reason. Adding a
    // fourth ignore — for a CVE or for anything else — fails here, and so does an
    // ignore with no reason, which an empty-list check could never have caught.
    const ALLOWED_IGNORES: &[&str] = &[
        "RUSTSEC-2026-0370",
        "RUSTSEC-2026-0371",
        "RUSTSEC-2026-0372",
    ];
    let ignored: Vec<&str> = deny
        .lines()
        .filter_map(|l| {
            let trimmed = l.trim_start();
            if !trimmed.starts_with("{ id = \"RUSTSEC-") {
                return None;
            }
            let start = trimmed.find("\"RUSTSEC-")? + 1;
            let rest = &trimmed[start..];
            let end = rest.find('"')?;
            Some(&rest[..end])
        })
        .collect();
    assert_eq!(
        ignored.len(),
        ALLOWED_IGNORES.len(),
        "deny.toml must ignore exactly the declared unmaintained advisories, found {ignored:?}"
    );
    for id in &ignored {
        assert!(
            ALLOWED_IGNORES.contains(id),
            "{id} is ignored without being declared in this test; a new waiver needs an audit"
        );
    }
    for line in deny.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("{ id = \"RUSTSEC-") {
            continue;
        }
        assert!(
            trimmed.contains("reason = \"")
                && trimmed
                    .split("reason = \"")
                    .nth(1)
                    .is_some_and(|r| r.len() > 20),
            "every ignored advisory must carry a written reason: {trimmed}"
        );
    }
    // A3: the assertion used to pin `multiple-versions = "warn"` literally, which meant
    // *tightening* the policy broke the test while loosening it to `allow` would have
    // passed unnoticed — the check was anchored to the wrong side of the risk. It now
    // requires the setting to be at least as strict as `warn`, so `deny` is accepted
    // and `allow` is rejected.
    assert!(
        deny.contains("multiple-versions = \"deny\"")
            || deny.contains("multiple-versions = \"warn\""),
        "duplicate-version policy must stay at warn or deny, never allow"
    );
    assert!(
        !deny.contains("multiple-versions = \"allow\""),
        "silently allowing duplicate versions hides dual crypto stacks (A3)"
    );
}

// --- CHG-001 / REL presence of changelog section ---

#[test]
fn gap_chg_001_changelog_tem_039() {
    let ch = std::fs::read_to_string("CHANGELOG.md").expect("CHANGELOG");
    assert!(
        ch.contains("## [0.3.9]"),
        "CHANGELOG must carry a 0.3.9 section"
    );
    assert!(
        ch.contains("[0.3.9]:") || ch.contains("compare/v0.3.8"),
        "CHANGELOG must carry a 0.3.9 anchor or link"
    );
}

// --- SEC-001..003: higiene anti-vazamento (auditoria workspace) ---

#[test]
fn gap_sec_001_setting_cyber_ignored_by_directory() {
    let gi = std::fs::read_to_string(".gitignore").expect(".gitignore");
    assert!(
        gi.lines().any(|l| l.trim() == ".setting.cyber/"),
        ".gitignore MUST ignore the .setting.cyber/ directory, not just *.log"
    );
    let cargo = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
    assert!(
        cargo.contains("\".setting.cyber/\""),
        "Cargo.toml exclude MUST list .setting.cyber/"
    );
    let cargoignore = std::fs::read_to_string(".cargoignore").expect(".cargoignore");
    assert!(
        cargoignore.lines().any(|l| l.trim() == ".setting.cyber/"),
        ".cargoignore MUST list .setting.cyber/"
    );
}

/// Every dotted tooling sidecar at the repo root MUST be barred from the published crate.
///
/// `gap_sec_001` above pins one sidecar by name, and naming them one at a time is exactly how
/// `.atomwrite/` slipped through: `.serena/`, `.claude/`, `.setting.cyber/` and `.cursor/` were
/// all listed, and the one directory that the agent's own editing tool creates was not. Measured
/// with the pre-publish gate, `cargo package --list` shipped `.atomwrite/scratch/old.txt` and
/// `.atomwrite/scratch/new.txt`, because Cargo packages any file that is neither tracked nor
/// ignored — being untracked is no protection at all.
///
/// So this gate discovers instead of enumerating: whatever dotted directory exists at the root
/// must be either deliberately packaged (the allowlist) or barred on all three surfaces. A new
/// sidecar therefore turns this test red on the day it appears, which a by-name list cannot do.
///
/// Only `.gitignore` and the manifest `exclude` are real protections; `.cargoignore` is inert as
/// far as Cargo's engine is concerned. It is asserted anyway because the repository maintains it
/// and a half-updated hygiene file is worse than a consistent one.
#[test]
fn gap_sec_001b_every_dotted_sidecar_is_barred_from_the_package() {
    // Deliberately packaged or invisible to Cargo; everything else must be barred.
    const ALLOWED: &[&str] = &[".git", ".cargo"];

    let mut sidecars: Vec<String> = std::fs::read_dir(".")
        .expect("repo root")
        .filter_map(|e| {
            let e = e.ok()?;
            if !e.file_type().ok()?.is_dir() {
                return None;
            }
            let name = e.file_name().to_string_lossy().into_owned();
            (name.starts_with('.') && !ALLOWED.contains(&name.as_str())).then_some(name)
        })
        .collect();
    // `read_dir` order is unspecified and platform-dependent, so sort to keep failures stable.
    sidecars.sort_unstable();

    let gitignore = std::fs::read_to_string(".gitignore").expect(".gitignore");
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
    let cargoignore = std::fs::read_to_string(".cargoignore").expect(".cargoignore");

    let mut missing = Vec::new();
    for name in &sidecars {
        let dir = format!("{name}/");
        if !gitignore.lines().any(|l| l.trim() == dir) {
            missing.push(format!("{dir} missing from .gitignore"));
        }
        if !cargo_toml.contains(&format!("\"{dir}\"")) {
            missing.push(format!("{dir} missing from the Cargo.toml exclude"));
        }
        if !cargoignore.lines().any(|l| l.trim() == dir) {
            missing.push(format!("{dir} missing from .cargoignore"));
        }
    }

    assert!(
        missing.is_empty(),
        "dotted sidecar with no packaging barrier:\n  {}\n\
         Cargo packages any file that is neither tracked nor ignored, so being absent from \
         git is NOT protection. Bar the directory in .gitignore, in the Cargo.toml exclude \
         and in .cargoignore, or add it to ALLOWED if it really is meant to ship.",
        missing.join("\n  ")
    );
}

#[test]
fn gap_sec_002_e2e_refuses_a_grok_config_inside_the_repo() {
    // The harness is split into three modules; the contract spans all of them.
    let script = ["e2e_real_ssh.py", "_e2e_sshd.py", "_e2e_cases.py"]
        .iter()
        .map(|name| std::fs::read_to_string(format!("scripts/{name}")).expect("e2e script"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        script.contains("must not live inside the repository"),
        "e2e_real_ssh.py MUST refuse a grok config under the repository root"
    );
    assert!(
        script.contains("GROK_CFG_ABS") && script.contains("ROOT_ABS"),
        "e2e_real_ssh.py MUST compare the grok config absolute path against ROOT"
    );
    let testing = std::fs::read_to_string("docs/TESTING.md").expect("TESTING");
    assert!(
        testing.contains("$HOME/.grok/config.toml")
            && testing.contains("never copy it into this repository"),
        "TESTING.md MUST document grok config as $HOME-only"
    );
}

#[test]
fn gap_sec_003_docs_use_a_demo_placeholder_not_s3cret() {
    for path in [
        "README.md",
        "README.pt-BR.md",
        "docs/COOKBOOK.md",
        "docs/COOKBOOK.pt-BR.md",
    ] {
        let body = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("ler {path}: {e}"));
        assert!(
            !body.contains("s3cret"),
            "{path} must not use the ambiguous demo password 's3cret'"
        );
        if body.contains("password-stdin") || body.contains("vps add") {
            assert!(
                body.contains("demo-password-not-real"),
                "{path} must use the demo-password-not-real placeholder"
            );
        }
    }
    let sec = std::fs::read_to_string("SECURITY.md").expect("SECURITY");
    assert!(
        sec.contains(".setting.cyber/") && sec.contains("demo-password-not-real"),
        "SECURITY.md must document SEC-001/003 hygiene"
    );
}
