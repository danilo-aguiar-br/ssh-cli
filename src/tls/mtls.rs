// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! mTLS client identity store under XDG `tls/mtls/<name>/`.

use std::path::{Path, PathBuf};

use super::paths::{
    cert_pem_path, ensure_dir, key_pem_path, mtls_identity_dir, write_secret_file,
};
use super::pem::{load_cert_chain, load_private_key};
use crate::errors::{SshCliError, SshCliResult};

/// Named mTLS client identity on disk.
#[derive(Debug, Clone)]
pub struct MtlsIdentity {
    /// Logical name (XDG leaf).
    pub name: String,
    /// Absolute path to certificate chain PEM.
    pub cert_path: PathBuf,
    /// Absolute path to private key PEM.
    pub key_path: PathBuf,
}

/// Imports PEM cert+key into XDG as identity `name` (overwrites).
pub fn mtls_import(
    config_override: Option<&Path>,
    name: &str,
    cert_src: &Path,
    key_src: &Path,
) -> SshCliResult<MtlsIdentity> {
    // Validate PEMs before writing.
    let _ = load_cert_chain(cert_src)?;
    let _ = load_private_key(key_src)?;

    let dir = mtls_identity_dir(config_override, name)?;
    ensure_dir(&dir)?;
    let cert_path = cert_pem_path(&dir);
    let key_path = key_pem_path(&dir);

    let cert_bytes = std::fs::read(cert_src)
        .map_err(|e| SshCliError::tls_msg(format!("read {}: {e}", cert_src.display())))?;
    let key_bytes = std::fs::read(key_src)
        .map_err(|e| SshCliError::tls_msg(format!("read {}: {e}", key_src.display())))?;
    write_secret_file(&cert_path, &cert_bytes)?;
    write_secret_file(&key_path, &key_bytes)?;

    Ok(MtlsIdentity {
        name: name.to_owned(),
        cert_path,
        key_path,
    })
}

/// Lists imported mTLS identity names.
pub fn mtls_list(config_override: Option<&Path>) -> SshCliResult<Vec<String>> {
    let root = super::paths::resolve_tls_root(config_override)?
        .join(crate::constants::TLS_MTLS_DIR_NAME);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&root)
        .map_err(|e| SshCliError::tls_msg(format!("list mtls: {e}")))?
    {
        let entry = entry.map_err(|e| SshCliError::tls_msg(format!("list mtls entry: {e}")))?;
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            if let Some(n) = entry.file_name().to_str() {
                let cert = cert_pem_path(&entry.path());
                let key = key_pem_path(&entry.path());
                if cert.is_file() && key.is_file() {
                    names.push(n.to_owned());
                }
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Shows paths for one identity.
pub fn mtls_show(config_override: Option<&Path>, name: &str) -> SshCliResult<MtlsIdentity> {
    let dir = mtls_identity_dir(config_override, name)?;
    let cert_path = cert_pem_path(&dir);
    let key_path = key_pem_path(&dir);
    if !cert_path.is_file() || !key_path.is_file() {
        return Err(SshCliError::FileNotFound(format!(
            "mTLS identity '{name}' not found under {}",
            dir.display()
        )));
    }
    // Parse to ensure integrity.
    let _ = load_cert_chain(&cert_path)?;
    let _ = load_private_key(&key_path)?;
    Ok(MtlsIdentity {
        name: name.to_owned(),
        cert_path,
        key_path,
    })
}

/// Removes an identity directory.
pub fn mtls_remove(config_override: Option<&Path>, name: &str) -> SshCliResult<()> {
    let dir = mtls_identity_dir(config_override, name)?;
    if !dir.exists() {
        return Err(SshCliError::FileNotFound(format!(
            "mTLS identity '{name}' not found"
        )));
    }
    std::fs::remove_dir_all(&dir)
        .map_err(|e| SshCliError::tls_msg(format!("remove mTLS '{name}': {e}")))?;
    Ok(())
}

/// Resolves mTLS paths: either explicit paths or an XDG identity name.
pub fn resolve_mtls_paths(
    config_override: Option<&Path>,
    identity: Option<&str>,
    cert: Option<&Path>,
    key: Option<&Path>,
) -> SshCliResult<(Option<PathBuf>, Option<PathBuf>)> {
    if let Some(id) = identity {
        let show = mtls_show(config_override, id)?;
        return Ok((Some(show.cert_path), Some(show.key_path)));
    }
    Ok((cert.map(Path::to_path_buf), key.map(Path::to_path_buf)))
}
