# Cookbook

> **0.5.4** — security and agent-native release. Fixes a remote pre-auth DoS in the SSH banner path (A1), stops server-sent setuid bits landing on downloaded files (A3), closes the world-readable window on ACME/mTLS private keys (A2), and adds payload-shaping flags (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) applied before serialization. BREAKING: partial multi-host failure now exits **1** (was 65); a non-loopback `--bind` requires `--i-accept-network-exposure`. New `tunnel_closed` event.


> Copy executable recipes that solve real multi-host SSH agent problems.

- Read this document in [Portuguese (pt-BR)](COOKBOOK.pt-BR.md).
- Product line: 0.5.4.


## Latency Note
- Expect sub-second local CRUD and cold SSH connect dominated by network RTT.
- Prefer one-shot commands over tunnels when a single remote action is enough.


## Default Values Reference
- Port default: 22
- Timeout default: 60000 ms
- max_command_chars default: 1000
- max_output_chars default: 100000
- Tracing default: error (`-v` → info, `-vv` → debug, `-vvv` → trace; crate-scoped; ambient `RUST_LOG` is ignored)
- Empty password in list/show JSON: `null` (key-only hosts); non-empty masks as `***`
- Telemetry: disabled
- Secrets at rest: encrypted by default (auto `secrets.key`)
- Install: `cargo install ssh-cli --locked`
- Supply chain: russh 0.62.5; `cargo deny` with `yanked=deny`, `multiple-versions=deny`
- SCP: regular files only (no `-r` / no directories). Directory trees and remote FS ops use **`sftp`** (`upload|download --recursive`, `ls`, `mkdir`, …). SCP download partial suffix `.ssh-cli.partial`; success JSON requires `event: "scp-transfer"`
- SCP wire: use 0.4.0+ (prefer product line 0.5.4); never 0.3.9 (crates.io 0.3.9 advertised SCP but was inoperant)
- SFTP: prefer **0.5.3+** (G1 upload integrity fix); verify destination with `sha256sum`; trees via `--recursive`
- Redacted export: the body follows the resolved format, so it is JSON on any non-TTY stdout and TOML only with `--output-format text`; empty secrets stay `""`; non-empty redacted secrets → `***` (`FIXED_MASK`, never `""` for non-empty); never `sshcli-enc:` blobs on redacted path; JSON only with `vps export --json`
- Host wire: schema v3 (English serialize; dual-read legacy Portuguese aliases)
- Tunnel post-bind: one-shot deadline exits 0 after `tunnel_listening` (TUN-002); pre-bind timeout remains 74
- Tunnel `--bind` default: `127.0.0.1`
- Tunnel/health auth: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` (0.4.1+)
- Secrets flags (CLI/XDG only; env secrets stores rejected fail-closed): `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- ACME permanent validation (e.g. `invalidContact`) → exit **64** (do not retry as 74)
- Timeout under 1000 ms: stderr warning (unit is milliseconds, not seconds)
- Password on argv: stderr warning; prefer `--*-stdin`
- CRUD/connect/import with `--json`: events `vps-added` / `vps-edited` / `vps-removed` / `vps-connected` / `vps-import`
- First secret write may set `secrets_key_auto_created: true` on the same `vps-added` document when the primary-key is provisioned
- Product notes (0.5.3 / G1–G19 user-facing):
  - G1: SFTP upload integrity (prefer 0.5.3+; verify destination with `sha256sum`)
  - G2/G14: graduated `-v`/`-vv`/`-vvv` crate-scoped; ambient `RUST_LOG` ignored
  - G3: SFTP SETSTAT sends `atime`+`mtime` together
  - G4: SFTP mutating `set_metadata` is fail-closed
  - G5/G17: multi-file / batch cancel keeps `results.len() == input.len()` (cancelled remainder filled)
  - G8: single-host `exec --json` emits exactly one JSON object
  - G9: SCP download propagates `sync_data` failure before rename
  - G12/G19: permission bits masked by direction — `SFTP_PERM_MASK` (`0o7777`) on upload, `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) on download (A3)
  - G15: accept destination-effect proof (checksum), not client byte counts alone
  - G18: SFTP download local `set_permissions` failures are surfaced
  - G7: real-SSH E2E matrix includes SFTP checksum + recursive tree (**E17/E18**)
  - G6/G11: test-isolation only (see E2E / TESTING) — not a runtime CLI contract


## How To Initialize Primary-Key Encryption

```bash
ssh-cli secrets init
ssh-cli secrets status --json
# never prints the key material
# agent envelopes:
ssh-cli secrets init --json
# → event: "secrets-init" (docs/schemas/secrets-init.schema.json)
ssh-cli secrets reencrypt --json
# → event: "secrets-reencrypt" (docs/schemas/secrets-reencrypt.schema.json)
# first secret write may auto-create secrets.key and emit:
# → event: "vps-added" with secrets_key_auto_created: true (one JSON document)
# CLI/XDG only (env secrets stores rejected fail-closed):
# ssh-cli --secrets-key-file /path/to/key secrets status --json
# ssh-cli --use-keyring secrets init --json
# ssh-cli --allow-plaintext-secrets vps add ...   # tests only
```


## How To Discover Contracts (schema / doctor / commands)

```bash
ssh-cli schema
ssh-cli schema vps-list
ssh-cli doctor --json
ssh-cli commands --json
```

- Root `schema` lists the agent schema catalog; `schema <name>` prints one schema document.
- Root `doctor --json` (or `vps doctor --json`) reports paths, secrets mode, and runtime.
- Root `commands --json` emits the full CLI command tree (`event: "commands"`) for agent discovery.


## How To Register a Password Host (stdin, no argv leak)

```bash
# prefer --password-stdin; password on argv also works but warns on stderr
printf '%s' 'demo-password-not-real' | ssh-cli vps add \
  --name prod \
  --host prod.example.com \
  --user deploy \
  --password-stdin
