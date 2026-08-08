// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Shared tunnel counters and the close-reason they resolve to.
//!
//! Extracted from `tunnel.rs` so that module stays under the component budget: the
//! counters are one responsibility (observable state of a running tunnel) and the
//! orchestration around them is another. `TunnelStats` is re-exported from
//! `crate::tunnel`, so every existing path keeps working unchanged.
//!
//! Workload: lock-free atomics only. Written by the accept loop, read by the
//! deadline wrapper on a different task, hence `Relaxed` for pure counters and
//! `Acquire`/`Release` for the flags whose ordering decides the emitted reason.

use std::sync::atomic::{AtomicBool, Ordering};

/// Counters shared between the accept loop and the deadline wrapper.
///
/// These deliberately live *outside* the future passed to [`tokio::time::timeout`].
/// On the deadline path — the most common ending — that future is dropped mid-poll and
/// never reaches its own tail, so anything owned by it would be lost exactly when the
/// shutdown event matters most. Holding the counters in an `Arc` the wrapper also owns
/// lets `tunnel_closed` be emitted on every ending.
#[derive(Debug, Default)]
pub struct TunnelStats {
    /// Connections accepted and handed to a forward task.
    pub forwards_served: std::sync::atomic::AtomicU64,
    /// Times a connection waited for a concurrency permit.
    pub capacity_waits: std::sync::atomic::AtomicU64,
    /// OS-assigned local port after bind (0 until the listener is up).
    pub effective_port: std::sync::atomic::AtomicU32,
    /// Set when the accept loop stopped because of a signal.
    pub stopped_by_signal: AtomicBool,
    /// Set when the accept loop stopped because of a fatal accept error.
    pub stopped_by_accept_error: AtomicBool,
}

impl TunnelStats {
    /// Resolves the close reason from the flags the loop managed to set.
    ///
    /// Defaults to [`crate::json_wire::TunnelCloseReason::Deadline`]: if neither flag is
    /// set the loop was still serving when it was cancelled, which is precisely the
    /// deadline ending.
    #[must_use]
    pub fn close_reason(&self) -> crate::json_wire::TunnelCloseReason {
        use crate::json_wire::TunnelCloseReason;
        if self.stopped_by_accept_error.load(Ordering::Acquire) {
            TunnelCloseReason::AcceptError
        } else if self.stopped_by_signal.load(Ordering::Acquire) {
            TunnelCloseReason::Signal
        } else {
            TunnelCloseReason::Deadline
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TunnelStats;
    use std::sync::atomic::Ordering;

    /// `tunnel_closed` and its counters once existed only inside the printer.
    ///
    /// The 0.5.4 audit found `tunnel_closed`, `forwards_served` and `capacity_waits`
    /// mentioned in exactly one place outside the emitter: a documentation test checking
    /// the CHANGELOG names them. Deleting the emission entirely would not have turned the
    /// suite red — the same failure mode `tests/test_quality.rs` exists to prevent, one
    /// step removed: prose validating prose instead of an assertion between two literals.
    #[test]
    fn close_reason_is_derived_from_the_flags_the_loop_managed_to_set() {
        let stats = TunnelStats::default();
        assert_eq!(
            stats.close_reason(),
            crate::json_wire::TunnelCloseReason::Deadline,
            "neither flag set means the loop was still serving when it was cancelled"
        );

        let signalled = TunnelStats::default();
        signalled.stopped_by_signal.store(true, Ordering::Release);
        assert_eq!(
            signalled.close_reason(),
            crate::json_wire::TunnelCloseReason::Signal
        );

        let failed = TunnelStats::default();
        failed
            .stopped_by_accept_error
            .store(true, Ordering::Release);
        assert_eq!(
            failed.close_reason(),
            crate::json_wire::TunnelCloseReason::AcceptError
        );

        // Accept error wins: a loop that died early and *then* saw a signal stopped for
        // the reason that actually needs operator attention.
        let both = TunnelStats::default();
        both.stopped_by_signal.store(true, Ordering::Release);
        both.stopped_by_accept_error.store(true, Ordering::Release);
        assert_eq!(
            both.close_reason(),
            crate::json_wire::TunnelCloseReason::AcceptError
        );
    }
}
