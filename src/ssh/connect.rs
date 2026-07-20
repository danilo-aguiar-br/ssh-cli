// SPDX-License-Identifier: MIT OR Apache-2.0
// G-TLS-07 / G-SECDEV-05 / G-SSH: pure module — no `unsafe` permitted.
#![forbid(unsafe_code)]
//! SSH connect helpers — client `Config` policy + TCP dial (G-TLS-04/07/09, G-SSH-02/05/08).
//!
//! Workload: I/O-bound dial; pure local construction of [`russh::client::Config`].
//! No Rayon. Multi-host fan-out stays in callers (`Semaphore` + `JoinSet`).
//!
//! # Crypto / compression policy (G-TLS)
//!
//! - Crypto backend is **aws-lc-rs** via russh feature `ssh-real` (not rustls).
//! - Channel compression prefers **`none` only** so zlib cannot be negotiated
//!   when secrets cross the channel (CRIME-class compression side-channels).
//! - `client_id` is product-generic (`SSH_CLIENT_ID`) — never the raw russh version string.

use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpStream;

use crate::constants::{
    SSH_CLIENT_ID, SSH_KEEPALIVE_INTERVAL_SECS, SSH_KEEPALIVE_MAX, SSH_MAX_PACKET_SIZE,
    SSH_REKEY_BYTE_LIMIT, SSH_REKEY_TIME_SECS, SSH_WINDOW_SIZE, TCP_KEEPALIVE_ENABLED,
};
use crate::errors::{SshCliError, SshCliResult};

/// Builds the process-local russh client config used for every real SSH dial.
///
/// # Policy
///
/// - `nodelay = true` for low-latency request/response SSH.
/// - Keepalives enabled for tunnel / long ops.
/// - **Compression:** only `none` is offered (G-TLS-04).
/// - **client_id:** generic product banner (G-SSH-02).
/// - **rekey / window:** explicit RFC 4253-class limits (G-SSH-08).
#[must_use]
pub(crate) fn build_ssh_client_config(timeout: Duration) -> Arc<russh::client::Config> {
    let mut preferred = russh::Preferred::DEFAULT;
    // Never offer zlib / zlib@openssh.com — secrets travel on this channel.
    preferred.compression = Cow::Borrowed(&[russh::compression::NONE]);

    Arc::new(russh::client::Config {
        client_id: russh::SshId::Standard(Cow::Borrowed(SSH_CLIENT_ID)),
        limits: russh::Limits::new(
            SSH_REKEY_BYTE_LIMIT,
            SSH_REKEY_BYTE_LIMIT,
            Duration::from_secs(SSH_REKEY_TIME_SECS),
        ),
        window_size: SSH_WINDOW_SIZE,
        maximum_packet_size: SSH_MAX_PACKET_SIZE,
        inactivity_timeout: Some(timeout),
        keepalive_interval: Some(Duration::from_secs(SSH_KEEPALIVE_INTERVAL_SECS)),
        keepalive_max: SSH_KEEPALIVE_MAX,
        nodelay: true,
        preferred,
        ..Default::default()
    })
}

/// Async DNS + Happy Eyeballs dial for `host:port`, mapped to product errors.
///
/// Applies `TCP_NODELAY` and optional `SO_KEEPALIVE` (G-SSH-05).
///
/// # Errors
///
/// Returns [`SshCliError::ConnectionFailed`] when every candidate fails or DNS
/// yields no addresses.
pub(crate) async fn dial_ssh(host: &str, port: u16) -> SshCliResult<TcpStream> {
    let socket = crate::net::dial_tcp(host, port).await.map_err(|e| {
        SshCliError::ConnectionFailed(format!("TCP dial failed for {host}:{port}: {e}"))
    })?;
    apply_socket_policy(&socket);
    Ok(socket)
}

/// TCP_NODELAY + SO_KEEPALIVE on an established stream (SSH or pre-TLS).
pub(crate) fn apply_socket_policy(socket: &TcpStream) {
    if let Err(e) = socket.set_nodelay(true) {
        tracing::debug!(err = %e, "set_nodelay on SSH socket failed");
    }
    if TCP_KEEPALIVE_ENABLED {
        let sock_ref = socket2::SockRef::from(socket);
        if let Err(e) = sock_ref.set_keepalive(true) {
            tracing::debug!(err = %e, "set_keepalive on SSH socket failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_ssh_client_config_policy_surface() {
        let cfg = build_ssh_client_config(Duration::from_secs(5));
        assert_eq!(cfg.preferred.compression.as_ref(), &[russh::compression::NONE]);
        assert!(cfg.nodelay);
        assert_eq!(cfg.keepalive_max, SSH_KEEPALIVE_MAX);
        assert!(cfg.inactivity_timeout.is_some());
        assert!(cfg.keepalive_interval.is_some());
        assert_eq!(cfg.window_size, SSH_WINDOW_SIZE);
        assert_eq!(cfg.maximum_packet_size, SSH_MAX_PACKET_SIZE);
        // client_id must not advertise the russh crate version string.
        let id = format!("{:?}", cfg.client_id);
        assert!(
            id.contains("ssh-cli") || id.contains(SSH_CLIENT_ID),
            "client_id should be product-generic: {id}"
        );
        assert!(
            !id.contains("russh_"),
            "client_id must not embed russh version: {id}"
        );
    }
}
