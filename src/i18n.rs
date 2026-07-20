// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! ssh-cli internationalization system (Rules Rust multi-idioma).
//!
//! Provides bilingual [`Language`] with [`Message`] as the **single source** of
//! human UI strings. Locale detection / BCP47 negotiation lives in [`crate::locale`].
//!
//! ## Design (agent-first one-shot)
//!
//! - **MVP locales:** neutral `en` + `pt-BR` (100% key parity via exhaustive `match`).
//! - **Not Fluent FTL at runtime:** size-sensitive CLI; compiler-enforced enum
//!   translations are the embedded equivalent of `i18n-embed` for two locales.
//! - **JSON / agent wire:** stable English field names and technical
//!   [`crate::errors::SshCliError`] `Display` (not locale-dependent).
//! - **Human UX** (success/status/cancel lines): always via [`Message`] / [`t`].
//! - Optional top-20 locales: Cargo features `i18n-*` (stubs until translations land).
//!
//! ## Precedence (see [`crate::locale`])
//!
//! 1. CLI `--lang` → 2. persisted XDG `lang` (`locale set`) →
//! 3. `sys_locale` → 4. `Language::English`.
//!
//! `SSH_CLI_LANG` is historical only — not read as a product store.

use anyhow::Result;
use unic_langid::LanguageIdentifier;

/// Text direction for terminal rendering (LTR MVP; RTL reserved for `i18n-rtl`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TextDirection {
    /// Left-to-right (Latin, CJK horizontal, etc.).
    Ltr,
    /// Right-to-left (Arabic, Hebrew) — not active in default build.
    Rtl,
}

/// Languages supported by the internationalization system.
///
/// Single source of truth for product locales in this binary. Do **not** use
/// `bool` / raw `String` / integers for language in APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Language {
    /// Neutral English (`en`) — default / agent-stable technical baseline.
    English,
    /// Brazilian Portuguese (`pt-BR`) — mandatory MVP pair with `en`.
    Portuguese,
}

impl Language {
    /// Locales compiled into the default binary (MVP: `en`, `pt-BR` only).
    pub const AVAILABLE: &'static [Language] = &[Language::English, Language::Portuguese];

    /// Canonical BCP47 tag for this product locale.
    ///
    /// English is neutral `en` (not `en-US` alone). Portuguese is always `pt-BR`.
    #[must_use]
    pub const fn bcp47(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Portuguese => "pt-BR",
        }
    }

    /// Structured BCP47 identifier (`unic-langid`).
    ///
    /// Built-in tags are compile-time constants (`en`, `pt-BR`). On parse
    /// failure (should never happen), falls back to the default undetermined
    /// identifier — **no panic** on product paths (G-SEC-07).
    #[must_use]
    pub fn language_identifier(self) -> LanguageIdentifier {
        self.bcp47()
            .parse()
            .unwrap_or_else(|_| LanguageIdentifier::default())
    }

    /// Base fallback language for regionals (MVP: English).
    #[must_use]
    pub const fn fallback(self) -> Language {
        match self {
            Self::English => Self::English,
            Self::Portuguese => Self::English,
        }
    }

    /// Writing direction for this locale.
    #[must_use]
    pub const fn direction(self) -> TextDirection {
        match self {
            Self::English | Self::Portuguese => TextDirection::Ltr,
        }
    }

    /// ISO 15924 script subtag (MVP Latin only).
    #[must_use]
    pub const fn script(self) -> &'static str {
        match self {
            Self::English | Self::Portuguese => "Latn",
        }
    }

    /// Maps a negotiated [`LanguageIdentifier`] to a product [`Language`].
    ///
    /// Matches primary language subtag: `en*` → English, `pt*` → Portuguese.
    /// Region-specific product choice for Portuguese is always `pt-BR` in MVP
    /// (no `pt-PT` variant compiled without a feature).
    #[must_use]
    pub fn from_langid(id: &LanguageIdentifier) -> Option<Language> {
        match id.language.as_str() {
            "en" => Some(Self::English),
            "pt" => Some(Self::Portuguese),
            _ => None,
        }
    }
}

