# Integrations


> Connect 9 AI agents and any shell to servers with 48 one-shot commands

- Read this document in [Portuguese (pt-BR)](INTEGRATIONS.pt-BR.md)
- Pair this catalog with [docs/AGENTS.md](docs/AGENTS.md) and [skills/ssh-cli-en/SKILL.md](skills/ssh-cli-en/SKILL.md)
- Integration contract: ssh-cli runs as a subprocess, so the whole contract is argv, stdin, stdout, stderr and the exit code
- Every ssh-cli command in the examples below was run against the 0.5.6 build that ships the keyring backend, while the agent-side setup is not tested by this project and follows each official documentation
- Current version: 0.5.6, with the newest changes at the top of the new flags section


## Flag Aliases
### camelCase aliases implemented in clap (do not invent others)
- Use `--sudoPassword` as alias of `--sudo-password`
- Use `--suPassword` as alias of `--su-password`
- Use `--maxChars` or `--max-chars` as legacy aliases that map to `max_command_chars`, and prefer `--max-command-chars`
- Use `--disableSudo` as alias of the global `--disable-sudo`, accepted before or after the subcommand wherever the global flag is accepted, while `vps add` and `vps edit` reject it and take `--disable-sudo`, which persists the block in the registry
- Use `--fields` as alias of `--select`
- Use `--json` as alias of `--output-format json`
- No camelCase aliases exist for `--config-dir`, `--output-format` or `--no-color`: use the kebab-case forms exactly


