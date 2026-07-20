// SPDX-License-Identifier: MIT OR Apache-2.0
//! Windows platform specifics.
//!
//! Configures the console **before** any user-facing I/O so:
//! 1. UTF-8 code page (65001) keeps accented / CJK characters intact under
//!    cmd.exe, PowerShell 5.1, PowerShell 7+, and Windows Terminal.
//! 2. Virtual Terminal Processing enables ANSI escape sequences for
//!    `termcolor` on older consoles that default to legacy GDI output.
//!
//! # Safety (G-UNSAFE-06)
//!
//! This module is the **only** product Win32 FFI surface. All `unsafe` calls are:
//! - single-operation blocks with `// SAFETY:` proofs
//! - encapsulated behind the safe API [`configure_console`]
//! - non-fatal on failure (agent pipes often reject console mode changes)
//!
//! Invariants:
//! - Code page `65001` is the documented Windows UTF-8 console page.
//! - Handles from `GetStdHandle(STD_*)` are validated against null / invalid
//!   before `GetConsoleMode` / `SetConsoleMode`.
//! - No raw pointer arithmetic; no ownership of C-allocated memory.

use anyhow::Result;

/// Configures UTF-8 code pages and virtual terminal processing.
///
/// Safe wrapper: all Win32 FFI lives in private helpers with `// SAFETY:`.
#[cfg(target_os = "windows")]
pub fn configure_console() -> Result<()> {
    configure_utf8_codepage();
    enable_virtual_terminal_processing();
    Ok(())
}

/// Configures the console code page to UTF-8 (65001).
#[cfg(target_os = "windows")]
fn configure_utf8_codepage() {
    use windows_sys::Win32::System::Console::{SetConsoleCP, SetConsoleOutputCP};
    const CP_UTF8: u32 = 65001;
    // SAFETY:
    // 1. Contract: SetConsoleOutputCP requires a valid code-page identifier.
    // 2. Invariant: CP_UTF8 (65001) is a well-defined Windows console code page.
    // 3. Caller guarantees this runs once at process start before other console I/O.
    // 4. See windows-sys Win32_System_Console docs for SetConsoleOutputCP.
    let ok_output = unsafe { SetConsoleOutputCP(CP_UTF8) };
    // SAFETY:
    // 1. Contract: SetConsoleCP requires a valid code-page identifier.
    // 2. Invariant: CP_UTF8 (65001) is a well-defined Windows console code page.
    // 3. Caller guarantees this runs once at process start before other console I/O.
    // 4. See windows-sys Win32_System_Console docs for SetConsoleCP.
    let ok_input = unsafe { SetConsoleCP(CP_UTF8) };
    if ok_output == 0 {
        tracing::warn!("failed to configure SetConsoleOutputCP(65001)");
    }
    if ok_input == 0 {
        tracing::warn!("failed to configure SetConsoleCP(65001)");
    }
}

/// Enables `ENABLE_VIRTUAL_TERMINAL_PROCESSING` on stdout and stderr handles.
///
/// Required for ANSI colors under legacy PowerShell 5.1 / conhost when the
/// process is attached to a real console. Failures are non-fatal (piped
/// agents and redirected handles often reject mode changes).
#[cfg(target_os = "windows")]
fn enable_virtual_terminal_processing() {
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
    };
    use windows_sys::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};

    for (handle_id, label) in [
        (STD_OUTPUT_HANDLE, "stdout"),
        (STD_ERROR_HANDLE, "stderr"),
    ] {
        // SAFETY: GetStdHandle with STD_* constants is documented Win32; returns
        // INVALID_HANDLE_VALUE when the handle is unavailable (e.g. fully detached).
        let handle: HANDLE = unsafe { GetStdHandle(handle_id) };
        if handle == 0 || handle == INVALID_HANDLE_VALUE {
            tracing::debug!(handle = label, "console handle unavailable for VT mode");
            continue;
        }
        let mut mode: u32 = 0;
        // SAFETY: handle is a live console handle from GetStdHandle; mode is a
        // valid out-pointer on the stack.
        let got = unsafe { GetConsoleMode(handle, &mut mode) };
        if got == 0 {
            // Not a console (pipe / file) — expected for agent subprocesses.
            tracing::debug!(handle = label, "GetConsoleMode failed (non-console handle)");
            continue;
        }
        let new_mode = mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        if new_mode == mode {
            continue;
        }
        // SAFETY: same handle; new_mode only adds VT flag to previously read mode.
        let set = unsafe { SetConsoleMode(handle, new_mode) };
        if set == 0 {
            tracing::warn!(
                handle = label,
                "failed to enable ENABLE_VIRTUAL_TERMINAL_PROCESSING"
            );
        } else {
            tracing::debug!(handle = label, "virtual terminal processing enabled");
        }
    }
}

/// No-op stub so non-Windows unit tests can name the entry point.
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn configure_console() -> Result<()> {
    Ok(())
}
