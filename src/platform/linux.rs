// SPDX-License-Identifier: MIT OR Apache-2.0
// G-CLOSE-04: pure module — no `unsafe` permitted.
#![forbid(unsafe_code)]
//! Linux platform specifics.
//!
//! Detects distribution sandboxes (Flatpak, Snap). XDG config paths are
//! resolved via `directories::ProjectDirs` in the VPS/secrets layers — not here.

use tracing::{debug, warn};

/// Detects whether ssh-cli is running inside a distribution sandbox.
///
/// Emits a **warning** (not silent debug) when Flatpak/Snap is detected so
/// operators notice constrained filesystem / network namespaces. ssh-cli does
/// not need browser sandboxes, but XDG paths and keyring backends can differ.
pub fn detect_sandbox() {
    if let Ok(id) = std::env::var("FLATPAK_ID") {
        warn!(
            flatpak_id = %id,
            "running inside Flatpak sandbox; prefer host install if XDG/keyring paths misbehave"
        );
    } else if let Ok(snap) = std::env::var("SNAP") {
        warn!(
            snap = %snap,
            "running inside Snap sandbox; prefer APT/RPM host install if filesystem access is restricted"
        );
    } else {
        debug!("no Flatpak/Snap sandbox markers detected");
    }
}

#[cfg(test)]
mod tests {
    use super::detect_sandbox;
    use serial_test::serial;

    #[test]
    #[serial]
    fn detect_flatpak_sandbox_no_panic() {
        let prev_f = std::env::var("FLATPAK_ID").ok();
        let prev_s = std::env::var("SNAP").ok();
        crate::test_util::env::set_var("FLATPAK_ID", "org.example.App");
        crate::test_util::env::remove_var("SNAP");
        detect_sandbox();
        match prev_f {
            Some(v) => crate::test_util::env::set_var("FLATPAK_ID", v),
            None => crate::test_util::env::remove_var("FLATPAK_ID"),
        }
        match prev_s {
            Some(v) => crate::test_util::env::set_var("SNAP", v),
            None => crate::test_util::env::remove_var("SNAP"),
        }
    }

    #[test]
    #[serial]
    fn detect_snap_sandbox_no_panic() {
        let prev_f = std::env::var("FLATPAK_ID").ok();
        let prev_s = std::env::var("SNAP").ok();
        crate::test_util::env::remove_var("FLATPAK_ID");
        crate::test_util::env::set_var("SNAP", "/snap/app");
        detect_sandbox();
        match prev_f {
            Some(v) => crate::test_util::env::set_var("FLATPAK_ID", v),
            None => crate::test_util::env::remove_var("FLATPAK_ID"),
        }
        match prev_s {
            Some(v) => crate::test_util::env::set_var("SNAP", v),
            None => crate::test_util::env::remove_var("SNAP"),
        }
    }
}
