// SPDX-License-Identifier: MIT OR Apache-2.0
// G-TYPE-14 / G-CLOSE: UTF-8 truncation helpers shared by the SSH session path.
#![forbid(unsafe_code)]
//! Session I/O helpers — UTF-8-safe truncation for captured stdout/stderr.
//!
//! Pure CPU, no network. Extracted from [`super::client`] so the exec path and
//! unit tests share one implementation (G-TYPE-14).

/// Truncates a UTF-8 string to at most `max_chars` codepoints.
///
/// Returns `(truncated_string, was_truncated)`. If `max_chars == 0` returns empty string.
/// Unicode-safe: never splits mid-codepoint (slice at `char_indices` boundary).
///
/// Resource: single pass via `char_indices().nth`; fast-path when `content.len() <= max_chars`
/// (each scalar is ≥ 1 byte, so byte length bounds char count from above).
///
/// Prefer [`take_utf8_capped`] on the exec path when you already own a `Vec<u8>` —
/// that reuses the buffer instead of allocating a second copy.
#[must_use]
pub fn truncate_utf8(content: &str, max_chars: usize) -> (String, bool) {
    if max_chars == 0 {
        return (String::new(), !content.is_empty());
    }
    // Fast path: byte len ≤ max_chars ⇒ codepoint count ≤ max_chars.
    if content.len() <= max_chars {
        return (content.to_string(), false);
    }
    match content.char_indices().nth(max_chars) {
        None => (content.to_string(), false),
        Some((idx, _)) => (content[..idx].to_string(), true),
    }
}

/// Decode captured bytes as UTF-8 and apply the codepoint cap, **reusing** the
/// `Vec` allocation on the valid-UTF-8 happy path (one heap buffer total).
///
/// Latency rule: avoid `from_utf8_lossy` + `to_string` double-copy after every exec.
/// Invalid UTF-8 falls back to lossy replacement (same contract as before).
#[must_use]
pub(crate) fn take_utf8_capped(bytes: Vec<u8>, max_chars: usize) -> (String, bool) {
    let s = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
    };
    if max_chars == 0 {
        return (String::new(), !s.is_empty());
    }
    // Fast path: byte len ≤ max_chars ⇒ codepoint count ≤ max_chars.
    if s.len() <= max_chars {
        return (s, false);
    }
    match s.char_indices().nth(max_chars) {
        None => (s, false),
        Some((idx, _)) => {
            let mut owned = s;
            owned.truncate(idx);
            (owned, true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_utf8_no_truncate_if_fits() {
        let (s, t) = truncate_utf8("ola mundo", 100);
        assert_eq!(s, "ola mundo");
        assert!(!t);
    }

    #[test]
    fn truncate_utf8_truncates_large_ascii() {
        let entrada: String = "a".repeat(200);
        let (s, t) = truncate_utf8(&entrada, 50);
        assert_eq!(s.chars().count(), 50);
        assert!(t);
    }

    #[test]
    fn truncate_utf8_preserves_accented_graphemes() {
        // 10 codepoints: "á" (1 char) * 10
        let entrada: String = "á".repeat(30);
        let (s, t) = truncate_utf8(&entrada, 10);
        assert_eq!(s.chars().count(), 10);
        // Each 'á' is 2 UTF-8 bytes → 10 chars = 20 bytes
        assert_eq!(s.len(), 20);
        assert!(t);
        // Does not split mid-byte
        assert!(s.chars().all(|c| c == 'á'));
    }

    #[test]
    fn truncate_utf8_emojis_does_not_break() {
        let entrada = "🚀🔒🛡🔑✨🎉💎⚡🌟🔥🎨";
        let (s, t) = truncate_utf8(entrada, 5);
        assert_eq!(s.chars().count(), 5);
        assert!(t);
    }

    #[test]
    fn truncate_utf8_zero_returns_empty() {
        let (s, t) = truncate_utf8("abc", 0);
        assert_eq!(s, "");
        assert!(t);
    }

    #[test]
    fn take_utf8_capped_reuses_valid_utf8_without_truncate() {
        let bytes = b"hello agent".to_vec();
        let (s, t) = take_utf8_capped(bytes, 100);
        assert_eq!(s, "hello agent");
        assert!(!t);
    }

    #[test]
    fn take_utf8_capped_truncates_and_replaces_invalid() {
        let mut bytes = b"abcdef".to_vec();
        let (s, t) = take_utf8_capped(bytes.clone(), 3);
        assert_eq!(s, "abc");
        assert!(t);
        // Invalid UTF-8: 0xFF is replaced lossily, still returns owned String.
        bytes = vec![0xFF, b'a', b'b'];
        let (s, t) = take_utf8_capped(bytes, 10);
        assert!(!s.is_empty());
        assert!(!t);
        assert!(s.contains('a'));
    }
}
