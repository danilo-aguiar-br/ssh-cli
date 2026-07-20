// SPDX-License-Identifier: MIT OR Apache-2.0
// G-CLOSE-04: pure module — no `unsafe` permitted.
#![forbid(unsafe_code)]
//! Operating-system conditional abstractions.
//!
//! Platform initialization ([`initialize_platform`]) is the **first I/O-related
//! step** after signal/telemetry bootstrap in [`crate::run`]. It configures:
//!
//! - **Windows**: console UTF-8 (code page 65001) + virtual terminal processing
//!   for ANSI colors under cmd.exe / PowerShell 5.1 / Windows Terminal
//! - **Linux / Unix**: sandbox detection (Flatpak/Snap) with observability warn
//! - **macOS**: no-op init (paths via `directories`; Gatekeeper is user-side)
//!
//! # Runtime environment
//!
//! [`detect_runtime`] classifies WSL, containers, CI, Termux, and distribution
//! sandboxes **without** spawning external processes. Results feed
//! `vps doctor --json` diagnostics (agent-visible, no secrets).
//!
//! # Product scope (N/A by design)
//!
//! - Browser / Chrome / chromedriver discovery — not an SSH concern
//! - WASM / WASI targets — `russh` requires real sockets; not shipped
//! - Job Objects / local `Command` children — no privileged local subprocess tree
//! - OpenBSD pledge/unveil, seccomp, setrlimit — optional hardening; not default
//!
//! # External processes (G-PROC audit)
//!
//! Runtime **never** shells out (`uname`, `ssh`, `scp`, `systemctl`, etc.).
//! SSH transport is pure [`russh`]. The only `std::process::Command` uses in the
//! tree are:
//!
//! | Site | Binary | When | Failure mode |
//! |------|--------|------|--------------|
//! | `build.rs` | `git` (optional) | embed commit hash | `unknown` / env / `.commit_hash` |
//! | integration tests | `ssh-keygen` (optional fixture) | OpenSSH key files | skip / assert |
//! | integration tests | `ssh-cli` under test | assert_cmd e2e | test failure |
//!
//! Toolchain MSRV **1.85.0** exceeds Rust **1.77.2** (CVE-2024-24576 / BatBadBut);
//! product still never invokes `.bat`/`.cmd` children.

use anyhow::Result;
use serde::Serialize;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Initializes the platform before user-facing I/O.
///
/// MUST be called early in [`crate::run`] (after signals + log bootstrap).
///
/// # Errors
/// Propagates platform setup failures (Windows console APIs currently warn and
/// still return `Ok` so agents are not blocked on console edge cases).
pub fn initialize_platform() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        windows::configure_console()?;
    }

    #[cfg(target_os = "linux")]
    {
        linux::detect_sandbox();
    }

    #[cfg(target_os = "macos")]
    {
        macos::initialize();
    }

    // Cross-platform observability: one structured debug line at boot.
    let env = detect_runtime();
    tracing::debug!(
        os = env.os,
        arch = env.arch,
        wsl = env.is_wsl,
        container = env.is_container,
        ci = env.is_ci,
        termux = env.is_termux,
        sandbox = env.sandbox.unwrap_or("none"),
        "runtime environment detected"
    );

    Ok(())
}

/// Normalizes a stdin line by stripping trailing `\r` (CRLF → LF).
///
/// Required on Windows where pipes may emit `\r\n`. Does not alter embedded
/// newlines in multi-line payloads (only trims end-of-line CR/LF).
#[must_use]
pub fn normalize_stdin_line(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}

/// Returns `true` if stdout is connected to a terminal (TTY).
///
/// Prefer [`crate::terminal::is_interactive`] for color decisions (also honors
/// `TERM=dumb`). This helper is the raw TTY probe for platform code.
#[must_use]
pub fn is_tty() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}

