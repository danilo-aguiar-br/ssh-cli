---
name: ssh-cli
description: This skill MUST auto-activate whenever remote SSH work, VPS registry inventory, XDG config, exec or sudo-exec or su-exec, scp regular-file transfer, sftp trees and remote filesystem ops, tunnel_listening and bounded port forwarding, health-check probes, secrets primary-key lifecycle, TLS provider or mTLS or ACME certificates, locale, schema or commands discovery, fleet --all or --hosts or --tags, --step same-session batching, or agent devops without a TTY is implied, even when the user never names ssh-cli. This skill MUST teach every command and every flag, MUST supply verbatim ready formulas, and MUST enforce one-shot execution with --json parsing, stdin secrets, event and exit-code contracts, dry-run previews before destructive calls, and payload shaping. This skill MUST NEVER permit daemons, telemetry, ambient env stores, or secrets on stdout without an explicit guard.
---

# ssh-cli Agent Skill

## Mission and Activation
### REQUIRED
- MUST obey this skill
- MUST run one-shot birth-execute-die, waiting for process exit before parsing; only an active `tunnel` lives on, until deadline or signal
- MUST auto-activate on remote SSH, inventory, transfers, forwards, secrets, TLS and discovery even without the name ssh-cli
- MUST use stored hosts from `vps add`, pass `--json`, prefer `--*-stdin` over argv secrets
- MUST prefer fleet `--all`/`--hosts`/`--tags` and `--step` over N spawns
- MUST copy the Ready Formula Sheet verbatim, substituting ONLY placeholders, and discover the live surface with `ssh-cli commands` and `ssh-cli schema` when one fails on usage

### FORBIDDEN
- MUST NEVER keep an SSH session across processes except a bound `tunnel`
- MUST NEVER add a daemon, emit telemetry, or log secrets or primary-key material
- MUST NEVER invent a flag, command, event or JSON key this skill does not name


## Action Prompts
### REQUIRED — MUST follow this order on every non-trivial remote task
1. DISCOVER with `commands` and `schema`; INSPECT with `doctor --json`, `vps path` and `secrets status --json`
2. REGISTER the host with password, `--key`, or `--use-agent` plus `--agent-socket`; attach `--tag` and TLS flags
3. PROBE with `health-check <NAME> --json`, or fleet `--all`/`--hosts`, NEVER `--tags` here
4. EXECUTE with `exec`/`sudo-exec`/`su-exec` and `--json`; batch same-session with `--step`
5. TRANSFER regular files with `scp`, trees and filesystem ops with `sftp`, NEVER `--tags` on either
6. FORWARD only with `tunnel`, always with `--timeout-ms`
7. PREVIEW every unattended destructive call with `--dry-run`
8. PARSE the process exit, then stdout success, then the stderr error envelope
9. SANITIZE durable logs so no secret and no primary-key material remains


## Command Catalog
### REQUIRED — MUST treat these 47 leaves as the whole surface
- `vps add`, `vps list`, `vps show`, `vps edit`, `vps remove`, `vps path`, `vps export`, `vps import` — registry, always masked
- `vps doctor`, `doctor` — XDG and schema diagnostics; `doctor` is the root ALIAS
- `connect` — writes the active-host marker ONLY, NEVER a session
- `exec`, `sudo-exec`, `su-exec` — one remote command; elevation is one-shot with safe `sh -c` packing
- `scp upload`, `scp download` — regular files ONLY
- `sftp upload`, `sftp download`, `sftp ls`, `sftp mkdir`, `sftp rmdir`, `sftp rm`, `sftp stat`, `sftp rename` — trees and remote filesystem
- `tunnel` — bounded forward in four modes
- `health-check` — connectivity probe
- `secrets status`, `secrets init`, `secrets reencrypt` — primary-key lifecycle
- `commands`, `schema`, `completions` — command tree, schema catalog, shell completions
- `locale show`, `locale set`, `locale clear` — UI language
- `tls provider`, `tls paths` — CryptoProvider status and XDG TLS layout
- `tls mtls list`, `tls mtls import`, `tls mtls show`, `tls mtls remove` — client identities
- `tls acme account create`, `tls acme account show`, `tls acme issue`, `tls acme complete`, `tls acme status`, `tls acme list` — account and DNS-01 lifecycle


