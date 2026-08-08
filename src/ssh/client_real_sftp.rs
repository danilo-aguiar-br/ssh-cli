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
                // E7: `channel_msg` with the error interpolated into the string flattens
                // the cause into text and drops the `source` chain, so the underlying
                // russh error is unavailable to anything downcasting or walking sources.
                // `channel_src` is the crate's existing helper for exactly this (G-ERR-05).
                .map_err(|e| {
                    SshCliError::channel_src(
                        format!("failed to open direct-tcpip channel to {remote_host}:{remote_port}"),
                        e,
                    )
                })?;

            Ok(Box::new(channel.into_stream()))
        }

        /// Opens a `direct-streamlocal@openssh.com` channel to a remote Unix socket.
        ///
        /// G-TUN-R03. The OpenSSH extension carries no origin fields, so unlike
        /// `direct-tcpip` there is nothing to advertise about the local side.
        ///
        /// # Errors
        /// [`SshCliError::ChannelFailed`] when the server refuses the extension or
        /// the socket path — servers without it reply with a plain open failure, so
        /// "not supported" and "no such socket" are indistinguishable on the wire.
        pub async fn open_streamlocal_channel(
            &self,
            socket_path: &str,
        ) -> SshCliResult<Box<dyn TunnelChannel>> {
            let channel = self
                .session
                .channel_open_direct_streamlocal(socket_path.to_string())
                .await
                .map_err(|e| {
                    SshCliError::channel_src(
                        format!(
                            "failed to open direct-streamlocal channel to {socket_path}; \
                             the server may not support the OpenSSH extension"
                        ),
                        e,
                    )
                })?;
            Ok(Box::new(channel.into_stream()))
        }

        /// Asks the server to listen on `address:port` and returns the bound port.
        ///
        /// G-TUN-R01. Passing `0` lets the server allocate, and the reply carries
        /// the real port — which is why the return value must be surfaced rather
        /// than echoing the request back to the caller.
        ///
        /// # Errors
        /// [`SshCliError::ChannelFailed`] when the server denies `tcpip-forward`
        /// (commonly `AllowTcpForwarding no` in `sshd_config`).
        pub async fn request_remote_forward(
            &self,
            address: &str,
            port: u16,
        ) -> SshCliResult<u16> {
            let allocated = self
                .session
                .tcpip_forward(address.to_string(), u32::from(port))
                .await
                .map_err(|e| {
                    SshCliError::channel_src(
                        format!(
                            "server refused tcpip-forward for {address}:{port}; \
                             check AllowTcpForwarding / GatewayPorts on the remote sshd"
                        ),
                        e,
                    )
                })?;
            // The server echoes 0 when it had nothing to allocate (a fixed port
            // request); reporting 0 to an agent would be unusable, so keep the
            // requested port in that case.
            let effective = u16::try_from(allocated).unwrap_or(port);
            Ok(if effective == 0 { port } else { effective })
        }

        /// Cancels a remote forward previously granted by the server.
        ///
        /// # Errors
        /// [`SshCliError::ChannelFailed`] when the cancel request is refused.
        pub async fn cancel_remote_forward(&self, address: &str, port: u16) -> SshCliResult<()> {
            self.session
                .cancel_tcpip_forward(address.to_string(), u32::from(port))
                .await
                .map_err(|e| {
                    SshCliError::channel_src(
                        format!("failed to cancel tcpip-forward for {address}:{port}"),
                        e,
                    )
                })
        }

        /// Waits for the next channel the server opens on an active reverse forward.
        ///
        /// Returns `None` once the session ends and no further channels can arrive,
        /// which is what lets the reverse accept loop terminate instead of hanging.
        pub async fn accept_forwarded_channel(&self) -> Option<Box<dyn TunnelChannel>> {
            let mut rx = self.forwarded.lock().await;
            let channel = rx.recv().await?;
            Some(Box::new(channel.into_stream()))
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

        async fn open_streamlocal_channel(
            &self,
            socket_path: &str,
        ) -> Result<Box<dyn TunnelChannel>, SshCliError> {
            Self::open_streamlocal_channel(self, socket_path).await
        }

        async fn request_remote_forward(
            &self,
            address: &str,
            port: u16,
        ) -> Result<u16, SshCliError> {
            Self::request_remote_forward(self, address, port).await
        }

        async fn cancel_remote_forward(&self, address: &str, port: u16) -> Result<(), SshCliError> {
            Self::cancel_remote_forward(self, address, port).await
        }

        async fn accept_forwarded_channel(&self) -> Option<Box<dyn TunnelChannel>> {
            Self::accept_forwarded_channel(self).await
        }

        async fn disconnect(&self) -> Result<(), SshCliError> {
            Self::disconnect(self).await
        }
    }
