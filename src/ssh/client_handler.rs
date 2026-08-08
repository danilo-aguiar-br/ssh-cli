// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SSH-01 / G-SSH-09 / G-SSH-14: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! russh [`client::Handler`] with TOFU known_hosts and typed host-key errors.
//!
//! The handler is moved into `connect_stream`; product errors that cannot be
//! expressed as `russh::Error` are stashed in [`HostKeyOutcome`] for the connect
//! caller to recover after a failed handshake (G-SSH-01).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::constants::AUTH_BANNER_MAX_CHARS;
use crate::errors::SshCliError;
use crate::ssh::connection::ConnectionConfig;

/// Shared slot for host-key / TOFU failures observed inside the russh handler.
///
/// After `connect_stream` returns `Err`, the connect path must `take()` this
/// value so agents receive typed [`SshCliError::HostKeyChanged`] (exit EX_NOPERM)
/// instead of a generic handshake failure.
pub type HostKeyOutcome = Arc<Mutex<Option<SshCliError>>>;

/// Create an empty shared outcome slot.
#[must_use]
pub fn new_host_key_outcome() -> HostKeyOutcome {
    Arc::new(Mutex::new(None))
}

/// Store a product error for the connect caller (best-effort if lock is poisoned).
pub fn stash_host_key_error(outcome: &HostKeyOutcome, err: SshCliError) {
    if let Ok(mut g) = outcome.lock() {
        *g = Some(err);
    }
}

/// Take a stashed host-key error if present.
#[must_use]
pub fn take_host_key_error(outcome: &HostKeyOutcome) -> Option<SshCliError> {
    outcome.lock().ok().and_then(|mut g| g.take())
}

/// Sink for channels the **server** opens against an active reverse forward.
///
/// G-TUN-R01: `forwarded-tcpip` channels are inbound — russh delivers them to the
/// handler, which lives inside the session task and cannot be reached from the
/// tunnel loop. This bounded queue is the bridge between the two.
pub type ForwardedSink = tokio::sync::mpsc::Sender<russh::Channel<russh::client::Msg>>;

/// Receiving half of [`ForwardedSink`], held by the client.
pub type ForwardedSource = tokio::sync::mpsc::Receiver<russh::Channel<russh::client::Msg>>;

/// russh handler with TOFU known_hosts (or test-only always-trust when path is absent).
pub struct ClientHandler {
    host: String,
    port: u16,
    known_hosts_path: Option<PathBuf>,
    replace_host_key: bool,
    outcome: HostKeyOutcome,
    forwarded: ForwardedSink,
}

impl ClientHandler {
    /// Build a handler from connection config, outcome slot and inbound-channel sink.
    #[must_use]
    pub fn new(cfg: &ConnectionConfig, outcome: HostKeyOutcome, forwarded: ForwardedSink) -> Self {
        Self {
            host: cfg.host.as_str().to_owned(),
            port: cfg.port.get(),
            known_hosts_path: cfg.known_hosts_path.clone(),
            replace_host_key: cfg.replace_host_key,
            outcome,
            forwarded,
        }
    }
}

