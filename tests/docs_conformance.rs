// SPDX-License-Identifier: MIT OR Apache-2.0
//! Documentation conformance — prose, not behaviour.
//!
//! # Why this is a separate suite (G-QA-R03)
//!
//! These assertions check that published documents mention the surface the product
//! actually ships. They were previously mixed into `gaps_v040_integration.rs`
//! alongside assertions that drive the binary, under one policy. The consequence was
//! that rewriting a paragraph turned the build gate red with no functional regression
//! whatsoever — which is exactly what happened before 0.5.4, when `CONTRIBUTING.md`
//! stopped naming a renamed suite and `SKILL.md` lost one field name.
//!
//! When "suite red" can mean either "a bug shipped" or "someone edited a sentence",
//! the signal stops being actionable and the pressure is to silence the test. Keeping
//! prose here means a documentation drift is still caught, but it is never confused
//! with a behavioural regression.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn gap_doc_004_product_line_040_e_file_only() {
    let readme = std::fs::read_to_string(root().join("README.md")).expect("README");
    assert!(
        readme.contains("0.5.0") || readme.contains("0.4.0") || readme.contains("0.4.1"),
        "README must mention product line 0.5.x / 0.4.x history"
    );
    let lower = readme.to_lowercase();
    assert!(
        lower.contains("regular file")
            || lower.contains("file-only")
            || lower.contains("files only")
            || lower.contains("not directories"),
        "README must document file-only SCP limit"
    );
    assert!(
        readme.contains("scp-transfer") || readme.contains("tunnel_listening"),
        "README must surface scp-transfer and/or tunnel_listening for agents"
    );
    assert!(
        readme.contains(".ssh-cli.partial") || lower.contains("partial"),
        "README must document partial download path"
    );
}

#[test]
fn gap_doc_004_root_security_integrations_honest() {
    let sec = std::fs::read_to_string(root().join("SECURITY.md")).expect("SECURITY");
    assert!(
        (sec.contains("0.5.x") || sec.contains("0.5.0") || sec.contains("0.4.x"))
            && (sec.contains("current line") || sec.contains("current") || sec.contains("atual")),
        "SECURITY Supported Versions must brand current product line"
    );
    assert!(
        !sec.contains("| 0.3.x | Supported | Yes, current line |"),
        "SECURITY must not claim 0.3.x is the current product line"
    );
    let integ = std::fs::read_to_string(root().join("INTEGRATIONS.md")).expect("INTEGRATIONS");
    assert!(
        (integ.contains("0.5.0") || integ.contains("0.4.0") || integ.contains("0.4.1"))
            && (integ.contains("scp-transfer") || integ.contains("tunnel_listening")),
        "INTEGRATIONS 0.4.x must document real SCP/tunnel surface"
    );
    assert!(
        integ.contains("0.3.9"),
        "INTEGRATIONS must keep 0.3.9 residual facts under their own version bullet"
    );
    let llms_full = std::fs::read_to_string(root().join("llms-full.txt")).expect("llms-full");
    assert!(
        llms_full.contains("scp-transfer.schema.json"),
        "llms-full must index scp-transfer schema"
    );
    assert!(
        llms_full.contains("tunnel-listening.schema.json"),
        "llms-full must index tunnel-listening schema"
    );
    // Both languages, not just English. Asserting the English file alone is how
    // `CONTRIBUTING.pt-BR.md` came to omit `gaps_v040` and the entire cross-target
    // gate section while this suite stayed green: the reader who does not read
    // English was told strictly less, and no gate could see it.
    for doc in ["CONTRIBUTING.md", "CONTRIBUTING.pt-BR.md"] {
        let contrib = std::fs::read_to_string(root().join(doc)).expect("CONTRIBUTING");
        assert!(
            contrib.contains("gaps_v040"),
            "{doc} must mention gaps_v040 regression suite"
        );
        assert!(
            contrib.contains("E10") || contrib.contains("E01–E14") || contrib.contains("E01-E14"),
            "{doc} must mention official e2e SCP matrix E10+"
        );
    }
}

