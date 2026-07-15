// SPDX-License-Identifier: MIT OR Apache-2.0
//! Transferência de arquivos via SCP sobre SSH (one-shot).
//!
//! Wrapper que usa os métodos `upload` e `download` do [`SshClient`].
//! Somente arquivos regulares (sem `-r` / sem SFTP subsystem).

use crate::cli::ScpAction;
use crate::erros::SshCliError;
use crate::i18n::{self, Message};
use crate::output;
use crate::ssh::client::{SshClient, SshClientTrait};
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
pub async fn run_scp(
    action: ScpAction,
    config_override: Option<PathBuf>,
    opts: OpcoesScp,
) -> anyhow::Result<()> {
    if crate::signals::cancelado() {
        return Err(anyhow::anyhow!(i18n::t(Message::OperationCancelled)));
    }

    match action {
        ScpAction::Upload {
            vps_name,
            local,
            remote,
            ..
        } => {
            // GAP-SSH-SCP-001 / SCP-019: validate arquivo local antes do connect.
            if local.is_dir() {
                return Err(SshCliError::InvalidArgument(i18n::t(
                    Message::ScpUploadFileOnly,
                ))
                .into());
            }
            if !local.is_file() {
                return Err(SshCliError::FileNotFound(local.display().to_string()).into());
            }

            let mut registro = vps::find_by_name(config_override.clone(), &vps_name)?
                .ok_or_else(|| SshCliError::VpsNotFound(vps_name.clone()))?;

            aplicar_opcoes_scp(&mut registro, &opts);

            let path = crate::vps::resolve_config_path(config_override.clone())?;
            let cfg = crate::vps::build_connection_config(
                &registro,
                Some(&path),
                opts.replace_host_key,
            );

            let cliente: Box<dyn SshClientTrait> =
                <SshClient as SshClientTrait>::connect(cfg).await?;
            run_scp_upload_with_client(&vps_name, &local, &remote, cliente, opts.json).await?;
        }
        ScpAction::Download {
            vps_name,
            remote,
            local,
            ..
        } => {
            if local.is_dir() {
                return Err(SshCliError::InvalidArgument(i18n::t(
                    Message::ScpDownloadLocalNotDirectory,
                ))
                .into());
            }

            let mut registro = vps::find_by_name(config_override.clone(), &vps_name)?
                .ok_or_else(|| SshCliError::VpsNotFound(vps_name.clone()))?;

            aplicar_opcoes_scp(&mut registro, &opts);

            let path = crate::vps::resolve_config_path(config_override.clone())?;
            let cfg = crate::vps::build_connection_config(
                &registro,
                Some(&path),
                opts.replace_host_key,
            );

            let cliente: Box<dyn SshClientTrait> =
                <SshClient as SshClientTrait>::connect(cfg).await?;
            run_scp_download_with_client(&vps_name, &remote, &local, cliente, opts.json)
                .await?;
        }
    }
    Ok(())
}

