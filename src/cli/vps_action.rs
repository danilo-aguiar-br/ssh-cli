// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: clap `vps` action tree extracted from cli/mod (SRP; line budget).
#![forbid(unsafe_code)]
//! Clap types for `ssh-cli vps …`.

use super::help_text::{
    VPS_ADD_EXAMPLES, VPS_DOCTOR_EXAMPLES, VPS_EDIT_EXAMPLES, VPS_EXPORT_EXAMPLES,
    VPS_IMPORT_EXAMPLES, VPS_LIST_EXAMPLES, VPS_PATH_EXAMPLES, VPS_REMOVE_EXAMPLES,
    VPS_RENAME_EXAMPLES, VPS_SHOW_EXAMPLES,
};
use super::parse_cli_char_limit;
use clap::{ArgAction, Subcommand, ValueHint};
use std::path::PathBuf;

/// Actions of the `vps` subcommand.
#[derive(Debug, Subcommand)]
pub enum VpsAction {
    /// Adds a new VPS to the registry.
    #[command(after_long_help = VPS_ADD_EXAMPLES)]
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
        #[arg(long, conflicts_with = "key_passphrase_stdin")]
        key_passphrase: Option<String>,
        /// Reads the key passphrase from stdin.
        ///
        /// Preferred over `--key-passphrase`: a value on the command line is
        /// visible to every local process in `ps`, while stdin is not.
        #[arg(long, action = ArgAction::SetTrue)]
        key_passphrase_stdin: bool,
        /// Authenticate via SSH agent (mutually exclusive with --password / --key).
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
        /// Legacy alias: maps to `max_command_chars`.
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
        /// Host tags for fleet selection. Repeatable: `--tag prod --tag web`.
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
    #[command(after_long_help = VPS_LIST_EXAMPLES)]
    List {
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
        /// Filter hosts that have **any** of these tags (OR).
        #[arg(long = "tag", value_name = "TAG", action = ArgAction::Append)]
        tags: Vec<String>,
    },

    /// Removes a VPS from the registry.
    #[command(after_long_help = VPS_REMOVE_EXAMPLES)]
    Remove {
        /// VPS name to remove.
        name: String,
    },

    /// Renames a registered VPS without touching credentials.
    ///
    /// Both positionals are required, so arity never carries meaning: a call with
    /// one or three positionals is a usage error, never a guess. The target always
    /// comes from argv and this verb never reads the `active` marker.
    ///
    /// Stored credentials stay readable under the new name, because record
    /// serialization seals them without associated data, so opening a blob never
    /// needs the name it was stored under. The ciphertext token itself is resealed
    /// with a fresh nonce on every save, which is ordinary AEAD behavior.
    ///
    /// Exit codes: **64** for an invalid or identical name and for a name already
    /// taken; **66** when the source host does not exist.
    #[command(after_long_help = VPS_RENAME_EXAMPLES)]
    Rename {
        /// Current VPS name (the host this invocation acts on).
        name: String,
        /// New VPS name.
        new_name: String,
    },

