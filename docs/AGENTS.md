# Agents Guide for ssh-cli


> Cut RAM waste from resident processes and keep multi-host SSH under agent control.

- Read this document in [Portuguese (pt-BR)](AGENTS.pt-BR.md)
- Pair with [../INTEGRATIONS.md](../INTEGRATIONS.md) and [../skills/ssh-cli-en/SKILL.md](../skills/ssh-cli-en/SKILL.md)
- Product line: 0.5.6
- This guide is a hybrid: a persuasive opening that sells the model, then an imperative contract from the section Agent Contract onward
- Discover the live surface with `ssh-cli commands`, `ssh-cli schema` and `ssh-cli doctor` (root alias of `vps doctor`)
- `ssh-cli --help` prints the exit codes under `Exit status:` and copyable `Examples:` at no network cost, and `--help` of all 48 leaves adds at least one example while `-h` stays short


## Why

### Replace long-lived Node SSH processes with a die-after-run binary
- Persistent long-lived SSH processes burn RAM while holding sockets idle
- One host per daemon process multiplies process count for N servers
- A single Rust command-line interface (CLI) binary with XDG Base Directory (XDG) multi-host storage collapses that sprawl
- Agents gain deterministic JSON and sysexits without extra protocol overhead


## Economy

### Measure the operational win
- Every call starts, answers and exits, so no process stays resident between calls, and `ssh-cli --version` checks the install without any network call
- Process memory returns to the OS after every command
- There is no Node runtime tax and no permanent SSH manager process
- One install serves Claude Code, Cursor, Windsurf, Codex and shell agents


## Sovereignty

### Keep credentials and host trust local
- Hosts live under XDG without `.env` sprawl
- Prefer private keys and stdin secrets over chat-pasted passwords
- Secrets are encrypted at rest with ChaCha20-Poly1305 once the primary-key exists, manage it with `secrets status`, `secrets init` and `secrets reencrypt`
- Trust on first use (TOFU) `known_hosts` makes a silent man-in-the-middle harder
- Elevation can be switched off whenever a workflow must stay unprivileged
- The config directory is born 0700 and the host-name charset blocks terminal tricks and shell metacharacters
- FORBIDDEN: log the primary-key, host passwords or decrypted secrets


## Superpowers for AI Agents
- One call, one JSON object, one exit code: no screen scraping, no prose to guess
- Proof of where it ran: `target_resolved` and `target_source` come back on every single-host call
- Fleet in one process: `--all`, `--hosts` and `--tags` run bounded concurrent sessions
- Rehearsal before risk: `--dry-run` previews eight operations and echoes the target
- Token thrift: eight shaping flags cut the payload before it is serialized
- Safe change: switch authentication, rename and clear fields in one call, with `updated` as the receipt
- Self-describing: `commands --json` publishes every argument and `schema` publishes every output shape


## Compatible Agents and Orchestrators
- Claude Code with the shipped skill package
- Cursor with shell or agent tools
- Windsurf shell tool
- Codex CLI shell tool
- OpenCode shell tool
- Aider, Continue, Gemini CLI, OpenHands and generic bash or zsh


## Command inventory (full tree)
- All 48 leaves are written out in full, not in brace notation
- An agent that greps for `tls acme account create` must find that exact string
- Discover the same tree at runtime with `ssh-cli commands` (`event: "commands"`)

| Surface | Commands |
| --- | --- |
| `vps` | `ssh-cli vps add` `ssh-cli vps list` `ssh-cli vps remove` `ssh-cli vps rename` `ssh-cli vps edit` `ssh-cli vps show` `ssh-cli vps path` `ssh-cli vps doctor` `ssh-cli vps export` `ssh-cli vps import` |
| Session | `ssh-cli connect` |
| Exec | `ssh-cli exec` `ssh-cli sudo-exec` `ssh-cli su-exec` |
| `scp` | `ssh-cli scp upload` `ssh-cli scp download` (regular files only) |
| `sftp` | `ssh-cli sftp upload` `ssh-cli sftp download` `ssh-cli sftp ls` `ssh-cli sftp mkdir` `ssh-cli sftp rmdir` `ssh-cli sftp rm` `ssh-cli sftp stat` `ssh-cli sftp rename` |
| Network | `ssh-cli tunnel` (four modes: local, `--reverse`, `--socks5`, `--remote-socket`) `ssh-cli health-check` |
| `secrets` | `ssh-cli secrets status` `ssh-cli secrets init` `ssh-cli secrets reencrypt` |
| Discovery | `ssh-cli completions` `ssh-cli commands` `ssh-cli schema` `ssh-cli doctor` (root alias of `ssh-cli vps doctor`) |
| `locale` | `ssh-cli locale show` `ssh-cli locale set` `ssh-cli locale clear` |
| `tls` | `ssh-cli tls provider` `ssh-cli tls paths` |
| `tls mtls` | `ssh-cli tls mtls list` `ssh-cli tls mtls import` `ssh-cli tls mtls show` `ssh-cli tls mtls remove` |
| `tls acme` | `ssh-cli tls acme account create` `ssh-cli tls acme account show` `ssh-cli tls acme issue` `ssh-cli tls acme complete` `ssh-cli tls acme status` `ssh-cli tls acme list` |

