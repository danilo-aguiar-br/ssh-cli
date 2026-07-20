// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SFTP: SFTP v3 client ops via russh-sftp (stream only — no full-file heap).
#![forbid(unsafe_code)]
//! Open the `sftp` subsystem and perform one-shot file/FS operations.
//!
//! # Memory
//!
//! Bulk transfer uses [`tokio::io`] on [`russh_sftp::client::fs::File`] in
//! [`crate::constants::SFTP_IO_CHUNK`] chunks. **Never** call
//! `SftpSession::read` / `SftpSession::write` for bulk (those allocate the whole
//! file — G-SFTP-11).
//!
//! # Symlinks
//!
//! Recursive walks **do not follow** symlinks (fail-safe default G-SFTP-06).

use super::client_handler::ClientHandler;
use super::client::TransferResult;
use super::sftp_path::{
    check_depth, ensure_local_under, join_remote, validate_entry_name, validate_remote_path,
};
use super::sftp_types::{SftpListEntry, SftpStat};
use super::scp_wire::partial_download_path;
use crate::constants::{SFTP_IO_CHUNK, SFTP_LIST_MAX_ENTRIES, SFTP_SUBSYSTEM};
use crate::errors::{SshCliError, SshCliResult};
use russh_sftp::client::SftpSession;
use russh_sftp::protocol::{FileAttributes, OpenFlags, StatusCode};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Wall-clock deadline for multi-op / FS paths that bypass `client.sftp_*` wrappers (G-SFTP-R05).
///
/// # Errors
/// [`SshCliError::SshTimeout`] when the future exceeds `timeout_ms`.
pub async fn under_timeout<T, F>(timeout_ms: u64, fut: F) -> SshCliResult<T>
where
    F: Future<Output = SshCliResult<T>>,
{
    tokio::time::timeout(Duration::from_millis(timeout_ms), fut)
        .await
        .map_err(|_| SshCliError::SshTimeout(timeout_ms))?
}

fn map_sftp_err(path: &str, e: russh_sftp::client::error::Error) -> SshCliError {
    match e {
        russh_sftp::client::error::Error::Status(st)
            if st.status_code == StatusCode::NoSuchFile =>
        {
            SshCliError::FileNotFound(path.to_owned())
        }
        russh_sftp::client::error::Error::Timeout => {
            SshCliError::channel_msg(format!("sftp timeout on {path}"))
        }
        other => SshCliError::channel_msg(format!("sftp {path}: {other}")),
    }
}

fn kind_from_attrs(attrs: &FileAttributes) -> String {
    let ft = attrs.file_type();
    if ft.is_dir() {
        "dir".into()
    } else if ft.is_symlink() {
        "symlink".into()
    } else if ft.is_file() {
        "file".into()
    } else {
        "other".into()
    }
}

/// Converts product timeout ms into SFTP response timeout seconds (crate default 10).
#[must_use]
pub fn sftp_timeout_secs(timeout_ms: u64) -> u64 {
    timeout_ms.div_ceil(1000).max(1)
}

/// Opens an SFTP session on a new channel of the authenticated SSH handle.
///
/// # Errors
/// Channel / subsystem / SFTP INIT failures.
pub async fn open_sftp_session(
    session: &russh::client::Handle<ClientHandler>,
    timeout_ms: u64,
) -> SshCliResult<SftpSession> {
    let channel = session
        .channel_open_session()
        .await
        .map_err(|e| SshCliError::channel_msg(format!("open SFTP session: {e}")))?;
    channel
        .request_subsystem(true, SFTP_SUBSYSTEM)
        .await
        .map_err(|e| SshCliError::channel_msg(format!("request subsystem {SFTP_SUBSYSTEM}: {e}")))?;
    let sftp = SftpSession::new(channel.into_stream())
        .await
        .map_err(|e| SshCliError::channel_msg(format!("SftpSession::new: {e}")))?;
    sftp.set_timeout(sftp_timeout_secs(timeout_ms));
    Ok(sftp)
}

/// Best-effort close (errors logged, not fatal to the transfer result).
pub async fn close_sftp(sftp: &SftpSession) {
    if let Err(e) = sftp.close().await {
        tracing::debug!(err = %e, "sftp close failed");
    }
}

