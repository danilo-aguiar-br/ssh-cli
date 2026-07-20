// SPDX-License-Identifier: MIT OR Apache-2.0
//! # ssh-cli
//!
//! Full-stack Rust CLI that gives an LLM (Claude Code, Cursor, Windsurf) the ability
//! to operate remote servers over SSH in a subprocess flow via stdin/stdout.
//!
//! ## Modules
//!
//! | Module          | Responsibility                                                |
//! |-----------------|---------------------------------------------------------------|
//! | `cli`           | Clap derive argument definitions (contract)                   |
//! | `commands`      | Subcommand dispatch layer (CAMADA 2)                          |
//! | `concurrency`   | Bounded multi-host / tunnel fan-out (`Semaphore` + `JoinSet`) |
//! | `constants`     | Named domain constants (XDG names, env keys, network/timing)  |
//! | `net`           | Async DNS + Happy Eyeballs TCP dial for SSH connect           |
//! | `error`/`errors`| Structured error types via `thiserror` + retry classification |
//! | `retry`         | Named `RetryConfig` + full-jitter backoff (agent contract)    |
//! | `vps`           | CRUD and persistence of VPS records (XDG + TOML + 0o600)      |
//! | `secrets`       | Primary key and default at-rest encryption (ChaCha20-Poly1305)|
//! | `ssh`           | Real one-shot SSH client via `russh` (password/key, TOFU)     |
//! | `tls`           | rustls (aws_lc_rs): SSH-over-TLS, mTLS, ACME (feature `tls`)  |
//! | `i18n`          | Bilingual UI (`Message` enum + exhaustive EN/pt-BR match)     |
//! | `json_wire`     | Typed agent JSON DTOs + compact emit (RFC 8259, not NDJSON)   |
//! | `locale`        | BCP47 detect/negotiate (`sys-locale` + `unic-langid` + langneg)|
//! | `platform`      | Windows UTF-8/VT, runtime env (WSL/container/CI), TTY         |
//! | `masking`       | Unicode-safe masking of sensitive values                      |
//! | `output`        | Sole module authorized for stdout/stderr data emission        |
//! | `paths`         | Path validation and normalization (anti-traversal, NFC)       |
//! | `signals`       | One-shot SIGINT/SIGTERM flags + cooperative `should_stop`     |
//! | `telemetry`     | Process-local `tracing` install (stderr; no OTEL / no files)  |
//! | `validation`    | Parse→serde→validator pipeline for config/import (no OTEL)    |
//! | `domain`        | Newtypes: VpsName, Rfc3339Utc, BatchRunId, HttpsUrl, Money…  |
//! | `terminal`      | TTY detection and color choice via `termcolor`                |
//!
//! ## Features
//!
//! | Feature            | Default | Effect                                                              |
//! |--------------------|---------|---------------------------------------------------------------------|
//! | `ssh-real`         | yes     | Real SSH via `russh` + `aws-lc-rs` (compression `none` only; G-TLS) |
//! | `tls`              | yes     | rustls ≥0.23.18 + aws_lc_rs: SSH-over-TLS, mTLS, ACME              |
//! | `musl-allocator`   | no      | Uses `mimalloc` as `#[global_allocator]` (binary only; musl/Alpine) |
//! | `i18n-full`        | no      | Reserved: top-20 economic locales (no extra strings yet)            |
//! | `i18n-cjk`         | no      | Reserved: zh-Hans / zh-Hant / ja / ko                               |
//! | `i18n-rtl`         | no      | Reserved: ar / he (RTL isolation)                                   |
//! | `i18n-europe`      | no      | Reserved: additional European locales                               |
//!
//! Disable real SSH only for dependency diagnosis: `--no-default-features`.
//! Documented feature gates use `#[doc(cfg(...))]` under the `docsrs` cfg.
//! Default binary always embeds **en** + **pt-BR** only (Rules: no full top-20).
//!
//! ## Entry point
//!
//! The public [`run`] function is the entry point called by `main.rs`.
//!
//! ## Safety
//!
//! - **docs.rs / rustdoc:** when built with `--cfg docsrs`, this crate enables
//!   `#![feature(doc_cfg)]` so `#[doc(cfg(...))]` labels render on feature-gated
//!   items (migration `doc_auto_cfg` → `doc_cfg`). Consumers of *this* crate do
//!   not need nightly; only the docs.rs build uses the feature gate.
//! - **unsafe:** product code avoids `unsafe` on the happy path; remaining blocks
//!   (platform console / Unix permissions) are documented at the call site.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
// G-SECDEV-05: crate root cannot `forbid(unsafe_code)` — Windows console FFI and
// Unix test env helpers need minimal `unsafe`. Pure modules apply
// `#![forbid(unsafe_code)]` individually. Undocumented/multi-op blocks are deny.
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
// G-SEC-01: each unsafe op inside `unsafe fn` must still sit in an explicit
// `unsafe {}` block (RFC 2585 / edition 2024 posture). Deny so regressions fail CI.
#![deny(unsafe_op_in_unsafe_fn)]
// G-SECDEV-06: `mem::forget` on Drop-critical types is a security antipattern.
#![deny(clippy::mem_forget)]
#![warn(rustdoc::broken_intra_doc_links)]
// Const/static rules: forbid static mut refs and interior-mutable `const`.
#![deny(static_mut_refs)]
#![deny(clippy::declare_interior_mutable_const)]
#![deny(clippy::borrow_interior_mutable_const)]

