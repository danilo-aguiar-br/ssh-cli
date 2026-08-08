// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! ssh-cli internationalization system (Rules Rust multi-idioma).
//!
//! Provides bilingual [`Language`] with [`Message`] as the **single source** of
//! human UI strings. Locale detection / BCP47 negotiation lives in [`crate::locale`].
//!
//! ## Design (agent-first one-shot)
//!
//! - **MVP locales:** neutral `en` + `pt-BR` (100% key parity via exhaustive `match`).
//! - **Not Fluent FTL at runtime:** size-sensitive CLI; compiler-enforced enum
//!   translations are the embedded equivalent of `i18n-embed` for two locales.
//! - **JSON / agent wire:** stable English field names and technical
//!   [`crate::errors::SshCliError`] `Display` (not locale-dependent).
//! - **Human UX** (success/status/cancel lines): always via [`Message`] / [`t`].
//! - Optional top-20 locales: Cargo features `i18n-*` (stubs until translations land).
//!
//! ## Precedence (see [`crate::locale`])
//!
//! 1. CLI `--lang` → 2. persisted XDG `lang` (`locale set`) →
//! 3. `sys_locale` → 4. `Language::English`.
//!
//! `SSH_CLI_LANG` is historical only — not read as a product store.

use anyhow::Result;
use unic_langid::LanguageIdentifier;

use crate::errors::SshCliError;

// C3: the translation tables are data, one exhaustive `match` arm per variant,
// and they made this file the second-largest in the crate. Splitting them out by
// locale keeps the exhaustiveness guarantee (each `match` still has to cover
// every variant) while making a one-sided edit visible as a single-file diff.
mod en;
mod pt;

/// Text direction for terminal rendering (LTR MVP; RTL reserved for `i18n-rtl`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TextDirection {
    /// Left-to-right (Latin, CJK horizontal, etc.).
    Ltr,
    /// Right-to-left (Arabic, Hebrew) — not active in default build.
    Rtl,
}

/// Languages supported by the internationalization system.
///
/// Single source of truth for product locales in this binary. Do **not** use
/// `bool` / raw `String` / integers for language in APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Language {
    /// Neutral English (`en`) — default / agent-stable technical baseline.
    English,
    /// Brazilian Portuguese (`pt-BR`) — mandatory MVP pair with `en`.
    Portuguese,
}

impl Language {
    /// Locales compiled into the default binary (MVP: `en`, `pt-BR` only).
    pub const AVAILABLE: &'static [Language] = &[Language::English, Language::Portuguese];

    /// Canonical BCP47 tag for this product locale.
    ///
    /// English is neutral `en` (not `en-US` alone). Portuguese is always `pt-BR`.
    #[must_use]
    pub const fn bcp47(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Portuguese => "pt-BR",
        }
    }

    /// Structured BCP47 identifier (`unic-langid`).
    ///
    /// Built-in tags are compile-time constants (`en`, `pt-BR`). On parse
    /// failure (should never happen), falls back to the default undetermined
    /// identifier — **no panic** on product paths (G-SEC-07).
    #[must_use]
    pub fn language_identifier(self) -> LanguageIdentifier {
        self.bcp47()
            .parse()
            .unwrap_or_else(|_| LanguageIdentifier::default())
    }

    /// Base fallback language for regionals (MVP: English).
    #[must_use]
    pub const fn fallback(self) -> Language {
        match self {
            Self::English => Self::English,
            Self::Portuguese => Self::English,
        }
    }

    /// Writing direction for this locale.
    #[must_use]
    pub const fn direction(self) -> TextDirection {
        match self {
            Self::English | Self::Portuguese => TextDirection::Ltr,
        }
    }

    /// ISO 15924 script subtag (MVP Latin only).
    #[must_use]
    pub const fn script(self) -> &'static str {
        match self {
            Self::English | Self::Portuguese => "Latn",
        }
    }

    /// Maps a negotiated [`LanguageIdentifier`] to a product [`Language`].
    ///
    /// Matches primary language subtag: `en*` → English, `pt*` → Portuguese.
    /// Region-specific product choice for Portuguese is always `pt-BR` in MVP
    /// (no `pt-PT` variant compiled without a feature).
    #[must_use]
    pub fn from_langid(id: &LanguageIdentifier) -> Option<Language> {
        match id.language.as_str() {
            "en" => Some(Self::English),
            "pt" => Some(Self::Portuguese),
            _ => None,
        }
    }
}

