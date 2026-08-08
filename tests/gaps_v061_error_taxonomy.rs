// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error taxonomy after G-ERR-R01 (exit 69 / 70) and the envelope contract.
//!
//! # Why this suite exists
//!
//! `SshCliError::Config` — exit 65, classified permanent — had become the landing spot
//! for every `map_err` without an obvious variant. Sixteen of its call sites lived in
//! `src/secrets.rs` alone, covering three unrelated failure modes:
//!
//! - a CSPRNG that would not produce bytes (a broken host),
//! - a full or read-only disk while writing `secrets.key` (I/O),
//! - an OS keyring that was locked or not running (a service being down).
//!
//! All three told an agent the same thing: "your input is malformed, do not retry".
//! For the keyring that advice is exactly backwards — retrying is the remedy — so the
//! one recoverable failure in the group was the one agents were told to give up on.
//!
//! These assertions drive the real product API, so deleting a variant or flipping a
//! classification turns the suite red instead of silently changing agent behaviour.

use ssh_cli::errors::{exit_codes, ErrorClass, SshCliError};

#[test]
fn unavailable_service_exits_69_and_is_retryable() {
    let e = SshCliError::unavailable("keyring");

    assert_eq!(e.exit_code(), exit_codes::EX_UNAVAILABLE);
    assert_eq!(e.error_code(), "unavailable");
    assert_eq!(e.classify(), ErrorClass::Transient);
    assert!(
        e.is_retryable(),
        "a locked keyring answers the same argv once it is unlocked; \
         reporting it permanent is what made agents abandon a recoverable failure"
    );
}

#[test]
fn software_failure_exits_70_and_is_not_retryable() {
    let e = SshCliError::software("rng");

    assert_eq!(e.exit_code(), exit_codes::EX_SOFTWARE);
    assert_eq!(e.error_code(), "software");
    assert_eq!(e.classify(), ErrorClass::Permanent);
    assert!(
        !e.is_retryable(),
        "no amount of waiting repairs a CSPRNG, so this must stay permanent \
         even though it is not the caller's fault"
    );
}

#[test]
fn unavailable_and_software_do_not_collide_with_the_old_data_error() {
    // The whole point of the split: three codes where there used to be one.
    let unavailable = SshCliError::unavailable("keyring").exit_code();
    let software = SshCliError::software("rng").exit_code();
    let data = SshCliError::Config("malformed".into()).exit_code();

    assert_eq!(data, exit_codes::EX_DATAERR);
    assert_ne!(unavailable, data);
    assert_ne!(software, data);
    assert_ne!(unavailable, software);
}

#[test]
fn the_two_new_variants_carry_actionable_suggestions() {
    // `suggestion()` is derived from `RetryKind`, and both variants share theirs with
    // unrelated failures. Without a variant-level override, a locked keyring would be
    // told to raise `--timeout` — advice pointing at the wrong knob entirely.
    let keyring = SshCliError::unavailable("keyring")
        .suggestion()
        .expect("unavailable must suggest a remedy");
    assert!(
        keyring.contains("keyring"),
        "suggestion must name the service that is down, got: {keyring}"
    );
    assert!(
        !keyring.contains("--timeout"),
        "a locked keyring is not a timeout; got: {keyring}"
    );

    let rng = SshCliError::software("rng")
        .suggestion()
        .expect("software must suggest a remedy");
    assert!(
        rng.contains("not help") || rng.contains("report"),
        "software failures must tell the caller retrying is pointless, got: {rng}"
    );
}

#[test]
fn exit_codes_follow_sysexits_positions() {
    // Guards against a renumbering that would silently redefine the agent contract.
    assert_eq!(exit_codes::EX_UNAVAILABLE, 69);
    assert_eq!(exit_codes::EX_SOFTWARE, 70);
}

/// The reclassification must not be undone by a future `map_err` reaching for `Config`.
///
/// This reads the product source rather than calling it because the failures involved —
/// a broken CSPRNG, a read-only home, an absent secret service — cannot be provoked from
/// a test process without root or a container. The check is narrow on purpose: it fails
/// only when `Config` reappears next to one of the three concrete markers that used to
/// wear it, so unrelated refactors of the file do not trip it.
#[test]
fn secrets_module_no_longer_routes_host_failures_through_config() {
    // Reads the whole subsystem, not a single file: the keyring and AEAD halves were
    // split into `src/secrets/` and an assertion pinned to `secrets.rs` alone would go
    // green simply because the code it guards moved next door.
    let mut src = std::fs::read_to_string("src/secrets.rs").expect("read src/secrets.rs");
    let dir = std::fs::read_dir("src/secrets").expect("read src/secrets/");
    for entry in dir.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") {
            src.push_str(&std::fs::read_to_string(&path).expect("read secrets submodule"));
        }
    }

    for (marker, expected) in [
        ("RNG failed", "SshCliError::software(\"rng\")"),
        (
            "keyring set failed",
            "SshCliError::unavailable(\"keyring\")",
        ),
        (
            "keyring get failed",
            "SshCliError::unavailable(\"keyring\")",
        ),
        ("fsync secrets.key", "SshCliError::Io"),
        ("chmod secrets.key", "SshCliError::Io"),
    ] {
        assert!(
            !src.contains(&format!("SshCliError::Config(format!(\"{marker}")),
            "`{marker}` must not exit 65 again — it belongs to {expected}"
        );
    }

    // Positive side: the replacements are actually present, so the test cannot pass by
    // the markers merely having been renamed.
    assert!(
        src.contains("SshCliError::software(\"rng\")"),
        "RNG failure must map to exit 70"
    );
    assert!(
        src.contains("SshCliError::unavailable(\"keyring\")"),
        "keyring failure must map to exit 69"
    );
}

/// Every `ErrorClass` variant must appear in the published schema.
///
/// A2: `ErrorClass::Partial` shipped with G-ERR-R02 and the schema kept enumerating
/// only three values, so a strict agent validator rejected the exact envelope the CLI
/// emits for a partial fan-out. Cross-checking the two here means adding a variant in
/// Rust without touching the contract fails the build.
#[test]
fn every_error_class_is_declared_in_the_published_schema() {
    let schema = std::fs::read_to_string("docs/schemas/error-envelope.schema.json")
        .expect("read error-envelope.schema.json");

    for class in [
        ErrorClass::Transient,
        ErrorClass::Permanent,
        ErrorClass::Cancelled,
        ErrorClass::Partial,
    ] {
        // Serialized form is the wire value; comparing against it rather than a
        // hand-written string keeps the two in step through a rename.
        let wire = serde_json::to_string(&class).expect("serialize ErrorClass");
        assert!(
            schema.contains(&wire),
            "error_class {wire} is emitted by the product but absent from the schema enum"
        );
    }
}

#[test]
fn error_code_is_contracted_in_the_schema() {
    let schema = std::fs::read_to_string("docs/schemas/error-envelope.schema.json")
        .expect("read error-envelope.schema.json");
    assert!(
        schema.contains("\"error_code\""),
        "error_code is emitted on every failure and must be documented, \
         otherwise agents branch on a field the contract never promised"
    );
}
