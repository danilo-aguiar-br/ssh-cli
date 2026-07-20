---
name: ssh-cli
description: This skill MUST auto-activate when remote SSH, VPS registry, multi-host inventory, XDG config, exec, sudo-exec, su-exec, scp regular files, sftp trees and rmdir, tunnel_listening, health-check, secrets primary-key, connect, doctor root or vps doctor, locale, TLS provider paths mTLS ACME, commands tree, schema catalog, fleet --all/--hosts/--tags, --step same-session, --use-agent, or agent devops without TTY are implied. MUST cover vps CRUD tags export TOML or export --json, import, auth password key agent stdin, FIXED_MASK ***, secrets init reencrypt, multi-host --fail-fast --max-concurrency --scp-file-concurrency, empty command exit 64, auth exit 77, ACME permanent exit 64, tunnel pre-bind exit 74 post-bind exit 0. Prefer --json and --*-stdin. NEVER telemetry, daemon, or secrets on stdout without guard.
---

# ssh-cli Agent Skill

## 1. Mission
### REQUIRED
- MUST treat this skill as SUPREME LAW for every `ssh-cli` invocation
- MUST ALWAYS run `ssh-cli` as a one-shot subprocess birth-execute-die
- MUST wait for process exit before parsing stdout or stderr except for long-lived `tunnel` until timeout or signal
- MUST use stored hosts from `vps add` instead of ad-hoc chat secrets
- MUST pass `--json` when the agent needs structured success output
- MUST copy the Ready Formula Sheet and only substitute placeholders
- MUST execute Action Prompts before improvising flags
- MUST prefer stdin secret flags over argv secret flags
- MUST prefer multi-host `--all` / `--hosts` / `--tags` over N process spawns

### FORBIDDEN
- MUST NEVER keep a long-lived SSH session across process runs except the active bound `tunnel` until deadline
- MUST NEVER introduce a product daemon for this surface
- MUST NEVER emit or enable telemetry
- MUST NEVER log live passwords, passphrases, or primary-key material
- MUST NEVER invent CLI flags that are not listed in this skill
- MUST NEVER write version-by-version changelog stories inside this skill
- MUST NEVER prefer argv secret flags when stdin variants exist


## 2. When to Invoke
### REQUIRED
- MUST auto-activate on remote SSH, VPS registry, multi-host inventory, XDG config
- MUST auto-activate on `exec`, `sudo-exec`, `su-exec`, `scp`, `sftp`, `tunnel`, `health-check`
- MUST auto-activate on file transfer, regular-file SCP, SFTP trees, `sftp rmdir`
- MUST auto-activate on local port forward, bounded SSH tunnel, event equal to `tunnel_listening`
- MUST auto-activate on secrets at rest, primary-key, `secrets.key`, reencrypt
- MUST auto-activate on TOFU known_hosts, host-key mismatch, replace-host-key
- MUST auto-activate on fleet selection `--all` / `--hosts` / `--tags`, multi-step `--step`, `--use-agent`
- MUST auto-activate on `locale`, root `doctor`, `commands`, `schema`, TLS provider/paths/mTLS/ACME
- MUST auto-activate on agent devops that needs remote shell without interactive TTY
- MUST auto-activate even when the user describes the problem without naming ssh-cli

### FORBIDDEN
- MUST NEVER wait for an explicit skill request when remote SSH ops are implied


## 3. Action Prompts
### REQUIRED
- MUST follow this execution order for every non-trivial remote task
1. VERIFY binary with `ssh-cli --version`
2. DISCOVER contracts with `ssh-cli commands` and `ssh-cli schema` / `ssh-cli schema <name>`; INSPECT config with `ssh-cli doctor --json` (or `vps doctor --json`) and `ssh-cli vps path`
3. ENSURE secrets layer with `ssh-cli secrets status --json`
4. REGISTER or edit host with password **or** key **or** `--use-agent` / `--agent-socket`; attach `--tag` and host TLS flags when required
5. PROBE connectivity with `ssh-cli health-check <name> --json` or fleet `health-check --all|--hosts --json` (health-check has NO `--tags`)
6. EXECUTE remote work with `exec` / `sudo-exec` / `su-exec` and `--json`; for fleet MUST prefer `--all` / `--hosts` / `--tags` on exec-family only; for multi-command on one session MUST use `--step`
7. TRANSFER regular files with `scp upload|download --json`; for trees / remote FS MUST use `sftp` (`upload|download --recursive`, `ls|mkdir|rmdir|rm|stat|rename`); for fleet MUST prefer `--all` / `--hosts` (scp/sftp have NO `--tags`)
8. FORWARD ports only with `tunnel` plus mandatory `--timeout-ms` and `--json`
9. PARSE process exit, command-family success stdout, or stderr error envelope
10. SANITIZE all durable logs so secrets and primary-key never remain

### FORBIDDEN
- MUST NEVER skip JSON parse after a non-zero exit in JSON mode
- MUST NEVER answer the user before reading the process exit code


## 4. Full Command Catalog
### REQUIRED
- MUST treat the following as the complete product root surface and use only these commands

