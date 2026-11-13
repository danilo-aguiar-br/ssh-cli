// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: primary-key initialization extracted from `secrets` (SRP; line budget).
#![forbid(unsafe_code)]
//! Primary-key initialization: one key per profile, in the OS store **or** the file.
//!
//! # Why this module exists
//!
//! `secrets init` writes one of two stores. Which store answers later is decided by
//! [`super::load_primary_key`], and that order is fixed: flag, then `secrets.key`, then
//! the OS store. A write that lands in the store while a file survives therefore loses
//! every later read, silently. Keeping both the guard and the retirement here makes the
//! invariant reviewable in one place: a profile owns exactly one primary key.

use super::{
    delete_key_from_keyring, generate_hex_key, read_keyring, secrets_key_path, secrets_status,
    write_key_file, write_key_to_keyring, SecretsStatus,
};
use crate::errors::{SshCliError, SshCliResult};
use std::path::Path;
use zeroize::Zeroize;

/// Initializes the primary key in the XDG file or in the OS store. **Never** prints it.
///
/// # Errors
/// Returns an error when the profile already has a key and `force` is false, when the
/// RNG fails, or when the selected store cannot be written.
pub fn init_primary_key(use_keyring: bool, force: bool) -> SshCliResult<SecretsStatus> {
    let path = secrets_key_path()?;
    // A profile has ONE primary key, and it may live in the OS store or in
    // `secrets.key`, never in both. The old code asked only the destination store
    // whether a key already existed, so a profile holding a file key accepted
    // `--keyring` and minted a second, different key. From then on the file answered
    // first on the next read, `secrets status` reported `xdg_file` although `init`
    // had answered `keyring`, and every secret already at rest became undecryptable
    // with the key the operator had just been told was in use.
    //
    // The guard is therefore about the profile, not about the destination: any
    // existing source blocks the write unless `force` was given, and `force` retires
    // the other source so exactly one key survives the rotation.
    if !force {
        if let Some(other) = existing_source(use_keyring, &path)? {
            return Err(SshCliError::InvalidArgument(format!(
                "profile already has a primary-key in {other}; use --force to rotate it"
            )));
        }
    }
    let mut hex = generate_hex_key()?;
    if use_keyring {
        let result = write_key_to_keyring(&hex);
        hex.zeroize();
        result?;
        // Moving into the store retires the file: leaving it behind would shadow the
        // new key on every later read.
        retire_file(&path);
        return secrets_status();
    }
    let result = write_key_file(&path, &hex, force);
    hex.zeroize();
    result?;
    // The symmetric case: rotating from a store profile back to a file. The store
    // entry is retired so the file is the only key left. Only under `force`, because
    // that is the only path that can leave two keys behind, and an unconditional
    // delete would probe the store on every plain `secrets init`.
    if force {
        let _ = delete_key_from_keyring();
    }
    secrets_status()
}

/// Names the store that already holds this profile's key, if any.
///
/// Returns `None` only when the profile is genuinely empty. A store that cannot be
/// reached is an error when it is the destination, because writing over a source that
/// could not be read is how a second key gets minted over a live one.
fn existing_source(use_keyring: bool, path: &Path) -> SshCliResult<Option<String>> {
    // The destination is asked first, so the message keeps naming the case the old
    // code already refused.
    if use_keyring {
        match read_keyring() {
            Ok(Some(_)) => return Ok(Some("the OS keyring".to_string())),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
        if path.is_file() {
            return Ok(Some(path.display().to_string()));
        }
        return Ok(None);
    }
    if path.is_file() {
        return Ok(Some(path.display().to_string()));
    }
    match read_keyring() {
        Ok(Some(_)) => Ok(Some("the OS keyring".to_string())),
        Ok(None) => Ok(None),
        // A store that does not answer cannot confirm the profile is empty, but the
        // caller is writing to the file, so this source is not the one at risk.
        // Failing here would make `secrets init` unusable on a host with no session
        // bus, which fails on a store the caller never asked for.
        Err(e) => {
            tracing::debug!(err = %e, "keyring not reachable while checking for an existing key");
            Ok(None)
        }
    }
}

/// Removes `secrets.key` after the profile's key moved into the OS store.
///
/// Best-effort by design: the key is already readable in the store at this point, so a
/// file that cannot be unlinked must not turn a successful rotation into a failure. It
/// is still logged, because the leftover file is exactly what would shadow the new key.
fn retire_file(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => {
            tracing::info!(
                path = %path.display(),
                "removed the XDG key file after moving the key into the OS store"
            );
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            tracing::warn!(
                err = %e,
                path = %path.display(),
                "the XDG key file survived the move into the OS store and will shadow it"
            );
        }
    }
}
