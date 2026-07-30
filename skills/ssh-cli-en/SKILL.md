---
name: ssh-cli
description: This skill MUST auto-activate when remote SSH, VPS registry, multi-host inventory, XDG config, exec, sudo-exec, su-exec, scp regular files, sftp trees and rmdir, tunnel_listening, health-check, secrets primary-key, connect, doctor root or vps doctor, locale, TLS provider paths mTLS ACME, commands tree, schema catalog, fleet --all/--hosts/--tags, --step same-session, --use-agent, or agent devops without TTY are implied. MUST cover vps CRUD tags export TOML or export --json, import, auth password key agent stdin, FIXED_MASK ***, secrets init reencrypt, multi-host --fail-fast --max-concurrency --scp-file-concurrency, empty command exit 64, auth exit 77, ACME permanent exit 64, tunnel pre-bind exit 74 post-bind exit 0. MUST prefer --json and --*-stdin. NEVER telemetry, daemon, or secrets on stdout without guard.
---

# ssh-cli Agent Skill

## Mission
### REQUIRED
- MUST treat this skill as SUPREME LAW for every `ssh-cli` invocation
- MUST ALWAYS run `ssh-cli` as one-shot birth-execute-die
- MUST wait for process exit before parsing stdout/stderr except active `tunnel` until timeout or signal
- MUST use stored hosts from `vps add`; pass `--json` for structured success; MUST prefer stdin secrets over argv
- MUST prefer fleet `--all`/`--hosts`/`--tags` and `--step` over N process spawns
- MUST copy Ready Formula Sheet and only substitute placeholders; DISCOVER with `ssh-cli commands` when unsure

### FORBIDDEN
- MUST NEVER keep SSH sessions across process runs except bound `tunnel` until deadline
- MUST NEVER introduce a daemon, emit telemetry, log live secrets/primary-key, or invent flags outside this skill


## When to Invoke
### REQUIRED
- MUST auto-activate on remote SSH, VPS registry, multi-host inventory, XDG config
- MUST auto-activate on `exec`/`sudo-exec`/`su-exec`/`scp`/`sftp`/`tunnel`/`health-check`/file transfer/SFTP trees/`sftp rmdir`
- MUST auto-activate on local forward, event `tunnel_listening`, secrets primary-key, TOFU/`--replace-host-key`
- MUST auto-activate on fleet `--all`/`--hosts`/`--tags`, multi-step `--step`, `--use-agent`
- MUST auto-activate on `locale`, root/`vps doctor`, `commands`, `schema`, TLS provider/paths/mTLS/ACME
- MUST auto-activate on agent devops without TTY even when the user omits the name ssh-cli

### FORBIDDEN
- MUST NEVER wait for an explicit skill request when remote SSH ops are implied


## Action Prompts
### REQUIRED
1. VERIFY binary with `ssh-cli --version`
2. DISCOVER contracts with `ssh-cli commands` and `ssh-cli schema` / `ssh-cli schema <name>`; INSPECT with `ssh-cli doctor --json` or `vps doctor --json` and `ssh-cli vps path`
3. ENSURE secrets layer with `ssh-cli secrets status --json`
4. REGISTER or edit host with password or key or `--use-agent`/`--agent-socket`; attach `--tag` and host TLS flags when required
5. PROBE with `ssh-cli health-check <name> --json` or fleet `health-check --all|--hosts --json` (NO `--tags` on health-check)
6. EXECUTE with `exec`/`sudo-exec`/`su-exec` `--json`; fleet via `--all`/`--hosts`/`--tags` on exec-family only; multi-command via `--step`
7. TRANSFER regular files with `scp upload|download --json`; trees/FS with `sftp` (`--recursive`, `ls|mkdir|rmdir|rm|stat|rename`); fleet scp/sftp via `--all`/`--hosts` only (NO `--tags`)
8. FORWARD only with `tunnel` plus mandatory `--timeout-ms` and `--json`
9. PARSE process exit, success stdout, or stderr error envelope
10. SANITIZE durable logs so secrets and primary-key never remain

