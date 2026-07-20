// SPDX-License-Identifier: MIT OR Apache-2.0
//! Operating system signal handling for **one-shot** graceful shutdown.
//!
//! # Shutdown model (Rules Rust — graceful shutdown, CLI one-shot)
//!
//! ssh-cli is **not** a long-lived daemon. Shutdown is **minimal but cooperative**:
//!
//! 1. **Detect** — SIGINT (Ctrl+C / `ctrlc`) and SIGTERM (`signal-hook` on Unix).
//! 2. **Signal** — set shared [`AtomicBool`] flags (`Release` stores).
//! 3. **Await** — long loops (exec / SCP / tunnel / VPS) poll [`should_stop`] and
//!    return; `main` flushes stdio, shuts down the Tokio runtime, then exits
//!    **130** (SIGINT) or **143** (SIGTERM). Broken pipe → **141** (see `errors`).
//!
//! ## What we deliberately do *not* do
//!
//! - No `CancellationToken` / `TaskTracker` tree (overkill for single-session CLI).
//! - No SIGHUP hot-reload, SIGUSR ops, readiness probes, or `sd_notify`.
//! - No `std::process::exit` inside signal handlers (async-signal-unsafe).
//! - SIGPIPE: Rust std ignores the signal so writers get `BrokenPipe` / exit 141.
//!
//! ## Double signal
//!
//! A second SIGINT/SIGTERM sets [`is_force_exit`]. Tunnel aborts outstanding
//! forwards immediately; other paths already return on the first signal.
//!
//! ## Interior mutability (Rules Rust)
//!
//! | Primitive | Role | Ordering |
//! |-----------|------|----------|
//! | `static AtomicBool` (×3) | cancel / term / force flags | store `Release`, load `Acquire` |
//! | `static AtomicU8` | cooperative hit counter | `Relaxed` (count only; flags publish) |
//! | `std::sync::Once` | register handlers once | n/a |
//!
//! No `RefCell`, `Mutex`, `OnceLock<Arc<_>>`, or `Arc` wrapper around the flags:
//! process-global atomics are the smallest correct primitive (handlers capture
//! `'static` references; readers poll without cloning).

use anyhow::Result;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Once;

/// Ensures [`register_handler`] runs at most once (`ctrlc` rejects a second set).
static REGISTER_ONCE: Once = Once::new();

/// Global cancellation flag (SIGINT / Ctrl+C).
///
/// Writers (signal handlers) use `Release`; readers use `Acquire` so a stop
/// observed in a poll loop happens-after the handler store.
static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

/// Global SIGTERM flag (Unix supervisors / `kill` without `-9`).
///
/// Same `Release`/`Acquire` pair as [`CANCEL_FLAG`]. Kept separate so exit
/// code can prefer **143** over **130**.
static FLAG_SIGTERM: AtomicBool = AtomicBool::new(false);

/// Set when a second cooperative signal arrives while already cancelling.
///
/// `Release` store / `Acquire` load — tunnel aborts forwards after observing it.
static FORCE_EXIT: AtomicBool = AtomicBool::new(false);

/// Counts cooperative signals (SIGINT + SIGTERM) for double-signal escalation.
///
/// Ordering: `Relaxed` — this counter only decides local force-exit escalation;
/// it does **not** publish dependent data. The actual stop state is published via
/// `Release` stores on the bool flags above (paired with `Acquire` loads).
static SIGNAL_HITS: AtomicU8 = AtomicU8::new(0);

/// Registers SIGINT (and Unix SIGTERM) handlers that set cancellation flags.
///
/// Must be called once **before** any long-running operation and, on the binary
/// path, **before** the Tokio multi-thread runtime is built (G-UNSAFE-13 /
/// signal-hook first-hook race). Additional calls are safe and silently ignored
/// (`Once` / idempotent).
///
/// # Errors
///
/// Returns an error if the first registration fails (e.g. `ctrlc` cannot
/// install a handler). Failures are **not** ignored.
pub fn register_handler() -> Result<()> {
    let mut register_result: Result<()> = Ok(());
    REGISTER_ONCE.call_once(|| {
        register_result = register_handler_inner();
    });
    register_result
}

