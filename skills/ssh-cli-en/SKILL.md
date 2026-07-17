---
name: ssh-cli
description: This skill MUST auto-activate when remote SSH VPS registry inventory XDG config exec sudo-exec su-exec scp tunnel health-check secrets or agent devops without TTY are implied even without naming ssh-cli. MUST cover vps CRUD emit_success events vps-added vps-edited vps-removed vps-connected vps-import, export TOML default even pipe non-TTY, vps export --json event vps-export, import dual TOML EN+PT JSON schema v3 dual-read --allow-incomplete, include-secrets guards --output/-o or --i-understand-secrets-on-stdout, secrets primary-key flags --allow-plaintext-secrets --secrets-key-file --use-keyring, events secrets-init secrets-reencrypt secrets-key-auto-created, doctor secrets_plaintext_opt_out bool, empty command exit 64, timeout ms warn under 1000, password argv warn prefer stdin, tunnel --bind default 127.0.0.1 --timeout-ms tunnel_listening, scp file-only exit 66, import TOML exit 65, auth exit 77, health-check, added_at optional import. NEVER telemetry daemon recursive scp secrets on stdout without guard.
---

# ssh-cli Agent Skill

## Mission
### REQUIRED
- MUST treat this skill as SUPREME LAW for every `ssh-cli` invocation
- MUST ALWAYS run `ssh-cli` as a one-shot subprocess birth-execute-die
- MUST wait for process exit before parsing stdout or stderr except for long-lived `tunnel` until timeout or signal
- MUST use stored hosts from `vps add` instead of ad-hoc chat secrets
- MUST pass `--json` when the agent needs structured success output
- MUST copy the ready formulas in this skill and only substitute placeholders
- MUST execute the Action Prompts and Ready Formula Sheet before improvising flags
- MUST prefer stdin secret flags over argv secret flags

### FORBIDDEN
- MUST NEVER keep a long-lived SSH session across process runs except the active bound `tunnel` until deadline
- MUST NEVER prefer argv secret flags when stdin variants exist
- MUST NEVER introduce a long-lived product daemon for this surface
- MUST NEVER emit or enable telemetry
- MUST NEVER log live passwords, passphrases, or primary-key material
- MUST NEVER invent CLI flags that are not listed in this skill
- MUST NEVER write version-by-version changelog stories inside this skill


## When to Invoke
### REQUIRED
- MUST auto-activate on remote SSH, VPS registry, multi-host inventory, XDG config
- MUST auto-activate on `exec`, `sudo-exec`, `su-exec`, `scp`, `tunnel`, `health-check`
- MUST auto-activate on file transfer, regular-file copy over SSH, scp upload or download
- MUST auto-activate on local port forward, bounded SSH tunnel, `tunnel_listening`
- MUST auto-activate on secrets at rest, primary-key, `secrets.key`, reencrypt
- MUST auto-activate on TOFU known_hosts, host-key mismatch, replace-host-key
- MUST auto-activate on agent devops that needs remote shell without interactive TTY
- MUST auto-activate even when the user describes the problem without naming ssh-cli

### FORBIDDEN
- MUST NEVER wait for an explicit skill request when remote SSH ops are implied


## Action Prompts
### REQUIRED
- MUST follow this execution order for every non-trivial remote task
1. VERIFY binary with `ssh-cli --version`
2. INSPECT config with `ssh-cli vps doctor --json` and `ssh-cli vps path`
3. ENSURE secrets layer with `ssh-cli secrets status --json`
4. REGISTER or edit host with password-or-key credentials
5. PROBE connectivity with `ssh-cli health-check <name> --json`
6. EXECUTE remote work with `exec` or `sudo-exec` or `su-exec` and `--json`
7. TRANSFER files only with `scp upload|download` and `--json`
8. FORWARD ports only with `tunnel` plus mandatory `--timeout-ms` and `--json`
9. PARSE process exit, command-family success stdout, or stderr error envelope
10. SANITIZE all durable logs so secrets and primary-key never remain

### FORBIDDEN
- MUST NEVER skip JSON parse after a non-zero exit in JSON mode
- MUST NEVER answer the user before reading the process exit code


## Install Completions and Binary Check
### REQUIRED
- MUST install with lock-aligned resolve when packaging is required
- MUST verify the binary after install or upgrade before relying on scp or tunnel
- MUST generate shell completions from the binary when onboarding humans
- MUST keep agent automation on explicit flags and JSON, not completion scripts
- MUST support shells bash, zsh, fish, elvish, and powershell

