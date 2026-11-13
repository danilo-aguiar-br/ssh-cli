// SPDX-License-Identifier: MIT OR Apache-2.0
//! SFTP result emitters — the only stdout surface (A7 split).
#![forbid(unsafe_code)]
#![allow(unused_imports)]

use super::*;
use crate::output::sanitize::escape_untrusted;

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
        output::try_print_success(&i18n::t(msg))?;
    }
    Ok(())
}

/// One `sftp ls` text line: `kind<TAB>size<TAB>path`.
///
/// The path and kind come from the remote server. Both are escaped with
/// [`escape_untrusted`] before the line is built, so a file named
/// `x\nfile\t999\tFORGED` shows as one entry instead of forging a second one:
/// the TAB and newline are the delimiters of this format, which is why the
/// whitespace-preserving variant is not used here.
pub(crate) fn list_line(e: &SftpListEntry) -> String {
    format!(
        "{}\t{}\t{}",
        escape_untrusted(&e.kind),
        e.size.map_or_else(|| "-".into(), |s| s.to_string()),
        escape_untrusted(&e.path)
    )
}

/// One `sftp stat` text line; remote fields escaped as in [`list_line`].
pub(crate) fn stat_line(st: &SftpStat) -> String {
    format!(
        "path={} kind={} size={} mode={:?} mtime={:?}",
        escape_untrusted(&st.path),
        escape_untrusted(&st.kind),
        st.size.map_or_else(|| "-".into(), |s| s.to_string()),
        st.mode,
        st.mtime
    )
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
            output::write_line(&list_line(e))?;
        }
    }
    Ok(())
}

pub(crate) fn emit_stat(vps: &str, st: &SftpStat, json: bool) -> anyhow::Result<()> {
    if json {
        output::print_sftp_stat_json(vps, st)?;
    } else {
        // A4: same facade violation as `emit_list` above.
        output::write_line(&stat_line(st))?;
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
        output::try_print_success(&i18n::t(msg))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Remote entry kind used by the fixtures below.
    const KIND_FILE: &str = "file";

    fn entry(path: &str) -> SftpListEntry {
        SftpListEntry {
            name: path.into(),
            path: path.into(),
            kind: KIND_FILE.into(),
            size: Some(3),
            mode: None,
        }
    }

    #[test]
    fn a_forged_remote_name_stays_on_one_line_with_three_fields() {
        let line = list_line(&entry("x\nfile\t999\tFORGED"));
        assert!(!line.contains('\n'), "{line:?}");
        assert_eq!(line.matches('\t').count(), 2, "{line:?}");
        assert!(
            line.ends_with("x\\u{a}file\\u{9}999\\u{9}FORGED"),
            "{line:?}"
        );
    }

    #[test]
    fn a_plain_entry_keeps_its_format() {
        assert_eq!(list_line(&entry("/srv/a.txt")), "file\t3\t/srv/a.txt");
    }

    #[test]
    fn stat_line_escapes_the_path_and_the_terminal_sequences() {
        let st = SftpStat {
            path: "/a\u{1b}[2J\nb".into(),
            kind: KIND_FILE.into(),
            size: None,
            mode: Some(0o644),
            mtime: None,
        };
        let line = stat_line(&st);
        assert!(!line.contains('\n') && !line.contains('\u{1b}'), "{line:?}");
        assert!(
            line.starts_with("path=/a\\u{1b}[2J\\u{a}b kind=file size=-"),
            "{line:?}"
        );
    }
}
