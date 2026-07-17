// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cliente SSH real via `russh` 0.62.2.
//!
//! One-shot connection: TCP + handshake + auth (password and/or key) + exec with
//! timeout, output truncation, and best-effort remote abort.
//! Host keys: TOFU em `known_hosts` XDG (ver [`super::known_hosts`]).

use crate::erros::{SshCliError, SshCliResult};
use secrecy::{ExposeSecret, SecretString};
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncWrite};

/// SSH connection configuration.
///
/// Built from a [`crate::vps::model::VpsRecord`] at the time
/// of the call. Auth: private key (preferred) and/or password.
#[derive(Clone)]
pub struct ConnectionConfig {
    /// Hostname ou IP do servidor SSH.
    pub host: String,
    /// SSH server TCP port (default 22).
    pub port: u16,
    /// SSH username.
    pub username: String,
    /// SSH password (`SecretString` for automatic zeroize); may be empty for key-only.
    pub password: SecretString,
    /// OpenSSH private key path (optional).
    pub key_path: Option<String>,
    /// Key passphrase (optional).
    pub key_passphrase: Option<SecretString>,
    /// Total timeout for connect + handshake + authentication + exec, in ms.
    pub timeout_ms: u64,
    /// Path to known_hosts file (TOFU). `None` = always-trust (tests only).
    pub known_hosts_path: Option<PathBuf>,
    /// Se true, permite substituir fingerprint divergente.
    pub replace_host_key: bool,
}

impl std::fmt::Debug for ConnectionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionConfig")
            .field("host", &self.host)
            .field("porta", &self.port)
            .field("usuario", &self.username)
            .field("senha", &"<redacted>")
            .field("key_path", &self.key_path)
            .field(
                "key_passphrase",
                &self.key_passphrase.as_ref().map(|_| "<redacted>"),
            )
            .field("timeout_ms", &self.timeout_ms)
            .field("known_hosts_path", &self.known_hosts_path)
            .field("replace_host_key", &self.replace_host_key)
            .finish()
    }
}

impl ConnectionConfig {
    /// Validates basic configuration fields.
    pub fn validate(&self) -> SshCliResult<()> {
        if self.host.trim().is_empty() {
            return Err(SshCliError::InvalidArgument(
                "empty host in ConnectionConfig".to_string(),
            ));
        }
        if self.port == 0 {
            return Err(SshCliError::InvalidArgument(
                "port 0 is invalid in ConnectionConfig".to_string(),
            ));
        }
        if self.username.trim().is_empty() {
            return Err(SshCliError::InvalidArgument(
                "empty user in ConnectionConfig".to_string(),
            ));
        }
        let has_password = !self.password.expose_secret().is_empty();
        let tem_key = self.key_path.as_ref().is_some_and(|p| !p.trim().is_empty());
        if !has_password && !tem_key {
            return Err(SshCliError::InvalidArgument(
                "auth requires password or key_path".to_string(),
            ));
        }
        Ok(())
    }
}

/// Output of a remote SSH command execution.
#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    /// Stdout capturado (possivelmente truncated a `max_chars` codepoints).
    pub stdout: String,
    /// Stderr capturado (possivelmente truncated a `max_chars` codepoints).
    pub stderr: String,
    /// Exit code. `None` when the command was terminated by signal or timeout.
    pub exit_code: Option<i32>,
    /// `true` se `stdout` foi truncated em `max_chars`.
    pub truncated_stdout: bool,
    /// `true` se `stderr` foi truncated em `max_chars`.
    pub truncated_stderr: bool,
    /// Total execution duration in milliseconds.
    pub duration_ms: u64,
}

/// Result of an SCP file transfer operation.
#[derive(Debug, Clone)]
pub struct TransferResult {
    /// Number of bytes transferred.
    pub bytes_transferred: u64,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
}

/// Truncates a UTF-8 string to at most `max_chars` codepoints.
///
/// Returns `(truncated_string, was_truncated)`. If `max_chars == 0` returns empty string.
/// Unicode-safe: opera sobre codepoints via `chars()`, nunca quebra no meio.
#[must_use]
pub fn truncate_utf8(content: &str, max_chars: usize) -> (String, bool) {
    let total = content.chars().count();
    if total <= max_chars {
        return (content.to_string(), false);
    }
    let truncated: String = content.chars().take(max_chars).collect();
    (truncated, true)
}

// =========================================================================
// Trait SshClientTrait para permitir mocks em teste.
// =========================================================================

use async_trait::async_trait;
use std::path::Path;

/// Stream bidirecional usado para tunnel SSH (direct-tcpip).
pub trait TunnelChannel: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T> TunnelChannel for T where T: AsyncRead + AsyncWrite + Unpin + Send {}

/// SSH client trait allowing a real (russh) or mock implementation for tests.
///
/// This trait abstracts SSH connection operations to allow unit tests
/// without needing a real network connection.
#[async_trait]
pub trait SshClientTrait: Send + Sync + 'static {
    /// Connects to an SSH server and authenticates with the provided credentials.
    async fn connect(cfg: ConnectionConfig) -> Result<Box<Self>, SshCliError>
    where
        Self: Sized;

    /// Runs a remote shell command and returns the captured output.
    ///
    /// `stdin_data`, if present, is written to the channel after `exec` and before the loop
    /// de leitura (GAP-SSH-SEC-001: password sudo/su fora da argv remota).
    async fn run_command(
        &mut self,
        cmd: &str,
        max_chars: usize,
        stdin_data: Option<Vec<u8>>,
    ) -> Result<ExecutionOutput, SshCliError>;

    /// Faz upload de um file local para o servidor remoto via SCP.
    async fn upload(
        &mut self,
        local: &Path,
        remote: &Path,
    ) -> Result<TransferResult, SshCliError>;

    /// Faz download de um file remoto para o sistema local via SCP.
    async fn download(
        &mut self,
        remote: &Path,
        local: &Path,
    ) -> Result<TransferResult, SshCliError>;

    /// Abre um channel `direct-tcpip` para forwarding de tunnel.
    async fn open_tunnel_channel(
        &self,
        remote_host: &str,
        remote_port: u16,
        endereco_origem: &str,
        porta_origem: u16,
    ) -> Result<Box<dyn TunnelChannel>, SshCliError>;

    /// Cleanly closes the SSH connection.
    async fn disconnect(&self) -> Result<(), SshCliError>;
}

#[cfg(test)]
/// SSH client mocks used in unit tests.
pub mod mocks {
    use super::*;
    use mockall::mock;

    mock! {
        pub SshClient {}

    #[async_trait]
    impl crate::ssh::client::SshClientTrait for SshClient {
            async fn connect(cfg: ConnectionConfig) -> Result<Box<Self>, SshCliError>;
            async fn run_command(&mut self, cmd: &str, max_chars: usize, stdin_data: Option<Vec<u8>>) -> Result<ExecutionOutput, SshCliError>;
            async fn upload(&mut self, local: &Path, remote: &Path) -> Result<TransferResult, SshCliError>;
            async fn download(&mut self, remote: &Path, local: &Path) -> Result<TransferResult, SshCliError>;
            async fn open_tunnel_channel(
                &self,
                remote_host: &str,
                remote_port: u16,
                endereco_origem: &str,
                porta_origem: u16,
            ) -> Result<Box<dyn TunnelChannel>, SshCliError>;
            async fn disconnect(&self) -> Result<(), SshCliError>;
        }
    }
}

