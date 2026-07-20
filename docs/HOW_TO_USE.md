# How to Use ssh-cli

> Go from install to first remote command in under 60 seconds.

- Read this document in [Portuguese (pt-BR)](HOW_TO_USE.pt-BR.md).
- Return to [README.md](../README.md) for the full command map.
- Product line documented here: 0.5.2.


## Prerequisites
- Install Rust MSRV 1.85.0 or newer via rustup.
- Ensure network reachability to the target SSH host.
- Hold either a password or an OpenSSH private key for that host.
- Prefer a writable XDG config home for multi-host storage.
- Install with `cargo install ssh-cli --locked` (0.5.2+ on crates.io; avoid 0.3.9 for SCP).
- Do not rely on crates.io 0.3.9 for SCP: that release advertised transfer but the wire protocol was broken (0-byte remote files or timeouts). Use 0.5.2+.


## First Command in 60 Seconds
### Install, register, execute

```bash
cargo install ssh-cli --locked
# Primary-key is auto-created on first secret write; optional explicit init:
ssh-cli secrets init
ssh-cli vps add --name demo --host 203.0.113.10 --user ubuntu --key ~/.ssh/id_ed25519
ssh-cli exec demo "uname -a" --json
```

- Confirm exit code 0 and inspect JSON fields `stdout`, `stderr`, `exit_code`, `duration_ms`.
- An empty remote command string fails with technical message `empty command` (always English) and domain usage exit 64.
- Run `ssh-cli secrets status --json` and `ssh-cli doctor --json` (or `vps doctor --json`) when paths or encryption mode are unclear.
- Discover contracts: `ssh-cli schema` / `ssh-cli commands`.
- Register agent-auth hosts with `vps add --use-agent` (optional `--agent-socket`).
- Prefer `--password-stdin` over `--password` when registering password hosts.


## Core Commands
### Daily operator loop
- List hosts with `ssh-cli vps list --json`.
- Show one host with `ssh-cli vps show demo --json` (secrets masked).
- Patch fields with `ssh-cli vps edit demo --timeout 90000`.
- Mark active host with `ssh-cli connect demo`.
- Run privileged work with `ssh-cli sudo-exec demo "systemctl status nginx" --json` (safe `sh -c` packing).
- Elevate with `ssh-cli su-exec` when `su` password is stored on the host record.
- Transfer **regular files** with `ssh-cli scp upload demo ./app.tgz /tmp/app.tgz` (no directories / no `-r`). For directory trees use `ssh-cli sftp upload --recursive demo ./dir /tmp/dir`.
- Download with `ssh-cli scp download demo /var/log/app.log ./app.log`.
- Prefer agent JSON: `ssh-cli scp upload demo ./app.tgz /tmp/app.tgz --json` (schema `docs/schemas/scp-transfer.schema.json`; required `event: "scp-transfer"`).
- SCP flags match exec parity: `--timeout` (connect + transfer), `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json`.
- Missing local/remote file on SCP exits 66 with message `file not found: <path>` (path is canonical/normalized; no stacked `SCP:` prefixes).
- Failed download keeps the final path untouched: writes `{path}.ssh-cli.partial`, applies mode/times on the partial, then atomic rename.
- Upload streams in 32 KiB chunks (does not load the whole file into RAM).
- mtime/mode are preserved both directions automatically (remote `scp -tp` / `-fp`; no extra user flag).
- Manage primary-key with `ssh-cli secrets status|init|reencrypt` (never prints the key). Keyring may still accept the legacy `secrets-master-key` alias on read.
- `secrets init --json` / `secrets reencrypt --json` emit success events (`secrets-init`, `secrets-reencrypt`; schemas `docs/schemas/secrets-init.schema.json`, `docs/schemas/secrets-reencrypt.schema.json`); first secret write may set field `secrets_key_auto_created: true` on the same `vps-added` JSON document (never a second stdout event). See [docs/schemas/README.md](schemas/README.md).
- CRUD success JSON events when JSON is effective: `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import` (with field `secrets_key_auto_created` when a key is auto-created — one document). Catalog: [docs/schemas/README.md](schemas/README.md).


## Daemon
### There is no daemon
- Treat every invocation as birth-execute-die (one-shot).
- Never expect a background SSH worker from this project.
- Bound tunnels with required `--timeout-ms` so the process still exits.


## Advanced Patterns
### Fleet multi-host (bounded concurrency)
- Prefer `exec|sudo-exec|su-exec|scp|health-check --all` when the registry has more than one host — one process, concurrent sessions gated by `--max-concurrency N` (auto CPUs×RAM when omitted, clamp 1..=64).
- Parse batch JSON via `docs/schemas/*-batch.schema.json` (`health-check-batch`, `exec-batch`, `scp-batch`); envelope includes `max_concurrency`.
- Example: `ssh-cli --max-concurrency 8 health-check --all --json` then `ssh-cli exec --all 'hostname' --json`.
- Do **not** spawn one CLI process per host for fleet work when `--all` is available.

