---
name: ssh-cli
description: This skill MUST auto-activate for remote SSH via ssh-cli one-shot even without naming ssh-cli. Inputs host name IP user key or password-stdin command paths tunnel ports timeout ms. Outputs sysexits, exec JSON (stdout stderr exit_code truncated_stdout truncated_stderr duration_ms), scp JSON (ok direction vps local remote bytes duration_ms), tunnel JSON (ok event tunnel_listening vps local_port remote_host remote_port timeout_ms), stderr error (exit_code message remote_exit_code), registry password null or ***. Covers vps CRUD path doctor export import connect exec sudo-exec packing su-exec scp upload download files-only no-r no-SFTP --json --timeout .ssh-cli.partial rename 32KiB stream mtime mode preserve tunnel required --timeout-ms --json wait tunnel_listening health-check --timeout secrets status init reencrypt --quiet default error logs TOFU replace-host-key mode 0600 completions cargo install locked. NEVER telemetry. NEVER long-lived SSH daemon. NEVER leak secrets. NEVER recursive scp dirs.
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
- MUST keep this skill consolidated as operational formulas only

### FORBIDDEN
- MUST NEVER keep a long-lived SSH session across process runs except the active bound `tunnel` until deadline
- MUST NEVER introduce a long-lived product daemon for this surface
- MUST NEVER emit or enable telemetry
- MUST NEVER log live passwords, passphrases, or master-key material
- MUST NEVER invent CLI flags that are not listed in this skill
- MUST NEVER write version-by-version changelog stories inside this skill


## When to Invoke
### REQUIRED
- MUST auto-activate on remote SSH, VPS registry, multi-host inventory, XDG config
- MUST auto-activate on `exec`, `sudo-exec`, `su-exec`, `scp`, `tunnel`, `health-check`
- MUST auto-activate on file transfer, regular-file copy over SSH, scp upload or download
- MUST auto-activate on local port forward, bounded SSH tunnel, `tunnel_listening`
- MUST auto-activate on secrets at rest, master-key, `secrets.key`, reencrypt
- MUST auto-activate on TOFU known_hosts, host-key mismatch, replace-host-key
- MUST auto-activate on agent devops that needs remote shell without interactive TTY
- MUST auto-activate even when the user describes the problem without naming ssh-cli

### FORBIDDEN
- MUST NEVER wait for an explicit skill request when remote SSH ops are implied


## Install and Binary Check
### REQUIRED
- MUST install with lock-aligned resolve when packaging is required
- MUST verify the binary after install or upgrade before relying on scp or tunnel
- MUST refuse to guide users past crypto pin failures without a fixed release

### Correct Pattern

```bash
cargo install ssh-cli --locked --force
ssh-cli --version
ssh-cli --help
```


## Lifecycle Contract
### REQUIRED
- MUST invoke one complete CLI process per product action
- MUST treat non-TTY stdout as JSON by default when `--output-format` is omitted
- MUST force JSON with `--json` or `--output-format json` for agent parsing
- MUST send human logs only to stderr and parse only stdout as success data
- MUST expect default log level `error` so stderr stays clean for agents
- MUST use `-v` or `RUST_LOG` only when debugging
- MUST use `-q` / `--quiet` to suppress non-JSON human prose when required
- MUST treat `scp --json`, `tunnel --json`, and global JSON format as activating stderr error envelopes on failure
- MUST parse failure envelopes from stderr JSON when the process exit is non-zero and JSON mode is active

### FORBIDDEN
- MUST NEVER mix stderr logs into the success JSON parse stream
- MUST NEVER assume a previous process left an open SSH channel
- MUST NEVER expect INFO progress prose on stderr by default
- MUST NEVER parse stderr as success JSON

### Correct Pattern

```bash
ssh-cli exec prod "uname -a" --json
echo $?
ssh-cli -q exec prod "true" --json
ssh-cli -v exec prod "true" --json
RUST_LOG=debug ssh-cli exec prod "true" --json
```


## Host Registry CRUD
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
- MUST use `vps export` without secrets by default
- MUST require human approval before `export --include-secrets`

### FORBIDDEN
- MUST NEVER create empty-credential hosts
- MUST NEVER invent fake passwords for key-only hosts
- MUST NEVER treat masked `***` as a real password value
- MUST NEVER commit raw secret inventories to git
- MUST NEVER assume `.env` files are read at runtime
- MUST NEVER print decrypted secrets into chat logs

