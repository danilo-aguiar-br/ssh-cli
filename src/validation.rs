// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared validation pipeline for external input (G-SERDE-07).
//!
//! Canonical order: **parse → serde → validator → domain**.
//! No product telemetry (local `tracing` only).
#![forbid(unsafe_code)]

use crate::errors::{SshCliError, SshCliResult};
use validator::ValidationErrors;

/// Hard ceiling for `timeout_ms` (1 hour) — G-SERDE-11.
pub const MAX_TIMEOUT_MS: u64 = 3_600_000;

/// Hard ceiling for command/output char limits (0 = unlimited still allowed) — G-SERDE-11.
pub const MAX_CHAR_LIMIT: usize = 10_000_000;

/// Max length for a single host tag.
pub const MAX_TAG_LEN: usize = 64;

/// Max number of tags per host.
pub const MAX_TAGS: usize = 32;

/// Max length for VPS name / host / username string fields.
pub const MAX_FIELD_LEN: usize = 255;

/// Formats `ValidationErrors` into a single agent-readable message (no secrets).
#[must_use]
pub fn format_validation_errors(errs: &ValidationErrors) -> String {
    let mut parts = Vec::new();
    for (field, field_errs) in errs.field_errors() {
        for e in field_errs {
            let code = e.code.as_ref();
            let msg = e
                .message
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or_else(|| code.to_string());
            parts.push(format!("{field}: {msg}"));
        }
    }
    // Nested / schema errors
    for (field, nested) in errs.errors() {
        if let validator::ValidationErrorsKind::Struct(inner) = nested {
            parts.push(format!(
                "{field}: {}",
                format_validation_errors(inner.as_ref())
            ));
        } else if let validator::ValidationErrorsKind::List(map) = nested {
            for (i, inner) in map {
                parts.push(format!(
                    "{field}[{i}]: {}",
                    format_validation_errors(inner.as_ref())
                ));
            }
        }
    }
    if parts.is_empty() {
        "validation failed".into()
    } else {
        parts.join("; ")
    }
}

/// Maps validation failures to [`SshCliError::InvalidArgument`] and logs locally.
pub fn validation_to_error(errs: ValidationErrors) -> SshCliError {
    let msg = format_validation_errors(&errs);
    tracing::warn!(error_class = "validation", %msg, "input validation failed");
    SshCliError::InvalidArgument(msg)
}

/// Convenience: run `Validate` and map errors.
pub fn validate_or_err<T: validator::Validate>(value: &T) -> SshCliResult<()> {
    value.validate().map_err(validation_to_error)
}

/// Custom validator: non-empty after trim.
pub fn validate_nonempty_trimmed(s: &str) -> Result<(), validator::ValidationError> {
    if s.trim().is_empty() {
        let mut e = validator::ValidationError::new("nonempty");
        e.message = Some(std::borrow::Cow::from("must not be empty"));
        return Err(e);
    }
    Ok(())
}

/// Custom validator: SSH port must be 1..=65535 (u16 already caps 65535).
pub fn validate_port_nonzero(port: u16) -> Result<(), validator::ValidationError> {
    if port == 0 {
        let mut e = validator::ValidationError::new("port");
        e.message = Some(std::borrow::Cow::from("invalid SSH port: 0 (use 1..=65535)"));
        return Err(e);
    }
    Ok(())
}

/// Validates host tags (length + cardinality).
pub fn validate_tags(tags: &[String]) -> Result<(), validator::ValidationError> {
    if tags.len() > MAX_TAGS {
        let mut e = validator::ValidationError::new("tags_count");
        e.message = Some(std::borrow::Cow::from(format!(
            "at most {MAX_TAGS} tags allowed"
        )));
        return Err(e);
    }
    for t in tags {
        let t = t.trim();
        if t.is_empty() || t.len() > MAX_TAG_LEN {
            let mut e = validator::ValidationError::new("tag");
            e.message = Some(std::borrow::Cow::from(format!(
                "each tag must be 1..={MAX_TAG_LEN} chars"
            )));
            return Err(e);
        }
        if t.chars().any(|c| c.is_control() || c == '/' || c == '\\') {
            let mut e = validator::ValidationError::new("tag_charset");
            e.message = Some(std::borrow::Cow::from(
                "tag must not contain control chars or path separators",
            ));
            return Err(e);
        }
    }
    Ok(())
}

/// Deserialize TOML with path-aware errors (G-SERDE-08).
pub fn from_toml_str<'de, T: serde::Deserialize<'de>>(s: &'de str) -> SshCliResult<T> {
    let de = toml::Deserializer::new(s);
    serde_path_to_error::deserialize(de).map_err(|e| {
        tracing::warn!(
            error_class = "parse",
            path = %e.path(),
            "TOML deserialize failed"
        );
        SshCliError::Config(format!("TOML at `{}`: {}", e.path(), e.inner()))
    })
}

/// Deserialize JSON with path-aware errors (G-SERDE-08).
pub fn from_json_str<'de, T: serde::Deserialize<'de>>(s: &'de str) -> SshCliResult<T> {
    let mut de = serde_json::Deserializer::from_str(s);
    serde_path_to_error::deserialize(&mut de).map_err(|e| {
        tracing::warn!(
            error_class = "parse",
            path = %e.path(),
            "JSON deserialize failed"
        );
        SshCliError::InvalidArgument(format!(
            "JSON at `{}`: {}",
            e.path(),
            e.inner()
        ))
    })
}

/// JSON deserialize that **warns** on unknown fields (Must-Ignore + G-SERDE-14).
pub fn from_json_str_warn_unused<'de, T: serde::Deserialize<'de>>(s: &'de str) -> SshCliResult<T> {
    let mut unused = Vec::new();
    let mut de = serde_json::Deserializer::from_str(s);
    let value: T = serde_ignored::deserialize(&mut de, |path| {
        unused.push(path.to_string());
    })
    .map_err(|e| {
        // Fall back to path_to_error for better location when structure fails hard.
        let _ = e;
        // Re-parse with path_to_error for the real error message.
        from_json_str::<T>(s).err().unwrap_or_else(|| {
            SshCliError::Json(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "JSON deserialize failed",
            )))
        })
    })?;
    for path in unused {
        tracing::warn!(
            error_class = "validation",
            %path,
            "ignored unknown JSON import field (Must-Ignore)"
        );
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Debug, Validate)]
    struct Sample {
        #[validate(custom(function = "validate_nonempty_trimmed"))]
        name: String,
        #[validate(custom(function = "validate_port_nonzero"))]
        port: u16,
    }

    #[test]
    fn nonempty_and_port() {
        assert!(Sample {
            name: "x".into(),
            port: 22
        }
        .validate()
        .is_ok());
        assert!(Sample {
            name: "  ".into(),
            port: 22
        }
        .validate()
        .is_err());
        assert!(Sample {
            name: "x".into(),
            port: 0
        }
        .validate()
        .is_err());
    }

    #[test]
    fn tags_limits() {
        assert!(validate_tags(&["prod".into()]).is_ok());
        assert!(validate_tags(&["".into()]).is_err());
        assert!(validate_tags(&["a/b".into()]).is_err());
        let many: Vec<_> = (0..MAX_TAGS + 1).map(|i| format!("t{i}")).collect();
        assert!(validate_tags(&many).is_err());
    }
}
