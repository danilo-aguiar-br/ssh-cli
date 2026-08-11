// SPDX-License-Identifier: MIT OR Apache-2.0
// GAP-SSH-EXEC-ARGC-001: target designation shared by the three exec surfaces.
#![forbid(unsafe_code)]
//! Resolves *which host* a command runs against.
//!
//! # Why the module is `targeting` and not `exec_target`
//!
//! It arrived serving the three exec surfaces and was named after them. It now also
//! resolves `health-check`, whose `--use-active` opt-in landed in 0.5.5, so a name
//! that promises "exec" understates what a reader may put here — and a module named
//! narrower than its job invites the next caller to copy it instead of extend it,
//! which is the duplication described below arriving a second time.
//!
//! # Why not the shorter `target`
//!
//! Because `target` is unpublishable, and silently so. This repository's
//! `.gitignore` carries a bare `target/`, and a gitignore pattern with no leading
//! slash matches at *every* depth, not just the root. `src/cli/target/` was
//! therefore ignored by git, and `cargo package` — which lists files through git
//! inside a git repository — dropped both of its files from the tarball. The crate
//! built, all tests passed and all eleven gates were green; only
//! `cargo publish --dry-run` failed, with `E0583: file not found for module
//! `target``, because the published crate was the first artefact that did not
//! contain the module.
//!
//! The ignore rule has since been anchored to `/target/`, which is what it always
//! meant. The rename stands anyway: a source directory whose name collides with
//! Cargo's build output invites the same failure from any tool that reasons about
//! `target` by name, and being right only because one line of `.gitignore` is right
//! is not a property worth depending on. `tests/gaps_v067_packaging_reach.rs` now
//! fails when a source path is unreachable to packaging, so the class cannot return
//! unnoticed.
//!
//! The *functions* keep their `_exec_target` suffixes on purpose. `resolve_exec_target`
//! names an operation, not a location, and renaming it would churn every call site to
//! say nothing new.
//!
//! # Why this is its own module
//!
//! `exec`, `sudo-exec` and `su-exec` used to carry byte-identical copies of this
//! resolution block inside the dispatcher, including a `starts_with("no active
//! VPS")` string test that recovered the error kind from the wording of a
//! human-facing sentence. Three copies meant a fix had to be applied three times or
//! not at all — and GAP-SSH-EXEC-ARGC-001 was precisely a defect that lived in all
//! three at once, because it lived in the block they shared.
//!
//! Deciding the target is also the single highest-consequence decision the CLI
//! makes: not *what* runs, but *where*. That deserves a named module rather than a
//! paragraph repeated inside a match arm.

use crate::cli::path_parse::{parse_exec_target, ExecTargetError, ExecTargetPlan};
use crate::errors::SshCliError;
use anyhow::Result;
use std::path::Path;

/// Positional and selector inputs shared by the three exec surfaces.
///
/// Bundled so the resolver stays a two-argument function instead of a six-argument
/// one, which is what `clippy::too_many_arguments` exists to prevent.
pub(crate) struct ExecTargetArgs {
    /// `--all`.
    pub(crate) all: bool,
    /// `--hosts LIST`.
    pub(crate) hosts: Option<String>,
    /// `--tags LIST`.
    pub(crate) tags: Option<String>,
    /// `--use-active`.
    pub(crate) use_active: bool,
    /// Raw positionals as clap collected them.
    pub(crate) target: Vec<String>,
}

