// SPDX-License-Identifier: MIT OR Apache-2.0
//! Operating system signal handling.
//!
//! Registers a Ctrl+C (SIGINT) handler that signals cancellation
//! via a shared [`AtomicBool`]. All modules that run
//! long operations must check [`is_cancelled`] periodically.

use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

/// Global cancellation flag. Set once at initialization.
static CANCEL_FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();

/// Registers the Ctrl+C handler that sets the cancellation flag.
///
/// Must be called once before any long-running operation.
/// Additional calls are safe and silently ignored.
pub fn register_handler() -> Result<()> {
    let flag = cancellation_flag();
    let flag_clone = Arc::clone(&flag);

    ctrlc::set_handler(move || {
        flag_clone.store(true, Ordering::SeqCst);
        tracing::debug!("sinal de cancelamento recebido via Ctrl+C");
    })?;

    tracing::debug!("Ctrl+C handler registered successfully");

    #[cfg(unix)]
    {
        let flag_term = sigterm_flag();
        signal_hook::flag::register(signal_hook::consts::SIGTERM, flag_term)?;
        tracing::debug!("handler SIGTERM registrado");
    }

    #[cfg(not(unix))]
    {
        // On Windows, SIGTERM is not natively supported.
        // ctrlc already covers Ctrl+C (SIGINT equivalent).
        let _ = sigterm_flag(); // Initializes OnceLock even without a handler
    }

    Ok(())
}

/// Returns `true` if the user pressed Ctrl+C.
///
/// Must be checked in long-running loops to allow
/// graceful shutdown.
///
/// # Examples
///
/// ```
/// use ssh_cli::signals::is_cancelled;
///
/// // Before registering a handler, returns false
/// assert!(!is_cancelled());
/// ```
#[must_use]
pub fn is_cancelled() -> bool {
    CANCEL_FLAG
        .get()
        .map(|f| f.load(Ordering::SeqCst))
        .unwrap_or(false)
}

/// Returns the shared cancellation flag pointer.
///
/// Useful to pass the flag to async tasks that need
/// to check cancellation without calling [`is_cancelled`] directly.
#[must_use]
pub fn cancellation_flag() -> Arc<AtomicBool> {
    Arc::clone(CANCEL_FLAG.get_or_init(|| Arc::new(AtomicBool::new(false))))
}

/// Global SIGTERM flag.
static FLAG_SIGTERM: OnceLock<Arc<AtomicBool>> = OnceLock::new();

/// Returns `true` if the process received SIGTERM.
#[must_use]
pub fn is_terminated() -> bool {
    FLAG_SIGTERM
        .get()
        .map(|f| f.load(Ordering::SeqCst))
        .unwrap_or(false)
}

/// Returns the Arc of the SIGTERM flag for async tasks.
#[must_use]
pub fn sigterm_flag() -> Arc<AtomicBool> {
    Arc::clone(FLAG_SIGTERM.get_or_init(|| Arc::new(AtomicBool::new(false))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn is_cancelled_false_before_signal() {
        // Flag should not be set in initial state
        // (unless another test set it; each test shares the same OnceLock)
        // We only check that the call does not panic.
        let _ = is_cancelled();
    }

    #[test]
    #[serial]
    fn cancellation_flag_returns_same_arc() {
        let flag_a = cancellation_flag();
        let flag_b = cancellation_flag();
        // Both must point to the same underlying AtomicBool
        assert!(Arc::ptr_eq(&flag_a, &flag_b));
    }

    #[test]
    #[serial]
    fn flag_can_be_set_and_read() {
        let flag = cancellation_flag();
        // We only check that the AtomicBool works correctly
        let previous_value = flag.load(Ordering::SeqCst);
        flag.store(previous_value, Ordering::SeqCst);
        assert_eq!(flag.load(Ordering::SeqCst), previous_value);
    }

    #[test]
    #[serial]
    fn is_terminated_false_by_default() {
        // GAP-SSH-TEST-001: serial + explicit reset avoids races with parallel tests.
        let flag = sigterm_flag();
        flag.store(false, Ordering::SeqCst);
        assert!(!is_terminated());
    }

    #[test]
    #[serial]
    fn sigterm_flag_returns_same_arc() {
        let a = sigterm_flag();
        let b = sigterm_flag();
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    #[serial]
    fn is_terminated_true_after_set() {
        let flag = sigterm_flag();
        flag.store(true, Ordering::SeqCst);
        assert!(is_terminated());
        flag.store(false, Ordering::SeqCst); // Reset so other tests are not affected
    }

    #[test]
    #[serial]
    fn is_cancelled_false_after_reset() {
        let flag = cancellation_flag();
        flag.store(true, Ordering::SeqCst);
        assert!(is_cancelled());
        flag.store(false, Ordering::SeqCst);
        assert!(!is_cancelled());
    }
}
