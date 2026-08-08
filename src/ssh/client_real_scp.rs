    impl SshClient {
        /// Uploads a local file to the remote host via SCP (OpenSSH sink protocol).
        ///
        /// One-shot: stream in chunks (without loading the whole file into RAM).
        ///
        /// # Errors
        /// - [`SshCliError::FileNotFound`] if the local file does not exist.
        /// - [`SshCliError::InvalidArgument`] if the local path is not a regular file.
        /// - [`SshCliError::ChannelFailed`] if opening the SCP channel or remote status fails.
        /// - [`SshCliError::SshTimeout`] if the deadline expires.
        pub async fn upload(
            &self,
            local: &std::path::Path,
            remote: &std::path::Path,
        ) -> SshCliResult<TransferResult> {
            use russh::ChannelMsg;
            use std::time::Instant;
            use tokio::io::AsyncReadExt;

            // Streaming window for the SCP payload (never load the whole file).
            use crate::constants::SCP_IO_CHUNK;

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

                    // Preserve times (T line) before the C header.
                    let t_line = format_scp_t_line(mtime, atime);
                    channel
                        .data(t_line.as_bytes())
                        .await
                        .map_err(|e| SshCliError::channel_msg(format!("send SCP T line: {e}")))?;
                    scp_wait_status(&mut channel).await?;

                    let header = format_scp_upload_header_with_mode(mode, size, file_name);
                    channel
                        .data(header.as_bytes())
                        .await
                        .map_err(|e| SshCliError::channel_msg(format!("send SCP header: {e}")))?;
                    scp_wait_status(&mut channel).await?;

                    // SCP-018 + latency: async disk read so the runtime worker is not
                    // blocked on synchronous `read(2)` mid-transfer.
                    let mut file = tokio::fs::File::open(local).await.map_err(SshCliError::Io)?;
                    let mut buf = vec![0u8; SCP_IO_CHUNK];
                    // B3 (TOCTOU): `size` was read by `metadata` *before* this open. A
                    // concurrent writer can grow or shrink the file in between, and the
                    // header already promised `size` bytes. Clamp every read to the
                    // announced remainder so a grown file cannot spill extra bytes into
                    // the next protocol frame, and refuse to finish short.
                    let mut sent: u64 = 0;
                    while sent < size {
                        if crate::signals::should_stop() {
                            return Err(SshCliError::InvalidArgument(crate::i18n::t(
                                crate::i18n::Message::OperationCancelled,
                            )));
                        }
                        let remaining =
                            usize::try_from(size.saturating_sub(sent)).unwrap_or(usize::MAX);
                        let window = remaining.min(buf.len());
                        let n = file
                            .read(&mut buf[..window])
                            .await
                            .map_err(SshCliError::Io)?;
                        if n == 0 {
                            break;
                        }
                        channel.data(&buf[..n]).await.map_err(|e| {
                            SshCliError::channel_msg(format!("send SCP payload block: {e}"))
                        })?;
                        sent = sent.saturating_add(u64::try_from(n).unwrap_or(u64::MAX));
                    }
                    if sent != size {
                        // Truncated mid-transfer: the sink is still waiting for
                        // `size - sent` bytes, so the stream is desynchronised. Fail
                        // loudly instead of letting the terminator land inside payload.
                        return Err(SshCliError::channel_msg(format!(
                            "local file shrank during SCP upload: announced {size} bytes, read {sent}"
                        )));
                    }

                    // File terminator = byte 0x00 (not empty data).
                    channel
                        .data([SCP_OK].as_slice())
                        .await
                        .map_err(|e| SshCliError::channel_msg(format!("send SCP EOF: {e}")))?;
                    scp_wait_status(&mut channel).await?;

                    let _ = channel.eof().await;
                    while let Some(msg) = channel.wait().await {
                        if let ChannelMsg::Close = msg {
                            break;
                        }
                    }

                    Ok::<_, SshCliError>(sent)
                })
                .await;

            // B2: report what actually crossed the wire, not what `metadata` promised.
            let sent = result.map_err(|_| SshCliError::SshTimeout(self.cfg.timeout_ms.get()))??;

            let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

            Ok(TransferResult {
                bytes_transferred: sent,
                duration_ms,
                ..Default::default()
            })
        }

        /// Downloads a remote file to the local path via SCP (OpenSSH source protocol).
        ///
        /// Writes to `{local}.ssh-cli.partial` and renames atomically (SCP-022).
        ///
        /// # Errors
        /// - [`SshCliError::Io`] if the local file cannot be written.
        /// - [`SshCliError::ChannelFailed`] if opening the SCP channel or remote status fails.
        /// - [`SshCliError::SshTimeout`] if the deadline expires.
        pub async fn download(
            &self,
            remote: &std::path::Path,
            local: &std::path::Path,
        ) -> SshCliResult<TransferResult> {
            use russh::ChannelMsg;
            use std::time::{Duration as StdDuration, Instant, UNIX_EPOCH};
            use tokio::io::AsyncWriteExt;

            // Streaming window for the SCP payload (never load the whole file).
            use crate::constants::SCP_IO_CHUNK;

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
                    .map_err(|e| SshCliError::channel_msg(format!("send initial SCP ack: {e}")))?;

                // B2: bytes the source coalesced after a header line. Reused as the
                // payload window below so nothing read here is ever discarded.
                let mut pending: Vec<u8> = Vec::with_capacity(SCP_IO_CHUNK);

                let mut times: Option<(u64, u64)> = None;
                let mut header_bytes = scp_read_until_newline(&mut channel, &mut pending).await?;
                // Remote error: status 1/2 in the first byte.
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
                        .map_err(|e| SshCliError::channel_msg(format!("send T-line ack: {e}")))?;
                    header_bytes = scp_read_until_newline(&mut channel, &mut pending).await?;
                    if !header_bytes.is_empty() && matches!(header_bytes[0], 1 | 2) {
                        interpret_scp_status(&header_bytes)?;
                    }
                    header = String::from_utf8_lossy(&header_bytes).into_owned();
                }
                let (remote_mode, size) = parse_scp_header(&header)?;

                channel
                    .data([SCP_OK].as_slice())
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("send header ack: {e}")))?;

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
                let mut received: u64 = 0;

                while received < size {
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
                    let need = usize::try_from(size.saturating_sub(received)).unwrap_or(usize::MAX);
                    let use_n = need.min(pending.len());
                    file.write_all(&pending[..use_n])
                        .await
                        .map_err(SshCliError::Io)?;
                    received = received.saturating_add(u64::try_from(use_n).unwrap_or(u64::MAX));
                    pending.drain(..use_n);
                }

                // B2: the payload loop only exits when `received == size`, so the old
                // `Err(_) if received == size` arm swallowed *every* terminator error —
                // including the channel dying mid-file. The terminator is mandatory in
                // the SCP source protocol, so a failure here means the transfer is not
                // provably complete and must surface.
                if pending.is_empty() {
                    let trail = scp_read_data(&mut channel).await?;
                    pending.extend_from_slice(&trail);
                }
                if received != size {
                    return Err(SshCliError::channel_msg(format!(
                        "truncated SCP download: announced {size} bytes, received {received}"
                    )));
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
                // G9: durability barrier — do not report success if fsync fails.
                file.sync_data().await.map_err(SshCliError::Io)?;
                drop(file);

                channel
                    .data([SCP_OK].as_slice())
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("send final ack: {e}")))?;

                let _ = channel.eof().await;
                while let Some(msg) = channel.wait().await {
                    if matches!(msg, ChannelMsg::Close) {
                        break;
                    }
                }

                // SCP-022b: apply mode/times on partial BEFORE atomic rename.
                // So metadata failure does not leave `local` with partial success content.
                // G-PAR-50: async permissions; FileTimes/parent fsync via spawn_blocking.
                apply_local_mode(&partial, remote_mode).await?;
                // G-SCP-R01: the outcome now travels back instead of being dropped. Both
                // the `open` and the `set_times` used to be swallowed by `let _ =`, so
                // three distinct failures — unsupported filesystem, missing permission,
                // incompatible open mode — were indistinguishable from success.
                let mut mtime_preserved = true;
                if let Some((mtime, atime)) = times {
                    let partial_c = partial.clone();
                    let stamped = tokio::task::spawn_blocking(move || {
                        let mtime_st = UNIX_EPOCH + StdDuration::from_secs(mtime);
                        let atime_st = UNIX_EPOCH + StdDuration::from_secs(atime);
                        let ft = std::fs::FileTimes::new()
                            .set_modified(mtime_st)
                            .set_accessed(atime_st);
                        let f = std::fs::File::options().write(true).open(&partial_c)?;
                        f.set_times(ft)
                    })
                    .await;
                    mtime_preserved = match stamped {
                        Ok(Ok(())) => true,
                        Ok(Err(e)) => {
                            // Deliberately not fatal: the payload is byte-exact and
                            // fsynced. Refusing the transfer over a timestamp would
                            // break every download onto a filesystem that cannot
                            // represent one.
                            tracing::debug!(
                                err = %e,
                                path = %partial.display(),
                                "scp: mtime not preserved"
                            );
                            false
                        }
                        Err(e) => {
                            tracing::warn!(err = %e, "scp: mtime task failed to join");
                            false
                        }
                    };
                }

                tokio::fs::rename(&partial, local)
                    .await
                    .map_err(SshCliError::Io)?;
                // G-SCP-R02: the rename is atomic, but the directory entry is not on
                // stable storage until the parent is flushed. Still best-effort — many
                // filesystems refuse `sync_all` on a directory handle — but the caller
                // is now told, instead of receiving an unqualified exit 0.
                let mut durable = true;
                if let Some(parent_dir) = local.parent() {
                    if !parent_dir.as_os_str().is_empty() {
                        let parent_dir = parent_dir.to_path_buf();
                        let flushed = tokio::task::spawn_blocking(move || {
                            std::fs::File::open(&parent_dir)?.sync_all()
                        })
                        .await;
                        durable = match flushed {
                            Ok(Ok(())) => true,
                            Ok(Err(e)) => {
                                tracing::warn!(err = %e, "scp: parent dir fsync failed");
                                false
                            }
                            Err(e) => {
                                tracing::warn!(err = %e, "scp: fsync task failed to join");
                                false
                            }
                        };
                    }
                }

                Ok::<_, SshCliError>((received, mtime_preserved, durable))
            })
            .await;

            match result {
                Ok(Ok((received, mtime_preserved, durable))) => {
                    let duration_ms =
                        u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                    Ok(TransferResult {
                        bytes_transferred: received,
                        duration_ms,
                        mtime_preserved,
                        durable,
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
