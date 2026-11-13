// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted.
#![forbid(unsafe_code)]
//! Closed set of host services whose failure maps to exit 69.
//!
//! A closed enum replaces the former `&'static str` so that `suggestion()` and
//! `layer()` must name every service: adding a variant fails the build until its
//! remediation text and stack layer are chosen, instead of inheriting the keyring
//! advice by accident.

/// Host service that did not answer ([`crate::errors::SshCliError::Unavailable`]).
///
/// `#[non_exhaustive]`: external crates must use a wildcard arm so a new service in a
/// minor release does not break `SemVer` consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnavailableService {
    /// The OS keyring (secret service) that holds the primary key.
    Keyring,
    /// The sibling `config.toml.lock` file held by another ssh-cli invocation.
    ConfigLock,
}

impl UnavailableService {
    /// Stable wire id, as printed after `service unavailable: `.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Keyring => "keyring",
            Self::ConfigLock => "config_lock",
        }
    }
}

impl core::fmt::Display for UnavailableService {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.slug())
    }
}
