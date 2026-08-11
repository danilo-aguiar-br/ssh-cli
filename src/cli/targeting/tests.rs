// SPDX-License-Identifier: MIT OR Apache-2.0
// GAP-SSH-EXEC-ARGC-001: regression suite for explicit target designation.
#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used)]
//! Unit tests for the exec-family target designation rules.
//!
//! Split out of `cli/tests.rs` because target designation is its own behaviour and
//! deserves its own file — the same reason the component budget refuses oversized
//! test modules.

use super::super::path_parse::{parse_exec_target, ExecTargetError};
use super::super::{CliArgs, Command};
use super::{resolve_exec_target, ExecTargetArgs};
use clap::Parser;
use std::path::Path;

/// Writes a registry containing `name`, using the same persistence the product uses.
///
/// Deliberately not a `#[cfg(test)]` helper on `crate::vps`: adding a seeding entry
/// point to the production surface to serve one test is exactly the kind of
/// single-use abstraction the component rules refuse.
fn seed_host(config: &Path, name: &str) {
    let path = crate::vps::resolve_config_path(Some(config)).expect("resolve config path");
    // Load-then-insert rather than overwrite, so successive calls accumulate: the
    // displaced-token case needs two hosts in the registry at once.
    let mut file = crate::vps::load(&path).unwrap_or_default();
    file.hosts.insert(
        name.to_string(),
        crate::vps::model::VpsRecord::test_new(
            name,
            "203.0.113.10",
            22,
            "u",
            secrecy::SecretString::from("seed-password-long-enough".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        ),
    );
    crate::vps::save(&path, &file).expect("write seeded registry");
}

/// Points the active marker at `name`, beside the config file.
fn seed_active(config: &Path, name: &str) {
    let path = crate::vps::resolve_config_path(Some(config)).expect("resolve config path");
    let marker = path
        .parent()
        .expect("config path has a parent")
        .join(crate::constants::ACTIVE_VPS_FILE_NAME);
    std::fs::write(marker, name).expect("write active marker");
}

/// Helper mirroring the one in `cli/tests.rs`, kept local so the modules stay
/// independent.
fn test_vps(name: &str) -> crate::domain::VpsName {
    crate::domain::VpsName::try_new(name).expect("valid test VpsName")
}

/// GAP-SSH-EXEC-ARGC-001, the displaced-token test.
///
/// This is the exact argv that sent a day of work to the wrong machine: a host name
/// followed by `--step`. `--step` is `ArgAction::Append` and never counted as a
/// positional, so the old parser saw one positional, read it as the command, and
/// resolved the host from the on-disk marker. With an active marker present the run
/// succeeded — on a host nobody named.
///
/// The marker is deliberately `Some(...)` here: with `None` the old code failed for
/// the wrong reason and the bug stayed invisible, which is precisely why every
/// pre-existing call in this file passed `None` and none of them caught it.
#[test]
fn parse_exec_target_rejects_host_only_with_steps() {
    let err = parse_exec_target(
        false,
        None,
        None,
        false,
        vec!["ssh-danilo".into()],
        Some("ssh-other".into()),
    )
    .expect_err("a lone positional must never resolve to the active marker");
    assert!(
        matches!(err, ExecTargetError::Invalid(_)),
        "expected a usage error, got {err:?}"
    );
}

/// The active marker is reachable only through the explicit opt-in.
#[test]
fn parse_exec_target_use_active_is_explicit() {
    let plan = parse_exec_target(
        false,
        None,
        None,
        true,
        vec!["uptime".into()],
        Some("prod".into()),
    )
    .unwrap();
    assert_eq!(
        plan.selection,
        crate::vps::HostSelection::Single(test_vps("prod"))
    );
    assert_eq!(plan.command, "uptime");
    assert_eq!(plan.source, crate::json_wire::TargetSource::ActiveMarker);
    assert!(plan.source.is_ambient());
}

/// `--use-active` with no marker is a typed miss, not a generic usage error.
#[test]
fn parse_exec_target_use_active_without_marker_is_typed() {
    let err = parse_exec_target(false, None, None, true, vec!["uptime".into()], None)
        .expect_err("no marker must fail");
    assert_eq!(err, ExecTargetError::NoActiveVps);
}

/// `--use-active` still refuses two positionals: the host slot does not exist there.
#[test]
fn parse_exec_target_use_active_rejects_two_positionals() {
    let err = parse_exec_target(
        false,
        None,
        None,
        true,
        vec!["prod".into(), "uptime".into()],
        Some("prod".into()),
    )
    .expect_err("two positionals under --use-active must fail");
    assert!(matches!(err, ExecTargetError::Invalid(_)));
}

/// `--use-active` cannot be combined with a fleet selector.
#[test]
fn parse_exec_target_use_active_conflicts_with_selectors() {
    let err = parse_exec_target(
        true,
        None,
        None,
        true,
        vec!["uptime".into()],
        Some("prod".into()),
    )
    .expect_err("--use-active with --all must fail");
    assert!(matches!(err, ExecTargetError::Invalid(_)));
}

/// The `--tags` branch had zero coverage before v0.5.5.
#[test]
fn parse_exec_target_tags_selects_and_reports_selector() {
    let plan = parse_exec_target(
        false,
        None,
        Some("prod,edge".into()),
        false,
        vec!["uptime".into()],
        None,
    )
    .unwrap();
    assert!(plan.selection.is_batch());
    assert_eq!(plan.command, "uptime");
    assert_eq!(plan.source, crate::json_wire::TargetSource::Selector);
}

/// A selector with two positionals is a usage error, never a silent host swap.
#[test]
fn parse_exec_target_selector_rejects_two_positionals() {
    for (all, hosts, tags) in [
        (true, None, None),
        (false, Some("a".to_string()), None),
        (false, None, Some("prod".to_string())),
    ] {
        let err = parse_exec_target(
            all,
            hosts,
            tags,
            false,
            vec!["prod".into(), "uptime".into()],
            None,
        )
        .expect_err("selector + two positionals must fail");
        assert!(matches!(err, ExecTargetError::Invalid(_)));
    }
}

/// Every rejection teaches the three valid forms instead of guessing intent.
#[test]
fn parse_exec_target_usage_message_names_all_forms() {
    let err = parse_exec_target(false, None, None, false, vec!["uptime".into()], None)
        .expect_err("one positional must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("<VPS> <COMMAND>"),
        "missing explicit form: {msg}"
    );
    assert!(msg.contains("--use-active"), "missing opt-in form: {msg}");
    assert!(msg.contains("--all"), "missing selector form: {msg}");
}

/// The clap layer must accept `--use-active` on all three exec surfaces.
#[test]
fn parser_accepts_use_active_on_every_exec_surface() {
    for verb in ["exec", "sudo-exec", "su-exec"] {
        let args = CliArgs::try_parse_from(["ssh-cli", verb, "--use-active", "uptime"]).unwrap();
        let ok = match args.command {
            Command::Exec { use_active, .. }
            | Command::SudoExec { use_active, .. }
            | Command::SuExec { use_active, .. } => use_active,
            _ => panic!("{verb} parsed into the wrong variant"),
        };
        assert!(ok, "{verb} did not set use_active");
    }
}

/// clap itself refuses `--use-active` alongside a fleet selector.
#[test]
fn parser_rejects_use_active_with_selectors() {
    for flag in ["--all", "--hosts=a", "--tags=prod"] {
        assert!(
            CliArgs::try_parse_from(["ssh-cli", "exec", "--use-active", flag, "uptime"]).is_err(),
            "clap accepted --use-active with {flag}"
        );
    }
}

/// GAP-SSH-EXEC-ARGC-001 rule 2: a registered name is never a command.
///
/// This is the displaced-token case one flag further along. The caller still
/// believes the first positional is the host, so `--use-active <HOST>` would run an
/// inventory name as a shell binary — on a *different* machine. Rule 1 (a lone
/// positional is a usage error) does not catch it, because `--use-active` makes one
/// positional legal; only the registry lookup can tell the two apart.
///
/// The lookup is why this test lives here rather than beside `parse_exec_target`:
/// the parser is deliberately pure, and the inventory is read one layer up.
#[test]
fn parse_exec_target_rejects_known_vps_as_command() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let config = tmp.path().to_path_buf();

    // A registry containing exactly the name the caller is about to misuse, plus a
    // marker pointing somewhere else. The marker matters: `--use-active` resolves it
    // before the displaced-token guard runs, so without one the caller gets the
    // generic "no active vps" instead — correct, but not the case under test. With a
    // marker present this argv would have *succeeded*, running an inventory name as a
    // shell binary on a machine the caller never named.
    seed_host(&config, "target-host");
    seed_host(&config, "other-host");
    seed_active(&config, "other-host");

    let err = resolve_exec_target(
        ExecTargetArgs {
            all: false,
            hosts: None,
            tags: None,
            use_active: true,
            target: vec!["target-host".into()],
        },
        Some(config.as_path()),
    )
    .expect_err("a registered name must not be accepted as a command");

    let msg = err.to_string();
    assert!(
        msg.contains("target-host"),
        "the message must name the token so the caller can see the mistake: {msg}"
    );
    assert!(
        msg.contains("--use-active"),
        "the message must teach the fix, not just refuse: {msg}"
    );
}

/// The same argv with an *unregistered* name is legitimate: it really is a command.
///
/// Without this the rule above could be implemented as "reject any positional under
/// `--use-active`", which would break the opt-in it exists to protect.
#[test]
fn use_active_still_accepts_a_command_that_is_not_a_host() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let config = tmp.path().to_path_buf();
    seed_host(&config, "target-host");
    seed_active(&config, "target-host");

    let plan = resolve_exec_target(
        ExecTargetArgs {
            all: false,
            hosts: None,
            tags: None,
            use_active: true,
            target: vec!["uptime".into()],
        },
        Some(config.as_path()),
    )
    .expect("a plain command under --use-active is the supported form");

    assert_eq!(plan.command, "uptime");
    assert_eq!(plan.source, crate::json_wire::TargetSource::ActiveMarker);
}