pub mod cli;
pub mod commands;
/// Bounded multi-host / tunnel fan-out (Semaphore + JoinSet).
pub mod concurrency;
/// Named domain constants (XDG file names, env keys, network/timing defaults).
pub mod constants;
/// TCP dial (async DNS + Happy Eyeballs multi-address connect).
pub mod net;
/// Canonical error module name per clap layout rules (`error.rs`).
pub mod error;
/// Structured error types (`thiserror`) and sysexits-style exit codes.
pub mod errors;
/// Explicit retry policy + full-jitter backoff (agent re-invoke; opt-in in-process).
pub mod retry;
pub mod i18n;
pub mod json_wire;
pub mod locale;
pub mod masking;
pub mod output;
pub mod paths;
/// Unix secret file/dir permission helpers (G-AUD-24).
pub mod fs_perm;
pub mod platform;
pub mod scp;
pub mod sftp;
pub mod secrets;
pub mod signals;
pub mod ssh;
/// rustls TLS stack (SSH-over-TLS, mTLS, ACME) — feature `tls` (default).
pub mod tls;
/// Process-local tracing subscriber (binary path only; libraries only emit).
pub mod telemetry;
pub mod terminal;
pub mod tunnel;
/// Shared parse→validate pipeline for external config/import (G-SERDE-07).
pub mod validation;
/// Domain newtypes (parse, don't validate — G-TYPE / G-DOM 4-crates).
pub mod domain;
pub mod vps;

/// Test-only helpers (env mutation with SAFETY). Not linked into release builds.
#[cfg(test)]
pub(crate) mod test_util;

use anyhow::Result;

