// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: unit tests split out of `json_wire/execution.rs` (line budget).
#![forbid(unsafe_code)]
//! Wire-shape tests for the one-shot result envelopes.

use super::*;

fn output() -> ExecutionOutput {
    ExecutionOutput {
        stdout: "out".into(),
        stderr: "err".into(),
        exit_code: Some(0),
        truncated_stdout: false,
        truncated_stderr: true,
        duration_ms: 42,
    }
}

/// GAP-SSH-EXEC-ENVELOPE-002 step 5: the envelope names the host that was typed.
///
/// Two positionals are the explicit form, so `host_source` must say `argv` and
/// `active_fallback` must be false. A reader that sees `argv` knows no on-disk
/// state took part in choosing the machine.
#[test]
fn envelope_names_the_host_designated_in_argv() {
    let target = ExecTarget::new("typed-host", TargetSource::Argv);
    let j = ExecutionJson::with_target(&output(), &target);

    assert_eq!(j.target.host_resolved, "typed-host");
    assert_eq!(j.target.host_source, TargetSource::Argv);
    assert!(!j.active_fallback, "argv is not a fallback");

    let s = serde_json::to_string(&j).expect("envelope serializes");
    assert!(s.contains(r#""host_resolved":"typed-host""#), "{s}");
    assert!(s.contains(r#""host_source":"argv""#), "{s}");
    assert!(s.contains(r#""duration_ms":42"#), "{s}");
    assert!(s.contains(r#""truncated_stderr":true"#), "{s}");
}

/// GAP-SSH-EXEC-ENVELOPE-002 step 6: an inherited host says so out loud.
///
/// `active_fallback` is redundant with `host_source` on purpose: a shell pipeline
/// can branch on a boolean without knowing how the enum is spelled on the wire.
#[test]
fn envelope_flags_a_host_inherited_from_the_marker() {
    let target = ExecTarget::new("inherited-host", TargetSource::ActiveMarker);
    let j = ExecutionJson::with_target(&output(), &target);

    assert_eq!(j.target.host_resolved, "inherited-host");
    assert!(
        j.active_fallback,
        "the marker is a fallback and must admit it"
    );

    let s = serde_json::to_string(&j).expect("envelope serializes");
    assert!(s.contains(r#""host_source":"active_marker""#), "{s}");
    assert!(s.contains(r#""active_fallback":true"#), "{s}");
}

/// A fleet host is neither typed nor inherited, and must not be flagged as either.
#[test]
fn selector_is_not_reported_as_a_fallback() {
    let target = ExecTarget::new("fleet-host", TargetSource::Selector);
    let j = ExecutionJson::with_target(&output(), &target);

    assert_eq!(j.target.host_source, TargetSource::Selector);
    assert!(!j.active_fallback, "a selector is an explicit designation");
}

/// A transfer envelope declares the machine the bytes reached.
///
/// `vps` already carried the name, but only the provenance separates a host the
/// caller typed from one a selector produced — and a write to a remote filesystem
/// is precisely where that distinction is worth auditing.
#[test]
fn the_scp_envelope_declares_its_target() {
    let v = ScpTransferJson {
        ok: true,
        event: "scp-transfer".into(),
        target: TargetEcho::from_parts("prod", TargetSource::Argv),
        direction: "upload".into(),
        vps: "prod".into(),
        local: "/tmp/a".into(),
        remote: "/tmp/b".into(),
        bytes: 7,
        duration_ms: 3,
        mtime_preserved: true,
        durable: true,
    };
    let s = serde_json::to_string(&v).expect("envelope serializes");

    assert!(s.contains(r#""target_resolved":"prod""#), "{s}");
    assert!(s.contains(r#""target_source":"argv""#), "{s}");
    assert!(s.contains(r#""host_resolved":"prod""#), "{s}");
    assert!(!s.contains('\n'), "the agent wire stays one line: {s}");
}

/// The SFTP envelope carries the same declaration as the SCP one.
#[test]
fn the_sftp_envelope_declares_its_target() {
    let v = SftpTransferJson {
        ok: true,
        event: "sftp-transfer".into(),
        target: TargetEcho::from_parts("prod", TargetSource::Argv),
        direction: "download".into(),
        vps: "prod".into(),
        local: "/tmp/a".into(),
        remote: "/tmp/b".into(),
        bytes: 9,
        duration_ms: 4,
        recursive: false,
    };
    let s = serde_json::to_string(&v).expect("envelope serializes");

    assert!(s.contains(r#""target_resolved":"prod""#), "{s}");
    assert!(s.contains(r#""host_source":"argv""#), "{s}");
}

/// A fleet batch states its provenance and deliberately names no single target.
///
/// Collapsing a host *set* into one scalar would be an assertion the run cannot
/// make; the members are already enumerated in `results[].name`.
#[test]
fn a_batch_states_selector_provenance_without_a_scalar_target() {
    let v = ScpBatchJson {
        event: "scp-batch".into(),
        target_source: TargetSource::Selector,
        host_source: TargetSource::Selector,
        batch_run_id: "id".into(),
        direction: "upload".into(),
        max_concurrency: 4,
        results: Vec::new(),
    };
    let s = serde_json::to_string(&v).expect("envelope serializes");

    assert!(s.contains(r#""target_source":"selector""#), "{s}");
    assert!(s.contains(r#""host_source":"selector""#), "{s}");
    assert!(
        !s.contains("target_resolved"),
        "a set has no single resolved name: {s}"
    );
}
