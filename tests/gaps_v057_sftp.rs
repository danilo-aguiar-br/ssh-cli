// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gate: G-SFTP (russh-sftp, CLI surface, path safety, agent transfer parity).
//! Local only — no product GH Actions / no OTEL.

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    fs::read_to_string(workspace_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn exists(rel: &str) -> bool {
    workspace_root().join(rel).is_file()
}

#[test]
fn g_sftp_01_cargo_has_russh_sftp_feature() {
    let cargo = read("Cargo.toml");
    assert!(
        cargo.contains("russh-sftp") && cargo.contains("dep:russh-sftp"),
        "G-SFTP-01: russh-sftp optional dep + ssh-real feature wire required"
    );
}

#[test]
fn g_sftp_02_open_subsystem_and_session() {
    let s = read("src/ssh/sftp_session.rs");
    assert!(
        s.contains("request_subsystem") && s.contains("SFTP_SUBSYSTEM"),
        "G-SFTP-02: request_subsystem(sftp) required"
    );
    assert!(
        s.contains("SftpSession::new") && s.contains("into_stream"),
        "G-SFTP-02: SftpSession::new(into_stream) required"
    );
    assert!(
        s.contains("forbid(unsafe_code)"),
        "sftp_session must forbid unsafe"
    );
}

#[test]
fn g_sftp_11_no_full_buffer_bulk_read_write() {
    let s = read("src/ssh/sftp_session.rs");
    // Bulk path must stream via File AsyncRead/Write — not SftpSession::read/write whole file.
    assert!(
        !s.contains("sftp.read(") && !s.contains(".read(remote") && !s.contains(".read(path"),
        "G-SFTP-11: must not call SftpSession::read for bulk"
    );
    // Allow comments mentioning write; forbid path-level sftp.write( bulk API
    assert!(
        !s.contains("sftp.write(") && !s.contains(".write(path"),
        "G-SFTP-11: must not call SftpSession::write for bulk"
    );
    assert!(
        s.contains("SFTP_IO_CHUNK") || s.contains("write_all"),
        "G-SFTP-11: stream chunk path required"
    );
}

#[test]
fn g_sftp_07_cli_sftp_action() {
    let args = read("src/cli/sftp_args.rs");
    assert!(
        args.contains("enum SftpAction")
            && args.contains("Upload")
            && args.contains("Download")
            && args.contains("Ls")
            && args.contains("Mkdir")
            && args.contains("Rename"),
        "G-SFTP-07: SftpAction surface required"
    );
    let cli = read("src/cli/mod.rs");
    assert!(
        cli.contains("Command::Sftp") || cli.contains("Sftp {"),
        "G-SFTP-07: Command::Sftp required"
    );
}

#[test]
fn g_sftp_17_18_agent_on_transfer_options() {
    let scp = read("src/scp/mod.rs");
    assert!(
        scp.contains("use_agent") && scp.contains("agent_socket"),
        "G-SFTP-17: ScpOptions agent fields required"
    );
    let sftp = read("src/sftp/mod.rs");
    assert!(
        sftp.contains("use_agent") && sftp.contains("agent_socket"),
        "G-SFTP-18: SftpOptions agent fields required"
    );
    let dispatch = read("src/cli/dispatch.rs");
    assert!(
        dispatch.contains("Command::Sftp") && dispatch.contains("use_agent"),
        "dispatch must wire agent for sftp/scp"
    );
}

#[test]
fn g_sftp_06_16_path_safety_module() {
    let p = read("src/ssh/sftp_path.rs");
    assert!(
        p.contains("validate_remote_path") && p.contains("check_depth"),
        "G-SFTP-06/16: path safety required"
    );
    assert!(
        p.contains("validate_entry_name") && p.contains("ensure_local_under"),
        "G-SFTP-R01/R02: entry name + local root guard required"
    );
    assert!(p.contains("forbid(unsafe_code)"));
}

#[test]
fn g_sftp_r01_tree_uses_entry_name_and_local_under() {
    let s = read("src/ssh/sftp_session.rs");
    assert!(
        s.contains("validate_entry_name") && s.contains("ensure_local_under"),
        "G-SFTP-R01/R02: download tree must validate entry names and local root"
    );
    assert!(
        s.contains("under_timeout"),
        "G-SFTP-R05: under_timeout helper required"
    );
    // Partial cleanup on any download error (not only cancel).
    assert!(
        s.contains("remove_file(&partial)") || s.contains("remove_file(&partial)"),
        "G-SFTP-R04: partial cleanup path required"
    );
}

#[test]
fn g_sftp_r05_multi_file_and_fs_timeout() {
    let m = read("src/sftp/mod.rs");
    assert!(
        m.contains("under_timeout"),
        "G-SFTP-R05: sftp/mod multi-file/FS must use under_timeout"
    );
    let b = read("src/sftp/batch.rs");
    assert!(
        b.contains("under_timeout") && b.contains("validate_entry_name"),
        "G-SFTP-R05/R03: batch multi-file safety + timeout"
    );
}

#[test]
fn g_sftp_r12_scp_args_split() {
    assert!(
        exists("src/cli/scp_args.rs"),
        "G-SFTP-R12: cli/scp_args.rs must exist (SRP monólito split)"
    );
    let scp = read("src/cli/scp_args.rs");
    assert!(
        scp.contains("enum ScpAction"),
        "G-SFTP-R12: ScpAction lives in scp_args"
    );
}

#[test]
fn g_sftp_r13_fallback_basename_constant() {
    let c = read("src/constants.rs");
    assert!(
        c.contains("SFTP_FALLBACK_BASENAME"),
        "G-SFTP-R13: named fallback basename constant"
    );
    let m = read("src/sftp/mod.rs");
    assert!(
        m.contains("SFTP_FALLBACK_BASENAME"),
        "multi-file must use SFTP_FALLBACK_BASENAME not bare \"file\""
    );
    assert!(
        !m.contains("\"file\".into()"),
        "hardcoded fallback \"file\" forbidden in sftp/mod"
    );
}

#[test]
fn g_sftp_r06_upload_tree_no_follow_root() {
    let s = read("src/ssh/sftp_session.rs");
    // upload_tree_rec must use symlink_metadata (not bare metadata) for no-follow.
    assert!(
        s.contains("symlink_metadata(local_dir)")
            || s.contains("symlink_metadata(local_dir)")
            || s.matches("symlink_metadata").count() >= 3,
        "G-SFTP-R06: upload tree root must use symlink_metadata (no-follow)"
    );
}

#[test]
fn g_sftp_09_schemas_exist() {
    for rel in [
        "docs/schemas/sftp-transfer.schema.json",
        "docs/schemas/sftp-list.schema.json",
        "docs/schemas/sftp-fs-op.schema.json",
        "docs/schemas/sftp-batch.schema.json",
    ] {
        assert!(exists(rel), "missing schema {rel}");
        let body = read(rel);
        assert!(body.contains("sftp"), "{rel} must mention sftp");
    }
}

#[test]
fn g_sftp_10_set_timeout_aligned() {
    let s = read("src/ssh/sftp_session.rs");
    assert!(
        s.contains("set_timeout") && s.contains("sftp_timeout_secs"),
        "G-SFTP-10: set_timeout from product timeout_ms required"
    );
}

#[test]
fn g_sftp_12_not_inline_in_client_real() {
    let facade = read("src/ssh/client_real.rs");
    let scp = read("src/ssh/client_real_scp.rs");
    let sftp = read("src/ssh/client_real_sftp.rs");
    let core = read("src/ssh/client_real_core.rs");
    let impl_body = format!("{scp}{sftp}{core}");
    assert!(
        !facade.contains("request_subsystem") && !facade.contains("SftpSession::new"),
        "G-SFTP-12: SFTP wire must live in sftp_session, not client_real façade"
    );
    assert!(
        impl_body.contains("sftp_session") || impl_body.contains("open_sftp") || facade.contains("include!"),
        "client_real may thin-wrap open_sftp"
    );
}

#[test]
fn g_sftp_15_gaps_inventory() {
    let gaps = read("gaps.md");
    assert!(
        gaps.contains("G-SFTP") && gaps.contains("residual"),
        "G-SFTP-15: gaps.md must inventory G-SFTP"
    );
}

#[test]
fn g_sftp_constants() {
    let c = read("src/constants.rs");
    assert!(
        c.contains("SFTP_SUBSYSTEM")
            && c.contains("SFTP_IO_CHUNK")
            && c.contains("SFTP_MAX_RECURSION_DEPTH"),
        "named SFTP constants required"
    );
}
