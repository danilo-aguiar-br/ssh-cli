// SPDX-License-Identifier: MIT OR Apache-2.0
//! B2 gate: every `Message` variant must be reachable from production code.
//!
//! # Why this suite exists
//!
//! Three separate rounds of this project shipped the same defect class: an
//! `i18n::Message` variant is written, translated into both locales, and then
//! bypassed at the real call site by an English literal. The recorded incident
//! is `i18n-variante-contornada-por-literal`; A4 was the second occurrence and
//! B2 the third, where the measurement came out at 17 dead variants out of 44.
//!
//! Nothing in the compiler catches it:
//!
//! - `Message` is `pub` in a `pub mod`, so `dead_code` never fires on a variant
//!   that no one constructs.
//! - `src/i18n_tests.rs` names the variants, which would mask the lint anyway.
//! - The exhaustive `match` in `en()` / `pt()` forces a *translation* to exist,
//!   never a *caller*.
//!
//! So the gate has to be a source-level assertion. It reads the declared variant
//! set out of `src/i18n.rs` and requires each one to appear as `Message::<Name>`
//! somewhere in production code. Adding a variant without using it turns this
//! suite red — which is exactly what did not happen the first three times.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Locale tables. Naming a variant here is a *translation*, never a caller.
///
/// C3 moved `en()` and `pt()` out of `src/i18n.rs` into these files. That move
/// would have silently gutted this suite: the scan below walks all of `src/`, so
/// the tables would have counted as production and every variant would have
/// looked reachable — a green gate measuring nothing. `translation_tables_are_
/// excluded_from_the_production_scan` pins the exclusion so the next relocation
/// fails loudly instead.
const TRANSLATION_TABLES: &[&str] = &["i18n/en.rs", "i18n/pt.rs"];

/// Files that only exercise the enum rather than consume it.
///
/// A variant referenced only from a test is still dead in the product: that is
/// the precise blind spot that let `dead_code` stay quiet for three rounds.
/// `i18n.rs` is handled separately by [`i18n_production_half`] — it is both the
/// declaration site and the home of a real caller.
fn is_excluded(rel: &str) -> bool {
    rel == "i18n.rs"
        || rel == "i18n_tests.rs"
        || TRANSLATION_TABLES.contains(&rel)
        || rel.ends_with("_tests.rs")
        || rel.ends_with("/tests.rs")
        || rel == "tests.rs"
}

/// The locale tables must exist where this suite expects them.
///
/// Without this, renaming or re-merging them turns the exclusion into a no-op
/// and the reachability assertion passes vacuously.
#[test]
fn translation_tables_are_excluded_from_the_production_scan() {
    for rel in TRANSLATION_TABLES {
        let path = workspace_root().join("src").join(rel);
        assert!(
            path.is_file(),
            "{rel} is missing: the reachability scan excludes it by name, so a \
             move leaves the exclusion pointing at nothing and every variant \
             starts looking reachable. Update TRANSLATION_TABLES with the new \
             path."
        );
        assert!(is_excluded(rel), "{rel} must be excluded from the scan");
    }

    // The tables must not have drifted back into the declaration file either.
    let i18n = fs::read_to_string(workspace_root().join("src/i18n.rs")).expect("read src/i18n.rs");
    assert!(
        !i18n.contains("fn en(msg: &Message)") && !i18n.contains("fn pt(msg: &Message)"),
        "the locale tables are back inside src/i18n.rs; `i18n_production_half` \
         would then have to cut them out again or they count as call sites"
    );
}

/// The part of `src/i18n.rs` that *consumes* `Message` rather than declaring or
/// translating it.
///
/// `localized_error_text` lives in this file and is genuine production code, so
/// excluding the whole file would report every error variant as dead. Cutting
/// out the `enum` body leaves exactly the consuming half.
///
/// There is no second cut for the `en()` / `pt()` tables: C3 moved them into
/// `src/i18n/en.rs` and `src/i18n/pt.rs`, and `the_locale_tables_live_in_their_own_files`
/// above asserts they have not drifted back. The old code searched for
/// `fn en(msg: &Message)` in this file and fell back to `src.len()` when absent,
/// which — given that assertion — was unreachable by construction: the guard
/// `enum_end < translations` was always true, so the branch only ever worked by
/// accident of its own fallback. A dead cut in a gate is how a gate stops meaning
/// what its comment claims, so it is gone rather than explained.
fn i18n_production_half(src: &str) -> String {
    let enum_start = src
        .find("pub enum Message")
        .expect("enum Message must exist");
    let enum_end = src[enum_start..]
        .find("\n}\n")
        .map_or(src.len(), |off| enum_start + off + 3);

    let mut out = String::with_capacity(src.len());
    out.push_str(&src[..enum_start]);
    out.push_str(&src[enum_end..]);
    out
}

fn walk_rs(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            walk_rs(&p, root, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            let rel = p
                .strip_prefix(root)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            if let Ok(text) = fs::read_to_string(&p) {
                out.push((rel, text));
            }
        }
    }
}