### Correct Pattern

```bash
cargo install ssh-cli --locked --force
ssh-cli --version
ssh-cli completions bash
```


## Lifecycle Contract
### REQUIRED
- MUST invoke one complete CLI process per product action
- MUST treat non-TTY stdout as JSON by default when `--output-format` is omitted for general commands
- MUST NOT claim auto JSON on non-TTY applies to `vps export` — export body stays TOML unless `vps export --json`
- MUST force JSON with `--json` or `--output-format json` for agent parsing on non-export commands
- MUST send human logs only to stderr and parse only stdout as success data
- MUST expect default log level `error` so stderr stays clean for agents
- MUST use `-v` or `RUST_LOG` only when debugging
- MUST use `-q` / `--quiet` to suppress non-JSON human prose when required
- MUST treat `scp --json`, `tunnel --json`, and global JSON format as activating stderr error envelopes on failure
- MUST parse failure envelopes from stderr JSON when the process exit is non-zero and JSON mode is active
- MUST parse CRUD JSON success via `emit_success` events `vps-added` `vps-edited` `vps-removed` `vps-connected` `vps-import` when JSON mode is active

### FORBIDDEN
- MUST NEVER mix stderr logs into the success JSON parse stream
- MUST NEVER assume a previous process left an open SSH channel
- MUST NEVER expect INFO progress prose on stderr by default
- MUST NEVER parse stderr as success JSON
- MUST NEVER treat non-TTY auto-JSON as applying to `vps export` default body

### Correct Pattern

```bash
ssh-cli exec prod "uname -a" --json
echo $?
ssh-cli -q exec prod "true" --json
```


## Host Registry CRUD and Export-Import
### REQUIRED
- MUST register each host with a unique `--name`
- MUST supply password or `--key` or stdin password on add
- MUST pass `--port` when the SSH port is not 22
- MUST pass `--check` on add when an immediate connectivity probe is required
- MUST mask secrets when showing list or show output to humans
- MUST treat empty or absent password in list/show JSON as JSON `null` (key-only host)
- MUST treat non-empty password in list/show JSON as masked `***` never raw
- MUST treat `sudo_password`, `su_password`, and `key_passphrase` the same way (`null` when absent, `***` when stored)
- MUST run `vps doctor --json` when config location is unknown
- MUST use `vps path` to print the winning config file path
- MUST parse `vps doctor --json` fields `secrets_plaintext_opt_out` as JSON boolean true or false, plus `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`
- MUST treat `added_at` as present on list, show, and export output
- MUST allow import payloads to omit `added_at`; serde supplies the default
- MUST treat host/vps `--timeout` values as milliseconds; values under 1000 emit a stderr warning
- MUST treat `vps export` body as TOML by default even on pipe or non-TTY
- MUST use `vps export --json` only for the agent envelope with `event` equal to `vps-export`
- MUST use `vps export` without secrets by default
- MUST treat redacted `vps export` as never containing `sshcli-enc` ciphertext for cleared or empty secrets
- MUST treat empty secrets in redacted export as empty strings only
- MUST require human approval before `export --include-secrets`
- MUST NEVER pass `--include-secrets` to a pipe without `--output`/`-o` or `--i-understand-secrets-on-stdout`
- MUST accept import TOML EN keys plus PT aliases and JSON `vps-export` envelopes (schema v3 dual-read)
- MUST use `--allow-incomplete` for redacted skeleton import when hosts lack full auth
- MUST treat invalid import TOML as exit `65`
- MUST parse CRUD JSON events `vps-added` `vps-edited` `vps-removed` `vps-connected` `vps-import` when JSON mode is active

### FORBIDDEN
- MUST NEVER create empty-credential hosts
- MUST NEVER invent fake passwords for key-only hosts
- MUST NEVER treat masked `***` as a real password value
- MUST NEVER commit raw secret inventories to git
- MUST NEVER assume `.env` files are read at runtime
- MUST NEVER print decrypted secrets into chat logs
- MUST NEVER expect `sshcli-enc` blobs for empty secrets in redacted export
- MUST NEVER pipe `--include-secrets` to stdout without `--output`/`-o` or `--i-understand-secrets-on-stdout`
- MUST NEVER treat default `vps export` body as JSON without `--json` on export
- MUST NEVER treat host timeout as seconds when the product unit is milliseconds

