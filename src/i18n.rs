// SPDX-License-Identifier: MIT OR Apache-2.0
//! ssh-cli internationalization system.
//!
//! Provides bilingual `Language` with `Message` as the single source of
//! UI strings. Locale detection is delegated to the `locale` module.
//!
//! Language selection precedence:
//! 1. CLI `--lang` flag
//! 2. `SSH_CLI_LANG` environment variable
//! 3. System locale via `sys_locale::get_locale()`
//! 4. Fallback: `Language::English`

use anyhow::Result;

/// Languages supported by the internationalization system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// American English (en-US) — default language.
    English,
    /// Brazilian Portuguese (pt-BR).
    Portuguese,
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

/// Initializes i18n by detecting the OS locale.
///
/// If `force_lang` is `Some(...)`, it overrides automatic detection.
pub fn initialize_language(force_lang: Option<&str>) -> Result<()> {
    let language = crate::locale::resolve_language(force_lang);
    crate::locale::set_language(language);
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
/// initialize_language(Some("en-US")).unwrap();
/// let text = t(Message::VpsRegistryEmpty);
/// assert!(!text.is_empty());
/// ```
#[must_use]
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
            "SSH tunnel active: localhost:{local_port} -> {remote_host}:{remote_port} via {vps_name}"
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
            "Tunnel SSH: localhost:{local_port} -> {remote_host}:{remote_port} via {vps_name}"
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_enum_is_copy() {
        let a = Language::English;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn message_is_not_copy_but_is_clone() {
        let m = Message::VpsAdded {
            name: "vps-01".to_string(),
        };
        let m2 = m.clone();
        assert_eq!(m, m2);
    }

    #[test]
    fn vps_registry_empty_en() {
        assert_eq!(
            Message::VpsRegistryEmpty.text(Language::English),
            "No VPS registered."
        );
    }

    #[test]
    fn vps_registry_empty_pt() {
        assert_eq!(
            Message::VpsRegistryEmpty.text(Language::Portuguese),
            "Nenhum VPS cadastrado."
        );
    }

    #[test]
    fn vps_added_includes_name_en() {
        let msg = Message::VpsAdded {
            name: "prod-01".to_string(),
        };
        assert_eq!(
            msg.text(Language::English),
            "VPS 'prod-01' added successfully."
        );
    }

    #[test]
    fn vps_added_includes_name_pt() {
        let msg = Message::VpsAdded {
            name: "prod-01".to_string(),
        };
        assert_eq!(
            msg.text(Language::Portuguese),
            "VPS 'prod-01' adicionada com sucesso."
        );
    }

    #[test]
    fn vps_removed_includes_name() {
        let msg = Message::VpsRemoved {
            name: "dev-01".to_string(),
        };
        assert!(msg.text(Language::English).contains("dev-01"));
        assert!(msg.text(Language::Portuguese).contains("dev-01"));
    }

    #[test]
    fn vps_duplicate_includes_name() {
        let msg = Message::VpsDuplicate {
            name: "staging".to_string(),
        };
        assert!(msg.text(Language::English).contains("staging"));
        assert!(msg.text(Language::Portuguese).contains("staging"));
    }

    #[test]
    fn vps_not_found_includes_name() {
        let msg = Message::VpsNotFound {
            name: "inexistente".to_string(),
        };
        assert!(msg.text(Language::English).contains("inexistente"));
        assert!(msg.text(Language::Portuguese).contains("inexistente"));
    }

    #[test]
    fn tunnel_active_includes_all_fields() {
        let msg = Message::TunnelActive {
            local_port: 8080,
            remote_host: "1.2.3.4".to_string(),
            remote_port: 22,
            vps_name: "meu-servidor".to_string(),
        };
        let en = msg.text(Language::English);
        assert!(en.contains("8080"));
        assert!(en.contains("1.2.3.4"));
        assert!(en.contains("22"));
        assert!(en.contains("meu-servidor"));
    }

    #[test]
    fn error_invalid_argument_includes_detail() {
        let msg = Message::ErrorInvalidArgument {
            detail: "port out of range".to_string(),
        };
        assert!(msg
            .text(Language::English)
            .contains("port out of range"));
        assert!(msg
            .text(Language::Portuguese)
            .contains("port out of range"));
    }

    #[test]
    fn health_check_ok_includes_name() {
        let msg = Message::HealthCheckOk {
            name: "prod-01".to_string(),
        };
        assert!(msg.text(Language::English).contains("prod-01"));
        assert!(msg.text(Language::Portuguese).contains("prod-01"));
    }

    #[test]
    fn all_unit_variants_en_nonempty() {
        let unit_variants = [
            Message::VpsRegistryEmpty,
            Message::VpsListTitle,
            Message::ConfigPathLabel,
            Message::ConfigNoKeys,
            Message::ErrorLoadConfig,
            Message::ErrorSaveConfig,
            Message::ErrorSshConnection,
            Message::ErrorCommandFailed,
            Message::TunnelPressCtrlC,
            Message::HealthCheckNoVps,
            Message::OperationCancelled,
        ];
        for v in &unit_variants {
            let text = v.text(Language::English);
            assert!(!text.is_empty(), "empty EN for {:?}", v);
        }
    }

    #[test]
    fn all_unit_variants_pt_nonempty() {
        let unit_variants = [
            Message::VpsRegistryEmpty,
            Message::VpsListTitle,
            Message::ConfigPathLabel,
            Message::ConfigNoKeys,
            Message::ErrorLoadConfig,
            Message::ErrorSaveConfig,
            Message::ErrorSshConnection,
            Message::ErrorCommandFailed,
            Message::TunnelPressCtrlC,
            Message::HealthCheckNoVps,
            Message::OperationCancelled,
        ];
        for v in &unit_variants {
            let text = v.text(Language::Portuguese);
            assert!(!text.is_empty(), "empty PT for {:?}", v);
        }
    }

    #[test]
    fn pt_translations_differ_from_en_for_units() {
        let pairs = [
            (Message::VpsRegistryEmpty, Message::VpsRegistryEmpty),
            (Message::ErrorSshConnection, Message::ErrorSshConnection),
            (Message::HealthCheckNoVps, Message::HealthCheckNoVps),
            (Message::OperationCancelled, Message::OperationCancelled),
        ];
        for (a, b) in &pairs {
            let en = a.text(Language::English);
            let pt = b.text(Language::Portuguese);
            assert_ne!(en, pt, "EN == PT for {:?}", a);
        }
    }

    #[test]
    fn health_check_failed_includes_name_and_detail() {
        let msg = Message::HealthCheckFailed {
            name: "prod-01".to_string(),
            detail: "timeout".to_string(),
        };
        assert!(msg.text(Language::English).contains("prod-01"));
        assert!(msg.text(Language::English).contains("timeout"));
        assert!(msg.text(Language::Portuguese).contains("prod-01"));
        assert!(msg.text(Language::Portuguese).contains("timeout"));
    }

    #[test]
    fn health_check_latency_includes_name_and_ms() {
        let msg = Message::HealthCheckLatency {
            name: "relay-01".to_string(),
            latency_ms: 42,
        };
        assert!(msg.text(Language::English).contains("relay-01"));
        assert!(msg.text(Language::English).contains("42"));
        assert!(msg.text(Language::Portuguese).contains("relay-01"));
        assert!(msg.text(Language::Portuguese).contains("42"));
    }

    #[test]
    fn initialize_language_without_force_no_panic() {
        let result = initialize_language(None);
        assert!(result.is_ok());
    }

    #[test]
    fn initialize_language_with_pt_br_works() {
        let result = initialize_language(Some("pt-BR"));
        assert!(result.is_ok());
    }

    #[test]
    fn current_language_returns_valid_value() {
        let language = current_language();
        assert!(language == Language::English || language == Language::Portuguese);
    }
}
