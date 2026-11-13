// SPDX-License-Identifier: MIT OR Apache-2.0
//! Identity-like string newtypes (VPS name, SSH host/user, tags).
#![forbid(unsafe_code)]

use super::error::DomainError;
use crate::constants::VPS_NAME_EXTRA_BYTES;
use crate::output::sanitize::is_untrusted_char;
use crate::validation::{MAX_FIELD_LEN, MAX_TAGS, MAX_TAG_LEN};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// Logical VPS registry name.
///
/// Three constructors, three strictness levels:
/// - [`VpsName::try_new`] is for names being created (add, rename target,
///   import): ASCII `[A-Za-z0-9._-]`, first character alphanumeric, plus the
///   file-name rules of [`crate::paths::validate_name`].
/// - [`VpsName::from_stored`] is for names typed to look a host up: any text
///   without control or invisible formatting characters.
/// - [`VpsName::from_config_file`] is the read door of an older `config.toml`:
///   only emptiness and length are checked, so no stored name can lock the CLI
///   out. `doctor` reports the names that fail [`VpsName::try_new`].
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct VpsName(String);

impl VpsName {
    /// Parses a name for creation: strict ASCII charset.
    ///
    /// # Errors
    /// Returns [`DomainError`] when the name is empty or longer than
    /// [`MAX_FIELD_LEN`] bytes, does not start with an ASCII letter or digit,
    /// has a byte outside `[A-Za-z0-9._-]`, contains `..`, ends with a dot, or
    /// is a Windows reserved name such as `CON`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssh_cli::domain::VpsName;
    ///
    /// assert!(VpsName::try_new("ssh-digitalocean01").is_ok());
    /// assert!(VpsName::try_new("host.example.com").is_ok());
    /// assert!(VpsName::try_new("-x").is_err());
    /// assert!(VpsName::try_new("a b").is_err());
    /// assert!(VpsName::try_new("a\u{1b}").is_err());
    /// ```
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let s = raw.as_ref();
        check_strict_charset(s)?;
        crate::paths::validate_name(s).map_err(|e| DomainError::new("vps_name", e.to_string()))?;
        Ok(Self(s.to_owned()))
    }

    /// Parses a name typed to look a host up (argv).
    ///
    /// Lenient on purpose: only emptiness, length, control characters and
    /// invisible formatting or bidirectional characters are rejected, so a
    /// host registered with an accented or spaced name stays reachable. The
    /// text is NFC-normalized.
    ///
    /// # Errors
    /// Returns [`DomainError`] when the name is empty, longer than
    /// [`MAX_FIELD_LEN`] bytes, or contains a control, format or bidi character.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssh_cli::domain::VpsName;
    ///
    /// assert!(VpsName::from_stored("servidor produção").is_ok());
    /// assert!(VpsName::from_stored("a\u{202e}b").is_err());
    /// assert!(VpsName::from_stored("").is_err());
    /// ```
    pub fn from_stored(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let nfc = Self::from_config_file(raw)?.0;
        if nfc.chars().any(is_untrusted_char) {
            return Err(DomainError::new(
                "vps_name",
                "name must not contain control or invisible formatting characters",
            ));
        }
        Ok(Self(nfc))
    }

    /// Reads a name from an existing `config.toml` (or an import payload).
    ///
    /// This is the read door for legacy data and the `Deserialize` path. It
    /// checks only that the name is not empty and fits [`MAX_FIELD_LEN`] bytes
    /// after NFC, and it accepts control and formatting characters on purpose:
    /// a registry written by an older version must still load so that
    /// `doctor` can list the bad names and `vps rename` can repair them.
    /// Every text surface escapes names before printing. Creation never uses
    /// this constructor; it goes through [`VpsName::try_new`].
    ///
    /// # Errors
    /// Returns [`DomainError`] when the name is empty or longer than
    /// [`MAX_FIELD_LEN`] bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssh_cli::domain::VpsName;
    ///
    /// assert!(VpsName::from_config_file("legacy \u{1b}name").is_ok());
    /// assert!(VpsName::from_config_file("").is_err());
    /// ```
    pub fn from_config_file(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let s = raw.as_ref();
        let nfc = if s.is_ascii() {
            s.to_owned()
        } else {
            crate::paths::normalize_nfc(s)
        };
        if nfc.is_empty() || nfc.len() > MAX_FIELD_LEN {
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
        Self::from_config_file(s).map_err(serde::de::Error::custom)
    }
}

/// Single pass over the bytes of a new name: length, first byte, charset.
fn check_strict_charset(s: &str) -> Result<(), DomainError> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_FIELD_LEN {
        return Err(DomainError::new(
            "vps_name",
            format!("name must be 1..={MAX_FIELD_LEN} chars"),
        ));
    }
    if !bytes[0].is_ascii_alphanumeric() {
        return Err(DomainError::new(
            "vps_name",
            "name must start with an ASCII letter or digit",
        ));
    }
    if !bytes
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || VPS_NAME_EXTRA_BYTES.contains(b))
    {
        return Err(DomainError::new(
            "vps_name",
            "name may contain only ASCII letters, digits, '.', '_' and '-'",
        ));
    }
    Ok(())
}

