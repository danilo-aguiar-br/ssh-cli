// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Two-file commit for `vps rename` (the `active` marker and `config.toml`).
//!
//! # Why the marker is written first
//!
//! The marker and the config are two files, so there is no single atomic step.
//! Writing the config first and the marker second leaves, when the marker write
//! fails, a renamed host with a marker that points at a name that no longer
//! exists, and the command still reports an error. Writing the marker first turns
//! that failure into a clean abort: nothing else has changed yet.
//!
//! If the config save then fails, the previous marker is restored (a small
//! compensating step). The restore can itself fail, and the window between the
//! two writes can be cut short by a crash. Both leave a marker naming a host that
//! does not exist, which is fail-closed: `--use-active` resolves it to
//! `vps_not_found` (exit 66) instead of acting on the wrong host.
//!
//! The helper is generic over closures so the three orders of failure are
//! covered by unit tests without a filesystem and without injecting faults
//! through argv, which would not be portable.

use crate::errors::SshCliResult;

/// Log target shared by every event of the edit/rename path.
const LOG_TARGET: &str = "ssh_cli::vps::edit";

/// Commits a rename: marker first, then config, restoring the marker on failure.
///
/// When `marker_rewrite` is false the marker does not depend on the rename and
/// the commit is only `save`.
///
/// # Errors
/// The error of `write_marker` (nothing else ran), or the error of `save` after
/// the previous marker was restored. A failed restore is reported through
/// `tracing` at `error` level, the only level shown by the default filter, and
/// never replaces the `save` error.
pub(super) fn commit_rename<S, W, R>(
    marker_rewrite: bool,
    save: S,
    write_marker: W,
    restore_marker: R,
) -> SshCliResult<()>
where
    S: FnOnce() -> SshCliResult<()>,
    W: FnOnce() -> SshCliResult<()>,
    R: FnOnce() -> SshCliResult<()>,
{
    if !marker_rewrite {
        return save();
    }
    write_marker()?;
    let Err(save_error) = save() else {
        return Ok(());
    };
    match restore_marker() {
        Ok(()) => tracing::warn!(
            target: LOG_TARGET,
            "config save failed after the active marker moved; previous marker restored"
        ),
        Err(restore_error) => tracing::error!(
            target: LOG_TARGET,
            restore_error = %restore_error,
            "config save failed and the previous active marker could not be restored; \
             the marker may still point at the new name"
        ),
    }
    Err(save_error)
}

#[cfg(test)]
mod tests {
    use super::commit_rename;
    use crate::errors::{SshCliError, SshCliResult};
    use std::cell::RefCell;

    fn failure(msg: &str) -> SshCliResult<()> {
        Err(SshCliError::InvalidArgument(msg.to_string()))
    }

    fn calls() -> RefCell<Vec<&'static str>> {
        RefCell::new(Vec::new())
    }

    fn message(result: SshCliResult<()>) -> String {
        result.expect_err("the commit must fail").to_string()
    }

    #[test]
    fn happy_path_writes_marker_then_saves() {
        let log = calls();
        let result = commit_rename(
            true,
            || {
                log.borrow_mut().push("save");
                Ok(())
            },
            || {
                log.borrow_mut().push("marker");
                Ok(())
            },
            || {
                log.borrow_mut().push("restore");
                Ok(())
            },
        );
        assert!(result.is_ok());
        assert_eq!(*log.borrow(), vec!["marker", "save"]);
    }

    #[test]
    fn marker_failure_aborts_before_the_save() {
        let log = calls();
        let result = commit_rename(
            true,
            || {
                log.borrow_mut().push("save");
                Ok(())
            },
            || {
                log.borrow_mut().push("marker");
                failure("marker")
            },
            || {
                log.borrow_mut().push("restore");
                Ok(())
            },
        );
        assert!(message(result).contains("marker"));
        assert_eq!(*log.borrow(), vec!["marker"], "save must never run");
    }

    #[test]
    fn save_failure_restores_the_marker_and_keeps_the_save_error() {
        let log = calls();
        let result = commit_rename(
            true,
            || {
                log.borrow_mut().push("save");
                failure("save")
            },
            || {
                log.borrow_mut().push("marker");
                Ok(())
            },
            || {
                log.borrow_mut().push("restore");
                Ok(())
            },
        );
        assert!(message(result).contains("save"));
        assert_eq!(*log.borrow(), vec!["marker", "save", "restore"]);
    }

    #[test]
    fn failed_restore_still_returns_the_save_error() {
        let log = calls();
        let result = commit_rename(
            true,
            || {
                log.borrow_mut().push("save");
                failure("save")
            },
            || {
                log.borrow_mut().push("marker");
                Ok(())
            },
            || {
                log.borrow_mut().push("restore");
                failure("restore")
            },
        );
        let text = message(result);
        assert!(text.contains("save"), "{text}");
        assert!(!text.contains("restore"), "{text}");
        assert_eq!(*log.borrow(), vec!["marker", "save", "restore"]);
    }

    #[test]
    fn without_a_marker_rewrite_only_the_save_runs() {
        let log = calls();
        let result = commit_rename(
            false,
            || {
                log.borrow_mut().push("save");
                Ok(())
            },
            || {
                log.borrow_mut().push("marker");
                Ok(())
            },
            || {
                log.borrow_mut().push("restore");
                Ok(())
            },
        );
        assert!(result.is_ok());
        assert_eq!(*log.borrow(), vec!["save"]);
    }

    #[test]
    fn without_a_marker_rewrite_a_save_failure_does_not_touch_the_marker() {
        let log = calls();
        let result = commit_rename(
            false,
            || {
                log.borrow_mut().push("save");
                failure("save")
            },
            || {
                log.borrow_mut().push("marker");
                Ok(())
            },
            || {
                log.borrow_mut().push("restore");
                Ok(())
            },
        );
        assert!(message(result).contains("save"));
        assert_eq!(*log.borrow(), vec!["save"]);
    }
}
