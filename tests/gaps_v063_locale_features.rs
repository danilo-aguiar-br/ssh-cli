// SPDX-License-Identifier: MIT OR Apache-2.0
//! D12 gate: locale Cargo features stay tied to locales that actually exist.
//!
//! # Why this suite exists
//!
//! `Cargo.toml` advertises four locale features — `i18n-full`, `i18n-cjk`,
//! `i18n-rtl`, `i18n-europe`. All four are inert: their feature arrays are
//! empty, `cargo check --features i18n-full` changes nothing, and the default
//! binary embeds `en` + `pt-BR` either way.
//!
//! That inertness is deliberate and documented — `src/lib.rs` marks every one of
//! them "Reserved". The gap is not the current state; it is that nothing holds
//! the two halves together. The day someone adds `zh-Hans` strings behind
//! `i18n-cjk` and forgets to extend [`Language::AVAILABLE`], the feature stays
//! inert in silence: it compiles, it ships, and the locale it promises is simply
//! not there. A reserved feature and a broken feature are indistinguishable from
//! outside, which is exactly what makes the failure quiet.
//!
//! So this suite asserts the invariant rather than the state: **a locale feature
//! is either reserved (empty) or it delivers a locale.** While all four are
//! reserved, it pins them as reserved and pins `AVAILABLE` to the MVP pair. The
//! first feature to stop being empty flips the assertion into a demand, and
//! [`REGISTERED_LOCALE_FEATURES`] below records what that demand is.

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

use ssh_cli::i18n::Language;

/// Locales the default binary is contracted to embed, with no feature enabled.
///
/// `src/lib.rs`: "Default binary always embeds **en** + **pt-BR** only".
const MVP_LOCALES: &[&str] = &["en", "pt-BR"];

/// Every locale feature, and the locales it must deliver once it stops being an
/// empty array.
///
/// # How to change this table
///
/// When a feature gains real content, replace its empty slice with the BCP47
/// tags it is expected to add. [`locale_features_deliver_what_they_promise`]
/// then requires each of those tags to appear in [`Language::AVAILABLE`], which
/// is the coupling that does not exist today.
///
/// The expected tags come from the reservation text in `src/lib.rs`.
const REGISTERED_LOCALE_FEATURES: &[(&str, &[&str])] = &[
    // "Reserved: top-20 economic locales" — an aggregate over the three below,
    // so it carries no locales of its own.
    ("i18n-full", &[]),
    // "Reserved: zh-Hans / zh-Hant / ja / ko"
    ("i18n-cjk", &[]),
    // "Reserved: ar / he (RTL isolation)"
    ("i18n-rtl", &[]),
    // "Reserved: additional European locales"
    ("i18n-europe", &[]),
];

/// The one feature that is an aggregate rather than a locale carrier, and the
/// leaves it must fan out to.
const AGGREGATE_FEATURE: (&str, &[&str]) = ("i18n-full", &["i18n-cjk", "i18n-rtl", "i18n-europe"]);

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

/// Feature name -> the entries of its array, as written in `[features]`.
///
/// Hand-parsed because the manifest is not a build dependency of the test
/// suite. `[features]` here is one `name = [...]` per line, which is all this
/// needs to read.
fn declared_features() -> Vec<(String, Vec<String>)> {
    let text = fs::read_to_string(manifest_path()).expect("read Cargo.toml");
    let mut out = Vec::new();
    let mut in_features = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_features = trimmed == "[features]";
            continue;
        }
        if !in_features || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((name, rhs)) = trimmed.split_once('=') else {
            continue;
        };
        let rhs = rhs.trim();
        let Some(inner) = rhs.strip_prefix('[').and_then(|r| r.strip_suffix(']')) else {
            continue;
        };
        let entries = inner
            .split(',')
            .map(|e| e.trim().trim_matches('"').to_string())
            .filter(|e| !e.is_empty())
            .collect();
        out.push((name.trim().to_string(), entries));
    }

    assert!(
        !out.is_empty(),
        "parsed no entries from [features] in {} — the parser and the manifest \
         layout have diverged, and every assertion below would pass vacuously",
        manifest_path().display()
    );
    out
}

fn feature_entries(name: &str) -> Vec<String> {
    declared_features()
        .into_iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("Cargo.toml declares no feature `{name}`"))
        .1
}

fn available_tags() -> Vec<&'static str> {
    Language::AVAILABLE.iter().map(|l| l.bcp47()).collect()
}

