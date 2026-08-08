// SPDX-License-Identifier: MIT OR Apache-2.0
//! Brazilian Portuguese translation table.
//!
//! # Why this is a separate file (C3)
//!
//! Mirrors `en.rs`. Splitting the two tables apart also makes a one-sided
//! edit visible in review: a commit that touches only one locale now shows up
//! as a single-file change instead of hiding inside a 200-line diff.
//!
//! Accented characters are mandatory here — see the pt-BR rules in the crate
//! documentation. `gaps_v062_i18n_reachability` asserts both arms exist.

#![forbid(unsafe_code)]

use super::Message;

/// Brazilian Portuguese translations.
pub(super) fn pt(msg: &Message) -> String {
    match msg {
        Message::VpsRegistryEmpty => "Nenhum VPS cadastrado.".to_string(),
        Message::VpsAdded { name } => format!("VPS '{name}' adicionada com sucesso."),
        Message::VpsRemoved { name } => format!("VPS '{name}' removida com sucesso."),
        Message::VpsDuplicate { name } => format!("VPS '{name}' já está cadastrada."),
        Message::VpsNotFound { name } => format!("VPS '{name}' não encontrada."),
        Message::VpsActiveSelected { name } => format!("VPS ativa: '{name}'."),
        Message::ErrorConfig { detail } => format!("Erro de configuração: {detail}"),
        Message::ErrorSshConnection { detail } => format!("Erro de conexão SSH: {detail}"),
        Message::ErrorAuthentication { detail } => format!("Falha de autenticação SSH: {detail}"),
        Message::ErrorCommandFailed { detail } => {
            format!("Falha na execução do comando: {detail}")
        }
        Message::ErrorHostKeyChanged { detail } => {
            format!("Chave do host remoto mudou: {detail}")
        }
        Message::ErrorTimeout { detail } => format!("Operação excedeu o tempo limite: {detail}"),
        Message::ErrorFileNotFound { path } => format!("Arquivo não encontrado: {path}"),
        Message::ErrorUnavailable { service } => format!("Serviço indisponível: {service}"),
        Message::ErrorSoftware { op } => {
            format!("Falha interna em {op}; repetir a operação não resolve")
        }
        Message::ErrorPartialFailure { detail } => format!("Falha parcial: {detail}"),
        Message::ErrorInvalidArgument { detail } => format!("Argumento inválido: {detail}"),
        Message::ErrorUnexpected { detail } => format!("Erro inesperado: {detail}"),
        Message::VpsEdited { name } => format!("VPS '{name}' editada."),
        Message::ExportCompleted { path } => format!("exportado para {path}"),
        Message::ImportCompleted => "importação concluída".to_string(),
        Message::PrimaryKeyReady { source, key_file } => {
            format!("primary-key pronta (source={source}; key_file={key_file})")
        }
        Message::ReencryptCompleted { hosts } => {
            format!("re-cifragem concluída para {hosts} host(s)")
        }
        Message::TunnelPressCtrlC => "Pressione Ctrl+C para encerrar.".to_string(),
        Message::HealthCheckOk { name } => format!("Health check bem-sucedido para '{name}'."),
        Message::OperationCancelled => "Operação cancelada pelo usuário.".to_string(),
        Message::ScpUploadCompleted { bytes, ms } => {
            format!("Upload concluído: {bytes} bytes em {ms}ms")
        }
        Message::ScpDownloadCompleted { bytes, ms } => {
            format!("Download concluído: {bytes} bytes em {ms}ms")
        }
        Message::ScpUploadFileOnly => {
            "upload só suporta arquivos regulares (sem diretórios / sem -r)".to_string()
        }
        Message::ScpDownloadLocalNotDirectory => {
            "caminho local de download deve ser arquivo, não diretório existente".to_string()
        }
        Message::SftpUploadCompleted { bytes, ms } => {
            format!("Upload SFTP concluído: {bytes} bytes em {ms}ms")
        }
        Message::SftpDownloadCompleted { bytes, ms } => {
            format!("Download SFTP concluído: {bytes} bytes em {ms}ms")
        }
        Message::SftpFsOpDone { op, path, ms } => {
            format!("sftp {op} concluído: {path} ({ms}ms)")
        }
        Message::SftpFsOpDoneTo { op, path, to, ms } => {
            format!("sftp {op} concluído: {path} -> {to} ({ms}ms)")
        }
        Message::LocalePreferenceSaved { lang, path } => {
            format!("preferência de idioma salva: {lang} ({path})")
        }
        Message::LocalePreferenceCleared => "preferência de idioma removida.".to_string(),
        Message::LocaleStatusTitle => "Status do locale:".to_string(),
        Message::TunnelLocalListening {
            bind,
            port,
            remote_host,
            remote_port,
            vps,
            timeout_ms,
        } => format!(
            "Tunnel SSH: {bind}:{port} -> {remote_host}:{remote_port} via {vps} (timeout {timeout_ms}ms)"
        ),
        Message::TunnelSocks5Listening {
            bind,
            port,
            vps,
            timeout_ms,
        } => format!("Proxy SOCKS5: {bind}:{port} via {vps} (timeout {timeout_ms}ms)"),
        Message::TunnelStreamLocalListening {
            bind,
            port,
            socket_path,
            vps,
            timeout_ms,
        } => format!(
            "Tunnel SSH: {bind}:{port} -> unix:{socket_path} via {vps} (timeout {timeout_ms}ms)"
        ),
        Message::TunnelReverseListening {
            remote_bind,
            remote_port,
            local_host,
            local_port,
            vps,
            timeout_ms,
        } => format!(
            "Tunnel reverso: {remote_bind}:{remote_port} (remoto) -> {local_host}:{local_port} \
             via {vps} (timeout {timeout_ms}ms)"
        ),
    }
}