| Root | Nested / form | One-liner purpose |
|------|---------------|-------------------|
| `vps add` | `--name --host --user` + auth | Register host |
| `vps list` | optional `--tag` filter | List hosts (secrets masked) |
| `vps remove` | `<NAME>` | Remove host |
| `vps edit` | `<NAME>` + field flags | Edit host fields / auth / TLS |
| `vps show` | `<NAME>` | Show one host (secrets masked) |
| `vps path` | | Print winning config.toml path |
| `vps doctor` | `--probe-ssh` `--hosts` | XDG / schema / optional SSH probe |
| `vps export` | `-o` `--include-secrets` `--json` | Export TOML body (or JSON envelope) |
| `vps import` | `--file` `--allow-incomplete` | Import TOML or JSON vps-export |
| `connect` | `<NAME>` | Write active host marker |
| `exec` | `<VPS> <CMD>` or fleet + `--step` | Remote shell capture |
| `sudo-exec` | same fleet + sudo password | Elevated via sudo packing |
| `su-exec` | same fleet + su password | Elevated via one-shot su |
| `scp upload` | multi-file, `--all`/`--hosts` | Regular-file upload only |
| `scp download` | multi-file, `--all`/`--hosts` | Regular-file download only |
| `sftp upload` | `--recursive`, fleet | SFTP upload file or tree |
| `sftp download` | `--recursive`, fleet | SFTP download file or tree |
| `sftp ls` | | List remote directory |
| `sftp mkdir` | | Create remote directory |
| `sftp rmdir` | | Remove empty remote directory |
| `sftp rm` | | Remove remote file |
| `sftp stat` | | Remote path metadata |
| `sftp rename` | | Rename remote path |
| `tunnel` | `<VPS> <local> <rhost> <rport> --timeout-ms` | Bounded local forward |
| `health-check` | name / `--all` / `--hosts` | SSH connectivity probe |
| `secrets status` | | Encryption status (no key material) |
| `secrets init` | `--force` `--keyring` | Create/store primary-key |
| `secrets reencrypt` | | Rewrite secrets under current key |
| `completions` | bash zsh fish elvish powershell | Shell completion scripts |
| `commands` | | JSON command tree discovery |
| `schema` | `[name]` | Schema catalog or one body |
| `doctor` | root alias of `vps doctor` | Same as vps doctor |
| `locale` | show / set / clear | UI language resolution |
| `tls provider` | | rustls CryptoProvider status |
| `tls paths` | | XDG TLS directory layout |
| `tls mtls list\|import\|show\|remove` | | mTLS identity store |
| `tls acme account create\|show` | | ACME account lifecycle |
| `tls acme issue\|complete\|status\|list` | | DNS-01 cert lifecycle |

### FORBIDDEN
- MUST NEVER invent root commands outside this catalog


## 5. Global Flags
### REQUIRED
- MUST document and use only these global flags
- `--lang <LOCALE>` — force BCP47 UI language (must negotiate to `en` or `pt-BR`)
- `-v` / `--verbose` — raise stderr log verbosity (ambient `RUST_LOG` is ignored)
- `-q` / `--quiet` — suppress non-JSON human prose
- `--config-dir <DIR>` — override config base directory (tests / sandboxes)
- `--no-color` — disable color
- `--output-format text|json` — global output format; if omitted, JSON when stdout is not a TTY (except `vps export` default body)
- `--json` — force JSON on stdout (alias of `--output-format json`)
- `--disable-sudo` — disable sudo-exec/su-exec for this invocation
- `--replace-host-key` — replace diverging TOFU host key (human approval first)
- `--allow-plaintext-secrets` — plaintext secrets at rest (tests only)
- `--secrets-key-file <PATH>` — 64-hex primary-key file for this one-shot
- `--use-keyring` — prefer OS keyring for primary-key
- `--timeout <MS>` — global default SSH op timeout (ms); local subcommand `--timeout` wins; tunnel still requires `--timeout-ms`
- `--max-concurrency <N>` — cap concurrent multi-host sessions / tunnel accepts (1..=64; auto = CPUs×4 vs free RAM formula when omitted)
- `--fail-fast` — stop admitting new multi-host units after first failure (default continues all)
- `--scp-file-concurrency <N>` — max concurrent SCP file transfers on one SSH session (default 1 serial)

### FORBIDDEN
- MUST NEVER treat `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` / `SSH_CLI_MAX_CONCURRENCY` as product config stores
- MUST NEVER treat `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` as stores (rejected fail-closed)
- MUST NEVER rely on ambient `RUST_LOG` to raise verbosity


## 6. Install Completions and Binary Check
### REQUIRED
- MUST verify the binary after install or upgrade before relying on scp, sftp, or tunnel
- MUST generate shell completions from the binary when onboarding humans
- MUST keep agent automation on explicit flags and JSON, not completion scripts
- MUST support shells bash, zsh, fish, elvish, and powershell

### Correct Pattern

```bash
cargo install ssh-cli --locked --force
ssh-cli --version
ssh-cli completions bash
ssh-cli completions zsh
ssh-cli completions fish
ssh-cli completions elvish
ssh-cli completions powershell
```


## 7. Lifecycle Contract
### REQUIRED
- MUST invoke one complete CLI process per product action
- MUST treat non-TTY stdout as JSON by default when `--output-format` is omitted for general commands
- MUST NOT claim auto JSON on non-TTY applies to `vps export` — export body stays TOML unless `vps export --json`
- MUST force JSON with `--json` or `--output-format json` for agent parsing on non-export commands
- MUST send human logs only to stderr and parse only stdout as success data
- MUST expect default log level `error` so stderr stays clean for agents
- MUST use `-v` only when debugging (ambient `RUST_LOG` is ignored)
- MUST use `-q` / `--quiet` to suppress non-JSON human prose when required
- MUST treat `scp --json`, `sftp --json`, `tunnel --json`, and global JSON format as activating stderr error envelopes on failure
- MUST parse failure envelopes from stderr JSON when process exit is non-zero and JSON mode is active
- MUST parse CRUD JSON success events `vps-added` `vps-edited` `vps-removed` `vps-connected` `vps-import` when JSON mode is active

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


