// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! Local-listener tunnel modes: plain forward, SOCKS5 proxy, remote Unix socket.
//!
//! All three bind a local TCP listener and hand every accepted connection to an
//! SSH channel, so they share one accept loop. Only the *destination* differs,
//! which is what [`ForwardKind`] selects. Copying the loop three times would have
//! meant maintaining the signal handling, the admission gate, the drain and the
//! saturation accounting in triplicate — and those are precisely the parts that
//! are easy to get subtly wrong in one copy and not the others.

use super::TunnelStats;
use crate::errors::SshCliError;
use crate::output;
use crate::ssh::client::SshClientTrait;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

/// What an accepted local connection is forwarded to.
#[derive(Debug, Clone)]
pub enum ForwardKind {
    /// Fixed `host:port` on the remote side (`direct-tcpip`).
    Tcp {
        /// Remote host, resolved by the SSH server.
        host: String,
        /// Remote port.
        port: u16,
    },
    /// SOCKS5 proxy: the client names a target per connection (G-TUN-R02).
    Socks5,
    /// Remote Unix domain socket (`direct-streamlocal`, G-TUN-R03).
    StreamLocal {
        /// Absolute path of the socket on the remote host.
        socket_path: String,
    },
}

impl ForwardKind {
    /// Wire label used in the `tunnel_listening` / `tunnel_closed` events.
    #[must_use]
    pub fn mode_label(&self) -> &'static str {
        match self {
            Self::Tcp { .. } => "local",
            Self::Socks5 => "socks5",
            Self::StreamLocal { .. } => "streamlocal",
        }
    }

    /// Host reported in the listening event (`*` when chosen per connection).
    #[must_use]
    pub fn event_host(&self) -> String {
        match self {
            Self::Tcp { host, .. } => host.clone(),
            // A SOCKS5 proxy has no single destination; reporting one would be a
            // guess an agent could act on.
            Self::Socks5 => "*".to_string(),
            Self::StreamLocal { socket_path } => socket_path.clone(),
        }
    }

    /// Port reported in the listening event (`0` when not applicable).
    #[must_use]
    pub fn event_port(&self) -> u16 {
        match self {
            Self::Tcp { port, .. } => *port,
            Self::Socks5 | Self::StreamLocal { .. } => 0,
        }
    }
}

/// Everything the accept loop needs, grouped so the signature stays readable.
pub struct LocalServe {
    /// Registry name of the host, echoed into events.
    pub vps_name: String,
    /// Requested local port (`0` asks the OS to allocate).
    pub local_port: u16,
    /// Local bind address.
    pub bind_addr: String,
    /// Deadline echoed into the listening event.
    pub timeout_ms: u64,
    /// Agent-first JSON output.
    pub json: bool,
    /// Destination selector.
    pub kind: ForwardKind,
}