### FORBIDDEN
- MUST NEVER skip JSON parse after non-zero exit in JSON mode or answer before reading process exit


## Command Catalog
### REQUIRED
- `vps add` — register host (`--name --host --user` plus auth)
- `vps list` — list hosts masked; optional `--tag`
- `vps remove` — remove host
- `vps edit` — edit fields/auth/TLS/limits
- `vps show` — show one host masked
- `vps path` — winning config.toml path
- `vps doctor` — XDG/schema diagnostics; optional SSH probe
- `vps export` — export registry (default TOML body; `--json` envelope)
- `vps import` — import TOML or JSON vps-export
- `connect` — write active host marker
- `exec` — remote shell; fleet and `--step`
- `sudo-exec` — sudo packing elevation; fleet and `--step`
- `su-exec` — one-shot su elevation; fleet and `--step`
- `scp upload` / `scp download` — regular-file only; multi-file and fleet
- `sftp upload` / `sftp download` — file or tree (`--recursive`)
- `sftp ls` / `sftp mkdir` / `sftp rmdir` / `sftp rm` / `sftp stat` / `sftp rename` — remote FS ops
- `tunnel` — bounded local forward; mandatory `--timeout-ms`
- `health-check` — connectivity probe; name / `--all` / `--hosts`
- `secrets status` / `secrets init` / `secrets reencrypt` — primary-key lifecycle
- `completions` — bash zsh fish elvish powershell
- `commands` — JSON command tree
- `schema` — catalog or one named body
- `doctor` — root alias of `vps doctor`
- `locale show|set|clear` — UI language
- `tls provider` / `tls paths` — CryptoProvider and XDG TLS layout
- `tls mtls list|import|show|remove` — mTLS identities
- `tls acme account create|show` — ACME account
- `tls acme issue|complete|status|list` — DNS-01 cert lifecycle

### FORBIDDEN
- MUST NEVER invent root commands outside this catalog


## Global Flags
### REQUIRED
- `--lang <LOCALE>` — force BCP47 UI (negotiate to `en` or `pt-BR`)
- `-v`/`-vv`/`-vvv` — graduated crate-scoped verbosity; maps to `warn,ssh_cli=info|debug|trace`; default `error`
- `-q` / `--quiet` — suppress non-JSON human prose
- `--config-dir <DIR>` — override config base
- `--no-color` — disable color
- `--output-format text|json` — if omitted, JSON when stdout non-TTY except `vps export` default body
- `--json` — force JSON (alias of `--output-format json`)
- `--disable-sudo` — disable sudo-exec/su-exec this invocation
- `--replace-host-key` — replace diverging TOFU key after human approval
- `--allow-plaintext-secrets` — plaintext at rest (tests only)
- `--secrets-key-file <PATH>` — 64-hex primary-key file for this one-shot
- `--use-keyring` — MUST use OS keyring for primary-key when set
- `--timeout <MS>` — global SSH op timeout ms; local subcommand `--timeout` wins; tunnel still requires `--timeout-ms`
- `--max-concurrency <N>` — multi-host/tunnel-accept cap 1..=64 (auto CPUs×4 vs free RAM when omitted)
- `--fail-fast` — stop admitting new multi-host units after first failure
- `--scp-file-concurrency <N>` — concurrent SCP files on one session (default 1)

### FORBIDDEN
- MUST NEVER treat `SSH_CLI_HOME`/`SSH_CLI_LANG`/`SSH_CLI_FORCE_TEXT`/`SSH_CLI_MAX_CONCURRENCY` as product stores
- MUST NEVER treat `SSH_CLI_SECRETS_KEY`/`SSH_CLI_SECRETS_KEY_FILE` as stores (fail-closed)
- MUST NEVER rely on ambient `RUST_LOG` (IGNORED); NEVER expect russh password dumps (ALWAYS crate-scoped `warn,ssh_cli=*`)