#[test]
fn gap_doc_004c_docs_folder_scp_tunnel_honest() {
    let agents = std::fs::read_to_string(root().join("docs/AGENTS.md")).expect("AGENTS");
    assert!(
        agents.contains("scp-transfer") && agents.contains("tunnel_listening"),
        "docs/AGENTS.md must document scp-transfer and tunnel_listening contracts"
    );
    assert!(
        agents.to_lowercase().contains("regular files only")
            || agents.contains("file-only")
            || agents.contains("no directories"),
        "docs/AGENTS.md must document SCP file-only"
    );
    let howto = std::fs::read_to_string(root().join("docs/HOW_TO_USE.md")).expect("HOW_TO_USE");
    assert!(
        howto.contains("0.3.9") && howto.contains(".ssh-cli.partial"),
        "docs/HOW_TO_USE.md must warn 0.3.9 and document partial downloads"
    );
    let cook = std::fs::read_to_string(root().join("docs/COOKBOOK.md")).expect("COOKBOOK");
    assert!(
        cook.contains("tunnel_listening") && cook.contains("scp-transfer"),
        "docs/COOKBOOK.md must include tunnel_listening and scp-transfer recipes"
    );
    let mig = std::fs::read_to_string(root().join("docs/MIGRATION.md")).expect("MIGRATION");
    assert!(
        mig.contains("tunnel_listening")
            && mig.contains(".ssh-cli.partial")
            && mig.contains("32 KiB"),
        "docs/MIGRATION.md 0.4.0 section must cover tunnel JSON, partial, stream"
    );
    for doc in ["docs/TESTING.md", "docs/TESTING.pt-BR.md"] {
        let testing = std::fs::read_to_string(root().join(doc)).expect("TESTING");
        assert!(
            testing.contains("gaps_v040")
                && (testing.contains("E10")
                    || testing.contains("E01–E14")
                    || testing.contains("E01-E14")),
            "{doc} must list gaps_v040 and e2e E10+"
        );
    }
    for doc in [
        "docs/RELEASE_CHECKLIST.md",
        "docs/RELEASE_CHECKLIST.pt-BR.md",
    ] {
        let release = std::fs::read_to_string(root().join(doc)).expect("RELEASE");
        assert!(
            release.contains("gaps_v040") && release.contains("DOC-004"),
            "{doc} must gate gaps_v040 and DOC-004"
        );
    }
    let cross = std::fs::read_to_string(root().join("docs/CROSS_PLATFORM.md")).expect("CROSS");
    let cross_l = cross.to_lowercase();
    assert!(
        cross.contains(".ssh-cli.partial")
            && (cross_l.contains("regular files only")
                || cross.contains("file-only")
                || cross_l.contains("regular files")),
        "docs/CROSS_PLATFORM.md must document SCP portability"
    );
    let schema_idx =
        std::fs::read_to_string(root().join("docs/schemas/README.md")).expect("schemas README");
    assert!(
        schema_idx.contains("scp-transfer.schema.json")
            && schema_idx.contains("tunnel-listening.schema.json"),
        "docs/schemas/README.md must index scp-transfer and tunnel-listening"
    );
    assert!(
        root()
            .join("docs/schemas/tunnel-listening.schema.json")
            .is_file(),
        "missing tunnel-listening.schema.json"
    );
}

#[test]
fn gap_doc_004d_skills_scp_tunnel_honest() {
    // Skills must teach agents the 0.4.0 scp/tunnel contracts without version-story prose.
    for rel in ["skills/ssh-cli-en/SKILL.md", "skills/ssh-cli-pt/SKILL.md"] {
        let body = std::fs::read_to_string(root().join(rel)).expect(rel);
        let lower = body.to_ascii_lowercase();
        assert!(
            body.contains("tunnel_listening"),
            "{rel} must document tunnel_listening ready event"
        );
        assert!(
            body.contains(".ssh-cli.partial"),
            "{rel} must document partial download path"
        );
        assert!(
            body.contains("32 KiB") || body.contains("32KiB"),
            "{rel} must document 32 KiB upload stream"
        );
        assert!(
            lower.contains("files-only")
                || lower.contains("file-only")
                || lower.contains("regular-file")
                || lower.contains("regular file")
                || body.contains("somente-arquivo")
                || body.contains("só-arquivo")
                || body.contains("arquivo regular"),
            "{rel} must document scp regular-files-only"
        );
        assert!(
            body.contains("ok")
                && body.contains("direction")
                && body.contains("bytes")
                && body.contains("duration_ms"),
            "{rel} must document scp-transfer success fields"
        );
        assert!(
            body.contains("local_port")
                && body.contains("remote_host")
                && body.contains("remote_port")
                && body.contains("timeout_ms"),
            "{rel} must document tunnel_listening fields"
        );
        assert!(
            body.contains("scp upload")
                && body.contains("--json")
                && body.contains("tunnel")
                && body.contains("--timeout-ms"),
            "{rel} must include scp --json and tunnel --timeout-ms formulas"
        );
        assert!(
            !body.contains("0.4.0 did")
                && !body.contains("0.3.9 did")
                && !body.contains("in version 0.3.9")
                && !body.contains("versão 0.3.9")
                && !body.contains("na versão 0.3.9"),
            "{rel} must stay consolidated without version-story prose"
        );
        let fm = body
            .strip_prefix("---\n")
            .and_then(|s| s.split_once("\n---"))
            .map(|(a, _)| a)
            .expect("frontmatter");
        let desc = fm
            .lines()
            .find(|l| l.starts_with("description:"))
            .expect("description")
            .trim_start_matches("description:")
            .trim();
        assert!(
            desc.chars().count() < 1024,
            "{rel} description must be < 1024 chars (got {})",
            desc.chars().count()
        );
        assert_eq!(
            desc.matches(':').count(),
            0,
            "{rel} description must not contain ':' in content"
        );
        assert!(
            desc.contains("tunnel_listening")
                && (desc.contains("files-only")
                    || desc.contains("só-arquivo")
                    || desc.contains("file-only")
                    || desc.contains("regular")),
            "{rel} description must surface scp file-only + tunnel_listening for auto-activation"
        );
    }
    for rel in [
        "skills/ssh-cli-en/evals/queries.json",
        "skills/ssh-cli-pt/evals/queries.json",
    ] {
        let q = std::fs::read_to_string(root().join(rel)).expect(rel);
        assert!(
            q.contains("tunnel_listening")
                && (q.contains(".ssh-cli.partial") || q.contains("ssh-cli.partial"))
                && (q.contains("files only")
                    || q.contains("regular files")
                    || q.contains("arquivos regulares")
                    || q.contains("somente arquivo")
                    || q.contains("directory")
                    || q.contains("diretorio")),
            "{rel} evals must cover tunnel_listening + partial + file-only surface"
        );
    }
}

