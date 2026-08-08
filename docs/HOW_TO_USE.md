# How to Use ssh-cli

> **0.5.4** — security and agent-native release. Fixes a remote pre-auth DoS in the SSH banner path (A1), stops server-sent setuid bits landing on downloaded files (A3), closes the world-readable window on ACME/mTLS private keys (A2), and adds payload-shaping flags (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) applied before serialization. BREAKING: partial multi-host failure now exits **1** (was 65); a non-loopback `--bind` requires `--i-accept-network-exposure`. New `tunnel_closed` event.


> Go from install to first remote command in under 60 seconds.

- Read this document in [Portuguese (pt-BR)](HOW_TO_USE.pt-BR.md).
- Return to [README.md](../README.md) for the full command map.
- Product line documented here: 0.5.4.


## Prerequisites
- Install Rust MSRV 1.85.0 or newer via rustup.
- Ensure network reachability to the target SSH host.
- Hold either a password or an OpenSSH private key for that host.
- Prefer a writable XDG config home for multi-host storage.
- Install with `cargo install ssh-cli --locked` (0.5.3+ on crates.io; avoid 0.3.9 for SCP).
- Do not rely on crates.io 0.3.9 for SCP: that release advertised transfer but the wire protocol was broken (0-byte remote files or timeouts). Use 0.5.3+.
- Prefer **0.5.3+** for SFTP: earlier builds could truncate remote files to zero bytes on upload (G1). Verify with `sha256sum` after transfer.


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
- On success with `--json`, parse **exactly one** JSON object on stdout (G8) — not multi-line dual events.
- An empty remote command string fails with technical message `empty command` (always English) and domain usage exit 64.
- Run `ssh-cli secrets status --json` and `ssh-cli doctor --json` (or `vps doctor --json`) when paths or encryption mode are unclear.
- Discover contracts: `ssh-cli schema` / `ssh-cli commands`.
- Register agent-auth hosts with `vps add --use-agent` (optional `--agent-socket`).
- Prefer `--password-stdin` over `--password` when registering password hosts.


## Core Commands
### Complete command inventory

| Command | Purpose |
| --- | --- |
| `vps add` | Register a host (password **or** key **or** `--use-agent`) |
| `vps list` | List registered hosts (secrets masked) |
| `vps remove` | Remove a host from the registry |
| `vps edit` | Patch host fields (timeout, keys, elevation secrets, …) |
| `vps show` | Show one host (secrets masked) |
| `vps path` | Print resolved config path |
| `vps doctor` | Diagnose paths, schema, secrets mode, optional SSH probe |
| `vps export` | Export inventory (default body **TOML**; redacted by default) |
| `vps import` | Import TOML or JSON `vps-export` envelope |
| `connect` | Mark the active host |
| `exec` | Run a remote command (active VPS if name omitted) |
| `sudo-exec` | Run via remote `sudo` + safe `sh -c` packing |
| `su-exec` | Run via remote `su` when su password is stored |
| `scp upload` | Upload **regular file(s)** (no directories / no `-r`) |
| `scp download` | Download **regular file(s)** (partial + atomic rename) |
| `sftp upload` | SFTP upload (optional `--recursive` trees) — prefer **0.5.3+** |
| `sftp download` | SFTP download (optional `--recursive`) |
| `sftp ls` | List a remote directory |
| `sftp mkdir` | Create a remote directory |
| `sftp rmdir` | Remove an empty remote directory |
| `sftp rm` | Remove a remote file |
| `sftp stat` | Stat a remote path |
| `sftp rename` | Rename/move a remote path |
| `tunnel` | Port-forward with required `--timeout-ms`; four modes (local, `--reverse`, `--socks5`, `--remote-socket`) |
| `health-check` | Probe connectivity / latency |
| `secrets status` | Encryption mode without printing the key |
| `secrets init` | Create primary-key (never prints it) |
| `secrets reencrypt` | Re-cipher inventory under current primary-key |
| `completions` | Shell completion scripts to stdout |
| `commands` | List CLI command surface for agents |
| `schema [NAME]` | List or emit one embedded JSON schema |
| `doctor` | Root alias of `vps doctor` |
| `locale show` | Show resolved UI language and winning layer |
| `locale set` | Persist UI language preference (XDG) |
| `locale clear` | Clear stored locale preference |
| `tls provider` | Show rustls `CryptoProvider` status (`aws_lc_rs`) |
| `tls paths` | Show XDG TLS layout paths |
| `tls mtls list` | List imported mTLS client identities |
| `tls mtls import` | Import mTLS client cert/key under XDG |
| `tls mtls show` | Show one mTLS identity (no private key material) |
| `tls mtls remove` | Remove an mTLS identity |
| `tls acme account create` | Create ACME account (needs `--contact mailto:…`) |
| `tls acme account show` | Show ACME account metadata |
| `tls acme issue` | Start ACME order (`--print-challenge` for DNS/HTTP) |
| `tls acme complete` | Complete ACME order after challenge |
| `tls acme status` | Show ACME order/cert status |
| `tls acme list` | List ACME domains under XDG |

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
- Failed download keeps the final path untouched: writes `{path}.ssh-cli.partial`, applies mode/times on the partial, then atomic rename. SCP download propagates `sync_data` failure before rename (G9).
- Upload streams in 32 KiB chunks (does not load the whole file into RAM).
- mtime/mode preservation is attempted both directions automatically (remote `scp -tp` / `-fp`; no extra user flag) and is **best-effort**: the `scp-transfer` event reports `mtime_preserved` and `durable` so you never have to guess (G-SCP-R01/R02).
- Manage primary-key with `ssh-cli secrets status|init|reencrypt` (never prints the key). Keyring may still accept the legacy `secrets-master-key` alias on read.
- `secrets init --json` / `secrets reencrypt --json` emit success events (`secrets-init`, `secrets-reencrypt`; schemas `docs/schemas/secrets-init.schema.json`, `docs/schemas/secrets-reencrypt.schema.json`); first secret write may set field `secrets_key_auto_created: true` on the same `vps-added` JSON document (never a second stdout event). See [docs/schemas/README.md](schemas/README.md).
- CRUD success JSON events when JSON is effective: `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import` (with field `secrets_key_auto_created` when a key is auto-created — one document). Catalog: [docs/schemas/README.md](schemas/README.md).


