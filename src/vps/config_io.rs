// SPDX-License-Identifier: MIT OR Apache-2.0
// G-UNSAFE-10: config path/load/save/permissions extracted from monólito vps/mod (SRP).
#![forbid(unsafe_code)]
//! Atomic TOML config I/O under XDG (load/save/flock/0o600).

use super::model::{self, VpsRecord};
use crate::errors::{SshCliError, SshCliResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Full configuration file.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    /// File schema version.
    #[serde(default)]
    pub schema_version: u32,
    /// Host map keyed by VPS name.
    #[serde(default)]
    pub hosts: BTreeMap<String, VpsRecord>,
}

/// Resolves the config file path from an optional override.
///
/// Takes `Option<&Path>` (not `Option<PathBuf>`) so callers can share one
/// override without cloning — ownership of the path stays with the caller.
pub fn resolve_config_path(override_path: Option<&Path>) -> SshCliResult<PathBuf> {
    match override_path {
        Some(p) => {
            if p.is_dir() {
                return Ok(p.join(crate::constants::CONFIG_FILE_NAME));
            }
            if p.extension().and_then(|e| e.to_str()) == Some("toml") {
                return Ok(p.to_path_buf());
            }
            Ok(p.join(crate::constants::CONFIG_FILE_NAME))
        }
        None => default_config_path(),
    }
}

/// Returns the config file path under XDG (`--config-dir` wins at call sites).
///
/// G-AUD-12: no `SSH_CLI_HOME` env store — use `--config-dir` for overrides.
pub fn default_config_path() -> SshCliResult<PathBuf> {
    Ok(crate::paths::xdg_config_dir()?.join(crate::constants::CONFIG_FILE_NAME))
}

/// Winning configuration layer (doctor).
#[derive(Debug, Clone)]
pub struct ConfigLayer {
    /// Layer name.
    pub name: &'static str,
    /// Resolved path.
    pub path: PathBuf,
}

/// Resolves and describes the winning config layer.
pub fn winning_layer(override_path: Option<&Path>) -> SshCliResult<ConfigLayer> {
    if override_path.is_some() {
        return Ok(ConfigLayer {
            name: "--config-dir",
            path: resolve_config_path(override_path)?,
        });
    }
    Ok(ConfigLayer {
        name: "XDG ProjectDirs",
        path: default_config_path()?,
    })
}

/// Loads the configuration file (returns empty if missing).
pub fn load(path: &Path) -> SshCliResult<ConfigFile> {
    if !path.exists() {
        return Ok(ConfigFile {
            schema_version: model::CURRENT_SCHEMA_VERSION,
            hosts: BTreeMap::new(),
        });
    }
    let content = crate::paths::read_text_capped(path, crate::paths::MAX_CONFIG_TOML_BYTES)?;
    // G-SERDE-02/08: parse → path-aware serde → structure validate (no auth required).
    let mut file: ConfigFile = crate::validation::from_toml_str(&content)?;
    // Sequential: in-memory schema normalize per record (CPU µs; no SSH).
    for (name, reg) in file.hosts.iter_mut() {
        reg.normalize_schema();
        reg.validate_structure().map_err(|e| {
            crate::errors::SshCliError::InvalidArgument(format!(
                "invalid host {name} in config: {e}"
            ))
        })?;
    }
    if file.schema_version < model::CURRENT_SCHEMA_VERSION {
        file.schema_version = model::CURRENT_SCHEMA_VERSION;
    }
    Ok(file)
}

/// Writes bytes to `path` atomically (tempfile + fsync + rename + 0o600).
///
/// Used by `save` and `export -o` (atomwrite rule).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> SshCliResult<()> {
    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }
    let parent_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut tmp = tempfile::NamedTempFile::new_in(&parent_dir)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_data()?;
    tmp.persist(path).map_err(|e| SshCliError::Io(e.error))?;
    apply_permissions_600(path)?;
    #[cfg(unix)]
    {
        if let Ok(dir) = std::fs::File::open(&parent_dir) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}

/// Saves the configuration file atomically with flock and 0o600.
///
/// # Errors
/// Returns an error if serialization, atomic write, or permission hardening fails.
pub fn save(path: &Path, file: &ConfigFile) -> SshCliResult<()> {
    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }
    let text = toml::to_string_pretty(file)
        .map_err(|e| SshCliError::Config(format!("failed to serialize TOML: {e}")))?;

    // Sibling lock file to serialize concurrent mutations (N one-shots).
    let lock_path = path.with_extension("toml.lock");
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)?;
    // GAP-SSH-PERM-001: lock with 0o600 (not umask 0644).
    apply_permissions_600(&lock_path)?;
    fs2::FileExt::lock_exclusive(&lock_file)?;

    write_atomic(path, text.as_bytes())?;

    let _ = fs2::FileExt::unlock(&lock_file);
    Ok(())
}

