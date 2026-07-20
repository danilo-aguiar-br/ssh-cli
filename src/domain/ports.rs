// SPDX-License-Identifier: MIT OR Apache-2.0
//! Port newtypes (SSH vs bind/ephemeral).
#![forbid(unsafe_code)]

use super::error::DomainError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::num::NonZeroU16;

/// SSH TCP port in `1..=65535` (zero is unrepresentable).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SshPort(NonZeroU16);

impl SshPort {
    /// Parses a non-zero SSH port.
    pub fn try_new(port: u16) -> Result<Self, DomainError> {
        NonZeroU16::new(port)
            .map(Self)
            .ok_or_else(|| DomainError::new("ssh_port", "invalid SSH port: 0 (use 1..=65535)"))
    }

    /// Port as `u16` (always ≥ 1).
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0.get()
    }

    /// Alias for [`Self::get`].
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.get()
    }
}

impl fmt::Display for SshPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl Serialize for SshPort {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u16(self.get())
    }
}

impl<'de> Deserialize<'de> for SshPort {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = u16::deserialize(d)?;
        Self::try_new(v).map_err(serde::de::Error::custom)
    }
}

/// Local bind port: `0` means ephemeral OS assignment; otherwise `1..=65535`.
///
/// Distinct from [`SshPort`] (DRY with discipline — do not unify).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BindPort(u16);

impl BindPort {
    /// Accepts any `u16` (including 0 for ephemeral).
    #[must_use]
    pub const fn new(port: u16) -> Self {
        Self(port)
    }

    /// Underlying port number.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// True when the OS should pick an ephemeral port.
    #[must_use]
    pub const fn is_ephemeral(self) -> bool {
        self.0 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn ssh_port_rejects_zero() {
        assert!(SshPort::try_new(0).is_err());
        assert_eq!(SshPort::try_new(22).unwrap().get(), 22);
    }

    #[test]
    fn ssh_port_zero_cost() {
        assert_eq!(size_of::<SshPort>(), size_of::<u16>());
        assert_eq!(size_of::<Option<SshPort>>(), size_of::<u16>());
        assert_eq!(align_of::<SshPort>(), align_of::<u16>());
    }

    #[test]
    fn bind_port_allows_zero() {
        assert!(BindPort::new(0).is_ephemeral());
        assert!(!BindPort::new(8080).is_ephemeral());
    }
}
