// SPDX-License-Identifier: MIT OR Apache-2.0
//! HTTPS URL newtypes for ACME and external endpoints (G-DOM-04).
//!
//! SSH hosts stay as [`super::names::SshHost`] (hostname/IP), never as URLs.
#![forbid(unsafe_code)]

use super::error::DomainError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use url::Url;

/// HTTPS URL with non-empty host (rejects `data:`, `javascript:`, non-https).
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HttpsUrl(Url);

/// ACME order URL (distinct newtype so it cannot be mixed with generic HTTPS URLs).
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AcmeOrderUrl(HttpsUrl);

impl HttpsUrl {
    /// Parses and validates an absolute HTTPS URL.
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        let s = raw.as_ref().trim();
        if s.is_empty() {
            return Err(DomainError::new("https_url", "URL must not be empty"));
        }
        let url = Url::parse(s).map_err(|e| DomainError::new("https_url", e.to_string()))?;
        validate_https(&url)?;
        Ok(Self(url))
    }

    /// Borrows the inner [`Url`].
    #[must_use]
    pub fn as_url(&self) -> &Url {
        &self.0
    }

    /// Canonical string (`Url::as_str`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consumes into [`Url`].
    #[must_use]
    pub fn into_inner(self) -> Url {
        self.0
    }
}

fn validate_https(url: &Url) -> Result<(), DomainError> {
    match url.scheme() {
        "https" => {}
        "http" => {
            // ACME directories and order URLs must be HTTPS in production agents.
            return Err(DomainError::new(
                "https_url",
                "only https scheme is allowed",
            ));
        }
        "data" | "javascript" => {
            return Err(DomainError::new(
                "https_url",
                format!("scheme '{}' is forbidden", url.scheme()),
            ));
        }
        other => {
            return Err(DomainError::new(
                "https_url",
                format!("unsupported scheme '{other}' (expected https)"),
            ));
        }
    }
    if url.host_str().map(str::is_empty).unwrap_or(true) {
        return Err(DomainError::new("https_url", "URL host must not be empty"));
    }
    if url.cannot_be_a_base() {
        return Err(DomainError::new(
            "https_url",
            "URL must be a hierarchical base URL",
        ));
    }
    Ok(())
}

impl fmt::Display for HttpsUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for HttpsUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Serialize for HttpsUrl {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for HttpsUrl {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<String> for HttpsUrl {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for HttpsUrl {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl AcmeOrderUrl {
    /// Parses an ACME order URL (HTTPS).
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self, DomainError> {
        HttpsUrl::try_new(raw)
            .map(Self)
            .map_err(|e| DomainError::new("acme_order_url", e.message))
    }

    /// From an already-validated HTTPS URL.
    #[must_use]
    pub fn from_https(url: HttpsUrl) -> Self {
        Self(url)
    }

    /// Canonical string for ACME client resume.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Borrows inner URL.
    #[must_use]
    pub fn as_url(&self) -> &Url {
        self.0.as_url()
    }

    /// Owned string for APIs that take `String`.
    #[must_use]
    pub fn to_string_owned(&self) -> String {
        self.as_str().to_owned()
    }
}

impl fmt::Display for AcmeOrderUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for AcmeOrderUrl {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AcmeOrderUrl {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<String> for AcmeOrderUrl {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https() {
        let u = HttpsUrl::try_new("https://acme-v02.api.letsencrypt.org/directory").unwrap();
        assert_eq!(u.as_url().scheme(), "https");
        assert!(u.as_url().host_str().is_some());
    }

    #[test]
    fn rejects_http_data_js() {
        assert!(HttpsUrl::try_new("http://example.com").is_err());
        assert!(HttpsUrl::try_new("data:text/plain,hi").is_err());
        assert!(HttpsUrl::try_new("javascript:alert(1)").is_err());
        assert!(HttpsUrl::try_new("not a url").is_err());
    }

    #[test]
    fn acme_order_serde() {
        let u = AcmeOrderUrl::try_new("https://example.com/acme/order/1").unwrap();
        let j = serde_json::to_string(&u).unwrap();
        let back: AcmeOrderUrl = serde_json::from_str(&j).unwrap();
        assert_eq!(u, back);
    }
}
