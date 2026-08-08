// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! Remote-to-local forwarding for `tunnel --reverse` (G-TUN-R01).
//!
//! The direction is inverted relative to every other mode: the **server** listens
//! and this process dials. There is no local `TcpListener` here, so the "bound"
//! milestone that the deadline wrapper waits on is the server's acceptance of
//! `tcpip-forward`, not a local bind.
//!
//! # Why the allocated port must be read back
//!
//! Asking for port `0` lets the server choose, and it answers with the port it
//! actually bound. Echoing the requested `0` into the event would leave an agent
//! with nothing to connect to, which is the same class of bug `GAP-SSH-TUN-003`
//! fixed for the local ephemeral bind.

use super::TunnelStats;
use crate::errors::SshCliError;
use crate::output;
use crate::ssh::client::SshClientTrait;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Parameters for one reverse forward.
pub struct ReverseServe {
    /// Registry name of the host, echoed into events.
    pub vps_name: String,
    /// Address the **server** is asked to bind.
    pub remote_bind: String,
    /// Port the server is asked to bind (`0` = let the server allocate).
    pub remote_port: u16,
    /// Local host that forwarded connections are delivered to.
    pub local_host: String,
    /// Local port that forwarded connections are delivered to.
    pub local_port: u16,
    /// Deadline echoed into the listening event.
    pub timeout_ms: u64,
    /// Agent-first JSON output.
    pub json: bool,
}

/// Requests a remote listener and pumps every forwarded channel to the local target.
///
/// # Errors
/// [`SshCliError::ChannelFailed`] when the server refuses `tcpip-forward` — most
/// often `AllowTcpForwarding no`, which is a server policy this CLI cannot work
/// around and must therefore report rather than retry.
pub async fn serve(
    params: ReverseServe,
    client: Box<dyn SshClientTrait>,
    bound_flag: Option<Arc<AtomicBool>>,
    stats: Option<Arc<TunnelStats>>,
) -> Result<()> {
    let stats = stats.unwrap_or_default();
    let client: Arc<dyn SshClientTrait> = Arc::from(client);
    let ReverseServe {
        vps_name,
        remote_bind,
        remote_port,
        local_host,
        local_port,
        timeout_ms,
        json,
    } = params;

    let allocated = client
        .request_remote_forward(&remote_bind, remote_port)
        .await?;

    stats
        .effective_port
        .store(u32::from(allocated), Ordering::Release);
    if let Some(flag) = bound_flag.as_ref() {
        // The remote listener is the milestone here; publishing it is what makes
        // the deadline a *success* ending rather than a connect timeout.
        flag.store(true, Ordering::Release);
    }

    tracing::info!(
        vps = %vps_name,
        remote_bind = %remote_bind,
        allocated,
        requested = remote_port,
        local_host = %local_host,
        local_port,
        "remote listener established"
    );

    if json {
        output::print_tunnel_listening_json(
            &vps_name,
            allocated,
            &local_host,
            local_port,
            timeout_ms,
            &remote_bind,
            "reverse",
        )?;
    } else {
        let banner = crate::i18n::t(crate::i18n::Message::TunnelReverseListening {
            remote_bind: remote_bind.clone(),
            remote_port: allocated,
            local_host: local_host.clone(),
            local_port,
            vps: vps_name.clone(),
            timeout_ms,
        });
        tracing::info!("{banner}");
        output::print_human_banner(&banner);
    }

    let mut forwards = tokio::task::JoinSet::new();
    let forward_limit = crate::concurrency::effective_limit();
    let forward_sem = crate::concurrency::semaphore(forward_limit);

    loop {
        if crate::signals::should_stop() {
            tracing::info!("reverse tunnel cancelled by signal");
            stats.stopped_by_signal.store(true, Ordering::Release);
            break;
        }

        tokio::select! {
            incoming = client.accept_forwarded_channel() => {
                let Some(channel) = incoming else {
                    // The session ended; no further channels can arrive. Treating
                    // this as an accept error rather than a silent exit is what lets
                    // `tunnel_closed` distinguish it from the deadline.
                    tracing::warn!("SSH session closed; no further forwarded channels");
                    stats.stopped_by_accept_error.store(true, Ordering::Release);
                    break;
                };
                let permit = match forward_sem.clone().try_acquire_owned() {
                    Ok(p) => p,
                    Err(_) => {
                        let prior = stats.capacity_waits.fetch_add(1, Ordering::Relaxed);
                        if prior == 0 {
                            tracing::warn!(
                                max_concurrency = forward_limit,
                                "reverse forward concurrency saturated; new channels are queuing"
                            );
                        }
                        crate::concurrency::acquire_owned(&forward_sem).await
                    }
                };
                let host = local_host.clone();
                let served = Arc::clone(&stats);
                forwards.spawn(async move {
                    let _permit = permit;
                    served.forwards_served.fetch_add(1, Ordering::Relaxed);
                    if let Err(e) = deliver(channel, &host, local_port).await {
                        tracing::warn!(err = %e, "reverse forward failed");
                    }
                });
            }
            Some(joined) = forwards.join_next() => {
                if let Err(e) = joined {
                    tracing::debug!(err = %e, "reverse forward task ended with join error");
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(
                crate::constants::TUNNEL_SIGNAL_POLL_INTERVAL_MS,
            )) => {}
        }
    }

    // Best-effort teardown: a server that already dropped the session cannot
    // answer, and failing the whole run over an unacknowledged cancel would turn a
    // successful tunnel into an error at the last instant.
    if let Err(e) = client.cancel_remote_forward(&remote_bind, allocated).await {
        tracing::debug!(err = %e, "cancel-tcpip-forward failed during teardown");
    }
    super::drain_forwards(&mut forwards).await;
    let _ = client.disconnect().await;
    Ok(())
}

/// Dials the local target and pumps one forwarded channel into it.
async fn deliver(
    channel: Box<dyn crate::ssh::client::TunnelChannel>,
    local_host: &str,
    local_port: u16,
) -> Result<()> {
    let target = format!("{local_host}:{local_port}");
    let socket = tokio::net::TcpStream::connect(&target)
        .await
        .map_err(SshCliError::Io)?;
    if let Err(e) = socket.set_nodelay(true) {
        tracing::debug!(err = %e, "reverse forward set_nodelay failed");
    }
    super::pump(socket, channel, local_host, local_port).await
}