/// SSH hostname or IP (non-empty after trim, max [`MAX_FIELD_LEN`]).
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct SshHost(String);

impl SshHost {
    /// Parses a host reference.
    ///
    /// # Errors
    /// Returns a [`DomainError`] when the host is empty, longer than `MAX_FIELD_LEN`, or contains
    /// control characters.
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
    ///
    /// # Errors
    /// Returns a [`DomainError`] when the username is empty, longer than `MAX_FIELD_LEN`, or
    /// contains control characters.
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

/// Single host tag (`1..=MAX_TAG_LEN`, no path separators/controls).
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct HostTag(String);

impl HostTag {
    /// Parses one tag.
    ///
    /// # Errors
    /// Returns a [`DomainError`] when the tag is empty or longer than `MAX_TAG_LEN`, or fails
    /// the tag character rules.
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
///
/// # Errors
/// Returns a [`DomainError`] when a tag is invalid or there are more than `MAX_TAGS` tags.
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
    fn strict_accepts_plain_ascii_names() {
        for n in [
            "ssh-danilo",
            "ssh-digitalocean01",
            "a.b_c-1",
            "host.example.com",
            "9",
        ] {
            assert_eq!(VpsName::try_new(n).unwrap().as_str(), n);
        }
        assert!(VpsName::try_new("x".repeat(MAX_FIELD_LEN)).is_ok());
    }

    #[test]
    fn strict_rejects_everything_else() {
        let long = "x".repeat(MAX_FIELD_LEN + 1);
        for n in [
            "",
            "-x",
            ".x",
            "_x",
            "a b",
            "a;b",
            "a$b",
            "a`b",
            "a&b",
            "a'b",
            "a(b",
            "a)b",
            "a!b",
            "a#b",
            "a\u{1b}",
            "a\u{7}",
            "a\u{202e}",
            "a\u{200b}",
            "a\u{ad}",
            "a\u{61c}",
            "a\u{180e}",
            "a\u{206a}",
            "a\u{e0041}",
            "h\u{0430}st",
            "caf\u{e9}",
            "a..b",
            "a.",
            "CON",
            "nul.txt",
            long.as_str(),
        ] {
            assert!(VpsName::try_new(n).is_err(), "{n:?} must be rejected");
        }
    }

    #[test]
    fn lenient_accepts_legacy_names_and_normalizes() {
        assert_eq!(
            VpsName::from_stored("servidor produ\u{e7}\u{e3}o")
                .unwrap()
                .as_str(),
            "servidor produção"
        );
        assert_eq!(
            VpsName::from_stored("cafe\u{301}").unwrap().as_str(),
            "caf\u{e9}"
        );
        assert!(VpsName::from_stored("a b").is_ok());
        assert!(VpsName::from_stored("-x").is_ok());
    }

    #[test]
    fn lenient_rejects_control_format_empty_and_long() {
        let long = "x".repeat(MAX_FIELD_LEN + 1);
        for n in [
            "",
            "a\u{1b}",
            "a\u{7}",
            "a\u{7f}",
            "a\u{202e}",
            "a\u{200b}",
            "a\u{2066}",
            "a\u{ad}",
            "a\u{61c}",
            "a\u{180e}",
            "a\u{206a}",
            "a\u{206f}",
            "a\u{e0000}",
            "a\u{e007f}",
            "\u{feff}a",
            long.as_str(),
        ] {
            assert!(VpsName::from_stored(n).is_err(), "{n:?} must be rejected");
        }
    }

    #[test]
    fn deserialize_reads_legacy_names_including_control_characters() {
        let ok: Result<VpsName, _> = serde_json::from_str("\"servidor produção\"");
        assert!(ok.is_ok());
        let esc: VpsName = serde_json::from_str("\"a\\u001b\\u202eb\"").unwrap();
        assert_eq!(esc.as_str(), "a\u{1b}\u{202e}b");
        let long = format!("\"{}\"", "x".repeat(MAX_FIELD_LEN + 1));
        assert!(serde_json::from_str::<VpsName>(&long).is_err());
        assert!(serde_json::from_str::<VpsName>("\"\"").is_err());
    }

    #[test]
    fn config_file_door_accepts_what_lookup_refuses() {
        assert!(VpsName::from_config_file("a\u{1b}b").is_ok());
        assert!(VpsName::from_stored("a\u{1b}b").is_err());
        assert!(VpsName::try_new("a\u{1b}b").is_err());
    }

    #[test]
    fn tags() {
        assert!(HostTag::try_new("prod").is_ok());
        assert!(HostTag::try_new("a/b").is_err());
    }
}
