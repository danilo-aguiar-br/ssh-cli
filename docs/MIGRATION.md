# Migration Guide

> Move from ssh-cli 0.3.3 (or later) to 0.5.2 without losing multi-host inventory.

- Read this document in [Portuguese (pt-BR)](MIGRATION.pt-BR.md).


## What Changes

### Since 0.3.4 (core SSH automation parity)
- Install crypto graph is pinned so `cargo install --locked` succeeds (GAP-014).
- Auth accepts private keys through `--key` / `key_path` (GAP-002).
- `max_chars` semantics split into `max_command_chars` and `max_output_chars` (GAP-004).
- `sudo-exec` packs commands with secure `sh -c` (GAP-005).
- `su-exec` consumes stored `su` password (GAP-003).
- Config writes are atomic with flock and mode 0600 (GAP-007).
- Host keys use TOFU known_hosts (GAP-008).
- `tunnel` requires `--timeout-ms` (GAP-010).
- Schema version for new records was 2 at that time (historical; current wire is schema v3).
- License is dual MIT OR Apache-2.0.

### Since 0.3.5
- Atomic `vps export`, stronger remote abort (TERM+KILL).
- Optional AEAD path matured; doctor reports `secrets_at_rest`.
- Auto JSON when stdout is not a TTY.

### Since 0.3.6
- Default at-rest encryption of secrets in `config.toml` (ChaCha20-Poly1305).
- Auto-creates XDG `secrets.key` (0o600) on first secret write.
- CLI `secrets status|init|reencrypt` (never prints the master key).
- Opt-out for tests only: `--allow-plaintext-secrets` (CLI-only; no env store).
- Doctor fields: `secrets_key_file`, `secrets_plaintext_opt_out`.