/// All system UI messages.
///
/// SINGLE source of user-visible strings. Each variant has an exhaustive
/// translation in `en()` and `pt()`. FORBIDDEN to use UI literals outside this enum.
///
/// Variants with dynamic fields (e.g. `{ name: String }`) allow including
/// contextual data in the message. Message is not `Copy` because
/// `String` fields are not `Copy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    // VPS
    /// No VPS registered in the configuration file.
    VpsRegistryEmpty,
    /// Header for the registered VPS listing.
    VpsListTitle,
    /// VPS successfully added to the registry.
    VpsAdded {
        /// Name of the added VPS.
        name: String,
    },
    /// VPS successfully removed from the registry.
    VpsRemoved {
        /// Name of the removed VPS.
        name: String,
    },
    /// Attempt to add a VPS that already exists.
    VpsDuplicate {
        /// Name of the duplicate VPS.
        name: String,
    },
    /// Requested VPS was not found in the registry.
    VpsNotFound {
        /// Name of the missing VPS.
        name: String,
    },
    /// Active VPS selected for subsequent operations.
    VpsActiveSelected {
        /// Name of the selected VPS.
        name: String,
    },
    // Config
    /// Label for the configuration file path.
    ConfigPathLabel,
    /// Current configuration file path.
    ConfigPath {
        /// Absolute configuration file path.
        path: String,
    },
    /// No API keys configured in the system.
    ConfigNoKeys,
    // Erros
    /// Failed to load the configuration file.
    ErrorLoadConfig,
    /// Failed to save the configuration file.
    ErrorSaveConfig,
    /// Error establishing SSH connection to the remote server.
    ErrorSshConnection,
    /// Remote SSH command execution failed.
    ErrorCommandFailed,
    /// Invalid argument supplied to the operation.
    ErrorInvalidArgument {
        /// Detail of the invalid argument.
        detail: String,
    },
    /// Generic error with a textual description.
    ErrorGeneric {
        /// Error description.
        detail: String,
    },
    /// VPS record edited successfully.
    VpsEdited {
        /// VPS name.
        name: String,
    },
    /// Export completed.
    ExportCompleted {
        /// Destination path.
        path: String,
    },
    /// Import completed.
    ImportCompleted,
    /// Primary key ready.
    PrimaryKeyReady {
        /// Key source identifier.
        source: String,
        /// Key file path.
        key_file: String,
    },
    /// Re-encrypt completed.
    ReencryptCompleted {
        /// Host count.
        hosts: usize,
    },
    /// Generic human success line (already localized payload).
    Success {
        /// Success text.
        detail: String,
    },
    // Tunnel
    /// Active SSH tunnel with port and host information.
    TunnelActive {
        /// Local tunnel port.
        local_port: u16,
        /// Remote destination host.
        remote_host: String,
        /// Remote destination port.
        remote_port: u16,
        /// Name of the VPS used as relay.
        vps_name: String,
    },
    /// Instruction to stop the tunnel via Ctrl+C.
    TunnelPressCtrlC,
    // Health Check
    /// Successful VPS connectivity check.
    HealthCheckOk {
        /// Name of the checked VPS.
        name: String,
    },
    /// No active VPS selected for health check.
    HealthCheckNoVps,
    /// VPS connectivity check failed.
    HealthCheckFailed {
        /// Name of the checked VPS.
        name: String,
        /// Error detail.
        detail: String,
    },
    /// Health-check result with latency.
    HealthCheckLatency {
        /// Name of the checked VPS.
        name: String,
        /// Latency in milliseconds.
        latency_ms: u64,
    },
    /// Operation cancelled by user signal (Ctrl+C or SIGTERM).
    OperationCancelled,
    // SCP (GAP-SSH-SCP-020)
    /// SCP upload completed.
    ScpUploadCompleted {
        /// Bytes transferred.
        bytes: u64,
        /// Duration in milliseconds.
        ms: u64,
    },
    /// SCP download completed.
    ScpDownloadCompleted {
        /// Bytes transferred.
        bytes: u64,
        /// Duration in milliseconds.
        ms: u64,
    },
    /// Upload refused: local path is a directory (file-only, no -r).
    ScpUploadFileOnly,
    /// Download refused: local path is already a directory.
    ScpDownloadLocalNotDirectory,
    /// SFTP upload completed (G-SFTP).
    SftpUploadCompleted {
        /// Bytes transferred.
        bytes: u64,
        /// Duration in milliseconds.
        ms: u64,
    },
    /// SFTP download completed (G-SFTP).
    SftpDownloadCompleted {
        /// Bytes transferred.
        bytes: u64,
        /// Duration in milliseconds.
        ms: u64,
    },
    // Locale diagnostics / preference
    /// Locale preference saved.
    LocalePreferenceSaved {
        /// BCP47 tag written.
        lang: String,
        /// Path of the preference file.
        path: String,
    },
    /// Locale preference cleared.
    LocalePreferenceCleared,
    /// Header for `locale` show output.
    LocaleStatusTitle,
}