/// Detected host runtime (process environment classification).
///
/// All fields are pure heuristics from env vars and a few well-known paths —
/// no shell-outs (`uname`, `systemd-detect-virt`, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RuntimeEnvironment {
    /// `std::env::consts::OS` (e.g. `linux`, `macos`, `windows`).
    pub os: &'static str,
    /// `std::env::consts::ARCH` (e.g. `x86_64`, `aarch64`).
    pub arch: &'static str,
    /// Running under Windows Subsystem for Linux (WSL1/WSL2).
    pub is_wsl: bool,
    /// Running inside a container (Docker/Podman/Kubernetes/etc.).
    pub is_container: bool,
    /// Continuous integration environment (`CI=true` or known vendor vars).
    pub is_ci: bool,
    /// Android Termux (bionic) environment.
    pub is_termux: bool,
    /// Distribution sandbox when known: `"flatpak"`, `"snap"`, or `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<&'static str>,
}

/// Classifies the current process runtime (cheap, side-effect free).
#[must_use]
pub fn detect_runtime() -> RuntimeEnvironment {
    RuntimeEnvironment {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        is_wsl: detect_wsl(),
        is_container: detect_container(),
        is_ci: detect_ci(),
        is_termux: detect_termux(),
        sandbox: detect_sandbox_kind(),
    }
}

fn detect_wsl() -> bool {
    if std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::env::var_os("WSL_INTEROP").is_some()
        || std::env::var_os("WSLENV").is_some()
    {
        return true;
    }
    // WSL1/2 often expose Microsoft in /proc/version (Linux only).
    #[cfg(target_os = "linux")]
    {
        if let Ok(v) = std::fs::read_to_string("/proc/version") {
            let lower = v.to_ascii_lowercase();
            if lower.contains("microsoft") || lower.contains("wsl") {
                return true;
            }
        }
    }
    false
}

fn detect_container() -> bool {
    if std::env::var_os("KUBERNETES_SERVICE_HOST").is_some()
        || std::env::var_os("container").is_some()
    {
        return true;
    }
    // Docker classic marker; Podman often uses /run/.containerenv.
    if std::path::Path::new("/.dockerenv").exists()
        || std::path::Path::new("/run/.containerenv").exists()
    {
        return true;
    }
    // cgroup hint (best-effort; may false-positive on some hosts — still useful).
    #[cfg(target_os = "linux")]
    {
        if let Ok(cg) = std::fs::read_to_string("/proc/1/cgroup") {
            let lower = cg.to_ascii_lowercase();
            if lower.contains("docker")
                || lower.contains("containerd")
                || lower.contains("kubepods")
                || lower.contains("libpod")
                || lower.contains("/lxc/")
            {
                return true;
            }
        }
    }
    false
}

fn detect_ci() -> bool {
    // Generic + common vendors (GitHub, GitLab, Azure, Circle, Buildkite, Travis, Jenkins).
    if std::env::var("CI").map(|v| !v.is_empty() && v != "0" && v != "false") == Ok(true) {
        return true;
    }
    const VENDOR_VARS: &[&str] = &[
        "GITHUB_ACTIONS",
        "GITLAB_CI",
        "TF_BUILD",
        "CIRCLECI",
        "BUILDKITE",
        "TRAVIS",
        "JENKINS_URL",
        "APPVEYOR",
        "TEAMCITY_VERSION",
        "BITBUCKET_BUILD_NUMBER",
    ];
    VENDOR_VARS.iter().any(|k| std::env::var_os(k).is_some())
}

fn detect_termux() -> bool {
    std::env::var_os("TERMUX_VERSION").is_some()
        || std::env::var_os("TERMUX_APK_RELEASE").is_some()
        || std::env::var("PREFIX")
            .map(|p| p.contains("com.termux"))
            .unwrap_or(false)
}

