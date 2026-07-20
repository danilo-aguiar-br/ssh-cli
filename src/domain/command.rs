// SPDX-License-Identifier: MIT OR Apache-2.0
//! Remote command payload and key path newtypes.
#![forbid(unsafe_code)]

use super::error::DomainError;
use super::limits::CharLimit;
use crate::errors::{SshCliError, SshCliResult};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::path::{Path, PathBuf};

/// Remote shell command payload (non-empty after trim, no NUL bytes).
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RemoteCommand(String);

impl RemoteCommand {
    /// Parses a remote command at the system boundary.
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let s = raw.as_ref();
        if s.trim().is_empty() {
            return Err(DomainError::new("remote_command", "empty command"));
        }
        if s.as_bytes().contains(&0) {
            return Err(DomainError::new(
                "remote_command",
                "command contains null byte",
            ));
        }
        Ok(Self(s.to_owned()))
    }

    /// Borrows the command text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes into inner `String`.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Validates against a [`CharLimit`] (command length).
    pub fn check_length(&self, max: CharLimit) -> SshCliResult<()> {
        let lim = max.effective();
        let len = self.0.chars().count();
        if len > lim {
            return Err(SshCliError::CommandTooLong {
                max: max.wire(),
                len,
            });
        }
        Ok(())
    }
}

impl AsRef<str> for RemoteCommand {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for RemoteCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RemoteCommand {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

/// Non-empty private key path.
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyPath(PathBuf);

impl KeyPath {
    /// Parses a non-empty key path.
    pub fn try_new(raw: impl AsRef<Path>) -> Result<Self, DomainError> {
        let p = raw.as_ref();
        let s = p.to_string_lossy();
        if s.trim().is_empty() {
            return Err(DomainError::new("key_path", "key path must not be empty"));
        }
        Ok(Self(p.to_path_buf()))
    }

    /// From optional string (None / empty → None).
    pub fn try_from_optional(raw: Option<impl AsRef<str>>) -> Result<Option<Self>, DomainError> {
        match raw {
            None => Ok(None),
            Some(s) => {
                let t = s.as_ref().trim();
                if t.is_empty() {
                    Ok(None)
                } else {
                    Self::try_new(t).map(Some)
                }
            }
        }
    }

    /// Borrows as [`Path`].
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Wire string (lossy if non-UTF8).
    #[must_use]
    pub fn to_string_lossy_owned(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }

    /// Consumes into [`PathBuf`].
    #[must_use]
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

impl AsRef<Path> for KeyPath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl fmt::Display for KeyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl Serialize for KeyPath {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string_lossy_owned())
    }
}

impl<'de> Deserialize<'de> for KeyPath {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_command_and_key() {
        assert!(RemoteCommand::try_new("  ").is_err());
        assert!(RemoteCommand::try_new("echo\0x").is_err());
        assert!(RemoteCommand::try_new("uptime").is_ok());
        assert!(KeyPath::try_new("").is_err());
        assert!(KeyPath::try_new("/home/u/.ssh/id_ed25519").is_ok());
        assert!(KeyPath::try_from_optional(Some("")).unwrap().is_none());
    }
}
