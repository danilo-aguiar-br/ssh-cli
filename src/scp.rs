//! Transferência de arquivos via SCP sobre SSH (one-shot).
//!
//! Wrapper que usa os métodos `upload` e `download` do [`ClienteSsh`].
//! Somente arquivos regulares (sem `-r` / sem SFTP subsystem).

use crate::cli::AcaoScp;
use crate::erros::ErroSshCli;
use crate::i18n::{self, Mensagem};
use crate::output;
use crate::ssh::cliente::{ClienteSsh, ClienteSshTrait};
use crate::vps;
use std::path::PathBuf;

/// Overrides de runtime para o subcomando `scp` (paridade com exec).
#[derive(Debug, Default, Clone)]
pub struct OpcoesScp {
    /// Senha SSH (já resolvida de flag ou stdin).
    pub password: Option<String>,
    /// Caminho da chave privada.
    pub key: Option<String>,
    /// Passphrase da chave (já resolvida).
    pub key_passphrase: Option<String>,
    /// Timeout total connect+transfer em ms.
    pub timeout: Option<u64>,
    /// Substitui host key divergente (global `--replace-host-key`).
    pub replace_host_key: bool,
    /// Emite JSON de sucesso (flag local ou formato global).
    pub json: bool,
}

/// Executa o subcomando SCP (upload/download).
pub async fn executar_scp(
    acao: AcaoScp,
    config_override: Option<PathBuf>,
    opts: OpcoesScp,
) -> anyhow::Result<()> {
    if crate::signals::cancelado() {
        return Err(anyhow::anyhow!(i18n::t(Mensagem::OperacaoCancelada)));
    }

    match acao {
        AcaoScp::Upload {
            vps_nome,
            local,
            remote,
            ..
        } => {
            // GAP-SSH-SCP-001 / SCP-019: validar arquivo local antes do connect.
            if local.is_dir() {
                return Err(ErroSshCli::ArgumentoInvalido(
                    "upload only supports regular files (no directories / no -r)".to_string(),
                )
                .into());
            }
            if !local.is_file() {
                return Err(ErroSshCli::ArquivoNaoEncontrado(local.display().to_string()).into());
            }

            let mut registro = vps::buscar_por_nome(config_override.clone(), &vps_nome)?
                .ok_or_else(|| ErroSshCli::VpsNaoEncontrada(vps_nome.clone()))?;

            aplicar_opcoes_scp(&mut registro, &opts);

            let caminho = crate::vps::resolver_caminho_config(config_override.clone())?;
            let cfg = crate::vps::construir_configuracao(
                &registro,
                Some(&caminho),
                opts.replace_host_key,
            );

            let cliente: Box<dyn ClienteSshTrait> =
                <ClienteSsh as ClienteSshTrait>::conectar(cfg).await?;
            executar_scp_upload_with_client(&vps_nome, &local, &remote, cliente, opts.json).await?;
        }
        AcaoScp::Download {
            vps_nome,
            remote,
            local,
            ..
        } => {
            if local.is_dir() {
                return Err(ErroSshCli::ArgumentoInvalido(
                    "download local path must be a file path, not an existing directory"
                        .to_string(),
                )
                .into());
            }

            let mut registro = vps::buscar_por_nome(config_override.clone(), &vps_nome)?
                .ok_or_else(|| ErroSshCli::VpsNaoEncontrada(vps_nome.clone()))?;

            aplicar_opcoes_scp(&mut registro, &opts);

            let caminho = crate::vps::resolver_caminho_config(config_override.clone())?;
            let cfg = crate::vps::construir_configuracao(
                &registro,
                Some(&caminho),
                opts.replace_host_key,
            );

            let cliente: Box<dyn ClienteSshTrait> =
                <ClienteSsh as ClienteSshTrait>::conectar(cfg).await?;
            executar_scp_download_with_client(&vps_nome, &remote, &local, cliente, opts.json)
                .await?;
        }
    }
    Ok(())
}

fn aplicar_opcoes_scp(registro: &mut crate::vps::modelo::VpsRegistro, opts: &OpcoesScp) {
    if let Some(ref pwd) = opts.password {
        registro.senha = secrecy::SecretString::from(pwd.clone());
    }
    if let Some(ref k) = opts.key {
        registro.key_path = Some(k.clone());
    }
    if let Some(ref kp) = opts.key_passphrase {
        registro.key_passphrase = Some(secrecy::SecretString::from(kp.clone()));
    }
    if let Some(t) = opts.timeout {
        registro.timeout_ms = t;
    }
}

