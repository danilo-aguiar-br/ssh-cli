// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Explicit retry policy for ssh-cli (Rules Rust — retry / backoff).
//!
//! # Workload and policy scope
//!
//! | Concern | Decision |
//! |---------|----------|
//! | Product identity | One-shot agent-first SSH CLI (no long-lived client pool) |
//! | Who retries | **The agent** re-invokes the process; the binary does **not** |
//! | | auto-retry `exec` / `scp` / `sudo-exec` / `su-exec` (side effects) |
//! | Classification | [`crate::errors::SshCliError::is_retryable`] + JSON envelope fields |
//! | Dependency class | SSH TCP + russh only — no HTTP/gRPC product client |
//! | In-process loops | Opt-in only via [`RetryConfig::enabled`] (default **false**) |
//!
//! # Why in-process product retry is off by default
//!
//! Remote shell commands and file transfers are **not** assumed idempotent.
//! Blind retry would violate least privilege and the one-shot contract
//! (rules: desativar retry em operações não marcadas como idempotentes).
//! Agents use exit `74` + `retryable: true` and apply this policy externally.
//!
//! # Agent contract (documented SLA)
//!
//! - Retry **at most** [`crate::constants::AGENT_RETRY_MAX_RETRIES`] times after
//!   the first failure on transient network/SSH IO
//!   (exit [`crate::errors::exit_codes::EX_IOERR`]).
//! - Never blind-retry exits `64`, `65`, `66`, `77`, `1` (remote command),
//!   signals (`130`/`143`), or pipe (`141`).
//! - Sleep with full-jitter exponential backoff ([`backoff_full_jitter`]).
//! - Kill switch for embedding tools: `RetryConfig { enabled: false, .. }` or
//!   `max_retries: 0`.
//!
//! # Delay formula
//!
//! ```text
//! cap(n)  = min(base_ms * 2^min(n, 16), max_delay_ms)
//! delay   = uniform(0..=cap)   // full jitter
//! ```
//!
//! Monotonic clock for entropy mix: [`std::time::Instant`] (not `SystemTime`).
//! Async waiters must use `tokio::time::sleep`, never `std::thread::sleep`.
//!
//! # Out of scope (identity N/A)
//!
//! HTTP `Retry-After`, circuit breaker, retry budget token-bucket, hedged
//! requests, gRPC, OAuth refresh, outbox/saga, Idempotency-Key headers.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use crate::constants::{
    AGENT_RETRY_BASE_MS, AGENT_RETRY_MAX_DELAY_MS, AGENT_RETRY_MAX_RETRIES, HARD_RETRY_MAX_DELAY_MS,
    HARD_RETRY_MAX_RETRIES,
};
use crate::errors::exit_codes;

/// Named retry policy (one dependency class: SSH connect / agent re-invoke).
///
/// Clone is cheap (`Copy`). Default has [`Self::enabled`] = **false** so product
/// paths never retry as a side effect (opt-in / least privilege).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryConfig {
    /// Retries after the first attempt (`0` = single try).
    pub max_retries: u32,
    /// Exponential base delay in milliseconds.
    pub base_ms: u64,
    /// Maximum single sleep in milliseconds.
    pub max_delay_ms: u64,
    /// Kill switch: when false, never retries.
    pub enabled: bool,
}

impl RetryConfig {
    /// Agent-facing defaults (enabled) for **process re-invocation** only.
    ///
    /// Justification: matches `docs/AGENTS.md` — at most two retries on exit 74
    /// with backoff; not used inside product `exec` paths.
    #[must_use]
    pub const fn agent_default() -> Self {
        Self {
            max_retries: AGENT_RETRY_MAX_RETRIES,
            base_ms: AGENT_RETRY_BASE_MS,
            max_delay_ms: AGENT_RETRY_MAX_DELAY_MS,
            enabled: true,
        }
    }

    /// Disabled policy (product default / incident kill switch).
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            max_retries: 0,
            base_ms: AGENT_RETRY_BASE_MS,
            max_delay_ms: AGENT_RETRY_MAX_DELAY_MS,
            enabled: false,
        }
    }

    /// Clamp fields to hard caps (prevents accidental retry storms).
    #[must_use]
    pub fn clamped(self) -> Self {
        let base_ms = if self.base_ms == 0 {
            AGENT_RETRY_BASE_MS
        } else {
            self.base_ms
        };
        let max_delay_ms = self
            .max_delay_ms
            .min(HARD_RETRY_MAX_DELAY_MS)
            .max(base_ms);
        let max_retries = self.max_retries.min(HARD_RETRY_MAX_RETRIES);
        let enabled = self.enabled && max_retries > 0;
        Self {
            max_retries,
            base_ms,
            max_delay_ms,
            enabled,
        }
    }

    /// Total attempts including the first try.
    #[must_use]
    pub fn max_attempts(self) -> u32 {
        let p = self.clamped();
        if !p.enabled {
            1
        } else {
            p.max_retries.saturating_add(1)
        }
    }

    /// Whether another attempt is allowed after `attempt` completed tries (1-based).
    #[must_use]
    pub fn may_retry(self, attempt: u32) -> bool {
        let p = self.clamped();
        p.enabled && attempt < p.max_attempts()
    }

    /// Full-jitter delay for this attempt number (1-based completed count).
    #[must_use]
    pub fn delay_for_attempt(self, attempt: u32) -> Duration {
        let p = self.clamped();
        backoff_full_jitter(p.base_ms, attempt, p.max_delay_ms)
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        // Least privilege: product code must opt in explicitly.
        Self::disabled()
    }
}

