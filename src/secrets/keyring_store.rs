// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted.
#![forbid(unsafe_code)]
//! OS keyring access for the primary key.
//!
//! Isolated because it is the only part of the secrets subsystem that talks to an
//! external service, and therefore the only part whose failures are transient
//! (exit 69, G-ERR-R01) rather than caused by the caller's input. Keeping it beside
//! pure file and AEAD logic is what let those failures inherit the wrong
//! classification for so long.

use super::{parse_hex_key, PrimaryKey};
use crate::constants::{KEYRING_SERVICE, KEYRING_USER_LEGACY, KEYRING_USER_PRIMARY};
use crate::errors::{SshCliError, SshCliResult};
use zeroize::Zeroize;

/// Stores the primary key (hex) in the OS keyring. Never prints the key.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] for malformed hex; [`SshCliError::Unavailable`]
/// (exit 69) when the keyring service does not answer.
pub fn write_key_to_keyring(hex64: &str) -> SshCliResult<()> {
    let _ = parse_hex_key(hex64)
        .map_err(|e| SshCliError::InvalidArgument(format!("invalid key: {e}")))?;
    // G-ERR-R01: the keyring is a host *service*. Formatting its error into a `Config`
    // string also leaked the backend's own message into the envelope, which on some
    // Linux secret-service implementations echoes the entry label back.
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER_PRIMARY).map_err(|e| {
        tracing::debug!(err = %e, "keyring Entry::new failed");
        SshCliError::unavailable("keyring")
    })?;
    entry.set_password(hex64.trim()).map_err(|e| {
        tracing::debug!(err = %e, "keyring set failed");
        SshCliError::unavailable("keyring")
    })?;
    Ok(())
}

/// Reads the primary key from the OS keyring, if present.
///
/// # Errors
/// [`SshCliError::Unavailable`] when the legacy entry exists but the service fails.
pub fn read_keyring() -> SshCliResult<Option<PrimaryKey>> {
    // Prefer inclusive primary-key id; fall back to legacy master-key user for migration.
    for user in [KEYRING_USER_PRIMARY, KEYRING_USER_LEGACY] {
        let entry = match keyring::Entry::new(KEYRING_SERVICE, user) {
            Ok(e) => e,
            Err(e) => {
                if user == "secrets-master-key" {
                    tracing::debug!(err = %e, "keyring Entry::new failed");
                    return Err(SshCliError::unavailable("keyring"));
                }
                continue;
            }
        };
        match entry.get_password() {
            Ok(mut s) => {
                let key = parse_hex_key(&s).map_err(|e| {
                    SshCliError::InvalidArgument(format!("invalid keyring primary-key: {e}"))
                });
                s.zeroize();
                return Ok(Some(key?));
            }
            Err(keyring::Error::NoEntry) => continue,
            Err(e) => {
                if user == "secrets-master-key" {
                    tracing::debug!(err = %e, "keyring get failed");
                    return Err(SshCliError::unavailable("keyring"));
                }
                continue;
            }
        }
    }
    Ok(None)
}
