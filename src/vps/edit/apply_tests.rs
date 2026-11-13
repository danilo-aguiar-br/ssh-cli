// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Unit tests for [`super::apply`]: what `updated` reports and what it must not.

use super::apply::{apply_fields, mark_if_changed, EditRequest, StdinSecrets};
use crate::domain::KeyPath;
use crate::vps::model::VpsRecord;
use secrecy::{ExposeSecret, SecretString};

fn record() -> VpsRecord {
    VpsRecord::try_new(
        "demo",
        "h.example",
        22,
        "u",
        SecretString::from("pw".to_string()),
        None::<String>,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    )
    .expect("record")
}

fn request() -> EditRequest {
    EditRequest {
        name: "demo".into(),
        ..EditRequest::default()
    }
}

fn apply(r: &mut VpsRecord, req: &EditRequest) -> Vec<&'static str> {
    apply_fields(r, req, &StdinSecrets::default()).expect("apply")
}

fn tags_of(r: &VpsRecord) -> Vec<&str> {
    r.tags.iter().map(|t| t.as_str()).collect()
}

#[test]
fn clearing_tags_then_appending_keeps_the_appended_only() {
    let mut r = record();
    r.set_tags_from_raw(vec!["old".to_string()]).expect("tags");
    let req = EditRequest {
        clear_tags: true,
        tags: vec!["new".into()],
        ..request()
    };
    let updated = apply(&mut r, &req);
    assert_eq!(tags_of(&r), vec!["new"]);
    assert_eq!(updated, vec!["tags"]);
}

#[test]
fn appending_tags_dedupes_and_preserves_order() {
    let mut r = record();
    r.set_tags_from_raw(vec!["a".to_string(), "b".to_string()])
        .expect("tags");
    let req = EditRequest {
        tags: vec!["b".into(), "c".into()],
        ..request()
    };
    let updated = apply(&mut r, &req);
    assert_eq!(tags_of(&r), vec!["a", "b", "c"]);
    assert_eq!(updated, vec!["tags"]);
}

#[test]
fn clearing_the_key_also_clears_the_passphrase() {
    let mut r = record();
    let req = EditRequest {
        no_key: true,
        ..request()
    };
    let secrets = StdinSecrets {
        password: Some(SecretString::from("pw".to_string())),
        ..StdinSecrets::default()
    };
    apply_fields(&mut r, &req, &secrets).expect("apply");
    assert!(r.key_path.is_none());
    assert!(r.key_passphrase.is_none());
}

#[test]
fn clearing_every_auth_method_is_rejected_by_validation() {
    let mut r = record();
    // A key-only record: clearing the key leaves zero methods.
    r.password = SecretString::from(String::new());
    let dir = tempfile::tempdir().expect("tempdir");
    let key = dir.path().join("id_test");
    std::fs::write(&key, b"k").expect("key file");
    r.key_path = Some(KeyPath::try_new(key.to_string_lossy().into_owned()).expect("path"));
    let req = EditRequest {
        no_key: true,
        ..request()
    };
    let updated = apply(&mut r, &req);
    assert_eq!(updated, vec!["key_path"]);
    assert!(r.validate().is_err());
}

#[test]
fn an_empty_request_is_empty() {
    assert!(EditRequest::default().is_empty());
    let with_tag = EditRequest {
        tags: vec!["a".into()],
        ..EditRequest::default()
    };
    assert!(!with_tag.is_empty());
}

#[test]
fn secrets_prefer_stdin_over_argv() {
    let mut r = record();
    let req = EditRequest {
        password: Some("argv".into()),
        ..request()
    };
    let secrets = StdinSecrets {
        password: Some(SecretString::from("stdin".to_string())),
        ..StdinSecrets::default()
    };
    let updated = apply_fields(&mut r, &req, &secrets).expect("apply");
    assert_eq!(r.password.expose_secret(), "stdin");
    assert_eq!(updated, vec!["password"]);
}

#[test]
fn mark_if_changed_skips_unchanged_and_deduplicates() {
    let mut updated: Vec<&'static str> = Vec::new();
    mark_if_changed(&mut updated, "port", false);
    assert!(updated.is_empty());
    mark_if_changed(&mut updated, "port", true);
    mark_if_changed(&mut updated, "host", true);
    mark_if_changed(&mut updated, "port", true);
    assert_eq!(updated, vec!["port", "host"]);
}

#[test]
fn same_port_is_not_reported_and_a_new_one_is() {
    let mut r = record();
    let same = EditRequest {
        port: Some(22),
        ..request()
    };
    assert!(apply(&mut r, &same).is_empty());
    let other = EditRequest {
        port: Some(2222),
        ..request()
    };
    assert_eq!(apply(&mut r, &other), vec!["port"]);
}

#[test]
fn same_host_user_and_limits_are_not_reported() {
    let mut r = record();
    let req = EditRequest {
        host: Some("h.example".into()),
        user: Some("u".into()),
        timeout: Some(r.timeout_ms.get()),
        ..request()
    };
    assert!(apply(&mut r, &req).is_empty());
}

