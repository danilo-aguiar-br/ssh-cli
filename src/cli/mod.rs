// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! CLI argument definitions via `clap` derive and dispatcher.
//!
//! 1. CRUD de VPS — `vps add|list|remove|edit|show|path|doctor|export|import`
//! 2. `connect` — writes sibling `active` file (not a TOML field)
//! 3. One-shot execution — `exec|sudo-exec|su-exec|scp|sftp|tunnel|health-check`
//! 4. `secrets` — primary-key status/init/reencrypt (cifragem at-rest default)
//! 5. Completions / `commands` (agent command-tree discovery)
//!
//! ZERO `.env` at runtime. ZERO telemetry. One-shot cycle: start → dispatch → exit.

mod scp_args;
mod sftp_args;
mod vps_action;
mod commands;
mod path_parse;
mod schema_cmd;

pub use scp_args::ScpAction;
pub use sftp_args::SftpAction;
pub use vps_action::VpsAction;
pub use commands::{
    Command, LocaleAction, SecretsAction, TlsAcmeAccountAction, TlsAcmeAction, TlsAction,
    TlsMtlsAction,
};
pub use schema_cmd::run_schema;
pub(crate) use path_parse::{parse_exec_target, parse_hosts_list, parse_scp_target, ScpPathPlan};

use anyhow::Result;
use clap::{ArgAction, Parser, ValueHint};
use clap_complete::Shell;
use std::path::PathBuf;

/// Output format supported by the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text (default).
    #[default]
    Text,
    /// Structured JSON.
    Json,
}

/// Parses `--max-*-chars` values: decimal `usize`, or `none`/`0` for unlimited.
pub(crate) fn parse_cli_char_limit(s: &str) -> Result<usize, String> {
    let t = s.trim();
    if t.eq_ignore_ascii_case("none") || t == "0" {
        return Ok(0);
    }
    t.parse::<usize>()
        .map_err(|e| format!("invalid char limit '{s}': {e}"))
}

/// Shared SSH authentication overrides (flatten into exec/scp/tunnel/health-check).
///
/// Converted to domain strings at the command boundary (G-08/G-09/G-24).
#[derive(Debug, Clone, Default, clap::Args)]
#[command(next_help_heading = "Authentication")]
pub struct SshAuthArgs {
    /// SSH password override.
    #[arg(long, conflicts_with = "password_stdin")]
    pub password: Option<String>,
    /// Reads the SSH password from stdin.
    #[arg(long, action = ArgAction::SetTrue)]
    pub password_stdin: bool,
    /// Private key path override.
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub key: Option<PathBuf>,
    /// Key passphrase.
    #[arg(long, conflicts_with = "key_passphrase_stdin")]
    pub key_passphrase: Option<String>,
    /// Reads the key passphrase from stdin.
    #[arg(long, action = ArgAction::SetTrue)]
    pub key_passphrase_stdin: bool,
    /// Authenticate via ssh-agent (G-SSH-04). Requires `--agent-socket` on Unix.
    #[arg(long, action = ArgAction::SetTrue)]
    pub use_agent: bool,
    /// Agent socket (Unix) or named pipe (Windows). CLI/XDG only — not env store.
    #[arg(long, value_name = "PATH", value_hint = ValueHint::AnyPath)]
    pub agent_socket: Option<PathBuf>,
}
impl SshAuthArgs {
    /// Domain boundary: `PathBuf` → owned path string for VPS/SSH layers.
    #[must_use]
    pub fn key_path_string(&self) -> Option<String> {
        self.key
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
    }
}