# with --json → one document event: "vps-added" (secrets_key_auto_created true/false)
# agent auth alternative:
# ssh-cli vps add --name lab --host 203.0.113.10 --user ubuntu --use-agent
# discovery: ssh-cli schema | ssh-cli doctor --json
```


## How To Register a Key-Only Host

```bash
ssh-cli vps add --name edge --host edge.example.com --user ubuntu --key ~/.ssh/id_ed25519
# ssh-cli vps add ... --json → event: "vps-added"
# ssh-cli vps edit edge --user ubuntu --json → event: "vps-edited"
# ssh-cli vps remove edge --json → event: "vps-removed"
# ssh-cli vps connect edge --json → event: "vps-connected"
```


## How To Run a Remote Command With JSON

```bash
ssh-cli exec prod "hostname && uptime" --json
```


## How To Run Safe sudo With Compound Commands

```bash
# packing uses secure `sh -c`; metacharacters stay inside the remote shell
ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --description "bootstrap curl"
```


## How To Elevate With su When sudo Is Unavailable

```bash
printf '%s' 'root-secret' | ssh-cli vps edit prod --su-password-stdin
ssh-cli su-exec prod "whoami"
```


## How To Reject Oversized Agent Commands Early

```bash
ssh-cli vps edit prod --max-command-chars 1000
# long command is rejected before SSH when over limit (max_command_chars)
```


## How To Bound Output for LLM Context

```bash
ssh-cli vps edit prod --max-output-chars 20000
ssh-cli exec prod "dmesg" --json
```


## How To Probe Connectivity After Add

```bash
ssh-cli vps add --name lab --host lab.example.com --user lab --key ~/.ssh/id_ed25519 --check
ssh-cli health-check lab --json
# optional auth overrides (parity with exec/scp since 0.4.1+):
# printf '%s' "$PASS" | ssh-cli health-check lab --json --password-stdin
# ssh-cli health-check lab --json --key ~/.ssh/id_ed25519
```


## How To Run Fleet Work Across All Registered Hosts

Prefer **one process** with `--all` or a subset via `--hosts a,b` (bounded concurrent SSH) over spawning N single-host processes. Cap fan-out with global `--max-concurrency` (1..=64; auto formula when omitted).

```bash
# probe every host in the registry (JSON batch: health-check-batch)
ssh-cli --max-concurrency 8 health-check --all --json
# subset of the registry (still batch JSON)
ssh-cli health-check --hosts web1,web2 --json

