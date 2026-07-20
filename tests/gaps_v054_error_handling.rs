// SPDX-License-Identifier: MIT OR Apache-2.0
//! G-ERR gates: error handling policy (tratamento de erros).
//!
//! Local product gates — not GH Actions.

use ssh_cli::domain::DomainError;
use ssh_cli::errors::{exit_codes, SshCliError};
use ssh_cli::json_wire::ErrorEnvelope;
use std::path::PathBuf;

#[test]
fn g_err_01_display_lowercase_no_trailing_period() {
    let samples = [
        SshCliError::VpsNotFound("x".into()).to_string(),
        SshCliError::NoActiveVps.to_string(),
        SshCliError::tls_msg("handshake").to_string(),
        SshCliError::channel_msg("open").to_string(),
        SshCliError::AuthenticationFailed.to_string(),
        SshCliError::XdgDirectory.to_string(),
    ];
    for s in samples {
        assert!(!s.ends_with('.'), "trailing period forbidden: {s}");
        let first = s.chars().next().expect("non-empty");
        assert!(
            !first.is_ascii_uppercase(),
            "Display must start lowercase (G-ERR-01): {s}"
        );
    }
}

#[test]
fn g_err_02_domain_is_first_class_variant() {
    let d = DomainError::new("port", "out of range");
    let e: SshCliError = d.into();
    assert!(matches!(e, SshCliError::Domain(_)));
    assert_eq!(e.error_code(), "domain_validation");
    assert_eq!(e.exit_code(), exit_codes::EX_USAGE);
    assert!(!e.is_retryable());
}

#[test]
fn g_err_04_tls_source_chain_preserved() {
    let io = std::io::Error::other("root-cause-xyz");
    let e = SshCliError::tls_src("handshake failed", io);
    let src = std::error::Error::source(&e).expect("source");
    assert!(src.to_string().contains("root-cause-xyz"));
    assert!(
        !e.to_string().contains("root-cause-xyz"),
        "Display must not duplicate source text: {}",
        e
    );
    assert_eq!(e.error_code(), "tls");
}

#[test]
fn g_err_05_channel_source_chain_preserved() {
    let io = std::io::Error::other("chan-root");
    let e = SshCliError::channel_src("open session", io);
    let src = std::error::Error::source(&e).expect("source");
    assert!(src.to_string().contains("chan-root"));
    assert_eq!(e.error_code(), "channel_failed");
}

#[test]
fn g_err_03_xdg_directory_variant_used() {
    // Construction path when ProjectDirs fails is rare; assert variant wiring.
    let e = SshCliError::XdgDirectory;
    assert_eq!(e.error_code(), "xdg_directory");
    assert_eq!(e.exit_code(), exit_codes::EX_CANTCREAT);
    assert!(!e.is_retryable());
}

#[test]
fn g_err_08_envelope_includes_error_code() {
    let env = ErrorEnvelope {
        exit_code: 66,
        error_code: "vps_not_found".into(),
        message: "vps 'x' not found in registry".into(),
        remote_exit_code: None,
        error_class: ssh_cli::errors::ErrorClass::Permanent,
        retryable: false,
        suggestion: None,
    };
    let s = serde_json::to_string(&env).expect("ser");
    assert!(s.contains("\"error_code\":\"vps_not_found\""), "{s}");
    assert!(s.contains("\"exit_code\":66"), "{s}");
}

#[test]
fn g_err_06_paths_return_ssh_cli_error_not_anyhow() {
    // Typed path validation.
    let err = ssh_cli::paths::validate_name("").expect_err("empty");
    assert!(matches!(err, SshCliError::InvalidArgument(_)));
    assert_eq!(err.error_code(), "invalid_argument");

    let err = ssh_cli::paths::validate_no_traversal("a/../b").expect_err("traversal");
    assert!(matches!(err, SshCliError::InvalidArgument(_)));
}

#[test]
fn g_err_13_secrets_env_key_is_rejected() {
    // Fail-closed: env material must not be used as store.
    std::env::set_var("SSH_CLI_SECRETS_KEY", "00".repeat(32));
    let res = ssh_cli::secrets::load_primary_key();
    std::env::remove_var("SSH_CLI_SECRETS_KEY");
    let err = res.expect_err("env key must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("not supported") || msg.contains("XDG") || msg.contains("secrets init"),
        "msg={msg}"
    );
}

#[test]
fn g_err_14_resolve_limit_ignores_env() {
    // Auto path must not depend on SSH_CLI_MAX_CONCURRENCY.
    std::env::set_var("SSH_CLI_MAX_CONCURRENCY", "1");
    let n = ssh_cli::concurrency::resolve_limit(None);
    std::env::remove_var("SSH_CLI_MAX_CONCURRENCY");
    // auto_limit clamps to MIN..=HARD; with env=1 formerly could force 1 — now ignored.
    assert!(n >= 2, "expected auto floor >= 2, got {n}");
}

#[test]
fn g_err_12_client_monolith_split_files_exist() {
    // Structural gate: monólito split artifacts present.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for rel in [
        "src/ssh/client_real.rs",
        "src/ssh/client_real.rs",
        "src/ssh/client_stub.rs",
        "src/ssh/client_tests.rs",
    ] {
        assert!(root.join(rel).is_file(), "missing {rel}");
    }
    let client_lines = std::fs::read_to_string(root.join("src/ssh/client_real.rs"))
        .unwrap()
        .lines()
        .count();
    assert!(
        client_lines < 400,
        "client.rs facade should stay thin (G-ERR-12), got {client_lines} lines"
    );
}
