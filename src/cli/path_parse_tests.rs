// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: transfer slot-parser tests split out of `cli/tests.rs` (line budget).
#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used)]
//! Unit tests for the `scp` / `sftp` slot parser.
//!
//! Kept apart from the general clap-surface tests because they cover one law rather
//! than one command: no token may change role when a non-positional flag enters or
//! leaves the line, and an argv that would require guessing is refused instead.

use super::{parse_scp_target, ScpPathPlan};
use std::path::PathBuf;

/// Refines a name the way the parser does, so equality compares like with like.
fn test_vps(name: &str) -> crate::domain::VpsName {
    crate::domain::VpsName::try_new(name).unwrap()
}

/// No named slots — the positional grammar under test.
fn no_slots() -> crate::cli::TransferSlots {
    crate::cli::TransferSlots::new(Vec::new(), None)
}

#[test]
fn parse_scp_target_multi_file_single_host() {
    let plan = parse_scp_target(
        false,
        None,
        no_slots(),
        vec!["prod".into(), "a.bin".into(), "b.bin".into(), "/tmp".into()],
    )
    .unwrap();
    match plan {
        ScpPathPlan::MultiFile {
            vps,
            sources,
            dest_dir,
        } => {
            assert_eq!(vps, "prod");
            assert_eq!(sources.len(), 2);
            assert_eq!(dest_dir, PathBuf::from("/tmp"));
        }
        other => panic!("expected MultiFile, got {other:?}"),
    }
}

#[test]
fn parse_scp_target_multi_file_with_all_is_cartesian() {
    // G-PAR-48: multi-host × multi-file, now reached through the named slots. The
    // positional spelling of this form is refused as ambiguous — see
    // `parse_scp_target_refuses_selector_with_three_positionals`.
    let plan = parse_scp_target(
        true,
        None,
        crate::cli::TransferSlots::new(vec!["a.bin".into(), "b.bin".into()], Some("/tmp".into())),
        Vec::new(),
    )
    .unwrap();
    match plan {
        ScpPathPlan::MultiHostMultiFile {
            selection,
            sources,
            dest_dir,
        } => {
            assert_eq!(selection, crate::vps::HostSelection::All);
            assert_eq!(sources.len(), 2);
            assert_eq!(dest_dir, PathBuf::from("/tmp"));
        }
        other => panic!("expected MultiHostMultiFile, got {other:?}"),
    }
}

#[test]
fn parse_scp_target_multi_file_with_hosts() {
    let plan = parse_scp_target(
        false,
        Some("x,y".into()),
        crate::cli::TransferSlots::new(
            vec!["a.bin".into(), "b.bin".into()],
            Some("/remote/dir".into()),
        ),
        Vec::new(),
    )
    .unwrap();
    match plan {
        ScpPathPlan::MultiHostMultiFile {
            selection,
            sources,
            dest_dir,
        } => {
            assert_eq!(
                selection,
                crate::vps::HostSelection::Named(vec![test_vps("x"), test_vps("y")])
            );
            assert_eq!(sources.len(), 2);
            assert_eq!(dest_dir, PathBuf::from("/remote/dir"));
        }
        other => panic!("expected MultiHostMultiFile, got {other:?}"),
    }
}

#[test]
fn parse_scp_target_classic_three() {
    let plan = parse_scp_target(
        false,
        None,
        no_slots(),
        vec!["prod".into(), "a.bin".into(), "/tmp/a.bin".into()],
    )
    .unwrap();
    match plan {
        ScpPathPlan::Single {
            selection,
            path_a,
            path_b,
        } => {
            assert_eq!(
                selection,
                crate::vps::HostSelection::Single(test_vps("prod"))
            );
            assert_eq!(path_a, PathBuf::from("a.bin"));
            assert_eq!(path_b, PathBuf::from("/tmp/a.bin"));
        }
        other => panic!("expected Single, got {other:?}"),
    }
}