## Lifecycle and JSON
### REQUIRED
- MUST invoke one complete CLI process per product action; human logs on stderr only
- MUST treat non-TTY as JSON by default when format omitted EXCEPT `vps export` body stays TOML unless `vps export --json`
- MUST force `--json` for agent parsing on non-export commands; default log level `error`
- MUST use `-v`/`-vv`/`-vvv` only when debugging; ambient `RUST_LOG` IGNORED
- MUST parse success only from stdout; hard failures as stderr error envelopes when JSON mode is active
- MUST parse CRUD events `vps-added`/`vps-edited`/`vps-removed`/`vps-connected`/`vps-import`
- MUST read exec fields `stdout`/`stderr`/`exit_code`/`truncated_stdout`/`truncated_stderr`/`duration_ms`
- MUST treat single-step `exec --json` as exactly ONE success object; multi-step emits one object per step with `step` and `command`
- MUST read scp event equal to `scp-transfer`; sftp events `sftp-transfer`/`sftp-list`/`sftp-fs-op`/`sftp-batch`
- MUST read tunnel event equal to `tunnel_listening` with `local_port`/`remote_host`/`remote_port`/`timeout_ms`
- MUST read envelope fields `exit_code`/`message`/`remote_exit_code`/`retryable`/`error_class`/`suggestion` when present
- MUST report truncation when `truncated_stdout` or `truncated_stderr` is true
- MUST verify binary with `ssh-cli --version`; generate human completions via `ssh-cli completions bash|zsh|fish|elvish|powershell`; agents MUST use flags and JSON

### FORBIDDEN
- MUST NEVER mix stderr into success parse, parse stderr as success, invent missing keys, or expect multiple objects from single-step `exec --json`
- MUST NEVER assume a prior process left an open SSH channel


## Registry Auth Secrets
### REQUIRED
- MUST register unique `--name`; supply password or `--key` or stdin password or `--use-agent`/`--agent-socket` (exactly one primary auth)
- MUST pass `--port` when not 22; `--check` for immediate probe; repeatable `--tag` on add; list filter `--tag`
- MUST enable SSH-over-TLS with `--tls` and optional `--tls-sni`/`--tls-client-cert`/`--tls-client-key`; edit via `--tls`/`--no-tls`
- MUST switch auth to agent with `vps edit --use-agent`
- MUST mask list/show secrets; empty password is JSON `null` (key-only); non-empty is FIXED_MASK `***`; same for `sudo_password`/`su_password`/`key_passphrase`
- MUST run `doctor --json`/`vps doctor --json` when config unknown; `vps path` for winning file
- MUST parse doctor as single event `vps-doctor` with `local.secrets_plaintext_opt_out` boolean plus other `local.*`; `ssh_probe` null or health-check-batch
- MUST treat `added_at` present on list/show/export; import is ALLOWED to omit `added_at` (serde default)
- MUST treat host/vps timeouts as milliseconds; values under 1000 emit stderr warning
- MUST treat `vps export` body as TOML by default even non-TTY; `vps export --json` only for envelope event `vps-export`
- MUST export without secrets by default; empty redacted secrets are empty strings (NEVER `sshcli-enc`); non-empty redacted are `***`
- MUST require human approval before `--include-secrets`; NEVER pipe `--include-secrets` without `-o`/`--output` or `--i-understand-secrets-on-stdout`
- MUST accept import TOML EN keys plus PT aliases and JSON `vps-export`; use `--allow-incomplete` for redacted skeletons
- MUST treat invalid import TOML as exit 65
- MUST parse `secrets_key_auto_created` on the SAME `vps-added` document on first auto primary-key — NEVER a second event
- MUST use `connect` only for active marker; still pass explicit VPS name when certainty is required; nameless `health-check` only after connect
- MUST prefer `--password-stdin`/`--key-passphrase-stdin`/`--sudo-password-stdin`/`--su-password-stdin` over argv
- MUST support runtime auth overrides on exec/scp/sftp/tunnel/health-check; exit 77 means auth failure — change credentials before retry
- MUST treat at-rest encryption as default; product term primary-key; accept legacy keyring alias `secrets-master-key` read-only beside `secrets-primary-key`
- MUST resolve primary-key order: (1) `--secrets-key-file` 64 hex; (2) OS keyring when `--use-keyring`; (3) XDG/`--config-dir` `secrets.key` auto-created on first secret write
- MUST run `secrets status --json` before decrypt diagnosis; `secrets init [--force] [--keyring] --json`; `secrets reencrypt --json` after rotation
- MUST keep plaintext only via `--allow-plaintext-secrets` in tests; expect atomic config/secrets writes mode 0600 on Unix
- MUST use `--config-dir` not `SSH_CLI_HOME`; `--lang`/`locale set` not `SSH_CLI_LANG`