- Every flag of every leaf is tabled in [HOW_TO_USE.md](HOW_TO_USE.md), and this page names the ones an agent reaches for by task
- Registration and edit share the record flags: `--host`, `--port`, `--user`, `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--sudo-password`, `--su-password`, `--tag`, `--timeout`, `--max-chars`, `--max-command-chars`, `--max-output-chars`
- Transport Layer Security (TLS) on a host record is set with `--tls`, `--tls-sni`, `--tls-client-cert` and `--tls-client-key`, and `vps edit --no-tls` turns it back off
- An agent endpoint is set with `--use-agent` and `--agent-socket`, and cleared with `--no-agent-socket`
- `vps add --check` runs one `health-check` right after registering, so a broken record is caught at write time
- The clear flags on `vps edit` are `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-key`, `--no-tls-sni`, `--no-tls-client-cert` and `--no-tls-client-key`
- `vps import --file <PATH>` reads a TOML file or a `vps-export` envelope, and `--allow-incomplete` accepts records missing optional fields
- A mutual TLS (mTLS) identity is imported with `tls mtls import --cert CERT --key KEY --name NAME`
- An Automatic Certificate Management Environment (ACME) account is created with `tls acme account create --contact mailto:ops@example.com`, and `--staging` points it at the staging directory
- `tls acme issue --domain NAME --print-challenge` starts DNS-01 and prints the challenge, and `--staging` keeps the order off production
- The OS store is selected by `--use-keyring` or by `secrets init --keyring`, and both resolve the same profile-scoped entry
- That store is Keychain on macOS, Credential Manager on Windows and Secret Service over zbus on Linux, and a store that does not answer a write exits 69 instead of a false success
- The primary key resolves in this order: `--secrets-key-file`, the OS store when `--use-keyring` is set, `secrets.key` in the config directory, then an entry left by an earlier `secrets init --keyring`
- A read under `--use-keyring` whose store does not answer logs a warning and falls back to `secrets.key`, so only the write path exits 69
- A profile owns one primary key, so `secrets init` exits 64 when a key already exists in either store, and `--force` rotates it and retires the other source
- `doctor --probe-ssh` proves the SSH dial, and `--hosts <LIST>` limits either `doctor` or `health-check` to that subset
- `--description` appends a shell comment to the packed remote command, so the audit trail lives in the command itself
- `--verbose` is the long spelling of `-v`, and the level is repeatable up to `-vvv`

