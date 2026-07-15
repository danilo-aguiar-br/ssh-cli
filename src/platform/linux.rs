// SPDX-License-Identifier: MIT OR Apache-2.0
//! Particularidades da plataforma Linux.
//!
//! Detects sandboxes (Flatpak, Snap) and resolves XDG paths.

use tracing::debug;

/// Detects whether ssh-cli is running inside a sandbox.
pub fn detectar_sandbox() {
    if std::env::var("FLATPAK_ID").is_ok() {
        debug!("executando dentro de sandbox Flatpak");
    } else if std::env::var("SNAP").is_ok() {
        debug!("executando dentro de sandbox Snap");
    }
}

#[cfg(test)]
mod tests {
    use super::detectar_sandbox;
    use serial_test::serial;

    #[test]
    #[serial]
    fn detect_flatpak_sandbox_no_panic() {
        std::env::set_var("FLATPAK_ID", "org.teste.App");
        std::env::remove_var("SNAP");
        detectar_sandbox();
        std::env::remove_var("FLATPAK_ID");
    }

    #[test]
    #[serial]
    fn detect_snap_sandbox_no_panic() {
        std::env::remove_var("FLATPAK_ID");
        std::env::set_var("SNAP", "/snap/app");
        detectar_sandbox();
        std::env::remove_var("SNAP");
    }
}