/// Runs ssh-cli from the command-line arguments.
///
/// One-shot lifecycle (Rules Rust CLI one-shot — six phases):
/// 1. **Init** — SIGINT/SIGTERM handlers, bootstrap tracing (stderr, error-only).
/// 2. Platform (UTF-8 / TTY).
/// 3. **Parse** — clap derive.
/// 4. **Validate / configure** — reload log filter from `-v` only (ambient `RUST_LOG` ignored), terminal, i18n.
/// 5. **Execute** — subcommand dispatch (bounded SSH timeouts + cancel flags).
/// 6. **Finalize / exit** — handled by `main` (flush + runtime shutdown + sysexits).
///
/// # Workload classification (resource economy + performance)
///
/// **I/O-bound** one-shot CLI (SSH/TCP, optional disk SCP). Not CPU-bound:
/// **no Rayon** on product paths (crypto/IO already on Tokio). Multi-host
/// fan-out is the default modus operandi for SSH ops that accept `--all`
/// (`health-check`, `exec`, `sudo-exec`, `su-exec`, `scp`) via
/// [`concurrency::map_bounded`] (`Semaphore` + `JoinSet`, cap from
/// `--max-concurrency` / auto CPU×RAM formula; no env-as-store).
/// Tunnel forwards use the same admission gate. Local TOML CRUD / locale /
/// completions stay sequential (work ≪ coordination). Heavy-memory
/// singletons use `OnceLock` / atomics (signals, locale, logs).
///
/// # Ownership / borrowing policy (Rules Rust)
///
/// - Prefer the **least** permission: `&T` → `&mut T` → `T` (consume) only when needed.
/// - Config override paths: APIs take `Option<&Path>` (`resolve_config_path`,
///   `find_by_name`, `winning_layer`, `read_active_vps`) so one CLI `PathBuf` is shared
///   without `clone` at every hop.
/// - Local `ConfigFile` maps: `remove` the `VpsRecord` for one-shot exec/sudo/su/health
///   when the file is discarded after load (move, not clone).
/// - Errors that own stderr: **move** `output.stderr` into `SshCliError::CommandFailed`.
/// - Secrets: `Option::take` / `SecretString` move; never clone passwords for convenience.
/// - Shared SSH client in tunnel: `Arc<dyn SshClientTrait>` + `Arc::clone` (refcount only).
/// - No `Rc` / `RefCell` / `Arc<RefCell<_>>` / `static mut` in product code.
/// - Lifetimes: elision preferred; no `'static` escapes for non-global data.
/// - `unsafe` only at OS boundaries (console, signals env tests) with `// SAFETY:`.
///
/// # Interior mutability policy (Rules Rust)
///
/// - Prefer **no** interior mutability: reorganize ownership first.
/// - Process flags: `static AtomicBool` with documented `Ordering` (signals, quiet/json).
/// - One-shot init: `OnceLock` (locale, color, log reload handle) — not `lazy_static`.
/// - Composite process state: single `std::sync::Mutex` + poison recovery with log
///   (`secrets`); never hold across `.await`.
/// - No `RefCell` / `Rc` / `Arc<RefCell<_>>` / `static mut` in product code.
/// - Tunnel deadline flag: `Arc<AtomicBool>` only where two tasks must share a bit.
///
/// # Graceful shutdown policy (Rules Rust — one-shot minimum)
///
/// - Detect SIGINT/SIGTERM early; long ops poll [`signals::should_stop`].
/// - Tunnel stops accepts, drains/aborts tracked forwards, then disconnects.
/// - Flush stdio; shut down Tokio runtime; exit **130** / **141** / **143**.
/// - Not a daemon: no readiness probes, SIGHUP reload, or `TaskTracker` tree.
///
/// # JSON wire policy (Rules Rust — JSON / NDJSON)
///
/// - Agent contracts are **classic single-root JSON** (object or array), RFC 8259 —
///   **not** NDJSON/JSONL streams. One document per invocation on the data path.
/// - Emit **compact** UTF-8 (`serde_json::to_string`) + trailing LF; no pretty-print,
///   no BOM, no JSON5 on the machine wire.
/// - Known payloads use typed DTOs in [`json_wire`]; `serde_json::Value` only at
///   dynamic edges (`meta command-tree`, flexible success-field maps).
/// - Import of `vps export --json` strips BOM, caps size, Must-Ignore unknown fields.
/// - Hand-versioned schemas live under `docs/schemas/`; no runtime schema engine.
/// - On-disk host registry remains **TOML** (not JSON config).
///
/// # Performance policy (Rules Rust)
///
/// - Measure before micro-optimizing; prefer algorithmic / allocation caps
///   (see `ssh::client` capture byte cap) over `#[inline(always)]` guesses.
/// - Publish default is **size-min** release (`opt-level = "z"` + fat LTO);
///   local speed A/B uses `--profile release-fast` / `release-lto` (`opt-level = 3`).
/// - Criterion covers local mask/paths only — not SSH flamegraphs.
///
/// # Multiplatform policy (Rules Rust — sistemas operacionais)
///
/// - **Boot:** Windows console UTF-8 (65001) + `ENABLE_VIRTUAL_TERMINAL_PROCESSING`;
///   Linux sandbox warn (Flatpak/Snap); runtime classify WSL/container/CI/Termux.
/// - **Paths:** `PathBuf` only; Windows reserved names; component ≤255; MAX_PATH
///   guard without `\\?\`; Unicode NFC normalization for comparisons.
/// - **Permissions:** Unix `0o600` behind `#[cfg(unix)]` only (no ACL leakage).
/// - **Config home:** `directories::ProjectDirs` + optional `--config-dir` (no `SSH_CLI_HOME` store).
/// - **Completions:** clap_complete shells (Bash, Elvish, Fish, PowerShell, Zsh).
/// - **Out of scope:** browser discovery, WASM/WASI, Job Objects, seccomp default,
///   macOS notarization inside the binary (release process — see CROSS_PLATFORM).
///
/// # i18n policy (Rules Rust — multi-idioma / locale do SO)
///
/// - **Boot order:** platform console + runtime detect → TTY/color → locale → rest.
/// - **Detection:** single `sys_locale::get_locale` call; never portable raw `LANG`.
/// - **Parse / negotiate:** `unic-langid` + `fluent-langneg` against [`i18n::Language::AVAILABLE`].
/// - **State:** one immutable [`std::sync::OnceLock`] language per process (no mid-session mix).
/// - **Overrides:** `--lang` > XDG `lang` file (`locale set`) > system > `en` (`SSH_CLI_LANG` not a store).
/// - **UI copy:** human success/status/cancel via [`i18n::Message`]; agent JSON +
///   [`errors::SshCliError`] Display stay **stable English** (pipe/agent contract).
/// - **MVP:** `en` + `pt-BR` only; optional locales behind `i18n-*` features (stubs).
/// - **Out of scope for default binary:** full Fluent FTL runtime, ICU calendars/collators,
///   pseudolocalization, RTL shaping (reserved features; size-sensitive one-shot).
///
/// # Parallelism / multiprocessing policy (Rules Rust — paralelismo)
///
/// - **Modus operandi:** bounded concurrent I/O on every multi-target SSH surface
///   (`--all` **or** `--hosts a,b`); sequential only when work is local/tiny
///   (documented at each call site — G-PAR-28).
/// - **Session reuse (G-PAR-47):** multi-file SCP on one host uses **one** SSH
///   session and serial transfers (auth once). Multi-host × multi-file (G-PAR-48)
///   bounds **sessions** via `map_bounded`, reusing the session for all files.
/// - **TOFU (G-PAR-49):** `known_hosts` mutations take exclusive flock + reload-merge.
/// - **Selection:** [`vps::HostSelection`] + [`vps::resolve_host_jobs`] is the
///   single path that builds fan-out jobs (G-PAR-31). Batch JSON when selection
///   is `All`/`Named` even if one name (G-PAR-36).
/// - **Gate:** `tokio::sync::Semaphore` in [`concurrency`]; `acquire_owned` + RAII
///   permit drop; `JoinSet` for dynamic fan-out; never unbounded `spawn` loops.
/// - **Budget:** `min(cpus×4, free_ram×50%/16MiB)` clamped `1..=64`; override
///   `--max-concurrency` (auto formula pre-parse; no env store).
/// - **Runtime:** multi_thread workers from [`concurrency::worker_threads`];
///   `max_blocking_threads` capped; no nested runtimes; no Rayon.
/// - **Tunnel:** one local bind + one SSH session per one-shot (G-PAR-30); multi-host
///   tunnels = N invocations. Accepts still use JoinSet + Semaphore.
/// - **N/A for this product:** loom lock models, parking_lot deadlock detector,
///   systemd-run MemoryMax child scopes, OTEL available_permits metrics, hierarchical
///   `CancellationToken` trees (one-shot uses atomic signal flags).
///
/// # Latency policy (Rules Rust — redução de latência)
///
/// - **Identity:** one-shot I/O-bound agent CLI. End-to-end latency is dominated by
///   **SSH/TCP RTT**, not CPU nanoseconds. HFT budgets (P9999 ns, isolcpus, mlockall,
///   huge pages, kernel bypass, PGO/BOLT pipelines) are **out of scope**.
/// - **What we optimize:** cold-start (capped Tokio workers), multi-host wall-clock
///   via bounded fan-out, zero extra copies on exec capture happy path, non-blocking
///   disk I/O on the async runtime (SCP), bounded capture RAM, cooperative cancel.
/// - **What we do not claim:** process-level P50/P99 histograms per release, HDR
///   export, or coordinated-omission load tests — there is no long-lived server.
/// - **Build:** fat LTO + `codegen-units = 1` + `panic = abort` on release;
///   `opt-level = "z"` for publish footprint; `release-fast`/`release-lto` for
///   local CPU A/B. No `target-cpu=native` on published artifacts.
/// - **Allocator:** system default on glibc; optional `mimalloc` via `musl-allocator`
///   (measure before making default).
///
/// # Logging / tracing policy (Rules Rust — logs com tracing e rotação)
///
/// - **Facade:** `tracing` only. Product code never uses `println!`/`dbg!` for
///   diagnostics; agent data is emitted only via [`output`].
/// - **Install once:** [`telemetry::bootstrap_logs`] before clap parse, then
///   [`telemetry::initialize_logs`] reloads `EnvFilter` (`reload::Layer`).
/// - **Sink:** stderr text with targets + thread names; default filter `error`.
/// - **Bridge:** `tracing-log` so `russh`/`keyring` `log` records appear under
///   the same filter.
/// - **Not installed:** OpenTelemetry, file rotation (`tracing-appender`),
///   admin log-level HTTP, `tokio-console` — out of product identity
///   (one-shot agent CLI, zero telemetry, stdout = data).
///
/// # Macro policy (Rules Rust — macros)
///
/// - **No product `macro_rules!` / proc-macro crates in this workspace.** Prefer
///   generics, traits, functions, and `const` before inventing syntax. A thin
///   rename macro over a function is an antipattern.
/// - **External derives only when justified:** `clap` / `serde` / `thiserror`
///   (`proc_macro_derive`) generate type-driven boilerplate that functions cannot
///   express; no hand-rolled derive crate.
/// - **Built-in std macros, idiomatically:**
///   - `format!` when an owned `String` is required (i18n, error payloads).
///   - `format_args!` + [`output::write_line_fmt`] / [`output::write_stderr_fmt`]
///     / `writeln!` for stream emission — **never** `write_*(&format!(…))`.
///   - `matches!` for boolean pattern checks; `env!`/`concat!` for version wire
///     (`cli` long version); `include_str!` only in tests that audit source.
/// - **Forbidden in product paths:** `todo!`, `unimplemented!`, `dbg!`, and
///   `panic!` for recoverable errors (tests may `panic!` on fixture mismatch).
/// - **Not applicable:** custom declarative/proc macro hygiene, `trybuild` UI
///   suites, `$crate` export crates — there is no macro surface to publish.
///
/// # Stream architecture (G-IO-11)
///
/// - **Binary path:** [`run`] parses `std::env::args` and uses process stdio.
/// - **Library path:** [`run_with_args`] accepts a pre-parsed [`cli::CliArgs`].
/// - **DI write primitives:** [`output::write_line_to`], [`output::write_stderr_line_to`],
///   [`json_wire::write_json_line`] — pass `Cursor`/`Vec` in tests.
/// - **Exit mapping:** [`resolve_exit_code`] keeps `main` thin (flush + runtime
///   shutdown + `process::exit` only).
pub async fn run() -> Result<()> {
    // Phase 1: signals BEFORE any work (rules: first). Binary `main` already
    // registers before Tokio multi_thread (G-UNSAFE-13); this call is idempotent.
    signals::register_handler()?;
    // Phase 1b: tracing BEFORE parse (rules: second); verbosity reloaded after argv.
    telemetry::bootstrap_logs();

    platform::initialize_platform()?;

    // Phase 3: parse real process argv
    let args = cli::parse_args();

    // Phases 4–5
    run_with_args(args).await
}