### Global flags of note
- `--lang`, `-v`, `-vv`, `-vvv` (G14 graduated, G2 crate-scoped `warn,ssh_cli=*`), `-q`, `--config-dir`, `--no-color`, `--output-format`, `--json`
- `--disable-sudo`, `--replace-host-key`
- `--disable-sudo` has two meanings: on `sudo-exec` and `su-exec` it blocks elevation for that one call, and on `vps add` and `vps edit` it persists `disable_sudo: true` in the record, `vps edit --enable-sudo` undoes it
- The persisting meaning follows the subcommand and not the position of the flag, so `ssh-cli --disable-sudo vps edit <NAME> --port 22` also saves `disable_sudo` and lists it in `updated`
- The alias `--disableSudo` is accepted before the subcommand and after every subcommand except `vps add` and `vps edit`, which have their own `--disable-sudo` and refuse the alias with exit 2 (checked with `--help` on all 48 leaves)
- `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- `--timeout`, `--max-concurrency`, `--fail-fast`, `--scp-file-concurrency`, `--lock-wait-ms`
- Write global flags before the subcommand, the binary also accepts them after it

### Payload shaping (0.5.4): spend no tokens on data you will discard
- Eight global flags shape the response before serialization, so the oversized envelope is never built
- `--select <PATHS>` (alias `--fields`) keeps only these dotted paths in each record, or in the single object
- `--filter key=value`, `key!=value` or `key~substring` keeps matching records, repeatable and combined with AND
- A malformed predicate is rejected at parse time with exit 64, so a typo is never mistaken for an empty result
- `--limit N` emits at most N records and is distinct from per-command query limits
- `--sort <PATH>` sorts ascending by dotted path, comparing numbers numerically
- `--dedupe-by <PATH>` drops later records that repeat the path value
- `--count-only` replaces the record collection with `{"count": N}`, counted after all filtering
- `--truncate-content <CHARS>` shortens long strings by characters, never bytes, so UTF-8 stays valid
- `--max-output-bytes <BYTES>` caps the envelope by dropping trailing records, never by slicing the JSON text
- Prefer these flags over piping stdout through an external JSON tool, because piping pays the full token cost first
- The bare array of `vps list` has no room for `agent_shape`, so its reduction report goes to stderr as one JSON line with `"event":"agent-shape"` whenever something was dropped or truncated, and `--quiet` suppresses it
- On a collection the shape reports itself in `agent_shape` with `input_count`, `output_count` and `dropped`
- Every flag reaches record collections: the array of `vps list`, `results` of the batch envelopes and `entries` of `sftp ls`
- Only `--select` and `--truncate-content` reduce a single object such as `vps show`, `doctor`, `commands` or `exec` on one host
- On a single object `agent_shape` carries `selected`, `content_truncated` and `ignored`, and `ignored` names each collection flag that had nothing to reduce
- A `vps list --select` item is the reduced shape that `ssh-cli schema vps-list` publishes beside the full record, so it validates against the published schema
- `--select` names fields inside each record, so `--select results` returns empty objects and the right call is `--select name,stdout`
- The health-check batch records carry `name`, `status` (`ok` or `error`), `latency_ms` and `error`, so filter with `--filter status!=ok`

### Editing and renaming a host (0.5.6)
- `vps rename <NAME> <NEW_NAME>` is the native rename and `vps edit <NAME> --name <NEW>` reaches the same path
- Both verbs take the subject from argv, publish `target_resolved` and `target_source: argv`, and never read the `active` marker as the subject
- Stored secrets stay readable because the record seals them without associated data, so the new name changes nothing about them
- Every save re-seals with a fresh nonce, which is normal and never touches the plaintext
- The export, edit, import round trip is no longer needed for a rename, and it silently dropped agent auth because the export carried no `use_agent` or `agent_socket`
- The `vps-renamed` envelope carries `name`, `previous_name`, `active_rewritten` (true only when the marker pointed at the old name), `updated`, `target_resolved` and `target_source`
- `vps edit --name` emits `vps-renamed` too, `vps-edited` is emitted only when nothing is renamed and never carries `previous_name`
- `updated` lists the fields whose stored value changed, in application order, each at most once, and it is always present
- It is empty when every requested value already matched or when the call only renamed the host
- An edit that changes nothing is a no-op: exit 0, `vps-edited`, `updated: []` and `config.toml` is not rewritten
- Exit 64 is reserved for a call with no field, rename or clear flag at all
- `vps rename` is NOT idempotent: repeating it after a success exits 66 because the old name no longer exists, and `vps rename ghost ghost` exits 66 too
- A taken destination exits 64 with `vps_duplicate`, an identical name exits 64 with `new name ... is identical to the current name` and an invalid one exits 64 too
- Confirm the result with `vps show <NEW_NAME>` or `vps list` instead of repeating the call
- Clear flags remove a field: `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key`, `--no-key` and `--no-agent-socket`
- `--no-key` also clears the passphrase, so a secret nothing reads never stays behind
- `--no-key` on a host whose only authentication is that key exits 64 with `error_code` `domain_validation` and a message prefixed `vps_auth:`
- `--no-agent-socket` exits 64 when the host would be left without an agent endpoint on a platform that has no default one
- The config lock waits a bounded time (`--lock-wait-ms`, default 5000, ceiling 300000) and then exits 69 with `unavailable`, which is retryable
- `vps import` validates TLS like `vps edit` (a client certificate without its key exits 64) and its `vps-import` envelope carries `replaced`, the sorted names of hosts it overwrote, always present
- Every flag is discoverable: `ssh-cli commands --json` publishes each argument with `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` and `hidden`

### Refusal and rehearsal flags
- `--no-input` refuses to read stdin and fails fast instead of blocking on an absent human
- `--no-input` also forbids secrets on stdin (`--password-stdin` and the other `--*-stdin` flags exit 64), so use it on runs that pass no secret through stdin
- `--dry-run` prints the plan for a destructive operation and exits without executing it
- `--dry-run` is accepted only by `vps remove`, `vps import`, `vps edit`, `vps rename`, `sftp rm`, `sftp rmdir`, `secrets init` and `secrets reencrypt`
- The two `sftp` rehearsals never open a connection
- `vps import --dry-run` lists `hosts` with `name`, `incomplete` and `replaces_existing` plus `imported`, a prediction, while the real run reports `replaced`
- `secrets init --dry-run` reports `hosts_to_reencrypt`, so read it before `--force`, and `secrets reencrypt --dry-run` reports `hosts`
- For `vps remove`, `vps edit` and `vps rename` the plan also carries `target_resolved`, `target_source` (`argv`), `host_resolved` and `host_source` once the lookup succeeded
- A missing host exits 66 in the real run and in the rehearsal, and creates no directory and no lock file
- Anywhere else `--dry-run` is rejected with exit 64 instead of accepted and ignored, so a rehearsal is never mistaken for a no-op that already ran


## Agent Contract

### Required: process and streams
- Invoke `ssh-cli` as a subprocess and wait for the exit (one-shot)
- Read success from stdout and failure from the error envelope on stderr
- Exception: when the remote command itself fails, stdout keeps the `exec` object with the remote `exit_code`, stderr gets a `command_failed` envelope with `remote_exit_code`, and the CLI exits 1

### Required: output and secrets input
- Parse stdout JSON when `--json` or `--output-format json` is set, JSON is automatic when stdout is not a TTY
- Parse exactly one JSON object on a success path (G8), the exceptions are `exec --step`, which prints one object per step with `step` and `command`, and `tunnel --json`, which prints `tunnel_listening` then `tunnel_closed`
- Treat stderr tracing as logs, never as the contract
- Feed secrets through `--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin` and `--key-passphrase-stdin`

### Forbidden: secrets on argv and with no-input
- Pass a secret on argv, the CLI warns on stderr when a password-like value appears there
- Combine `--no-input` with a `--*-stdin` secret, that call exits 64
- `completions` prints a shell script and `vps export` in text mode prints a TOML body, every other success is JSON

### JSON, errors and exit codes
- REQUIRED: when JSON errors mode is active, parse the failure envelope on stderr, it has seven fields: `exit_code`, `error_code`, `message`, `remote_exit_code`, `error_class`, `retryable` and `suggestion`
- Five are always present, `remote_exit_code` and `suggestion` are optional, and the target echo comes beside them, see `docs/schemas/error-envelope.schema.json`
- `error_class` is `transient`, `permanent`, `cancelled` or `partial`, and `partial` marks a multi-host fan-out where some targets succeeded
- A clap usage error (exit 2) and a malformed `--filter` (exit 64, a failure before dispatch) print plain text on stderr even with `--json`, and that text follows the host locale, so pass `--lang en` to read it in English, only execution errors carry the envelope, so do not feed every stderr to a JSON parser
- REQUIRED: branch on `error_code`, the only stable discriminator, then on `error_class` and `retryable`, never on `message`
- A partial multi-host failure still prints the batch envelope on stdout, then stderr gets `error_code` `partial_failure` with `error_class` `partial`, and the exit is 1
- REQUIRED: expect the default tracing level error and use `-v`, `-vv` or `-vvv` only to debug
- FORBIDDEN: rely on ambient `RUST_LOG`, it is ignored

### Idempotency

| Call | Repeat behavior |
| --- | --- |
| `vps add` | Exit 64 `vps_duplicate`, the host exists |
| `vps edit` | Idempotent: exit 0 with `updated: []`, no rewrite |
| `vps rename` | Not idempotent: exit 66 because the old name is gone |
| `vps remove` | Not idempotent: exit 66 `vps_not_found` |
| `vps import` | Idempotent: exit 0 and `replaced` lists the overwritten names |
| `connect` | Idempotent: exit 0 |
| `locale set` and `locale clear` | Idempotent: exit 0 |
| `secrets init` | Exit 64 unless `--force`, the key exists |
| `secrets reencrypt` | Safe to repeat, it re-ciphers under the current key |
| `exec`, `sudo-exec`, `su-exec` | Idempotent only as far as the remote command is, the binary never retries by itself |
| `scp` and `sftp` transfers | Overwrite the destination, verify the checksum afterwards |

### Payload and execution limits
- `max_command_chars` (default 1000) rejects an oversized command with exit 64 `command_too_long` before any SSH work
- `max_output_chars` (default 100000) cuts remote output and sets `truncated_stdout` or `truncated_stderr`
- `--max-output-bytes` caps the envelope and `--truncate-content` caps each string
- `--max-concurrency` accepts 1..=64 and exits 2 outside that range, the automatic value follows CPUs and free RAM
- `--fail-fast` stops admitting new fleet units after the first failure, units already running finish, and each host never started comes back with an `error` that starts with `not attempted`: `not attempted (fail-fast stopped admission)` for `exec` and `not attempted (fan-out admission stopped)` for `scp` and `sftp`
- `--scp-file-concurrency` sets how many Secure Copy Protocol (SCP) channels run in parallel on one session, and the default is 1
- `--timeout` is in milliseconds and the default is 60000
- Only `vps add --timeout` warns on a value under 1000, while `vps edit`, `exec` and `health-check` accept it silently
- `--lock-wait-ms` bounds the config lock, accepts 1..=300000 (exit 2 outside), default 5000
- A `tunnel` always needs `--timeout-ms`, and omitting it is exit 2

### Language with --lang
- `--lang en` or `--lang pt-BR` sets the human text language for one call, `locale set` persists it
- Branch on `error_code`: B2 made the human error line localizable, so `--lang pt-BR` renders it in Portuguese in text mode
- The JSON envelope `message` stays stable English by contract, so parsing never depends on the host locale
- The seam lives in `emit_resolved_ssh_error`: the `--json` branch keeps the English `Display` and the human branch consults `i18n::localized_error_text`
- Error codes without a translation fall back to the English `Display`, the localization is fail-open and never blanks an error line


## Agent Integration Details

### Required: registering and switching hosts
- Register hosts with `vps add` before repeated remote work, with a password, a key or `--use-agent` and `--agent-socket`
- Switch a host between password, key and agent with one `vps edit` call: `--key <PATH>`, `--password-stdin` or `--use-agent`
- Each clears the other methods and `updated` lists what changed
- A different `--key` drops the old passphrase unless the call brings a new one, and `--agent-socket` alone implies `--use-agent`
- FORBIDDEN: two primary auth methods in one `vps edit` call (clap exits 2) or a passphrase with no key (exit 64, record untouched)
- A `--key` path that does not exist exits 66 with `error_code` `file_not_found`
- REQUIRED: read each `updated` entry as one of `host`, `port`, `user`, `password`, `key_path`, `key_passphrase`, `use_agent`, `agent_socket`, `timeout_ms`, `max_command_chars`, `max_output_chars`, `sudo_password`, `su_password`, `disable_sudo`, `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key` and `tags`
- Those 19 names are also the enum in `docs/schemas/vps-edit.schema.json`
- REQUIRED: name hosts with ASCII letters, digits, dot, hyphen and underscore, starting with a letter or digit, 1 to 255 bytes
- `vps add`, `vps rename`, `vps edit --name` and `vps import` refuse other names with exit 64
- `vps import` validates every item first and lists every invalid name at once, writing nothing, also under `--dry-run`
- An older `config.toml` with other names still loads, even with control characters
- `doctor` lists them in `local.invalid_host_names` and `vps rename '<old name>' <new name>` repairs them
- A name with a control character is refused as a target by `exec`, `health-check`, `--hosts` and `connect` with exit 64, and `connect` writes no marker
- `tls mtls import --name` refuses control, formatting and bidi characters with exit 64 and creates no directory

### Required: credentials, commands and SCP scope
- Supply a password or a key, empty credentials are rejected at write time
- Treat an empty password in list and show JSON as `null` (key-only hosts) and a non-empty one as masked with three asterisks
- An empty remote command fails with the technical message `empty command` (always English) and exit 64
- Pass `--timeout-ms` for every `tunnel` invocation
- Treat `scp` as regular files only, no directories and no `-r`
- For trees and remote filesystem operations use `sftp` (`upload` and `download` with `--recursive`, `ls`, `mkdir`, `rm`, `stat`, `rename`)
- REQUIRED: use product line 0.5.3 as the floor for SSH File Transfer Protocol (SFTP) (the current line is 0.5.6), G1 fixed upload truncation to zero bytes, and never depend on crates.io 0.3.9 for SCP because that wire was broken (a stale release, kept here as history)
- Verify transfers with a destination `sha256` (G15), not with client-reported sizes alone

### Required: SCP results and export format
- Parse an SCP success with `docs/schemas/scp-transfer.schema.json` (`ok`, `event` `scp-transfer`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`)
- A missing SCP local or remote file exits 66 with `file not found: <path>`, canonical path and no stacked `SCP:` prefix
- Read `mtime_preserved` and `durable` on the `scp-transfer` event, preserving mtime and mode is best effort (G-SCP-R01/R02)
- `vps export` body follows the resolved output format, so a non-TTY agent gets the JSON envelope `vps-export` even when `-o` names a `.toml` file
- A TOML body requires `--output-format text`
- With `-o`, stdout carries only the summary `ok`, `event`, `format`, `include_secrets` and `path`, and `docs/schemas/vps-export.schema.json` accepts both shapes

