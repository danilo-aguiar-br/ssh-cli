// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Bounded concurrency for multi-host SSH fan-out (Rules Rust — paralelismo).
//!
//! # Workload classification
//!
//! **I/O-bound** (SSH/TCP, SCP streams, tunnel accepts/forwards). Network RTT
//! dominates; CPU is secondary (crypto inside russh is already async on Tokio).
//! **Not** CPU-bound ML/batch — do **not** pull Rayon into product paths.
//! **Heavy-memory** singletons stay on `OnceLock` / atomics elsewhere.
//! **Subprocess / systemd-run MemoryMax:** N/A (no child fan-out of heavy
//! workers; this binary *is* the one-shot process).
//!
//! # Where parallelism lives
//!
//! | Surface | Gate | Saturates |
//! |---------|------|-----------|
//! | `health-check|exec|scp --all` / `--hosts` | [`map_bounded`] + `Semaphore` | sockets, remote auth, RAM/session |
//! | `scp` multi-file single-host | **1 session**, serial files (G-PAR-47) | one TCP + auth |
//! | `scp` multi-host × multi-file | `map_bounded` per host + serial files | sessions (not files) |
//! | Tunnel local accepts → channel forwards | `JoinSet` + `Semaphore` | FDs, SSH channels |
//! | Tokio runtime workers | capped from concurrency budget | scheduler threads |
//!
//! Host lists are built only via [`crate::vps::resolve_host_jobs`] (G-PAR-31).
//! Sequential paths (local TOML CRUD, locale, completions, secrets key ops)
//! are **intentionally** serial: work is tiny vs coordination overhead — see
//! module docs on each command handler (G-PAR-28).
//!
//! Fan-out units get `tracing` span `fan_out_unit` (G-PAR-52) + `available_permits`
//! debug on admit (G-PAR-40).
//!
//! # Permit formula
//!
//! ```text
//! permits = clamp(
//!   min(
//!     available_parallelism() * IO_OVERSUBSCRIBE,
//!     (MemAvailable * SAFETY_NUM / SAFETY_DEN) / RAM_PER_TASK_BYTES
//!   ),
//!   MIN_CONCURRENCY ..= HARD_CAP
//! )
//! ```
//!
//! - **IO_OVERSUBSCRIBE = 4** — async I/O may exceed cores without CPU thrash.
//! - **SAFETY_NUM/DEN = 1/2** — 50% of free RAM reserved for OS / peer tools.
//! - **RAM_PER_TASK_BYTES = 16 MiB** — ballpark for one authenticated russh
//!   session + capture buffers (revalidate with `/usr/bin/time -v` after major
//!   dependency bumps; Maximum resident set size of a single `health-check`).
//! - **Non-Linux / no MemAvailable:** CPU budget capped at **8** so `cpus×4`
//!   cannot alone open too many sessions on low-RAM macOS/Windows (G-PAR-25).
//! - Override: CLI `--max-concurrency=N` > auto formula (no env-as-store; G-UNSAFE-14) >
//!   auto formula. `N=0` is rejected at clap parse.
//!
//! # Cancel / panic
//!
//! Permits are held as `OwnedSemaphorePermit` and dropped on task end (RAII),
//! including panic unwind of the task future. Callers must still handle
//! [`tokio::task::JoinError::is_panic`].

use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::task::{Id as TaskId, JoinError, JoinSet};

/// Minimum concurrency (always at least one in-flight op).
pub use crate::constants::MIN_CONCURRENCY;
/// Hard upper bound — protects FD / RAM even on huge hosts (G-AUD-19/23).
pub use crate::constants::HARD_CAP;
/// Async I/O may oversubscribe cores (SSH waits on RTT, not CPU).
pub use crate::constants::IO_OVERSUBSCRIBE;
/// When free RAM cannot be read (non-Linux), cap CPU×IO budget conservatively
/// so low-RAM hosts do not open `cpus×4` sessions blindly (G-PAR-25).
pub use crate::constants::NON_LINUX_CPU_CAP;
/// Documented per-session RAM budget (bytes). See module docs + [`crate::constants::RAM_PER_TASK_BYTES`].
pub use crate::constants::RAM_PER_TASK_BYTES;
/// Keep half of free RAM for the OS and sibling processes.
const RAM_SAFETY_NUM: u64 = 1;
const RAM_SAFETY_DEN: u64 = 2;

