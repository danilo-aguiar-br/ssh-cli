// SPDX-License-Identifier: MIT OR Apache-2.0
//! File transfer via SCP over SSH (one-shot).
//!
//! Wrapper around [`SshClient`] `upload` and `download` methods.
//! Regular files only (no `-r` / no SFTP subsystem).

use crate::cli::ScpAction;
use crate::erros::SshCliError;
use crate::i18n::{self, Message};
use crate::output;
use crate::ssh::client::{SshClient, SshClientTrait};
use crate::vps;
use std::path::PathBuf;

/// Runtime overrides for the `scp` subcommand (parity with exec).
#[derive(Debug, Default, Clone)]
pub struct ScpOptions {
    /// SSH password (already resolved from flag or stdin).
    pub password: Option<String>,
    /// Private key path.
    pub key: Option<String>,
    /// Key passphrase (already resolved).
    pub key_passphrase: Option<String>,
    /// Total connect+transfer timeout in ms.
    pub timeout: Option<u64>,
    /// Replace divergent host key (global `--replace-host-key`).
    pub replace_host_key: bool,
    /// Emit success JSON (local flag or global format).
    pub json: bool,
}

/// Runs the SCP subcommand (upload/download).
pub async fn run_scp(
    action: ScpAction,
    config_override: Option<PathBuf>,
    opts: ScpOptions,
) -> anyhow::Result<()> {
    if crate::signals::is_cancelled() {
        return Err(anyhow::anyhow!(i18n::t(Message::OperationCancelled)));
    }

    match action {
        ScpAction::Upload {
            vps_name,
            local,
            remote,
            ..
        } => {
            // GAP-SSH-SCP-001 / SCP-019: validate file local antes do connect.
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

            apply_scp_options(&mut registro, &opts);

            let path = crate::vps::resolve_config_path(config_override.clone())?;
            let cfg = crate::vps::build_connection_config(
                &registro,
                Some(&path),
                opts.replace_host_key,
            );

            let client: Box<dyn SshClientTrait> =
                <SshClient as SshClientTrait>::connect(cfg).await?;
            run_scp_upload_with_client(&vps_name, &local, &remote, client, opts.json).await?;
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

            apply_scp_options(&mut registro, &opts);

            let path = crate::vps::resolve_config_path(config_override.clone())?;
            let cfg = crate::vps::build_connection_config(
                &registro,
                Some(&path),
                opts.replace_host_key,
            );

            let client: Box<dyn SshClientTrait> =
                <SshClient as SshClientTrait>::connect(cfg).await?;
            run_scp_download_with_client(&vps_name, &remote, &local, client, opts.json)
                .await?;
        }
    }
    Ok(())
}

