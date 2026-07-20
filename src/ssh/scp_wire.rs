// SPDX-License-Identifier: MIT OR Apache-2.0
//! SCP wire protocol helpers (SRP extract — G-COMP-06a).
//!
//! Pure protocol framing/status classification used by [`super::client`]
//! upload/download. Kept free of session ownership so the SSH client can stay
//! focused on connect/auth/exec (one-shot lifecycle).
#![forbid(unsafe_code)]

use crate::errors::{SshCliError, SshCliResult};

/// SCP protocol ACK/OK byte (also used as payload terminator).
pub(crate) const SCP_OK: u8 = 0;

/// Safe basename for the SCP wire (no path separators / control chars).
pub(crate) fn basename_scp(file_name: &str) -> String {
    file_name
        .split(['/', '\\'])
        .next_back()
        .unwrap_or("file")
        .replace(['\n', '\r', '\0'], "_")
}

/// Header `C`-line do protocolo SCP (newline real `0x0a`, nunca `\\n` literal).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn format_scp_upload_header(size: u64, file_name: &str) -> String {
    format_scp_upload_header_with_mode(0o644, size, file_name)
}

/// Header `C` with octal mode (e.g. `0644`).
pub(crate) fn format_scp_upload_header_with_mode(mode: u32, size: u64, file_name: &str) -> String {
    let name = basename_scp(file_name);
    let mode = mode & 0o7777;
    format!("C{mode:04o} {size} {name}\n")
}

/// Linha `T` do protocolo SCP (preserve times / `-p`).
pub(crate) fn format_scp_t_line(mtime_secs: u64, atime_secs: u64) -> String {
    format!("T{mtime_secs} 0 {atime_secs} 0\n")
}

/// Parse da line `T mtime 0 atime 0`.
pub(crate) fn parse_scp_t_line(line: &str) -> SshCliResult<(u64, u64)> {
    let line = line.trim_end_matches(['\0', '\r', '\n']).trim();
    if !line.starts_with('T') {
        return Err(SshCliError::channel_msg(format!(
            "unexpected SCP T line: {line}"
        )));
    }
    let resto = &line[1..];
    let partes: Vec<&str> = resto.split_whitespace().collect();
    if partes.len() < 3 {
        return Err(SshCliError::channel_msg(format!(
            "malformed SCP T line: {line}"
        )));
    }
    let mtime: u64 = partes[0].parse().map_err(|_| {
        SshCliError::channel_msg(format!("invalid mtime in T line: {}", partes[0]))
    })?;
    let atime: u64 = partes[2].parse().map_err(|_| {
        SshCliError::channel_msg(format!("invalid atime in T line: {}", partes[2]))
    })?;
    Ok((mtime, atime))
}

/// Parse do header `C0mmm size name` → `(mode, size)`.
pub(crate) fn parse_scp_header(header: &str) -> SshCliResult<(u32, u64)> {
    let header = header.trim_end_matches(['\0', '\r', '\n']).trim();

    if !header.starts_with('C') {
        return Err(SshCliError::channel_msg(format!(
            "unexpected SCP header: {}",
            header
        )));
    }

    let partes: Vec<&str> = header.split_whitespace().collect();
    if partes.len() < 3 {
        return Err(SshCliError::channel_msg(format!(
            "malformed SCP header: {}",
            header
        )));
    }

    // Mode field: `C0644` (`C` prefix + 4 octal digits).
    let mode_token = partes[0];
    if mode_token.len() < 2 {
        return Err(SshCliError::channel_msg(format!(
            "missing SCP mode in header: {header}"
        )));
    }
    let mode_oct = &mode_token[1..];
    let mode: u32 = u32::from_str_radix(mode_oct, 8)
        .map_err(|_| SshCliError::channel_msg(format!("invalid SCP mode: {mode_oct}")))?;

    let size = partes[1].parse().map_err(|_| {
        SshCliError::channel_msg(format!("invalid size in header: {}", partes[1]))
    })?;
    Ok((mode & 0o7777, size))
}

