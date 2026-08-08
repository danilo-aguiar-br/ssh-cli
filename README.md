# ssh-cli

> **0.5.4** — security and agent-native release. Fixes a remote pre-auth DoS in the SSH banner path (A1), stops server-sent setuid bits landing on downloaded files (A3), closes the world-readable window on ACME/mTLS private keys (A2), and adds payload-shaping flags (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) applied before serialization. BREAKING: partial multi-host failure now exits **1** (was 65); a non-loopback `--bind` requires `--i-accept-network-exposure`. New `tunnel_closed` event.


- Historical note: **0.4.2** closed TUN-003 / IO-010; **0.5.0** was the EN/API rename + secrets force-init reencrypt; current product line is **0.5.4** (tunnel `--reverse` / `--socks5` / `--remote-socket`, eight payload-shaping flags, `tunnel_closed`, partial multi-host exit **1**, security fixes A1–A3). **0.5.3** brought export/import agent roundtrip, wire schema v3 dual-read, secrets CLI flags and tunnel `--bind`, and also closed **G1–G19**: SFTP integrity (no zero-byte truncate), crate-scoped graduated verbosity `-v`/`-vv`/`-vvv` (no russh password dump), SETSTAT `atime`+`mtime`, batch cancel cardinality, single-step `exec` JSON, SCP `sync_data`, E2E SFTP checksum E17/E18.

