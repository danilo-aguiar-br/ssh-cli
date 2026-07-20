// SPDX-License-Identifier: MIT OR Apache-2.0
//! Encapsulated `std::env::set_var` / `remove_var` for unit tests (G-UNSAFE-03/04).
//!
//! `set_var`/`remove_var` are `unsafe fn` (edition 2024 requires `unsafe {}` at
//! the call site; edition 2021 still treats them as deprecated-safe). We always
//! use an explicit `unsafe` block with a `SAFETY:` proof so the crate is
//! ed2024-ready and audit-friendly.

use std::ffi::OsStr;

/// Sets a process environment variable for an isolated serial test.
///
/// # Safety contract (caller)
///
/// Callers must guarantee no concurrent threads in this process read or write
/// the environment for the duration of the mutation window (use
/// `#[serial_test::serial]` on the test).
pub fn set_var<K, V>(key: K, value: V)
where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    // SAFETY:
    // 1. Contract: `std::env::set_var` requires single-threaded env access
    //    (or no concurrent readers/writers) on Unix; always safe on Windows.
    // 2. Invariant: callers use `#[serial_test::serial]` (or single-threaded
    //    test isolation) so no other test mutates/reads the same keys.
    // 3. Scope: only test code; product paths never call this helper.
    // 4. See `std::env::set_var` # Safety and `#[rustc_deprecated_safe_2024]`.
    unsafe {
        std::env::set_var(key, value);
    }
}

/// Removes a process environment variable for an isolated serial test.
///
/// # Safety contract (caller)
///
/// Same as [`set_var`]: serial/single-thread isolation required.
pub fn remove_var<K>(key: K)
where
    K: AsRef<OsStr>,
{
    // SAFETY:
    // 1. Contract: `std::env::remove_var` has the same single-thread / no
    //    concurrent env access requirement as `set_var` on Unix.
    // 2. Invariant: callers use `#[serial_test::serial]` (or equivalent).
    // 3. Scope: test-only restore/cleanup after temporary env mutation.
    // 4. See `std::env::remove_var` # Safety.
    unsafe {
        std::env::remove_var(key);
    }
}