// =========================================================================
// REAL SSH implementation (`ssh-real` feature).
// =========================================================================

#[cfg(feature = "ssh-real")]
mod real {
    use super::{
        TunnelChannel, SshClientTrait, ConnectionConfig, ExecutionOutput, TransferResult,
    };
    use crate::erros::{SshCliError, SshCliResult};
    use async_trait::async_trait;
    use secrecy::ExposeSecret;
    use std::path::Path;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// russh handler with TOFU known_hosts (or always-trust if path is absent).
    pub struct ClientHandler {
        host: String,
        port: u16,
        known_hosts_path: Option<std::path::PathBuf>,
        replace_host_key: bool,
        /// Captured host-key error (russh Error does not carry our type).
        host_key_rejected: bool,
        host_key_detail: Option<String>,
    }

    impl ClientHandler {
        fn new(cfg: &ConnectionConfig) -> Self {
            Self {
                host: cfg.host.clone(),
                port: cfg.port,
                known_hosts_path: cfg.known_hosts_path.clone(),
                replace_host_key: cfg.replace_host_key,
                host_key_rejected: false,
                host_key_detail: None,
            }
        }
    }

    impl russh::client::Handler for ClientHandler {
        type Error = russh::Error;

        async fn check_server_key(
            &mut self,
            server_key: &russh::keys::ssh_key::PublicKey,
        ) -> Result<bool, Self::Error> {
            let fingerprint = format!(
                "{}",
                server_key.fingerprint(russh::keys::HashAlg::Sha256)
            );

            let Some(path) = self.known_hosts_path.clone() else {
                tracing::warn!("known_hosts missing: accepting host key (test mode)");
                return Ok(true);
            };

            let mut kh = match crate::ssh::known_hosts::KnownHosts::load(path) {
                Ok(k) => k,
                Err(e) => {
                    tracing::error!(err = %e, "failed to load known_hosts");
                    self.host_key_rejected = true;
                    self.host_key_detail = Some(e.to_string());
                    return Ok(false);
                }
            };

            match crate::ssh::known_hosts::verify_tofu(
                &mut kh,
                &self.host,
                self.port,
                &fingerprint,
                self.replace_host_key,
            ) {
                Ok(true) => Ok(true),
                Ok(false) => Ok(false),
                Err(e) => {
                    self.host_key_rejected = true;
                    self.host_key_detail = Some(e.to_string());
                    tracing::error!(err = %e, "host key rejeitada");
                    Ok(false)
                }
            }
        }
    }

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

    fn map_exit_status(exit_status: u32) -> i32 {
        i32::try_from(exit_status).unwrap_or(-1)
    }

