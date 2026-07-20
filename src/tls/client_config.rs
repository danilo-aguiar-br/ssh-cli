// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! rustls [`ClientConfig`] construction (library-safe: no `install_default`).

use std::sync::Arc;

use rustls::ClientConfig;

use super::pem::{load_cert_chain, load_private_key};
use super::provider::ensure_provider;
use crate::errors::{SshCliError, SshCliResult};

/// Inputs for building a shared [`ClientConfig`].
#[derive(Debug, Clone, Default)]
pub struct TlsClientOptions {
    /// Client certificate PEM path (mTLS).
    pub client_cert: Option<std::path::PathBuf>,
    /// Client private key PEM path (mTLS).
    pub client_key: Option<std::path::PathBuf>,
    /// Extra PEM roots to **append** to Mozilla webpki-roots (optional).
    pub extra_root_pem: Option<std::path::PathBuf>,
}

/// Builds an [`Arc<ClientConfig>`] with webpki-roots and optional mTLS.
///
/// Uses the process default [`rustls::crypto::CryptoProvider`] (must be
/// installed by the binary via [`super::install_default_provider`]). Falls back
/// to installing aws_lc_rs once if missing (tests / embedders).
///
/// # Errors
/// Provider missing after ensure, PEM load failure, or rustls config error.
pub fn build_client_config(opts: &TlsClientOptions) -> SshCliResult<Arc<ClientConfig>> {
    ensure_provider()?;

    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    if let Some(ref extra) = opts.extra_root_pem {
        let extra_certs = load_cert_chain(extra)?;
        for c in extra_certs {
            roots
                .add(c)
                .map_err(|e| SshCliError::tls_msg(format!("add extra root: {e}")))?;
        }
    }

    let builder = ClientConfig::builder().with_root_certificates(roots);

    let config = match (&opts.client_cert, &opts.client_key) {
        (Some(cert), Some(key)) => {
            let chain = load_cert_chain(cert)?;
            let key_der = load_private_key(key)?;
            builder
                .with_client_auth_cert(chain, key_der)
                .map_err(|e| SshCliError::tls_msg(format!("mTLS client auth cert: {e}")))?
        }
        (None, None) => builder.with_no_client_auth(),
        _ => {
            return Err(SshCliError::InvalidArgument(
                "mTLS requires both client_cert and client_key".into(),
            ));
        }
    };

    Ok(Arc::new(config))
}

/// Loads only roots (no client auth) — used by unit tests / library defaults.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn root_only_config() -> SshCliResult<Arc<ClientConfig>> {
    build_client_config(&TlsClientOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_only_builds() {
        let cfg = root_only_config().expect("client config");
        // ClientConfig is opaque; Arc refcount proves construction.
        assert!(Arc::strong_count(&cfg) >= 1);
    }
}
