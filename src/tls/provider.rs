// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Process-wide rustls [`CryptoProvider`] bootstrap (binary only).
//!
//! Libraries must **not** call [`install_default_provider`]; they use
//! [`rustls::ClientConfig::builder`] which reads the process default, or
//! `builder_with_provider` when an explicit provider is required.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::errors::SshCliResult;

/// Set after a successful install (or when another party already installed).
static PROVIDER_READY: AtomicBool = AtomicBool::new(false);

/// Human-readable provider id for diagnostics / `tls provider` command.
pub const PROVIDER_NAME: &str = "aws_lc_rs";

/// Installs `aws_lc_rs::default_provider` once for the process.
///
/// Call from binary `main` **before** any TLS dial and preferably before the
/// Tokio runtime is built (rules: install → log → config → runtime → sockets).
///
/// Idempotent: a second call succeeds if the default is already installed
/// (including by another crate in the same process).
///
/// # Errors
/// Returns [`SshCliError::Tls`] when install fails for a reason other than
/// "already installed".
pub fn install_default_provider() -> SshCliResult<()> {
    if PROVIDER_READY.load(Ordering::Acquire) {
        return Ok(());
    }
    // Prefer get_default first — another bootstrap may have won the race.
    if rustls::crypto::CryptoProvider::get_default().is_some() {
        PROVIDER_READY.store(true, Ordering::Release);
        return Ok(());
    }
    match rustls::crypto::aws_lc_rs::default_provider().install_default() {
        Ok(()) => {
            PROVIDER_READY.store(true, Ordering::Release);
            tracing::debug!(provider = PROVIDER_NAME, "rustls CryptoProvider installed");
            Ok(())
        }
        Err(_already) => {
            // install_default returns Err(Arc<CryptoProvider>) when already set.
            PROVIDER_READY.store(true, Ordering::Release);
            tracing::debug!(
                provider = PROVIDER_NAME,
                "rustls CryptoProvider already installed by another caller"
            );
            Ok(())
        }
    }
}

/// Returns `true` when a process default provider is available.
#[must_use]
pub fn provider_is_installed() -> bool {
    PROVIDER_READY.load(Ordering::Acquire)
        || rustls::crypto::CryptoProvider::get_default().is_some()
}

/// Static provider name string (`aws_lc_rs`).
#[must_use]
pub fn provider_name() -> &'static str {
    PROVIDER_NAME
}

/// Ensures a provider is available for library builders (install if missing).
///
/// Prefer [`install_default_provider`] from `main`. This helper exists so
/// library code paths that run under tests without a binary bootstrap still
/// succeed once per process.
pub(crate) fn ensure_provider() -> SshCliResult<()> {
    install_default_provider()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_is_idempotent() {
        install_default_provider().expect("first");
        install_default_provider().expect("second");
        assert!(provider_is_installed());
        assert_eq!(provider_name(), "aws_lc_rs");
    }
}
