// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gate: the source tree is written in English.
//!
//! # What this asserts, and what it deliberately does not
//!
//! The rule for this repository is that code and comments are English, while the
//! Portuguese surface lives in `src/i18n/pt.rs` and in the `*.pt-BR.md` docs. The
//! rule held by convention alone, so it decayed the way conventions do: a lone
//! test function called `disable_sudo_global_bloqueia_sudo_exec` survived several
//! audits because nothing but a human reader could see it.
//!
//! The naive assertion — *no accented character anywhere* — is wrong here, and
//! measurably so. Four lines in this tree carry accents for a good reason:
//!
//! - `tests/docs_conformance.rs` explains that the section map pairs `Security`
//!   with `Segurança`, and it has to name the thing it pairs.
//! - `src/ssh/session_io.rs` counts the UTF-8 bytes of `'á'` in a comment beside
//!   the test that counts them in code.
//! - `tests/gaps_v062_i18n_reachability.rs` quotes the exact localized string that
//!   must never reach a JSON wire value.
//!
//! Each one *cites* Portuguese; none of them *is* Portuguese. So the invariant is
//! not "no accents", it is: an accented character must sit inside a citation. A
//! comment may quote localized data, and must not be written in another language.
//! That distinction is what makes the gate survivable — a rule that forbade the
//! four lines above would be silenced within a week, and a silenced gate is worse
//! than none.
//!
//! # How the citation test is made mechanical
//!
//! [`strip_quoted_spans`] blanks every span a Rust line uses to carry data or a
//! citation: string literals, char literals and backtick spans. Whatever survives
//! is bare code or bare prose, and a non-ASCII letter there is the defect.
//!
//! The first draft treated `"` as always escapable, and `src/paths.rs:227` broke
//! it within a minute: `r"\\?\"` is a raw string, where `\` is an ordinary
//! character, so reading the trailing `\"` as an escape swallowed the terminator
//! and desynchronised every following line of the file. The symptom was perfect
//! camouflage — three `assert!` lines full of legitimate Portuguese *test data*
//! were reported, while a genuinely bare `// é precomposed` eleven lines further
//! down was swallowed by the same desync and never reported at all. A scanner
//! that mis-parses does not merely miss; it accuses the innocent and shelters the
//! guilty, and both halves of that were visible in one run.
//!
//! So raw strings are parsed as raw strings, hashes and all, and both plain and
//! raw literals carry across lines — `src/vps/model.rs:551` opens a multi-line
//! `r#"` block that would otherwise reopen the same wound.
//!
//! Remaining edges, stated rather than hidden:
//!
//! - `'` opens a char literal only when it closes within eight characters, which
//!   is what keeps a lifetime (`'a`) from swallowing the rest of the line. Two
//!   lifetimes on one line can pair with each other; the span between them is
//!   type syntax, which cannot hold prose.
//! - A `"` inside a line comment opens a literal that the scanner believes. The
//!   cost is a false positive on the next line, never a miss.
//!
//! Both edges fail toward a false *positive*: the worst case is a maintainer being
//! told to put quotes around a citation.
//!
//! # What this supersedes, and what it does not
//!
//! `gaps_v062_i18n_reachability.rs` carried an earlier accent scan over the same
//! two directories. It is gone, and its work is here, because the two differed in
//! exactly one thing that matters: it exempted *files*, thirteen of them. Once
//! `tests/docs_conformance.rs` is on such a list, all twelve hundred of its lines
//! are exempt, so genuinely Portuguese prose written there next year passes. Each
//! entry was individually justified — the accents really were the payload — and the
//! blindness was a side effect nobody chose.
//!
//! Exempting citations instead keeps those same thirteen files honest line by line,
//! and the allowlist here is **empty**. That is a measurement, not a goal: every
//! accent in all thirteen, `src/i18n/pt.rs` included, already sits inside a string
//! literal, so the citation rule reads them as the data they are. Choosing the
//! right unit to exempt removed the need to exempt anything.
//!
//! Its ratchet on stale entries came along too, in
//! [`every_language_exemption_is_still_load_bearing`]; that half was right.
//!
//! # Why this does not replace `scripts/check_en_identifiers.sh`
//!
//! That gate runs five checks built from denylists of Portuguese words and from
//! connective grammar (`_nao_`, `_sem_`, `_contem_`). It reads ASCII, because the
//! words it hunts are spelled in ASCII. This gate reads the opposite half: any
//! non-ASCII letter that is not inside a citation, wherever it appears.
//!
//! The two halves are disjoint, and the gap between them is measured, not assumed.
//! `disable_sudo_global_bloqueia_sudo_exec` lived in `tests/gaps_v035_integration.rs`
//! through several audits: `bloqueia` is not in any denylist and carries no
//! connective, so the shell gate could not see it — and it has no accent, so this
//! gate cannot see it either. It was found by reading. Neither file is redundant,
//! and deleting either one reopens a half of the surface.