### Safer agent automation
- Feed secrets through stdin flags (`--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--key-passphrase-stdin`) instead of argv.
- Attach shell comments with `--description` for audit-friendly remote history.
- Disable elevation for untrusted tasks with `--disable-sudo`.
- Replace a legitimate host key only after human confirmation using `--replace-host-key` (TOFU).
- Export redacted inventory with `ssh-cli vps export -o hosts.toml` (default body is TOML, including non-TTY/pipe; non-empty secrets mask as `***` (`FIXED_MASK`); empty secrets stay `""`; never writes fake empty `sshcli-enc:` ciphertext) (EXP-001 / G-E2E-10). List/show empty password is JSON `null` — a different path from export. Help text matches this TOML-default behavior.
- Agent JSON export only with `ssh-cli vps export --json` → envelope `event: "vps-export"` (auto JSON non-TTY does **not** apply to `vps export`).
- `--include-secrets` requires `-o`/`--output` or `--i-understand-secrets-on-stdout` (pipe/stdout without ack is refused, exit 64).
- Import hosts with `ssh-cli vps import --file hosts.toml` (TOML EN keys or legacy PT aliases) or a JSON `vps-export` envelope; use `--allow-incomplete` for redacted/skeleton hosts missing full auth.
- `added_at` / `adicionado_em` are optional on import (serde defaults to now when omitted).
- Wire inventory uses schema v3: new writes serialize English keys (`name`, `port`, `username`, `password`, `added_at`, …); loads still accept legacy Portuguese aliases (`nome`, `porta`, `usuario`, `senha`, `adicionado_em`).
- Re-encrypt a plaintext inventory after upgrade: `ssh-cli secrets reencrypt`.
- Expect auto JSON when stdout is not a TTY unless `--output-format` is set (except `vps export`, which stays TOML unless `--json`).
- Expect empty password on key-only hosts as JSON `null` (not `"***"`); non-empty passwords mask as `***`; human text show uses "(não definida)" for empty.
- On `scp --json` failure, parse the JSON error envelope on **stderr** (`exit_code`, `message`), not human prose.
- Timeout values under 1000 ms warn on stderr (milliseconds, not seconds); password-like values on argv also warn — prefer `--*-stdin`.


## Configuration
### XDG multi-host registry
- Resolve config path with `ssh-cli vps path`.
- Expect atomic writes to `config.toml` mode 0600 (tempfile + fsync + flock).
- Expect sibling files `active`, `known_hosts`, and `secrets.key` beside the config.
- Override directory only for tests with `--config-dir`.
- Store timeout, max_command_chars, max_output_chars, sudo and su secrets per host.
- Default at-rest encryption (ChaCha20-Poly1305): secrets become `sshcli-enc:v1:…` blobs.
- Primary-key control is CLI/XDG only: `--secrets-key-file <PATH>`, `--use-keyring`, or XDG `secrets.key`. Keyring may still accept legacy `secrets-master-key` alias on read.
- `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` are **rejected fail-closed** (not a store).
- Tests-only plaintext opt-out: `--allow-plaintext-secrets` only (no env store).
- `vps doctor --json` reports paths, schema, host count, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, and `secrets_plaintext_opt_out` (JSON boolean).


## Subcommands Not Covered Above
- `health-check [--timeout <ms>]` probes connectivity and prints latency (`vps add --check` after register); override timeout when the host default is too long or short.
- Health-check auth parity (0.4.1+ / CLI-006): `--password-stdin` / `--key` / `--key-passphrase` / `--key-passphrase-stdin`.
- Default tracing level is error so JSON and tunnel stderr stay clean; use `-v` (debug) when diagnosing (ambient `RUST_LOG` is ignored).
- `tunnel` requires local port, remote host, remote port, and `--timeout-ms`.
- Tunnel `--bind` defaults to `127.0.0.1` (loopback); override only when you intentionally expose the listener.
- Optional `tunnel --json` emits structured `event: "tunnel_listening"` on stdout after the local bind (`docs/schemas/tunnel-listening.schema.json`); after the agent receives the event, the post-bind deadline ends with exit 0 (TUN-002); pre-bind timeout still 74.
- Tunnel auth parity (CLI-005): `--password-stdin` / `--key` / `--key-passphrase` / `--key-passphrase-stdin`.
- `completions` writes shell completion scripts to stdout.
- `su-exec` requires configured `su` password on the host record.
- `secrets` manages encryption primary-key without ever printing it.


## Exit codes (sysexits)

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General runtime failure (e.g. remote non-zero exit with `remote_exit_code` in JSON envelope) |
| 2 | Clap usage (invalid flags) |
| 64 (`EX_USAGE`) | Invalid argument / domain usage (includes empty command, refused `--include-secrets` without `-o` or ack, ACME permanent validation e.g. `invalidContact`) |
| 65 (`EX_DATAERR`) | Invalid TOML/JSON input data (`TomlDe` / JSON parse / schema incompatibility) |
| 66 (`EX_NOINPUT`) | VPS not found, no active VPS, or missing file (`file not found: <path>` on SCP) |
| 73 (`EX_CANTCREAT`) | Config write / create failure |
| 74 (`EX_IOERR`) | Connection/IO/timeout |
| 77 (`EX_NOPERM`) | Authentication failed / host-key policy / permission / sudo disabled |
| 130 | SIGINT |
| 143 | SIGTERM |

Product line: 0.5.2.


## Integration With AI Agents
- Load the skill package under `skills/ssh-cli-en/`.
- Prefer JSON output for tool parsing.
- Follow exit-code routing before retries (see README or [AGENTS.md](AGENTS.md)).
- Read [AGENTS.md](AGENTS.md) and [../INTEGRATIONS.md](../INTEGRATIONS.md).
- Event and payload shapes: [docs/schemas/README.md](schemas/README.md).
- Never log primary-key, host passwords, or decrypted secrets.
