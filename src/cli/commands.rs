// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: top-level clap Command tree extracted from cli/mod (SRP; line budget).
#![forbid(unsafe_code)]
//! Top-level and nested clap action enums (except `vps` / `scp` / `sftp`).

use super::scp_args::ScpAction;
use super::sftp_args::SftpAction;
use super::vps_action::VpsAction;
use super::SshAuthArgs;
use clap::{ArgAction, Subcommand, ValueHint};
use clap_complete::Shell;
use std::path::PathBuf;

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
    ///
    /// Positionals: `VPS COMMAND`, or with `--all`/`--hosts`, only `COMMAND`
    /// (`ssh-cli exec --all uptime`, `ssh-cli exec --hosts a,b uptime`).
    /// Extra steps on the **same** SSH session: `--step cmd2 --step cmd3` (G-O3).
    Exec {
        /// Run on every registered host (bounded concurrency). When set, pass
        /// only the shell command as the single positional.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["hosts", "tags"])]
        all: bool,
        /// Comma-separated host subset (bounded fan-out). Batch JSON even for one name.
        #[arg(long, value_name = "LIST", conflicts_with_all = ["all", "tags"])]
        hosts: Option<String>,
        /// Select hosts that have **any** of these tags (OR). Batch JSON (G-O2).
        #[arg(long, value_name = "LIST", conflicts_with_all = ["all", "hosts"])]
        tags: Option<String>,
        /// `VPS COMMAND` (one host) or `COMMAND` only when `--all` / `--hosts` / `--tags`.
        #[arg(required = true, num_args = 1..=2, value_names = ["VPS", "COMMAND"])]
        target: Vec<String>,
        /// Additional commands on the same SSH session after the primary (G-O3).
        #[arg(long = "step", value_name = "CMD", action = ArgAction::Append)]
        steps: Vec<String>,
        /// JSON output (from global `--json` / format; G-AUD-01).
        #[arg(from_global)]
        json: bool,
        /// SSH authentication overrides (password/key/passphrase).
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Timeout override in milliseconds.
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// Shell comment appended for audit trails.
        #[arg(long)]
        description: Option<String>,
    },

    /// Runs a command with `sudo` (safe `sh -c` packing).
    ///
    /// Positionals: `VPS COMMAND` or, with `--all`/`--hosts`/`--tags`, only `COMMAND`.
    SudoExec {
        /// Run on every registered host (bounded concurrency).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["hosts", "tags"])]
        all: bool,
        /// Comma-separated host subset (bounded fan-out).
        #[arg(long, value_name = "LIST", conflicts_with_all = ["all", "tags"])]
        hosts: Option<String>,
        /// Select hosts by tag (OR). Batch JSON (G-O2).
        #[arg(long, value_name = "LIST", conflicts_with_all = ["all", "hosts"])]
        tags: Option<String>,
        /// `VPS COMMAND` (one host) or `COMMAND` only when batch selection.
        #[arg(required = true, num_args = 1..=2, value_names = ["VPS", "COMMAND"])]
        target: Vec<String>,
        /// Extra commands on the same session (G-O3).
        #[arg(long = "step", value_name = "CMD", action = ArgAction::Append)]
        steps: Vec<String>,
        /// JSON output (from global `--json` / format; G-AUD-01).
        #[arg(from_global)]
        json: bool,
        /// SSH authentication overrides (password/key/passphrase).
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Sudo password override.
        #[arg(
            long,
            alias = "sudoPassword",
            alias = "sudo_password",
            conflicts_with = "sudo_password_stdin"
        )]
        sudo_password: Option<String>,
        /// Reads the sudo password from stdin.
        #[arg(long, action = ArgAction::SetTrue)]
        sudo_password_stdin: bool,
        /// Timeout override in milliseconds.
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// Shell comment appended for audit.
        #[arg(long)]
        description: Option<String>,
    },

    /// Runs a command with one-shot `su -` elevation.
    ///
    /// Positionals: `VPS COMMAND` or, with `--all`/`--hosts`, only `COMMAND`.
    SuExec {
        /// Run on every registered host (bounded concurrency).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["hosts", "tags"])]
        all: bool,
        /// Comma-separated host subset (bounded fan-out).
        #[arg(long, value_name = "LIST", conflicts_with_all = ["all", "tags"])]
        hosts: Option<String>,
        /// Select hosts by tag (OR). Batch JSON (G-O2).
        #[arg(long, value_name = "LIST", conflicts_with_all = ["all", "hosts"])]
        tags: Option<String>,
        /// `VPS COMMAND` (one host) or `COMMAND` only when batch selection.
        #[arg(required = true, num_args = 1..=2, value_names = ["VPS", "COMMAND"])]
        target: Vec<String>,
        /// Extra commands on the same session (G-O3).
        #[arg(long = "step", value_name = "CMD", action = ArgAction::Append)]
        steps: Vec<String>,
        /// JSON output (from global `--json` / format; G-AUD-01).
        #[arg(from_global)]
        json: bool,
        /// SSH authentication overrides (password/key/passphrase).
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Su password override.
        #[arg(
            long,
            alias = "suPassword",
            alias = "su_password",
            conflicts_with = "su_password_stdin"
        )]
        su_password: Option<String>,
        /// Reads the su password from stdin.
        #[arg(long, action = ArgAction::SetTrue)]
        su_password_stdin: bool,
        /// Timeout override.
        #[arg(long, value_name = "MS")]
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

    /// SFTP subsystem transfer and remote filesystem ops (G-SFTP).
    Sftp {
        /// Specific SFTP action.
        #[command(subcommand)]
        action: SftpAction,
    },

    /// SSH tunnel with mandatory deadline (bounded one-shot).
    ///
    /// Contract: **one** local bind + **one** SSH session per invocation (G-PAR-30).
    /// Multi-host tunnels = N one-shots with distinct `--bind`/ports. Forward
    /// accepts still use JoinSet + Semaphore (`--max-concurrency`).
    Tunnel {
        /// VPS name (single host only — no `--all` / `--hosts`).
        vps_name: String,
        /// Local port.
        local_port: u16,
        /// Remote host.
        remote_host: String,
        /// Remote port.
        #[arg(value_parser = clap::value_parser!(u16).range(1..=65535))]
        remote_port: u16,
        /// Mandatory tunnel timeout in milliseconds.
        #[arg(long, value_name = "MS")]
        timeout_ms: u64,
        /// SSH authentication overrides (password/key/passphrase).
        #[command(flatten)]
        auth: SshAuthArgs,
        /// Agent-first JSON output when the local listener is up (GAP-SSH-IO-008).
        #[arg(from_global)]
        json: bool,
        /// Local bind address (default loopback for security).
        #[arg(long, default_value = crate::constants::DEFAULT_TUNNEL_BIND_ADDR, value_name = "ADDR")]
        bind: String,
    },

    /// Checks SSH connectivity to a VPS (or multi-host with `--all` / `--hosts`).
    HealthCheck {
        /// VPS name (uses active if omitted; ignored with `--all` / `--hosts`).
        #[arg(conflicts_with_all = ["all", "hosts"])]
        vps_name: Option<String>,
        /// Probe every registered host in parallel (bounded concurrency).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "hosts")]
        all: bool,
        /// Comma-separated host subset (bounded fan-out). Batch JSON even for one name.
        #[arg(long, value_name = "LIST", conflicts_with = "all")]
        hosts: Option<String>,
        /// JSON output (GAP-SSH-IO-002). Single host: classic object; multi: batch.
        #[arg(from_global)]
        json: bool,
        /// SSH authentication overrides (password/key/passphrase).
        #[command(flatten)]
        auth: SshAuthArgs,
        /// SSH timeout override in milliseconds (GAP-SSH-CLI-004).
        #[arg(long, value_name = "MS")]
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

    /// Emits the full command tree as JSON (agent discovery / rules `mycli commands`).
    Commands {
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Emits embedded JSON Schema catalog or one schema body (G-E2E-02).
    Schema {
        /// Schema name (omit to list catalog). Example: `vps-list`.
        name: Option<String>,
        /// JSON catalog envelope when listing (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Root alias for `vps doctor` (XDG / schema diagnostics; G-E2E-03).
    Doctor {
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
        /// Also probe SSH health on registered hosts.
        #[arg(long, action = ArgAction::SetTrue)]
        probe_ssh: bool,
        /// Comma-separated host subset for `--probe-ssh`.
        #[arg(long, value_name = "LIST")]
        hosts: Option<String>,
    },

    /// Diagnoses and manages UI language (locale resolution / XDG preference).
    Locale {
        /// JSON diagnostics (from global `--json` / format).
        #[arg(from_global)]
        json: bool,
        /// Optional locale action (default: show status).
        #[command(subcommand)]
        action: Option<LocaleAction>,
    },
    /// TLS stack: provider status, mTLS identities, ACME certs (XDG; rustls only).
    Tls {
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
        /// TLS action.
        #[command(subcommand)]
        action: TlsAction,
    },
}

/// Actions of the `tls` subcommand (SSH-over-TLS / mTLS / ACME).
#[derive(Debug, Subcommand)]
pub enum TlsAction {
    /// Shows rustls CryptoProvider status (`aws_lc_rs`).
    Provider,
    /// Prints XDG TLS directory layout paths.
    Paths,
    /// Manages imported mTLS client identities under XDG `tls/mtls/`.
    Mtls {
        /// mTLS action.
        #[command(subcommand)]
        action: TlsMtlsAction,
    },
    /// ACME (Let's Encrypt) account + DNS-01 certificate lifecycle.
    Acme {
        /// ACME action.
        #[command(subcommand)]
        action: TlsAcmeAction,
    },
}

/// mTLS identity store actions.
#[derive(Debug, Subcommand)]
pub enum TlsMtlsAction {
    /// Lists imported identity names.
    List,
    /// Imports PEM cert+key as a named identity.
    Import {
        /// Identity name (XDG leaf).
        #[arg(long)]
        name: String,
        /// Certificate chain PEM path.
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        cert: PathBuf,
        /// Private key PEM path.
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        key: PathBuf,
    },
    /// Shows paths for one identity.
    Show {
        /// Identity name.
        name: String,
    },
    /// Removes an identity directory.
    Remove {
        /// Identity name.
        name: String,
    },
}

/// ACME actions (DNS-01, agent two-step).
#[derive(Debug, Subcommand)]
pub enum TlsAcmeAction {
    /// ACME account management.
    Account {
        /// Account action.
        #[command(subcommand)]
        action: TlsAcmeAccountAction,
    },
    /// Starts DNS-01 order and prints the TXT challenge (persists order URL under XDG).
    Issue {
        /// Domain name (DNS identifier).
        #[arg(long)]
        domain: String,
        /// Use Let's Encrypt staging directory.
        #[arg(long, action = ArgAction::SetTrue)]
        staging: bool,
        /// Required: print challenge and exit (agent-friendly; no interactive wait).
        #[arg(long, action = ArgAction::SetTrue)]
        print_challenge: bool,
    },
    /// Completes a pending order after DNS TXT is published.
    Complete {
        /// Domain name.
        #[arg(long)]
        domain: String,
    },
    /// Shows certificate / pending status for one domain or all.
    Status {
        /// Optional domain filter.
        #[arg(long)]
        domain: Option<String>,
    },
    /// Lists ACME domain directories under XDG.
    List,
}

/// ACME account sub-actions.
#[derive(Debug, Subcommand)]
pub enum TlsAcmeAccountAction {
    /// Creates an ACME account (credentials under XDG `tls/acme/account.json`, 0o600).
    Create {
        /// Use Let's Encrypt staging.
        #[arg(long, action = ArgAction::SetTrue)]
        staging: bool,
        /// Contact URLs (e.g. `mailto:ops@example.com`). Required; repeatable (G-AUD-06).
        #[arg(long = "contact", value_name = "URL", action = ArgAction::Append, required = true, num_args = 1..)]
        contact: Vec<String>,
        /// Replace existing account credentials.
        #[arg(long, action = ArgAction::SetTrue)]
        force: bool,
    },
    /// Shows whether an account exists and its path.
    Show,
}

/// Actions of the `locale` subcommand.
#[derive(Debug, Subcommand)]
pub enum LocaleAction {
    /// Shows resolved language, winning layer, and available locales (default).
    Show,
    /// Persists preferred language under the config directory (`lang` file, 0o600).
    Set {
        /// BCP47 tag that negotiates to a supported locale (`en`, `pt-BR`, …).
        #[arg(value_name = "LOCALE", value_parser = crate::locale::parse_lang_cli_arg)]
        lang: String,
    },
    /// Removes the persisted language preference.
    Clear,
}

/// Actions of the `secrets` subcommand (primary-key / AEAD).
#[derive(Debug, Subcommand)]
pub enum SecretsAction {
    /// Shows encryption status (no sensitive material).
    Status {
        /// JSON output (from global `--json`).
        #[arg(from_global)]
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
        /// JSON success envelope (`event: secrets-init`; from global `--json`).
        #[arg(from_global)]
        json: bool,
    },
    /// Rewrites `config.toml` re-encrypting secrets with the current key.
    Reencrypt {
        /// JSON success envelope (`event: secrets-reencrypt`; from global `--json`).
        #[arg(from_global)]
        json: bool,
    },
}