## Global Flags
### REQUIRED — MUST place these BEFORE the subcommand
- `--json`, `-q`/`--quiet`, `--no-color`, `--config-dir <DIR>` — `--json` is MANDATORY for agent parsing
- `--output-format text|json` — when omitted, JSON whenever stdout is not a TTY, the `vps export` body INCLUDED
- `--lang <LOCALE>`, `-v`/`-vv`/`-vvv` — BCP47 UI negotiated to `en` or `pt-BR`; crate-scoped info, debug, trace over a default of `error`
- `--no-input` — refuse stdin declaratively, so every `--*-stdin` flag fails with exit 64 instead of blocking on an absent human
- `--dry-run` — print the plan and exit
- `--disable-sudo` — suppress elevation for THIS invocation only
- `--replace-host-key`, `--allow-plaintext-secrets` — need human intent; plaintext is tests only
- `--secrets-key-file <PATH>`, `--use-keyring` — primary-key from a 64-hex file, or from the OS keyring
- `--timeout <MS>` — SSH operation timeout in MILLISECONDS
- `--max-concurrency <N>` — fleet and accept cap, 1 to 64, auto when omitted
- `--fail-fast`, `--scp-file-concurrency <N>` — stop after the first fleet failure; parallel SCP channels on one session, default 1

### REQUIRED — Payload shaping
- MUST shape with these flags rather than piping through `jaq`, because the cut happens BEFORE serialization
- `--select <KEYS>` — keep only these comma-separated keys; `--fields` is the same flag; a missing key is skipped, never null
- `--filter <EXPR>` — `key=value`, `key!=value`, `key~substring`, repeat to AND
- `--limit <N>`, `--sort <KEY>`, `--dedupe-by <KEY>`, `--count-only` — cap, sort ascending with missing keys last, drop repeats, or return a count
- `--truncate-content <N>` — shorten strings above N CHARACTERS, never bytes, never splitting UTF-8
- `--max-output-bytes <N>` — drop trailing elements, never slice the JSON text
- MUST know `vps list --json` emits a BARE ARRAY at the root, so its shaping report goes to stderr

### REQUIRED — Dry-run preview
- MUST preview every unattended destructive call; the plan is JSON on stdout even in text mode
- MUST read the `dry-run` event with `operation`, `dry_run` true, `executed` false; RUN `ssh-cli schema dry-run` for the body
- MUST know the preview exists ONLY on `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt`, so exit 64 elsewhere means "no preview here" and NEVER a broken plan
- MUST expect preconditions first, so `--dry-run vps remove <absent>` still exits 66
- MUST read `hosts[].replaces_existing` on `vps import`, because that field and not the count decides whether the import is safe
- MUST read `hosts_to_reencrypt` on `secrets init --force`, because rotating without re-encrypting those hosts loses their at-rest secrets
- MUST know `sftp rm` and `sftp rmdir` previews never connect
- MUST NEVER read a missing success event as inaction; read `executed` false

### FORBIDDEN
- MUST NEVER treat `SSH_CLI_HOME`, `SSH_CLI_LANG`, `SSH_CLI_FORCE_TEXT`, `SSH_CLI_MAX_CONCURRENCY`, `SSH_CLI_SECRETS_KEY` or `SSH_CLI_SECRETS_KEY_FILE` as configuration stores; they are fail-closed
- MUST NEVER rely on ambient `RUST_LOG`, which is IGNORED, and NEVER expect russh password dumps


## Local Flags That Look Global
### REQUIRED — MUST place these AFTER the subcommand
- MUST watch the four that read like globals and are NOT, because before the subcommand clap exits 2 — `tunnel --bind <ADDR>`, `tunnel --i-accept-network-exposure`, `tunnel --timeout-ms <MS>`, `vps doctor --probe-ssh`
- MUST know `tunnel --bind` is loopback by default and parsed as an IP, so a typo fails at parse time, while `--i-accept-network-exposure` is REQUIRED whenever the exposed end is not loopback
- MUST prefer `--max-command-chars <N>` and `--max-output-chars <N>` on `vps add|edit`, because `--max-chars <N>` is only a LEGACY ALIAS of the command cap

