// SPDX-License-Identifier: MIT OR Apache-2.0
//! Legacy Portuguese module path for errors (GAP-AUD-017).
//!
//! Re-exports [`crate::errors`]. Prefer importing from `crate::errors` with English names.
//! Portuguese type aliases are deprecated and will be removed in a future major release.

pub use crate::errors::{exit_codes, SshCliError, SshCliResult};

/// Deprecated Portuguese alias for [`SshCliError`].
#[deprecated(since = "0.5.1", note = "use SshCliError from crate::errors")]
pub type ErroSshCli = crate::errors::SshCliError;

/// Deprecated Portuguese alias for [`SshCliResult`].
#[deprecated(since = "0.5.1", note = "use SshCliResult from crate::errors")]
pub type ResultadoSshCli<T> = crate::errors::SshCliResult<T>;
