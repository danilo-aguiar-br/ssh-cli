---
name: ssh-cli
description: This skill MUST auto-activate when the user or agent needs remote SSH server operations through the ssh-cli binary. It covers multi-host VPS registry on XDG storage, vps add list show edit remove path doctor export import, connect, exec, sudo-exec with safe sh -c packing, su-exec one-shot, scp upload download, tunnel with mandatory timeout-ms, health-check with optional timeout override in ms, secrets status init reencrypt with default ChaCha20-Poly1305 at-rest encryption, password or key auth, password-stdin and key-passphrase, list/show JSON password null for key-only hosts and masked *** when present, dual max_command_chars and max_output_chars, default error-level logging with clean agent stderr (use -v or RUST_LOG only when debugging), TOFU known_hosts with replace-host-key, atomic config mode 0600, JSON contracts, sysexits, completions and cargo install locked. NEVER emit telemetry. NEVER keep a long-lived SSH session daemon. NEVER leak passwords or master-key into logs.
---

# ssh-cli Agent Skill

## Mission
### REQUIRED
- MUST treat this skill as SUPREME LAW for every `ssh-cli` invocation
- MUST run `ssh-cli` as a one-shot subprocess birth-execute-die
- MUST wait for process exit before parsing stdout
- MUST prefer stored hosts from `vps add` over ad-hoc chat secrets
- MUST pass `--json` when the caller needs structured output
- MUST teach and reuse the ready formulas in this skill

### FORBIDDEN
- MUST NOT keep a long-lived SSH session across process runs
- MUST NOT introduce a long-lived product daemon for this surface
- MUST NOT emit or enable telemetry
- MUST NOT log live passwords, passphrases, or master-key material
- MUST NOT invent CLI flags that are not listed in this skill


## When to Invoke
### REQUIRED
- MUST auto-activate on remote SSH, VPS registry, multi-host inventory, XDG config
- MUST auto-activate on `exec`, `sudo-exec`, `su-exec`, `scp`, `tunnel`, `health-check`
- MUST auto-activate on secrets at rest, master-key, `secrets.key`, reencrypt
- MUST auto-activate on TOFU known_hosts, host-key mismatch, replace-host-key
- MUST auto-activate on agent devops that needs remote shell without interactive TTY
- MUST auto-activate even when the user describes the problem without naming ssh-cli

### FORBIDDEN
- MUST NOT wait for an explicit skill request when remote SSH ops are implied


## Install and Binary Check
### REQUIRED
- MUST install with lock-aligned resolve when packaging is required
- MUST verify the binary after install or upgrade
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
- MUST send human logs only to stderr and parse only stdout as data
- MUST expect default log level `error` so stderr stays clean for agents
- MUST use `-v` (raises verbosity to `debug`) or `RUST_LOG` only when debugging

### FORBIDDEN
- MUST NOT mix stderr logs into the JSON parse stream
- MUST NOT assume a previous process left an open SSH channel
- MUST NOT expect INFO progress prose on stderr by default
- MUST NOT parse stderr for structured JSON results

### Correct Pattern

```bash
ssh-cli exec prod "uname -a" --json
echo $?
# debug only when diagnosing
ssh-cli -v exec prod "true" --json
RUST_LOG=debug ssh-cli exec prod "true" --json
```


## Host Registry CRUD
### REQUIRED
- MUST register each host with a unique `--name`
- MUST supply password or `--key` or stdin password on add
- MUST mask secrets when showing list or show output to humans
- MUST treat empty or absent password in list/show JSON as JSON `null` (key-only host)
- MUST treat non-empty password in list/show JSON as masked `***` never raw
- MUST run `vps doctor --json` when config location is unknown
- MUST use `vps path` to print the winning config file path
- MUST use `vps export` without secrets by default
- MUST require human approval before `export --include-secrets`

### FORBIDDEN
- MUST NOT create empty-credential hosts
- MUST NOT invent fake passwords for key-only hosts
- MUST NOT treat masked `***` as a real password value
- MUST NOT commit raw secret inventories to git
- MUST NOT assume `.env` files are read at runtime
- MUST NOT print decrypted secrets into chat logs

### Correct Pattern

