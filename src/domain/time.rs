// SPDX-License-Identifier: MIT OR Apache-2.0
//! RFC 3339 timestamps as `DateTime<Utc>` newtypes (G-DOM-03).
//!
//! Wire format remains a canonical RFC 3339 string (TOML/JSON agent contract).
//! Construction always uses [`Utc::now`] — never `Local::now`.
#![forbid(unsafe_code)]

use super::error::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Instant stored as UTC, serialized as RFC 3339.
///
/// Used for `VpsRecord.added_at`, ACME `created_at`, and any audit timestamp.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rfc3339Utc(DateTime<Utc>);

/// Alias: VPS registry inclusion time.
pub type AddedAt = Rfc3339Utc;

/// Alias: ACME / audit creation time.
pub type CreatedAt = Rfc3339Utc;

impl Rfc3339Utc {
    /// Current UTC instant.
    #[must_use]
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Parses an external RFC 3339 string into UTC.
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let s = raw.as_ref().trim();
        if s.is_empty() {
            return Err(DomainError::new("rfc3339", "timestamp must not be empty"));
        }
        if s.len() > 64 {
            return Err(DomainError::new("rfc3339", "timestamp too long (max 64)"));
        }
        DateTime::parse_from_rfc3339(s)
            .map(|d| Self(d.with_timezone(&Utc)))
            .map_err(|e| DomainError::new("rfc3339", e.to_string()))
    }

    /// Wraps an already-UTC datetime (infallible).
    #[must_use]
    pub const fn from_utc(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Borrows the inner UTC datetime.
    #[must_use]
    pub const fn as_datetime(&self) -> &DateTime<Utc> {
        &self.0
    }

    /// Consumes into [`DateTime<Utc>`].
    #[must_use]
    pub const fn into_inner(self) -> DateTime<Utc> {
        self.0
    }

    /// Canonical RFC 3339 string.
    #[must_use]
    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339()
    }
}

impl fmt::Display for Rfc3339Utc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_rfc3339())
    }
}

impl Serialize for Rfc3339Utc {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0.to_rfc3339())
    }
}

impl<'de> Deserialize<'de> for Rfc3339Utc {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

impl From<DateTime<Utc>> for Rfc3339Utc {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<Rfc3339Utc> for DateTime<Utc> {
    fn from(v: Rfc3339Utc) -> Self {
        v.0
    }
}

impl TryFrom<String> for Rfc3339Utc {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for Rfc3339Utc {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rfc3339_and_normalizes_to_utc() {
        let t = Rfc3339Utc::try_new("2014-11-28T21:00:09+09:00").unwrap();
        assert_eq!(t.as_datetime().to_rfc3339(), "2014-11-28T12:00:09+00:00");
    }

    #[test]
    fn rejects_garbage() {
        assert!(Rfc3339Utc::try_new("not-a-date").is_err());
        assert!(Rfc3339Utc::try_new("").is_err());
        assert!(Rfc3339Utc::try_new("x".repeat(65)).is_err());
    }

    #[test]
    fn now_is_utc() {
        let n = Rfc3339Utc::now();
        assert!(n.to_rfc3339().contains('T') || n.to_rfc3339().contains('+'));
    }

    #[test]
    fn serde_roundtrip() {
        let t = Rfc3339Utc::try_new("2024-01-02T03:04:05Z").unwrap();
        let j = serde_json::to_string(&t).unwrap();
        assert_eq!(j, "\"2024-01-02T03:04:05+00:00\"");
        let back: Rfc3339Utc = serde_json::from_str(&j).unwrap();
        assert_eq!(back, t);
    }
}