# same remote command on every host or a subset (exec-batch; also sudo-exec / su-exec)
ssh-cli exec --all 'uptime' --json
ssh-cli exec --hosts web1,web2 'uptime' --json
ssh-cli --max-concurrency 4 sudo-exec --all 'systemctl is-active nginx' --json

# by tag, without enumerating names — exec family only (exec / sudo-exec / su-exec).
# Tags come from `vps add --tag prod --tag edge`; any host carrying any listed tag matches.
ssh-cli exec --tags prod,edge 'uptime' --json
ssh-cli sudo-exec --tags prod 'systemctl restart nginx' --json
# --all, --hosts and --tags are mutually exclusive: clap rejects any pair with exit 2.
# scp, sftp and health-check take --all and --hosts, but NOT --tags.

# copy one local file to the same remote path on every host (scp-batch)
ssh-cli scp upload --all ./app.tgz /tmp/app.tgz --json
ssh-cli scp upload --hosts web1,web2 ./app.tgz /tmp/app.tgz --json

# download: local path is a prefix → writes ./app.log.<vps>
ssh-cli scp download --all /var/log/app.log ./app.log --json

# local doctor + optional fleet SSH probe
ssh-cli vps doctor --probe-ssh --json
```

- Batch schemas: `docs/schemas/health-check-batch.schema.json`, `exec-batch.schema.json`, `scp-batch.schema.json` (envelope includes `max_concurrency`).
- Cap fan-out with global `--max-concurrency` (1..=64; auto formula when omitted).
- Multi-file / batch cancel keeps `results.len() == input.len()` with cancelled remainder filled (G5/G17).
- Empty registry + `--all` / `--hosts` → usage exit **64**.
- Unknown name in `--hosts` → usage exit **64** (`unknown host(s) for --hosts`).
- Single-host commands remain valid when the target is one positional name (classic JSON).
- `tunnel` is single-host by contract (one bind + one session); multi-host = N invocations.


## How To Probe With Custom Timeout

```bash
# --timeout is milliseconds (not seconds); values under 1000 warn on stderr
# override host timeout when the default is too long or too short for a quick probe
ssh-cli health-check lab --timeout 15000 --json
# optional: combine timeout with key or password-stdin
# ssh-cli health-check lab --timeout 15000 --json --key ~/.ssh/id_ed25519
# avoid accidental sub-second probes unless intentional:
# ssh-cli health-check lab --timeout 500 --json   # works, but stderr warns (<1000 ms)
```


## How To Keep Agent stderr Clean

```bash
# default tracing is error: JSON/tunnel stderr stays free of INFO prose
ssh-cli exec lab "true" --json
# only when debugging (crate-scoped; no password leak via russh — G2/G14):
# ssh-cli -v exec lab "true" --json    # info
# ssh-cli -vv exec lab "true" --json   # debug
# ssh-cli -vvv exec lab "true" --json  # trace
# ambient RUST_LOG is ignored
```


## How To Diagnose XDG Paths and Secrets Mode

```bash
ssh-cli vps doctor --json
# expect secrets_at_rest, secrets_key_source, secrets_key_file, telemetry=false
ssh-cli vps path
ssh-cli secrets status --json
```


## How To Re-encrypt a Legacy Plaintext Inventory

```bash
ssh-cli secrets init
ssh-cli secrets reencrypt
# config.toml passwords become sshcli-enc:v1:… blobs
```


## How To Export and Import Inventory Without Secrets

```bash
# the body follows the resolved format: JSON on non-TTY, TOML only with --output-format text
ssh-cli vps export -o /tmp/hosts.redacted.json
ssh-cli --output-format text vps export -o /tmp/hosts.redacted.toml
# empty secrets stay "" (never fake sshcli-enc: ciphertext of empty; EXP-001)
# non-empty redacted secrets → "***" (FIXED_MASK; never "" for non-empty; G-E2E-10)
# agent envelope only with --json → event: "vps-export"
ssh-cli vps export --json -o /tmp/hosts.redacted.json
# import accepts TOML (EN keys or legacy PT aliases) or JSON vps-export
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.redacted.toml
# redacted/skeleton hosts missing full auth:
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.redacted.toml \
  --allow-incomplete
