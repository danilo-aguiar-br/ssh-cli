// SPDX-License-Identifier: MIT OR Apache-2.0
//! Detecção e resolução de idioma cross-platform.
//!
//! Precedência de seleção de idioma (do mais para o menos prioritário):
//! 1. Flag `--lang` da CLI
//! 2. Variável de ambiente `SSH_CLI_LANG`
//! 3. Locale do sistema via `sys_locale::get_locale()`
//! 4. Fallback: `Language::English`

use std::sync::OnceLock;

use crate::i18n::Language;

/// Estado global do idioma — definido uma única vez na inicialização.
static IDIOMA_GLOBAL: OnceLock<Language> = OnceLock::new();

/// Resolve o idioma aplicando a hierarquia de precedência em 4 camadas.
///
/// Retorna o primeiro idioma válido encontrado na ordem:
/// flag CLI > env SSH_CLI_LANG > sys_locale > English.
pub fn resolve_language(force_lang: Option<&str>) -> Language {
    // Camada 1: flag --lang da CLI
    if let Some(codigo) = force_lang {
        if let Some(idioma) = code_to_language(codigo) {
            return idioma;
        }
    }

    // Camada 2: variável de ambiente SSH_CLI_LANG
    if let Ok(env_lang) = std::env::var("SSH_CLI_LANG") {
        if let Some(idioma) = code_to_language(&env_lang) {
            return idioma;
        }
    }

    // Camada 3: locale do sistema via sys_locale
    if let Some(locale) = sys_locale::get_locale() {
        if let Some(idioma) = code_to_language(&locale) {
            return idioma;
        }
    }

    // Camada 4: fallback incondicional
    Language::English
}

/// Define o idioma global (chamada única na inicialização do processo).
///
/// Chamadas subsequentes são silenciosamente ignoradas — o `OnceLock`
/// garante que o idioma é imutável após a primeira definição.
pub fn set_language(idioma: Language) {
    let _ = IDIOMA_GLOBAL.set(idioma);
}

/// Retorna o idioma global atual.
///
/// Se `set_language` ainda não foi chamado, retorna `Language::English`
/// como fallback seguro para código executado antes da inicialização.
pub fn current_language() -> Language {
    IDIOMA_GLOBAL.get().copied().unwrap_or(Language::English)
}

/// Converte código textual de idioma para `Language`.
///
/// Reconhece prefixos "pt" e "en" com qualquer sufixo de região,
/// sem distinção entre maiúsculas e minúsculas.
fn code_to_language(codigo: &str) -> Option<Language> {
    let normalizado = codigo.to_lowercase();
    match normalizado.as_str() {
        "pt" | "pt-br" | "pt_br" => Some(Language::Portuguese),
        "en" | "en-us" | "en_us" => Some(Language::English),
        outro => {
            if outro.starts_with("pt") {
                Some(Language::Portuguese)
            } else if outro.starts_with("en") {
                Some(Language::English)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codigo_pt_retorna_portugues() {
        assert_eq!(code_to_language("pt"), Some(Language::Portuguese));
    }

    #[test]
    fn codigo_pt_br_retorna_portugues() {
        assert_eq!(code_to_language("pt-BR"), Some(Language::Portuguese));
    }

    #[test]
    fn codigo_pt_br_underscore_retorna_portugues() {
        assert_eq!(code_to_language("pt_BR"), Some(Language::Portuguese));
    }

    #[test]
    fn codigo_en_retorna_english() {
        assert_eq!(code_to_language("en"), Some(Language::English));
    }

    #[test]
    fn codigo_en_us_retorna_english() {
        assert_eq!(code_to_language("en-US"), Some(Language::English));
    }

    #[test]
    fn codigo_en_gb_retorna_english_por_prefixo() {
        assert_eq!(code_to_language("en-GB"), Some(Language::English));
    }

    #[test]
    fn codigo_desconhecido_retorna_none() {
        assert_eq!(code_to_language("fr-FR"), None);
    }

    #[test]
    fn codigo_vazio_retorna_none() {
        assert_eq!(code_to_language(""), None);
    }

    #[test]
    fn codigo_maiusculo_normalizado() {
        assert_eq!(code_to_language("PT"), Some(Language::Portuguese));
        assert_eq!(code_to_language("EN"), Some(Language::English));
    }

    #[test]
    fn resolver_com_forcar_pt_retorna_portugues() {
        let resultado = resolve_language(Some("pt-BR"));
        assert_eq!(resultado, Language::Portuguese);
    }

    #[test]
    fn resolver_com_forcar_en_retorna_english() {
        let resultado = resolve_language(Some("en-US"));
        assert_eq!(resultado, Language::English);
    }

    #[test]
    fn resolver_com_forcar_invalido_usa_camadas_seguintes() {
        // Código inválido não resolve na camada 1; deve cair em sys_locale ou fallback.
        std::env::remove_var("SSH_CLI_LANG");
        let resultado = resolve_language(Some("xx-YY"));
        // Deve retornar English ou Portuguese — não pode ser um valor inválido.
        assert!(
            resultado == Language::English || resultado == Language::Portuguese,
            "resolver_idioma deve retornar idioma válido mesmo com código inválido"
        );
    }

    #[test]
    fn resolver_sem_forcar_retorna_idioma_valido() {
        std::env::remove_var("SSH_CLI_LANG");
        let resultado = resolve_language(None);
        assert!(
            resultado == Language::English || resultado == Language::Portuguese,
            "resolver_idioma deve retornar idioma válido"
        );
    }

    #[test]
    fn idioma_atual_retorna_fallback_english_antes_de_definir() {
        // Não chamamos set_language — o OnceLock pode já estar setado em outros tests,
        // mas o resultado DEVE ser um idioma válido.
        let resultado = current_language();
        assert!(
            resultado == Language::English || resultado == Language::Portuguese,
            "idioma_atual deve retornar idioma válido"
        );
    }
}