### REQUIRED — The elevation flag means two different things
- MUST know `--disable-sudo` BEFORE the subcommand suppresses elevation for ONE invocation and changes nothing on disk
- MUST know `vps edit --disable-sudo` AFTER the subcommand writes `disable_sudo` into the config and PERMANENTLY disables elevation on that host
- MUST use `vps edit --enable-sudo` as the ONLY undo; the pair conflicts on `vps edit` and clap rejects it
- MUST NEVER reach for the persistent form when ephemeral suppression was wanted, because the spellings are identical and only the position reveals the lifetime


## Lifecycle and JSON
### REQUIRED
- MUST parse success ONLY from stdout; prose and hard-failure envelopes go to stderr
- MUST know the `vps export` BODY follows the resolved format, so an agent, whose stdout is never a TTY, gets JSON even into a `.toml` filename; TOML needs `--output-format text`
- MUST parse `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import`, `vps-export`, and read `secrets_key_auto_created` inside the SAME `vps-added` document, NEVER a second event
- MUST parse `vps-doctor` with a BOOLEAN `local.secrets_plaintext_opt_out`, and `ssh_probe` either null or a health-check batch
- MUST read exec `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr`, `duration_ms`, and report truncation when either flag is true
- MUST treat single-step `exec --json` as exactly ONE object; `--step` emits one object per step with a 0-based `step` and its `command`
- MUST read `scp-transfer` with `ok`/`direction`/`bytes`/`duration_ms`, and `sftp-transfer`, `sftp-list`, `sftp-fs-op`, `sftp-batch`
- MUST read fleet batches `health-check-batch`, `exec-batch`, `scp-batch`, `sftp-batch`, each with `max_concurrency`
- MUST read `tunnel_listening` with `local_port`, `remote_host`, `remote_port`, `timeout_ms`, `bind`, `mode`
- MUST read `tunnel_closed` with `reason`, `forwards_served`, `capacity_waits`, `duration_ms`, `mode`, distinguishing `reason` between `deadline`, `signal` and `accept_error`, which all share exit 0
- MUST read envelope fields `exit_code`, `message`, `remote_exit_code`, `retryable`, `error_class`, `suggestion`
- MUST RUN `ssh-cli schema` with NO name to list every valid schema, and NEVER guess a name

### FORBIDDEN
- MUST NEVER invent a missing key, expect several objects from single-step `exec --json`, or assume an open channel from a prior process


## Registry Auth Secrets
### REQUIRED
- MUST give each host a unique `--name` and EXACTLY ONE primary auth — password, `--password-stdin`, `--key`, or `--use-agent` with `--agent-socket`, switchable later with `vps edit --use-agent`
- MUST pass `--port` when it is not 22, `--check` for an immediate probe, repeatable `--tag` for fleet selection, and `--tls` plus optional `--tls-sni`, `--tls-client-cert`, `--tls-client-key` for SSH-over-TLS, undone by `--no-tls`
- MUST read masks correctly — absent is JSON `null`, stored is the fixed mask, for `password`, `sudo_password`, `su_password` and `key_passphrase` alike
- MUST treat host timeouts as MILLISECONDS; under 1000 emits a stderr warning
- MUST export without secrets by default, where an empty redacted secret is an EMPTY string and never an encrypted blob, and MUST require human approval plus `--output` or `--i-understand-secrets-on-stdout` for `--include-secrets`
- MUST accept import from TOML with English keys plus legacy Portuguese aliases, or a JSON `vps-export` where `added_at` may be omitted; invalid TOML is exit 65
- MUST know TOML says `username` where the JSON envelope says `user`, so a TOML import written with `user` is exit 65
- MUST prefer `--password-stdin`, `--key-passphrase-stdin`, `--sudo-password-stdin`, `--su-password-stdin` over argv, which warns, and MUST apply these as runtime overrides on `exec`, `scp`, `sftp`, `tunnel` and `health-check` when stored credentials are insufficient
- MUST pass the explicit host name whenever certainty matters, since a nameless `health-check` targets whatever `connect` last marked active
- MUST resolve the primary-key in order — `--secrets-key-file`, then the OS keyring under `--use-keyring`, then a `secrets.key` auto-created under the config directory; the keyring also accepts the legacy alias name on read
- MUST expect atomic writes, mode 0600 on Unix
- MUST treat a host-key mismatch as a HARD STOP, using `--replace-host-key` only after human confirmation