/// All system UI messages.
///
/// SINGLE source of user-visible strings. Each variant has an exhaustive
/// translation in `en()` and `pt()`. FORBIDDEN to use UI literals outside this enum.
///
/// Variants with dynamic fields (e.g. `{ name: String }`) allow including
/// contextual data in the message. Message is not `Copy` because
/// `String` fields are not `Copy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    // VPS
    /// No VPS registered in the configuration file.
    VpsRegistryEmpty,
    /// VPS successfully added to the registry.
    VpsAdded {
        /// Name of the added VPS.
        name: String,
    },
    /// VPS successfully removed from the registry.
    VpsRemoved {
        /// Name of the removed VPS.
        name: String,
    },
    /// Attempt to add a VPS that already exists.
    VpsDuplicate {
        /// Name of the duplicate VPS.
        name: String,
    },
    /// Requested VPS was not found in the registry.
    VpsNotFound {
        /// Name of the missing VPS.
        name: String,
    },
    /// Active VPS selected for subsequent operations.
    VpsActiveSelected {
        /// Name of the selected VPS.
        name: String,
    },
    // Errors (B2).
    //
    // These are reached exclusively from [`localized_error_text`], which is
    // called on the *human* branch of the top-level error emitter. The `--json`
    // envelope keeps `SshCliError`'s English `Display`, because agents branch on
    // the stable `error_code` discriminator and must never parse localized
    // prose. Every variant carries the upstream detail verbatim so no diagnostic
    // information is lost in translation.
    /// Configuration could not be read or written.
    ErrorConfig {
        /// Underlying failure detail (English, from the source error).
        detail: String,
    },
    /// Error establishing an SSH connection to the remote server.
    ErrorSshConnection {
        /// Underlying failure detail.
        detail: String,
    },
    /// SSH authentication was rejected by the server.
    ErrorAuthentication {
        /// Underlying failure detail.
        detail: String,
    },
    /// Remote SSH command execution failed.
    ErrorCommandFailed {
        /// Underlying failure detail.
        detail: String,
    },
    /// Remote host key no longer matches the pinned entry.
    ErrorHostKeyChanged {
        /// Underlying failure detail.
        detail: String,
    },
    /// Operation exceeded its deadline.
    ErrorTimeout {
        /// Underlying failure detail.
        detail: String,
    },
    /// Requested file was not found.
    ErrorFileNotFound {
        /// Path that was not found.
        path: String,
    },
    /// A required external service is unavailable.
    ErrorUnavailable {
        /// Service name (for example `keyring`).
        service: String,
    },
    /// The program itself failed in a way retrying cannot fix.
    ErrorSoftware {
        /// Failing operation name (for example `rng`).
        op: String,
    },
    /// A multi-host fan-out succeeded only in part.
    ErrorPartialFailure {
        /// Underlying failure detail.
        detail: String,
    },
    /// Invalid argument supplied to the operation.
    ErrorInvalidArgument {
        /// Detail of the invalid argument.
        detail: String,
    },
    /// A failure that carries no product error type.
    ///
    /// Reached from the last branch of `resolve_exit_code`, where an `anyhow`
    /// chain held neither `SshCliError` nor `DomainError`. That branch printed
    /// the raw English chain regardless of `--lang`, so the one error a user is
    /// least equipped to interpret was also the only one never translated.
    ErrorUnexpected {
        /// Underlying failure detail (English, from the `anyhow` chain).
        detail: String,
    },
    /// VPS record edited successfully.
    VpsEdited {
        /// VPS name.
        name: String,
    },
    /// Export completed.
    ExportCompleted {
        /// Destination path.
        path: String,
    },
    /// Import completed.
    ImportCompleted,
    /// Primary key ready.
    PrimaryKeyReady {
        /// Key source identifier.
        source: String,
        /// Key file path.
        key_file: String,
    },
    /// Re-encrypt completed.
    ReencryptCompleted {
        /// Host count.
        hosts: usize,
    },
    // Tunnel
    /// Instruction to stop the tunnel via Ctrl+C.
    TunnelPressCtrlC,
    // Health Check
    /// Successful VPS connectivity check.
    HealthCheckOk {
        /// Name of the checked VPS.
        name: String,
    },
    /// Operation cancelled by user signal (Ctrl+C or SIGTERM).
    OperationCancelled,
    // SCP (GAP-SSH-SCP-020)
    /// SCP upload completed.
    ScpUploadCompleted {
        /// Bytes transferred.
        bytes: u64,
        /// Duration in milliseconds.
        ms: u64,
    },
    /// SCP download completed.
    ScpDownloadCompleted {
        /// Bytes transferred.
        bytes: u64,
        /// Duration in milliseconds.
        ms: u64,
    },
    /// Upload refused: local path is a directory (file-only, no -r).
    ScpUploadFileOnly,
    /// Download refused: local path is already a directory.
    ScpDownloadLocalNotDirectory,
    /// SFTP upload completed (G-SFTP).
    SftpUploadCompleted {
        /// Bytes transferred.
        bytes: u64,
        /// Duration in milliseconds.
        ms: u64,
    },
    /// SFTP download completed (G-SFTP).
    SftpDownloadCompleted {
        /// Bytes transferred.
        bytes: u64,
        /// Duration in milliseconds.
        ms: u64,
    },
    /// SFTP filesystem operation completed on a single path (A4).
    ///
    /// A4: `mkdir`, `rmdir`, `rm` and `stat` built their human line with an inline
    /// English `format!`, so `--lang pt-BR` silently produced English for exactly the
    /// commands an operator reads most often. This is the same defect already recorded
    /// for the tunnel banner: a translation that existed but no call site could reach.
    SftpFsOpDone {
        /// Operation name (`mkdir`, `rmdir`, `rm`, `stat`).
        op: String,
        /// Remote path acted upon.
        path: String,
        /// Duration in milliseconds.
        ms: u64,
    },
    /// SFTP filesystem operation completed with a destination path (`rename`).
    SftpFsOpDoneTo {
        /// Operation name (`rename`).
        op: String,
        /// Source path.
        path: String,
        /// Destination path.
        to: String,
        /// Duration in milliseconds.
        ms: u64,
    },
    // Locale diagnostics / preference
    /// Locale preference saved.
    LocalePreferenceSaved {
        /// BCP47 tag written.
        lang: String,
        /// Path of the preference file.
        path: String,
    },
    /// Locale preference cleared.
    LocalePreferenceCleared,
    /// Header for `locale` show output.
    LocaleStatusTitle,
    // Tunnel banners (human TTY only; agents read the JSON events instead)
    /// Local forward is listening.
    TunnelLocalListening {
        /// Effective local bind address.
        bind: String,
        /// Effective local port (OS-assigned when 0 was requested).
        port: u16,
        /// Remote destination host.
        remote_host: String,
        /// Remote destination port.
        remote_port: u16,
        /// Registry name of the host.
        vps: String,
        /// Deadline in milliseconds.
        timeout_ms: u64,
    },
    /// SOCKS5 proxy is listening.
    TunnelSocks5Listening {
        /// Effective local bind address.
        bind: String,
        /// Effective local port.
        port: u16,
        /// Registry name of the host.
        vps: String,
        /// Deadline in milliseconds.
        timeout_ms: u64,
    },
    /// Forward to a remote Unix socket is listening.
    TunnelStreamLocalListening {
        /// Effective local bind address.
        bind: String,
        /// Effective local port.
        port: u16,
        /// Remote Unix socket path.
        socket_path: String,
        /// Registry name of the host.
        vps: String,
        /// Deadline in milliseconds.
        timeout_ms: u64,
    },
    /// Reverse forward established on the server.
    TunnelReverseListening {
        /// Address the server bound.
        remote_bind: String,
        /// Port the server allocated.
        remote_port: u16,
        /// Local delivery host.
        local_host: String,
        /// Local delivery port.
        local_port: u16,
        /// Registry name of the host.
        vps: String,
        /// Deadline in milliseconds.
        timeout_ms: u64,
    },
}

