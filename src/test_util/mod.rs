// SPDX-License-Identifier: MIT OR Apache-2.0
//! Test-only helpers that encapsulate process-global mutations requiring `unsafe`.
//!
//! Product modules keep `#![forbid(unsafe_code)]` and call these safe wrappers.
//! Call sites **must** run under `#[serial_test::serial]` (or equivalent isolation).

#![cfg(test)]

pub mod env;