/// Streams a local regular file to the remote path (create+truncate).
///
/// # Errors
/// Local I/O, missing local file, or SFTP write failures.
pub async fn upload_file(
    sftp: &SftpSession,
    local: &Path,
    remote: &str,
) -> SshCliResult<TransferResult> {
    validate_remote_path(remote)?;
    let start = Instant::now();
    // symlink_metadata: no-follow (G-SFTP-06).
    let meta = tokio::fs::symlink_metadata(local).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            SshCliError::FileNotFound(local.display().to_string())
        } else {
            SshCliError::Io(e)
        }
    })?;
    if meta.file_type().is_symlink() {
        return Err(SshCliError::InvalidArgument(
            "sftp upload refuses local symlinks (no-follow policy)".into(),
        ));
    }
    if meta.is_dir() {
        return Err(SshCliError::InvalidArgument(
            "sftp upload of a directory requires --recursive".into(),
        ));
    }
    if !meta.is_file() {
        return Err(SshCliError::InvalidArgument(
            "sftp upload only supports regular files (or --recursive for trees)".into(),
        ));
    }

    let mut local_file = tokio::fs::File::open(local)
        .await
        .map_err(SshCliError::Io)?;

    let flags = OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE;
    let mut attrs = FileAttributes::default();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        attrs.permissions = Some(meta.permissions().mode());
    }
    if let Ok(mtime) = meta.modified() {
        if let Ok(d) = mtime.duration_since(std::time::UNIX_EPOCH) {
            attrs.mtime = Some(u32::try_from(d.as_secs()).unwrap_or(u32::MAX));
        }
    }

    let mut remote_file = sftp
        .open_with_flags_and_attributes(remote.to_owned(), flags, attrs.clone())
        .await
        .map_err(|e| map_sftp_err(remote, e))?;

    let mut buf = vec![0_u8; SFTP_IO_CHUNK];
    let mut bytes = 0_u64;
    loop {
        if crate::signals::should_stop() {
            return Err(SshCliError::InvalidArgument(
                crate::i18n::t(crate::i18n::Message::OperationCancelled),
            ));
        }
        let n = local_file.read(&mut buf).await.map_err(SshCliError::Io)?;
        if n == 0 {
            break;
        }
        remote_file
            .write_all(&buf[..n])
            .await
            .map_err(|e| SshCliError::channel_msg(format!("sftp write {remote}: {e}")))?;
        bytes = bytes.saturating_add(n as u64);
    }
    remote_file
        .shutdown()
        .await
        .map_err(|e| SshCliError::channel_msg(format!("sftp shutdown {remote}: {e}")))?;

    // Best-effort metadata (mode/mtime) if open flags did not stick.
    let _ = sftp.set_metadata(remote.to_owned(), attrs).await;

    Ok(TransferResult {
        bytes_transferred: bytes,
        duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

/// Streams a remote regular file to a local path (partial + atomic rename).
///
/// On any failure after the partial is created, the partial file is removed
/// (G-SFTP-R04 — parity with SCP cleanup).
///
/// # Errors
/// Missing remote, SFTP read, or local I/O failures.
pub async fn download_file(
    sftp: &SftpSession,
    remote: &str,
    local: &Path,
) -> SshCliResult<TransferResult> {
    validate_remote_path(remote)?;
    let start = Instant::now();

    // Symlink no-follow: refuse when the path itself is a symlink.
    let link_meta = sftp
        .symlink_metadata(remote.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote, e))?;
    if link_meta.file_type().is_symlink() {
        return Err(SshCliError::InvalidArgument(format!(
            "sftp download refuses remote symlink (no-follow): {remote}"
        )));
    }
    if link_meta.file_type().is_dir() {
        return Err(SshCliError::InvalidArgument(
            "sftp download of a directory requires --recursive".into(),
        ));
    }

    let mut remote_file = sftp
        .open(remote.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote, e))?;

    if let Some(parent) = local.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(SshCliError::Io)?;
        }
    }

    let partial = partial_download_path(local);
    let result = download_file_to_partial(
        &mut remote_file,
        &partial,
        local,
        remote,
        &link_meta,
        start,
    )
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&partial).await;
    }
    result
}

