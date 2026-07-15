# Cookbook

> Copy executable recipes that solve real multi-host SSH agent problems.

- Read this document in [Portuguese (pt-BR)](COOKBOOK.pt-BR.md).
- Product line: **0.4.2**.


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
- SCP wire: require **0.4.2+** (crates.io **0.3.9** advertised SCP but was inoperant)
- Redacted export: empty secrets stay `""` (never `sshcli-enc:` blobs)
- Tunnel post-bind: one-shot deadline exits **0** after `tunnel_listening` (TUN-002); pre-bind timeout remains **74**
- Tunnel/health auth: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`


## How To Initialize Master-Key Encryption

```bash
ssh-cli secrets init
ssh-cli secrets status --json
# never prints the key material
```


## How To Register a Password Host (stdin, no argv leak)

```bash
printf '%s' 'demo-password-not-real' | ssh-cli vps add \
  --name prod \
  --host prod.example.com \
  --user deploy \
  --password-stdin
```


## How To Register a Key-Only Host

```bash
ssh-cli vps add --name edge --host edge.example.com --user ubuntu --key ~/.ssh/id_ed25519
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
# optional auth overrides (parity with exec/scp since 0.4.2):
# printf '%s' "$PASS" | ssh-cli health-check lab --json --password-stdin
# ssh-cli health-check lab --json --key ~/.ssh/id_ed25519
```


## How To Probe With Custom Timeout

```bash
# override host timeout when the default is too long or too short for a quick probe
ssh-cli health-check lab --timeout 15000 --json
# optional: combine timeout with key or password-stdin
# ssh-cli health-check lab --timeout 15000 --json --key ~/.ssh/id_ed25519
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
# redacted export (EXP-001 / 0.4.2): empty secrets stay "" (never fake sshcli-enc: ciphertext of empty)
ssh-cli vps export -o /tmp/hosts.redacted.toml
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.redacted.toml
```


## How To Open a Bounded Tunnel

```bash
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000
# agents: wait for tunnel_listening before using the local port
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json
# stdout: {"ok":true,"event":"tunnel_listening","vps":"prod","local_port":18080,...}
# schema: docs/schemas/tunnel-listening.schema.json
# after tunnel_listening, post-bind one-shot deadline exits 0 (not 74; TUN-002); pre-bind timeout remains 74
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
# auth parity 0.4.2 (CLI-006):
printf '%s' "$PASS" | ssh-cli health-check prod --json --password-stdin
ssh-cli health-check prod --json --key ~/.ssh/id_ed25519
printf '%s' "$KEY_PASS" | ssh-cli health-check prod --json \
  --key ~/.ssh/id_ed25519_enc --key-passphrase-stdin
```


## How To Transfer a Release Artifact (regular file only)

```bash
# Require 0.4.2+ — crates.io 0.3.9 SCP wire was broken (0-byte remote / timeout)
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
# official matrix E01–E14 (E10–E14: SCP upload/download/cmp/missing/preserve)
# prints only PASS/FAIL — never host/user/password
bash scripts/e2e_real_ssh.sh --from-grok-config
```
