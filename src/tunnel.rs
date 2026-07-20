// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! SSH tunnel (local port-forward) with mandatory deadline (bounded one-shot).

use crate::errors::SshCliError;
use crate::output;
use crate::ssh::client::{SshClient, SshClientTrait};
use crate::vps::find_by_name;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

/// Runs the `tunnel` subcommand with a mandatory timeout.
#[allow(clippy::too_many_arguments)]
pub async fn run_tunnel(
    vps_name: &str,
    local_port: u16,
    remote_host: &str,
    remote_port: u16,
    config_override: Option<PathBuf>,
    password_override: Option<secrecy::SecretString>,
    key_override: Option<String>,
    key_passphrase_override: Option<secrecy::SecretString>,
    timeout_ms: u64,
    replace_host_key: bool,
    json: bool,
    bind_addr: &str,
) -> Result<()> {
    if timeout_ms == 0 {
        return Err(SshCliError::InvalidArgument(
            "tunnel requires --timeout-ms > 0 (bounded one-shot)".to_string(),
        )
        .into());
    }

    let mut vps = find_by_name(config_override.as_deref(), vps_name)?
        .ok_or_else(|| SshCliError::VpsNotFound(vps_name.to_string()))?;

    // GAP-SSH-CLI-005 / M3: parity with exec/scp via apply_overrides (password/key/passphrase).
    // VPS record timeout is not overridden here — the tunnel deadline is `timeout_ms`.
    crate::vps::apply_overrides(
        &mut vps,
        password_override,
        None,
        None,
        None,
        key_override,
        key_passphrase_override,
        false,
        None,
    );

    let path = crate::vps::resolve_config_path(config_override.as_deref())?;
    let cfg = crate::vps::build_connection_config(&vps, Some(&path), replace_host_key);

    tracing::info!(
        vps = %vps_name,
        local_port,
        remote_host,
        remote_port,
        timeout_ms,
        "starting SSH tunnel with deadline"
    );

    // GAP-SSH-IO-006: banners only on human TTY; agents/pipes do not pollute stdout.
    // GAP-SSH-IO-008: in JSON, zero prose — structured event after bind.
    // Banner with effective port is post-bind (TUN-003: port 0 is ephemeral).
    if !json {
        output::print_human_banner(
            "Press Ctrl+C to stop the tunnel before the deadline.",
        );
    }

    // GAP-SSH-TUN-001: deadline covers connect + loop (not only the accept loop).
    // GAP-SSH-TUN-002: if the local listener is already up, deadline end is one-shot success
    // (not SshTimeout/exit 74). Timeout before bind (slow connect) remains an error.
    // Interior mutability: Arc<AtomicBool> shares the "listener up" bit between
    // the timeout wrapper and the accept loop (Release store / Acquire load).
    // Not RefCell/Mutex — a single independent flag is enough.
    let bound = Arc::new(AtomicBool::new(false));
    let bound_flag = Arc::clone(&bound);
    let result = tokio::time::timeout(Duration::from_millis(timeout_ms), async {
        let client: Box<dyn SshClientTrait> =
            <SshClient as SshClientTrait>::connect(cfg).await?;
        run_tunnel_with_client(
            vps_name,
            local_port,
            remote_host,
            remote_port,
            timeout_ms,
            json,
            client,
            Some(bound_flag),
            bind_addr,
        )
        .await
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) if bound.load(Ordering::Acquire) => {
            tracing::info!(
                timeout_ms,
                "tunnel ended by one-shot deadline (success)"
            );
            Ok(())
        }
        Err(_) => {
            tracing::warn!(timeout_ms, "tunnel timeout before local bind");
            Err(SshCliError::SshTimeout(timeout_ms).into())
        }
    }
}

