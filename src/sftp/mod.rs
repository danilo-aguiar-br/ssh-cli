// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SFTP: SFTP CLI surface (one-shot; stream transfers; no full-file heap).
#![forbid(unsafe_code)]
//! SFTP subsystem operations over SSH (upload/download/ls/mkdir/rm/stat/rename).
//!
//! Complements SCP (regular-file wire). SFTP adds directory trees and FS ops.
//! Multi-host fan-out uses [`crate::concurrency::map_bounded`]. Multi-file on one
//! host reuses **one** SFTP session (G-SFTP-19).

use crate::cli::SftpAction;
use crate::constants::SFTP_FALLBACK_BASENAME;
use crate::errors::SshCliError;
use crate::i18n::{self, Message};
use crate::output;
use crate::ssh::client::{SshClient, TransferResult};
use crate::ssh::sftp_path::{ensure_local_under, validate_entry_name};
use crate::ssh::sftp_session;
use crate::ssh::sftp_types::{SftpListEntry, SftpStat};
use crate::vps;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub(crate) mod batch;

/// Runtime overrides for the `sftp` subcommand (parity with scp + agent G-SFTP-18).
mod dispatch;
mod emit;
mod setup;

pub use dispatch::run_sftp;
pub(crate) use emit::{emit_fs_op, emit_list, emit_stat, emit_transfer};
pub(crate) use setup::apply_sftp_options;
pub use setup::SftpOptions;
pub(crate) use setup::{connect_client, remote_str};