### Required: export and import of secrets
- A redacted `vps export` clears every secret: `password` is an empty string and `key_passphrase`, `sudo_password` and `su_password` are `null`, never `sshcli-enc:` ciphertext of empty (EXP-001)
- `--include-secrets` requires `-o` or `--output`, or `--i-understand-secrets-on-stdout`
- `vps import` accepts TOML (English keys and legacy Portuguese load aliases) or a JSON `vps-export`, and `--allow-incomplete` accepts redacted or skeleton hosts
- The user key is `username` in TOML (legacy `usuario`) and `user` in JSON

### Required: wire format and secrets flags
- `added_at` is optional on import (serde defaults to now) and the wire format is schema v3, which serializes English keys
- Prefer the secrets CLI flags `--allow-plaintext-secrets`, `--secrets-key-file` and `--use-keyring` over environment variables, because `SSH_CLI_SECRETS_KEY` and `SSH_CLI_SECRETS_KEY_FILE` make every command that reads or writes registry secrets exit 64 fail-closed
- Measured with `SSH_CLI_SECRETS_KEY` set on a registry that holds a secret: `vps list`, `vps show`, `vps add`, `vps rename`, `vps remove`, `vps export`, `doctor`, `secrets status`, `secrets reencrypt` and `connect` exit 64, while `vps path`, `locale show`, `schema`, `commands` and `tls provider` never touch secrets and exit 0
- `secrets init --json` and `secrets reencrypt --json` emit `secrets-init` and `secrets-reencrypt` (`docs/schemas/secrets-init.schema.json`, `docs/schemas/secrets-reencrypt.schema.json`)
- A first secret write may set `secrets_key_auto_created: true` on the same success JSON, one document
- The schema catalog is in [docs/schemas/README.md](schemas/README.md)
- REQUIRED: on `tunnel --json`, wait for the stdout object with `event: "tunnel_listening"` (`docs/schemas/tunnel-listening.schema.json`) before using the local port
- The process stays alive until the timeout or a signal, the post-bind deadline ends with exit 0 (TUN-002), a SIGINT or SIGTERM prints `tunnel_closed` with `reason` `signal` and exits 130 or 143, and a pre-bind timeout remains 74
- REQUIRED: the tunnel `--bind` defaults to `127.0.0.1` (loopback)