fn detect_sandbox_kind() -> Option<&'static str> {
    if std::env::var_os("FLATPAK_ID").is_some() {
        return Some("flatpak");
    }
    if std::env::var_os("SNAP").is_some() {
        return Some("snap");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn normalize_strips_trailing_cr() {
        assert_eq!(normalize_stdin_line("test\r"), "test");
        assert_eq!(normalize_stdin_line("test\r\n"), "test");
        assert_eq!(normalize_stdin_line("test\n"), "test");
        assert_eq!(normalize_stdin_line("test"), "test");
    }

    #[test]
    fn normalize_empty_string() {
        assert_eq!(normalize_stdin_line(""), "");
    }

    #[test]
    fn normalize_newlines_only() {
        assert_eq!(normalize_stdin_line("\n\n\n"), "");
    }

    #[test]
    fn normalize_mixed_crlf_lf_keeps_interior() {
        assert_eq!(
            normalize_stdin_line("line1\r\nline2\r\nline3"),
            "line1\r\nline2\r\nline3"
        );
    }

    #[test]
    fn normalize_with_spaces() {
        assert_eq!(
            normalize_stdin_line("text with spaces  \r\n"),
            "text with spaces  "
        );
    }

    #[test]
    fn is_tty_returns_bool() {
        let _ = is_tty();
    }

    #[test]
    fn detect_runtime_has_os_and_arch() {
        let env = detect_runtime();
        assert!(!env.os.is_empty());
        assert!(!env.arch.is_empty());
    }

    #[test]
    #[serial]
    fn detect_ci_honors_ci_env() {
        let prev = std::env::var("CI").ok();
        crate::test_util::env::set_var("CI", "true");
        assert!(detect_ci());
        match prev {
            Some(v) => crate::test_util::env::set_var("CI", v),
            None => crate::test_util::env::remove_var("CI"),
        }
    }

    #[test]
    #[serial]
    fn detect_sandbox_flatpak() {
        let prev_f = std::env::var("FLATPAK_ID").ok();
        let prev_s = std::env::var("SNAP").ok();
        crate::test_util::env::set_var("FLATPAK_ID", "org.example.App");
        crate::test_util::env::remove_var("SNAP");
        assert_eq!(detect_sandbox_kind(), Some("flatpak"));
        match prev_f {
            Some(v) => crate::test_util::env::set_var("FLATPAK_ID", v),
            None => crate::test_util::env::remove_var("FLATPAK_ID"),
        }
        match prev_s {
            Some(v) => crate::test_util::env::set_var("SNAP", v),
            None => crate::test_util::env::remove_var("SNAP"),
        }
    }

    #[test]
    #[serial]
    fn detect_sandbox_snap() {
        let prev_f = std::env::var("FLATPAK_ID").ok();
        let prev_s = std::env::var("SNAP").ok();
        crate::test_util::env::remove_var("FLATPAK_ID");
        crate::test_util::env::set_var("SNAP", "/snap/app");
        assert_eq!(detect_sandbox_kind(), Some("snap"));
        match prev_f {
            Some(v) => crate::test_util::env::set_var("FLATPAK_ID", v),
            None => crate::test_util::env::remove_var("FLATPAK_ID"),
        }
        match prev_s {
            Some(v) => crate::test_util::env::set_var("SNAP", v),
            None => crate::test_util::env::remove_var("SNAP"),
        }
    }

    #[test]
    #[serial]
    fn detect_termux_via_version() {
        let prev = std::env::var("TERMUX_VERSION").ok();
        crate::test_util::env::set_var("TERMUX_VERSION", "0.118");
        assert!(detect_termux());
        match prev {
            Some(v) => crate::test_util::env::set_var("TERMUX_VERSION", v),
            None => crate::test_util::env::remove_var("TERMUX_VERSION"),
        }
    }

    #[test]
    #[serial]
    fn detect_wsl_via_distro_name() {
        let prev = std::env::var("WSL_DISTRO_NAME").ok();
        crate::test_util::env::set_var("WSL_DISTRO_NAME", "Ubuntu");
        assert!(detect_wsl());
        match prev {
            Some(v) => crate::test_util::env::set_var("WSL_DISTRO_NAME", v),
            None => crate::test_util::env::remove_var("WSL_DISTRO_NAME"),
        }
    }

    #[test]
    fn runtime_environment_serializes_json_keys() {
        let env = RuntimeEnvironment {
            os: "linux",
            arch: "x86_64",
            is_wsl: false,
            is_container: true,
            is_ci: true,
            is_termux: false,
            sandbox: Some("flatpak"),
        };
        let v = serde_json::to_value(env).unwrap();
        assert_eq!(v["os"], "linux");
        assert_eq!(v["is_container"], true);
        assert_eq!(v["sandbox"], "flatpak");
    }
}
