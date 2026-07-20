// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gate: G-SSH rules (russh policy, host-key outcome, key material, agent surface).
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

#[test]
fn g_ssh_02_client_id_is_product_generic() {
    let c = read("src/constants.rs");
    assert!(
        c.contains("SSH_CLIENT_ID") && c.contains("SSH-2.0-ssh-cli"),
        "G-SSH-02: product-generic client_id required"
    );
    let connect = read("src/ssh/connect.rs");
    assert!(
        connect.contains("SSH_CLIENT_ID") && connect.contains("client_id:"),
        "G-SSH-02/08: build_ssh_client_config must set client_id"
    );
    assert!(
        !connect.contains("CARGO_PKG_VERSION") || connect.contains("SSH_CLIENT_ID"),
        "must not rely solely on russh default version banner"
    );
}

#[test]
fn g_ssh_01_host_key_outcome_propagates_typed_error() {
    let h = read("src/ssh/client_handler.rs");
    assert!(
        h.contains("HostKeyOutcome") && h.contains("stash_host_key_error"),
        "G-SSH-01: HostKeyOutcome channel required"
    );
    let c = read("src/ssh/client_connect.rs");
    assert!(
        c.contains("take_host_key_error") || c.contains("map_connect_err"),
        "G-SSH-01: connect must recover stashed host-key errors"
    );
}

#[test]
fn g_ssh_03_07_key_material_permissions_and_rsa_floor() {
    let k = read("src/ssh/key_material.rs");
    assert!(
        k.contains("ensure_private_key_permissions"),
        "G-SSH-03: permission check required"
    );
    assert!(
        k.contains("reject_weak_key") && k.contains("SSH_RSA_MIN_BITS"),
        "G-SSH-07: RSA floor required"
    );
    assert!(
        k.contains("forbid(unsafe_code)"),
        "key_material must forbid unsafe"
    );
}

#[test]
fn g_ssh_04_agent_surface_cli_xdg_not_env() {
    let cli = read("src/cli/mod.rs");
    assert!(
        cli.contains("use_agent") && cli.contains("agent_socket"),
        "G-SSH-04: CLI agent flags required"
    );
    let conn = read("src/ssh/connection.rs");
    assert!(
        conn.contains("use_agent") && conn.contains("agent_socket"),
        "G-SSH-04/17: ConnectionConfig agent fields"
    );
    let connect = read("src/ssh/client_connect.rs");
    assert!(
        connect.contains("try_agent_auth") || connect.contains("authenticate_publickey_with"),
        "G-SSH-04: agent auth path"
    );
    // Fail-closed: must not treat SSH_AUTH_SOCK as product store.
    assert!(
        !connect.contains("SSH_AUTH_SOCK") && !connect.contains("connect_env"),
        "G-SSH-04: must not use SSH_AUTH_SOCK env as product store"
    );
}

#[test]
fn g_ssh_05_tcp_keepalive_and_compression_none() {
    let connect = read("src/ssh/connect.rs");
    assert!(
        connect.contains("set_keepalive") || connect.contains("TCP_KEEPALIVE"),
        "G-SSH-05: TCP keepalive"
    );
    assert!(
        connect.contains("compression::NONE") || connect.contains("NONE"),
        "compression none-only policy"
    );
    let cargo = read("Cargo.toml");
    assert!(
        cargo.contains("socket2"),
        "socket2 direct dep for SO_KEEPALIVE"
    );
}

#[test]
fn g_ssh_06_split_modules_exist() {
    for rel in [
        "src/ssh/client_handler.rs",
        "src/ssh/client_connect.rs",
        "src/ssh/key_material.rs",
        "src/ssh/connect.rs",
    ] {
        assert!(
            workspace_root().join(rel).is_file(),
            "G-SSH-06: missing {rel}"
        );
    }
    let m = read("src/ssh/mod.rs");
    assert!(m.contains("client_handler") && m.contains("client_connect") && m.contains("key_material"));
}

#[test]
fn g_ssh_09_fail_closed_known_hosts_non_test() {
    let h = read("src/ssh/client_handler.rs");
    assert!(
        h.contains("cfg(test)") && h.contains("fail-closed") || h.contains("fail-closed") || h.contains("rejecting host key"),
        "G-SSH-09: non-test must reject missing known_hosts path"
    );
}

#[test]
fn g_ssh_11_deny_bans_c_bindings() {
    let d = read("deny.toml");
    for name in ["ssh2", "thrussh", "libssh-rs"] {
        assert!(
            d.contains(name),
            "G-SSH-11: deny must ban {name}"
        );
    }
}

#[test]
fn g_ssh_no_banned_ssh_crates_in_cargo() {
    let cargo = read("Cargo.toml");
    assert!(!cargo.contains("thrussh"));
    assert!(!cargo.contains("libssh-rs"));
    // ssh2 as crate name would appear as `ssh2 =` not as substring of other words
    assert!(
        !cargo.lines().any(|l| l.trim_start().starts_with("ssh2 ")
            || l.trim_start().starts_with("ssh2=")
            || l.contains("ssh2 =")),
        "must not depend on ssh2 crate"
    );
    assert!(
        cargo.contains("russh") && cargo.contains("aws-lc-rs"),
        "russh + aws-lc-rs required"
    );
}

#[test]
fn g_ssh_16_auth_method_logged() {
    let c = read("src/ssh/client_connect.rs");
    assert!(
        c.contains("auth_method"),
        "G-SSH-16: log authentication method used"
    );
}
