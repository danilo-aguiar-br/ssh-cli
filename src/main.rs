// SPDX-License-Identifier: MIT OR Apache-2.0
//! Entry point of the ssh-cli binary.
//!
//! Keeps logic minimal: configures the tokio runtime and calls `ssh_cli::run()`.

#[cfg(feature = "musl-allocator")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("failed to create runtime: {e}");
            std::process::exit(ssh_cli::erros::exit_codes::EX_IOERR);
        }
    };

    let result = runtime.block_on(ssh_cli::run());

    match result {
        Ok(()) => {
            if ssh_cli::signals::is_terminated() {
                std::process::exit(ssh_cli::erros::exit_codes::EX_SIGTERM);
            }
            if ssh_cli::signals::is_cancelled() {
                std::process::exit(ssh_cli::erros::exit_codes::EX_SIGINT);
            }
            std::process::exit(ssh_cli::erros::exit_codes::EX_OK);
        }
        Err(e) => {
            if ssh_cli::signals::is_terminated() {
                std::process::exit(ssh_cli::erros::exit_codes::EX_SIGTERM);
            }
            if ssh_cli::signals::is_cancelled() {
                std::process::exit(ssh_cli::erros::exit_codes::EX_SIGINT);
            }
            let quer_json = ssh_cli::output::wants_json_errors();
            if let Some(erro_ssh) = e.downcast_ref::<ssh_cli::erros::SshCliError>() {
                let code = erro_ssh.exit_code();
                let remote = match erro_ssh {
                    ssh_cli::erros::SshCliError::CommandFailed { exit_code, .. } => Some(*exit_code),
                    _ => None,
                };
                if quer_json {
                    let _ = ssh_cli::output::print_error_envelope(
                        code,
                        &erro_ssh.to_string(),
                        remote,
                    );
                } else {
                    eprintln!("{erro_ssh}");
                }
                std::process::exit(code);
            }
            let code = ssh_cli::erros::exit_codes::EX_GENERAL;
            if quer_json {
                let _ = ssh_cli::output::print_error_envelope(code, &e.to_string(), None);
            } else {
                eprintln!("{e}");
            }
            std::process::exit(code);
        }
    }
}