/// Global ssh-cli arguments.
#[derive(Debug, Parser)]
#[command(
    name = crate::constants::APP_NAME,
    version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("SSH_CLI_COMMIT_HASH"), ")"),
    about = "One-shot multi-host XDG Rust CLI for LLMs to operate servers over SSH.",
    long_about = "ssh-cli: lightweight one-shot binary (spawn→run→exit). Multi-host XDG storage without .env. \
Password or key auth. No telemetry.",
    after_help = "Examples:\n  \
ssh-cli vps add --name prod --host h.example --user deploy --key ~/.ssh/id_ed25519\n  \
printf '%s' \"$PASS\" | ssh-cli exec prod 'hostname' --json --password-stdin\n  \
ssh-cli scp upload prod ./a.bin /tmp/a.bin --json\n  \
ssh-cli tunnel prod 8080 127.0.0.1 80 --timeout-ms 60000 --json\n  \
ssh-cli vps export -o /tmp/hosts.toml",
    propagate_version = true,
    arg_required_else_help = true,
    subcommand_required = true,
    next_help_heading = "Global options"
)]
pub struct CliArgs {
    /// Forces the CLI language (BCP47; must negotiate to `en` or `pt-BR`).
    ///
    /// Examples: `en`, `en-US`, `pt-BR`, `pt`. Invalid tags fail clap validation.
    #[arg(
        long,
        global = true,
        value_name = "LOCALE",
        value_parser = crate::locale::parse_lang_cli_arg
    )]
    pub lang: Option<String>,

    /// Increases log verbosity on stderr.
    #[arg(
        short,
        long,
        global = true,
        action = ArgAction::SetTrue,
        conflicts_with = "quiet"
    )]
    pub verbose: bool,

    /// Suppresses non-JSON output (quiet mode).
    #[arg(
        short,
        long,
        global = true,
        action = ArgAction::SetTrue,
        conflicts_with = "verbose"
    )]
    pub quiet: bool,

    /// Configuration directory override (useful for tests).
    #[arg(
        long,
        global = true,
        value_name = "DIR",
        value_hint = ValueHint::DirPath
    )]
    pub config_dir: Option<PathBuf>,

    /// Disables colored output.
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub no_color: bool,

    /// Global output format (text, json). If omitted: JSON when stdout is not a TTY.
    #[arg(long, global = true, value_enum)]
    pub output_format: Option<OutputFormat>,

    /// Force JSON on stdout (agent; alias of `--output-format json`; G-AUD-01).
    ///
    /// Global — appears before or after subcommands. Subcommand fields use
    /// `from_global` so there is a single `--json` long name (clap uniqueness).
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub json: bool,

    /// Disables sudo-exec/su-exec for this invocation (alias --disableSudo).
    #[arg(long, global = true, alias = "disableSudo", action = ArgAction::SetTrue)]
    pub disable_sudo: bool,

    /// Replaces a diverging host key in TOFU known_hosts.
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub replace_host_key: bool,

    /// Allow plaintext secrets at rest (no auto `secrets.key`). Prefer for tests only.
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub allow_plaintext_secrets: bool,

    /// Path to a 64-hex primary-key file (overrides XDG `secrets.key` for this one-shot).
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        value_hint = ValueHint::FilePath
    )]
    pub secrets_key_file: Option<PathBuf>,

    /// Prefer OS keyring for the primary key (CLI flag only; no product env store).
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub use_keyring: bool,

    /// Global default timeout in milliseconds for SSH ops (exec/scp/health-check).
    /// Local `--timeout` on a subcommand wins. Tunnel still requires `--timeout-ms`.
    #[arg(long, global = true, value_name = "MS")]
    pub timeout: Option<u64>,

    /// Cap concurrent multi-host SSH sessions / tunnel forwards (1..=MAX_CONCURRENCY).
    ///
    /// Default: auto from CPUs × I/O oversubscribe vs free RAM (see `concurrency`).
    /// Applies to `--all` fan-out and tunnel accepts (no env store; G-UNSAFE-14).
    #[arg(
        long,
        global = true,
        value_name = "N",
        value_parser = clap::value_parser!(u16).range(1..=(crate::constants::MAX_CONCURRENCY as i64))
    )]
    pub max_concurrency: Option<u16>,

    /// Stop admitting new multi-host units after the first failure (G-O1).
    ///
    /// Default: continue all hosts (agent-friendly partial success). In-flight
    /// units still finish; never-started hosts are omitted from batch results
    /// unless callers pad skipped rows.
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub fail_fast: bool,

    /// Max concurrent SCP file transfers on **one** SSH session (G-O4).
    ///
    /// Default: 1 (serial multi-file, session reuse). Values >1 open parallel
    /// SCP channels on the same session (bounded). Env: not used; CLI only.
    #[arg(
        long,
        global = true,
        value_name = "N",
        value_parser = clap::value_parser!(u16).range(1..=(crate::constants::MAX_CONCURRENCY as i64))
    )]
    pub scp_file_concurrency: Option<u16>,

    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Parses CLI arguments.
