# How to Use ssh-cli


> Go from install to your first remote command.

- Read this document in [Portuguese (pt-BR)](HOW_TO_USE.pt-BR.md)
- Return to [README.md](../README.md) for the full command map
- Product line documented here: 0.5.6
- This page covers all 48 leaf commands and all 89 distinct flags: the 27 global flags and the 62 flags that exist only after a subcommand
- `-h, --help` and `-V, --version` sit outside that count, and `ssh-cli --version` prints `ssh-cli 0.5.6` followed by the build commit in parentheses
- The outputs in the sections "Expected output without a host" and "Host-bound commands" were measured on the 0.5.6 binary, offline commands against a throwaway config directory and host-bound ones against a throwaway local `sshd`
- The `su-exec` success output and the three Automatic Certificate Management Environment (ACME) network commands were not run, so they list schema or source fields
- Host names, addresses and `duration_ms` in the other examples are illustrative, and their shapes match the measured runs


## Prerequisites
- Install Rust 1.89 or newer via rustup, the minimum supported Rust version (MSRV)
- Keep a C compiler on the build machine, because the `aws-lc-sys` crate behind the `aws_lc_rs` Transport Layer Security (TLS) provider compiles C
- Ensure network reachability to the target SSH host
- Hold either a password or an OpenSSH private key for that host
- Prefer a writable XDG Base Directory (XDG) config home for multi-host storage
- Run on Linux, macOS or Windows, the platform notes live in [CROSS_PLATFORM.md](CROSS_PLATFORM.md)
- Install with `cargo install ssh-cli --locked` and use 0.5.3 or newer, that is the floor for Secure Copy Protocol (SCP) and SSH File Transfer Protocol (SFTP) integrity and this page documents 0.5.6
- Do not rely on crates.io 0.3.9 for SCP, that release advertised transfer but the wire protocol was broken
- Prefer 0.5.3 or newer for SFTP, earlier builds could truncate remote files to zero bytes on upload (G1)
- Verify with `sha256sum` after every transfer


## What Is New in 0.5.6
- `russh` 0.65.1 closes RUSTSEC-2026-0492, RUSTSEC-2026-0495 and two GHSA advisories
- `secrets init --keyring` now writes to the operating system store instead of an in-process mock, so the following `secrets status` reports `key_source: keyring` and `encryption_active: true`
- The store entry is scoped by the resolved config directory, so a profile created with `--config-dir` never reads or overwrites the key of the real profile
- `--use-keyring` and `--keyring` both reach the write path, and an unavailable store exits 69 instead of reporting success
- A profile owns exactly one primary key, so `secrets init` refuses with exit 64 when one already exists in either store, and `--force` retires the other source so the rotation leaves one key and the inventory readable
- The minimum Rust version is 1.89, which russh 0.65 requires
- `--select` and `--truncate-content` now reduce a single record, so `ssh-cli doctor --select local.config_dir_exposed` returns that field
- A reduced bare array writes one `{"event":"agent-shape",...}` line to stderr with `input_count`, `output_count` and `dropped`, and `--quiet` suppresses it
- A collection flag passed on a single record is named in `agent_shape.ignored`
- A reduced `vps list` item still validates against `ssh-cli schema vps-list`, which publishes the full item and the reduced item
- `vps list` in text mode sizes each column from its widest cell, and `vps show` in text mode aligns its 22 labels
- Every leaf shows an `Examples:` block in `--help`
- `vps rename <NAME> <NEW_NAME>` and `vps edit <NAME> --name <NEW>` rename natively and every credential stays readable
- `vps edit` switches the authentication method in one call with `--key`, `--password-stdin` or `--use-agent`
- Eight clear flags remove a field instead of overwriting it
- `--dry-run` covers eight operations and the plan of `vps edit`, `vps rename` and `vps remove` echoes the target
- New host names use the strict charset `[A-Za-z0-9._-]`, an older `config.toml` still loads
- `vps doctor` reports `config_dir_exposed`, `config_dir_permissions` and `invalid_host_names` under `local`
- Text mode escapes control and bidi characters as `\u{1b}`, JSON stays unchanged
- `--lock-wait-ms` bounds the config lock and a stuck lock exits 69 instead of hanging
- `commands --json` publishes every argument with `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` and `hidden`


## First Command in 60 Seconds

### Install, register, execute

```bash
cargo install ssh-cli --locked
ssh-cli vps add --name demo --host 203.0.113.10 --user ubuntu --key ~/.ssh/id_ed25519 --json
ssh-cli exec demo "echo hi" --json
```

- `vps add` prints one document and registers the host

```json
{"ok":true,"event":"vps-added","name":"demo","secrets_key_auto_created":false}
```

- `exec` prints one JSON object, `duration_ms` varies between runs

```json
{"target_resolved":"demo","target_source":"argv","host_resolved":"demo","host_source":"argv","active_fallback":false,"stdout":"hi\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":112}
```

- Confirm exit code 0 and read `stdout`, `stderr`, `exit_code` and `duration_ms`
- Read `target_resolved`, `target_source` and `active_fallback` to know which host answered, `active_fallback` is true when the host came from the active marker
- `target_source` says where the host came from: `argv`, `active_marker` (from `--use-active`) or `selector` (from `--all`, `--hosts` or `--tags`)
- `host_resolved` and `host_source` repeat the same values as read aliases kept for compatibility, so bind new parsers to `target_resolved` and `target_source`
- A success without `--step` parses as exactly one JSON object on stdout (G8), `--step` prints one object per step
- An empty remote command fails with the technical message `empty command` (always English) and exit 64
- Register a password host with `--password-stdin` so the secret never lands on argv
- The first password write creates `secrets.key` and prints `secrets_key_auto_created: true` on the same `vps-added` document, together with `key_file` and `key_source`
- Expected output for the first password host, `CONFIG_DIR` stands for your config directory

```bash
printf '%s' "$PASS" | ssh-cli vps add --name web1 --host 203.0.113.11 --user ubuntu --password-stdin --json
```

```json
{"ok":true,"event":"vps-added","key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","name":"web1","secrets_key_auto_created":true}
```

- Register an agent host with `vps add --use-agent`, add `--agent-socket` for a custom socket
- Run `ssh-cli secrets status --json` and `ssh-cli doctor --json` when paths or encryption mode are unclear
- Discover contracts with `ssh-cli schema` and `ssh-cli commands`


## Core Commands

### Complete command inventory
- The table lists all 48 leaves with the shape of the JSON they print
- `object` means a plain object without an `event` field, `array` means a JSON array of records

| Command | Purpose | Prints |
| --- | --- | --- |
| `ssh-cli vps add` | Register a host with a password, a key or the agent | `vps-added` |
| `ssh-cli vps list` | List registered hosts with secrets masked | array |
| `ssh-cli vps remove` | Remove a host from the registry | `vps-removed` |
| `ssh-cli vps rename` | Rename a host without touching its credentials | `vps-renamed` |
| `ssh-cli vps edit` | Patch, switch or clear host fields | `vps-edited` |
| `ssh-cli vps show` | Show one host with secrets masked | object |
| `ssh-cli vps path` | Print the resolved config path | `vps-path` |
| `ssh-cli vps doctor` | Diagnose paths, schema, secrets mode and the config directory | `vps-doctor` |
| `ssh-cli vps export` | Export the inventory, redacted by default | `vps-export` |
| `ssh-cli vps import` | Import a TOML file or a JSON `vps-export` envelope | `vps-import` |
| `ssh-cli connect` | Mark the active host | `vps-connected` |
| `ssh-cli exec` | Run a remote command | object or `exec-batch` |
| `ssh-cli sudo-exec` | Run a remote command through `sudo` with safe `sh -c` packing | object or `exec-batch` |
| `ssh-cli su-exec` | Run a remote command through `su` when its password is stored | object or `exec-batch` |
| `ssh-cli scp upload` | Upload regular files, no directories | `scp-transfer` |
| `ssh-cli scp download` | Download regular files with partial file and atomic rename | `scp-transfer` |
| `ssh-cli sftp upload` | Upload a file or a tree with `--recursive` | `sftp-transfer` |
| `ssh-cli sftp download` | Download a file or a tree with `--recursive` | `sftp-transfer` |
| `ssh-cli sftp ls` | List a remote directory | `sftp-list` |
| `ssh-cli sftp mkdir` | Create a remote directory | `sftp-fs-op` |
| `ssh-cli sftp rmdir` | Remove an empty remote directory | `sftp-fs-op` |
| `ssh-cli sftp rm` | Remove a remote file | `sftp-fs-op` |
| `ssh-cli sftp stat` | Stat a remote path | `sftp-fs-op` |
| `ssh-cli sftp rename` | Rename or move a remote path | `sftp-fs-op` |
| `ssh-cli tunnel` | Port-forward with a mandatory deadline, four modes | `tunnel_listening` then `tunnel_closed` |
| `ssh-cli health-check` | Probe connectivity and latency | object or `health-check-batch` |
| `ssh-cli secrets status` | Show the encryption mode without the key | object |
| `ssh-cli secrets init` | Create the primary-key and never print it | `secrets-init` |
| `ssh-cli secrets reencrypt` | Re-cipher the inventory under the current primary-key | `secrets-reencrypt` |
| `ssh-cli completions` | Print shell completion scripts | script text |
| `ssh-cli commands` | Print the command tree with every argument | `commands` |
| `ssh-cli schema` | List the schema catalog or print one schema | `schema-catalog` or schema body |
| `ssh-cli doctor` | Root alias of `vps doctor` | `vps-doctor` |
| `ssh-cli locale show` | Show the resolved language and the winning layer | object |
| `ssh-cli locale set` | Persist the language preference under XDG | object |
| `ssh-cli locale clear` | Clear the stored language preference | object |
| `ssh-cli tls provider` | Show the rustls `CryptoProvider` status | `tls-provider` |
| `ssh-cli tls paths` | Show the XDG TLS layout | `tls-paths` |
| `ssh-cli tls mtls list` | List imported mutual TLS (mTLS) identities | `tls-mtls-list` |
| `ssh-cli tls mtls import` | Import a client certificate and key under a valid name | `tls-mtls-import` |
| `ssh-cli tls mtls show` | Show one identity without private key material | `tls-mtls-show` |
| `ssh-cli tls mtls remove` | Remove an mTLS identity | `tls-mtls-remove` |
| `ssh-cli tls acme account create` | Create an ACME account with `--contact` | `tls-acme-account-create` |
| `ssh-cli tls acme account show` | Show ACME account metadata | `tls-acme-account-show` |
| `ssh-cli tls acme issue` | Start an ACME order with `--domain` and print the challenge | `tls-acme-challenge` |
| `ssh-cli tls acme complete` | Complete the ACME order after the challenge | `tls-acme-complete` |
| `ssh-cli tls acme status` | Show order and certificate status | `tls-acme-status` |
| `ssh-cli tls acme list` | List ACME domains under XDG | `tls-acme-list` |