## New Flags by Version
### Track surface growth without reading source
- `0.5.6` rename and complete edit (GAP-RENAME-001): `vps rename <NAME> <NEW_NAME>` and `vps edit <NAME> --name <NEW>` move the registry key and keep every credential, the record seals without associated data, so the blob is not bound to the host name and the credential stays readable under the new one
- Eight clear flags remove a field instead of overwriting it: `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key`, `--no-key` (key path plus passphrase), `--no-agent-socket`
- Each conflicts with the value flag of its own field, and clearing the last auth method exits 64
- `0.5.6` auth switching: `vps edit --key`, `--password`, `--password-stdin` and `--use-agent` switch the stored method and clear the opposite fields in one call
- `updated` lists the new method and only the cleared fields that held a value
- Two primary auth flags in one call exit 2 at parse time, and a passphrase without any key exits 64 with the record untouched
- A different `--key` drops the old passphrase unless a new one comes along, and `--agent-socket` alone implies `--use-agent`
- `0.5.6` host names and terminal safety: new names are ASCII letters, digits, dot, hyphen and underscore, 1 to 255 bytes, starting with a letter or digit
- `vps import` lists every invalid name at once with exit 64 and writes nothing, also under `--dry-run`
- Names stored before 0.5.6 still load, `doctor` lists them in `local.invalid_host_names`, and `vps rename` repairs them
- Text mode writes control and bidi characters as escapes such as `\u{1b}`, and `sftp ls` and `sftp stat` also escape `\n` and `\t` in remote names
- `tls mtls import --name` refuses control, formatting and bidi characters with exit 64 and creates no directory
- Two `[hosts]` tables that normalize to the same NFC name exit 65 instead of losing one host in silence
- `0.5.6` config directory: a directory the command-line interface (CLI) creates is born 0700, an existing non-default directory is never chmod-ed, and `doctor` reports `local.config_dir_permissions` and `local.config_dir_exposed`
- `--config-dir <path>.toml` uses the parent as the directory for every file, `vps export -o` creates missing directories 0700, and `known_hosts.lock` and `lang` are created 0600
- `0.5.6` dry-run echo: the plans of `vps edit`, `vps rename` and `vps remove` carry `target_resolved`, `target_source`, `host_resolved` and `host_source`
- `vps edit`, `vps rename` and `vps remove` on a missing registry exit 66 and create no directory and no lock
- `0.5.6` events and previews: `vps edit <NAME> --name <NEW>` and `vps rename <NAME> <NEW_NAME>` both emit `vps-renamed` with `name`, `previous_name`, `active_rewritten`, `updated`, `target_resolved` and `target_source`
- `vps-edited` is emitted only when nothing was renamed and carries no `previous_name`
- `updated` is always present: the fields whose stored value changed, in application order, each at most once
- Empty when every requested value already matched or when the invocation only renamed the host
- An edit that changes nothing is an idempotent no-op (exit 0, `vps-edited`, `updated: []`, `config.toml` not rewritten), and an invocation with no field, rename or clear flag exits 64
- `--dry-run` now covers eight operations
- The config lock honours global `--lock-wait-ms` (default 5000, max 300000) and exits 69 `unavailable` instead of blocking forever, and `commands --json` publishes every command's `args` with `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` and `hidden`
- `0.5.6` rename semantics and lock behavior: `vps rename ghost ghost` exits 66 (the existing-host precondition wins)
- An identical name for an existing host exits 64 and the error carries `target_resolved`
- A destination that already exists exits 64 with `vps_duplicate`
- Rename writes the `active` marker first and `config.toml` second, restores the old marker if the save fails, and renames nothing if the marker write fails
- A crash between the two steps leaves an orphan marker, which is fail-closed (`--use-active` exits 66)
- `vps rename` is not idempotent: repeating it after a success exits 66, and `vps show <NEW>` confirms the result
- The lock wait runs in 100 ms slices that check the signal, so SIGINT/SIGTERM exit 130/143 within one slice
- `config.toml.lock` is created with mode 0600
- The marker write fsyncs its directory
- The exit 69 hint now says to repeat the argv or raise `--lock-wait-ms`
- `0.5.6` text mode, import and version: a stdout write error now propagates in every verb in text mode (a broken pipe exits 141, a full disk is an I/O error, before, the exit was 0), and on `exec` with stdout closed the write error wins over the remote command failure
- `vps import` validates Transport Layer Security (TLS) as `vps edit` does (a certificate without a key exits 64), and the `vps-import` success event gains `replaced` (overwritten names, sorted, always present, the dry-run keeps `replaces_existing`)
- `--version` reads `.commit_hash` as `<short hash> <version>` and resolves env, then git, then the file, then `unknown`
- Without env and git the file counts only when its version is the package version
- `0.5.6` BREAKING (library) and scripts: `SshCliError::Unavailable { service: UnavailableService }` and `SshCliError::unavailable(UnavailableService)` replace the string service (`UnavailableService` is `#[non_exhaustive]` with `Keyring` and `ConfigLock`)
- The wire text is unchanged (`service unavailable: config_lock`)
- The nine release scripts became `scripts/*.py` (Python 3.9+, stdlib only, on Windows `py -3 scripts/<name>.py`) and their environment variables became flags
- See [docs/MIGRATION.md](docs/MIGRATION.md)
- `0.5.5` BREAKING + security: explicit target designation (GAP-SSH-EXEC-ARGC-001, CWE-441): the exec family inferred the host from positional count, so `exec <HOST> --step <CMD>` ran the host name as a remote binary on whatever host `connect` last marked
- A lone positional is now a usage error and a nameless `health-check` exits 64
- Reach the marker with `--use-active`, which exits 66 when no marker exists
- `0.5.5` target provenance on every envelope (GAP-SSH-EXEC-ENVELOPE-002): `target_resolved`, `target_source` (`argv` | `active_marker` | `selector`) and `active_fallback`, always on success, and on the error envelope once the target was resolved, where they are absent rather than empty
- Bind new consumers to those canonical names
- `host_resolved` and `host_source` ship alongside as 0.5.5 read aliases
- `doctor` reports `local.active_vps`
- `0.5.4` tunnel modes (G-TUN-R01/R02/R03): `tunnel --reverse` (server listens and delivers back, `REMOTE_PORT 0` = server allocates), `tunnel --socks5` (local SOCKS5 proxy, RFC 1928 no-auth + CONNECT, destination per connection, `REMOTE_HOST`/`REMOTE_PORT` omitted), `tunnel --remote-socket <PATH>` (remote Unix socket via `direct-streamlocal@openssh.com`, absolute path or exit 64)
- JSON `mode` is `local` | `reverse` | `socks5` | `streamlocal`
- `0.5.4` agent-native payload shaping: global `--select` (alias `--fields`), `--filter` (`key=value` | `key!=value` | `key~substring`, repeatable AND), `--limit`, `--sort`, `--dedupe-by`, `--count-only` (→ `{"count": N}`), `--truncate-content` (characters, never bytes), `--max-output-bytes` (drops trailing records, never slices JSON), all applied before serialization
- Plus global `--no-input` and `--dry-run` (honoured by the eight operations listed under 0.5.6, elsewhere exit 64)
- `0.5.4` BREAKING + security: partial multi-host failure exits 1 (was 65, G-ERR-R02)
- Non-loopback `tunnel --bind` requires `--i-accept-network-exposure` (G-TUN-R13)
- New `tunnel_closed` event (`reason`, `forwards_served`, `capacity_waits`)
- `schema dry-run` / `schema tunnel-closed` now exit 0
- A1 applies the pre-existing `AUTH_BANNER_MAX_CHARS` (512) banner log cap on character boundaries instead of byte indices, removing a remotely triggerable abort, A2 creates Automatic Certificate Management Environment (ACME) and mutual TLS (mTLS) keys at `0600` instead of tightening after creation, A3 masks inbound modes with `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) so setuid/setgid/sticky cannot ride a download
- `0.5.3` SSH File Transfer Protocol (SFTP) integrity + agent safety (G1–G19): SFTP upload no longer truncates destinations to 0 bytes (`FileAttributes::empty`)
- SETSTAT sends atime+mtime together
- `set_metadata` fail-closed
- Permission bits via `SFTP_PERM_MASK` (`0o7777`)
- Download surfaces local `set_permissions` failures
- SCP download `sync_data` before rename
- Batch cancel fills cancelled remainder (`results.len() == input.len()`)
- `exec --json` single NDJSON object
- E2E SFTP checksums E17/E18
- Prefer 0.5.3+ for any SFTP transfer
- `0.5.3` graduated verbosity (G2/G14): `-v` info / `-vv` debug / `-vvv` trace via `ArgAction::Count`, always crate-scoped `warn,ssh_cli=*`, never bare global debug (no russh password dump)
- Ambient `RUST_LOG` still ignored
- Even `-vvv` does not enable russh encrypted channel dumps
- `0.5.2+` SFTP subsystem (G-SFTP): `ssh-cli sftp upload|download|ls|mkdir|rmdir|rm|stat|rename` via `russh-sftp` 2.3 (SFTP v3)
- Upload/download support `--recursive` trees (symlink no-follow), multi-host `--all`/`--hosts`, and JSON events `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch`
- Stream I/O only (no full-file heap)
- SCP remains regular-file-only
- Use 0.5.3+ for SFTP integrity fixes above
- `0.5.2+` multi-host fan-out (bounded concurrency): `exec|sudo-exec|su-exec|scp|sftp|health-check --all` runs concurrent SSH sessions gated by global `--max-concurrency N` (1..=64, auto CPUs×4 vs free RAM/2 / 16 MiB when omitted)
- Batch envelopes: `health-check-batch` / `exec-batch` / `scp-batch` / `sftp-batch` (`docs/schemas/*-batch.schema.json`, field `max_concurrency`)
- Prefer one process with `--all` over N single-host spawns for fleet work
- Tunnel accepts share the same admission gate
- `0.5.2` residual E2E + agent-first export/import + wire v3: root `schema`/`doctor`
- Single `vps-added` + `secrets_key_auto_created`
- Ambient `RUST_LOG` ignored
- ACME permanent 64
- `vps add --use-agent`
- Redacted export writes `password` as an empty string and the other secrets as `null`, while `FIXED_MASK` masks only `vps list` and `vps show`
- Plus the `vps export` body follows the resolved output format (JSON on any non-TTY stdout, TOML only with `--output-format text`)
- `vps import` accepts TOML (EN keys + PT aliases) or JSON `vps-export` envelopes
- Dual-read EN serialize + PT aliases
- Host schema v3
- CLI flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (never environment variables)
- `secrets init|reencrypt --json` events `secrets-init` / `secrets-reencrypt`
- First secret write sets `secrets_key_auto_created` on the same `vps-added` document
- CRUD success paths use `emit_success` JSON events
- `--include-secrets` to pipe/non-TTY requires `-o`/`--output` or `--i-understand-secrets-on-stdout`
- Tunnel `--bind` (default `127.0.0.1`)
- Import `TomlDe` → exit 65
- `SshAuthentication` → 77
- SCP missing message `file not found: <path>` (exit 66)
- Timeout warn in `vps add` if `<1000` ms
- Password-on-argv stderr warn
- Doctor `secrets_plaintext_opt_out` is bool
- `0.4.2` tunnel ephemeral port `local_port=0` reports OS-assigned port after bind (TUN-003)
- SCP remote missing → exit 66 (IO-010)
- `vps export --json` envelope `event: "vps-export"`
- E2e E15/E16
- Suite `gaps_v042`
- `0.4.1` SCP regular files only auth parity + scp event + export/tunnel fixes (inherits 0.4.0 wire, no `-r` / no SFTP)
- Solid SCP wire (avoid crates.io 0.3.9 broken SCP)
- Scp flags `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` → `docs/schemas/scp-transfer.schema.json` with required `event: "scp-transfer"` (IO-009)
- Download writes `{path}.ssh-cli.partial` then rename
- Preserve mtime/mode bi-dir
- Upload streams 32 KiB
- `tunnel --json` emits `tunnel_listening` after bind
- Scp JSON error envelope on stderr when `--json`
- EXP-001 redacted export empty secrets stay `""` (never `sshcli-enc:` of empty)
- TUN-002 tunnel post-bind deadline exits 0 after `tunnel_listening` (pre-bind timeout still 74)
- CLI-005 tunnel auth stdin/passphrase parity
- CLI-006 health-check auth parity
- `0.4.0` solid SCP wire protocol (fixes crates.io 0.3.9 broken SCP)
- File-only transfers
- `tunnel --json` / `tunnel_listening`
- `0.3.9` default tracing filter `error` (agent-first)
- Empty password serializes as JSON `null` on key-only hosts
- `health-check --timeout <ms>`
- Product-line docs audit
- `0.3.8` russh 0.62.2
- Tunnel agent stdout clean
- No-active-VPS exits `66` (`EX_NOINPUT`)
- `cargo deny` with `yanked=deny`
- `0.3.7` `--output-format` on VPS CRUD
- `health-check --json`
- `--quiet`
- JSON error envelope
- Tunnel timeout covers connect
- `0.3.6` adds default at-rest encryption, `secrets status|init|reencrypt`, doctor fields `secrets_key_file` / `secrets_plaintext_opt_out`, `scripts/e2e_real_ssh.py`
- `0.3.5` adds `--key-passphrase-stdin` runtime paths, auto JSON on non-TTY, doctor `secrets_at_rest`, residual atomic export and AEAD (then optional)
- `0.3.4` adds `--key`, `--key-passphrase`, `--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--timeout-ms` (tunnel), `--disable-sudo`, `--description`, `--replace-host-key`, `max_command_chars`, `max_output_chars`, `vps doctor`, `vps export`, `vps import`, `su-exec`
- `0.2.0` adds runtime `--password`, `--sudo-password`, `--timeout` overrides and camelCase aliases
- Prefer 0.5.3+ for SFTP integrity (G1–G19), graduated crate-scoped verbosity, export/import roundtrip, wire schema v3, working SCP + `tunnel --json` / `--bind`, full SSH automation, default secret encryption, and clean supply-chain


## Discovery for agents
- `ssh-cli commands`: full command tree with all 48 leaf commands, and `--json` publishes every argument with `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` and `hidden`
- `ssh-cli --help`: ends with an `Exit status:` section that lists the 14 exit codes, and all 48 leaf commands carry an `Examples:` section in their own `--help`
- `ssh-cli schema [NAME]`: embedded JSON Schema catalog with 25 schemas, or one schema body
- `ssh-cli doctor`: root alias of `vps doctor`, with the local report and an optional SSH probe
- `ssh-cli locale show|set|clear`: UI locale preference under XDG, never a product environment variable
- `ssh-cli tls provider|paths|mtls|acme`: optional SSH-over-TLS material under XDG `tls/`


## Summary Table

| Agent / Platform | Integration style | JSON | Notes | Official docs |
| --- | --- | --- | --- | --- |
| Claude Code | subprocess CLI + skill | yes | Prefer the skill package | [docs](https://code.claude.com/docs/en/overview) |
| Cursor | shell / agent tools | yes | Use `--json` | [docs](https://cursor.com/docs) |
| Windsurf | shell tool | yes | One-shot per task | [docs](https://docs.windsurf.com) |
| Codex CLI | shell tool | yes | Map sysexits | [docs](https://github.com/openai/codex) |
| OpenCode | shell tool | yes | One-shot only | [docs](https://opencode.ai/docs) |
| Aider | shell commands | yes | Store hosts once | [docs](https://aider.chat/docs/) |
| Continue | custom command | yes | XDG multi-host | [docs](https://docs.continue.dev) |
| Gemini CLI | shell tool | yes | Prefer stdin secrets | [docs](https://github.com/google-gemini/gemini-cli) |
| OpenHands | sandbox shell | yes | Bound tunnel timeouts | [docs](https://docs.openhands.dev/overview/introduction) |
| Generic Shell | direct install in bash, zsh, fish, elvish or PowerShell | yes | Completions available | [docs](https://www.gnu.org/software/bash/manual/) |


## Claude Code
- Official documentation: [Claude Code](https://code.claude.com/docs/en/overview)
- Integration style: the Bash tool runs `ssh-cli` as a subprocess, and the skill package teaches the whole surface
- Install the binary on the host PATH with `cargo install ssh-cli --locked`
- Load [skills/ssh-cli-en/SKILL.md](skills/ssh-cli-en/SKILL.md) or the pt package
- Register hosts once with `vps add --password-stdin` and then call `exec` per task
- Prefer `--json` envelopes, parse stdout and read the failure envelope from stderr
- Default stderr is silent at tracing level `error`, and `-v`, `-vv` or `-vvv` raise it crate-scoped while ambient `RUST_LOG` is ignored
- Run `ssh-cli secrets status`, `ssh-cli doctor --json` and `ssh-cli commands --json` as preflight for encryption, paths and surface discovery

```bash
printf '%s' "$PASS" | ssh-cli vps add --name prod --host h.example --user deploy --password-stdin --json
ssh-cli exec prod 'hostname' --json
ssh-cli doctor --json
```


## Cursor
- Official documentation: [Cursor](https://cursor.com/docs)
- Integration style: the terminal tool of the agent runs `ssh-cli`, with `--json` for structured results
- Add a project rule that prefers `ssh-cli` over long-lived Node SSH processes
- Keep credentials out of the chat by using stored hosts and stdin secret flags
- Parse JSON from stdout only and read the exit code first
- Default stderr is silent at tracing level `error`, so ignore tracing unless you pass `-v`, `-vv` or `-vvv`
- Use `--select name,stdout,exit_code` on fleet runs to keep each result small

```bash
ssh-cli exec --hosts web1,web2 'uptime' --json --select name,stdout,exit_code
```


## Windsurf
- Official documentation: [Windsurf](https://docs.windsurf.com)
- Integration style: the shell tool runs one `ssh-cli` command per task
- Invoke one-shot commands after the host registration
- Never keep a tunnel open without `--timeout-ms`, which `tunnel` requires anyway
- Wait for the `tunnel_listening` event before using the local port
- Read `tunnel_closed` to learn why the tunnel ended
- A deadline after the bind exits 0, so treat exit 0 as the normal end of a tunnel

```bash
ssh-cli tunnel prod 8080 127.0.0.1 80 --timeout-ms 60000 --json
```


## Codex CLI
- Official documentation: [Codex CLI](https://github.com/openai/codex)
- Integration style: the shell tool runs `ssh-cli` and the agent reads the exit code
- Treat non-zero exits as typed failures using the `Exit status:` table at the end of `ssh-cli --help`
- Map the exit code before any retry: 77 is auth, 64 is usage, 65 is data and 66 is not found
- Retry only when `retryable` is true in the failure envelope, never on auth or usage errors
- Pass the command as one quoted argument, so the remote shell receives it intact
- A remote command that failed exits 1, and the stderr envelope carries `remote_exit_code` while the stdout JSON has `exit_code`

```bash
ssh-cli exec prod 'systemctl is-active nginx' --json
echo "exit=$?"
```


## OpenCode
- Official documentation: [OpenCode](https://opencode.ai/docs)
- Integration style: the shell tool runs `ssh-cli` with explicit argv
- Use shell tool mode with explicit argv arrays
- Avoid embedding passwords in prompt text
- Use the registry or stdin secrets instead
- One-shot only: there is no session to keep between calls
- Run `ssh-cli commands --json` once to discover every flag of every command

```bash
ssh-cli exec prod 'df -h /' --json
```


## Aider
- Official documentation: [Aider](https://aider.chat/docs/)
- Integration style: shell commands run from the session
- Document host names in the repository without secrets
- Call `ssh-cli exec <name> "..."` for remote operations during edit loops
- Register the host once with `vps add`, so the repository never carries a credential
- Lower `max_output_chars` on the host with `vps edit` to bound remote output, and read `truncated_stdout` for the cut
- Use `vps list --select name,host` to show the registry without any secret

```bash
ssh-cli vps list --select name,host --json
ssh-cli exec prod 'journalctl -u app -n 20 --no-pager' --json
```


## Continue
- Official documentation: [Continue](https://docs.continue.dev)
- Integration style: a custom command maps to an `ssh-cli` subcommand
- Map custom commands to `ssh-cli` subcommands with `--json`
- Use `vps doctor --json` as a health preflight for agent sessions
- Read `local.config_dir_exposed` from the preflight and warn when it is true
- Probe the whole registry with `health-check --all --json` and list failures with `--filter status!=ok`
- The XDG registry is shared by every session of the same user

```bash
ssh-cli doctor --json --select local.config_dir_exposed
ssh-cli health-check --all --json
```


## Gemini CLI
- Official documentation: [Gemini CLI](https://github.com/google-gemini/gemini-cli)
- Integration style: the shell tool runs `ssh-cli` and reads its JSON
- Prefer key auth and the masked `vps show` for verification
- Keep elevation disabled with `vps edit <name> --disable-sudo` unless the task requires root
- Re-enable elevation later with `vps edit <name> --enable-sudo`
- Prefer stdin secrets, because a password on argv shows in process lists
- Use `--use-agent` to authenticate through an SSH agent without a key file

```bash
ssh-cli vps show prod --json
ssh-cli vps edit prod --disable-sudo --json
```


## OpenHands
- Official documentation: [OpenHands](https://docs.openhands.dev/overview/introduction)
- Integration style: the sandbox shell runs `ssh-cli` inside the container
- Install the binary in the image with `cargo install ssh-cli --locked`
- Run inside the sandbox with a network policy that allows only the target hosts
- Force bounded tunnels and short timeouts
- Cap fan-out with `--max-concurrency`, from 1 to 64, when the sandbox is small
- Use `--fail-fast` to stop admitting hosts after the first failure

```bash
ssh-cli tunnel prod 8080 127.0.0.1 80 --timeout-ms 30000 --json
ssh-cli health-check prod --timeout 5000 --json
```


## Generic Shell
- Official documentation: [Generic Shell](https://www.gnu.org/software/bash/manual/)
- Integration style: direct install, with no agent in the middle
- Install completions with `ssh-cli completions <SHELL>` for bash, zsh, fish, elvish or powershell
- Use `--config-dir` only for isolated test sandboxes, because the product does not read an environment variable for it
- Pipe JSON from stdout into `jaq` to pick a field
- Read the exit code with `$?` before parsing anything
- Redirect stderr to a file when you want to keep the failure envelope

```bash
ssh-cli completions bash > ssh-cli.bash
ssh-cli commands --json | jaq '.tree.subcommands | length'
```


## Command reference
- All 48 leaf commands from `ssh-cli commands`, one bullet each
- `ssh-cli vps add`: Adds a new VPS to the registry
- `ssh-cli vps list`: Lists all VPS hosts (passwords masked)
- `ssh-cli vps remove`: Removes a VPS from the registry
- `ssh-cli vps rename`: Renames a registered VPS without touching credentials
- `ssh-cli vps edit`: Edits fields of an existing VPS
- `ssh-cli vps show`: Shows VPS details (passwords masked)
- `ssh-cli vps path`: Shows the configuration file path
- `ssh-cli vps doctor`: Diagnostics for XDG layers / path / schema
- `ssh-cli vps export`: Exports hosts (passwords redacted by default)
- `ssh-cli vps import`: Imports hosts from a TOML file or JSON `vps-export` envelope (EN + legacy PT keys)
- `ssh-cli connect`: Sets the active VPS (writes sibling `active` file in the config directory)
- `ssh-cli exec`: Runs a command on the VPS over SSH (stdout/stderr captured)
- `ssh-cli sudo-exec`: Runs a command with `sudo` (safe `sh -c` packing)
- `ssh-cli su-exec`: Runs a command with one-shot `su -` elevation
- `ssh-cli scp upload`: Uploads a local file to the remote host (regular files only)
- `ssh-cli scp download`: Downloads a remote file to the local host (regular files only)
- `ssh-cli sftp upload`: Uploads a local file or tree to the remote host via SFTP
- `ssh-cli sftp download`: Downloads a remote file or tree to the local host via SFTP
- `ssh-cli sftp ls`: Lists a remote directory
- `ssh-cli sftp mkdir`: Creates a remote directory
- `ssh-cli sftp rmdir`: Removes an empty remote directory
- `ssh-cli sftp rm`: Removes a remote file
- `ssh-cli sftp stat`: Shows metadata for a remote path
- `ssh-cli sftp rename`: Renames a remote path
- `ssh-cli tunnel`: SSH tunnel with mandatory deadline (bounded one-shot)
- `ssh-cli health-check`: Checks SSH connectivity to a VPS (`--all` / `--hosts` / `--use-active`)
- `ssh-cli secrets status`: Shows encryption status (no sensitive material)
- `ssh-cli secrets init`: Generates and stores the primary key (`secrets.key` or keyring) and never prints the key
- `ssh-cli secrets reencrypt`: Rewrites `config.toml` re-encrypting secrets with the current key
- `ssh-cli completions`: Generates shell completions
- `ssh-cli commands`: Emits the full command tree as JSON (agent discovery / rules `mycli commands`)
- `ssh-cli schema`: Emits embedded JSON Schema catalog or one schema body (G-E2E-02)
- `ssh-cli doctor`: Root alias for `vps doctor` (XDG / schema diagnostics, G-E2E-03)
- `ssh-cli locale show`: Shows resolved language, winning layer, and available locales (default)
- `ssh-cli locale set`: Persists preferred language under the config directory (`lang` file, 0o600)
- `ssh-cli locale clear`: Removes the persisted language preference
- `ssh-cli tls provider`: Shows rustls CryptoProvider status (`aws_lc_rs`)
- `ssh-cli tls paths`: Prints XDG TLS directory layout paths
- `ssh-cli tls mtls list`: Lists imported identity names
- `ssh-cli tls mtls import`: Imports PEM cert+key as a named identity
- `ssh-cli tls mtls show`: Shows paths for one identity
- `ssh-cli tls mtls remove`: Removes an identity directory
- `ssh-cli tls acme account create`: Creates an ACME account (credentials under XDG `tls/acme/account.json`, 0o600)
- `ssh-cli tls acme account show`: Shows whether an account exists and its path
- `ssh-cli tls acme issue`: Starts DNS-01 order and prints the TXT challenge (persists order URL under XDG)
- `ssh-cli tls acme complete`: Completes a pending order after DNS TXT is published
- `ssh-cli tls acme status`: Shows certificate / pending status for one domain or all
- `ssh-cli tls acme list`: Lists ACME domain directories under XDG


## Per-leaf flags
- The flags an integration needs to call each leaf, in the same set `ssh-cli commands --json` publishes
- `ssh-cli vps add`: `--name`, `--host`, `--port`, `--user`, `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--sudo-password`, `--sudo-password-stdin`, `--su-password`, `--su-password-stdin`, `--tag`, `--timeout`, `--max-chars`, `--max-command-chars`, `--max-output-chars`, `--tls`, `--tls-sni`, `--tls-client-cert`, `--tls-client-key`, `--use-agent`, `--agent-socket`, `--check`, `--disable-sudo`
- `ssh-cli vps edit`: the same record flags plus `--enable-sudo`, `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-key`, `--no-agent-socket`, `--no-tls`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key`
- `ssh-cli vps list`: `--tag`
- `ssh-cli vps import`: `--file` and `--allow-incomplete`
- `ssh-cli vps export`: `--output`, `--include-secrets`, `--i-understand-secrets-on-stdout`
- `ssh-cli vps doctor` and `ssh-cli doctor`: `--hosts` and `--probe-ssh`
- `ssh-cli exec`, `ssh-cli sudo-exec`, `ssh-cli su-exec`: `--all`, `--hosts`, `--tags`, `--use-active`, `--use-agent`, `--agent-socket`, `--step`, `--description`, `--timeout`, and on the elevation leaves `--sudo-password` or `--su-password` with their `-stdin` twins
- `ssh-cli health-check`: `--all`, `--hosts`, `--use-active`, `--use-agent`, `--agent-socket`, `--timeout`
- `ssh-cli scp upload` and `ssh-cli scp download`: `--src` and `--dest`, plus `--all`, `--hosts` and the auth trio
- `ssh-cli sftp upload` and `ssh-cli sftp download`: `--recursive`, `--src` and `--dest`, plus `--all` and `--hosts`
- The remaining SFTP leaves (`ls`, `mkdir`, `rmdir`, `rm`, `stat`, `rename`) take the auth trio and `--timeout`
- `ssh-cli tunnel`: `--reverse`, `--socks5`, `--remote-socket`, `--bind`, `--i-accept-network-exposure`, `--timeout-ms`
- `ssh-cli secrets init`: `--keyring` and `--force`
- `ssh-cli tls mtls import`: `--cert`, `--key` and `--name`
- `ssh-cli tls acme account create`: `--contact`, `--staging` and `--force`
- `ssh-cli tls acme issue`: `--domain`, `--staging` and `--print-challenge`
- `ssh-cli tls acme complete` and `ssh-cli tls acme status`: `--domain`


## Global flags
- All 27 global flags, valid before or after the subcommand
- `--lang <LOCALE>`: Forces the CLI language (`en` or `pt-BR`, and BCP47 tags negotiate to them)
- `-v, --verbose`: Raises log verbosity on stderr (`-v` info, `-vv` debug, `-vvv` trace), always scoped to this crate
- `-q, --quiet`: Suppresses non-JSON output
- `--config-dir <DIR>`: Overrides the configuration directory, and a path ending in `.toml` makes its parent the directory
- `--no-color`: Disables colored output
- `--output-format <text|json>`: Chooses `text` or `json`, and JSON is the default when stdout is not a TTY
- `--json`: Forces JSON output, alias of `--output-format json`
- `--select <PATHS>`: Keeps only these dotted paths in each record (CSV, alias `--fields`)
- `--filter <EXPR>`: Keeps records matching `key=value`, `key!=value` or `key~substring`, repeatable with AND
- `--limit <N>`: Emits at most N records
- `--sort <PATH>`: Sorts records ascending by dotted path, comparing numbers numerically
- `--dedupe-by <PATH>`: Drops later records that repeat the value of this dotted path
- `--count-only`: Replaces the record collection with `{"count": N}`, counted after filtering
- `--truncate-content <CHARS>`: Shortens strings longer than N characters, never bytes
- `--max-output-bytes <BYTES>`: Caps the envelope size by dropping trailing records, never by slicing the JSON text
- `--no-input`: Refuses to read stdin and fails fast instead of blocking
- `--dry-run`: Prints the plan and exits without executing, accepted only by eight operations and rejected with exit 64 elsewhere
- `--lock-wait-ms <MS>`: Bounds the wait for the config lock (default 5000, maximum 300000) before exit 69
- `--disable-sudo`: Disables `sudo-exec` and `su-exec` for this invocation (alias `--disableSudo`), and placed before `vps add` or `vps edit` it also writes `disable_sudo` to the host
- `--replace-host-key`: Replaces a diverging host key in the TOFU `known_hosts`
- `--allow-plaintext-secrets`: Allows plaintext secrets at rest, for tests only
- `--secrets-key-file <PATH>`: Reads the 64-hex primary key from this file for one invocation
- `--use-keyring`: Prefers the OS keyring for the primary key
- `--timeout <MS>`: Sets the global SSH timeout in milliseconds, and a local `--timeout` wins while `tunnel` still needs `--timeout-ms`
- `--max-concurrency <N>`: Caps concurrent multi-host sessions and tunnel forwards, from 1 to 64
- `--fail-fast`: Stops admitting new multi-host units after the first failure
- `--scp-file-concurrency <N>`: Sets concurrent SCP transfers on one SSH session (default 1)