## SFTP (prefer 0.5.3+)
### Integrity, trees, and metadata
- Prefer product line **0.5.3+** for all SFTP work. **G1** fixed upload truncation: earlier builds could open the remote file with attributes that zeroed destination content. Always verify with destination checksum (`sha256sum` / remote `sha256sum`) — do not trust client-reported byte counts alone (G15).
- Recursive trees: `ssh-cli sftp upload --recursive demo ./tree /tmp/tree` and `sftp download --recursive …` (no symlink follow; depth and listing caps apply).
- SETSTAT sends `atime`+`mtime` together (G3); mutating `set_metadata` is fail-closed (G4); permission bits are masked by direction — upload uses `SFTP_PERM_MASK` `0o7777` (G12), download uses `SFTP_PERM_MASK_UNTRUSTED` `0o0777` so a hostile server cannot put setuid, setgid or sticky on the file you just pulled (A3).
- Multi-file / batch cancel keeps `results.len() == input.len()` with cancelled remainder filled (G5/G17).
- Agent JSON: `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch` schemas under `docs/schemas/`.
- Example integrity check:

```bash
ssh-cli sftp upload demo ./payload.bin /tmp/payload.bin --json
ssh-cli exec demo "sha256sum /tmp/payload.bin" --json
sha256sum ./payload.bin
# compare digests — destination effect is the acceptance criterion
```


## Verbosity (-v / -vv / -vvv)
- Default tracing level is **error** so JSON and tunnel stderr stay clean.
- Graduated verbosity (G14): `-v` → **info**, `-vv` → **debug**, `-vvv` → **trace**.
- Filters are always **crate-scoped** (`warn,ssh_cli=…`) — never bare global `debug` (G2). This prevents password leaks via `russh::client::encrypted` logs.
- Ambient `RUST_LOG` is **ignored**; only CLI `-v`/`-vv`/`-vvv` control product tracing.
- Quiet: `-q` silences human success output.
- Example diagnosis without password leak: `ssh-cli -vv exec demo "true" --json`.


