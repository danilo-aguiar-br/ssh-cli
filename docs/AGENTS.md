# Agents Guide for ssh-cli

> Cut RAM waste from resident processes and keep multi-host SSH under agent control.

- Read this document in [Portuguese (pt-BR)](AGENTS.pt-BR.md).
- Pair with [../INTEGRATIONS.md](../INTEGRATIONS.md) and [../skills/ssh-cli-en/SKILL.md](../skills/ssh-cli-en/SKILL.md).
- Product line: 0.5.1.


## Why
### Replace long-lived Node SSH processes with a die-after-run binary
- Persistent long-lived SSH processes burn RAM while holding sockets idle.
- One host per daemon process multiplies process count for N servers.
- A single Rust binary with XDG multi-host storage collapses that sprawl.
- Agents gain deterministic JSON and sysexits without extra protocol overhead.


## Economy
### Measure the operational win
- Cold start targets stay under 100 ms for typical Linux hosts.
- Process memory returns to the OS after every command.
- No Node runtime tax and no permanent SSH manager process.
- One install serves Claude Code, Cursor, Windsurf, Codex, and shell agents.


## Sovereignty
### Keep credentials and host trust local
- Store hosts under XDG without `.env` sprawl.
- Prefer private keys and stdin secrets over chat-pasted passwords.
- Default at-rest encryption (ChaCha20-Poly1305 + auto `secrets.key`); manage with `secrets status|init|reencrypt`.
- Enforce TOFU known_hosts so silent MITM is harder.
- Disable elevation when a workflow must stay unprivileged.
- FORBIDDEN: log primary-key, host passwords, or decrypted secrets.


## Compatible Agents and Orchestrators
- Claude Code with the shipped skill package
- Cursor with shell or agent tools
- Windsurf shell tool
- Codex CLI shell tool
- OpenCode shell tool
- Aider, Continue, Gemini CLI, OpenHands, generic bash/zsh