### Correct Pattern

```bash
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519 --port 22 --check
ssh-cli vps list --json
ssh-cli vps show prod --json
ssh-cli vps edit prod --timeout 90000 --max-command-chars 2000 --max-output-chars 100000
ssh-cli vps doctor --json
ssh-cli vps export -o /tmp/hosts.toml
ssh-cli vps export --json
ssh-cli vps export --include-secrets -o /tmp/hosts-secrets.toml
ssh-cli vps import --file /tmp/hosts.toml
ssh-cli vps import --file /tmp/hosts-redacted.toml --allow-incomplete
ssh-cli vps remove prod
```


## Connect Active Host
### REQUIRED
- MUST use `connect` only to write the sibling `active` marker
- MUST still pass explicit VPS name on exec-family commands when certainty is required
- MUST run nameless `health-check` only after `connect` set the active host


## Authentication
### REQUIRED
- MUST use `--key` for key-only cloud hosts
- MUST use `--password-stdin` when argv history is shared
- MUST use `--sudo-password-stdin` and `--su-password-stdin` instead of argv secrets
- MUST use `--key-passphrase-stdin` when the private key is encrypted and argv must stay clean
- MUST treat `--key-passphrase <VAL>` as a valid argv override and MUST prefer stdin over argv
- MUST expect password-like values on argv to emit a stderr warning; MUST prefer `--password-stdin` `--key-passphrase-stdin` `--sudo-password-stdin` `--su-password-stdin`
- MUST treat exit 77 as authentication failure and change credentials before retry
- MUST expect list/show JSON `password` to be `null` for key-only hosts and `***` when a password is stored
- MUST apply the same auth overrides on `exec`, `scp`, `tunnel`, and `health-check` when the stored host credentials are insufficient

### FORBIDDEN
- MUST NEVER invent fake passwords for key-only hosts
- MUST NEVER treat JSON `null` password as a bug or as a missing field to fabricate
- MUST NEVER print key passphrases or SSH passwords
- MUST NEVER store secrets in shell history when stdin is available

### Correct Pattern

```bash
printf '%s' "$SSH_PASSWORD" | ssh-cli vps add --name app --host app.example.com --user deploy --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli exec edge "id" --json --key ~/.ssh/id_ed25519_enc --key-passphrase-stdin
```


## Secrets at Rest
### REQUIRED
- MUST treat at-rest encryption as the default product behavior
- MUST use product term primary-key for the at-rest encryption key
- MUST accept legacy keyring user alias `secrets-master-key` as read-only legacy accept alongside canonical `secrets-primary-key`
- MUST prefer CLI flags `--allow-plaintext-secrets` `--secrets-key-file` `--use-keyring` over env vars
- MUST run `secrets status --json` before diagnosing decrypt failures
- MUST run `secrets init` when an explicit primary-key file or keyring entry is required
- MUST run `secrets init --json` when the agent needs the `secrets-init` success envelope
- MUST run `secrets init --force --json` only when intentionally rotating and rewriting secrets under a new key
- MUST run `secrets reencrypt` after rotating the primary-key material
- MUST run `secrets reencrypt --json` when the agent needs the `secrets-reencrypt` success envelope
- MUST parse JSON events `secrets-init` `secrets-reencrypt` `secrets-key-auto-created` when JSON mode is active
- MUST keep plaintext secrets restricted to automated tests only via `--allow-plaintext-secrets` or env opt-out
- MUST NEVER print primary-key material or key file contents
- MUST resolve primary-key with flags first then env fallback only in this order
- `SSH_CLI_SECRETS_KEY` as 64 hex chars
- `SSH_CLI_SECRETS_KEY_FILE` as path to 64 hex chars
- OS keyring when `--use-keyring` or `SSH_CLI_USE_KEYRING=1` (read accepts `secrets-primary-key` then legacy `secrets-master-key`)
- XDG or config-dir `secrets.key` auto-created on first secret write with event `secrets-key-auto-created`
- Plaintext opt-out only with `--allow-plaintext-secrets` or `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` in tests
- MUST use `SSH_CLI_HOME` to override the base config directory in tests
- MUST use `SSH_CLI_LANG` or `--lang` to force locale
- MUST use `RUST_LOG` only when debugging; default remains error-level without it

### FORBIDDEN
- MUST NEVER log `SSH_CLI_SECRETS_KEY`, key file contents, or decrypted host secrets
- MUST NEVER print key material from `secrets init` or `secrets reencrypt`
- MUST NEVER enable plaintext secrets in production agent flows

