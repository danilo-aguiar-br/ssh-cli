// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SFTP-07 / G-SFTP-12: SFTP clap surface extracted from cli/mod (SRP).
#![forbid(unsafe_code)]
//! Clap types for `ssh-cli sftp …`.

use super::SshAuthArgs;
use clap::{ArgAction, Subcommand};

/// Actions of the `sftp` subcommand (SFTP v3 subsystem).
#[derive(Debug, Subcommand)]
pub enum SftpAction {
    /// Uploads a local file or tree to the remote host via SFTP.
    Upload {
        /// Upload to every registered host (bounded concurrency).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "hosts")]
        all: bool,
        /// Comma-separated host subset (bounded fan-out).
        #[arg(long, value_name = "LIST", conflicts_with = "all")]
        hosts: Option<String>,
        /// Paths: `VPS LOCAL REMOTE`, multi-file `VPS LOCAL... REMOTE_DIR`, or fleet forms with `--all`/`--hosts`.
        #[arg(required = true, num_args = 2.., value_names = ["VPS", "LOCAL", "REMOTE"])]
        target: Vec<String>,
        /// Recursively upload a local directory tree (no symlink follow).
        #[arg(short = 'r', long, action = ArgAction::SetTrue)]
        recursive: bool,
        /// SSH authentication overrides.
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Timeout override in milliseconds (connect+transfer).
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Downloads a remote file or tree to the local host via SFTP.
    Download {
        /// Download from every registered host (bounded concurrency).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "hosts")]
        all: bool,
        /// Comma-separated host subset (bounded fan-out).
        #[arg(long, value_name = "LIST", conflicts_with = "all")]
        hosts: Option<String>,
        /// Paths: `VPS REMOTE LOCAL`, multi-file forms, or fleet forms with `--all`/`--hosts`.
        #[arg(required = true, num_args = 2.., value_names = ["VPS", "REMOTE", "LOCAL"])]
        target: Vec<String>,
        /// Recursively download a remote directory tree (no symlink follow).
        #[arg(short = 'r', long, action = ArgAction::SetTrue)]
        recursive: bool,
        /// SSH authentication overrides.
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Timeout override in milliseconds (connect+transfer).
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Lists a remote directory.
    Ls {
        /// VPS name.
        vps_name: String,
        /// Remote directory path.
        remote: String,
        /// SSH authentication overrides.
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Timeout override in milliseconds.
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Creates a remote directory.
    Mkdir {
        /// VPS name.
        vps_name: String,
        /// Remote directory path.
        remote: String,
        /// SSH authentication overrides.
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Timeout override in milliseconds.
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Removes an empty remote directory.
    Rmdir {
        /// VPS name.
        vps_name: String,
        /// Remote directory path.
        remote: String,
        /// SSH authentication overrides.
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Timeout override in milliseconds.
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Removes a remote file.
    Rm {
        /// VPS name.
        vps_name: String,
        /// Remote file path.
        remote: String,
        /// SSH authentication overrides.
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Timeout override in milliseconds.
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Shows metadata for a remote path.
    Stat {
        /// VPS name.
        vps_name: String,
        /// Remote path.
        remote: String,
        /// SSH authentication overrides.
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Timeout override in milliseconds.
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Renames a remote path.
    Rename {
        /// VPS name.
        vps_name: String,
        /// Current remote path.
        from: String,
        /// New remote path.
        to: String,
        /// SSH authentication overrides.
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Timeout override in milliseconds.
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },
}
