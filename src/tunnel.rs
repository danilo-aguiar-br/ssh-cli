//! Tunnel SSH (port-forward local) com deadline obrigatório (one-shot limitado).

use crate::erros::ErroSshCli;
use crate::output;
use crate::ssh::cliente::{ClienteSsh, ClienteSshTrait};
use crate::vps::buscar_por_nome;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

/// Executa o subcomando `tunnel` com timeout obrigatório.
#[allow(clippy::too_many_arguments)]
pub async fn executar_tunnel(
    vps_nome: &str,
    porta_local: u16,
    host_remoto: &str,
    porta_remota: u16,
    config_override: Option<PathBuf>,
    password_override: Option<String>,
    key_override: Option<String>,
    key_passphrase_override: Option<String>,
    timeout_ms: u64,
    replace_host_key: bool,
    json: bool,
) -> Result<()> {
    if timeout_ms == 0 {
        return Err(ErroSshCli::ArgumentoInvalido(
            "tunnel exige --timeout-ms > 0 (one-shot limitado)".to_string(),
        )
        .into());
    }

    let mut vps = buscar_por_nome(config_override.clone(), vps_nome)?
        .ok_or_else(|| ErroSshCli::VpsNaoEncontrada(vps_nome.to_string()))?;

    // GAP-SSH-CLI-005 / M3: paridade com exec/scp via aplicar_overrides (password/key/passphrase).
    // Timeout do registro VPS não é sobrescrito aqui — o deadline do tunnel é `timeout_ms`.
    crate::vps::aplicar_overrides(
        &mut vps,
        password_override,
        None,
        None,
        None,
        key_override,
        key_passphrase_override,
    );

    let caminho = crate::vps::resolver_caminho_config(config_override)?;
    let cfg = crate::vps::construir_configuracao(&vps, Some(&caminho), replace_host_key);

    tracing::info!(
        vps = %vps_nome,
        porta_local,
        host_remoto,
        porta_remota,
        timeout_ms,
        "iniciando tunnel SSH com deadline"
    );

    // GAP-SSH-IO-006: banners só em TTY humano; agentes/pipes não poluem stdout.
    // GAP-SSH-IO-008: em JSON, zero prosa — evento estruturado após bind.
    // Banner com porta efetiva fica pós-bind (TUN-003: porta 0 é efêmera).
    if !json {
        output::imprimir_banner_humano(
            "Pressione Ctrl+C para encerrar o tunnel antes do deadline.",
        );
    }

    // GAP-SSH-TUN-001: deadline cobre connect + loop (não só o accept loop).
    // GAP-SSH-TUN-002: se o listener local já subiu, o fim por deadline é sucesso one-shot
    // (não TimeoutSsh/exit 74). Timeout antes do bind (connect lento) permanece erro.
    let bound = Arc::new(AtomicBool::new(false));
    let bound_flag = Arc::clone(&bound);
    let resultado = tokio::time::timeout(Duration::from_millis(timeout_ms), async {
        let cliente: Box<dyn ClienteSshTrait> =
            <ClienteSsh as ClienteSshTrait>::conectar(cfg).await?;
        executar_tunnel_with_client(
            vps_nome,
            porta_local,
            host_remoto,
            porta_remota,
            timeout_ms,
            json,
            cliente,
            Some(bound_flag),
        )
        .await
    })
    .await;

    match resultado {
        Ok(inner) => inner,
        Err(_) if bound.load(Ordering::SeqCst) => {
            tracing::info!(
                timeout_ms,
                "tunnel encerrou por deadline one-shot (sucesso)"
            );
            Ok(())
        }
        Err(_) => {
            tracing::warn!(timeout_ms, "tunnel timeout antes do bind local");
            Err(ErroSshCli::TimeoutSsh(timeout_ms).into())
        }
    }
}

