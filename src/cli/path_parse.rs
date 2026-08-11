// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: exec/scp target positional parsers extracted from cli/mod (SRP).
#![forbid(unsafe_code)]
//! Parses multi-host / multi-file positionals for exec and scp/sftp.

use anyhow::Result;
use std::path::PathBuf;

/// Split a `--hosts` LIST (`a,b,c`) into names.
pub(crate) fn parse_hosts_list(raw: &str) -> Vec<String> {
    crate::vps::dedupe_host_names(raw.split(',').map(|s| s.trim().to_string()).collect())
}

/// Why [`parse_exec_target`] refused the argv it was handed.
///
/// GAP-SSH-EXEC-ARGC-001 remediation: the dispatcher used to recover the error kind
/// by testing `starts_with("no active VPS")` on the message string, so the wording of
/// a human-facing sentence silently decided which exit code the process returned.
/// Renaming a message would have changed an exit code. The kind is now a type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExecTargetError {
    /// `--use-active` was requested but no marker is set.
    NoActiveVps,
    /// The argv shape is not a valid designation.
    Invalid(String),
}

impl std::fmt::Display for ExecTargetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoActiveVps => {
                f.write_str("no active VPS; run `connect <name>` or pass `exec <VPS> <COMMAND>`")
            }
            Self::Invalid(s) => f.write_str(s),
        }
    }
}

/// A fully designated execution target: which hosts, which command, and how the
/// hosts were chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecTargetPlan {
    /// Hosts to run against.
    pub(crate) selection: crate::vps::HostSelection,
    /// Primary shell command.
    pub(crate) command: String,
    /// Provenance of the host designation, echoed into every envelope.
    pub(crate) source: crate::json_wire::TargetSource,
}

/// The one usage sentence, so every rejection path teaches the same three forms.
const EXEC_USAGE: &str = concat!(
    "designate the target explicitly: `<VPS> <COMMAND>`, ",
    "or a selector with one positional (`--all`/`--hosts`/`--tags` `<COMMAND>`), ",
    "or the active marker deliberately (`--use-active <COMMAND>`)"
);

/// Rejection message for a selector that received the wrong positional count.
fn selector_arity_error(flag: &str) -> ExecTargetError {
    ExecTargetError::Invalid(format!(
        "with {flag} pass only the shell command (exactly one positional); {EXEC_USAGE}"
    ))
}

/// Extracts the sole positional after the arity gate already proved there is one.
///
/// G-SEC-02: no `unwrap` after a length check — the extraction stays a `Result`.
fn only_command(target: Vec<String>, err: ExecTargetError) -> Result<String, ExecTargetError> {
    target.into_iter().next().ok_or(err)
}

