// SPDX-License-Identifier: MIT OR Apache-2.0
//! Timeout and character-budget newtypes.
#![forbid(unsafe_code)]

use super::error::DomainError;
use crate::validation::{MAX_CHAR_LIMIT, MAX_TIMEOUT_MS};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::num::NonZeroUsize;

/// Timeout in milliseconds (`1..=MAX_TIMEOUT_MS`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeoutMs(u64);

impl TimeoutMs {
    /// Parses a timeout in the allowed range.
    pub fn try_new(ms: u64) -> Result<Self, DomainError> {
        if ms == 0 || ms > MAX_TIMEOUT_MS {
            return Err(DomainError::new(
                "timeout_ms",
                format!("timeout_ms must be 1..={MAX_TIMEOUT_MS}"),
            ));
        }
        Ok(Self(ms))
    }

    /// Milliseconds value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Alias for [`Self::get`].
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for TimeoutMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for TimeoutMs {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for TimeoutMs {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = u64::deserialize(d)?;
        Self::try_new(v).map_err(serde::de::Error::custom)
    }
}

/// Character budget: `0` on the wire means unlimited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharLimit {
    /// No limit (wire `0`).
    Unlimited,
    /// Finite limit `1..=MAX_CHAR_LIMIT`.
    Limited(NonZeroUsize),
}

impl CharLimit {
    /// Parses wire/config `usize` (`0` → unlimited).
    pub fn try_new(n: usize) -> Result<Self, DomainError> {
        if n == 0 {
            return Ok(Self::Unlimited);
        }
        if n > MAX_CHAR_LIMIT {
            return Err(DomainError::new(
                "char_limit",
                format!("char limit must be 0..={MAX_CHAR_LIMIT}"),
            ));
        }
        Ok(Self::Limited(NonZeroUsize::new(n).expect("n > 0")))
    }

    /// Effective comparison limit (`usize::MAX` when unlimited).
    #[must_use]
    pub fn effective(self) -> usize {
        match self {
            Self::Unlimited => usize::MAX,
            Self::Limited(n) => n.get(),
        }
    }

    /// Wire / config representation (`0` = unlimited).
    #[must_use]
    pub fn wire(self) -> usize {
        match self {
            Self::Unlimited => 0,
            Self::Limited(n) => n.get(),
        }
    }
}

impl Serialize for CharLimit {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(self.wire() as u64)
    }
}

impl<'de> Deserialize<'de> for CharLimit {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = u64::deserialize(d)?;
        let n = usize::try_from(v).map_err(serde::de::Error::custom)?;
        Self::try_new(n).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{MAX_CHAR_LIMIT, MAX_TIMEOUT_MS};

    #[test]
    fn timeout_range() {
        assert!(TimeoutMs::try_new(0).is_err());
        assert!(TimeoutMs::try_new(MAX_TIMEOUT_MS + 1).is_err());
        assert_eq!(TimeoutMs::try_new(1000).unwrap().get(), 1000);
    }

    #[test]
    fn char_limit() {
        assert_eq!(CharLimit::try_new(0).unwrap().effective(), usize::MAX);
        assert_eq!(CharLimit::try_new(10).unwrap().wire(), 10);
        assert!(CharLimit::try_new(MAX_CHAR_LIMIT + 1).is_err());
    }
}