#[test]
fn gap_rel_004_changelog_039_scp_broken_e_040() {
    let ch = std::fs::read_to_string(root().join("CHANGELOG.md")).expect("CHANGELOG");
    assert!(ch.contains("0.4.0"), "CHANGELOG must have 0.4.0 section");
    let lower = ch.to_lowercase();
    assert!(
        lower.contains("0.3.9")
            && (lower.contains("broken") || lower.contains("inoperant") || lower.contains("wire")),
        "CHANGELOG must honestly mention 0.3.9 SCP wire issue"
    );
}

// ── LOTE F: EN / pt-BR parity ──────────────────────────────────────────────

/// Every agent-facing token must appear in **both** skills.
///
/// G-DOC-R02 was an asymmetry exactly like this: `timeout_ms` was documented in the
/// English skill and missing from the Portuguese one, and the test that should have
/// caught it asserted `contains("ok")` — a substring that also matches "token" and
/// "protocolo", so it was satisfied by prose that said nothing. An agent reading the
/// pt-BR skill therefore had a strictly smaller contract than one reading the EN
/// skill, with no signal anywhere that the two had diverged.
///
/// Tokens are compared, not sentences: the two files are written in different
/// languages, so requiring equal prose would be meaningless, while requiring the same
/// flags, events and field names is exactly the parity that matters.
#[test]
fn skills_en_and_pt_document_the_same_agent_surface() {
    let en = std::fs::read_to_string(root().join("skills/ssh-cli-en/SKILL.md")).expect("EN skill");
    let pt =
        std::fs::read_to_string(root().join("skills/ssh-cli-pt/SKILL.md")).expect("pt-BR skill");

    let tokens = [
        // Global agent-native surface
        "--select",
        "--filter",
        "--limit",
        "--sort",
        "--dedupe-by",
        "--count-only",
        "--truncate-content",
        "--max-output-bytes",
        "--no-input",
        "--dry-run",
        "--i-accept-network-exposure",
        // Tunnel modes (0.5.4)
        "--socks5",
        "--remote-socket",
        "--reverse",
        "--timeout-ms",
        // Wire events and discriminators
        "tunnel_listening",
        "tunnel_closed",
        "dry-run",
        "socks5",
        "streamlocal",
        "reverse",
        "local_port",
        "capacity_waits",
        "forwards_served",
        "executed",
        "replaces_existing",
        "hosts_to_reencrypt",
        // scp-transfer fields. The bare token `ok` is worthless here — it is a
        // substring of "token" and "protocolo", which is the exact hole that let
        // G-DOC-R02 through — so the delimited run is asserted instead. It is
        // language-neutral: both skills write the field list the same way.
        "`ok`/`direction`",
        "mtime_preserved",
        "durable",
        // Commands that implement --dry-run
        "vps remove",
        "vps import",
        "sftp rm",
        "sftp rmdir",
        "secrets init",
        "secrets reencrypt",
    ];

    let mut missing_en = Vec::new();
    let mut missing_pt = Vec::new();
    for token in tokens {
        if !en.contains(token) {
            missing_en.push(token);
        }
        if !pt.contains(token) {
            missing_pt.push(token);
        }
    }
    assert!(
        missing_en.is_empty(),
        "EN skill is missing agent-facing tokens: {missing_en:?}"
    );
    assert!(
        missing_pt.is_empty(),
        "pt-BR skill is missing agent-facing tokens: {missing_pt:?}"
    );
}

