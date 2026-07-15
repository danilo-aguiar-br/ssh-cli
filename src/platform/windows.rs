// SPDX-License-Identifier: MIT OR Apache-2.0
//! Windows platform specifics.
//!
//! Configures the UTF-8 console code page (65001) BEFORE any I/O so accented
//! characters are not corrupted on stdin/stdout under cmd.exe, PowerShell 5.1,
//! or PowerShell 7.

use anyhow::Result;

/// Configures the console code page to UTF-8 (65001).
#[cfg(target_os = "windows")]
pub fn configure_utf8_codepage() -> Result<()> {
    use windows_sys::Win32::System::Console::{SetConsoleCP, SetConsoleOutputCP};
    const CP_UTF8: u32 = 65001;
    // SAFETY:
    // 1. Contract: SetConsoleCP/SetConsoleOutputCP require valid code-page identifiers.
    // 2. Invariant: CP_UTF8 (65001) is a well-defined Windows console code page.
    // 3. Caller guarantees this runs once at process start before other console I/O.
    // 4. See windows-sys Win32_System_Console docs for SetConsoleCP/SetConsoleOutputCP.
    unsafe {
        let ok_output = SetConsoleOutputCP(CP_UTF8);
        let ok_input = SetConsoleCP(CP_UTF8);
        if ok_output == 0 {
            tracing::warn!("failed to configure SetConsoleOutputCP(65001)");
        }
        if ok_input == 0 {
            tracing::warn!("failed to configure SetConsoleCP(65001)");
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn configure_utf8_codepage() -> Result<()> {
    Ok(())
}
