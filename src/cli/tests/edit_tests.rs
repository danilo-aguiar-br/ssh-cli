// SPDX-License-Identifier: MIT OR Apache-2.0
//! Clap-surface tests for `vps rename` and the `vps edit` clear flags.
//!
//! Split from `cli/tests.rs` once that module crossed the 600-line component
//! budget: the tests here cover one verb pair's argument surface, which is a
//! different subject from the general parser laws kept next door.
#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used)]

use crate::cli::*;
use clap::CommandFactory;

/// `vps rename` carries the subject and the destination as two positionals.
///
/// Arity is fixed on purpose: a verb that guesses which word is the host is the
/// defect class behind GAP-SSH-EXEC-ARGC-001, and a rename that guesses would
/// move the wrong registry key.
#[test]
fn parser_vps_rename_takes_exactly_two_positionals() {
    let args = CliArgs::try_parse_from(["ssh-cli", "vps", "rename", "old", "new"]).expect("rename");
    match args.command {
        Command::Vps {
            action: VpsAction::Rename { name, new_name },
        } => {
            assert_eq!(name, "old");
            assert_eq!(new_name, "new");
        }
        _ => panic!("esperado rename"),
    }

    // One positional would have to be read as both source and destination, and
    // three leaves a leftover word whose meaning nothing defines.
    assert!(CliArgs::try_parse_from(["ssh-cli", "vps", "rename", "old"]).is_err());
    assert!(CliArgs::try_parse_from(["ssh-cli", "vps", "rename", "old", "new", "extra"]).is_err());
}

/// `--name` on `vps edit` is the same rename expressed through the patch verb.
///
/// Its clap id is `new_name` so it cannot collide with the positional `name`,
/// which names the host being edited rather than the result.
#[test]
fn parser_edit_name_flag_is_the_rename_path() {
    let args = CliArgs::try_parse_from(["ssh-cli", "vps", "edit", "old", "--name", "new"])
        .expect("edit --name");
    match args.command {
        Command::Vps {
            action: VpsAction::Edit { name, new_name, .. },
        } => {
            assert_eq!(name, "old");
            assert_eq!(new_name.as_deref(), Some("new"));
        }
        _ => panic!("esperado edit"),
    }
}

/// Every clear flag parses standalone, and each is mutually exclusive with the
/// value flag for the *same* field.
///
/// Without the conflict, `--no-key --key /tmp/id` would be two instructions that
/// cannot both hold, and whichever the implementation applied would silently
/// discard the other.
#[test]
fn parser_edit_clear_flags_all_parse_and_conflict_with_their_value() {
    let clears: &[(&str, &str)] = &[
        ("--no-key", "--key"),
        ("--no-agent-socket", "--agent-socket"),
        ("--clear-sudo-password", "--sudo-password"),
        ("--clear-su-password", "--su-password"),
        ("--no-tls-sni", "--tls-sni"),
        ("--no-tls-client-cert", "--tls-client-cert"),
        ("--no-tls-client-key", "--tls-client-key"),
    ];

    for (clear, value) in clears {
        let alone = CliArgs::try_parse_from(["ssh-cli", "vps", "edit", "prod", clear]);
        assert!(alone.is_ok(), "{clear} must parse standalone");

        let both = CliArgs::try_parse_from(["ssh-cli", "vps", "edit", "prod", clear, value, "x"]);
        let err = both.expect_err(&format!("{clear} must conflict with {value}"));
        assert_eq!(
            err.kind(),
            clap::error::ErrorKind::ArgumentConflict,
            "{clear} vs {value}"
        );
    }

    // `--clear-tags` is the one clear flag that composes: clearing and then
    // appending is how a tag set is *replaced* in one locked write.
    let clear_and_append = CliArgs::try_parse_from([
        "ssh-cli",
        "vps",
        "edit",
        "prod",
        "--clear-tags",
        "--tag",
        "a",
    ]);
    assert!(
        clear_and_append.is_ok(),
        "--clear-tags must compose with --tag, otherwise replacing a tag set needs two commands"
    );
}

/// The clear flags are visible in help, because a flag an agent cannot discover
/// is a flag it will not use.
#[test]
fn parser_edit_clear_flags_are_documented_in_help() {
    let text = CliArgs::command()
        .find_subcommand_mut("vps")
        .expect("vps")
        .find_subcommand_mut("edit")
        .expect("edit")
        .render_long_help()
        .to_string();
    for clear in [
        "--clear-tags",
        "--no-key",
        "--no-agent-socket",
        "--clear-sudo-password",
        "--clear-su-password",
        "--no-tls-sni",
        "--no-tls-client-cert",
        "--no-tls-client-key",
    ] {
        assert!(
            text.contains(clear),
            "vps edit --help must document {clear}"
        );
    }
}

/// Parses `vps edit prod <flags...>` and returns the clap error kind, if any.
fn edit_error_kind(flags: &[&str]) -> Option<clap::error::ErrorKind> {
    let mut argv = vec!["ssh-cli", "vps", "edit", "prod"];
    argv.extend_from_slice(flags);
    CliArgs::try_parse_from(argv).err().map(|e| e.kind())
}

/// Two primary-auth flags in one `vps edit` call are two instructions that
/// cannot both hold, so clap refuses the pair instead of letting the last one win.
#[test]
fn parser_edit_auth_switch_flags_conflict_in_one_call() {
    let conflicting: &[&[&str]] = &[
        &["--key", "k", "--password", "x"],
        &["--key", "k", "--password-stdin"],
        &["--agent-socket", "s", "--key", "k"],
        &["--agent-socket", "s", "--password", "x"],
        &["--agent-socket", "s", "--password-stdin"],
        &["--key-passphrase", "p", "--use-agent"],
        &["--key-passphrase-stdin", "--use-agent"],
        &["--key-passphrase", "p", "--password", "x"],
        &["--key-passphrase", "p", "--password-stdin"],
        &["--key-passphrase-stdin", "--password", "x"],
        &["--key-passphrase-stdin", "--password-stdin"],
    ];
    for flags in conflicting {
        assert_eq!(
            edit_error_kind(flags),
            Some(clap::error::ErrorKind::ArgumentConflict),
            "{flags:?} must be refused by clap"
        );
    }
}

/// The documented agent formula and the single-method switches still parse.
#[test]
fn parser_edit_auth_switch_flags_parse_alone_and_in_the_agent_formula() {
    let accepted: &[&[&str]] = &[
        &["--key", "k"],
        &["--key", "k", "--key-passphrase-stdin"],
        &["--key", "k", "--key-passphrase", "p"],
        &["--password", "x"],
        &["--password-stdin"],
        &["--use-agent"],
        &["--use-agent", "--agent-socket", "s"],
        &["--agent-socket", "s"],
    ];
    for flags in accepted {
        assert_eq!(edit_error_kind(flags), None, "{flags:?} must parse");
    }
}

/// `vps edit` takes the host from argv only, so the ambient `--use-active` does
/// not exist on this verb.
#[test]
fn parser_edit_has_no_use_active_flag() {
    assert_eq!(
        edit_error_kind(&["--use-active", "--timeout", "9000"]),
        Some(clap::error::ErrorKind::UnknownArgument)
    );
}