```


## How To Export With Secrets (guarded)

```bash
# --include-secrets requires -o/--output (mode 0o600) or explicit stdout ack
ssh-cli vps export --include-secrets -o /tmp/hosts.secrets.toml
# pipe without ack is refused (exit 64):
# ssh-cli vps export --include-secrets | cat   # fails
# only if you truly need stdout:
# ssh-cli vps export --include-secrets --i-understand-secrets-on-stdout
```


## How To Open a Bounded Tunnel

```bash
# --bind defaults to 127.0.0.1 (loopback)
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000
# agents: wait for tunnel_listening before using the local port
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json
# stdout: {"ok":true,"event":"tunnel_listening","vps":"prod","local_port":18080,...}
# schema: docs/schemas/tunnel-listening.schema.json
# after tunnel_listening, post-bind one-shot deadline exits 0 (not 74; TUN-002); pre-bind timeout remains 74
# optional bind override — a routable bind REQUIRES the acknowledgement (G-TUN-R13):
# ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 \
#   --bind 0.0.0.0 --i-accept-network-exposure
# without --i-accept-network-exposure the routable bind is refused, not silently published
# tunnel --json also emits tunnel_closed on shutdown, with reason/forwards_served/capacity_waits
# optional auth overrides (CLI-005 parity with exec/scp):
printf '%s' "$PASS" | ssh-cli tunnel prod 18080 127.0.0.1 8080 \
  --timeout-ms 30000 --json --password-stdin
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json \
  --key ~/.ssh/id_ed25519
```


## How To Reach Many Destinations Through One Handshake (`--socks5`)

```bash
# G-TUN-R02: one SSH session serves every destination, chosen per connection
ssh-cli tunnel prod 1080 --socks5 --timeout-ms 300000 --json
# stdout: {"ok":true,"event":"tunnel_listening","mode":"socks5","local_port":1080,...}
# REMOTE_HOST / REMOTE_PORT are omitted: SOCKS5 CONNECT carries the target
# then point any SOCKS5-aware client at 127.0.0.1:1080
curl --socks5-hostname 127.0.0.1:1080 http://10.0.0.5:8080/health
curl --socks5-hostname 127.0.0.1:1080 http://10.0.0.9:9200/_cluster/health
# prefer this over N `ssh-cli tunnel` processes: the handshake is paid once, not N times
```


## How To Forward to a Remote Unix Socket (`--remote-socket`)

```bash
# G-TUN-R03: direct-streamlocal@openssh.com reaches targets that never listen on TCP
ssh-cli tunnel prod 2375 --remote-socket /var/run/docker.sock --timeout-ms 60000 --json
# stdout: {"ok":true,"event":"tunnel_listening","mode":"streamlocal",...}
DOCKER_HOST=tcp://127.0.0.1:2375 docker ps
# PostgreSQL peer socket, same shape:
ssh-cli tunnel db 15432 --remote-socket /var/run/postgresql/.s.PGSQL.5432 \
  --timeout-ms 60000 --json
# the path must be ABSOLUTE — a relative path fails at exit 64 before any SSH work
# local existence is never checked: the socket lives on the server's filesystem
# treat this as privilege delegation — the local port inherits the socket's authority
```


## How To Let the Server Reach Back (`--reverse`)

```bash
# G-TUN-R01: the server listens and delivers connections to your local port
# use case: callback webhook, remote debugger pointing at a local IDE, inverted bastion
ssh-cli tunnel prod 8080 127.0.0.1 9000 --reverse --timeout-ms 120000 --json
# stdout: {"ok":true,"event":"tunnel_listening","mode":"reverse",...}
# REMOTE_PORT 0 asks the server to allocate and report the port it bound:
ssh-cli tunnel prod 8080 127.0.0.1 0 --reverse --timeout-ms 120000 --json
# a local forward cannot accept 0 — there would be nothing to connect to
# exposing the SERVER's bind needs the acknowledgement, since that is the exposed end here:
ssh-cli tunnel prod 8080 0.0.0.0 9000 --reverse \
  --i-accept-network-exposure --timeout-ms 120000 --json
