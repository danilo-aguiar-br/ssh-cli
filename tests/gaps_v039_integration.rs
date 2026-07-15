// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regressão e2e dos gaps residuais da auditoria pós-0.3.8 (v0.3.9).
//!
//! IDs: LOG-001, JSON-001, CLI-004, DOC-003 (version string), DENY-002 (policy),
//! REL-003 (tag/version), CHG-001 (docs), SEC-001..003 (higiene de exposição).
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
    c.env("SSH_CLI_ALLOW_PLAINTEXT_SECRETS", "1");
    c.env("SSH_CLI_FORCE_TEXT", "1");
    c.arg("--config-dir").arg(tmp.path());
    c
}

fn cmd_json(tmp: &TempDir) -> Command {
    let mut c = cmd(tmp);
    c.env_remove("SSH_CLI_FORCE_TEXT");
    c.arg("--output-format").arg("json");
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

fn write_ed25519(tmp: &TempDir) -> std::path::PathBuf {
    let key = tmp.path().join("id_ed25519_test");
    let status = std::process::Command::new("ssh-keygen")
        .args(["-t", "ed25519", "-f", key.to_str().unwrap(), "-N", "", "-q"])
        .status()
        .expect("ssh-keygen");
    assert!(status.success(), "ssh-keygen failed");
    key
}

// --- LOG-001 ---

#[test]
#[serial]
fn gap_log_001_tunnel_json_stderr_sem_info_prosa() {
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
        "stderr não deve ter prosa INFO: {stderr}"
    );
    assert!(
        !stderr.contains("iniciando tunnel"),
        "stderr não deve ter INFO default: {stderr}"
    );
    assert!(
        stderr.contains("\"exit_code\""),
        "stderr deve conter envelope JSON: {stderr}"
    );
}

// --- JSON-001 ---

#[test]
#[serial]
fn gap_json_001_key_only_password_null() {
    let tmp = TempDir::new().unwrap();
    let key = write_ed25519(&tmp);
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
        .stdout(predicate::str::contains("\"password\": null"))
        .stdout(predicate::str::contains("\"password\": \"***\"").not());
}

#[test]
#[serial]
fn gap_json_001_com_password_continua_mascara() {
    let tmp = TempDir::new().unwrap();
    add_host_password(&tmp, "withpwd");
    cmd_json(&tmp)
        .args(["vps", "show", "withpwd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"password\": \"***\""))
        .stdout(predicate::str::contains("fake-test-password").not());
}

// --- CLI-004 ---

#[test]
#[serial]
fn gap_cli_004_health_check_aceita_timeout() {
    let tmp = TempDir::new().unwrap();
    // clap aceita a flag (falha de domínio sem VPS, não parse error)
    let assert = cmd(&tmp)
        .args(["health-check", "--json", "--timeout", "50"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "health-check deve aceitar --timeout: {stderr}"
    );
    assert!(
        stderr.contains("66") || stderr.contains("Nenhuma VPS") || stderr.contains("ativa"),
        "expected domain error: {stderr}"
    );
}

// --- DOC-003 / product line ---

#[test]
#[serial]
fn gap_doc_003_version_contem_039() {
    // Suite histórica 0.3.9: product line current é 0.4.0+ (mantém regressão de behaviours LOG/JSON/CLI).
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// Product-line public docs must state current package version (not stale 0.3.6-as-current).
#[test]
fn gap_doc_003_product_line_docs_contem_039() {
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
            "{path} deve mencionar product line {current}"
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
            "{label} deve documentar default tracing error / RUST_LOG"
        );
        assert!(
            body.contains("null"),
            "{label} deve documentar password JSON null"
        );
        assert!(
            body.contains("health-check") && body.contains("--timeout"),
            "{label} deve documentar health-check --timeout"
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
            "skill {label} deve consolidar null/timeout/error/envelope/quiet sem changelog por versão"
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
            "skill {label} description deve ter < 1024 chars (got {})",
            desc.chars().count()
        );
        assert_eq!(
            desc.matches(':').count(),
            0,
            "skill {label} description NÃO DEVE conter ':' no conteúdo"
        );
        assert!(
            desc.starts_with("This skill MUST") || desc.starts_with("Esta skill DEVE"),
            "skill {label} description DEVE ser terceira pessoa com auto-ativação"
        );
        assert!(
            desc.contains("auto-activate") || desc.contains("auto-ativar"),
            "skill {label} description DEVE declarar auto-ativação"
        );
    }

    // JSON-001 schema contract: password type includes null (not string-only).
    let schema =
        std::fs::read_to_string("docs/schemas/vps-show.schema.json").expect("vps-show.schema.json");
    let password_block = schema
        .split("\"password\"")
        .nth(1)
        .expect("schema deve declarar propriedade password");
    // First property after "password" key: type array must list string and null.
    let window: String = password_block.chars().take(120).collect();
    assert!(
        window.contains("null") && window.contains("string"),
        "vps-show.schema.json password deve permitir type null|string (JSON-001): {window}"
    );
}