/// Expands leading `~` in a path (user home).
pub(crate) fn expand_tilde(path: &str) -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home {
            return home.join(rest);
        }
    }
    if path == "~" {
        if let Some(home) = home {
            return home;
        }
    }
    PathBuf::from(path)
}

/// Validates that `key_path` points to an existing local file (VAL-003)
/// and, with `ssh-real`, that the content is a parseable OpenSSH key (VAL-004).
pub(crate) fn validate_key_path_exists(key_path: &str) -> Result<(), SshCliError> {
    validate_key_path_exists_with_passphrase(key_path, None)
}

/// Like [`validate_key_path_exists`], with optional passphrase from add/edit.
pub(crate) fn validate_key_path_exists_with_passphrase(
    key_path: &str,
    passphrase: Option<&str>,
) -> Result<(), SshCliError> {
    let p = expand_tilde(key_path);
    if !p.is_file() {
        return Err(SshCliError::FileNotFound(format!(
            "private key not found: {}",
            p.display()
        )));
    }
    #[cfg(feature = "ssh-real")]
    {
        match russh::keys::load_secret_key(&p, passphrase) {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string().to_lowercase();
                // Valid encrypted key without passphrase on the write-path.
                if msg.contains("password")
                    || msg.contains("passphrase")
                    || msg.contains("encrypted")
                    || msg.contains("decrypt")
                {
                    return Ok(());
                }
                Err(SshCliError::InvalidArgument(format!(
                    "invalid OpenSSH private key at {}: {e}",
                    p.display()
                )))
            }
        }
    }
    #[cfg(not(feature = "ssh-real"))]
    {
        let _ = passphrase;
        Ok(())
    }
}

fn apply_permissions_600(path: &Path) -> SshCliResult<()> {
    crate::fs_perm::set_secret_file_mode(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::{ExposeSecret, SecretString};
    use tempfile::TempDir;

    fn reg_min() -> VpsRecord {
        VpsRecord::test_new(
            "srv",
            "host.example.com",
            2222,
            "admin",
            SecretString::from("pass".to_string()),
            None,
            None,
            Some(60_000),
            Some(1_000),
            Some(50_000),
            None,
            None,
            false,
        )
    }

    #[test]
    fn empty_file_serializes_with_schema() {
        let cfg_file = ConfigFile {
            schema_version: model::CURRENT_SCHEMA_VERSION,
            hosts: BTreeMap::new(),
        };
        let text = toml::to_string(&cfg_file).unwrap();
        assert!(text.contains("schema_version = 3"));
    }

    #[test]
    #[serial_test::serial]
    fn atomic_save_roundtrip() {
        let tmp = TempDir::new().unwrap();
        crate::secrets::set_config_dir(Some(tmp.path().to_path_buf()));
        crate::secrets::set_runtime_flags(true, None, false);
        let path = tmp.path().join("config.toml");
        let mut cfg_file = ConfigFile {
            schema_version: 2,
            hosts: BTreeMap::new(),
        };
        cfg_file.hosts.insert("a".into(), reg_min());
        save(&path, &cfg_file).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.hosts.len(), 1);
        assert_eq!(loaded.hosts["a"].password.expose_secret(), "pass");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
            let lock = path.with_extension("toml.lock");
            if lock.exists() {
                let lm = std::fs::metadata(&lock).unwrap().permissions().mode() & 0o777;
                assert_eq!(lm, 0o600);
            }
        }
        crate::secrets::set_runtime_flags(false, None, false);
        crate::secrets::set_config_dir(None);
    }

    #[test]
    fn resolve_config_path_with_dir_override() {
        let result = resolve_config_path(Some(Path::new("/tmp/test-dir")));
        assert_eq!(
            result.unwrap(),
            PathBuf::from("/tmp/test-dir/config.toml")
        );
    }

    #[test]
    fn resolve_config_path_toml_file_override_keeps_path() {
        let p = Path::new("/tmp/custom-hosts.toml");
        let result = resolve_config_path(Some(p)).unwrap();
        assert_eq!(result, PathBuf::from("/tmp/custom-hosts.toml"));
    }

    #[test]
    fn config_override_shared_without_clone() {
        let owned = PathBuf::from("/tmp/share-me");
        let a = resolve_config_path(Some(owned.as_path())).unwrap();
        let b = winning_layer(Some(owned.as_path())).unwrap();
        assert_eq!(a, PathBuf::from("/tmp/share-me/config.toml"));
        assert_eq!(b.name, "--config-dir");
        assert_eq!(b.path, a);
        assert_eq!(owned, PathBuf::from("/tmp/share-me"));
    }
}