### FORBIDDEN
- MUST NEVER create a host with empty credentials, invent a password when JSON shows `null`, treat the mask as a real secret, commit raw secrets, print primary-key material, or enable plaintext outside tests
- MUST NEVER combine a password or key with `--use-agent` on `vps add`, and NEVER disable TOFU for convenience


## Fleet Exec Elevation
### REQUIRED
- MUST use `--all` or `--hosts <A>,<B>` on `exec`, `sudo-exec`, `su-exec`, `scp`, `sftp` and `health-check`
- MUST use `--tags <LIST>` ONLY on `exec`, `sudo-exec` and `su-exec`, matching every host carrying ANY listed tag
- MUST treat `--all`, `--hosts` and `--tags` as mutually exclusive; clap rejects any pair with exit 2, and an empty registry plus a selector is exit 64
- MUST treat `tunnel` as single-host, so multi-host forwarding needs N one-shots
- MUST send a non-empty remote command; an empty command or an empty `--step` is exit 64 with the message `empty command`, always in English
- MUST respect the command and output caps, raising them with `vps edit` when a payload legitimately exceeds them
- MUST use `--step <CMD>` repeatably to run several commands on ONE session
- MUST honour `--disable-sudo` and the host setting, treat elevation as one-shot, and attach `--description` when the remote audit trail matters

### FORBIDDEN
- MUST NEVER invent `--tags` on `health-check`, `scp` or `sftp`, spawn one process per host when a fleet selector covers the set, prepend a raw `sudo` to `exec`, or assume a sticky elevated shell


## SCP SFTP Tunnel Health
### REQUIRED
- MUST use `scp` for regular files ONLY, ordering upload local then remote, download remote then local
- MUST expect a 32 KiB stream, with downloads via `.ssh-cli.partial` then an atomic rename
- MUST treat mtime and mode carriage as BEST-EFFORT and read `mtime_preserved` and `durable` in `scp-transfer`; the exit code does NOT say the timestamp landed or the entry is durable
- MUST treat a missing remote file on SCP as exit 66 with the message `file not found`, NEVER as 74
- MUST use `sftp` for trees with `--recursive`, and `sftp rmdir` only on an EMPTY directory
- MUST know recursive SFTP NEVER follows a symlink, and that permission masks are DIRECTIONAL — the outbound upload mask keeps setuid, setgid and sticky on a file you already own, while the inbound download mask `SFTP_PERM_MASK_UNTRUSTED` strips them so server-sent elevation bits never reach the local file
- MUST verify destination size or checksum after a critical SFTP upload, NEVER trusting the byte count alone
- MUST pass `--timeout` on `exec`, `scp`, `sftp` and `health-check`, and `--timeout-ms` ONLY on `tunnel`, on EVERY tunnel
- MUST use the positional form `tunnel <VPS> <LOCAL_PORT> [REMOTE_HOST] [REMOTE_PORT]`, treating local port `0` as ephemeral and reading the real `local_port` after bind
- MUST read `mode` in `tunnel_listening` and `tunnel_closed` — `local`, `socks5`, `streamlocal` or `reverse` — because it says how to read the sibling fields
- MUST OMIT `REMOTE_HOST` and `REMOTE_PORT` with `--socks5` and `--remote-socket`, since neither has a single destination, and MUST PASS them with `--reverse`, where they are what the SERVER binds; `REMOTE_PORT 0` is legal ONLY under `--reverse`, reported back in `local_port`. Every other combination is exit 64
- MUST pass `--i-accept-network-exposure` when the exposed end leaves loopback; under `--reverse` that end is the positional remote host, compared as TEXT because a name and the empty string both carry meaning, so a typo there is exit 64 and not a parse error
- MUST know `--bind` is accepted and then DISCARDED under `--reverse`; set the server-side listener through the positional remote host
- MUST expect `--socks5` to speak no-auth CONNECT only, refusing BIND and UDP ASSOCIATE, and forwarding host names unresolved
- MUST pass an ABSOLUTE POSIX path to `--remote-socket`, naming a socket on the SERVER; a relative path is exit 64
- MUST treat a refused forward request as POLICY, never a transient error
- MUST WAIT for `tunnel_listening` before using the local port, leaving the process alive until deadline or signal