#[must_use]
pub fn parse_args() -> CliArgs {
    CliArgs::parse()
}

/// Merges local subcommand timeout with global `--timeout` (local wins).
#[must_use]
pub fn effective_timeout(local: Option<u64>, global: Option<u64>) -> Option<u64> {
    local.or(global)
}

/// Merges local/global timeout and refines to [`crate::domain::TimeoutMs`] (G-TYPE-18).
///
/// # Errors
/// Returns domain error text when the effective value is out of range.
pub fn effective_timeout_ms(
    local: Option<u64>,
    global: Option<u64>,
) -> Result<Option<crate::domain::TimeoutMs>, String> {
    match effective_timeout(local, global) {
        None => Ok(None),
        Some(ms) => crate::domain::TimeoutMs::try_new(ms)
            .map(Some)
            .map_err(|e| e.to_string()),
    }
}

/// Maps CLI `--step` strings into refined remote commands (G-TYPE-19).
///
/// # Errors
/// Returns domain error text when any step is empty or contains NUL.
pub fn parse_remote_steps(
    steps: Vec<String>,
) -> Result<Vec<crate::domain::RemoteCommand>, String> {
    steps
        .into_iter()
        .map(|s| crate::domain::RemoteCommand::try_new(s).map_err(|e| e.to_string()))
        .collect()
}

/// Installs stderr tracing before clap parse (delegates to [`crate::telemetry`]).
#[inline]
pub fn bootstrap_logs() {
    crate::telemetry::bootstrap_logs();
}

/// Reloads the tracing filter from CLI flags (delegates to [`crate::telemetry`]).
#[inline]
pub fn initialize_logs(args: &CliArgs) {
    crate::telemetry::initialize_logs(args.verbose);
}

/// Writes shell completions to stdout.
///
/// GAP-SSH-CLI-003 / G-IO-08: broken pipe (EPIPE) does not panic — returns
/// [`crate::errors::SshCliError::Io`] so `main` exits **141**.
///
/// # Errors
/// Stdout write failures (including BrokenPipe).
pub fn generate_completions(shell: Shell) -> Result<()> {
    use clap::CommandFactory;
    use std::io::Write;
    let mut cmd = CliArgs::command();
    let mut buf: Vec<u8> = Vec::new();
    clap_complete::generate(shell, &mut cmd, crate::constants::APP_NAME, &mut buf);
    let mut out = std::io::stdout().lock();
    out.write_all(&buf).and_then(|()| out.flush())?;
    Ok(())
}

/// Builds a JSON command tree from the clap `Command` graph (G-IO-10).
#[must_use]
pub fn command_tree_json() -> serde_json::Value {
    use clap::CommandFactory;
    fn walk(cmd: &clap::Command) -> serde_json::Value {
        let name = cmd.get_name().to_string();
        let about = cmd.get_about().map(|s| s.to_string());
        let mut children = Vec::new();
        for sub in cmd.get_subcommands() {
            if sub.is_hide_set() {
                continue;
            }
            children.push(walk(sub));
        }
        serde_json::json!({
            "name": name,
            "about": about,
            "subcommands": children,
        })
    }
    let root = CliArgs::command();
    serde_json::json!({
        "ok": true,
        "event": "commands",
        "bin": root.get_name(),
        "version": env!("CARGO_PKG_VERSION"),
        "tree": walk(&root),
    })
}

/// Renders a man page for `ssh-cli` (G-12 / clap_mangen).
pub fn render_manpage() -> Result<Vec<u8>, std::io::Error> {
    use clap::CommandFactory;
    use std::io::Write;
    let cmd = CliArgs::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buf = Vec::new();
    man.render(&mut buf)?;
    // Ensure trailing newline for POSIX man consumers.
    if !buf.ends_with(b"\n") {
        buf.write_all(b"\n")?;
    }
    Ok(buf)
}

