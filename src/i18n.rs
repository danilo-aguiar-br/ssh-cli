// SPDX-License-Identifier: MIT OR Apache-2.0
//! Sistema de internacionalização do ssh-cli.
//!
//! Fornece o enum `Language` bilíngue com enum `Message` como única fonte de
//! strings de UI. A detecção de locale é delegada ao módulo `locale`.
//!
//! Precedência de seleção de idioma:
//! 1. Flag `--lang` da CLI
//! 2. Variável de ambiente `SSH_CLI_LANG`
//! 3. Locale do sistema via `sys_locale::get_locale()`
//! 4. Fallback: `Language::English`

use anyhow::Result;

/// Language suportado pelo sistema de internacionalização.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// Inglês americano (en-US) — idioma padrão.
    English,
    /// Português brasileiro (pt-BR).
    Portuguese,
}

/// Todas as mensagens de UI do sistema.
///
/// ÚNICA fonte de strings visíveis ao usuário. Cada variante possui tradução
/// exaustiva em `en()` e `pt()`. PROIBIDO usar string literal de UI fora deste enum.
///
/// Variantes com campos dinâmicos (ex.: `{ name: String }`) permitem incluir
/// dados contextuais na mensagem. Message não implementa `Copy` pois campos
/// `String` não são `Copy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    // VPS
    /// Nenhuma VPS cadastrada no arquivo de configuração.
    VpsRegistryEmpty,
    /// Cabeçalho da listagem de VPS registradas.
    VpsListTitle,
    /// VPS adicionada com sucesso ao registro.
    VpsAdded {
        /// Nome da VPS adicionada.
        name: String,
    },
    /// VPS removida com sucesso do registro.
    VpsRemoved {
        /// Nome da VPS removida.
        name: String,
    },
    /// Tentativa de adicionar VPS já existente no registro.
    VpsDuplicate {
        /// Nome da VPS duplicada.
        name: String,
    },
    /// VPS solicitada não foi encontrada no registro.
    VpsNotFound {
        /// Nome da VPS não encontrada.
        name: String,
    },
    /// VPS ativa selecionada para operações subsequentes.
    VpsActiveSelected {
        /// Nome da VPS selecionada.
        name: String,
    },
    // Config
    /// Rótulo do path do arquivo de configuração.
    ConfigPathLabel,
    /// Caminho atual do arquivo de configuração.
    ConfigPath {
        /// Caminho absoluto do arquivo de configuração.
        path: String,
    },
    /// Nenhuma chave de API configurada no sistema.
    ConfigNoKeys,
    // Erros
    /// Falha ao carregar o arquivo de configuração.
    ErrorLoadConfig,
    /// Falha ao salvar o arquivo de configuração.
    ErrorSaveConfig,
    /// Erro ao estabelecer conexão SSH com o servidor remoto.
    ErrorSshConnection,
    /// Falha na execução de command remoto via SSH.
    ErrorCommandFailed,
    /// Argumento inválido fornecido à operação.
    ErrorInvalidArgument {
        /// Detalhe do argumento inválido.
        detail: String,
    },
    /// Erro genérico com descrição textual.
    ErrorGeneric {
        /// Descrição do erro.
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
    /// Tunnel SSH ativo com informações de port e host.
    TunnelActive {
        /// Porta local do tunnel.
        local_port: u16,
        /// Host remoto destino.
        remote_host: String,
        /// Porta remota destino.
        remote_port: u16,
        /// Nome da VPS usada como relay.
        vps_name: String,
    },
    /// Instrução para encerrar o tunnel via Ctrl+C.
    TunnelPressCtrlC,
    // Health Check
    /// Verificação de conectividade com VPS bem-sucedida.
    HealthCheckOk {
        /// Nome da VPS verificada.
        name: String,
    },
    /// Nenhuma VPS ativa selecionada para health check.
    HealthCheckNoVps,
    /// Falha na verificação de conectividade com VPS.
    HealthCheckFailed {
        /// Nome da VPS verificada.
        name: String,
        /// Detalhe do erro.
        detail: String,
    },
    /// Resultado de health check com latência.
    HealthCheckLatency {
        /// Nome da VPS verificada.
        name: String,
        /// Latência em milissegundos.
        latency_ms: u64,
    },
    /// Operação cancelada por sinal do usuário (Ctrl+C ou SIGTERM).
    OperationCancelled,
    // SCP (GAP-SSH-SCP-020)
    /// Upload SCP concluído.
    ScpUploadCompleted {
        /// Bytes transferidos.
        bytes: u64,
        /// Duração em milissegundos.
        ms: u64,
    },
    /// Download SCP concluído.
    ScpDownloadCompleted {
        /// Bytes transferidos.
        bytes: u64,
        /// Duração em milissegundos.
        ms: u64,
    },
    /// Upload recusado: path local é diretório (file-only, sem -r).
    ScpUploadFileOnly,
    /// Download recusado: path local já é diretório.
    ScpDownloadLocalNotDirectory,
}

impl Message {
    /// Retorna a string da mensagem no idioma especificado.
    ///
    /// Método determinístico para uso em tests — não depende de estado global.
    pub fn text(&self, idioma: Language) -> String {
        match idioma {
            Language::English => en(self),
            Language::Portuguese => pt(self),
        }
    }
}

/// Inicializa o sistema de i18n detectando o locale do SO.
///
/// Se `force_lang` for `Some(...)`, esse idioma sobrescreve a detecção automática.
pub fn initialize_language(force_lang: Option<&str>) -> Result<()> {
    let idioma = crate::locale::resolve_language(force_lang);
    crate::locale::set_language(idioma);
    Ok(())
}

