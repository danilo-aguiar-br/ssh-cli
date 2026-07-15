// SPDX-License-Identifier: MIT OR Apache-2.0
//! Masking of sensitive values (passwords, tokens) — agent-safe.
//!
//! Rule (GAP-SSH-SEC-002): **always** returns `"***"`, without exposing prefix/suffix.

/// Placeholder fixo para qualquer valor sensível.
pub const FIXED_MASK: &str = "***";

/// Mascara um valor sensível sem vazar caracteres úteis.
///
/// # Examples
///
/// ```
/// use ssh_cli::masking::mask;
///
/// assert_eq!(mask("curto"), "***");
/// assert_eq!(mask("password-secreta-muito-longa-aqui-123456"), "***");
/// ```
#[must_use]
pub fn mask(_valor: &str) -> String {
    FIXED_MASK.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valor_vazio_retorna_triplo_asterisco() {
        assert_eq!(mask(""), "***");
    }

    #[test]
    fn valor_curto_retorna_triplo_asterisco() {
        assert_eq!(mask("abc"), "***");
    }

    #[test]
    fn valor_longo_nunca_expoe_prefixo_ou_sufixo() {
        let password = "senha-secreta-muito-longa-aqui-123456";
        assert_eq!(mask(password), "***");
        assert!(!mask(password).contains("senha"));
        assert!(!mask(password).contains("1234"));
    }

    #[test]
    fn valor_com_unicode_nao_crasha() {
        let emojis = "🔒🔑🛡🔐✨🎉💎⚡🌟🔥🎨🚀🌈🍀🎯🎪🎭🎬🎮🎲";
        assert_eq!(mask(emojis), "***");
    }
}