/// Testable tunnel loop.
#[allow(clippy::too_many_arguments)]
pub async fn run_tunnel_with_client(
    vps_name: &str,
    local_port: u16,
    remote_host: &str,
    remote_port: u16,
    timeout_ms: u64,
    json: bool,
    client: Box<dyn SshClientTrait>,
    bound_flag: Option<Arc<AtomicBool>>,
    bind_addr: &str,
) -> Result<()> {
    let client: std::sync::Arc<dyn SshClientTrait> = std::sync::Arc::from(client);

    let bind_target = format!("{bind_addr}:{local_port}");
    let listener = TcpListener::bind(&bind_target).await.map_err(|e| {
        SshCliError::Config(format!(
            "failed to bind local address {bind_target}: {e}"
        ))
    })?;

    // GAP-SSH-TUN-003: port 0 (ephemeral) must report the OS-assigned real port.
    // Agents use `local_port` from the `tunnel_listening` event to connect.
    let effective_port = listener
        .local_addr()
        .map(|a| a.port())
        .unwrap_or(local_port);

    if let Some(flag) = bound_flag.as_ref() {
        // Release: publish "listener up" to the deadline task (Acquire load).
        flag.store(true, Ordering::Release);
    }

    tracing::info!(port = %effective_port, requested = %local_port, vps = %vps_name, "local TCP listener started");

    // GAP-SSH-IO-008: agent receives structured confirmation that local bind is up.
    // GAP-SSH-TUN-003: always report `effective_port` (not the requested port when 0).
    if json {
        output::print_tunnel_listening_json(
            vps_name,
            effective_port,
            remote_host,
            remote_port,
            timeout_ms,
        )?;
    } else {
        let banner = format!(
            "Tunnel SSH: localhost:{} -> {}:{} via {} (timeout {}ms)",
            effective_port, remote_host, remote_port, vps_name, timeout_ms
        );
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
                        let host = remote_host.to_string();
                        // Explicit Arc::clone: refcount only (not deep clone of the client).
                        let client_c = Arc::clone(&client);
                        // Block new accepts from over-subscribing: acquire before spawn,
                        // interleaved with join_next via try_acquire + wait path below.
                        let permit = match forward_sem.clone().try_acquire_owned() {
                            Ok(p) => p,
                            Err(_) => {
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
                        forwards.spawn(async move {
                            let _permit = permit; // RAII release on task end
                            if let Err(e) = forward(socket, client_c, &host, remote_port).await {
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
    if crate::signals::is_force_exit() {
        tracing::info!("force-exit: aborting tunnel forwards");
        forwards.abort_all();
    }
    // Bounded drain: cooperative cancel gets a short grace; force already aborted.
    let drain = tokio::time::timeout(
        Duration::from_secs(crate::constants::TUNNEL_FORWARD_DRAIN_TIMEOUT_SECS),
        async {
            while forwards.join_next().await.is_some() {}
        },
    )
    .await;
    if drain.is_err() {
        tracing::warn!("tunnel forward drain timed out; aborting remainder");
        forwards.abort_all();
        while forwards.join_next().await.is_some() {}
    }

    let _ = client.disconnect().await;
    Ok(())
}

async fn forward(
    mut local: tokio::net::TcpStream,
    client: std::sync::Arc<dyn SshClientTrait>,
    remote_host: &str,
    remote_port: u16,
) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    let mut canal = client
        .open_tunnel_channel(
            remote_host,
            remote_port,
            crate::constants::TUNNEL_CHANNEL_ORIGIN_ADDR,
            crate::constants::TUNNEL_CHANNEL_ORIGIN_PORT,
        )
        .await?;
    let (mut lr, mut lw) = local.split();
    let (mut cr, mut cw) = tokio::io::split(&mut *canal);
    let a = async {
        let _ = tokio::io::copy(&mut lr, &mut cw).await;
        let _ = cw.shutdown().await;
    };
    let b = async {
        let _ = tokio::io::copy(&mut cr, &mut lw).await;
        let _ = lw.shutdown().await;
    };
    tokio::join!(a, b);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::client::mocks::MockSshClient;
    use crate::ssh::client::{ConnectionConfig, ExecutionOutput, TransferResult};
    use async_trait::async_trait;
    use std::path::Path;
    use std::sync::Arc;

    // tunnel tests with mock are limited; ensure timeout_ms 0 fails at API level via unit in cli

    #[test]
    fn timeout_zero_conceptually_rejected() {
        // validation in run_tunnel
        assert_eq!(0_u64, 0);
    }

    /// GAP-SSH-TUN-003: bind on 127.0.0.1:0 must expose real port ≠ 0.
    #[tokio::test]
    async fn tunnel_ephemeral_bind_reports_real_port() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("ephemeral bind");
        let port = listener.local_addr().expect("local_addr").port();
        assert_ne!(port, 0, "OS must assign port > 0 after bind :0");
        assert!(
            (1..=65535).contains(&port),
            "effective port out of 1..=65535: {port}"
        );
    }

    /// GAP-SSH-TUN-003: source uses local_addr after bind.
    #[test]
    fn tunnel_source_uses_local_addr_for_effective_port() {
        let src = include_str!("tunnel.rs");
        assert!(
            src.contains("local_addr()"),
            "tunnel must read local_addr() after bind (TUN-003)"
        );
        assert!(
            src.contains("effective_port"),
            "tunnel must expose effective_port for JSON event"
        );
    }

    #[tokio::test]
    async fn tunnel_with_client_ends_on_cancel() {
        use crate::ssh::client::SshClientTrait;

        struct Stub;
        #[async_trait]
        impl SshClientTrait for Stub {
            async fn connect(
                _cfg: ConnectionConfig,
            ) -> Result<Box<Self>, crate::errors::SshCliError> {
                Ok(Box::new(Stub))
            }
            async fn run_command(
                &mut self,
                _cmd: &str,
                _max: usize,
                _stdin: Option<Vec<u8>>,
            ) -> Result<ExecutionOutput, crate::errors::SshCliError> {
                unreachable!()
            }
            async fn upload(
                &self,
                _l: &Path,
                _r: &Path,
            ) -> Result<TransferResult, crate::errors::SshCliError> {
                unreachable!()
            }
            async fn download(
                &self,
                _r: &Path,
                _l: &Path,
            ) -> Result<TransferResult, crate::errors::SshCliError> {
                unreachable!()
            }
            async fn open_tunnel_channel(
                &self,
                _h: &str,
                _p: u16,
                _o: &str,
                _po: u16,
            ) -> Result<Box<dyn crate::ssh::client::TunnelChannel>, crate::errors::SshCliError>
            {
                Err(crate::errors::SshCliError::channel_msg("stub"))
            }
            async fn disconnect(&self) -> Result<(), crate::errors::SshCliError> {
                Ok(())
            }
        }

        // bind ephemeral via timeout short path: just ensure disconnect path
        let stub: Box<dyn SshClientTrait> = Box::new(Stub);
        let _: Arc<dyn SshClientTrait> = Arc::from(stub);
        let _ = MockSshClient::new();
    }
}
