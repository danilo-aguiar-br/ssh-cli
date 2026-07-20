// SPDX-License-Identifier: MIT OR Apache-2.0
// G-CLOSE-04: pure module — no `unsafe` permitted.
#![forbid(unsafe_code)]
//! Canonical error module path (`error.rs`) required by clap project layout rules.
//!
//! Implementation lives in [`crate::errors`]; this module re-exports it so both
//! `use ssh_cli::error::*` and `use ssh_cli::errors::*` work.

pub use crate::errors::*;