/// Resolves the execution target for `exec`, `sudo-exec` and `su-exec`.
///
/// # Why the registry lookup lives here and not in the parser
///
/// [`parse_exec_target`] stays pure so it can be exercised without a filesystem.
/// The guard that rejects a displaced token needs to read the inventory, and this
/// layer already owns config I/O.
///
/// # Errors
///
/// Returns [`SshCliError::NoActiveVps`] when `--use-active` finds no marker, and
/// [`SshCliError::InvalidArgument`] for every argv shape that does not designate a
/// target unambiguously.
pub(crate) fn resolve_exec_target(
    args: ExecTargetArgs,
    config_override: Option<&Path>,
) -> Result<ExecTargetPlan> {
    let active = crate::vps::read_active_vps(config_override)?;
    let use_active = args.use_active;
    let plan = parse_exec_target(
        args.all,
        args.hosts,
        args.tags,
        use_active,
        args.target,
        active,
    )
    .map_err(|e| match e {
        ExecTargetError::NoActiveVps => SshCliError::NoActiveVps,
        ExecTargetError::Invalid(s) => SshCliError::InvalidArgument(s),
    })?;

    // Displaced-token guard: `--use-active <registered-name>` is the caller still
    // believing the first positional is the host. Executing it would run an
    // inventory name as a shell binary on a *different* machine — the exact shape of
    // GAP-SSH-EXEC-ARGC-001, one flag further along.
    if use_active && crate::vps::find_by_name(config_override, &plan.command)?.is_some() {
        return Err(SshCliError::InvalidArgument(format!(
            "`{name}` is a registered VPS, not a command; \
             drop --use-active and pass `{name} <COMMAND>`",
            name = plan.command
        ))
        .into());
    }
    Ok(plan)
}
/// Positional and selector inputs for `health-check`.
///
/// Separate from [`ExecTargetArgs`] because the two surfaces do not share a shape:
/// `health-check` has no command to run, so it carries an optional host name rather
/// than a positional vector, and it has no `--tags`.
pub(crate) struct HealthTargetArgs {
    /// `--all`.
    pub(crate) all: bool,
    /// `--hosts LIST`.
    pub(crate) hosts: Option<String>,
    /// `--use-active`.
    pub(crate) use_active: bool,
    /// Optional positional host name.
    pub(crate) vps_name: Option<String>,
}

/// Resolves the target for `health-check`, with the same opt-in rule as exec.
///
/// # Why `health-check` now demands the opt-in too
///
/// It used to inherit the active marker silently whenever the name was omitted, and
/// that was defended as safe because a connectivity probe is an idempotent read. The
/// defence is true about the *probe* and false about the *habit*: an operator who
/// learns that `health-check` needs no target carries that expectation to `exec`,
/// which is the surface where GAP-SSH-EXEC-ARGC-001 cost a day of work. Ergonomic
/// asymmetry between a read and a write is a trap, and the read is the cheaper half
/// to make strict.
///
/// The exit taxonomy is now identical across both families: 64 means no target was
/// designated, 66 means the designated target does not exist.
///
/// # Errors
///
/// [`SshCliError::InvalidArgument`] when nothing designates a target,
/// [`SshCliError::NoActiveVps`] when `--use-active` finds no marker, and refinement
/// failures from [`crate::domain::VpsName`].
pub(crate) fn resolve_health_target(
    args: HealthTargetArgs,
    config_override: Option<&Path>,
) -> Result<(crate::vps::HostSelection, crate::json_wire::TargetSource)> {
    use crate::domain::VpsName;
    use crate::json_wire::TargetSource;
    use crate::vps::HostSelection;

    let invalid = |s: &str| -> anyhow::Error { SshCliError::InvalidArgument(s.to_string()).into() };
    let refine = |n: String| -> Result<VpsName> {
        VpsName::try_new(n).map_err(|e| SshCliError::InvalidArgument(e.to_string()).into())
    };

    if args.all {
        return Ok((HostSelection::All, TargetSource::Selector));
    }
    if let Some(h) = args.hosts {
        let names = crate::cli::path_parse::parse_hosts_list(&h);
        if names.is_empty() {
            return Err(invalid("--hosts requires at least one host name"));
        }
        let names = names.into_iter().map(refine).collect::<Result<Vec<_>>>()?;
        return Ok((HostSelection::Named(names), TargetSource::Selector));
    }
    if let Some(n) = args.vps_name {
        return Ok((HostSelection::Single(refine(n)?), TargetSource::Argv));
    }
    if args.use_active {
        let active = crate::vps::read_active_vps(config_override)?;
        let name = active.ok_or(SshCliError::NoActiveVps)?;
        return Ok((
            HostSelection::Single(refine(name)?),
            TargetSource::ActiveMarker,
        ));
    }
    Err(invalid(HEALTH_USAGE))
}