## Locale
```bash
ssh-cli locale show
ssh-cli locale set pt-BR
ssh-cli locale clear
# one-shot override (does not persist):
ssh-cli --lang en vps list
```
- Preference is stored under XDG (no `.env` / no product env language store).
- `locale show` reports resolved language and winning layer.


## TLS (SSH-over-TLS / mTLS / ACME)
```bash
ssh-cli tls provider
ssh-cli tls paths
ssh-cli tls mtls list
ssh-cli tls mtls import --name edge --cert ./client.pem --key ./client-key.pem
ssh-cli tls mtls show edge
ssh-cli tls mtls remove edge
ssh-cli tls acme account create --contact mailto:ops@example.com
ssh-cli tls acme account show
ssh-cli tls acme issue example.com --print-challenge
ssh-cli tls acme complete example.com
ssh-cli tls acme status example.com
ssh-cli tls acme list
```
- Stack is **rustls** + **aws_lc_rs** only (no OpenSSL / native-tls product path).
- mTLS identities and ACME material live under XDG `tls/` (secrets mode 0o600).
- ACME permanent validation (e.g. `invalidContact`) → exit **64** (do not retry as 74).


## Daemon
### There is no daemon
- Treat every invocation as birth-execute-die (one-shot).
- Never expect a background SSH worker from this project.
- Bound tunnels with required `--timeout-ms` so the process still exits.


## Advanced Patterns
### Fleet multi-host (bounded concurrency)
- Prefer `exec|sudo-exec|su-exec|scp|sftp|health-check --all` when the registry has more than one host — one process, concurrent sessions gated by `--max-concurrency N` (auto CPUs×RAM when omitted, clamp 1..=64).
- Parse batch JSON via `docs/schemas/*-batch.schema.json` (`health-check-batch`, `exec-batch`, `scp-batch`, `sftp-batch`); envelope includes `max_concurrency`.
- Example: `ssh-cli --max-concurrency 8 health-check --all --json` then `ssh-cli exec --all 'hostname' --json`.
- Three selectors exist, and they are mutually exclusive: `--all` (whole registry), `--hosts a,b,c` (explicit subset), and `--tags t1,t2` (every host carrying any of those tags, set with `vps add --tag`). `--tags` is accepted by `exec`, `sudo-exec` and `su-exec` only; `scp`, `sftp` and `health-check` take `--all` and `--hosts`.
- Example by tag: `ssh-cli exec --tags prod,edge 'uptime' --json` — one process, one `exec-batch` envelope, no need to enumerate names.
- Do **not** spawn one CLI process per host for fleet work when `--all` is available.
- On cancel, multi-file SCP/SFTP batch results keep input cardinality (G5/G17).