impl russh::client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        let fingerprint = format!("{}", server_key.fingerprint(russh::keys::HashAlg::Sha256));

        // `take`: host-key check runs once per connection; move path, no clone.
        let Some(path) = self.known_hosts_path.take() else {
            // G-SSH-09: always-trust only in unit tests — product builds reject.
            #[cfg(test)]
            {
                tracing::warn!("known_hosts missing: accepting host key (test mode)");
                return Ok(true);
            }
            #[cfg(not(test))]
            {
                stash_host_key_error(
                    &self.outcome,
                    SshCliError::InvalidArgument(
                        "known_hosts_path is required for host-key verification".into(),
                    ),
                );
                tracing::error!("known_hosts path missing; rejecting host key (fail-closed)");
                return Ok(false);
            }
        };

        // G-NET: known_hosts load/save is sync FS + flock — keep it off the
        // async worker so multi-host fan-out does not stall Tokio threads.
        let host = self.host.clone();
        let port = self.port;
        let replace = self.replace_host_key;
        let outcome = tokio::task::spawn_blocking(move || {
            let mut kh = crate::ssh::known_hosts::KnownHosts::load(path)?;
            crate::ssh::known_hosts::verify_tofu(&mut kh, &host, port, &fingerprint, replace)
        })
        .await;

        match outcome {
            Ok(Ok(true)) => Ok(true),
            Ok(Ok(false)) => Ok(false),
            Ok(Err(e)) => {
                // G-SSH-01: preserve typed HostKeyChanged for the connect caller.
                stash_host_key_error(&self.outcome, e);
                tracing::error!("host key rejected");
                Ok(false)
            }
            Err(e) => {
                stash_host_key_error(
                    &self.outcome,
                    SshCliError::ConnectionFailed(format!("known_hosts task failed: {e}")),
                );
                tracing::error!(err = %e, "known_hosts task failed");
                Ok(false)
            }
        }
    }

    async fn auth_banner(
        &mut self,
        banner: &str,
        _session: &mut russh::client::Session,
    ) -> Result<(), Self::Error> {
        // G-SSH-14: surface pre-auth banners for diagnostics (truncate; no secrets expected).
        //
        // A1 (remote pre-auth DoS): `banner` is server-controlled and arrives *before*
        // authentication, so it is hostile input by definition. Slicing it by byte index
        // panics whenever the cut lands inside a multi-byte character, and the release
        // profile sets `panic = "abort"` — no unwind, the whole process dies, taking a
        // multi-host fan-out down with it. Truncate on character boundaries instead.
        let (truncated, was_truncated) =
            crate::ssh::session_io::truncate_utf8(banner, AUTH_BANNER_MAX_CHARS);
        if was_truncated {
            tracing::info!(banner = %truncated, truncated = true, "SSH auth banner");
        } else {
            tracing::info!(banner = %truncated, "SSH auth banner");
        }
        Ok(())
    }

    /// Accepts a `forwarded-tcpip` channel opened by the server (G-TUN-R01).
    ///
    /// Only channels that fit in the bounded queue are accepted. A full queue means
    /// nobody is draining the reverse forward, so accepting would buffer indefinitely
    /// on the peer's schedule; rejecting applies backpressure over the wire instead.
    ///
    /// The default russh implementation accepts unconditionally and drops the
    /// channel, which for a client that never requested a forward means silently
    /// letting a server open channels into the process.
    async fn server_channel_open_forwarded_tcpip(
        &mut self,
        channel: russh::Channel<russh::client::Msg>,
        connected_address: &str,
        connected_port: u32,
        originator_address: &str,
        originator_port: u32,
        reply: russh::client::ChannelOpenHandle,
        _session: &mut russh::client::Session,
    ) -> Result<(), Self::Error> {
        tracing::debug!(
            connected_address,
            connected_port,
            originator_address,
            originator_port,
            "server opened forwarded-tcpip channel"
        );
        match self.forwarded.try_reserve() {
            Ok(permit) => {
                reply.accept().await;
                permit.send(channel);
                Ok(())
            }
            Err(e) => {
                tracing::warn!(
                    err = %e,
                    connected_address,
                    connected_port,
                    "rejecting forwarded-tcpip channel: no active reverse forward or queue full"
                );
                reply
                    .reject(russh::ChannelOpenFailure::AdministrativelyProhibited)
                    .await;
                Ok(())
            }
        }
    }

    /// Rejects `forwarded-streamlocal` channels.
    ///
    /// This CLI only ever requests *outbound* streamlocal channels
    /// (`--remote-socket`), never a remote Unix-socket listener, so an inbound one
    /// was not asked for. Accepting and dropping it — the russh default — would make
    /// the refusal invisible to both sides.
    async fn server_channel_open_forwarded_streamlocal(
        &mut self,
        _channel: russh::Channel<russh::client::Msg>,
        socket_path: &str,
        reply: russh::client::ChannelOpenHandle,
        _session: &mut russh::client::Session,
    ) -> Result<(), Self::Error> {
        tracing::warn!(
            socket_path,
            "rejecting unsolicited forwarded-streamlocal channel"
        );
        reply
            .reject(russh::ChannelOpenFailure::AdministrativelyProhibited)
            .await;
        Ok(())
    }
}