#[test]
fn clear_tags_on_an_empty_list_is_not_reported() {
    let mut r = record();
    let req = EditRequest {
        clear_tags: true,
        ..request()
    };
    assert!(apply(&mut r, &req).is_empty());
}

#[test]
fn clear_tags_with_the_same_single_tag_is_not_reported() {
    let mut r = record();
    r.set_tags_from_raw(vec!["z".to_string()]).expect("tags");
    let req = EditRequest {
        clear_tags: true,
        tags: vec!["z".into()],
        ..request()
    };
    assert!(apply(&mut r, &req).is_empty());
}

#[test]
fn clear_flags_on_absent_values_are_not_reported() {
    let mut r = record();
    let req = EditRequest {
        clear_sudo_password: true,
        clear_su_password: true,
        no_key: true,
        no_agent_socket: true,
        no_tls_sni: true,
        no_tls_client_cert: true,
        no_tls_client_key: true,
        ..request()
    };
    assert!(apply(&mut r, &req).is_empty());
}

#[test]
fn clear_sudo_password_reports_only_when_one_was_stored() {
    let mut r = record();
    r.sudo_password = Some(SecretString::from("s".to_string()));
    let req = EditRequest {
        clear_sudo_password: true,
        ..request()
    };
    assert_eq!(apply(&mut r, &req), vec!["sudo_password"]);
    assert!(r.sudo_password.is_none());
    assert!(apply(&mut r, &req).is_empty());
}

#[test]
fn identical_secrets_are_not_reported_and_different_ones_are() {
    let mut r = record();
    r.sudo_password = Some(SecretString::from("s".to_string()));
    let same = EditRequest {
        password: Some("pw".into()),
        sudo_password: Some("s".into()),
        ..request()
    };
    assert!(apply(&mut r, &same).is_empty());
    let different = EditRequest {
        sudo_password: Some("other".into()),
        ..request()
    };
    assert_eq!(apply(&mut r, &different), vec!["sudo_password"]);
}

#[test]
fn use_agent_reports_the_first_switch_and_not_the_repeat() {
    let mut r = record();
    let req = EditRequest {
        use_agent: true,
        ..request()
    };
    // The password the record held is cleared too, and `updated` says so.
    assert_eq!(apply(&mut r, &req), vec!["use_agent", "password"]);
    assert!(r.use_agent);
    assert!(apply(&mut r, &req).is_empty());
}

/// Writes a real OpenSSH key inside `dir`, or `None` when `ssh-keygen` is absent.
///
/// `--key` validates the key content when `ssh-real` is on, so a placeholder is
/// rejected. Test-only: the product never spawns `ssh-keygen`.
fn openssh_key(dir: &tempfile::TempDir) -> Option<std::path::PathBuf> {
    let key = dir.path().join("id_ed25519_test");
    let status = match std::process::Command::new("ssh-keygen")
        .args(["-t", "ed25519", "-N", "", "-q", "-f"])
        .arg(&key)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => panic!("ssh-keygen spawn failed: {e}"),
    };
    assert!(status.success(), "ssh-keygen failed: {status}");
    Some(key)
}

fn keyed_record(dir: &tempfile::TempDir) -> VpsRecord {
    let mut r = record();
    r.password = SecretString::from(String::new());
    // Set on the record directly: no content validation, so no real key needed.
    let path = dir.path().join("stored_key");
    std::fs::write(&path, b"k").expect("key file");
    r.key_path = Some(KeyPath::try_new(path.to_string_lossy().into_owned()).expect("key path"));
    r.key_passphrase = Some(SecretString::from("phrase".to_string()));
    r
}

#[test]
fn key_on_a_password_host_switches_the_method() {
    let dir = tempfile::tempdir().expect("tempdir");
    let Some(key) = openssh_key(&dir) else { return };
    let mut r = record();
    let req = EditRequest {
        key: Some(key),
        ..request()
    };
    assert_eq!(apply(&mut r, &req), vec!["key_path", "password"]);
    assert!(r.key_path.is_some());
    assert!(!r.has_password());
    assert!(r.validate().is_ok());
}

#[test]
fn password_on_a_key_host_clears_the_key_and_its_passphrase() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut r = keyed_record(&dir);
    let req = EditRequest {
        password: Some("new".into()),
        ..request()
    };
    let updated = apply(&mut r, &req);
    assert_eq!(updated, vec!["password", "key_path", "key_passphrase"]);
    assert!(r.key_path.is_none());
    assert!(r.key_passphrase.is_none());
    assert!(r.validate().is_ok());
}

