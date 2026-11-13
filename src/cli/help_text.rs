//! Static help text for the clap command tree, kept out of the type definitions.
//!
//! Examples are plain commands that parse against the current flags; the leaf examples are
//! shown by `--help` only (`after_long_help`), so `-h` stays short.
#![forbid(unsafe_code)]

/// Usage block for `exec`, spelling the three ways to designate a target.
///
/// # Why this is overridden rather than left to clap
///
/// The positional is one vector (`num_args = 1..=2`) because clap fills positionals
/// by order and cannot see which selector is active — two named slots would bind
/// `exec --all uptime` to a `VPS` slot and leave the command empty. The cost of that
/// correct choice is a generated usage line that reads `--use-active <VPS>
/// [COMMAND]...`, advertising a shape the parser rejects. That line is printed at
/// exactly the moment the caller already got it wrong, so it teaches the error twice.
///
/// Layout follows the clap contract: first line unindented, the rest indented by
/// seven spaces, no trailing newline.
pub(super) const EXEC_USAGE_BLOCK: &str = "ssh-cli exec [OPTIONS] <VPS> <COMMAND>\n       \
     ssh-cli exec [OPTIONS] --use-active <COMMAND>\n       \
     ssh-cli exec [OPTIONS] --all|--hosts <LIST>|--tags <LIST> <COMMAND>";

/// Usage block for `sudo-exec`. Same three forms, different verb.
pub(super) const SUDO_EXEC_USAGE_BLOCK: &str =
    "ssh-cli sudo-exec [OPTIONS] <VPS> <COMMAND>\n       \
     ssh-cli sudo-exec [OPTIONS] --use-active <COMMAND>\n       \
     ssh-cli sudo-exec [OPTIONS] --all|--hosts <LIST>|--tags <LIST> <COMMAND>";

/// Usage block for `su-exec`. Same three forms, different verb.
pub(super) const SU_EXEC_USAGE_BLOCK: &str = "ssh-cli su-exec [OPTIONS] <VPS> <COMMAND>\n       \
     ssh-cli su-exec [OPTIONS] --use-active <COMMAND>\n       \
     ssh-cli su-exec [OPTIONS] --all|--hosts <LIST>|--tags <LIST> <COMMAND>";

/// Usage block for `health-check`, which has a target but no command.
pub(super) const HEALTH_CHECK_USAGE: &str = "ssh-cli health-check [OPTIONS] <VPS_NAME>\n       \
     ssh-cli health-check [OPTIONS] --use-active\n       \
     ssh-cli health-check [OPTIONS] --all|--hosts <LIST>";

/// Footer of the root `--help`: copyable examples and the exit codes the product emits.
///
/// The codes follow `sysexits.h`; the authoritative mapping is `SshCliError::exit_code`.
pub(super) const ROOT_AFTER_HELP: &str = "Examples:\n  \
ssh-cli vps add --name prod --host h.example --user deploy --key ~/.ssh/id_ed25519\n  \
printf '%s' \"$PASS\" | ssh-cli exec prod 'hostname' --json --password-stdin\n  \
ssh-cli scp upload prod ./a.bin /tmp/a.bin --json\n  \
ssh-cli tunnel prod 8080 127.0.0.1 80 --timeout-ms 60000 --json\n  \
ssh-cli vps export -o /tmp/hosts.toml\n\n\
Exit status:\n  \
0    success\n  \
1    runtime failure, remote command failure or partial multi-host failure\n  \
2    usage error from the argument parser\n  \
64   invalid arguments, including invalid host names\n  \
65   data error in JSON, TOML, schema or config\n  \
66   VPS or input file not found\n  \
69   required service unavailable (locked OS keyring, expired config lock wait)\n  \
70   internal software failure\n  \
73   cannot create the config or output file\n  \
74   I/O error, SSH connection failure or timeout\n  \
77   authentication rejected, host key changed or sudo policy\n  \
130  interrupted by SIGINT\n  \
141  broken pipe on stdout or stderr\n  \
143  terminated by SIGTERM";

/// Copyable example shown by `ssh-cli vps add --help`.
pub(super) const VPS_ADD_EXAMPLES: &str = "Examples:\n  ssh-cli vps add --name prod --host h.example --user deploy --key ~/.ssh/id_ed25519 --tag web";

/// Copyable example shown by `ssh-cli vps edit --help`.
pub(super) const VPS_EDIT_EXAMPLES: &str =
    "Examples:\n  ssh-cli vps edit prod --port 2222 --timeout 30000";

/// Copyable example shown by `ssh-cli vps rename --help`.
pub(super) const VPS_RENAME_EXAMPLES: &str = "Examples:\n  ssh-cli vps rename prod prod-eu";