### Correct Pattern

```bash
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519 --port 22 --check
ssh-cli vps list --json
ssh-cli vps show prod --json
ssh-cli vps edit prod --timeout 90000 --max-command-chars 2000 --max-output-chars 100000
ssh-cli vps path
ssh-cli vps doctor --json
ssh-cli vps export -o /tmp/hosts-redacted.toml
ssh-cli vps import --file /tmp/hosts-redacted.toml
ssh-cli vps remove prod
```


## Connect Active Host
### REQUIRED
- MUST use `connect` only to write the sibling `active` marker
- MUST still pass explicit VPS name on exec-family commands when certainty is required
- MUST allow `health-check` without a name only after `connect` set the active host

### Correct Pattern

```bash
ssh-cli connect prod
ssh-cli health-check --json
ssh-cli health-check prod --json
```


## Authentication
### REQUIRED
- MUST use `--key` for key-only cloud hosts
- MUST use `--password-stdin` when argv history is shared
- MUST use `--sudo-password-stdin` and `--su-password-stdin` instead of argv secrets
- MUST use `--key-passphrase-stdin` when the private key is encrypted and argv must stay clean
- MUST treat exit 77 as authentication failure and change credentials before retry
- MUST expect list/show JSON `password` to be `null` for key-only hosts and `***` when a password is stored

### FORBIDDEN
- MUST NEVER invent fake passwords for key-only hosts
- MUST NEVER treat JSON `null` password as a bug or as a missing field to fabricate
- MUST NEVER print key passphrases or SSH passwords
- MUST NEVER store secrets in shell history when stdin is available

### Correct Pattern

```bash
ssh-cli vps add --name edge --host edge.example.com --user ubuntu --key ~/.ssh/id_ed25519
printf '%s' "$SSH_PASSWORD" | ssh-cli vps add --name app --host app.example.com --user deploy --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli exec edge "id" --json --key ~/.ssh/id_ed25519_enc --key-passphrase-stdin
ssh-cli exec edge "id" --json
printf '%s' "$SSH_PASSWORD" | ssh-cli exec app "id" --json --password-stdin
```


## Secrets at Rest
### REQUIRED
- MUST treat at-rest encryption as the default product behavior
- MUST run `secrets status --json` before diagnosing decrypt failures
- MUST run `secrets init` when an explicit master-key file or keyring entry is required
- MUST run `secrets reencrypt` after rotating the master-key material
- MUST keep `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` restricted to automated tests only
- MUST NEVER print the master-key value

### FORBIDDEN
- MUST NEVER log `SSH_CLI_SECRETS_KEY`, key file contents, or decrypted host secrets
- MUST NEVER enable plaintext secrets in production agent flows

### Correct Pattern

```bash
ssh-cli secrets status --json
ssh-cli secrets init
ssh-cli secrets init --keyring
ssh-cli secrets reencrypt
```

### Env Precedence Formulas
- MUST resolve master-key in this order only
- `SSH_CLI_SECRETS_KEY` as 64 hex chars
- `SSH_CLI_SECRETS_KEY_FILE` as path to 64 hex chars
- OS keyring when `SSH_CLI_USE_KEYRING=1`
- XDG or config-dir `secrets.key` auto-created on first secret write
- Plaintext opt-out only with `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`


## Remote Execution
### REQUIRED
- MUST validate command length against `max_command_chars` before sending huge agent commands
- MUST parse `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr`, and `duration_ms` from success JSON
- MUST append `--description` when remote shell history benefits from an audit comment
- MUST raise host `max_command_chars` via `vps edit` when the agent needs longer commands
- MUST honor default max_command_chars 1000 and max_output_chars 100000 unless overridden
- MUST pass exec-family `--timeout <ms>` when the host default deadline is too short

### FORBIDDEN
- MUST NEVER ignore `truncated_stdout` or `truncated_stderr` when summarizing output to the user
- MUST NEVER retry exit 64 65 66 77 without changing inputs

### Correct Pattern

```bash
ssh-cli exec prod "hostname && uptime" --json --description "inventory"
ssh-cli exec prod "true" --json --timeout 120000
ssh-cli vps edit prod --max-command-chars 4000 --max-output-chars 200000
ssh-cli exec prod "long-agent-command-here" --json
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
ssh-cli --disable-sudo exec prod "id" --json
```