/// Parses `exec`/`sudo-exec`/`su-exec` positionals into an [`ExecTargetPlan`].
///
/// # Why arity no longer carries meaning (GAP-SSH-EXEC-ARGC-001)
///
/// This function used to read the *number* of positionals as the mode selector: two
/// meant `VPS COMMAND`, one meant `COMMAND` against whatever host the on-disk active
/// marker named. `--step` is [`clap::ArgAction::Append`] and therefore never counts
/// as a positional, so `exec HOST --step CMD` collapsed to the one-positional branch
/// and the *host name* was executed as a remote binary on a *different* machine. The
/// step-zero `command not found` was survivable; the remaining `--step` commands
/// landing on the wrong host was not, and the aggregate exit stayed 0.
///
/// `clap`'s own documentation for `Arg::num_args` warns about this shape: a
/// positional with a range keeps consuming values until "it finds another flag or
/// option". `--step` is that flag.
///
/// The mode is now declared by a flag, never inferred from a count:
///
/// - `--all` / `--hosts` / `--tags`: exactly one positional, the command.
/// - `--use-active`: exactly one positional, the command; host from the marker.
/// - neither: exactly two positionals, `VPS COMMAND`.
///
/// A single positional with no flag is a usage error. It used to be the dangerous
/// shortcut, and there is no argv that reaches a host the caller did not name.
///
/// G-TYPE-09: names/tags are refined (`VpsName` / `HostTag`) at this boundary.
pub(crate) fn parse_exec_target(
    all: bool,
    hosts: Option<String>,
    tags: Option<String>,
    use_active: bool,
    target: Vec<String>,
    active_vps: Option<String>,
) -> Result<ExecTargetPlan, ExecTargetError> {
    use crate::domain::{try_tags, VpsName};
    use crate::json_wire::TargetSource;
    use crate::vps::HostSelection;

    let invalid = |s: String| ExecTargetError::Invalid(s);
    let modes = u8::from(all) + u8::from(hosts.is_some()) + u8::from(tags.is_some());
    if modes > 1 {
        return Err(invalid(
            "--all, --hosts, and --tags are mutually exclusive".into(),
        ));
    }
    if use_active && modes > 0 {
        return Err(invalid(
            "--use-active conflicts with --all, --hosts and --tags".into(),
        ));
    }

    if all {
        let err = selector_arity_error("--all");
        if target.len() != 1 {
            return Err(err);
        }
        return Ok(ExecTargetPlan {
            selection: HostSelection::All,
            command: only_command(target, err)?,
            source: TargetSource::Selector,
        });
    }

    if let Some(h) = hosts {
        let names = parse_hosts_list(&h);
        if names.is_empty() {
            return Err(invalid("--hosts requires at least one host name".into()));
        }
        let names = names
            .into_iter()
            .map(|n| VpsName::try_new(n).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?;
        let err = selector_arity_error("--hosts");
        if target.len() != 1 {
            return Err(err);
        }
        return Ok(ExecTargetPlan {
            selection: HostSelection::Named(names),
            command: only_command(target, err)?,
            source: TargetSource::Selector,
        });
    }

    if let Some(t) = tags {
        let tag_list = parse_hosts_list(&t);
        if tag_list.is_empty() {
            return Err(invalid("--tags requires at least one tag".into()));
        }
        let tag_list = try_tags(tag_list).map_err(|e| invalid(e.to_string()))?;
        let err = selector_arity_error("--tags");
        if target.len() != 1 {
            return Err(err);
        }
        return Ok(ExecTargetPlan {
            selection: HostSelection::Tagged(tag_list),
            command: only_command(target, err)?,
            source: TargetSource::Selector,
        });
    }

    if use_active {
        let err = selector_arity_error("--use-active");
        if target.len() != 1 {
            return Err(err);
        }
        let command = only_command(target, err)?;
        let name = active_vps.ok_or(ExecTargetError::NoActiveVps)?;
        let vps = VpsName::try_new(name).map_err(|e| invalid(e.to_string()))?;
        return Ok(ExecTargetPlan {
            selection: HostSelection::Single(vps),
            command,
            source: TargetSource::ActiveMarker,
        });
    }

    if target.len() != 2 {
        return Err(invalid(format!(
            "expected exactly two positionals `<VPS> <COMMAND>`, got {}; {EXEC_USAGE}",
            target.len()
        )));
    }
    let mut it = target.into_iter();
    let missing = || invalid("expected VPS and COMMAND".to_string());
    let vps = it.next().ok_or_else(missing)?;
    let cmd = it.next().ok_or_else(missing)?;
    let vps = VpsName::try_new(vps).map_err(|e| invalid(e.to_string()))?;
    Ok(ExecTargetPlan {
        selection: HostSelection::Single(vps),
        command: cmd,
        source: TargetSource::Argv,
    })
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

/// The one usage sentence for transfers, so every rejection teaches the same forms.
const TRANSFER_USAGE: &str = concat!(
    "designate the slots explicitly: single host `<VPS> <SRC>... <DEST>`, ",
    "fleet one-file `--all`/`--hosts` `<SRC> <DEST>`, ",
    "or the named slots `--src <PATH>` (repeatable) with `--dest <DIR>`"
);

/// Refusal emitted when a selector meets three or more path positionals.
///
/// # Why this argv cannot be executed (Explicit Target Designation)
///
/// `scp upload a b c` reads `a` as the *host slot*. Adding `--all` used to reread
/// the very same `a` as a *source path*, so a non-positional flag silently remapped
/// a positional and the first token changed meaning without changing shape. A caller
/// that appended `--all` to a working single-host line got a fleet upload whose first
/// file was a host name — and, because the token was a valid relative path, the
/// failure surfaced as a confusing "file not found" rather than as a usage error.
///
/// Fleet transfers with more than one source now go through the named slots, where
/// each path states its own role and no reading depends on the flag set.
fn selector_positional_ambiguity() -> String {
    format!(
        "ambiguous argv: with --all/--hosts and three or more positionals the first token \
         would change role (host slot under single-host, source path under a selector); \
         {TRANSFER_USAGE}"
    )
}

/// Named path slots (`--src` / `--dest`) captured from argv.
///
/// `dest` is always a *directory* here, whatever the number of sources: the role of a
/// slot is fixed by its name, never by how many siblings it has.
#[derive(Debug, Clone, Default)]
pub(crate) struct TransferSlots {
    /// Repeated `--src <PATH>` values, in argv order.
    pub(crate) src: Vec<String>,
    /// The single `--dest <DIR>` value.
    pub(crate) dest: Option<String>,
}

impl TransferSlots {
    /// Collects the `--src` / `--dest` values straight off the clap surface.
    #[must_use]
    pub(crate) fn new(src: Vec<String>, dest: Option<String>) -> Self {
        Self { src, dest }
    }

    /// Whether the caller used the named slots at all.
    fn is_named(&self) -> bool {
        !self.src.is_empty() || self.dest.is_some()
    }
}

/// Resolves the host slot from the selector flags, or [`None`] for single-host.
fn transfer_host_slot(
    all: bool,
    hosts: Option<String>,
) -> Result<Option<crate::vps::HostSelection>, String> {
    use crate::domain::VpsName;
    use crate::vps::HostSelection;
    if all && hosts.is_some() {
        return Err("--all conflicts with --hosts".into());
    }
    if all {
        return Ok(Some(HostSelection::All));
    }
    let Some(list) = hosts else {
        return Ok(None);
    };
    let names = parse_hosts_list(&list);
    if names.is_empty() {
        return Err("--hosts requires at least one host name".into());
    }
    let names = names
        .into_iter()
        .map(|n| VpsName::try_new(n).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(HostSelection::Named(names)))
}

/// Builds a plan out of the named slots, once the host slot is known.
fn plan_from_named_slots(
    selection: Option<crate::vps::HostSelection>,
    slots: TransferSlots,
    target: Vec<String>,
) -> Result<ScpPathPlan, String> {
    use crate::domain::VpsName;
    if slots.src.is_empty() {
        return Err(format!(
            "--dest requires at least one --src; {TRANSFER_USAGE}"
        ));
    }
    let Some(dest) = slots.dest else {
        return Err(format!("--src requires --dest; {TRANSFER_USAGE}"));
    };
    let sources: Vec<PathBuf> = slots.src.into_iter().map(PathBuf::from).collect();
    let dest_dir = PathBuf::from(dest);

    match selection {
        Some(selection) => {
            if !target.is_empty() {
                return Err(format!(
                    "with --all/--hosts and --src/--dest the host and path slots are already \
                     named, so positionals have no slot to fill; {TRANSFER_USAGE}"
                ));
            }
            Ok(ScpPathPlan::MultiHostMultiFile {
                selection,
                sources,
                dest_dir,
            })
        }
        None => {
            if target.len() != 1 {
                return Err(format!(
                    "with --src/--dest and no selector pass exactly one positional, the VPS name; \
                     got {}; {TRANSFER_USAGE}",
                    target.len()
                ));
            }
            let vps = target
                .into_iter()
                .next()
                .ok_or_else(|| "expected the VPS name".to_string())?;
            // G-TYPE-08/09: refine at the boundary (the lookup key stays a `String`).
            let _ = VpsName::try_new(&vps).map_err(|e| e.to_string())?;
            Ok(ScpPathPlan::MultiFile {
                vps,
                sources,
                dest_dir,
            })
        }
    }
}

/// Parses `scp upload|download` / `sftp upload|download` slots into [`ScpPathPlan`].
///
/// # Slots, never arity, never flag-dependent rereading
///
/// The host slot is filled by exactly one mechanism: the leading positional, or a
/// selector (`--all` / `--hosts`). The path slots are filled either by the named
/// `--src` / `--dest` pair or by trailing positionals. No token changes role when a
/// non-positional flag enters or leaves the line:
///
/// - single host, positional: `<VPS> <SRC>... <DEST>` (3+ tokens; slot 0 is the host).
/// - fleet, positional: `--all`/`--hosts` `<SRC> <DEST>` (exactly 2 tokens).
/// - any mode, named: `--src <PATH>`… `--dest <DIR>`, with the VPS positional only
///   when no selector is present.
///
/// A selector with three or more positionals is refused as ambiguous rather than
/// reinterpreted — see [`selector_positional_ambiguity`]. The refusal is an
/// [`crate::errors::SshCliError::InvalidArgument`] at every call site, so the process
/// exits 64 without opening a session.
pub(crate) fn parse_scp_target(
    all: bool,
    hosts: Option<String>,
    slots: TransferSlots,
    target: Vec<String>,
) -> Result<ScpPathPlan, String> {
    use crate::domain::VpsName;
    use crate::vps::HostSelection;

    let selection = transfer_host_slot(all, hosts)?;
    if slots.is_named() {
        return plan_from_named_slots(selection, slots, target);
    }

    if let Some(selection) = selection {
        return match target.len() {
            0 | 1 => Err(format!(
                "with --all/--hosts pass the two path slots `<SRC> <DEST>`, or use \
                 --src/--dest for more than one source; {TRANSFER_USAGE}"
            )),
            2 => {
                // G-SEC-02: Result extraction instead of unwrap after length gate.
                let mut it = target.into_iter();
                let a = PathBuf::from(
                    it.next()
                        .ok_or_else(|| "with --all/--hosts pass SRC DEST paths".to_string())?,
                );
                let b = PathBuf::from(
                    it.next()
                        .ok_or_else(|| "with --all/--hosts pass SRC DEST paths".to_string())?,
                );
                Ok(ScpPathPlan::Single {
                    selection,
                    path_a: a,
                    path_b: b,
                })
            }
            // G-PAR-48 fleet multi-file now travels through the named slots.
            _ => Err(selector_positional_ambiguity()),
        };
    }
    match target.len() {
        0 | 1 => Err(format!(
            "expected the host slot and the path slots \
             (upload: VPS LOCAL... REMOTE; download: VPS REMOTE... LOCAL); {TRANSFER_USAGE}"
        )),
        2 => Err(format!(
            "missing a path slot (single host needs VPS plus two paths, \
             or VPS plus sources plus a destination directory); {TRANSFER_USAGE}"
        )),
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

// The slot-parser tests live beside the parser rather than in `cli/tests.rs`:
// they cover one law across four subcommands, not one command's clap surface.
#[cfg(test)]
#[path = "path_parse_tests.rs"]
mod tests;
