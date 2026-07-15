# Cookbook

> Copy executable recipes that solve real multi-host SSH agent problems.

- Read this document in [Portuguese (pt-BR)](COOKBOOK.pt-BR.md).
- Product line: **0.3.6**.


## Latency Note
- Expect sub-second local CRUD and cold SSH connect dominated by network RTT.
- Prefer one-shot commands over tunnels when a single remote action is enough.


## Default Values Reference
- Port default: 22
- Timeout default: 60000 ms
- max_command_chars default: 1000
- max_output_chars default: 100000
- Telemetry: disabled
- Secrets at rest: **encrypted by default** (auto `secrets.key`)
- Install: `cargo install ssh-cli --locked`


## How To Initialize Master-Key Encryption

```bash
ssh-cli secrets init
ssh-cli secrets status --json
# never prints the key material
```


## How To Register a Password Host (stdin, no argv leak)

```bash
printf '%s' 's3cret' | ssh-cli vps add \
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
ssh-cli vps export -o /tmp/hosts.redacted.toml
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.redacted.toml
```


## How To Open a Bounded Tunnel

```bash
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000
```


## How To Transfer a Release Artifact

```bash
ssh-cli scp upload prod ./dist/app.tar.gz /opt/app/app.tar.gz
ssh-cli exec prod "tar -tzf /opt/app/app.tar.gz | head"
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
# uses local env or --from-grok-config; prints only PASS/FAIL
bash scripts/e2e_real_ssh.sh --from-grok-config
```