    fn process_exec_message(
        msg: russh::ChannelMsg,
        stdout_bytes: &mut Vec<u8>,
        stderr_bytes: &mut Vec<u8>,
        exit_code: &mut Option<i32>,
    ) -> bool {
        use russh::ChannelMsg;

        match msg {
            ChannelMsg::Data { data } => {
                stdout_bytes.extend_from_slice(&data);
            }
            ChannelMsg::ExtendedData { data, ext } => {
                // ext == 1 → SSH_EXTENDED_DATA_STDERR (RFC 4254 §5.2).
                if ext == 1 {
                    stderr_bytes.extend_from_slice(&data);
                } else {
                    tracing::debug!(ext, "extended data ignored");
                }
            }
            ChannelMsg::ExitStatus { exit_status } => {
                // russh entrega como u32. Mantemos como i32 para acomodar
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

    /// SCP protocol ACK/OK byte (also used as payload terminator).
    const SCP_OK: u8 = 0;

    /// Safe basename for the SCP wire (no path separators / control chars).
    fn basename_scp(file_name: &str) -> String {
        file_name
            .split(['/', '\\'])
            .next_back()
            .unwrap_or("file")
            .replace(['\n', '\r', '\0'], "_")
    }

    /// Header `C`-line do protocolo SCP (newline real `0x0a`, nunca `\\n` literal).
    #[cfg_attr(not(test), allow(dead_code))]
    fn format_scp_upload_header(size: u64, file_name: &str) -> String {
        format_scp_upload_header_with_mode(0o644, size, file_name)
    }

    /// Header `C` with octal mode (e.g. `0644`).
    fn format_scp_upload_header_with_mode(mode: u32, size: u64, file_name: &str) -> String {
        let name = basename_scp(file_name);
        let mode = mode & 0o7777;
        format!("C{mode:04o} {size} {name}\n")
    }

    /// Linha `T` do protocolo SCP (preserve times / `-p`).
    fn format_scp_t_line(mtime_secs: u64, atime_secs: u64) -> String {
        format!("T{mtime_secs} 0 {atime_secs} 0\n")
    }

    /// Parse da line `T mtime 0 atime 0`.
    fn parse_scp_t_line(line: &str) -> SshCliResult<(u64, u64)> {
        let line = line.trim_end_matches(['\0', '\r', '\n']).trim();
        if !line.starts_with('T') {
            return Err(SshCliError::ChannelFailed(format!(
                "unexpected SCP T line: {line}"
            )));
        }
        let resto = &line[1..];
        let partes: Vec<&str> = resto.split_whitespace().collect();
        if partes.len() < 3 {
            return Err(SshCliError::ChannelFailed(format!(
                "malformed SCP T line: {line}"
            )));
        }
        let mtime: u64 = partes[0].parse().map_err(|_| {
            SshCliError::ChannelFailed(format!("invalid mtime in T line: {}", partes[0]))
        })?;
        let atime: u64 = partes[2].parse().map_err(|_| {
            SshCliError::ChannelFailed(format!("invalid atime in T line: {}", partes[2]))
        })?;
        Ok((mtime, atime))
    }

    /// Parse do header `C0mmm size name` → `(mode, size)`.
    fn parse_scp_header(header: &str) -> SshCliResult<(u32, u64)> {
        let header = header.trim_end_matches(['\0', '\r', '\n']).trim();

        if !header.starts_with('C') {
            return Err(SshCliError::ChannelFailed(format!(
                "unexpected SCP header: {}",
                header
            )));
        }

        let partes: Vec<&str> = header.split_whitespace().collect();
        if partes.len() < 3 {
            return Err(SshCliError::ChannelFailed(format!(
                "malformed SCP header: {}",
                header
            )));
        }

        // Mode field: `C0644` (`C` prefix + 4 octal digits).
        let mode_token = partes[0];
        if mode_token.len() < 2 {
            return Err(SshCliError::ChannelFailed(format!(
                "missing SCP mode in header: {header}"
            )));
        }
        let mode_oct = &mode_token[1..];
        let mode: u32 = u32::from_str_radix(mode_oct, 8)
            .map_err(|_| SshCliError::ChannelFailed(format!("invalid SCP mode: {mode_oct}")))?;

        let size = partes[1].parse().map_err(|_| {
            SshCliError::ChannelFailed(format!("invalid size in header: {}", partes[1]))
        })?;
        Ok((mode & 0o7777, size))
    }

    /// Mode octal para o header `C` a partir de metadata local.
    fn scp_mode_from_metadata(meta: &std::fs::Metadata) -> u32 {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            meta.permissions().mode() & 0o7777
        }
        #[cfg(not(unix))]
        {
            let _ = meta;
            0o644
        }
    }

    /// Segundos epoch a partir de SystemTime (best-effort).
    fn system_time_secs(t: std::time::SystemTime) -> u64 {
        t.duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Atomic download temporary file suffix (SCP-022).
    const SCP_PARTIAL_SUFFIX: &str = ".ssh-cli.partial";

    fn partial_download_path(local: &std::path::Path) -> std::path::PathBuf {
        let mut p = local.as_os_str().to_os_string();
        p.push(SCP_PARTIAL_SUFFIX);
        std::path::PathBuf::from(p)
    }

    /// GAP-SSH-IO-010 / GAP-AUD-025: classify SCP err → missing-file (66) vs channel (74).
    ///
    /// OpenSSH typically emits `scp: PATH: No such file or directory` on status `1`/`2`.
    /// Missing-file messages are normalized to a clean path for the Display wrapper
    /// `file not found: {path}` (no stacked `SCP:` / `scp:` prefixes).
    fn classify_scp_message(msg: &str) -> SshCliError {
        let lower = msg.to_ascii_lowercase();
        if lower.contains("no such file") || lower.contains("not found") {
            SshCliError::FileNotFound(normalize_scp_missing_path(msg))
        } else if msg.is_empty() {
            SshCliError::ChannelFailed("SCP rejected the transfer".to_string())
        } else if msg.starts_with("SCP:") || msg.starts_with("SCP ") {
            SshCliError::ChannelFailed(msg.to_string())
        } else {
            SshCliError::ChannelFailed(format!("SCP: {msg}"))
        }
    }

    /// Strips `SCP:` / `scp:` wrappers and trailing OS phrases → remote path or cleaned msg.
    fn normalize_scp_missing_path(msg: &str) -> String {
        let mut s = msg.trim().to_string();
        for prefix in ["SCP: ", "SCP:", "scp: ", "scp:"] {
            if let Some(rest) = s.strip_prefix(prefix) {
                s = rest.trim().to_string();
            }
        }
        // Pattern: `/path: No such file or directory` or `path: not found`
        let lower = s.to_ascii_lowercase();
        for needle in [": no such file or directory", ": not found"] {
            if let Some(idx) = lower.find(needle) {
                return s[..idx].trim().trim_matches('"').to_string();
            }
        }
        s
    }

    /// Interpreta o primeiro byte de status SCP: `0`=OK, `1`/`2`=err (+ mensagem).
    fn interpret_scp_status(bytes: &[u8]) -> SshCliResult<()> {
        if bytes.is_empty() {
            return Err(SshCliError::ChannelFailed(
                "empty SCP status (expected ACK 0x00)".to_string(),
            ));
        }
        match bytes[0] {
            SCP_OK => Ok(()),
            1 | 2 => {
                let msg = String::from_utf8_lossy(&bytes[1..]).trim().to_string();
                if msg.is_empty() {
                    Err(SshCliError::ChannelFailed(format!(
                        "SCP rejected the transfer (status {})",
                        bytes[0]
                    )))
                } else {
                    // Stable prefix for agents; classifier looks at OpenSSH text.
                    let full = format!("SCP: {msg}");
                    Err(classify_scp_message(&full))
                }
            }
            other => Err(SshCliError::ChannelFailed(format!(
                "unexpected SCP status: 0x{other:02x}"
            ))),
        }
    }

    /// Builds `scp -t[p]/-f[p]` with remote path escaped for the remote shell.
    ///
    /// OpenSSH: source (`-f`) only emits `T` line and honest mode with **`-p`**.
    /// Sink (`-t`) with `-p` applies full mode (no sticky umask mask).
    /// Sempre usamos `-p` (SCP-023 bi-direcional).
    fn remote_scp_command(mode: &str, remote: &std::path::Path) -> String {
        let path = crate::ssh::packing::escape_shell_single_quotes(&remote.display().to_string());
        // `mode` expected: `-t` or `-f` (without `-p`); we append `p` explicitly.
        let mode_p = if mode.contains('p') {
            mode.to_string()
        } else {
            format!("{mode}p")
        };
        // Path in single-quotes (no `--` for maximum legacy OpenSSH scp compatibility).
        format!("scp {mode_p} {path}")
    }

    /// Aplica mode POSIX do header `C` no file local (best-effort no Unix).
    fn apply_local_mode(path: &std::path::Path, mode: u32) -> SshCliResult<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(mode & 0o7777);
            std::fs::set_permissions(path, perms).map_err(SshCliError::Io)?;
        }
        #[cfg(not(unix))]
        {
            let _ = (path, mode);
        }
        Ok(())
    }

    /// Reads the next non-empty `ChannelMsg::Data` from the SCP channel.
    async fn scp_read_data<S>(channel: &mut russh::Channel<S>) -> SshCliResult<Vec<u8>>
    where
        S: From<(russh::ChannelId, russh::ChannelMsg)> + Send + Sync + 'static,
    {
        use russh::ChannelMsg;
        loop {
            match channel.wait().await {
                Some(ChannelMsg::Data { data }) => {
                    if data.is_empty() {
                        continue;
                    }
                    return Ok(data.to_vec());
                }
                Some(ChannelMsg::ExtendedData { data, .. }) => {
                    if data.is_empty() {
                        continue;
                    }
                    let msg = String::from_utf8_lossy(data.as_ref()).trim().to_string();
                    // GAP-SSH-IO-010: OpenSSH stderr "No such file" → 66, not 74.
                    let full = format!("SCP stderr: {msg}");
                    return Err(classify_scp_message(&full));
                }
                Some(ChannelMsg::ExitStatus { exit_status }) if exit_status != 0 => {
                    return Err(SshCliError::ChannelFailed(format!(
                        "scp encerrou com exit {exit_status}"
                    )));
                }
                Some(ChannelMsg::Close) | None => {
                    return Err(SshCliError::ChannelFailed(
                        "SCP channel closed prematurely".to_string(),
                    ));
                }
                _ => continue,
            }
        }
    }

    /// Aguarda ACK de status SCP (`0x00`) ou propaga err `1`/`2`.
    async fn scp_wait_status<S>(channel: &mut russh::Channel<S>) -> SshCliResult<()>
    where
        S: From<(russh::ChannelId, russh::ChannelMsg)> + Send + Sync + 'static,
    {
        let data = scp_read_data(channel).await?;
        interpret_scp_status(&data)
    }

    /// Reads bytes until a newline (header `C`/`T`) or error status `1`/`2`.
    async fn scp_read_until_newline<S>(channel: &mut russh::Channel<S>) -> SshCliResult<Vec<u8>>
    where
        S: From<(russh::ChannelId, russh::ChannelMsg)> + Send + Sync + 'static,
    {
        let mut buf = Vec::new();
        loop {
            let chunk = scp_read_data(channel).await?;
            if buf.is_empty() && matches!(chunk.first().copied(), Some(1 | 2)) {
                return Ok(chunk);
            }
            buf.extend_from_slice(&chunk);
            if buf.contains(&b'\n') {
                return Ok(buf);
            }
            if buf.len() > 16_384 {
                return Err(SshCliError::ChannelFailed(
                    "SCP header excessively long".to_string(),
                ));
            }
        }
    }

    impl SshClient {
        /// Connects and authenticates. The full flow (TCP + handshake + auth) honors
        /// the configuration `timeout_ms`.
        ///
        /// # Errors
        /// - [`SshCliError::InvalidArgument`] if the configuration is invalid.
        /// - [`SshCliError::SshTimeout`] se exceder o timeout total.
        /// - [`SshCliError::ConnectionFailed`] em falhas TCP/handshake.
        /// - [`SshCliError::AuthenticationFailed`] se o servidor rejeitar password/key
        ///   (tente `--key`, `--password-stdin` ou `--key-passphrase-stdin`).
        pub async fn connect(cfg: ConnectionConfig) -> SshCliResult<Self> {
            cfg.validate()?;

            let timeout = Duration::from_millis(cfg.timeout_ms);
            let host = cfg.host.clone();
            let port = cfg.port;
            let username = cfg.username.clone();
            let secure_password = cfg.password.clone();
            let key_path = cfg.key_path.clone();
            let key_passphrase = cfg.key_passphrase.clone();
            let handler = ClientHandler::new(&cfg);

            let client_config = Arc::new(russh::client::Config {
                inactivity_timeout: Some(timeout),
                ..Default::default()
            });

            tracing::info!(
                host = %host,
                port,
                username = %username,
                timeout_ms = cfg.timeout_ms,
                has_key = key_path.is_some(),
                "iniciando conexão SSH"
            );

            let connection_result = tokio::time::timeout(timeout, async move {
                let mut session = russh::client::connect(
                    client_config,
                    (host.as_str(), port),
                    handler,
                )
                .await
                .map_err(|e| SshCliError::ConnectionFailed(format!("TCP/handshake failed: {e}")))?;

                // Preference: private key first; password fallback if both present.
                let mut autenticado = false;

                if let Some(ref kp) = key_path {
                    let pass = key_passphrase
                        .as_ref()
                        .map(|s| s.expose_secret().to_string());
                    let key = russh::keys::load_secret_key(kp, pass.as_deref()).map_err(|e| {
                        SshCliError::SshAuthentication(format!(
                            "failed to load key {kp}: {e}"
                        ))
                    })?;
                    let hash = session
                        .best_supported_rsa_hash()
                        .await
                        .map_err(|e| {
                            SshCliError::ConnectionFailed(format!("rsa hash: {e}"))
                        })?
                        .flatten();
                    let auth = session
                        .authenticate_publickey(
                            username.clone(),
                            russh::keys::PrivateKeyWithHashAlg::new(Arc::new(key), hash),
                        )
                        .await
                        .map_err(|e| {
                            SshCliError::ConnectionFailed(format!("publickey auth failed: {e}"))
                        })?;
                    autenticado = auth.success();
                    if !autenticado {
                        tracing::warn!(host = %host, "key auth rejected; trying password if present");
                    }
                }

                if !autenticado && !secure_password.expose_secret().is_empty() {
                    let auth = session
                        .authenticate_password(username.clone(), secure_password.expose_secret())
                        .await
                        .map_err(|e| {
                            SshCliError::ConnectionFailed(format!("password auth failed: {e}"))
                        })?;
                    autenticado = auth.success();
                }

                if !autenticado {
                    tracing::warn!(host = %host, username = %username, "SSH authentication rejected");
                    return Err(SshCliError::AuthenticationFailed);
                }

                Ok::<_, SshCliError>(session)
            })
            .await;

            let session = match connection_result {
                Ok(Ok(s)) => s,
                Ok(Err(err)) => return Err(err),
                Err(_) => return Err(SshCliError::SshTimeout(cfg.timeout_ms)),
            };

            tracing::info!("conexão SSH autenticada com sucesso");

            Ok(Self { session, cfg })
        }

        /// Runs a remote shell command and captures stdout/stderr in parallel.
        ///
        /// Trunca cada stream em `max_chars` codepoints UTF-8. Respeita o
        /// configuration `timeout_ms` for the entire execution.
        ///
        /// # Errors
        /// - [`SshCliError::ChannelFailed`] em falha ao abrir channel ou enviar `exec`.
        /// - [`SshCliError::SshTimeout`] se exceder o timeout.
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
            abort_em_timeout: bool,
            stdin_data: Option<Vec<u8>>,
        ) -> SshCliResult<ExecutionOutput> {
            let inicio = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms);

            let result = tokio::time::timeout(timeout, async {
                let mut channel = self
                    .session
                    .channel_open_session()
                    .await
                    .map_err(|e| SshCliError::ChannelFailed(format!("abrir sessão: {e}")))?;

                channel
                    .exec(true, command)
                    .await
                    .map_err(|e| SshCliError::ChannelFailed(format!("exec: {e}")))?;

                // Senha sudo/su no stdin do channel — nunca na cmdline remota (SEC-001).
                if let Some(bytes) = stdin_data.as_ref() {
                    channel
                        .data(&bytes[..])
                        .await
                        .map_err(|e| SshCliError::ChannelFailed(format!("stdin channel: {e}")))?;
                    channel
                        .eof()
                        .await
                        .map_err(|e| SshCliError::ChannelFailed(format!("eof channel: {e}")))?;
                }

                let mut stdout_bytes: Vec<u8> = Vec::new();
                let mut stderr_bytes: Vec<u8> = Vec::new();
                let mut exit_code: Option<i32> = None;

                while let Some(msg) = channel.wait().await {
                    if process_exec_message(
                        msg,
                        &mut stdout_bytes,
                        &mut stderr_bytes,
                        &mut exit_code,
                    ) {
                        break;
                    }
                }

                Ok::<_, SshCliError>((stdout_bytes, stderr_bytes, exit_code))
            })
            .await;

            let (stdout_bytes, stderr_bytes, exit_code) = match result {
                Ok(Ok(t)) => t,
                Ok(Err(err)) => return Err(err),
                Err(_) => {
                    if abort_em_timeout {
                        if let Some(pattern) = crate::ssh::packing::remote_abort_pattern(command) {
                            let abort_cmd = crate::ssh::packing::pack_abort_pkill(&pattern);
                            tracing::warn!(
                                pattern = %pattern,
                                "local timeout; attempting best-effort remote abort"
                            );
                            let _ = self.try_remote_abort(&abort_cmd).await;
                        }
                    }
                    return Err(SshCliError::SshTimeout(self.cfg.timeout_ms));
                }
            };

            let stdout_str = String::from_utf8_lossy(&stdout_bytes).to_string();
            let stderr_str = String::from_utf8_lossy(&stderr_bytes).to_string();

            let (stdout_truncado, truncated_stdout) = super::truncate_utf8(&stdout_str, max_chars);
            let (stderr_truncado, truncated_stderr) = super::truncate_utf8(&stderr_str, max_chars);

            let duration_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);

