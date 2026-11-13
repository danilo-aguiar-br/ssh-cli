// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: unit tests extracted for line budget.
#![forbid(unsafe_code)]

use super::*;
use serial_test::serial;
use std::sync::atomic::AtomicUsize;
use std::time::Duration;

#[test]
fn auto_limit_is_clamped() {
    let n = auto_limit();
    assert!(n >= MIN_CONCURRENCY);
    assert!(n <= HARD_CAP);
}

#[test]
fn resolve_cli_override_clamps() {
    assert_eq!(resolve_limit(Some(0)), MIN_CONCURRENCY);
    assert_eq!(resolve_limit(Some(1)), 1);
    assert_eq!(resolve_limit(Some(9999)), HARD_CAP);
}

#[test]
fn worker_threads_sane() {
    let w = worker_threads();
    assert!((2..=16).contains(&w));
}

fn reset_signal_flags() {
    crate::signals::cancellation_flag().store(false, Ordering::Release);
    crate::signals::sigterm_flag().store(false, Ordering::Release);
    crate::signals::force_exit_flag().store(false, Ordering::Release);
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn map_bounded_respects_peak() {
    // Shared process flags — clear so parallel cancel tests cannot starve this one.
    reset_signal_flags();
    let limit = 3usize;
    let items: Vec<u32> = (0..20).collect();
    // Local counters (not process globals) so parallel cargo tests cannot race.
    let current = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let cur_c = Arc::clone(&current);
    let peak_c = Arc::clone(&peak);
    let results = map_bounded(items, limit, move |n| {
        let cur_c = Arc::clone(&cur_c);
        let peak_c = Arc::clone(&peak_c);
        async move {
            let now = cur_c.fetch_add(1, Ordering::SeqCst) + 1;
            peak_c.fetch_max(now, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(15)).await;
            cur_c.fetch_sub(1, Ordering::SeqCst);
            n * 2
        }
    })
    .await;
    assert_eq!(results.len(), 20);
    let observed = peak.load(Ordering::SeqCst);
    assert!(observed <= limit, "peak={observed} limit={limit}");
    assert!(observed >= 1);
    for (i, r) in results.iter().enumerate() {
        assert_eq!(r.index, i);
        assert_eq!(r.outcome.as_ref().unwrap(), &(i as u32 * 2));
    }
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn permit_released_after_panic_in_task() {
    reset_signal_flags();
    let limit = 2usize;
    let progressed = Arc::new(AtomicUsize::new(0));
    let current = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let p = Arc::clone(&progressed);
    let cur_c = Arc::clone(&current);
    let peak_c = Arc::clone(&peak);
    // One panicking task must not permanently exhaust permits.
    let items: Vec<u32> = (0..6).collect();
    let results = map_bounded(items, limit, move |n| {
        let p = Arc::clone(&p);
        let cur_c = Arc::clone(&cur_c);
        let peak_c = Arc::clone(&peak_c);
        async move {
            struct Guard(Arc<AtomicUsize>);
            impl Drop for Guard {
                fn drop(&mut self) {
                    self.0.fetch_sub(1, Ordering::SeqCst);
                }
            }
            let now = cur_c.fetch_add(1, Ordering::SeqCst) + 1;
            peak_c.fetch_max(now, Ordering::SeqCst);
            let _g = Guard(Arc::clone(&cur_c));
            p.fetch_add(1, Ordering::SeqCst);
            if n == 1 {
                panic!("boom");
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
            n
        }
    })
    .await;
    // All join slots accounted for (ok or join error).
    assert_eq!(results.len(), 6);
    let panics = results.iter().filter(|r| r.outcome.is_err()).count();
    assert_eq!(panics, 1);
    // G-PAR-24: panicking unit keeps its input index (item value 1 → index 1).
    let panic_row = results
        .iter()
        .find(|r| r.outcome.is_err())
        .expect("one panic");
    assert_eq!(panic_row.index, 1, "panic must preserve input index");
    assert!(progressed.load(Ordering::SeqCst) >= 5);
    let observed = peak.load(Ordering::SeqCst);
    assert!(observed <= limit, "peak={observed} limit={limit}");
    // After all joins, no stuck in-flight slots.
    assert_eq!(current.load(Ordering::SeqCst), 0);
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn panic_preserves_input_index_not_usize_max() {
    reset_signal_flags();
    let items: Vec<u32> = vec![10, 20, 30];
    let results = map_bounded(items, 2, |n| async move {
        if n == 20 {
            panic!("mid");
        }
        n
    })
    .await;
    assert_eq!(results.len(), 3);
    let panic_row = results.iter().find(|r| r.outcome.is_err()).expect("panic");
    assert_ne!(panic_row.index, usize::MAX);
    assert_eq!(panic_row.index, 1);
    assert!(results[0].outcome.is_ok());
    assert!(results[2].outcome.is_ok());
}

#[tokio::test]
async fn semaphore_acquire_owned_raii() {
    let sem = semaphore(1);
    let p1 = acquire_owned(&sem).await;
    assert_eq!(sem.available_permits(), 0);
    drop(p1);
    assert_eq!(sem.available_permits(), 1);
}

/// G-PAR-44: cooperative cancel stops new admissions; only in-flight complete.
#[tokio::test(flavor = "current_thread")]
#[serial]
async fn map_bounded_stops_admission_on_should_stop() {
    reset_signal_flags();

    let started = Arc::new(AtomicUsize::new(0));
    let started_c = Arc::clone(&started);
    let items: Vec<u32> = (0..20).collect();
    let limit = 2usize;

    // Arm cancel after first tasks have a chance to start.
    let arm = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        crate::signals::cancellation_flag().store(true, Ordering::Release);
    });

    let results = map_bounded(items, limit, move |_n| {
        let started_c = Arc::clone(&started_c);
        async move {
            started_c.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(40)).await;
            1u32
        }
    })
    .await;

    let _ = arm.await;
    reset_signal_flags();

    let n_started = started.load(Ordering::SeqCst);
    // Must not run all 20 units once cancel is observed.
    assert!(
        n_started < 20,
        "admission must stop after should_stop; started={n_started}"
    );
    assert!(n_started >= 1, "at least seed batch should start");
    assert_eq!(results.len(), n_started);
    assert!(results.iter().all(|r| r.outcome.is_ok()));
}

/// G-PAR-39: `force_exit` aborts in-flight `JoinSet` tasks.
#[tokio::test(flavor = "current_thread")]
#[serial]
async fn map_bounded_force_aborts_inflight() {
    reset_signal_flags();

    let items: Vec<u32> = (0..8).collect();
    let limit = 4usize;

    let arm = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(15)).await;
        // Cooperative first, then force (second signal semantics).
        crate::signals::cancellation_flag().store(true, Ordering::Release);
        crate::signals::force_exit_flag().store(true, Ordering::Release);
    });

    let results = map_bounded(items, limit, move |_n| async move {
        // Long sleep so force can abort mid-flight.
        tokio::time::sleep(Duration::from_secs(30)).await;
        1u32
    })
    .await;

    let _ = arm.await;
    reset_signal_flags();

    assert!(!results.is_empty(), "some tasks should have been admitted");
    // At least one join should be cancel/abort (not a clean Ok after 30s sleep).
    let cancelled_or_done = results
        .iter()
        .filter(|r| match &r.outcome {
            Ok(_) => true,
            Err(e) => e.is_cancelled() || e.is_panic(),
        })
        .count();
    assert_eq!(cancelled_or_done, results.len());
    // Must not wait for full 30s×N — test would time out if abort failed.
    let any_cancel = results
        .iter()
        .any(|r| r.outcome.as_ref().err().is_some_and(|e| e.is_cancelled()));
    assert!(
        any_cancel || results.len() < 8,
        "force_exit should abort in-flight or stop admission early"
    );
}