    /// Edits fields of an existing VPS.
    #[command(after_long_help = VPS_EDIT_EXAMPLES)]
    Edit {
        /// VPS name to edit.
        name: String,
        /// New VPS name (rename without leaving this verb).
        #[arg(long = "name", value_name = "NEW")]
        new_name: Option<String>,
        /// New hostname/IP.
        #[arg(long)]
        host: Option<String>,
        /// New SSH port.
        #[arg(long, value_parser = clap::value_parser!(u16).range(1..=65535))]
        port: Option<u16>,
        /// New username.
        #[arg(long)]
        user: Option<String>,
        /// New password; switches this host to password auth.
        ///
        /// Clears the key, its passphrase, the agent flag and the agent socket.
        #[arg(long, conflicts_with_all = ["password_stdin", "key"])]
        password: Option<String>,
        /// Reads the password from stdin; switches this host to password auth.
        ///
        /// Clears the key, its passphrase, the agent flag and the agent socket.
        #[arg(long, conflicts_with = "key")]
        password_stdin: bool,
        /// New private key path; switches this host to key auth.
        ///
        /// Clears the password, the agent flag and the agent socket. A path different
        /// from the stored one also drops the stored passphrase, unless the same call
        /// passes a new one.
        #[arg(
            long,
            value_name = "PATH",
            value_hint = ValueHint::FilePath,
            conflicts_with_all = ["password", "password_stdin"]
        )]
        key: Option<PathBuf>,
        /// New key passphrase; needs a key already on the host or `--key`.
        #[arg(
            long,
            conflicts_with_all = [
                "key_passphrase_stdin",
                "use_agent",
                "agent_socket",
                "password",
                "password_stdin"
            ]
        )]
        key_passphrase: Option<String>,
        /// Reads the new key passphrase from stdin; preferred over `--key-passphrase`,
        /// whose value is visible in `ps`.
        #[arg(
            long,
            action = ArgAction::SetTrue,
            conflicts_with_all = ["use_agent", "agent_socket", "password", "password_stdin"]
        )]
        key_passphrase_stdin: bool,
        /// Switch this host to SSH agent auth.
        ///
        /// Clears the password, the key and its passphrase.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["password", "password_stdin", "key"])]
        use_agent: bool,
        /// SSH agent socket path; implies `--use-agent` (clears the password, the key
        /// and its passphrase).
        #[arg(
            long,
            value_name = "PATH",
            value_hint = ValueHint::AnyPath,
            conflicts_with_all = ["key", "password", "password_stdin"]
        )]
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
        /// Host tags for fleet selection; appended with dedupe, preserving order.
        #[arg(long = "tag", value_name = "TAG", action = ArgAction::Append)]
        tags: Vec<String>,
        /// Clears every tag before applying `--tag` values.
        ///
        /// Deliberately **not** conflicting with `--tag`: `--clear-tags --tag a`
        /// must end with the list `[a]`, which is the only way to replace a tag
        /// set in one invocation. Refusing the pair would force two commands and
        /// leave a window where the host has no tags at all.
        #[arg(long = "clear-tags", action = ArgAction::SetTrue)]
        clear_tags: bool,
        /// Removes the private key path **and** its passphrase.
        ///
        /// Refused with exit 64 when it would leave the host with no primary
        /// authentication method at all.
        #[arg(long = "no-key", action = ArgAction::SetTrue, conflicts_with = "key")]
        no_key: bool,
        /// Removes the explicit agent socket, falling back to the platform endpoint.
        ///
        /// Refused with exit 64 when the host authenticates through the agent and
        /// the platform has no default endpoint, because that host would stop
        /// authenticating.
        #[arg(
            long = "no-agent-socket",
            action = ArgAction::SetTrue,
            conflicts_with = "agent_socket"
        )]
        no_agent_socket: bool,
        /// Removes the stored sudo password.
        #[arg(
            long = "clear-sudo-password",
            action = ArgAction::SetTrue,
            conflicts_with_all = ["sudo_password", "sudo_password_stdin"]
        )]
        clear_sudo_password: bool,
        /// Removes the stored su password.
        #[arg(
            long = "clear-su-password",
            action = ArgAction::SetTrue,
            conflicts_with_all = ["su_password", "su_password_stdin"]
        )]
        clear_su_password: bool,
        /// Removes the TLS SNI override, leaving the host name as SNI.
        #[arg(long = "no-tls-sni", action = ArgAction::SetTrue, conflicts_with = "tls_sni")]
        no_tls_sni: bool,
        /// Removes the mTLS client certificate path.
        #[arg(
            long = "no-tls-client-cert",
            action = ArgAction::SetTrue,
            conflicts_with = "tls_client_cert"
        )]
        no_tls_client_cert: bool,
        /// Removes the mTLS client key path.
        #[arg(
            long = "no-tls-client-key",
            action = ArgAction::SetTrue,
            conflicts_with = "tls_client_key"
        )]
        no_tls_client_key: bool,
    },

    /// Shows VPS details (passwords masked).
    #[command(after_long_help = VPS_SHOW_EXAMPLES)]
    Show {
        /// VPS name.
        name: String,
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
    },

    /// Shows the configuration file path.
    #[command(after_long_help = VPS_PATH_EXAMPLES)]
    Path,

    /// Diagnostics for XDG layers / path / schema.
    #[command(after_long_help = VPS_DOCTOR_EXAMPLES)]
    Doctor {
        /// JSON output (from global `--json`).
        #[arg(from_global)]
        json: bool,
        /// After local diagnostics, probe hosts over SSH (bounded fan-out).
        /// Default scope is every registered host; use `--hosts` for a subset.
        #[arg(long, action = ArgAction::SetTrue)]
        probe_ssh: bool,
        /// Comma-separated host subset for `--probe-ssh` only (ignored without probe).
        #[arg(long, value_name = "LIST")]
        hosts: Option<String>,
    },

    /// Exports hosts (passwords redacted by default).
    #[command(after_long_help = VPS_EXPORT_EXAMPLES)]
    Export {
        /// Include secrets in the export.
        #[arg(long)]
        include_secrets: bool,
        /// Output file (stdout if omitted). Written atomically with mode 0o600.
        #[arg(long, short, value_name = "PATH", value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
        /// Agent-first JSON envelope (`event: vps-export`; from global `--json`).
        /// Default body is **TOML** on text; JSON when format/json.
        #[arg(from_global)]
        json: bool,
        /// Acknowledge writing plaintext secrets to stdout (pipe/non-TTY). Prefer `--output`.
        #[arg(long)]
        i_understand_secrets_on_stdout: bool,
    },

    /// Imports hosts from a TOML file or JSON `vps-export` envelope (EN + legacy PT keys).
    #[command(after_long_help = VPS_IMPORT_EXAMPLES)]
    Import {
        /// Source file (TOML wire or JSON export envelope).
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        file: PathBuf,
        /// Allow hosts without full auth (redacted export / skeleton).
        #[arg(long)]
        allow_incomplete: bool,
    },
}