### Since 0.3.7
- Agent I/O polish: global `--output-format` on VPS CRUD, `health-check --json`, JSON error envelope, `--quiet` silences human success.
- Tunnel `--timeout-ms` covers SSH connect + loop.
- SCP validates local file before connect; `vps remove` clears orphan `active`.
- `su-exec --password-stdin`; clap conflicts for password/*_stdin.
- Remote non-zero exit maps to process exit `1` with `remote_exit_code` in JSON error envelope.
- Long secrets always mask as `***` (no 12+4 prefix leak).
- sudo/su password on channel stdin, not remote argv.

### Since 0.3.8
- russh upgraded to 0.62.2 (security floor ≥0.60.3).
- Tunnel human banners stay off agent stdout (JSON/non-TTY/quiet).
- No active VPS returns sysexits 66 (`EX_NOINPUT`) via typed error.
- `cargo deny`: `yanked=deny`, empty ignore list; `multiple-versions=warn` for transitive duplicates.
- Version string reports `-dirty` when the working tree is dirty.
- Full residual suite `tests/gaps_v038_integration.rs`.

### Since 0.4.1 (historical)
- AUD-POST patch: empty secrets never become `sshcli-enc` blobs on redacted export (EXP-001); tunnel post-bind deadline exits 0 (TUN-002); `tunnel`/`health-check` auth flag parity with exec/scp (CLI-005/006); SCP JSON includes `event: "scp-transfer"` (IO-009). Additive only — no breaking CLI changes.
- SCP wire fix (0.4.0): crates.io 0.3.9 advertised SCP but the protocol was broken. Upgrade to 0.4.0+ (prefer product line 0.5.2) before relying on `scp`.
- SCP is regular files only (no `-r`). Directory trees use `sftp --recursive`. Use `--timeout` for large files (covers connect + transfer). Success JSON via `--json` / `--output-format json` (`docs/schemas/scp-transfer.schema.json`; SFTP: `sftp-transfer.schema.json`).
- SCP download writes `{path}.ssh-cli.partial` then atomic rename; mode/times applied on the partial before rename.
- SCP upload streams in 32 KiB chunks (no full-file `fs::read` into RAM).
- Preserve mtime/mode bi-directional (remote `scp -tp` / `-fp`; parse `T` + `C` mode).
- SCP flag parity with exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json`.
- `scp --json` failures emit JSON error envelope on stderr (`exit_code`, `message`) — parity with tunnel (IO-007b).
- `tunnel --json` emits one stdout object `event: "tunnel_listening"` after local bind (`docs/schemas/tunnel-listening.schema.json`); still requires `--timeout-ms`.
- Default tracing level is error (not info); `-v` enables debug; ambient `RUST_LOG` is ignored — JSON/tunnel stderr stays clean by default.
- Empty or missing password on key-only VPS serializes as JSON `null` (not `"***"`); non-empty still masks as `***`; human text show uses "(não definida)" for empty.
- `health-check` accepts `--timeout <ms>` override (aligned with exec).
- Product-line docs of that era aligned to 0.4.1; suites `tests/gaps_v039_integration.rs` + `tests/gaps_v040_integration.rs` + `tests/gaps_v041_integration.rs`; official e2e E01–E14 (E10–E14 cover SCP).

### Since 0.4.2 (historical, additive)
- Tunnel ephemeral local port 0: after bind, JSON/banner report the OS-assigned port (never 0 post-bind) (TUN-003).
- Formal `vps export --json` envelope (`event: "vps-export"`) matured; empty secrets stay `""` on redacted export.
- Official e2e E15 (tunnel port 0) + E16 (symlink); suite `tests/gaps_v042_integration.rs`.


## Step-by-Step Migration
### Upgrade the binary

```bash
cargo install ssh-cli --locked --force
ssh-cli --version
```

### Validate inventory and secrets mode

```bash
ssh-cli secrets status --json
ssh-cli vps doctor --json
ssh-cli vps list --json
```

### If you still have plaintext secrets on disk
- On first save with 0.3.6+, a `secrets.key` is auto-created and new writes encrypt.
- To re-cipher an existing plaintext inventory:

```bash
ssh-cli secrets init   # if secrets.key does not exist yet
ssh-cli secrets reencrypt
```

- Backup `config.toml` and `secrets.key` offline; losing the key makes encrypted blobs unreadable.

### Add keys to key-only hosts

```bash
ssh-cli vps edit prod --key ~/.ssh/id_ed25519
```

### Re-check elevation secrets (prefer stdin)

```bash
printf '%s' '...' | ssh-cli vps edit prod --sudo-password-stdin
ssh-cli sudo-exec prod "id"
ssh-cli su-exec prod "id"
```

### Update agent wrappers
- Pass `--timeout-ms` for tunnels.
- On `tunnel --json`, wait for `event == "tunnel_listening"` before using the local port.
- TUN-002: after `tunnel_listening`, post-bind one-shot deadline exits 0 (do not treat 74 as failure if bind already signaled). Pre-bind timeout remains 74.
- EXP-001: on redacted `vps export`, do not expect or parse `sshcli-enc:` for empty secrets — empties serialize as `""`.
- IO-009: parse SCP success with `docs/schemas/scp-transfer.schema.json` including required `event: "scp-transfer"`.
- CLI-005: `tunnel` accepts `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`.
- CLI-006: `health-check` accepts `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`.
- If you came from 0.4.0: redacted export could show fake empty-password ciphertext; tunnel could emit `ok:true` and still exit 74 — upgrade wrappers and the binary to 0.5.2.
- On SCP/tunnel `--json` failure, parse stderr error envelope (not human prose).
- Treat SCP as regular files only; do not send directory trees.
- Re-test transfers after leaving 0.3.9 (that release SCP was not trustworthy).
- Treat `--maxChars` as input limit, not output limit.
- Prefer `--password-stdin` for secrets; password on argv warns on stderr (0.5.2+).
- Timeout values under 1000 ms warn on stderr (unit is milliseconds, not seconds).
- Empty remote command fails with technical message `empty command` (any locale).
- Handle host-key mismatch errors before forcing replace.
- Expect encrypted `config.toml` values prefixed with `sshcli-enc:v1:`.
- Expect default tracing error; use `-v` only when debugging (ambient `RUST_LOG` is ignored); do not parse stderr as success JSON.
- ACME `invalidContact` / permanent validation → exit **64** (do not retry as 74) (G-E2E-01).
- First `vps add` with auto-key: **one** JSON document `event: "vps-added"` with field `secrets_key_auto_created` (G-E2E-04).
- Prefer root `ssh-cli schema` / `ssh-cli doctor` for agent discovery (G-E2E-02/03).
- Register agent-only hosts with `vps add --use-agent` / `--agent-socket` (G-E2E-19).
- Redacted export: non-empty secrets → `***` (`FIXED_MASK`); empty secrets stay `""` (G-E2E-10).
- clap feature `env` removed — no `#[arg(env=…)]` product config (G-E2E-08).
- Version stamp appends `-dirty` when the working tree is dirty even with `.commit_hash` (G-E2E-06).
- Treat empty password in list/show JSON as `null` for key-only hosts.
- May pass `health-check --timeout <ms>` when host default timeout is too long or short.
- Expect process exit `1` (with `remote_exit_code` in JSON envelope) when the remote command fails.
- Expect no active VPS as exit 66; missing SCP file as exit 66 with `file not found: <path>`.
- Expect tunnel banners only in human/TTY paths, not on agent JSON stdout.
- Secrets control is CLI/XDG only (`--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, XDG `secrets.key`); env secrets stores are rejected fail-closed.
- Do not assume auto JSON non-TTY applies to `vps export` — export stays TOML unless `--json`.


## JSON Schema Changes

- Historical (0.3.4 era): new host records wrote `schema_version` 2 with the field set of that release.
- Current (0.5.2): new writes use schema v3 and English TOML keys; loads dual-read legacy Portuguese key aliases.
- Agent event schemas live under `docs/schemas/` (see [schemas/README.md](schemas/README.md)).

### After 0.3.4+ host fields
- `timeout_ms`
- `max_command_chars`
- `max_output_chars`
- `key_path`
- `key_passphrase` (masked)
- `disable_sudo`
- `schema_version` 2 (historical writes only; current wire is schema v3)

### At-rest secrets (0.3.6 era; still current)
- Password/sudo/su/passphrase fields may store `sshcli-enc:v1:…` blobs.
- Prefer CLI flags: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`.
- Primary-key sources: CLI `--secrets-key-file` / `--use-keyring`, or XDG `secrets.key`. `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` are **rejected fail-closed** (not a store).

### Masking (0.4.0)
- Empty password → JSON `null`; non-empty → string `***`.
- Human text show still uses "(não definida)" for empty password.

### Transfer / tunnel events (0.4.0 / 0.4.1+)
- SCP success JSON includes required `event: "scp-transfer"` (IO-009).
- Tunnel still emits `event: "tunnel_listening"` after bind.
- SCP success: `docs/schemas/scp-transfer.schema.json`
- Tunnel listening: `docs/schemas/tunnel-listening.schema.json`
- Failures in JSON mode: `docs/schemas/error-envelope.schema.json` on stderr


## Compatibility Notes
- Existing TOML hosts load and migrate field defaults on read/save paths.
- Legacy alias `--maxChars` maps to command input limit.
- Default timeout is 60000 ms for agent automation.
- Always-trust host key behavior is gone in release builds.
- Default encryption is on; plaintext requires explicit CLI opt-out `--allow-plaintext-secrets` only (env secrets stores are rejected fail-closed).
- Default tracing is error; INFO prose is not expected on agent stderr.
- SCP remains file-only by design in 0.4.0+ (still true in 0.5.2; not a temporary limitation).


## Rollback
- Reinstall a previous version with an exact version pin if required.
- Keep a redacted export via `vps export` before major experiments.
- If rolling back below 0.3.6, encrypted blobs need the matching primary-key or a plaintext re-export while still on 0.3.6+.
- If rolling back to 0.3.9, do not expect working SCP wire (upgrade again to 0.4.0+ for transfers).

## 0.5.2 wire format (schema v3) — current

- Current `schema_version` for new writes is 3 (not 2).
- New writes use English TOML keys: `name`, `port`, `username`, `password`, `added_at`, …
- Loads still accept legacy Portuguese keys (`nome`, `porta`, `usuario`, `senha`, `adicionado_em`) — dual-read EN serialize / PT load aliases.
- `added_at` is optional on import (defaults to now when missing).
- `vps export` default body is TOML (even on pipe/non-TTY); use `--json` for the agent envelope (`event: "vps-export"`). Auto JSON non-TTY does not apply to export.
- `vps import` accepts TOML (EN + PT aliases) or JSON `vps-export` envelopes; `--allow-incomplete` for redacted/skeleton hosts.
- `--include-secrets` requires `-o`/`--output` or `--i-understand-secrets-on-stdout`.
- Secrets control is CLI/XDG only: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, or XDG `secrets.key` (env secrets stores rejected fail-closed).
- Preferred term for the at-rest key is primary-key; legacy keyring entries labeled master-key are still readable.
- Redacted export: non-empty secrets → `***` (`FIXED_MASK`); empty secrets stay `""` (G-E2E-10).
- `vps add --use-agent` / `--agent-socket` registers agent-only hosts (G-E2E-19).
- clap feature `env` removed — no `#[arg(env=…)]` product config (G-E2E-08).
- Version stamp appends `-dirty` when the working tree is dirty even with `.commit_hash` (G-E2E-06).
- ACME `invalidContact` / permanent validation → exit **64** (do not retry as 74) (G-E2E-01).
- First `vps add` with auto-key: **one** JSON document `event: "vps-added"` with field `secrets_key_auto_created` (G-E2E-04).
- Root `ssh-cli schema` / `ssh-cli doctor` for agent discovery (G-E2E-02/03).
- Timeout values under 1000 ms emit a stderr warning (milliseconds, not seconds).
- Password-like values on argv warn on stderr; prefer `--password-stdin` / `--*-stdin`.
- Empty remote command fails with technical English message `empty command` under any locale.
- `secrets init --json` → `event: "secrets-init"`; `secrets reencrypt --json` → `event: "secrets-reencrypt"`; first secret write may set `secrets_key_auto_created: true` on the same success JSON (one document).
- CRUD success JSON events when JSON is effective: `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import`.
- Tunnel `--bind` defaults to `127.0.0.1` (loopback).
- Exit 65 covers `TomlDe` / bad import data; exit 77 is auth/host-key/permission; missing SCP file is exit 66 with `file not found: <path>`.
- Suites: `tests/gaps_v042_integration.rs` + `tests/gaps_v051_integration.rs`; official e2e E01–E16.

Product line: 0.5.2.

## See Also
- [HOW_TO_USE.md](HOW_TO_USE.md) — end-user command surface
- [AGENTS.md](AGENTS.md) — agent contracts and exit routing
- [COOKBOOK.md](COOKBOOK.md) — copy-paste recipes
- [schemas/README.md](schemas/README.md) — JSON schema index
