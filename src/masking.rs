// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Masking of sensitive values (passwords, tokens) — agent-safe.
//!
//! Rule (GAP-SSH-SEC-002): **always** returns `"***"`, without exposing prefix/suffix.
//!
//! Performance: returns `&'static str` (zero heap). Callers that need owned
//! values can `.to_string()` / `.into()` at the edge — never allocate in the
//! mask path itself (Rules Rust: treat every allocation as measurable cost).

/// Fixed placeholder for any sensitive value.
pub const FIXED_MASK: &str = "***";

/// Masks a sensitive value without leaking useful characters.
///
/// Always returns the static placeholder [`FIXED_MASK`]. No heap allocation.
///
/// # Examples
///
/// ```
/// use ssh_cli::masking::mask;
///
/// assert_eq!(mask("curto"), "***");
/// assert_eq!(mask("password-secreta-muito-longa-aqui-123456"), "***");
/// assert_eq!(mask("a"), ssh_cli::masking::FIXED_MASK);
/// ```
#[must_use]
#[inline]
pub fn mask(_valor: &str) -> &'static str {
    FIXED_MASK
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

    #[test]
    fn mask_returns_static_placeholder() {
        // API contract: &'static str equal to FIXED_MASK (no owned String).
        let m: &'static str = mask("secret");
        assert_eq!(m, FIXED_MASK);
        assert_eq!(m, "***");
    }
}
