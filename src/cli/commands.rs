// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: top-level clap Command tree extracted from cli/mod (SRP; line budget).
#![forbid(unsafe_code)]
//! Top-level and nested clap action enums (except `vps` / `scp` / `sftp`).

use super::help_text::{
    ACME_ACCOUNT_CREATE_EXAMPLES, ACME_ACCOUNT_SHOW_EXAMPLES, ACME_COMPLETE_EXAMPLES,
    ACME_ISSUE_EXAMPLES, ACME_LIST_EXAMPLES, ACME_STATUS_EXAMPLES, COMMANDS_EXAMPLES,
    COMPLETIONS_EXAMPLES, CONNECT_EXAMPLES, DOCTOR_EXAMPLES, EXEC_EXAMPLES, EXEC_USAGE_BLOCK,
    HEALTH_CHECK_EXAMPLES, HEALTH_CHECK_USAGE, LOCALE_CLEAR_EXAMPLES, LOCALE_SET_EXAMPLES,
    LOCALE_SHOW_EXAMPLES, MTLS_IMPORT_EXAMPLES, MTLS_LIST_EXAMPLES, MTLS_REMOVE_EXAMPLES,
    MTLS_SHOW_EXAMPLES, SCHEMA_EXAMPLES, SECRETS_INIT_EXAMPLES, SECRETS_REENCRYPT_EXAMPLES,
    SECRETS_STATUS_EXAMPLES, SUDO_EXEC_EXAMPLES, SUDO_EXEC_USAGE_BLOCK, SU_EXEC_EXAMPLES,
    SU_EXEC_USAGE_BLOCK, TLS_PATHS_EXAMPLES, TLS_PROVIDER_EXAMPLES, TUNNEL_EXAMPLES,
};
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
    #[command(after_long_help = CONNECT_EXAMPLES)]
    Connect {
        /// Name of the VPS previously added via `vps add`.
        name: String,
    },

    /// Runs a command on the VPS over SSH (stdout/stderr captured).
    ///
    /// The target is always designated explicitly, in one of three ways: two
    /// positionals `VPS COMMAND`; a selector with one positional
    /// (`--all`/`--hosts <LIST>`/`--tags <LIST>` `COMMAND`); or the active marker
    /// under the deliberate opt-in (`--use-active COMMAND`). A lone positional with
    /// no selector is a usage error, never a command aimed at whatever `connect`
    /// last wrote (GAP-SSH-EXEC-ARGC-001).
    ///
    /// Extra steps on the **same** SSH session: `--step cmd2 --step cmd3` (G-O3).
    /// Exit 127 on the first step aborts the batch, because that code there is the
    /// signature of a host name having been read as a command.
    #[command(override_usage = EXEC_USAGE_BLOCK, after_long_help = EXEC_EXAMPLES)]
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
        /// Run against the host recorded by `connect`, deliberately (GAP-SSH-EXEC-ARGC-001).
        ///
        /// Inheriting the target from on-disk state is ambient authority, so it must be
        /// asked for. Without this flag a lone positional is a usage error, never a
        /// command aimed at whatever host `connect` last wrote.
        #[arg(long = "use-active", action = ArgAction::SetTrue, conflicts_with_all = ["all", "hosts", "tags"])]
        use_active: bool,
        /// `VPS COMMAND`, or `COMMAND` alone with `--all`/`--hosts`/`--tags`/`--use-active`.
        ///
        /// Kept as a vector rather than two named positional slots on purpose: `clap`
        /// fills positionals by order and cannot know which selector is active, so
        /// `exec --all uptime` would bind `uptime` to a `VPS` slot and leave the
        /// command empty — silently accepting a shape worse than the one being fixed.
        /// The arity rule therefore lives in `parse_exec_target`, which can see the
        /// flags. See GAP-SSH-EXEC-ARGC-001.
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
    /// Same target rules as `exec`: `VPS COMMAND`, or one positional with a selector
    /// (`--all`/`--hosts`/`--tags`), or `--use-active COMMAND`. Elevation makes an
    /// undesignated target worse, not better, so nothing here is inferred.
    #[command(override_usage = SUDO_EXEC_USAGE_BLOCK, after_long_help = SUDO_EXEC_EXAMPLES)]
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
        /// Run against the host recorded by `connect`, deliberately (GAP-SSH-EXEC-ARGC-001).
        ///
        /// Elevation makes ambient authority worse, not better: a misdirected `sudo`
        /// step writes root-owned state on a host the caller never named.
        #[arg(long = "use-active", action = ArgAction::SetTrue, conflicts_with_all = ["all", "hosts", "tags"])]
        use_active: bool,
        /// `VPS COMMAND`, or `COMMAND` alone with a selector or `--use-active`.
        ///
        /// See the note on `Exec::target` for why this stays a vector.
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
    /// Same target rules as `exec`: `VPS COMMAND`, or one positional with a selector
    /// (`--all`/`--hosts`/`--tags`), or `--use-active COMMAND`.
    #[command(override_usage = SU_EXEC_USAGE_BLOCK, after_long_help = SU_EXEC_EXAMPLES)]
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
        /// Run against the host recorded by `connect`, deliberately (GAP-SSH-EXEC-ARGC-001).
        ///
        /// Elevation makes ambient authority worse, not better: a misdirected `su -`
        /// step writes root-owned state on a host the caller never named.
        #[arg(long = "use-active", action = ArgAction::SetTrue, conflicts_with_all = ["all", "hosts", "tags"])]
        use_active: bool,
        /// `VPS COMMAND`, or `COMMAND` alone with a selector or `--use-active`.
        ///
        /// See the note on `Exec::target` for why this stays a vector.
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
    /// accepts still use `JoinSet` + `Semaphore` (`--max-concurrency`).
    #[command(after_long_help = TUNNEL_EXAMPLES)]
    Tunnel {
        /// VPS name (single host only — no `--all` / `--hosts`).
        vps_name: String,
        /// Local port to bind — with `--reverse`, the local port that receives.
        local_port: u16,
        /// Remote host — with `--reverse`, the address the **server** binds.
        ///
        /// Optional since 0.5.4: `--socks5` chooses a destination per connection
        /// and `--remote-socket` names a Unix socket, so neither has one.
        remote_host: Option<String>,
        /// Remote port — with `--reverse`, the server port (`0` = server allocates).
        ///
        /// Reverse accepts `0` because the server then reports the port it bound;
        /// a local forward cannot, since there is nothing to connect to.
        #[arg(value_parser = clap::value_parser!(u16).range(0..=65535))]
        remote_port: Option<u16>,
        /// Serve a SOCKS5 proxy locally instead of a fixed forward (G-TUN-R02).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["reverse", "remote_socket"])]
        socks5: bool,
        /// Forward to a Unix domain socket on the remote host (G-TUN-R03).
        #[arg(long, value_name = "PATH", conflicts_with_all = ["reverse", "socks5"])]
        remote_socket: Option<String>,
        /// Ask the server to listen and deliver connections back here (G-TUN-R01).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["socks5", "remote_socket"])]
        reverse: bool,
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
        ///
        /// G-TUN-R08: validated by clap as an IP address, so a typo like
        /// `127.0.0..1` fails at parse time (exit 2) instead of after resolving the
        /// host, opening the SSH session and authenticating — which on a host with
        /// MFA or slow auth meant paying a full handshake to learn about a typo.
        #[arg(
            long,
            default_value = crate::constants::DEFAULT_TUNNEL_BIND_ADDR,
            value_name = "ADDR",
            value_parser = clap::value_parser!(std::net::IpAddr)
        )]
        bind: std::net::IpAddr,
        /// Acknowledge that a non-loopback bind exposes the forwarded service.
        ///
        /// G-TUN-R13: required for any routable bind. Without it, `--bind 0.0.0.0`
        /// silently published the remote service to the local network. Under
        /// `--reverse` it guards the **server's** bind address instead, which is
        /// the end that is exposed in that direction.
        #[arg(long, action = ArgAction::SetTrue)]
        i_accept_network_exposure: bool,
    },

    /// Checks SSH connectivity to a VPS (`--all` / `--hosts` / `--use-active`).
    ///
    /// The target must be designated: a name, a selector, or the explicit opt-in.
    /// Omitting all three is a usage error rather than a probe against whatever
    /// `connect` last wrote (GAP-SSH-EXEC-ARGC-001).
    #[command(override_usage = HEALTH_CHECK_USAGE, after_long_help = HEALTH_CHECK_EXAMPLES)]
    HealthCheck {
        /// VPS name. Required unless a selector or `--use-active` is present.
        #[arg(conflicts_with_all = ["all", "hosts", "use_active"])]
        vps_name: Option<String>,
        /// Probe every registered host in parallel (bounded concurrency).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "hosts")]
        all: bool,
        /// Comma-separated host subset (bounded fan-out). Batch JSON even for one name.
        #[arg(long, value_name = "LIST", conflicts_with = "all")]
        hosts: Option<String>,
        /// Probe the host recorded by `connect`, deliberately.
        ///
        /// This surface used to inherit the marker with no opt-in, defended as safe
        /// because a probe is an idempotent read. The probe is safe; the habit is
        /// not. An operator who learns that a target is optional here carries the
        /// expectation to `exec`, where the same shortcut sent a day of work to the
        /// wrong machine. The cheap half of the asymmetry is the one to make strict.
        #[arg(long = "use-active", action = ArgAction::SetTrue, conflicts_with_all = ["all", "hosts"])]
        use_active: bool,
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
    #[command(after_long_help = COMPLETIONS_EXAMPLES)]
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Emits the full command tree as JSON (agent discovery / rules `mycli commands`).
    #[command(after_long_help = COMMANDS_EXAMPLES)]
    Commands {
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Emits embedded JSON Schema catalog or one schema body (G-E2E-02).
    #[command(after_long_help = SCHEMA_EXAMPLES)]
    Schema {
        /// Schema name (omit to list catalog). Example: `vps-list`.
        name: Option<String>,
        /// JSON catalog envelope when listing (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Root alias for `vps doctor` (XDG / schema diagnostics; G-E2E-03).
    #[command(after_long_help = DOCTOR_EXAMPLES)]
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
    /// Shows rustls `CryptoProvider` status (`aws_lc_rs`).
    #[command(after_long_help = TLS_PROVIDER_EXAMPLES)]
    Provider,
    /// Prints XDG TLS directory layout paths.
    #[command(after_long_help = TLS_PATHS_EXAMPLES)]
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
    #[command(after_long_help = MTLS_LIST_EXAMPLES)]
    List,
    /// Imports PEM cert+key as a named identity.
    #[command(after_long_help = MTLS_IMPORT_EXAMPLES)]
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
    #[command(after_long_help = MTLS_SHOW_EXAMPLES)]
    Show {
        /// Identity name.
        name: String,
    },
    /// Removes an identity directory.
    #[command(after_long_help = MTLS_REMOVE_EXAMPLES)]
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
    #[command(after_long_help = ACME_ISSUE_EXAMPLES)]
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
    #[command(after_long_help = ACME_COMPLETE_EXAMPLES)]
    Complete {
        /// Domain name.
        #[arg(long)]
        domain: String,
    },
    /// Shows certificate / pending status for one domain or all.
    #[command(after_long_help = ACME_STATUS_EXAMPLES)]
    Status {
        /// Optional domain filter.
        #[arg(long)]
        domain: Option<String>,
    },
    /// Lists ACME domain directories under XDG.
    #[command(after_long_help = ACME_LIST_EXAMPLES)]
    List,
}

/// ACME account sub-actions.
#[derive(Debug, Subcommand)]
pub enum TlsAcmeAccountAction {
    /// Creates an ACME account (credentials under XDG `tls/acme/account.json`, 0o600).
    #[command(after_long_help = ACME_ACCOUNT_CREATE_EXAMPLES)]
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
    #[command(after_long_help = ACME_ACCOUNT_SHOW_EXAMPLES)]
    Show,
}

/// Actions of the `locale` subcommand.
#[derive(Debug, Subcommand)]
pub enum LocaleAction {
    /// Shows resolved language, winning layer, and available locales (default).
    #[command(after_long_help = LOCALE_SHOW_EXAMPLES)]
    Show,
    /// Persists preferred language under the config directory (`lang` file, 0o600).
    #[command(after_long_help = LOCALE_SET_EXAMPLES)]
    Set {
        /// BCP47 tag that negotiates to a supported locale (`en`, `pt-BR`, …).
        #[arg(value_name = "LOCALE", value_parser = crate::locale::parse_lang_cli_arg)]
        lang: String,
    },
    /// Removes the persisted language preference.
    #[command(after_long_help = LOCALE_CLEAR_EXAMPLES)]
    Clear,
}

/// Actions of the `secrets` subcommand (primary-key / AEAD).
#[derive(Debug, Subcommand)]
pub enum SecretsAction {
    /// Shows encryption status (no sensitive material).
    #[command(after_long_help = SECRETS_STATUS_EXAMPLES)]
    Status {
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },
    /// Generates and stores the primary key (`secrets.key` or keyring). Never prints the key.
    #[command(after_long_help = SECRETS_INIT_EXAMPLES)]
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
    #[command(after_long_help = SECRETS_REENCRYPT_EXAMPLES)]
    Reencrypt {
        /// JSON success envelope (`event: secrets-reencrypt`; from global `--json`).
        #[arg(from_global)]
        json: bool,
    },
}
