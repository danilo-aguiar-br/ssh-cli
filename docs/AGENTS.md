# Agents Guide for ssh-cli

> **0.5.4** — security and agent-native release. Fixes a remote pre-auth DoS in the SSH banner path (A1), stops server-sent setuid bits landing on downloaded files (A3), closes the world-readable window on ACME/mTLS private keys (A2), and adds payload-shaping flags (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) applied before serialization. BREAKING: partial multi-host failure now exits **1** (was 65); a non-loopback `--bind` requires `--i-accept-network-exposure`. New `tunnel_closed` event.


> **G-E2E-16:** Prefer GraphRAG `list` / `read` by exact memory name over `hybrid-search` under load.
>
> **G-E2E-04 / G8 wire:** REQUIRED one JSON document per one-shot success on the data path.
> FORBIDDEN: parse multi-line NDJSON dual events as the success data path.
> Field `secrets_key_auto_created` (when present) lives on the **same** `vps-added` document — never a second stdout event.
> `exec --json` single-step emits exactly **one** object (G8).
>
> **Discovery:** `ssh-cli commands`, `ssh-cli schema`, `ssh-cli doctor` (root alias of `vps doctor`).
>
> Cut RAM waste from resident processes and keep multi-host SSH under agent control.

- Read this document in [Portuguese (pt-BR)](AGENTS.pt-BR.md).
- Pair with [../INTEGRATIONS.md](../INTEGRATIONS.md) and [../skills/ssh-cli-en/SKILL.md](../skills/ssh-cli-en/SKILL.md).
- Product line: 0.5.4.


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


## Command inventory (full tree)

All 47 leaves are written out in full, not in brace notation: an agent that greps for
`tls acme account create` must find that exact string. Discover the same tree at runtime
with `ssh-cli commands` (`event: "commands"`).

| Surface | Commands |
| --- | --- |
| `vps` | `vps add` `vps list` `vps remove` `vps edit` `vps show` `vps path` `vps doctor` `vps export` `vps import` |
| Session | `connect` |
| Exec | `exec` `sudo-exec` `su-exec` |
| `scp` | `scp upload` `scp download` (regular files only) |
| `sftp` | `sftp upload` `sftp download` `sftp ls` `sftp mkdir` `sftp rmdir` `sftp rm` `sftp stat` `sftp rename` |
| Network | `tunnel` (four modes: local, `--reverse`, `--socks5`, `--remote-socket`) `health-check` |
| `secrets` | `secrets status` `secrets init` `secrets reencrypt` |
| Discovery | `completions` `commands` `schema` `doctor` (root alias of `vps doctor`) |
| `locale` | `locale show` `locale set` `locale clear` |
| `tls` | `tls provider` `tls paths` |
| `tls mtls` | `tls mtls list` `tls mtls import` `tls mtls show` `tls mtls remove` |
| `tls acme` | `tls acme account create` `tls acme account show` `tls acme issue` `tls acme complete` `tls acme status` `tls acme list` |

