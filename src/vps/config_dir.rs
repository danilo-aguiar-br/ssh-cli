// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! The one rule that turns a `--config-dir` value into the configuration DIRECTORY.
//!
//! `--config-dir` accepts either a directory or the registry file itself
//! (`sub/config.toml`, `sub/c.toml`). The registry, `secrets.key`, `known_hosts`,
//! the `active` marker, the `lang` file and the TLS tree all live in the directory,
//! so every module must agree on it. Resolving it in one place stops a `*.toml` path
//! from being used as a directory by whichever module forgot the file case.

use std::path::{Path, PathBuf};

use crate::constants::CONFIG_FILE_EXTENSION;

/// Directory that holds the configuration files for a `--config-dir` value.
///
/// - An existing directory is returned as is, whatever its name.
/// - A path with the `.toml` extension that is not an existing directory names the
///   registry file, so its parent is returned (`.` for a bare file name).
/// - Anything else is taken as the directory to create.
///
/// This matches [`super::resolve_config_path`], which joins `config.toml` to the
/// directory forms and keeps the `.toml` form as the file.
///
/// # Examples
/// ```
/// use std::path::Path;
/// use ssh_cli::vps::config_dir_for_override;
///
/// assert_eq!(config_dir_for_override(Path::new("sub/c.toml")), Path::new("sub"));
/// assert_eq!(config_dir_for_override(Path::new("sub/dir")), Path::new("sub/dir"));
/// assert_eq!(config_dir_for_override(Path::new("c.toml")), Path::new("."));
/// ```
#[must_use]
pub fn config_dir_for_override(path: &Path) -> PathBuf {
    let names_the_file = path.extension().and_then(|e| e.to_str()) == Some(CONFIG_FILE_EXTENSION);
    if names_the_file && !path.is_dir() {
        return path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_path_resolves_to_its_parent() {
        assert_eq!(
            config_dir_for_override(Path::new("a/b/c.toml")),
            Path::new("a/b")
        );
        assert_eq!(
            config_dir_for_override(Path::new("config.toml")),
            Path::new(".")
        );
    }

    #[test]
    fn plain_path_is_the_directory() {
        assert_eq!(config_dir_for_override(Path::new("a/b")), Path::new("a/b"));
    }

    #[test]
    fn existing_directory_named_like_a_toml_stays_a_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().join("odd.toml");
        std::fs::create_dir(&dir).unwrap();
        assert_eq!(config_dir_for_override(&dir), dir);
    }
}
