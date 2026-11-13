//! Operator-language rendering of product errors (human output only).
#![forbid(unsafe_code)]

use super::{t, Message};
use crate::errors::SshCliError;

/// Localizes the human line for a failure that carries no product error type.
///
/// # Why this exists (C2)
///
/// [`localized_error_text`] only accepts an [`SshCliError`]. The last branch of
/// `resolve_exit_code` handles an `anyhow` chain that downcast to neither
/// [`SshCliError`] nor `DomainError`, and it printed the raw chain regardless of
/// `--lang`. B2 localized every *typed* error and left that one untranslated, so
/// the failure a user is least equipped to interpret stayed English-only.
///
/// Unlike the typed path there is nothing to fail open to — the caller has no
/// alternative rendering — so this returns [`String`], never [`Option`]. The
/// `detail` is the upstream chain verbatim and stays English; only the label
/// that classifies it is translated.
///
/// The `--json` envelope is untouched and keeps `error_code` `"unexpected"`.
#[must_use]
pub fn localized_unexpected_text(detail: &str) -> String {
    t(Message::ErrorUnexpected {
        detail: detail.to_string(),
    })
}

/// Renders a domain error in the operator's language, for **human output only**.
///
/// # Why this exists (B2)
///
/// Six `Error*` variants shipped with full English and Brazilian Portuguese
/// translations and never had a single call site: every failure reached the user
/// through `thiserror`'s `#[error("…")]` attribute literal instead, so
/// `--lang pt-BR` produced byte-identical English output. This is the seam that
/// makes the translations reachable.
///
/// # Contract boundary
///
/// This is **not** used for the `--json` envelope. There, `message` stays the
/// English [`std::fmt::Display`] of [`SshCliError`] by contract: agents branch on
/// the stable `error_code` discriminator, and a locale-dependent `message` would
/// silently change the payload an agent parses when the host locale changes.
///
/// Returns [`None`] for any error code without a translation, so the caller falls
/// back to the English `Display`. That fail-open shape means adding a new
/// [`SshCliError`] variant can never blank out the human error line.
#[must_use]
pub fn localized_error_text(err: &SshCliError) -> Option<String> {
    use std::fmt::Write as _;

    // Matched on the variant, not on `error_code()`. Every `#[error("…")]`
    // attribute already carries an English label ("vps '{0}' not found in
    // registry"), so feeding `to_string()` into a template that adds its own
    // label produces "VPS 'vps 'x' not found in registry' not found." Only the
    // inner payload may cross into the translated sentence.
    let msg = match err {
        SshCliError::Config(detail) => Message::ErrorConfig {
            detail: detail.clone(),
        },
        SshCliError::SshConnection(detail) | SshCliError::ConnectionFailed(detail) => {
            Message::ErrorSshConnection {
                detail: detail.clone(),
            }
        }
        SshCliError::SshAuthentication(detail) => Message::ErrorAuthentication {
            detail: detail.clone(),
        },
        SshCliError::AuthenticationFailed => Message::ErrorAuthentication {
            // Unit variant: the remedy hint is the only payload worth carrying.
            detail: "try --password-stdin, --key PATH, --key-passphrase-stdin, \
                     or verify the user"
                .to_string(),
        },
        SshCliError::CommandFailed { exit_code, stderr } => {
            let mut detail = format!("exit {exit_code}");
            if !stderr.is_empty() {
                let _ = write!(detail, ": {stderr}");
            }
            Message::ErrorCommandFailed { detail }
        }
        SshCliError::HostKeyChanged {
            host,
            port,
            expected,
            obtained,
        } => Message::ErrorHostKeyChanged {
            detail: format!(
                "{host}:{port} expected {expected}, got {obtained} \
                 (use --replace-host-key if legitimate)"
            ),
        },
        SshCliError::SshTimeout(ms) | SshCliError::Timeout(ms) => Message::ErrorTimeout {
            detail: format!("{ms}ms"),
        },
        SshCliError::FileNotFound(path) => Message::ErrorFileNotFound { path: path.clone() },
        SshCliError::Unavailable { service } => Message::ErrorUnavailable {
            service: service.slug().to_string(),
        },
        SshCliError::Software { op } => Message::ErrorSoftware {
            op: (*op).to_string(),
        },
        SshCliError::PartialFailure { failed, total, op } => Message::ErrorPartialFailure {
            detail: format!("{failed}/{total} ({op})"),
        },
        SshCliError::InvalidArgument(detail) => Message::ErrorInvalidArgument {
            detail: detail.clone(),
        },
        SshCliError::NoActiveVps => Message::ErrorNoActiveVps,
        SshCliError::VpsNotFound(name) => Message::VpsNotFound { name: name.clone() },
        SshCliError::VpsDuplicate(name) => Message::VpsDuplicate { name: name.clone() },
        // Untranslated variants (io, json, toml_*, broken_pipe, crypto, tls, …)
        // keep the English Display: they are machine-facing plumbing, not
        // operator prose.
        _ => return None,
    };
    Some(t(msg))
}