### Global flags of note
- `--lang`, `-v`/`-vv`/`-vvv` (G14 graduated; G2 crate-scoped `warn,ssh_cli=*`), `-q`, `--config-dir`, `--no-color`, `--output-format`, `--json`
- `--disable-sudo`, `--replace-host-key`
- `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- `--timeout`, `--max-concurrency`, `--fail-fast`, `--scp-file-concurrency`

### Payload shaping (0.5.4) — spend no tokens on data you will discard
- Eight global flags shape the response **before** serialization, so the oversized envelope is never built.
- `--select <PATHS>` (alias `--fields`) keeps only these dotted paths in each record.
- `--filter key=value` | `key!=value` | `key~substring` keeps matching records; repeatable, combined with AND.
- A malformed predicate is rejected at parse time rather than silently matching nothing, so a typo can never be mistaken for an empty result.
- `--limit N` emits at most N records and is distinct from per-command query limits.
- `--sort <PATH>` sorts ascending by dotted path, comparing numbers numerically.
- `--dedupe-by <PATH>` drops later records repeating that path's value.
- `--count-only` replaces the record collection with `{"count": N}`, counted after all filtering.
- `--truncate-content <CHARS>` shortens long strings by **characters**, never bytes, so UTF-8 stays valid.
- `--max-output-bytes <BYTES>` caps the envelope by dropping trailing records, never by slicing the JSON text.
- REQUIRED: prefer these flags over piping stdout through an external JSON tool. Piping pays the full token cost first and shrinks the payload afterwards; the flag never writes it.

### Refusal and rehearsal flags
- `--no-input` refuses to read stdin and fails fast instead of blocking on an absent human.
- REQUIRED: pass `--no-input` in any unattended run, because a prompt with no operator is an indefinite hang, not an error.
- `--dry-run` prints the plan for a destructive operation and exits without executing it.
- `--dry-run` is accepted only by `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init` and `secrets reencrypt`.
- Anywhere else `--dry-run` is rejected with exit **64** instead of accepted and ignored, so a rehearsal can never be mistaken for a no-op that already ran.


## Agent Integration Details
### Imperative contract for authors
- REQUIRED: invoke `ssh-cli` as a subprocess and wait for exit (one-shot).
- REQUIRED: parse stdout JSON when `--json` or `--output-format json` is set (auto JSON when stdout is not a TTY).
- REQUIRED: parse **exactly one** JSON object on success paths (G8 / G-E2E-04) — never multi-line dual events.
- REQUIRED: treat stderr tracing as non-contract logs; do not parse stderr as success JSON.
- REQUIRED: when JSON errors mode is active (`--json` / effective JSON on scp|sftp|tunnel|global format), parse failure envelopes on **stderr** (`exit_code`, `message`, optional `remote_exit_code`) via `docs/schemas/error-envelope.schema.json`.
- REQUIRED: expect default tracing level error; use `-v` / `-vv` / `-vvv` only when debugging (info/debug/trace; always crate-scoped allowlist — G2/G14).
- FORBIDDEN: relying on ambient `RUST_LOG` — it is ignored; use `-v`/`-vv`/`-vvv` only.
- REQUIRED: register hosts with `vps add` before repeated remote work (auth: password **or** key **or** `--use-agent` / `--agent-socket`).
- REQUIRED: supply password or key; empty credentials are rejected at write time.
- REQUIRED: treat empty password in list/show JSON as `null` (key-only hosts); non-empty is masked `***`.
- REQUIRED: empty remote command fails with technical message `empty command` (always English) and domain usage exit 64.
- REQUIRED: pass `--timeout-ms` for every `tunnel` invocation.
- REQUIRED: treat `scp` as **regular files only** (no directories, no `-r`). For trees / remote FS ops use `sftp` (`upload|download --recursive`, `ls`, `mkdir`, `rm`, `stat`, `rename`).
- REQUIRED: prefer product line **0.5.3+** for SFTP — G1 fixed upload truncation to zero bytes; verify transfers with destination `sha256` (G15), not client-reported sizes alone.
- REQUIRED: never depend on crates.io 0.3.9 for SCP; that wire was broken — require 0.5.3+.
- REQUIRED: parse SCP success with `docs/schemas/scp-transfer.schema.json` (`ok`, `event` (`scp-transfer`), `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`) on **stdout**.
- REQUIRED: missing SCP local/remote file exits 66 with message `file not found: <path>` (canonical/normalized path; no stacked `SCP:` prefixes).
- REQUIRED: `vps export` body follows the resolved output format, so an agent, whose stdout is never a TTY, gets the JSON envelope `event: "vps-export"` even without `--json` and even when `-o` names a `.toml` file; a TOML body requires `--output-format text`. Auto JSON non-TTY DOES apply to export.
- REQUIRED: redacted `vps export` empty secrets are empty strings, never `sshcli-enc:` ciphertext of empty (EXP-001).
- REQUIRED: `--include-secrets` requires `-o`/`--output` or `--i-understand-secrets-on-stdout`.
- REQUIRED: `vps import` accepts TOML (EN keys + legacy PT load aliases) or JSON `vps-export`; use `--allow-incomplete` for redacted/skeleton hosts.
- REQUIRED: `added_at` / `adicionado_em` are optional on import (serde defaults to now when omitted).
- REQUIRED: wire format schema v3 dual-read — serialize EN keys, load still accepts PT aliases (`nome`/`porta`/`usuario`/`senha`/…).
- REQUIRED: prefer secrets CLI flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` over env vars; prefer term primary-key; keyring may still accept legacy `secrets-master-key` alias on read.
- REQUIRED: `secrets init --json` / `secrets reencrypt --json` emit `secrets-init` / `secrets-reencrypt` (`docs/schemas/secrets-init.schema.json`, `docs/schemas/secrets-reencrypt.schema.json`); first secret write may set `secrets_key_auto_created: true` on the same success JSON (one document). Catalog: [docs/schemas/README.md](schemas/README.md).
- REQUIRED: on `tunnel --json`, wait for one stdout object with `event: "tunnel_listening"` (`docs/schemas/tunnel-listening.schema.json`) before using the local port; process stays alive until timeout or signal; after `tunnel_listening`, post-bind deadline ends with exit 0 (TUN-002); pre-bind timeout remains 74.
- REQUIRED: tunnel `--bind` defaults to `127.0.0.1` (loopback).
- ALLOWED: `tunnel` / `health-check` may use `--password-stdin` / `--key` / `--key-passphrase` / `--key-passphrase-stdin` (CLI-005/006 parity with exec/scp).
- ALLOWED: may pass `health-check --timeout <ms>` when host default timeout is too long or short.
- REQUIRED: prefer multi-host fan-out for fleet work — `exec|sudo-exec|su-exec|scp|sftp|health-check --all` **or** `--hosts a,b,c` runs **bounded concurrent** sessions (`Semaphore` + `JoinSet`), not one host per process spawn. Batch JSON applies to both multi modes (even if `--hosts` lists one name).
- REQUIRED: a third selector exists on the exec family only — `exec|sudo-exec|su-exec --tags t1,t2` addresses every host carrying any of those tags (`vps add --tag`). `--all`, `--hosts` and `--tags` are mutually exclusive; clap rejects any pair. `scp`, `sftp` and `health-check` accept `--all` and `--hosts` but **not** `--tags`.
- REQUIRED: parse multi-host JSON via batch schemas: `health-check-batch` / `exec-batch` / `scp-batch` / `sftp-batch` (`docs/schemas/*-batch.schema.json`); field `max_concurrency` is present in the envelope.
- ALLOWED: cap fan-out with global `--max-concurrency N` (1..=64; auto = CPUs×4 vs free RAM/2 / 16 MiB, clamp 1..=64). Same gate limits tunnel accept forwards.
- FORBIDDEN: assume sequential multi-host by default when `--all` is available — wall-clock is dominated by SSH RTT; concurrent sessions are the product modus operandi.
- REQUIRED: multi-file SCP/SFTP cancel fills cancelled remainder so `results.len() == input.len()` (G5/G17).
- REQUIRED: timeout values under 1000 ms and password-like values on argv emit warn on stderr — do not parse those lines as a JSON error envelope.
- REQUIRED: prefer `--password-stdin` / `--key` over argv secrets.
- REQUIRED: install with `cargo install ssh-cli --locked` (or path install with pins).
- FORBIDDEN: assume a long-lived SSH connection across process runs.
- FORBIDDEN: reintroduce long-lived daemon packaging into this repository.
- FORBIDDEN: enable or emit product telemetry.
- FORBIDDEN: retry blindly on exit 64, 65, 66, or 77.
- FORBIDDEN: parse multi-line NDJSON dual events on the success data path — one JSON document per one-shot success; `secrets_key_auto_created` (when set) is on the same `vps-added` object.
- FORBIDDEN: treat ambient `RUST_LOG` as product config (ignored; only `-v`/`-vv`/`-vvv`).
- FORBIDDEN: print or store primary-key material from `secrets` commands.
- FORBIDDEN: treat SCP directory trees or recursive `-r` as supported.
- FORBIDDEN: assume the agent host runs OpenSSH client binaries for product work —
  `ssh-cli` is pure Rust (`russh`); no local `ssh`/`scp`/`ssh-keygen` spawn at runtime.