/// `--fail-fast` must be a process-wide policy, not a per-call argument.
#[test]
#[serial]
fn fail_fast_flag_reaches_the_admission_policy() {
    install_fail_fast(false);
    assert!(!fail_fast_enabled(), "the default is partial success");
    install_fail_fast(true);
    assert!(
        fail_fast_enabled(),
        "the flag must be readable from the admission loop that decides whether to keep admitting units"
    );
    install_fail_fast(false);
    assert!(!fail_fast_enabled(), "the policy must be resettable");
}

/// `--scp-file-concurrency` must be what the SCP batch window reads.
///
/// The flag reaching a parser is worth nothing: the transfer path reads
/// [`scp_file_concurrency`]. A flag that installs nothing leaves every multi-file copy
/// serial while the CLI claims otherwise. This drives the same installer `dispatch` calls
/// and reads the same accessor the batch window calls, so both ends are proved to meet.
#[test]
#[serial]
fn scp_file_concurrency_flag_reaches_the_batch_window() {
    install_scp_file_concurrency(4);
    assert_eq!(
        scp_file_concurrency(),
        4,
        "the installer must publish the value the batch window reads"
    );
    // A later install must still win, because the value is a policy the caller set, and
    // the clamp is the only thing that may change it.
    install_scp_file_concurrency(9);
    assert_eq!(scp_file_concurrency(), 9, "the last install is the policy");
}

/// The installer must clamp to the declared budget, never trust the caller.
///
/// `window.max(1)` downstream would hide a zero, so the clamp has to happen here where
/// the value enters the process.
#[test]
#[serial]
fn scp_file_concurrency_installer_clamps() {
    install_scp_file_concurrency(0);
    assert_eq!(
        scp_file_concurrency(),
        MIN_CONCURRENCY,
        "zero would make every transfer silently serial under a flag that says otherwise"
    );
    install_scp_file_concurrency(HARD_CAP + 1000);
    assert_eq!(
        scp_file_concurrency(),
        HARD_CAP,
        "the hard cap must hold at the installer, not only at the parser"
    );
}