fn register_handler_inner() -> Result<()> {
    // SIGINT only — do **not** enable ctrlc `termination` (would also catch
    // SIGTERM, collide with signal-hook, and collapse exit 143 → 130).
    // Closures capture `'static` references to process atomics (no Arc).
    ctrlc::set_handler(|| {
        note_cooperative_signal(&CANCEL_FLAG, &FORCE_EXIT, "SIGINT");
    })?;

    tracing::debug!("Ctrl+C (SIGINT) handler registered successfully");

    #[cfg(unix)]
    {
        // One SIGTERM path only: atomics inside async-signal-safe callback.
        // Distinguishes 143 (term flag) from 130 (cancel flag) and escalates
        // force-exit on a second hit (any cooperative signal via SIGNAL_HITS).
        // `flag::register` only sets one AtomicBool — insufficient for double-hit.
        // No tracing/alloc/mutex here — signal-hook low-level runs in async-signal context.
        // SAFETY:
        // 1. Contract: `signal_hook::low_level::register` requires the action to be
        //    async-signal-safe (POSIX): no alloc, no mutex, no panic, no non-safe OS calls.
        // 2. Callback body: only AtomicBool/AtomicU8 stores and local integer ops
        //    (async-signal-safe subset); cannot panic.
        // 3. SIGTERM is not in signal-hook FORBIDDEN set (SIGKILL/SIGSTOP/SIGILL/…).
        // 4. Binary `main` registers before Tokio multi_thread workers (G-UNSAFE-13);
        //    library `run()` re-entry is idempotent via REGISTER_ONCE.
        // 5. See docs.rs signal_hook::low_level::register # Safety.
        unsafe {
            signal_hook::low_level::register(signal_hook::consts::SIGTERM, || {
                // Publish term before counting so a concurrent poll sees stop even
                // if the second-hit branch races with another handler.
                FLAG_SIGTERM.store(true, Ordering::Release);
                let prev = record_signal_hit();
                if prev >= 1 {
                    FORCE_EXIT.store(true, Ordering::Release);
                }
            })?;
        }
        tracing::debug!("SIGTERM handler registered");
    }

    #[cfg(not(unix))]
    {
        // Windows: no native SIGTERM. ctrlc covers Ctrl+C.
        // Ctrl+Break / console close are not required for this agent one-shot CLI.
    }

    Ok(())
}

/// Atomically bumps [`SIGNAL_HITS`]; returns the previous count.
///
/// `Relaxed` is enough: only relative ordering of hits matters for force-exit;
/// visibility of stop state uses `Release`/`Acquire` on the bool flags.
#[inline]
fn record_signal_hit() -> u8 {
    SIGNAL_HITS.fetch_add(1, Ordering::Relaxed)
}

/// Records a cooperative SIGINT with double-signal force escalation.
///
/// Safe to call from the `ctrlc` dedicated thread (may log). Not for use inside
/// async-signal-safe SIGTERM callbacks.
fn note_cooperative_signal(cancel: &AtomicBool, force: &AtomicBool, kind: &str) {
    let prev = record_signal_hit();
    if prev == 0 {
        cancel.store(true, Ordering::Release);
        // tracing in SIGINT handler: ctrlc runs the handler on a dedicated
        // thread (not the async-signal context), so logging is safe here.
        tracing::debug!(signal = kind, "cancellation signal received");
    } else {
        force.store(true, Ordering::Release);
        cancel.store(true, Ordering::Release);
        tracing::debug!(signal = kind, "force-exit signal (second hit)");
    }
}

/// Returns `true` if the user pressed Ctrl+C (SIGINT).
///
/// Prefer [`should_stop`] when either SIGINT or SIGTERM should abort work.
///
/// # Examples
///
/// ```
/// use ssh_cli::signals::is_cancelled;
///
/// // Before a signal is delivered, returns false
/// assert!(!is_cancelled());
/// ```
#[must_use]
pub fn is_cancelled() -> bool {
    CANCEL_FLAG.load(Ordering::Acquire)
}

/// Returns `true` if the process received SIGTERM.
#[must_use]
pub fn is_terminated() -> bool {
    FLAG_SIGTERM.load(Ordering::Acquire)
}

/// Returns `true` when any cooperative stop signal was observed (SIGINT or SIGTERM).
///
/// Use this in hot loops instead of duplicating `is_cancelled() || is_terminated()`.
///
/// # Examples
///
/// ```
/// use ssh_cli::signals::should_stop;
///
/// // Fresh process / test thread without a delivered signal.
/// assert!(!should_stop());
/// ```
#[must_use]
pub fn should_stop() -> bool {
    is_cancelled() || is_terminated()
}

/// Returns `true` after a second SIGINT/SIGTERM while shutdown is already in progress.
///
/// Tunnel uses this to abort outstanding forwards without waiting on I/O.
#[must_use]
pub fn is_force_exit() -> bool {
    FORCE_EXIT.load(Ordering::Acquire)
}