/// Both changelogs must describe the same release surface.
///
/// The changelog is what a consumer reads before upgrading; a BREAKING entry present
/// in one language and absent in the other is a trap for exactly the readers who do
/// not read English.
#[test]
fn changelogs_describe_the_same_release_surface() {
    let en = std::fs::read_to_string(root().join("CHANGELOG.md")).expect("CHANGELOG");
    let pt = std::fs::read_to_string(root().join("CHANGELOG.pt-BR.md")).expect("CHANGELOG pt-BR");

    for token in [
        "0.5.4",
        "--dry-run",
        "--socks5",
        "--remote-socket",
        "--reverse",
        "G-TUN-R01",
        "G-TUN-R02",
        "G-TUN-R03",
        "B2",
        "BREAKING",
        "direct-streamlocal@openssh.com",
        "RFC 1928",
    ] {
        assert!(en.contains(token), "CHANGELOG.md must mention {token}");
        assert!(
            pt.contains(token),
            "CHANGELOG.pt-BR.md must mention {token}"
        );
    }

    // Both must carry the same number of BREAKING markers: one language silently
    // dropping a breaking change is the failure this guards.
    let breaking_en = en.matches("BREAKING").count();
    let breaking_pt = pt.matches("BREAKING").count();
    assert_eq!(
        breaking_en, breaking_pt,
        "BREAKING count diverges: EN {breaking_en}, pt-BR {breaking_pt}"
    );
}

// ── LOTE G: the 0.5.4 surface must reach the documents people actually read ──

/// Per-document token contract for the 0.5.4 surface.
///
/// The contract is deliberately **per document** rather than one blanket list.
/// Requiring every token in every file would be noise that trains a maintainer to
/// paste sentences where they do not belong: the cookbook owes a runnable recipe,
/// the migration guide owes an upgrade note, the security policy owes the exposure
/// acknowledgement, and the cross-platform guide owes the portability caveat. Each
/// entry below is what that document is uniquely responsible for.
///
/// Tokens are flags, event names and wire labels, so the same list is valid for a
/// pt-BR file: parity of contract does not require parity of prose.
const SURFACE_054: &[(&str, &[&str])] = &[
    // Discovery surface: the first thing a human or an agent reads.
    (
        "README.md",
        &[
            "--reverse",
            "--socks5",
            "--remote-socket",
            "--i-accept-network-exposure",
            "tunnel_closed",
            "--select",
            "--count-only",
        ],
    ),
    (
        "README.pt-BR.md",
        &[
            "--reverse",
            "--socks5",
            "--remote-socket",
            "--i-accept-network-exposure",
            "tunnel_closed",
            "--select",
            "--count-only",
        ],
    ),
    (
        "llms.txt",
        &[
            "0.5.4",
            "--reverse",
            "--socks5",
            "--remote-socket",
            "--i-accept-network-exposure",
            "tunnel_closed",
        ],
    ),
    (
        "llms.pt-BR.txt",
        &[
            "0.5.4",
            "--reverse",
            "--socks5",
            "--remote-socket",
            "--i-accept-network-exposure",
            "tunnel_closed",
        ],
    ),
    // Announces itself as the complete discovery map, so it owes the most.
    (
        "llms-full.txt",
        &[
            "--reverse",
            "--socks5",
            "--remote-socket",
            "--i-accept-network-exposure",
            "tunnel_closed",
            "--select",
            "--dry-run",
            "--no-input",
            "streamlocal",
        ],
    ),
    // The agent contract: every shaping flag, because an agent that does not know
    // a flag exists pays the token cost the flag was built to avoid.
    (
        "docs/AGENTS.md",
        &[
            "--select",
            "--filter",
            "--limit",
            "--sort",
            "--dedupe-by",
            "--count-only",
            "--truncate-content",
            "--max-output-bytes",
            "--dry-run",
            "--no-input",
            "--reverse",
            "--socks5",
            "--remote-socket",
            "tunnel_closed",
        ],
    ),
    (
        "docs/AGENTS.pt-BR.md",
        &[
            "--select",
            "--filter",
            "--limit",
            "--sort",
            "--dedupe-by",
            "--count-only",
            "--truncate-content",
            "--max-output-bytes",
            "--dry-run",
            "--no-input",
            "--reverse",
            "--socks5",
            "--remote-socket",
            "tunnel_closed",
        ],
    ),
    (
        "docs/HOW_TO_USE.md",
        &[
            "--reverse",
            "--socks5",
            "--remote-socket",
            "--i-accept-network-exposure",
            "--select",
            "--dry-run",
            "tunnel_closed",
        ],
    ),
    (
        "docs/HOW_TO_USE.pt-BR.md",
        &[
            "--reverse",
            "--socks5",
            "--remote-socket",
            "--i-accept-network-exposure",
            "--select",
            "--dry-run",
            "tunnel_closed",
        ],
    ),
    // The recipe book: a mode with no recipe is a mode nobody runs.
    (
        "docs/COOKBOOK.md",
        &[
            "--reverse",
            "--socks5",
            "--remote-socket",
            "--i-accept-network-exposure",
            "--select",
            "--count-only",
        ],
    ),
    (
        "docs/COOKBOOK.pt-BR.md",
        &[
            "--reverse",
            "--socks5",
            "--remote-socket",
            "--i-accept-network-exposure",
            "--select",
            "--count-only",
        ],
    ),
    (
        "INTEGRATIONS.md",
        &["tunnel_closed", "--reverse", "--socks5", "--remote-socket"],
    ),
    (
        "INTEGRATIONS.pt-BR.md",
        &["tunnel_closed", "--reverse", "--socks5", "--remote-socket"],
    ),
    // The upgrade path: both BREAKING changes plus the new modes.
    (
        "docs/MIGRATION.md",
        &[
            "0.5.4",
            "--i-accept-network-exposure",
            "--reverse",
            "--socks5",
            "--remote-socket",
        ],
    ),
    (
        "docs/MIGRATION.pt-BR.md",
        &[
            "0.5.4",
            "--i-accept-network-exposure",
            "--reverse",
            "--socks5",
            "--remote-socket",
        ],
    ),
    // The exposure guard is a security control, so the security policy must name it.
    ("SECURITY.md", &["--i-accept-network-exposure", "0.5.4"]),
    (
        "SECURITY.pt-BR.md",
        &["--i-accept-network-exposure", "0.5.4"],
    ),
    // A remote Unix socket target is exactly the kind of fact a portability guide
    // exists to settle: the *client* may be Windows, the *socket* may not.
    (
        "docs/CROSS_PLATFORM.md",
        &["--remote-socket", "streamlocal"],
    ),
    (
        "docs/CROSS_PLATFORM.pt-BR.md",
        &["--remote-socket", "streamlocal"],
    ),
    // A release checklist that does not check the release is the purest form of the
    // defect this file exists to catch. Measured: both checklists mentioned
    // `tunnel_closed`, `--select` and `--count-only` on line 3 *only* — the release
    // banner — and named none of the three tunnel modes anywhere. The banner is an
    // announcement; the checklist is the contract, and it was empty.
    (
        "docs/RELEASE_CHECKLIST.md",
        &[
            "--reverse",
            "--socks5",
            "--remote-socket",
            "SFTP_PERM_MASK_UNTRUSTED",
        ],
    ),
    (
        "docs/RELEASE_CHECKLIST.pt-BR.md",
        &[
            "--reverse",
            "--socks5",
            "--remote-socket",
            "SFTP_PERM_MASK_UNTRUSTED",
        ],
    ),
    (
        "docs/TESTING.md",
        &["--reverse", "--socks5", "--remote-socket"],
    ),
    (
        "docs/TESTING.pt-BR.md",
        &["--reverse", "--socks5", "--remote-socket"],
    ),
];

