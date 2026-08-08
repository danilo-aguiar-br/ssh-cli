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

use super::client::TransferResult;
use super::client_handler::ClientHandler;
use super::scp_wire::partial_download_path;
use super::sftp_path::{
    check_depth, ensure_local_under, join_remote, validate_entry_name, validate_remote_path,
};
use super::sftp_types::{SftpListEntry, SftpStat};
use crate::constants::{
    SFTP_IO_CHUNK, SFTP_LIST_MAX_ENTRIES, SFTP_PERM_MASK, SFTP_PERM_MASK_UNTRUSTED, SFTP_SUBSYSTEM,
};
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

/// Rejects a directory listing that grew past [`SFTP_LIST_MAX_ENTRIES`].
///
/// `seen` is the number of entries consumed **so far**, so the caller stops before
/// allocating entry `SFTP_LIST_MAX_ENTRIES + 1` instead of after. A hostile server
/// picks the entry count, so this is the only bound between it and our heap.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] once `seen` exceeds the cap.
fn check_list_cap(seen: usize, op: &str) -> SshCliResult<()> {
    if seen > SFTP_LIST_MAX_ENTRIES {
        return Err(SshCliError::InvalidArgument(format!(
            "sftp {op} exceeds max entries ({SFTP_LIST_MAX_ENTRIES})"
        )));
    }
    Ok(())
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
        .map_err(|e| {
            SshCliError::channel_msg(format!("request subsystem {SFTP_SUBSYSTEM}: {e}"))
        })?;
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
    // Build attrs from `empty()` only (G1/G3/G12): never `Default` (directory template).
    let attrs = upload_file_attrs(&meta);

    let mut remote_file = sftp
        .open_with_flags_and_attributes(remote.to_owned(), flags, attrs.clone())
        .await
        .map_err(|e| map_sftp_err(remote, e))?;

    let mut buf = vec![0_u8; SFTP_IO_CHUNK];
    let mut bytes = 0_u64;
    loop {
        if crate::signals::should_stop() {
            return Err(SshCliError::InvalidArgument(crate::i18n::t(
                crate::i18n::Message::OperationCancelled,
            )));
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

    // G4: SETSTAT is a mutating op — never discard its Result (was the silent
    // path that made G1 undetectable). Failure aborts the transfer.
    sftp.set_metadata(remote.to_owned(), attrs)
        .await
        .map_err(|e| map_sftp_err(remote, e))?;

    // B2: SCP verified byte counts in both directions while SFTP verified in
    // neither, so a truncated SFTP upload reported `ok` with a plausible
    // `bytes` field — the transfer looked complete because the loop counted what
    // it *wrote*, never what the server *kept*.
    verify_local_read(local, meta.len(), bytes)?;
    verify_remote_size(sftp, remote, bytes).await?;

    Ok(TransferResult {
        bytes_transferred: bytes,
        duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
        ..Default::default()
    })
}

/// Fails when the local file shrank between `symlink_metadata` and the read loop.
///
/// Mirrors the SCP upload guard: the announced size is the contract the caller
/// reports, so silently sending fewer bytes turns a truncation into a success.
///
/// # Errors
/// [`SshCliError::ChannelFailed`] when `read` differs from `announced`.
fn verify_local_read(local: &Path, announced: u64, read: u64) -> SshCliResult<()> {
    if read == announced {
        return Ok(());
    }
    Err(SshCliError::channel_msg(format!(
        "local file changed during SFTP upload of {}: announced {announced} bytes, read {read}",
        local.display()
    )))
}

/// Fails when a download received a different byte count than the server announced.
///
/// A server that omits `size` from its `stat` cannot be checked, and that case is
/// accepted rather than turned into a false failure — asserting on an attribute
/// the protocol makes optional would break valid servers to catch nothing.
///
/// # Errors
/// [`SshCliError::ChannelFailed`] when a declared size differs from `received`.
fn verify_received(remote: &str, announced: Option<u64>, received: u64) -> SshCliResult<()> {
    match announced {
        Some(size) if size != received => Err(SshCliError::channel_msg(format!(
            "truncated SFTP download: {remote} announced {size} bytes, received {received}"
        ))),
        _ => Ok(()),
    }
}

/// Confirms the server persisted exactly the bytes that were streamed.
///
/// This is the destination-effect proof: counting bytes handed to `write_all`
/// only proves what the client attempted. A server that silently applied a quota,
/// a full filesystem, or a truncating `SETSTAT` is invisible without a re-`stat`.
/// A server that omits `size` from the reply cannot be checked, and is accepted
/// rather than turned into a false failure.
///
/// # Errors
/// [`SshCliError::ChannelFailed`] when the remote size is present and differs.
async fn verify_remote_size(sftp: &SftpSession, remote: &str, sent: u64) -> SshCliResult<()> {
    let attrs = sftp
        .metadata(remote.to_owned())
        .await
        .map_err(|e| map_sftp_err(remote, e))?;
    match attrs.size {
        Some(size) if size != sent => Err(SshCliError::channel_msg(format!(
            "truncated SFTP upload: sent {sent} bytes, server holds {size} for {remote}"
        ))),
        _ => Ok(()),
    }
}

/// Builds wire `FileAttributes` for an SFTP upload from local metadata (G1/G3/G12/G19).
///
/// Uses [`FileAttributes::empty`] (neutral) — never [`Default`] (directory template
/// with `size: Some(0)` that truncates after CLOSE).
fn upload_file_attrs(meta: &std::fs::Metadata) -> FileAttributes {
    let mut attrs = FileAttributes::empty();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Mask S_IFMT: `mode()` carries file-type bits; protocol wants perms only.
        attrs.permissions = Some(meta.permissions().mode() & SFTP_PERM_MASK);
    }
    // SFTP v3 ACMODTIME is atomic: missing atime serializes as 0 (epoch). Always
    // pair atime+mtime; fall back to mtime when local atime is unreadable.
    if let Ok(mtime) = meta.modified() {
        if let Ok(d) = mtime.duration_since(std::time::UNIX_EPOCH) {
            let mtime_secs = u32::try_from(d.as_secs()).unwrap_or(u32::MAX);
            attrs.mtime = Some(mtime_secs);
            attrs.atime = Some(
                meta.accessed()
                    .ok()
                    .and_then(|a| a.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(mtime_secs, |a| {
                        u32::try_from(a.as_secs()).unwrap_or(u32::MAX)
                    }),
            );
        }
    }
    attrs
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
    let result =
        download_file_to_partial(&mut remote_file, &partial, local, remote, &link_meta, start)
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
            return Err(SshCliError::InvalidArgument(crate::i18n::t(
                crate::i18n::Message::OperationCancelled,
            )));
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
    // B2: verified *before* the atomic rename, so a short read never reaches the
    // final path. Checking after the rename would leave a truncated file in place
    // with an error beside it — the worst of both outcomes, since a retry would
    // then have to distinguish "absent" from "present but wrong".
    verify_received(remote, link_meta.size, bytes)?;

    local_file.flush().await.map_err(SshCliError::Io)?;
    // Durability barrier before atomic rename (parity with SCP G9).
    local_file.sync_data().await.map_err(SshCliError::Io)?;
    drop(local_file);

    tokio::fs::rename(partial, local)
        .await
        .map_err(SshCliError::Io)?;

    // G18: local mode is a mutation — surface failure (do not silent-ok).
    //
    // A3: this mode comes from the *server*, so it is untrusted. Clamping with
    // `SFTP_PERM_MASK` (0o7777) would let a hostile server set setuid/setgid on the
    // file we just wrote to local disk. Inbound modes use the untrusted mask, which
    // drops every elevation bit.
    #[cfg(unix)]
    if let Some(mode) = link_meta.permissions {
        use std::os::unix::fs::PermissionsExt;
        let mode_bits = mode & SFTP_PERM_MASK_UNTRUSTED;
        tokio::fs::set_permissions(local, std::fs::Permissions::from_mode(mode_bits))
            .await
            .map_err(|e| {
                SshCliError::Io(std::io::Error::new(
                    e.kind(),
                    format!("sftp download set_permissions {}: {e}", local.display()),
                ))
            })?;
    }

    Ok(TransferResult {
        bytes_transferred: bytes,
        duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
        ..Default::default()
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
        ..Default::default()
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

    // A4: `try_exists` resolves through `stat`, which *follows* symlinks — a remote
    // symlink pointing at an unrelated directory would report "exists" and the whole
    // subtree would land wherever the link aims. The module contract is no-follow, so
    // probe with `symlink_metadata` and refuse a symlinked destination outright.
    match sftp.symlink_metadata(remote_dir.to_owned()).await {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                return Err(SshCliError::InvalidArgument(format!(
                    "sftp upload tree refuses remote symlink destination (no-follow): {remote_dir}"
                )));
            }
            if !meta.file_type().is_dir() {
                return Err(SshCliError::InvalidArgument(format!(
                    "sftp upload tree destination exists and is not a directory: {remote_dir}"
                )));
            }
        }
        Err(russh_sftp::client::error::Error::Status(st))
            if st.status_code == StatusCode::NoSuchFile =>
        {
            sftp.create_dir(remote_dir.to_owned())
                .await
                .map_err(|e| map_sftp_err(remote_dir, e))?;
        }
        Err(e) => return Err(map_sftp_err(remote_dir, e)),
    }

    let mut rd = tokio::fs::read_dir(local_dir)
        .await
        .map_err(SshCliError::Io)?;
    while let Some(entry) = rd.next_entry().await.map_err(SshCliError::Io)? {
        if crate::signals::should_stop() {
            return Err(SshCliError::InvalidArgument(crate::i18n::t(
                crate::i18n::Message::OperationCancelled,
            )));
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
        ..Default::default()
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
        check_list_cap(count, "directory listing")?;
        if crate::signals::should_stop() {
            return Err(SshCliError::InvalidArgument(crate::i18n::t(
                crate::i18n::Message::OperationCancelled,
            )));
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
    // Bound our own copy as we consume, so a server that answers with millions of
    // names cannot make us build a second full-size Vec on top of the wire response.
    let mut out = Vec::new();
    for entry in entries {
        check_list_cap(out.len().saturating_add(1), "ls")?;
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

    /// B2: SCP verified byte counts in both directions while SFTP verified in
    /// neither, so a truncated transfer reported `ok` with a plausible `bytes`
    /// field — the loop counted what it *wrote*, never what survived.
    #[test]
    fn upload_rejects_a_file_that_shrank_mid_read() {
        let err = verify_local_read(Path::new("/tmp/x"), 4096, 1024)
            .expect_err("a short read must not report success");
        let text = err.to_string();
        assert!(text.contains("4096"), "{text}");
        assert!(text.contains("1024"), "{text}");
    }

    #[test]
    fn upload_accepts_an_exact_read() {
        assert!(verify_local_read(Path::new("/tmp/x"), 4096, 4096).is_ok());
    }

    #[test]
    fn upload_accepts_an_empty_file() {
        // Zero-length uploads are legitimate; treating 0 as "nothing happened"
        // would fail every empty-file transfer.
        assert!(verify_local_read(Path::new("/tmp/x"), 0, 0).is_ok());
    }

    #[test]
    fn download_rejects_a_short_read() {
        let err = verify_received("/srv/db.dump", Some(9_000), 8_192)
            .expect_err("a truncated download must not be renamed into place");
        assert!(err.to_string().contains("9000"), "{err}");
    }

    #[test]
    fn download_accepts_an_exact_read() {
        assert!(verify_received("/srv/db.dump", Some(8_192), 8_192).is_ok());
    }

    #[test]
    fn download_accepts_a_server_that_omits_the_size() {
        // `size` is optional in the SFTP attribute set. Asserting on an attribute
        // the protocol does not require would break valid servers to catch nothing.
        assert!(verify_received("/srv/db.dump", None, 8_192).is_ok());
    }

    /// B9: the cap must trip on the entry that crosses it, while it is still being
    /// consumed — not after the whole listing has been copied into our own Vec.
    #[test]
    fn list_cap_trips_on_first_entry_past_limit() {
        assert!(check_list_cap(0, "ls").is_ok());
        assert!(check_list_cap(1, "ls").is_ok());
        assert!(check_list_cap(SFTP_LIST_MAX_ENTRIES, "ls").is_ok());
        let err = check_list_cap(SFTP_LIST_MAX_ENTRIES + 1, "ls").unwrap_err();
        assert!(matches!(err, SshCliError::InvalidArgument(_)));
        assert!(err.to_string().contains(&SFTP_LIST_MAX_ENTRIES.to_string()));
    }

    /// The op label reaches the message so `ls` and tree walks are distinguishable.
    #[test]
    fn list_cap_message_carries_op() {
        let err = check_list_cap(SFTP_LIST_MAX_ENTRIES + 1, "directory listing").unwrap_err();
        assert!(err.to_string().contains("directory listing"));
    }

    #[test]
    fn timeout_secs_ceil() {
        assert_eq!(sftp_timeout_secs(1), 1);
        assert_eq!(sftp_timeout_secs(1000), 1);
        assert_eq!(sftp_timeout_secs(1001), 2);
        assert_eq!(sftp_timeout_secs(30_000), 30);
    }

    /// Tripwire for russh-sftp #89: `Default` is a dummy NEW-DIRECTORY template,
    /// not a neutral attribute set. Uploading with it makes the post-CLOSE
    /// SETSTAT carry `size = 0`, truncating every uploaded file to zero bytes
    /// while every request still answers `Status::Ok`. Upload paths MUST build
    /// from `empty()`. If a dependency bump ever changes either invariant, this
    /// fails loudly instead of silently destroying user data again.
    #[test]
    fn upstream_default_attributes_are_destructive_use_empty() {
        let dummy = FileAttributes::default();
        assert_eq!(
            dummy.size,
            Some(0),
            "upstream Default no longer carries size=0; revisit the upload attrs"
        );
        let neutral = FileAttributes::empty();
        assert!(
            neutral.size.is_none(),
            "empty() must omit size (no truncate)"
        );
        assert!(neutral.uid.is_none(), "empty() must omit uid (no chown)");
        assert!(neutral.gid.is_none(), "empty() must omit gid (no chown)");
    }
}
