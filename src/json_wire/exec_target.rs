// SPDX-License-Identifier: MIT OR Apache-2.0
// GAP-SSH-EXEC-ENVELOPE-002: target identity and its process-wide slot.
#![forbid(unsafe_code)]
//! Which host an execution ran against, and how that host was chosen.
//!
//! # Why this is not in `execution.rs`
//!
//! That module is a set of serialization DTOs: inert structs that describe a result.
//! What lives here is different in kind — a mutable slot holding a decision this
//! process made, written once and read at the failure seam. Mixing a piece of
//! process state into a file of wire types makes the file harder to reason about
//! than either half alone, and pushed it past the component budget besides.

use serde::{Deserialize, Serialize};

/// Where the executing host came from, reported by `ExecutionJson::host_source`.
///
/// GAP-SSH-EXEC-ENVELOPE-002: the single-host envelope named no host at all, so a
/// caller reading stdout could not tell a host typed in argv from one inherited from
/// the on-disk active marker. Those two are the same bytes on the wire and wildly
/// different in blast radius, and the distinction is exactly what an agent needs
/// before it writes a systemd unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TargetSource {
    /// The host name was supplied as a positional in this invocation.
    #[default]
    Argv,
    /// The host came from the on-disk active marker under an explicit `--use-active`.
    ActiveMarker,
    /// The host set came from `--all`, `--hosts` or `--tags`.
    Selector,
}

impl TargetSource {
    /// Wire spelling, kept in one place so text and JSON never drift apart.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Argv => "argv",
            Self::ActiveMarker => "active_marker",
            Self::Selector => "selector",
        }
    }

    /// Whether the host was inherited rather than designated in this invocation.
    #[must_use]
    pub fn is_ambient(self) -> bool {
        matches!(self, Self::ActiveMarker)
    }
}

/// Identity of the host a single-host execution actually ran against.
///
/// Carried separately from [`crate::ssh::ExecutionOutput`] because the output knows
/// what happened and not where: the host is decided during argv parsing, long before
/// any channel is opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecTarget {
    /// Resolved VPS name.
    pub host: String,
    /// How that name was obtained.
    pub source: TargetSource,
}

impl ExecTarget {
    /// Builds a target from a resolved name and its provenance.
    #[must_use]
    pub fn new(host: impl Into<String>, source: TargetSource) -> Self {
        Self {
            host: host.into(),
            source,
        }
    }
}

/// The resolved target, echoed under both the canonical and the legacy names.
///
/// # Why two spellings of one fact
///
/// Explicit Target Designation names these fields `target_resolved` / `target_source`,
/// and the 0.5.5 release shipped them as `host_resolved` / `host_source` before the law
/// was written down. Renaming in place would break every consumer that already reads
/// the 0.5.5 spelling — including the schemas under `docs/schemas/`, which list
/// `host_resolved` in their `required` arrays. Emitting both is the only move that
/// makes the canonical name available without invalidating a contract already in the
/// field: `target_*` is what new readers should bind to, `host_*` is a read alias kept
/// for compatibility and never a second source of truth. They are written from the
/// same [`ExecTarget`] in one place, so they cannot drift.
///
/// Flattened rather than nested: the alias only helps if it appears at the same depth
/// as the field it replaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TargetEcho {
    /// Canonical name of the host this invocation resolved.
    #[serde(default)]
    pub target_resolved: String,
    /// Canonical provenance of [`Self::target_resolved`].
    #[serde(default)]
    pub target_source: TargetSource,
    /// Compatibility alias of [`Self::target_resolved`] (0.5.5 spelling).
    #[serde(default)]
    pub host_resolved: String,
    /// Compatibility alias of [`Self::target_source`] (0.5.5 spelling).
    #[serde(default)]
    pub host_source: TargetSource,
}

impl TargetEcho {
    /// Builds the echo from a resolved target, filling canonical and alias alike.
    #[must_use]
    pub fn new(target: &ExecTarget) -> Self {
        Self {
            target_resolved: target.host.clone(),
            target_source: target.source,
            host_resolved: target.host.clone(),
            host_source: target.source,
        }
    }

    /// Builds the echo from a name plus a provenance already decided by the caller.
    #[must_use]
    pub fn from_parts(host: impl Into<String>, source: TargetSource) -> Self {
        Self::new(&ExecTarget::new(host, source))
    }
}