### Global flags: 27 flags that go before the subcommand
- Write global flags before the subcommand, as in `ssh-cli --config-dir DIR --json vps list`
- The binary also accepts them after the subcommand, so older examples keep working
- The alias `--disableSudo` is accepted wherever the global is, before or after the subcommand, and refused by `vps add` and `vps edit`, which take `--disable-sudo`
- `--timeout` and `--disable-sudo` also exist as local flags and the local one wins
- Shaping flags reduce record collections (the array of `vps list`, `results` of batch envelopes, `entries` of `sftp ls`)
- On a single object such as `vps show`, `exec` on one host, `commands` or `doctor`, only `--select` and `--truncate-content` reduce it, and the report lands under `agent_shape`
- The other six, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` and `--max-output-bytes`, leave a single object whole and are named in `agent_shape.ignored`
- `--select` names fields inside each record, so `--select results` returns empty objects and the right call is `--select name,stdout`

| Global flag | Effect |
| --- | --- |
| `--lang LOCALE` | One-shot UI language, `en` or `pt-BR` |
| `-v, --verbose` | Log level `-v` info, `-vv` debug, `-vvv` trace |
| `-q, --quiet` | Silence human success output |
| `--config-dir DIR` | Isolate the config directory, a `.toml` path uses its parent |
| `--no-color` | Disable ANSI colors |
| `--output-format FORMAT` | Pick the output format, JSON when stdout is not a TTY |
| `--json` | Force JSON, alias of `--output-format json` |
| `--select PATHS` | Keep only these dotted paths in each record, alias `--fields` |
| `--filter EXPR` | Keep records matching `key=value`, `key!=value` or `key~substring` |
| `--limit N` | Emit at most N records |
| `--sort PATH` | Sort records ascending by a dotted path |
| `--dedupe-by PATH` | Drop later records that repeat a path value |
| `--count-only` | Replace the records with a count |
| `--truncate-content CHARS` | Shorten long strings by characters |
| `--max-output-bytes BYTES` | Cap the envelope by dropping trailing records |
| `--no-input` | Refuse to read stdin and fail fast, a stdin flag such as `--password-stdin` then exits 64 |
| `--dry-run` | Print the plan of a destructive operation and stop |
| `--lock-wait-ms MS` | Bound the config lock wait in milliseconds, default 5000, range 1 to 300000, then exit 69 |
| `--disable-sudo` | Block `sudo-exec` and `su-exec` for this call, written before `vps add` or `vps edit` it is stored in the record and `--enable-sudo` undoes it |
| `--replace-host-key` | Replace a diverging host key after human review |
| `--allow-plaintext-secrets` | Tests only: allow plaintext secrets at rest |
| `--secrets-key-file PATH` | Use this 64-hex primary-key file for one call |
| `--use-keyring` | Prefer the OS keyring for the primary-key, and `secrets init` writes the new key there |
| `--timeout MS` | Default SSH timeout in milliseconds, a local `--timeout` wins |
| `--max-concurrency N` | Cap fleet fan-out and tunnel forwards, 1 to 64, automatic when omitted, any other value exits 2 |
| `--fail-fast` | Stop admitting new hosts after the first failure |
| `--scp-file-concurrency N` | Concurrent SCP files on one session |

### Local flags: after the subcommand
- Write local flags after the subcommand, positionals first when the usage line says so
- `[X]` marks an optional positional and `<X>` a required one
- Every command also accepts the 27 global flags above
- A flag marked `(required)` makes the command exit 2 when it is missing, and the positionals in angle brackets are required too
- `--bind`, `--i-accept-network-exposure`, `--timeout-ms` and `--probe-ssh` look global but are local, written before the subcommand they exit 2
- `--max-chars` is the legacy alias of `--max-command-chars`, prefer `--max-command-chars` and `--max-output-chars` on `vps add` and `vps edit`

| Command | Positionals and local flags |
| --- | --- |
| `ssh-cli vps add` | `--name <NAME>` (required) `--host <HOST>` (required) `--port <PORT>` `--user <USER>` (required) `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` `--max-command-chars <N>` `--max-output-chars <N>` `--max-chars <N>` `--sudo-password <SUDO_PASSWORD>` `--sudo-password-stdin` `--su-password <SU_PASSWORD>` `--su-password-stdin` `--disable-sudo` `--tag <TAG>` `--tls` `--tls-sni <NAME>` `--tls-client-cert <PATH>` `--tls-client-key <PATH>` `--check` |
| `ssh-cli vps list` | `--tag <TAG>` |
| `ssh-cli vps remove` | `<NAME>` |
| `ssh-cli vps rename` | `<NAME>` `<NEW_NAME>` |
| `ssh-cli vps edit` | `<NAME>` `--name <NEW>` `--host <HOST>` `--port <PORT>` `--user <USER>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` `--max-command-chars <N>` `--max-output-chars <N>` `--max-chars <N>` `--sudo-password <SUDO_PASSWORD>` `--sudo-password-stdin` `--su-password <SU_PASSWORD>` `--su-password-stdin` `--disable-sudo` `--enable-sudo` `--tls` `--no-tls` `--tls-sni <NAME>` `--tls-client-cert <PATH>` `--tls-client-key <PATH>` `--tag <TAG>` `--clear-tags` `--no-key` `--no-agent-socket` `--clear-sudo-password` `--clear-su-password` `--no-tls-sni` `--no-tls-client-cert` `--no-tls-client-key` |
| `ssh-cli vps show` | `<NAME>` |
| `ssh-cli vps path` | none |
| `ssh-cli vps doctor` | `--probe-ssh` `--hosts <LIST>` |
| `ssh-cli vps export` | `--include-secrets` `--output <PATH>` `--i-understand-secrets-on-stdout` |
| `ssh-cli vps import` | `--file <PATH>` (required) `--allow-incomplete` |
| `ssh-cli connect` | `<NAME>` |
| `ssh-cli exec` | `<VPS> <COMMAND>` `--all` `--hosts <LIST>` `--tags <LIST>` `--use-active` `--step <CMD>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` `--description <DESCRIPTION>` |
| `ssh-cli sudo-exec` | `<VPS> <COMMAND>` `--all` `--hosts <LIST>` `--tags <LIST>` `--use-active` `--step <CMD>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--sudo-password <SUDO_PASSWORD>` `--sudo-password-stdin` `--timeout <MS>` `--description <DESCRIPTION>` |
| `ssh-cli su-exec` | `<VPS> <COMMAND>` `--all` `--hosts <LIST>` `--tags <LIST>` `--use-active` `--step <CMD>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--su-password <SU_PASSWORD>` `--su-password-stdin` `--timeout <MS>` `--description <DESCRIPTION>` |
| `ssh-cli scp upload` | `[VPS] [LOCAL] [REMOTE]...` `--all` `--hosts <LIST>` `--src <PATH>` `--dest <DIR>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli scp download` | `[VPS] [LOCAL] [REMOTE]...` `--all` `--hosts <LIST>` `--src <PATH>` `--dest <DIR>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp upload` | `[VPS] [LOCAL] [REMOTE]...` `--all` `--hosts <LIST>` `--src <PATH>` `--dest <DIR>` `--recursive` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp download` | `[VPS] [LOCAL] [REMOTE]...` `--all` `--hosts <LIST>` `--src <PATH>` `--dest <DIR>` `--recursive` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp ls` | `<VPS_NAME>` `<REMOTE>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp mkdir` | `<VPS_NAME>` `<REMOTE>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp rmdir` | `<VPS_NAME>` `<REMOTE>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp rm` | `<VPS_NAME>` `<REMOTE>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp stat` | `<VPS_NAME>` `<REMOTE>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp rename` | `<VPS_NAME>` `<FROM>` `<TO>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli tunnel` | `<VPS_NAME>` `<LOCAL_PORT>` `[REMOTE_HOST]` `[REMOTE_PORT]` `--socks5` `--remote-socket <PATH>` `--reverse` `--timeout-ms <MS>` (required) `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--bind <ADDR>` `--i-accept-network-exposure` |
| `ssh-cli health-check` | `[VPS_NAME]` `--all` `--hosts <LIST>` `--use-active` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli secrets status` | none |
| `ssh-cli secrets init` | `--keyring` `--force` |
| `ssh-cli secrets reencrypt` | none |
| `ssh-cli completions` | `<SHELL>` |
| `ssh-cli commands` | none |
| `ssh-cli schema` | `[NAME]` |
| `ssh-cli doctor` | `--probe-ssh` `--hosts <LIST>` |
| `ssh-cli locale show` | none |
| `ssh-cli locale set` | `<LOCALE>` |
| `ssh-cli locale clear` | none |
| `ssh-cli tls provider` | none |
| `ssh-cli tls paths` | none |
| `ssh-cli tls mtls list` | none |
| `ssh-cli tls mtls import` | `--name <NAME>` (required) `--cert <PATH>` (required) `--key <PATH>` (required) |
| `ssh-cli tls mtls show` | `<NAME>` |
| `ssh-cli tls mtls remove` | `<NAME>` |
| `ssh-cli tls acme account create` | `--staging` `--contact <URL>` (required) `--force` |
| `ssh-cli tls acme account show` | none |
| `ssh-cli tls acme issue` | `--domain <DOMAIN>` (required) `--staging` `--print-challenge` |
| `ssh-cli tls acme complete` | `--domain <DOMAIN>` (required) |
| `ssh-cli tls acme status` | `--domain <DOMAIN>` |
| `ssh-cli tls acme list` | none |