/// Executes phases 4–5 with **pre-parsed** arguments (G-IO-11 library entry).
///
/// Callers that already own a [`cli::CliArgs`] (tests, embedders, alternate
/// front-ends) skip clap parse. Process stdout/stderr remain the default sinks
/// via [`output`]; injectable writers live on `write_*_to` / `write_json_line`.
///
/// Does **not** re-register signals or re-bootstrap tracing — call
/// [`signals::register_handler`] + [`telemetry::bootstrap_logs`] first when
/// embedding outside [`run`].
///
/// # Errors
/// Propagates domain / I/O errors from command dispatch.
pub async fn run_with_args(args: cli::CliArgs) -> Result<()> {
    // Phase 4: configure from args (logs → terminal/TTY → locale before any UI)
    telemetry::initialize_logs(args.verbose);
    terminal::initialize(args.no_color)?;
    i18n::initialize_language(args.lang.as_deref(), args.config_dir.as_deref())?;
    // Phase 5: execute
    commands::run(args).await
}

/// Prints a product error envelope and returns its exit code.
fn emit_resolved_ssh_error(ssh_err: &errors::SshCliError, wants_json: bool) -> i32 {
    let code = ssh_err.exit_code();
    let remote = match ssh_err {
        errors::SshCliError::CommandFailed { exit_code, .. } => Some(*exit_code),
        _ => None,
    };
    if wants_json {
        // Envelope DTO owns the message String (required by serde).
        // G-RETRY / G-ERR-08: error_code + error_class + retryable.
        let _ = output::print_error_envelope(
            code,
            ssh_err.error_code(),
            &ssh_err.to_string(),
            remote,
            ssh_err.classify(),
            ssh_err.is_retryable(),
            ssh_err.suggestion(),
        );
    } else {
        // G-MAC-01: Display via write_fmt — no temporary String.
        let _ = output::print_error_fmt(format_args!("{ssh_err}"));
    }
    let _ = std::io::Write::flush(&mut std::io::stderr());
    code
}

