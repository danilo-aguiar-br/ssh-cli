// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Map `instant-acme` errors to permanent vs transient [`SshCliError`] (G-E2E-01).
//!
//! # Policy
//!
//! ACME **validation / client** problem types (RFC 8555) must **not** be
//! retried by agents (`retryable: false`, exit 64). Transport / rate-limit /
//! 5xx stay on [`SshCliError::Tls`] (exit 74, transient).
//!
//! Workload: pure string/URN classification (CPU sequential; no I/O).

use crate::errors::SshCliError;

/// ACME error type leaf names that are permanent client/validation failures.
///
/// Matched against the last segment of `urn:ietf:params:acme:error:<leaf>`
/// (case-insensitive). Sourced from RFC 8555 §6.7 + Let's Encrypt practice.
const PERMANENT_ACME_LEAVES: &[&str] = &[
    "invalidcontact",
    "unsupportedcontact",
    "malformed",
    "rejectedidentifier",
    "unauthorized",
    "accountdoesnotexist",
    "badcsr",
    "badpublickey",
    "badrevocationreason",
    "caa",
    "dns",
    "useractionrequired",
    "alreadyrevoked",
    "ordernotready",
    "compound",
    "unsupportedidentifier",
    "externalaccountrequired",
];

/// Transient ACME leaves (may retry with backoff).
const TRANSIENT_ACME_LEAVES: &[&str] = &[
    "ratelimited",
    "serverinternal",
    "badnonce", // fetch fresh nonce and retry
];

/// Maps an `instant-acme` error into product taxonomy.
///
/// * `Error::Api(Problem)` with permanent URN → [`SshCliError::InvalidArgument`]
/// * rate limit / 5xx / network / timeout → [`SshCliError::Tls`]
/// * Display fallback scans for permanent leaves (when variant is opaque)
#[must_use]
pub fn map_acme_error(context: &str, err: instant_acme::Error) -> SshCliError {
    match err {
        instant_acme::Error::Api(problem) => map_acme_problem(context, &problem),
        instant_acme::Error::Timeout(_) => {
            SshCliError::tls_msg(format!("{context}: timeout: {err}"))
        }
        other => classify_display(context, &other.to_string()),
    }
}

fn map_acme_problem(context: &str, problem: &instant_acme::Problem) -> SshCliError {
    let urn = problem.r#type.as_deref().unwrap_or("");
    let leaf = acme_type_leaf(urn);
    let detail = problem
        .detail
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    let status = problem.status.unwrap_or(0);

    if is_transient_leaf(leaf) {
        return SshCliError::tls_msg(format!("{context}: ACME {leaf}: {problem}"));
    }
    if is_permanent_leaf(leaf)
        || detail.contains("invalidcontact")
        || detail.contains("invalid contact")
    {
        return SshCliError::InvalidArgument(format!("{context}: ACME {leaf}: {problem}"));
    }
    // HTTP 4xx without known transient leaf → permanent client.
    if (400..500).contains(&status) {
        return SshCliError::InvalidArgument(format!("{context}: ACME HTTP {status}: {problem}"));
    }
    // 5xx or unknown → treat as transient transport/API.
    if status >= 500 {
        return SshCliError::tls_msg(format!("{context}: ACME HTTP {status}: {problem}"));
    }
    // Unknown type without status: scan display for permanent tokens.
    classify_display(context, &problem.to_string())
}

fn classify_display(context: &str, text: &str) -> SshCliError {
    let lower = text.to_ascii_lowercase();
    if lower.contains("ratelimited") || lower.contains("rate limited") {
        return SshCliError::tls_msg(format!("{context}: {text}"));
    }
    for leaf in PERMANENT_ACME_LEAVES {
        let n = leaf.to_ascii_lowercase();
        if lower.contains(&n) {
            return SshCliError::InvalidArgument(format!("{context}: {text}"));
        }
    }
    // Default API/network noise stays transient (historical Tls path).
    SshCliError::tls_msg(format!("{context}: {text}"))
}

/// Last segment of an ACME error URN (lowercased).
#[must_use]
pub fn acme_type_leaf(urn: &str) -> &str {
    urn.rsplit(':').next().unwrap_or(urn)
}

fn is_permanent_leaf(leaf: &str) -> bool {
    let n = leaf.to_ascii_lowercase();
    PERMANENT_ACME_LEAVES
        .iter()
        .any(|p| p.eq_ignore_ascii_case(&n) || *p == n.as_str())
}

fn is_transient_leaf(leaf: &str) -> bool {
    let n = leaf.to_ascii_lowercase();
    TRANSIENT_ACME_LEAVES
        .iter()
        .any(|p| p.eq_ignore_ascii_case(&n))
}

#[cfg(test)]
mod tests {
    use super::*;
    use instant_acme::Problem;

    fn problem(type_urn: &str, status: Option<u16>, detail: &str) -> Problem {
        Problem {
            r#type: Some(type_urn.to_owned()),
            detail: Some(detail.to_owned()),
            status,
            subproblems: Vec::new(),
        }
    }

    #[test]
    fn invalid_contact_is_permanent_invalid_argument() {
        let p = problem(
            "urn:ietf:params:acme:error:invalidContact",
            Some(400),
            "Error creating new account :: invalid contact",
        );
        let e = map_acme_problem("ACME create account", &p);
        assert!(matches!(e, SshCliError::InvalidArgument(_)));
        assert!(!e.is_retryable());
        assert_eq!(e.exit_code(), crate::errors::exit_codes::EX_USAGE);
    }

    #[test]
    fn rate_limited_is_transient_tls() {
        let p = problem(
            "urn:ietf:params:acme:error:rateLimited",
            Some(429),
            "too many requests",
        );
        let e = map_acme_problem("ACME create account", &p);
        assert!(matches!(e, SshCliError::Tls { .. }));
        assert!(e.is_retryable());
    }

    #[test]
    fn malformed_is_permanent() {
        let p = problem(
            "urn:ietf:params:acme:error:malformed",
            Some(400),
            "malformed request",
        );
        let e = map_acme_problem("ACME", &p);
        assert!(matches!(e, SshCliError::InvalidArgument(_)));
    }

    #[test]
    fn display_fallback_catches_invalid_contact() {
        let e = classify_display(
            "ACME create account",
            "API error: invalidContact: Error creating new account",
        );
        assert!(matches!(e, SshCliError::InvalidArgument(_)));
    }

    #[test]
    fn leaf_parser() {
        assert_eq!(
            acme_type_leaf("urn:ietf:params:acme:error:invalidContact"),
            "invalidContact"
        );
    }
}