### Allowed: auth and timeout overrides
- `tunnel` and `health-check` may use `--password-stdin`, `--key`, `--key-passphrase` and `--key-passphrase-stdin` (CLI-005/006 parity with exec and scp)
- `health-check --timeout <ms>` when the host default is too long or too short
- REQUIRED: prefer multi-host fan-out for fleet work: `exec`, `sudo-exec`, `su-exec`, `scp`, `sftp` and `health-check` with `--all` or `--hosts a,b,c` run bounded concurrent sessions (`Semaphore` and `JoinSet`), not one process per host
- Batch JSON applies to both multi modes, even when `--hosts` lists one name
- REQUIRED: a third selector exists on the exec family only: `exec`, `sudo-exec` and `su-exec` with `--tags t1,t2` address every host carrying any of those tags (`vps add --tag`)
- `--all`, `--hosts` and `--tags` are mutually exclusive and clap rejects any pair
- `scp`, `sftp` and `health-check` accept `--all` and `--hosts` but not `--tags`
- REQUIRED: parse multi-host JSON with `health-check-batch`, `exec-batch`, `scp-batch` and `sftp-batch` (`docs/schemas/*-batch.schema.json`), every envelope carries `batch_run_id` and `max_concurrency`
- `scp-batch` and `sftp-batch` also carry `target_source` `selector` and its alias `host_source`, and `exec-batch` does not
- ALLOWED: cap fan-out with the global `--max-concurrency N` (1..=64), the same gate limits tunnel accept forwards
- FORBIDDEN: assume sequential multi-host by default when `--all` is available, wall-clock is dominated by the SSH round-trip time (RTT) and concurrent sessions are the modus operandi