#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Roots scanned for Rust sources.
const SCANNED_DIRS: &[&str] = &["src", "tests"];

/// Files allowed to be written in another language, each with its reason.
///
/// It is empty, and that is the measured result rather than an aspiration. The
/// superseded allowlist carried thirteen entries; `src/i18n/pt.rs` was drafted as
/// the fourteenth here and then deleted, because
/// [`every_language_exemption_is_still_load_bearing`] refused it: every accent in
/// the pt-BR message table sits inside a string literal, so the citation rule
/// already treats it as the data it is. The same turned out to hold for the other
/// twelve. Exempting the right *unit* removed the need to exempt anything.
///
/// The machinery stays for the next author who believes they need an entry. It
/// costs a reason column and an expiry check, which is the price of the claim.
const LANGUAGE_EXEMPT: &[(&str, &str)] = &[];

/// Non-ASCII characters that are English technical notation, not another language.
///
/// The SI micro prefix, in both codepoints that render identically: U+00B5 MICRO
/// SIGN and U+03BC GREEK SMALL LETTER MU. Demanding `us` instead would trade a
/// precise symbol for an ambiguous one, and the next author would type `µ` again.
/// An exemption nobody obeys is a gate nobody runs.
const TECHNICAL_SYMBOLS: &[char] = &['\u{00B5}', '\u{03BC}'];

/// Literal the scanner is inside when a line begins.
///
/// `Plain` is an ordinary `"…"`, where `\` escapes the next character. `Raw` is
/// `r#*"…"#*` with the recorded hash count, where `\` escapes nothing. Keeping
/// them apart is the entire reason the second draft of this scanner works.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanState {
    Code,
    Plain,
    Raw(usize),
}

/// Hash count if a raw-string literal opens at `i`, else `None`.
fn raw_opener(chars: &[char], i: usize) -> Option<usize> {
    if chars[i] != 'r' {
        return None;
    }
    let mut j = i + 1;
    while chars.get(j) == Some(&'#') {
        j += 1;
    }
    (chars.get(j) == Some(&'"')).then_some(j - i - 1)
}

/// Index just past the closing `"` of a plain literal that began before `from`.
fn close_plain(chars: &[char], from: usize) -> Option<usize> {
    let mut j = from;
    while j < chars.len() {
        match chars[j] {
            '\\' => j += 2,
            '"' => return Some(j + 1),
            _ => j += 1,
        }
    }
    None
}

/// Index just past the `"` plus `hashes` terminator of a raw literal.
fn close_raw(chars: &[char], from: usize, hashes: usize) -> Option<usize> {
    let mut j = from;
    while j < chars.len() {
        if chars[j] == '"' && (1..=hashes).all(|k| chars.get(j + k) == Some(&'#')) {
            return Some(j + hashes + 1);
        }
        j += 1;
    }
    None
}