async fn download_file_to_partial(
    remote_file: &mut russh_sftp::client::fs::File,
    partial: &Path,
    local: &Path,
    remote: &str,
    link_meta: &FileAttributes,
    start: Instant,
) -> SshCliResult<TransferResult> {
    let mut local_file = tokio::fs::File::create(partial)
        .await
        .map_err(SshCliError::Io)?;

    let mut buf = vec![0_u8; SFTP_IO_CHUNK];
    let mut bytes = 0_u64;
    loop {
        if crate::signals::should_stop() {
            return Err(SshCliError::InvalidArgument(
                crate::i18n::t(crate::i18n::Message::OperationCancelled),
            ));
        }
        let n = remote_file
            .read(&mut buf)
            .await
            .map_err(|e| SshCliError::channel_msg(format!("sftp read {remote}: {e}")))?;
        if n == 0 {
            break;
        }
        local_file
            .write_all(&buf[..n])
            .await
            .map_err(SshCliError::Io)?;
        bytes = bytes.saturating_add(n as u64);
    }
    local_file.flush().await.map_err(SshCliError::Io)?;
    drop(local_file);

    tokio::fs::rename(partial, local)
        .await
        .map_err(SshCliError::Io)?;

    // Best-effort local mode (Unix).
    #[cfg(unix)]
    if let Some(mode) = link_meta.permissions {
        use std::os::unix::fs::PermissionsExt;
        let _ = tokio::fs::set_permissions(local, std::fs::Permissions::from_mode(mode)).await;
    }

    Ok(TransferResult {
        bytes_transferred: bytes,
        duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

/// Recursively uploads a local directory tree (no symlink follow).
pub async fn upload_tree(
    sftp: &SftpSession,
    local_dir: &Path,
    remote_dir: &str,
) -> SshCliResult<TransferResult> {
    validate_remote_path(remote_dir)?;
    let start = Instant::now();
    let mut bytes = 0_u64;
    upload_tree_rec(sftp, local_dir, remote_dir, 0, &mut bytes).await?;
    Ok(TransferResult {
        bytes_transferred: bytes,
        duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

async fn upload_tree_rec(
    sftp: &SftpSession,
    local_dir: &Path,
    remote_dir: &str,
    depth: u32,
    bytes: &mut u64,
) -> SshCliResult<()> {
    check_depth(depth)?;
    // G-SFTP-R06: no-follow at every level (incl. root).
    let meta = tokio::fs::symlink_metadata(local_dir).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            SshCliError::FileNotFound(local_dir.display().to_string())
        } else {
            SshCliError::Io(e)
        }
    })?;
    if meta.file_type().is_symlink() {
        return Err(SshCliError::InvalidArgument(format!(
            "sftp upload tree refuses local symlink (no-follow): {}",
            local_dir.display()
        )));
    }
    if !meta.is_dir() {
        return Err(SshCliError::InvalidArgument(
            "sftp --recursive upload source must be a directory".into(),
        ));
    }

    // Ensure remote dir exists (ignore already-exists style errors by try_exists).
    if !sftp
        .try_exists(remote_dir.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote_dir, e))?
    {
        sftp.create_dir(remote_dir.to_owned())
            .await
            .map_err(|e| map_sftp_err(remote_dir, e))?;
    }

    let mut rd = tokio::fs::read_dir(local_dir)
        .await
        .map_err(SshCliError::Io)?;
    while let Some(entry) = rd.next_entry().await.map_err(SshCliError::Io)? {
        if crate::signals::should_stop() {
            return Err(SshCliError::InvalidArgument(
                crate::i18n::t(crate::i18n::Message::OperationCancelled),
            ));
        }
        let ft = entry.file_type().await.map_err(SshCliError::Io)?;
        if ft.is_symlink() {
            tracing::debug!(
                path = %entry.path().display(),
                "sftp upload tree: skipping local symlink"
            );
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // G-SFTP-R07: fail-closed on hostile/odd basenames.
        validate_entry_name(&name_str)?;
        let remote_child = join_remote(remote_dir, &name_str);
        let local_child = entry.path();
        if ft.is_dir() {
            Box::pin(upload_tree_rec(
                sftp,
                &local_child,
                &remote_child,
                depth + 1,
                bytes,
            ))
            .await?;
        } else if ft.is_file() {
            let r = upload_file(sftp, &local_child, &remote_child).await?;
            *bytes = bytes.saturating_add(r.bytes_transferred);
        }
    }
    Ok(())
}

/// Recursively downloads a remote directory tree (no symlink follow).
///
/// Entry basenames are untrusted (G-SFTP-R01/R02): validated and constrained
/// under `local_dir` via [`ensure_local_under`].
pub async fn download_tree(
    sftp: &SftpSession,
    remote_dir: &str,
    local_dir: &Path,
) -> SshCliResult<TransferResult> {
    validate_remote_path(remote_dir)?;
    let start = Instant::now();
    let mut bytes = 0_u64;
    let local_root = local_dir.to_path_buf();
    download_tree_rec(sftp, remote_dir, local_dir, &local_root, 0, &mut bytes).await?;
    Ok(TransferResult {
        bytes_transferred: bytes,
        duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

async fn download_tree_rec(
    sftp: &SftpSession,
    remote_dir: &str,
    local_dir: &Path,
    local_root: &Path,
    depth: u32,
    bytes: &mut u64,
) -> SshCliResult<()> {
    check_depth(depth)?;
    let meta = sftp
        .symlink_metadata(remote_dir.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote_dir, e))?;
    if meta.file_type().is_symlink() {
        tracing::debug!(path = %remote_dir, "sftp download tree: skipping remote symlink");
        return Ok(());
    }
    if !meta.file_type().is_dir() {
        return Err(SshCliError::InvalidArgument(
            "sftp --recursive download source must be a directory".into(),
        ));
    }

    ensure_local_under(local_root, local_dir)?;
    tokio::fs::create_dir_all(local_dir)
        .await
        .map_err(SshCliError::Io)?;

    let entries = sftp
        .read_dir(remote_dir.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote_dir, e))?;
    let mut count = 0_usize;
    for entry in entries {
        count = count.saturating_add(1);
        if count > SFTP_LIST_MAX_ENTRIES {
            return Err(SshCliError::InvalidArgument(format!(
                "sftp directory listing exceeds max entries ({SFTP_LIST_MAX_ENTRIES})"
            )));
        }
        if crate::signals::should_stop() {
            return Err(SshCliError::InvalidArgument(
                crate::i18n::t(crate::i18n::Message::OperationCancelled),
            ));
        }
        let name = entry.file_name();
        // G-SFTP-R01: malicious server basenames must not escape local_root.
        validate_entry_name(&name)?;
        let remote_child = entry.path();
        let local_child: PathBuf = local_dir.join(&name);
        ensure_local_under(local_root, &local_child)?;
        let ft = entry.file_type();
        if ft.is_symlink() {
            tracing::debug!(path = %remote_child, "sftp download tree: skipping remote symlink");
            continue;
        }
        if ft.is_dir() {
            Box::pin(download_tree_rec(
                sftp,
                &remote_child,
                &local_child,
                local_root,
                depth + 1,
                bytes,
            ))
            .await?;
        } else if ft.is_file() {
            let r = download_file(sftp, &remote_child, &local_child).await?;
            *bytes = bytes.saturating_add(r.bytes_transferred);
        }
    }
    Ok(())
}

/// Lists a remote directory (non-recursive).
pub async fn list_dir(sftp: &SftpSession, remote: &str) -> SshCliResult<Vec<SftpListEntry>> {
    validate_remote_path(remote)?;
    let entries = sftp
        .read_dir(remote.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote, e))?;
    let mut out = Vec::new();
    for entry in entries {
        if out.len() >= SFTP_LIST_MAX_ENTRIES {
            return Err(SshCliError::InvalidArgument(format!(
                "sftp ls exceeds max entries ({SFTP_LIST_MAX_ENTRIES})"
            )));
        }
        let meta = entry.metadata();
        out.push(SftpListEntry {
            name: entry.file_name(),
            path: entry.path(),
            kind: kind_from_attrs(&meta),
            size: meta.size,
            mode: meta.permissions,
        });
    }
    Ok(out)
}

/// Creates a remote directory.
pub async fn mkdir(sftp: &SftpSession, remote: &str) -> SshCliResult<()> {
    validate_remote_path(remote)?;
    sftp.create_dir(remote.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote, e))
}

/// Removes an empty remote directory.
pub async fn rmdir(sftp: &SftpSession, remote: &str) -> SshCliResult<()> {
    validate_remote_path(remote)?;
    sftp.remove_dir(remote.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote, e))
}

/// Removes a remote file.
pub async fn rm(sftp: &SftpSession, remote: &str) -> SshCliResult<()> {
    validate_remote_path(remote)?;
    sftp.remove_file(remote.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote, e))
}

/// Stats a remote path (follows for metadata; reports symlink via symlink_metadata).
pub async fn stat(sftp: &SftpSession, remote: &str) -> SshCliResult<SftpStat> {
    validate_remote_path(remote)?;
    let meta = sftp
        .symlink_metadata(remote.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote, e))?;
    Ok(SftpStat {
        path: remote.to_owned(),
        kind: kind_from_attrs(&meta),
        size: meta.size,
        mode: meta.permissions,
        mtime: meta.mtime,
    })
}

/// Renames a remote path.
pub async fn rename(sftp: &SftpSession, from: &str, to: &str) -> SshCliResult<()> {
    validate_remote_path(from)?;
    validate_remote_path(to)?;
    sftp.rename(from.to_owned(), to.to_owned())
        .await
        .map_err(|e| map_sftp_err(from, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_secs_ceil() {
        assert_eq!(sftp_timeout_secs(1), 1);
        assert_eq!(sftp_timeout_secs(1000), 1);
        assert_eq!(sftp_timeout_secs(1001), 2);
        assert_eq!(sftp_timeout_secs(30_000), 30);
    }
}
