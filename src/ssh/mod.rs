// SPDX-License-Identifier: MIT OR Apache-2.0
// G-UNSAFE-01 / G-SECDEV-05: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! SSH engine via `russh` 0.62.x.
//!
//! - `client`: one-shot connection, password/key/agent auth, exec with timeout and abort
//! - `known_hosts`: TOFU fingerprints under XDG
//! - `packing`: safe sudo/su packing for one-shot automation
//! - `key_material`: private-key permissions + weak-RSA policy (G-SSH-03/07)
//! - `client_handler` / `client_connect`: host-key TOFU + auth chain (G-SSH-01/04/06)
//!
//! ## Features
//!
//! The concrete [`SshClient`] is feature-gated:
//! - **`ssh-real`** (default): real client via `russh`
//! - without default features: stub that fails connect with a clear error

pub mod client;
pub mod connection;
#[cfg(feature = "ssh-real")]
pub(crate) mod client_connect;
#[cfg(feature = "ssh-real")]
pub(crate) mod client_handler;
#[cfg(feature = "ssh-real")]
pub(crate) mod connect;
#[cfg(feature = "ssh-real")]
pub(crate) mod key_material;
pub mod known_hosts;
pub mod packing;
pub(crate) mod scp_wire;
pub(crate) mod session_io;
/// Pure remote path validation (G-SFTP-06/16).
pub(crate) mod sftp_path;
/// SFTP list/stat DTOs (always available).
pub(crate) mod sftp_types;
/// SFTP subsystem session + stream ops (G-SFTP; feature `ssh-real`).
#[cfg(feature = "ssh-real")]
pub(crate) mod sftp_session;

pub use sftp_types::{SftpListEntry, SftpStat};

pub use client::ExecutionOutput;
pub use connection::ConnectionConfig;
pub use session_io::truncate_utf8;
/// SSH client type (real or stub depending on feature `ssh-real`).
#[cfg_attr(docsrs, doc(cfg(feature = "ssh-real")))]
pub use client::SshClient;
pub use packing::{
    append_description, pack_abort_pkill, pack_su, pack_sudo,
    escape_shell_single_quotes, remote_abort_pattern,
};
