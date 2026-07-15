// SPDX-License-Identifier: MIT OR Apache-2.0
//! Testes de integração do módulo tunnel.
//!
//! Testa o subcomando `tunnel` via CLI, validando help e parâmetros obrigatórios.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use tempfile::TempDir;

fn cmd(tmp: &TempDir) -> Command {
    let llvm_profile_file = std::env::var_os("LLVM_PROFILE_FILE");
    let mut c = Command::new(env!("CARGO_BIN_EXE_ssh-cli"));
    c.env_clear();
    c.env("PATH", std::env::var_os("PATH").unwrap_or_default());
    if let Some(value) = llvm_profile_file {
        c.env("LLVM_PROFILE_FILE", value);
    }
    c.env("HOME", tmp.path());
    c.env("XDG_CONFIG_HOME", tmp.path());
    c.arg("--config-dir").arg(tmp.path());
    c
}

#[test]
#[serial]
fn tunnel_help_shows_usage() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["tunnel", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tunnel"))
        .stdout(predicate::str::contains("VPS_NAME"))
        .stdout(predicate::str::contains("LOCAL_PORT"))
        // GAP-SSH-IO-008
        .stdout(predicate::str::contains("--json"));
}

#[test]
#[serial]
fn tunnel_without_params_returns_error() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp).args(["tunnel"]).assert().failure();
}

#[test]
#[serial]
fn tunnel_with_only_vps_returns_error() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp).args(["tunnel", "minha-vps"]).assert().failure();
}

#[test]
#[serial]
fn tunnel_invalid_params_returns_error() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["tunnel", "vps-teste", "abc", "host-remoto", "8080"])
        .assert()
        .failure();
}

#[test]
#[serial]
fn tunnel_missing_vps_returns_error() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["tunnel", "fantasma-tunnel", "8080", "localhost", "3000"])
        .assert()
        .failure();
}

#[test]
#[serial]
fn tunnel_local_port_out_of_range_returns_error() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["tunnel", "vps-inexistente", "999999", "localhost", "8080"])
        .assert()
        .failure();
}

#[test]
#[serial]
fn tunnel_unknown_command_returns_error() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["tunnel", "comando-inexistente"])
        .assert()
        .failure();
}

#[test]
#[serial]
fn tunnel_help_shows_port_forward_description() {
    let tmp = TempDir::new().unwrap();
    cmd(&tmp)
        .args(["tunnel", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deadline"))
        .stdout(predicate::str::contains("SSH"));
}
