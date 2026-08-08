// SPDX-License-Identifier: MIT OR Apache-2.0
//! SCP durability reporting (G-SCP-R01 / G-SCP-R02).
//!
//! # Why this suite exists
//!
//! The product documented mtime/mode preservation as a guarantee — "preserve mtime/mode
//! without extra flag" in both skills, both READMEs, both `llms` files — while the
//! implementation discarded the failure at two nesting levels: the `spawn_blocking` was
//! dropped with `let _ =`, and inside it the `set_times` was dropped again. The same
//! pattern hid the parent-directory fsync after the atomic rename.
//!
//! That combination is worse than either half. A build pipeline deciding whether to
//! recompile by comparing mtime could act on a timestamp that was never applied, and an
//! agent receiving exit 0 could not tell a durable write from one a power cut would
//! erase. The fix reports both outcomes instead of choosing between "lie" and "fail
//! transfers on filesystems that cannot represent a timestamp".

use ssh_cli::json_wire::ScpTransferJson;
use ssh_cli::ssh::client::TransferResult;

#[test]
fn transfer_result_defaults_to_no_loss() {
    // Uploads and SFTP paths never stamp a local file or rename into place, so their
    // default must be "nothing was attempted, therefore nothing was lost". Defaulting
    // to `false` would have reported spurious durability loss on every upload.
    let r = TransferResult::default();
    assert!(r.mtime_preserved);
    assert!(r.durable);
}

#[test]
fn the_two_flags_are_independent() {
    // They answer different questions: one about file metadata, one about the directory
    // entry. A filesystem can fail either without the other.
    let only_mtime_lost = TransferResult {
        bytes_transferred: 10,
        duration_ms: 1,
        mtime_preserved: false,
        durable: true,
    };
    let only_durability_lost = TransferResult {
        bytes_transferred: 10,
        duration_ms: 1,
        mtime_preserved: true,
        durable: false,
    };
    assert!(!only_mtime_lost.mtime_preserved && only_mtime_lost.durable);
    assert!(only_durability_lost.mtime_preserved && !only_durability_lost.durable);
}

#[test]
fn envelope_carries_both_flags_on_the_wire() {
    let v = ScpTransferJson {
        ok: true,
        event: "scp-transfer".into(),
        direction: "download".into(),
        vps: "host".into(),
        local: "/tmp/a".into(),
        remote: "/tmp/b".into(),
        bytes: 4096,
        duration_ms: 31,
        mtime_preserved: false,
        durable: false,
    };
    let json = serde_json::to_string(&v).expect("serialize");
    assert!(json.contains("\"mtime_preserved\":false"), "got: {json}");
    assert!(json.contains("\"durable\":false"), "got: {json}");
    assert!(
        !json.contains('\n'),
        "agent wire stays compact single-line: {json}"
    );
}

#[test]
fn events_written_before_the_fields_existed_still_deserialize() {
    // Additive contract: a consumer replaying a stored 0.5.4 event must not break, and
    // the absent fields must read as "no loss" rather than inventing a failure.
    let legacy = r#"{"ok":true,"event":"scp-transfer","direction":"upload","vps":"h",
        "local":"/a","remote":"/b","bytes":1,"duration_ms":2}"#;
    let v: ScpTransferJson = serde_json::from_str(legacy).expect("legacy event must parse");
    assert!(v.mtime_preserved);
    assert!(v.durable);
}

#[test]
fn schema_declares_both_fields_as_optional_booleans() {
    let schema = std::fs::read_to_string("docs/schemas/scp-transfer.schema.json")
        .expect("read scp-transfer.schema.json");
    let parsed: serde_json::Value = serde_json::from_str(&schema).expect("valid JSON Schema");

    for field in ["mtime_preserved", "durable"] {
        let prop = &parsed["properties"][field];
        assert_eq!(
            prop["type"], "boolean",
            "{field} must be contracted as a boolean"
        );
        // Deliberately outside `required`: adding them there would invalidate every
        // event emitted before this release.
        let required = parsed["required"]
            .as_array()
            .expect("required is an array")
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|r| r == field);
        assert!(
            !required,
            "{field} is additive and must stay out of `required`"
        );
    }
}

/// Documentation must no longer promise preservation unconditionally.
///
/// The gap asked for one of two things: make the failure fatal, or say best-effort.
/// Keeping the contradiction was called out as the worst of the three options, so this
/// asserts the docs actually moved rather than trusting a CHANGELOG line.
#[test]
fn public_docs_state_best_effort_rather_than_a_guarantee() {
    const FILES: &[&str] = &[
        "skills/ssh-cli-en/SKILL.md",
        "skills/ssh-cli-pt/SKILL.md",
        "docs/HOW_TO_USE.md",
        "docs/HOW_TO_USE.pt-BR.md",
        "docs/AGENTS.md",
        "docs/AGENTS.pt-BR.md",
        "README.md",
        "README.pt-BR.md",
        "llms.txt",
        "llms.pt-BR.txt",
        "llms-full.txt",
    ];

    for path in FILES {
        let body = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        assert!(
            body.contains("mtime_preserved"),
            "{path} must point readers at the field that reports the real outcome"
        );
        // The exact phrasings that claimed a guarantee, in both languages.
        for banned in [
            "preserve mtime/mode without extra flag",
            "preserve mtime/mode sem flag extra",
            "mtime/mode are preserved both directions automatically (remote",
        ] {
            assert!(
                !body.contains(banned),
                "{path} still promises unconditional preservation: {banned}"
            );
        }
    }
}