/// Maps a [`run`] / [`run_with_args`] result to a sysexits-aligned exit code.
///
/// Side effect: on domain errors (and not signal/pipe), prints the human or
/// JSON error envelope to stderr via [`output`] (same contract as the binary).
///
/// Prefer this from `main` so exit policy stays in the library (G-IO-11).
///
/// Recovers [`errors::SshCliError`] and bare [`crate::domain::DomainError`]
/// that bubbled through `anyhow` without the product wrapper (R-04 / R-14).
///
/// # Examples
///
/// ```
/// use ssh_cli::{errors::exit_codes, resolve_exit_code};
///
/// assert_eq!(resolve_exit_code(Ok(())), exit_codes::EX_OK);
/// ```
#[must_use]
pub fn resolve_exit_code(result: Result<()>) -> i32 {
    match result {
        Ok(()) => signals::signal_exit_code().unwrap_or(errors::exit_codes::EX_OK),
        Err(e) => {
            if let Some(sig) = signals::signal_exit_code() {
                return sig;
            }
            if errors::anyhow_is_broken_pipe(&e) {
                return errors::exit_codes::EX_PIPE;
            }
            let wants_json = output::wants_json_errors();
            if let Some(ssh_err) = e.downcast_ref::<errors::SshCliError>() {
                return emit_resolved_ssh_error(ssh_err, wants_json);
            }
            if let Some(domain) = e.downcast_ref::<crate::domain::DomainError>() {
                let ssh_err = errors::SshCliError::from(domain.clone());
                return emit_resolved_ssh_error(&ssh_err, wants_json);
            }
            // Walk the chain for DomainError / SshCliError nested under context.
            for cause in e.chain().skip(1) {
                if let Some(ssh_err) = cause.downcast_ref::<errors::SshCliError>() {
                    return emit_resolved_ssh_error(ssh_err, wants_json);
                }
                if let Some(domain) = cause.downcast_ref::<crate::domain::DomainError>() {
                    let ssh_err = errors::SshCliError::from(domain.clone());
                    return emit_resolved_ssh_error(&ssh_err, wants_json);
                }
            }
            let code = errors::exit_codes::EX_GENERAL;
            if wants_json {
                let _ = output::print_error_envelope(
                    code,
                    "unexpected",
                    &e.to_string(),
                    None,
                    errors::ErrorClass::Permanent,
                    false,
                    Some("unexpected non-domain error; do not blind-retry"),
                );
            } else {
                let _ = output::print_error_fmt(format_args!("{e}"));
            }
            let _ = std::io::Write::flush(&mut std::io::stderr());
            code
        }
    }
}