### FORBIDDEN
- MUST NEVER create empty-credential hosts, invent fake passwords for `null`, treat `***` as real, commit raw secrets, print primary-key material, or enable plaintext in production agent flows
- MUST NEVER treat doctor `secrets_plaintext_opt_out` as a string or expect a second secrets-key-auto-created event


## Fleet Exec Elevation
### REQUIRED
- MUST prefer fleet flags over N spawns; use `--all`/`--hosts` on exec/sudo-exec/su-exec/scp/sftp/health-check
- MUST use `--tags` ONLY on exec/sudo-exec/su-exec (FORBIDDEN on health-check/scp/sftp)
- MUST parse batch schemas `health-check-batch`/`exec-batch`/`scp-batch`/`sftp-batch` with `max_concurrency`
- MUST use global `--max-concurrency` (1..=64) and `--fail-fast` for fan-out control; batch cancel has correct cardinality
- MUST treat empty registry plus fleet selectors as exit 64; tunnel is single-host only (N one-shots for multi-host)
- MUST use `doctor --probe-ssh [--hosts a,b] --json` for local plus multi-host health under `vps-doctor.ssh_probe`
- MUST use multi-file SCP forms and `--scp-file-concurrency` when parallel channels on one session are required
- MUST treat empty remote command as exit 64 message exactly `empty command` (English always)
- MUST honor defaults max_command_chars 1000 and max_output_chars 100000; raise via `vps edit`; pass exec `--timeout <ms>` and optional `--description`
- MUST use `--step <CMD>` (repeatable) on same SSH session; parse one object per step with 0-based `step`
- MUST use `sudo-exec`/`su-exec` for elevation; honor `--disable-sudo` and host `disable_sudo`; elevation is one-shot only
- MUST prefer `--sudo-password-stdin`/`--su-password-stdin`

### FORBIDDEN
- MUST NEVER invent `--tags` on health-check/scp/sftp, spawn one process per host when fleet covers the set, send empty commands, prepend raw `sudo` to `exec`, or assume sticky elevated shells
- MUST NEVER retry exit 64/65/66/77 without changing inputs


## SCP SFTP Tunnel Health
### REQUIRED
- MUST use scp for regular files only; argument order upload local→remote, download remote→local; multi-file and fleet supported
- MUST parse scp success event equal to `scp-transfer` with `direction`/`bytes`/`duration_ms`
- MUST expect SCP download via `.ssh-cli.partial` then atomic rename; mtime/mode preserve without extra flag; stream 32 KiB chunks
- MUST treat remote missing SCP as exit 66 message `file not found: <path>`
- MUST use sftp for files or trees (`--recursive`) and FS ops; parse transfer/list/fs-op/batch events
- MUST treat SFTP recursive as NEVER following symlinks; set permissions with mask `0o7777` (no S_IFMT bits)
- MUST treat SFTP upload as writing real destination bytes; after critical uploads MUST verify destination size or checksum — NEVER trust JSON `bytes` alone
- MUST use `sftp rmdir` only for empty remote directories; pass `--timeout <ms>` on scp/sftp when needed
- MUST pass mandatory `--timeout-ms` on every tunnel; default bind `127.0.0.1`; positional `tunnel <vps> <local_port> <remote_host> <remote_port>`
- MUST treat local port `0` as ephemeral; after bind trust JSON `local_port` (>=1)
- MUST WAIT for event equal to `tunnel_listening` before using local port; leave process until deadline/signal
- MUST treat tunnel post-bind deadline as exit 0 after `tunnel_listening`; pre-bind timeout as exit 74
- MUST use health-check after host changes; fleet `--all`/`--hosts`; auth overrides supported; `--replace-host-key` only after human confirmation
- MUST pass `--timeout` (not `--timeout-ms`) on scp/sftp/exec/health-check; `--timeout-ms` ONLY on tunnel

