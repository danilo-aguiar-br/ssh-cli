// SPDX-License-Identifier: MIT OR Apache-2.0
// G-CLOSE-05 / G-COMP-01: host selection extracted from `vps/mod` (SRP + componentization).
#![forbid(unsafe_code)]
//! Multi-host selection resolution for bounded fan-out (G-PAR-27 / G-PAR-31).
//!
//! Pure local map lookup — parallelism starts only after callers pass jobs to
//! [`crate::concurrency::map_bounded`].

use super::ConfigFile;
use crate::domain::{HostTag, VpsName};
use crate::errors::{SshCliError, SshCliResult};
use crate::vps::model::VpsRecord;

/// How multi-host SSH ops select target hosts (G-PAR-27 / G-PAR-31 / G-TYPE-09).
///
/// Workload: selection is pure local map lookup (tiny). Parallelism starts only
/// after [`resolve_host_jobs`] when callers fan out via [`crate::concurrency::map_bounded`].
///
/// Names/tags are refined at the CLI boundary (`VpsName` / `HostTag`); map keys stay
/// owned `String` for wire/storage compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostSelection {
    /// One host by name (positional). Classic single-host JSON wire.
    Single(VpsName),
    /// Every registered host. Batch JSON wire.
    All,
    /// Explicit subset (`--hosts`). Batch JSON wire even if `len == 1` (G-PAR-36).
    Named(Vec<VpsName>),
    /// Hosts matching **any** of the given tags (OR). Batch JSON wire (G-O2).
    Tagged(Vec<HostTag>),
}

impl HostSelection {
    /// True when output must use the multi-host batch envelope.
    #[must_use]
    pub fn is_batch(&self) -> bool {
        matches!(self, Self::All | Self::Named(_) | Self::Tagged(_))
    }
}

/// Deduplicate host names preserving first-seen order (trim empty segments).
#[must_use]
pub fn dedupe_host_names(names: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(names.len());
    let mut seen = std::collections::HashSet::with_capacity(names.len());
    for n in names {
        let t = n.trim().to_string();
        if t.is_empty() {
            continue;
        }
        if seen.insert(t.clone()) {
            out.push(t);
        }
    }
    out
}

/// Resolve a [`HostSelection`] into owned jobs for bounded multi-host fan-out.
///
/// # Errors
/// - Empty registry for `--all` / `--hosts`
/// - Unknown names in `--hosts` (fail-closed)
/// - Missing single host (`VpsNotFound`)
pub fn resolve_host_jobs(
    selection: &HostSelection,
    file: &ConfigFile,
) -> SshCliResult<Vec<(String, VpsRecord)>> {
    match selection {
        HostSelection::All => {
            if file.hosts.is_empty() {
                return Err(SshCliError::InvalidArgument(
                    "no hosts registered for --all".into(),
                ));
            }
            Ok(file
                .hosts
                .iter()
                .map(|(n, r)| (n.clone(), r.clone()))
                .collect())
        }
        HostSelection::Named(names) => {
            if names.is_empty() {
                return Err(SshCliError::InvalidArgument(
                    "--hosts requires at least one host name".into(),
                ));
            }
            if file.hosts.is_empty() {
                return Err(SshCliError::InvalidArgument(
                    "no hosts registered for --hosts".into(),
                ));
            }
            let mut jobs = Vec::with_capacity(names.len());
            let mut missing = Vec::new();
            for n in names {
                match file.hosts.get(n.as_str()) {
                    Some(r) => jobs.push((n.as_str().to_owned(), r.clone())),
                    None => missing.push(n.as_str().to_owned()),
                }
            }
            if !missing.is_empty() {
                return Err(SshCliError::InvalidArgument(format!(
                    "unknown host(s) for --hosts: {}",
                    missing.join(", ")
                )));
            }
            Ok(jobs)
        }
        HostSelection::Tagged(tags) => {
            if tags.is_empty() {
                return Err(SshCliError::InvalidArgument(
                    "--tags requires at least one tag".into(),
                ));
            }
            if file.hosts.is_empty() {
                return Err(SshCliError::InvalidArgument(
                    "no hosts registered for --tags".into(),
                ));
            }
            // G-TYPE-09: tags already refined as HostTag — no second try_tags pass.
            let jobs: Vec<_> = file
                .hosts
                .iter()
                .filter(|(_, r)| r.has_any_tag(tags))
                .map(|(n, r)| (n.clone(), r.clone()))
                .collect();
            if jobs.is_empty() {
                return Err(SshCliError::InvalidArgument(format!(
                    "no hosts match tag(s): {}",
                    tags.iter()
                        .map(HostTag::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
            Ok(jobs)
        }
        HostSelection::Single(name) => {
            let key = name.as_str();
            let r = file
                .hosts
                .get(key)
                .ok_or_else(|| SshCliError::VpsNotFound(key.to_owned()))?
                .clone();
            Ok(vec![(key.to_owned(), r)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vps::model::VpsRecord;
    use secrecy::SecretString;
    use std::collections::BTreeMap;

    fn file_with(names: &[&str]) -> ConfigFile {
        let mut hosts = BTreeMap::new();
        for n in names {
            hosts.insert(
                (*n).to_string(),
                VpsRecord::test_new(
                    *n,
                    "h",
                    22,
                    "u",
                    SecretString::from(String::new()),
                    Some("/k"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    false,
                ),
            );
        }
        ConfigFile {
            schema_version: 3,
            hosts,
        }
    }

    fn vps(name: &str) -> VpsName {
        VpsName::try_new(name).expect("valid test VpsName")
    }

    #[test]
    fn is_batch_named_and_all() {
        assert!(!HostSelection::Single(vps("a")).is_batch());
        assert!(HostSelection::All.is_batch());
        assert!(HostSelection::Named(vec![vps("a")]).is_batch());
    }

    #[test]
    fn dedupe_preserves_order() {
        assert_eq!(
            dedupe_host_names(vec!["b".into(), "a".into(), "b".into(), " ".into()]),
            vec!["b".to_string(), "a".to_string()]
        );
    }

    #[test]
    fn resolve_named_unknown_fails() {
        let f = file_with(&["a"]);
        let err = resolve_host_jobs(&HostSelection::Named(vec![vps("x")]), &f).unwrap_err();
        assert!(matches!(err, SshCliError::InvalidArgument(_)));
    }
}