// G-UNSAFE-14: concurrency is CLI `--max-concurrency` + auto formula only
// (ENV_MAX_CONCURRENCY env store removed).

/// Process-wide limit set after CLI parse (or defaults from auto formula).
static PROCESS_LIMIT: OnceLock<usize> = OnceLock::new();

/// G-O1: stop admitting new fan-out units after the first unit failure.
static FAIL_FAST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// G-O4: max concurrent SCP file transfers on one session (default 1 = serial).
static SCP_FILE_CONCURRENCY: OnceLock<usize> = OnceLock::new();

/// Peak in-flight tasks observed by [`map_bounded`] (tests / diagnostics).
static PEAK_IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);
static CURRENT_IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);

/// Install the process concurrency limit once (CLI `--max-concurrency` or auto).
///
/// Subsequent calls are ignored (`OnceLock`). Prefer calling from `dispatch`
/// before any multi-host fan-out.
pub fn install_process_limit(limit: usize) {
    let capped = limit.clamp(MIN_CONCURRENCY, HARD_CAP);
    let _ = PROCESS_LIMIT.set(capped);
    tracing::debug!(max_concurrency = capped, "installed process concurrency limit");
}

/// Install global fail-fast policy (G-O1). Default: false (partial success).
pub fn install_fail_fast(enabled: bool) {
    FAIL_FAST.store(enabled, Ordering::Relaxed);
    if enabled {
        tracing::debug!("installed fail-fast multi-host policy");
    }
}

/// Whether multi-host fan-out should stop admission after the first unit failure.
#[must_use]
pub fn fail_fast_enabled() -> bool {
    FAIL_FAST.load(Ordering::Relaxed)
}

/// Install max concurrent SCP files per host session (G-O4). `1` = serial (default).
pub fn install_scp_file_concurrency(n: usize) {
    let capped = n.clamp(MIN_CONCURRENCY, HARD_CAP);
    let _ = SCP_FILE_CONCURRENCY.set(capped);
    tracing::debug!(scp_file_concurrency = capped, "installed scp file concurrency");
}

/// Effective SCP per-session file concurrency (default 1).
#[must_use]
pub fn scp_file_concurrency() -> usize {
    SCP_FILE_CONCURRENCY.get().copied().unwrap_or(MIN_CONCURRENCY)
}

/// Effective concurrency for this process.
///
/// Order: installed process limit (CLI) → auto formula.
#[must_use]
pub fn effective_limit() -> usize {
    if let Some(&n) = PROCESS_LIMIT.get() {
        return n;
    }
    resolve_limit(None)
}

/// Resolve a limit from optional CLI override without installing it.
///
/// Pre-parse bootstrap uses [`auto_limit`]; post-parse installs CLI via
/// `install_process_limit`. Env is **not** a config store (G-ERR-14 / G-UNSAFE-14).
#[must_use]
pub fn resolve_limit(cli_override: Option<usize>) -> usize {
    if let Some(n) = cli_override {
        return n.clamp(MIN_CONCURRENCY, HARD_CAP);
    }
    auto_limit()
}