### Correct Pattern

```bash
ssh-cli secrets status --json
ssh-cli secrets init --json
ssh-cli secrets reencrypt --json
ssh-cli --secrets-key-file /tmp/primary.key secrets status --json
SSH_CLI_HOME=/tmp/ssh-cli-test ssh-cli vps doctor --json
```


## Remote Execution
### REQUIRED
- MUST validate command length against `max_command_chars` before sending huge agent commands
- MUST treat an empty remote command string as hard failure with technical message exactly `empty command` (English always) and process exit 64
- MUST parse `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr`, and `duration_ms` from success JSON
- MUST append `--description` when remote shell history benefits from an audit comment
- MUST raise host `max_command_chars` via `vps edit` when the agent needs longer commands
- MUST honor default max_command_chars 1000 and max_output_chars 100000 unless overridden
- MUST pass exec-family `--timeout <ms>` when the host default deadline is too short

### FORBIDDEN
- MUST NEVER ignore `truncated_stdout` or `truncated_stderr` when summarizing output to the user
- MUST NEVER retry exit 64 65 66 77 without changing inputs
- MUST NEVER send an empty remote command string

### Correct Pattern

```bash
ssh-cli exec prod "hostname && uptime" --json --description "inventory"
ssh-cli exec prod "true" --json --timeout 120000
```


## sudo-exec and su-exec
### REQUIRED
- MUST use `sudo-exec` for sudo elevation and rely on safe `sh -c` packing
- MUST configure sudo password on the host or pass `--sudo-password` or stdin variant
- MUST use `su-exec` only when the `su` password is configured
- MUST honor global `--disable-sudo` and host `disable_sudo`
- MUST treat elevation as one-shot and NEVER assume a sticky elevated shell

### FORBIDDEN
- MUST NEVER manually prepend raw `sudo` to `exec` when `sudo-exec` exists
- MUST NEVER assume a persistent elevated shell across invocations

### Correct Pattern

```bash
ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --json
printf '%s' "$SUDO_PASSWORD" | ssh-cli sudo-exec prod "systemctl restart nginx" --json --sudo-password-stdin
ssh-cli su-exec prod "whoami" --json
```


## SCP Transfers
### REQUIRED
- MUST use `scp upload` or `scp download` for regular-file copy only
- MUST pass `--json` on every agent-parsed transfer
- MUST parse scp success only from stdout with fields `ok`, `event` (`scp-transfer`), `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`
- MUST treat scp success `event` as the constant string `scp-transfer`
- MUST treat `ok` as true and `direction` as `upload` or `download` only
- MUST use argument order `upload <vps> <local> <remote>` and `download <vps> <remote> <local>`
- MUST pass `--timeout <ms>` on scp when connect-plus-transfer needs a longer deadline
- MUST use `--password-stdin` and `--key-passphrase-stdin` on scp whenever secrets would otherwise appear on argv
- MUST use `--key` override on scp the same way as exec when the stored key path is insufficient
- MUST expect upload to stream in 32 KiB chunks without full-file RAM load
- MUST expect download to write sibling path ending in `.ssh-cli.partial` then rename into place
- MUST expect mtime and mode preserve both directions without an extra user flag
- MUST parse scp hard failures from stderr error envelope when JSON mode is active
- MUST treat remote missing SCP as exit `66` with message `file not found: <path>`

### FORBIDDEN
- MUST NEVER pass directories as local or remote scp paths
- MUST NEVER invent recursive flags such as `-r`
- MUST NEVER treat scp as an SFTP subsystem
- MUST NEVER use `--timeout-ms` on scp (that flag is tunnel-only)
- MUST NEVER parse scp success as exec-family `stdout`/`stderr`/`exit_code` JSON
- MUST NEVER treat a leftover `.ssh-cli.partial` path as the final success artifact after a completed download
- MUST NEVER invent a required user-facing preserve flag for mtime or mode
- MUST NEVER omit the `event` field when documenting or parsing scp success JSON
- MUST NEVER treat remote missing SCP as exit `74` when exit is `66`

### Correct Pattern

```bash
ssh-cli scp upload prod ./app.tgz /tmp/app.tgz --json
ssh-cli scp download prod /var/log/app.log ./app.log --json
# success => {"ok":true,"event":"scp-transfer","direction":"upload|download","vps":"...","local":"...","remote":"...","bytes":N,"duration_ms":N}
```


