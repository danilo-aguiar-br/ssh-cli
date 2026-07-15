// SPDX-License-Identifier: MIT OR Apache-2.0
//! Validação e normalização de caminhos de arquivo.
//!
//! Fornece funções para validate nomes de arquivo de forma segura e
//! cross-platform, prevenindo path traversal, nomes reservados do Windows
//! e caracteres proibidos.

use anyhow::{bail, Result};
use unicode_normalization::UnicodeNormalization;

/// Nomes reservados pelo sistema de arquivos do Windows (case-insensitive).
const NOMES_RESERVADOS_WINDOWS: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Caracteres proibidos em nomes de arquivo (proibidos no Windows ou problemáticos
/// em sistemas Unix).
const CHARS_PROIBIDOS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];

/// Valida um name de arquivo (sem separadores de path).
///
/// Rejeita:
/// - Strings vazias.
/// - Nomes com componentes `..` (path traversal).
/// - Caracteres proibidos.
/// - Nomes reservados do Windows (case-insensitive).
/// - Nomes que terminam com ponto ou espaço (problemáticos no Windows).
///
/// # Examples
///
/// ```
/// use ssh_cli::paths::validate_name;
///
/// assert!(validate_name("meu-servidor").is_ok());
/// assert!(validate_name("../etc/passwd").is_err());
/// assert!(validate_name("CON").is_err());
/// ```
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("nome de arquivo não pode ser vazio");
    }

    if name.contains("..") {
        bail!("nome de arquivo contém componente de path traversal: '{name}'");
    }

    for c in CHARS_PROIBIDOS {
        if name.contains(*c) {
            bail!(
                "nome de arquivo contém caractere proibido '{}': '{name}'",
                c.escape_default()
            );
        }
    }

    let nome_upper = name.to_uppercase();
    // Verifica também sem extensão (ex.: "NUL.txt" é proibido no Windows)
    let raiz = nome_upper.split('.').next().unwrap_or(&nome_upper);
    if NOMES_RESERVADOS_WINDOWS.contains(&raiz) {
        bail!("nome de arquivo usa nome reservado do Windows: '{name}'");
    }

    if name.ends_with('.') || name.ends_with(' ') {
        bail!("nome de arquivo não pode terminar com ponto ou espaço: '{name}'");
    }

    Ok(())
}

/// Normaliza um name de arquivo para a forma NFC do Unicode.
///
/// A normalização NFC é necessária para garantir comparações consistentes
/// entre diferentes sistemas operacionais (macOS usa NFD, Linux usa NFC).
#[must_use]
pub fn normalizar_nfc(name: &str) -> String {
    name.nfc().collect()
}

/// Valida e normaliza um name de arquivo em uma única operação.
///
/// Retorna o name normalizado para NFC se passar em todas as validações.
pub fn validate_and_normalize(name: &str) -> Result<String> {
    validate_name(name)?;
    Ok(normalizar_nfc(name))
}

/// Valida que um path não contém componentes de path traversal.
///
/// Verifica todos os segmentos do path separados por `/` ou `\`.
pub fn validate_no_traversal(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("caminho não pode ser vazio");
    }

    let segmentos = path.split(['/', '\\']);
    for segmento in segmentos {
        if segmento == ".." {
            bail!("caminho contém componente de path traversal: '{path}'");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nome_valido_comum_passa() {
        assert!(validate_name("meu-servidor").is_ok());
        assert!(validate_name("vps_01").is_ok());
        assert!(validate_name("servidor.produção").is_ok());
    }

    #[test]
    fn nome_vazio_rejeitado() {
        assert!(validate_name("").is_err());
    }

    #[test]
    fn path_traversal_rejeitado() {
        assert!(validate_name("..").is_err());
        assert!(validate_name("../etc/passwd").is_err());
        assert!(validate_name("foo/../bar").is_err());
    }

    #[test]
    fn chars_proibidos_rejeitados() {
        assert!(validate_name("foo/bar").is_err());
        assert!(validate_name("foo\\bar").is_err());
        assert!(validate_name("foo:bar").is_err());
        assert!(validate_name("foo*bar").is_err());
        assert!(validate_name("foo?bar").is_err());
    }

    #[test]
    fn nomes_reservados_windows_rejeitados() {
        assert!(validate_name("CON").is_err());
        assert!(validate_name("con").is_err());
        assert!(validate_name("NUL.txt").is_err());
        assert!(validate_name("COM1").is_err());
        assert!(validate_name("LPT9").is_err());
    }

    #[test]
    fn nome_terminando_com_ponto_rejeitado() {
        assert!(validate_name("arquivo.").is_err());
    }

    #[test]
    fn nome_terminando_com_espaco_rejeitado() {
        assert!(validate_name("arquivo ").is_err());
    }

    #[test]
    fn normalizar_nfc_retorna_string() {
        let resultado = normalizar_nfc("servidor");
        assert_eq!(resultado, "servidor");
    }

    #[test]
    fn validar_e_normalizar_retorna_string_valida() {
        let resultado = validate_and_normalize("meu-servidor").unwrap();
        assert_eq!(resultado, "meu-servidor");
    }

    #[test]
    fn validar_sem_traversal_aceita_caminho_normal() {
        assert!(validate_no_traversal("/home/usuario/arquivo.txt").is_ok());
        assert!(validate_no_traversal("relative/path/file.txt").is_ok());
    }

    #[test]
    fn validar_sem_traversal_rejeita_traversal() {
        assert!(validate_no_traversal("/home/../etc/passwd").is_err());
        assert!(validate_no_traversal("../secreto").is_err());
    }

    #[test]
    fn validar_sem_traversal_rejeita_vazio() {
        assert!(validate_no_traversal("").is_err());
    }

    #[test]
    fn nome_com_acentos_brasileiros_valido() {
        assert!(validate_name("produção").is_ok());
        assert!(validate_name("ação-configuração").is_ok());
    }

    #[test]
    fn nome_com_unicode_cjk_valido() {
        assert!(validate_name("server-\u{4e16}\u{754c}").is_ok());
    }

    #[test]
    fn nome_com_emoji_valido() {
        assert!(validate_name("server-\u{1f680}").is_ok());
    }

    #[test]
    fn nome_windows_reservado_case_misto_rejeitado() {
        assert!(validate_name("cOn").is_err());
        assert!(validate_name("Nul").is_err());
        assert!(validate_name("lPt1").is_err());
    }

    #[test]
    fn normalizar_nfc_converte_nfd_para_nfc() {
        let nfd = "e\u{0301}"; // e + combining acute
        let nfc = "\u{00e9}"; // é precomposed
        assert_eq!(normalizar_nfc(nfd), nfc);
    }

    #[test]
    fn normalizar_nfc_preserva_nfc() {
        let nfc = "\u{00e9}";
        assert_eq!(normalizar_nfc(nfc), nfc);
    }

    #[test]
    fn normalizar_nfc_idempotente() {
        let input = "cafe\u{0301}";
        let once = normalizar_nfc(input);
        let twice = normalizar_nfc(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn validar_e_normalizar_nfd_converte() {
        let resultado = validate_and_normalize("cafe\u{0301}").unwrap();
        assert_eq!(resultado, "caf\u{00e9}");
    }

    #[test]
    fn validar_sem_traversal_com_backslash_rejeitado() {
        assert!(validate_no_traversal("foo\\..\\bar").is_err());
    }

    #[test]
    fn validar_sem_traversal_dot_solo_aceita() {
        assert!(validate_no_traversal("./arquivo").is_ok());
    }
}