/// Shared cancellation flag (SIGINT) for tests and advanced integration.
///
/// Prefer [`should_stop`] / [`is_cancelled`] in product paths. Writing the flag
/// is intended for tests and cooperative library callers.
#[must_use]
pub fn cancellation_flag() -> &'static AtomicBool {
    &CANCEL_FLAG
}

/// Shared SIGTERM flag for tests and advanced integration.
#[must_use]
pub fn sigterm_flag() -> &'static AtomicBool {
    &FLAG_SIGTERM
}

/// Shared force-exit flag (second cooperative signal).
#[must_use]
pub fn force_exit_flag() -> &'static AtomicBool {
    &FORCE_EXIT
}

/// Preferred process exit code after cooperative signal handling.
///
/// Order: SIGTERM (143) > SIGINT (130) > `None` (caller decides).
#[must_use]
pub fn signal_exit_code() -> Option<i32> {
    if is_terminated() {
        Some(crate::errors::exit_codes::EX_SIGTERM)
    } else if is_cancelled() {
        Some(crate::errors::exit_codes::EX_SIGINT)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn reset_flags_for_test() {
        cancellation_flag().store(false, Ordering::Release);
        sigterm_flag().store(false, Ordering::Release);
        force_exit_flag().store(false, Ordering::Release);
        SIGNAL_HITS.store(0, Ordering::Relaxed);
    }

    #[test]
    #[serial]
    fn is_cancelled_false_before_signal() {
        reset_flags_for_test();
        assert!(!is_cancelled());
    }

    #[test]
    #[serial]
    fn cancellation_flag_is_stable_static() {
        let flag_a = cancellation_flag();
        let flag_b = cancellation_flag();
        assert!(std::ptr::eq(flag_a, flag_b));
    }

    #[test]
    #[serial]
    fn flag_can_be_set_and_read() {
        let flag = cancellation_flag();
        let previous_value = flag.load(Ordering::Acquire);
        flag.store(previous_value, Ordering::Release);
        assert_eq!(flag.load(Ordering::Acquire), previous_value);
    }

    #[test]
    #[serial]
    fn is_terminated_false_by_default() {
        reset_flags_for_test();
        assert!(!is_terminated());
    }

    #[test]
    #[serial]
    fn sigterm_flag_is_stable_static() {
        let a = sigterm_flag();
        let b = sigterm_flag();
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    #[serial]
    fn is_terminated_true_after_set() {
        let flag = sigterm_flag();
        flag.store(true, Ordering::Release);
        assert!(is_terminated());
        flag.store(false, Ordering::Release);
    }

    #[test]
    #[serial]
    fn is_cancelled_false_after_reset() {
        let flag = cancellation_flag();
        flag.store(true, Ordering::Release);
        assert!(is_cancelled());
        flag.store(false, Ordering::Release);
        assert!(!is_cancelled());
    }

    #[test]
    #[serial]
    fn should_stop_true_when_either_flag_set() {
        reset_flags_for_test();
        assert!(!should_stop());
        cancellation_flag().store(true, Ordering::Release);
        assert!(should_stop());
        cancellation_flag().store(false, Ordering::Release);
        sigterm_flag().store(true, Ordering::Release);
        assert!(should_stop());
        sigterm_flag().store(false, Ordering::Release);
        assert!(!should_stop());
    }

    #[test]
    #[serial]
    fn note_cooperative_signal_sets_force_on_second_hit() {
        reset_flags_for_test();
        let cancel = cancellation_flag();
        let force = force_exit_flag();
        note_cooperative_signal(cancel, force, "SIGINT");
        assert!(is_cancelled());
        assert!(!is_force_exit());
        note_cooperative_signal(cancel, force, "SIGINT");
        assert!(is_force_exit());
        reset_flags_for_test();
    }

    #[test]
    #[serial]
    fn signal_exit_code_prefers_sigterm() {
        reset_flags_for_test();
        assert_eq!(signal_exit_code(), None);
        cancellation_flag().store(true, Ordering::Release);
        assert_eq!(
            signal_exit_code(),
            Some(crate::errors::exit_codes::EX_SIGINT)
        );
        sigterm_flag().store(true, Ordering::Release);
        assert_eq!(
            signal_exit_code(),
            Some(crate::errors::exit_codes::EX_SIGTERM)
        );
        reset_flags_for_test();
    }

    #[test]
    #[serial]
    fn register_handler_is_idempotent() {
        let _ = register_handler();
        assert!(register_handler().is_ok());
    }

    #[test]
    #[serial]
    fn record_signal_hit_escalates_on_second() {
        reset_flags_for_test();
        assert_eq!(record_signal_hit(), 0);
        assert_eq!(record_signal_hit(), 1);
        reset_flags_for_test();
    }
}