### FORBIDDEN
- MUST NEVER scp a directory or invent a recursive flag on scp; trees MUST use `sftp --recursive`
- MUST NEVER swap `--timeout` and `--timeout-ms`, or treat a leftover partial file as a final artifact
- MUST NEVER bind a tunnel to every interface without an explicit security decision, and NEVER auto-replace a host key


## Locale TLS Discovery
### REQUIRED
- MUST resolve locale by precedence — `--lang`, then `locale set`, then the system, then English
- MUST manage identities with `tls mtls import --name --cert --key`, then `list`/`show`/`remove`
- MUST create an ACME account with a repeatable `--contact` in mailto form
- MUST issue in TWO steps — `tls acme issue --domain --print-challenge`, publish the DNS TXT record, then `tls acme complete --domain` — and NEVER invent an interactive wait loop
- MUST treat permanent ACME validation failure as exit 64 with `retryable` false, and a transient one as exit 74 only when the envelope marks it retryable

### FORBIDDEN
- MUST NEVER store certificates outside the XDG layout, omit `--print-challenge` on issue, or invent a schema name


## Exit Codes and Retry
### REQUIRED
- MUST map 0 success, 1 general, 64 usage, 65 data, 66 not found, 69 unavailable, 70 internal software, 73 cannot create, 74 IO or SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- MUST treat 69 as RETRYABLE, because a locked or absent OS keyring answers the same argv once the service is up
- MUST treat 70 as PERMANENT, because a CSPRNG that will not produce bytes is fixed neither by waiting nor by changing arguments
- MUST treat empty command as 64, invalid import TOML as 65, missing remote SCP file as 66, auth failure as 77, permanent ACME validation as 64
- MUST treat a tunnel deadline reached AFTER bind as exit 0, and a timeout BEFORE bind as exit 74
- MUST retry at most twice on 74 with backoff and ONLY when the envelope says `retryable` is true, and MUST fail fast on 64, 65, 66 and 77, changing the inputs first
- MUST surface the remote `exit_code` from success JSON and `remote_exit_code` from an error envelope separately from the CLI process exit


## Ready Formula Sheet
### REQUIRED — MUST RUN these VERBATIM and substitute ONLY placeholders
- Discovery and identity
  - `ssh-cli --version`
  - `ssh-cli commands`
  - `ssh-cli schema`
  - `ssh-cli schema <NAME>`
  - `ssh-cli completions bash|zsh|fish|elvish|powershell`
  - `ssh-cli locale show --json`
  - `ssh-cli locale set <LOCALE>`
  - `ssh-cli locale clear`
  - `ssh-cli --lang <LOCALE> vps list --json`
  - `ssh-cli -v exec <NAME> "true" --json`, also `-vv` and `-vvv`
- Registry
  - `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --port <PORT> --tag <TAG> --tag <TAG2> --check`
  - `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --use-agent --agent-socket <SOCK> --tag <TAG>`
  - `printf '%s' "$PASS" | ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --password-stdin`
  - `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --tls --tls-sni <SNI> --tls-client-cert <CERT> --tls-client-key <KEY>`
  - `printf '%s' "$SUDO" | ssh-cli vps edit <NAME> --sudo-password-stdin`
  - `printf '%s' "$SU" | ssh-cli vps edit <NAME> --su-password-stdin`
  - `ssh-cli vps edit <NAME> --timeout <MS> --max-command-chars <N> --max-output-chars <N>`
  - `ssh-cli vps edit <NAME> --use-agent --agent-socket <SOCK>`
  - `ssh-cli vps edit <NAME> --tls --tls-sni <SNI>`, and `ssh-cli vps edit <NAME> --no-tls`
  - `ssh-cli vps edit <NAME> --disable-sudo` PERSISTS the block; `ssh-cli vps edit <NAME> --enable-sudo` is the ONLY undo
  - `ssh-cli vps list --json`, and `ssh-cli vps list --tag <TAG> --json`
  - `ssh-cli vps show <NAME> --json`
  - `ssh-cli vps path`
  - `ssh-cli doctor --json`, and the identical `ssh-cli vps doctor --json`
  - `ssh-cli doctor --probe-ssh --json`, and `ssh-cli vps doctor --probe-ssh --hosts <A>,<B> --json`
  - `ssh-cli vps export -o /tmp/hosts.json`, and `ssh-cli --output-format text vps export -o /tmp/hosts.toml`
  - `ssh-cli vps export --json`
  - `ssh-cli vps export --include-secrets -o /tmp/hosts-secrets.toml`
  - `ssh-cli vps import --file /tmp/hosts.toml`
  - `ssh-cli vps import --file /tmp/hosts.json`
  - `ssh-cli vps import --file /tmp/hosts-redacted.toml --allow-incomplete`
  - `ssh-cli connect <NAME>`
  - `ssh-cli vps remove <NAME>`