/// Binds locally and forwards every accepted connection until signal or drop.
///
/// # Errors
/// Bind failures, classified by [`std::io::ErrorKind`] so a busy port (retryable)
/// stays distinguishable from a malformed address (never retryable).
pub async fn serve(
    params: LocalServe,
    client: Box<dyn SshClientTrait>,
    bound_flag: Option<Arc<AtomicBool>>,
    stats: Option<Arc<TunnelStats>>,
) -> Result<()> {
    let stats = stats.unwrap_or_default();
    let client: Arc<dyn SshClientTrait> = Arc::from(client);
    let LocalServe {
        vps_name,
        local_port,
        bind_addr,
        timeout_ms,
        json,
        kind,
    } = params;

    let bind_target = format!("{bind_addr}:{local_port}");
    // G-TUN-R09: every bind failure used to collapse into `Config` (exit 65, classified
    // permanent), so an agent got "data error" for three situations needing opposite
    // responses. Inspecting `ErrorKind` keeps the distinction: a busy port is worth
    // retrying on another port, a malformed address never is, and formatting the
    // `io::Error` into a String would have destroyed the very information that decides.
    let listener = TcpListener::bind(&bind_target).await.map_err(|e| {
        let kind = e.kind();
        match kind {
            std::io::ErrorKind::AddrNotAvailable | std::io::ErrorKind::InvalidInput => {
                SshCliError::InvalidArgument(format!("cannot bind {bind_target}: {e}"))
            }
            _ => SshCliError::Io(e),
        }
    })?;

    // GAP-SSH-TUN-003: port 0 (ephemeral) must report the OS-assigned real port.
    // Agents use `local_port` from the `tunnel_listening` event to connect.
    let effective_port = listener
        .local_addr()
        .map(|a| a.port())
        .unwrap_or(local_port);

    // Published before the bound flag so a wrapper that observes `bound` can already
    // read the real port.
    stats
        .effective_port
        .store(u32::from(effective_port), Ordering::Release);
    if let Some(flag) = bound_flag.as_ref() {
        // Release: publish "listener up" to the deadline task (Acquire load).
        flag.store(true, Ordering::Release);
    }

    tracing::info!(
        port = %effective_port,
        requested = %local_port,
        vps = %vps_name,
        mode = kind.mode_label(),
        "local TCP listener started"
    );

    // GAP-SSH-IO-008: agent receives structured confirmation that local bind is up.
    // GAP-SSH-TUN-003: always report `effective_port` (not the requested port when 0).
    if json {
        output::print_tunnel_listening_json(
            &vps_name,
            effective_port,
            &kind.event_host(),
            kind.event_port(),
            timeout_ms,
            &bind_addr,
            kind.mode_label(),
        )?;
    } else {
        // E4: the banner hard-coded `localhost:` regardless of `--bind`, so a run
        // bound to 0.0.0.0 still told the operator it was listening on loopback —
        // the exact opposite of the security-relevant fact.
        // Through `i18n::t` rather than an inline `format!`: these are human-facing
        // strings, and the project keeps every one of them in a single exhaustive
        // match so a new language cannot silently miss one.
        let banner = crate::i18n::t(match &kind {
            ForwardKind::Tcp { host, port } => crate::i18n::Message::TunnelLocalListening {
                bind: bind_addr.clone(),
                port: effective_port,
                remote_host: host.clone(),
                remote_port: *port,
                vps: vps_name.clone(),
                timeout_ms,
            },
            ForwardKind::Socks5 => crate::i18n::Message::TunnelSocks5Listening {
                bind: bind_addr.clone(),
                port: effective_port,
                vps: vps_name.clone(),
                timeout_ms,
            },
            ForwardKind::StreamLocal { socket_path } => {
                crate::i18n::Message::TunnelStreamLocalListening {
                    bind: bind_addr.clone(),
                    port: effective_port,
                    socket_path: socket_path.clone(),
                    vps: vps_name.clone(),
                    timeout_ms,
                }
            }
        });
        tracing::info!("{banner}");
        output::print_human_banner(&banner);
    }

    // Track forwards so shutdown can drain/abort instead of detaching `tokio::spawn`.
    // Admission gate: Semaphore (Rules Rust — never unbounded spawn on accept).
    // Workload: I/O-bound bidirectional copy; saturates FDs + SSH channels.
    let mut forwards = tokio::task::JoinSet::new();
    let forward_limit = crate::concurrency::effective_limit();
    let forward_sem = crate::concurrency::semaphore(forward_limit);
    tracing::debug!(
        max_concurrency = forward_limit,
        "tunnel forward admission gate ready"
    );

    loop {
        if crate::signals::should_stop() {
            tracing::info!(
                force = crate::signals::is_force_exit(),
                "tunnel cancelled by signal"
            );
            stats.stopped_by_signal.store(true, Ordering::Release);
            break;
        }

        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((socket, addr)) => {
                        tracing::debug!(address = %addr, "new local connection");
                        // G-NET: low-latency local forward (Nagle off on accepted peer).
                        if let Err(e) = socket.set_nodelay(true) {
                            tracing::debug!(err = %e, %addr, "tunnel set_nodelay failed");
                        }
                        let kind_c = kind.clone();
                        // Explicit Arc::clone: refcount only (not deep clone of the client).
                        let client_c = Arc::clone(&client);
                        // Block new accepts from over-subscribing: acquire before spawn,
                        // interleaved with join_next via try_acquire + wait path below.
                        let permit = match forward_sem.clone().try_acquire_owned() {
                            Ok(p) => p,
                            Err(_) => {
                                // G-TUN-R12: record and announce saturation. Previously the
                                // wait was completely silent, so the only symptom was rising
                                // latency with no stated cause — and the operator had no way
                                // to know the bottleneck was their own --max-concurrency.
                                let prior = stats
                                    .capacity_waits
                                    .fetch_add(1, Ordering::Relaxed);
                                if prior == 0 {
                                    tracing::warn!(
                                        max_concurrency = forward_limit,
                                        "tunnel forward concurrency saturated; new connections are queuing"
                                    );
                                }
                                // At capacity: wait for a permit or a completed forward.
                                tokio::select! {
                                    p = crate::concurrency::acquire_owned(&forward_sem) => p,
                                    Some(joined) = forwards.join_next() => {
                                        if let Err(e) = joined {
                                            tracing::debug!(err = %e, "tunnel forward task ended with join error");
                                        }
                                        crate::concurrency::acquire_owned(&forward_sem).await
                                    }
                                }
                            }
                        };
                        let served = Arc::clone(&stats);
                        forwards.spawn(async move {
                            let _permit = permit; // RAII release on task end
                            served.forwards_served.fetch_add(1, Ordering::Relaxed);
                            if let Err(e) = handle_connection(socket, client_c, &kind_c).await {
                                tracing::warn!(err = %e, "tunnel forwarding failed");
                            }
                        });
                    }
                    Err(e) => {
                        // G-NET: do not tear down the accept loop on transient errors.
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::Interrupted
                                | std::io::ErrorKind::WouldBlock
                                | std::io::ErrorKind::ConnectionAborted
                                | std::io::ErrorKind::ConnectionReset
                        ) {
                            tracing::debug!(err = %e, "transient accept error; continuing");
                            continue;
                        }
                        tracing::error!(err = %e, "accept failed (fatal)");
                        // G-TUN-R07: this ends the loop while `bound` is already true,
                        // so the deadline wrapper returns Ok and the process exits 0.
                        // Recording the reason is what lets an agent tell a tunnel that
                        // died three seconds in from one that served its full lifetime.
                        stats.stopped_by_accept_error.store(true, Ordering::Release);
                        break;
                    }
                }
            }
            // Reap completed forwards so JoinSet does not grow unbounded.
            Some(joined) = forwards.join_next() => {
                if let Err(e) = joined {
                    tracing::debug!(err = %e, "tunnel forward task ended with join error");
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(
                crate::constants::TUNNEL_SIGNAL_POLL_INTERVAL_MS,
            )) => {
                // signal polling interval
            }
        }
    }

    // Stop accepting new local connections, then drain or abort active forwards.
    drop(listener);
    super::drain_forwards(&mut forwards).await;
    let _ = client.disconnect().await;

    // `tunnel_closed` is emitted by `run_tunnel`, not here: on the deadline path this
    // future is cancelled mid-poll and never reaches this line.
    Ok(())
}

