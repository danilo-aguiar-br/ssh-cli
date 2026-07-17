// SPDX-License-Identifier: MIT OR Apache-2.0
//! CLI argument definitions via `clap` derive and dispatcher.
//!
//! 1. CRUD de VPS — `vps add|list|remove|edit|show|path|doctor|export|import`
//! 2. `connect` — writes sibling `active` file (not a TOML field)
//! 3. One-shot execution — `exec|sudo-exec|su-exec|scp|tunnel|health-check`
//! 4. `secrets` — primary-key status/init/reencrypt (cifragem at-rest default)
//! 5. Completions
//!
//! ZERO `.env` em runtime. ZERO telemetria. Ciclo one-shot: nascer → dispatch → morrer.

use anyhow::Result;
use clap::{Parser, Subcommand};
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

/// Global ssh-cli arguments.
#[derive(Debug, Parser)]
#[command(
    name = "ssh-cli",
    version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("SSH_CLI_COMMIT_HASH"), ")"),
    about = "One-shot multi-host XDG Rust CLI for LLMs to operate servers over SSH.",
    long_about = "ssh-cli: lightweight one-shot binary (spawn→run→exit). Multi-host XDG storage without .env. \
Password or key auth. No telemetry."
)]
pub struct CliArgs {
    /// Forces the CLI language (e.g. `pt-BR`, `en-US`).
    #[arg(long, global = true, value_name = "LOCALE")]
    pub lang: Option<String>,

    /// Increases log verbosity on stderr.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppresses non-JSON output (quiet mode).
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Configuration directory override (useful for tests).
    #[arg(long, global = true, value_name = "DIR")]
    pub config_dir: Option<PathBuf>,

    /// Disables colored output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Global output format (text, json). If omitted: JSON when stdout is not a TTY.
    #[arg(long, global = true, value_enum)]
    pub output_format: Option<OutputFormat>,

    /// Disables sudo-exec/su-exec for this invocation (alias --disableSudo).
    #[arg(long, global = true, alias = "disableSudo")]
    pub disable_sudo: bool,

    /// Replaces a diverging host key in TOFU known_hosts.
    #[arg(long, global = true)]
    pub replace_host_key: bool,

    /// Allow plaintext secrets at rest (no auto `secrets.key`). Prefer for tests only.
    #[arg(long, global = true)]
    pub allow_plaintext_secrets: bool,

    /// Path to a 64-hex primary-key file (overrides env / XDG secrets.key for this one-shot).
    #[arg(long, global = true, value_name = "PATH")]
    pub secrets_key_file: Option<PathBuf>,

    /// Prefer OS keyring for the primary key (deprecated env: SSH_CLI_USE_KEYRING).
    #[arg(long, global = true)]
    pub use_keyring: bool,

    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Manages registered VPS hosts.
    Vps {
        /// Specific VPS CRUD action.
        #[command(subcommand)]
        action: VpsAction,
    },

    /// Sets the active VPS (writes sibling `active` file in the config directory).
    Connect {
        /// Name of the VPS previously added via `vps add`.
        name: String,
    },

    /// Runs a command on the VPS over SSH (stdout/stderr captured).
    Exec {
        /// VPS name.
        vps_name: String,
        /// Shell command to run.
        command: String,
        /// JSON output.
        #[arg(long)]
        json: bool,
        /// SSH password override.
        #[arg(long, conflicts_with = "password_stdin")]
        password: Option<String>,
        /// Reads the SSH password from stdin.
        #[arg(long)]
        password_stdin: bool,
        /// Private key path override.
        #[arg(long)]
        key: Option<String>,
        /// Key passphrase (runtime).
        #[arg(long, conflicts_with = "key_passphrase_stdin")]
        key_passphrase: Option<String>,
        /// Reads the key passphrase from stdin.
        #[arg(long)]
        key_passphrase_stdin: bool,
        /// Timeout override in milliseconds.
        #[arg(long)]
        timeout: Option<u64>,
        /// Shell comment appended for audit trails.
        #[arg(long)]
        description: Option<String>,
    },