# server-side AllowTcpForwarding / GatewayPorts still govern whether the bind is permitted
```


## How To Shrink an Agent Payload Before It Is Written

```bash
# the eight shaping flags apply BEFORE serialization: the big envelope is never built
ssh-cli health-check --all --json --select name,ok --filter ok=false
# only the failures, two fields each
ssh-cli vps list --json --select name,host --sort name --limit 10
ssh-cli exec --all 'uptime' --json --truncate-content 200 --max-output-bytes 65536
# how many hosts are unreachable, without transferring any host record:
ssh-cli health-check --all --json --filter ok=false --count-only
# stdout: {"count":2}
# a malformed predicate is rejected at parse time, so a typo never looks like "no matches"
ssh-cli vps list --json --filter 'nameprod'     # exit 64: no operator at all
ssh-cli vps list --json --filter '=prod'        # exit 64: empty key
# note what IS valid: `name~~prod` parses as key `name`, substring `~prod` — exit 0, `[]`.
# The refusal covers missing operators, not every surprising-looking string.
```


## How To Rehearse a Destructive Operation (`--dry-run`)

```bash
# accepted ONLY by these six; anywhere else --dry-run is refused with exit 64
ssh-cli vps remove old-host --dry-run --json
ssh-cli vps import --file hosts.toml --dry-run --json
ssh-cli sftp rm prod /srv/app/stale.log --dry-run --json
ssh-cli sftp rmdir prod /srv/app/empty-dir --dry-run --json
ssh-cli secrets init --dry-run --json
ssh-cli secrets reencrypt --dry-run --json
# refusal is the point: a flag that silently does nothing on half the surface
# is worse than no flag, because the operator believes the rehearsal happened
ssh-cli exec prod 'rm -rf /tmp/x' --dry-run --json   # exit 64, not a silent no-op
# unattended runs should also refuse to wait on a human that is not there:
ssh-cli vps list --json --no-input
```


## How To Health-Check with Agent-Safe Auth

```bash
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 5000 --json
# auth parity 0.4.1+ (CLI-006):
printf '%s' "$PASS" | ssh-cli health-check prod --json --password-stdin
ssh-cli health-check prod --json --key ~/.ssh/id_ed25519
printf '%s' "$KEY_PASS" | ssh-cli health-check prod --json \
  --key ~/.ssh/id_ed25519_enc --key-passphrase-stdin
```


## How To Transfer a Release Artifact (regular file only)

```bash
# Use 0.4.0+ (prefer product line 0.5.4); never 0.3.9 — that SCP wire was broken
# SCP: no directories / no -r (use `sftp --recursive` for trees)
ssh-cli scp upload prod ./dist/app.tar.gz /opt/app/app.tar.gz \
  --timeout 120000 --json
# success stdout → docs/schemas/scp-transfer.schema.json
# includes required event: "scp-transfer" (IO-009)
# failures with --json → error envelope on stderr
ssh-cli exec prod "tar -tzf /opt/app/app.tar.gz | head"
```


## How To Download a Remote File Safely

```bash
ssh-cli scp download prod /var/log/app.log ./app.log --json
# on failure the final path is untouched; intermediate is ./app.log.ssh-cli.partial
# mtime/mode preserved both directions (remote scp -tp/-fp)
# sync_data failure is propagated before rename (G9)
```


## How To Upload via SFTP and Verify Checksum (prefer 0.5.3+)

```bash
# G1 fixed SFTP upload truncation in 0.5.3 — always prove destination effect
ssh-cli sftp upload prod ./payload.bin /tmp/payload.bin --json
ssh-cli exec prod "sha256sum /tmp/payload.bin" --json
sha256sum ./payload.bin
# download (partial + atomic rename; G18 surfaces local set_permissions failures):
ssh-cli sftp download prod /tmp/payload.bin ./payload.remote.bin --json
# recursive tree (no symlink follow):
ssh-cli sftp upload --recursive prod ./dist/tree /opt/app/tree --json
# multi-host fleet:
ssh-cli sftp upload --all ./payload.bin /tmp/payload.bin --json
```


## How To Manage Remote Paths via SFTP

```bash
ssh-cli sftp ls prod /tmp --json
ssh-cli sftp mkdir prod /tmp/app --json
ssh-cli sftp stat prod /tmp/app --json
ssh-cli sftp rename prod /tmp/app /tmp/app.bak --json
ssh-cli sftp rm prod /tmp/file.bin --json
ssh-cli sftp rmdir prod /tmp/empty --json
# schemas: sftp-list / sftp-fs-op
```

- Prefer product line **0.5.3+** for all SFTP work (G1 upload integrity).
- SETSTAT sends `atime`+`mtime` together (G3); mutating metadata is fail-closed (G4).
- Permission bits are masked by direction: `SFTP_PERM_MASK` (`0o7777`; G12/G19) on upload, `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) on download, so setuid/setgid/sticky sent by the server never reach the local file (A3).
- Download local `set_permissions` failures are surfaced (G18).