/// Copyable example shown by `ssh-cli vps doctor --help`.
pub(super) const VPS_DOCTOR_EXAMPLES: &str =
    "Examples:\n  ssh-cli vps doctor --probe-ssh --hosts prod,prod-eu";

/// Copyable example shown by `ssh-cli exec --help`.
pub(super) const EXEC_EXAMPLES: &str = "Examples:\n  ssh-cli exec prod 'uptime' --json";

/// Copyable example shown by `ssh-cli scp upload --help`.
pub(super) const SCP_UPLOAD_EXAMPLES: &str =
    "Examples:\n  ssh-cli scp upload prod ./app.tar.gz /tmp/app.tar.gz --json";

/// Copyable example shown by `ssh-cli sftp ls --help`.
pub(super) const SFTP_LS_EXAMPLES: &str = "Examples:\n  ssh-cli sftp ls prod /var/log --json";

/// Copyable example shown by `ssh-cli tunnel --help`.
pub(super) const TUNNEL_EXAMPLES: &str =
    "Examples:\n  ssh-cli tunnel prod 8080 127.0.0.1 80 --timeout-ms 60000 --json";

/// Copyable example shown by `ssh-cli health-check --help`.
pub(super) const HEALTH_CHECK_EXAMPLES: &str = "Examples:\n  ssh-cli health-check prod --json";

/// Copyable example shown by `ssh-cli secrets status --help`.
pub(super) const SECRETS_STATUS_EXAMPLES: &str = "Examples:\n  ssh-cli secrets status --json";

/// Copyable example shown by `ssh-cli locale set --help`.
pub(super) const LOCALE_SET_EXAMPLES: &str = "Examples:\n  ssh-cli locale set pt-BR";

/// Copyable example shown by `ssh-cli tls mtls import --help`.
pub(super) const MTLS_IMPORT_EXAMPLES: &str =
    "Examples:\n  ssh-cli tls mtls import --name ci --cert ./client.pem --key ./client.key";

/// Copyable example shown by `ssh-cli tls acme issue --help`.
pub(super) const ACME_ISSUE_EXAMPLES: &str =
    "Examples:\n  ssh-cli tls acme issue --domain example.com --staging --print-challenge";

/// Copyable example shown by `ssh-cli vps list --help`.
pub(super) const VPS_LIST_EXAMPLES: &str = "Examples:\n  ssh-cli vps list --tag web --json";

/// Copyable example shown by `ssh-cli vps remove --help`.
pub(super) const VPS_REMOVE_EXAMPLES: &str = "Examples:\n  ssh-cli vps remove prod";

/// Copyable example shown by `ssh-cli vps show --help`.
pub(super) const VPS_SHOW_EXAMPLES: &str = "Examples:\n  ssh-cli vps show prod --json";

/// Copyable example shown by `ssh-cli vps path --help`.
pub(super) const VPS_PATH_EXAMPLES: &str = "Examples:\n  ssh-cli vps path --json";

/// Copyable example shown by `ssh-cli vps export --help`.
pub(super) const VPS_EXPORT_EXAMPLES: &str =
    "Examples:\n  ssh-cli vps export --output ./hosts.toml";

/// Copyable example shown by `ssh-cli vps import --help`.
pub(super) const VPS_IMPORT_EXAMPLES: &str =
    "Examples:\n  ssh-cli vps import --file ./hosts.toml --json";

/// Copyable example shown by `ssh-cli connect --help`.
pub(super) const CONNECT_EXAMPLES: &str = "Examples:\n  ssh-cli connect prod";

/// Copyable example shown by `ssh-cli sudo-exec --help`.
pub(super) const SUDO_EXEC_EXAMPLES: &str =
    "Examples:\n  ssh-cli sudo-exec prod 'systemctl restart nginx' --json";

/// Copyable example shown by `ssh-cli su-exec --help`.
pub(super) const SU_EXEC_EXAMPLES: &str = "Examples:\n  ssh-cli su-exec prod 'id' --json";

/// Copyable example shown by `ssh-cli scp download --help`.
pub(super) const SCP_DOWNLOAD_EXAMPLES: &str =
    "Examples:\n  ssh-cli scp download prod /var/log/app.log ./logs/ --json";

/// Copyable example shown by `ssh-cli sftp upload --help`.
pub(super) const SFTP_UPLOAD_EXAMPLES: &str =
    "Examples:\n  ssh-cli sftp upload prod ./dist /srv/app --recursive --json";

/// Copyable example shown by `ssh-cli sftp download --help`.
pub(super) const SFTP_DOWNLOAD_EXAMPLES: &str =
    "Examples:\n  ssh-cli sftp download prod /srv/app ./dist --recursive --json";