    /// Runs a command with `sudo` (safe `sh -c` packing).
    SudoExec {
        /// VPS name.
        vps_name: String,
        /// Shell command.
        command: String,
        /// JSON output.
        #[arg(long)]
        json: bool,
        /// SSH password override.
        #[arg(long, conflicts_with = "password_stdin")]
        password: Option<String>,
        /// Reads the SSH password from stdin.
        #[arg(long)]
        password_stdin: bool,
        /// Sudo password override.
        #[arg(
            long,
            alias = "sudoPassword",
            alias = "sudo_password",
            conflicts_with = "sudo_password_stdin"
        )]
        sudo_password: Option<String>,
        /// Reads the sudo password from stdin.
        #[arg(long)]
        sudo_password_stdin: bool,
        /// Key path override.
        #[arg(long)]
        key: Option<String>,
        /// Key passphrase (runtime).
        #[arg(long, conflicts_with = "key_passphrase_stdin")]
        key_passphrase: Option<String>,
        /// Reads the key passphrase from stdin.
        #[arg(long)]
        key_passphrase_stdin: bool,
        /// Timeout override in milliseconds.
        #[arg(long)]
        timeout: Option<u64>,
        /// Shell comment appended for audit.
        #[arg(long)]
        description: Option<String>,
    },

    /// Runs a command with one-shot `su -` elevation.
    SuExec {
        /// VPS name.
        vps_name: String,
        /// Shell command.
        command: String,
        /// JSON output.
        #[arg(long)]
        json: bool,
        /// SSH password override.
        #[arg(long, conflicts_with = "password_stdin")]
        password: Option<String>,
        /// Reads the SSH password from stdin (GAP-SSH-CLI-001).
        #[arg(long)]
        password_stdin: bool,
        /// Su password override.
        #[arg(
            long,
            alias = "suPassword",
            alias = "su_password",
            conflicts_with = "su_password_stdin"
        )]
        su_password: Option<String>,
        /// Reads the su password from stdin.
        #[arg(long)]
        su_password_stdin: bool,
        /// Key path override.
        #[arg(long)]
        key: Option<String>,
        /// Key passphrase (runtime).
        #[arg(long, conflicts_with = "key_passphrase_stdin")]
        key_passphrase: Option<String>,
        /// Reads the key passphrase from stdin.
        #[arg(long)]
        key_passphrase_stdin: bool,
        /// Timeout override.
        #[arg(long)]
        timeout: Option<u64>,
        /// Shell comment appended for audit.
        #[arg(long)]
        description: Option<String>,
    },

    /// SCP file transfer (upload/download).
    Scp {
        /// Specific SCP action.
        #[command(subcommand)]
        action: ScpAction,
    },

    /// SSH tunnel with mandatory deadline (bounded one-shot).
    Tunnel {
        /// VPS name.
        vps_name: String,
        /// Local port.
        local_port: u16,
        /// Remote host.
        remote_host: String,
        /// Remote port.
        remote_port: u16,
        /// Mandatory tunnel timeout in milliseconds.
        #[arg(long)]
        timeout_ms: u64,
        /// SSH password override.
        #[arg(long, conflicts_with = "password_stdin")]
        password: Option<String>,
        /// Reads the SSH password from stdin (GAP-SSH-CLI-005).
        #[arg(long)]
        password_stdin: bool,
        /// Private key path override.
        #[arg(long)]
        key: Option<String>,
        /// Key passphrase.
        #[arg(long, conflicts_with = "key_passphrase_stdin")]
        key_passphrase: Option<String>,
        /// Reads the key passphrase from stdin (GAP-SSH-CLI-005).
        #[arg(long)]
        key_passphrase_stdin: bool,
        /// Agent-first JSON output when the local listener is up (GAP-SSH-IO-008).
        #[arg(long)]
        json: bool,
        /// Local bind address (default 127.0.0.1 loopback for security).
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
    },

    /// Checks SSH connectivity to a VPS.
    HealthCheck {
        /// VPS name (uses active if omitted).
        vps_name: Option<String>,
        /// JSON output (GAP-SSH-IO-002).
        #[arg(long)]
        json: bool,
        /// SSH password override.
        #[arg(long, conflicts_with = "password_stdin")]
        password: Option<String>,
        /// Reads the SSH password from stdin (GAP-SSH-CLI-006).
        #[arg(long)]
        password_stdin: bool,
        /// Private key path override (GAP-SSH-CLI-006).
        #[arg(long)]
        key: Option<String>,
        /// Key passphrase.
        #[arg(long, conflicts_with = "key_passphrase_stdin")]
        key_passphrase: Option<String>,
        /// Reads the key passphrase from stdin (GAP-SSH-CLI-006).
        #[arg(long)]
        key_passphrase_stdin: bool,
        /// SSH timeout override in milliseconds (GAP-SSH-CLI-004).
        #[arg(long)]
        timeout: Option<u64>,
    },

    /// Manages the primary key and at-rest secret encryption (one-shot).
    Secrets {
        /// Secrets action.
        #[command(subcommand)]
        action: SecretsAction,
    },

    /// Generates shell completions.
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Actions of the `vps` subcommand.
#[derive(Debug, Subcommand)]
pub enum VpsAction {
    /// Adds a new VPS to the registry.
    Add {
        /// Unique VPS name.
        #[arg(long)]
        name: String,
        /// Hostname or IP.
        #[arg(long)]
        host: String,
        /// SSH port.
        #[arg(long, default_value_t = 22)]
        port: u16,
        /// SSH username.
        #[arg(long)]
        user: String,
        /// SSH password.
        #[arg(long, conflicts_with = "password_stdin")]
        password: Option<String>,
        /// Reads the password from stdin.
        #[arg(long)]
        password_stdin: bool,
        /// OpenSSH private key path.
        #[arg(long)]
        key: Option<String>,
        /// Key passphrase.
        #[arg(long)]
        key_passphrase: Option<String>,
        /// Timeout in milliseconds (default 60000).
        #[arg(long, default_value_t = 60_000)]
        timeout: u64,
        /// Command character limit (input). Legacy alias: maxChars.
        #[arg(long)]
        max_command_chars: Option<String>,
        /// Output character limit.
        #[arg(long)]
        max_output_chars: Option<String>,
        /// Legacy alias: maps to max_command_chars.
        #[arg(long, alias = "maxChars")]
        max_chars: Option<String>,
        /// Password for `sudo`.
        #[arg(
            long,
            alias = "sudoPassword",
            alias = "sudo_password",
            conflicts_with = "sudo_password_stdin"
        )]
        sudo_password: Option<String>,
        /// Reads the sudo password from stdin.
        #[arg(long)]
        sudo_password_stdin: bool,
        /// Password for `su -`.
        #[arg(
            long,
            alias = "suPassword",
            alias = "su_password",
            conflicts_with = "su_password_stdin"
        )]
        su_password: Option<String>,
        /// Reads the su password from stdin.
        #[arg(long)]
        su_password_stdin: bool,
        /// Disables sudo/su on this host.
        #[arg(long, default_value_t = false)]
        disable_sudo: bool,
        /// Runs health-check after add.
        #[arg(long)]
        check: bool,
    },

    /// Lists all VPS hosts (passwords masked).
    List {
        /// JSON output.
        #[arg(long)]
        json: bool,
    },

    /// Removes a VPS from the registry.
    Remove {
        /// VPS name to remove.
        name: String,
    },

    /// Edits fields of an existing VPS.
    Edit {
        /// VPS name to edit.
        name: String,
        /// New hostname/IP.
        #[arg(long)]
        host: Option<String>,
        /// New SSH port.
        #[arg(long)]
        port: Option<u16>,
        /// New username.
        #[arg(long)]
        user: Option<String>,
        /// New password.
        #[arg(long, conflicts_with = "password_stdin")]
        password: Option<String>,
        /// Reads the password from stdin.
        #[arg(long)]
        password_stdin: bool,
        /// New private key path.
        #[arg(long)]
        key: Option<String>,
        /// New key passphrase.
        #[arg(long)]
        key_passphrase: Option<String>,
        /// New timeout.
        #[arg(long)]
        timeout: Option<u64>,
        /// New max command chars.
        #[arg(long)]
        max_command_chars: Option<String>,
        /// New max output chars.
        #[arg(long)]
        max_output_chars: Option<String>,
        /// Legacy alias maxChars → command.
        #[arg(long, alias = "maxChars")]
        max_chars: Option<String>,
        /// New sudo password.
        #[arg(
            long,
            alias = "sudoPassword",
            alias = "sudo_password",
            conflicts_with = "sudo_password_stdin"
        )]
        sudo_password: Option<String>,
        /// Reads the sudo password from stdin.
        #[arg(long)]
        sudo_password_stdin: bool,
        /// New su password.
        #[arg(
            long,
            alias = "suPassword",
            alias = "su_password",
            conflicts_with = "su_password_stdin"
        )]
        su_password: Option<String>,
        /// Reads the su password from stdin.
        #[arg(long)]
        su_password_stdin: bool,
        /// Sets disable_sudo.
        #[arg(long)]
        disable_sudo: Option<bool>,
    },

    /// Shows VPS details (passwords masked).
    Show {
        /// VPS name.
        name: String,
        /// JSON output.
        #[arg(long)]
        json: bool,
    },

    /// Shows the configuration file path.
    Path,

    /// Diagnostics for XDG layers / path / schema.
    Doctor {
        /// JSON output.
        #[arg(long)]
        json: bool,
    },

    /// Exports hosts (passwords redacted by default).
    Export {
        /// Include secrets in the export.
        #[arg(long)]
        include_secrets: bool,
        /// Output file (stdout if omitted). Written atomically with mode 0o600.
        #[arg(long, short)]
        output: Option<String>,
        /// Agent-first JSON envelope (`event: vps-export`). Default body is **TOML**
        /// even on non-TTY pipes (GAP-AUD-001/022). Redacted unless `--include-secrets`.
        #[arg(long)]
        json: bool,
        /// Acknowledge writing plaintext secrets to stdout (pipe/non-TTY). Prefer `--output`.
        #[arg(long)]
        i_understand_secrets_on_stdout: bool,
    },

    /// Imports hosts from a TOML file or JSON `vps-export` envelope (EN + legacy PT keys).
    Import {
        /// Source file (TOML wire or JSON export envelope).
        #[arg(long)]
        file: PathBuf,
        /// Allow hosts without full auth (redacted export / skeleton) — GAP-SSH-IMP-001.
        #[arg(long)]
        allow_incomplete: bool,
    },
}