/// Auto formula: CPUs × oversubscribe vs free-RAM budget, clamped.
///
/// When free RAM is unknown (non-Linux /proc path), the CPU budget is further
/// clamped by [`NON_LINUX_CPU_CAP`] so low-RAM macOS/Windows hosts do not
/// oversubscribe solely from `cpus × IO_OVERSUBSCRIBE` (G-PAR-25).
#[must_use]
pub fn auto_limit() -> usize {
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    let cpu_budget = cpus.saturating_mul(IO_OVERSUBSCRIBE).max(MIN_CONCURRENCY);

    let ram_budget = match free_ram_bytes() {
        Some(free) => {
            let usable = free.saturating_mul(RAM_SAFETY_NUM) / RAM_SAFETY_DEN;
            // G-CLOSE-02: avoid truncating `as usize` on RAM budget math.
            let tasks = usize::try_from(usable / RAM_PER_TASK_BYTES.max(1)).unwrap_or(usize::MAX);
            tasks.max(MIN_CONCURRENCY)
        }
        // No MemAvailable: do not trust unbounded CPU×IO alone on low-RAM hosts.
        None => cpu_budget.clamp(MIN_CONCURRENCY, NON_LINUX_CPU_CAP),
    };

    cpu_budget.min(ram_budget).clamp(MIN_CONCURRENCY, HARD_CAP)
}

/// Tokio worker thread count for `main` (before clap).
///
/// Workers track concurrency budget but stay modest for cold-start: at least 2,
/// at most `min(effective, available_parallelism, 16)`.
#[must_use]
pub fn worker_threads() -> usize {
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    let budget = resolve_limit(None);
    budget.min(cpus).clamp(2, 16)
}

/// Blocking-pool size for rare `spawn_blocking` (crypto edge / sync FS).
#[must_use]
pub fn max_blocking_threads() -> usize {
    resolve_limit(None).clamp(2, 32)
}

/// Reads free RAM (Linux `MemAvailable`; other OS → `None` → CPU-only formula).
#[must_use]
pub fn free_ram_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/meminfo").ok()?;
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("MemAvailable:") {
                let kb: u64 = rest
                    .split_whitespace()
                    .next()?
                    .parse()
                    .ok()?;
                return Some(kb.saturating_mul(1024));
            }
        }
        None
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Shared admission gate for a fan-out scope.
#[must_use]
pub fn semaphore(limit: usize) -> Arc<Semaphore> {
    Arc::new(Semaphore::new(limit.clamp(MIN_CONCURRENCY, HARD_CAP)))
}

/// Acquire one owned permit (for `spawn`ed tasks).
///
/// Product code never calls [`Semaphore::close`]; a closed semaphore is a
/// programming fault. We still **must not panic** on product paths (G-SEC-03):
/// recover by admitting through an ephemeral open semaphore of capacity 1 so
/// one-shot work can finish and surface a normal error elsewhere if needed.
pub async fn acquire_owned(sem: &Arc<Semaphore>) -> OwnedSemaphorePermit {
    match Arc::clone(sem).acquire_owned().await {
        Ok(p) => p,
        Err(_) => {
            tracing::error!(
                "concurrency semaphore was closed unexpectedly; admitting via ephemeral permit (G-SEC-03)"
            );
            // Fresh open semaphore: `acquire_owned` only fails if closed, which
            // a just-created semaphore is not. Loop with yield if the runtime
            // ever reports otherwise (defensive; avoids expect/unwrap).
            loop {
                let emergency = Arc::new(Semaphore::new(1));
                if let Ok(p) = emergency.acquire_owned().await {
                    return p;
                }
                tokio::task::yield_now().await;
            }
        }
    }
}

/// Peak in-flight observed since process start (test helper).
#[must_use]
pub fn peak_in_flight() -> usize {
    PEAK_IN_FLIGHT.load(Ordering::Relaxed)
}

/// Reset peak counters (tests only).
pub fn reset_peak_counters() {
    PEAK_IN_FLIGHT.store(0, Ordering::Relaxed);
    CURRENT_IN_FLIGHT.store(0, Ordering::Relaxed);
}

fn track_enter() {
    let cur = CURRENT_IN_FLIGHT.fetch_add(1, Ordering::Relaxed) + 1;
    PEAK_IN_FLIGHT.fetch_max(cur, Ordering::Relaxed);
}