/// The one usage sentence for `health-check`, mirroring `EXEC_USAGE`.
pub(crate) const HEALTH_USAGE: &str = concat!(
    "designate the target explicitly: `<VPS>`, ",
    "or a selector (`--all`/`--hosts <LIST>`), ",
    "or the active marker deliberately (`--use-active`)"
);

/// Everything the three exec surfaces turn into [`crate::vps::ExecOptions`].
///
/// The elevation password is the only field that differs between them, so it is
/// modelled as one optional slot rather than three near-identical builders.
pub(crate) struct ExecCommonArgs {
    /// `--step` commands, still unvalidated.
    pub(crate) steps: Vec<String>,
    /// Flattened SSH auth overrides.
    pub(crate) auth: crate::cli::SshAuthArgs,
    /// Elevation secret already resolved from argv or stdin (`sudo` / `su`).
    pub(crate) elevation_password: Option<secrecy::SecretString>,
    /// Which elevation slot [`Self::elevation_password`] fills.
    pub(crate) elevation: Elevation,
    /// Subcommand-local `--timeout`.
    pub(crate) timeout: Option<u64>,
    /// Global `--timeout`, used when the local one is absent.
    pub(crate) global_timeout: Option<u64>,
    /// `--description` audit comment.
    pub(crate) description: Option<String>,
    /// Global `--replace-host-key`.
    pub(crate) replace_host_key: bool,
    /// Global `--disable-sudo`.
    pub(crate) disable_sudo: bool,
    /// Provenance of the resolved host.
    pub(crate) target_source: crate::json_wire::TargetSource,
}

/// Which elevation slot an exec surface fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Elevation {
    /// `exec` — no elevation.
    None,
    /// `sudo-exec`.
    Sudo,
    /// `su-exec`.
    Su,
}

/// Builds [`crate::vps::ExecOptions`] once instead of three times.
///
/// # Why
///
/// Each surface spelled the same twenty-odd lines of option assembly. Duplication
/// there is not just noise: `target_source` had to be threaded into all three, and a
/// field silently missing from one copy would have produced an envelope that names
/// no host on exactly one code path.
///
/// # Errors
///
/// Propagates stdin read failures and an invalid effective timeout.
pub(crate) fn build_exec_options(c: ExecCommonArgs) -> Result<crate::vps::ExecOptions> {
    let ExecCommonArgs {
        steps,
        auth,
        elevation_password,
        elevation,
        timeout,
        global_timeout,
        description,
        replace_host_key,
        disable_sudo,
        target_source,
    } = c;
    let key = auth.key_path_string();
    let password = crate::cli::read_stdin_if(auth.password_stdin, auth.password.clone())?;
    let key_passphrase =
        crate::cli::read_stdin_if(auth.key_passphrase_stdin, auth.key_passphrase.clone())?;
    let steps = crate::cli::parse_remote_steps(steps).map_err(SshCliError::InvalidArgument)?;
    let (sudo_password, su_password) = match elevation {
        Elevation::None => (None, None),
        Elevation::Sudo => (elevation_password, None),
        Elevation::Su => (None, elevation_password),
    };
    Ok(crate::vps::ExecOptions {
        password,
        sudo_password,
        su_password,
        key,
        key_passphrase,
        timeout: crate::cli::effective_timeout_ms(timeout, global_timeout)
            .map_err(SshCliError::InvalidArgument)?,
        description,
        replace_host_key,
        disable_sudo,
        steps,
        target_source,
        use_agent: auth.use_agent,
        agent_socket: auth
            .agent_socket
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned()),
    })
}

#[cfg(test)]
mod tests;