fn aplicar_opcoes_scp(registro: &mut crate::vps::model::VpsRecord, opts: &OpcoesScp) {
    if let Some(ref pwd) = opts.password {
        registro.password = secrecy::SecretString::from(pwd.clone());
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
pub async fn run_scp_upload_with_client(
    vps_name: &str,
    local: &std::path::Path,
    remote: &std::path::Path,
    mut cliente: Box<dyn SshClientTrait>,
    json: bool,
) -> anyhow::Result<()> {
    let resultado = cliente.upload(local, remote).await?;
    cliente.disconnect().await?;
    if json {
        output::print_transfer_json(
            "upload",
            vps_name,
            &local.display().to_string(),
            &remote.display().to_string(),
            resultado.bytes_transferred,
            resultado.duration_ms,
        );
    } else {
        output::print_success(&i18n::t(Message::ScpUploadCompleted {
            bytes: resultado.bytes_transferred,
            ms: resultado.duration_ms,
        }));
    }
    Ok(())
}

/// Versão testável de download SCP que aceita o cliente como parâmetro.
pub async fn run_scp_download_with_client(
    vps_name: &str,
    remote: &std::path::Path,
    local: &std::path::Path,
    mut cliente: Box<dyn SshClientTrait>,
    json: bool,
) -> anyhow::Result<()> {
    let resultado = cliente.download(remote, local).await?;
    cliente.disconnect().await?;
    if json {
        output::print_transfer_json(
            "download",
            vps_name,
            &local.display().to_string(),
            &remote.display().to_string(),
            resultado.bytes_transferred,
            resultado.duration_ms,
        );
    } else {
        output::print_success(&i18n::t(Message::ScpDownloadCompleted {
            bytes: resultado.bytes_transferred,
            ms: resultado.duration_ms,
        }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erros::SshCliError;
    use crate::ssh::client::{
        TunnelChannel, ConnectionConfig, ExecutionOutput, TransferResult,
    };
    use crate::vps::model::{VpsRecord, CURRENT_SCHEMA_VERSION};
    use crate::vps::{self, ConfigFile};
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
    impl SshClientTrait for ClienteFakeScp {
        async fn connect(_cfg: ConnectionConfig) -> Result<Box<Self>, SshCliError> {
            Err(SshCliError::ConnectionFailed(
                "não implementado em teste".to_string(),
            ))
        }

        async fn run_command(
            &mut self,
            _cmd: &str,
            _max_chars: usize,
            _stdin_data: Option<Vec<u8>>,
        ) -> Result<ExecutionOutput, SshCliError> {
            Err(SshCliError::ChannelFailed(
                "não implementado em teste".to_string(),
            ))
        }

        async fn upload(
            &mut self,
            _local: &Path,
            _remote: &Path,
        ) -> Result<TransferResult, SshCliError> {
            if self.upload_ok {
                Ok(TransferResult {
                    bytes_transferred: self.bytes_upload,
                    duration_ms: 10,
                })
            } else {
                Err(SshCliError::ChannelFailed("upload falhou".to_string()))
            }
        }

        async fn download(
            &mut self,
            _remote: &Path,
            _local: &Path,
        ) -> Result<TransferResult, SshCliError> {
            if self.download_ok {
                Ok(TransferResult {
                    bytes_transferred: self.bytes_download,
                    duration_ms: 20,
                })
            } else {
                Err(SshCliError::ChannelFailed("download falhou".to_string()))
            }
        }

        async fn open_tunnel_channel(
            &self,
            _host_remoto: &str,
            _porta_remota: u16,
            _endereco_origem: &str,
            _porta_origem: u16,
        ) -> Result<Box<dyn TunnelChannel>, SshCliError> {
            Err(SshCliError::ChannelFailed(
                "não implementado em teste".to_string(),
            ))
        }

        async fn disconnect(&self) -> Result<(), SshCliError> {
            Ok(())
        }
    }

    fn registro_teste(name: &str) -> VpsRecord {
        VpsRecord::new(
            name.to_string(),
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

    fn salvar_config_com_vps(tmp: &TempDir, name: &str) {
        let mut hosts = BTreeMap::new();
        hosts.insert(name.to_string(), registro_teste(name));
        let arquivo = ConfigFile {
            schema_version: CURRENT_SCHEMA_VERSION,
            hosts,
        };
        let path = tmp.path().join("config.toml");
        vps::salvar(&path, &arquivo).expect("salvar config teste");
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
        let resultado = run_scp_upload_with_client("v1", local, remote, cliente, false).await;
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
        let resultado = run_scp_download_with_client(
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
        let resultado = run_scp_upload_with_client(
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
        let resultado = run_scp_download_with_client(
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
        let action = ScpAction::Upload {
            vps_name: "vps-upload".to_string(),
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
        let r = run_scp(
            action,
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
        let action = ScpAction::Download {
            vps_name: "vps-download".to_string(),
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
        let r = run_scp(
            action,
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
        let action = ScpAction::Upload {
            vps_name: "vps-dir".to_string(),
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
        let r = run_scp(action, Some(tmp.path().to_path_buf()), OpcoesScp::default()).await;
        assert!(r.is_err());
        let msg = format!("{:?}", r.err().unwrap());
        assert!(
            msg.contains("regular files")
                || msg.contains("arquivos regulares")
                || msg.contains("InvalidArgument"),
            "msg={msg}"
        );
    }
}
