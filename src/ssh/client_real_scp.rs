    impl SshClient {
        /// Uploads a local file to the remote host via SCP (OpenSSH sink protocol).
        ///
        /// One-shot: stream in chunks (without loading the whole file into RAM).
        ///
        /// # Errors
        /// - [`SshCliError::FileNotFound`] if the local file does not exist.
        /// - [`SshCliError::InvalidArgument`] if the local path is not a regular file.
        /// - [`SshCliError::ChannelFailed`] if opening the SCP channel or remote status fails.
        /// - [`SshCliError::SshTimeout`] se exceder o timeout.
        pub async fn upload(
            &self,
            local: &std::path::Path,
            remote: &std::path::Path,
        ) -> SshCliResult<TransferResult> {
            use russh::ChannelMsg;
            use std::time::Instant;
            use tokio::io::AsyncReadExt;

            let local_str = local.display().to_string();

            // G-PAR-41: async metadata so multi-host SCP does not block Tokio workers.
            let metadata = tokio::fs::metadata(local).await.map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    SshCliError::FileNotFound(local_str.clone())
                } else {
                    SshCliError::Io(e)
                }
            })?;

            if metadata.is_dir() {
                return Err(SshCliError::InvalidArgument(crate::i18n::t(
                    crate::i18n::Message::ScpUploadFileOnly,
                )));
            }

            if !metadata.is_file() {
                return Err(SshCliError::InvalidArgument(crate::i18n::t(
                    crate::i18n::Message::ScpUploadFileOnly,
                )));
            }

            let size = metadata.len();
            let mode = scp_mode_from_metadata(&metadata);
            let mtime = metadata.modified().ok().map(system_time_secs).unwrap_or(0);
            let atime = metadata
                .accessed()
                .ok()
                .map(system_time_secs)
                .unwrap_or(mtime);
            let file_name = local.file_name().and_then(|n| n.to_str()).unwrap_or("file");

            let start = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms.get());

            let result =
                tokio::time::timeout(timeout, async {
                    if crate::signals::should_stop() {
                        return Err(SshCliError::InvalidArgument(crate::i18n::t(
                            crate::i18n::Message::OperationCancelled,
                        )));
                    }

                    let mut channel =
                        self.session.channel_open_session().await.map_err(|e| {
                            SshCliError::channel_msg(format!("open SCP session: {e}"))
                        })?;

                    let command = remote_scp_command("-t", remote);
                    channel
                        .exec(true, command.as_str())
                        .await
                        .map_err(|e| SshCliError::channel_msg(format!("exec SCP: {e}")))?;

                    // Remote sink sends ACK (0x00) before accepting the header.
                    scp_wait_status(&mut channel).await?;

                    // Preserve times (line T) antes do header C.
                    let linha_t = format_scp_t_line(mtime, atime);
                    channel
                        .data(linha_t.as_bytes())
                        .await
                        .map_err(|e| SshCliError::channel_msg(format!("enviar linha T SCP: {e}")))?;
                    scp_wait_status(&mut channel).await?;

                    let header = format_scp_upload_header_with_mode(mode, size, file_name);
                    channel
                        .data(header.as_bytes())
                        .await
                        .map_err(|e| SshCliError::channel_msg(format!("enviar header SCP: {e}")))?;
                    scp_wait_status(&mut channel).await?;

                    // SCP-018 + latency: async disk read so the runtime worker is not
                    // blocked on synchronous `read(2)` mid-transfer.
                    let mut file = tokio::fs::File::open(local).await.map_err(SshCliError::Io)?;
                    let mut buf = vec![0u8; 32_768];
                    loop {
                        if crate::signals::should_stop() {
                            return Err(SshCliError::InvalidArgument(crate::i18n::t(
                                crate::i18n::Message::OperationCancelled,
                            )));
                        }
                        let n = file.read(&mut buf).await.map_err(SshCliError::Io)?;
                        if n == 0 {
                            break;
                        }
                        channel.data(&buf[..n]).await.map_err(|e| {
                            SshCliError::channel_msg(format!("enviar bloco SCP: {e}"))
                        })?;
                    }

                    // File terminator = byte 0x00 (not empty data).
                    channel
                        .data([SCP_OK].as_slice())
                        .await
                        .map_err(|e| SshCliError::channel_msg(format!("enviar EOF SCP: {e}")))?;
                    scp_wait_status(&mut channel).await?;

                    let _ = channel.eof().await;
                    while let Some(msg) = channel.wait().await {
                        if let ChannelMsg::Close = msg {
                            break;
                        }
                    }

                    Ok::<_, SshCliError>(())
                })
                .await;

            result.map_err(|_| SshCliError::SshTimeout(self.cfg.timeout_ms.get()))??;

            let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

            Ok(TransferResult {
                bytes_transferred: size,
                duration_ms,
            })
        }

        /// Downloads a remote file to the local path via SCP (OpenSSH source protocol).
        ///
        /// Writes to `{local}.ssh-cli.partial` and renames atomically (SCP-022).
        ///
        /// # Errors
        /// - [`SshCliError::Io`] if the local file cannot be written.
        /// - [`SshCliError::ChannelFailed`] if opening the SCP channel or remote status fails.
        /// - [`SshCliError::SshTimeout`] se exceder o timeout.
        pub async fn download(
            &self,
            remote: &std::path::Path,
            local: &std::path::Path,
        ) -> SshCliResult<TransferResult> {
            use russh::ChannelMsg;
            use std::time::{Duration as StdDuration, Instant, UNIX_EPOCH};
            use tokio::io::AsyncWriteExt;

            if local.is_dir() {
                return Err(SshCliError::InvalidArgument(crate::i18n::t(
                    crate::i18n::Message::ScpDownloadLocalNotDirectory,
                )));
            }

            let start = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms.get());
            let partial = partial_download_path(local);

            let result = tokio::time::timeout(timeout, async {
                if crate::signals::should_stop() {
                    return Err(SshCliError::InvalidArgument(crate::i18n::t(
                        crate::i18n::Message::OperationCancelled,
                    )));
                }

                let mut channel = self
                    .session
                    .channel_open_session()
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("open SCP session: {e}")))?;

                let command = remote_scp_command("-f", remote);
                channel
                    .exec(true, command.as_str())
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("exec SCP: {e}")))?;

                // Remote source only sends the header after the local sink's initial ACK.
                channel
                    .data([SCP_OK].as_slice())
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("enviar ack inicial: {e}")))?;

                let mut times: Option<(u64, u64)> = None;
                let mut header_bytes = scp_read_until_newline(&mut channel).await?;
                // Erro remoto: status 1/2 no primeiro byte.
                if !header_bytes.is_empty() && matches!(header_bytes[0], 1 | 2) {
                    interpret_scp_status(&header_bytes)?;
                }
                let mut header = String::from_utf8_lossy(&header_bytes).into_owned();
                // Linha T opcional (preserve times).
                if header.trim_start().starts_with('T') {
                    times = Some(parse_scp_t_line(&header)?);
                    channel
                        .data([SCP_OK].as_slice())
                        .await
                        .map_err(|e| SshCliError::channel_msg(format!("enviar ack T: {e}")))?;
                    header_bytes = scp_read_until_newline(&mut channel).await?;
                    if !header_bytes.is_empty() && matches!(header_bytes[0], 1 | 2) {
                        interpret_scp_status(&header_bytes)?;
                    }
                    header = String::from_utf8_lossy(&header_bytes).into_owned();
                }
                let (mode_remoto, size) = parse_scp_header(&header)?;

                channel
                    .data([SCP_OK].as_slice())
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("enviar ack header: {e}")))?;

                if let Some(parent_dir) = local.parent() {
                    if !parent_dir.as_os_str().is_empty() {
                        tokio::fs::create_dir_all(parent_dir)
                            .await
                            .map_err(SshCliError::Io)?;
                    }
                }

                // SCP-022 + latency: async create/write so workers are not blocked.
                let mut file = tokio::fs::File::create(&partial)
                    .await
                    .map_err(SshCliError::Io)?;
                let mut recebidos: u64 = 0;
                // Resource: reuse pending window (~upload chunk size); stream to disk, not full file.
                let mut pending: Vec<u8> = Vec::with_capacity(32_768);

                while recebidos < size {
                    if crate::signals::should_stop() {
                        return Err(SshCliError::InvalidArgument(crate::i18n::t(
                            crate::i18n::Message::OperationCancelled,
                        )));
                    }
                    if pending.is_empty() {
                        let chunk = scp_read_data(&mut channel).await?;
                        pending.extend_from_slice(&chunk);
                    }
                    // G-CLOSE-03: TryFrom for remaining bytes (no silent truncate on huge sizes).
                    let need = usize::try_from(size.saturating_sub(recebidos)).unwrap_or(usize::MAX);
                    let use_n = need.min(pending.len());
                    file.write_all(&pending[..use_n])
                        .await
                        .map_err(SshCliError::Io)?;
                    recebidos = recebidos.saturating_add(u64::try_from(use_n).unwrap_or(u64::MAX));
                    pending.drain(..use_n);
                }

                // After payload, source sends final 0x00 (may already be in `pending`).
                if pending.is_empty() {
                    match scp_read_data(&mut channel).await {
                        Ok(trail) => pending.extend_from_slice(&trail),
                        Err(_) if recebidos == size => {}
                        Err(e) => return Err(e),
                    }
                }
                if pending.first() == Some(&SCP_OK) {
                    pending.remove(0);
                } else if !pending.is_empty() {
                    return Err(SshCliError::channel_msg(format!(
                        "unexpected SCP terminator after payload (0x{:02x})",
                        pending[0]
                    )));
                }

                file.flush().await.map_err(SshCliError::Io)?;
                let _ = file.sync_data().await;
                drop(file);

                channel
                    .data([SCP_OK].as_slice())
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("enviar ack final: {e}")))?;

                let _ = channel.eof().await;
                while let Some(msg) = channel.wait().await {
                    if matches!(msg, ChannelMsg::Close) {
                        break;
                    }
                }

                // SCP-022b: apply mode/times on partial BEFORE atomic rename.
                // So metadata failure does not leave `local` with partial success content.
                // G-PAR-50: async permissions; FileTimes/parent fsync via spawn_blocking.
                apply_local_mode(&partial, mode_remoto).await?;
                if let Some((mtime, atime)) = times {
                    let partial_c = partial.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        let mtime_st = UNIX_EPOCH + StdDuration::from_secs(mtime);
                        let atime_st = UNIX_EPOCH + StdDuration::from_secs(atime);
                        let ft = std::fs::FileTimes::new()
                            .set_modified(mtime_st)
                            .set_accessed(atime_st);
                        if let Ok(f) = std::fs::File::options().write(true).open(&partial_c) {
                            let _ = f.set_times(ft);
                        }
                    })
                    .await;
                }

                tokio::fs::rename(&partial, local)
                    .await
                    .map_err(SshCliError::Io)?;
                // Atomic write: fsync parent_dir after rename (best-effort, blocking pool).
                if let Some(parent_dir) = local.parent() {
                    if !parent_dir.as_os_str().is_empty() {
                        let parent_dir = parent_dir.to_path_buf();
                        let _ = tokio::task::spawn_blocking(move || {
                            if let Ok(dir) = std::fs::File::open(&parent_dir) {
                                let _ = dir.sync_all();
                            }
                        })
                        .await;
                    }
                }

                Ok::<_, SshCliError>(recebidos)
            })
            .await;

            match result {
                Ok(Ok(recebidos)) => {
                    let duration_ms =
                        u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                    Ok(TransferResult {
                        bytes_transferred: recebidos,
                        duration_ms,
                    })
                }
                Ok(Err(e)) => {
                    let _ = tokio::fs::remove_file(&partial).await;
                    // If rename already happened and something failed later (best-effort fsync does not fail),
                    // still remove partial; `local` only exists after a successful rename.
                    Err(e)
                }
                Err(_) => {
                    let _ = tokio::fs::remove_file(&partial).await;
                    Err(SshCliError::SshTimeout(self.cfg.timeout_ms.get()))
                }
            }
        }

        /// Best-effort remote abort: reconnects with a short timeout and runs pkill.
        async fn try_remote_abort(&self, abort_cmd: &str) -> SshCliResult<()> {
            // Inline implementation (without calling run_command_internal) avoids
            // async recursion detected by the compiler.
            let mut cfg_abort = self.cfg.clone();
            cfg_abort.timeout_ms = crate::domain::TimeoutMs::try_new(cfg_abort.timeout_ms.get().clamp(3_000, 10_000)).expect("clamp in range");
            let abort_client = match Self::connect(cfg_abort).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::debug!(err = %e, "remote abort could not reconnect");
                    return Err(e);
                }
            };
            let timeout = Duration::from_millis(abort_client.cfg.timeout_ms.get());
            let _ = tokio::time::timeout(timeout, async {
                let mut channel = abort_client
                    .session
                    .channel_open_session()
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("abort channel: {e}")))?;
                channel
                    .exec(true, abort_cmd)
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("abort exec: {e}")))?;
                while let Some(msg) = channel.wait().await {
                    if matches!(msg, russh::ChannelMsg::Close) {
                        break;
                    }
                }
                Ok::<(), SshCliError>(())
            })
            .await;
            let _ = abort_client.disconnect().await;
            Ok(())
        }
    }
