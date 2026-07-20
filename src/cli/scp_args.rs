// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SFTP-R12: SCP clap surface extracted from cli/mod (SRP; parity with sftp_args).
#![forbid(unsafe_code)]
//! Clap types for `ssh-cli scp …` (regular files only).

use super::SshAuthArgs;
use clap::{ArgAction, Subcommand};

/// Actions of the `scp` subcommand (regular files only; no `-r`).
///
/// Directory trees use [`super::SftpAction`] (`ssh-cli sftp … --recursive`).
#[derive(Debug, Subcommand)]
pub enum ScpAction {
    /// Uploads a local file to the remote host (regular files only).
    Upload {
        /// Upload to every registered host (bounded concurrency).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "hosts")]
        all: bool,
        /// Comma-separated host subset (bounded fan-out).
        #[arg(long, value_name = "LIST", conflicts_with = "all")]
        hosts: Option<String>,
        /// Paths: `VPS LOCAL REMOTE`, or `VPS LOCAL... REMOTE_DIR` (multi-file, one session),
        /// or with `--all`/`--hosts`: `LOCAL REMOTE` (one file) or `LOCAL... REMOTE_DIR` (multi-file × fleet).
        #[arg(required = true, num_args = 2.., value_names = ["VPS", "LOCAL", "REMOTE"])]
        target: Vec<String>,
        /// SSH authentication overrides (password/key/passphrase).
        #[command(flatten)]
        auth: SshAuthArgs,
        /// SSH timeout override in milliseconds (covers connect+transfer).
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Downloads a remote file to the local host (regular files only).
    Download {
        /// Download from every registered host (bounded concurrency).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "hosts")]
        all: bool,
        /// Comma-separated host subset (bounded fan-out).
        #[arg(long, value_name = "LIST", conflicts_with = "all")]
        hosts: Option<String>,
        /// Paths: `VPS REMOTE LOCAL`, or `VPS REMOTE... LOCAL_DIR` (multi-file, one session),
        /// or with `--all`/`--hosts`: `REMOTE LOCAL` (prefix) or `REMOTE... LOCAL_DIR` (per-host subdirs).
        #[arg(required = true, num_args = 2.., value_names = ["VPS", "REMOTE", "LOCAL"])]
        target: Vec<String>,
        /// SSH authentication overrides (password/key/passphrase).
        #[command(flatten)]
        auth: SshAuthArgs,
        /// SSH timeout override in milliseconds (covers connect+transfer).
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },
}
