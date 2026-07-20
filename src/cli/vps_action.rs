// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: clap `vps` action tree extracted from cli/mod (SRP; line budget).
#![forbid(unsafe_code)]
//! Clap types for `ssh-cli vps …`.

use super::parse_cli_char_limit;
use clap::{ArgAction, Subcommand, ValueHint};
use std::path::PathBuf;

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
        #[arg(
            long,
            default_value_t = crate::constants::DEFAULT_SSH_PORT,
            value_parser = clap::value_parser!(u16).range(1..=65535)
        )]
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
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        key: Option<PathBuf>,
        /// Key passphrase.
        #[arg(long)]
        key_passphrase: Option<String>,
        /// Authenticate via SSH agent (mutually exclusive with --password / --key; G-E2E-19).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["password", "password_stdin", "key"])]
        use_agent: bool,
        /// Optional SSH agent socket path (defaults to platform agent when omitted).
        #[arg(long, value_name = "PATH", value_hint = ValueHint::AnyPath)]
        agent_socket: Option<PathBuf>,
        /// Timeout in milliseconds (default [`crate::vps::model::DEFAULT_TIMEOUT_MS`]).
        #[arg(long, default_value_t = crate::vps::model::DEFAULT_TIMEOUT_MS, value_name = "MS")]
        timeout: u64,
        /// Command character limit (input). Use `0` or `none` for unlimited.
        #[arg(long, value_name = "N", value_parser = parse_cli_char_limit)]
        max_command_chars: Option<usize>,
        /// Output character limit. Use `0` or `none` for unlimited.
        #[arg(long, value_name = "N", value_parser = parse_cli_char_limit)]
        max_output_chars: Option<usize>,
        /// Legacy alias: maps to max_command_chars.
        #[arg(long, alias = "maxChars", value_name = "N", value_parser = parse_cli_char_limit)]
        max_chars: Option<usize>,
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
        /// Host tags for fleet selection (G-O2). Repeatable: `--tag prod --tag web`.
        #[arg(long = "tag", value_name = "TAG", action = ArgAction::Append)]
        tags: Vec<String>,
        /// Enable SSH-over-TLS (rustls) for this host.
        #[arg(long, action = ArgAction::SetTrue)]
        tls: bool,
        /// TLS SNI / cert name (defaults to `--host` when omitted).
        #[arg(long, value_name = "NAME")]
        tls_sni: Option<String>,
        /// mTLS client certificate PEM path.
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        tls_client_cert: Option<PathBuf>,
        /// mTLS client private key PEM path.
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        tls_client_key: Option<PathBuf>,
        /// Runs health-check after add.
        #[arg(long)]
        check: bool,
    },

    /// Lists all VPS hosts (passwords masked).
    List {
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
        /// Filter hosts that have **any** of these tags (OR, G-O2).
        #[arg(long = "tag", value_name = "TAG", action = ArgAction::Append)]
        tags: Vec<String>,
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
        #[arg(long, value_parser = clap::value_parser!(u16).range(1..=65535))]
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
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        key: Option<PathBuf>,
        /// New key passphrase.
        #[arg(long)]
        key_passphrase: Option<String>,
        /// Switch primary auth to SSH agent (clears password/key when set; G-E2E-19).
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["password", "password_stdin", "key"])]
        use_agent: bool,
        /// Optional SSH agent socket path.
        #[arg(long, value_name = "PATH", value_hint = ValueHint::AnyPath)]
        agent_socket: Option<PathBuf>,
        /// New timeout.
        #[arg(long, value_name = "MS")]
        timeout: Option<u64>,
        /// New max command chars. Use `0` or `none` for unlimited.
        #[arg(long, value_name = "N", value_parser = parse_cli_char_limit)]
        max_command_chars: Option<usize>,
        /// New max output chars. Use `0` or `none` for unlimited.
        #[arg(long, value_name = "N", value_parser = parse_cli_char_limit)]
        max_output_chars: Option<usize>,
        /// Legacy alias maxChars → command.
        #[arg(long, alias = "maxChars", value_name = "N", value_parser = parse_cli_char_limit)]
        max_chars: Option<usize>,
        /// New sudo password.
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
        /// New su password.
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
        /// Disable sudo/su elevation for this host.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "enable_sudo")]
        disable_sudo: bool,
        /// Re-enable sudo/su elevation for this host.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "disable_sudo")]
        enable_sudo: bool,
        /// Enable SSH-over-TLS for this host.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "no_tls")]
        tls: bool,
        /// Disable SSH-over-TLS for this host.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "tls")]
        no_tls: bool,
        /// TLS SNI / cert name.
        #[arg(long, value_name = "NAME")]
        tls_sni: Option<String>,
        /// mTLS client certificate PEM path.
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        tls_client_cert: Option<PathBuf>,
        /// mTLS client private key PEM path.
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        tls_client_key: Option<PathBuf>,
    },

    /// Shows VPS details (passwords masked).
    Show {
        /// VPS name.
        name: String,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Shows the configuration file path.
    Path,

    /// Diagnostics for XDG layers / path / schema.
    Doctor {
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
        /// After local diagnostics, probe hosts over SSH (bounded fan-out, G-PAR-29).
        /// Default scope is every registered host; use `--hosts` for a subset (G-PAR-38).
        #[arg(long, action = ArgAction::SetTrue)]
        probe_ssh: bool,
        /// Comma-separated host subset for `--probe-ssh` only (ignored without probe).
        #[arg(long, value_name = "LIST")]
        hosts: Option<String>,
    },

    /// Exports hosts (passwords redacted by default).
    Export {
        /// Include secrets in the export.
        #[arg(long)]
        include_secrets: bool,
        /// Output file (stdout if omitted). Written atomically with mode 0o600.
        #[arg(long, short, value_name = "PATH", value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
        /// Agent-first JSON envelope (`event: vps-export`; from global `--json`).
        /// Default body is **TOML** on text; JSON when format/json (G-AUD-03).
        #[arg(from_global)]
        json: bool,
        /// Acknowledge writing plaintext secrets to stdout (pipe/non-TTY). Prefer `--output`.
        #[arg(long)]
        i_understand_secrets_on_stdout: bool,
    },

    /// Imports hosts from a TOML file or JSON `vps-export` envelope (EN + legacy PT keys).
    Import {
        /// Source file (TOML wire or JSON export envelope).
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        file: PathBuf,
        /// Allow hosts without full auth (redacted export / skeleton) — GAP-SSH-IMP-001.
        #[arg(long)]
        allow_incomplete: bool,
    },
}