/// Actions of the `scp` subcommand (regular files only; no `-r` / no SFTP).
#[derive(Debug, Subcommand)]
pub enum ScpAction {
    /// Uploads a local file to the remote host (regular files only).
    Upload {
        /// VPS name.
        vps_name: String,
        /// Local path.
        local: PathBuf,
        /// Remote path.
        remote: PathBuf,
        /// SSH password override.
        #[arg(long, conflicts_with = "password_stdin")]
        password: Option<String>,
        /// Reads the SSH password from stdin.
        #[arg(long)]
        password_stdin: bool,
        /// Private key path override.
        #[arg(long)]
        key: Option<String>,
        /// Key passphrase.
        #[arg(long, conflicts_with = "key_passphrase_stdin")]
        key_passphrase: Option<String>,
        /// Reads the key passphrase from stdin.
        #[arg(long)]
        key_passphrase_stdin: bool,
        /// SSH timeout override in milliseconds (covers connect+transfer).
        #[arg(long)]
        timeout: Option<u64>,
        /// Emits transfer JSON on stdout (GAP-SSH-IO-007).
        #[arg(long)]
        json: bool,
    },

    /// Downloads a remote file to the local host (regular files only).
    Download {
        /// VPS name.
        vps_name: String,
        /// Remote path.
        remote: PathBuf,
        /// Local path.
        local: PathBuf,
        /// SSH password override.
        #[arg(long, conflicts_with = "password_stdin")]
        password: Option<String>,
        /// Reads the SSH password from stdin.
        #[arg(long)]
        password_stdin: bool,
        /// Private key path override.
        #[arg(long)]
        key: Option<String>,
        /// Key passphrase.
        #[arg(long, conflicts_with = "key_passphrase_stdin")]
        key_passphrase: Option<String>,
        /// Reads the key passphrase from stdin.
        #[arg(long)]
        key_passphrase_stdin: bool,
        /// SSH timeout override in milliseconds (covers connect+transfer).
        #[arg(long)]
        timeout: Option<u64>,
        /// Emits transfer JSON on stdout (GAP-SSH-IO-007).
        #[arg(long)]
        json: bool,
    },
}