## SCP Transfers
### REQUIRED
- MUST use `scp upload` or `scp download` for regular-file copy only
- MUST pass `--json` on every agent-parsed transfer
- MUST parse scp success only from stdout with fields `ok`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`
- MUST treat `ok` as true and `direction` as `upload` or `download` only
- MUST use argument order `upload <vps> <local> <remote>` and `download <vps> <remote> <local>`
- MUST pass optional `--timeout <ms>` on scp when connect-plus-transfer needs a longer deadline
- MUST prefer `--password-stdin` and `--key-passphrase-stdin` over argv secrets on scp
- MUST allow `--key` override on scp the same way as exec
- MUST expect upload to stream in 32 KiB chunks without full-file RAM load
- MUST expect download to write sibling path ending in `.ssh-cli.partial` then rename into place
- MUST expect mtime and mode preserve both directions without an extra user flag
- MUST parse scp hard failures from stderr error envelope when JSON mode is active

### FORBIDDEN
- MUST NEVER pass directories as local or remote scp paths
- MUST NEVER invent recursive flags such as `-r`
- MUST NEVER treat scp as an SFTP subsystem
- MUST NEVER use `--timeout-ms` on scp (that flag is tunnel-only)
- MUST NEVER parse scp success as exec-family `stdout`/`stderr`/`exit_code` JSON
- MUST NEVER treat a leftover `.ssh-cli.partial` path as the final success artifact after a completed download
- MUST NEVER invent a required user-facing preserve flag for mtime or mode

### Correct Pattern

```bash
ssh-cli scp upload prod ./app.tgz /tmp/app.tgz --json
ssh-cli scp download prod /var/log/app.log ./app.log --json
ssh-cli scp upload prod ./big.bin /tmp/big.bin --json --timeout 300000
printf '%s' "$PASS" | ssh-cli scp download prod /etc/app.env ./app.env --json --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli scp upload prod ./payload.bin /tmp/payload.bin --json --key ~/.ssh/id_ed25519_enc --key-passphrase-stdin
# success stdout => {"ok":true,"direction":"upload|download","vps":"...","local":"...","remote":"...","bytes":N,"duration_ms":N}
# non-zero exit => stderr {"exit_code":N,"message":"..."} optional remote_exit_code
```


## Tunnel
### REQUIRED
- MUST pass `--timeout-ms` on every `tunnel` command
- MUST pass `--json` when the agent needs a structured ready signal
- MUST wait for one stdout object with `event` equal to `tunnel_listening` before using the local port
- MUST parse tunnel ready fields `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`
- MUST leave the tunnel process running until `--timeout-ms` deadline or signal
- MUST parse tunnel hard failures from stderr error envelope when JSON mode is active
- MUST allow `--key` override on tunnel when required

### FORBIDDEN
- MUST NEVER open unbounded tunnels
- MUST NEVER leave tunnel processes intentionally detached forever
- MUST NEVER use the local port before `tunnel_listening` when `--json` is set
- MUST NEVER treat tunnel start as complete on process spawn alone
- MUST NEVER use `--timeout` instead of `--timeout-ms` on tunnel
- MUST NEVER invent `--password-stdin` on tunnel if not listed here

### Correct Pattern

```bash
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json
# wait for stdout => {"ok":true,"event":"tunnel_listening","vps":"prod","local_port":18080,"remote_host":"127.0.0.1","remote_port":8080,"timeout_ms":30000}
# then use 127.0.0.1:18080; process stays alive until deadline or SIGINT/SIGTERM
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json --key ~/.ssh/id_ed25519
```


## Health-check
### REQUIRED
- MUST use `health-check` to verify connectivity after host changes
- MUST pass optional `--timeout <ms>` on `health-check` when a non-default deadline is needed
- MUST NEVER use `--timeout-ms` on health-check

### Correct Pattern

```bash
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 5000 --json
ssh-cli health-check --json
```


## Timeout Flag Matrix
### REQUIRED
- MUST pass `--timeout-ms` only on `tunnel` and ALWAYS as mandatory
- MUST pass `--timeout` on `scp`, exec-family, and `health-check` when overriding deadlines
- MUST NEVER interchange `--timeout` and `--timeout-ms` across subcommands


## Host Keys and Storage Safety
### REQUIRED
- MUST treat host-key mismatch as a hard stop until a human confirms rotation
- MUST use `--replace-host-key` only after confirmation
- MUST expect atomic `config.toml` writes and mode 0600 on Unix
- MUST expect atomic `secrets.key` writes and mode 0600 on Unix
- MUST use `--config-dir` or `SSH_CLI_HOME` for isolated agent sandboxes

### FORBIDDEN
- MUST NEVER auto-replace host keys without user approval
- MUST NEVER disable TOFU for convenience in production agent flows

### Correct Pattern

```bash
ssh-cli vps doctor --json
# only after human review of mismatch details
ssh-cli --replace-host-key exec prod "true"
ssh-cli --config-dir /tmp/ssh-cli-sandbox vps list --json
```


## Completions
### REQUIRED
- MUST generate shell completions from the binary when onboarding humans
- MUST keep agent automation on explicit flags and JSON, not completion scripts
- MUST support shells bash, zsh, fish, elvish, and powershell

### Correct Pattern

```bash
ssh-cli completions bash
ssh-cli completions zsh
ssh-cli completions fish
ssh-cli completions elvish
ssh-cli completions powershell
```


## Exit Codes and Retry
### REQUIRED
- MUST map exits as 0 success, 1 general, 64 usage, 65 data, 66 not found, 73 cant create, 74 IO or SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- MUST retry at most twice on 74 with backoff
- MUST fail fast on 64 65 66 77 without blind retry
- MUST surface remote `exit_code` from success JSON separately from the CLI process exit
- MUST surface `remote_exit_code` from the stderr error envelope when present

### FORBIDDEN
- MUST NEVER swallow non-zero exits
- MUST NEVER confuse remote command failure with local CLI usage failure

### Correct Pattern

```bash
ssh-cli exec prod "true" --json
echo $?
ssh-cli exec missing-host "true" --json; echo $?
```


## JSON Parsing Contract
### REQUIRED
- MUST parse only stdout as success JSON when JSON mode is active and exit is success-path
- MUST read exec-family fields `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr`, `duration_ms`
- MUST read scp success fields `ok`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`
- MUST read tunnel ready fields `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`
- MUST treat tunnel `event` as the constant string `tunnel_listening`
- MUST parse stderr error envelope fields `exit_code`, `message`, and optional `remote_exit_code` on hard failures in JSON mode including scp and tunnel
- MUST treat list show doctor secrets status payloads as opaque typed objects and only use documented fields
- MUST treat list/show `password` as JSON `null` when empty or absent and as `***` when stored
- MUST treat list/show `sudo_password`, `su_password`, and `key_passphrase` as `null` or `***` the same way
- MUST report truncation to the user when `truncated_stdout` or `truncated_stderr` is true