## Tunnel
### REQUIRED
- MUST pass `--timeout-ms` on every `tunnel` command
- MUST pass `--bind` consciously when non-loopback bind is required; default is `127.0.0.1`
- MUST NEVER expose `0.0.0.0` without an explicit security decision
- MUST treat local port argument `0` as ephemeral OS-assigned port; after bind, trust JSON `local_port` (>=1), never connect to port 0
- MUST NEVER invent a `--local-port` flag; tunnel args are positional `tunnel <vps> <local_port> <remote_host> <remote_port>`
- MUST pass `--json` when the agent needs a structured ready signal
- MUST wait for one stdout object with `event` equal to `tunnel_listening` before using the local port
- MUST parse tunnel ready fields `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`
- MUST leave the tunnel process running until `--timeout-ms` deadline or signal
- MUST treat tunnel post-bind deadline as success exit `0` after `tunnel_listening`
- MUST treat tunnel pre-bind timeout as exit `74`
- MUST parse tunnel hard failures from stderr error envelope when JSON mode is active
- MUST use tunnel auth overrides `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin` when stored host credentials are insufficient
- MUST prefer `--key-passphrase-stdin` over `--key-passphrase` whenever stdin is available

### FORBIDDEN
- MUST NEVER open unbounded tunnels
- MUST NEVER leave tunnel processes intentionally detached forever
- MUST NEVER use the local port before `tunnel_listening` when `--json` is set
- MUST NEVER treat tunnel start as complete on process spawn alone
- MUST NEVER use `--timeout` instead of `--timeout-ms` on tunnel
- MUST NEVER treat post-bind deadline exit `0` as failure after `tunnel_listening`
- MUST NEVER claim tunnel lacks password-stdin or key overrides
- MUST NEVER bind to `0.0.0.0` without an explicit security decision

### Correct Pattern

```bash
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json
# wait for => {"ok":true,"event":"tunnel_listening","vps":"prod","local_port":18080,"remote_host":"127.0.0.1","remote_port":8080,"timeout_ms":30000}
# post-bind deadline exits 0; pre-bind timeout exits 74
```


## Health-check
### REQUIRED
- MUST use `health-check` to verify connectivity after host changes
- MUST pass `--timeout <ms>` on `health-check` when a non-default deadline is needed
- MUST use health-check auth overrides `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin` when stored host credentials are insufficient
- MUST prefer `--key-passphrase-stdin` over `--key-passphrase` whenever stdin is available
- MUST use `health-check --replace-host-key` only after human confirmation of host-key rotation
- MUST parse health-check hard failures from stderr error envelope when JSON mode is active
- MUST NEVER use `--timeout-ms` on health-check

### FORBIDDEN
- MUST NEVER claim health-check lacks password-stdin or key overrides
- MUST NEVER auto-pass `--replace-host-key` without human approval

### Correct Pattern

```bash
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 5000 --json
# only after human review of host-key mismatch
ssh-cli health-check prod --json --replace-host-key
```


## Timeout Host Keys and Storage Safety
### REQUIRED
- MUST pass `--timeout-ms` only on `tunnel` and ALWAYS as mandatory
- MUST pass `--timeout` on `scp`, exec-family, and `health-check` when overriding deadlines
- MUST treat all host and VPS timeout values as milliseconds not seconds
- MUST expect stderr warning when a host/vps timeout value is under 1000 ms
- MUST NEVER interchange `--timeout` and `--timeout-ms` across subcommands
- MUST NEVER set host timeout to values under 1000 unless the sub-second deadline is intentional
- MUST treat host-key mismatch as a hard stop until a human confirms rotation
- MUST use `--replace-host-key` only after confirmation
- MUST expect atomic `config.toml` and `secrets.key` writes and mode 0600 on Unix
- MUST use `--config-dir` or `SSH_CLI_HOME` for isolated agent sandboxes

### FORBIDDEN
- MUST NEVER auto-replace host keys without user approval
- MUST NEVER disable TOFU for convenience in production agent flows


