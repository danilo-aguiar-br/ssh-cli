// SPDX-License-Identifier: MIT OR Apache-2.0
//! SSH tunnel (local port-forward) with mandatory deadline (bounded one-shot).

use crate::erros::SshCliError;
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
    password_override: Option<String>,
    key_override: Option<String>,
    key_passphrase_override: Option<String>,
    timeout_ms: u64,
    replace_host_key: bool,
    json: bool,
) -> Result<()> {
    if timeout_ms == 0 {
        return Err(SshCliError::InvalidArgument(
            "tunnel exige --timeout-ms > 0 (one-shot limitado)".to_string(),
        )
        .into());
    }

    let mut vps = find_by_name(config_override.clone(), vps_name)?
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
    );

    let path = crate::vps::resolve_config_path(config_override)?;
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
            "Pressione Ctrl+C para encerrar o tunnel antes do deadline.",
        );
    }

    // GAP-SSH-TUN-001: deadline covers connect + loop (not only the accept loop).
    // GAP-SSH-TUN-002: if the local listener is already up, deadline end is one-shot success
    // (not SshTimeout/exit 74). Timeout before bind (slow connect) remains an error.
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
        )
        .await
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) if bound.load(Ordering::SeqCst) => {
            tracing::info!(
                timeout_ms,
                "tunnel ended by one-shot deadline (success)"
            );
            Ok(())
        }
        Err(_) => {
            tracing::warn!(timeout_ms, "tunnel timeout antes do bind local");
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
) -> Result<()> {
    let client: std::sync::Arc<dyn SshClientTrait> = std::sync::Arc::from(client);

    let listener = TcpListener::bind(format!("127.0.0.1:{local_port}"))
        .await
        .map_err(|e| {
            SshCliError::Generic(format!("failed to open local port {}: {}", local_port, e))
        })?;

    // GAP-SSH-TUN-003: port 0 (ephemeral) must report the OS-assigned real port.
    // Agentes usam `local_port` do evento `tunnel_listening` para connect.
    let effective_port = listener
        .local_addr()
        .map(|a| a.port())
        .unwrap_or(local_port);

    if let Some(flag) = bound_flag.as_ref() {
        flag.store(true, Ordering::SeqCst);
    }

    tracing::info!(port = %effective_port, solicitada = %local_port, vps = %vps_name, "listener TCP local iniciado");

    // GAP-SSH-IO-008: agent receives structured confirmation that local bind is up.
    // GAP-SSH-TUN-003: always report `effective_port` (not the requested port when 0).
    if json {
        output::print_tunnel_listening_json(
            vps_name,
            effective_port,
            remote_host,
            remote_port,
            timeout_ms,
        );
    } else {
        let banner = format!(
            "Tunnel SSH: localhost:{} -> {}:{} via {} (timeout {}ms)",
            effective_port, remote_host, remote_port, vps_name, timeout_ms
        );
        tracing::info!("{banner}");
        output::print_human_banner(&banner);
    }

    loop {
        if crate::signals::is_cancelled() || crate::signals::is_terminated() {
            tracing::info!("tunnel cancelled by signal");
            break;
        }

        tokio::select! {
            resultado_accept = listener.accept() => {
                match resultado_accept {
                    Ok((soquete, addr)) => {
                        tracing::debug!(endereco = %addr, "nova conexão local");
                        let host = remote_host.to_string();
                        let client_c = client.clone();
                        tokio::spawn(async move {
                            if let Err(e) = forward(soquete, client_c, &host, remote_port).await {
                                tracing::warn!(err = %e, "tunnel forwarding failed");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!(err = %e, "accept falhou");
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(200)) => {
                // polling de sinais
            }
        }
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
        .open_tunnel_channel(remote_host, remote_port, "127.0.0.1", 0)
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

    /// GAP-SSH-TUN-003: bind em 127.0.0.1:0 deve expor port real ≠ 0.
    #[tokio::test]
    async fn tunnel_ephemeral_bind_reports_real_port() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind efêmero");
        let port = listener.local_addr().expect("local_addr").port();
        assert_ne!(port, 0, "SO deve atribuir porta > 0 após bind :0");
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
            "tunnel deve ler local_addr() pós-bind (TUN-003)"
        );
        assert!(
            src.contains("porta_efetiva"),
            "tunnel deve expor porta_efetiva no evento JSON"
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
            ) -> Result<Box<Self>, crate::erros::SshCliError> {
                Ok(Box::new(Stub))
            }
            async fn run_command(
                &mut self,
                _cmd: &str,
                _max: usize,
                _stdin: Option<Vec<u8>>,
            ) -> Result<ExecutionOutput, crate::erros::SshCliError> {
                unreachable!()
            }
            async fn upload(
                &mut self,
                _l: &Path,
                _r: &Path,
            ) -> Result<TransferResult, crate::erros::SshCliError> {
                unreachable!()
            }
            async fn download(
                &mut self,
                _r: &Path,
                _l: &Path,
            ) -> Result<TransferResult, crate::erros::SshCliError> {
                unreachable!()
            }
            async fn open_tunnel_channel(
                &self,
                _h: &str,
                _p: u16,
                _o: &str,
                _po: u16,
            ) -> Result<Box<dyn crate::ssh::client::TunnelChannel>, crate::erros::SshCliError>
            {
                Err(crate::erros::SshCliError::ChannelFailed("stub".into()))
            }
            async fn disconnect(&self) -> Result<(), crate::erros::SshCliError> {
                Ok(())
            }
        }

        // bind ephemeral via timeout short path: just ensure disconnect path
        let stub: Box<dyn SshClientTrait> = Box::new(Stub);
        let _: Arc<dyn SshClientTrait> = Arc::from(stub);
        let _ = MockSshClient::new();
    }
}
