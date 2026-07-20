// SPDX-License-Identifier: MIT OR Apache-2.0
// G-CLOSE-04: pure module — no `unsafe` permitted.
#![forbid(unsafe_code)]
//! macOS platform specifics (G-E2E-18 multi-OS).
//!
//! Config and data dirs are resolved via `directories::ProjectDirs`
//! (`~/Library/Application Support/…`). No custom entitlements are required
//! for this one-shot SSH client (no camera/mic/network extension claims).
//! Gatekeeper quarantine is a distribution concern — see `docs/CROSS_PLATFORM.md`.
//!
//! ## SSH agent
//!
//! Default agent socket is typically `$SSH_AUTH_SOCK` (OS session). Product
//! code never stores agent paths in env product config; registry may persist
//! `agent_socket` from `--agent-socket` (XDG `config.toml` only).

use tracing::debug;

/// Initializes macOS-specific behavior (currently observational only).
pub fn initialize() {
    debug!(
        os = "macos",
        arch = std::env::consts::ARCH,
        "macOS platform initialized (ProjectDirs for config; no entitlements)"
    );
}

/// Returns true when the process appears to run under macOS (compile-time).
#[must_use]
pub const fn is_macos_target() -> bool {
    cfg!(target_os = "macos")
}

/// Documented default agent socket env name (OS boundary — not product store).
pub const OS_SSH_AUTH_SOCK_NAME: &str = "SSH_AUTH_SOCK";
