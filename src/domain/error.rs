// SPDX-License-Identifier: MIT OR Apache-2.0
//! Domain construction errors and secret helpers.
#![forbid(unsafe_code)]

use secrecy::{ExposeSecret, SecretString};
use std::fmt;

/// Error constructing a domain value (maps to [`crate::errors::SshCliError::Domain`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError {
    /// Stable machine-oriented code (field/invariant).
    pub code: &'static str,
    /// Human/agent message (no secrets).
    pub message: String,
}

impl DomainError {
    /// Builds a domain error.
    #[must_use]
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DomainError {}

/// Maps [`DomainError`] into the crate result type via [`From`].
#[inline]
pub fn domain_err(e: DomainError) -> crate::errors::SshCliError {
    e.into()
}

/// True when a [`SecretString`] holds a non-empty secret (G-TYPE-12 — single helper).
#[must_use]
pub fn secret_nonempty(s: &SecretString) -> bool {
    !s.expose_secret().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_nonempty_helper() {
        assert!(!secret_nonempty(&SecretString::from(String::new())));
        assert!(secret_nonempty(&SecretString::from("x".to_string())));
    }

    #[test]
    fn domain_into_ssh_cli_error() {
        let e: crate::errors::SshCliError =
            DomainError::new("host", "empty").into();
        assert!(matches!(e, crate::errors::SshCliError::Domain(_)));
    }
}