/// Full jitter: `uniform(0..=min(base*2^attempt, max_delay))`.
///
/// Entropy from monotonic [`Instant`] + thread id + stack marker (no `rand` dep).
/// Not cryptographic; enough to desynchronize multi-agent thundering herds.
#[must_use]
pub fn backoff_full_jitter(base_ms: u64, attempt: u32, max_delay_ms: u64) -> Duration {
    let base = base_ms.max(1);
    let max_delay = max_delay_ms.max(base);
    let exp = base.saturating_mul(1u64 << attempt.min(16));
    let cap = exp.min(max_delay);
    let pick = mix_u64(attempt) % (cap.saturating_add(1));
    Duration::from_millis(pick)
}

/// Sysexits mapping: which process exits agents may re-invoke with backoff.
///
/// Only [`exit_codes::EX_IOERR`] (74) is network/SSH-IO retryable. Auth (`77`),
/// usage (`64`), data (`65`), no-input (`66`), remote command (`1`), signals and
/// pipe are permanent for the same argv.
#[must_use]
pub fn exit_code_is_retryable(code: i32) -> bool {
    code == exit_codes::EX_IOERR
}

/// Prefer operator-hinted wait when present; otherwise full-jitter formula.
///
/// SSH product has no HTTP `Retry-After`; `hint` lets embedding tools pass a
/// cool-down without forking the formula.
#[must_use]
pub fn wait_for_retry(policy: RetryConfig, attempt: u32, hint: Option<Duration>) -> Duration {
    let p = policy.clamped();
    if let Some(d) = hint {
        return d.min(Duration::from_millis(p.max_delay_ms));
    }
    p.delay_for_attempt(attempt)
}

fn mix_u64(attempt: u32) -> u64 {
    let mut h = DefaultHasher::new();
    Instant::now().hash(&mut h);
    attempt.hash(&mut h);
    std::thread::current().id().hash(&mut h);
    let marker = &h as *const DefaultHasher as usize;
    marker.hash(&mut h);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_is_disabled() {
        let p = RetryConfig::default();
        assert!(!p.enabled);
        assert_eq!(p.max_attempts(), 1);
        assert!(!p.may_retry(1));
    }

    #[test]
    fn agent_default_allows_two_retries() {
        let p = RetryConfig::agent_default().clamped();
        assert!(p.enabled);
        assert_eq!(p.max_retries, 2);
        assert_eq!(p.max_attempts(), 3);
        assert!(p.may_retry(1));
        assert!(p.may_retry(2));
        assert!(!p.may_retry(3));
    }

    #[test]
    fn kill_switch_zero_retries() {
        let p = RetryConfig {
            max_retries: 0,
            base_ms: 200,
            max_delay_ms: 5_000,
            enabled: true,
        }
        .clamped();
        assert!(!p.enabled);
        assert_eq!(p.max_attempts(), 1);
    }

    #[test]
    fn backoff_respects_cap() {
        let d = backoff_full_jitter(200, 20, 1_000);
        assert!(d.as_millis() <= 1_000);
    }

    #[test]
    fn backoff_never_exceeds_max() {
        for attempt in 0..20 {
            let d = backoff_full_jitter(100, attempt, 500);
            assert!(d.as_millis() <= 500, "attempt {attempt}: {d:?}");
        }
    }

    #[test]
    fn only_ioerr_exit_is_retryable() {
        assert!(exit_code_is_retryable(exit_codes::EX_IOERR));
        assert!(!exit_code_is_retryable(exit_codes::EX_OK));
        assert!(!exit_code_is_retryable(exit_codes::EX_USAGE));
        assert!(!exit_code_is_retryable(exit_codes::EX_DATAERR));
        assert!(!exit_code_is_retryable(exit_codes::EX_NOINPUT));
        assert!(!exit_code_is_retryable(exit_codes::EX_NOPERM));
        assert!(!exit_code_is_retryable(exit_codes::EX_GENERAL));
        assert!(!exit_code_is_retryable(exit_codes::EX_PIPE));
        assert!(!exit_code_is_retryable(exit_codes::EX_SIGINT));
    }

    #[test]
    fn wait_hint_caps_to_max_delay() {
        let p = RetryConfig::agent_default();
        let d = wait_for_retry(p, 1, Some(Duration::from_secs(3600)));
        assert_eq!(d, Duration::from_millis(AGENT_RETRY_MAX_DELAY_MS));
    }

    #[test]
    fn hard_caps_clamp_pathological_config() {
        let p = RetryConfig {
            max_retries: 10_000,
            base_ms: 1,
            max_delay_ms: u64::MAX,
            enabled: true,
        }
        .clamped();
        assert!(p.max_retries <= HARD_RETRY_MAX_RETRIES);
        assert!(p.max_delay_ms <= HARD_RETRY_MAX_DELAY_MS);
    }
}
