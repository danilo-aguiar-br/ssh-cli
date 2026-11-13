// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Escaping of untrusted text before it reaches a terminal.
//!
//! Host names, users, tags and paths come from config files, argv or remote
//! servers. A raw ESC, BEL or right-to-left override in them can rewrite what
//! the operator sees, so the text paths pass them through [`escape_untrusted`].
//! The JSON paths do not need this: `serde_json` already escapes control characters.

use crate::constants::UNTRUSTED_FORMAT_CHAR_RANGES;
use std::borrow::Cow;

/// Returns whether `c` is a control, format or bidirectional character.
///
/// These are the characters that are rejected in a stored host name and
/// escaped as `\u{..}` in text output.
///
/// # Examples
///
/// ```
/// use ssh_cli::output::sanitize::is_untrusted_char;
///
/// assert!(is_untrusted_char('\u{1b}'));
/// assert!(is_untrusted_char('\u{202e}'));
/// assert!(!is_untrusted_char('a'));
/// ```
#[must_use]
pub fn is_untrusted_char(c: char) -> bool {
    c.is_control()
        || UNTRUSTED_FORMAT_CHAR_RANGES
            .iter()
            .any(|&(first, last)| (first..=last).contains(&c))
}

/// Escapes every control, format and bidirectional character as `\u{..}`.
///
/// Returns [`Cow::Borrowed`] when nothing needs escaping, so the common case
/// allocates nothing. Use [`escape_untrusted_keep_whitespace`] for messages
/// that legitimately span several lines.
///
/// # Examples
///
/// ```
/// use ssh_cli::output::sanitize::escape_untrusted;
///
/// assert_eq!(escape_untrusted("lab-01"), "lab-01");
/// assert_eq!(escape_untrusted("a\u{1b}[31m"), "a\\u{1b}[31m");
/// assert_eq!(escape_untrusted("a\nb"), "a\\u{a}b");
/// ```
#[must_use]
pub fn escape_untrusted(s: &str) -> Cow<'_, str> {
    escape_where(s, is_untrusted_char)
}

/// Like [`escape_untrusted`] but leaves `\n` and `\t` as they are.
///
/// # Examples
///
/// ```
/// use ssh_cli::output::sanitize::escape_untrusted_keep_whitespace;
///
/// assert_eq!(escape_untrusted_keep_whitespace("a\n\tb"), "a\n\tb");
/// assert_eq!(escape_untrusted_keep_whitespace("a\u{7}\nb"), "a\\u{7}\nb");
/// ```
#[must_use]
pub fn escape_untrusted_keep_whitespace(s: &str) -> Cow<'_, str> {
    escape_where(s, |c| c != '\n' && c != '\t' && is_untrusted_char(c))
}

fn escape_where(s: &str, needs_escape: impl Fn(char) -> bool) -> Cow<'_, str> {
    let Some(first) = s.find(&needs_escape) else {
        return Cow::Borrowed(s);
    };
    let mut out = String::with_capacity(s.len() + 8);
    out.push_str(&s[..first]);
    for c in s[first..].chars() {
        if needs_escape(c) {
            out.extend(c.escape_unicode());
        } else {
            out.push(c);
        }
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_text_is_borrowed() {
        assert!(matches!(
            escape_untrusted("host-01.example"),
            Cow::Borrowed(_)
        ));
        assert!(matches!(escape_untrusted("produção"), Cow::Borrowed(_)));
        assert!(matches!(
            escape_untrusted_keep_whitespace("a\nb\tc"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn controls_and_bidi_are_escaped() {
        assert_eq!(escape_untrusted("x\u{1b}y"), "x\\u{1b}y");
        assert_eq!(escape_untrusted("x\u{7}"), "x\\u{7}");
        assert_eq!(escape_untrusted("\u{202e}x"), "\\u{202e}x");
        assert_eq!(escape_untrusted("a\u{200b}b"), "a\\u{200b}b");
        assert_eq!(escape_untrusted("\u{feff}"), "\\u{feff}");
        assert_eq!(escape_untrusted("\u{7f}"), "\\u{7f}");
        assert_eq!(escape_untrusted("\u{85}"), "\\u{85}");
        assert!(matches!(escape_untrusted("a\u{1b}"), Cow::Owned(_)));
    }

    #[test]
    fn keep_variant_preserves_newline_and_tab_only() {
        assert_eq!(escape_untrusted_keep_whitespace("a\nb\tc"), "a\nb\tc");
        assert_eq!(
            escape_untrusted_keep_whitespace("a\r\n\u{1b}b"),
            "a\\u{d}\n\\u{1b}b"
        );
    }

    #[test]
    fn boundaries_of_format_ranges() {
        for c in [
            '\u{ad}',
            '\u{61c}',
            '\u{180e}',
            '\u{200b}',
            '\u{200f}',
            '\u{202a}',
            '\u{202e}',
            '\u{2060}',
            '\u{2064}',
            '\u{2066}',
            '\u{2069}',
            '\u{206a}',
            '\u{206f}',
            '\u{feff}',
            '\u{e0000}',
            '\u{e0020}',
            '\u{e007f}',
        ] {
            assert!(is_untrusted_char(c), "{c:?} must be escaped");
        }
        // U+2065 is unassigned and is deliberately left out of the set.
        for c in [
            '\u{ac}',
            '\u{ae}',
            '\u{61b}',
            '\u{61d}',
            '\u{180d}',
            '\u{200a}',
            '\u{2010}',
            '\u{202f}',
            '\u{205f}',
            '\u{2065}',
            '\u{2070}',
            '\u{fefe}',
            '\u{dffff}',
            '\u{e0080}',
        ] {
            assert!(!is_untrusted_char(c), "{c:?} must pass through");
        }
    }

    #[test]
    fn newly_covered_characters_are_escaped_in_text() {
        assert_eq!(escape_untrusted("a\u{ad}b"), "a\\u{ad}b");
        assert_eq!(escape_untrusted("a\u{61c}b"), "a\\u{61c}b");
        assert_eq!(escape_untrusted("a\u{180e}b"), "a\\u{180e}b");
        assert_eq!(escape_untrusted("a\u{206a}b"), "a\\u{206a}b");
        assert_eq!(escape_untrusted("a\u{e0041}b"), "a\\u{e0041}b");
    }
}