// --- DENY-002 policy still yanked=deny ignore empty ---

#[test]
fn gap_deny_002_deny_toml_sem_ignore_cve() {
    let deny = std::fs::read_to_string("deny.toml").expect("deny.toml");
    assert!(deny.contains("yanked = \"deny\"") || deny.contains("yanked=\"deny\""));
    assert!(
        deny.contains("ignore = []") || deny.contains("ignore=[]"),
        "ignore deve permanecer vazio"
    );
    assert!(
        deny.contains("multiple-versions = \"warn\"") || deny.contains("GAP-SSH-DENY-002"),
        "política DENY-002 documentada"
    );
}

// --- CHG-001 / REL presence of changelog section ---

#[test]
fn gap_chg_001_changelog_tem_039() {
    let ch = std::fs::read_to_string("CHANGELOG.md").expect("CHANGELOG");
    assert!(ch.contains("## [0.3.9]"), "CHANGELOG deve ter seção 0.3.9");
    assert!(
        ch.contains("[0.3.9]:") || ch.contains("compare/v0.3.8"),
        "CHANGELOG deve ter âncora/link 0.3.9"
    );
}

// --- SEC-001..003: higiene anti-vazamento (auditoria workspace) ---

#[test]
fn gap_sec_001_setting_cyber_ignorado_por_diretorio() {
    let gi = std::fs::read_to_string(".gitignore").expect(".gitignore");
    assert!(
        gi.lines().any(|l| l.trim() == ".setting.cyber/"),
        ".gitignore DEVE ignorar o diretório .setting.cyber/ (não só *.log)"
    );
    let cargo = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
    assert!(
        cargo.contains("\".setting.cyber/\""),
        "Cargo.toml exclude DEVE listar .setting.cyber/"
    );
    let cargoignore = std::fs::read_to_string(".cargoignore").expect(".cargoignore");
    assert!(
        cargoignore.lines().any(|l| l.trim() == ".setting.cyber/"),
        ".cargoignore DEVE listar .setting.cyber/"
    );
}

#[test]
fn gap_sec_002_e2e_recusa_grok_config_dentro_do_repo() {
    let script = std::fs::read_to_string("scripts/e2e_real_ssh.sh").expect("e2e script");
    assert!(
        script.contains("must not live inside the repository"),
        "e2e_real_ssh.sh DEVE recusar grok config sob a raiz do repo"
    );
    assert!(
        script.contains("GROK_CFG_ABS") && script.contains("ROOT_ABS"),
        "e2e_real_ssh.sh DEVE comparar path absoluto do grok config com ROOT"
    );
    let testing = std::fs::read_to_string("docs/TESTING.md").expect("TESTING");
    assert!(
        testing.contains("$HOME/.grok/config.toml")
            && testing.contains("never copy it into this repository"),
        "TESTING.md DEVE documentar grok config só em $HOME"
    );
}

#[test]
fn gap_sec_003_docs_sem_s3cret_usa_placeholder_demo() {
    for path in [
        "README.md",
        "README.pt-BR.md",
        "docs/COOKBOOK.md",
        "docs/COOKBOOK.pt-BR.md",
    ] {
        let body = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("ler {path}: {e}"));
        assert!(
            !body.contains("s3cret"),
            "{path} não deve usar senha demo ambígua 's3cret'"
        );
        if body.contains("password-stdin") || body.contains("vps add") {
            assert!(
                body.contains("demo-password-not-real"),
                "{path} deve usar placeholder demo-password-not-real"
            );
        }
    }
    let sec = std::fs::read_to_string("SECURITY.md").expect("SECURITY");
    assert!(
        sec.contains(".setting.cyber/") && sec.contains("demo-password-not-real"),
        "SECURITY.md deve documentar higiene SEC-001/003"
    );
}
