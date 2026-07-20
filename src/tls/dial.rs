// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! TCP + rustls handshake for SSH-over-TLS.

use std::sync::Arc;

use rustls::pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream as TokioTlsStream;
use tokio_rustls::TlsConnector;

use super::client_config::{build_client_config, TlsClientOptions};
use super::TlsConnectOptions;
use crate::errors::{SshCliError, SshCliResult};

/// Product TLS stream type (client half over TCP).
pub type TlsStream = TokioTlsStream<TcpStream>;

/// Dials `host:port`, completes TLS with SNI/`TlsConnectOptions`, returns the stream.
///
/// # Errors
/// DNS/TCP failure, invalid SNI, handshake failure, or PEM/mTLS config errors.
pub async fn dial_tls(host: &str, port: u16, opts: &TlsConnectOptions) -> SshCliResult<TlsStream> {
    let server_name = ServerName::try_from(opts.sni.as_str())
        .map_err(|e| SshCliError::tls_msg(format!("invalid TLS SNI '{}': {e}", opts.sni)))?
        .to_owned();

    let client_opts = TlsClientOptions {
        client_cert: opts.client_cert.clone(),
        client_key: opts.client_key.clone(),
        extra_root_pem: None,
    };
    let config = build_client_config(&client_opts)?;
    let connector = TlsConnector::from(config);

    let tcp = crate::net::dial_tcp(host, port).await.map_err(|e| {
        SshCliError::ConnectionFailed(format!("TCP dial failed for {host}:{port}: {e}"))
    })?;
    if let Err(e) = tcp.set_nodelay(true) {
        tracing::debug!(err = %e, "set_nodelay on TLS socket failed");
    }

    tracing::info!(
        host,
        port,
        sni = %opts.sni,
        mtls = opts.client_cert.is_some(),
        "starting TLS handshake (SSH-over-TLS)"
    );

    connector
        .connect(server_name, tcp)
        .await
        .map_err(|e| SshCliError::tls_msg(format!("TLS handshake failed for {host}:{port}: {e}")))
}

/// Dials with a pre-built shared [`rustls::ClientConfig`] (library extension point).
#[allow(dead_code)] // public extension for embedders / future CLI overrides
pub async fn dial_tls_with_config(
    host: &str,
    port: u16,
    sni: &str,
    config: Arc<rustls::ClientConfig>,
) -> SshCliResult<TlsStream> {
    let server_name = ServerName::try_from(sni)
        .map_err(|e| SshCliError::tls_msg(format!("invalid TLS SNI '{sni}': {e}")))?
        .to_owned();
    let connector = TlsConnector::from(config);
    let tcp = crate::net::dial_tcp(host, port).await.map_err(|e| {
        SshCliError::ConnectionFailed(format!("TCP dial failed for {host}:{port}: {e}"))
    })?;
    let _ = tcp.set_nodelay(true);
    connector
        .connect(server_name, tcp)
        .await
        .map_err(|e| SshCliError::tls_msg(format!("TLS handshake failed for {host}:{port}: {e}")))
}