### FORBIDDEN
- MUST NEVER invent missing JSON keys
- MUST NEVER invent fake passwords when `password` is `null`
- MUST NEVER pretty-print secrets found inside unexpected fields
- MUST NEVER parse stderr for success JSON data
- MUST NEVER parse scp success as exec-family fields
- MUST NEVER parse tunnel ready as exec-family fields

### Correct Pattern

```bash
ssh-cli vps list --json
ssh-cli vps show prod --json
# key-only host => "password": null
# password host  => "password": "***"
ssh-cli exec prod "uname -a" --json
# exec success => stdout/stderr/exit_code/truncated_*/duration_ms
ssh-cli scp upload prod ./f.bin /tmp/f.bin --json
# scp success => ok/direction/vps/local/remote/bytes/duration_ms
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 10000 --json
# tunnel ready => ok/event/vps/local_port/remote_host/remote_port/timeout_ms
```


## Environment Variables
### REQUIRED
- MUST use `SSH_CLI_HOME` to override the base config directory in tests
- MUST use `SSH_CLI_LANG` or `--lang` to force locale
- MUST use `SSH_CLI_SECRETS_KEY` only as a 64-hex master key and never log it
- MUST use `SSH_CLI_SECRETS_KEY_FILE` when the master key lives in a file
- MUST use `SSH_CLI_USE_KEYRING=1` when OS keyring storage is required
- MUST reserve `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` for tests only
- MUST use `RUST_LOG` only when debugging; default remains error-level without it

### Correct Pattern

```bash
SSH_CLI_HOME=/tmp/ssh-cli-test ssh-cli vps doctor --json
SSH_CLI_LANG=en-US ssh-cli --help
RUST_LOG=debug ssh-cli -v exec prod "true" --json
```