- Execution and fleet
  - `ssh-cli exec <NAME> "<CMD>" --json`
  - `ssh-cli exec <NAME> "<CMD>" --json --timeout <MS> --description "<AUDIT>"`
  - `ssh-cli -q exec <NAME> "<CMD>" --json`
  - `ssh-cli exec <NAME> "<CMD>" --step "<CMD2>" --step "<CMD3>" --json`
  - `ssh-cli exec <NAME> "id" --json --use-agent --agent-socket <SOCK>`
  - `printf '%s' "$KEY_PASS" | ssh-cli exec <NAME> "id" --json --key <KEY_PATH> --key-passphrase-stdin`
  - `ssh-cli sudo-exec <NAME> "<CMD>" --json`, and with `--step "<CMD2>"`
  - `printf '%s' "$SUDO" | ssh-cli sudo-exec <NAME> "<CMD>" --json --sudo-password-stdin`
  - `ssh-cli su-exec <NAME> "<CMD>" --json`
  - `printf '%s' "$SU" | ssh-cli su-exec <NAME> "<CMD>" --json --su-password-stdin`
  - `ssh-cli --max-concurrency <N> exec --all "<CMD>" --json`
  - `ssh-cli --fail-fast exec --all "<CMD>" --json`
  - `ssh-cli exec --hosts <A>,<B> "<CMD>" --json`
  - `ssh-cli exec --tags <TAG1>,<TAG2> "<CMD>" --json`
  - `ssh-cli sudo-exec --all "<CMD>" --json`, and `ssh-cli sudo-exec --tags <TAG> "<CMD>" --json`
  - `ssh-cli su-exec --all "<CMD>" --json`
- Transfer
  - `ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json`
  - `ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json`
  - `ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --timeout <MS>`
  - `ssh-cli scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json`
  - `ssh-cli scp download <NAME> <R1> <R2> <LOCAL_DIR> --json`
  - `ssh-cli --scp-file-concurrency <N> scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json`
  - `ssh-cli scp upload --all <LOCAL_FILE> <REMOTE_FILE> --json`
  - `ssh-cli scp upload --all <F1> <F2> <REMOTE_DIR> --json`
  - `ssh-cli scp download --all <REMOTE_FILE> <LOCAL_PREFIX> --json`
  - `ssh-cli scp upload --hosts <A>,<B> <LOCAL_FILE> <REMOTE_FILE> --json`
  - `printf '%s' "$PASS" | ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json --password-stdin`
  - `printf '%s' "$KEY_PASS" | ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --key <KEY_PATH> --key-passphrase-stdin`
  - `ssh-cli sftp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json`
  - `ssh-cli sftp upload --recursive <NAME> <LOCAL_DIR> <REMOTE_DIR> --json`
  - `ssh-cli sftp download --recursive <NAME> <REMOTE_DIR> <LOCAL_DIR> --json`
  - `ssh-cli sftp ls <NAME> <REMOTE_DIR> --json`
  - `ssh-cli sftp mkdir <NAME> <REMOTE_DIR> --json`
  - `ssh-cli sftp rmdir <NAME> <REMOTE_DIR> --json`
  - `ssh-cli sftp rm <NAME> <REMOTE_FILE> --json`
  - `ssh-cli sftp stat <NAME> <REMOTE_PATH> --json`
  - `ssh-cli sftp rename <NAME> <FROM> <TO> --json`
  - `ssh-cli sftp upload --all <LOCAL_FILE> <REMOTE_FILE> --json`
  - `ssh-cli sftp upload --hosts <A>,<B> <LOCAL_FILE> <REMOTE_FILE> --json`