### Required: cancel, warnings and install
- Multi-file SCP and SFTP cancel fills the cancelled remainder so `results.len() == input.len()` (G5/G17)
- A `vps add --timeout` under 1000 and a password-like value on argv emit a warning on stderr, do not parse those lines as a JSON error envelope
- Install with `cargo install ssh-cli --locked`

### Forbidden: unsafe assumptions
- Assume a long-lived SSH connection across process runs
- Reintroduce long-lived daemon packaging into this repository
- Enable or emit product telemetry
- Retry blindly on exit 64, 65, 66 or 77
- Print or store primary-key material from `secrets` commands
- Treat SCP directory trees or a recursive `-r` as supported
- Install or call OpenSSH client binaries on the agent host for `ssh-cli`, because it speaks SSH in-process through `russh` and spawns no local `ssh`, `scp` or `ssh-keygen`
- REQUIRED: treat remote command strings as hostile input, NUL bytes are rejected with an invalid-argument error before the SSH channel exec (G-PROC-03)
- `sftp ls` and `sftp stat` escape `\n` and `\t` of a remote name in text mode, so a forged name cannot add lines

### Required: hostile input and SFTP integrity
- SFTP SETSTAT sends `atime` and `mtime` together (G3) and `set_metadata` is fail-closed (G4)
- Permission bits are masked by direction, upload uses `SFTP_PERM_MASK` (`0o7777`) and keeps setuid, setgid and sticky on a file you already own (G12/G19), download uses `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) so server-sent elevation bits never land locally (A3)
- SFTP download local `set_permissions` failures are errors, never silent (G18)
- The SCP client wire path (`client_real_scp.rs`) uses English identifiers and English channel errors (G16)


## CRUD and JSON Contract

### Machine-readable operations
- List hosts: `ssh-cli vps list --json` returns an array of masked host objects
- Show a host: `ssh-cli vps show <name> --json` returns one masked host object
- Discovery: `ssh-cli commands`, `ssh-cli schema [NAME]` and `ssh-cli doctor`
- Secrets: `ssh-cli secrets status --json` returns the encryption mode without key material
- `secrets init --json` emits `event: "secrets-init"` and `secrets reencrypt --json` emits `event: "secrets-reencrypt"`
- CRUD success events when JSON is effective: `vps-added`, `vps-edited` (or `vps-renamed` when `vps edit --name` renames), `vps-removed`, `vps-connected` and `vps-import`, with `secrets_key_auto_created` on the same document when a key is auto-created
- Exec family on one host: `exec`, `sudo-exec` and `su-exec` with `--json` return `target_resolved`, `target_source`, their 0.5.5 read aliases `host_resolved` and `host_source`, `active_fallback`, `stdout`, `stderr`, `exit_code`, the truncation flags and `duration_ms` as one object (G8)
- REQUIRED: designate the target host in argv on every surface, including `health-check`
- A lone positional is a usage error (exit 64)
- The marker written by `connect` is reachable only through `--use-active` (GAP-SSH-EXEC-ARGC-001)
- The split is stable across the CLI: 64 means no target designated and 66 means the designated target does not exist
- Read `target_source` (`argv`, `active_marker` or `selector`), or its alias `host_source`, before trusting where a mutation landed
- REQUIRED: read `target_resolved`, `target_source` and `active_fallback` on the error envelope too
- They are present whenever the failure happened after the host was resolved and absent, never empty, when it did not, `vps_not_found` is the absent case
- `host_resolved` and `host_source` ship beside them as 0.5.5 read aliases, written from the same value, so bind new consumers to the canonical `target_*` spelling
- REQUIRED: `exec --step` aborts the batch when step zero exits 127, that code on the first step is the signature of a host name read as a command
- A 127 at any later index still runs the remaining steps
- Fleet exec: `ssh-cli exec --all '<cmd>' --json`, `sudo-exec --all`, `su-exec --all` or `--hosts a,b '<cmd>'` emit `event: "exec-batch"` (`exec-batch.schema.json`), and per-host partial failure does not abort the siblings
- Tunnel is single host only, one bind and one session per one-shot, multi-host tunnels are N invocations with distinct ports or `--bind`
- Doctor: `ssh-cli vps doctor [--json]` emits a single root `event: vps-doctor` with `local` and `ssh_probe: null`, never two JSON roots
- Add `--probe-ssh` for a multi-host health fan-out embedded in `ssh_probe`, with an optional `--hosts a,b` subset
- When a probed host fails, stdout still holds the single `vps-doctor` document, stderr gets a `partial_failure` envelope and the exit is 1
- SCP multi-file on one host: `ssh-cli scp upload <VPS> f1 f2 … <REMOTE_DIR>` and `download <VPS> r1 r2 … <LOCAL_DIR>` use one SSH session and serial transfers (G-PAR-47)
- SCP multi-host and multi-file: `ssh-cli scp upload --all --src f1 --src f2 --dest <REMOTE_DIR>` or `--hosts a,b`, files serial on each session (G-PAR-48)
- A one-file fleet download writes `<local>.<vps>` and a multi-file fleet download writes under `<LOCAL_DIR>/<host>/`
- REQUIRED: with `--all` or `--hosts`, more than one source travels through the named slots `--src` (repeatable) and `--dest`
- A selector plus three or more positionals is exit 64, because a non-positional flag may not remap a positional
- `--dest` is always a directory, whatever the source count, and the named slots also work on a single host
- Health on one host: `ssh-cli health-check <vps> [--timeout <ms>] --json` returns `name`, `status` and `latency_ms` plus the target echo
- Health on a fleet: `--all` or `--hosts a,b` emits `event: "health-check-batch"` (`health-check-batch.schema.json`)
- SCP on one host: `ssh-cli scp upload|download <vps> <local> <remote> --json` returns `scp-transfer` on stdout and failures use the error envelope on stderr
- SCP fleet and multi-file batches emit `event: "scp-batch"`, a multi-host and multi-file result `name` may be `host:path`
- SCP operational facts: use 0.5.3 or newer, the upload streams 32 KiB chunks, the download writes `{path}.ssh-cli.partial` and then renames, and a `sync_data` failure propagates before the rename (G9)
- SFTP: `ssh-cli sftp upload`, `ssh-cli sftp download`, `ssh-cli sftp ls`, `ssh-cli sftp mkdir`, `ssh-cli sftp rmdir`, `ssh-cli sftp rm`, `ssh-cli sftp stat` and `ssh-cli sftp rename` with the schemas `sftp-transfer`, `sftp-list`, `sftp-fs-op` and `sftp-batch`
- Locale: `ssh-cli locale show`, `ssh-cli locale set` and `ssh-cli locale clear`, with the one-shot `--lang`
- TLS: `ssh-cli tls provider`, `ssh-cli tls paths`, `ssh-cli tls mtls list`, `ssh-cli tls mtls import`, `ssh-cli tls mtls show`, `ssh-cli tls mtls remove`, `ssh-cli tls acme account create`, `ssh-cli tls acme account show`, `ssh-cli tls acme issue`, `ssh-cli tls acme complete`, `ssh-cli tls acme status` and `ssh-cli tls acme list`
- REQUIRED: the ACME commands take the domain through `--domain`, a bare positional exits 2
- Tunnel: `ssh-cli tunnel <vps> <local_port> [remote_host] [remote_port] --timeout-ms <ms> [--bind 127.0.0.1] --json` emits `tunnel_listening` on stdout after the bind and `tunnel_closed` on shutdown
- Tunnel modes (0.5.4): `--socks5` serves an RFC 1928 no-auth `CONNECT` proxy, `--remote-socket <PATH>` forwards to a remote Unix socket and `--reverse` asks the server to listen and deliver back to `<local_port>`
- The three are mutually exclusive
- `remote_host` and `remote_port` are omitted under `--socks5` and `--remote-socket` (passing them is exit 64) and mean the server's bind under `--reverse`, where `remote_port 0` lets the server allocate the port
- Both tunnel events carry `mode` (`local`, `socks5`, `streamlocal` or `reverse`), read it before interpreting the sibling fields because under `--reverse` `local_port` is the port the server allocated, while `remote_host` and `remote_port` carry the local delivery host and port
- `--i-accept-network-exposure` guards the local `--bind` in the three local modes and the remote bind under `--reverse`, the exposed end in that direction
- The local bind is IP-parsed by clap (a typo exits 2) and the remote bind is compared as text because RFC 4254 admits names and the empty string (a typo exits 64 from the guard)
- FORBIDDEN: pass `--bind` together with `--reverse` and expect it to matter, the flag is accepted by clap and then discarded silently because reverse delivery is forced to loopback
- Address the server-side listener through the positional `<remote_host>`
- Export: the `ssh-cli vps export` body is the JSON envelope `event: "vps-export"` on any non-TTY stdout and `--output-format text` is the only route to TOML
- Import: `ssh-cli vps import --file <path> [--allow-incomplete]`, a host that looks like a redacted export exits 64 unless `--allow-incomplete` is passed
- A missing `--file` exits 66 with `error_code` `file_not_found`, `error_class` `permanent` and `retryable` false, in the real run and under `--dry-run`, so fix the path and never retry the same argv
- List and show print an empty password as JSON `null` and a non-empty secret masked with `FIXED_MASK` (three asterisks)
- Validate payloads against the schemas under `docs/schemas/`, the index is [docs/schemas/README.md](schemas/README.md)

### Doctor fields and the config directory
- REQUIRED: read `ssh-cli doctor --json` as one document with `event: "vps-doctor"`, `ok`, `local` and `ssh_probe`
- `local` carries `layer`, `config_path`, `exists`, `permissions`, `config_dir_exposed`, `config_dir_permissions`, `invalid_host_names`, `schema_version`, `hosts`, `known_hosts`, `active_file`, `active_vps`, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out`, `telemetry` and `runtime`
- `local.runtime` carries `os`, `arch`, `is_wsl`, `is_container`, `is_ci`, `is_termux` and `sandbox`

