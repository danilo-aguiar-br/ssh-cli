// SPDX-License-Identifier: MIT OR Apache-2.0
// G-CLOSE-04 / G-CLOSE-09: command layer (clap CAMADA 2) — real thin handlers.
#![forbid(unsafe_code)]
//! Per-subcommand command layer (rules clap CAMADA 2).
//!
//! CLI types live in [`crate::cli`]; this module owns the dispatch entry so
//! `main` / `lib::run` route through a stable command surface.

use crate::cli::CliArgs;
use anyhow::Result;

/// Runs the requested subcommand (command-layer entry).
pub async fn run(args: CliArgs) -> Result<()> {
    crate::cli::dispatch_impl(args).await
}

/// Exec-family handlers (domain: [`crate::vps`] exec_ops).
pub mod exec {
    pub use crate::vps::{
        run_exec as run, run_su_exec as run_su, run_sudo_exec as run_sudo, ExecOptions,
    };
}

/// VPS inventory handlers.
pub mod vps {
    pub use crate::vps::{run_connect as connect, run_vps_command as run};
}

/// SCP handlers.
pub mod scp {
    pub use crate::scp::{run_scp as run, ScpOptions};
}

/// SFTP handlers (G-SFTP).
pub mod sftp {
    pub use crate::sftp::{run_sftp as run, SftpOptions};
}

/// Tunnel handler.
pub mod tunnel {
    pub use crate::tunnel::run_tunnel as run;
}

/// Health-check handler.
pub mod health {
    pub use crate::vps::run_health_check as run;
}

/// Secrets handlers.
pub mod secrets {
    pub use crate::vps::run_secrets_command as run;
}

/// Completions generation.
pub mod completions {
    pub use crate::cli::generate_completions as run;
}
