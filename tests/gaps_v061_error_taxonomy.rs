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

use ssh_cli::errors::{exit_codes, ErrorClass, ErrorLayer, SshCliError, UnavailableService};

#[test]
fn unavailable_service_exits_69_and_is_retryable() {
    let e = SshCliError::unavailable(UnavailableService::Keyring);

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
    let unavailable = SshCliError::unavailable(UnavailableService::Keyring).exit_code();
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
    let keyring = SshCliError::unavailable(UnavailableService::Keyring)
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

/// B4: the registry lock used to inherit the keyring advice because `suggestion()`
/// matched `Unavailable { .. }`. The service is now a closed enum matched per variant.
#[test]
fn config_lock_suggestion_names_the_lock_knob_and_not_the_keyring() {
    let lock = SshCliError::unavailable(UnavailableService::ConfigLock)
        .suggestion()
        .expect("config lock must suggest a remedy");
    assert!(
        lock.contains("--lock-wait-ms"),
        "the lock remedy must name the knob that fixes it, got: {lock}"
    );
    assert!(
        !lock.contains("keyring"),
        "a held registry lock has nothing to do with the keyring, got: {lock}"
    );
}

#[test]
fn keyring_suggestion_names_the_keyring_and_not_the_timeout() {
    let keyring = SshCliError::unavailable(UnavailableService::Keyring)
        .suggestion()
        .expect("keyring must suggest a remedy");
    assert!(keyring.contains("keyring"), "got: {keyring}");
    assert!(!keyring.contains("--timeout"), "got: {keyring}");
}

/// The service enum must not change what agents already parse.
#[test]
fn unavailable_display_keeps_the_wire_slug() {
    assert_eq!(
        SshCliError::unavailable(UnavailableService::ConfigLock).to_string(),
        "service unavailable: config_lock"
    );
    assert_eq!(
        SshCliError::unavailable(UnavailableService::Keyring).to_string(),
        "service unavailable: keyring"
    );
    assert_eq!(UnavailableService::ConfigLock.slug(), "config_lock");
    assert_eq!(UnavailableService::Keyring.slug(), "keyring");
}

#[test]
fn config_lock_is_an_io_layer_failure_and_keyring_stays_auth() {
    assert_eq!(
        SshCliError::unavailable(UnavailableService::ConfigLock).layer(),
        ErrorLayer::Io
    );
    assert_eq!(
        SshCliError::unavailable(UnavailableService::Keyring).layer(),
        ErrorLayer::Auth
    );
}

/// X11: repeating `vps rename A B` after a success lands on `VpsNotFound`; the hint
/// must say the first call may have worked.
#[test]
fn vps_not_found_suggests_listing_before_repeating() {
    let hint = SshCliError::VpsNotFound("a".into())
        .suggestion()
        .expect("vps_not_found must suggest a remedy");
    assert!(hint.contains("vps list"), "got: {hint}");
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
            "SshCliError::unavailable(UnavailableService::Keyring)",
        ),
        (
            "keyring get failed",
            "SshCliError::unavailable(UnavailableService::Keyring)",
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
        src.contains("SshCliError::unavailable(UnavailableService::Keyring)"),
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
