// SPDX-License-Identifier: MIT OR Apache-2.0
//! UUID newtypes — v4 correlation vs v7 ordered batch runs (G-DOM-05).
//!
//! - [`CorrelationId`]: pure random (`Uuid::new_v4`) for session/token-like ids.
//! - [`BatchRunId`]: time-ordered (`Uuid::now_v7`) for multi-host fan-out envelopes.
//!
//! Wire: canonical hyphenated lowercase string. Rejects nil/max sentinels on parse.
#![forbid(unsafe_code)]

use super::error::DomainError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use uuid::{Uuid, Variant, Version};

/// Random correlation id (UUID v4).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CorrelationId(Uuid);

/// Time-ordered batch run id (UUID v7) for multi-host JSON envelopes.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BatchRunId(Uuid);

fn reject_sentinel(u: Uuid) -> Result<Uuid, DomainError> {
    if u.is_nil() {
        return Err(DomainError::new("uuid", "nil UUID is a forbidden sentinel"));
    }
    if u.as_u128() == u128::MAX {
        return Err(DomainError::new("uuid", "max UUID is a forbidden sentinel"));
    }
    match u.get_variant() {
        Variant::RFC4122 => {}
        _ => {
            return Err(DomainError::new(
                "uuid",
                "UUID variant must be RFC 4122/9562",
            ));
        }
    }
    Ok(u)
}

impl CorrelationId {
    /// Generates a new random v4 id.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parses a hyphenated UUID string (must be v4 when version is present).
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let u = Uuid::parse_str(raw.as_ref().trim())
            .map_err(|e| DomainError::new("correlation_id", e.to_string()))?;
        let u = reject_sentinel(u)?;
        if let Some(v) = u.get_version() {
            if v != Version::Random {
                return Err(DomainError::new(
                    "correlation_id",
                    format!("expected UUID v4, got {v:?}"),
                ));
            }
        }
        Ok(Self(u))
    }

    /// Borrows inner UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Canonical hyphenated string.
    #[must_use]
    pub fn to_string_canonical(&self) -> String {
        self.0.to_string()
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for CorrelationId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for CorrelationId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

impl BatchRunId {
    /// Generates a new time-ordered v7 id (call **once** per multi-host command).
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Parses a hyphenated UUID string (must be v7).
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let u = Uuid::parse_str(raw.as_ref().trim())
            .map_err(|e| DomainError::new("batch_run_id", e.to_string()))?;
        let u = reject_sentinel(u)?;
        if let Some(v) = u.get_version() {
            if v != Version::SortRand {
                return Err(DomainError::new(
                    "batch_run_id",
                    format!("expected UUID v7, got {v:?}"),
                ));
            }
        }
        Ok(Self(u))
    }

    /// Borrows inner UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Canonical hyphenated string.
    #[must_use]
    pub fn to_string_canonical(&self) -> String {
        self.0.to_string()
    }
}

impl Default for BatchRunId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BatchRunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for BatchRunId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for BatchRunId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_v4_roundtrip() {
        let id = CorrelationId::new();
        let s = id.to_string_canonical();
        let back = CorrelationId::try_new(&s).unwrap();
        assert_eq!(id, back);
        assert_eq!(id.as_uuid().get_version(), Some(Version::Random));
    }

    #[test]
    fn batch_v7_roundtrip() {
        let a = BatchRunId::new();
        let b = BatchRunId::new();
        // Same millisecond may collide on ordering equality; just check parse.
        let s = a.to_string_canonical();
        let back = BatchRunId::try_new(&s).unwrap();
        assert_eq!(a, back);
        assert_eq!(a.as_uuid().get_version(), Some(Version::SortRand));
        // Distinct generations almost always differ.
        let _ = b;
    }

    #[test]
    fn rejects_nil() {
        assert!(CorrelationId::try_new("00000000-0000-0000-0000-000000000000").is_err());
        assert!(BatchRunId::try_new("00000000-0000-0000-0000-000000000000").is_err());
    }

    #[test]
    fn distinct_types_not_interchangeable() {
        let c = CorrelationId::new();
        // BatchRunId rejects v4.
        assert!(BatchRunId::try_new(c.to_string_canonical()).is_err());
    }
}