/// Variant identifiers declared inside `pub enum Message { … }`.
fn declared_variants(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut inside = false;
    for line in src.lines() {
        if !inside {
            if line.starts_with("pub enum Message") {
                inside = true;
            }
            continue;
        }
        // The enum body is the only block at this indentation; a bare `}` in
        // column zero closes it.
        if line == "}" {
            break;
        }
        // Variants sit at exactly four spaces; doc comments and inner comments
        // start with `/` and are skipped.
        let Some(rest) = line.strip_prefix("    ") else {
            continue;
        };
        if rest.starts_with(' ') || rest.starts_with('/') {
            continue;
        }
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            out.insert(name);
        }
    }
    out
}

#[test]
fn every_message_variant_has_a_production_call_site() {
    let root = workspace_root().join("src");
    let i18n = fs::read_to_string(root.join("i18n.rs")).expect("read src/i18n.rs");
    let declared = declared_variants(&i18n);

    assert!(
        declared.len() > 20,
        "variant extraction broke: parsed only {} variants",
        declared.len()
    );

    let mut files = Vec::new();
    walk_rs(&root, &root, &mut files);
    let mut production: String = files
        .iter()
        .filter(|(rel, _)| !is_excluded(rel))
        .map(|(_, text)| text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    production.push('\n');
    production.push_str(&i18n_production_half(&i18n));

    let dead: Vec<&String> = declared
        .iter()
        .filter(|name| !production.contains(&format!("Message::{name}")))
        .collect();

    assert!(
        dead.is_empty(),
        "these `Message` variants are fully translated but never constructed in \
         production code, so `--lang pt-BR` can never reach them:\n  {}\n\
         Either wire them into the real call site or delete them together with \
         their EN and pt-BR arms.",
        dead.iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn every_variant_has_both_language_arms() {
    let root = workspace_root().join("src");
    let i18n = fs::read_to_string(root.join("i18n.rs")).expect("read src/i18n.rs");
    let declared = declared_variants(&i18n);

    // `en()` and `pt()` are exhaustive matches, so a missing arm is a compile
    // error already. What is *not* enforced by the compiler is that both
    // functions were updated in the same edit — a variant can be folded into a
    // catch-all in one locale and spelled out in the other.
    //
    // C3: the tables now live in their own files. Reading them from disk keeps
    // this a source-level check; `translation_tables_are_excluded_from_the_
    // production_scan` guarantees the paths still resolve.
    let en_body = fs::read_to_string(root.join("i18n/en.rs")).expect("read src/i18n/en.rs");
    let pt_body = fs::read_to_string(root.join("i18n/pt.rs")).expect("read src/i18n/pt.rs");

    let mut missing = Vec::new();
    for name in &declared {
        let needle = format!("Message::{name}");
        if !en_body.contains(&needle) {
            missing.push(format!("{name} (EN)"));
        }
        if !pt_body.contains(&needle) {
            missing.push(format!("{name} (pt-BR)"));
        }
    }

    assert!(
        missing.is_empty(),
        "variants missing an explicit locale arm: {}",
        missing.join(", ")
    );
}

/// The human error branch must consult i18n; the JSON branch must not.
///
/// B2 root cause: `thiserror`'s `#[error("…")]` attribute renders at the call
/// site, so six translated `Error*` variants were shadowed by English literals
/// and `--lang pt-BR` produced byte-identical output. This pins the seam.
#[test]
fn error_emitter_localizes_text_but_not_the_json_envelope() {
    let lib = fs::read_to_string(workspace_root().join("src/lib.rs")).expect("read src/lib.rs");
    let start = lib
        .find("fn emit_resolved_ssh_error")
        .expect("emit_resolved_ssh_error must exist");
    // Bound the slice to the function's own closing brace rather than a fixed
    // 1600-byte window: a window silently slides off the target as the function
    // grows, and the assertions below would then pass on unrelated code.
    let rest = &lib[start..];
    let body_end = rest.find("\n}\n").map_or(rest.len(), |off| off + 2);
    let body = &rest[..body_end];
    assert!(
        body.contains("print_error_envelope"),
        "the emit_resolved_ssh_error slice is broken, not the product: \
         found {} bytes with no envelope call",
        body.len()
    );

    assert!(
        body.contains("i18n::localized_error_text"),
        "the human branch must render errors through i18n, otherwise --lang is a no-op"
    );

    // Bound the slice to the envelope call's own argument list, not a fixed
    // window: the human branch follows immediately and a loose window would
    // swallow it and defeat the assertion. `} else` is the branch seam, so it
    // survives reindentation of the call.
    let json_call = body
        .find("print_error_envelope")
        .expect("JSON branch must emit an envelope");
    let json_end = body[json_call..]
        .find("} else")
        .map_or(body.len(), |off| json_call + off);
    let json_args = &body[json_call..json_end];
    assert!(
        !json_args.contains("localized_error_text"),
        "the JSON envelope must keep the English Display: agents branch on \
         error_code and must never receive locale-dependent prose"
    );
}