### FORBIDDEN
- MUST NEVER scp directories or invent `-r` on scp (trees MUST use `sftp --recursive`)
- MUST NEVER use `--timeout-ms` on scp/health-check or `--timeout` on tunnel
- MUST NEVER treat leftover `.ssh-cli.partial` as final success artifact or remote missing SCP as exit 74
- MUST NEVER invent SFTP REPL, follow symlinks in recursive trees, open unbounded tunnels, or use local port before `tunnel_listening`
- MUST NEVER bind tunnel to `0.0.0.0` without explicit security decision or auto-replace host keys without approval


## Locale TLS Discovery Completions
### REQUIRED
- MUST use `locale show` (default), `locale set <LOCALE>`, `locale clear`
- MUST treat precedence CLI `--lang` then XDG lang file then system then `en`; negotiate to `en` or `pt-BR`
- MUST use `tls provider` / `tls paths`; mTLS via `tls mtls list|import|show|remove`
- MUST use ACME via `tls acme account create|show` and `tls acme issue|complete|status|list` with DNS-01 two-step (`--print-challenge` then complete)
- MUST treat ACME permanent validation (`invalidContact`, permanent 4xx) as exit 64 — NEVER retry as 74; MUST treat transient ACME (rateLimited/timeout) as exit 74 only when product marks retryable
- MUST attach host SSH-over-TLS via `vps add|edit --tls` plus optional SNI/client cert/key
- MUST run `ssh-cli commands` and `ssh-cli schema`/`schema <name>` before improvising flag shapes
- MUST support completions shells bash zsh fish elvish powershell

### FORBIDDEN
- MUST NEVER treat `SSH_CLI_LANG` as a store or invent interactive ACME wait loops when two-step issue/complete exists


## Exit Codes and Retry
### REQUIRED
- MUST map exits 0 success, 1 general, 64 usage, 65 data, 66 not found, 73 cant create, 74 IO/SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- MUST treat empty command exit 64; invalid import TOML exit 65; remote missing SCP exit 66; auth exit 77; ACME permanent exit 64
- MUST treat tunnel post-bind deadline exit 0 after `tunnel_listening`; pre-bind timeout exit 74
- MUST retry at most twice on 74 with backoff when envelope `retryable` is true; fail fast on 64/65/66/77
- MUST surface remote `exit_code` from success JSON and `remote_exit_code` from error envelopes separately from CLI process exit

### FORBIDDEN
- MUST NEVER swallow non-zero exits, confuse remote command failure with local usage failure, retry post-bind tunnel exit 0, or retry ACME permanent exit 64 as IO


