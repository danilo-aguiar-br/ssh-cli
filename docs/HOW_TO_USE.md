# How to Use ssh-cli

> Go from install to first remote command in under 60 seconds.

- Read this document in [Portuguese (pt-BR)](HOW_TO_USE.pt-BR.md).
- Return to [README.md](../README.md) for the full command map.
- Product line documented here: **0.4.0** (GAP-001–014 closed; residual LOG/JSON/CLI closed; AUD-SCP wire fix + agent JSON for scp/tunnel closed).


## Prerequisites
- Install Rust MSRV 1.85.0 or newer via rustup.
- Ensure network reachability to the target SSH host.
- Hold either a password or an OpenSSH private key for that host.
- Prefer a writable XDG config home for multi-host storage.
- Install with `cargo install ssh-cli --locked` (**0.4.0+** on crates.io; avoid **0.3.9** for SCP).
- Do not rely on crates.io **0.3.9** for SCP: that release advertised transfer but the wire protocol was broken (0-byte remote files or timeouts). Use **0.4.0+**.


## First Command in 60 Seconds
### Install, register, execute

```bash
cargo install ssh-cli --locked
# Master-key is auto-created on first secret write; optional explicit init:
ssh-cli secrets init
ssh-cli vps add --name demo --host 203.0.113.10 --user ubuntu --key ~/.ssh/id_ed25519
ssh-cli exec demo "uname -a" --json
```

- Confirm exit code 0 and inspect JSON fields `stdout`, `stderr`, `exit_code`, `duration_ms`.
- Run `ssh-cli secrets status --json` and `ssh-cli vps doctor --json` when paths or encryption mode are unclear.
- Prefer `--password-stdin` over `--password` when registering password hosts.


## Core Commands
### Daily operator loop
- List hosts with `ssh-cli vps list --json`.
- Show one host with `ssh-cli vps show demo --json` (secrets masked).
- Patch fields with `ssh-cli vps edit demo --timeout 90000`.
- Mark active host with `ssh-cli connect demo`.
- Run privileged work with `ssh-cli sudo-exec demo "systemctl status nginx" --json` (safe `sh -c` packing).
- Elevate with `ssh-cli su-exec` when `su` password is stored on the host record.
- Transfer **regular files only** (no directories, no `-r`, no SFTP) with `ssh-cli scp upload demo ./app.tgz /tmp/app.tgz`.
- Download with `ssh-cli scp download demo /var/log/app.log ./app.log`.
- Prefer agent JSON: `ssh-cli scp upload demo ./app.tgz /tmp/app.tgz --json` (schema `docs/schemas/scp-transfer.schema.json`).
- SCP flags match exec parity: `--timeout` (connect + transfer), `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json`.
- Failed download keeps the final path untouched: writes `{path}.ssh-cli.partial`, applies mode/times on the partial, then atomic rename.
- Upload streams in 32 KiB chunks (does not load the whole file into RAM).
- mtime/mode are preserved both directions automatically (remote `scp -tp` / `-fp`; no extra user flag).
- Manage master-key with `ssh-cli secrets status|init|reencrypt` (never prints the key).


## Daemon
### There is no daemon
- Treat every invocation as birth-execute-die (one-shot).
- Never expect a background SSH worker from this project.
- Bound tunnels with required `--timeout-ms` so the process still exits.


## Advanced Patterns
### Safer agent automation
- Feed secrets through stdin flags (`--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--key-passphrase-stdin`) instead of argv.
- Attach shell comments with `--description` for audit-friendly remote history.
- Disable elevation for untrusted tasks with `--disable-sudo`.
- Replace a legitimate host key only after human confirmation using `--replace-host-key` (TOFU).
- Export redacted inventory with `ssh-cli vps export -o hosts.toml`.
- Import hosts with `ssh-cli vps import --file hosts.toml`.
- Re-encrypt a plaintext inventory after upgrade: `ssh-cli secrets reencrypt`.
- Expect auto JSON when stdout is not a TTY unless `--output-format` is set.
- Expect empty password on key-only hosts as JSON `null` (not `"***"`); non-empty passwords mask as `***`; human text show uses "(não definida)" for empty.
- On `scp --json` failure, parse the JSON error envelope on **stderr** (`exit_code`, `message`), not human prose.


## Configuration
### XDG multi-host registry
- Resolve config path with `ssh-cli vps path`.
- Expect atomic writes to `config.toml` mode 0600 (tempfile + fsync + flock).
- Expect sibling files `active`, `known_hosts`, and `secrets.key` beside the config.
- Override directory only for tests with `--config-dir` or `SSH_CLI_HOME`.
- Store timeout, max_command_chars, max_output_chars, sudo and su secrets per host.
- Default at-rest encryption (ChaCha20-Poly1305): secrets become `sshcli-enc:v1:…` blobs.
- Master-key order: `SSH_CLI_SECRETS_KEY` → `SSH_CLI_SECRETS_KEY_FILE` → keyring (`SSH_CLI_USE_KEYRING=1`) → XDG `secrets.key`.
- Tests-only opt-out: `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`.


## Subcommands Not Covered Above
- `health-check [--timeout <ms>]` probes connectivity and prints latency (`vps add --check` after register); override timeout when the host default is too long or short.
- Default tracing level is error so JSON and tunnel stderr stay clean; use `RUST_LOG` or `-v` (debug) when diagnosing.
- `tunnel` requires local port, remote host, remote port, and `--timeout-ms`.
- Optional `tunnel --json` emits structured `event: "tunnel_listening"` on stdout after the local bind (`docs/schemas/tunnel-listening.schema.json`).
- `completions` writes shell completion scripts to stdout.
- `su-exec` requires configured `su` password on the host record.
- `secrets` manages encryption master-key without ever printing it.


## Integration With AI Agents
- Load the skill package under `skills/ssh-cli-en/`.
- Prefer JSON output for tool parsing.
- Follow exit-code routing before retries (see README or [AGENTS.md](AGENTS.md)).
- Read [AGENTS.md](AGENTS.md) and [../INTEGRATIONS.md](../INTEGRATIONS.md).
- Never log master-key, host passwords, or decrypted secrets.