/// Actions of the `secrets` subcommand (primary-key / AEAD).
#[derive(Debug, Subcommand)]
pub enum SecretsAction {
    /// Shows encryption status (no sensitive material).
    Status {
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Generates and stores the primary key (`secrets.key` or keyring). Never prints the key.
    Init {
        /// Store in the OS keyring instead of `secrets.key`.
        #[arg(long)]
        keyring: bool,
        /// Overwrites an existing key.
        #[arg(long)]
        force: bool,
        /// JSON success envelope (`event: secrets-init`).
        #[arg(long)]
        json: bool,
    },
    /// Rewrites `config.toml` re-encrypting secrets with the current key.
    Reencrypt {
        /// JSON success envelope (`event: secrets-reencrypt`).
        #[arg(long)]
        json: bool,
    },
}

/// Parses CLI arguments.
#[must_use]
pub fn parse_args() -> CliArgs {
    CliArgs::parse()
}

/// Initializes `tracing-subscriber`.
///
/// GAP-SSH-LOG-001 (0.3.9): default **error** (agent-first). `-v` → debug.
/// `RUST_LOG` wins. Never defaults to INFO for JSON/non-TTY.
pub fn initialize_logs(args: &CliArgs) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else if args.verbose {
        EnvFilter::new("debug")
    } else {
        // quiet and human/agent default: error (no INFO prose on stderr).
        let _ = args.quiet;
        EnvFilter::new("error")
    };

