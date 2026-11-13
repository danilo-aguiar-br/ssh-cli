// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Command-tree discovery for `ssh-cli commands --json`.
//!
//! # Why the tree carries arguments
//!
//! The tree used to publish `name`, `about` and `subcommands` only, so an agent
//! asking `commands --json` learned that `vps edit` exists and nothing about
//! what it accepts. Every flag was reachable only by scraping `--help`, which
//! is prose, or by reading this source. Ten new flags would have shipped
//! invisible on the one surface built for discovery.
//!
//! # Why the payload stays compact
//!
//! No defaults, no `value_parser` types, no possible-value lists: those are
//! schema material and `ssh-cli schema <name>` already owns it. The tree says
//! which flags exist, whether each is required or repeatable, and what value
//! name to show — the minimum an agent needs to build a valid invocation.

use clap::Command;

/// Builds the `event: commands` envelope with the full command tree.
#[must_use]
pub fn command_tree_json() -> serde_json::Value {
    use clap::CommandFactory;
    let root = crate::cli::CliArgs::command();
    serde_json::json!({
        "ok": true,
        "event": "commands",
        "bin": root.get_name(),
        "version": env!("CARGO_PKG_VERSION"),
        "tree": walk(&root),
    })
}

/// Walks one command and its visible subcommands.
fn walk(cmd: &Command) -> serde_json::Value {
    let mut children = Vec::new();
    for sub in cmd.get_subcommands() {
        if sub.is_hide_set() {
            continue;
        }
        children.push(walk(sub));
    }
    let args: Vec<serde_json::Value> = cmd
        .get_arguments()
        .filter(|a| !a.is_hide_set())
        // `help` and `version` are clap plumbing, not product surface.
        .filter(|a| {
            let id = a.get_id().as_str();
            id != "help" && id != "version"
        })
        .map(describe)
        .collect();
    let aliases: Vec<&str> = cmd.get_all_aliases().collect();
    serde_json::json!({
        "name": cmd.get_name(),
        "about": cmd.get_about().map(ToString::to_string),
        "aliases": aliases,
        "args": args,
        "subcommands": children,
    })
}

/// Describes one argument without inventing a schema engine.
fn describe(arg: &clap::Arg) -> serde_json::Value {
    use clap::ArgAction;
    let action = arg.get_action();
    serde_json::json!({
        "id": arg.get_id().as_str(),
        "long": arg.get_long(),
        "short": arg.get_short().map(|c| c.to_string()),
        "value_name": arg
            .get_value_names()
            .and_then(|names| names.first())
            .map(ToString::to_string),
        "required": arg.is_required_set(),
        // SetTrue/Count never take a value, and Append accumulates; both facts
        // decide whether an agent can pass the flag twice or at all.
        "repeatable": matches!(action, ArgAction::Append | ArgAction::Count),
        "takes_value": !matches!(action, ArgAction::SetTrue | ArgAction::Count),
        "hidden": arg.is_hide_set(),
    })
}
