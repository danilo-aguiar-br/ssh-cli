//! Process-wide switches set once from the global flags of a one-shot run.
#![forbid(unsafe_code)]

/// Process-wide `--no-input` switch.
static NO_INPUT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Installs the `--no-input` policy for this one-shot process.
pub fn set_no_input(value: bool) {
    NO_INPUT.store(value, std::sync::atomic::Ordering::Relaxed);
}

/// Whether stdin reads must fail instead of blocking.
#[must_use]
pub fn is_no_input() -> bool {
    NO_INPUT.load(std::sync::atomic::Ordering::Relaxed)
}

/// Process-wide `--lock-wait-ms` override (`None` = documented default).
static LOCK_WAIT_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Installs the `--lock-wait-ms` budget for this one-shot process.
pub fn set_lock_wait_ms(value: Option<u64>) {
    LOCK_WAIT_MS.store(value.unwrap_or(0), std::sync::atomic::Ordering::Relaxed);
}

/// Effective config-lock budget in milliseconds.
#[must_use]
pub fn lock_wait_ms() -> u64 {
    match LOCK_WAIT_MS.load(std::sync::atomic::Ordering::Relaxed) {
        0 => crate::constants::CONFIG_LOCK_WAIT_MS,
        v => v,
    }
}
