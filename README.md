# ssh-cli

[![crates.io](https://img.shields.io/crates/v/ssh-cli.svg)](https://crates.io/crates/ssh-cli)
[![docs.rs](https://docs.rs/ssh-cli/badge.svg)](https://docs.rs/ssh-cli)
[![MSRV](https://img.shields.io/badge/MSRV-1.85.0-blue)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

> Give any LLM remote SSH power in one memory-safe one-shot binary.

- Read this document in [Portuguese (pt-BR)](README.pt-BR.md).
- Install with `cargo install ssh-cli --locked` for a lockfile-aligned graph.
- Upgrade with `cargo install ssh-cli --locked --force`.
- Verify with `ssh-cli --version`.
- Read full history in [CHANGELOG.md](CHANGELOG.md).
- Integrate agents via [docs/AGENTS.md](docs/AGENTS.md) and [INTEGRATIONS.md](INTEGRATIONS.md).
- Follow first use in [docs/HOW_TO_USE.md](docs/HOW_TO_USE.md).
- Copy recipes from [docs/COOKBOOK.md](docs/COOKBOOK.md).
- Check platforms in [docs/CROSS_PLATFORM.md](docs/CROSS_PLATFORM.md).
- Migrate from 0.3.3+ in [docs/MIGRATION.md](docs/MIGRATION.md) (target line **0.4.2**).
- Run tests via [docs/TESTING.md](docs/TESTING.md).
- Consume JSON contracts under [docs/schemas/README.md](docs/schemas/README.md).
- Teach LLMs with [skills/ssh-cli-en/SKILL.md](skills/ssh-cli-en/SKILL.md).


## What is it?
### One-shot multi-host SSH CLI for agents
- Ship a single Rust binary with zero Node runtime and zero daemon.
- Operate N VPS hosts from XDG storage without `.env` files.
- Authenticate with password or private key per host.
- Execute `exec`, `sudo-exec`, and `su-exec` as pure one-shot processes.
- Capture stdout and stderr with structured JSON for orchestration.
- Auto-detect locale between `en-US` and `pt-BR`.
- Disable telemetry completely in every build.


## Why ssh-cli?
### Replace long-lived SSH processes with a die-after-run binary
- Avoid resident Node processes that hold sockets open between tasks.
- Cut RAM and CPU waste from long-lived SSH sessions.
- Register multi-host credentials once under XDG with atomic writes.
- Align command packing and dual maxChars semantics with the one-shot agent contract.
- Trust host keys via TOFU known_hosts instead of always-trust.
- Route errors with sysexits codes that agents classify reliably.


## Superpowers
### Capabilities that make agents productive
- Multi-host CRUD with `vps add|list|show|edit|remove|path|doctor|export|import`
- One-shot remote execution with `exec`, `sudo-exec`, `su-exec`
- Safe `sudo` packing via `sh -c` and shell escape
- Private key auth with optional passphrase
- Dual limits `max_command_chars` and `max_output_chars`
- Timeout with best-effort remote abort
- Bounded tunnel via mandatory `--timeout-ms`
- SCP upload and download of **regular files only** (no recursive directories / no SFTP subsystem; first solid wire fix in **0.4.0**; patch **0.4.2** — avoid crates.io 0.3.9 SCP)
- SCP flag parity with exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` (contract `docs/schemas/scp-transfer.schema.json`)
- SCP download writes `{path}.ssh-cli.partial` then atomic rename; preserve remote mtime/mode bi-directional; upload streams 32 KiB chunks
- SCP JSON success requires `event: "scp-transfer"` (0.4.1 IO-009)
- `tunnel --json` emits structured `tunnel_listening` after local bind
- Tunnel post-bind deadline exits **0** after `tunnel_listening` (pre-bind timeout still **74**) (0.4.1 TUN-002)
- Tunnel and health-check auth parity with exec/scp: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` as applicable (0.4.1 CLI-005/006)
- Redacted `vps export` never emits `sshcli-enc:` for empty password — empty secrets serialize as empty strings (0.4.1 EXP-001)
- Health-check latency probe with optional `--timeout`
- Shell completions for bash zsh fish powershell
- Secrets via stdin flags to avoid argv leaks
- **Default at-rest encryption** (ChaCha20-Poly1305) with auto XDG `secrets.key`
- Master-key UX: `secrets status|init|reencrypt`
- TOFU `known_hosts` and atomic config writes with flock
- Key-only hosts: empty password serializes as JSON `null` (not `"***"`) in `vps list` / `show`
- Default tracing filter is `error` (agent-first clean stderr); override with `RUST_LOG` or `-v` (debug)
- Install with russh 0.62.2 for clean `cargo install --locked`


## Quick Start
### Install and run the first remote command

```bash
cargo install ssh-cli --locked
ssh-cli secrets init   # optional explicit master-key; auto-created on first secret write
printf %s 'demo-password-not-real' | ssh-cli vps add \
  --name prod \
  --host prod.example.com \
  --port 22 \
  --user admin \
  --password-stdin
ssh-cli connect prod
ssh-cli exec prod "hostname" --json
```


## Installation
### Choose the install path that matches your environment
- Prefer crates.io with lockfile: `cargo install ssh-cli --locked` (**0.4.2+** on crates.io; avoid **0.3.9** for SCP).
- Rebuild from a checkout: `cargo install --path . --locked`
- Do **not** use install without `--locked` unless you verified the crypto pins resolve cleanly.
- Force upgrade after a release: `cargo install ssh-cli --locked --force`
- Build musl with allocator feature when targeting Alpine: `--features musl-allocator`
- Require Rust MSRV 1.85.0 or newer


## Usage
### Register hosts then execute one-shot commands
- **Default at-rest encryption** (ChaCha20-Poly1305): auto `secrets.key` on first secret write; override via `SSH_CLI_SECRETS_KEY` / `_FILE` / keyring; manage with `ssh-cli secrets status|init|reencrypt`. Opt-out: `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` (tests only).
- Prefer `--password-stdin` / `--key` over argv secrets.
- Add password hosts with `vps add --password` or `--password-stdin`.
- Add key hosts with `vps add --key ~/.ssh/id_ed25519`.
- On key-only hosts, empty password fields serialize as JSON `null` in `vps list` / `show` (non-empty secrets mask as `"***"`).
- Mark active host with `connect <name>`.
- Run remote shells with `exec <vps> "<cmd>"`.
- Elevate with `sudo-exec` or `su-exec` when configured.
- Diagnose XDG paths with `vps doctor --json`.
- Export redacted inventory with `vps export`.


## Commands
### Product surface for humans and agents

| Command | Purpose |
|---|---|
| `ssh-cli vps add` | Register a host (password or key) |
| `ssh-cli vps list [--json]` | List hosts with secrets masked |
| `ssh-cli vps show <name> [--json]` | Show one host masked |
| `ssh-cli vps edit <name>` | Patch host fields |
| `ssh-cli vps remove <name>` | Delete host |
| `ssh-cli vps path` | Print `config.toml` path |
| `ssh-cli vps doctor [--json]` | Show XDG layer schema and paths |
| `ssh-cli vps export` | Export hosts (secrets redacted by default; empty secrets stay `""`, never fake `sshcli-enc:` ciphertext) |
| `ssh-cli vps import --file` | Import hosts from TOML |
| `ssh-cli connect <name>` | Write sibling `active` file |
| `ssh-cli exec <vps> <cmd>` | One-shot remote command |
| `ssh-cli sudo-exec <vps> <cmd>` | One-shot sudo with safe packing |
| `ssh-cli su-exec <vps> <cmd>` | One-shot `su -` elevation |
| `ssh-cli scp upload|download` | Regular files only (no `-r`/SFTP); flags `--timeout`, `--password-stdin`, `--key`, `--key-passphrase[-stdin]`, `--json` → `scp-transfer` schema (required `event: "scp-transfer"`); preserve mtime/mode |
| `ssh-cli tunnel ... --timeout-ms N [--json]` | Bounded local port forward; auth `--password-stdin` / `--key` / `--key-passphrase[-stdin]`; `--json` emits `tunnel_listening` after bind; post-bind deadline exits **0** (pre-bind timeout still **74**) |
| `ssh-cli health-check [<vps>] [--timeout N]` | Connectivity probe; optional `--timeout` ms; auth `--password-stdin` / `--key` / `--key-passphrase[-stdin]` |
| `ssh-cli secrets status|init|reencrypt` | Master-key and at-rest encryption (never prints key) |
| `ssh-cli completions <shell>` | Shell completion scripts |


## Environment Variables
### Overrides allowed for tests and locales

| Variable | Description | Example |
|---|---|---|
| `SSH_CLI_HOME` | Override base config directory | `/tmp/ssh-cli-test` |
| `SSH_CLI_LANG` | Override locale | `pt-BR` |
| `SSH_CLI_SECRETS_KEY` | Master key as 64 hex chars (encrypt at rest) | *(never log)* |
| `SSH_CLI_SECRETS_KEY_FILE` | Path to file with 64 hex master key | `~/.config/ssh-cli/secrets.key` |
| `SSH_CLI_USE_KEYRING` | Load/store master key in OS keyring | `1` |
| `SSH_CLI_ALLOW_PLAINTEXT_SECRETS` | Opt-out of default encryption (**tests only**) | `1` |
| `NO_COLOR` | Disable ANSI colors | `1` |
| `CLICOLOR_FORCE` | Force ANSI colors | `1` |
| `RUST_LOG` | Tracing filter override (default level is `error`) | `debug` |

- Prefer CLI flags over environment for production agent runs.
- Default tracing filter is `error` so agent stderr stays clean; set `RUST_LOG` only when debugging (or pass `-v` for debug).
- Never put host passwords in environment variables; use registry + stdin.
- Master-key env vars are for **encryption of secrets at rest**, not SSH passwords.


## Integration Patterns
### Wire agents with one-shot subprocesses only
- Invoke `ssh-cli` as a subprocess with explicit argv.
- Prefer `--json` or `--output-format json` for machine parsing.
- Parse stdout only; default log level is `error` so stderr stays silent for JSON pipelines — set `RUST_LOG` only to debug when needed.
- Map non-zero exits with sysexits semantics before retry.
- Store hosts once via `vps add` then call `exec` per task.
- Pass secrets through `--password-stdin` when argv history is risky.
- Read [INTEGRATIONS.md](INTEGRATIONS.md) for agent-specific notes.


## Exit Codes
### Sysexits-style codes agents must map before retry

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | General runtime error |
| `64` | Usage / invalid arguments |
| `65` | Data error (JSON/TOML/schema) |
| `66` | VPS or input file not found |
| `73` | Cannot create config/output |
| `74` | IO or SSH connection/timeout |
| `77` | Authentication rejected or host-key / sudo policy |
| `130` | SIGINT |
| `143` | SIGTERM |

- Prefer `--json` or auto JSON when stdout is not a TTY (`--output-format` overrides).
- Default tracing is `error`, so exit handling and JSON stdout stay free of INFO noise; use `RUST_LOG=debug` or `-v` only when diagnosing.
- Retry only on transient IO/timeout (`74`), never on auth (`77`) or usage (`64`).


## Performance
### Cold start and memory goals
- Target cold start under 100 ms on modern Linux hosts.
- Keep process memory far below a resident long-lived Node SSH process.
- Die after each command so RAM returns to the OS immediately.
- Avoid long-lived tunnels without `--timeout-ms`.


## Memory Requirements
### Plan capacity for multi-host registries
- Config TOML size grows with host count and path lengths.
- Output buffers respect `max_output_chars` per stream.
- Known_hosts file grows slowly with unique host:port pairs.
- No embedding model and no Node heap are required.


## Troubleshooting FAQ
### Fix common install and runtime failures
- Install fails on crypto RC drift: rerun with `--locked` or use release **0.4.1+** (russh 0.62.2) (`scripts/verify_install_resolve.sh`).
- Auth fails on key-only hosts: set `--key` on `vps add` or pass `--key` / `--password-stdin` to `exec`.
- Auth fails with passphrase keys: use `--key-passphrase-stdin`.
- Host key changed: confirm legitimacy then rerun with `--replace-host-key`.
- Command rejected as too long: raise `max_command_chars` or shorten the command.
- Config has encrypted secrets but no key: run `ssh-cli secrets init` or restore `secrets.key` / env master-key.
- sudo-exec disabled: remove `--disable-sudo` and set `disable_sudo=false` on the host.
- Unexpected stderr noise in JSON pipelines: default log level is already `error`; set `RUST_LOG` only to `debug` (or `-v`) when diagnosing.
- SCP from crates.io **0.3.9** fails or writes 0-byte remotes: upgrade to **0.4.1+** (wire fix); only regular files, not directories.
- SCP download fails mid-transfer: destination stays absent or previous file intact (partial uses `.ssh-cli.partial`).
- Redacted `vps export` on **0.4.0** wrote fake `sshcli-enc:` blobs for empty secrets: upgrade to **0.4.1+** (empty secrets stay empty strings).
- Tunnel emitted `ok: true` / `tunnel_listening` then process exit **74** when the post-bind deadline hit on **0.4.0**: upgrade to **0.4.1+** (post-bind deadline exits **0**; pre-bind timeout still **74**).
- macOS Gatekeeper blocks binary: run `xattr -d com.apple.quarantine /path/to/ssh-cli`.
- Permission denied on config: ensure `chmod 600` on the XDG `config.toml` and `secrets.key`.


## Contributing
- Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.
- Follow the bilingual documentation framework for every public doc change.


## Security
- Read [SECURITY.md](SECURITY.md) for private vulnerability reporting.
- Prefer stdin secret flags and key files over argv passwords.


## Changelog
- Read version history in [CHANGELOG.md](CHANGELOG.md).
- Do not paste release notes into this README.


## License
- Dual-licensed under MIT or Apache-2.0.
- See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and [LICENSE-APACHE](LICENSE-APACHE).
