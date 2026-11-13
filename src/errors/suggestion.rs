// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted.
#![forbid(unsafe_code)]
//! Agent-facing remediation text for the JSON error envelope.

use super::{RetryKind, SshCliError, UnavailableService};
use crate::constants::{
    SUGGESTION_UNAVAILABLE_CONFIG_LOCK, SUGGESTION_UNAVAILABLE_KEYRING, SUGGESTION_VPS_NOT_FOUND,
};

impl SshCliError {
    /// Short agent-facing suggestion for the JSON envelope.
    #[must_use]
    pub fn suggestion(&self) -> Option<&'static str> {
        // G-ERR-R01: these variants share a `RetryKind` with unrelated failures, and the
        // generic hint derived from it would be actively misleading — telling an
        // operator to raise `--timeout` when the OS keyring is locked sends them to
        // the wrong knob entirely. Matching the variant first keeps the advice true.
        // `Unavailable` is matched per service, without a wildcard, so a new service
        // cannot compile until it has its own text.
        match self {
            Self::Unavailable {
                service: UnavailableService::Keyring,
            } => return Some(SUGGESTION_UNAVAILABLE_KEYRING),
            Self::Unavailable {
                service: UnavailableService::ConfigLock,
            } => return Some(SUGGESTION_UNAVAILABLE_CONFIG_LOCK),
            // X11: a repeated `vps rename A B` after a success lands here, and the
            // generic "do not retry unchanged" hid that the first call had worked.
            Self::VpsNotFound(_) => return Some(SUGGESTION_VPS_NOT_FOUND),
            Self::Software { .. } => {
                return Some(
                    "internal failure with no user-fixable input; report it — retrying \
                     unchanged will not help (exit 70)",
                );
            }
            _ => {}
        }
        match self.retry_kind() {
            RetryKind::TransientNetwork | RetryKind::TransientSsh => {
                Some("retry at most twice with exponential full-jitter backoff (exit 74)")
            }
            RetryKind::TransientTimeout => {
                Some("increase --timeout / --timeout-ms, then retry at most twice with backoff")
            }
            RetryKind::PermanentAuth => {
                Some("change credentials (--key / --password-stdin) or host-key policy; do not blind-retry")
            }
            RetryKind::PermanentRemoteCommand => {
                Some("inspect remote stderr; fix remote command — transport retry will not help")
            }
            RetryKind::PermanentClient => {
                Some("fix CLI arguments, registry state, or schema; do not retry unchanged")
            }
            RetryKind::Cancelled => Some("do not retry after signal or broken pipe"),
            RetryKind::NotRetryable => None,
        }
    }
}