impl Message {
    /// Returns the message string in the specified language.
    ///
    /// Deterministic method for tests — does not depend on global state.
    pub fn text(&self, language: Language) -> String {
        match language {
            Language::English => en::en(self),
            Language::Portuguese => pt::pt(self),
        }
    }
}

/// Initializes i18n by resolving locale (5-layer precedence) and publishing
/// once to the global [`crate::locale`] `OnceLock`.
///
/// `force_lang` is the CLI `--lang` value (already clap-validated when present).
/// `config_dir_override` is `--config-dir` for persisted preference lookup.
pub fn initialize_language(
    force_lang: Option<&str>,
    config_dir_override: Option<&std::path::Path>,
) -> Result<()> {
    let resolution = crate::locale::resolve_language_detailed(force_lang, config_dir_override);
    tracing::debug!(
        target: "ssh_cli::i18n",
        language = resolution.language.bcp47(),
        source = resolution.source.as_str(),
        "locale resolved"
    );
    crate::locale::set_language(resolution.language);
    Ok(())
}

/// Returns the currently configured language.
#[must_use]
pub fn current_language() -> Language {
    crate::locale::current_language()
}

/// Returns the message string in the current global language.
///
/// Usa o estado global inicializado por `initialize_language`.
/// In tests, prefer `Message::text(language)` for determinism.
///
/// # Examples
///
/// ```
/// use ssh_cli::i18n::{t, initialize_language, Message};
///
/// initialize_language(Some("en"), None).unwrap();
/// let text = t(Message::VpsRegistryEmpty);
/// assert!(!text.is_empty());
/// ```
/// Takes [`Message`] by value: call sites construct ephemeral messages with
/// owned payloads; consuming them is intentional (not a needless copy).
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn t(msg: Message) -> String {
    msg.text(current_language())
}

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
            service: (*service).to_string(),
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
        SshCliError::VpsNotFound(name) => Message::VpsNotFound { name: name.clone() },
        SshCliError::VpsDuplicate(name) => Message::VpsDuplicate { name: name.clone() },
        // Untranslated variants (io, json, toml_*, broken_pipe, crypto, tls, …)
        // keep the English Display: they are machine-facing plumbing, not
        // operator prose.
        _ => return None,
    };
    Some(t(msg))
}

#[cfg(test)]
#[path = "i18n_tests.rs"]
mod tests;