## How To Generate Shell Completions

```bash
ssh-cli completions bash
ssh-cli completions zsh
ssh-cli completions fish
ssh-cli completions powershell
ssh-cli completions elvish
```


## How To Diagnose With -vv Without Password Leak

```bash
# filters are crate-scoped (warn,ssh_cli=…) — never bare global debug (G2)
ssh-cli -vv exec prod "true" --json
# do NOT set RUST_LOG; ambient RUST_LOG is ignored
```


## How To Set UI Locale

```bash
ssh-cli locale show
ssh-cli locale set pt-BR
ssh-cli --lang en vps list
ssh-cli locale clear
```


## How To Inspect TLS Provider and Paths

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

- TLS stack is **rustls** + **aws_lc_rs** only (no OpenSSL runtime).
- Material lives under XDG `tls/` (see `ssh-cli tls paths`).
- ACME permanent validation failures (e.g. `invalidContact`) → exit **64** (do not retry as 74).


## How To Full VPS CRUD

```bash
ssh-cli vps add --name web1 --host 203.0.113.10 --user deploy --key ~/.ssh/id_ed25519 --json
ssh-cli vps list --json
ssh-cli vps show web1 --json
ssh-cli vps edit web1 --timeout 90000 --json
ssh-cli connect web1
ssh-cli vps path
ssh-cli doctor --json
ssh-cli vps remove web1 --json
```


## How To Handle Host Key Rotation Safely (TOFU)

```bash
# first failure reports mismatch; only after human review:
ssh-cli --replace-host-key exec prod "true"
```


## How To Disable Elevation for Untrusted Automation

```bash
ssh-cli --disable-sudo exec prod "id"
# sudo-exec/su-exec remain blocked for this invocation
```


## How To Run Real SSH E2E Without Logging Secrets

```bash
# Preferred (XDG / --config-dir first): isolated config-dir with hosts already registered
ssh-cli --config-dir /tmp/ssh-cli-e2e-lab vps add --name e2e --host … --user … --password-stdin
bash scripts/e2e_real_ssh.sh --config-dir /tmp/ssh-cli-e2e-lab

# Harness-only env (NOT product runtime store) — never commit these values
# export SSH_CLI_E2E_HOST=… SSH_CLI_E2E_USER=… SSH_CLI_E2E_PASSWORD=…
# bash scripts/e2e_real_ssh.sh

# Maintainer-local only: parse $HOME/.grok/config.toml ($HOME only; never copy into the repo)
# bash scripts/e2e_real_ssh.sh --from-grok-config
```

- Default binary: `target/release/ssh-cli` (override with harness `SSH_CLI_E2E_BIN` only).
- Without a lab host / credentials, the script exits **0** with **SKIP** (offline-safe; do not treat SKIP as red gate).
- Official matrix **E01–E18**; **E10–E14** = SCP upload, download, integrity (`cmp`), missing remote, preserve mode+mtime; **E17/E18** = SFTP checksum + recursive tree (G7).
- G6/G11 are test-isolation only (serial signal-flag tests / `reset_flags_for_tests`) — not product CLI surface.
- Script prints only PASS/FAIL/SKIP labels — never host, user, or password.
- Prefer local `sshd` / throwaway VPS; never auth-failure storms on production (fail2ban).