## Exit Codes and Retry
### REQUIRED
- MUST map exits as 0 success, 1 general, 64 usage, 65 data, 66 not found, 73 cant create, 74 IO or SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- MUST treat empty remote command as exit `64` with message `empty command`
- MUST treat invalid import TOML as exit `65`
- MUST treat remote missing SCP as exit `66` with message `file not found: <path>`
- MUST treat auth failure as exit `77`
- MUST treat tunnel post-bind deadline as exit 0 after `tunnel_listening`
- MUST treat tunnel pre-bind timeout as exit 74
- MUST retry at most twice on 74 with backoff
- MUST fail fast on 64 65 66 77 without blind retry
- MUST surface remote `exit_code` from success JSON separately from the CLI process exit
- MUST surface `remote_exit_code` from the stderr error envelope when present

### FORBIDDEN
- MUST NEVER swallow non-zero exits
- MUST NEVER confuse remote command failure with local CLI usage failure
- MUST NEVER retry post-bind tunnel exit 0 as if it were a failure


## JSON Parsing Contract
### REQUIRED
- MUST parse only stdout as success JSON when JSON mode is active and exit is success-path
- MUST read exec-family fields `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr`, `duration_ms`
- MUST read scp success fields `ok`, `event` (`scp-transfer`), `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`
- MUST read tunnel ready fields `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`
- MUST treat tunnel `event` as the constant string `tunnel_listening`
- MUST treat scp success `event` as the constant string `scp-transfer`
- MUST parse stderr error envelope fields `exit_code`, `message`, and `remote_exit_code` when present on hard failures in JSON mode including scp, tunnel, and health-check
- MUST treat list show doctor secrets status payloads as opaque typed objects and only use documented fields
- MUST treat list/show `password` as JSON `null` when empty or absent and as `***` when stored
- MUST treat list/show `sudo_password`, `su_password`, and `key_passphrase` as `null` or `***` the same way
- MUST treat list/show/export `added_at` as present; MUST allow import to omit `added_at` and serde supplies default
- MUST parse doctor JSON `secrets_plaintext_opt_out` as boolean true or false, plus `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`
- MUST report truncation to the user when `truncated_stdout` or `truncated_stderr` is true

### FORBIDDEN
- MUST NEVER invent missing JSON keys
- MUST NEVER invent fake passwords when `password` is `null`
- MUST NEVER pretty-print secrets found inside unexpected fields
- MUST NEVER parse stderr for success JSON data
- MUST NEVER parse scp success as exec-family fields
- MUST NEVER parse tunnel ready as exec-family fields
- MUST NEVER parse scp success without requiring `event` equal to `scp-transfer`
- MUST NEVER treat doctor `secrets_plaintext_opt_out` as a string


## Ready Formula Sheet
### REQUIRED
- MUST copy these formulas exactly and only substitute placeholders