/// Copyable example shown by `ssh-cli sftp mkdir --help`.
pub(super) const SFTP_MKDIR_EXAMPLES: &str = "Examples:\n  ssh-cli sftp mkdir prod /srv/app --json";

/// Copyable example shown by `ssh-cli sftp rmdir --help`.
pub(super) const SFTP_RMDIR_EXAMPLES: &str = "Examples:\n  ssh-cli sftp rmdir prod /srv/app --json";

/// Copyable example shown by `ssh-cli sftp rm --help`.
pub(super) const SFTP_RM_EXAMPLES: &str =
    "Examples:\n  ssh-cli sftp rm prod /srv/app/old.tar.gz --json";

/// Copyable example shown by `ssh-cli sftp stat --help`.
pub(super) const SFTP_STAT_EXAMPLES: &str = "Examples:\n  ssh-cli sftp stat prod /srv/app --json";

/// Copyable example shown by `ssh-cli sftp rename --help`.
pub(super) const SFTP_RENAME_EXAMPLES: &str =
    "Examples:\n  ssh-cli sftp rename prod /srv/app /srv/app-old --json";

/// Copyable example shown by `ssh-cli secrets init --help`.
pub(super) const SECRETS_INIT_EXAMPLES: &str = "Examples:\n  ssh-cli secrets init --keyring";

/// Copyable example shown by `ssh-cli secrets reencrypt --help`.
pub(super) const SECRETS_REENCRYPT_EXAMPLES: &str = "Examples:\n  ssh-cli secrets reencrypt --json";

/// Copyable example shown by `ssh-cli completions --help`.
pub(super) const COMPLETIONS_EXAMPLES: &str = "Examples:\n  ssh-cli completions bash";

/// Copyable example shown by `ssh-cli commands --help`.
pub(super) const COMMANDS_EXAMPLES: &str = "Examples:\n  ssh-cli commands --json";

/// Copyable example shown by `ssh-cli schema --help`.
pub(super) const SCHEMA_EXAMPLES: &str = "Examples:\n  ssh-cli schema vps-show";

/// Copyable example shown by `ssh-cli doctor --help`.
pub(super) const DOCTOR_EXAMPLES: &str =
    "Examples:\n  ssh-cli doctor --probe-ssh --hosts prod --json";

/// Copyable example shown by `ssh-cli locale show --help`.
pub(super) const LOCALE_SHOW_EXAMPLES: &str = "Examples:\n  ssh-cli locale show --json";

/// Copyable example shown by `ssh-cli locale clear --help`.
pub(super) const LOCALE_CLEAR_EXAMPLES: &str = "Examples:\n  ssh-cli locale clear";

/// Copyable example shown by `ssh-cli tls provider --help`.
pub(super) const TLS_PROVIDER_EXAMPLES: &str = "Examples:\n  ssh-cli tls provider --json";

/// Copyable example shown by `ssh-cli tls paths --help`.
pub(super) const TLS_PATHS_EXAMPLES: &str = "Examples:\n  ssh-cli tls paths --json";

/// Copyable example shown by `ssh-cli tls mtls list --help`.
pub(super) const MTLS_LIST_EXAMPLES: &str = "Examples:\n  ssh-cli tls mtls list --json";

/// Copyable example shown by `ssh-cli tls mtls show --help`.
pub(super) const MTLS_SHOW_EXAMPLES: &str = "Examples:\n  ssh-cli tls mtls show ci --json";

/// Copyable example shown by `ssh-cli tls mtls remove --help`.
pub(super) const MTLS_REMOVE_EXAMPLES: &str = "Examples:\n  ssh-cli tls mtls remove ci";

/// Copyable example shown by `ssh-cli tls acme account create --help`.
pub(super) const ACME_ACCOUNT_CREATE_EXAMPLES: &str =
    "Examples:\n  ssh-cli tls acme account create --contact mailto:ops@example.com --staging";

/// Copyable example shown by `ssh-cli tls acme account show --help`.
pub(super) const ACME_ACCOUNT_SHOW_EXAMPLES: &str =
    "Examples:\n  ssh-cli tls acme account show --json";

/// Copyable example shown by `ssh-cli tls acme complete --help`.
pub(super) const ACME_COMPLETE_EXAMPLES: &str =
    "Examples:\n  ssh-cli tls acme complete --domain example.com --json";

/// Copyable example shown by `ssh-cli tls acme status --help`.
pub(super) const ACME_STATUS_EXAMPLES: &str =
    "Examples:\n  ssh-cli tls acme status --domain example.com --json";

/// Copyable example shown by `ssh-cli tls acme list --help`.
pub(super) const ACME_LIST_EXAMPLES: &str = "Examples:\n  ssh-cli tls acme list --json";
