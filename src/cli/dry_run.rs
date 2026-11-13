// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! `--dry-run` policy: which commands honour it, and the plan they emit.
//!
//! # Why this module exists
//!
//! The switch was four functions and a global inside `cli/mod.rs`, which sits on
//! the frozen component budget. Adding `vps edit` and `vps rename` to the
//! accepted set meant touching that file, and a budget file that only ever
//! grows is a budget that stops meaning anything. The policy now lives here and
//! `cli/mod.rs` re-exports the same names, so call sites did not move.
//!
//! # Why the plan is always JSON
//!
//! Even in text mode the preview goes to stdout as JSON: the point of a preview
//! is that a machine can diff it against what it intended, and prose cannot be
//! diffed. The text renderings stay for human probes, where the envelope is
//! still readable.

use crate::cli::{Command, SecretsAction, SftpAction, VpsAction};
use crate::errors::SshCliError;
use std::sync::atomic::{AtomicBool, Ordering};

/// Process-wide `--dry-run` switch.
static DRY_RUN: AtomicBool = AtomicBool::new(false);

/// Installs the `--dry-run` policy for this one-shot process.
pub fn set_dry_run(value: bool) {
    DRY_RUN.store(value, Ordering::Relaxed);
}

/// Whether destructive operations must emit a plan instead of executing.
#[must_use]
pub fn is_dry_run() -> bool {
    DRY_RUN.load(Ordering::Relaxed)
}

/// Whether this command implements `--dry-run`.
///
/// Covers the operations that destroy or rewrite state without a prior read:
/// registry removal, import, rename and edit, remote unlink and rmdir, and the
/// two `secrets` writes that can invalidate every stored credential at once.
/// Transfers and `exec` are deliberately absent — their effect is the remote
/// command itself, which this CLI cannot preview without running it.
#[must_use]
pub fn supports_dry_run(command: &Command) -> bool {
    match command {
        Command::Vps { action } => matches!(
            action,
            VpsAction::Remove { .. }
                | VpsAction::Import { .. }
                | VpsAction::Edit { .. }
                | VpsAction::Rename { .. }
        ),
        Command::Sftp { action, .. } => {
            matches!(action, SftpAction::Rm { .. } | SftpAction::Rmdir { .. })
        }
        Command::Secrets { action } => matches!(
            action,
            SecretsAction::Init { .. } | SecretsAction::Reencrypt { .. }
        ),
        _ => false,
    }
}

/// Rejects `--dry-run` on a command that cannot honour it.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] (exit 64) when the flag was passed to an
/// unsupported command.
pub fn guard_dry_run_supported(command: &Command) -> Result<(), SshCliError> {
    if !is_dry_run() || supports_dry_run(command) {
        return Ok(());
    }
    Err(SshCliError::InvalidArgument(
        "--dry-run is not implemented for this command; it is accepted only by \
         `vps remove`, `vps import`, `vps edit`, `vps rename`, `sftp rm`, `sftp rmdir`, \
         `secrets init` and `secrets reencrypt`"
            .to_string(),
    ))
}

/// Emits the plan for a destructive operation and reports whether to stop.
///
/// Returns `true` when the caller must return without mutating anything.
///
/// # Errors
/// stdout write failures (including `BrokenPipe`).
pub fn dry_run_stop(
    operation: &str,
    fields: &[(&str, serde_json::Value)],
) -> Result<bool, SshCliError> {
    if !is_dry_run() {
        return Ok(false);
    }
    let mut map = std::collections::BTreeMap::new();
    map.insert("operation".to_string(), serde_json::json!(operation));
    map.insert("dry_run".to_string(), serde_json::json!(true));
    map.insert("executed".to_string(), serde_json::json!(false));
    for (k, v) in fields {
        map.insert((*k).to_string(), v.clone());
    }
    crate::json_wire::print_json_line(&crate::json_wire::SuccessEnvelope::new("dry-run", map))
        .map_err(SshCliError::Io)?;
    Ok(true)
}