/// Retorna o idioma atualmente configurado.
#[must_use]
pub fn current_language() -> Language {
    crate::locale::current_language()
}

/// Retorna a string da mensagem no idioma global atual.
///
/// Usa o estado global inicializado por `initialize_language`.
/// Em tests, prefira `Message::texto(idioma)` para determinismo.
///
/// # Examples
///
/// ```
/// use ssh_cli::i18n::{t, initialize_language, Message};
///
/// initialize_language(Some("en-US")).unwrap();
/// let texto = t(Message::VpsRegistryEmpty);
/// assert!(!texto.is_empty());
/// ```
#[must_use]
pub fn t(msg: Message) -> String {
    msg.text(current_language())
}

/// Traduções para inglês americano.
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

/// Traduções para português brasileiro.
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
    fn idioma_enum_e_copy() {
        let a = Language::English;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn mensagem_nao_e_copy_mas_e_clone() {
        let m = Message::VpsAdded {
            name: "vps-01".to_string(),
        };
        let m2 = m.clone();
        assert_eq!(m, m2);
    }

    #[test]
    fn vps_registro_vazio_en() {
        assert_eq!(
            Message::VpsRegistryEmpty.text(Language::English),
            "No VPS registered."
        );
    }

    #[test]
    fn vps_registro_vazio_pt() {
        assert_eq!(
            Message::VpsRegistryEmpty.text(Language::Portuguese),
            "Nenhum VPS cadastrado."
        );
    }

    #[test]
    fn vps_adicionada_inclui_nome_en() {
        let msg = Message::VpsAdded {
            name: "prod-01".to_string(),
        };
        assert_eq!(
            msg.text(Language::English),
            "VPS 'prod-01' added successfully."
        );
    }

    #[test]
    fn vps_adicionada_inclui_nome_pt() {
        let msg = Message::VpsAdded {
            name: "prod-01".to_string(),
        };
        assert_eq!(
            msg.text(Language::Portuguese),
            "VPS 'prod-01' adicionada com sucesso."
        );
    }

    #[test]
    fn vps_removida_inclui_nome() {
        let msg = Message::VpsRemoved {
            name: "dev-01".to_string(),
        };
        assert!(msg.text(Language::English).contains("dev-01"));
        assert!(msg.text(Language::Portuguese).contains("dev-01"));
    }

    #[test]
    fn vps_duplicada_inclui_nome() {
        let msg = Message::VpsDuplicate {
            name: "staging".to_string(),
        };
        assert!(msg.text(Language::English).contains("staging"));
        assert!(msg.text(Language::Portuguese).contains("staging"));
    }

    #[test]
    fn vps_nao_encontrada_inclui_nome() {
        let msg = Message::VpsNotFound {
            name: "inexistente".to_string(),
        };
        assert!(msg.text(Language::English).contains("inexistente"));
        assert!(msg.text(Language::Portuguese).contains("inexistente"));
    }

    #[test]
    fn tunnel_ativo_inclui_todos_os_campos() {
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
    fn erro_argumento_invalido_inclui_detalhe() {
        let msg = Message::ErrorInvalidArgument {
            detail: "porta fora do intervalo".to_string(),
        };
        assert!(msg
            .text(Language::English)
            .contains("porta fora do intervalo"));
        assert!(msg
            .text(Language::Portuguese)
            .contains("porta fora do intervalo"));
    }

    #[test]
    fn health_check_ok_inclui_nome() {
        let msg = Message::HealthCheckOk {
            name: "prod-01".to_string(),
        };
        assert!(msg.text(Language::English).contains("prod-01"));
        assert!(msg.text(Language::Portuguese).contains("prod-01"));
    }

    #[test]
    fn todas_variantes_unitarias_en_nao_vazias() {
        let unitarias = [
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
        for v in &unitarias {
            let texto = v.text(Language::English);
            assert!(!texto.is_empty(), "EN vazia para {:?}", v);
        }
    }

    #[test]
    fn todas_variantes_unitarias_pt_nao_vazias() {
        let unitarias = [
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
        for v in &unitarias {
            let texto = v.text(Language::Portuguese);
            assert!(!texto.is_empty(), "PT vazia para {:?}", v);
        }
    }

    #[test]
    fn traducoes_pt_diferentes_de_en_para_unitarias() {
        let pares = [
            (Message::VpsRegistryEmpty, Message::VpsRegistryEmpty),
            (Message::ErrorSshConnection, Message::ErrorSshConnection),
            (Message::HealthCheckNoVps, Message::HealthCheckNoVps),
            (Message::OperationCancelled, Message::OperationCancelled),
        ];
        for (a, b) in &pares {
            let en = a.text(Language::English);
            let pt = b.text(Language::Portuguese);
            assert_ne!(en, pt, "EN == PT para {:?}", a);
        }
    }

    #[test]
    fn health_check_falhou_inclui_nome_e_detalhe() {
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
    fn health_check_latencia_inclui_nome_e_ms() {
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
    fn inicializar_idioma_sem_forcar_nao_panic() {
        let resultado = initialize_language(None);
        assert!(resultado.is_ok());
    }

    #[test]
    fn inicializar_idioma_com_pt_br_funciona() {
        let resultado = initialize_language(Some("pt-BR"));
        assert!(resultado.is_ok());
    }

    #[test]
    fn idioma_atual_retorna_valor_valido() {
        let idioma = current_language();
        assert!(idioma == Language::English || idioma == Language::Portuguese);
    }
}