## 8. Host Registry CRUD + Tags + Agent + Host TLS + Export/Import
### REQUIRED
- MUST register each host with a unique `--name`
- MUST supply password or `--key` or stdin password **or** `--use-agent` / `--agent-socket` on add (exactly one primary auth)
- MUST pass `--port` when the SSH port is not 22
- MUST pass `--check` on add when an immediate connectivity probe is required
- MUST attach fleet tags with repeatable `--tag <TAG>` on add; filter list with `--tag`
- MUST enable SSH-over-TLS on a host with `--tls` and optional `--tls-sni` `--tls-client-cert` `--tls-client-key`
- MUST edit TLS with `vps edit --tls` / `--no-tls` and the same cert flags
- MUST switch primary auth to agent with `vps edit --use-agent` (clears password/key when set)
- MUST mask secrets when showing list or show output to humans
- MUST treat empty or absent password in list/show JSON as JSON `null` (key-only host)
- MUST treat non-empty password in list/show JSON as masked `***` (`FIXED_MASK`) never raw
- MUST treat `sudo_password`, `su_password`, and `key_passphrase` the same way (`null` when absent, `***` when stored)
- MUST run `doctor --json` or `vps doctor --json` when config location is unknown
- MUST use `vps path` to print the winning config file path
- MUST parse doctor JSON as single root event equal to `vps-doctor` with nested `local.*` (`secrets_plaintext_opt_out` bool, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`) and `ssh_probe` (null or health-check-batch object)
- MUST treat `added_at` as present on list, show, and export output
- MUST allow import payloads to omit `added_at`; serde supplies the default
- MUST treat host/vps `--timeout` values as milliseconds; values under 1000 emit a stderr warning
- MUST treat `vps export` body as TOML by default even on pipe or non-TTY
- MUST use `vps export --json` only for the agent envelope with event equal to `vps-export`
- MUST use `vps export` without secrets by default
- MUST treat redacted `vps export` as never containing `sshcli-enc` ciphertext for cleared or empty secrets
- MUST treat empty secrets in redacted export as empty strings; non-empty redacted secrets as `***` (`FIXED_MASK`)
- MUST require human approval before `export --include-secrets`
- MUST NEVER pass `--include-secrets` to a pipe without `--output`/`-o` or `--i-understand-secrets-on-stdout`
- MUST accept import TOML EN keys plus PT aliases and JSON `vps-export` envelopes
- MUST use `--allow-incomplete` for redacted skeleton import when hosts lack full auth
- MUST treat invalid import TOML as exit `65`
- MUST parse field `secrets_key_auto_created` on the SAME single `vps-added` JSON document on first auto primary-key creation — NEVER a second NDJSON event

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
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519 --port 22 --tag prod --tag web --check
ssh-cli vps add --name edge --host edge.example.com --user deploy --use-agent --agent-socket "$SSH_AUTH_SOCK" --tag edge
ssh-cli vps add --name tls1 --host bastion.example.com --user deploy --key ~/.ssh/id_ed25519 --tls --tls-sni bastion.example.com --tls-client-cert /path/client.crt --tls-client-key /path/client.key
ssh-cli vps list --json
ssh-cli vps list --tag prod --json
ssh-cli vps show prod --json
ssh-cli vps edit prod --timeout 90000 --max-command-chars 2000 --max-output-chars 100000
ssh-cli vps edit prod --use-agent --agent-socket "$SSH_AUTH_SOCK"
ssh-cli vps edit prod --tls --tls-sni prod.example.com
ssh-cli vps edit prod --no-tls
ssh-cli doctor --json
ssh-cli vps doctor --json
ssh-cli vps path
ssh-cli vps export -o /tmp/hosts.toml
ssh-cli vps export --json
ssh-cli vps export --include-secrets -o /tmp/hosts-secrets.toml
ssh-cli vps import --file /tmp/hosts.toml
ssh-cli vps import --file /tmp/hosts-redacted.toml --allow-incomplete
ssh-cli vps remove prod
```


## 9. Connect
### REQUIRED
- MUST use `connect` only to write the sibling `active` marker
- MUST still pass explicit VPS name on exec-family commands when certainty is required
- MUST run nameless `health-check` only after `connect` set the active host

### Correct Pattern

```bash
ssh-cli connect prod
ssh-cli health-check --json
```


## 10. Authentication
### REQUIRED
- MUST use `--key` for key-only cloud hosts
- MUST use `--password-stdin` when argv history is shared
- MUST use `--sudo-password-stdin` and `--su-password-stdin` instead of argv secrets
- MUST use `--key-passphrase-stdin` when the private key is encrypted and argv must stay clean
- MUST treat `--key-passphrase <VAL>` as a valid argv override and MUST prefer stdin over argv
- MUST use `--use-agent` with `--agent-socket` on Unix for ssh-agent auth (CLI/XDG only — not env store)
- MUST expect password-like values on argv to emit a stderr warning; MUST prefer `--password-stdin` `--key-passphrase-stdin` `--sudo-password-stdin` `--su-password-stdin`
- MUST treat exit 77 as authentication failure and change credentials before retry
- MUST expect list/show JSON `password` to be `null` for key-only hosts and `***` when a password is stored
- MUST apply auth overrides on `exec`, `sudo-exec`, `su-exec`, `scp`, `sftp`, `tunnel`, and `health-check` when stored host credentials are insufficient
- MUST support runtime overrides `--password` / `--password-stdin` / `--key` / `--key-passphrase` / `--key-passphrase-stdin` / `--use-agent` / `--agent-socket`

### FORBIDDEN
- MUST NEVER invent fake passwords for key-only hosts
- MUST NEVER treat JSON `null` password as a bug or as a missing field to fabricate
- MUST NEVER print key passphrases or SSH passwords
- MUST NEVER store secrets in shell history when stdin is available

### Correct Pattern

```bash
printf '%s' "$SSH_PASSWORD" | ssh-cli vps add --name app --host app.example.com --user deploy --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli exec edge "id" --json --key ~/.ssh/id_ed25519_enc --key-passphrase-stdin
ssh-cli exec edge "id" --json --use-agent --agent-socket "$SSH_AUTH_SOCK"
```


## 11. Secrets at Rest
### REQUIRED
- MUST treat at-rest encryption as the default product behavior
- MUST use product term primary-key for the at-rest encryption key
- MUST accept legacy keyring user alias `secrets-master-key` as read-only legacy accept alongside canonical `secrets-primary-key`
- MUST prefer CLI flags `--allow-plaintext-secrets` `--secrets-key-file` `--use-keyring` (no env stores)
- MUST run `secrets status --json` before diagnosing decrypt failures
- MUST run `secrets init` when an explicit primary-key file or keyring entry is required
- MUST run `secrets init --json` when the agent needs the `secrets-init` success envelope
- MUST run `secrets init --keyring --json` when the primary-key MUST be stored in the OS keyring instead of `secrets.key`
- MUST run `secrets init --force --json` only when intentionally rotating and rewriting secrets under a new key
- MUST run `secrets reencrypt` after rotating the primary-key material
- MUST run `secrets reencrypt --json` when the agent needs the `secrets-reencrypt` success envelope
- MUST parse JSON events `secrets-init` / `secrets-reencrypt`; on first `vps add` with auto-key parse ONE `vps-added` document with field `secrets_key_auto_created` (never a second event)
- MUST keep plaintext secrets restricted to automated tests only via `--allow-plaintext-secrets`
- MUST NEVER print primary-key material or key file contents
- MUST resolve primary-key in this order only
  1. CLI `--secrets-key-file` as path to 64 hex chars
  2. OS keyring when `--use-keyring` (read accepts `secrets-primary-key` then legacy `secrets-master-key`)
  3. XDG or config-dir `secrets.key` auto-created on first secret write; field `secrets_key_auto_created` on the same `vps-added` JSON
- MUST treat `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` as **rejected fail-closed** (not a store)
- MUST use `--config-dir` to override the base config directory in tests (not `SSH_CLI_HOME`)
- MUST use `--lang` or `locale set` to force locale (not `SSH_CLI_LANG`)
- MUST use `-v` only when debugging; ambient `RUST_LOG` is ignored; default remains error-level

### FORBIDDEN
- MUST NEVER log primary-key material, key file contents, or decrypted host secrets
- MUST NEVER print key material from `secrets init` or `secrets reencrypt`
- MUST NEVER enable plaintext secrets in production agent flows
- MUST NEVER treat `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` / `SSH_CLI_MAX_CONCURRENCY` as product config stores

### Correct Pattern

```bash
ssh-cli secrets status --json
ssh-cli secrets init --json
ssh-cli secrets reencrypt --json
ssh-cli --secrets-key-file /tmp/primary.key secrets status --json
ssh-cli --config-dir /tmp/ssh-cli-test doctor --json
```


## 12. Multi-host Fleet
### REQUIRED
- MUST prefer fleet flags over N single-host CLI spawns (one process, concurrent SSH sessions with admission gate)
- MUST use `--all` / `--hosts` on `exec` / `sudo-exec` / `su-exec` / `scp` / `sftp` / `health-check`
- MUST use `--tags` ONLY on `exec` / `sudo-exec` / `su-exec` for OR tag selection (health-check, scp, and sftp have NO `--tags`)
- MUST use `--hosts` when only a subset of the registry is needed (batch JSON even if one name)
- MUST parse multi-host JSON via batch schemas `health-check-batch` / `exec-batch` / `scp-batch` / `sftp-batch`; envelope includes `max_concurrency`
- MUST treat global `--max-concurrency N` (1..=64) as the fan-out and tunnel-accept cap (auto when omitted; CLI-only)
- MUST use `--fail-fast` only when the agent MUST stop admitting new hosts after the first failure
- MUST treat empty registry + `--all` / `--hosts` (or exec-family `--tags`) as usage exit 64
- MUST treat `tunnel` as single-host only; multi-host tunnels = N one-shots
- MUST use `doctor --probe-ssh [--hosts a,b] --json` for local diagnostics + multi-host health in one root event equal to `vps-doctor` (`ssh_probe` field)
- MUST use single-host multi-file SCP `scp upload <VPS> f1 f2 … <REMOTE_DIR>` (one session; serial files by default)
- MUST use multi-host × multi-file `scp upload --all f1 f2 … <REMOTE_DIR>` or `--hosts a,b`
- MUST raise `--scp-file-concurrency N` only when parallel SCP channels on one session are required
- MUST keep single-host forms when the target is one explicit VPS name (classic JSON)

### FORBIDDEN
- MUST NEVER assume sequential multi-host is the default when fleet flags are available
- MUST NEVER invent `--tags` on `health-check`, `scp`, or `sftp`
- MUST NEVER spawn one process per host for fleet work when fleet flags can cover the set

### Correct Pattern

```bash
ssh-cli --max-concurrency 8 health-check --all --json
ssh-cli health-check --hosts web1,web2 --json
ssh-cli exec --all 'uptime' --json
ssh-cli exec --hosts web1,web2 'uptime' --json
ssh-cli exec --tags prod,web 'uptime' --json
ssh-cli --fail-fast exec --all 'true' --json
ssh-cli scp upload --all ./a.bin /tmp/a.bin --json
ssh-cli scp upload --hosts web1,web2 ./a.bin /tmp/a.bin --json
ssh-cli scp download --all /tmp/a.bin ./a --json
ssh-cli scp upload prod ./a.bin ./b.bin /tmp/ --json
ssh-cli --scp-file-concurrency 4 scp upload prod ./a.bin ./b.bin /tmp/ --json
ssh-cli doctor --probe-ssh --json
ssh-cli doctor --probe-ssh --hosts web1,web2 --json
```


## 13. Remote Execution + --step
### REQUIRED
- MUST validate command length against `max_command_chars` before sending huge agent commands
- MUST treat an empty remote command string as hard failure with technical message exactly `empty command` (English always) and process exit 64
- MUST parse `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr`, and `duration_ms` from success JSON
- MUST append `--description` when remote shell history benefits from an audit comment
- MUST raise host `max_command_chars` via `vps edit` when the agent needs longer commands
- MUST honor default max_command_chars 1000 and max_output_chars 100000 unless overridden
- MUST pass exec-family `--timeout <ms>` when the host default deadline is too short
- MUST prefer `exec --all` / `--hosts` / `--tags` for multi-host fleet
- MUST use `--step <CMD>` (repeatable) to run extra commands on the **same** SSH session after the primary
- MUST parse multi-step JSON as one stdout object per step with fields `step` (0-based index), `command`, plus exec fields
- MUST apply auth overrides and fleet flags on `exec` the same as other SSH ops

### FORBIDDEN
- MUST NEVER ignore `truncated_stdout` or `truncated_stderr` when summarizing output to the user
- MUST NEVER retry exit 64 65 66 77 without changing inputs
- MUST NEVER send an empty remote command string
- MUST NEVER open a new process per step when `--step` can chain on one session

### Correct Pattern

```bash
ssh-cli exec prod "hostname && uptime" --json --description "inventory"
ssh-cli exec prod "true" --json --timeout 120000
ssh-cli exec --all 'hostname && uptime' --json
ssh-cli exec prod "uname -a" --step "df -h" --step "free -m" --json
# multi-step => one JSON line per step with "step":0|1|2 and "command"
```


## 14. sudo-exec and su-exec
### REQUIRED
- MUST use `sudo-exec` for sudo elevation and rely on safe `sh -c` packing
- MUST configure sudo password on the host or pass `--sudo-password` or stdin variant
- MUST use `su-exec` only when the `su` password is configured
- MUST honor global `--disable-sudo` and host `disable_sudo`
- MUST treat elevation as one-shot and NEVER assume a sticky elevated shell
- MUST support fleet flags `--all` / `--hosts` / `--tags`, `--step`, auth overrides, `--timeout`, `--description` on both elevation commands
- MUST prefer `--sudo-password-stdin` / `--su-password-stdin` over argv

### FORBIDDEN
- MUST NEVER manually prepend raw `sudo` to `exec` when `sudo-exec` exists
- MUST NEVER assume a persistent elevated shell across invocations

### Correct Pattern

```bash
ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --json
printf '%s' "$SUDO_PASSWORD" | ssh-cli sudo-exec prod "systemctl restart nginx" --json --sudo-password-stdin
ssh-cli su-exec prod "whoami" --json
printf '%s' "$SU_PASSWORD" | ssh-cli su-exec prod "whoami" --json --su-password-stdin
ssh-cli sudo-exec --all "systemctl status nginx" --json
ssh-cli sudo-exec prod "id" --step "whoami" --json
```


## 15. SCP
### REQUIRED
- MUST use `scp upload` or `scp download` for regular-file copy only
- MUST pass `--json` on every agent-parsed transfer
- MUST parse scp success only from stdout with fields `ok`, event equal to `scp-transfer`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`
- MUST treat `ok` as true and `direction` as `upload` or `download` only
- MUST use argument order `upload <vps> <local> <remote>` and `download <vps> <remote> <local>`
- MUST support multi-file single-host and fleet forms with `--all` / `--hosts`
- MUST pass `--timeout <ms>` on scp when connect-plus-transfer needs a longer deadline
- MUST use `--password-stdin` and `--key-passphrase-stdin` on scp whenever secrets would otherwise appear on argv
- MUST use `--key` / `--use-agent` overrides on scp the same way as exec when stored credentials are insufficient
- MUST expect upload to stream in 32 KiB chunks without full-file RAM load
- MUST expect download to write sibling path ending in `.ssh-cli.partial` then rename into place
- MUST expect mtime and mode preserve both directions without an extra user flag
- MUST parse scp hard failures from stderr error envelope when JSON mode is active
- MUST treat remote missing SCP as exit `66` with message `file not found: <path>`
- MUST use global `--scp-file-concurrency` when parallel multi-file on one session is required

### FORBIDDEN
- MUST NEVER pass directories as local or remote scp paths
- MUST NEVER invent recursive flags such as `-r` on scp (use `sftp --recursive` for trees)
- MUST NEVER treat scp as an SFTP subsystem
- MUST NEVER use `--timeout-ms` on scp (that flag is tunnel-only)
- MUST NEVER parse scp success as exec-family `stdout`/`stderr`/`exit_code` JSON
- MUST NEVER treat a leftover `.ssh-cli.partial` path as the final success artifact after a completed download
- MUST NEVER invent a required user-facing preserve flag for mtime or mode
- MUST NEVER omit the event field when documenting or parsing scp success JSON
- MUST NEVER treat remote missing SCP as exit `74` when exit is `66`

### Correct Pattern

```bash
ssh-cli scp upload prod ./app.tgz /tmp/app.tgz --json
ssh-cli scp download prod /var/log/app.log ./app.log --json
ssh-cli scp upload prod ./a.bin ./b.bin /tmp/ --json
ssh-cli scp upload --all ./a.bin /tmp/a.bin --json
# success event MUST equal scp-transfer
```


## 16. SFTP
### REQUIRED
- MUST use `sftp upload|download` for regular files or trees (`--recursive`) and `sftp ls|mkdir|rmdir|rm|stat|rename` for remote FS ops
- MUST pass `--json` on every agent-parsed SFTP transfer or FS op
- MUST parse transfer success with event equal to `sftp-transfer` (fields include `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`, `recursive`)
- MUST parse list success with event equal to `sftp-list`; FS ops with event equal to `sftp-fs-op`; fleet with event equal to `sftp-batch`
- MUST treat recursive walks as **no symlink follow** (fail-closed)
- MUST expect stream I/O in 32 KiB chunks (never full-file heap) and download partial + atomic rename
- MUST pass `--timeout <ms>` when connect-plus-op needs a longer wall-clock deadline
- MUST prefer `sftp upload|download --all` / `--hosts` for fleet (bounded concurrency)
- MUST use `sftp rmdir` only for empty remote directories

### FORBIDDEN
- MUST NEVER invent SFTP REPL / interactive shell
- MUST NEVER follow remote or local symlinks during recursive tree transfer
- MUST NEVER treat SFTP success JSON as exec-family fields
- MUST NEVER use scp for directory trees when sftp --recursive exists

### Correct Pattern

```bash
ssh-cli sftp upload prod ./app.tgz /tmp/app.tgz --json
ssh-cli sftp upload --recursive prod ./dist /tmp/dist --json
ssh-cli sftp download --recursive prod /tmp/dist ./dist --json
ssh-cli sftp ls prod /var/log --json
ssh-cli sftp mkdir prod /tmp/newdir --json
ssh-cli sftp rmdir prod /tmp/newdir --json
ssh-cli sftp rm prod /tmp/app.tgz --json
ssh-cli sftp stat prod /tmp/app.tgz --json
ssh-cli sftp rename prod /tmp/a /tmp/b --json
ssh-cli sftp upload --all ./a.bin /tmp/a.bin --json
```


## 17. Tunnel
### REQUIRED
- MUST pass `--timeout-ms` on every `tunnel` command
- MUST pass `--bind` consciously when non-loopback bind is required; default is `127.0.0.1`
- MUST NEVER expose `0.0.0.0` without an explicit security decision
- MUST treat local port argument `0` as ephemeral OS-assigned port; after bind, trust JSON `local_port` (>=1), never connect to port 0
- MUST NEVER invent a `--local-port` flag; tunnel args are positional `tunnel <vps> <local_port> <remote_host> <remote_port>`
- MUST pass `--json` when the agent needs a structured ready signal
- MUST wait for one stdout object with event equal to `tunnel_listening` before using the local port
- MUST parse tunnel ready fields `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`
- MUST leave the tunnel process running until `--timeout-ms` deadline or signal
- MUST treat tunnel post-bind deadline as success exit `0` after `tunnel_listening`
- MUST treat tunnel pre-bind timeout as exit `74`
- MUST parse tunnel hard failures from stderr error envelope when JSON mode is active
- MUST use tunnel auth overrides `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--use-agent`, `--agent-socket` when stored host credentials are insufficient
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
# wait for event equal to tunnel_listening
# post-bind deadline exits 0; pre-bind timeout exits 74
```


## 18. Health-check
### REQUIRED
- MUST use `health-check` to verify connectivity after host changes
- MUST pass `--timeout <ms>` on `health-check` when a non-default deadline is needed
- MUST use health-check auth overrides `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--use-agent`, `--agent-socket` when stored host credentials are insufficient
- MUST prefer `--key-passphrase-stdin` over `--key-passphrase` whenever stdin is available
- MUST use `health-check --replace-host-key` only after human confirmation of host-key rotation
- MUST parse health-check hard failures from stderr error envelope when JSON mode is active
- MUST NEVER use `--timeout-ms` on health-check
- MUST prefer `health-check --all --json` when probing the entire registry (batch schema `health-check-batch`)
- MUST support `--hosts` for subset probes

### FORBIDDEN
- MUST NEVER claim health-check lacks password-stdin or key overrides
- MUST NEVER auto-pass `--replace-host-key` without human approval

### Correct Pattern

```bash
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 5000 --json
ssh-cli --max-concurrency 8 health-check --all --json
ssh-cli health-check --hosts web1,web2 --json
# only after human review of host-key mismatch
ssh-cli health-check prod --json --replace-host-key
```


## 19. Locale
### REQUIRED
- MUST use `locale` (default `show`) to diagnose resolved language, winning layer, and available locales
- MUST use `locale set <LOCALE>` to persist preferred language under config dir (`lang` file, mode 0o600)
- MUST use `locale clear` to remove the persisted preference
- MUST treat precedence as CLI `--lang` > XDG `lang` file (`locale set`) > system > `en`
- MUST force one-shot language with `--lang` without writing the preference file
- MUST negotiate BCP47 tags to product locales `en` or `pt-BR`

### FORBIDDEN
- MUST NEVER treat `SSH_CLI_LANG` as a product store

### Correct Pattern

```bash
ssh-cli locale --json
ssh-cli locale show --json
ssh-cli locale set pt-BR
ssh-cli locale clear
ssh-cli --lang en vps list --json
```


## 20. TLS stack
### REQUIRED
- MUST use `tls provider` to show rustls CryptoProvider status (`aws_lc_rs`)
- MUST use `tls paths` to print XDG TLS directory layout paths
- MUST manage mTLS identities under XDG `tls/mtls/` with
  - `tls mtls list`
  - `tls mtls import --name <NAME> --cert <PATH> --key <PATH>`
  - `tls mtls show <NAME>`
  - `tls mtls remove <NAME>`
- MUST manage ACME (Let's Encrypt) account + DNS-01 cert lifecycle with
  - `tls acme account create --contact mailto:ops@example.com [--staging] [--force]`
  - `tls acme account show`
  - `tls acme issue --domain <DOMAIN> --print-challenge [--staging]`
  - `tls acme complete --domain <DOMAIN>`
  - `tls acme status [--domain <DOMAIN>]`
  - `tls acme list`
- MUST treat ACME validation failures (`invalidContact`, other permanent 4xx problem types) as permanent exit **64** and MUST NEVER retry them as exit 74
- MUST treat transient ACME failures (e.g. rateLimited, timeout) as retryable IO class when the product maps them to exit 74
- MUST attach host SSH-over-TLS via `vps add|edit --tls` plus optional `--tls-sni` `--tls-client-cert` `--tls-client-key`
- MUST pass `--json` on agent-parsed TLS ops

### FORBIDDEN
- MUST NEVER retry ACME permanent validation failures (exit 64) as if they were exit 74
- MUST NEVER invent interactive ACME wait loops when `--print-challenge` two-step flow exists

### Correct Pattern

```bash
ssh-cli tls provider --json
ssh-cli tls paths --json
ssh-cli tls mtls import --name client1 --cert /path/client.crt --key /path/client.key --json
ssh-cli tls mtls list --json
ssh-cli tls mtls show client1 --json
ssh-cli tls mtls remove client1 --json
ssh-cli tls acme account create --contact mailto:ops@example.com --json
ssh-cli tls acme account show --json
ssh-cli tls acme issue --domain app.example.com --print-challenge --json
# publish DNS TXT, then
ssh-cli tls acme complete --domain app.example.com --json
ssh-cli tls acme status --domain app.example.com --json
ssh-cli tls acme list --json
```


## 21. Discovery (commands + schema)
### REQUIRED
- MUST run `ssh-cli commands` to emit the full command tree as JSON for agent discovery
- MUST run `ssh-cli schema` to list the embedded schema catalog (event equal to `schema-catalog`)
- MUST run `ssh-cli schema <name>` to emit one schema body (examples `exec`, `exec-batch`, `scp-transfer`, `error-envelope`, `sftp-list`)
- MUST use discovery before improvising unknown flag shapes

### Correct Pattern

```bash
ssh-cli commands
ssh-cli schema
ssh-cli schema exec
ssh-cli schema error-envelope
ssh-cli schema scp-transfer
```


## 22. Timeout Host Keys Storage
### REQUIRED
- MUST pass `--timeout-ms` only on `tunnel` and ALWAYS as mandatory
- MUST pass `--timeout` on `scp`, `sftp`, exec-family, and `health-check` when overriding deadlines
- MUST treat all host and VPS timeout values as milliseconds not seconds
- MUST expect stderr warning when a host/vps timeout value is under 1000 ms
- MUST NEVER interchange `--timeout` and `--timeout-ms` across subcommands
- MUST NEVER set host timeout to values under 1000 unless the sub-second deadline is intentional
- MUST treat host-key mismatch as a hard stop until a human confirms rotation
- MUST use `--replace-host-key` only after confirmation
- MUST expect atomic `config.toml` and `secrets.key` writes and mode 0600 on Unix
- MUST use `--config-dir` for isolated agent sandboxes (product does not read `SSH_CLI_HOME`)

### FORBIDDEN
- MUST NEVER auto-replace host keys without user approval
- MUST NEVER disable TOFU for convenience in production agent flows


## 23. Exit Codes Retry
### REQUIRED
- MUST map exits as 0 success, 1 general, 64 usage, 65 data, 66 not found, 73 cant create, 74 IO or SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- MUST treat empty remote command as exit `64` with message `empty command`
- MUST treat invalid import TOML as exit `65`
- MUST treat remote missing SCP as exit `66` with message `file not found: <path>`
- MUST treat auth failure as exit `77`
- MUST treat ACME permanent validation failures (`invalidContact`, other permanent 4xx problem types) as exit `64` and MUST NEVER retry as 74
- MUST treat tunnel post-bind deadline as exit 0 after `tunnel_listening`
- MUST treat tunnel pre-bind timeout as exit 74
- MUST retry at most twice on 74 with backoff when envelope `retryable` is true
- MUST fail fast on 64 65 66 77 without blind retry
- MUST surface remote `exit_code` from success JSON separately from the CLI process exit
- MUST surface `remote_exit_code` from the stderr error envelope when present
- MUST read envelope fields `exit_code`, `message`, `remote_exit_code`, `retryable`, `error_class`, `suggestion` when present

### FORBIDDEN
- MUST NEVER swallow non-zero exits
- MUST NEVER confuse remote command failure with local CLI usage failure
- MUST NEVER retry post-bind tunnel exit 0 as if it were a failure
- MUST NEVER retry ACME permanent exit 64 as transient IO


## 24. JSON Parsing Contract
### REQUIRED
- MUST parse only stdout as success JSON when JSON mode is active and exit is success-path
- MUST read exec-family fields `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr`, `duration_ms`
- MUST read multi-step exec as one object per step with `step` index and `command`
- MUST read scp success fields `ok`, event equal to `scp-transfer`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`
- MUST read sftp transfer event equal to `sftp-transfer`; list `sftp-list`; fs-op `sftp-fs-op`; batch `sftp-batch`
- MUST read tunnel ready fields `ok`, event equal to `tunnel_listening`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`
- MUST parse stderr error envelope fields `exit_code`, `message`, `remote_exit_code`, `retryable`, `error_class`, `suggestion` when present on hard failures in JSON mode
- MUST treat list show doctor secrets status payloads as opaque typed objects and only use documented fields
- MUST treat list/show `password` as JSON `null` when empty or absent and as `***` (`FIXED_MASK`) when stored
- MUST treat list/show `sudo_password`, `su_password`, and `key_passphrase` as `null` or `***` the same way
- MUST treat list/show/export `added_at` as present; MUST allow import to omit `added_at` and serde supplies default
- MUST parse doctor JSON as single root event equal to `vps-doctor` with `local.secrets_plaintext_opt_out` boolean plus other `local.*` secret fields; `ssh_probe` may be null
- MUST parse first auto-key `vps-added` as ONE document with boolean `secrets_key_auto_created` (never a second NDJSON event)
- MUST report truncation to the user when `truncated_stdout` or `truncated_stderr` is true

### FORBIDDEN
- MUST NEVER invent missing JSON keys
- MUST NEVER invent fake passwords when `password` is `null`
- MUST NEVER pretty-print secrets found inside unexpected fields
- MUST NEVER parse stderr for success JSON data
- MUST NEVER parse scp success as exec-family fields
- MUST NEVER parse tunnel ready as exec-family fields
- MUST NEVER parse scp success without requiring event equal to `scp-transfer`
- MUST NEVER treat doctor `secrets_plaintext_opt_out` as a string
- MUST NEVER expect a second `secrets-key-auto-created` event document


## 25. Ready Formula Sheet
### REQUIRED
- MUST copy these formulas exactly and only substitute placeholders

```bash
# binary + discovery
ssh-cli --version
ssh-cli commands
ssh-cli schema
ssh-cli schema <NAME>
ssh-cli completions bash

# registry
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --port <PORT> --tag <TAG> --check
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --use-agent --agent-socket <SOCK> --tag <TAG>
printf '%s' "$PASS" | ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --password-stdin
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --tls --tls-sni <SNI> --tls-client-cert <CERT> --tls-client-key <KEY>
printf '%s' "$SUDO" | ssh-cli vps edit <NAME> --sudo-password-stdin
ssh-cli vps edit <NAME> --timeout <MS> --max-command-chars <N> --max-output-chars <N>
ssh-cli vps edit <NAME> --use-agent --agent-socket <SOCK>
ssh-cli vps edit <NAME> --tls --tls-sni <SNI>
ssh-cli vps edit <NAME> --no-tls
ssh-cli vps list --json
ssh-cli vps list --tag <TAG> --json
ssh-cli vps show <NAME> --json
ssh-cli doctor --json
ssh-cli vps doctor --json
ssh-cli doctor --probe-ssh --json
ssh-cli doctor --probe-ssh --hosts <A>,<B> --json
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
ssh-cli connect <NAME>
ssh-cli vps remove <NAME>

# remote ops
ssh-cli exec <NAME> "<CMD>" --json
ssh-cli exec <NAME> "<CMD>" --json --timeout <MS> --description "<AUDIT>"
ssh-cli -q exec <NAME> "<CMD>" --json
ssh-cli exec <NAME> "<CMD>" --step "<CMD2>" --step "<CMD3>" --json
ssh-cli sudo-exec <NAME> "<CMD>" --json
printf '%s' "$SUDO" | ssh-cli sudo-exec <NAME> "<CMD>" --json --sudo-password-stdin
ssh-cli su-exec <NAME> "<CMD>" --json
printf '%s' "$SU" | ssh-cli su-exec <NAME> "<CMD>" --json --su-password-stdin
# empty remote command => message "empty command" and exit 64 (English always)
# fleet multi-host (bounded concurrent; prefer over N single-host spawns)
ssh-cli --max-concurrency <N> exec --all "<CMD>" --json
ssh-cli exec --hosts <A>,<B> "<CMD>" --json
ssh-cli exec --tags <TAG1>,<TAG2> "<CMD>" --json
ssh-cli --fail-fast exec --all "<CMD>" --json
ssh-cli sudo-exec --all "<CMD>" --json
ssh-cli su-exec --all "<CMD>" --json
ssh-cli sudo-exec --tags <TAG> "<CMD>" --json

# scp transfers (regular files only; agent MUST use --json; event MUST equal scp-transfer)
ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json
ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json
ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --timeout <MS>
ssh-cli scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json
ssh-cli --scp-file-concurrency <N> scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json
ssh-cli scp upload --all <F1> <F2> <REMOTE_DIR> --json
ssh-cli scp download <NAME> <R1> <R2> <LOCAL_DIR> --json
printf '%s' "$PASS" | ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --key <KEY_PATH> --key-passphrase-stdin
ssh-cli scp upload --all <LOCAL_FILE> <REMOTE_FILE> --json
ssh-cli scp download --all <REMOTE_FILE> <LOCAL_PREFIX> --json
ssh-cli scp upload --hosts <A>,<B> <LOCAL_FILE> <REMOTE_FILE> --json

# sftp (trees + FS ops; event sftp-transfer / sftp-list / sftp-fs-op / sftp-batch)
ssh-cli sftp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json
ssh-cli sftp upload --recursive <NAME> <LOCAL_DIR> <REMOTE_DIR> --json
ssh-cli sftp download --recursive <NAME> <REMOTE_DIR> <LOCAL_DIR> --json
ssh-cli sftp ls <NAME> <REMOTE_DIR> --json
ssh-cli sftp mkdir <NAME> <REMOTE_DIR> --json
ssh-cli sftp rmdir <NAME> <REMOTE_DIR> --json
ssh-cli sftp rm <NAME> <REMOTE_FILE> --json
ssh-cli sftp stat <NAME> <REMOTE_PATH> --json
ssh-cli sftp rename <NAME> <FROM> <TO> --json
ssh-cli sftp upload --all <LOCAL_FILE> <REMOTE_FILE> --json
ssh-cli sftp upload --hosts <A>,<B> <LOCAL_FILE> <REMOTE_FILE> --json

# tunnel (mandatory --timeout-ms; --bind default 127.0.0.1; wait for tunnel_listening; post-bind deadline exit 0)
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --bind 127.0.0.1
printf '%s' "$PASS" | ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --password-stdin
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --key <KEY_PATH>
printf '%s' "$KEY_PASS" | ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --key <KEY_PATH> --key-passphrase-stdin
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --use-agent --agent-socket <SOCK>

# health
ssh-cli health-check <NAME> --json
ssh-cli health-check <NAME> --timeout <MS> --json
ssh-cli health-check --json
ssh-cli --max-concurrency <N> health-check --all --json
ssh-cli health-check --hosts <A>,<B> --json
printf '%s' "$PASS" | ssh-cli health-check <NAME> --json --password-stdin
ssh-cli health-check <NAME> --json --key <KEY_PATH>
printf '%s' "$KEY_PASS" | ssh-cli health-check <NAME> --json --key <KEY_PATH> --key-passphrase-stdin
ssh-cli health-check <NAME> --json --use-agent --agent-socket <SOCK>
ssh-cli health-check <NAME> --json --replace-host-key

# secrets and safety (prefer CLI flags over env; product term primary-key)
ssh-cli secrets status --json
ssh-cli secrets init
ssh-cli secrets init --json
ssh-cli secrets init --force --json
ssh-cli secrets init --keyring --json
ssh-cli secrets reencrypt
ssh-cli secrets reencrypt --json
ssh-cli --allow-plaintext-secrets --config-dir <DIR> secrets status --json
ssh-cli --secrets-key-file <KEY_FILE> secrets status --json
ssh-cli --use-keyring secrets status --json
ssh-cli --replace-host-key exec <NAME> "true"
ssh-cli --config-dir <DIR> vps list --json
printf '%s' "$KEY_PASS" | ssh-cli exec <NAME> "id" --json --key <KEY_PATH> --key-passphrase-stdin
ssh-cli exec <NAME> "id" --json --use-agent --agent-socket <SOCK>
# prefer stdin secrets; password-like argv emits stderr warning

# locale
ssh-cli locale --json
ssh-cli locale show --json
ssh-cli locale set <LOCALE>
ssh-cli locale clear
ssh-cli --lang <LOCALE> vps list --json

# TLS / mTLS / ACME
ssh-cli tls provider --json
ssh-cli tls paths --json
ssh-cli tls mtls list --json
ssh-cli tls mtls import --name <NAME> --cert <CERT> --key <KEY> --json
ssh-cli tls mtls show <NAME> --json
ssh-cli tls mtls remove <NAME> --json
ssh-cli tls acme account create --contact mailto:<EMAIL> --json
ssh-cli tls acme account create --contact mailto:<EMAIL> --staging --force --json
ssh-cli tls acme account show --json
ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --json
ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --staging --json
ssh-cli tls acme complete --domain <DOMAIN> --json
ssh-cli tls acme status --json
ssh-cli tls acme status --domain <DOMAIN> --json
ssh-cli tls acme list --json
# ACME invalidContact / permanent 4xx => exit 64 NEVER retry as 74

# debug only when diagnosing; default log level is error
ssh-cli -v exec <NAME> "true" --json

# install
cargo install ssh-cli --locked --force
ssh-cli --version
```


## 26. Absolute Prohibitions
### FORBIDDEN
- MUST NEVER keep SSH sessions open between agent turns except an active bound tunnel until deadline
- MUST NEVER reintroduce long-lived product daemons for this surface
- MUST NEVER leak secrets into argv when stdin variants exist
- MUST NEVER prefer `--key-passphrase` argv when `--key-passphrase-stdin` is available
- MUST NEVER ignore host-key mismatch
- MUST NEVER open tunnels without `--timeout-ms`
- MUST NEVER use the tunnel local port before `tunnel_listening` when JSON mode is on
- MUST NEVER scp directories; for trees MUST use `sftp --recursive` (no symlink follow)
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
- MUST NEVER retry ACME permanent validation failures (exit 64) as exit 74
- MUST NEVER treat `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` as stores
- MUST NEVER treat `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` / `SSH_CLI_MAX_CONCURRENCY` as product env stores
- MUST NEVER emit telemetry
- MUST NEVER invent flags outside this skill

### REQUIRED
- MUST re-read this skill before every non-trivial ssh-cli workflow
- MUST use stored hosts, stdin secrets, JSON output, and one-shot execution
- MUST parse only stdout for success JSON and stderr error envelopes on hard failures
- MUST wait for `tunnel_listening` before using a tunnel local port
- MUST treat post-bind tunnel deadline as exit 0 and pre-bind timeout as exit 74
- MUST treat empty command as exit 64, remote missing SCP as exit 66, invalid import TOML as exit 65, auth as exit 77, ACME permanent as exit 64
- MUST treat `vps export` body as TOML unless `vps export --json`
- MUST parse doctor event equal to `vps-doctor` with boolean `secrets_plaintext_opt_out` and treat `added_at` as optional only on import
- MUST treat timeouts as milliseconds and expect under-1000 warning on host/vps timeout
- MUST prefer fleet flags and `--step` over N process spawns
- MUST fail closed on auth, host-key, and usage errors