/// Every feature named in the table must exist in the manifest, and vice versa.
///
/// Without this, deleting a feature from `Cargo.toml` — or adding a fifth one —
/// leaves the table describing a world that no longer matches, and the gate goes
/// on measuring nothing.
#[test]
fn the_locale_feature_table_matches_the_manifest() {
    let manifest: Vec<String> = declared_features()
        .into_iter()
        .map(|(n, _)| n)
        .filter(|n| n.starts_with("i18n-"))
        .collect();

    let registered: Vec<&str> = REGISTERED_LOCALE_FEATURES.iter().map(|(n, _)| *n).collect();

    for name in &registered {
        assert!(
            manifest.iter().any(|m| m == name),
            "REGISTERED_LOCALE_FEATURES names `{name}`, which Cargo.toml does \
             not declare"
        );
    }
    for name in &manifest {
        assert!(
            registered.contains(&name.as_str()),
            "Cargo.toml declares locale feature `{name}`, which is absent from \
             REGISTERED_LOCALE_FEATURES — add it with the locales it delivers"
        );
    }
}

/// The aggregate fans out to its leaves and adds nothing else.
#[test]
fn the_aggregate_locale_feature_only_aggregates() {
    let (name, expected_leaves) = AGGREGATE_FEATURE;
    let mut entries = feature_entries(name);
    entries.sort();
    let mut expected: Vec<String> = expected_leaves.iter().map(|s| (*s).to_string()).collect();
    expected.sort();

    assert_eq!(
        entries, expected,
        "`{name}` must enable exactly its locale leaves. Anything else it \
         switches on is a non-locale effect hiding behind a locale name."
    );
}

/// The invariant: a locale feature is reserved, or it delivers its locales.
///
/// While a feature's array is empty it is reserved, and the table must say it
/// carries no locales. The moment it stops being empty, every tag the table
/// records for it has to be present in [`Language::AVAILABLE`] — which is the
/// check whose absence D12 flagged.
#[test]
fn locale_features_deliver_what_they_promise() {
    let available = available_tags();
    let (aggregate, _) = AGGREGATE_FEATURE;

    for (name, promised) in REGISTERED_LOCALE_FEATURES {
        let entries = feature_entries(name);
        // The aggregate is non-empty by construction; it carries no locales of
        // its own, and its leaves are checked on their own rows.
        let is_reserved = entries.is_empty() || *name == aggregate;

        if is_reserved {
            assert!(
                promised.is_empty(),
                "`{name}` is still an empty (reserved) feature, but the table \
                 promises the locales {promised:?}. Either the feature gained \
                 content and this row is stale, or the row was filled in early."
            );
            continue;
        }

        for tag in *promised {
            assert!(
                available.contains(tag),
                "feature `{name}` is no longer empty but `Language::AVAILABLE` \
                 does not contain `{tag}` (it has {available:?}).\n\
                 Enabling the feature therefore changes nothing at runtime — \
                 the exact silent inertness this gate exists to catch. Add the \
                 variant in `src/i18n.rs` and list it in `AVAILABLE`."
            );
        }
    }
}

/// The default binary embeds the MVP pair and nothing more.
///
/// This is the other half of the invariant: a locale must not appear in
/// `AVAILABLE` without a feature that accounts for it. While all four features
/// are reserved, the only accountable set is the MVP pair.
#[test]
fn available_locales_are_accounted_for_by_a_feature() {
    let available = available_tags();

    let unreserved: Vec<&str> = REGISTERED_LOCALE_FEATURES
        .iter()
        .filter(|(name, _)| *name != AGGREGATE_FEATURE.0 && !feature_entries(name).is_empty())
        .map(|(name, _)| *name)
        .collect();

    if unreserved.is_empty() {
        assert_eq!(
            available, MVP_LOCALES,
            "every locale feature is still reserved (empty), so the binary must \
             embed exactly the MVP pair. An extra locale here arrived without a \
             feature to gate it."
        );
        return;
    }

    let mut accounted: Vec<&str> = MVP_LOCALES.to_vec();
    for (name, promised) in REGISTERED_LOCALE_FEATURES {
        if unreserved.contains(name) {
            accounted.extend(promised.iter().copied());
        }
    }

    for tag in &available {
        assert!(
            accounted.contains(tag),
            "`Language::AVAILABLE` embeds `{tag}`, which no locale feature \
             accounts for. Record it against the feature that delivers it in \
             REGISTERED_LOCALE_FEATURES."
        );
    }
}
