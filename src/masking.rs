// SPDX-License-Identifier: MIT OR Apache-2.0
//! Masking of sensitive values (passwords, tokens) — agent-safe.
//!
//! Rule (GAP-SSH-SEC-002): **always** returns `"***"`, without exposing prefix/suffix.

/// Fixed placeholder for any sensitive value.
pub const FIXED_MASK: &str = "***";

/// Masks a sensitive value without leaking useful characters.
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
    fn empty_value_returns_triple_asterisk() {
        assert_eq!(mask(""), "***");
    }

    #[test]
    fn short_value_returns_triple_asterisk() {
        assert_eq!(mask("abc"), "***");
    }

    #[test]
    fn long_value_never_exposes_prefix_or_suffix() {
        let password = "senha-secreta-muito-longa-aqui-123456";
        assert_eq!(mask(password), "***");
        assert!(!mask(password).contains("senha"));
        assert!(!mask(password).contains("1234"));
    }

    #[test]
    fn unicode_value_does_not_crash() {
        let emojis = "🔒🔑🛡🔐✨🎉💎⚡🌟🔥🎨🚀🌈🍀🎯🎪🎭🎬🎮🎲";
        assert_eq!(mask(emojis), "***");
    }
}
