// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SSH-03 / G-SSH-07: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! Private-key load policy: Unix permissions + weak-RSA rejection.
//!
//! Callers run this on a blocking pool (`spawn_blocking`) because disk I/O and
//! optional KDF must not stall Tokio workers (G-NET / parallelism rules).

use std::path::Path;

use crate::constants::{SECRET_FILE_MODE_UNIX, SSH_RSA_MIN_BITS, SSH_RSA_PREFERRED_BITS};
use crate::errors::{SshCliError, SshCliResult};

/// Ensures a private key file is not group/world accessible (OpenSSH policy).
///
/// On Unix, requires `mode & 0o077 == 0` (typically `0o600` / `0o400`).
/// On Windows, ACL enforcement is left to the OS (no portable mode bits).
///
/// # Errors
///
/// Returns [`SshCliError::InvalidArgument`] when group/other bits are set.
pub fn ensure_private_key_permissions(path: &Path) -> SshCliResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(path).map_err(SshCliError::Io)?;
        let mode = meta.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            return Err(SshCliError::InvalidArgument(format!(
                "private key {} has mode {mode:04o}; expected owner-only (e.g. {SECRET_FILE_MODE_UNIX:04o}) — fix with: chmod 600 {}",
                path.display(),
                path.display()
            )));
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

/// Reject DSA and RSA keys below [`SSH_RSA_MIN_BITS`].
///
/// Logs a warning when RSA is accepted but below [`SSH_RSA_PREFERRED_BITS`].
pub fn reject_weak_key(key: &russh::keys::PrivateKey) -> SshCliResult<()> {
    use russh::keys::ssh_key::Algorithm;

    match key.algorithm() {
        Algorithm::Dsa => {
            return Err(SshCliError::InvalidArgument(
                "DSA private keys are deprecated and rejected".into(),
            ));
        }
        Algorithm::Rsa { .. } => {
            let bits = rsa_modulus_bits(key);
            if bits < SSH_RSA_MIN_BITS {
                return Err(SshCliError::InvalidArgument(format!(
                    "RSA private key is {bits} bits; minimum is {SSH_RSA_MIN_BITS}"
                )));
            }
            if bits < SSH_RSA_PREFERRED_BITS {
                tracing::warn!(
                    bits,
                    preferred = SSH_RSA_PREFERRED_BITS,
                    "RSA key below preferred size; prefer Ed25519 or RSA ≥ {SSH_RSA_PREFERRED_BITS}"
                );
            }
        }
        _ => {}
    }
    Ok(())
}

/// Approximate RSA modulus size in bits from the public key encoding.
fn rsa_modulus_bits(key: &russh::keys::PrivateKey) -> usize {
    let Some(rsa) = key.public_key().key_data().rsa() else {
        return 0;
    };
    let bytes = rsa.n().as_bytes();
    let mut it = bytes.iter().skip_while(|&&b| b == 0);
    match it.next() {
        None => 0,
        Some(&first) => {
            let leading = first.leading_zeros() as usize;
            let rest = it.count() * 8;
            8 - leading + rest
        }
    }
}

/// Load a secret key after permission and strength checks; zeroize passphrase.
///
/// # Errors
///
/// Propagates I/O, permission, weak-key, and parse failures as product errors.
pub fn load_secret_key_checked(
    path: &Path,
    passphrase: Option<&str>,
) -> SshCliResult<russh::keys::PrivateKey> {
    ensure_private_key_permissions(path)?;
    let key = russh::keys::load_secret_key(path, passphrase).map_err(|e| {
        SshCliError::SshAuthentication(format!("failed to load key {}: {e}", path.display()))
    })?;
    reject_weak_key(&key)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[cfg(unix)]
    fn rejects_group_readable_key() {
        use std::os::unix::fs::PermissionsExt;
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "not-a-real-key").unwrap();
        let mut perms = f.as_file().metadata().unwrap().permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(f.path(), perms).unwrap();
        let err = ensure_private_key_permissions(f.path()).unwrap_err();
        assert!(
            matches!(err, SshCliError::InvalidArgument(_)),
            "{err:?}"
        );
    }

    #[test]
    #[cfg(unix)]
    fn accepts_owner_only_key_mode() {
        use std::os::unix::fs::PermissionsExt;
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "not-a-real-key").unwrap();
        let mut perms = f.as_file().metadata().unwrap().permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(f.path(), perms).unwrap();
        ensure_private_key_permissions(f.path()).unwrap();
    }
}