- Tunnel modes, then MUST WAIT for `tunnel_listening`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json`
  - `ssh-cli tunnel <NAME> 0 <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json`, then read the ephemeral `local_port`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --bind 127.0.0.1`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> --socks5 --timeout-ms <MS> --json`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> --remote-socket /var/run/docker.sock --timeout-ms <MS> --json`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> 127.0.0.1 <REMOTE_PORT> --reverse --timeout-ms <MS> --json`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> 127.0.0.1 0 --reverse --timeout-ms <MS> --json`, then read the server-allocated `local_port`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> 0.0.0.0 <REMOTE_PORT> --reverse --i-accept-network-exposure --timeout-ms <MS> --json`
  - Append `--key <KEY_PATH>`, `--use-agent --agent-socket <SOCK>`, `--password-stdin` or `--key-passphrase-stdin` to any tunnel formula
- Preview, health and secrets
  - `ssh-cli --json --dry-run vps remove <NAME>`
  - `ssh-cli --json --dry-run vps import --file <PATH>`
  - `ssh-cli --json --dry-run sftp rm <NAME> <REMOTE_FILE>`
  - `ssh-cli --json --dry-run sftp rmdir <NAME> <REMOTE_DIR>`
  - `ssh-cli --json --dry-run secrets init --force`
  - `ssh-cli --json --dry-run secrets reencrypt`
  - `ssh-cli health-check <NAME> --json`, and with `--timeout <MS>`
  - `ssh-cli health-check --json`
  - `ssh-cli health-check --all --json`, and `ssh-cli --max-concurrency <N> health-check --all --json`
  - `ssh-cli health-check --hosts <A>,<B> --json`
  - `ssh-cli health-check <NAME> --json --key <KEY_PATH>`
  - `ssh-cli health-check <NAME> --json --use-agent --agent-socket <SOCK>`
  - `printf '%s' "$PASS" | ssh-cli health-check <NAME> --json --password-stdin`
  - `printf '%s' "$KEY_PASS" | ssh-cli health-check <NAME> --json --key <KEY_PATH> --key-passphrase-stdin`
  - `ssh-cli health-check <NAME> --json --replace-host-key`
  - `ssh-cli secrets status --json`
  - `ssh-cli secrets init --json`, `ssh-cli secrets init --force --json`, `ssh-cli secrets init --keyring --json`
  - `ssh-cli secrets reencrypt --json`
  - `ssh-cli --secrets-key-file <KEY_FILE> secrets status --json`
  - `ssh-cli --use-keyring secrets status --json`
  - `ssh-cli --allow-plaintext-secrets --config-dir <DIR> secrets status --json`
  - `ssh-cli --config-dir <DIR> vps list --json`
  - `ssh-cli --replace-host-key exec <NAME> "true"`
  - `ssh-cli --no-input vps add --name <NAME> --host <HOST> --user <USER> --password-stdin`
- Payload shaping
  - `ssh-cli --select name,host,user vps list --json`
  - `ssh-cli --filter user=root --limit 5 vps list --json`
  - `ssh-cli --sort name --dedupe-by host vps list --json`
  - `ssh-cli --count-only vps list --json`
  - `ssh-cli --truncate-content 500 --max-output-bytes 65536 exec --all "<CMD>" --json`
- TLS
  - `ssh-cli tls provider --json`
  - `ssh-cli tls paths --json`
  - `ssh-cli tls mtls list --json`
  - `ssh-cli tls mtls import --name <NAME> --cert <CERT_PEM> --key <KEY_PEM> --json`
  - `ssh-cli tls mtls show <NAME> --json`
  - `ssh-cli tls mtls remove <NAME> --json`
  - `ssh-cli tls acme account create --contact mailto:<EMAIL> --json`
  - `ssh-cli tls acme account create --contact mailto:<EMAIL> --staging --force --json`
  - `ssh-cli tls acme account show --json`
  - `ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --json`
  - `ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --staging --json`
  - `ssh-cli tls acme complete --domain <DOMAIN> --json`
  - `ssh-cli tls acme status --json`, and `ssh-cli tls acme status --domain <DOMAIN> --json`
  - `ssh-cli tls acme list --json`

### FORBIDDEN
- MUST NEVER pipe `--include-secrets` without `--output` or `--i-understand-secrets-on-stdout`
- MUST NEVER invent `--local-port`; the local port is POSITIONAL