    let _ = fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_ansi(false)
        .try_init();
}

/// Writes shell completions to stdout.
///
/// GAP-SSH-CLI-003: broken pipe (EPIPE) does not panic — Unix pipe behavior.
pub fn generate_completions(shell: Shell) {
    use clap::CommandFactory;
    use std::io::Write;
    let mut cmd = CliArgs::command();
    let mut buf: Vec<u8> = Vec::new();
    clap_complete::generate(shell, &mut cmd, "ssh-cli", &mut buf);
    let mut out = std::io::stdout().lock();
    if let Err(e) = out.write_all(&buf).and_then(|_| out.flush()) {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            return;
        }
        // Other errors: best-effort on stderr without panic.
        let _ = writeln!(std::io::stderr(), "failed to write completions: {e}");
    }
}

fn read_stdin_if(flag: bool, value: Option<String>) -> Result<Option<String>> {
    if flag {
        Ok(Some(crate::vps::read_secret_stdin()?))
    } else {
        Ok(value)
    }
}

fn warn_if_password_argv(args: &CliArgs) {
    // Best-effort: inspect Debug of command for password-like flags present.
    let s = format!("{:?}", args.command);
    let sensitive = ["password:", "key_passphrase:", "sudo_password:", "su_password:"];
    // clap Debug of Option::Some("…") — coarse but covers argv secrets.
    let has = sensitive.iter().any(|k| s.contains(k) && s.contains("Some("));
    if has {
        eprintln!(
            "warning: a password-like value was passed on the command line (visible in process lists); prefer --*-stdin"
        );
    }
}

/// Resolves output format: explicit > `SSH_CLI_FORCE_TEXT` > JSON if non-TTY > Text.
#[must_use]
pub fn resolve_format(explicit: Option<OutputFormat>) -> OutputFormat {
    if let Some(f) = explicit {
        return f;
    }
    // Isolation for tests/scripts that force human prose in a pipe.
    if std::env::var_os("SSH_CLI_FORCE_TEXT").is_some() {
        return OutputFormat::Text;
    }
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    }
}