impl Message {
    /// Returns the message string in the specified language.
    ///
    /// Deterministic method for tests — does not depend on global state.
    pub fn text(&self, language: Language) -> String {
        match language {
            Language::English => en(self),
            Language::Portuguese => pt(self),
        }
    }
}

/// Initializes i18n by resolving locale (5-layer precedence) and publishing
/// once to the global [`crate::locale`] `OnceLock`.
///
/// `force_lang` is the CLI `--lang` value (already clap-validated when present).
/// `config_dir_override` is `--config-dir` for persisted preference lookup.
pub fn initialize_language(
    force_lang: Option<&str>,
    config_dir_override: Option<&std::path::Path>,
) -> Result<()> {
    let resolution =
        crate::locale::resolve_language_detailed(force_lang, config_dir_override);
    tracing::debug!(
        target: "ssh_cli::i18n",
        language = resolution.language.bcp47(),
        source = resolution.source.as_str(),
        "locale resolved"
    );
    crate::locale::set_language(resolution.language);
    Ok(())
}

/// Returns the currently configured language.
#[must_use]
pub fn current_language() -> Language {
    crate::locale::current_language()
}

/// Returns the message string in the current global language.
///
/// Usa o estado global inicializado por `initialize_language`.
/// In tests, prefer `Message::text(language)` for determinism.
///
/// # Examples
///
/// ```
/// use ssh_cli::i18n::{t, initialize_language, Message};
///
/// initialize_language(Some("en"), None).unwrap();
/// let text = t(Message::VpsRegistryEmpty);
/// assert!(!text.is_empty());
/// ```
/// Takes [`Message`] by value: call sites construct ephemeral messages with
/// owned payloads; consuming them is intentional (not a needless copy).
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn t(msg: Message) -> String {
    msg.text(current_language())
}

/// American English translations.
fn en(msg: &Message) -> String {
    match msg {
        Message::VpsRegistryEmpty => "No VPS registered.".to_string(),
        Message::VpsListTitle => "Registered VPS:".to_string(),
        Message::VpsAdded { name } => format!("VPS '{name}' added successfully."),
        Message::VpsRemoved { name } => format!("VPS '{name}' removed successfully."),
        Message::VpsDuplicate { name } => format!("VPS '{name}' is already registered."),
        Message::VpsNotFound { name } => format!("VPS '{name}' not found."),
        Message::VpsActiveSelected { name } => format!("Active VPS: '{name}'."),
        Message::ConfigPathLabel => "Configuration file:".to_string(),
        Message::ConfigPath { path } => path.clone(),
        Message::ConfigNoKeys => "No API keys configured.".to_string(),
        Message::ErrorLoadConfig => "Failed to load configuration.".to_string(),
        Message::ErrorSaveConfig => "Failed to save configuration.".to_string(),
        Message::ErrorSshConnection => "SSH connection error.".to_string(),
        Message::ErrorCommandFailed => "Command execution failed.".to_string(),
        Message::ErrorInvalidArgument { detail } => format!("Invalid argument: {detail}"),
        Message::ErrorGeneric { detail } => detail.clone(),
        Message::VpsEdited { name } => format!("VPS '{name}' edited."),
        Message::ExportCompleted { path } => format!("exported to {path}"),
        Message::ImportCompleted => "import completed".to_string(),
        Message::PrimaryKeyReady { source, key_file } => {
            format!("primary-key ready (source={source}; key_file={key_file})")
        }
        Message::ReencryptCompleted { hosts } => {
            format!("re-encrypt completed for {hosts} host(s)")
        }
        Message::Success { detail } => detail.clone(),
        Message::TunnelActive {
            local_port,
            remote_host,
            remote_port,
            vps_name,
        } => format!(
            "SSH tunnel active: {}:{local_port} -> {remote_host}:{remote_port} via {vps_name}",
            crate::constants::DEFAULT_TUNNEL_BIND_ADDR
        ),
        Message::TunnelPressCtrlC => "Press Ctrl+C to terminate.".to_string(),
        Message::HealthCheckOk { name } => format!("Health check passed for '{name}'."),
        Message::HealthCheckNoVps => {
            "No active VPS. Use 'ssh-cli connect <NAME>' first.".to_string()
        }
        Message::HealthCheckFailed { name, detail } => {
            format!("Health check FAILED for '{name}': {detail}")
        }
        Message::HealthCheckLatency { name, latency_ms } => {
            format!("Health check OK for '{name}' ({latency_ms}ms)")
        }
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
        Message::LocalePreferenceSaved { lang, path } => {
            format!("language preference saved: {lang} ({path})")
        }
        Message::LocalePreferenceCleared => "language preference cleared.".to_string(),
        Message::LocaleStatusTitle => "Locale status:".to_string(),
    }
}

