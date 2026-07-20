// SPDX-License-Identifier: MIT OR Apache-2.0
//! Identity-like string newtypes (VPS name, SSH host/user, tags).
#![forbid(unsafe_code)]

use super::error::DomainError;
use crate::validation::{MAX_FIELD_LEN, MAX_TAG_LEN, MAX_TAGS};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// Logical VPS registry name (NFC, path-safe, non-empty).
///
/// Invariant: passes [`crate::paths::validate_name`] and is NFC-normalized.
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct VpsName(String);

impl VpsName {
    /// Parses and normalizes a registry name.
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let s = raw.as_ref();
        crate::paths::validate_name(s).map_err(|e| DomainError::new("vps_name", e.to_string()))?;
        let nfc = crate::paths::normalize_nfc(s);
        if nfc.len() > MAX_FIELD_LEN {
            return Err(DomainError::new(
                "vps_name",
                format!("name must be 1..={MAX_FIELD_LEN} chars"),
            ));
        }
        Ok(Self(nfc))
    }

    /// Borrows the validated name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes into the inner `String`.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for VpsName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for VpsName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for VpsName {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

/// SSH hostname or IP (non-empty after trim, max [`MAX_FIELD_LEN`]).
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct SshHost(String);

impl SshHost {
    /// Parses a host reference.
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let t = raw.as_ref().trim();
        if t.is_empty() {
            return Err(DomainError::new("ssh_host", "host must not be empty"));
        }
        if t.len() > MAX_FIELD_LEN {
            return Err(DomainError::new(
                "ssh_host",
                format!("host must be 1..={MAX_FIELD_LEN} chars"),
            ));
        }
        if t.chars().any(|c| c.is_control()) {
            return Err(DomainError::new(
                "ssh_host",
                "host must not contain control characters",
            ));
        }
        Ok(Self(t.to_owned()))
    }

    /// Borrows the host string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes into inner `String`.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for SshHost {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for SshHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SshHost {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

/// SSH username (non-empty after trim, max [`MAX_FIELD_LEN`]).
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct SshUser(String);

impl SshUser {
    /// Parses a username.
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let t = raw.as_ref().trim();
        if t.is_empty() {
            return Err(DomainError::new("ssh_user", "username must not be empty"));
        }
        if t.len() > MAX_FIELD_LEN {
            return Err(DomainError::new(
                "ssh_user",
                format!("username must be 1..={MAX_FIELD_LEN} chars"),
            ));
        }
        if t.chars().any(|c| c.is_control()) {
            return Err(DomainError::new(
                "ssh_user",
                "username must not contain control characters",
            ));
        }
        Ok(Self(t.to_owned()))
    }

    /// Borrows the username.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes into inner `String`.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for SshUser {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for SshUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SshUser {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

/// Single host tag (1..=MAX_TAG_LEN, no path separators/controls).
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct HostTag(String);

impl HostTag {
    /// Parses one tag.
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let t = raw.as_ref().trim();
        if t.is_empty() || t.len() > MAX_TAG_LEN {
            return Err(DomainError::new(
                "host_tag",
                format!("each tag must be 1..={MAX_TAG_LEN} chars"),
            ));
        }
        if t.chars().any(|c| c.is_control() || c == '/' || c == '\\') {
            return Err(DomainError::new(
                "host_tag",
                "tag must not contain control chars or path separators",
            ));
        }
        Ok(Self(t.to_owned()))
    }

    /// Borrows the tag.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes into inner `String`.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for HostTag {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for HostTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for HostTag {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

/// Parses a tag list with cardinality cap (G-TYPE-05 container).
pub fn try_tags(
    raw: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<Vec<HostTag>, DomainError> {
    let mut out = Vec::new();
    for t in raw {
        out.push(HostTag::try_new(t)?);
        if out.len() > MAX_TAGS {
            return Err(DomainError::new(
                "tags_count",
                format!("at most {MAX_TAGS} tags allowed"),
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_user_name() {
        assert!(SshHost::try_new("  ").is_err());
        assert!(SshUser::try_new("").is_err());
        assert_eq!(SshHost::try_new(" a.b ").unwrap().as_str(), "a.b");
        assert!(VpsName::try_new("lab-01").is_ok());
        assert!(VpsName::try_new("../x").is_err());
    }

    #[test]
    fn tags() {
        assert!(HostTag::try_new("prod").is_ok());
        assert!(HostTag::try_new("a/b").is_err());
    }
}