### Daily operator loop
- List hosts with `ssh-cli vps list --json`
- Show one host with `ssh-cli vps show demo --json`, secrets stay masked
- Patch fields with `ssh-cli vps edit demo --timeout 90000`
- Switch the auth method with `ssh-cli vps edit demo --key ~/.ssh/id_ed25519`, `--password-stdin` or `--use-agent`, each clears the other methods
- Mark the active host with `ssh-cli connect demo`, reach it only with `--use-active`, as in `ssh-cli exec --use-active "uptime" --json`
- Run privileged work with `ssh-cli sudo-exec demo "systemctl status nginx" --json`
- Elevate with `ssh-cli su-exec` when the `su` password is stored on the host record
- Transfer regular files with `ssh-cli scp upload demo ./app.tgz /tmp/app.tgz`, no directories and no `-r`
- Move directory trees with `ssh-cli sftp upload --recursive demo ./dir /tmp/dir`
- Download with `ssh-cli scp download demo /var/log/app.log ./app.log`
- Parse the transfer with `--json`, the event is `scp-transfer` (`docs/schemas/scp-transfer.schema.json`)
- SCP flags match `exec`: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` and `--key-passphrase-stdin`
- A missing local or remote file exits 66 with `file not found: <path>` and no stacked `SCP:` prefix
- A failed download leaves the final path untouched, it writes `{path}.ssh-cli.partial` and renames it atomically
- SCP download propagates a `sync_data` failure before the rename (G9)
- Upload streams in 32 KiB chunks and never loads the whole file into RAM
- Preserving mtime and mode is best effort, so read `mtime_preserved` and `durable` on the `scp-transfer` event instead of guessing
- Manage the primary-key with `ssh-cli secrets status`, `ssh-cli secrets init` and `ssh-cli secrets reencrypt`, none prints the key
- The keyring may still accept the legacy `secrets-master-key` alias on read
- Success events when JSON is effective: `vps-added`, `vps-edited` (or `vps-renamed` when `--name` renames), `vps-removed`, `vps-connected` and `vps-import`
- The schema catalog lives in [docs/schemas/README.md](schemas/README.md)

### Expected output without a host
- These commands need no SSH server, so every output below was measured on the 0.5.6 binary with a throwaway `--config-dir`
- `CONFIG_DIR` stands for that directory
- The outputs are JSON because stdout is not a TTY, add `--json` on a terminal
- `vps list` prints an array, `vps show` prints one object, and `vps path`, `connect` and `vps remove` print one event document

```bash
ssh-cli vps list --json --select name,host,user
ssh-cli vps show web1 --json
ssh-cli vps path --json
ssh-cli connect demo --json
ssh-cli vps remove web1 --json
```

```json
[{"host":"203.0.113.10","name":"demo","user":"ubuntu"},{"host":"203.0.113.11","name":"web1","user":"ubuntu"}]
{"name":"web1","host":"203.0.113.11","port":22,"user":"ubuntu","password":"***","key_path":null,"key_passphrase":null,"sudo_password":null,"su_password":null,"timeout_ms":60000,"max_command_chars":1000,"max_output_chars":100000,"disable_sudo":false,"schema_version":3,"added_at":"2026-10-20T01:20:42.076576907+00:00","tags":["prod"]}
{"ok":true,"event":"vps-path","path":"CONFIG_DIR/config.toml"}
{"ok":true,"event":"vps-connected","name":"demo"}
{"ok":true,"event":"vps-removed","name":"web1"}
```

- `vps show` masks a stored secret with three asterisks and prints an absent one as `null`, `added_at` varies
- With `--output-format text`, `vps list` fills the `PASSWORD` column with three asterisks for a host that stores a password and with `(not set)` for a host that uses only a key or the agent
- `vps show` in text mode prints `(not set)` for every empty field, such as `Key path`, `Key passphrase` and `Agent socket`, and the label is not translated
- `doctor` and `vps doctor` print the same `vps-doctor` document, `ssh_probe` stays null until `--probe-ssh` runs, and `--select` reduces it to the requested paths plus `agent_shape`, which is the second shape the schema publishes

```bash
ssh-cli doctor --json
```

```json
{"event":"vps-doctor","local":{"active_file":"CONFIG_DIR/active","active_vps":null,"config_dir_exposed":false,"config_dir_permissions":"0700","config_path":"CONFIG_DIR/config.toml","exists":true,"hosts":2,"invalid_host_names":[],"known_hosts":"CONFIG_DIR/known_hosts","layer":"--config-dir","permissions":"600","runtime":{"arch":"x86_64","is_ci":false,"is_container":false,"is_termux":false,"is_wsl":false,"os":"linux","sandbox":null},"schema_version":3,"secrets_at_rest":"encrypted","secrets_key_file":"CONFIG_DIR/secrets.key","secrets_key_source":"xdg_file","secrets_plaintext_opt_out":false,"telemetry":false},"ok":true,"ssh_probe":null}
```

- A fresh directory shows `config_dir_exposed` false, `config_dir_permissions` `0700` and an empty `invalid_host_names`
- `secrets status` prints an object without `event`, `secrets init` and `secrets reencrypt` print an event, and none prints the key

```bash
ssh-cli secrets status --json
ssh-cli secrets init --json
ssh-cli secrets reencrypt --json
```

```json
{"at_rest":"encrypted","encryption_active":true,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","plaintext_opt_out":false}
{"ok":true,"event":"secrets-init","force":false,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","reencrypted_hosts":0}
{"ok":true,"event":"secrets-reencrypt","hosts":2}
```

- `secrets init` ran in a new directory with no hosts, so `reencrypted_hosts` is 0
- `secrets init` refuses to overwrite an existing key with exit 64 until `--force` is passed
- `commands` lists the whole tree, `schema` lists the 25 embedded schemas and `locale` prints objects without `event`
- `locale show` names the winning layer in `source`, for example `cli_flag`, `persisted` or `system`
- `completions <SHELL>` prints the script as plain text for `bash`, `elvish`, `fish`, `powershell` or `zsh`, any other shell exits 2

```bash
ssh-cli commands | jaq -c '{bin,event,ok,version,top:(.tree.subcommands|map(.name))}'
ssh-cli schema | jaq -c '{event,ok,count:(.schemas|length)}'
ssh-cli locale show --json
ssh-cli locale set pt-BR --json
ssh-cli locale clear --json
```

```json
{"bin":"ssh-cli","event":"commands","ok":true,"version":"0.5.6","top":["vps","connect","exec","sudo-exec","su-exec","scp","sftp","tunnel","health-check","secrets","completions","commands","schema","doctor","locale","tls"]}
{"event":"schema-catalog","ok":true,"count":25}
{"available":["en","pt-BR"],"current":"en","direction":"ltr","persisted_raw":null,"preference_path":"CONFIG_DIR/lang","resolved":"en","script":"Latn","source":"cli_flag","system_raw":"pt-BR"}
{"applies":"next_invocation_unless_overridden","lang":"pt-BR","ok":true,"path":"CONFIG_DIR/lang"}
{"cleared":true,"ok":true}
```

- The TLS commands read and write the `tls/` tree under the config directory and never touch the network, except `tls acme account create`, `tls acme issue` and `tls acme complete`

```bash
ssh-cli tls provider --json
ssh-cli tls paths --json
ssh-cli tls mtls list --json
ssh-cli tls mtls show edge --json
ssh-cli tls mtls remove edge --json
ssh-cli tls acme account show --json
ssh-cli tls acme list --json
ssh-cli tls acme status --json
```

```json
{"ok":true,"event":"tls-provider","installed":true,"min_rustls":"0.23.18","provider":"aws_lc_rs","stack":"rustls+aws_lc_rs"}
{"ok":true,"event":"tls-paths","acme_account":"CONFIG_DIR/tls/acme/account.json","acme_dir":"CONFIG_DIR/tls/acme","mtls_dir":"CONFIG_DIR/tls/mtls","tls_root":"CONFIG_DIR/tls"}
{"ok":true,"event":"tls-mtls-list","identities":[]}
{"ok":true,"event":"tls-mtls-show","cert_path":"CONFIG_DIR/tls/mtls/edge/cert.pem","key_path":"CONFIG_DIR/tls/mtls/edge/key.pem","name":"edge"}
{"ok":true,"event":"tls-mtls-remove","name":"edge"}
{"ok":true,"event":"tls-acme-account-show","directory":null,"path":"CONFIG_DIR/tls/acme/account.json","present":false}
{"ok":true,"event":"tls-acme-list","domains":[]}
{"ok":true,"event":"tls-acme-status","domains":[]}
```

- `tls mtls show` and `tls mtls remove` ran after `tls mtls import --name edge`, whose output is in the TLS section
- `tls acme status` without `--domain` lists every domain, and with `--domain example.com` it prints one entry with `has_cert`, `has_key` and `has_pending_order`

### Host-bound commands: what they print
- These commands need an SSH server or the ACME server, so the table lists the fields of the published schemas and the blocks after it show the measured runs
- `object` means a plain object without an `event` field, and `Fields` names the fields that always appear
- Run `ssh-cli schema <NAME>` to read the full contract of any event

| Command | Prints | Fields |
| --- | --- | --- |
| `ssh-cli exec` | object or `exec-batch` | `target_resolved`, `target_source`, `active_fallback`, `stdout`, `stderr`, `truncated_stdout`, `truncated_stderr`, `duration_ms` and `exit_code` |
| `ssh-cli sudo-exec` | object or `exec-batch` | the same fields as `exec` |
| `ssh-cli su-exec` | object or `exec-batch` | the same fields as `exec` |
| `ssh-cli scp upload` | `scp-transfer` or `scp-batch` | `ok`, `event`, `target_resolved`, `target_source`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`, plus `mtime_preserved` and `durable` |
| `ssh-cli scp download` | `scp-transfer` or `scp-batch` | the same fields as `scp upload` |
| `ssh-cli sftp upload` | `sftp-transfer` or `sftp-batch` | `ok`, `event`, `target_resolved`, `target_source`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`, plus `recursive` |
| `ssh-cli sftp download` | `sftp-transfer` or `sftp-batch` | the same fields as `sftp upload` |
| `ssh-cli sftp ls` | `sftp-list` | `ok`, `event`, `vps`, `path` and `entries` with `name`, `path`, `kind` and, when present, `size` and `mode` |
| `ssh-cli sftp mkdir` | `sftp-fs-op` | `ok`, `event`, `op`, `vps`, `path` and `duration_ms` |
| `ssh-cli sftp rmdir` | `sftp-fs-op` | the same fields as `sftp mkdir` |
| `ssh-cli sftp rm` | `sftp-fs-op` | the same fields as `sftp mkdir` |
| `ssh-cli sftp stat` | `sftp-fs-op` | the same fields as `sftp mkdir`, plus `kind`, `size`, `mode` and `mtime` |
| `ssh-cli sftp rename` | `sftp-fs-op` | the same fields as `sftp mkdir`, plus `to` |
| `ssh-cli tunnel` | `tunnel_listening` then `tunnel_closed` | `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`, `bind` and `mode`, then `reason`, `forwards_served`, `capacity_waits` and `duration_ms` |
| `ssh-cli health-check` | object or `health-check-batch` | `name`, `target_resolved`, `target_source`, `status` and `latency_ms` |
| `ssh-cli tls acme account create` | `tls-acme-account-create` | `present`, `path` and `directory` |
| `ssh-cli tls acme issue` | `tls-acme-challenge` | `domain`, `directory`, `order_url`, `dns_name`, `dns_value` and `created_at` |
| `ssh-cli tls acme complete` | `tls-acme-complete` | `domain`, `has_cert`, `has_key`, `has_pending_order`, `cert_path` and `key_path` |

- Batch envelopes carry `batch_run_id`, `max_concurrency` and `results`, and each result starts with `name` plus `ok` or `status`
- These rows ran against a throwaway `sshd` on 127.0.0.1 with a host named `lab`, local and remote paths share one disk under `/tmp/lab`, and `duration_ms` varies
- The test account has passwordless sudo, so `sudo-exec` printed uid 0, and `su-exec` without a stored su password exits 64 with `su_password_missing`
- The ACME rows come from the source structs, because no ACME server was contacted

```bash
ssh-cli sudo-exec lab "id -u" --json
ssh-cli exec lab "echo one" --step "echo two" --json
ssh-cli connect lab --json
ssh-cli exec --use-active "echo active" --json
ssh-cli health-check lab --json
ssh-cli health-check --use-active --json
```

```json
{"target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","active_fallback":false,"stdout":"0\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":195}
{"active_fallback":false,"command":"echo one","duration_ms":94,"exit_code":0,"host_resolved":"lab","host_source":"argv","stderr":"","stdout":"one\n","step":0,"target_resolved":"lab","target_source":"argv","truncated_stderr":false,"truncated_stdout":false}
{"active_fallback":false,"command":"echo two","duration_ms":96,"exit_code":0,"host_resolved":"lab","host_source":"argv","stderr":"","stdout":"two\n","step":1,"target_resolved":"lab","target_source":"argv","truncated_stderr":false,"truncated_stdout":false}
{"ok":true,"event":"vps-connected","name":"lab"}
{"target_resolved":"lab","target_source":"active_marker","host_resolved":"lab","host_source":"active_marker","active_fallback":true,"stdout":"active\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":103}
{"name":"lab","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","status":"ok","latency_ms":36}
{"name":"lab","target_resolved":"lab","target_source":"active_marker","host_resolved":"lab","host_source":"active_marker","status":"ok","latency_ms":37}
```

- `--step` prints one object per step with `step` (0-based) and `command`, and the positional command is step 0
- `--use-active` shows `target_source` `active_marker` and, on `exec`, `active_fallback` true

```bash
ssh-cli scp upload lab /tmp/lab/payload.bin /tmp/lab/up.bin --json
ssh-cli scp download lab /tmp/lab/up.bin /tmp/lab/down.bin --json
ssh-cli scp upload --all /tmp/lab/payload.bin /tmp/lab/up_all.bin --json
ssh-cli --scp-file-concurrency 2 scp upload lab /tmp/lab/mm1 /tmp/lab/mm2 /tmp/lab/multi --json
```

```json
{"ok":true,"event":"scp-transfer","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","direction":"upload","vps":"lab","local":"/tmp/lab/payload.bin","remote":"/tmp/lab/up.bin","bytes":14,"duration_ms":97,"mtime_preserved":true,"durable":true}
{"ok":true,"event":"scp-transfer","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","direction":"download","vps":"lab","local":"/tmp/lab/down.bin","remote":"/tmp/lab/up.bin","bytes":14,"duration_ms":100,"mtime_preserved":true,"durable":true}
{"event":"scp-batch","target_source":"selector","host_source":"selector","batch_run_id":"01a156eb-42d4-7693-a849-b0b98d461e6e","direction":"upload","max_concurrency":64,"results":[{"name":"lab","ok":true,"bytes":14,"duration_ms":98,"local":"/tmp/lab/payload.bin"},{"name":"lab2","ok":true,"bytes":14,"duration_ms":99,"local":"/tmp/lab/payload.bin"}]}
{"event":"scp-batch","target_source":"argv","host_source":"argv","batch_run_id":"01a156eb-b918-7371-93eb-5f3104740b8a","direction":"upload","max_concurrency":64,"results":[{"name":"/tmp/lab/mm1","ok":true,"bytes":2,"duration_ms":100,"local":"/tmp/lab/mm1"},{"name":"/tmp/lab/mm2","ok":true,"bytes":2,"duration_ms":100,"local":"/tmp/lab/mm2"}]}
```

- A batch result names the host with `name`, except a multi-file SCP, where `name` is the local file and `target_source` is `argv`
- The target directory of a multi-file SCP must already exist, otherwise each result reports `file not found`
- `--scp-file-concurrency N` opens N parallel SCP channels on one session for a multi-file transfer, and the default is 1

```bash
ssh-cli sftp upload lab /tmp/lab/payload.bin /tmp/lab/sf.bin --json
ssh-cli sftp download lab /tmp/lab/sf.bin /tmp/lab/sf_down.bin --json
ssh-cli sftp upload --recursive lab /tmp/lab/tree /tmp/lab/tree2 --json
ssh-cli sftp ls lab /tmp/lab/tree2 --json
ssh-cli sftp mkdir lab /tmp/lab/newdir --json
ssh-cli sftp stat lab /tmp/lab/payload.bin --json
ssh-cli sftp rename lab /tmp/lab/newdir /tmp/lab/newdir2 --json
ssh-cli sftp rmdir lab /tmp/lab/newdir2 --json
ssh-cli sftp rm lab /tmp/lab/sf.bin --json
ssh-cli sftp upload --hosts lab,lab2 /tmp/lab/payload.bin /tmp/lab/hosts.bin --json
```

```json
{"ok":true,"event":"sftp-transfer","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","direction":"upload","vps":"lab","local":"/tmp/lab/payload.bin","remote":"/tmp/lab/sf.bin","bytes":14,"duration_ms":0}
{"ok":true,"event":"sftp-transfer","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","direction":"download","vps":"lab","local":"/tmp/lab/sf_down.bin","remote":"/tmp/lab/sf.bin","bytes":14,"duration_ms":0}
{"ok":true,"event":"sftp-transfer","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","direction":"upload","vps":"lab","local":"/tmp/lab/tree","remote":"/tmp/lab/tree2","bytes":4,"duration_ms":2,"recursive":true}
{"ok":true,"event":"sftp-list","vps":"lab","path":"/tmp/lab/tree2","entries":[{"name":"sub","path":"/tmp/lab/tree2/sub","kind":"dir","size":60,"mode":16877},{"name":"a.txt","path":"/tmp/lab/tree2/a.txt","kind":"file","size":2,"mode":33188}]}
{"ok":true,"event":"sftp-fs-op","op":"mkdir","vps":"lab","path":"/tmp/lab/newdir","duration_ms":137}
{"ok":true,"event":"sftp-fs-op","op":"stat","vps":"lab","path":"/tmp/lab/payload.bin","duration_ms":0,"kind":"file","size":14,"mode":33188,"mtime":1792459618}
{"ok":true,"event":"sftp-fs-op","op":"rename","vps":"lab","path":"/tmp/lab/newdir","to":"/tmp/lab/newdir2","duration_ms":135}
{"ok":true,"event":"sftp-fs-op","op":"rmdir","vps":"lab","path":"/tmp/lab/newdir2","duration_ms":140}
{"ok":true,"event":"sftp-fs-op","op":"rm","vps":"lab","path":"/tmp/lab/sf.bin","duration_ms":326}
{"event":"sftp-batch","target_source":"selector","host_source":"selector","batch_run_id":"01a156eb-4ad7-7af2-899a-f5dbc1b0a62a","direction":"upload","max_concurrency":64,"results":[{"name":"lab","ok":true,"bytes":14,"duration_ms":0,"local":"/tmp/lab/payload.bin"},{"name":"lab2","ok":true,"bytes":14,"duration_ms":0,"local":"/tmp/lab/payload.bin"}]}
```

- `sftp ls` entries carry `kind` (`dir` or `file`), `size` and `mode` as a decimal number, and `sftp stat` adds `mtime` in epoch seconds
- A recursive transfer adds `recursive` true and `bytes` counts the files copied

```bash
ssh-cli tunnel lab 18081 --socks5 --timeout-ms 1500 --json
ssh-cli tunnel lab 18082 --remote-socket /run/user/1000/bus --timeout-ms 1500 --json
ssh-cli tunnel lab 18083 127.0.0.1 0 --reverse --timeout-ms 1500 --json
```

```json
{"ok":true,"event":"tunnel_listening","vps":"lab","local_port":18081,"remote_host":"*","remote_port":0,"timeout_ms":1500,"bind":"127.0.0.1","mode":"socks5"}
{"ok":true,"event":"tunnel_listening","vps":"lab","local_port":18082,"remote_host":"/run/user/1000/bus","remote_port":0,"timeout_ms":1500,"bind":"127.0.0.1","mode":"streamlocal"}
{"ok":true,"event":"tunnel_listening","vps":"lab","local_port":34155,"remote_host":"127.0.0.1","remote_port":18083,"timeout_ms":1500,"bind":"127.0.0.1","mode":"reverse"}
```

- Each tunnel also printed its `tunnel_closed` document with `reason` `deadline` and exit 0, only the first document of each run is shown
- Under `--reverse` with `REMOTE_PORT` 0 the server allocated the port and it came back in `local_port`, while `remote_port` echoes the local port you passed

```bash
ssh-cli health-check dead --json
ssh-cli exec badkey "id" --json
ssh-cli --fail-fast --max-concurrency 1 exec --hosts dead,lab "echo hi" --json
```

```json
{"exit_code":74,"error_code":"connection_failed","message":"ssh connection failed: TCP dial failed for 127.0.0.1:1: Connection refused (os error 111)","error_class":"transient","retryable":true,"suggestion":"retry at most twice with exponential full-jitter backoff (exit 74)"}
{"exit_code":77,"error_code":"authentication_failed","message":"ssh authentication failed; try --password-stdin, --key PATH, --key-passphrase-stdin, or verify the user","error_class":"permanent","retryable":false,"suggestion":"change credentials (--key / --password-stdin) or host-key policy; do not blind-retry","target_resolved":"badkey","target_source":"argv","host_resolved":"badkey","host_source":"argv","active_fallback":false}
{"event":"exec-batch","batch_run_id":"01a156eb-b7e7-7691-8e00-c7bdb3a6bc6c","max_concurrency":1,"results":[{"name":"dead","ok":false,"stdout":"","stderr":"ssh connection failed: TCP dial failed for 127.0.0.1:1: Connection refused (os error 111)","duration_ms":0,"error":"ssh connection failed: TCP dial failed for 127.0.0.1:1: Connection refused (os error 111)"},{"name":"lab","ok":false,"stdout":"","stderr":"not attempted (fail-fast stopped admission)","duration_ms":0,"error":"not attempted (fail-fast stopped admission)"}]}
{"exit_code":1,"error_code":"partial_failure","message":"2/2 hosts failed during exec","error_class":"partial","retryable":false}
```

- `dead` points to a closed loopback port and `badkey` uses a key the server does not trust, so the envelopes are exit 74 `connection_failed` (retryable) and exit 77 `authentication_failed` (permanent)
- A `health-check` error carries no `target_resolved` or `target_source`, while an `exec` error carries both
- With `--fail-fast` the host that was never started appears in `results` with `ok` false and `not attempted (fail-fast stopped admission)`, and the process exits 1 with `partial_failure` on stderr


## SFTP

### Integrity, trees and metadata
- Prefer product line 0.5.3 or newer for all SFTP work
- G1 fixed upload truncation, earlier builds could open the remote file with attributes that zeroed the content
- Verify with a destination checksum and never trust client-reported byte counts alone (G15)
- Recursive trees use `sftp upload --recursive` and `sftp download --recursive`, with no symlink follow and with depth and listing caps
- SETSTAT sends `atime` and `mtime` together (G3) and a mutating `set_metadata` is fail-closed (G4)
- Permission bits are masked by direction: upload uses `SFTP_PERM_MASK` (`0o7777`, G12) and download uses `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`)
- A hostile server therefore cannot put setuid, setgid or sticky on the file you just pulled (A3)
- Multi-file and batch cancel keeps `results.len() == input.len()` with the cancelled remainder filled (G5/G17)
- `sftp ls` and `sftp stat` escape `\n` and `\t` of a remote name in text mode, so a forged name cannot add lines
- Agent JSON uses the schemas `sftp-transfer`, `sftp-list`, `sftp-fs-op` and `sftp-batch` under `docs/schemas/`

```bash
ssh-cli sftp upload demo ./payload.bin /tmp/payload.bin --json
ssh-cli exec demo "sha256sum /tmp/payload.bin" --json --select stdout
sha256sum ./payload.bin
```

- Expected `sftp upload` output, `bytes` is the file size and `duration_ms` varies

```json
{"ok":true,"event":"sftp-transfer","target_resolved":"demo","target_source":"argv","host_resolved":"demo","host_source":"argv","direction":"upload","vps":"demo","local":"./payload.bin","remote":"/tmp/payload.bin","bytes":8,"duration_ms":1}
```

- Compare the two digests, the destination effect is the acceptance criterion


## Verbosity
- The default tracing level is error, so JSON and tunnel stderr stay clean
- The one exception is a reduced bare array: `vps list --json --limit 2` writes `{"event":"agent-shape","input_count":N,"output_count":N,"dropped":N}` to stderr, because a bare array has no room for the report without changing its type
- That line never appears on an unshaped run, and `--quiet` suppresses it like any other stderr line
- Graduated verbosity (G14): `-v` is info, `-vv` is debug and `-vvv` is trace
- Filters are always crate-scoped (`warn,ssh_cli=…`) and never a bare global `debug` (G2)
- That scoping prevents password leaks through `russh::client::encrypted` logs
- Ambient `RUST_LOG` is ignored and only `-v`, `-vv` and `-vvv` control product tracing
- `-q` silences human success output
- Diagnose without leaking a password with `ssh-cli -vv exec demo "true" --json`


## Locale

```bash
ssh-cli locale show
ssh-cli locale set pt-BR
ssh-cli locale clear
ssh-cli --lang en vps list
```

- The preference lives under XDG in the `lang` file and there is no `.env` or product environment language store
- `locale show` reports the resolved language and the winning layer
- `--lang en` overrides for one call and persists nothing
- Error bodies in JSON stay in English by design, so agents get stable text


## TLS

### SSH-over-TLS, mTLS and ACME

```bash
ssh-cli tls provider
ssh-cli tls paths
ssh-cli tls mtls list
ssh-cli tls mtls import --name edge --cert ./client.pem --key ./client-key.pem
ssh-cli tls mtls show edge
ssh-cli tls mtls remove edge
ssh-cli tls acme account create --contact mailto:ops@example.com
ssh-cli tls acme account show
ssh-cli tls acme issue --domain example.com --print-challenge
ssh-cli tls acme complete --domain example.com
ssh-cli tls acme status --domain example.com
ssh-cli tls acme list
```

- The ACME commands name the domain with `--domain`, a bare positional exits 2
- The stack is rustls with `aws_lc_rs` only, with no OpenSSL or native-tls path
- mTLS identities and ACME material live under XDG `tls/` with secrets at mode 0600
- `tls mtls import --name` refuses control, formatting and bidi characters with exit 64 and creates no directory
- Ordinary Unicode such as `serviço-1` stays accepted
- ACME permanent validation such as `invalidContact` exits 64, do not retry it as 74
- Expected `tls mtls import` output for the name `edge`, the paths follow the config directory

```json
{"ok":true,"event":"tls-mtls-import","cert_path":"CONFIG_DIR/tls/mtls/edge/cert.pem","key_path":"CONFIG_DIR/tls/mtls/edge/key.pem","name":"edge"}
```


## Daemon

### There is no daemon
- Treat every invocation as birth, execute, die (one-shot)
- Never expect a background SSH worker from this project
- Bound tunnels with the required `--timeout-ms` so the process still exits


## Advanced Patterns

### Fleet multi-host with bounded concurrency
- Prefer `exec`, `sudo-exec`, `su-exec`, `scp`, `sftp` and `health-check` with `--all` when the registry has more than one host
- One process runs concurrent sessions gated by `--max-concurrency N`, auto from CPUs and RAM, clamped to 1..=64
- Three selectors exist and they are mutually exclusive: `--all`, `--hosts a,b,c` and `--tags t1,t2`
- `--tags` is accepted by `exec`, `sudo-exec` and `su-exec` only, tags come from `vps add --tag`
- `scp`, `sftp` and `health-check` take `--all` and `--hosts`
- The active marker is a fourth, deliberate route: `--use-active` on `exec`, `sudo-exec`, `su-exec` and `health-check`
- Parse batch JSON with `health-check-batch`, `exec-batch`, `scp-batch` and `sftp-batch`, the envelope carries `max_concurrency`
- Do not spawn one command-line interface (CLI) process per host when `--all` is available

```bash
ssh-cli --max-concurrency 8 health-check --all --json
ssh-cli exec --tags prod,edge "echo hi" --json
```

- Expected `health-check --all` output, `batch_run_id` and `latency_ms` vary

```json
{"event":"health-check-batch","batch_run_id":"00000000-0000-7000-8000-000000000000","max_concurrency":8,"results":[{"name":"demo","status":"ok","latency_ms":38},{"name":"demo2","status":"ok","latency_ms":41}]}
```

- Expected `exec --tags` output, one `exec-batch` envelope and no need to enumerate names

```json
{"event":"exec-batch","batch_run_id":"00000000-0000-7000-8000-000000000000","max_concurrency":64,"results":[{"name":"demo2","ok":true,"exit_code":0,"stdout":"hi\n","stderr":"","duration_ms":147}]}
```

- A partial failure exits 1 and prints the error envelope `partial_failure` on stderr

### Shape the payload before it is written
- The eight shaping flags apply before serialization, so the oversized envelope is never built
- Piping stdout through an external JSON tool pays the full token cost first and shrinks the payload afterwards
- The health-check batch reports `status` (`ok` or `error`), so filter on `status` and not on `ok`
- On a batch, `--truncate-content` sets `agent_shape.content_truncated` and `--count-only` keeps the envelope and replaces `results` with `count`

```bash
ssh-cli health-check --all --json --select name,status --filter status!=ok
```

- Expected output, `agent_shape` tells how many records were dropped and the exit code is 1 because one host failed

```json
{"agent_shape":{"dropped":2,"input_count":3,"output_count":1},"batch_run_id":"00000000-0000-7000-8000-000000000000","event":"health-check-batch","max_concurrency":64,"results":[{"name":"down1","status":"error"}]}
```

- The bare array of `vps list` has no room for `agent_shape`, so its reduction report goes to stderr as one JSON line with `"event":"agent-shape"` whenever something was dropped or truncated, and `--quiet` suppresses it
- `--count-only` on a list prints only the count

```bash
ssh-cli vps list --json --count-only
```

```json
{"count":3}
```

### Switch the authentication method in one call
- `--key` clears `password`, `use_agent` and `agent_socket`
- `--password` and `--password-stdin` clear `key_path`, `key_passphrase`, `use_agent` and `agent_socket`
- `--use-agent` clears `password`, `key_path` and `key_passphrase`
- `updated` lists the new method first and then only the cleared fields that held a value
- A different `--key` drops the old passphrase unless the call brings a new one, and the same key path changes nothing
- `--agent-socket` alone implies `--use-agent` and clears `password`, `key_path` and `key_passphrase`
- Two primary auth flags in one call exit 2, and a passphrase with no key exits 64 and leaves the record untouched

```bash
ssh-cli vps edit web1 --key ~/.ssh/id_ed25519 --json
```

```json
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":["key_path","password"]}
```

### Rename without retyping a credential
- `ssh-cli vps rename <NAME> <NEW_NAME>` moves the registry key, the stored name and the sibling `active` marker when it points at this host
- `ssh-cli vps edit <NAME> --name <NEW>` reaches the same code path and emits `vps-renamed` too, `vps-edited` is emitted only when nothing is renamed
- Stored secrets stay readable because the record seals them without associated data, so the new name changes nothing about them
- The old export, edit, import detour is no longer needed and it dropped agent authentication
- A name that is invalid, identical to the current one or already taken exits 64 (`vps_duplicate` for a taken name), an unknown source exits 66

```bash
ssh-cli vps rename web1 web-1 --json
```

```json
{"ok":true,"event":"vps-renamed","active_rewritten":false,"host_resolved":"web1","host_source":"argv","name":"web-1","previous_name":"web1","target_resolved":"web1","target_source":"argv","updated":[]}
```

- The eight clear flags are `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key`, `--no-key` and `--no-agent-socket`

### Host fields, SSH-over-TLS and the flags the tables only list
- `vps add` requires `--name`, `--host` and `--user`, the port is 22 unless `--port` says otherwise
- `--sudo-password` and `--su-password` store the elevation passwords per host, prefer `--sudo-password-stdin` and `--su-password-stdin` so nothing lands on argv
- `--check` runs a health-check right after the add, and a failing probe exits 74 after the host was already registered
- `--tls` turns on SSH-over-TLS (rustls) for the host, and `vps edit --no-tls` turns it off again
- `--tls-sni` sets the TLS server name and defaults to `--host` when omitted, `vps edit --no-tls-sni` clears it
- `--tls-client-cert` and `--tls-client-key` point to the mTLS client certificate and key PEM files, `vps edit --no-tls-client-cert` and `--no-tls-client-key` clear them
- `--tls-sni` without `--tls` is stored without an error

```bash
ssh-cli vps add --name t1 --host 127.0.0.1 --user u --key KEY_PATH --tls --tls-sni sni.example.com --tls-client-cert ./client.pem --tls-client-key ./client-key.pem --json
ssh-cli vps edit t1 --no-tls --json
```

```json
{"ok":true,"event":"vps-added","name":"t1","secrets_key_auto_created":false}
{"ok":true,"event":"vps-edited","host_resolved":"t1","host_source":"argv","name":"t1","target_resolved":"t1","target_source":"argv","updated":["tls"]}
```

- `vps show` then prints `tls` null and keeps `tls_sni`
- `scp` and `sftp` take `--src` (repeatable) with `--dest` (a directory) when one transfer has several sources
- `secrets init --keyring` keeps the primary-key in the OS keyring instead of `secrets.key`
- `tls mtls import` requires `--name`, `--cert` (certificate chain PEM) and `--key` (private key PEM)
- `tls acme account create` requires one or more `--contact` URLs such as `mailto:ops@example.com`, `--staging` picks the Let's Encrypt staging directory and `--force` replaces an existing account
- `tls acme issue --staging` uses the staging directory too, and `--print-challenge` is mandatory in practice because without it the call exits 64

### Rehearse with --dry-run
- `--dry-run` is accepted only by `vps remove`, `vps import`, `vps edit`, `vps rename`, `sftp rm`, `sftp rmdir`, `secrets init` and `secrets reencrypt`
- Anywhere else it is rejected with exit 64 rather than accepted and ignored, so a rehearsal is never mistaken for an operation that already ran
- The plan of `vps remove`, `vps edit` and `vps rename` echoes `target_resolved`, `target_source` (`argv`), `host_resolved` and `host_source` after the lookup succeeds
- A missing host still exits 66 and creates no directory and no lock file
- The `vps import` preview lists every host under `hosts` with `name`, `replaces_existing` and `incomplete`, so read `replaces_existing` before the real import
- The `secrets init --force` preview reports `hosts_to_reencrypt`, and rotating the key without re-encrypting those hosts loses their secrets at rest
- The `sftp rm` and `sftp rmdir` previews never connect and never check that the path exists, they echo `vps` and `remote`

```bash
ssh-cli vps remove web1 --dry-run --json
```

```json
{"ok":true,"event":"dry-run","config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"host_resolved":"web1","host_source":"argv","name":"web1","operation":"vps-remove","target_resolved":"web1","target_source":"argv"}
```

- Expected output of three other previews, `CONFIG_DIR` stands for your config directory

```bash
ssh-cli --json --dry-run vps import --file hosts.json
ssh-cli --json --dry-run secrets init --force
ssh-cli --json --dry-run sftp rm demo /tmp/old.log
```

```json
{"ok":true,"event":"dry-run","config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"hosts":[{"incomplete":false,"name":"demo","replaces_existing":true},{"incomplete":false,"name":"edge","replaces_existing":true}],"imported":2,"operation":"vps-import","source":"CONFIG_DIR/hosts.json"}
{"ok":true,"event":"dry-run","config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"force":true,"hosts_to_reencrypt":2,"keyring":false,"operation":"secrets-init"}
{"ok":true,"event":"dry-run","dry_run":true,"executed":false,"operation":"sftp-rm","remote":"/tmp/old.log","vps":"demo"}
```

### Repair a legacy host name and check the config directory
- An older `config.toml` with a name outside `[A-Za-z0-9._-]` still loads, lists and resolves
- `doctor` lists such names, escaped, in `local.invalid_host_names`
- `vps rename '<legacy name>' <new name>` repairs one because the source is not validated
- A name with a control character is refused as a target by `exec`, `health-check`, `--hosts` and `connect` with exit 64
- `doctor` also reports `config_dir_exposed` and `config_dir_permissions` for the config directory
- `--select` reaches the `doctor` object too, as in `ssh-cli doctor --select local.invalid_host_names`, and the blocks below use `jaq` only to print the bare value

```bash
ssh-cli doctor --json | jaq -c '.local.invalid_host_names'
ssh-cli vps rename 'legacy host' legacy-host --json
ssh-cli doctor --json | jaq -c '.local | {invalid_host_names,config_dir_exposed,config_dir_permissions}'
```

```json
["legacy host"]
{"ok":true,"event":"vps-renamed","active_rewritten":false,"host_resolved":"legacy host","host_source":"argv","name":"legacy-host","previous_name":"legacy host","target_resolved":"legacy host","target_source":"argv","updated":[]}
{"invalid_host_names":[],"config_dir_exposed":false,"config_dir_permissions":"0700"}
```

### Safer agent automation
- Feed secrets through stdin flags (`--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--key-passphrase-stdin`) instead of argv
- Attach shell comments with `--description` for audit-friendly remote history
- Disable elevation for untrusted tasks with `--disable-sudo`, per call as a global flag, except before `vps add` or `vps edit`, where it is stored in the record
- Persist the block in the record with `vps edit <NAME> --disable-sudo` and lift it with `--enable-sudo`
- Replace a legitimate host key only after human confirmation, with `--replace-host-key` (trust on first use, TOFU)
- Pass `--no-input` in unattended runs so a missing stdin fails fast
- `--no-input` with a stdin flag such as `--password-stdin` exits 64, because one forbids what the other reads
- The remote `stdout` and `stderr` of `exec` stay raw by design, so strip terminal control bytes before showing them to a person
- Only `vps add --timeout` under 1000 ms warns on stderr, while `vps edit`, `exec` and `health-check` accept it silently
- A password-like value on argv (`--password` and its siblings) warns on stderr in `vps add`, `vps edit` and `exec`, so do not parse those lines as an error envelope
- On `scp --json` or `sftp --json` failure, parse the JSON error envelope on stderr (`exit_code`, `message`)

### Export and import
- Export a redacted inventory with `ssh-cli --output-format text vps export -o hosts.toml`
- The body follows the resolved format, so without `--output-format text` it comes out JSON because an agent's stdout is never a TTY
- A redacted export clears every secret: `password` is `""` and `key_passphrase`, `sudo_password` and `su_password` are `null`
- A fake empty `sshcli-enc:` ciphertext is never written
- `vps list` and `vps show` mask non-empty secrets with `FIXED_MASK` (three asterisks)
- Agent JSON export is `ssh-cli vps export --json`, which prints the envelope `vps-export`
- `--include-secrets` requires `-o` or `--output`, or `--i-understand-secrets-on-stdout`, otherwise it exits 64
- Import with `ssh-cli vps import --file hosts.toml` (TOML with English keys or legacy Portuguese aliases) or a JSON `vps-export` envelope
- The user key is `username` in TOML (legacy `usuario`) and `user` in JSON
- `vps import` checks every name first and lists all invalid names at once with exit 64 and nothing written, also under `--dry-run`
- The success event carries `imported` and `replaced`, the sorted names of hosts it overwrote
- Use `--allow-incomplete` for redacted or skeleton hosts without full auth
- `vps import --file` with a file that does not exist exits 66 with `file_not_found`, `error_class` permanent and `retryable` false, so check the path instead of retrying
- `added_at` is optional on import and defaults to now
- The wire inventory is schema v3, new writes use English keys and loads still accept the legacy Portuguese aliases
- Re-encrypt a plaintext inventory after an upgrade with `ssh-cli secrets reencrypt`
- An export of a legacy host whose name fails the strict charset restores only after `vps rename`
- The export creates missing output directories with mode 0700 and never changes an existing one
- List and show print an empty password as JSON `null`, a different path from export
- With `-o` the stdout is a short summary of the written file, and the body stays in the file

```bash
ssh-cli vps export -o hosts.json --json
ssh-cli vps import --file hosts.json --json
```

```json
{"ok":true,"event":"vps-export","format":"json","include_secrets":false,"path":"CONFIG_DIR/hosts.json"}
{"ok":true,"event":"vps-import","imported":2,"replaced":["demo","edge"]}
```

- A redacted export of a password host is refused by `vps import` with exit 64, complete the host with `vps edit` or pass `--allow-incomplete`


## Tunnel Modes

### One bind, one SSH session, four shapes
- `tunnel` opens one local bind and one SSH session per invocation (G-PAR-30)
- Several tunnels mean several one-shots with distinct ports
- Every mode requires `--timeout-ms`, a tunnel without a deadline is a daemon and this CLI does not ship one
- Read the JSON `mode` field to know the shape: `local`, `reverse`, `socks5` or `streamlocal`
- Default local forward reaches one fixed remote address

```bash
ssh-cli tunnel prod 15432 10.0.0.5 5432 --timeout-ms 60000 --json
```

- Expected output of a 3 second local tunnel on port 18080, two documents and exit 0

```json
{"ok":true,"event":"tunnel_listening","vps":"demo","local_port":18080,"remote_host":"127.0.0.1","remote_port":22922,"timeout_ms":3000,"bind":"127.0.0.1","mode":"local"}
{"ok":true,"event":"tunnel_closed","vps":"demo","reason":"deadline","bind":"127.0.0.1","local_port":18080,"forwards_served":0,"capacity_waits":0,"duration_ms":3001,"mode":"local"}
```

- Wait for `tunnel_listening` before using the local port
- `tunnel_closed` carries `reason`, `forwards_served` and `capacity_waits`, and it tells a clean deadline from a saturated semaphore
- `reason` is `deadline` (exit 0), `signal` or `accept_error`, so read it together with the exit code
- A SIGTERM after `tunnel_listening` printed this document and exited 143, and a SIGINT printed the same document and exited 130

```json
{"ok":true,"event":"tunnel_closed","vps":"lab","reason":"signal","bind":"127.0.0.1","local_port":34439,"forwards_served":0,"capacity_waits":0,"duration_ms":239,"mode":"local"}
```

- A tunnel without `--timeout-ms` is not a daemon, it is a usage error that exits 2
- `--socks5` serves a local SOCKS5 proxy (RFC 1928, no-auth plus CONNECT) and picks the destination per connection, so `REMOTE_HOST` and `REMOTE_PORT` are omitted (G-TUN-R02)
- The proxy refuses BIND and UDP ASSOCIATE and sends host names to the server unresolved, so DNS happens on the remote side
- Under `--socks5` the events report `remote_host` `*` and `remote_port` 0, and under `--remote-socket` they report the socket path in `remote_host` and `remote_port` 0

```bash
ssh-cli tunnel prod 1080 --socks5 --timeout-ms 300000 --json
```

- `--remote-socket` reaches a Unix socket on the server through `direct-streamlocal@openssh.com`, which is how Docker, PostgreSQL and systemd become reachable (G-TUN-R03)
- The socket path must be absolute or the call exits 64, and local existence is never consulted
- The client may run on Windows because locally it only speaks TCP, the server must support the extension

```bash
ssh-cli tunnel prod 2375 --remote-socket /var/run/docker.sock --timeout-ms 60000 --json
```

- `--reverse` lets the server listen and deliver back to your local port, for a callback webhook, a remote debugger or an inverted bastion (G-TUN-R01)
- `REMOTE_PORT` may be `0` under `--reverse`, the server allocates the port and reports it in `local_port`
- A local forward cannot accept `0` because there would be nothing to connect to

```bash
ssh-cli tunnel prod 8080 0.0.0.0 9000 --reverse --i-accept-network-exposure --timeout-ms 120000 --json
```

### Bind safety and shutdown
- The local `--bind` defaults to `127.0.0.1` and clap validates it as an IP, so a typo such as `127.0.0..1` exits 2 at parse time (G-TUN-R08)
- The remote bind under `--reverse` is the positional `<remote_host>` and it is deliberately not parsed as an IP, because RFC 4254 gives meaning to names and to the empty string
- A typo there exits 64 from the exposure guard and not 2 from clap
- Under `--reverse` the `--bind` flag is accepted and then ignored, delivery is forced to loopback and nothing warns
- Set the server-side address through `<remote_host>`
- Any routable bind requires `--i-accept-network-exposure` (G-TUN-R13), without it the forwarded service would reach the whole local network in silence
- Under `--reverse` that flag guards the server's bind address, since that is the exposed end
- A pre-bind timeout still exits 74 and a post-bind deadline exits 0 (TUN-002)
- `tunnel` takes the same auth flags as `exec`: `--password-stdin`, `--key`, `--key-passphrase` and `--key-passphrase-stdin` (CLI-005)


## Configuration

### XDG multi-host registry
- Resolve the config path with `ssh-cli vps path`
- Writes to `config.toml` are atomic with mode 0600 (tempfile, fsync and flock)
- The sibling files `active`, `known_hosts`, `lang` and `secrets.key` live beside the config
- `config.toml.lock` is the registry lock that `--lock-wait-ms` waits on, and a writer that loses the race exits 69 with `retryable` true
- A directory the CLI creates is born 0700, an existing default XDG directory is tightened to 0700 when it is not a symlink and belongs to you
- Any other existing directory is never chmodded, `doctor` shows `config_dir_exposed` and `config_dir_permissions` instead
- `--config-dir something.toml` uses the parent directory for everything: `secrets.key`, `known_hosts`, `active`, `lang` and the TLS files
- Two config tables that normalize to the same name exit 65 with `config has two hosts named ...`, so no host is lost silently
- `known_hosts.lock` and `lang` are created 0600 with no window before the chmod, and `vps export -o new/deep/e.json` creates the missing levels 0700
- Windows has no chmod, so the directory relies on the profile ACL
- Store timeout, `max_command_chars`, `max_output_chars`, sudo and su secrets per host
- Default at-rest encryption is ChaCha20-Poly1305 and secrets become `sshcli-enc:v2:…` blobs, and the legacy `v1` blobs are still read
- Primary-key control is CLI and XDG only, resolved in this order: `--secrets-key-file <PATH>`, the OS keyring under `--use-keyring`, XDG `secrets.key`, then the keyring entry of an earlier `secrets init --keyring`
- A keyring with no backend answering exits 69 on write, and under `--use-keyring` a read falls back to `secrets.key`
- `SSH_CLI_SECRETS_KEY` and `SSH_CLI_SECRETS_KEY_FILE` are rejected fail-closed, an environment variable is not a store
- When either is set, every command that loads registry secrets exits 64 with `are not supported`: `doctor`, `secrets status`, `secrets reencrypt`, `vps add` with a password, and `vps list`, `vps show`, `vps export` and `vps edit` once the registry holds a stored secret
- Commands that never touch secrets, such as `vps path`, `locale show`, `schema`, `commands` and `tls provider`, exit 0 with the variable set, and the other `SSH_CLI_*` variables are ignored
- The tests-only plaintext opt-out is `--allow-plaintext-secrets`
- `vps doctor --json` reports paths, schema, host count, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file` and `secrets_plaintext_opt_out`
- It also reports `active_vps`, `config_dir_exposed`, `invalid_host_names` and `config_dir_permissions` under `local`
- `config_dir_permissions` is a four-digit octal string such as `0700` and is absent off Unix
- `permissions` still describes `config.toml` only
- There is no product `.env` runtime store


## Common Errors
- A clap usage error (exit 2) and a malformed `--filter` (exit 64) print plain text on stderr even with `--json`, only execution errors carry the JSON envelope
- Execution errors print one JSON envelope on stderr and leave stdout empty
- The envelope always carries `exit_code`, `error_code`, `message`, `error_class` and `retryable`, and may add `suggestion`, `remote_exit_code`, `target_resolved`, `target_source` and `active_fallback`
- Read `retryable` before any retry, the example below is a permanent error that stderr printed on one line

```bash
ssh-cli vps rename ghost new --json
```

```json
{"exit_code":66,"error_code":"vps_not_found","message":"vps 'ghost' not found in registry","error_class":"permanent","retryable":false,"suggestion":"the host is not in the registry; if an earlier `vps rename` or `vps remove` may have succeeded, run `vps list` before repeating"}
```

- Read the exit codes and copyable examples offline with `ssh-cli --help`, they sit under `Examples:` and `Exit status:` and cost no network
- `--help` of every one of the 48 leaves adds an `Examples:` block, and `-h` stays short
- A failure before dispatch, such as an invalid `--filter`, exits 64 in prose without an envelope, and the prose follows the system language, so pass `--lang en` when a script matches the text
- A `--filter` without an operator and one with an empty key such as `=prod` exit 64, while `name~~prod` and an empty value as in `name=` are accepted

| Symptom | Exit | Fix |
| --- | --- | --- |
| `exec demo --step "cmd"` with no command | 64 | A lone positional is a usage error, pass `exec demo "cmd"` or `exec --use-active "cmd"` |
| `vps edit --key K --password-stdin` | 2 | Pick one auth flag, clap refuses two primary methods |
| `vps edit --key K` with a missing file | 66 | The key path must exist, check it with `ls` |
| `vps edit --no-key` on a key-only host | 64 | Add another method first, the host would authenticate nowhere |
| `vps edit demo` with no field | 64 | Pass at least one field, `--name`, `--tag` or a clear flag |
| `vps add --name 'bad name'` | 64 | Use ASCII letters, digits, dot, hyphen and underscore |
| `vps import` lists invalid names | 64 | Fix the names in the file, nothing was imported |
| `vps rename ghost new` | 66 | The source does not exist, run `vps list` |
| `tls acme issue example.com` | 2 | Use `--domain example.com` |
| `health-check --all` with a dead host | 1 | Read `results[].status` and `error`, siblings are not aborted |
| `tls acme issue --domain example.com` without `--print-challenge` | 64 | Add `--print-challenge`, the flow has two steps and never waits |
| `tls acme complete --domain example.com` with no pending order | 66 | Run `tls acme issue --domain example.com --print-challenge` first |
| `vps import` of a redacted export | 64 | Complete the hosts with `vps edit` or pass `--allow-incomplete` |
| `--no-input` with `--password-stdin` | 64 | Drop `--no-input` when a secret arrives on stdin on purpose |
| `tunnel` without `--timeout-ms` | 2 | Pass `--timeout-ms`, a tunnel without a deadline does not exist |
| `--max-concurrency 0` or above 64 | 2 | Pick 1 to 64 or omit the flag for the automatic value |
| `--dry-run` on a command without a preview | 64 | Remove `--dry-run`, only the eight previews listed above exist |
| `sudo-exec` on a host with sudo disabled | 77 | Run `vps edit NAME --enable-sudo` when the block is persistent |
| `su-exec` without a stored su password | 64 | Run `vps edit NAME --su-password-stdin`, the error code is `su_password_missing` |
| `--filter nameprod` or `--filter =prod` | 64 | Write `key=value`, `key!=value` or `key~substring` with a non-empty key, the answer is prose and not an envelope |
| `exec --use-active demo "cmd"` with a host name | 64 | Pass only the command, `--use-active` already names the host |
| `exec --use-active --all "cmd"` | 2 | Pick one route, the active marker and a selector exclude each other |
| `exec --use-active` with no active marker | 66 | Run `ssh-cli connect NAME` first, the error code is `no_active_vps` |
| `SSH_CLI_SECRETS_KEY` set in the environment, on a command that loads secrets | 64 | Unset it, the key comes from `secrets.key`, `--secrets-key-file` or `--use-keyring` |
| Config lock held by another process | 69 | Retry, the error is retryable, tune `--lock-wait-ms` |
| Connection refused or timeout | 74 | Check host, port and network, retry at most twice |
| Authentication failed or host key mismatch | 77 | Change credentials, never blind-retry, confirm key changes with a human |


## Exit codes (sysexits)
- This table matches the `Exit status:` section printed by `ssh-cli --help`, both list the same 14 codes

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | General runtime failure, for example a remote non-zero exit (stdout keeps the `exec` object and stderr gets `command_failed` with `remote_exit_code`) or a partial multi-host failure |
| 2 | Clap usage error (invalid flags), stderr holds prose with `error:`, `tip:` and `Usage:` lines and no JSON envelope |
| 64 (`EX_USAGE`) | Invalid argument or domain usage, including an empty command, a refused `--include-secrets` without `-o` or ack, a refused `--dry-run` and ACME permanent validation |
| 65 (`EX_DATAERR`) | Invalid TOML or JSON input data, schema incompatibility, two hosts with the same normalized name |
| 66 (`EX_NOINPUT`) | VPS not found, no active VPS or missing file such as a key, a local transfer source or a `vps import --file` (`file not found: <path>`), permanent and never retried |
| 69 (`EX_UNAVAILABLE`) | A host service is not answering (OS keyring or config lock), transient and retryable |
| 70 (`EX_SOFTWARE`) | Internal failure with no user-fixable input, permanent |
| 73 (`EX_CANTCREAT`) | Config write or create failure |
| 74 (`EX_IOERR`) | Connection, IO or timeout failure |
| 77 (`EX_NOPERM`) | Authentication failed, host-key policy, permission, sudo disabled or a secret that fails to decrypt |
| 130 | SIGINT |
| 141 | Broken pipe on stdout or stderr, the reader closed the pipe and the output is incomplete, never retry it blindly |
| 143 | SIGTERM |

- Product line: 0.5.6


## Next Steps
- Copy ready recipes from [COOKBOOK.md](COOKBOOK.md)
- Follow the agent contract in [AGENTS.md](AGENTS.md)
- Check platform notes in [CROSS_PLATFORM.md](CROSS_PLATFORM.md)
- Upgrade with [MIGRATION.md](MIGRATION.md)


## Integration With AI Agents
- Load the skill package under `skills/ssh-cli-en/`
- Prefer JSON output for tool parsing
- Follow exit-code routing before retries, see [AGENTS.md](AGENTS.md)
- Read [AGENTS.md](AGENTS.md) and [../INTEGRATIONS.md](../INTEGRATIONS.md)
- Event and payload shapes live in [docs/schemas/README.md](schemas/README.md)
- The binary embeds the same 25 schemas, list them with `ssh-cli schema` and print one with `ssh-cli schema <NAME>`
- Never log the primary-key, host passwords or decrypted secrets
