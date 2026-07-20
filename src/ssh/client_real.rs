// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05 / G-AUD-13: thin façade — implementation split under line budget.
#![forbid(unsafe_code)]
//! Real SSH client via `russh` (one-shot connect, exec, SCP).
//!
//! Implementation lives in `client_real_*.rs` include fragments so this module
//! stays under the monólito line budget (G-ERR-12 / G-AUD-13).

include!("client_real_core.rs");
include!("client_real_scp.rs");
include!("client_real_sftp.rs");

#[cfg(test)]
mod real_tests {
    include!("client_real_tests_body.rs");
}
