// SPDX-License-Identifier: MIT OR Apache-2.0
// G-CLOSE-04: pure module — no `unsafe` permitted.
#![forbid(unsafe_code)]
//! Entry point of the ssh-cli binary.
//!
//! Keeps logic minimal (G-IO-11 thin main):
//! 1. Optional `human_panic` in release
//! 2. **Signal handlers** (SIGINT/SIGTERM) — before any worker threads
//! 3. Build Tokio multi_thread runtime (capped workers)
//! 4. `block_on(ssh_cli::run())`
//! 5. Flush stdout
//! 6. [`ssh_cli::resolve_exit_code`] (library owns exit policy + error envelopes)
//! 7. Runtime shutdown timeout, then `process::exit`
//!
//! # Graceful shutdown (one-shot)
//!
//! 1. Cooperative cancel flags set by SIGINT/SIGTERM (see `signals`).
//! 2. Command paths return after polling `should_stop` / disconnecting SSH.
//! 3. This `main` flushes stdout, maps the result to an exit code, shuts the
//!    runtime down, then exits **0** / **130** / **141** / **143** / domain codes.
//!
//! `process::exit` is used only **after** flush + runtime shutdown so buffers
//! and worker threads are not abandoned mid-write.

#[cfg(feature = "musl-allocator")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    // G-15: human-readable panic reports in release; agent JSON path still uses Result.
    #[cfg(not(debug_assertions))]
    human_panic::setup_panic!();

    // G-TLS: install rustls CryptoProvider (aws_lc_rs) once before any TLS dial
    // and before the Tokio runtime (rules: install → logs → config → runtime → sockets).
    // Libraries never call install_default; only this binary bootstrap does.
    if let Err(e) = ssh_cli::tls::install_default_provider() {
        let _ = ssh_cli::output::print_error_fmt(format_args!(
            "failed to install rustls CryptoProvider: {e}"
        ));
        std::process::exit(ssh_cli::errors::exit_codes::EX_IOERR);
    }

    // G-UNSAFE-13: register SIGINT/SIGTERM *before* Tokio multi_thread workers
    // start. signal-hook documents a first-hook race if other threads already
    // exist; binary bootstrap is still single-threaded here. `run()` re-calls
    // register_handler which is idempotent (`Once`).
    if let Err(e) = ssh_cli::signals::register_handler() {
        let _ = ssh_cli::output::print_error_fmt(format_args!(
            "failed to register signal handlers: {e}"
        ));
        std::process::exit(ssh_cli::errors::exit_codes::EX_IOERR);
    }

    // Workload: I/O-bound one-shot (SSH/TCP + multi-host fan-out + tunnel).
    // Not CPU-bound — no Rayon. multi_thread (not Runtime::new / current_thread):
    // russh crypto + concurrent sessions + tunnel accepts need work-stealing;
    // current_thread would serialize multi-host ops.
    //
    // Workers follow auto CPU×RAM formula (pre-parse); CLI `--max-concurrency`
    // is applied after parse for fan-out gates via `install_process_limit`.
    // Optional mimalloc via `musl-allocator`.
    let workers = ssh_cli::concurrency::worker_threads();
    let max_blocking = ssh_cli::concurrency::max_blocking_threads();
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .max_blocking_threads(max_blocking)
        .thread_name("ssh-cli-worker")
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            // Bootstrap path: lib not yet initialized; stderr is the only channel.
            // G-MAC-01: format_args + write_fmt (no temporary String).
            let _ = ssh_cli::output::print_error_fmt(format_args!(
                "failed to create runtime: {e}"
            ));
            std::process::exit(ssh_cli::errors::exit_codes::EX_IOERR);
        }
    };

    let result = runtime.block_on(ssh_cli::run());

    // G-IO-09 / graceful shutdown: flush stdout before deciding the exit code.
    let _ = std::io::Write::flush(&mut std::io::stdout());

    // Exit policy + error envelopes live in the library (G-IO-11).
    let code = ssh_cli::resolve_exit_code(result);

    // Graceful runtime teardown before process::exit (drops workers; avoids
    // abandoning in-flight IO after cooperative cancel returned).
    // 2s is enough for one-shot; tunnel forwards are aborted in tunnel.rs first.
    runtime.shutdown_timeout(std::time::Duration::from_secs(
        ssh_cli::constants::RUNTIME_SHUTDOWN_TIMEOUT_SECS,
    ));

    std::process::exit(code);
}
