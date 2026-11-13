// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Loads `config.toml` and keeps the registry keyed by each record's own name.
//!
//! # Why the map is re-keyed
//!
//! A hand-edited or legacy file may key a table by a spelling that differs from the
//! record's `name` (a decomposed Unicode key, a key carrying a control byte). Lookups
//! (`show`, `edit`, `rename`, `--hosts`) use the record name, which is already
//! normalized, so a stale key made a host listed by `vps list` impossible to address.
//! Re-keying on load makes the list and every lookup agree, and the next save writes
//! the clean key back.

use std::collections::BTreeMap;
use std::path::Path;

use super::config_io::ConfigFile;
use super::model::{self, VpsRecord};
use crate::errors::{SshCliError, SshCliResult};
use crate::output::sanitize::escape_untrusted;

/// Loads the configuration file (returns empty if missing).
///
/// # Errors
/// I/O or parse failure, an invalid record ([`SshCliError::InvalidArgument`]), or two
/// tables whose record names are the same host ([`SshCliError::Config`], exit 65).
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
            SshCliError::InvalidArgument(format!(
                "invalid host {} in config: {e}",
                escape_untrusted(name)
            ))
        })?;
    }
    file.hosts = rekey_by_record_name(file.hosts)?;
    if file.schema_version < model::CURRENT_SCHEMA_VERSION {
        file.schema_version = model::CURRENT_SCHEMA_VERSION;
    }
    Ok(file)
}

/// Re-keys `hosts` by `VpsRecord::name`.
///
/// # Errors
/// [`SshCliError::Config`] when two tables resolve to the same name: dropping one
/// would silently lose a host and its credentials. Names are escaped in the message.
fn rekey_by_record_name(
    hosts: BTreeMap<String, VpsRecord>,
) -> SshCliResult<BTreeMap<String, VpsRecord>> {
    let mut keyed: BTreeMap<String, (String, VpsRecord)> = BTreeMap::new();
    for (old_key, record) in hosts {
        let new_key = record.name.as_str().to_owned();
        if let Some((first_key, _)) = keyed.get(&new_key) {
            return Err(SshCliError::Config(format!(
                "config has two hosts named {}: tables {} and {}; rename or remove one",
                escape_untrusted(&new_key),
                escape_untrusted(first_key),
                escape_untrusted(&old_key)
            )));
        }
        keyed.insert(new_key, (old_key, record));
    }
    Ok(keyed.into_iter().map(|(k, (_, r))| (k, r)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(toml_text: &str) -> BTreeMap<String, VpsRecord> {
        toml::from_str::<ConfigFile>(toml_text)
            .expect("parse config")
            .hosts
    }

    fn table(key: &str, name: &str) -> String {
        format!(
            "[hosts.\"{key}\"]\nname = \"{name}\"\nhost = \"10.0.0.1\"\nport = 22\nusername = \"u\"\npassword = \"x\"\ntimeout_ms = 60000\nschema_version = 3\n\n"
        )
    }

    #[test]
    fn decomposed_key_is_replaced_by_the_normalized_record_name() {
        let nfd = "cafe\u{301}";
        let hosts = parse(&table(nfd, nfd));
        let rekeyed = rekey_by_record_name(hosts).expect("rekey");
        let keys: Vec<&String> = rekeyed.keys().collect();
        assert_eq!(keys, ["caf\u{e9}"]);
    }

    #[test]
    fn key_with_a_control_byte_is_replaced_by_the_record_name() {
        let hosts = parse(&table("a\\u001bb", "ab"));
        let rekeyed = rekey_by_record_name(hosts).expect("rekey");
        assert_eq!(rekeyed.keys().collect::<Vec<_>>(), ["ab"]);
    }

    #[test]
    fn two_tables_with_the_same_normalized_name_are_an_error() {
        let text = format!(
            "{}{}",
            table("caf\u{e9}", "caf\u{e9}"),
            table("cafe\u{301}", "cafe\u{301}")
        );
        let err = rekey_by_record_name(parse(&text)).expect_err("collision");
        assert!(matches!(err, SshCliError::Config(_)));
        assert!(err.to_string().contains("two hosts named"));
    }
}