fn track_leave() {
    CURRENT_IN_FLIGHT.fetch_sub(1, Ordering::Relaxed);
}

/// RAII counter so panicking tasks still release the in-flight slot.
struct InFlightGuard;
impl Drop for InFlightGuard {
    fn drop(&mut self) {
        track_leave();
    }
}

/// Result of one fan-out unit (preserves input order index).
#[derive(Debug)]
pub struct IndexedResult<R> {
    /// Original position in the input collection.
    pub index: usize,
    /// Task outcome (`Ok` work result, `Err` join panic/cancel).
    pub outcome: Result<R, JoinError>,
}

/// Bounded map over independent I/O units.
///
/// Admission: `Semaphore` + `acquire_owned` **before** `JoinSet::spawn`, interleaved
/// with `join_next` so completed tasks free permits (no deadlock).
///
/// # Panic index (G-PAR-24)
///
/// Input indices are tracked by Tokio [`TaskId`] outside the task payload so a
/// panicking unit still reports the correct `IndexedResult::index` (not
/// `usize::MAX`). Callers that need to re-raise panics still use
/// [`JoinError::is_panic`].
///
/// # Cancel safety
///
/// Not cancel-safe as a whole: dropping the future aborts the `JoinSet` (pending
/// tasks cancelled). Individual unit futures should tolerate cancel if they hold
/// remote resources (SSH disconnect on drop of client).
///
/// # Cooperative cancel (G-PAR-39 / G-PAR-44)
///
/// - [`crate::signals::should_stop`]: **stops admission** (no new `spawn_one`);
///   in-flight units drain cooperatively (units should poll `should_stop`).
/// - [`crate::signals::is_force_exit`]: **`JoinSet::abort_all`** then drain
///   (same pattern as tunnel forwards). Aborted units surface as
///   [`JoinError::is_cancelled`].
/// - Units never admitted are omitted from the result vec (callers treat partial
///   batch as cancelled remainder).
pub async fn map_bounded<T, R, F, Fut>(
    items: Vec<T>,
    limit: usize,
    work: F,
) -> Vec<IndexedResult<R>>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
{
    map_bounded_with(items, limit, work, |_r| false).await
}

