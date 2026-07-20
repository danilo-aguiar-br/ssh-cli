// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! PEM load helpers via `rustls-pki-types` [`PemObject`] (no rustls-pemfile).

use std::path::Path;

use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use crate::errors::{SshCliError, SshCliResult};

/// Loads a certificate chain from a PEM file (one or more CERTIFICATE blocks).
///
/// # Errors
/// Missing path → [`SshCliError::FileNotFound`] (permanent). Empty/invalid PEM →
/// [`SshCliError::InvalidArgument`] (permanent). Network dial stays on [`SshCliError::Tls`].
pub fn load_cert_chain(path: &Path) -> SshCliResult<Vec<CertificateDer<'static>>> {
    if !path.exists() {
        return Err(SshCliError::FileNotFound(path.display().to_string()));
    }
    let iter = CertificateDer::pem_file_iter(path).map_err(|e| {
        SshCliError::InvalidArgument(format!("read/parse cert {}: {e}", path.display()))
    })?;
    let mut certs = Vec::new();
    for item in iter {
        let cert = item.map_err(|e| {
            SshCliError::InvalidArgument(format!("parse cert item in {}: {e}", path.display()))
        })?;
        certs.push(cert);
    }
    if certs.is_empty() {
        return Err(SshCliError::InvalidArgument(format!(
            "no certificates in {}",
            path.display()
        )));
    }
    Ok(certs)
}

/// Loads a single private key from a PEM file (PKCS#8 / RSA / SEC1).
///
/// # Errors
/// Missing path → [`SshCliError::FileNotFound`]. Invalid PEM → [`SshCliError::InvalidArgument`].
pub fn load_private_key(path: &Path) -> SshCliResult<PrivateKeyDer<'static>> {
    if !path.exists() {
        return Err(SshCliError::FileNotFound(path.display().to_string()));
    }
    PrivateKeyDer::from_pem_file(path).map_err(|e| {
        SshCliError::InvalidArgument(format!("read/parse key {}: {e}", path.display()))
    })
}