- REQUIRED: treat remote command strings as hostile input; NUL bytes are rejected
  with invalid-argument before the SSH channel exec (G-PROC-03).
- REQUIRED: SFTP SETSTAT sends atime+mtime together (G3); set_metadata is fail-closed (G4); permission bits are masked **directionally** — outbound upload uses `SFTP_PERM_MASK` `0o7777` and keeps setuid/setgid/sticky on a file you already own (G12/G19), inbound download uses `SFTP_PERM_MASK_UNTRUSTED` `0o0777` so server-sent elevation bits cannot land on the local file (A3).
- REQUIRED: SFTP download local `set_permissions` failures are errors, not silent (G18).
- REQUIRED: SCP client wire path (`client_real_scp.rs`) uses English identifiers and English channel errors (G16).
- NOTE: G6 (`serial_test` isolation for signal/cancel global state) is a **test harness** concern, not agent runtime.
- FORBIDDEN: assert FIXED text inside local `gaps.md` as a product test (G13/G15) — that file is a maintainer-local audit inventory (gitignored / cargo-excluded), not a published contract.


## Crate Integrations
- Publish consumers depend on the CLI contract, not an unstable library API.
- Pin library experiments to an exact crate version if linking `ssh_cli` as a lib.
- Prefer PATH-installed binary integration for agents.


