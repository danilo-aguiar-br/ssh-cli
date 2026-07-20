// SPDX-License-Identifier: MIT OR Apache-2.0
// G-TLS product stack — pure module: no `unsafe`.
#![forbid(unsafe_code)]
//! Product TLS via **rustls** (aws_lc_rs only): SSH-over-TLS, mTLS, ACME.
//!
//! # Design (Rules Rust — rustls)
//!
//! | Concern | Rule |
//! |---------|------|
//! | Provider | `CryptoProvider::install_default` **once** in binary `main` |
//! | Libraries | use `ClientConfig::builder` / `get_default` — never reinstall |
//! | Stack | rustls ≥ 0.23.18 only; no `native-tls` / OpenSSL / `ring` |
//! | Storage | XDG under `tls/` — no product env vars for cert material |
//! | Secrets | PEM keys 0o600; never log private key material |
//!
//! # Workload
//!
//! I/O-bound (TCP + TLS handshake). No Rayon. Multi-host fan-out stays in callers.

#[cfg(feature = "tls")]
mod acme;
#[cfg(feature = "tls")]
mod acme_error_map;
#[cfg(feature = "tls")]
mod client_config;
#[cfg(feature = "tls")]
pub mod commands;
#[cfg(feature = "tls")]
mod dial;
#[cfg(feature = "tls")]
mod mtls;
#[cfg(feature = "tls")]
mod paths;
#[cfg(feature = "tls")]
mod pem;
#[cfg(feature = "tls")]
mod provider;

#[cfg(feature = "tls")]
pub use acme::{
    acme_complete, acme_issue_print_challenge, acme_list, acme_status, create_account,
    load_account_status, AcmeDirectory,
};
#[cfg(feature = "tls")]
pub use client_config::{build_client_config, TlsClientOptions};
#[cfg(feature = "tls")]
pub use dial::{dial_tls, TlsStream};
#[cfg(feature = "tls")]
pub use mtls::{
    mtls_import, mtls_list, mtls_remove, mtls_show, resolve_mtls_paths, MtlsIdentity,
};
#[cfg(feature = "tls")]
pub use paths::{
    acme_account_path, acme_domain_dir, mtls_identity_dir, resolve_tls_root, tls_root_dir,
};
#[cfg(feature = "tls")]
pub use provider::{install_default_provider, provider_is_installed, provider_name};

/// Options for wrapping the SSH TCP path in TLS (SSH-over-TLS).
///
/// When present on [`crate::ssh::ConnectionConfig`], the client dials TCP,
/// completes a rustls handshake (optional mTLS), then runs SSH on the TLS stream.
#[derive(Debug, Clone)]
pub struct TlsConnectOptions {
    /// DNS name for SNI + certificate verification (usually the VPS host).
    pub sni: String,
    /// Optional client certificate PEM path (mTLS).
    pub client_cert: Option<std::path::PathBuf>,
    /// Optional client private key PEM path (mTLS; required with cert).
    pub client_key: Option<std::path::PathBuf>,
}

impl TlsConnectOptions {
    /// Builds options from host + optional mTLS paths.
    ///
    /// # Errors
    /// Returns [`crate::errors::SshCliError::InvalidArgument`] when only one of
    /// cert/key is set or SNI is empty.
    pub fn try_new(
        sni: impl Into<String>,
        client_cert: Option<std::path::PathBuf>,
        client_key: Option<std::path::PathBuf>,
    ) -> crate::errors::SshCliResult<Self> {
        let sni = sni.into();
        let sni_trim = sni.trim();
        if sni_trim.is_empty() {
            return Err(crate::errors::SshCliError::InvalidArgument(
                "TLS SNI cannot be empty".into(),
            ));
        }
        match (&client_cert, &client_key) {
            (Some(_), None) | (None, Some(_)) => {
                return Err(crate::errors::SshCliError::InvalidArgument(
                    "mTLS requires both client cert and key paths".into(),
                ));
            }
            _ => {}
        }
        Ok(Self {
            sni: sni_trim.to_owned(),
            client_cert,
            client_key,
        })
    }
}

/// Stub when feature `tls` is disabled: install is a no-op; dials fail closed.
#[cfg(not(feature = "tls"))]
pub mod disabled {
    use crate::errors::SshCliResult;

    /// No-op without the TLS feature (binary still runs plain SSH).
    pub fn install_default_provider() -> SshCliResult<()> {
        Ok(())
    }

    /// Always false without the feature.
    #[must_use]
    pub fn provider_is_installed() -> bool {
        false
    }

    /// Placeholder name when TLS is not compiled in.
    #[must_use]
    pub fn provider_name() -> &'static str {
        "disabled"
    }
}

#[cfg(not(feature = "tls"))]
pub use disabled::{install_default_provider, provider_is_installed, provider_name};

#[cfg(all(test, feature = "tls"))]
mod tests {
    use super::*;

    #[test]
    fn tls_options_reject_partial_mtls() {
        let err = TlsConnectOptions::try_new(
            "example.com",
            Some(std::path::PathBuf::from("c.pem")),
            None,
        )
        .unwrap_err();
        assert!(err.to_string().contains("mTLS"));
    }

    #[test]
    fn tls_options_reject_empty_sni() {
        let err = TlsConnectOptions::try_new("  ", None, None).unwrap_err();
        assert!(err.to_string().contains("SNI"));
    }

    #[test]
    fn tls_options_ok() {
        let o = TlsConnectOptions::try_new("host.example", None, None).unwrap();
        assert_eq!(o.sni, "host.example");
    }
}
