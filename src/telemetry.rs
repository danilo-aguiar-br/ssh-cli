// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Process-local **tracing** setup for the ssh-cli binary path.
//!
//! # Product identity (Rules Rust — logs / tracing / rotation)
//!
//! ssh-cli is a **one-shot agent CLI**, not a long-lived server:
//!
//! | Signal | Policy |
//! |--------|--------|
//! | Facade | `tracing` only (no `println!` / `env_logger` / dual `log` consumer) |
//! | Sink | **stderr** text (data JSON stays on **stdout**) |
//! | Default filter | `error` (agent-first quiet stderr) |
//! | Override | **CLI only:** `-v` → `debug`; ambient `RUST_LOG` is **ignored**; `-q` does not change the filter |
//! | Reload | `reload::Layer` so bootstrap (pre-parse) can reconfigure after argv |
//! | Bridge | `tracing-log::LogTracer` for deps that emit via the `log` crate (`russh`, `keyring`) |
//! | Errors | `tracing_error::ErrorLayer` for `SpanTrace` capture |
//!
//! # Explicitly out of scope
//!
//! - OpenTelemetry / OTLP / metrics backends (product: **zero telemetry**)
//! - `tracing-appender` file rotation + `WorkerGuard` (no local log files; short process)
//! - Admin HTTP `/admin/log-level` (no daemon / no network control plane)
//! - `tokio-console` / Chrome tracing / journald / Docker log drivers as product features
//! - Encrypted log-at-rest (no log files written by this binary)
//!
//! Libraries that depend on `ssh_cli` as a crate should **not** call these
//! installers; only the binary entry (`run` → `bootstrap_logs`) installs the
//! global subscriber. Product modules only emit events/spans.
//!
//! # Lifecycle
//!
//! 1. [`bootstrap_logs`] — before clap parse (phase 1b).
//! 2. [`initialize_logs`] — after parse, reloads `EnvFilter` from `-v` only (G-E2E-09).
//! 3. Process exit — `main` flushes stderr; no file worker to join.

use std::sync::OnceLock;

use tracing_error::ErrorLayer;
use tracing_subscriber::reload;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

/// Reload handle: bootstrap installs once; [`initialize_logs`] reloads the filter.
///
/// `OnceLock` (not `lazy_static`): value is created only when the binary path
/// installs the global subscriber.
static LOG_FILTER_RELOAD: OnceLock<reload::Handle<EnvFilter, Registry>> = OnceLock::new();

/// Builds the process-local tracing filter from CLI `--verbose` only (G-AUD-22).
///
/// - `verbose == true` (`-v`) → `debug`
/// - otherwise → `error`
///
/// Ambient `RUST_LOG` is **ignored** (not an env store of product config).
/// `--quiet` affects human stdout only ([`crate::output::set_quiet`]).
#[must_use]
pub fn build_env_filter(verbose: bool) -> EnvFilter {
    if verbose {
        return EnvFilter::new("debug");
    }
    EnvFilter::new("error")
}

/// Installs stderr tracing **before** clap parse (one-shot lifecycle phase 1b).
///
/// Default filter is `error` so agents stay quiet; [`initialize_logs`] reloads
/// from `-v` after parse (ambient `RUST_LOG` ignored).
///
/// Safe to call more than once: subsequent calls are no-ops once the reload
/// handle is stored (or if another test subscriber already owns the global).
pub fn bootstrap_logs() {
    if LOG_FILTER_RELOAD.get().is_some() {
        return;
    }

    let (filter_layer, handle) = reload::Layer::new(EnvFilter::new("error"));
    let subscriber = Registry::default()
        .with(filter_layer)
        .with(ErrorLayer::default())
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                // Targets remain in the log line for human diagnostics.
                .with_target(true)
                // Tokio workers are named `ssh-cli-worker` in `main`.
                .with_thread_names(true)
                // Agents / CI: never emit ANSI on the diagnostics channel.
                .with_ansi(false),
        );

    // Prefer `set_global_default` so the reload handle stays valid.
    // Ignore failure when tests already installed a subscriber.
    if tracing::subscriber::set_global_default(subscriber).is_ok() {
        let _ = LOG_FILTER_RELOAD.set(handle);
        // Bridge `log` crate records (russh, keyring, …) into `tracing`.
        // Ignore if already installed (re-entrant tests).
        let _ = tracing_log::LogTracer::builder()
            .with_max_level(log::LevelFilter::Trace)
            .init();
        tracing::debug!("tracing subscriber installed (stderr, filter=error)");
    }
}

/// Initializes or reloads `tracing-subscriber` from the verbose CLI flag.
///
/// GAP-SSH-LOG-001 / G-AUD-22: default **error** (agent-first). `-v` → debug.
/// Ambient `RUST_LOG` is ignored (CLI-only filter).
pub fn initialize_logs(verbose: bool) {
    let filter = build_env_filter(verbose);
    if let Some(handle) = LOG_FILTER_RELOAD.get() {
        match handle.reload(filter) {
            Ok(()) => {
                // Visible only when the new filter admits `debug` (e.g. `-v`).
                tracing::debug!(
                    verbose,
                    rust_log_set = std::env::var_os("RUST_LOG").is_some(),
                    "tracing filter reloaded"
                );
            }
            Err(e) => {
                // Keep the previous filter; surface the reload failure.
                tracing::warn!(err = %e, "failed to reload tracing filter");
            }
        }
        return;
    }

    // Tests / alternate entry: no bootstrap — try_init once (no reload handle).
    let _ = fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(true)
        .with_thread_names(true)
        .with_ansi(false)
        .try_init();
    let _ = tracing_log::LogTracer::builder()
        .with_max_level(log::LevelFilter::Trace)
        .init();
}

/// Returns whether the process owns a reloadable global filter (binary path).
#[must_use]
pub fn has_reload_handle() -> bool {
    LOG_FILTER_RELOAD.get().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_env_filter_default_is_error() {
        // Isolate from ambient RUST_LOG in the test runner.
        let prev = std::env::var_os("RUST_LOG");
        crate::test_util::env::remove_var("RUST_LOG");
        let f = build_env_filter(false);
        assert_eq!(f.to_string(), "error");
        match prev {
            Some(v) => crate::test_util::env::set_var("RUST_LOG", v),
            None => crate::test_util::env::remove_var("RUST_LOG"),
        }
    }

    #[test]
    fn build_env_filter_verbose_is_debug() {
        let prev = std::env::var_os("RUST_LOG");
        crate::test_util::env::remove_var("RUST_LOG");
        let f = build_env_filter(true);
        assert_eq!(f.to_string(), "debug");
        match prev {
            Some(v) => crate::test_util::env::set_var("RUST_LOG", v),
            None => crate::test_util::env::remove_var("RUST_LOG"),
        }
    }

    #[test]
    fn bootstrap_logs_is_idempotent() {
        bootstrap_logs();
        bootstrap_logs();
        // In a full binary path the handle is set; under parallel tests another
        // suite may have taken the global subscriber first — both outcomes OK.
        let _ = has_reload_handle();
    }
}
