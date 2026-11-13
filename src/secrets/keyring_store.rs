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
//!
//! # Why the store is checked before the call
//!
//! A `keyring` build without a store feature has no producer at all, and every
//! operation answered `Ok` from an in-process mock while nothing reached the system.
//! `Entry::store_status()` is the only honest signal: it reports the one-time store
//! initialization, so a missing backend fails closed with exit 69 instead of returning
//! success for a write that never happened.
//!
//! # Why the entry is scoped by profile
//!
//! The OS store is machine-wide, while `secrets.key` and `config.toml` are per config
//! directory. Without a scope, an isolated run with `--config-dir` would overwrite the
//! key of the real profile, and two profiles would read each other's key. The user name
//! carries a digest of the resolved config directory, so a profile only ever finds its
//! own entry.
//!
//! # Why the calls run on their own thread
//!
//! The Linux store talks D-Bus through `zbus`, whose blocking API parks the calling
//! thread on an `async-io` reactor. `dispatch_impl` is async and runs inside the
//! multi-thread Tokio runtime, so the call is handed to a scoped OS thread instead of
//! occupying a runtime worker. It happens at most once per process.

use super::{parse_hex_key, PrimaryKey};
use crate::constants::{KEYRING_SERVICE, KEYRING_USER_LEGACY, KEYRING_USER_PRIMARY};
use crate::errors::{SshCliError, SshCliResult, UnavailableService};
use zeroize::Zeroize;

/// Digest of the profile directory that scopes the keyring entry.
///
/// FNV-1a over the normalized path: no dependency, no allocation churn and stable
/// across runs. It is an identifier, not a secret, so it does not need to resist
/// attack. Normalization keeps the same profile addressable on Windows, where the
/// separator and the case of a path are not significant.
fn scope_key(dir: &std::path::Path) -> String {
    let text = dir.to_string_lossy().replace('\\', "/");
    let text = if cfg!(windows) {
        text.to_lowercase()
    } else {
        text
    };
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// User names to try, most specific first.
///
/// The scoped name is the current one. The unscoped primary-key name and the legacy
/// master-key name stay readable so a profile created by an earlier version keeps
/// working after the upgrade.
fn candidate_users() -> Vec<String> {
    let mut users = Vec::with_capacity(3);
    match super::secrets_config_dir() {
        Ok(dir) => users.push(format!("{KEYRING_USER_PRIMARY}:{}", scope_key(&dir))),
        Err(e) => {
            tracing::debug!(err = %e, "config dir unresolved; using the unscoped keyring user");
        }
    }
    users.push(KEYRING_USER_PRIMARY.to_string());
    users.push(KEYRING_USER_LEGACY.to_string());
    users
}

/// Fails closed when the compiled-in credential store did not initialize.
///
/// # Errors
/// [`SshCliError::Unavailable`] (exit 69) when no store backend answers.
fn ensure_store() -> SshCliResult<()> {
    match keyring::Entry::store_status() {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::debug!(err = %e, "keyring credential store unavailable");
            Err(SshCliError::unavailable(UnavailableService::Keyring))
        }
    }
}

/// Runs a keyring operation on a dedicated thread and joins it.
///
/// A panic inside the backend becomes exit 69 instead of aborting the whole command,
/// and the runtime worker is never parked on the external service.
fn on_keyring_thread<T>(op: impl FnOnce() -> SshCliResult<T> + Send) -> SshCliResult<T>
where
    T: Send,
{
    std::thread::scope(|scope| match scope.spawn(op).join() {
        Ok(result) => result,
        Err(_) => {
            tracing::error!("keyring worker panicked");
            Err(SshCliError::unavailable(UnavailableService::Keyring))
        }
    })
}

/// Stores the primary key (hex) in the OS keyring. Never prints the key.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] for malformed hex; [`SshCliError::Unavailable`]
/// (exit 69) when the keyring service does not answer.
pub fn write_key_to_keyring(hex64: &str) -> SshCliResult<()> {
    let _ = parse_hex_key(hex64)
        .map_err(|e| SshCliError::InvalidArgument(format!("invalid key: {e}")))?;
    ensure_store()?;
    let user = candidate_users()
        .into_iter()
        .next()
        .unwrap_or_else(|| KEYRING_USER_PRIMARY.to_string());
    on_keyring_thread(move || {
        // G-ERR-R01: the keyring is a host *service*. Formatting its error into a
        // `Config` string also leaked the backend's own message into the envelope,
        // which on some Linux secret-service implementations echoes the label back.
        let entry = keyring::Entry::new(KEYRING_SERVICE, &user).map_err(|e| {
            tracing::debug!(err = %e, "keyring Entry::new failed");
            SshCliError::unavailable(UnavailableService::Keyring)
        })?;
        entry.set_password(hex64.trim()).map_err(|e| {
            tracing::debug!(err = %e, "keyring set failed");
            SshCliError::unavailable(UnavailableService::Keyring)
        })
    })
}

/// Removes the entry this profile wrote, and only that one.
///
/// The store is machine-wide, so deleting "every candidate name" would reach the unscoped
/// and legacy entries, which belong to whichever profile wrote them. A cleanup that
/// touches a name it cannot attribute is a cleanup that can destroy another profile's
/// key, so this removes the scoped name only and refuses to fall back.
///
/// # Errors
/// [`SshCliError::Unavailable`] when the store does not answer or the config directory
/// cannot be resolved, because without the scope there is no safe name to delete. A
/// missing entry is not an error: the caller asked for it to be gone, and it is.
pub fn delete_key_from_keyring() -> SshCliResult<()> {
    let dir = super::secrets_config_dir()?;
    let user = format!("{KEYRING_USER_PRIMARY}:{}", scope_key(&dir));
    ensure_store()?;
    on_keyring_thread(move || {
        let entry = keyring::Entry::new(KEYRING_SERVICE, &user).map_err(|e| {
            tracing::debug!(err = %e, "keyring Entry::new failed");
            SshCliError::unavailable(UnavailableService::Keyring)
        })?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => {
                tracing::debug!(err = %e, "keyring delete failed");
                Err(SshCliError::unavailable(UnavailableService::Keyring))
            }
        }
    })
}

/// Reads the primary key from the OS keyring, if present.
///
/// # Errors
/// [`SshCliError::Unavailable`] when the store answers but the read fails for a reason
/// other than a missing entry.
pub fn read_keyring() -> SshCliResult<Option<PrimaryKey>> {
    ensure_store()?;
    on_keyring_thread(|| {
        let users = candidate_users();
        let last = users.len().saturating_sub(1);
        for (index, user) in users.into_iter().enumerate() {
            let is_last = index == last;
            let entry = match keyring::Entry::new(KEYRING_SERVICE, &user) {
                Ok(e) => e,
                Err(e) => {
                    if is_last {
                        tracing::debug!(err = %e, "keyring Entry::new failed");
                        return Err(SshCliError::unavailable(UnavailableService::Keyring));
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
                    if is_last {
                        tracing::debug!(err = %e, "keyring get failed");
                        return Err(SshCliError::unavailable(UnavailableService::Keyring));
                    }
                    continue;
                }
            }
        }
        Ok(None)
    })
}
