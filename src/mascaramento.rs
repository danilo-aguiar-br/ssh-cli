//! Mascaramento de valores sensíveis (senhas, tokens) — agent-safe.
//!
//! Regra (GAP-SSH-SEC-002): **sempre** retorna `"***"`, sem expor prefixo/sufixo.

/// Placeholder fixo para qualquer valor sensível.
pub const MASCARA_FIXA: &str = "***";

/// Mascara um valor sensível sem vazar caracteres úteis.
///
/// # Exemplos
///
/// ```
/// use ssh_cli::mascaramento::mascarar;
///
/// assert_eq!(mascarar("curto"), "***");
/// assert_eq!(mascarar("senha-secreta-muito-longa-aqui-123456"), "***");
/// ```
#[must_use]
pub fn mascarar(_valor: &str) -> String {
    MASCARA_FIXA.to_string()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn valor_vazio_retorna_triplo_asterisco() {
        assert_eq!(mascarar(""), "***");
    }

    #[test]
    fn valor_curto_retorna_triplo_asterisco() {
        assert_eq!(mascarar("abc"), "***");
    }

    #[test]
    fn valor_longo_nunca_expoe_prefixo_ou_sufixo() {
        let senha = "senha-secreta-muito-longa-aqui-123456";
        assert_eq!(mascarar(senha), "***");
        assert!(!mascarar(senha).contains("senha"));
        assert!(!mascarar(senha).contains("1234"));
    }

    #[test]
    fn valor_com_unicode_nao_crasha() {
        let emojis = "🔒🔑🛡🔐✨🎉💎⚡🌟🔥🎨🚀🌈🍀🎯🎪🎭🎬🎮🎲";
        assert_eq!(mascarar(emojis), "***");
    }
}
