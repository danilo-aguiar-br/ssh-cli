// Implementation body for `client_real` (included).

// SPDX-License-Identifier: MIT OR Apache-2.0
// G-ERR-12: real russh client implementation (split from monólito client.rs).
    use super::{
        take_utf8_capped, TunnelChannel, SshClientTrait, ConnectionConfig, ExecutionOutput,
        TransferResult,
    };
    use crate::errors::{SshCliError, SshCliResult};
    use async_trait::async_trait;
    use std::path::Path;
    use std::time::{Duration, Instant};
    use zeroize::Zeroizing;

    // Handler lives in `client_handler` (G-SSH-01/06/09/14).
    pub use crate::ssh::client_handler::ClientHandler;

    /// Active SSH client with an authenticated session.
    pub struct SshClient {
        /// Authenticated SSH session for low-level operations.
        pub session: russh::client::Handle<ClientHandler>,
        cfg: ConnectionConfig,
    }

    impl std::fmt::Debug for SshClient {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("SshClient")
                .field("host", &self.cfg.host)
                .field("port", &self.cfg.port)
                .field("user", &self.cfg.username)
                .field("timeout_ms", &self.cfg.timeout_ms)
                .finish()
        }
    }

    impl SshClient {
        /// Wall-clock timeout (ms) from connection config (CLI/XDG) — G-SFTP-R05.
        #[must_use]
        pub fn timeout_ms(&self) -> u64 {
            self.cfg.timeout_ms.get()
        }
    }

    fn map_exit_status(exit_status: u32) -> i32 {
        i32::try_from(exit_status).unwrap_or(-1)
    }

    fn process_exec_message(
        msg: russh::ChannelMsg,
        stdout_bytes: &mut Vec<u8>,
        stderr_bytes: &mut Vec<u8>,
        exit_code: &mut Option<i32>,
        byte_cap: usize,
        truncated_stdout: &mut bool,
        truncated_stderr: &mut bool,
    ) -> bool {
        use russh::ChannelMsg;

        match msg {
            ChannelMsg::Data { data } => {
                // Resource: bound RAM to byte_cap (max_chars×4, hard 16 MiB) before UTF-8 truncate.
                super::append_capped(stdout_bytes, data.as_ref(), byte_cap, truncated_stdout);
            }
            ChannelMsg::ExtendedData { data, ext } => {
                // ext == 1 → SSH_EXTENDED_DATA_STDERR (RFC 4254 §5.2).
                if ext == 1 {
                    super::append_capped(stderr_bytes, data.as_ref(), byte_cap, truncated_stderr);
                } else {
                    tracing::debug!(ext, "extended data ignored");
                }
            }
            ChannelMsg::ExitStatus { exit_status } => {
                // russh delivers exit status as u32; keep i32 for negative conventions
                // Unix conventions (shells may emit codes as u8 in
                // wait-status; here it is already the application exit code, 0..=255).
                *exit_code = Some(map_exit_status(exit_status));
                // Do NOT return true: wait for Eof/Close after ExitStatus.
            }
            ChannelMsg::ExitSignal {
                signal_name,
                core_dumped,
                error_message,
                ..
            } => {
                tracing::warn!(
                    ?signal_name,
                    core_dumped,
                    %error_message,
                    "remote process terminated by signal"
                );
                // Sem exit_status → mantemos None.
            }
            ChannelMsg::Eof => {
                tracing::debug!("EOF on SSH channel");
            }
            ChannelMsg::Close => {
                tracing::debug!("SSH channel closed by server");
                return true;
            }
            _ => {}
        }

        false
    }

    // SCP wire helpers: see `crate::ssh::scp_wire` (G-COMP-06a).
    // Re-export wire helpers into this module so `real_tests` can `use super::…`.
    // SCP wire helpers: see `crate::ssh::scp_wire` (G-COMP-06a).
    // Re-export wire helpers into this module so `real_tests` can `use super::…`.
    use crate::ssh::scp_wire::{
        apply_local_mode, format_scp_t_line, format_scp_upload_header_with_mode,
        interpret_scp_status, parse_scp_header, parse_scp_t_line, partial_download_path,
        remote_scp_command, scp_mode_from_metadata, scp_read_data, scp_read_until_newline,
        scp_wait_status, system_time_secs, SCP_OK,
    };

    impl SshClient {
        /// Connects and authenticates. The full flow (TCP + handshake + auth) honors
        /// the configuration `timeout_ms`.
        ///
        /// # Errors
        /// - [`SshCliError::InvalidArgument`] if the configuration is invalid.
        /// - [`SshCliError::SshTimeout`] if the total timeout is exceeded.
        /// - [`SshCliError::ConnectionFailed`] on TCP/handshake failures.
        /// - [`SshCliError::HostKeyChanged`] when TOFU rejects a divergent host key.
        /// - [`SshCliError::AuthenticationFailed`] if the server rejects password/key/agent
        ///   (try `--key`, `--use-agent`, `--password-stdin`, or `--key-passphrase-stdin`).
        pub async fn connect(cfg: ConnectionConfig) -> SshCliResult<Self> {
            let auth = crate::ssh::client_connect::connect_authenticated(cfg).await?;
            Ok(Self {
                session: auth.session,
                cfg: auth.cfg,
            })
        }

        /// Runs a remote shell command and captures stdout/stderr in parallel.
        pub async fn run_command(
            &mut self,
            command: &str,
            max_chars: usize,
            stdin_data: Option<Vec<u8>>,
        ) -> SshCliResult<ExecutionOutput> {
            self.run_command_internal(command, max_chars, true, stdin_data)
                .await
        }

        async fn run_command_internal(
            &mut self,
            command: &str,
            max_chars: usize,
            abort_on_timeout: bool,
            stdin_data: Option<Vec<u8>>,
        ) -> SshCliResult<ExecutionOutput> {
            let start = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms.get());

            // Zeroizing: scrub password bytes on drop even if timeout cancels the future.

            // Zeroizing: scrub password bytes on drop even if timeout cancels the future.
            // Zeroizing: scrub password bytes on drop even if timeout cancels the future.
            let stdin_data: Option<Zeroizing<Vec<u8>>> = stdin_data.map(Zeroizing::new);
            let result = tokio::time::timeout(timeout, async {
                let mut channel = self
                    .session
                    .channel_open_session()
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("open session: {e}")))?;

                channel
                    .exec(true, command)
                    .await
                    .map_err(|e| SshCliError::channel_msg(format!("exec: {e}")))?;

                // Senha sudo/su no stdin do channel — nunca na cmdline remota (SEC-001).
                if let Some(ref bytes) = stdin_data {
                    channel
                        .data(bytes.as_slice())
                        .await
                        .map_err(|e| SshCliError::channel_msg(format!("stdin channel: {e}")))?;
                    channel
                        .eof()
                        .await
                        .map_err(|e| SshCliError::channel_msg(format!("eof channel: {e}")))?;
                }
                // Drop Zeroizing early so secrets do not sit through the capture loop.
                drop(stdin_data);

                // Resource: pre-size for typical capture; hard-capped by max_chars×4 / 16 MiB.
                let byte_cap = super::exec_capture_byte_cap(max_chars);
                let initial = byte_cap.min(8 * 1024);
                let mut stdout_bytes: Vec<u8> = Vec::with_capacity(initial);
                let mut stderr_bytes: Vec<u8> = Vec::with_capacity(initial);
                let mut exit_code: Option<i32> = None;
                let mut byte_trunc_stdout = false;
                let mut byte_trunc_stderr = false;

                while let Some(msg) = channel.wait().await {
                    // G-OS-03 / G-SHUT: cooperative cancel on SIGINT/SIGTERM mid-exec.
                    if crate::signals::should_stop() {
                        return Err(SshCliError::Config(
                            "operation cancelled by signal".to_string(),
                        ));
                    }
                    if process_exec_message(
                        msg,
                        &mut stdout_bytes,
                        &mut stderr_bytes,
                        &mut exit_code,
                        byte_cap,
                        &mut byte_trunc_stdout,
                        &mut byte_trunc_stderr,
                    ) {
                        break;
                    }
                }

                Ok::<_, SshCliError>((
                    stdout_bytes,
                    stderr_bytes,
                    exit_code,
                    byte_trunc_stdout,
                    byte_trunc_stderr,
                ))
            })
            .await;

            let (stdout_bytes, stderr_bytes, exit_code, byte_trunc_stdout, byte_trunc_stderr) =
                match result {
                    Ok(Ok(t)) => t,
                    Ok(Err(err)) => return Err(err),
                    Err(_) => {
                        if abort_on_timeout {
                            if let Some(pattern) = crate::ssh::packing::remote_abort_pattern(command)
                            {
                                let abort_cmd = crate::ssh::packing::pack_abort_pkill(&pattern);
                                tracing::warn!(
                                    pattern = %pattern,
                                    "local timeout; attempting best-effort remote abort"
                                );
                                let _ = self.try_remote_abort(&abort_cmd).await;
                            }
                        }
                        return Err(SshCliError::SshTimeout(self.cfg.timeout_ms.get()));
                    }
                };

            // Latency: reuse capture Vec as String on valid UTF-8 (no double-copy).
            let (stdout_truncado, trunc_stdout_chars) =
                take_utf8_capped(stdout_bytes, max_chars);
            let (stderr_truncado, trunc_stderr_chars) =
                take_utf8_capped(stderr_bytes, max_chars);
            let truncated_stdout = trunc_stdout_chars || byte_trunc_stdout;
            let truncated_stderr = trunc_stderr_chars || byte_trunc_stderr;

            let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

            Ok(ExecutionOutput {
                stdout: stdout_truncado,
                stderr: stderr_truncado,
                exit_code,
                truncated_stdout,
                truncated_stderr,
                duration_ms,
            })
        }
    }
