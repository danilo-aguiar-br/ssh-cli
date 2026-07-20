// SPDX-License-Identifier: MIT OR Apache-2.0
//! G-TLS residual suite — transport/crypto policy with product rustls stack.
//!
//! Product network security:
//! - **SSH-2** (`russh` + `aws-lc-rs`) on plain TCP by default
//! - **SSH-over-TLS** optional via feature `tls` (`rustls` ≥ 0.23.18 + `aws_lc_rs`)
//!
//! These tests lock the supply-chain bans so OpenSSL / `native-tls` / dual
//! provider `ring` cannot re-enter the default product graph.

#![forbid(unsafe_code)]

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// G-TLS-03: lockfile must not contain forbidden OpenSSL / native-tls crates.
#[test]
fn lockfile_forbids_openssl_and_native_tls() {
    let lock = read("Cargo.lock");
    for bad in [
        "name = \"native-tls\"",
        "name = \"openssl\"",
        "name = \"openssl-sys\"",
        "name = \"libssh2-sys\"",
    ] {
        assert!(
            !lock.contains(bad),
            "G-TLS-03: forbidden crate present in Cargo.lock: {bad}"
        );
    }
}

/// G-TLS product: rustls is present and version floor ≥ 0.23.18.
#[test]
fn lockfile_has_rustls_floor() {
    let lock = read("Cargo.lock");
    assert!(
        lock.contains("name = \"rustls\""),
        "G-TLS: rustls must be in Cargo.lock when feature tls is default"
    );
    // Extract first rustls version line after package name.
    let mut found = false;
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "name = \"rustls\"" {
            if let Some(ver) = lines.next() {
                // version = "0.23.xx"
                let v = ver
                    .split('"')
                    .nth(1)
                    .expect("version string");
                let parts: Vec<_> = v.split('.').collect();
                assert!(parts.len() >= 3, "unexpected rustls version {v}");
                let major: u32 = parts[0].parse().unwrap();
                let minor: u32 = parts[1].parse().unwrap();
                let patch: u32 = parts[2]
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();
                assert!(
                    major > 0 || minor > 23 || (minor == 23 && patch >= 18),
                    "rustls {v} below floor 0.23.18"
                );
                // Explicitly ban Acceptor CVE window when on 0.23.x
                if major == 0 && minor == 23 {
                    assert!(
                        !(13..=17).contains(&patch),
                        "rustls {v} in CVE Acceptor window 0.23.13–0.23.17"
                    );
                }
                found = true;
                break;
            }
        }
    }
    assert!(found, "could not parse rustls version from Cargo.lock");
}

/// G-TLS-02/03: deny.toml must ban OpenSSL stacks and dual provider `ring`.
#[test]
fn deny_toml_lists_tls_bans() {
    let deny = read("deny.toml");
    for name in [
        "openssl-sys",
        "openssl",
        "native-tls",
        "libssh2-sys",
        "ring",
    ] {
        assert!(
            deny.contains(name),
            "G-TLS-02: deny.toml missing ban for {name}"
        );
    }
    // rustls is allowed (product feature); ban list must not still name it.
    let bans_section = deny
        .split("[bans]")
        .nth(1)
        .and_then(|s| s.split('[').next())
        .unwrap_or("");
    assert!(
        !bans_section.contains("\"rustls\"") && !bans_section.contains("name = \"rustls\""),
        "G-TLS: deny.toml must not ban product rustls"
    );
}

/// G-TLS-05: product russh features must not enable flate2 (compression stack).
#[test]
fn cargo_toml_russh_without_flate2() {
    let cargo = read("Cargo.toml");
    let russh_line = cargo
        .lines()
        .find(|l| l.trim_start().starts_with("russh ="))
        .expect("russh dependency line");
    assert!(
        !russh_line.contains("flate2"),
        "G-TLS-05: russh features still enable flate2: {russh_line}"
    );
    assert!(
        russh_line.contains("aws-lc-rs"),
        "G-TLS-05: russh must keep aws-lc-rs: {russh_line}"
    );
}

/// Feature `tls` is default and wires rustls + instant-acme.
#[test]
fn cargo_toml_tls_feature_default() {
    let cargo = read("Cargo.toml");
    assert!(
        cargo.contains("default = [\"ssh-real\", \"tls\"]")
            || cargo.contains("default = [\"tls\", \"ssh-real\"]"),
        "default features must include tls"
    );
    assert!(
        cargo.contains("rustls ="),
        "Cargo.toml must declare rustls"
    );
    assert!(
        cargo.contains("instant-acme"),
        "Cargo.toml must declare instant-acme for ACME"
    );
    // Dependency line must not pull the unmaintained crate (comments may mention the ban).
    let has_dep = cargo.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with("rustls-pemfile") || t.starts_with("rustls-pemfile =")
    });
    assert!(!has_dep, "must not depend on unmaintained rustls-pemfile");
}

/// G-TLS-01/06: SECURITY documents transport + rustls policy.
#[test]
fn security_md_documents_transport_crypto_policy() {
    let sec = read("SECURITY.md");
    assert!(
        sec.contains("Transport & crypto policy") || sec.contains("Transport and crypto policy"),
        "G-TLS-01: SECURITY.md missing Transport & crypto policy section"
    );
    assert!(
        sec.contains("aws-lc-rs") || sec.contains("aws_lc_rs"),
        "G-TLS-06: SECURITY.md must name aws-lc-rs provider"
    );
    assert!(
        sec.to_lowercase().contains("rustls"),
        "G-TLS-01: SECURITY.md must mention rustls"
    );
    assert!(
        sec.to_lowercase().contains("compression") || sec.contains("none"),
        "G-TLS-04: SECURITY.md should document compression policy"
    );
}

/// G-TLS-01: README surfaces crypto policy for humans/agents.
#[test]
fn readme_mentions_ssh_crypto_policy() {
    let readme = read("README.md");
    assert!(
        readme.contains("aws-lc-rs") || readme.contains("aws_lc_rs"),
        "G-TLS-06: README must mention aws-lc-rs"
    );
    let ok = readme.contains("crypto policy")
        || readme.contains("Crypto policy")
        || readme.contains("Transport & crypto")
        || readme.contains("SSH-over-TLS")
        || (readme.contains("SSH") && readme.to_lowercase().contains("rustls"));
    assert!(
        ok,
        "G-TLS-01: README should state SSH / rustls crypto policy"
    );
}

/// Binary installs CryptoProvider before runtime.
#[test]
fn main_installs_crypto_provider() {
    let main = read("src/main.rs");
    assert!(
        main.contains("install_default_provider"),
        "main must call install_default_provider before runtime"
    );
}

/// ConnectionConfig carries optional TLS options.
#[test]
fn connection_config_has_tls_field() {
    let conn = read("src/ssh/connection.rs");
    assert!(
        conn.contains("tls: Option") || conn.contains("pub tls:"),
        "ConnectionConfig must expose tls options"
    );
}