/// Runs the requested subcommand.
pub async fn dispatch(args: CliArgs) -> Result<()> {
    let config_override = args.config_dir.clone();
    // Aligns `secrets.key` with `--config-dir` / isolated tests.
    crate::secrets::set_config_dir(config_override.clone());
    crate::secrets::set_runtime_flags(
        args.allow_plaintext_secrets,
        args.secrets_key_file.clone(),
        args.use_keyring,
    );
    let formato = resolve_format(args.output_format);
    // GAP-SSH-IO-003 / IO-004: centralized I/O policy.
    crate::output::set_quiet(args.quiet);
    crate::output::set_json_errors(formato == OutputFormat::Json);
    let disable_sudo = args.disable_sudo;
    let replace_host_key = args.replace_host_key;

    // GAP-AUD-010: warn when secrets appear on argv (visible in `ps`).
    warn_if_password_argv(&args);

    match args.command {
        Command::Vps { action } => {
            crate::vps::run_vps_command(action, config_override, formato).await
        }
        Command::Connect { name } => {
            crate::vps::run_connect(&name, config_override, formato).await
        },
        Command::Exec {
            vps_name,
            command,
            json,
            password,
            password_stdin,
            key,
            key_passphrase,
            key_passphrase_stdin,
            timeout,
            description,
        } => {
            let password = read_stdin_if(password_stdin, password)?;
            let key_passphrase = read_stdin_if(key_passphrase_stdin, key_passphrase)?;
            let opts = crate::vps::ExecOptions {
                password,
                key,
                key_passphrase,
                timeout,
                description,
                replace_host_key,
                disable_sudo,
                ..Default::default()
            };
            crate::vps::run_exec(&vps_name, &command, config_override, formato, json, opts)
                .await
        }
        Command::SudoExec {
            vps_name,
            command,
            json,
            password,
            password_stdin,
            sudo_password,
            sudo_password_stdin,
            key,
            key_passphrase,
            key_passphrase_stdin,
            timeout,
            description,
        } => {
            let password = read_stdin_if(password_stdin, password)?;
            let sudo_password = read_stdin_if(sudo_password_stdin, sudo_password)?;
            let key_passphrase = read_stdin_if(key_passphrase_stdin, key_passphrase)?;
            let opts = crate::vps::ExecOptions {
                password,
                sudo_password,
                key,
                key_passphrase,
                timeout,
                description,
                replace_host_key,
                disable_sudo,
                ..Default::default()
            };
            crate::vps::run_sudo_exec(
                &vps_name,
                &command,
                config_override,
                formato,
                json,
                opts,
            )
            .await
        }
        Command::SuExec {
            vps_name,
            command,
            json,
            password,
            password_stdin,
            su_password,
            su_password_stdin,
            key,
            key_passphrase,
            key_passphrase_stdin,
            timeout,
            description,
        } => {
            let password = read_stdin_if(password_stdin, password)?;
            let su_password = read_stdin_if(su_password_stdin, su_password)?;
            let key_passphrase = read_stdin_if(key_passphrase_stdin, key_passphrase)?;
            let opts = crate::vps::ExecOptions {
                password,
                su_password,
                key,
                key_passphrase,
                timeout,
                description,
                replace_host_key,
                disable_sudo,
                ..Default::default()
            };
            crate::vps::run_su_exec(&vps_name, &command, config_override, formato, json, opts)
                .await
        }
        Command::Scp { action } => {
            let (
                password,
                password_stdin,
                key,
                key_passphrase,
                key_passphrase_stdin,
                timeout,
                json_local,
            ) = match &action {
                ScpAction::Upload {
                    password,
                    password_stdin,
                    key,
                    key_passphrase,
                    key_passphrase_stdin,
                    timeout,
                    json,
                    ..
                }
                | ScpAction::Download {
                    password,
                    password_stdin,
                    key,
                    key_passphrase,
                    key_passphrase_stdin,
                    timeout,
                    json,
                    ..
                } => (
                    password.clone(),
                    *password_stdin,
                    key.clone(),
                    key_passphrase.clone(),
                    *key_passphrase_stdin,
                    *timeout,
                    *json,
                ),
            };
            let password = read_stdin_if(password_stdin, password)?;
            let key_passphrase = read_stdin_if(key_passphrase_stdin, key_passphrase)?;
            // GAP-SSH-IO-007b: local --json or global --format json → JSON error envelope.
            let json_efetivo = json_local || formato == OutputFormat::Json;
            if json_efetivo {
                crate::output::set_json_errors(true);
            }
            crate::scp::run_scp(
                action,
                config_override,
                crate::scp::ScpOptions {
                    password,
                    key,
                    key_passphrase,
                    timeout,
                    replace_host_key,
                    json: json_efetivo,
                },
            )
            .await
        }
        Command::Tunnel {
            vps_name,
            local_port,
            remote_host,
            remote_port,
            timeout_ms,
            password,
            password_stdin,
            key,
            key_passphrase,
            key_passphrase_stdin,
            json,
            bind,
        } => {
            // GAP-SSH-IO-008: --json local or global format.
            let json_efetivo = json || formato == OutputFormat::Json;
            if json_efetivo {
                crate::output::set_json_errors(true);
            }
            // GAP-SSH-CLI-005: auth parity with exec/scp (stdin + passphrase).
            let password = read_stdin_if(password_stdin, password)?;
            let key_passphrase = read_stdin_if(key_passphrase_stdin, key_passphrase)?;
            crate::tunnel::run_tunnel(
                &vps_name,
                local_port,
                &remote_host,
                remote_port,
                config_override,
                password,
                key,
                key_passphrase,
                timeout_ms,
                replace_host_key,
                json_efetivo,
                &bind,
            )
            .await
        }
        Command::HealthCheck {
            vps_name,
            json,
            password,
            password_stdin,
            key,
            key_passphrase,
            key_passphrase_stdin,
            timeout,
        } => {
            // GAP-SSH-CLI-006: auth parity with exec/scp (stdin + key + passphrase).
            let password = read_stdin_if(password_stdin, password)?;
            let key_passphrase = read_stdin_if(key_passphrase_stdin, key_passphrase)?;
            crate::vps::run_health_check(
                vps_name.as_deref(),
                config_override,
                formato,
                json,
                password,
                timeout,
                key,
                key_passphrase,
                replace_host_key,
            )
            .await
        }
        Command::Secrets { action } => {
            crate::vps::run_secrets_command(action, config_override, formato).await
        }
        Command::Completions { shell } => {
            generate_completions(shell);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parser_understands_tunnel_with_timeout() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "tunnel",
            "vps-a",
            "8080",
            "127.0.0.1",
            "5432",
            "--timeout-ms",
            "5000",
            "--json",
        ])
        .expect("tunnel");
        match args.command {
            Command::Tunnel {
                timeout_ms,
                local_port,
                json,
                ..
            } => {
                assert_eq!(timeout_ms, 5000);
                assert_eq!(local_port, 8080);
                assert!(json);
            }
            _ => panic!("esperado tunnel"),
        }
    }

    #[test]
    fn parser_vps_add_key() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "vps",
            "add",
            "--name",
            "x",
            "--host",
            "h",
            "--user",
            "u",
            "--key",
            "/tmp/id_ed25519",
        ])
        .expect("add key");
        match args.command {
            Command::Vps {
                action: VpsAction::Add { key, password, .. },
            } => {
                assert_eq!(key.as_deref(), Some("/tmp/id_ed25519"));
                assert!(password.is_none());
            }
            _ => panic!("esperado add"),
        }
    }

    #[test]
    fn parser_sudo_exec_description() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "sudo-exec",
            "v",
            "id",
            "--description",
            "who am i",
        ])
        .unwrap();
        match args.command {
            Command::SudoExec { description, .. } => {
                assert_eq!(description.as_deref(), Some("who am i"));
            }
            _ => panic!("sudo-exec"),
        }
    }

    #[test]
    fn parser_su_exec() {
        let args = CliArgs::try_parse_from(["ssh-cli", "su-exec", "v", "whoami"]).unwrap();
        assert!(matches!(args.command, Command::SuExec { .. }));
    }

    #[test]
    fn parser_disable_sudo_global() {
        let args =
            CliArgs::try_parse_from(["ssh-cli", "--disable-sudo", "vps", "path"]).unwrap();
        assert!(args.disable_sudo);
    }

    #[test]
    fn parser_doctor() {
        let args = CliArgs::try_parse_from(["ssh-cli", "vps", "doctor", "--json"]).unwrap();
        match args.command {
            Command::Vps {
                action: VpsAction::Doctor { json },
            } => assert!(json),
            _ => panic!("doctor"),
        }
    }
}