/// Brazilian Portuguese translations.
fn pt(msg: &Message) -> String {
    match msg {
        Message::VpsRegistryEmpty => "Nenhum VPS cadastrado.".to_string(),
        Message::VpsListTitle => "VPS cadastrados:".to_string(),
        Message::VpsAdded { name } => format!("VPS '{name}' adicionada com sucesso."),
        Message::VpsRemoved { name } => format!("VPS '{name}' removida com sucesso."),
        Message::VpsDuplicate { name } => format!("VPS '{name}' já está cadastrada."),
        Message::VpsNotFound { name } => format!("VPS '{name}' não encontrada."),
        Message::VpsActiveSelected { name } => format!("VPS ativa: '{name}'."),
        Message::ConfigPathLabel => "Arquivo de configuração:".to_string(),
        Message::ConfigPath { path } => path.clone(),
        Message::ConfigNoKeys => "Nenhuma chave de API configurada.".to_string(),
        Message::ErrorLoadConfig => "Falha ao carregar configuração.".to_string(),
        Message::ErrorSaveConfig => "Falha ao salvar configuração.".to_string(),
        Message::ErrorSshConnection => "Erro de conexão SSH.".to_string(),
        Message::ErrorCommandFailed => "Falha na execução do comando.".to_string(),
        Message::ErrorInvalidArgument { detail } => format!("Argumento inválido: {detail}"),
        Message::ErrorGeneric { detail } => detail.clone(),
        Message::VpsEdited { name } => format!("VPS '{name}' editada."),
        Message::ExportCompleted { path } => format!("exportado para {path}"),
        Message::ImportCompleted => "importação concluída".to_string(),
        Message::PrimaryKeyReady { source, key_file } => {
            format!("primary-key pronta (source={source}; key_file={key_file})")
        }
        Message::ReencryptCompleted { hosts } => {
            format!("re-cifragem concluída para {hosts} host(s)")
        }
        Message::Success { detail } => detail.clone(),
        Message::TunnelActive {
            local_port,
            remote_host,
            remote_port,
            vps_name,
        } => format!(
            "Tunnel SSH: {}:{local_port} -> {remote_host}:{remote_port} via {vps_name}",
            crate::constants::DEFAULT_TUNNEL_BIND_ADDR
        ),
        Message::TunnelPressCtrlC => "Pressione Ctrl+C para encerrar.".to_string(),
        Message::HealthCheckOk { name } => format!("Health check bem-sucedido para '{name}'."),
        Message::HealthCheckNoVps => {
            "Nenhuma VPS ativa. Use 'ssh-cli connect <NOME>' primeiro.".to_string()
        }
        Message::HealthCheckFailed { name, detail } => {
            format!("Health check FALHOU para '{name}': {detail}")
        }
        Message::HealthCheckLatency { name, latency_ms } => {
            format!("Health check OK para '{name}' ({latency_ms}ms)")
        }
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
        Message::LocalePreferenceSaved { lang, path } => {
            format!("preferência de idioma salva: {lang} ({path})")
        }
        Message::LocalePreferenceCleared => "preferência de idioma removida.".to_string(),
        Message::LocaleStatusTitle => "Status do locale:".to_string(),
    }
}


#[cfg(test)]
#[path = "i18n_tests.rs"]
mod tests;
