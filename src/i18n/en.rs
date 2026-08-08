// SPDX-License-Identifier: MIT OR Apache-2.0
//! American English translation table.
//!
//! # Why this is a separate file (C3)
//!
//! The table is data, not logic: one exhaustive `match` arm per `Message`.
//! Keeping it beside the enum declaration and the resolution helpers pushed
//! `src/i18n.rs` past the component budget, and every new message grew the
//! same file three times over (variant, EN arm, pt-BR arm).
//!
//! The exhaustiveness guarantee is unchanged: this `match` still has to cover
//! every variant, so a new message cannot ship without an English rendering.

#![forbid(unsafe_code)]

use super::Message;

/// American English translations.
pub(super) fn en(msg: &Message) -> String {
    match msg {
        Message::VpsRegistryEmpty => "No VPS registered.".to_string(),
        Message::VpsAdded { name } => format!("VPS '{name}' added successfully."),
        Message::VpsRemoved { name } => format!("VPS '{name}' removed successfully."),
        Message::VpsDuplicate { name } => format!("VPS '{name}' is already registered."),
        Message::VpsNotFound { name } => format!("VPS '{name}' not found."),
        Message::VpsActiveSelected { name } => format!("Active VPS: '{name}'."),
        Message::ErrorConfig { detail } => format!("Configuration error: {detail}"),
        Message::ErrorSshConnection { detail } => format!("SSH connection error: {detail}"),
        Message::ErrorAuthentication { detail } => format!("SSH authentication failed: {detail}"),
        Message::ErrorCommandFailed { detail } => format!("Command execution failed: {detail}"),
        Message::ErrorHostKeyChanged { detail } => format!("Remote host key changed: {detail}"),
        Message::ErrorTimeout { detail } => format!("Operation timed out: {detail}"),
        Message::ErrorFileNotFound { path } => format!("File not found: {path}"),
        Message::ErrorUnavailable { service } => format!("Service unavailable: {service}"),
        Message::ErrorSoftware { op } => format!("Internal failure in {op}; retrying will not help"),
        Message::ErrorPartialFailure { detail } => format!("Partial failure: {detail}"),
        Message::ErrorInvalidArgument { detail } => format!("Invalid argument: {detail}"),
        Message::ErrorUnexpected { detail } => format!("Unexpected error: {detail}"),
        Message::VpsEdited { name } => format!("VPS '{name}' edited."),
        Message::ExportCompleted { path } => format!("exported to {path}"),
        Message::ImportCompleted => "import completed".to_string(),
        Message::PrimaryKeyReady { source, key_file } => {
            format!("primary-key ready (source={source}; key_file={key_file})")
        }
        Message::ReencryptCompleted { hosts } => {
            format!("re-encrypt completed for {hosts} host(s)")
        }
        Message::TunnelPressCtrlC => "Press Ctrl+C to terminate.".to_string(),
        Message::HealthCheckOk { name } => format!("Health check passed for '{name}'."),
        Message::OperationCancelled => "Operation cancelled by user.".to_string(),
        Message::ScpUploadCompleted { bytes, ms } => {
            format!("Upload completed: {bytes} bytes in {ms}ms")
        }
        Message::ScpDownloadCompleted { bytes, ms } => {
            format!("Download completed: {bytes} bytes in {ms}ms")
        }
        Message::ScpUploadFileOnly => {
            "upload only supports regular files (no directories / no -r)".to_string()
        }
        Message::ScpDownloadLocalNotDirectory => {
            "download local path must be a file path, not an existing directory".to_string()
        }
        Message::SftpUploadCompleted { bytes, ms } => {
            format!("SFTP upload completed: {bytes} bytes in {ms}ms")
        }
        Message::SftpDownloadCompleted { bytes, ms } => {
            format!("SFTP download completed: {bytes} bytes in {ms}ms")
        }
        Message::SftpFsOpDone { op, path, ms } => {
            format!("sftp {op} ok: {path} ({ms}ms)")
        }
        Message::SftpFsOpDoneTo { op, path, to, ms } => {
            format!("sftp {op} ok: {path} -> {to} ({ms}ms)")
        }
        Message::LocalePreferenceSaved { lang, path } => {
            format!("language preference saved: {lang} ({path})")
        }
        Message::LocalePreferenceCleared => "language preference cleared.".to_string(),
        Message::LocaleStatusTitle => "Locale status:".to_string(),
        Message::TunnelLocalListening {
            bind,
            port,
            remote_host,
            remote_port,
            vps,
            timeout_ms,
        } => format!(
            "SSH tunnel: {bind}:{port} -> {remote_host}:{remote_port} via {vps} (timeout {timeout_ms}ms)"
        ),
        Message::TunnelSocks5Listening {
            bind,
            port,
            vps,
            timeout_ms,
        } => format!("SOCKS5 proxy: {bind}:{port} via {vps} (timeout {timeout_ms}ms)"),
        Message::TunnelStreamLocalListening {
            bind,
            port,
            socket_path,
            vps,
            timeout_ms,
        } => format!(
            "SSH tunnel: {bind}:{port} -> unix:{socket_path} via {vps} (timeout {timeout_ms}ms)"
        ),
        Message::TunnelReverseListening {
            remote_bind,
            remote_port,
            local_host,
            local_port,
            vps,
            timeout_ms,
        } => format!(
            "Reverse tunnel: {remote_bind}:{remote_port} (remote) -> {local_host}:{local_port} \
             via {vps} (timeout {timeout_ms}ms)"
        ),
    }
}
