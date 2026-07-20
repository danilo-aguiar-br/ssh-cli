    impl SshClient {
        /// Cleanly closes the SSH session.
        ///
        /// # Errors
        /// Propagates transport errors returned by `disconnect`.
        pub async fn disconnect(&self) -> SshCliResult<()> {
            let result = self
                .session
                .disconnect(russh::Disconnect::ByApplication, "closing", "en-US")
                .await;
            match result {
                Ok(()) => {
                    tracing::info!("SSH session closed");
                    Ok(())
                }
                Err(e) => {
                    tracing::warn!(err = %e, "failed to close SSH session");
                    Err(SshCliError::ConnectionFailed(format!(
                        "failed to disconnect: {e}"
                    )))
                }
            }
        }

        /// Opens a direct-tcpip channel for SSH forwarding.
        pub async fn open_tunnel_channel(
            &self,
            remote_host: &str,
            remote_port: u16,
            origin_addr: &str,
            origin_port: u16,
        ) -> SshCliResult<Box<dyn TunnelChannel>> {
            let channel = self
                .session
                .channel_open_direct_tcpip(
                    remote_host.to_string(),
                    u32::from(remote_port),
                    origin_addr.to_string(),
                    u32::from(origin_port),
                )
                .await
                .map_err(|e| {
                    SshCliError::channel_msg(format!(
                        "failed to open direct-tcpip channel to {}:{}: {}",
                        remote_host, remote_port, e
                    ))
                })?;

            Ok(Box::new(channel.into_stream()))
        }

        // ── SFTP (G-SFTP) — wire lives in `sftp_session` (SRP; do not inline) ──

        /// Opens one SFTP subsystem session (reuse for multi-file / multi-op).
        pub async fn open_sftp(&self) -> SshCliResult<russh_sftp::client::SftpSession> {
            crate::ssh::sftp_session::open_sftp_session(&self.session, self.cfg.timeout_ms.get())
                .await
        }

        /// One-shot SFTP upload of a regular file (opens+closes subsystem).
        pub async fn sftp_upload(
            &self,
            local: &std::path::Path,
            remote: &str,
        ) -> SshCliResult<TransferResult> {
            let timeout = Duration::from_millis(self.cfg.timeout_ms.get());
            tokio::time::timeout(timeout, async {
                let sftp = self.open_sftp().await?;
                let result = crate::ssh::sftp_session::upload_file(&sftp, local, remote).await;
                crate::ssh::sftp_session::close_sftp(&sftp).await;
                result
            })
            .await
            .map_err(|_| SshCliError::SshTimeout(self.cfg.timeout_ms.get()))?
        }

        /// One-shot SFTP download of a regular file.
        pub async fn sftp_download(
            &self,
            remote: &str,
            local: &std::path::Path,
        ) -> SshCliResult<TransferResult> {
            let timeout = Duration::from_millis(self.cfg.timeout_ms.get());
            tokio::time::timeout(timeout, async {
                let sftp = self.open_sftp().await?;
                let result = crate::ssh::sftp_session::download_file(&sftp, remote, local).await;
                crate::ssh::sftp_session::close_sftp(&sftp).await;
                result
            })
            .await
            .map_err(|_| SshCliError::SshTimeout(self.cfg.timeout_ms.get()))?
        }

        /// One-shot recursive SFTP upload tree.
        pub async fn sftp_upload_tree(
            &self,
            local_dir: &std::path::Path,
            remote_dir: &str,
        ) -> SshCliResult<TransferResult> {
            let timeout = Duration::from_millis(self.cfg.timeout_ms.get());
            tokio::time::timeout(timeout, async {
                let sftp = self.open_sftp().await?;
                let result =
                    crate::ssh::sftp_session::upload_tree(&sftp, local_dir, remote_dir).await;
                crate::ssh::sftp_session::close_sftp(&sftp).await;
                result
            })
            .await
            .map_err(|_| SshCliError::SshTimeout(self.cfg.timeout_ms.get()))?
        }

        /// One-shot recursive SFTP download tree.
        pub async fn sftp_download_tree(
            &self,
            remote_dir: &str,
            local_dir: &std::path::Path,
        ) -> SshCliResult<TransferResult> {
            let timeout = Duration::from_millis(self.cfg.timeout_ms.get());
            tokio::time::timeout(timeout, async {
                let sftp = self.open_sftp().await?;
                let result =
                    crate::ssh::sftp_session::download_tree(&sftp, remote_dir, local_dir).await;
                crate::ssh::sftp_session::close_sftp(&sftp).await;
                result
            })
            .await
            .map_err(|_| SshCliError::SshTimeout(self.cfg.timeout_ms.get()))?
        }
    }

    #[async_trait]
    impl SshClientTrait for SshClient {
        async fn connect(cfg: ConnectionConfig) -> Result<Box<Self>, SshCliError> {
            Self::connect(cfg).await.map(Box::new)
        }

        async fn run_command(
            &mut self,
            cmd: &str,
            max_chars: usize,
            stdin_data: Option<Vec<u8>>,
        ) -> Result<ExecutionOutput, SshCliError> {
            Self::run_command(self, cmd, max_chars, stdin_data).await
        }

        async fn upload(
            &self,
            local: &Path,
            remote: &Path,
        ) -> Result<TransferResult, SshCliError> {
            Self::upload(self, local, remote).await
        }

        async fn download(
            &self,
            remote: &Path,
            local: &Path,
        ) -> Result<TransferResult, SshCliError> {
            Self::download(self, remote, local).await
        }

        async fn open_tunnel_channel(
            &self,
            remote_host: &str,
            remote_port: u16,
            origin_addr: &str,
            origin_port: u16,
        ) -> Result<Box<dyn TunnelChannel>, SshCliError> {
            Self::open_tunnel_channel(
                self,
                remote_host,
                remote_port,
                origin_addr,
                origin_port,
            )
            .await
        }

        async fn disconnect(&self) -> Result<(), SshCliError> {
            Self::disconnect(self).await
        }
    }
