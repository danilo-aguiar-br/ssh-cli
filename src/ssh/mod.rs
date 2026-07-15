// SPDX-License-Identifier: MIT OR Apache-2.0
//! SSH engine via `russh` 0.62.x.
//!
//! - `client`: one-shot connection, password/key auth, exec with timeout and abort
//! - `known_hosts`: TOFU fingerprints under XDG
//! - `packing`: safe sudo/su packing for one-shot automation

pub mod client;
pub mod known_hosts;
pub mod packing;

pub use client::{truncate_utf8, SshClient, ConnectionConfig, ExecutionOutput};
pub use packing::{
    append_description, pack_abort_pkill, pack_su, pack_sudo,
    escape_shell_single_quotes, remote_abort_pattern,
};