```bash
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519
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

### Correct Pattern

```bash
ssh-cli connect prod
ssh-cli health-check prod --json
```


## Authentication
### REQUIRED
- MUST use `--key` for key-only cloud hosts
- MUST prefer `--password-stdin` when argv history is shared
- MUST prefer `--sudo-password-stdin` and `--su-password-stdin` over argv secrets
- MUST treat exit 77 as authentication failure and change credentials before retry
- MUST pass `--key-passphrase` or passphrase stdin only when the key is encrypted
- MUST expect list/show JSON `password` to be `null` for key-only hosts and `***` when a password is stored

### FORBIDDEN
- MUST NOT invent fake passwords for key-only hosts
- MUST NOT treat JSON `null` password as a bug or as a missing field to fabricate
- MUST NOT print key passphrases or SSH passwords
- MUST NOT store secrets in shell history when stdin is available

### Correct Pattern

```bash
ssh-cli vps add --name edge --host edge.example.com --user ubuntu --key ~/.ssh/id_ed25519
printf '%s' "$SSH_PASSWORD" | ssh-cli vps add --name app --host app.example.com --user deploy --password-stdin
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
- MUST never print the master-key value

### FORBIDDEN
- MUST NOT log `SSH_CLI_SECRETS_KEY`, key file contents, or decrypted host secrets
- MUST NOT enable plaintext secrets in production agent flows

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
- MUST parse `stdout`, `stderr`, `exit_code`, truncation flags, and `duration_ms` from JSON
- MUST append `--description` when remote shell history benefits from an audit comment
- MUST raise host `max_command_chars` via `vps edit` when the agent needs longer commands
- MUST honor default max_command_chars 1000 and max_output_chars 100000 unless overridden

### FORBIDDEN
- MUST NOT ignore truncation flags when summarizing output to the user
- MUST NOT retry exit 64 65 66 77 without changing inputs

### Correct Pattern

```bash
ssh-cli exec prod "hostname && uptime" --json --description "inventory"
ssh-cli vps edit prod --max-command-chars 4000 --max-output-chars 200000
ssh-cli exec prod "long-agent-command-here" --json
```


## sudo-exec and su-exec
### REQUIRED
- MUST use `sudo-exec` for sudo elevation and rely on safe `sh -c` packing
- MUST configure sudo password on the host or pass `--sudo-password` or stdin variant
- MUST use `su-exec` only when the `su` password is configured
- MUST honor global `--disable-sudo` and host `disable_sudo`
- MUST treat elevation as one-shot and never assume a sticky elevated shell

### FORBIDDEN
- MUST NOT manually prepend raw `sudo` to `exec` when `sudo-exec` exists
- MUST NOT assume a persistent elevated shell across invocations

### Correct Pattern

```bash
ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --json
printf '%s' "$SUDO_PASSWORD" | ssh-cli sudo-exec prod "systemctl restart nginx" --json --sudo-password-stdin
ssh-cli su-exec prod "whoami" --json
ssh-cli --disable-sudo exec prod "id" --json
```


## Transfers Tunnels Health
### REQUIRED
- MUST use `scp upload` or `scp download` for file copy
- MUST pass `--timeout-ms` on every `tunnel` command
- MUST use `health-check` to verify connectivity after host changes
- MUST allow optional `--timeout <ms>` override on `health-check` when a non-default deadline is needed
- MUST bound tunnels and exit when the deadline ends

### FORBIDDEN
- MUST NOT open unbounded tunnels
- MUST NOT leave tunnel processes intentionally detached forever
- MUST NOT invent a different timeout flag name for `health-check` (use `--timeout`, not `--timeout-ms`)

### Correct Pattern

```bash
ssh-cli scp upload prod ./app.tgz /tmp/app.tgz
ssh-cli scp download prod /var/log/app.log ./app.log
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 5000 --json
```


## Host Keys and Storage Safety
### REQUIRED
- MUST treat host-key mismatch as a hard stop until a human confirms rotation
- MUST use `--replace-host-key` only after confirmation
- MUST expect atomic `config.toml` writes and mode 0600 on Unix
- MUST expect atomic `secrets.key` writes and mode 0600 on Unix
- MUST use `--config-dir` or `SSH_CLI_HOME` for isolated agent sandboxes