#[cfg(test)]
mod resolve_exit_tests {
    use super::*;
    use crate::errors::{exit_codes, SshCliError};

    #[test]
    fn resolve_ok_is_ex_ok_without_signal() {
        // If a prior test left signal flags set, signal_exit_code wins — only
        // assert the pure Ok path when no signal is active.
        if signals::signal_exit_code().is_none() {
            assert_eq!(resolve_exit_code(Ok(())), exit_codes::EX_OK);
        }
    }

    #[test]
    fn resolve_broken_pipe_is_141() {
        let err = SshCliError::Io(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "pipe",
        ));
        // Prefer signal exit if a concurrent test set flags; otherwise EPIPE.
        let code = resolve_exit_code(Err(err.into()));
        assert!(
            code == exit_codes::EX_PIPE
                || code == exit_codes::EX_SIGINT
                || code == exit_codes::EX_SIGTERM,
            "unexpected exit code {code}"
        );
    }

    #[test]
    fn resolve_auth_failed_is_77_without_signal() {
        if signals::signal_exit_code().is_some() {
            return;
        }
        let code = resolve_exit_code(Err(SshCliError::AuthenticationFailed.into()));
        assert_eq!(code, exit_codes::EX_NOPERM);
    }

    #[test]
    fn resolve_bare_domain_error_is_usage_not_unexpected() {
        if signals::signal_exit_code().is_some() {
            return;
        }
        let d = crate::domain::DomainError::new(
            "vps_auth",
            "primary auth methods are mutually exclusive",
        );
        let code = resolve_exit_code(Err(anyhow::Error::new(d)));
        assert_eq!(code, exit_codes::EX_USAGE);
    }
}