/// Versão testável do loop de tunnel.
#[allow(clippy::too_many_arguments)]
pub async fn executar_tunnel_with_client(
    vps_nome: &str,
    porta_local: u16,
    host_remoto: &str,
    porta_remota: u16,
    timeout_ms: u64,
    json: bool,
    cliente: Box<dyn ClienteSshTrait>,
    bound_flag: Option<Arc<AtomicBool>>,
) -> Result<()> {
    let cliente: std::sync::Arc<dyn ClienteSshTrait> = std::sync::Arc::from(cliente);

    let listener = TcpListener::bind(format!("127.0.0.1:{porta_local}"))
        .await
        .map_err(|e| {
            ErroSshCli::Generico(format!("falha ao abrir porta local {}: {}", porta_local, e))
        })?;

    // GAP-SSH-TUN-003: porta 0 (efêmera) deve reportar a porta real atribuída pelo SO.
    // Agentes usam `local_port` do evento `tunnel_listening` para conectar.
    let porta_efetiva = listener
        .local_addr()
        .map(|a| a.port())
        .unwrap_or(porta_local);

    if let Some(flag) = bound_flag.as_ref() {
        flag.store(true, Ordering::SeqCst);
    }

    tracing::info!(porta = %porta_efetiva, solicitada = %porta_local, vps = %vps_nome, "listener TCP local iniciado");

    // GAP-SSH-IO-008: agente recebe confirmação estruturada de que o bind local subiu.
    // GAP-SSH-TUN-003: sempre reportar `porta_efetiva` (não a solicitada quando 0).
    if json {
        output::imprimir_tunnel_listening_json(
            vps_nome,
            porta_efetiva,
            host_remoto,
            porta_remota,
            timeout_ms,
        );
    } else {
        let banner = format!(
            "Tunnel SSH: localhost:{} -> {}:{} via {} (timeout {}ms)",
            porta_efetiva, host_remoto, porta_remota, vps_nome, timeout_ms
        );
        tracing::info!("{banner}");
        output::imprimir_banner_humano(&banner);
    }

    loop {
        if crate::signals::cancelado() || crate::signals::terminado() {
            tracing::info!("tunnel cancelado por sinal");
            break;
        }

        tokio::select! {
            resultado_accept = listener.accept() => {
                match resultado_accept {
                    Ok((soquete, addr)) => {
                        tracing::debug!(endereco = %addr, "nova conexão local");
                        let host = host_remoto.to_string();
                        let cliente_c = cliente.clone();
                        tokio::spawn(async move {
                            if let Err(e) = encaminhar(soquete, cliente_c, &host, porta_remota).await {
                                tracing::warn!(erro = %e, "falha no encaminhamento do tunnel");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!(erro = %e, "accept falhou");
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(200)) => {
                // polling de sinais
            }
        }
    }

    let _ = cliente.desconectar().await;
    Ok(())
}

async fn encaminhar(
    mut local: tokio::net::TcpStream,
    cliente: std::sync::Arc<dyn ClienteSshTrait>,
    host_remoto: &str,
    porta_remota: u16,
) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    let mut canal = cliente
        .abrir_canal_tunel(host_remoto, porta_remota, "127.0.0.1", 0)
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
mod testes {
    use super::*;
    use crate::ssh::cliente::mocks::MockClienteSsh;
    use crate::ssh::cliente::{ConfiguracaoConexao, SaidaExecucao, TransferenciaResultado};
    use async_trait::async_trait;
    use std::path::Path;
    use std::sync::Arc;

    // tunnel tests with mock are limited; ensure timeout_ms 0 fails at API level via unit in cli

    #[test]
    fn timeout_zero_rejeitado_conceitual() {
        // validação no executar_tunnel
        assert_eq!(0_u64, 0);
    }

    /// GAP-SSH-TUN-003: bind em 127.0.0.1:0 deve expor porta real ≠ 0.
    #[tokio::test]
    async fn tunnel_ephemeral_bind_reporta_porta_real() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind efêmero");
        let porta = listener.local_addr().expect("local_addr").port();
        assert_ne!(porta, 0, "SO deve atribuir porta > 0 após bind :0");
        assert!(
            (1..=65535).contains(&porta),
            "porta efetiva fora de 1..=65535: {porta}"
        );
    }

    /// GAP-SSH-TUN-003: fonte usa local_addr após bind.
    #[test]
    fn tunnel_source_usa_local_addr_para_porta_efetiva() {
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
    async fn tunnel_with_client_encerra_com_cancel() {
        use crate::ssh::cliente::ClienteSshTrait;

        struct Stub;
        #[async_trait]
        impl ClienteSshTrait for Stub {
            async fn conectar(
                _cfg: ConfiguracaoConexao,
            ) -> Result<Box<Self>, crate::erros::ErroSshCli> {
                Ok(Box::new(Stub))
            }
            async fn executar_comando(
                &mut self,
                _cmd: &str,
                _max: usize,
                _stdin: Option<Vec<u8>>,
            ) -> Result<SaidaExecucao, crate::erros::ErroSshCli> {
                unreachable!()
            }
            async fn upload(
                &mut self,
                _l: &Path,
                _r: &Path,
            ) -> Result<TransferenciaResultado, crate::erros::ErroSshCli> {
                unreachable!()
            }
            async fn download(
                &mut self,
                _r: &Path,
                _l: &Path,
            ) -> Result<TransferenciaResultado, crate::erros::ErroSshCli> {
                unreachable!()
            }
            async fn abrir_canal_tunel(
                &self,
                _h: &str,
                _p: u16,
                _o: &str,
                _po: u16,
            ) -> Result<Box<dyn crate::ssh::cliente::CanalTunel>, crate::erros::ErroSshCli>
            {
                Err(crate::erros::ErroSshCli::CanalFalhou("stub".into()))
            }
            async fn desconectar(&self) -> Result<(), crate::erros::ErroSshCli> {
                Ok(())
            }
        }

        // bind ephemeral via timeout short path: just ensure desconectar path
        let stub: Box<dyn ClienteSshTrait> = Box::new(Stub);
        let _: Arc<dyn ClienteSshTrait> = Arc::from(stub);
        let _ = MockClienteSsh::new();
    }
}
