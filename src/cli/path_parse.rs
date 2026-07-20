// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: exec/scp target positional parsers extracted from cli/mod (SRP).
#![forbid(unsafe_code)]
//! Parses multi-host / multi-file positionals for exec and scp/sftp.

use anyhow::Result;
use std::path::PathBuf;

/// Split a `--hosts` LIST (`a,b,c`) into names.
pub(crate) fn parse_hosts_list(raw: &str) -> Vec<String> {
    crate::vps::dedupe_host_names(
        raw.split(',')
            .map(|s| s.trim().to_string())
            .collect(),
    )
}

/// Parses `exec`/`sudo-exec`/`su-exec` positionals into [`crate::vps::HostSelection`].
///
/// - `--all` / `--hosts`: one positional = command (batch wire).
/// - single host: two positionals = VPS + command (classic wire).
///
/// G-TYPE-09: names/tags are refined (`VpsName` / `HostTag`) at this boundary.
pub(crate) fn parse_exec_target(
    all: bool,
    hosts: Option<String>,
    tags: Option<String>,
    target: Vec<String>,
    active_vps: Option<String>,
) -> Result<(crate::vps::HostSelection, String), String> {
    use crate::domain::{try_tags, VpsName};
    use crate::vps::HostSelection;
    let modes = u8::from(all) + u8::from(hosts.is_some()) + u8::from(tags.is_some());
    if modes > 1 {
        return Err("--all, --hosts, and --tags are mutually exclusive".into());
    }
    if all {
        return match target.len() {
            // G-SEC-02: no unwrap after length gate — explicit Result extraction.
            1 => {
                let cmd = target
                    .into_iter()
                    .next()
                    .ok_or_else(|| "with --all pass only the shell command (one positional)".to_string())?;
                Ok((HostSelection::All, cmd))
            }
            _ => Err("with --all pass only the shell command (one positional)".into()),
        };
    }
    if let Some(h) = hosts {
        let names = parse_hosts_list(&h);
        if names.is_empty() {
            return Err("--hosts requires at least one host name".into());
        }
        let names = names
            .into_iter()
            .map(|n| VpsName::try_new(n).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        return match target.len() {
            1 => {
                let cmd = target
                    .into_iter()
                    .next()
                    .ok_or_else(|| "with --hosts pass only the shell command (one positional)".to_string())?;
                Ok((HostSelection::Named(names), cmd))
            }
            _ => Err("with --hosts pass only the shell command (one positional)".into()),
        };
    }
    if let Some(t) = tags {
        let tag_list = parse_hosts_list(&t);
        if tag_list.is_empty() {
            return Err("--tags requires at least one tag".into());
        }
        let tag_list = try_tags(tag_list).map_err(|e| e.to_string())?;
        return match target.len() {
            1 => {
                let cmd = target
                    .into_iter()
                    .next()
                    .ok_or_else(|| "with --tags pass only the shell command (one positional)".to_string())?;
                Ok((HostSelection::Tagged(tag_list), cmd))
            }
            _ => Err("with --tags pass only the shell command (one positional)".into()),
        };
    }
    match target.len() {
        2 => {
            let mut it = target.into_iter();
            let vps = it
                .next()
                .ok_or_else(|| "expected VPS and COMMAND".to_string())?;
            let cmd = it
                .next()
                .ok_or_else(|| "expected VPS and COMMAND".to_string())?;
            let vps = VpsName::try_new(vps).map_err(|e| e.to_string())?;
            Ok((HostSelection::Single(vps), cmd))
        }
        1 => {
            // G-AUD-04: single positional = COMMAND against active VPS (parity with health-check).
            let cmd = target
                .into_iter()
                .next()
                .ok_or_else(|| "expected COMMAND when using active VPS".to_string())?;
            let name = active_vps.ok_or_else(|| {
                "no active VPS; run `connect <name>` or pass `exec <VPS> <COMMAND>`".to_string()
            })?;
            let vps = VpsName::try_new(name).map_err(|e| e.to_string())?;
            Ok((HostSelection::Single(vps), cmd))
        }
        _ => Err(
            "expected VPS and COMMAND (or --all/--hosts COMMAND, or active VPS + COMMAND)".into(),
        ),
    }
}
/// Parsed SCP path plan (single-file, multi-file single-host, or multi-host multi-file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScpPathPlan {
    /// One source + one dest (classic single-host or multi-host one file).
    Single {
        selection: crate::vps::HostSelection,
        path_a: PathBuf,
        path_b: PathBuf,
    },
    /// Single host, N sources + one destination directory (G-PAR-37 / G-PAR-47 session reuse).
    MultiFile {
        vps: String,
        sources: Vec<PathBuf>,
        dest_dir: PathBuf,
    },
    /// Multi-host (`--all` / `--hosts`), N sources + one dest directory (G-PAR-48).
    /// Bound = one SSH session per host; files transfer serially on that session.
    MultiHostMultiFile {
        selection: crate::vps::HostSelection,
        sources: Vec<PathBuf>,
        dest_dir: PathBuf,
    },
}