## CRUD and JSON Contract
### Machine-readable operations
- List hosts: `ssh-cli vps list --json` returns an array of masked host objects.
- Show host: `ssh-cli vps show <name> --json` returns one masked host object.
- Discovery: `ssh-cli commands`, `ssh-cli schema [NAME]`, `ssh-cli doctor` (alias of `vps doctor`).
- Doctor: `ssh-cli vps doctor --json` (or `ssh-cli doctor --json`) returns layer, paths, schema, host count, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out` (JSON boolean), telemetry false.
- Secrets: `ssh-cli secrets status --json` returns encryption mode without key material; `secrets init --json` → `event: "secrets-init"`; `secrets reencrypt --json` → `event: "secrets-reencrypt"`.
- CRUD success events when JSON is effective (`--json` / `--output-format json` / non-TTY auto JSON): `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import` (with optional field `secrets_key_auto_created` when a key is auto-created — still one document).
- Exec family (single host): `ssh-cli exec|sudo-exec|su-exec <vps> <cmd> --json` returns stdout, stderr, exit_code, truncation flags, duration_ms — **one object** (G8).
- Exec family (fleet): `ssh-cli exec|sudo-exec|su-exec --all '<cmd>' --json` or `--hosts a,b '<cmd>'` → `event: "exec-batch"` (`exec-batch.schema.json`); per-host partial failure does not abort siblings.
- Tunnel: **single host only** (one bind + one session per one-shot). Multi-host tunnels = N invocations with distinct ports/`--bind`. Forwards still gated by `--max-concurrency`.
- Doctor: `ssh-cli vps doctor [--json]` emits single root `event: vps-doctor` (`local` + `ssh_probe: null`). Add `--probe-ssh` for multi-host health fan-out embedded in `ssh_probe` (optional `--hosts a,b` subset). Never two JSON roots.
- SCP multi-file (single-host): `ssh-cli scp upload <VPS> f1 f2 … <REMOTE_DIR>` / `download <VPS> r1 r2 … <LOCAL_DIR>` uses **one SSH session** and serial transfers (auth once; G-PAR-47).
- SCP multi-host × multi-file: `ssh-cli scp upload --all f1 f2 … <REMOTE_DIR>` or `--hosts a,b` — bounded **sessions** per host; files serial on each session (G-PAR-48). Multi-file fleet download writes under `<LOCAL_DIR>/<host>/`.
- Health (single): `ssh-cli health-check [<vps>] [--timeout <ms>] [--password-stdin|--key|--key-passphrase[-stdin]] --json` returns name, status, latency_ms.
- Health (fleet): `ssh-cli health-check --all --json` or `--hosts a,b --json` → `event: "health-check-batch"` (`health-check-batch.schema.json`).
- SCP (single): `ssh-cli scp upload|download <vps> <local> <remote> --json` returns transfer success on stdout (`scp-transfer.schema.json` with required `event: "scp-transfer"`); failures use error envelope on stderr; missing file → exit 66 `file not found: <path>` (canonical/normalized path).
- SCP (fleet / multi-file batch): `event: "scp-batch"` (`scp-batch.schema.json`); one-file fleet download writes `local.<vps>`; multi-file fleet download uses host subdirs; multi-host×multi-file result `name` may be `host:path`.
- SCP operational facts: require 0.5.3+; upload streams 32 KiB; download writes `{path}.ssh-cli.partial` then renames; `sync_data` failure is propagated before rename (G9); mtime/mode preservation is best-effort and reported as `mtime_preserved`, parent-dir fsync as `durable` (G-SCP-R01/R02).
- SFTP: `ssh-cli sftp upload|download|ls|mkdir|rmdir|rm|stat|rename` with schemas `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch`; prefer 0.5.3+ (G1 integrity).
- Locale: `ssh-cli locale show|set|clear`; one-shot `--lang`.
- TLS: `ssh-cli tls provider|paths`; `tls mtls list|import|show|remove`; `tls acme account create|show`; `tls acme issue|complete|status|list`.
- Tunnel: `ssh-cli tunnel <vps> <local_port> [remote_host] [remote_port] --timeout-ms <ms> [--bind 127.0.0.1] [--password-stdin|--key|--key-passphrase[-stdin]] --json` emits `tunnel_listening` on stdout after bind; `--bind` defaults to `127.0.0.1`; post-bind deadline exits 0; pre-bind timeout remains 74.
- Tunnel modes (0.5.4): `--socks5` serves an RFC 1928 no-auth `CONNECT` proxy, `--remote-socket <PATH>` forwards to a remote Unix socket, `--reverse` asks the server to listen and delivers back to `<local_port>`. The three are mutually exclusive. `remote_host`/`remote_port` are omitted under `--socks5` and `--remote-socket` (passing them is exit 64) and mean the **server's** bind under `--reverse`, where `remote_port 0` lets the server allocate and report the port in `local_port`.
- Both tunnel events carry `mode` (`local` / `socks5` / `streamlocal` / `reverse`); read it before interpreting the sibling fields, since `local_port` is the server's port under `--reverse` and there is no single destination under `--socks5`.
- `--i-accept-network-exposure` guards the local `--bind` in the three local modes and the **remote** bind under `--reverse`, which is the exposed end in that direction. The local bind is IP-parsed by clap (typo → exit 2); the remote bind is compared as text, since RFC 4254 admits names and the empty string (typo → exit 64 from the guard).
- FORBIDDEN: pass `--bind` together with `--reverse` and expect it to matter — the flag is accepted by clap and then discarded, silently, because reverse delivery is forced to loopback. Address the server-side listener through the positional `<remote_host>`.
- Export: the `ssh-cli vps export` body follows the resolved format, so it is the JSON envelope `event: "vps-export"` on any non-TTY stdout and `--output-format text` is the only route to TOML; empty secrets serialize as `""` (never `sshcli-enc:`). `--include-secrets` needs `-o` or `--i-understand-secrets-on-stdout`.
- Import: `ssh-cli vps import --file <path> [--allow-incomplete]` accepts TOML (EN serialize / PT load aliases) or JSON `vps-export`; `added_at` / `adicionado_em` optional (default now).
- Empty password fields serialize as JSON `null`; non-empty secrets mask as `***` (`FIXED_MASK`). Redacted `vps export` non-empty → `***`; empty → `""`.
- Validate payloads against schemas under `docs/schemas/`; index: [docs/schemas/README.md](schemas/README.md).


## Exit Code Routing
- Exit 0 means success.
- Exit 1 means general runtime failure; inspect stderr.
- Exit 64 means usage or argument error (including empty command) **or** permanent ACME validation (`invalidContact` / 4xx); fix argv/contact, do not retry.
- Exit 65 (`TomlDe` / JSON / schema) means parse/data error; fix input payload.
- Exit 66 means missing VPS or file (`file not found: <path>` on SCP); register or correct the name/path.
- Exit 73 means config write failure; check permissions and disk.
- Exit 74 means IO/SSH connection failure; network retry may help.
- Exit 77 means auth failure or host-key policy; try `--key` / `--password-stdin` / passphrase stdin; do not blind-retry.
- Exit 130/143 means signal termination.


## Retry Strategy
- Prefer JSON error envelope fields `retryable` + `error_class` over exit-only heuristics (`docs/schemas/error-envelope.schema.json`).
- Branch on `error_code`, never on `message`: B2 made the human error line localizable, and `--lang pt-BR` now renders it in Portuguese in **text** mode.
- The JSON envelope's `message` stays **stable English** by contract, so an agent's parsing never depends on the host locale.
- The seam lives in `emit_resolved_ssh_error`: the `--json` branch keeps `SshCliError`'s English `Display`, the human branch consults `i18n::localized_error_text`.
- Error codes without a translation fall back to the English `Display` — the localization is fail-open and can never blank an error line.
- Retry at most twice on `retryable: true` / exit 74 with **exponential full-jitter** backoff (base 200ms, cap 5s; see `ssh_cli::retry::RetryConfig::agent_default`).
- Never retry on `retryable: false` or exits 64, 65, 66, 77, 1 (remote command), 130/143/141 without changing inputs.
- ACME permanent validation (`invalidContact` / 4xx) is exit **64**, not exit 74 — do **not** treat it as retryable network IO.
- The binary does **not** auto-retry non-idempotent `exec`/`scp`/`sftp` in-process (one-shot least privilege); the agent re-invokes the process.
- Shorten or split commands when exit indicates max_command_chars rejection.
- Confirm host key changes with a human before `--replace-host-key`.
