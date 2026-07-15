# Migration Guide

> Move from ssh-cli 0.3.3 (or later) to **0.4.0** without losing multi-host inventory.

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
- Schema version for new records is 2.
- License is dual MIT OR Apache-2.0.

### Since 0.3.5
- Atomic `vps export`, stronger remote abort (TERM+KILL).
- Optional AEAD path matured; doctor reports `secrets_at_rest`.
- Auto JSON when stdout is not a TTY.

### Since 0.3.6
- Default at-rest encryption of secrets in `config.toml` (ChaCha20-Poly1305).
- Auto-creates XDG `secrets.key` (0o600) on first secret write.
- CLI `secrets status|init|reencrypt` (never prints the master key).
- Opt-out for tests only: `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`.
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

### Since 0.4.0 (current)
- **SCP wire fix:** crates.io **0.3.9** advertised SCP but the protocol implementation was broken (literal `\\n` header, empty ACK). Upgrade to **0.4.0** before relying on `scp upload|download`.
- SCP is **regular files only** (no `-r` / no SFTP). Use `--timeout` for large files (covers connect + transfer). Success JSON via `--json` / `--output-format json` (`docs/schemas/scp-transfer.schema.json`).
- Default tracing level is error (not info); `-v` enables debug; `RUST_LOG` overrides — JSON/tunnel stderr stays clean by default.
- Empty or missing password on key-only VPS serializes as JSON `null` (not `"***"`); non-empty still masks as `***`; human text show uses "(não definida)" for empty.
- `health-check` accepts `--timeout <ms>` override (aligned with exec).
- Product-line docs aligned to 0.4.0; suites `tests/gaps_v039_integration.rs` + `tests/gaps_v040_integration.rs`.


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
- Treat `--maxChars` as input limit, not output limit.
- Prefer `--password-stdin` for secrets.
- Handle host-key mismatch errors before forcing replace.
- Expect encrypted `config.toml` values prefixed with `sshcli-enc:v1:`.
- Expect default tracing error; set `RUST_LOG` or `-v` only when debugging; do not parse stderr as JSON.
- Treat empty password in list/show JSON as `null` for key-only hosts.
- May pass `health-check --timeout <ms>` when host default timeout is too long or short.
- Expect process exit `1` (with `remote_exit_code` in JSON envelope) when the remote command fails.
- Expect no active VPS as exit 66.
- Expect tunnel banners only in human/TTY paths, not on agent JSON stdout.


## JSON Schema / host fields (stable through 0.4.0)

### After 0.3.4+ host fields
- `timeout_ms`
- `max_command_chars`
- `max_output_chars`
- `key_path`
- `key_passphrase` (masked)
- `disable_sudo`
- `schema_version` 2

### At-rest secrets (0.3.6 era; still current)
- Password/sudo/su/passphrase fields may store `sshcli-enc:v1:…` blobs.
- Master key sources: `SSH_CLI_SECRETS_KEY`, `SSH_CLI_SECRETS_KEY_FILE`, keyring, or XDG `secrets.key`.

### Masking (0.4.0)
- Empty password → JSON `null`; non-empty → string `***`.
- Human text show still uses "(não definida)" for empty password.


## Compatibility Notes
- Existing TOML hosts load and migrate field defaults on read/save paths.
- Legacy alias `--maxChars` maps to command input limit.
- Default timeout is 60000 ms for agent automation.
- Always-trust host key behavior is gone in release builds.
- Default encryption is on; plaintext requires explicit opt-out env (tests).
- Default tracing is error; INFO prose is not expected on agent stderr.


## Rollback
- Reinstall a previous version with an exact version pin if required.
- Keep a redacted export via `vps export` before major experiments.
- If rolling back below 0.3.6, encrypted blobs need the matching master key or a plaintext re-export while still on 0.3.6+.