/// Octal mode for the SCP `C` header derived from local metadata.
pub(crate) fn scp_mode_from_metadata(meta: &std::fs::Metadata) -> u32 {
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
pub(crate) fn system_time_secs(t: std::time::SystemTime) -> u64 {
    t.duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Atomic download temporary file suffix (SCP-022).
pub(crate) const SCP_PARTIAL_SUFFIX: &str = ".ssh-cli.partial";

pub(crate) fn partial_download_path(local: &std::path::Path) -> std::path::PathBuf {
    let mut p = local.as_os_str().to_os_string();
    p.push(SCP_PARTIAL_SUFFIX);
    std::path::PathBuf::from(p)
}

/// GAP-SSH-IO-010 / GAP-AUD-025: classify SCP err → missing-file (66) vs channel (74).
///
/// OpenSSH typically emits `scp: PATH: No such file or directory` on status `1`/`2`.
/// Missing-file messages are normalized to a clean path for the Display wrapper
/// `file not found: {path}` (no stacked `SCP:` / `scp:` prefixes).
pub(crate) fn classify_scp_message(msg: &str) -> SshCliError {
    let lower = msg.to_ascii_lowercase();
    if lower.contains("no such file") || lower.contains("not found") {
        SshCliError::FileNotFound(normalize_scp_missing_path(msg))
    } else if msg.is_empty() {
        SshCliError::channel_msg("SCP rejected the transfer")
    } else if msg.starts_with("SCP:") || msg.starts_with("SCP ") {
        SshCliError::channel_msg(msg)
    } else {
        SshCliError::channel_msg(format!("SCP: {msg}"))
    }
}

/// Strips `SCP:` / `scp:` wrappers and trailing OS phrases → remote path or cleaned msg.
pub(crate) fn normalize_scp_missing_path(msg: &str) -> String {
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
pub(crate) fn interpret_scp_status(bytes: &[u8]) -> SshCliResult<()> {
    if bytes.is_empty() {
        return Err(SshCliError::channel_msg(
            "empty SCP status (expected ACK 0x00)",
        ));
    }
    match bytes[0] {
        SCP_OK => Ok(()),
        1 | 2 => {
            let msg = String::from_utf8_lossy(&bytes[1..]).trim().to_string();
            if msg.is_empty() {
                Err(SshCliError::channel_msg(format!(
                    "SCP rejected the transfer (status {})",
                    bytes[0]
                )))
            } else {
                // Stable prefix for agents; classifier looks at OpenSSH text.
                let full = format!("SCP: {msg}");
                Err(classify_scp_message(&full))
            }
        }
        other => Err(SshCliError::channel_msg(format!(
            "unexpected SCP status: 0x{other:02x}"
        ))),
    }
}

/// Builds `scp -t[p]/-f[p]` with remote path escaped for the remote shell.
///
/// OpenSSH: source (`-f`) only emits `T` line and honest mode with **`-p`**.
/// Sink (`-t`) with `-p` applies full mode (no sticky umask mask).
/// Sempre usamos `-p` (SCP-023 bi-direcional).
pub(crate) fn remote_scp_command(mode: &str, remote: &std::path::Path) -> String {
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

/// Applies the POSIX mode from the SCP `C` header (best-effort on Unix).
///
/// G-PAR-50: `tokio::fs` so multi-host download does not block Tokio workers.
pub(crate) async fn apply_local_mode(path: &std::path::Path, mode: u32) -> SshCliResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(mode & 0o7777);
        tokio::fs::set_permissions(path, perms)
            .await
            .map_err(SshCliError::Io)?;
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
    Ok(())
}

/// Reads the next non-empty `ChannelMsg::Data` from the SCP channel.
pub(crate) async fn scp_read_data<S>(channel: &mut russh::Channel<S>) -> SshCliResult<Vec<u8>>
where
    S: From<(russh::ChannelId, russh::ChannelMsg)> + Send + Sync + 'static,
{
    use russh::ChannelMsg;
    loop {
        if crate::signals::should_stop() {
            return Err(SshCliError::Config(
                "operation cancelled by signal".to_string(),
            ));
        }
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
                return Err(SshCliError::channel_msg(format!(
                    "scp exited with status {exit_status}"
                )));
            }
            Some(ChannelMsg::Close) | None => {
                return Err(SshCliError::channel_msg(
                    "SCP channel closed prematurely",
                ));
            }
            _ => continue,
        }
    }
}

/// Aguarda ACK de status SCP (`0x00`) ou propaga err `1`/`2`.
pub(crate) async fn scp_wait_status<S>(channel: &mut russh::Channel<S>) -> SshCliResult<()>
where
    S: From<(russh::ChannelId, russh::ChannelMsg)> + Send + Sync + 'static,
{
    let data = scp_read_data(channel).await?;
    interpret_scp_status(&data)
}

/// Reads bytes until a newline (header `C`/`T`) or error status `1`/`2`.
pub(crate) async fn scp_read_until_newline<S>(channel: &mut russh::Channel<S>) -> SshCliResult<Vec<u8>>
where
    S: From<(russh::ChannelId, russh::ChannelMsg)> + Send + Sync + 'static,
{
    // Resource: headers are short; pre-size and hard-cap to avoid remote flood.
    let mut buf = Vec::with_capacity(256);
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
            return Err(SshCliError::channel_msg(
                "SCP header excessively long",
            ));
        }
    }
}

#[cfg(test)]
mod wire_adversarial_tests {
    use super::*;

    #[test]
    fn parse_header_adversarial_no_panic() {
        for s in ["", "C", "Cxxxx", "T", "T1", "\0\n", "C0644 not_a_size name\n", "C9999 1 x\n"] {
            let _ = parse_scp_header(s);
            let _ = parse_scp_t_line(s);
            let _ = interpret_scp_status(s.as_bytes());
        }
    }
}