/// Versão testável de upload SCP que aceita o cliente como parâmetro.
pub async fn executar_scp_upload_with_client(
    vps_nome: &str,
    local: &std::path::Path,
    remote: &std::path::Path,
    mut cliente: Box<dyn ClienteSshTrait>,
    json: bool,
) -> anyhow::Result<()> {
    let resultado = cliente.upload(local, remote).await?;
    cliente.desconectar().await?;
    if json {
        output::imprimir_transferencia_json(
            "upload",
            vps_nome,
            &local.display().to_string(),
            &remote.display().to_string(),
            resultado.bytes_transferidos,
            resultado.duracao_ms,
        );
    } else {
        output::imprimir_sucesso(&i18n::t(Mensagem::ScpUploadConcluido {
            bytes: resultado.bytes_transferidos,
            ms: resultado.duracao_ms,
        }));
    }
    Ok(())
}

/// Versão testável de download SCP que aceita o cliente como parâmetro.
pub async fn executar_scp_download_with_client(
    vps_nome: &str,
    remote: &std::path::Path,
    local: &std::path::Path,
    mut cliente: Box<dyn ClienteSshTrait>,
    json: bool,
) -> anyhow::Result<()> {
    let resultado = cliente.download(remote, local).await?;
    cliente.desconectar().await?;
    if json {
        output::imprimir_transferencia_json(
            "download",
            vps_nome,
            &local.display().to_string(),
            &remote.display().to_string(),
            resultado.bytes_transferidos,
            resultado.duracao_ms,
        );
    } else {
        output::imprimir_sucesso(&i18n::t(Mensagem::ScpDownloadConcluido {
            bytes: resultado.bytes_transferidos,
            ms: resultado.duracao_ms,
        }));
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::erros::ErroSshCli;
    use crate::ssh::cliente::{
        CanalTunel, ConfiguracaoConexao, SaidaExecucao, TransferenciaResultado,
    };
    use crate::vps::modelo::{VpsRegistro, SCHEMA_VERSION_ATUAL};
    use crate::vps::{self, ArquivoConfig};
    use async_trait::async_trait;
    use secrecy::SecretString;
    use serial_test::serial;
    use std::collections::BTreeMap;
    use std::path::Path;
    use tempfile::TempDir;

    struct ClienteFakeScp {
        upload_ok: bool,
        download_ok: bool,
        bytes_upload: u64,
        bytes_download: u64,
    }

    #[async_trait]
    impl ClienteSshTrait for ClienteFakeScp {
        async fn conectar(_cfg: ConfiguracaoConexao) -> Result<Box<Self>, ErroSshCli> {
            Err(ErroSshCli::ConexaoFalhou(
                "não implementado em teste".to_string(),
            ))
        }

        async fn executar_comando(
            &mut self,
            _cmd: &str,
            _max_chars: usize,
            _stdin_data: Option<Vec<u8>>,
        ) -> Result<SaidaExecucao, ErroSshCli> {
            Err(ErroSshCli::CanalFalhou(
                "não implementado em teste".to_string(),
            ))
        }

        async fn upload(
            &mut self,
            _local: &Path,
            _remote: &Path,
        ) -> Result<TransferenciaResultado, ErroSshCli> {
            if self.upload_ok {
                Ok(TransferenciaResultado {
                    bytes_transferidos: self.bytes_upload,
                    duracao_ms: 10,
                })
            } else {
                Err(ErroSshCli::CanalFalhou("upload falhou".to_string()))
            }
        }

        async fn download(
            &mut self,
            _remote: &Path,
            _local: &Path,
        ) -> Result<TransferenciaResultado, ErroSshCli> {
            if self.download_ok {
                Ok(TransferenciaResultado {
                    bytes_transferidos: self.bytes_download,
                    duracao_ms: 20,
                })
            } else {
                Err(ErroSshCli::CanalFalhou("download falhou".to_string()))
            }
        }

        async fn abrir_canal_tunel(
            &self,
            _host_remoto: &str,
            _porta_remota: u16,
            _endereco_origem: &str,
            _porta_origem: u16,
        ) -> Result<Box<dyn CanalTunel>, ErroSshCli> {
            Err(ErroSshCli::CanalFalhou(
                "não implementado em teste".to_string(),
            ))
        }

        async fn desconectar(&self) -> Result<(), ErroSshCli> {
            Ok(())
        }
    }

    fn registro_teste(nome: &str) -> VpsRegistro {
        VpsRegistro::novo(
            nome.to_string(),
            "127.0.0.1".to_string(),
            1,
            "root".to_string(),
            SecretString::from("senha-teste".to_string()),
            None,
            None,
            Some(100),
            Some(1000),
            Some(1000),
            None,
            None,
            false,
        )
    }

    fn salvar_config_com_vps(tmp: &TempDir, nome: &str) {
        let mut hosts = BTreeMap::new();
        hosts.insert(nome.to_string(), registro_teste(nome));
        let arquivo = ArquivoConfig {
            schema_version: SCHEMA_VERSION_ATUAL,
            hosts,
        };
        let caminho = tmp.path().join("config.toml");
        vps::salvar(&caminho, &arquivo).expect("salvar config teste");
    }

    #[tokio::test]
    async fn executar_scp_upload_with_client_retorna_ok() {
        let cliente = Box::new(ClienteFakeScp {
            upload_ok: true,
            download_ok: true,
            bytes_upload: 128,
            bytes_download: 0,
        });
        let local = Path::new("/tmp/local.txt");
        let remote = Path::new("/tmp/remote.txt");
        let resultado = executar_scp_upload_with_client("v1", local, remote, cliente, false).await;
        assert!(resultado.is_ok());
    }

    #[tokio::test]
    async fn executar_scp_download_with_client_retorna_ok() {
        let cliente = Box::new(ClienteFakeScp {
            upload_ok: true,
            download_ok: true,
            bytes_upload: 0,
            bytes_download: 256,
        });
        let resultado = executar_scp_download_with_client(
            "v1",
            Path::new("/tmp/remote.txt"),
            Path::new("/tmp/local.txt"),
            cliente,
            false,
        )
        .await;
        assert!(resultado.is_ok());
    }

    #[tokio::test]
    async fn executar_scp_upload_with_client_retorna_erro() {
        let cliente = Box::new(ClienteFakeScp {
            upload_ok: false,
            download_ok: true,
            bytes_upload: 0,
            bytes_download: 0,
        });
        let resultado = executar_scp_upload_with_client(
            "v1",
            Path::new("/tmp/local.txt"),
            Path::new("/tmp/remote.txt"),
            cliente,
            false,
        )
        .await;
        assert!(resultado.is_err());
    }

    #[tokio::test]
    async fn executar_scp_download_with_client_retorna_erro() {
        let cliente = Box::new(ClienteFakeScp {
            upload_ok: true,
            download_ok: false,
            bytes_upload: 0,
            bytes_download: 0,
        });
        let resultado = executar_scp_download_with_client(
            "v1",
            Path::new("/tmp/remote.txt"),
            Path::new("/tmp/local.txt"),
            cliente,
            false,
        )
        .await;
        assert!(resultado.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn executar_scp_upload_tenta_conectar_quando_vps_existe() {
        let tmp = TempDir::new().unwrap();
        salvar_config_com_vps(&tmp, "vps-upload");
        let local = tmp.path().join("local.bin");
        std::fs::write(&local, b"abc").unwrap();
        let acao = AcaoScp::Upload {
            vps_nome: "vps-upload".to_string(),
            local,
            remote: PathBuf::from("/tmp/x"),
            password: None,
            password_stdin: false,
            key: None,
            key_passphrase: None,
            key_passphrase_stdin: false,
            timeout: Some(100),
            json: false,
        };
        let r = executar_scp(
            acao,
            Some(tmp.path().to_path_buf()),
            OpcoesScp {
                timeout: Some(100),
                ..Default::default()
            },
        )
        .await;
        // Conexão a 127.0.0.1:1 deve falhar (sem hang longo por timeout 100ms).
        assert!(r.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn executar_scp_download_tenta_conectar_quando_vps_existe() {
        let tmp = TempDir::new().unwrap();
        salvar_config_com_vps(&tmp, "vps-download");
        let acao = AcaoScp::Download {
            vps_nome: "vps-download".to_string(),
            remote: PathBuf::from("/tmp/x"),
            local: tmp.path().join("out.bin"),
            password: None,
            password_stdin: false,
            key: None,
            key_passphrase: None,
            key_passphrase_stdin: false,
            timeout: Some(100),
            json: false,
        };
        let r = executar_scp(
            acao,
            Some(tmp.path().to_path_buf()),
            OpcoesScp {
                timeout: Some(100),
                ..Default::default()
            },
        )
        .await;
        assert!(r.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn executar_scp_upload_rejeita_diretorio() {
        let tmp = TempDir::new().unwrap();
        salvar_config_com_vps(&tmp, "vps-dir");
        let acao = AcaoScp::Upload {
            vps_nome: "vps-dir".to_string(),
            local: tmp.path().to_path_buf(),
            remote: PathBuf::from("/tmp/x"),
            password: None,
            password_stdin: false,
            key: None,
            key_passphrase: None,
            key_passphrase_stdin: false,
            timeout: None,
            json: false,
        };
        let r = executar_scp(acao, Some(tmp.path().to_path_buf()), OpcoesScp::default()).await;
        assert!(r.is_err());
        let msg = format!("{:?}", r.err().unwrap());
        assert!(
            msg.contains("regular files") || msg.contains("ArgumentoInvalido"),
            "msg={msg}"
        );
    }
}