/// The token whose role used to flip when `--all` entered the line.
///
/// Without a selector `"prod"` fills the host slot; with one, the old parser reread it
/// as a source path. Both readings are defensible in isolation, which is what made the
/// bug survivable — so the parser refuses instead of choosing.
#[test]
fn parse_scp_target_refuses_selector_with_three_positionals() {
    for (all, hosts) in [(true, None), (false, Some("x,y".to_string()))] {
        let err = parse_scp_target(
            all,
            hosts.clone(),
            no_slots(),
            vec!["prod".into(), "a.bin".into(), "/tmp/dir".into()],
        )
        .expect_err("a selector with three positionals is ambiguous");
        assert!(
            err.contains("ambiguous"),
            "the refusal must state the reason: {err}"
        );
        assert!(
            err.contains("--src") && err.contains("--dest"),
            "the refusal must point at the named slots: {err}"
        );
    }
}

/// The same positional vector keeps one reading with the selector absent.
///
/// Paired with the refusal above, this is the property the law asks for: no token
/// changes role when a non-positional flag enters or leaves the line, because the
/// flagged form no longer has a reading at all.
#[test]
fn parse_scp_target_keeps_slot_zero_as_the_host_without_a_selector() {
    let plan = parse_scp_target(
        false,
        None,
        no_slots(),
        vec!["prod".into(), "a.bin".into(), "/tmp/dir".into()],
    )
    .expect("single-host three-positional form is unambiguous");
    match plan {
        ScpPathPlan::Single { selection, .. } => {
            assert_eq!(
                selection,
                crate::vps::HostSelection::Single(test_vps("prod"))
            );
        }
        other => panic!("expected Single, got {other:?}"),
    }
}

/// Two positionals under a selector never collided, so they still parse.
#[test]
fn parse_scp_target_allows_selector_with_two_positionals() {
    let plan = parse_scp_target(
        true,
        None,
        no_slots(),
        vec!["a.bin".into(), "/tmp/a.bin".into()],
    )
    .expect("one source under a selector has no rival reading");
    assert!(matches!(plan, ScpPathPlan::Single { .. }));
}

/// Named slots on a single host need the VPS positional and nothing else.
#[test]
fn parse_scp_target_named_slots_single_host() {
    let plan = parse_scp_target(
        false,
        None,
        crate::cli::TransferSlots::new(vec!["a.bin".into()], Some("/tmp/dir".into())),
        vec!["prod".into()],
    )
    .expect("VPS positional plus named slots");
    match plan {
        ScpPathPlan::MultiFile {
            vps,
            sources,
            dest_dir,
        } => {
            assert_eq!(vps, "prod");
            assert_eq!(sources, vec![PathBuf::from("a.bin")]);
            // `--dest` is a directory whatever the source count: the role comes from
            // the slot name, never from how many siblings the slot has.
            assert_eq!(dest_dir, PathBuf::from("/tmp/dir"));
        }
        other => panic!("expected MultiFile, got {other:?}"),
    }
}

/// A half-filled slot pair is refused rather than completed by guesswork.
#[test]
fn parse_scp_target_refuses_a_lone_named_slot() {
    assert!(parse_scp_target(
        false,
        None,
        crate::cli::TransferSlots::new(vec!["a.bin".into()], None),
        vec!["prod".into()],
    )
    .is_err());
    assert!(parse_scp_target(
        false,
        None,
        crate::cli::TransferSlots::new(Vec::new(), Some("/tmp".into())),
        vec!["prod".into()],
    )
    .is_err());
}

/// With every slot named there is nowhere for a positional to go.
#[test]
fn parse_scp_target_refuses_stray_positionals_beside_named_slots() {
    assert!(parse_scp_target(
        true,
        None,
        crate::cli::TransferSlots::new(vec!["a.bin".into()], Some("/tmp".into())),
        vec!["leftover".into()],
    )
    .is_err());
    assert!(parse_scp_target(
        false,
        None,
        crate::cli::TransferSlots::new(vec!["a.bin".into()], Some("/tmp".into())),
        vec!["prod".into(), "leftover".into()],
    )
    .is_err());
}