### Required: doctor fields
- Read `local.active_vps` (the host name, `null` when unset) beside `local.active_file` (its path)
- Read `local.config_dir_exposed` (boolean, always present), `local.invalid_host_names` (array of escaped names, always present, empty when every name is valid) and `local.config_dir_permissions` (four-digit octal such as `0700`, absent off Unix)
- Off Unix only `config_dir_permissions` is missing and `config_dir_exposed` reads `false`
- `permissions` still describes `config.toml` only
- Read them with `ssh-cli doctor --json --select local.config_dir_exposed`, because `--select` projects the `doctor` object directly and `--filter` and `--count-only` do not apply to one record
- A directory the CLI creates is born 0700 at every level, an existing default XDG directory is tightened to 0700 when it is not a symlink and belongs to the user
- Any other existing directory is never chmodded: `config_dir_exposed` is `true` and `config_dir_permissions` shows the mode
- `--config-dir something.toml` makes the parent directory the config directory for `secrets.key`, `known_hosts`, `active`, `lang` and the TLS files
- `known_hosts.lock` and `lang` are created 0600 and the export creates missing directories 0700
- REQUIRED: expect text mode to escape control, formatting and bidi characters as `\u{1b}`
- The escaped set is every `char::is_control` character plus U+00AD, U+061C, U+180E, U+200B to U+200F, U+202A to U+202E, U+2060 to U+2064, U+2066 to U+206F, U+FEFF and U+E0000 to U+E007F
- U+2065 is unassigned and stays out, and the same set is refused in host names and mTLS identity names
- JSON is unchanged and keeps those characters raw, and remote `exec` output stays raw
- A registry entry with a control character in a tag, host or user (fields validated since before) still exits 65, only the host NAME is tolerant
- Two config tables that normalize to the same name exit 65 with `config has two hosts named ...`