/// Blanks string literals, char literals and backtick spans on one line.
///
/// `state` carries an unterminated literal, plain or raw, across lines.
fn strip_quoted_spans(line: &str, state: &mut ScanState) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(chars.len());
    let mut i = 0;

    // Finish whatever a previous line left open before reading any code.
    match *state {
        ScanState::Plain => match close_plain(&chars, 0) {
            Some(next) => {
                i = next;
                *state = ScanState::Code;
            }
            None => return out,
        },
        ScanState::Raw(hashes) => match close_raw(&chars, 0, hashes) {
            Some(next) => {
                i = next;
                *state = ScanState::Code;
            }
            None => return out,
        },
        ScanState::Code => {}
    }

    while i < chars.len() {
        if let Some(hashes) = raw_opener(&chars, i) {
            match close_raw(&chars, i + hashes + 2, hashes) {
                Some(next) => i = next,
                None => {
                    *state = ScanState::Raw(hashes);
                    return out;
                }
            }
            continue;
        }
        match chars[i] {
            '"' => match close_plain(&chars, i + 1) {
                Some(next) => i = next,
                None => {
                    *state = ScanState::Plain;
                    return out;
                }
            },
            '`' => {
                i += 1;
                while i < chars.len() && chars[i] != '`' {
                    i += 1;
                }
                i = (i + 1).min(chars.len());
            }
            '\'' => {
                // A lifetime never closes; a char literal closes within eight.
                let close = chars[i + 1..]
                    .iter()
                    .take(8)
                    .position(|c| *c == '\'')
                    .map(|p| i + 1 + p);
                match close {
                    Some(end) => i = end + 1,
                    None => {
                        out.push('\'');
                        i += 1;
                    }
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }

    out
}

fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in SCANNED_DIRS {
        collect(&root.join(dir), &mut out);
    }
    out.sort();
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn relative(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

/// A non-ASCII letter outside a citation is prose in another language.
#[test]
fn source_prose_and_identifiers_are_english() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for file in rust_sources(&root) {
        let rel = relative(&root, &file);
        if LANGUAGE_EXEMPT.iter().any(|(p, _)| rel == *p) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let mut state = ScanState::Code;
        for (idx, line) in text.lines().enumerate() {
            let bare = strip_quoted_spans(line, &mut state);
            if let Some(c) = bare
                .chars()
                .find(|c| c.is_alphabetic() && !c.is_ascii() && !TECHNICAL_SYMBOLS.contains(c))
            {
                offenders.push(format!(
                    "{rel}:{} carries `{c}` outside any citation",
                    idx + 1
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "these lines are written in a language other than English:\n  {}\n\
         Translate the comment or the identifier. If the line is *citing* localized \
         data, put the citation in backticks or quotes — that is the form the rest \
         of the tree already uses, and it is what makes the citation legible as one.",
        offenders.join("\n  ")
    );
}

/// A stale exemption is a claim nobody is checking any more.
///
/// Two ways to go stale, and the ratchet has to catch both: the path can vanish,
/// and the file can stop needing the exemption. The second is the quiet one — an
/// entry that no longer earns its place still reads like coverage while silencing
/// a file nobody is checking. This is the property the superseded allowlist in
/// `gaps_v062_i18n_reachability.rs` got right, kept here on purpose.
#[test]
fn every_language_exemption_is_still_load_bearing() {
    let root = workspace_root();
    let mut stale = Vec::new();

    for (rel, _) in LANGUAGE_EXEMPT {
        let path = root.join(rel);
        let Ok(text) = fs::read_to_string(&path) else {
            stale.push(format!("{rel}: the path no longer exists"));
            continue;
        };
        let mut state = ScanState::Code;
        let needed = text.lines().any(|line| {
            strip_quoted_spans(line, &mut state)
                .chars()
                .any(|c| c.is_alphabetic() && !c.is_ascii() && !TECHNICAL_SYMBOLS.contains(&c))
        });
        if !needed {
            stale.push(format!(
                "{rel}: the file is already English outside citations"
            ));
        }
    }

    assert!(
        stale.is_empty(),
        "these entries in LANGUAGE_EXEMPT no longer earn their place:\n  {}\n\
         Delete the entry. An exemption that is not load-bearing reads like coverage \
         and provides none — it silences a file for a reason that stopped being true.",
        stale.join("\n  ")
    );
}

/// The scanner is the whole gate, so it is tested like product code.
mod scanner {
    use super::{strip_quoted_spans, ScanState};

    fn strip(line: &str) -> String {
        strip_quoted_spans(line, &mut ScanState::Code)
    }

    #[test]
    fn a_citation_in_backticks_is_data() {
        assert_eq!(strip("/// maps to `Segurança`"), "/// maps to ");
    }

    #[test]
    fn bare_prose_survives_stripping() {
        let bare = strip("// isto está em português");
        assert!(
            bare.contains('á'),
            "bare prose must reach the assertion: {bare}"
        );
    }

    #[test]
    fn a_lifetime_does_not_swallow_the_rest_of_the_line() {
        let bare = strip("fn f<'a>(x: &'a str) {} // não");
        assert!(
            bare.contains('ã'),
            "a lifetime is not a char literal: {bare}"
        );
    }

    #[test]
    fn a_char_literal_is_data() {
        assert!(!strip("// bytes of 'á' here").contains('á'));
    }

    /// The regression that killed the first draft: `\` escapes nothing in a raw
    /// string, so reading `\"` as an escape ate the terminator and desynchronised
    /// every line below it. This is `src/paths.rs:227`, verbatim.
    #[test]
    fn a_backslash_does_not_escape_the_end_of_a_raw_string() {
        let mut state = ScanState::Code;
        strip_quoted_spans(
            r#"    s.starts_with(r"\\?\") || s.starts_with("//?/")"#,
            &mut state,
        );
        assert_eq!(
            state,
            ScanState::Code,
            "both literals close on this line; anything else desynchronises the file"
        );
    }

    #[test]
    fn a_multi_line_raw_string_carries_its_hash_count() {
        let mut state = ScanState::Code;
        strip_quoted_spans(r##"        let legacy = r#"{"##, &mut state);
        assert_eq!(state, ScanState::Raw(1), "the block is still open");
        let bare = strip_quoted_spans(r#"  "nome" = "produção""#, &mut state);
        assert!(!bare.contains('ç'), "raw body is data: {bare}");
        assert_eq!(state, ScanState::Raw(1), "no bare quote may close it");
        strip_quoted_spans(r##"        "#;"##, &mut state);
        assert_eq!(state, ScanState::Code, "quote plus one hash closes it");
    }

    #[test]
    fn an_unterminated_plain_string_carries_to_the_next_line() {
        let mut state = ScanState::Code;
        strip_quoted_spans("let x = \"opens here", &mut state);
        assert_eq!(state, ScanState::Plain, "the literal is still open");
        assert!(!strip_quoted_spans("ação closes\";", &mut state).contains('ç'));
        assert_eq!(state, ScanState::Code, "the literal closed");
    }
}
