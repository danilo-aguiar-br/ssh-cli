// SPDX-License-Identifier: MIT OR Apache-2.0
//! Legacy Portuguese module path for errors.
//!
//! Re-exports [`crate::errors`]. Prefer importing from `crate::errors` with English names.

pub use crate::errors::{
    exit_codes, SshCliError, SshCliError as ErroSshCli, SshCliResult, SshCliResult as ResultadoSshCli,
};
