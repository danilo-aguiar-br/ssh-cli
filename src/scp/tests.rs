// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: unit tests extracted for line budget.
#![forbid(unsafe_code)]

use super::*;
    use crate::cli::SshAuthArgs;
    use crate::ssh::client::{
        ConnectionConfig, ExecutionOutput, TransferResult, TunnelChannel, SshClientTrait,
    };
    use crate::vps::{self, ConfigFile};
    use crate::vps::model::{CURRENT_SCHEMA_VERSION, VpsRecord};
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
                "not implemented in test".into(),
            ))
        }

        async fn run_command(
            &mut self,
            _cmd: &str,
            _max_chars: usize,
            _stdin_data: Option<Vec<u8>>,
        ) -> Result<ExecutionOutput, SshCliError> {
            Err(SshCliError::channel_msg(
                "not implemented in test",
            ))
        }

        async fn upload(
            &self,
            _local: &Path,
            _remote: &Path,
        ) -> Result<TransferResult, SshCliError> {
            if self.upload_ok {
                Ok(TransferResult {
                    bytes_transferred: self.bytes_upload,
                    duration_ms: 10,
                })
            } else {
                Err(SshCliError::channel_msg("upload failed"))
            }
        }

        async fn download(
            &self,
            _remote: &Path,
            _local: &Path,
        ) -> Result<TransferResult, SshCliError> {
            if self.download_ok {
                Ok(TransferResult {
                    bytes_transferred: self.bytes_download,
                    duration_ms: 20,
                })
            } else {
                Err(SshCliError::channel_msg("download failed"))
            }
        }

        async fn open_tunnel_channel(
            &self,
            _host_remoto: &str,
            _porta_remota: u16,
            _endereco_origem: &str,
            _porta_origem: u16,
        ) -> Result<Box<dyn TunnelChannel>, SshCliError> {
            Err(SshCliError::channel_msg(
                "not implemented in test",
            ))
        }

        async fn disconnect(&self) -> Result<(), SshCliError> {
            Ok(())
        }
    }

    fn registro_teste(name: &str) -> VpsRecord {
        VpsRecord::test_new(
            name,
            "127.0.0.1",
            1,
            "root",
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

    fn empty_auth() -> SshAuthArgs {
        SshAuthArgs {
            password: None,
            password_stdin: false,
            key: None,
            key_passphrase: None,
            key_passphrase_stdin: false,
            use_agent: false,
            agent_socket: None,
        }
    }

    /// G-PAR-47 / G-PAR-54: multi-file uses one session — N uploads, zero extra connects.
    struct CountingSessionClient {
        uploads: std::sync::atomic::AtomicUsize,
        downloads: std::sync::atomic::AtomicUsize,
    }

    #[async_trait]
    impl SshClientTrait for CountingSessionClient {
        async fn connect(_cfg: ConnectionConfig) -> Result<Box<Self>, SshCliError> {
            // Production multi-file calls connect once outside this helper.
            // If this is invoked, session-reuse was broken.
            Err(SshCliError::ConnectionFailed(
                "connect must not be called from multi_file_*_on_session".into(),
            ))
        }

        async fn run_command(
            &mut self,
            _cmd: &str,
            _max_chars: usize,
            _stdin_data: Option<Vec<u8>>,
        ) -> Result<ExecutionOutput, SshCliError> {
            Err(SshCliError::channel_msg("unused"))
        }

        async fn upload(
            &self,
            _local: &Path,
            _remote: &Path,
        ) -> Result<TransferResult, SshCliError> {
            self.uploads
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(TransferResult {
                bytes_transferred: 1,
                duration_ms: 1,
            })
        }

        async fn download(
            &self,
            _remote: &Path,
            _local: &Path,
        ) -> Result<TransferResult, SshCliError> {
            self.downloads
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(TransferResult {
                bytes_transferred: 2,
                duration_ms: 1,
            })
        }

        async fn open_tunnel_channel(
            &self,
            _host_remoto: &str,
            _porta_remota: u16,
            _endereco_origem: &str,
            _porta_origem: u16,
        ) -> Result<Box<dyn TunnelChannel>, SshCliError> {
            Err(SshCliError::channel_msg("unused"))
        }

        async fn disconnect(&self) -> Result<(), SshCliError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn multi_file_upload_on_session_n_files_one_client() {
        let client = CountingSessionClient {
            uploads: std::sync::atomic::AtomicUsize::new(0),
            downloads: std::sync::atomic::AtomicUsize::new(0),
        };
        let sources = vec![
            PathBuf::from("a.bin"),
            PathBuf::from("b.bin"),
            PathBuf::from("c.bin"),
        ];
        let results =
            batch::multi_file_upload_on_session(&client, &sources, Path::new("/tmp"), None).await;
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.ok));
        assert_eq!(
            client.uploads.load(std::sync::atomic::Ordering::SeqCst),
            3,
            "three serial uploads on the same session"
        );
        assert_eq!(
            client.downloads.load(std::sync::atomic::Ordering::SeqCst),
            0
        );
    }

    #[tokio::test]
    async fn multi_file_download_on_session_n_files_one_client() {
        let client = CountingSessionClient {
            uploads: std::sync::atomic::AtomicUsize::new(0),
            downloads: std::sync::atomic::AtomicUsize::new(0),
        };
        let remotes = vec![PathBuf::from("/r/a"), PathBuf::from("/r/b")];
        let tmp = TempDir::new().unwrap();
        let results =
            batch::multi_file_download_on_session(&client, &remotes, tmp.path(), Some("prod")).await;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.ok));
        assert!(results[0].name.starts_with("prod:"));
        assert_eq!(
            client.downloads.load(std::sync::atomic::Ordering::SeqCst),
            2
        );
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
            all: false,
            hosts: None,
            target: vec![
                "vps-upload".to_string(),
                local.display().to_string(),
                "/tmp/x".to_string(),
            ],
            auth: empty_auth(),
            timeout: Some(100),
            json: false,
        };
        let r = run_scp(
            action,
            Some(tmp.path().to_path_buf()),
            ScpOptions {
                timeout: Some(crate::domain::TimeoutMs::try_new(100).expect("timeout")),
                ..Default::default()
            },
        )
        .await;
        assert!(r.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn scp_download_tries_connect_when_vps_exists() {
        let tmp = TempDir::new().unwrap();
        save_config_with_vps(&tmp, "vps-download");
        let action = ScpAction::Download {
            all: false,
            hosts: None,
            target: vec![
                "vps-download".to_string(),
                "/tmp/x".to_string(),
                tmp.path().join("out.bin").display().to_string(),
            ],
            auth: empty_auth(),
            timeout: Some(100),
            json: false,
        };
        let r = run_scp(
            action,
            Some(tmp.path().to_path_buf()),
            ScpOptions {
                timeout: Some(crate::domain::TimeoutMs::try_new(100).expect("timeout")),
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
            all: false,
            hosts: None,
            target: vec![
                "vps-dir".to_string(),
                tmp.path().display().to_string(),
                "/tmp/x".to_string(),
            ],
            auth: empty_auth(),
            timeout: None,
            json: false,
        };
        let r = run_scp(action, Some(tmp.path().to_path_buf()), ScpOptions::default()).await;
        assert!(r.is_err());
    }