/// Resolves a secret from `--*-stdin` or an argv value into [`secrecy::SecretString`].
///
/// G-SECDEV-01: wrap credentials at the CLI boundary — never forward bare
/// `String` passwords into exec/scp/tunnel/health overrides.
pub(crate) fn read_stdin_if(
    flag: bool,
    value: Option<String>,
) -> Result<Option<secrecy::SecretString>> {
    if flag {
        Ok(Some(crate::vps::read_secret_stdin()?))
    } else {
        Ok(value.map(secrecy::SecretString::from))
    }
}

/// Warns only when a secret value is present on argv (not stdin flags).
///
/// G-AUD-08: inspect concrete `Option` fields — never `Debug` string heuristics
/// (`password: None` + any `Some(` elsewhere was a false positive).
pub(crate) fn warn_if_password_argv(args: &CliArgs) {
    let has = match &args.command {
        Command::Exec { auth, .. }
        | Command::HealthCheck { auth, .. }
        | Command::Tunnel { auth, .. } => {
            auth.password.is_some() || auth.key_passphrase.is_some()
        }
        Command::SudoExec {
            auth,
            sudo_password,
            ..
        } => {
            auth.password.is_some()
                || auth.key_passphrase.is_some()
                || sudo_password.is_some()
        }
        Command::SuExec {
            auth,
            su_password,
            ..
        } => {
            auth.password.is_some() || auth.key_passphrase.is_some() || su_password.is_some()
        }
        Command::Scp { action } => match action {
            ScpAction::Upload { auth, .. } | ScpAction::Download { auth, .. } => {
                auth.password.is_some() || auth.key_passphrase.is_some()
            }
        },
        Command::Sftp { action } => sftp_auth_has_argv_secret(action),
        Command::Vps { action } => vps_action_has_argv_secret(action),
        _ => false,
    };

    if has {
        crate::output::print_warning(
            "a password-like value was passed on the command line (visible in process lists); prefer --*-stdin",
        );
    }
}

fn sftp_auth_has_argv_secret(action: &SftpAction) -> bool {
    let auth = match action {
        SftpAction::Upload { auth, .. }
        | SftpAction::Download { auth, .. }
        | SftpAction::Ls { auth, .. }
        | SftpAction::Mkdir { auth, .. }
        | SftpAction::Rmdir { auth, .. }
        | SftpAction::Rm { auth, .. }
        | SftpAction::Rename { auth, .. }
        | SftpAction::Stat { auth, .. } => auth,
    };
    auth.password.is_some() || auth.key_passphrase.is_some()
}

fn vps_action_has_argv_secret(action: &VpsAction) -> bool {
    match action {
        VpsAction::Add {
            password,
            key_passphrase,
            sudo_password,
            su_password,
            ..
        }
        | VpsAction::Edit {
            password,
            key_passphrase,
            sudo_password,
            su_password,
            ..
        } => {
            password.is_some()
                || key_passphrase.is_some()
                || sudo_password.is_some()
                || su_password.is_some()
        }
        _ => false,
    }
}

/// Resolves output format: `--json` global / explicit enum > non-TTY JSON > Text.
///
/// G-AUD-01/12: no `SSH_CLI_FORCE_TEXT` env store — use `--output-format text`.
#[must_use]
pub fn resolve_format(explicit: Option<OutputFormat>) -> OutputFormat {
    if let Some(f) = explicit {
        return f;
    }
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    }
}

/// Resolves format from global `--json` + `--output-format` (G-AUD-01).
///
/// `# Errors`
/// `--json` together with `--output-format text`.
pub fn resolve_format_from_cli(
    json: bool,
    explicit: Option<OutputFormat>,
) -> Result<OutputFormat, crate::errors::SshCliError> {
    // G-AUD-01: `--json` always wins (including when tests pass `--output-format text`
    // for human stderr isolation while still requesting JSON success bodies).
    if json {
        return Ok(OutputFormat::Json);
    }
    Ok(resolve_format(explicit))
}


mod dispatch;

pub use dispatch::{dispatch, dispatch_impl};

#[cfg(test)]
mod tests;
