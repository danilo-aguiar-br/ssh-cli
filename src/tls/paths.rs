// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! XDG layout for TLS material (no product env for cert storage).

use std::path::{Path, PathBuf};

use crate::constants::{
    TLS_ACME_ACCOUNT_FILE_NAME, TLS_ACME_DIR_NAME, TLS_ACME_ORDER_FILE_NAME, TLS_CERT_FILE_NAME,
    TLS_DIR_NAME, TLS_KEY_FILE_NAME, TLS_MTLS_DIR_NAME,
};
use crate::errors::{SshCliError, SshCliResult};
use crate::paths::{validate_and_normalize, xdg_config_dir};

/// Resolves the TLS root directory under the active config dir.
///
/// Priority: explicit `config_override` parent → XDG config dir for the app.
pub fn resolve_tls_root(config_override: Option<&Path>) -> SshCliResult<PathBuf> {
    let base = if let Some(dir) = config_override {
        dir.to_path_buf()
    } else {
        xdg_config_dir()?
    };
    Ok(base.join(TLS_DIR_NAME))
}

/// Same as [`resolve_tls_root`] (alias for call-site clarity).
pub fn tls_root_dir(config_override: Option<&Path>) -> SshCliResult<PathBuf> {
    resolve_tls_root(config_override)
}

/// `…/tls/mtls/<name>/`
pub fn mtls_identity_dir(config_override: Option<&Path>, name: &str) -> SshCliResult<PathBuf> {
    let safe = validate_and_normalize(name).map_err(|e| {
        SshCliError::InvalidArgument(format!("invalid mTLS identity name: {e}"))
    })?;
    Ok(resolve_tls_root(config_override)?
        .join(TLS_MTLS_DIR_NAME)
        .join(safe.as_str()))
}

/// `…/tls/acme/account.json`
pub fn acme_account_path(config_override: Option<&Path>) -> SshCliResult<PathBuf> {
    Ok(resolve_tls_root(config_override)?
        .join(TLS_ACME_DIR_NAME)
        .join(TLS_ACME_ACCOUNT_FILE_NAME))
}

/// `…/tls/acme/<domain>/` — domain leaf is NFC-normalized validated name.
pub fn acme_domain_dir(config_override: Option<&Path>, domain: &str) -> SshCliResult<PathBuf> {
    // Domains may contain dots — use a conservative sanitizer (no traversal).
    let leaf = sanitize_domain_leaf(domain)?;
    Ok(resolve_tls_root(config_override)?
        .join(TLS_ACME_DIR_NAME)
        .join(leaf))
}

/// Certificate PEM path under a domain or identity directory.
#[must_use]
pub fn cert_pem_path(dir: &Path) -> PathBuf {
    dir.join(TLS_CERT_FILE_NAME)
}

/// Private key PEM path under a domain or identity directory.
#[must_use]
pub fn key_pem_path(dir: &Path) -> PathBuf {
    dir.join(TLS_KEY_FILE_NAME)
}

/// Pending ACME order JSON path.
#[must_use]
pub fn order_json_path(dir: &Path) -> PathBuf {
    dir.join(TLS_ACME_ORDER_FILE_NAME)
}

/// Sanitizes a DNS name for use as a single path component.
///
/// Allows letters, digits, `.`, `-`, `_`. Rejects empty, `..`, separators.
fn sanitize_domain_leaf(domain: &str) -> SshCliResult<String> {
    let d = domain.trim().to_ascii_lowercase();
    if d.is_empty() {
        return Err(SshCliError::InvalidArgument("domain cannot be empty".into()));
    }
    if d.contains("..") || d.contains('/') || d.contains('\\') {
        return Err(SshCliError::InvalidArgument(format!(
            "invalid domain path leaf: {domain}"
        )));
    }
    if !d
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        return Err(SshCliError::InvalidArgument(format!(
            "domain contains forbidden characters: {domain}"
        )));
    }
    Ok(d)
}

/// Ensures directory exists with restrictive permissions on Unix.
pub(crate) fn ensure_dir(path: &Path) -> SshCliResult<()> {
    std::fs::create_dir_all(path).map_err(|e| {
        SshCliError::tls_msg(format!("create TLS dir {}: {e}", path.display()))
    })?;
    // Best-effort secret dir mode (matches prior ignore-on-error chmod).
    let _ = crate::fs_perm::set_secret_dir_mode(path);
    Ok(())
}

/// Writes bytes atomically-ish (tmp + rename) with 0o600 on Unix.
pub(crate) fn write_secret_file(path: &Path, data: &[u8]) -> SshCliResult<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data)
        .map_err(|e| SshCliError::tls_msg(format!("write {}: {e}", tmp.display())))?;
    // Best-effort secret file mode before/after rename.
    let _ = crate::fs_perm::set_secret_file_mode(&tmp);
    std::fs::rename(&tmp, path)
        .map_err(|e| SshCliError::tls_msg(format!("rename {}: {e}", path.display())))?;
    let _ = crate::fs_perm::set_secret_file_mode(path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn sanitize_domain_ok() {
        assert_eq!(sanitize_domain_leaf("Example.COM").unwrap(), "example.com");
    }

    #[test]
    fn sanitize_domain_rejects_traversal() {
        assert!(sanitize_domain_leaf("../etc").is_err());
        assert!(sanitize_domain_leaf("a/b").is_err());
    }

    #[test]
    fn mtls_dir_layout() {
        let t = TempDir::new().unwrap();
        let d = mtls_identity_dir(Some(t.path()), "agent-1").unwrap();
        assert!(d.ends_with("tls/mtls/agent-1") || d.ends_with(r"tls\mtls\agent-1"));
    }
}