## Ready Formula Sheet
### REQUIRED
- MUST copy formulas exactly and only substitute placeholders
- MUST RUN `ssh-cli --version`
- MUST RUN `ssh-cli commands`
- MUST RUN `ssh-cli schema`
- MUST RUN `ssh-cli schema <NAME>`
- MUST RUN `ssh-cli completions bash`
- MUST RUN `ssh-cli completions zsh`
- MUST RUN `ssh-cli completions fish`
- MUST RUN `ssh-cli completions elvish`
- MUST RUN `ssh-cli completions powershell`
- MUST RUN `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --port <PORT> --tag <TAG> --check`
- MUST RUN `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --use-agent --agent-socket <SOCK> --tag <TAG>`
- MUST RUN `printf '%s' "$PASS" | ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --password-stdin`
- MUST RUN `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --tls --tls-sni <SNI> --tls-client-cert <CERT> --tls-client-key <KEY>`
- MUST RUN `printf '%s' "$SUDO" | ssh-cli vps edit <NAME> --sudo-password-stdin`
- MUST RUN `ssh-cli vps edit <NAME> --timeout <MS> --max-command-chars <N> --max-output-chars <N>`
- MUST RUN `ssh-cli vps edit <NAME> --use-agent --agent-socket <SOCK>`
- MUST RUN `ssh-cli vps edit <NAME> --tls --tls-sni <SNI>`
- MUST RUN `ssh-cli vps edit <NAME> --no-tls`
- MUST RUN `ssh-cli vps list --json`
- MUST RUN `ssh-cli vps list --tag <TAG> --json`
- MUST RUN `ssh-cli vps show <NAME> --json`
- MUST RUN `ssh-cli doctor --json`
- MUST RUN `ssh-cli vps doctor --json`
- MUST RUN `ssh-cli doctor --probe-ssh --json`
- MUST RUN `ssh-cli doctor --probe-ssh --hosts <A>,<B> --json`
- MUST RUN `ssh-cli vps path`
- MUST RUN `ssh-cli vps export -o /tmp/hosts.toml`
- MUST RUN `ssh-cli vps export --json`
- MUST RUN `ssh-cli vps export --include-secrets -o /tmp/hosts-secrets.toml`
- MUST RUN `ssh-cli vps import --file /tmp/hosts.toml`
- MUST RUN `ssh-cli vps import --file /tmp/hosts.json`
- MUST RUN `ssh-cli vps import --file /tmp/hosts-redacted.toml --allow-incomplete`
- MUST RUN `ssh-cli connect <NAME>`
- MUST RUN `ssh-cli vps remove <NAME>`
- MUST RUN `ssh-cli exec <NAME> "<CMD>" --json`
- MUST RUN `ssh-cli exec <NAME> "<CMD>" --json --timeout <MS> --description "<AUDIT>"`
- MUST RUN `ssh-cli -q exec <NAME> "<CMD>" --json`
- MUST RUN `ssh-cli exec <NAME> "<CMD>" --step "<CMD2>" --step "<CMD3>" --json`
- MUST RUN `ssh-cli sudo-exec <NAME> "<CMD>" --json`
- MUST RUN `printf '%s' "$SUDO" | ssh-cli sudo-exec <NAME> "<CMD>" --json --sudo-password-stdin`
- MUST RUN `ssh-cli su-exec <NAME> "<CMD>" --json`
- MUST RUN `printf '%s' "$SU" | ssh-cli su-exec <NAME> "<CMD>" --json --su-password-stdin`
- MUST RUN `ssh-cli --max-concurrency <N> exec --all "<CMD>" --json`
- MUST RUN `ssh-cli exec --hosts <A>,<B> "<CMD>" --json`
- MUST RUN `ssh-cli exec --tags <TAG1>,<TAG2> "<CMD>" --json`
- MUST RUN `ssh-cli --fail-fast exec --all "<CMD>" --json`
- MUST RUN `ssh-cli sudo-exec --all "<CMD>" --json`
- MUST RUN `ssh-cli su-exec --all "<CMD>" --json`
- MUST RUN `ssh-cli sudo-exec --tags <TAG> "<CMD>" --json`
- MUST RUN `ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json`
- MUST RUN `ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json`
- MUST RUN `ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --timeout <MS>`
- MUST RUN `ssh-cli scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json`
- MUST RUN `ssh-cli --scp-file-concurrency <N> scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json`
- MUST RUN `ssh-cli scp upload --all <LOCAL_FILE> <REMOTE_FILE> --json`
- MUST RUN `ssh-cli scp download --all <REMOTE_FILE> <LOCAL_PREFIX> --json`
- MUST RUN `ssh-cli scp upload --hosts <A>,<B> <LOCAL_FILE> <REMOTE_FILE> --json`
- MUST RUN `printf '%s' "$PASS" | ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json --password-stdin`
- MUST RUN `printf '%s' "$KEY_PASS" | ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --key <KEY_PATH> --key-passphrase-stdin`
- MUST RUN `ssh-cli sftp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json`
- MUST RUN `ssh-cli sftp upload --recursive <NAME> <LOCAL_DIR> <REMOTE_DIR> --json`
- MUST RUN `ssh-cli sftp download --recursive <NAME> <REMOTE_DIR> <LOCAL_DIR> --json`
- MUST RUN `ssh-cli sftp ls <NAME> <REMOTE_DIR> --json`
- MUST RUN `ssh-cli sftp mkdir <NAME> <REMOTE_DIR> --json`
- MUST RUN `ssh-cli sftp rmdir <NAME> <REMOTE_DIR> --json`
- MUST RUN `ssh-cli sftp rm <NAME> <REMOTE_FILE> --json`
- MUST RUN `ssh-cli sftp stat <NAME> <REMOTE_PATH> --json`
- MUST RUN `ssh-cli sftp rename <NAME> <FROM> <TO> --json`
- MUST RUN `ssh-cli sftp upload --all <LOCAL_FILE> <REMOTE_FILE> --json`
- MUST RUN `ssh-cli sftp upload --hosts <A>,<B> <LOCAL_FILE> <REMOTE_FILE> --json`
- MUST RUN `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json`
- MUST WAIT for event equal to `tunnel_listening` before using local port
- MUST RUN `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --bind 127.0.0.1`
- MUST RUN `printf '%s' "$PASS" | ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --password-stdin`
- MUST RUN `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --key <KEY_PATH>`
- MUST RUN `printf '%s' "$KEY_PASS" | ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --key <KEY_PATH> --key-passphrase-stdin`
- MUST RUN `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --use-agent --agent-socket <SOCK>`
- MUST RUN `ssh-cli health-check <NAME> --json`
- MUST RUN `ssh-cli health-check <NAME> --timeout <MS> --json`
- MUST RUN `ssh-cli health-check --json`
- MUST RUN `ssh-cli --max-concurrency <N> health-check --all --json`
- MUST RUN `ssh-cli health-check --hosts <A>,<B> --json`
- MUST RUN `printf '%s' "$PASS" | ssh-cli health-check <NAME> --json --password-stdin`
- MUST RUN `ssh-cli health-check <NAME> --json --key <KEY_PATH>`
- MUST RUN `printf '%s' "$KEY_PASS" | ssh-cli health-check <NAME> --json --key <KEY_PATH> --key-passphrase-stdin`
- MUST RUN `ssh-cli health-check <NAME> --json --use-agent --agent-socket <SOCK>`
- MUST RUN `ssh-cli health-check <NAME> --json --replace-host-key`
- MUST RUN `ssh-cli secrets status --json`
- MUST RUN `ssh-cli secrets init --json`
- MUST RUN `ssh-cli secrets init --force --json`
- MUST RUN `ssh-cli secrets init --keyring --json`
- MUST RUN `ssh-cli secrets reencrypt --json`
- MUST RUN `ssh-cli --allow-plaintext-secrets --config-dir <DIR> secrets status --json`
- MUST RUN `ssh-cli --secrets-key-file <KEY_FILE> secrets status --json`
- MUST RUN `ssh-cli --use-keyring secrets status --json`
- MUST RUN `ssh-cli --replace-host-key exec <NAME> "true"`
- MUST RUN `ssh-cli --config-dir <DIR> vps list --json`
- MUST RUN `printf '%s' "$KEY_PASS" | ssh-cli exec <NAME> "id" --json --key <KEY_PATH> --key-passphrase-stdin`
- MUST RUN `ssh-cli exec <NAME> "id" --json --use-agent --agent-socket <SOCK>`
- MUST RUN `ssh-cli locale show --json`
- MUST RUN `ssh-cli locale set <LOCALE>`
- MUST RUN `ssh-cli locale clear`
- MUST RUN `ssh-cli --lang <LOCALE> vps list --json`
- MUST RUN `ssh-cli tls provider --json`
- MUST RUN `ssh-cli tls paths --json`
- MUST RUN `ssh-cli tls mtls list --json`
- MUST RUN `ssh-cli tls mtls import --name <NAME> --cert <CERT> --key <KEY> --json`
- MUST RUN `ssh-cli tls mtls show <NAME> --json`
- MUST RUN `ssh-cli tls mtls remove <NAME> --json`
- MUST RUN `ssh-cli tls acme account create --contact mailto:<EMAIL> --json`
- MUST RUN `ssh-cli tls acme account create --contact mailto:<EMAIL> --staging --force --json`
- MUST RUN `ssh-cli tls acme account show --json`
- MUST RUN `ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --json`
- MUST RUN `ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --staging --json`
- MUST RUN `ssh-cli tls acme complete --domain <DOMAIN> --json`
- MUST RUN `ssh-cli tls acme status --json`
- MUST RUN `ssh-cli tls acme status --domain <DOMAIN> --json`
- MUST RUN `ssh-cli tls acme list --json`
- MUST RUN `ssh-cli -v exec <NAME> "true" --json`
- MUST RUN `ssh-cli -vv exec <NAME> "true" --json`
- MUST RUN `ssh-cli -vvv exec <NAME> "true" --json`
- MUST RUN `cargo install ssh-cli --locked --force`
- NEVER pipe `--include-secrets` without `-o`/`--output` or `--i-understand-secrets-on-stdout`
- NEVER invent `-r` on scp; trees MUST use `sftp --recursive`
- NEVER trust SFTP JSON `bytes` alone for critical integrity
- NEVER send empty remote command strings
- NEVER retry ACME permanent exit 64 as 74


