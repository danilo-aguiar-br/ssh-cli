// SPDX-License-Identifier: MIT OR Apache-2.0
//! Remote exec / sudo-exec / su-exec (SRP extract — G-COMP-05).
//!
//! Workload: **I/O-bound** SSH. Multi-host fan-out uses
//! [`crate::concurrency::map_bounded`]. Single-host is one-shot
//! connect → run → disconnect (rules one-shot).
//!
//! Secrets: [`secrecy::SecretString`] only; prefer `take` over clone (rules memory).
#![forbid(unsafe_code)]

use super::selection::{resolve_host_jobs, HostSelection};
use super::{
    apply_overrides, build_connection_config, load, resolve_config_path, validate_command_length,
};
use crate::cli::OutputFormat;
use crate::errors::{finish_batch, SshCliError};
use crate::output;
use crate::ssh::client::{ExecutionOutput, SshClient, SshClientTrait};
use crate::ssh::packing::{append_description, pack_su, pack_sudo, PackedCommand};
use crate::vps::model::{effective_limit, VpsRecord};
use anyhow::Result;
use secrecy::SecretString;
use std::path::PathBuf;

/// Common remote execution options.
///
/// G-SECDEV-02: password fields are [`SecretString`] so zeroize-on-drop applies
/// through multi-host clone/fan-out (secrecy 0.10 `SecretString: Clone`).
///
/// G-TYPE-18/19: `timeout` is [`crate::domain::TimeoutMs`]; `steps` are [`crate::domain::RemoteCommand`].
mod elevation;
mod fleet;
mod single;
mod types;

pub use elevation::{run_su_exec, run_sudo_exec, run_sudo_exec_with_client};
pub(crate) use fleet::run_exec_all;
pub use single::{run_exec, run_exec_with_client};
// The step engine is shared: `sudo-exec` / `su-exec` reuse it verbatim so the
// three elevation paths cannot drift in how they sequence steps (DRY).
pub(crate) use single::{run_prepared_steps, step_labels, PreparedStep};
pub(crate) use types::{cancelled_err, expect_single, ExecKind};
pub use types::{ExecOptions, HostExecResult};

/// Step-execution tests.
///
/// Marked `#[serial]`: the exec paths poll the process-wide cancel flags, and the
/// `signals` tests toggle those flags. Without serialization a parallel run can make
/// these look cancelled (see `crate::signals::reset_flags_for_tests`).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::client::mocks::MockSshClient;
    use crate::ssh::client::ExecutionOutput;
    use secrecy::SecretString;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn record(sudo: Option<&str>, su: Option<&str>) -> VpsRecord {
        VpsRecord::test_new(
            "srv",
            "host.example.com",
            22,
            "admin",
            SecretString::from("pass".to_string()),
            None,
            None,
            Some(60_000),
            Some(1_000),
            Some(50_000),
            sudo.map(|s| SecretString::from(s.to_string())),
            su.map(|s| SecretString::from(s.to_string())),
            false,
        )
    }

    fn steps(raw: &[&str]) -> Vec<crate::domain::RemoteCommand> {
        raw.iter()
            .map(|s| {
                crate::domain::RemoteCommand::try_new(*s)
                    .unwrap_or_else(|_| unreachable!("test step is a valid remote command: {s}"))
            })
            .collect()
    }

    /// Counts `run_command` calls and records the commands actually sent.
    fn counting_mock(seen: &Arc<std::sync::Mutex<Vec<String>>>) -> MockSshClient {
        let mut mock = MockSshClient::new();
        let seen = Arc::clone(seen);
        mock.expect_run_command().returning(move |cmd, _, _| {
            if let Ok(mut v) = seen.lock() {
                v.push(cmd.to_owned());
            }
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
        mock
    }

    /// B1: `sudo-exec --step A --step B` used to run only the primary command and
    /// still exit 0. Every step must reach the wire, each with its own `sudo` pack.
    #[tokio::test]
    #[serial_test::serial]
    async fn sudo_exec_runs_every_step() {
        crate::signals::reset_flags_for_tests();
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mock = counting_mock(&seen);
        let vps = record(Some("sudopw"), None);

        elevation::run_sudo_exec_with_client_steps(
            &vps,
            "first",
            &steps(&["second", "third"]),
            Box::new(mock),
            OutputFormat::Text,
            false,
        )
        .await
        .unwrap_or_else(|e| unreachable!("sudo steps must succeed: {e}"));

        let sent = seen.lock().unwrap_or_else(|e| e.into_inner()).clone();
        assert_eq!(sent.len(), 3, "one remote command per step: {sent:?}");
        for (i, raw) in ["first", "second", "third"].iter().enumerate() {
            assert!(sent[i].contains("sudo -S -p '' sh -c"), "{}", sent[i]);
            assert!(sent[i].contains(raw), "{}", sent[i]);
            assert!(!sent[i].contains("sudopw"), "password must stay off argv");
        }
    }

    /// Same contract for the plain `exec` path (regression guard for the shared loop).
    #[tokio::test]
    #[serial_test::serial]
    async fn exec_runs_every_step() {
        crate::signals::reset_flags_for_tests();
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mock = counting_mock(&seen);
        let vps = record(None, None);

        single::run_exec_with_client_steps(
            &vps,
            "first",
            &steps(&["second"]),
            Box::new(mock),
            OutputFormat::Text,
            false,
        )
        .await
        .unwrap_or_else(|e| unreachable!("exec steps must succeed: {e}"));

        let sent = seen.lock().unwrap_or_else(|e| e.into_inner()).clone();
        assert_eq!(sent, vec!["first".to_string(), "second".to_string()]);
    }

    /// A failing step must surface as a non-zero exit instead of a silent success,
    /// and the remaining steps still run.
    #[tokio::test]
    #[serial_test::serial]
    async fn sudo_exec_step_failure_is_reported() {
        crate::signals::reset_flags_for_tests();
        let calls = Arc::new(AtomicUsize::new(0));
        let mut mock = MockSshClient::new();
        let calls_c = Arc::clone(&calls);
        mock.expect_run_command().returning(move |_, _, _| {
            let n = calls_c.fetch_add(1, Ordering::Relaxed);
            Ok(ExecutionOutput {
                stdout: String::new(),
                stderr: if n == 0 { "boom".into() } else { String::new() },
                exit_code: Some(if n == 0 { 7 } else { 0 }),
                truncated_stdout: false,
                truncated_stderr: false,
                duration_ms: 1,
            })
        });
        mock.expect_disconnect().returning(|| Ok(()));
        let vps = record(Some("sudopw"), None);

        let err = elevation::run_sudo_exec_with_client_steps(
            &vps,
            "first",
            &steps(&["second"]),
            Box::new(mock),
            OutputFormat::Text,
            false,
        )
        .await
        .expect_err("first step exited 7");

        assert_eq!(calls.load(Ordering::Relaxed), 2, "later steps still run");
        let downcast = err.downcast_ref::<SshCliError>();
        assert!(
            matches!(
                downcast,
                Some(SshCliError::CommandFailed { exit_code: 7, .. })
            ),
            "{err}"
        );
    }
}