/// Parses `scp upload|download` positionals into [`ScpPathPlan`].
///
/// - single host, 3 tokens: VPS + path_a + path_b
/// - single host, 4+ tokens: VPS + sources... + dest_dir (multi-file)
/// - `--all` / `--hosts`, 2 tokens: one file × multi-host
/// - `--all` / `--hosts`, 3+ tokens: sources... + dest_dir (multi-host multi-file, G-PAR-48)
pub(crate) fn parse_scp_target(
    all: bool,
    hosts: Option<String>,
    target: Vec<String>,
) -> Result<ScpPathPlan, String> {
    use crate::domain::VpsName;
    use crate::vps::HostSelection;
    if all && hosts.is_some() {
        return Err("--all conflicts with --hosts".into());
    }
    let multi_host = all || hosts.is_some();
    if multi_host {
        let selection = if all {
            HostSelection::All
        } else {
            let names = parse_hosts_list(&hosts.unwrap_or_default());
            if names.is_empty() {
                return Err("--hosts requires at least one host name".into());
            }
            let names = names
                .into_iter()
                .map(|n| VpsName::try_new(n).map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            HostSelection::Named(names)
        };
        return match target.len() {
            0 | 1 => Err(
                "with --all/--hosts pass paths: one file (LOCAL REMOTE) or multi-file (SRC... DEST_DIR)"
                    .into(),
            ),
            2 => {
                // G-SEC-02: Result extraction instead of unwrap after length gate.
                let mut it = target.into_iter();
                let a = PathBuf::from(
                    it.next()
                        .ok_or_else(|| "with --all/--hosts pass LOCAL REMOTE paths".to_string())?,
                );
                let b = PathBuf::from(
                    it.next()
                        .ok_or_else(|| "with --all/--hosts pass LOCAL REMOTE paths".to_string())?,
                );
                Ok(ScpPathPlan::Single {
                    selection,
                    path_a: a,
                    path_b: b,
                })
            }
            _ => {
                // G-PAR-48: SRC1 SRC2 ... DEST_DIR
                let mut paths: Vec<PathBuf> = target.into_iter().map(PathBuf::from).collect();
                let dest_dir = paths
                    .pop()
                    .ok_or_else(|| "multi-file scp requires DEST_DIR".to_string())?;
                if paths.is_empty() {
                    return Err("multi-file scp requires at least one source path".into());
                }
                Ok(ScpPathPlan::MultiHostMultiFile {
                    selection,
                    sources: paths,
                    dest_dir,
                })
            }
        };
    }
    match target.len() {
        0 | 1 => Err(
            "expected VPS and paths (upload: VPS LOCAL... REMOTE; download: VPS REMOTE... LOCAL)"
                .into(),
        ),
        2 => Err(
            "missing path (single-host needs VPS + two paths, or VPS + multiple sources + dest dir)"
                .into(),
        ),
        3 => {
            let mut it = target.into_iter();
            let vps = it
                .next()
                .ok_or_else(|| "expected VPS and two paths".to_string())?;
            let a = PathBuf::from(
                it.next()
                    .ok_or_else(|| "expected VPS and two paths".to_string())?,
            );
            let b = PathBuf::from(
                it.next()
                    .ok_or_else(|| "expected VPS and two paths".to_string())?,
            );
            let vps = VpsName::try_new(vps).map_err(|e| e.to_string())?;
            Ok(ScpPathPlan::Single {
                selection: HostSelection::Single(vps),
                path_a: a,
                path_b: b,
            })
        }
        _ => {
            // G-PAR-37: VPS src1 src2 ... dest_dir
            let mut it = target.into_iter();
            let vps = it
                .next()
                .ok_or_else(|| "expected VPS and multi-file paths".to_string())?;
            // G-TYPE-08/09: refine multi-file VPS name at the boundary (lookup key still String).
            let _ = VpsName::try_new(&vps).map_err(|e| e.to_string())?;
            let mut paths: Vec<PathBuf> = it.map(PathBuf::from).collect();
            let dest_dir = paths
                .pop()
                .ok_or_else(|| "multi-file scp requires DEST_DIR".to_string())?;
            if paths.is_empty() {
                return Err("multi-file scp requires at least one source path".into());
            }
            Ok(ScpPathPlan::MultiFile {
                vps,
                sources: paths,
                dest_dir,
            })
        }
    }
}