### Safer agent automation
- Feed secrets through stdin flags (`--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--key-passphrase-stdin`) instead of argv.
- Attach shell comments with `--description` for audit-friendly remote history.
- Disable elevation for untrusted tasks with `--disable-sudo`.
- Replace a legitimate host key only after human confirmation using `--replace-host-key` (TOFU).
- Export redacted inventory with `ssh-cli --output-format text vps export -o hosts.toml` (without `--output-format text` the body comes out JSON, because it follows the resolved format and an agent's stdout is never a TTY; non-empty secrets mask as `***` (`FIXED_MASK`); empty secrets stay `""`; never writes fake empty `sshcli-enc:` ciphertext) (EXP-001 / G-E2E-10). List/show empty password is JSON `null` — a different path from export.
- Agent JSON export only with `ssh-cli vps export --json` → envelope `event: "vps-export"` (auto JSON non-TTY does **not** apply to `vps export`).
- `--include-secrets` requires `-o`/`--output` or `--i-understand-secrets-on-stdout` (pipe/stdout without ack is refused, exit 64).
- Import hosts with `ssh-cli vps import --file hosts.toml` (TOML EN keys or legacy PT aliases) or a JSON `vps-export` envelope; use `--allow-incomplete` for redacted/skeleton hosts missing full auth.
- `added_at` / `adicionado_em` are optional on import (serde defaults to now when omitted).
- Wire inventory uses schema v3: new writes serialize English keys (`name`, `port`, `username`, `password`, `added_at`, …); loads still accept legacy Portuguese aliases (`nome`, `porta`, `usuario`, `senha`, `adicionado_em`).
- Re-encrypt a plaintext inventory after upgrade: `ssh-cli secrets reencrypt`.
- Expect auto JSON when stdout is not a TTY unless `--output-format` is set; `vps export` is NOT an exception, so its body is JSON too, and a TOML body needs `--output-format text`.
- Expect empty password on key-only hosts as JSON `null` (not `"***"`); non-empty passwords mask as `***`; human text show uses "(não definida)" for empty.
- On `scp --json` / `sftp --json` failure, parse the JSON error envelope on **stderr** (`exit_code`, `message`), not human prose.
- Timeout values under 1000 ms warn on stderr (milliseconds, not seconds); password-like values on argv also warn — prefer `--*-stdin`.


## Tunnel modes (0.5.4)
### One bind, one SSH session, four shapes
- `tunnel` opens **one** local bind and **one** SSH session per invocation (G-PAR-30). Several tunnels means several one-shots with distinct ports.
- Every mode still requires `--timeout-ms`; a tunnel without a deadline is a daemon, and this CLI does not ship one.
- Read the JSON `mode` field to know which shape is serving: `local`, `reverse`, `socks5` or `streamlocal`.

Default local forward — reach one fixed remote address:

```bash
ssh-cli tunnel prod 15432 10.0.0.5 5432 --timeout-ms 60000 --json
```

`--socks5` — reach **many** destinations through one handshake (G-TUN-R02):

```bash
ssh-cli tunnel prod 1080 --socks5 --timeout-ms 300000 --json
```

- Serves a local SOCKS5 proxy (RFC 1928, no-auth plus CONNECT) and chooses the destination **per connection**, so `REMOTE_HOST` and `REMOTE_PORT` are omitted.
- Prefer this over N `tunnel` processes when an agent must reach N hosts behind one bastion: the SSH handshake is paid once instead of N times.

`--remote-socket` — reach a Unix socket on the server (G-TUN-R03):

```bash
ssh-cli tunnel prod 2375 --remote-socket /var/run/docker.sock --timeout-ms 60000 --json
```

- Opens a `direct-streamlocal@openssh.com` channel, which is how targets that never listen on TCP become reachable: Docker, PostgreSQL, systemd.
- The path must be absolute or the call fails with exit **64**. It is validated as a remote path, so local existence is never consulted — the socket lives on a filesystem this machine cannot see.
- The client may run on Windows: locally it only speaks TCP. What must support the extension is the server.

`--reverse` — let the **server** listen and deliver back to you (G-TUN-R01):

```bash
ssh-cli tunnel prod 8080 0.0.0.0 9000 --reverse --i-accept-network-exposure --timeout-ms 120000 --json
```

- Inverts the direction: the remote host accepts connections and hands them to your local port. This is the path for a callback webhook, a remote debugger pointing at a local IDE, or an inverted bastion.
- `REMOTE_PORT` may be `0`, which asks the server to allocate and report the port it bound. A local forward cannot accept `0`, because there would be nothing to connect to.
- Under `--reverse`, `--i-accept-network-exposure` guards the **server's** bind address, since that is the exposed end in this direction.

### Bind safety and shutdown
- The **local** `--bind` defaults to `127.0.0.1` and is validated by clap as an IP address, so a typo like `127.0.0..1` fails at parse time with exit **2** instead of after resolving the host, opening the session and authenticating (G-TUN-R08).
- The **remote** bind under `--reverse` is the positional `<remote_host>`, and it is deliberately *not* parsed as an IP: RFC 4254 gives meaning to names and to the empty string (all interfaces), so an IP parser would reject the very forms that matter. A typo there fails at exit **64** from the exposure guard, not exit 2 from clap.
- Under `--reverse` the `--bind` flag itself is accepted by clap and then **ignored** — delivery is forced to loopback and the flag never reaches the reverse path. Passing `--bind` together with `--reverse` changes nothing and warns about nothing; set the server-side address through `<remote_host>`.
- Any routable bind requires `--i-accept-network-exposure` (G-TUN-R13). Without it, `--bind 0.0.0.0` used to publish the forwarded remote service to the whole local network in silence.
- `tunnel --json` emits `tunnel_listening` after the bind — wait for it before using the local port — and `tunnel_closed` on shutdown, carrying `reason`, `forwards_served` and `capacity_waits`.
- `tunnel_closed` is what distinguishes a clean deadline from a saturated semaphore: all three endings exit 0, so the counters are the only discriminator.


## Global flags of note
- `--lang` — one-shot UI language override
- `-v` / `-vv` / `-vvv` — graduated verbosity (info/debug/trace; crate-scoped; G2/G14)
- `-q` — quiet human success
- `--config-dir` — isolate XDG config (tests / parallel labs)
- `--no-color` — disable ANSI colors
- `--output-format` / `--json` — force machine JSON
- `--disable-sudo` — block elevation for this invocation
- `--replace-host-key` — TOFU host-key replace after human review
- `--allow-plaintext-secrets` / `--secrets-key-file` / `--use-keyring` — secrets control (CLI/XDG only)
- `--timeout` — connect/transfer override (ms)
- `--max-concurrency` — fleet fan-out clamp 1..=64
- `--fail-fast` — abort remaining multi-host work after first failure
- `--scp-file-concurrency` — multi-file transfer concurrency bound
- `--no-input` — refuse to read stdin and fail fast instead of blocking on an absent human
- `--dry-run` — print the plan for a destructive operation and exit without executing
- `--select` / `--fields` — keep only these dotted paths in each record
- `--filter` — keep records matching `key=value`, `key!=value` or `key~substring` (repeatable, AND)
- `--limit` — emit at most N records (distinct from per-command query limits)
- `--sort` — sort records ascending by dotted path
- `--dedupe-by` — drop later records repeating a dotted path's value
- `--count-only` — replace records with `{"count": N}`, counted after filtering
- `--truncate-content` — shorten long strings by characters (never bytes; UTF-8 stays valid)
- `--max-output-bytes` — cap envelope size by dropping trailing records, never slicing JSON

### Where `--dry-run` is accepted
- Only `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init` and `secrets reencrypt` implement it.
- Anywhere else it is rejected with exit **64** rather than accepted and ignored, so a rehearsal is never mistaken for an operation that already ran.
- Example: `ssh-cli vps remove old-host --dry-run --json` prints the plan and changes nothing.

### Why shaping beats piping
- The eight shaping flags apply **before** serialization, so the oversized envelope is never built.
- Piping stdout through an external JSON tool pays the full token cost first and shrinks the payload afterwards.
- Example: `ssh-cli health-check --all --json --select name,ok --filter ok=false` returns only the failures, with two fields each.


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
- No product `.env` runtime store.


## Subcommands Not Covered Above
- `health-check [--timeout <ms>]` probes connectivity and prints latency (`vps add --check` after register); override timeout when the host default is too long or short.
- Health-check auth parity (0.4.1+ / CLI-006): `--password-stdin` / `--key` / `--key-passphrase` / `--key-passphrase-stdin`.
- Default tracing level is error; use `-v`/`-vv`/`-vvv` when diagnosing (ambient `RUST_LOG` is ignored).
- `tunnel` requires a local port and `--timeout-ms`; remote host and port are required for the default local forward but omitted under `--socks5` and `--remote-socket` (see Tunnel modes above).
- Tunnel `--bind` defaults to `127.0.0.1` (loopback); a routable bind requires `--i-accept-network-exposure`.
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
| 69 (`EX_UNAVAILABLE`) | A host service the CLI depends on is not answering (OS keyring). **Transient** — the same argv succeeds once it is up (G-ERR-R01) |
| 70 (`EX_SOFTWARE`) | Internal failure with no user-fixable input (CSPRNG unavailable). **Permanent** — retrying unchanged will not help |
| 77 (`EX_NOPERM`) | Authentication failed / host-key policy / permission / sudo disabled |
| 130 | SIGINT |
| 143 | SIGTERM |

Product line: 0.5.4.


## Integration With AI Agents
- Load the skill package under `skills/ssh-cli-en/`.
- Prefer JSON output for tool parsing.
- Follow exit-code routing before retries (see README or [AGENTS.md](AGENTS.md)).
- Read [AGENTS.md](AGENTS.md) and [../INTEGRATIONS.md](../INTEGRATIONS.md).
- Event and payload shapes: [docs/schemas/README.md](schemas/README.md).
- Never log primary-key, host passwords, or decrypted secrets.
