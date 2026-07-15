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
//! | `cli`           | Clap derive argument definitions and dispatcher               |
//! | `vps`           | CRUD and persistence of VPS records (XDG + TOML + 0o600)      |
//! | `secrets`       | Primary key and default at-rest encryption (ChaCha20-Poly1305)|
//! | `ssh`           | Real one-shot SSH client via `russh` (password/key, TOFU)     |
//! | `i18n`          | Internationalization with bilingual `Message` enum            |
//! | `locale`        | OS locale detection and resolution                            |
//! | `platform`      | Platform adjustments (Windows UTF-8, TTY detection)           |
//! | `masking`       | Unicode-safe masking of sensitive values                      |
//! | `errors`        | Structured error types via `thiserror`                        |
//! | `output`        | Sole module authorized for `println!` (CRUD formatting)       |
//! | `paths`         | Path validation and normalization (anti-traversal, NFC)       |
//! | `signals`       | Ctrl+C handler with cancellation flag via `AtomicBool`        |
//! | `terminal`      | TTY detection and color choice via `termcolor`                |
//!
//! ## Entry point
//!
//! The public [`run`] function is the entry point called by `main.rs`.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::multiple_unsafe_ops_per_block)]
#![warn(unsafe_op_in_unsafe_fn)]

pub mod cli;
pub mod errors;
pub mod erros;
pub mod i18n;
pub mod locale;
pub mod masking;
pub mod output;
pub mod paths;
pub mod platform;
pub mod scp;
pub mod secrets;
pub mod signals;
pub mod ssh;
pub mod terminal;
pub mod tunnel;
pub mod vps;

use anyhow::Result;

/// Runs ssh-cli from the command-line arguments.
///
/// Entry point called by `main.rs`. It:
/// 1. Registers the Ctrl+C handler for graceful cancellation.
/// 2. Initializes the platform (Windows UTF-8 code page, TTY detection).
/// 3. Parses arguments via clap.
/// 4. Initializes logging via `tracing-subscriber`.
/// 5. Initializes terminal color configuration.
/// 6. Initializes i18n with the detected language.
/// 7. Dispatches to the appropriate subcommand (`vps`, `connect`, `exec`, `sudo-exec`, `scp`, `tunnel`).
pub async fn run() -> Result<()> {
    signals::register_handler()?;

    platform::initialize_platform()?;

    let argumentos = cli::parse_args();

    cli::initialize_logs(&argumentos);

    terminal::initialize(argumentos.no_color)?;

    i18n::initialize_language(argumentos.lang.as_deref())?;

    cli::dispatch(argumentos).await
}