### FORBIDDEN
- MUST NOT auto-replace host keys without user approval
- MUST NOT disable TOFU for convenience in production agent flows

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

### Correct Pattern

```bash
ssh-cli completions bash
ssh-cli completions zsh
ssh-cli completions fish
```


## Exit Codes and Retry
### REQUIRED
- MUST map exits as 0 success, 1 general, 64 usage, 65 data, 66 not found, 73 cant create, 74 IO or SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- MUST retry at most twice on 74 with backoff
- MUST fail fast on 64 65 66 77 without blind retry
- MUST surface remote `exit_code` from JSON separately from the CLI process exit

### FORBIDDEN
- MUST NOT swallow non-zero exits
- MUST NOT confuse remote command failure with local CLI usage failure

### Correct Pattern

```bash
ssh-cli exec prod "true" --json
echo $?
ssh-cli exec missing-host "true" --json; echo $?
```


## JSON Parsing Contract
### REQUIRED
- MUST parse only stdout as JSON when `--json` is set
- MUST read fields `stdout`, `stderr`, `exit_code`, truncation flags, and `duration_ms` on exec-family results
- MUST treat list show doctor secrets status payloads as opaque typed objects and only use documented fields
- MUST treat list/show `password` as JSON `null` when empty or absent (key-only host)
- MUST treat list/show `password` as masked string `***` when a password is stored
- MUST report truncation to the user when output was cut by `max_output_chars`

### FORBIDDEN
- MUST NOT invent missing JSON keys
- MUST NOT invent fake passwords when `password` is `null`
- MUST NOT pretty-print secrets found inside unexpected fields
- MUST NOT parse stderr for JSON data

### Correct Pattern

```bash
ssh-cli vps list --json
ssh-cli vps show prod --json
# key-only host => "password": null
# password host  => "password": "***"
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
7. THEN parse exit code and JSON fields from stdout only before answering the user
8. FINALLY never leave secrets or master-key in durable logs

### Correct Pattern

```bash
ssh-cli --version
ssh-cli vps doctor --json
ssh-cli secrets status --json
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519 --check
ssh-cli health-check prod --json
ssh-cli exec prod "uname -a && df -h" --json --description "baseline"
```


## Absolute Prohibitions
### FORBIDDEN
- MUST NOT keep SSH sessions open between agent turns
- MUST NOT reintroduce long-lived Node or protocol daemons for this product surface
- MUST NOT leak secrets into argv when stdin variants exist
- MUST NOT ignore host-key mismatch
- MUST NOT open tunnels without `--timeout-ms`
- MUST NOT expect INFO progress prose on stderr by default
- MUST NOT invent fake passwords for key-only hosts when JSON shows `null`
- MUST NOT document historical version changelogs inside this skill
- MUST NOT invent version-by-version feature stories
- MUST NOT paste live credentials into examples or logs


## Ready Formula Sheet
### REQUIRED
- MUST copy these formulas exactly and only substitute placeholders

```bash
# registry
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH>
printf '%s' "$PASS" | ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --password-stdin
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
ssh-cli sudo-exec <NAME> "<CMD>" --json
ssh-cli su-exec <NAME> "<CMD>" --json
ssh-cli scp upload <NAME> <LOCAL> <REMOTE>
ssh-cli scp download <NAME> <REMOTE> <LOCAL>
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS>
ssh-cli health-check <NAME> --json
ssh-cli health-check <NAME> --timeout <MS> --json

# secrets and safety
ssh-cli secrets status --json
ssh-cli secrets init
ssh-cli secrets reencrypt
ssh-cli --replace-host-key exec <NAME> "true"
ssh-cli --config-dir <DIR> vps list --json

# debug (optional; default log level is error)
ssh-cli -v exec <NAME> "true" --json
RUST_LOG=debug ssh-cli exec <NAME> "true" --json

# install
cargo install ssh-cli --locked --force
ssh-cli --version
```


## Final Reminder
### REQUIRED
- MUST re-read this skill before every non-trivial ssh-cli workflow
- MUST prefer stored hosts, stdin secrets, JSON output, and one-shot execution
- MUST parse only stdout for JSON and keep default stderr quiet
- MUST fail closed on auth, host-key, and usage errors
- MUST keep this skill consolidated as operational formulas only