## Agent Workflow
### REQUIRED
1. FIRST verify binary with `ssh-cli --version`
2. THEN inspect config with `ssh-cli vps doctor --json` and `ssh-cli vps path`
3. THEN ensure secrets layer with `ssh-cli secrets status --json`
4. THEN register or edit host with password-or-key credentials
5. THEN run `ssh-cli health-check <name> --json` (add `--timeout <ms>` when needed)
6. THEN run `exec` or `sudo-exec` or `su-exec` with `--json`
7. THEN for file transfer run `scp upload|download` with `--json` and parse scp-transfer fields
8. THEN for port forward run `tunnel` with mandatory `--timeout-ms` and `--json`; wait for `tunnel_listening` before use
9. THEN parse process exit, success stdout schema for the command family, or stderr error envelope before answering the user
10. FINALLY NEVER leave secrets or master-key in durable logs

### Correct Pattern

```bash
ssh-cli --version
ssh-cli vps doctor --json
ssh-cli secrets status --json
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519 --check
ssh-cli health-check prod --json
ssh-cli exec prod "uname -a && df -h" --json --description "baseline"
ssh-cli scp upload prod ./artifact.tgz /tmp/artifact.tgz --json
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json
```


## Absolute Prohibitions
### FORBIDDEN
- MUST NEVER keep SSH sessions open between agent turns except an active bound tunnel until deadline
- MUST NEVER reintroduce long-lived Node or protocol daemons for this product surface
- MUST NEVER leak secrets into argv when stdin variants exist
- MUST NEVER ignore host-key mismatch
- MUST NEVER open tunnels without `--timeout-ms`
- MUST NEVER use the tunnel local port before `tunnel_listening` when JSON mode is on
- MUST NEVER scp directories or invent recursive transfer
- MUST NEVER treat scp success JSON as exec-family fields
- MUST NEVER leave download `.ssh-cli.partial` paths as the final deliverable after success
- MUST NEVER expect INFO progress prose on stderr by default
- MUST NEVER invent fake passwords for key-only hosts when JSON shows `null`
- MUST NEVER document historical version changelogs inside this skill
- MUST NEVER invent version-by-version feature stories
- MUST NEVER paste live credentials into examples or logs


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
ssh-cli vps export -o <FILE>
ssh-cli vps import --file <FILE>
ssh-cli connect <NAME>

# remote ops
ssh-cli exec <NAME> "<CMD>" --json
ssh-cli exec <NAME> "<CMD>" --json --timeout <MS> --description "<AUDIT>"
ssh-cli -q exec <NAME> "<CMD>" --json
ssh-cli sudo-exec <NAME> "<CMD>" --json
printf '%s' "$SUDO" | ssh-cli sudo-exec <NAME> "<CMD>" --json --sudo-password-stdin
ssh-cli su-exec <NAME> "<CMD>" --json

# scp transfers (regular files only; agent MUST use --json)
ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json
ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json
ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --timeout <MS>
printf '%s' "$PASS" | ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --key <KEY_PATH> --key-passphrase-stdin

# tunnel (mandatory --timeout-ms; wait for tunnel_listening before use)
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json

# health
ssh-cli health-check <NAME> --json
ssh-cli health-check <NAME> --timeout <MS> --json
ssh-cli health-check --json

# secrets and safety
ssh-cli secrets status --json
ssh-cli secrets init
ssh-cli secrets reencrypt
ssh-cli --replace-host-key exec <NAME> "true"
ssh-cli --config-dir <DIR> vps list --json
printf '%s' "$KEY_PASS" | ssh-cli exec <NAME> "id" --json --key <KEY_PATH> --key-passphrase-stdin

# debug (optional; default log level is error)
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


## Final Reminder
### REQUIRED
- MUST re-read this skill before every non-trivial ssh-cli workflow
- MUST use stored hosts, stdin secrets, JSON output, and one-shot execution
- MUST parse only stdout for success JSON and keep default stderr quiet
- MUST parse stderr error envelopes on hard failures including scp and tunnel
- MUST wait for `tunnel_listening` before using a tunnel local port
- MUST treat scp as regular files only with partial-then-rename downloads
- MUST fail closed on auth, host-key, and usage errors
- MUST keep this skill consolidated as operational formulas only