fn apply_scp_options(registro: &mut crate::vps::model::VpsRecord, opts: &ScpOptions) {
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

/// Testable SCP upload that accepts the client as a parameter.
pub async fn run_scp_upload_with_client(
    vps_name: &str,
    local: &std::path::Path,
    remote: &std::path::Path,
    mut client: Box<dyn SshClientTrait>,
    json: bool,
) -> anyhow::Result<()> {
    let result = client.upload(local, remote).await?;
    client.disconnect().await?;
    if json {
        output::print_transfer_json(
            "upload",
            vps_name,
            &local.display().to_string(),
            &remote.display().to_string(),
            result.bytes_transferred,
            result.duration_ms,
        );
    } else {
        output::print_success(&i18n::t(Message::ScpUploadCompleted {
            bytes: result.bytes_transferred,
            ms: result.duration_ms,
        }));
    }
    Ok(())
}

/// Testable SCP download that accepts the client as a parameter.
pub async fn run_scp_download_with_client(
    vps_name: &str,
    remote: &std::path::Path,
    local: &std::path::Path,
    mut client: Box<dyn SshClientTrait>,
    json: bool,
) -> anyhow::Result<()> {
    let result = client.download(remote, local).await?;
    client.disconnect().await?;
    if json {
        output::print_transfer_json(
            "download",
            vps_name,
            &local.display().to_string(),
            &remote.display().to_string(),
            result.bytes_transferred,
            result.duration_ms,
        );
    } else {
        output::print_success(&i18n::t(Message::ScpDownloadCompleted {
            bytes: result.bytes_transferred,
            ms: result.duration_ms,
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

    struct FakeScpClient {
        upload_ok: bool,
        download_ok: bool,
        bytes_upload: u64,
        bytes_download: u64,
    }

    #[async_trait]
    impl SshClientTrait for FakeScpClient {
        async fn connect(_cfg: ConnectionConfig) -> Result<Box<Self>, SshCliError> {
            Err(SshCliError::ConnectionFailed(
                "not implemented in test".to_string(),
            ))
        }

        async fn run_command(
            &mut self,
            _cmd: &str,
            _max_chars: usize,
            _stdin_data: Option<Vec<u8>>,
        ) -> Result<ExecutionOutput, SshCliError> {
            Err(SshCliError::ChannelFailed(
                "not implemented in test".to_string(),
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
                "not implemented in test".to_string(),
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

    fn save_config_with_vps(tmp: &TempDir, name: &str) {
        let mut hosts = BTreeMap::new();
        hosts.insert(name.to_string(), registro_teste(name));
        let file = ConfigFile {
            schema_version: CURRENT_SCHEMA_VERSION,
            hosts,
        };
        let path = tmp.path().join("config.toml");
        vps::save(&path, &file).expect("save test config");
    }

    #[tokio::test]
    async fn scp_upload_with_client_returns_ok() {
        let client = Box::new(FakeScpClient {
            upload_ok: true,
            download_ok: true,
            bytes_upload: 128,
            bytes_download: 0,
        });
        let local = Path::new("/tmp/local.txt");
        let remote = Path::new("/tmp/remote.txt");
        let result = run_scp_upload_with_client("v1", local, remote, client, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn scp_download_with_client_returns_ok() {
        let client = Box::new(FakeScpClient {
            upload_ok: true,
            download_ok: true,
            bytes_upload: 0,
            bytes_download: 256,
        });
        let result = run_scp_download_with_client(
            "v1",
            Path::new("/tmp/remote.txt"),
            Path::new("/tmp/local.txt"),
            client,
            false,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn scp_upload_with_client_returns_error() {
        let client = Box::new(FakeScpClient {
            upload_ok: false,
            download_ok: true,
            bytes_upload: 0,
            bytes_download: 0,
        });
        let result = run_scp_upload_with_client(
            "v1",
            Path::new("/tmp/local.txt"),
            Path::new("/tmp/remote.txt"),
            client,
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn scp_download_with_client_returns_error() {
        let client = Box::new(FakeScpClient {
            upload_ok: true,
            download_ok: false,
            bytes_upload: 0,
            bytes_download: 0,
        });
        let result = run_scp_download_with_client(
            "v1",
            Path::new("/tmp/remote.txt"),
            Path::new("/tmp/local.txt"),
            client,
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn scp_upload_tries_connect_when_vps_exists() {
        let tmp = TempDir::new().unwrap();
        save_config_with_vps(&tmp, "vps-upload");
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
            ScpOptions {
                timeout: Some(100),
                ..Default::default()
            },
        )
        .await;
        // Connection to 127.0.0.1:1 must fail (no long hang; 100ms timeout).
        assert!(r.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn scp_download_tries_connect_when_vps_exists() {
        let tmp = TempDir::new().unwrap();
        save_config_with_vps(&tmp, "vps-download");
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
            ScpOptions {
                timeout: Some(100),
                ..Default::default()
            },
        )
        .await;
        assert!(r.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn scp_upload_rejects_directory() {
        let tmp = TempDir::new().unwrap();
        save_config_with_vps(&tmp, "vps-dir");
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
        let r = run_scp(action, Some(tmp.path().to_path_buf()), ScpOptions::default()).await;
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