#[test]
fn key_and_password_leave_agent_mode_and_drop_the_socket() {
    let dir = tempfile::tempdir().expect("tempdir");
    let agent = |r: &mut VpsRecord| {
        r.password = SecretString::from(String::new());
        r.use_agent = true;
        r.agent_socket = Some("/run/agent.sock".to_string());
    };
    let Some(key_path) = openssh_key(&dir) else {
        return;
    };
    let mut r = record();
    agent(&mut r);
    let key = EditRequest {
        key: Some(key_path),
        ..request()
    };
    assert_eq!(
        apply(&mut r, &key),
        vec!["key_path", "use_agent", "agent_socket"]
    );
    assert!(!r.use_agent && r.agent_socket.is_none());

    let mut r = record();
    agent(&mut r);
    let password = EditRequest {
        password: Some("pw2".into()),
        ..request()
    };
    assert_eq!(
        apply(&mut r, &password),
        vec!["password", "use_agent", "agent_socket"]
    );
    assert!(!r.use_agent && r.agent_socket.is_none());
}

#[test]
fn use_agent_reports_the_key_material_it_clears() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut r = keyed_record(&dir);
    let req = EditRequest {
        use_agent: true,
        ..request()
    };
    assert_eq!(
        apply(&mut r, &req),
        vec!["use_agent", "key_path", "key_passphrase"]
    );
    assert!(r.key_path.is_none() && r.key_passphrase.is_none());
}

#[test]
fn use_agent_with_an_agent_socket_keeps_the_documented_formula() {
    let mut r = record();
    let req = EditRequest {
        use_agent: true,
        agent_socket: Some("/run/agent.sock".into()),
        ..request()
    };
    let updated = apply(&mut r, &req);
    assert_eq!(updated, vec!["use_agent", "password", "agent_socket"]);
    assert_eq!(r.agent_socket.as_deref(), Some("/run/agent.sock"));
    assert!(r.validate().is_ok());
}

#[test]
fn a_passphrase_without_a_key_is_rejected() {
    let mut r = record();
    let req = EditRequest {
        key_passphrase: Some("orphan".into()),
        ..request()
    };
    let err = apply_fields(&mut r, &req, &StdinSecrets::default()).expect_err("orphan");
    assert!(matches!(
        err,
        crate::errors::SshCliError::InvalidArgument(_)
    ));
}

#[test]
fn a_passphrase_with_the_key_in_the_same_call_is_accepted() {
    let dir = tempfile::tempdir().expect("tempdir");
    let Some(key) = openssh_key(&dir) else { return };
    let mut r = record();
    let req = EditRequest {
        key: Some(key),
        ..request()
    };
    let secrets = StdinSecrets {
        key_passphrase: Some(SecretString::from("phrase".to_string())),
        ..StdinSecrets::default()
    };
    let updated = apply_fields(&mut r, &req, &secrets).expect("apply");
    assert_eq!(updated, vec!["key_path", "password", "key_passphrase"]);
}

#[test]
fn switches_report_only_a_flip() {
    let mut r = record();
    let off = EditRequest {
        enable_sudo: true,
        no_tls: true,
        ..request()
    };
    assert!(apply(&mut r, &off).is_empty());
    let on = EditRequest {
        disable_sudo: true,
        tls: true,
        tls_sni: Some("sni.example".into()),
        ..request()
    };
    assert_eq!(apply(&mut r, &on), vec!["disable_sudo", "tls", "tls_sni"]);
    assert!(apply(&mut r, &on).is_empty());
}

#[test]
fn a_different_key_drops_the_old_passphrase_and_the_same_key_keeps_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let Some(key) = openssh_key(&dir) else { return };
    let mut r = keyed_record(&dir);
    let req = EditRequest {
        key: Some(key.clone()),
        ..request()
    };
    assert_eq!(apply(&mut r, &req), vec!["key_path", "key_passphrase"]);
    assert!(r.key_passphrase.is_none());

    // Same path again: nothing changes, so a stored passphrase would survive.
    r.key_passphrase = Some(SecretString::from("kept".to_string()));
    assert!(apply(&mut r, &req).is_empty());
    assert_eq!(
        r.key_passphrase
            .as_ref()
            .map(|p| p.expose_secret().to_owned()),
        Some("kept".to_string())
    );
}

#[test]
fn a_different_key_with_a_new_passphrase_in_the_same_call_keeps_the_new_one() {
    let dir = tempfile::tempdir().expect("tempdir");
    let Some(key) = openssh_key(&dir) else { return };
    let mut r = keyed_record(&dir);
    let req = EditRequest {
        key: Some(key),
        key_passphrase: Some("fresh".into()),
        ..request()
    };
    assert_eq!(apply(&mut r, &req), vec!["key_path", "key_passphrase"]);
    assert_eq!(
        r.key_passphrase
            .as_ref()
            .map(|p| p.expose_secret().to_owned()),
        Some("fresh".to_string())
    );
}

#[test]
fn an_agent_socket_alone_switches_the_host_to_the_agent() {
    let mut r = record();
    let req = EditRequest {
        agent_socket: Some("/run/agent.sock".into()),
        ..request()
    };
    let updated = apply(&mut r, &req);
    assert_eq!(updated, vec!["use_agent", "password", "agent_socket"]);
    assert!(r.use_agent && !r.has_password());
    assert!(r.validate().is_ok());
    assert!(apply(&mut r, &req).is_empty());
}