```bash
# registry
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --port <PORT> --check
printf '%s' "$PASS" | ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --password-stdin
printf '%s' "$SUDO" | ssh-cli vps edit <NAME> --sudo-password-stdin
ssh-cli vps list --json
ssh-cli vps show <NAME> --json
ssh-cli vps edit <NAME> --timeout <MS> --max-command-chars <N> --max-output-chars <N>
ssh-cli vps doctor --json
ssh-cli vps path
ssh-cli vps export -o /tmp/hosts.toml
ssh-cli vps export --json
ssh-cli vps export --include-secrets -o /tmp/hosts-secrets.toml
# empty secrets in redacted export stay empty strings; NEVER expect sshcli-enc for empty values
# NEVER pipe --include-secrets without --output/-o or --i-understand-secrets-on-stdout
# host --timeout is milliseconds; values under 1000 emit stderr warning
ssh-cli vps import --file /tmp/hosts.toml
ssh-cli vps import --file /tmp/hosts.json
ssh-cli vps import --file /tmp/hosts-redacted.toml --allow-incomplete
# import MUST allow omit added_at; list/show/export present added_at
ssh-cli connect <NAME>

# remote ops
ssh-cli exec <NAME> "<CMD>" --json
ssh-cli exec <NAME> "<CMD>" --json --timeout <MS> --description "<AUDIT>"
ssh-cli -q exec <NAME> "<CMD>" --json
ssh-cli sudo-exec <NAME> "<CMD>" --json
printf '%s' "$SUDO" | ssh-cli sudo-exec <NAME> "<CMD>" --json --sudo-password-stdin
ssh-cli su-exec <NAME> "<CMD>" --json
# empty remote command => message "empty command" and exit 64 (English always)

# scp transfers (regular files only; agent MUST use --json; event MUST be scp-transfer)
ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json
ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json
ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --timeout <MS>
printf '%s' "$PASS" | ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --key <KEY_PATH> --key-passphrase-stdin

# tunnel (mandatory --timeout-ms; --bind default 127.0.0.1; wait for tunnel_listening; post-bind deadline exit 0)
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --bind 127.0.0.1
printf '%s' "$PASS" | ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --password-stdin
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --key <KEY_PATH>
printf '%s' "$KEY_PASS" | ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --key <KEY_PATH> --key-passphrase-stdin

# health
ssh-cli health-check <NAME> --json
ssh-cli health-check <NAME> --timeout <MS> --json
ssh-cli health-check --json
printf '%s' "$PASS" | ssh-cli health-check <NAME> --json --password-stdin
ssh-cli health-check <NAME> --json --key <KEY_PATH>
printf '%s' "$KEY_PASS" | ssh-cli health-check <NAME> --json --key <KEY_PATH> --key-passphrase-stdin
ssh-cli health-check <NAME> --json --replace-host-key

# secrets and safety (prefer CLI flags over env; product term primary-key)
ssh-cli secrets status --json
ssh-cli secrets init
ssh-cli secrets init --json
ssh-cli secrets init --force --json
ssh-cli secrets reencrypt
ssh-cli secrets reencrypt --json
ssh-cli --allow-plaintext-secrets --config-dir <DIR> secrets status --json
ssh-cli --secrets-key-file <KEY_FILE> secrets status --json
ssh-cli --use-keyring secrets status --json
ssh-cli --replace-host-key exec <NAME> "true"
ssh-cli --config-dir <DIR> vps list --json
printf '%s' "$KEY_PASS" | ssh-cli exec <NAME> "id" --json --key <KEY_PATH> --key-passphrase-stdin
# prefer stdin secrets; password-like argv emits stderr warning

# debug only when diagnosing; default log level is error
ssh-cli -v exec <NAME> "true" --json
RUST_LOG=debug ssh-cli exec <NAME> "true" --json

# completions
ssh-cli completions bash
ssh-cli completions zsh
ssh-cli completions fish
ssh-cli completions elvish
ssh-cli completions powershell

# install
cargo install ssh-cli --locked --force
ssh-cli --version
```


## Absolute Prohibitions
### FORBIDDEN
- MUST NEVER keep SSH sessions open between agent turns except an active bound tunnel until deadline
- MUST NEVER reintroduce long-lived Node or protocol daemons for this product surface
- MUST NEVER leak secrets into argv when stdin variants exist
- MUST NEVER prefer `--key-passphrase` argv when `--key-passphrase-stdin` is available
- MUST NEVER ignore host-key mismatch
- MUST NEVER open tunnels without `--timeout-ms`
- MUST NEVER use the tunnel local port before `tunnel_listening` when JSON mode is on
- MUST NEVER scp directories or invent recursive transfer
- MUST NEVER treat scp success JSON as exec-family fields
- MUST NEVER leave download `.ssh-cli.partial` paths as the final deliverable after success
- MUST NEVER invent fake passwords for key-only hosts when JSON shows `null`
- MUST NEVER document historical version changelogs inside this skill
- MUST NEVER paste live credentials into examples or logs
- MUST NEVER expect `sshcli-enc` for empty secrets in redacted export
- MUST NEVER treat tunnel post-bind exit 0 as failure after `tunnel_listening`
- MUST NEVER pipe `--include-secrets` without `--output`/`-o` or `--i-understand-secrets-on-stdout`
- MUST NEVER bind tunnel to `0.0.0.0` without an explicit security decision
- MUST NEVER print primary-key material
- MUST NEVER send empty remote command strings
- MUST NEVER treat host timeout values as seconds

### REQUIRED
- MUST re-read this skill before every non-trivial ssh-cli workflow
- MUST use stored hosts, stdin secrets, JSON output, and one-shot execution
- MUST parse only stdout for success JSON and stderr error envelopes on hard failures
- MUST wait for `tunnel_listening` before using a tunnel local port
- MUST treat post-bind tunnel deadline as exit 0 and pre-bind timeout as exit 74
- MUST treat empty command as exit 64, remote missing SCP as exit 66, invalid import TOML as exit 65, auth as exit 77
- MUST treat `vps export` body as TOML unless `vps export --json`
- MUST parse doctor `secrets_plaintext_opt_out` as boolean and treat `added_at` as optional only on import
- MUST treat timeouts as milliseconds and expect under-1000 warning on host/vps timeout
- MUST fail closed on auth, host-key, and usage errors