/// Routes one accepted socket to the destination its mode dictates.
async fn handle_connection(
    socket: tokio::net::TcpStream,
    client: Arc<dyn SshClientTrait>,
    kind: &ForwardKind,
) -> Result<()> {
    match kind {
        ForwardKind::Tcp { host, port } => {
            let channel = client
                .open_tunnel_channel(
                    host,
                    *port,
                    crate::constants::TUNNEL_CHANNEL_ORIGIN_ADDR,
                    crate::constants::TUNNEL_CHANNEL_ORIGIN_PORT,
                )
                .await?;
            super::pump(socket, channel, host, *port).await
        }
        ForwardKind::StreamLocal { socket_path } => {
            let channel = client.open_streamlocal_channel(socket_path).await?;
            super::pump(socket, channel, socket_path, 0).await
        }
        ForwardKind::Socks5 => handle_socks5(socket, client).await,
    }
}

/// Completes a SOCKS5 handshake, then pumps the negotiated channel.
async fn handle_socks5(
    mut socket: tokio::net::TcpStream,
    client: Arc<dyn SshClientTrait>,
) -> Result<()> {
    use super::socks;

    let target = match socks::handshake(&mut socket).await {
        Ok(Ok(target)) => target,
        Ok(Err(refusal)) => {
            // The reply frame was already written by `handshake`; nothing further
            // is owed to the client beyond a clean close.
            tracing::debug!(reason = %refusal.reason, "SOCKS5 request refused");
            return Ok(());
        }
        Err(e) => {
            // Malformed input: the peer is not speaking SOCKS5, so a protocol reply
            // would be meaningless. Drop the connection and say why in the log.
            tracing::warn!(err = %e, "SOCKS5 handshake failed");
            return Ok(());
        }
    };

    let channel = match client
        .open_tunnel_channel(
            &target.host,
            target.port,
            crate::constants::TUNNEL_CHANNEL_ORIGIN_ADDR,
            crate::constants::TUNNEL_CHANNEL_ORIGIN_PORT,
        )
        .await
    {
        Ok(channel) => channel,
        Err(e) => {
            // The client is waiting for a reply frame; closing here would leave it
            // guessing between "refused" and "proxy died".
            tracing::warn!(
                err = %e, host = %target.host, port = target.port,
                "SOCKS5 CONNECT could not open an SSH channel"
            );
            socks::write_reply(&mut socket, socks::REP_HOST_UNREACHABLE).await?;
            return Ok(());
        }
    };

    socks::write_reply(&mut socket, socks::REP_SUCCEEDED).await?;
    super::pump(socket, channel, &target.host, target.port).await
}
