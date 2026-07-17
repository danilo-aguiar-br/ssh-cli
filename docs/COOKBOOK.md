# Cookbook

> Copy executable recipes that solve real multi-host SSH agent problems.

- Read this document in [Portuguese (pt-BR)](COOKBOOK.pt-BR.md).
- Product line: 0.5.1.


## Latency Note
- Expect sub-second local CRUD and cold SSH connect dominated by network RTT.
- Prefer one-shot commands over tunnels when a single remote action is enough.


## Default Values Reference
- Port default: 22
- Timeout default: 60000 ms
- max_command_chars default: 1000
- max_output_chars default: 100000
- Tracing default: error (`-v` → debug; `RUST_LOG` overrides)
- Empty password in list/show JSON: `null` (key-only hosts); non-empty masks as `***`
- Telemetry: disabled
- Secrets at rest: encrypted by default (auto `secrets.key`)
- Install: `cargo install ssh-cli --locked`
- Supply chain: russh 0.62.2; `cargo deny` with `yanked=deny`, `multiple-versions=warn`
- SCP: regular files only (no `-r` / no directories / no SFTP); download partial suffix `.ssh-cli.partial`; success JSON requires `event: "scp-transfer"`
- SCP wire: use 0.4.0+ (prefer product line 0.5.1); never 0.3.9 (crates.io 0.3.9 advertised SCP but was inoperant)
- Redacted export: default body is TOML (even pipes); empty secrets stay `""` (never `sshcli-enc:` blobs); JSON only with `vps export --json`
- Host wire: schema v3 (English serialize; dual-read legacy Portuguese aliases)
- Tunnel post-bind: one-shot deadline exits 0 after `tunnel_listening` (TUN-002); pre-bind timeout remains 74
- Tunnel `--bind` default: `127.0.0.1`
- Tunnel/health auth: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` (0.4.1+)
- Secrets flags (prefer over env): `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- Timeout under 1000 ms: stderr warning (unit is milliseconds, not seconds)
- Password on argv: stderr warning; prefer `--*-stdin`
- CRUD/connect/import with `--json`: events `vps-added` / `vps-edited` / `vps-removed` / `vps-connected` / `vps-import`
- First secret write may emit `secrets-key-auto-created` when the primary-key is provisioned


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
# → event: "secrets-key-auto-created"
# flags preferred over env:
# ssh-cli --secrets-key-file /path/to/key secrets status --json
# ssh-cli --use-keyring secrets init --json
# ssh-cli --allow-plaintext-secrets vps add ...   # tests only
```


## How To Register a Password Host (stdin, no argv leak)

```bash
# prefer --password-stdin; password on argv also works but warns on stderr
printf '%s' 'demo-password-not-real' | ssh-cli vps add \
  --name prod \
  --host prod.example.com \
  --user deploy \
  --password-stdin
# with --json → event: "vps-added" (and possibly secrets-key-auto-created on first secret write)
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
# only when debugging:
# RUST_LOG=debug ssh-cli exec lab "true" --json
# ssh-cli -v exec lab "true" --json
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
# default export body is TOML even on pipe/non-TTY (not auto-JSON)
ssh-cli vps export -o /tmp/hosts.redacted.toml
# empty secrets stay "" (never fake sshcli-enc: ciphertext of empty; EXP-001)
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
# optional bind override (only when intentional):
# ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --bind 0.0.0.0
# optional auth overrides (CLI-005 parity with exec/scp):
printf '%s' "$PASS" | ssh-cli tunnel prod 18080 127.0.0.1 8080 \
  --timeout-ms 30000 --json --password-stdin
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json \
  --key ~/.ssh/id_ed25519
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
# Use 0.4.0+ (prefer product line 0.5.1); never 0.3.9 — that SCP wire was broken
# No directories / no -r / no SFTP
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
# prefer env SSH_CLI_E2E_*; --from-grok-config is maintainer-local ($HOME only)
# official matrix E01–E16 (E10–E14: SCP upload/download/cmp/missing/preserve)
# prints only PASS/FAIL — never host/user/password
# prefer local sshd / throwaway VPS; never auth-failure storms on production (fail2ban)
bash scripts/e2e_real_ssh.sh --from-grok-config
```