/// Bounded fan-out with optional per-result fail-fast (G-O1).
///
/// When `is_failure(&result)` returns true **and** [`fail_fast_enabled`],
/// admission stops (same as cooperative cancel). In-flight units drain;
/// never-admitted input indices are **not** present in the returned vec
/// (callers should pad skipped hosts for agent JSON).
///
/// Also respects [`crate::signals::should_stop`] / force abort (G-PAR-39).
pub async fn map_bounded_with<T, R, F, Fut, P>(
    items: Vec<T>,
    limit: usize,
    work: F,
    is_failure: P,
) -> Vec<IndexedResult<R>>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    P: Fn(&R) -> bool + Send + Sync + 'static,
{
    let limit = limit.clamp(MIN_CONCURRENCY, HARD_CAP);
    let sem = semaphore(limit);
    let work = Arc::new(work);
    let is_failure = Arc::new(is_failure);
    let mut set: JoinSet<R> = JoinSet::new();
    let mut task_index: HashMap<TaskId, usize> = HashMap::new();
    let mut iter = items.into_iter().enumerate();
    let mut results: Vec<IndexedResult<R>> = Vec::new();
    let mut admit = true;

    while set.len() < limit {
        if crate::signals::should_stop() {
            admit = false;
            tracing::debug!("fan-out: stop admission (should_stop) during seed");
            break;
        }
        let Some((index, item)) = iter.next() else {
            break;
        };
        spawn_one(&mut set, &mut task_index, &sem, &work, index, item).await;
    }

    loop {
        if set.is_empty() {
            if !admit || crate::signals::should_stop() {
                break;
            }
            if let Some((index, item)) = iter.next() {
                spawn_one(&mut set, &mut task_index, &sem, &work, index, item).await;
                continue;
            }
            break;
        }

        tokio::select! {
            joined = set.join_next_with_id() => {
                match joined {
                    Some(j) => {
                        let before = results.len();
                        push_joined(&mut results, &mut task_index, j);
                        // G-O1: stop admission on first unit failure when enabled.
                        if fail_fast_enabled() {
                            if let Some(last) = results.get(before..) {
                                for r in last {
                                    if let Ok(ref val) = r.outcome {
                                        if is_failure(val) && admit {
                                            admit = false;
                                            tracing::debug!(
                                                index = r.index,
                                                "fan-out: fail-fast stop admission"
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    None => break,
                }

                if crate::signals::is_force_exit() {
                    tracing::debug!(remaining = set.len(), "fan-out: force_exit abort_all");
                    set.abort_all();
                    while let Some(j) = set.join_next_with_id().await {
                        push_joined(&mut results, &mut task_index, j);
                    }
                    break;
                }

                if crate::signals::should_stop() {
                    if admit {
                        admit = false;
                        tracing::debug!("fan-out: stop admission (should_stop); draining");
                    }
                    continue;
                }

                if admit {
                    if let Some((index, item)) = iter.next() {
                        spawn_one(&mut set, &mut task_index, &sem, &work, index, item).await;
                    }
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(
                crate::constants::FAN_OUT_SIGNAL_POLL_INTERVAL_MS,
            )) => {
                if crate::signals::is_force_exit() {
                    tracing::debug!(remaining = set.len(), "fan-out: force_exit abort_all (timer)");
                    set.abort_all();
                    while let Some(j) = set.join_next_with_id().await {
                        push_joined(&mut results, &mut task_index, j);
                    }
                    break;
                }
                if admit && crate::signals::should_stop() {
                    admit = false;
                    tracing::debug!("fan-out: stop admission (should_stop via timer)");
                }
            }
        }
    }

    results.sort_by_key(|r| r.index);
    results
}


fn push_joined<R>(
    results: &mut Vec<IndexedResult<R>>,
    task_index: &mut HashMap<TaskId, usize>,
    joined: Result<(TaskId, R), JoinError>,
) {
    match joined {
        Ok((id, value)) => {
            let index = task_index.remove(&id).unwrap_or(usize::MAX);
            results.push(IndexedResult {
                index,
                outcome: Ok(value),
            });
        }
        Err(e) => {
            let index = task_index.remove(&e.id()).unwrap_or(usize::MAX);
            results.push(IndexedResult {
                index,
                outcome: Err(e),
            });
        }
    }
}

async fn spawn_one<T, R, F, Fut>(
    set: &mut JoinSet<R>,
    task_index: &mut HashMap<TaskId, usize>,
    sem: &Arc<Semaphore>,
    work: &Arc<F>,
    index: usize,
    item: T,
) where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
{
    use tracing::Instrument;

    let permit = acquire_owned(sem).await;
    let available = sem.available_permits();
    tracing::debug!(index, available_permits = available, "fan-out admit");
    let span = tracing::info_span!("fan_out_unit", index, available_permits = available);
    let work = Arc::clone(work);
    let abort = set.spawn(
        async move {
            track_enter();
            let _inflight = InFlightGuard;
            let _permit = permit;
            work(item).await
        }
        .instrument(span),
    );
    task_index.insert(abort.id(), index);
}

/// Convenience: map and unwrap join panics via `resume_unwind`.
///
/// Prefer [`map_bounded`] / [`map_bounded_with`] when partial failure must be reported.
pub async fn map_bounded_ok<T, R, F, Fut>(items: Vec<T>, limit: usize, work: F) -> Vec<R>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
{
    let mut out = Vec::with_capacity(items.len());
    for r in map_bounded(items, limit, work).await {
        match r.outcome {
            Ok(v) => out.push(v),
            Err(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
            Err(_) => {
                // cancelled — skip
            }
        }
    }
    out
}


#[cfg(test)]
#[path = "concurrency_tests.rs"]
mod tests;