## Exit Code Routing
- Exit 0 means success
- Exit 1 means a general runtime failure or a partial multi-host failure, inspect stderr and `remote_exit_code`
- Exit 2 means a clap usage error, printed as plain text on stderr with no envelope, fix the flags
- Exit 64 means a usage or argument error (including an empty command, an invalid name or a refused `--dry-run`) or ACME permanent validation (`invalidContact`), fix argv and do not retry
- Exit 65 (`TomlDe`, JSON, schema) means a parse or data error, fix the input payload
- Exit 66 means a missing VPS or file (`file not found: <path>` on SCP and on `vps import --file`), register it or correct the name or path
- Exit 69 means a host service is not answering (a keyring write or the config lock), it is transient and retryable
- Exit 70 means an internal failure with no user-fixable input, retrying unchanged will not help
- Exit 73 means a config write failure, check permissions and disk
- Exit 74 means an IO or SSH connection failure, a network retry may help
- Exit 77 means an auth failure, a host-key policy, a disabled sudo or a stored secret that fails to decrypt, try `--key`, `--password-stdin` or a passphrase on stdin and do not blind-retry
- Exit 141 means a broken pipe: the reader closed stdout or stderr, the output is incomplete, the state must be rechecked and the call is never retried blindly
- Exit 130 and 143 mean signal termination (SIGINT and SIGTERM), and a `tunnel` prints `tunnel_closed` with `reason` `signal` first
- These codes match the `Exit status:` section of `ssh-cli --help`


## Retry Strategy
- Prefer the JSON error envelope fields `retryable` and `error_class` over exit-only heuristics (`docs/schemas/error-envelope.schema.json`)
- Retry at most twice on `retryable: true` or exit 74, with exponential full-jitter backoff (base 200 ms, cap 5 s, see `ssh_cli::retry::RetryConfig::agent_default`)
- Never retry on `retryable: false` or on exit 64, 65, 66, 77, 1 (remote command), 130, 143 or 141 without changing the inputs
- ACME permanent validation (`invalidContact` or 4xx) is exit 64, not exit 74, so do not treat it as retryable network IO
- After a lost response, look up the state instead of repeating: the 66 hint of `vps rename` and `vps remove` says to run `vps list` first
- The binary does not auto-retry non-idempotent `exec`, `scp` or `sftp` in-process (one-shot least privilege)
- The agent re-invokes the process
- Shorten or split commands when an exit signals a `max_command_chars` rejection
- Confirm host key changes with a human before `--replace-host-key`


## Declared Limits
- Remote `exec` stdout and stderr stay raw by design, text mode escaping does not reach them
- On a single record, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` and `--max-output-bytes` stay inert and are named under `agent_shape.ignored`, while `--select` and `--truncate-content` do reduce the record
- The name appears even when a collection flag is the only one passed, so `ssh-cli doctor --limit 2` answers with `agent_shape.ignored` instead of an unshaped body
- JSON keeps every character of the escaped set raw (for example U+202E and the zero-width space) because it is for machines
- Error bodies stay in English by design so agents receive stable text
- A legacy name raises no warning on load, `doctor` lists it
- macOS and Windows only pass a type check in the `cross-targets` gate, so mode 0700 and `getuid` were not executed there
- Windows has no chmod, so the config directory relies on the profile access control list (ACL)
- The OS keyring runs for real only on Linux in this repository, while Keychain and Credential Manager pass only the type check of the `cross-targets` gate


## Crate Integrations
- Publish consumers depend on the CLI contract, not on an unstable library API
- Pin library experiments to an exact crate version when linking `ssh_cli` as a lib
- Prefer PATH-installed binary integration for agents
