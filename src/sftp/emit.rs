// SPDX-License-Identifier: MIT OR Apache-2.0
//! SFTP result emitters — the only stdout surface (A7 split).
#![forbid(unsafe_code)]
#![allow(unused_imports)]

use super::*;

/// The target an SFTP subcommand designated in argv.
///
/// SFTP has no active-marker path: every subcommand takes the host as a positional,
/// so the provenance is always [`crate::json_wire::TargetSource::Argv`].
pub(crate) fn designated_target(name: &str) -> crate::json_wire::ExecTarget {
    crate::json_wire::ExecTarget::new(name, crate::json_wire::TargetSource::Argv)
}

/// Records the host this SFTP invocation resolved, so failures can name it.
///
/// Called only once the registry lookup has succeeded, so a `VpsNotFound` failure
/// never claims a host that was never resolved — the same rule `exec` follows in
/// `vps::exec_ops::single`. Publishing here rather than after `connect` is what makes
/// a refused connection carry its target into the error envelope.
pub(crate) fn publish_target(name: &str) -> crate::json_wire::ExecTarget {
    let target = designated_target(name);
    crate::json_wire::set_resolved_target(&target);
    target
}

/// Emits one `sftp-transfer` event.
///
/// Takes the resolved [`crate::json_wire::ExecTarget`] rather than a bare name: the
/// envelope must state *how* the host was designated, and folding the provenance into
/// the existing host argument keeps this function under the
/// `clippy::too_many_arguments` ceiling it already sits on.
pub(crate) fn emit_transfer(
    direction: &str,
    target: &crate::json_wire::ExecTarget,
    local: &str,
    remote: &str,
    result: TransferResult,
    json: bool,
    recursive: bool,
) -> anyhow::Result<()> {
    if json {
        output::print_sftp_transfer_json(
            direction,
            target,
            local,
            remote,
            result.bytes_transferred,
            result.duration_ms,
            recursive,
        )?;
    } else {
        let msg = if direction == "upload" {
            Message::SftpUploadCompleted {
                bytes: result.bytes_transferred,
                ms: result.duration_ms,
            }
        } else {
            Message::SftpDownloadCompleted {
                bytes: result.bytes_transferred,
                ms: result.duration_ms,
            }
        };
        output::print_success(&i18n::t(msg));
    }
    Ok(())
}

pub(crate) fn emit_list(
    vps: &str,
    path: &str,
    entries: &[SftpListEntry],
    json: bool,
) -> anyhow::Result<()> {
    if json {
        output::print_sftp_list_json(vps, path, entries)?;
    } else {
        // A4: `println!` bypassed the `output` facade, which is the only layer that
        // honours `--quiet` and the broken-pipe policy. `src/lib.rs` states product code
        // never calls it; these two listings were the exceptions that proved nobody was
        // checking. Piping `sftp ls` into `head` used to abort on EPIPE instead of
        // exiting 141.
        for e in entries {
            output::write_line_fmt(format_args!(
                "{}\t{}\t{}",
                e.kind,
                e.size.map(|s| s.to_string()).unwrap_or_else(|| "-".into()),
                e.path
            ))?;
        }
    }
    Ok(())
}

pub(crate) fn emit_stat(vps: &str, st: &SftpStat, json: bool) -> anyhow::Result<()> {
    if json {
        output::print_sftp_stat_json(vps, st)?;
    } else {
        // A4: same facade violation as `emit_list` above.
        output::write_line_fmt(format_args!(
            "path={} kind={} size={} mode={:?} mtime={:?}",
            st.path,
            st.kind,
            st.size.map(|s| s.to_string()).unwrap_or_else(|| "-".into()),
            st.mode,
            st.mtime
        ))?;
    }
    Ok(())
}

pub(crate) fn emit_fs_op(
    op: &str,
    vps: &str,
    path: &str,
    to: Option<&str>,
    duration_ms: u64,
    json: bool,
) -> anyhow::Result<()> {
    if json {
        output::print_sftp_fs_op_json(op, vps, path, to, duration_ms)?;
    } else {
        // A4: routed through `i18n::t` instead of an inline English `format!`, so
        // `--lang pt-BR` stops producing English for the most-read SFTP output.
        let msg = match to {
            Some(t) => i18n::Message::SftpFsOpDoneTo {
                op: op.to_string(),
                path: path.to_string(),
                to: t.to_string(),
                ms: duration_ms,
            },
            None => i18n::Message::SftpFsOpDone {
                op: op.to_string(),
                path: path.to_string(),
                ms: duration_ms,
            },
        };
        output::print_success(&i18n::t(msg));
    }
    Ok(())
}