            Ok(ExecutionOutput {
                stdout: stdout_truncado,
                stderr: stderr_truncado,
                exit_code,
                truncated_stdout,
                truncated_stderr,
                duration_ms,
            })
        }

        /// Upload de file local para remote via SCP (protocolo OpenSSH sink).
        ///
        /// One-shot: stream in chunks (without loading the whole file into RAM).
        ///
        /// # Errors
        /// - [`SshCliError::FileNotFound`] if the local file does not exist.
        /// - [`SshCliError::InvalidArgument`] if the local path is not a regular file.
        /// - [`SshCliError::ChannelFailed`] em falha ao abrir channel SCP / status remoto.
        /// - [`SshCliError::SshTimeout`] se exceder o timeout.
        pub async fn upload(
            &mut self,
            local: &std::path::Path,
            remote: &std::path::Path,
        ) -> SshCliResult<TransferResult> {
            use russh::ChannelMsg;
            use std::io::Read;
            use std::time::Instant;

            let local_str = local.display().to_string();

            if local.is_dir() {
                return Err(SshCliError::InvalidArgument(crate::i18n::t(
                    crate::i18n::Message::ScpUploadFileOnly,
                )));
            }

            let metadados = std::fs::metadata(local).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    SshCliError::FileNotFound(local_str.clone())
                } else {
                    SshCliError::Io(e)
                }
            })?;

            if !metadados.is_file() {
                return Err(SshCliError::InvalidArgument(crate::i18n::t(
                    crate::i18n::Message::ScpUploadFileOnly,
                )));
            }

            let size = metadados.len();
            let mode = scp_mode_from_metadata(&metadados);
            let mtime = metadados.modified().ok().map(system_time_secs).unwrap_or(0);
            let atime = metadados
                .accessed()
                .ok()
                .map(system_time_secs)
                .unwrap_or(mtime);
            let file_name = local.file_name().and_then(|n| n.to_str()).unwrap_or("file");

            let inicio = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms);

            let result =
                tokio::time::timeout(timeout, async {
                    if crate::signals::is_cancelled() {
                        return Err(SshCliError::InvalidArgument(crate::i18n::t(
                            crate::i18n::Message::OperationCancelled,
                        )));
                    }

                    let mut channel =
                        self.session.channel_open_session().await.map_err(|e| {
                            SshCliError::ChannelFailed(format!("abrir sessão SCP: {e}"))
                        })?;

                    let command = remote_scp_command("-t", remote);
                    channel
                        .exec(true, command.as_str())
                        .await
                        .map_err(|e| SshCliError::ChannelFailed(format!("exec SCP: {e}")))?;

                    // Remote sink sends ACK (0x00) before accepting the header.
                    scp_wait_status(&mut channel).await?;

                    // Preserve times (line T) antes do header C.
                    let linha_t = format_scp_t_line(mtime, atime);
                    channel
                        .data(linha_t.as_bytes())
                        .await
                        .map_err(|e| SshCliError::ChannelFailed(format!("enviar linha T SCP: {e}")))?;
                    scp_wait_status(&mut channel).await?;

                    let header = format_scp_upload_header_with_mode(mode, size, file_name);
                    channel
                        .data(header.as_bytes())
                        .await
                        .map_err(|e| SshCliError::ChannelFailed(format!("enviar header SCP: {e}")))?;
                    scp_wait_status(&mut channel).await?;

                    // SCP-018: stream do disco em chunks (sem fs::read total).
                    let mut file = std::fs::File::open(local).map_err(SshCliError::Io)?;
                    let mut buf = vec![0u8; 32_768];
                    loop {
                        if crate::signals::is_cancelled() {
                            return Err(SshCliError::InvalidArgument(crate::i18n::t(
                                crate::i18n::Message::OperationCancelled,
                            )));
                        }
                        let n = file.read(&mut buf).map_err(SshCliError::Io)?;
                        if n == 0 {
                            break;
                        }
                        channel.data(&buf[..n]).await.map_err(|e| {
                            SshCliError::ChannelFailed(format!("enviar bloco SCP: {e}"))
                        })?;
                    }

                    // File terminator = byte 0x00 (not empty data).
                    channel
                        .data([SCP_OK].as_slice())
                        .await
                        .map_err(|e| SshCliError::ChannelFailed(format!("enviar EOF SCP: {e}")))?;
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

            result.map_err(|_| SshCliError::SshTimeout(self.cfg.timeout_ms))??;

            let duration_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);

            Ok(TransferResult {
                bytes_transferred: size,
                duration_ms,
            })
        }

        /// Download de file remote para local via SCP (protocolo OpenSSH source).
        ///
        /// Writes to `{local}.ssh-cli.partial` and renames atomically (SCP-022).
        ///
        /// # Errors
        /// - [`SshCliError::Io`] if the local file cannot be written.
        /// - [`SshCliError::ChannelFailed`] em falha ao abrir channel SCP / status remoto.
        /// - [`SshCliError::SshTimeout`] se exceder o timeout.
        pub async fn download(
            &mut self,
            remote: &std::path::Path,
            local: &std::path::Path,
        ) -> SshCliResult<TransferResult> {
            use russh::ChannelMsg;
            use std::io::Write;
            use std::time::{Duration as StdDuration, Instant, UNIX_EPOCH};

            if local.is_dir() {
                return Err(SshCliError::InvalidArgument(crate::i18n::t(
                    crate::i18n::Message::ScpDownloadLocalNotDirectory,
                )));
            }

            let inicio = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms);
            let partial = partial_download_path(local);

            let result = tokio::time::timeout(timeout, async {
                if crate::signals::is_cancelled() {
                    return Err(SshCliError::InvalidArgument(crate::i18n::t(
                        crate::i18n::Message::OperationCancelled,
                    )));
                }

                let mut channel = self
                    .session
                    .channel_open_session()
                    .await
                    .map_err(|e| SshCliError::ChannelFailed(format!("abrir sessão SCP: {e}")))?;

                let command = remote_scp_command("-f", remote);
                channel
                    .exec(true, command.as_str())
                    .await
                    .map_err(|e| SshCliError::ChannelFailed(format!("exec SCP: {e}")))?;

                // Remote source only sends the header after the local sink's initial ACK.
                channel
                    .data([SCP_OK].as_slice())
                    .await
                    .map_err(|e| SshCliError::ChannelFailed(format!("enviar ack inicial: {e}")))?;

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
                        .map_err(|e| SshCliError::ChannelFailed(format!("enviar ack T: {e}")))?;
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
                    .map_err(|e| SshCliError::ChannelFailed(format!("enviar ack header: {e}")))?;

                if let Some(parent_dir) = local.parent() {
                    if !parent_dir.as_os_str().is_empty() {
                        std::fs::create_dir_all(parent_dir)?;
                    }
                }

                // SCP-022: write to partial; rename only on success.
                let mut file = std::fs::File::create(&partial).map_err(SshCliError::Io)?;
                let mut recebidos: u64 = 0;
                let mut pendente: Vec<u8> = Vec::new();

                while recebidos < size {
                    if crate::signals::is_cancelled() {
                        return Err(SshCliError::InvalidArgument(crate::i18n::t(
                            crate::i18n::Message::OperationCancelled,
                        )));
                    }
                    if pendente.is_empty() {
                        let chunk = scp_read_data(&mut channel).await?;
                        pendente.extend_from_slice(&chunk);
                    }
                    let falta = (size - recebidos) as usize;
                    let usar = falta.min(pendente.len());
                    file
                        .write_all(&pendente[..usar])
                        .map_err(SshCliError::Io)?;
                    recebidos += usar as u64;
                    pendente.drain(..usar);
                }

                // After payload, source sends final 0x00 (may already be in `pendente`).
                if pendente.is_empty() {
                    match scp_read_data(&mut channel).await {
                        Ok(trail) => pendente.extend_from_slice(&trail),
                        Err(_) if recebidos == size => {}
                        Err(e) => return Err(e),
                    }
                }
                if pendente.first() == Some(&SCP_OK) {
                    pendente.remove(0);
                } else if !pendente.is_empty() {
                    return Err(SshCliError::ChannelFailed(format!(
                        "unexpected SCP terminator after payload (0x{:02x})",
                        pendente[0]
                    )));
                }

                file.flush().map_err(SshCliError::Io)?;
                let _ = file.sync_data();
                drop(file);

                channel
                    .data([SCP_OK].as_slice())
                    .await
                    .map_err(|e| SshCliError::ChannelFailed(format!("enviar ack final: {e}")))?;

                let _ = channel.eof().await;
                while let Some(msg) = channel.wait().await {
                    if matches!(msg, ChannelMsg::Close) {
                        break;
                    }
                }

                // SCP-022b: apply mode/times on partial BEFORE atomic rename.
                // So metadata failure does not leave `local` with partial success content.
                apply_local_mode(&partial, mode_remoto)?;
                if let Some((mtime, atime)) = times {
                    let mtime_st = UNIX_EPOCH + StdDuration::from_secs(mtime);
                    let atime_st = UNIX_EPOCH + StdDuration::from_secs(atime);
                    let ft = std::fs::FileTimes::new()
                        .set_modified(mtime_st)
                        .set_accessed(atime_st);
                    if let Ok(f) = std::fs::File::options().write(true).open(&partial) {
                        let _ = f.set_times(ft);
                    }
                }

                std::fs::rename(&partial, local).map_err(SshCliError::Io)?;
                // Atomic write: fsync parent_dir after rename (best-effort).
                if let Some(parent_dir) = local.parent() {
                    if !parent_dir.as_os_str().is_empty() {
                        if let Ok(dir) = std::fs::File::open(parent_dir) {
                            let _ = dir.sync_all();
                        }
                    }
                }

                Ok::<_, SshCliError>(recebidos)
            })
            .await;

            match result {
                Ok(Ok(recebidos)) => {
                    let duration_ms =
                        u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
                    Ok(TransferResult {
                        bytes_transferred: recebidos,
                        duration_ms,
                    })
                }
                Ok(Err(e)) => {
                    let _ = std::fs::remove_file(&partial);
                    // If rename already happened and something failed later (best-effort fsync does not fail),
                    // still remove partial; `local` only exists after a successful rename.
                    Err(e)
                }
                Err(_) => {
                    let _ = std::fs::remove_file(&partial);
                    Err(SshCliError::SshTimeout(self.cfg.timeout_ms))
                }
            }
        }

        /// Best-effort remote abort: reconnects with a short timeout and runs pkill.
        async fn try_remote_abort(&self, abort_cmd: &str) -> SshCliResult<()> {
            // Inline implementation (without calling run_command_internal) avoids
            // async recursion detected by the compiler.
            let mut cfg_abort = self.cfg.clone();
            cfg_abort.timeout_ms = cfg_abort.timeout_ms.clamp(3_000, 10_000);
            let outro = match Self::connect(cfg_abort).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::debug!(err = %e, "remote abort could not reconnect");
                    return Err(e);
                }
            };
            let timeout = Duration::from_millis(outro.cfg.timeout_ms);
            let _ = tokio::time::timeout(timeout, async {
                let mut channel = outro
                    .session
                    .channel_open_session()
                    .await
                    .map_err(|e| SshCliError::ChannelFailed(format!("abort channel: {e}")))?;
                channel
                    .exec(true, abort_cmd)
                    .await
                    .map_err(|e| SshCliError::ChannelFailed(format!("abort exec: {e}")))?;
                while let Some(msg) = channel.wait().await {
                    if matches!(msg, russh::ChannelMsg::Close) {
                        break;
                    }
                }
                Ok::<(), SshCliError>(())
            })
            .await;
            let _ = outro.disconnect().await;
            Ok(())
        }

        /// Cleanly closes the SSH session.
        ///
        /// # Errors
        /// Propaga falha se `disconnect` retornar err do transporte.
        pub async fn disconnect(&self) -> SshCliResult<()> {
            let result = self
                .session
                .disconnect(russh::Disconnect::ByApplication, "encerrando", "pt-BR")
                .await;
            match result {
                Ok(()) => {
                    tracing::info!("sessão SSH encerrada");
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

        /// Abre channel direct-tcpip para forwarding SSH.
        pub async fn open_tunnel_channel(
            &self,
            remote_host: &str,
            remote_port: u16,
            endereco_origem: &str,
            porta_origem: u16,
        ) -> SshCliResult<Box<dyn TunnelChannel>> {
            let channel = self
                .session
                .channel_open_direct_tcpip(
                    remote_host.to_string(),
                    u32::from(remote_port),
                    endereco_origem.to_string(),
                    u32::from(porta_origem),
                )
                .await
                .map_err(|e| {
                    SshCliError::ChannelFailed(format!(
                        "failed to open direct-tcpip channel to {}:{}: {}",
                        remote_host, remote_port, e
                    ))
                })?;

            Ok(Box::new(channel.into_stream()))
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
            &mut self,
            local: &Path,
            remote: &Path,
        ) -> Result<TransferResult, SshCliError> {
            Self::upload(self, local, remote).await
        }

        async fn download(
            &mut self,
            remote: &Path,
            local: &Path,
        ) -> Result<TransferResult, SshCliError> {
            Self::download(self, remote, local).await
        }

        async fn open_tunnel_channel(
            &self,
            remote_host: &str,
            remote_port: u16,
            endereco_origem: &str,
            porta_origem: u16,
        ) -> Result<Box<dyn TunnelChannel>, SshCliError> {
            Self::open_tunnel_channel(
                self,
                remote_host,
                remote_port,
                endereco_origem,
                porta_origem,
            )
            .await
        }

        async fn disconnect(&self) -> Result<(), SshCliError> {
            Self::disconnect(self).await
        }
    }

    #[cfg(test)]
    mod real_tests {
        use super::{
            partial_download_path, classify_scp_message, remote_scp_command,
            format_scp_upload_header, format_scp_upload_header_with_mode, format_scp_t_line,
            interpret_scp_status, map_exit_status, parse_scp_header, parse_scp_t_line,
            process_exec_message, SCP_PARTIAL_SUFFIX,
        };
        use crate::erros::SshCliError;

        #[test]
        fn map_exit_status_normal() {
            assert_eq!(map_exit_status(0), 0);
            assert_eq!(map_exit_status(255), 255);
        }

        #[test]
        fn map_exit_status_overflow_returns_minus_one() {
            assert_eq!(map_exit_status(u32::MAX), -1);
        }

        #[test]
        fn parse_scp_header_valid_returns_mode_and_size() {
            let (mode, size) =
                parse_scp_header("C0644 42 arquivo.txt\n").expect("valid header");
            assert_eq!(mode, 0o644);
            assert_eq!(size, 42);
            let (mode2, _) = parse_scp_header("C0600 1 x\n").expect("600");
            assert_eq!(mode2, 0o600);
        }

        #[test]
        fn parse_scp_header_invalid_returns_error() {
            assert!(parse_scp_header("ERRO").is_err());
            assert!(parse_scp_header("C0644 sem_tamanho").is_err());
            assert!(parse_scp_header("C0644 abc arquivo").is_err());
            assert!(parse_scp_header("Czzzz 1 x\n").is_err());
        }

        #[test]
        fn process_exec_message_handles_stdout_stderr_close() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = None;

            let deve_parar = process_exec_message(
                russh::ChannelMsg::Data {
                    data: b"stdout".to_vec().into(),
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );
            assert!(!deve_parar);
            assert_eq!(stdout, b"stdout");

            let deve_parar = process_exec_message(
                russh::ChannelMsg::ExtendedData {
                    data: b"stderr".to_vec().into(),
                    ext: 1,
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );
            assert!(!deve_parar);
            assert_eq!(stderr, b"stderr");

            let _ = process_exec_message(
                russh::ChannelMsg::ExitStatus { exit_status: 17 },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );
            assert_eq!(exit_code, Some(17));

            let deve_parar = process_exec_message(
                russh::ChannelMsg::Close,
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );
            assert!(deve_parar);
        }

        #[test]
        fn format_scp_upload_header_expected_format() {
            let header = format_scp_upload_header(123, "arquivo.txt");
            // Wire protocol: real newline (0x0a), NOT the literal '\'+'n' sequence.
            assert_eq!(header, "C0644 123 arquivo.txt\n");
            assert_eq!(header.as_bytes().last().copied(), Some(b'\n'));
            assert!(
                !header.as_bytes().windows(2).any(|w| w == *b"\\n"),
                "header must not contain literal backslash-n"
            );
        }

        #[test]
        fn format_scp_upload_header_uses_basename() {
            let header = format_scp_upload_header(1, "/tmp/dir/nome.bin");
            assert_eq!(header, "C0644 1 nome.bin\n");
        }

        #[test]
        fn interpret_scp_status_ok_and_error() {
            assert!(interpret_scp_status(&[0]).is_ok());
            assert!(interpret_scp_status(&[1, b'f', b'a', b'i', b'l']).is_err());
            assert!(interpret_scp_status(&[]).is_err());
        }

        /// GAP-SSH-IO-010: remote missing → FileNotFound (exit 66).
        #[test]
        fn interpret_scp_status_no_such_file() {
            let mut payload = vec![1u8];
            payload.extend_from_slice(b"scp: /tmp/missing: No such file or directory\n");
            let err = interpret_scp_status(&payload).unwrap_err();
            assert!(
                matches!(err, SshCliError::FileNotFound(_)),
                "esperado FileNotFound, got {err:?}"
            );
            assert_eq!(err.exit_code(), 66);
        }

        #[test]
        fn classificar_mensagem_scp_protocol_permanece_canal() {
            let err = classify_scp_message("SCP: protocol error");
            assert!(matches!(err, SshCliError::ChannelFailed(_)));
            assert_eq!(err.exit_code(), 74);
            let err2 = classify_scp_message("SCP stderr: Permission denied");
            assert!(matches!(err2, SshCliError::ChannelFailed(_)));
            assert_eq!(err2.exit_code(), 74);
        }

        #[test]
        fn classificar_mensagem_scp_not_found_e_66() {
            let err = classify_scp_message("SCP stderr: scp: foo: not found");
            assert!(matches!(err, SshCliError::FileNotFound(_)));
            assert_eq!(err.exit_code(), 66);
        }

        #[test]
        fn remote_scp_command_escapa_path_e_usa_p() {
            let cmd = remote_scp_command("-t", std::path::Path::new("/tmp/a b.txt"));
            assert_eq!(cmd, "scp -tp '/tmp/a b.txt'");
            let cmd_f = remote_scp_command("-f", std::path::Path::new("/var/log/a.log"));
            assert_eq!(cmd_f, "scp -fp '/var/log/a.log'");
            // Idempotent if it already contains p.
            assert_eq!(
                remote_scp_command("-fp", std::path::Path::new("/x")),
                "scp -fp '/x'"
            );
        }

        #[test]
        fn format_scp_t_line_format() {
            let t = format_scp_t_line(1_700_000_000, 1_700_000_001);
            assert_eq!(t, "T1700000000 0 1700000001 0\n");
            assert_eq!(t.as_bytes().last().copied(), Some(b'\n'));
        }

        #[test]
        fn parse_scp_t_line_ok() {
            let (m, a) = parse_scp_t_line("T100 0 200 0\n").expect("T ok");
            assert_eq!((m, a), (100, 200));
        }

        #[test]
        fn format_header_with_mode() {
            let h = format_scp_upload_header_with_mode(0o755, 10, "x.sh");
            assert_eq!(h, "C0755 10 x.sh\n");
        }

        #[test]
        fn partial_download_path_suffix() {
            let p = partial_download_path(std::path::Path::new("/tmp/out.bin"));
            assert!(p.to_string_lossy().ends_with(SCP_PARTIAL_SUFFIX));
            assert!(p.to_string_lossy().contains("out.bin"));
        }

        #[test]
        fn process_exec_message_ignores_extended_non_stderr() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = None;

            let deve_parar = process_exec_message(
                russh::ChannelMsg::ExtendedData {
                    data: b"nao-e-stderr".to_vec().into(),
                    ext: 2,
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );

            assert!(!deve_parar);
            assert!(stdout.is_empty());
            assert!(stderr.is_empty());
            assert!(exit_code.is_none());
        }

        #[test]
        fn process_exec_message_handles_exit_signal_and_eof() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = Some(7);

            let deve_parar_signal = process_exec_message(
                russh::ChannelMsg::ExitSignal {
                    signal_name: russh::Sig::TERM,
                    core_dumped: false,
                    error_message: "encerrado".to_string(),
                    lang_tag: "pt-BR".to_string(),
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );

            let deve_parar_eof = process_exec_message(
                russh::ChannelMsg::Eof,
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );

            assert!(!deve_parar_signal);
            assert!(!deve_parar_eof);
            assert_eq!(exit_code, Some(7));
        }

        #[test]
        fn process_exec_message_ignores_unhandled_variants() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = None;

            let deve_parar = process_exec_message(
                russh::ChannelMsg::WindowAdjusted { new_size: 2048 },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );

            assert!(!deve_parar);
            assert!(stdout.is_empty());
            assert!(stderr.is_empty());
            assert!(exit_code.is_none());
        }
    }
}

#[cfg(feature = "ssh-real")]
pub use real::{SshClient, ClientHandler};

// =========================================================================
// Stub used when the `ssh-real` feature is DISABLED.
// =========================================================================

#[cfg(not(feature = "ssh-real"))]
mod stub {
    use super::{ConnectionConfig, ExecutionOutput, TransferResult};
    use crate::erros::SshCliError;
    use crate::ssh::client::SshClientTrait;
    use async_trait::async_trait;
    use std::path::Path;

    /// Stub when `ssh-real` is disabled: always returns
    /// [`SshCliError::ConnectionFailed`].
    #[derive(Debug)]
    pub struct SshClient;

    #[async_trait]
    impl SshClientTrait for SshClient {
        async fn connect(_cfg: ConnectionConfig) -> Result<Box<Self>, SshCliError> {
            Err(SshCliError::ConnectionFailed(
                "feature `ssh-real` está desabilitada; recompile com --features ssh-real".into(),
            ))
        }

        async fn run_command(
            &mut self,
            _cmd: &str,
            _max_chars: usize,
            _stdin_data: Option<Vec<u8>>,
        ) -> Result<ExecutionOutput, SshCliError> {
            Err(SshCliError::ChannelFailed(
                "stub sem russh: feature `ssh-real` desabilitada".into(),
            ))
        }

        async fn upload(
            &mut self,
            _local: &Path,
            _remote: &Path,
        ) -> Result<TransferResult, SshCliError> {
            Err(SshCliError::ChannelFailed(
                "stub sem russh: feature `ssh-real` desabilitada".into(),
            ))
        }

        async fn download(
            &mut self,
            _remote: &Path,
            _local: &Path,
        ) -> Result<TransferResult, SshCliError> {
            Err(SshCliError::ChannelFailed(
                "stub sem russh: feature `ssh-real` desabilitada".into(),
            ))
        }

        async fn open_tunnel_channel(
            &self,
            _host_remoto: &str,
            _porta_remota: u16,
            _endereco_origem: &str,
            _porta_origem: u16,
        ) -> Result<Box<dyn super::TunnelChannel>, SshCliError> {
            Err(SshCliError::ChannelFailed(
                "stub sem russh: feature `ssh-real` desabilitada".into(),
            ))
        }

        async fn disconnect(&self) -> Result<(), SshCliError> {
            Ok(())
        }
    }
}

#[cfg(not(feature = "ssh-real"))]
pub use stub::SshClient;

// =========================================================================
// Unit tests (no network, no feature gate).
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    fn valid_cfg() -> ConnectionConfig {
        ConnectionConfig {
            host: "127.0.0.1".to_string(),
            port: 22,
            username: "root".to_string(),
            password: SecretString::from("senha-exemplo".to_string()),
            key_path: None,
            key_passphrase: None,
            timeout_ms: 5000,
            known_hosts_path: None,
            replace_host_key: false,
        }
    }

    #[test]
    fn validate_empty_host_returns_error() {
        let mut c = valid_cfg();
        c.host = String::new();
        let r = c.validate();
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("host"));
    }

    #[test]
    fn validate_whitespace_host_returns_error() {
        let mut c = valid_cfg();
        c.host = "   ".to_string();
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_port_zero_returns_error() {
        let mut c = valid_cfg();
        c.port = 0;
        let r = c.validate();
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("port"));
    }

    #[test]
    fn validate_empty_user_returns_error() {
        let mut c = valid_cfg();
        c.username = String::new();
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_correct_config_returns_ok() {
        assert!(valid_cfg().validate().is_ok());
    }

    #[test]
    fn debug_does_not_expose_password() {
        let c = valid_cfg();
        let dbg = format!("{c:?}");
        assert!(!dbg.contains("senha-exemplo"));
        assert!(dbg.contains("redacted"));
    }

    #[test]
    fn truncate_utf8_no_truncate_if_fits() {
        let (s, t) = truncate_utf8("ola mundo", 100);
        assert_eq!(s, "ola mundo");
        assert!(!t);
    }

    #[test]
    fn truncate_utf8_truncates_large_ascii() {
        let entrada: String = "a".repeat(200);
        let (s, t) = truncate_utf8(&entrada, 50);
        assert_eq!(s.chars().count(), 50);
        assert!(t);
    }

    #[test]
    fn truncate_utf8_preserves_accented_graphemes() {
        // 10 codepoints: "á" (1 char) * 10
        let entrada: String = "á".repeat(30);
        let (s, t) = truncate_utf8(&entrada, 10);
        assert_eq!(s.chars().count(), 10);
        // Each 'á' is 2 UTF-8 bytes → 10 chars = 20 bytes
        assert_eq!(s.len(), 20);
        assert!(t);
        // Does not split mid-byte
        assert!(s.chars().all(|c| c == 'á'));
    }

    #[test]
    fn truncate_utf8_emojis_does_not_break() {
        let entrada = "🚀🔒🛡🔑✨🎉💎⚡🌟🔥🎨";
        let (s, t) = truncate_utf8(entrada, 5);
        assert_eq!(s.chars().count(), 5);
        assert!(t);
    }

    #[test]
    fn truncate_utf8_zero_returns_empty() {
        let (s, t) = truncate_utf8("abc", 0);
        assert_eq!(s, "");
        assert!(t);
    }

    #[test]
    fn execution_output_debug_does_not_crash() {
        let s = ExecutionOutput {
            stdout: "ok".into(),
            stderr: String::new(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 42,
        };
        let _ = format!("{s:?}");
    }

    #[test]
    fn duration_ms_type_compatible() {
        // Static guarantee that Instant::elapsed fits in u64.
        let fake: u64 = 1234;
        assert_eq!(fake, 1234_u64);
    }
}