## Absolute Prohibitions
### FORBIDDEN
- MUST NEVER keep SSH sessions between turns except active tunnel until deadline
- MUST NEVER reintroduce product daemons or emit telemetry
- MUST NEVER leak secrets into argv when stdin variants exist
- MUST NEVER open tunnels without `--timeout-ms` or use local port before `tunnel_listening`
- MUST NEVER scp directories; for trees MUST use `sftp --recursive` with no symlink follow
- MUST NEVER invent fake passwords when JSON shows `null` or print primary-key material
- MUST NEVER treat tunnel post-bind exit 0 as failure or remote missing SCP as exit 74
- MUST NEVER pipe `--include-secrets` without `-o`/`--output` or `--i-understand-secrets-on-stdout`
- MUST NEVER bind tunnel to `0.0.0.0` without explicit security decision
- MUST NEVER treat host timeouts as seconds
- MUST NEVER retry ACME permanent exit 64 as 74
- MUST NEVER treat `SSH_CLI_SECRETS_KEY`/`SSH_CLI_SECRETS_KEY_FILE`/`SSH_CLI_HOME`/`SSH_CLI_LANG`/`SSH_CLI_FORCE_TEXT`/`SSH_CLI_MAX_CONCURRENCY` as stores
- MUST NEVER rely on ambient `RUST_LOG` or expect russh password dumps
- MUST NEVER invent `-r` on scp or multiple NDJSON objects from single-step `exec --json`
- MUST NEVER invent flags outside this skill

### REQUIRED
- MUST re-read this skill before every non-trivial workflow
- MUST use stored hosts, stdin secrets, JSON output, one-shot execution
- MUST parse stdout success and stderr envelopes on hard failures
- MUST wait for `tunnel_listening`; post-bind exit 0; pre-bind exit 74
- MUST treat empty command 64, missing SCP 66, invalid import TOML 65, auth 77, ACME permanent 64
- MUST treat `vps export` body as TOML unless `vps export --json`
- MUST parse doctor event `vps-doctor` with boolean `secrets_plaintext_opt_out`
- MUST prefer fleet flags and `--step`; fail closed on auth, host-key, and usage errors