[![docs.rs](https://img.shields.io/docsrs/ssh-cli)](https://docs.rs/ssh-cli)
[![crates.io](https://img.shields.io/crates/v/ssh-cli)](https://crates.io/crates/ssh-cli)
[![License](https://img.shields.io/crates/l/ssh-cli)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85.0-orange)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-blue)](https://www.rust-lang.org)
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
- Migrate from 0.3.3+ in [docs/MIGRATION.md](docs/MIGRATION.md) (target line **0.5.4**).
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
- Bounded tunnel via mandatory `--timeout-ms`; optional `--bind` (default `127.0.0.1`)
- Wire **schema v3**: serialize English TOML keys; dual-read EN + legacy PT aliases on load
- `vps export` body **follows the resolved output format**: JSON whenever stdout is not a TTY, which is every agent invocation and holds even when `-o` names a `.toml` file; a TOML body requires `--output-format text`; redacted by default; empty secrets stay `""`; non-empty redacted secrets mask as `***` (`FIXED_MASK`); `--include-secrets` to pipe/non-TTY needs `-o`/`--output` or `--i-understand-secrets-on-stdout`
- `vps import` accepts TOML (EN keys + PT aliases) **or** JSON `vps-export` envelopes; redacted skeletons need `--allow-incomplete`
- SCP upload and download of **regular files only** (no recursive directories on SCP; first solid wire fix in **0.4.0**; prefer **0.5.3+** — avoid crates.io 0.3.9 SCP); SCP download propagates `sync_data` failure before atomic rename (G9)
- **SFTP** subsystem (`ssh-cli sftp`, prefer **0.5.3+** for integrity): upload/download (optional `--recursive` trees, symlink no-follow), `ls`/`mkdir`/`rmdir`/`rm`/`stat`/`rename`; JSON events `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch`
- Graduated verbosity `-v`/`-vv`/`-vvv` (info/debug/trace) always crate-scoped (`warn,ssh_cli=*`) so russh never dumps passwords (G2/G14)
- SFTP upload preserves payload bytes (G1); SETSTAT `atime`+`mtime` (G3); fail-closed metadata (G4); permission mask `SFTP_PERM_MASK` (G12)
- Batch cancel pads cancelled remainder so results length matches input (G5/G17)
- `exec --json` single-step emits exactly one NDJSON success object (G8)
- SCP flag parity with exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` (contract `docs/schemas/scp-transfer.schema.json`)
- SCP download writes `{path}.ssh-cli.partial` then atomic rename; upload streams 32 KiB chunks; remote mtime/mode preservation is best-effort and reported as `mtime_preserved` / `durable`
- SCP JSON success requires `event: "scp-transfer"` (0.4.1 IO-009); missing remote → `file not found: <path>` exit **66**
- `tunnel --json` emits structured `tunnel_listening` after local bind
- Tunnel post-bind deadline exits **0** after `tunnel_listening` (pre-bind timeout still **74**) (0.4.1 TUN-002)
- Tunnel and health-check auth parity with exec/scp: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` as applicable (0.4.1 CLI-005/006)
- Redacted `vps export` never emits `sshcli-enc:` for empty password — empty secrets serialize as empty strings (0.4.1 EXP-001)
- Health-check latency probe with optional `--timeout`
- Shell completions for bash zsh fish powershell
- Secrets via stdin flags to avoid argv leaks
- **Default at-rest encryption** (ChaCha20-Poly1305) with auto XDG `secrets.key`; CLI flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` preferred over env
- Master-key UX: `secrets status|init|reencrypt` with `--json` events `secrets-init` / `secrets-reencrypt`; first secret write folds `secrets_key_auto_created: true` into the same `vps-added` JSON (one document)
- TOFU `known_hosts` and atomic config writes with flock
- Key-only hosts: empty password serializes as JSON `null` (not `"***"`) in `vps list` / `show`
- Default tracing filter is `error` (agent-first clean stderr); ambient `RUST_LOG` is ignored (use `-v`/`-vv`/`-vvv`)
- Install with russh 0.62.5 for clean `cargo install --locked`


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
- Prefer crates.io with lockfile: `cargo install ssh-cli --locked` (**0.5.3+** preferred; avoid **0.3.9** for SCP).
- Rebuild from a checkout: `cargo install --path . --locked`
- Do **not** use install without `--locked` unless you verified the crypto pins resolve cleanly.
- Force upgrade after a release: `cargo install ssh-cli --locked --force`
- Build musl with allocator feature when targeting Alpine: `--features musl-allocator`
- Require Rust MSRV 1.85.0 or newer


## Features
### Cargo feature flags
| Feature | Default | Description |
|---------|---------|-------------|
| `ssh-real` | yes | Real SSH via `russh` + `aws-lc-rs` (compression `none` only) |
| `tls` | yes | rustls ≥0.23.18 + `aws_lc_rs`: SSH-over-TLS, mTLS, ACME |
| `musl-allocator` | no | `mimalloc` as global allocator (binary; useful on musl/Alpine) |

- Install path always enables defaults (`ssh-real` + `tls`).
- Disable real SSH / TLS only for dependency diagnosis: `cargo build --no-default-features`.
- docs.rs builds with `all-features = true` and `--cfg docsrs` (see `Cargo.toml` `[package.metadata.docs.rs]`).

### Crypto policy (G-TLS)
- Default transport is **SSH-2** on plain TCP (`russh` + **aws-lc-rs**). Host keys use **TOFU** under XDG (`known_hosts`).
- Optional **SSH-over-TLS** (`vps add --tls`, record fields `tls` / `tls_sni` / mTLS paths): rustls handshake then SSH on the TLS stream. Binary installs `CryptoProvider` (`aws_lc_rs`) once in `main`.
- **mTLS / ACME:** `ssh-cli tls mtls …` and `ssh-cli tls acme …` store material under XDG `tls/` (no product env for cert storage).
- No OpenSSL / `native-tls` / dual `ring` provider. SSH channel compression forced to **`none`**.
- ACME validation / `invalidContact` / 4xx problem types → exit **64** non-retryable (G-E2E-01); rate limits remain transient exit **74**.
- Details: [SECURITY.md](SECURITY.md#transport--crypto-policy-g-tls).


## Targets
### Platforms covered by docs.rs metadata
- `x86_64-unknown-linux-gnu` (default)
- `x86_64-apple-darwin`, `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `aarch64-unknown-linux-musl`
- See [docs/CROSS_PLATFORM.md](docs/CROSS_PLATFORM.md) for runtime notes.


## Usage
### Register hosts then execute one-shot commands
- **Default at-rest encryption** (ChaCha20-Poly1305): auto `secrets.key` on first secret write; prefer CLI flags `--secrets-key-file`, `--use-keyring`, `--allow-plaintext-secrets` (or XDG `secrets.key`); manage with `ssh-cli secrets status|init|reencrypt`. `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` are **rejected fail-closed** (not a store). Opt-out for tests only via `--allow-plaintext-secrets`.
- Prefer `--password-stdin` / `--key` over argv secrets.
- Add password hosts with `vps add --password` or `--password-stdin`.
- Add key hosts with `vps add --key ~/.ssh/id_ed25519`.
- On key-only hosts, empty password fields serialize as JSON `null` in `vps list` / `show` (non-empty secrets mask as `"***"`).
- Mark active host with `connect <name>`.
- Run remote shells with `exec <vps> "<cmd>"`.
- Elevate with `sudo-exec` or `su-exec` when configured.
- Diagnose XDG paths with `doctor --json` (or `vps doctor --json`).
- Discover contracts with `ssh-cli schema` / `ssh-cli commands`.
- Export redacted inventory with `vps export` (JSON envelope on any non-TTY stdout; add `--output-format text` for a TOML body).


## Commands
### Product surface for humans and agents

| Command | Purpose |
|---|---|
| `ssh-cli vps add` | Register a host (password **or** key **or** `--use-agent` / `--agent-socket`) |
| `ssh-cli vps list [--json]` | List hosts with secrets masked |
| `ssh-cli vps show <name> [--json]` | Show one host masked |
| `ssh-cli vps edit <name>` | Patch host fields |
| `ssh-cli vps remove <name>` | Delete host |
| `ssh-cli vps path` | Print `config.toml` path |
| `ssh-cli vps doctor [--json]` | Show XDG layer schema and paths |
| `ssh-cli doctor [--json]` | Root alias of `vps doctor` (G-E2E-03) |
| `ssh-cli schema [NAME]` | Emit embedded JSON Schema catalog or one schema body (G-E2E-02) |
| `ssh-cli commands` | Emit full command tree as JSON (agent discovery) |
| `ssh-cli vps export` | Export hosts in the **resolved output format**: JSON on any non-TTY stdout, even into a `.toml` filename; `--output-format text` for a TOML body; secrets redacted by default; empty secrets stay `""` (never fake `sshcli-enc:`); `--include-secrets` to pipe/non-TTY requires `-o`/`--output` or `--i-understand-secrets-on-stdout` |
| `ssh-cli vps import --file` | Import hosts from **TOML** (EN keys + PT aliases) **or** JSON `vps-export` envelope; redacted skeletons need `--allow-incomplete` |
| `ssh-cli connect <name>` | Write sibling `active` file |
| `ssh-cli exec <vps> <cmd>` | One-shot remote command |
| `ssh-cli exec --all '<cmd>'` | **Bounded concurrent** remote command on every registered host (`exec-batch` JSON) |
| `ssh-cli sudo-exec <vps> <cmd>` / `--all` | One-shot sudo with safe packing (fleet with `--all`) |
| `ssh-cli su-exec <vps> <cmd>` / `--all` | One-shot `su -` elevation (fleet with `--all`) |
| `ssh-cli scp upload|download` | Regular files only (no `-r` on SCP); flags `--timeout`, `--password-stdin`, `--key`, `--key-passphrase[-stdin]`, `--use-agent`, `--json` → `scp-transfer` schema; missing remote → exit **66**; **`--all`** → `scp-batch` |
| `ssh-cli sftp upload\|download\|ls\|mkdir\|rmdir\|rm\|stat\|rename` | SFTP v3 subsystem (**0.5.3** integrity: no zero-byte truncate); `--recursive` trees (no symlink follow); auth parity with scp; JSON `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch` |
| `ssh-cli tunnel ... --timeout-ms N [--bind ADDR] [--json]` | Bounded local port forward; `--bind` default `127.0.0.1`; auth `--password-stdin` / `--key` / `--key-passphrase[-stdin]`; `--json` emits `tunnel_listening` after bind; post-bind deadline exits **0** (pre-bind timeout still **74**); concurrent accepts gated by `--max-concurrency` |
| `ssh-cli tunnel <vps> <port> --socks5 --timeout-ms N` | SOCKS5 proxy (RFC 1928, no-auth `CONNECT`); each connection opens one SSH channel; host names resolve on the remote side; `BIND`/`UDP ASSOCIATE` answered `0x07` |
| `ssh-cli tunnel <vps> <port> --remote-socket PATH --timeout-ms N` | Forwards a local port to a Unix domain socket on the remote host; `PATH` must be absolute (it names the **server's** filesystem) |
| `ssh-cli tunnel <vps> <port> <bind> <rport> --reverse --timeout-ms N` | Server listens on `<bind>:<rport>` and delivers back to local `<port>`; `<rport> 0` lets the server allocate and report it in `local_port`; a non-loopback `<bind>` requires `--i-accept-network-exposure` |
| `ssh-cli --dry-run <destructive command>` | Prints the plan and exits without executing (`vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt`); rejected with exit **64** on any other command |
| `ssh-cli health-check [<vps>] [--timeout N]` / `--all` | Connectivity probe; optional `--timeout` ms; auth `--password-stdin` / `--key` / `--key-passphrase[-stdin]`; **`--all`** → fleet probe (`health-check-batch`) |
| `ssh-cli secrets status|init|reencrypt` | Master-key and at-rest encryption (never prints key); `--json` emits `secrets-init` / `secrets-reencrypt`; first secret write folds `secrets_key_auto_created: true` into the same `vps-added` JSON (one document); flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` |
| `ssh-cli completions <shell>` | Shell completion scripts |
| `ssh-cli locale show\|set\|clear` | UI language resolution (`show` default; `set` persists XDG `lang`; `clear` removes preference) |
| `ssh-cli tls provider` | rustls `CryptoProvider` status (`aws_lc_rs`) |
| `ssh-cli tls paths` | XDG TLS directory layout |
| `ssh-cli tls mtls list\|import\|show\|remove` | mTLS identity store under XDG `tls/mtls/` |
| `ssh-cli tls acme account create\|show` | ACME account lifecycle |
| `ssh-cli tls acme issue\|complete\|status\|list` | ACME DNS-01 cert lifecycle |
| `ssh-cli --max-concurrency N …` | Global cap (1..=64) for multi-host fan-out and tunnel forwards (auto CPUs×RAM formula when omitted) |
| `ssh-cli -v` / `-vv` / `-vvv` | Graduated verbosity: info / debug / trace; always crate-scoped (`warn,ssh_cli=*`); ambient `RUST_LOG` ignored (G2/G14) |


## Configuration (CLI-only product store)
### Product knobs are flags and XDG — not `SSH_CLI_*` env stores

| Control | How | Example |
|---|---|---|
| Config directory | `--config-dir` (else XDG/`directories`) | `ssh-cli --config-dir /tmp/ssh-cli-test vps list` |
| Language | `--lang` or `ssh-cli locale set <code>` (XDG `lang`) | `ssh-cli --lang pt-BR …` |
| Output format | `--json` / `--output-format json\|text` | `ssh-cli exec h uptime --json` |
| Concurrency | `--max-concurrency N` (1..=64; auto formula when omitted) | `ssh-cli --max-concurrency 8 exec --all id --json` |
| Verbosity | `-v` / `-vv` / `-vvv` (info / debug / trace; crate-scoped) | `ssh-cli -vv exec h id --json` |
| Primary-key | `--secrets-key-file`, `--use-keyring`, or XDG `secrets.key` | `ssh-cli --secrets-key-file ./k secrets status` |
| Plaintext opt-out | `--allow-plaintext-secrets` (**tests only**) | `ssh-cli --allow-plaintext-secrets …` |

### Fail-closed secrets env (not a store)
| Variable | Behavior |
|---|---|
| `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` | **Rejected** if present — use XDG `secrets.key`, `--secrets-key-file`, or `--use-keyring` |

### OS / host boundary (detection only — not product config store)
| Variable | Role |
|---|---|
| `HOME` | OS home for XDG path resolution |
| `TERM` / `NO_COLOR` / `CLICOLOR_FORCE` | Terminal / color capability |
| `CI` / Flatpak-related markers | Runtime detection (`vps doctor` `runtime.*`) |
| `RUST_LOG` | **Ignored** by product (not a config store); use `-v`/`-vv`/`-vvv` |

- Default tracing filter is `error` so agent stderr stays clean; pass `-v`/`-vv`/`-vvv` for info/debug/trace (crate-scoped; ambient `RUST_LOG` is ignored).
- Never put host passwords in environment variables; use registry + stdin.
- Product does **not** read `SSH_CLI_HOME`, `SSH_CLI_LANG`, `SSH_CLI_FORCE_TEXT`, or `SSH_CLI_MAX_CONCURRENCY` as config stores.


## Integration Patterns
### Wire agents with one-shot subprocesses only
- Invoke `ssh-cli` as a subprocess with explicit argv.
- Prefer `--json` or `--output-format json` for machine parsing.
- Parse stdout only; default log level is `error` so stderr stays silent for JSON pipelines — pass `-v`/`-vv`/`-vvv` when diagnosing (crate-scoped; ambient `RUST_LOG` is ignored).
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
- Default tracing is `error`, so exit handling and JSON stdout stay free of INFO noise; use `-v`/`-vv`/`-vvv` only when diagnosing (crate-scoped; ambient `RUST_LOG` is ignored).
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
- Install fails on crypto RC drift: rerun with `--locked` or use release **0.5.3+** (russh 0.62.5) (`scripts/verify_install_resolve.sh`).
- Auth fails on key-only hosts: set `--key` on `vps add` or pass `--key` / `--password-stdin` to `exec` (rejected auth exits **77**).
- Auth fails with passphrase keys: use `--key-passphrase-stdin` (exit **77** on reject).
- Host key changed: confirm legitimacy then rerun with `--replace-host-key`.
- Command rejected as too long: raise `max_command_chars` or shorten the command.
- Config has encrypted secrets but no key: run `ssh-cli secrets init` or restore `secrets.key` / env master-key / `--secrets-key-file`.
- sudo-exec disabled: remove `--disable-sudo` and set `disable_sudo=false` on the host.
- Unexpected stderr noise in JSON pipelines: default log level is already `error`; pass `-v`/`-vv`/`-vvv` when diagnosing (crate-scoped; ambient `RUST_LOG` is ignored).
- Need more logs without password leak: use `-v`/`-vv`/`-vvv` (crate-scoped `warn,ssh_cli=*`; G2/G14) — never rely on ambient `RUST_LOG` (ignored).
- Ambient `RUST_LOG` ignored: product filter is CLI-only; setting `RUST_LOG=debug` has no effect.
- SFTP upload wrote 0-byte remote on pre-**0.5.3**: upgrade to **0.5.3+** (G1 integrity fix).
- SCP from crates.io **0.3.9** fails or writes 0-byte remotes: upgrade to **0.5.3+** (wire fix since 0.4.0; SFTP integrity fix in 0.5.3); only regular files, not directories.
- SCP remote missing: message is `file not found: <path>` and exit **66** (prefer **0.5.3+**; 0.4.2 IO-010).
- SCP download fails mid-transfer: destination stays absent or previous file intact (partial uses `.ssh-cli.partial`).
- Redacted `vps export` on **0.4.0** wrote fake `sshcli-enc:` blobs for empty secrets: upgrade to **0.5.3+** (empty secrets stay empty strings since 0.4.1).
- Tunnel emitted `ok: true` / `tunnel_listening` then process exit **74** when the post-bind deadline hit on **0.4.0**: upgrade to **0.5.3+** (post-bind deadline exits **0** since 0.4.1; pre-bind timeout still **74**).
- Import bad TOML: parse errors map to exit **65** (`TomlDe` / data error).
- Import redacted skeleton without secrets: pass `--allow-incomplete`.
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
