// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: vps unit tests extracted from vps/mod (line budget).
#![forbid(unsafe_code)]

use super::*;
use crate::cli::OutputFormat;
use crate::errors::SshCliError;
use secrecy::{ExposeSecret, SecretString};

fn reg_min() -> VpsRecord {
    VpsRecord::test_new(
        "srv",
        "host.example.com",
        2222,
        "admin",
        SecretString::from("pass".to_string()),
        None,
        None,
        Some(60_000),
        Some(1_000),
        Some(50_000),
        None,
        None,
        false,
    )
}

#[test]
fn parse_limit_none() {
    assert_eq!(model::parse_char_limit("none"), 0);
    assert_eq!(model::parse_char_limit("0"), 0);
    assert_eq!(model::parse_char_limit("  42 "), 42);
}

#[test]
fn effective_limit_unlimited() {
    assert_eq!(model::effective_limit(0), usize::MAX);
    assert_eq!(model::effective_limit(10), 10);
}

#[test]
fn validate_command_length_ok() {
    assert!(validate_command_length("echo hi", 100).is_ok());
}

#[test]
fn validate_command_length_too_long() {
    let err = validate_command_length("abcdefghij", 5).unwrap_err();
    assert!(matches!(err, SshCliError::CommandTooLong { .. }));
}

#[test]
fn reg_min_fields() {
    let v = reg_min();
    assert_eq!(v.name.as_str(), "srv");
    assert_eq!(v.host.as_str(), "host.example.com");
    assert_eq!(v.port.get(), 2222);
    assert_eq!(v.username.as_str(), "admin");
    assert_eq!(v.password.expose_secret(), "pass");
}

#[test]
fn reg_min_with_key() {
    let v = VpsRecord::test_new(
        "srv",
        "h",
        22,
        "u",
        SecretString::from(String::new()),
        Some("/k"),
        None,
        Some(5_000),
        Some(1_000),
        Some(50_000),
        None,
        None,
        false,
    );
    assert_eq!(
        v.key_path.as_ref().map(|k| k.to_string_lossy_owned()),
        Some("/k".into())
    );
}

#[tokio::test]
async fn sudo_exec_with_client_ok() {
    use crate::ssh::client::mocks::MockSshClient;
    use crate::ssh::client::ExecutionOutput;
    let mut mock = MockSshClient::new();
    mock.expect_run_command().returning(|c, _, stdin| {
        assert!(c.contains("sudo -n sh -c"));
        assert!(stdin.is_none());
        Ok(ExecutionOutput {
            stdout: "ok".into(),
            stderr: String::new(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 1,
        })
    });
    mock.expect_disconnect().returning(|| Ok(()));

    let vps = reg_min();
    run_sudo_exec_with_client(&vps, "id", Box::new(mock), OutputFormat::Text, false)
        .await
        .unwrap();
}