## Agent Integration Details
### Imperative contract for authors
- REQUIRED: invoke `ssh-cli` as a subprocess and wait for exit (one-shot).
- REQUIRED: parse stdout JSON when `--json` or `--output-format json` is set (auto JSON when stdout is not a TTY).
- REQUIRED: treat stderr tracing as non-contract logs; do not parse stderr as success JSON.
- REQUIRED: when JSON errors mode is active (`--json` / effective JSON on scp|tunnel|global format), parse failure envelopes on **stderr** (`exit_code`, `message`, optional `remote_exit_code`) via `docs/schemas/error-envelope.schema.json`.
- REQUIRED: expect default tracing level error; set `RUST_LOG` or `-v` only when debugging.
- REQUIRED: register hosts with `vps add` before repeated remote work.
- REQUIRED: supply password or key; empty credentials are rejected at write time.
- REQUIRED: treat empty password in list/show JSON as `null` (key-only hosts); non-empty is masked `***`.
- REQUIRED: empty remote command fails with technical message `empty command` (always English) and domain usage exit 64.
- REQUIRED: pass `--timeout-ms` for every `tunnel` invocation.
- REQUIRED: treat `scp` as **regular files only** (no directories, no `-r`, no SFTP subsystem).
- REQUIRED: never depend on crates.io 0.3.9 for SCP; that wire was broken — require 0.5.1+.
- REQUIRED: parse SCP success with `docs/schemas/scp-transfer.schema.json` (`ok`, `event` (`scp-transfer`), `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`) on **stdout**.
- REQUIRED: missing SCP local/remote file exits 66 with message `file not found: <path>` (canonical/normalized path; no stacked `SCP:` prefixes).
- REQUIRED: `vps export` default body is TOML even on pipe/non-TTY; JSON agent envelope only with `vps export --json` → `event: "vps-export"` (auto JSON non-TTY does **not** apply to export).
- REQUIRED: redacted `vps export` empty secrets are empty strings, never `sshcli-enc:` ciphertext of empty (EXP-001).
- REQUIRED: `--include-secrets` requires `-o`/`--output` or `--i-understand-secrets-on-stdout`.
- REQUIRED: `vps import` accepts TOML (EN keys + legacy PT load aliases) or JSON `vps-export`; use `--allow-incomplete` for redacted/skeleton hosts.
- REQUIRED: `added_at` / `adicionado_em` are optional on import (serde defaults to now when omitted).
- REQUIRED: wire format schema v3 dual-read — serialize EN keys, load still accepts PT aliases (`nome`/`porta`/`usuario`/`senha`/…).
- REQUIRED: prefer secrets CLI flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` over env vars; prefer term primary-key; keyring may still accept legacy `secrets-master-key` alias on read.
- REQUIRED: `secrets init --json` / `secrets reencrypt --json` emit `secrets-init` / `secrets-reencrypt` (`docs/schemas/secrets-init.schema.json`, `docs/schemas/secrets-reencrypt.schema.json`); auto key may emit `secrets-key-auto-created`. Catalog: [docs/schemas/README.md](schemas/README.md).
- REQUIRED: on `tunnel --json`, wait for one stdout object with `event: "tunnel_listening"` (`docs/schemas/tunnel-listening.schema.json`) before using the local port; process stays alive until timeout or signal; after `tunnel_listening`, post-bind deadline ends with exit 0 (TUN-002); pre-bind timeout remains 74.
- REQUIRED: tunnel `--bind` defaults to `127.0.0.1` (loopback).
- ALLOWED: `tunnel` / `health-check` may use `--password-stdin` / `--key` / `--key-passphrase` / `--key-passphrase-stdin` (CLI-005/006 parity with exec/scp).
- ALLOWED: may pass `health-check --timeout <ms>` when host default timeout is too long or short.
- REQUIRED: timeout values under 1000 ms and password-like values on argv emit warn on stderr — do not parse those lines as a JSON error envelope.
- REQUIRED: prefer `--password-stdin` / `--key` over argv secrets.
- REQUIRED: install with `cargo install ssh-cli --locked` (or path install with pins).
- FORBIDDEN: assume a long-lived SSH connection across process runs.
- FORBIDDEN: reintroduce long-lived daemon packaging into this repository.
- FORBIDDEN: enable or emit product telemetry.
- FORBIDDEN: retry blindly on exit 64, 65, 66, or 77.
- FORBIDDEN: print or store primary-key material from `secrets` commands.
- FORBIDDEN: treat SCP directory trees or recursive `-r` as supported.


## Crate Integrations
- Publish consumers depend on the CLI contract, not an unstable library API.
- Pin library experiments to an exact crate version if linking `ssh_cli` as a lib.
- Prefer PATH-installed binary integration for agents.


## CRUD and JSON Contract
### Machine-readable operations
- List hosts: `ssh-cli vps list --json` returns an array of masked host objects.
- Show host: `ssh-cli vps show <name> --json` returns one masked host object.
- Doctor: `ssh-cli vps doctor --json` returns layer, paths, schema, host count, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out` (JSON boolean), telemetry false.
- Secrets: `ssh-cli secrets status --json` returns encryption mode without key material; `secrets init --json` → `event: "secrets-init"`; `secrets reencrypt --json` → `event: "secrets-reencrypt"`.
- CRUD success events when JSON is effective (`--json` / `--output-format json` / non-TTY auto JSON): `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import` (and `secrets-key-auto-created` when a key is auto-created).
- Exec family: `ssh-cli exec|sudo-exec|su-exec ... --json` returns stdout, stderr, exit_code, truncation flags, duration_ms.
- Health: `ssh-cli health-check [--timeout <ms>] [--password-stdin|--key|--key-passphrase[-stdin]] --json` returns name, status, latency_ms.
- SCP: `ssh-cli scp upload|download <vps> <local> <remote> --json` returns transfer success on stdout (`scp-transfer.schema.json` with required `event: "scp-transfer"`); failures use error envelope on stderr; missing file → exit 66 `file not found: <path>` (canonical/normalized path).
- SCP operational facts: require 0.5.1+; upload streams 32 KiB; download writes `{path}.ssh-cli.partial` then renames; mtime/mode preserved both directions.
- Tunnel: `ssh-cli tunnel <vps> <local_port> <remote_host> <remote_port> --timeout-ms <ms> [--bind 127.0.0.1] [--password-stdin|--key|--key-passphrase[-stdin]] --json` emits `tunnel_listening` on stdout after bind; `--bind` defaults to `127.0.0.1`; post-bind deadline exits 0; pre-bind timeout remains 74.
- Export: default `ssh-cli vps export` body is TOML (even pipes); empty secrets serialize as `""` (never `sshcli-enc:`). Use `vps export --json` for agent envelope `event: "vps-export"`. `--include-secrets` needs `-o` or `--i-understand-secrets-on-stdout`.
- Import: `ssh-cli vps import --file <path> [--allow-incomplete]` accepts TOML (EN serialize / PT load aliases) or JSON `vps-export`; `added_at` / `adicionado_em` optional (default now).
- Empty password fields serialize as JSON `null`; non-empty secrets mask as `***`.
- Validate payloads against schemas under `docs/schemas/`; index: [docs/schemas/README.md](schemas/README.md).


## Exit Code Routing
- Exit 0 means success.
- Exit 1 means general runtime failure; inspect stderr.
- Exit 64 means usage or argument error (including empty command); fix argv, do not retry.
- Exit 65 (`TomlDe` / JSON / schema) means parse/data error; fix input payload.
- Exit 66 means missing VPS or file (`file not found: <path>` on SCP); register or correct the name/path.
- Exit 73 means config write failure; check permissions and disk.
- Exit 74 means IO/SSH connection failure; network retry may help.
- Exit 77 means auth failure or host-key policy; try `--key` / `--password-stdin` / passphrase stdin; do not blind-retry.
- Exit 130/143 means signal termination.


## Retry Strategy
- Retry at most twice on exit 74 with backoff.
- Never retry on 64, 65, 66, 77 without changing inputs.
- Shorten or split commands when exit indicates max_command_chars rejection.
- Confirm host key changes with a human before `--replace-host-key`.