/// Target resolved by *this* process, readable from the top-level error handler.
///
/// # Why this is process state and not a field on the error
///
/// GAP-SSH-EXEC-ENVELOPE-002 asks for the audit fields on the error path too, and
/// that is where the information matters most: the failing step is the moment a
/// caller most needs to know *where* it failed. The obvious shape — a field on
/// [`crate::errors::SshCliError`] — cannot work here. `resolve_exit_code` recovers
/// errors by `downcast_ref` and *rebuilds* `SshCliError` from `DomainError` and
/// `std::io::Error`, so a per-variant field is discarded on exactly the paths that
/// need it. Threading the target through instead would add an eighth parameter to
/// `print_error_envelope`, which already sits at the `clippy::too_many_arguments`
/// ceiling that `gaps_v062_component_budget` caps at two crate-wide allows.
///
/// The host is not a property of the failure; it is a property of the *process*.
/// This is the same shape `agent_shape::SHAPE` already uses for output shaping, for
/// the same reason: one write after argv resolution, one read at the emission seam.
static RESOLVED_TARGET: std::sync::Mutex<Option<ExecTarget>> = std::sync::Mutex::new(None);

/// Locks the slot, recovering from poisoning rather than aborting the run.
///
/// A poisoned mutex here means another thread panicked while holding it. In a
/// one-shot CLI the correct response is to keep reporting: losing the audit field is
/// bad, losing the error envelope entirely is worse.
fn lock_target() -> std::sync::MutexGuard<'static, Option<ExecTarget>> {
    RESOLVED_TARGET.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("resolved-target mutex was poisoned; recovering (one-shot CLI)");
        poisoned.into_inner()
    })
}

/// Records the host this process resolved, so failures can name it.
///
/// Call this *after* the registry lookup succeeds. Setting it earlier would make a
/// `VpsNotFound` envelope claim a resolved host that was never resolved.
pub fn set_resolved_target(target: &ExecTarget) {
    *lock_target() = Some(target.clone());
}

/// Returns the host resolved by this process, if any.
///
/// [`None`] is meaningful: the failure happened before a host was chosen, so there
/// is nothing to report. Emitting an empty string instead would be an assertion
/// about the target rather than an admission of not having one.
#[must_use]
pub fn resolved_target() -> Option<ExecTarget> {
    lock_target().clone()
}

// Deliberately no test-only reset helper. `serial_test` only serializes serial tests
// against each other, so a non-serial test still runs concurrently with them: an
// in-process assertion on this slot could observe another test's write. The error
// envelope is therefore proven end-to-end in a separate process
// (`tests/gaps_v065_exec_target_designation.rs`), where the one-shot lifecycle makes
// the slot unambiguous by construction.

#[cfg(test)]
mod tests {
    use super::*;

    /// The canonical `target_*` names ship alongside the 0.5.5 `host_*` aliases.
    ///
    /// Both spellings must appear at the *same depth* once flattened — a nested object
    /// would satisfy the struct and break every consumer that reads `.host_resolved`
    /// off the top level, which is the only reason the alias exists.
    #[test]
    fn the_canonical_and_alias_spellings_travel_together() {
        let echo = TargetEcho::from_parts("typed-host", TargetSource::Argv);
        let s = serde_json::to_string(&echo).expect("echo serializes");

        assert!(s.contains(r#""target_resolved":"typed-host""#), "{s}");
        assert!(s.contains(r#""target_source":"argv""#), "{s}");
        assert!(s.contains(r#""host_resolved":"typed-host""#), "{s}");
        assert!(s.contains(r#""host_source":"argv""#), "{s}");
    }

    /// A reader pinned to either spelling deserializes to the same target.
    #[test]
    fn either_spelling_round_trips() {
        let echo = TargetEcho::from_parts("h", TargetSource::Selector);
        let s = serde_json::to_string(&echo).expect("echo serializes");
        let back: TargetEcho = serde_json::from_str(&s).expect("echo deserializes");

        assert_eq!(back, echo);
        assert_eq!(back.target_resolved, back.host_resolved);
        assert_eq!(back.target_source, back.host_source);
    }

    /// The alias is one value under two names, never an independent field.
    #[test]
    fn the_alias_cannot_drift_from_the_canonical_name() {
        let echo = TargetEcho::new(&ExecTarget::new("h", TargetSource::ActiveMarker));
        assert_eq!(echo.target_resolved, echo.host_resolved);
        assert_eq!(echo.target_source, echo.host_source);
    }
}
