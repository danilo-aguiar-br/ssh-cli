// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SFTP: shared SFTP DTO types (always available; no russh-sftp dep).
#![forbid(unsafe_code)]
//! SFTP list/stat DTOs shared by the SSH engine and output layer.

/// Directory entry for `sftp ls` JSON/text.
#[derive(Debug, Clone)]
pub struct SftpListEntry {
    /// Base name.
    pub name: String,
    /// Full remote path.
    pub path: String,
    /// `file` | `dir` | `symlink` | `other`.
    pub kind: String,
    /// Size when known.
    pub size: Option<u64>,
    /// Mode bits when known.
    pub mode: Option<u32>,
}

/// Stat payload for `sftp stat`.
#[derive(Debug, Clone)]
pub struct SftpStat {
    /// Remote path queried.
    pub path: String,
    /// `file` | `dir` | `symlink` | `other`.
    pub kind: String,
    /// Size when known.
    pub size: Option<u64>,
    /// Mode bits when known.
    pub mode: Option<u32>,
    /// mtime unix seconds when known.
    pub mtime: Option<u32>,
}