/// Documents that own the fleet surface, and therefore owe every selector.
///
/// `--all` and `--hosts` were documented everywhere; `--tags` was documented
/// nowhere — zero occurrences across all fifteen files under `docs/`. The flag is
/// real on `exec`, `sudo-exec` and `su-exec`, and it is the only selector that
/// addresses a fleet by label. An agent reading these files concludes tag fan-out
/// does not exist and opens N processes instead of one.
const FLEET_SELECTOR_DOCS: &[&str] = &[
    "docs/AGENTS.md",
    "docs/AGENTS.pt-BR.md",
    "docs/HOW_TO_USE.md",
    "docs/HOW_TO_USE.pt-BR.md",
    "docs/COOKBOOK.md",
    "docs/COOKBOOK.pt-BR.md",
];

/// Every fleet selector clap accepts must appear where the fleet is documented.
#[test]
fn fleet_documents_name_every_selector() {
    let mut missing = Vec::new();
    for doc in FLEET_SELECTOR_DOCS {
        let text = std::fs::read_to_string(root().join(doc))
            .unwrap_or_else(|e| panic!("cannot read {doc}: {e}"));
        for selector in ["--all", "--hosts", "--tags"] {
            if !text.contains(selector) {
                missing.push(format!("{doc}: {selector}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "these documents describe fleet execution without naming every selector:\n  {}\n\
         `--tags` is declared on exec/sudo-exec/su-exec in src/cli/commands.rs. A \
         selector absent from the docs is a selector the agent never uses.",
        missing.join("\n  ")
    );
}

/// The SFTP permission mask is directional, and half of it is the security fix.
///
/// `SFTP_PERM_MASK` (`0o7777`, src/constants.rs) applies **outbound** on upload and
/// deliberately preserves setuid/setgid/sticky on a local file the caller already
/// owns. `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) applies **inbound** on download,
/// where the mode arrives from the server and elevation bits must not survive.
///
/// A document that names `0o7777` alone teaches that server-sent elevation bits are
/// preserved on download — precisely the condition A3 removed. Measured: twelve such
/// lines across ten files, several in the same file that announces A3 twenty lines
/// above. Naming one mask without the other is worse than naming neither, because
/// the reader concludes the fix does not exist.
///
/// This gate was scoped wrong twice before it was right. First it walked `docs/`
/// alone — the directory being edited — and went green while the identical claim
/// stood in `llms.txt`, `llms.pt-BR.txt`, `llms-full.txt` and both `SKILL.md`.
/// Widening it to a hand-written list of four directories then missed
/// `docs/schemas/README.md`, because `read_dir` does not recurse. A hand-written
/// list of places to look is itself a thing that falls behind the disk, so the walk
/// is now recursive: a document added in a new subdirectory tomorrow is covered
/// without anyone remembering to add it here.
#[test]
fn permission_mask_claims_are_directional() {
    /// Collects every publishable document under `dir`, recursively.
    fn collect(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for path in entries.filter_map(|e| e.ok().map(|e| e.path())) {
            if path.is_dir() {
                // Build output and VCS internals publish nothing.
                if path
                    .file_name()
                    .is_some_and(|n| n == "target" || n == ".git")
                {
                    continue;
                }
                collect(&path, out);
            } else if path.extension().is_some_and(|x| x == "md" || x == "txt") {
                out.push(path);
            }
        }
    }
    let mut offenders = Vec::new();
    let mut docs: Vec<std::path::PathBuf> = Vec::new();
    collect(&root(), &mut docs);
    // `read_dir` order is platform- and filesystem-dependent; sort so the failure
    // message reads identically on two hosts looking at the same tree.
    docs.sort();
    for path in &docs {
        // CLAUDE.md is the operator's own instruction file, not a product document.
        if path.file_name().is_some_and(|n| n == "CLAUDE.md") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if text.contains("0o7777") && !text.contains("SFTP_PERM_MASK_UNTRUSTED") {
            let name = path
                .strip_prefix(root())
                .unwrap_or(path)
                .to_string_lossy()
                .into_owned();
            offenders.push(name);
        }
    }
    assert!(
        offenders.is_empty(),
        "these documents name the outbound mask `0o7777` without the inbound mask:\n  {}\n\
         Always qualify the direction and cite both constants: `SFTP_PERM_MASK` \
         (0o7777, upload) and `SFTP_PERM_MASK_UNTRUSTED` (0o0777, download).",
        offenders.join("\n  ")
    );
}

/// The 0.5.4 headline surface must reach every document a user or agent reads.
///
/// This suite already asserted these tokens — but only over
/// `skills/ssh-cli-{en,pt}/SKILL.md` and the two changelogs. The gap that followed
/// was measured, not hypothetical: `llms-full.txt` calls itself the complete
/// discovery map and mentioned none of `--reverse`, `--socks5` or
/// `--remote-socket`, and `docs/HOW_TO_USE` and `docs/COOKBOOK` mentioned none of
/// the three either. Three shipped features were invisible to every reader who did
/// not open the skill package, and this file was green the whole time.
///
/// A gate that watches the wrong subset measures the wrong thing. Adding a token to
/// [`SURFACE_054`] is how a future release makes its own surface non-optional.
#[test]
fn the_054_surface_reaches_every_user_facing_document() {
    let mut missing = Vec::new();

    for (doc, tokens) in SURFACE_054 {
        let text = std::fs::read_to_string(root().join(doc))
            .unwrap_or_else(|e| panic!("cannot read {doc}: {e}"));
        for token in *tokens {
            if !text.contains(token) {
                missing.push(format!("{doc}: {token}"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "these documents do not mention 0.5.4 surface they are responsible for:\n  {}\n\
         Document the feature where the reader will look for it. A release note in \
         the banner is an announcement, not a contract.",
        missing.join("\n  ")
    );
}

/// Every leaf command of the CLI, as full invocation paths.
///
/// Kept as literal strings rather than derived from `ssh-cli commands` because this
/// suite is prose-only by design and never drives the binary (see the module docs).
/// The count assertion below is what keeps the list honest: a new subcommand changes
/// the tree, and a maintainer who adds one without touching this list gets a red gate
/// naming the mismatch instead of silently shipping an undocumented command.
const LEAF_COMMANDS: &[&str] = &[
    "vps add",
    "vps list",
    "vps remove",
    "vps edit",
    "vps show",
    "vps path",
    "vps doctor",
    "vps export",
    "vps import",
    "connect",
    "exec",
    "sudo-exec",
    "su-exec",
    "scp upload",
    "scp download",
    "sftp upload",
    "sftp download",
    "sftp ls",
    "sftp mkdir",
    "sftp rmdir",
    "sftp rm",
    "sftp stat",
    "sftp rename",
    "tunnel",
    "health-check",
    "secrets status",
    "secrets init",
    "secrets reencrypt",
    "completions",
    "commands",
    "schema",
    "doctor",
    "locale show",
    "locale set",
    "locale clear",
    "tls provider",
    "tls paths",
    "tls mtls list",
    "tls mtls import",
    "tls mtls show",
    "tls mtls remove",
    "tls acme account create",
    "tls acme account show",
    "tls acme issue",
    "tls acme complete",
    "tls acme status",
    "tls acme list",
];

/// Documents that claim a *complete* command inventory and must therefore contain
/// every leaf as a literally searchable string.
///
/// Compact brace notation (`vps {add,list,…}`) is fine in a short index a human skims,
/// but these files are read by retrieval: an agent grepping `tls acme account create`
/// finds nothing in a brace list, and concludes the command does not exist.
const FULL_INVENTORY_DOCS: &[&str] = &[
    "llms-full.txt",
    "docs/AGENTS.md",
    "docs/AGENTS.pt-BR.md",
    "docs/HOW_TO_USE.md",
    "docs/HOW_TO_USE.pt-BR.md",
    "docs/CROSS_PLATFORM.md",
    "docs/CROSS_PLATFORM.pt-BR.md",
];

/// The two files an agent loads before anything else.
const SKILLS: &[&str] = &["skills/ssh-cli-en/SKILL.md", "skills/ssh-cli-pt/SKILL.md"];

/// Surfaces that describe the `vps export` body and are not in the other two lists.
///
/// Kept separate rather than folded into `FULL_INVENTORY_DOCS`, because that constant also
/// carries the full-command-inventory obligation and these files do not all claim it.
const EXPORT_CLAIM_DOCS: &[&str] = &[
    "README.md",
    "README.pt-BR.md",
    "llms.txt",
    "llms.pt-BR.txt",
    "INTEGRATIONS.md",
    "INTEGRATIONS.pt-BR.md",
    "docs/COOKBOOK.md",
    "docs/COOKBOOK.pt-BR.md",
    "docs/MIGRATION.md",
    "docs/MIGRATION.pt-BR.md",
    "docs/TESTING.md",
    "docs/TESTING.pt-BR.md",
    "docs/RELEASE_CHECKLIST.md",
    "docs/RELEASE_CHECKLIST.pt-BR.md",
    "docs/schemas/README.md",
];

/// Every CLI command must be documented in every document claiming a full inventory.
#[test]
fn every_command_appears_in_every_full_inventory_document() {
    assert_eq!(
        LEAF_COMMANDS.len(),
        47,
        "LEAF_COMMANDS drifted from the shipped tree; run `ssh-cli commands` and \
         reconcile, then document the new command in every file in FULL_INVENTORY_DOCS"
    );

    let mut missing = Vec::new();
    for doc in FULL_INVENTORY_DOCS {
        let text = std::fs::read_to_string(root().join(doc))
            .unwrap_or_else(|e| panic!("cannot read {doc}: {e}"));
        for cmd in LEAF_COMMANDS {
            if !text.contains(cmd) {
                missing.push(format!("{doc}: {cmd}"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "these documents claim a complete command inventory but omit commands:\n  {}\n\
         Write the full path (`tls acme account create`), not brace notation \
         (`tls acme {{account create,…}}`): a retriever matches literal strings.",
        missing.join("\n  ")
    );
}

/// Every JSON schema on disk must also be indexed in the full LLM discovery map.
///
/// `docs/schemas/README.md` was already gated, but `llms-full.txt` was not, and the
/// drift was measured: 22 schemas existed on disk while that file indexed 14. The eight
/// it omitted were `dry-run` plus every `sftp-*` and `*-batch` contract — precisely the
/// envelopes an agent needs to parse fleet and SFTP work. A discovery map that silently
/// covers two thirds of the contracts is worse than none, because the reader stops
/// looking after consulting it.
#[test]
fn every_schema_is_indexed_in_the_full_llm_map() {
    let dir = root().join("docs/schemas");
    let map = std::fs::read_to_string(root().join("llms-full.txt")).expect("llms-full");

    let mut schemas: Vec<String> = std::fs::read_dir(&dir)
        .expect("read schemas dir")
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".schema.json"))
        .collect();
    // `read_dir` order is platform- and filesystem-dependent and may change between
    // calls, so sort before asserting: otherwise the failure message reads differently
    // on two hosts looking at the same tree.
    schemas.sort_unstable();
    assert!(!schemas.is_empty(), "no schemas found to index");

    let missing: Vec<&String> = schemas
        .iter()
        .filter(|n| !map.contains(n.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "llms-full.txt indexes {} of {} schemas; it omits: {missing:?}\n\
         It advertises itself as the complete discovery map, so every contract on disk \
         must be listed there.",
        schemas.len() - missing.len(),
        schemas.len()
    );
}

/// Every JSON schema on disk must be indexed in both language sections of the README.
#[test]
fn every_schema_is_indexed_in_both_languages() {
    let dir = root().join("docs/schemas");
    let readme = std::fs::read_to_string(dir.join("README.md")).expect("schemas README");

    let mut schemas: Vec<String> = std::fs::read_dir(&dir)
        .expect("read schemas dir")
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".schema.json"))
        .collect();
    schemas.sort();
    assert!(!schemas.is_empty(), "no schemas found to index");

    for name in &schemas {
        // Counted, not merely present: a single mention means one language section
        // documents it and the other does not.
        let mentions = readme.matches(name.as_str()).count();
        assert!(
            mentions >= 2,
            "{name} is mentioned {mentions} time(s) in docs/schemas/README.md; \
             both the English and the pt-BR section must index it"
        );
    }
}

/// The two skills are the files an agent loads first, so they carry a hard size cap.
///
/// The cap is a standing product rule and nothing enforced it: both skills were measured
/// at 4906 and 5008 words against a 4000-word budget, over by a fifth, and every gate in
/// this file stayed green because none of them counted anything. That is the same defect
/// class this file exists to catch — a rule with no gate is indistinguishable from no
/// rule — and it is worse here than elsewhere, because a skill that overruns its budget
/// silently displaces the context the agent needed for the task itself.
///
/// Counted the way the budget is stated, in whitespace-separated words over the whole
/// file including frontmatter, so the number in the failure message is the number a
/// human gets from `wc -w` and there is nothing to reconcile.
const SKILL_WORD_BUDGET: usize = 4000;

#[test]
fn skills_stay_within_the_word_budget() {
    for rel in ["skills/ssh-cli-en/SKILL.md", "skills/ssh-cli-pt/SKILL.md"] {
        let body = std::fs::read_to_string(root().join(rel)).expect(rel);
        let words = body.split_whitespace().count();
        assert!(
            words <= SKILL_WORD_BUDGET,
            "{rel} has {words} words, over the {SKILL_WORD_BUDGET}-word budget by {}. \
             Cut duplicated prose before cutting formulas: the formulas are the part an \
             agent copies, while a prohibition that merely mirrors a REQUIRED bullet in \
             the same section teaches nothing.",
            words - SKILL_WORD_BUDGET
        );
    }
}

/// Both skills claim the whole command surface, so both must actually name all of it.
///
/// `FULL_INVENTORY_DOCS` deliberately lists the seven long-form documents, and the two
/// skills were never in it — yet each skill opens its catalog by claiming to be the whole
/// surface. The claim was true when measured, but nothing held it true: a forty-eighth
/// command could ship, every gate here would pass, and the two files an agent reads
/// first would quietly describe a smaller CLI than the one installed.
///
/// Asserted separately from `FULL_INVENTORY_DOCS` rather than by appending to it, because
/// that constant is also used to argue against brace notation in prose documents, and a
/// skill is a different kind of artifact with a different budget.
#[test]
fn skills_name_every_command() {
    let mut missing = Vec::new();
    for rel in ["skills/ssh-cli-en/SKILL.md", "skills/ssh-cli-pt/SKILL.md"] {
        let body = std::fs::read_to_string(root().join(rel)).expect(rel);
        for cmd in LEAF_COMMANDS {
            if !body.contains(cmd) {
                missing.push(format!("{rel}: {cmd}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "these skills claim the whole command surface but omit commands:\n  {}\n\
         Write the full path (`tls acme account create`): an agent greps literal strings \
         and concludes an unmatched command does not exist.",
        missing.join("\n  ")
    );
}

/// No document may claim the `vps export` body is TOML by default.
///
/// It is not, and the claim survived three releases across roughly twenty files. The body
/// follows the resolved output format and that resolves to JSON whenever stdout is not a
/// TTY, which is every agent invocation; measured, `vps export -o hosts.toml` writes a JSON
/// envelope into a file named `.toml`, and only `--output-format text` yields TOML.
///
/// What let the falsehood spread is instructive and is the reason this gate is phrase-based
/// rather than command-based. `tests/gaps_v051_integration.rs` held BOTH
/// `export_pipe_defaults_to_json_when_non_tty`, which proved the truth, and a test then named
/// `export_import_toml_roundtrip`, which asserted only that `sshcli-enc:` was absent and that
/// import accepted the file. Since import accepts TOML *and* JSON envelopes, the second test
/// closed on the wrong body while its name advertised coverage of the exact claim that was
/// false. A green suite therefore said nothing, and the prose drifted unchallenged.
///
/// The CHANGELOG history is exempt on purpose: those entries record what a past release
/// claimed, and rewriting them would erase the evidence that the claim was ever made.
#[test]
fn no_document_claims_export_defaults_to_toml() {
    const BANNED: &[&str] = &[
        "TOML by default",
        "stays TOML",
        "export TOML default",
        "body is TOML",
        "default body is TOML",
        "TOML por padrão",
        "permanece TOML",
        "export TOML padrão",
        "corpo padrão é TOML",
        "padrão de `vps export` é TOML",
    ];
    let mut hits = Vec::new();
    for doc in FULL_INVENTORY_DOCS
        .iter()
        .chain(SKILLS.iter())
        .chain(EXPORT_CLAIM_DOCS.iter())
    {
        let path = root().join(doc);
        let Ok(body) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (idx, line) in body.lines().enumerate() {
            for phrase in BANNED {
                if line.contains(phrase) {
                    hits.push(format!("{doc}:{}: {phrase}", idx + 1));
                }
            }
        }
    }
    assert!(
        hits.is_empty(),
        "these lines claim a TOML default that the binary does not have:\n  {}\n\
         The body follows the resolved output format: JSON on any non-TTY stdout, even into \
         a `.toml` filename, and TOML only with `--output-format text`. Say that instead.",
        hits.join("\n  ")
    );
}
