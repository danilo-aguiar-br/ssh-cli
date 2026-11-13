# ssh-cli: one-shot SSH CLI for LLM agents


[![docs.rs](https://img.shields.io/docsrs/ssh-cli)](https://docs.rs/ssh-cli)
[![crates.io](https://img.shields.io/crates/v/ssh-cli)](https://crates.io/crates/ssh-cli)
[![License](https://img.shields.io/crates/l/ssh-cli)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.89-orange)](https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/)
[![Downloads](https://img.shields.io/crates/d/ssh-cli)](https://crates.io/crates/ssh-cli)
[![Rust](https://img.shields.io/badge/rust-1.89%2B-blue)](https://www.rust-lang.org)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

> Give any large language model (LLM) remote SSH power through one memory-safe one-shot command-line interface (CLI) binary.

- Read this document in [Portuguese (pt-BR)](README.pt-BR.md)
- Install with `cargo install ssh-cli --locked` and verify with `ssh-cli --version`
- Current version is `0.5.6` with minimum supported Rust version (MSRV) `1.89`
- Map the product for LLMs in [llms.txt](llms.txt) and [llms-full.txt](llms-full.txt)
- Jump to [Quick Start](#quick-start), [Documentation](#documentation) or the [Cookbook](docs/COOKBOOK.md)


## What is it?
### One-shot multi-host SSH CLI for agents
- Ship a single Rust binary with zero Node runtime and zero daemon
- Operate N VPS hosts from XDG storage without `.env` files
- Authenticate per host with a password, a private key or the SSH agent
- Run `exec`, `sudo-exec` and `su-exec` as pure one-shot processes that die after the command
- Return structured JSON on stdout for orchestration, with execution failures in a stable envelope on stderr
- Collect no telemetry in any build


## Why ssh-cli?
### Replace long-lived SSH processes with a die-after-run binary
- Avoid resident processes that hold sockets and RAM between tasks
- Register multi-host credentials once, with at-rest encryption and atomic writes
- Trust host keys through trust on first use (TOFU) `known_hosts` instead of always-trust
- Route errors with sysexits codes that agents classify reliably


## What's New in 0.5.6
- `russh` 0.65.1 closes RUSTSEC-2026-0492, RUSTSEC-2026-0495 and two GHSA advisories, and the MSRV is now Rust 1.89 because russh 0.65 requires it
- `secrets init --keyring` writes to the operating system store, and an unavailable store exits 69 instead of reporting success
- `--select` and `--truncate-content` now reduce a single record, so `commands --select tree.about` returns one field
- A reduced bare array writes one `{"event":"agent-shape",...}` line to stderr, and `--quiet` suppresses it
- A collection flag passed on a single record is named in `agent_shape.ignored` instead of being dropped in silence
- Every one of the 48 leaves shows an `Examples:` block in `--help`
- The TLS root store no longer carries a distrusted authority, and a new gate compares the advisory database with the resolved tree
- Rename a host natively with `vps rename` or `vps edit --name`, keeping every credential
- Switch the authentication method of a host in one `vps edit` call
- Host names follow a strict ASCII rule, and legacy names still load and can be repaired
- Text mode escapes terminal control sequences
- A new config directory is born `0700`, and `vps doctor` reports `config_dir_exposed`, `config_dir_permissions` and `invalid_host_names`
- `--dry-run` covers `vps edit` and `vps rename` and echoes the target
- `--lock-wait-ms` bounds the config lock wait and exits 69
- Read the full list in [CHANGELOG.md](CHANGELOG.md)


## Superpowers for AI Agents
### Remote execution
- Run `exec`, `sudo-exec` and `su-exec` one-shot, with safe `sudo` packing via `sh -c` and shell escape
- Name the target on purpose with `<VPS> <COMMAND>`, a selector (`--all`, `--hosts a,b`, `--tags prod`) or `--use-active`
- A lone positional exits 64, so a host name is never run as a remote binary
- Verify the target afterwards with `target_resolved`, `target_source` and `active_fallback` in the single-host envelope
- Read `host_resolved` and `host_source` as the read aliases of the same fields
- Add extra commands on the same SSH session with `--step`
- `exec --json` with one step emits exactly one NDJSON success object
- Apply the dual limits `max_command_chars` and `max_output_chars`
- Abort on timeout with a best-effort remote abort
- The remote stdout and stderr of `exec` stay raw by design

### Host registry
- Manage hosts with `vps add`, `vps list`, `vps show`, `vps edit`, `vps remove`, `vps rename`, `vps path`, `vps doctor`, `vps export` and `vps import`
- Rename natively with `vps rename <NAME> <NEW_NAME>` or `vps edit <NAME> --name <NEW>`, and every credential stays readable
- The rename emits `vps-renamed` with `previous_name` and `active_rewritten`
- Switch the authentication method in one call: `vps edit --key` on a password host, `--password-stdin` on a key host, or `--use-agent`
- Each method clears the other ones, and `updated` lists the new method plus only the fields that had a value
- Two primary auth flags in one call exit 2, and a passphrase without a key exits 64 with the record untouched
- A different `--key` drops the old passphrase unless the call brings a new one
- `--agent-socket` alone implies `--use-agent`
- Remove a field with the eight clear flags `--no-key`, `--no-agent-socket`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key` and `--clear-tags`
- Accept host names only in ASCII `[A-Za-z0-9._-]`, starting with a letter or digit, from 1 to 255 bytes
- The strict rule applies on `vps add`, on the destination of `vps rename` and on `vps import`
- `vps import` validates every item, lists all invalid names at once, exits 64 and writes nothing, also under `--dry-run`
- A legacy name with control or bidi characters still loads, and `vps rename '<legacy name>' <new>` repairs it
- Two `config.toml` tables that normalize to the same NFC name exit 65 with `config has two hosts named ...`
- `vps add --disable-sudo` and `vps edit --disable-sudo` persist in the host record, `vps edit --enable-sudo` undoes it, and `--disableSudo` is refused on those two verbs
- Wait for the config lock up to `--lock-wait-ms` (default 5000 ms, maximum 300000) and then exit 69 instead of hanging
- Store wire schema v3 with English TOML keys, and read legacy PT aliases on load

### Preview with `--dry-run`
- Preview eight operations: `vps remove`, `vps import`, `vps edit`, `vps rename`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt`
- Any other command with `--dry-run` exits 64 instead of ignoring the flag
- The plan of `vps edit`, `vps rename` and `vps remove` echoes `target_resolved`, `target_source`, `host_resolved` and `host_source`
- The three verbs exit 66 on a missing config without creating a directory or a lock

### Output safety
- Text mode escapes ESC, BEL, bidi and format characters as `\u{1b}`, in messages, errors, `vps list`, `vps show`, `health-check`, `vps doctor` and batch output
- `sftp ls` and `sftp stat` also escape `\n` and `\t` in remote names, so a hostile name cannot forge a listing line
- JSON keeps the original value, because it is for machines
- Shape the payload before serialization with `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content` and `--max-output-bytes`
- The collection flags reduce the `vps list` array, the `results` of batch envelopes and the `entries` of `sftp ls`
- A single object such as `vps show`, a one-host `exec` or `vps doctor` is reduced by `--select` and `--truncate-content`, so `vps show prod --select name` returns one field
- The collection flags (`--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--max-output-bytes`) cannot apply to one record, and they are named under `agent_shape.ignored` instead of being silently dropped
- A reduced envelope gains `agent_shape` with `input_count`, `output_count` and `dropped`, and a reduced single record gains `selected`, `content_truncated` and `ignored`
- A reduced bare array keeps its type and reports the drop on stderr as `{"event":"agent-shape",…}`, which `--quiet` suppresses
- Default tracing filter is `error`, which keeps stderr clean
- Graduated verbosity `-v`, `-vv` and `-vvv` is always crate-scoped (`warn,ssh_cli=*`), so russh never dumps passwords
- Ambient `RUST_LOG` is ignored

### Private by default
- A new config directory is born `0700`, and `vps doctor` reports `config_dir_permissions`, `config_dir_exposed` and `invalid_host_names` under `local`
- An existing default XDG directory is tightened to `0700` when it is not a symlink and belongs to you
- Any other existing directory is never chmodded: it logs a warning (visible with `-v`) and shows up in `doctor`
- `--config-dir algo.toml` uses the parent of the `.toml` path as the config directory for every file
- The directories created by `vps export -o` are born `0700`, and `known_hosts.lock` and `lang` are born `0600`
- `tls mtls import --name` refuses control, format and bidi characters with exit 64 and creates no directory, while Unicode such as `serviço-1` stays valid

### Transfers
- Secure Copy Protocol (SCP) uploads and downloads regular files only, with no recursive directories
- SCP flags follow `exec`: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--json` (contract `docs/schemas/scp-transfer.schema.json`)
- SCP JSON success carries `event: "scp-transfer"`, and a missing remote file exits 66 with `file not found: <path>`
- An SCP download writes `{path}.ssh-cli.partial` and then renames atomically, and a failed `sync_data` stops the rename
- Upload streams 32 KiB chunks
- Remote mtime and mode preservation is best-effort and reported as `mtime_preserved` and `durable`
- SSH File Transfer Protocol (SFTP) uses the subsystem: `sftp upload`, `sftp download` (optional `--recursive`, symlinks never followed), `sftp ls`, `sftp mkdir`, `sftp rmdir`, `sftp rm`, `sftp stat`, `sftp rename`
- SFTP JSON events are `sftp-transfer`, `sftp-list`, `sftp-fs-op` and `sftp-batch`
- SFTP upload preserves payload bytes and never truncates to zero
- `SFTP_PERM_MASK` (0o7777) applies on upload, and `SFTP_PERM_MASK_UNTRUSTED` (0o0777) strips setuid bits that a server sends on download
- Batch cancel pads the cancelled remainder, so the results length matches the input

### Tunnels and health
- Open a bounded tunnel with the mandatory `--timeout-ms` and an optional `--bind` (default `127.0.0.1`)
- A non-loopback `--bind` requires `--i-accept-network-exposure`
- `tunnel --json` emits `tunnel_listening` after the local bind and `tunnel_closed` at the end
- A post-bind deadline exits 0, while a pre-bind timeout exits 74
- A SIGINT exits 130 and a SIGTERM exits 143, and after the bind `tunnel_closed` carries `reason: signal`
- Use `--socks5` for a SOCKS5 proxy, `--remote-socket PATH` for a Unix socket on the remote host and `--reverse` for a server-side listener
- Probe latency with `health-check` and an optional `--timeout`, for one host or the fleet with `--all`

### Secrets and TLS
- Encrypt secrets at rest by default with ChaCha20-Poly1305 and an automatic XDG `secrets.key`
- Choose the key with `--secrets-key-file`, `--use-keyring` or XDG `secrets.key`, and opt out only for tests with `--allow-plaintext-secrets`
- Manage the master key with `secrets status`, `secrets init` and `secrets reencrypt`, with events `secrets-init` and `secrets-reencrypt`
- The first secret write folds `secrets_key_auto_created: true` into the same `vps-added` JSON
- Pass secrets through stdin flags to keep them out of argv
- Keep TOFU `known_hosts` and atomic config writes under flock
- Use SSH-over-TLS, mutual TLS (mTLS) identities and Automatic Certificate Management Environment (ACME) DNS-01 certificates through `ssh-cli tls`

### Export and import
- `vps export` body follows the resolved output format: JSON whenever stdout is not a TTY, even when `-o` names a `.toml` file
- A TOML body requires `--output-format text`
- Export is redacted by default: a password comes out as an empty string `""` and absent secrets as `null`, never as ciphertext
- The fixed three-asterisk mask `FIXED_MASK` exists only in `vps list` and `vps show`
- `--include-secrets` into a pipe or non-TTY needs `-o` or `--i-understand-secrets-on-stdout`
- A key-only host shows an empty password as JSON `null` in `vps list` and `vps show`
- `vps import` accepts TOML (EN keys plus PT aliases) or a JSON `vps-export` envelope, and redacted skeletons need `--allow-incomplete`
- A host exported with a legacy name outside the strict charset restores only after `vps rename`

### Discovery
- Print every contract with `ssh-cli schema` (25 embedded JSON Schemas) and the full command tree with `ssh-cli commands`
- Complete the shell with `ssh-cli completions` for bash, zsh, fish, powershell and elvish
- Pick the UI language between `en-US` and `pt-BR` automatically


## Quick Start
### Install and run the first remote command

```bash
cargo install ssh-cli --locked
printf %s 'demo-password-not-real' | ssh-cli vps add --name prod --host prod.example.com --port 22 --user admin --password-stdin
ssh-cli exec prod "hostname" --json
```

### Example output of the second command (paths vary)

```json
{"ok":true,"event":"vps-added","key_file":"~/.config/ssh-cli/secrets.key","key_source":"xdg_file","name":"prod","secrets_key_auto_created":true}
```


## Installation
### Choose the install path that matches your environment
- Prefer crates.io with the lockfile: `cargo install ssh-cli --locked`
- Upgrade after a release with `cargo install ssh-cli --locked --force`
- Rebuild from a checkout with `cargo install --path . --locked`
- Do not install without `--locked` unless you verified that the crypto pins resolve cleanly
- Build musl with the allocator feature on Alpine: `--features musl-allocator`
- Require Rust `1.89` or newer (MSRV, declared as `rust-version` in `Cargo.toml`)

### Shell completions
- Bash: `ssh-cli completions bash > ssh-cli.bash` and source the file from your profile
- Zsh: `ssh-cli completions zsh > _ssh-cli` and place it in a directory of your `fpath`
- Fish: `ssh-cli completions fish > ~/.config/fish/completions/ssh-cli.fish`
- PowerShell: `ssh-cli completions powershell > ssh-cli.ps1` and dot-source it from your profile
- Elvish: `ssh-cli completions elvish > ssh-cli.elv`

### Languages
- The UI speaks `en` and `pt-BR`
- Resolution order is `--lang`, then the XDG `lang` file written by `ssh-cli locale set <code>`, then the system locale, then `en`
- Error bodies stay in English by design, so an agent receives stable text


## Features
### Cargo feature flags

| Feature | Default | Description |
|---|---|---|
| `ssh-real` | yes | Real SSH via `russh` + `aws-lc-rs` (compression `none` only) |
| `tls` | yes | rustls + `aws_lc_rs`: SSH-over-TLS, mTLS, ACME, where `Cargo.toml` asks for 0.23.25 and the security floor that `tls provider` prints as `min_rustls` is 0.23.18 |
| `musl-allocator` | no | `mimalloc` as global allocator (binary, useful on musl and Alpine) |

- The install path always enables the defaults (`ssh-real` and `tls`)
- Disable real SSH and TLS only for dependency diagnosis: `cargo build --no-default-features`
- docs.rs builds with `all-features = true` and `--cfg docsrs` (see `[package.metadata.docs.rs]` in `Cargo.toml`)

### Crypto policy
- Default transport is SSH-2 on plain TCP (`russh` + aws-lc-rs)
- Host keys use TOFU under XDG (`known_hosts`)
- Optional SSH-over-TLS (`vps add --tls`) runs a rustls handshake and then SSH on the TLS stream
- The binary installs the `aws_lc_rs` `CryptoProvider` once in `main`
- mTLS and ACME material lives under XDG `tls/`, with no product env for certificate storage
- No OpenSSL, no `native-tls` and no dual `ring` provider
- SSH channel compression is forced to `none`
- ACME validation errors, `invalidContact` and 4xx problem types exit 64 and are not retryable, while rate limits stay transient with exit 74
- Details in [SECURITY.md](SECURITY.md#transport--crypto-policy-g-tls)


## Targets
### Platforms covered by docs.rs metadata
- `x86_64-unknown-linux-gnu` (default)
- `x86_64-apple-darwin` and `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `aarch64-unknown-linux-musl`
- Windows and macOS pass a type-check only, and the `0700` mode and `getuid` were not executed there
- Windows has no chmod, so the config directory depends on the profile ACL
- See [docs/CROSS_PLATFORM.md](docs/CROSS_PLATFORM.md) for runtime notes


## Usage
### Register hosts
- Prefer `--password-stdin` or `--key` over secrets in argv
- Add a password host with `vps add --password-stdin` (or `--password`)
- Add a key host with `vps add --key ~/.ssh/id_ed25519`
- Name hosts with ASCII letters, digits, `.`, `_` and `-`, starting with a letter or digit
- Mark the active host with `connect <name>`, which only writes a marker and opens no session

### Change hosts
- Switch from password to key in one call with `vps edit prod --key ~/.ssh/id_ed25519`
- Switch from key to password with `vps edit prod --password-stdin`
- Rename with `vps rename prod prod-old`, where a repeat after success exits 66 and `vps show prod-old` confirms
- Preview any of these with `--dry-run` and read `target_resolved` before the real call
- Repair a legacy name with `vps rename '<legacy name>' <new>`

### Execute
- Run a remote shell command with `exec <vps> "<cmd>"`
- Reach the whole fleet with `exec --all '<cmd>'`, a subset with `--hosts a,b` or a label with `--tags prod`
- Reach the `connect` marker only on purpose with `exec --use-active '<cmd>'`, which exits 66 when no marker exists
- Elevate with `sudo-exec` or `su-exec` when configured
- Disable elevation for one call with `--disable-sudo`, or re-enable it on a host with `vps edit --enable-sudo`

### Inspect
- Diagnose paths and the local state with `doctor --json` (or `vps doctor --json`)
- Read `local.config_dir_exposed`, `local.config_dir_permissions`, `local.invalid_host_names` and `local.active_vps` from that object
- Probe hosts over SSH with `vps doctor --probe-ssh` and a `--hosts` subset
- Discover contracts with `ssh-cli schema` and `ssh-cli commands`
- Export the inventory with `vps export`, which writes a JSON envelope on any non-TTY stdout


## Commands
### Shared flag sets
- Auth set: `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--use-agent`, `--agent-socket`
- Fleet selectors: `--all`, `--hosts <LIST>`, plus `--tags <LIST>` and `--use-active` where the table shows them
- Every row lists the leaf with its local flags, and the global flags are in the next sections

### Command groups

| Group | Purpose | Leaves |
|---|---|---|
| `ssh-cli vps` | Manage registered VPS hosts | `add`, `list`, `remove`, `rename`, `edit`, `show`, `path`, `doctor`, `export`, `import` |
| `ssh-cli scp` | SCP file transfer | `upload`, `download` |
| `ssh-cli sftp` | SFTP transfer and remote filesystem operations | `upload`, `download`, `ls`, `mkdir`, `rmdir`, `rm`, `stat`, `rename` |
| `ssh-cli secrets` | Manage the primary key and at-rest secret encryption | `status`, `init`, `reencrypt` |
| `ssh-cli locale` | Diagnose and manage the UI language | `show`, `set`, `clear` |
| `ssh-cli tls` | TLS stack: provider status, mTLS identities and ACME certificates | `provider`, `paths`, `mtls`, `acme` |
| `ssh-cli tls mtls` | Manage imported mTLS client identities under XDG `tls/mtls/` | `list`, `import`, `show`, `remove` |
| `ssh-cli tls acme` | ACME account and DNS-01 certificate lifecycle | `account`, `issue`, `complete`, `status`, `list` |
| `ssh-cli tls acme account` | Manage the ACME account | `create`, `show` |

### Host registry

| Command | Purpose | Local flags |
|---|---|---|
| `ssh-cli vps add` | Register a host | `--name`, `--host`, `--port`, `--user`, auth set, `--timeout`, `--max-command-chars`, `--max-output-chars`, `--max-chars` (legacy alias of `--max-command-chars`), `--sudo-password`, `--sudo-password-stdin`, `--su-password`, `--su-password-stdin`, `--disable-sudo`, `--tag`, `--tls`, `--tls-sni`, `--tls-client-cert`, `--tls-client-key`, `--check` |
| `ssh-cli vps list` | List hosts with secrets masked | `--tag` |
| `ssh-cli vps show` | Show one host masked | `<NAME>` |
| `ssh-cli vps edit` | Patch fields, switch the auth method and rename | `<NAME>`, `--name`, `--host`, `--port`, `--user`, auth set, `--timeout`, `--max-command-chars`, `--max-output-chars`, `--max-chars`, `--sudo-password`, `--sudo-password-stdin`, `--su-password`, `--su-password-stdin`, `--disable-sudo`, `--enable-sudo`, `--tls`, `--no-tls`, `--tls-sni`, `--tls-client-cert`, `--tls-client-key`, `--tag`, `--clear-tags`, `--no-key`, `--no-agent-socket`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key` |
| `ssh-cli vps remove` | Delete a host | `<NAME>` |
| `ssh-cli vps rename` | Rename a host and keep every credential | `<NAME> <NEW_NAME>` |
| `ssh-cli vps path` | Print the `config.toml` path | none |
| `ssh-cli vps doctor` | Show XDG layers, schema, paths and local state | `--probe-ssh`, `--hosts` |
| `ssh-cli vps export` | Export hosts, redacted by default | `--include-secrets`, `--output` (`-o`), `--i-understand-secrets-on-stdout` |
| `ssh-cli vps import` | Import TOML or a JSON `vps-export` envelope | `--file`, `--allow-incomplete` |

### Execution and probes

| Command | Purpose | Local flags |
|---|---|---|
| `ssh-cli connect` | Write the sibling `active` marker, which opens no session | `<NAME>` |
| `ssh-cli exec` | One-shot remote command, `exec-batch` JSON with a selector | `<VPS> <COMMAND>`, `--all`, `--hosts`, `--tags`, `--use-active`, `--step`, auth set, `--timeout`, `--description` |
| `ssh-cli sudo-exec` | One-shot `sudo` with safe `sh -c` packing | `<VPS> <COMMAND>`, `--all`, `--hosts`, `--tags`, `--use-active`, `--step`, auth set, `--sudo-password`, `--sudo-password-stdin`, `--timeout`, `--description` |
| `ssh-cli su-exec` | One-shot `su -` elevation | `<VPS> <COMMAND>`, `--all`, `--hosts`, `--tags`, `--use-active`, `--step`, auth set, `--su-password`, `--su-password-stdin`, `--timeout`, `--description` |
| `ssh-cli health-check` | Connectivity probe, `health-check-batch` JSON for a fleet | `<VPS_NAME>`, `--all`, `--hosts`, `--use-active`, auth set, `--timeout` |
| `ssh-cli tunnel` | Bounded port forward with a mandatory deadline | `<VPS_NAME> <LOCAL_PORT> [REMOTE_HOST] [REMOTE_PORT]`, `--timeout-ms`, `--bind`, `--i-accept-network-exposure`, `--socks5`, `--remote-socket`, `--reverse`, auth set |

### Transfers

| Command | Purpose | Local flags |
|---|---|---|
| `ssh-cli scp upload` | Upload a regular file, `scp-transfer` JSON | `--src`, `--dest`, `<VPS>`, `--all`, `--hosts`, auth set, `--timeout` |
| `ssh-cli scp download` | Download a regular file, `scp-transfer` JSON | `--src`, `--dest`, `<VPS>`, `--all`, `--hosts`, auth set, `--timeout` |
| `ssh-cli sftp upload` | Upload a file or tree over SFTP | `--src`, `--dest`, `<VPS>`, `--all`, `--hosts`, `--recursive`, auth set, `--timeout` |
| `ssh-cli sftp download` | Download a file or tree over SFTP | `--src`, `--dest`, `<VPS>`, `--all`, `--hosts`, `--recursive`, auth set, `--timeout` |
| `ssh-cli sftp ls` | List a remote directory, `sftp-list` JSON | `<VPS_NAME> <REMOTE>`, auth set, `--timeout` |
| `ssh-cli sftp mkdir` | Create a remote directory | `<VPS_NAME> <REMOTE>`, auth set, `--timeout` |
| `ssh-cli sftp rmdir` | Remove an empty remote directory, previews under `--dry-run` | `<VPS_NAME> <REMOTE>`, auth set, `--timeout` |
| `ssh-cli sftp rm` | Remove a remote file, previews under `--dry-run` | `<VPS_NAME> <REMOTE>`, auth set, `--timeout` |
| `ssh-cli sftp stat` | Show metadata of a remote path | `<VPS_NAME> <REMOTE>`, auth set, `--timeout` |
| `ssh-cli sftp rename` | Rename a remote path | `<VPS_NAME> <FROM> <TO>`, auth set, `--timeout` |

### Secrets and TLS

| Command | Purpose | Local flags |
|---|---|---|
| `ssh-cli secrets status` | Show encryption status, never the key | none |
| `ssh-cli secrets init` | Generate and store the primary key, previews under `--dry-run` | `--keyring`, `--force` |
| `ssh-cli secrets reencrypt` | Re-encrypt `config.toml` with the current key, previews under `--dry-run` | none |
| `ssh-cli tls provider` | Show the rustls `CryptoProvider` status (`aws_lc_rs`) | none |
| `ssh-cli tls paths` | Print the XDG TLS directory layout | none |
| `ssh-cli tls mtls list` | List imported identity names | none |
| `ssh-cli tls mtls import` | Import a PEM cert and key as a named identity | `--name`, `--cert`, `--key` |
| `ssh-cli tls mtls show` | Show the paths of one identity | `<NAME>` |
| `ssh-cli tls mtls remove` | Remove an identity directory | `<NAME>` |
| `ssh-cli tls acme account create` | Create an ACME account under XDG `tls/acme/account.json` | `--staging`, `--contact`, `--force` |
| `ssh-cli tls acme account show` | Show whether an account exists and its path | none |
| `ssh-cli tls acme issue` | Start a DNS-01 order and print the TXT challenge | `--domain`, `--staging`, `--print-challenge` |
| `ssh-cli tls acme complete` | Complete the order after the DNS TXT is published | `--domain` |
| `ssh-cli tls acme status` | Show certificate or pending status | `--domain` |
| `ssh-cli tls acme list` | List ACME domain directories | none |

### Discovery and language

| Command | Purpose | Local flags |
|---|---|---|
| `ssh-cli completions` | Generate shell completions | `<SHELL>` |
| `ssh-cli commands` | Emit the full command tree as JSON | none |
| `ssh-cli schema` | Emit the schema catalog or one schema body | `<NAME>` |
| `ssh-cli doctor` | Root alias of `vps doctor` | `--probe-ssh`, `--hosts` |
| `ssh-cli locale show` | Show the resolved language and the winning layer | none |
| `ssh-cli locale set` | Persist the language in the `lang` file (`0600`) | `<LOCALE>` |
| `ssh-cli locale clear` | Remove the persisted preference | none |

### Global flags
- Every global flag works before or after the subcommand

| Flag | Purpose |
|---|---|
| `--lang <LOCALE>` | Force the UI language (`en` or `pt-BR`) |
| `-v`, `--verbose` | Log verbosity on stderr: `-v` info, `-vv` debug, `-vvv` trace |
| `-q`, `--quiet` | Suppress non-JSON output |
| `--config-dir <DIR>` | Override the configuration directory |
| `--no-color` | Disable colored output |
| `--output-format <FORMAT>` | `json` or `text` (JSON when stdout is not a TTY) |
| `--json` | Force JSON on stdout |
| `--select <PATHS>` | Keep only these dotted paths in each record, with the alias `--fields` |
| `--filter <EXPR>` | Keep records matching `key=value`, `key!=value` or `key~substring` |
| `--limit <N>` | Emit at most N records |
| `--sort <PATH>` | Sort records ascending by dotted path |
| `--dedupe-by <PATH>` | Drop later records that repeat the value of the path |
| `--count-only` | Replace the records with `{"count": N}` |
| `--truncate-content <CHARS>` | Shorten strings longer than N characters |
| `--max-output-bytes <BYTES>` | Cap the envelope size by dropping trailing records |
| `--no-input` | Refuse to read stdin |
| `--dry-run` | Print the plan of a destructive operation and exit without executing it |
| `--lock-wait-ms <MS>` | Wait for the config lock, default 5000, then exit 69 |
| `--disable-sudo` | Disable `sudo-exec` and `su-exec` for this call only, with the alias `--disableSudo` accepted where the global flag is accepted, and before `vps add` or `vps edit` it also writes `disable_sudo: true` to the host record |
| `--replace-host-key` | Replace a diverging TOFU host key |
| `--allow-plaintext-secrets` | Allow plaintext secrets at rest, for tests only |
| `--secrets-key-file <PATH>` | Use a 64-hex primary-key file for this call |
| `--use-keyring` | Prefer the OS keyring for the primary key |
| `--timeout <MS>` | Default timeout for SSH operations, and a local `--timeout` wins |
| `--max-concurrency <N>` | Cap multi-host sessions and tunnel forwards, from 1 to 64 |
| `--fail-fast` | Stop admitting new multi-host units after the first failure |
| `--scp-file-concurrency <N>` | Concurrent SCP transfers on one session, default 1 |

### Tunnel modes
- `ssh-cli tunnel <vps> <port> <host> <rport> --timeout-ms N` forwards a local port, and concurrent accepts are gated by `--max-concurrency`
- `ssh-cli tunnel <vps> <port> --socks5 --timeout-ms N` serves SOCKS5 (RFC 1928, no-auth `CONNECT`), resolves names on the remote side and answers `BIND` and `UDP ASSOCIATE` with `0x07`
- `ssh-cli tunnel <vps> <port> --remote-socket PATH --timeout-ms N` forwards to a Unix socket, and `PATH` must be absolute
- `ssh-cli tunnel <vps> <port> <bind> <rport> --reverse --timeout-ms N` makes the server listen on `<bind>:<rport>` and deliver back to the local `<port>`
- With `--reverse`, `<rport> 0` lets the server allocate the port and report it in `local_port`


## Environment Variables
### Product knobs are flags and XDG files, never `SSH_CLI_*` stores

| Variable | Behavior | Example |
|---|---|---|
| `SSH_CLI_SECRETS_KEY`, `SSH_CLI_SECRETS_KEY_FILE` | Fail-closed: a command that resolves the primary key exits 64 when either is present | `SSH_CLI_SECRETS_KEY=x ssh-cli secrets status` |
| `SSH_CLI_HOME`, `SSH_CLI_LANG`, `SSH_CLI_FORCE_TEXT`, `SSH_CLI_USE_KEYRING` | Never read as config stores | `ssh-cli --config-dir ./c vps list` |
| `RUST_LOG` | Ignored by the product | `ssh-cli -vv exec h id --json` |
| `HOME` | OS home for XDG path resolution | `ssh-cli --config-dir ./c vps path` |
| `USERPROFILE` | Home fallback for `~` expansion when `HOME` is absent, as on Windows | `ssh-cli vps add --name w --host h --user u --key ~/.ssh/id_ed25519` |
| `TERM`, `NO_COLOR`, `CLICOLOR_FORCE` | Terminal and color detection only | `ssh-cli --no-color vps list` |
| `CI`, `FLATPAK_ID`, `SNAP`, `WSL_DISTRO_NAME`, `KUBERNETES_SERVICE_HOST`, `container`, `TERMUX_VERSION`, `TERMUX_APK_RELEASE`, `PREFIX` | Runtime detection only, shown as `local.runtime.*` in `vps doctor` | `ssh-cli vps doctor --json` |

- Never put host passwords in environment variables
- Use the registry and stdin flags instead

### Configuration precedence

| Control | How | Example |
|---|---|---|
| Config directory | `--config-dir`, else the XDG default | `ssh-cli --config-dir /tmp/ssh-cli-test vps list` |
| Language | `--lang`, then `ssh-cli locale set <code>`, then the system, then `en` | `ssh-cli --lang pt-BR vps list` |
| Output format | `--json` or `--output-format json\|text` | `ssh-cli exec h uptime --json` |
| Concurrency | `--max-concurrency N` (1 to 64, automatic formula when omitted) | `ssh-cli --max-concurrency 8 exec --all id --json` |
| Verbosity | `-v`, `-vv`, `-vvv` (info, debug, trace, crate-scoped) | `ssh-cli -vv exec h id --json` |
| Primary key | `--secrets-key-file`, `--use-keyring` or XDG `secrets.key` | `ssh-cli --secrets-key-file ./k secrets status` |
| Plaintext opt-out | `--allow-plaintext-secrets` (tests only) | `ssh-cli --allow-plaintext-secrets vps list` |


## Integration Patterns
### Wire agents with one-shot subprocesses only
- Parse one document per call: `ssh-cli exec prod "uptime" --json | jaq -r .stdout`
- Fan out over a label: `ssh-cli exec --tags prod "id" --json | jaq -c '.results'`
- Shape the payload before it exists: `ssh-cli --select name,host vps list --json`
- Keep secrets off argv: `printf %s "$PASS" | ssh-cli exec prod "hostname" --password-stdin --json`
- Rehearse a mutation, then run it: `ssh-cli --dry-run vps rename prod prod-old` and then `ssh-cli vps rename prod prod-old`
- Branch on the exit code before parsing stdout, and read `retryable` in the JSON error envelope that goes to stderr
- Exit 2 from the parser and exit 64 from a malformed `--filter` print plain text on stderr even with `--json`, so only execution errors carry the envelope
- Read [INTEGRATIONS.md](INTEGRATIONS.md) for agent-specific notes


## Exit Codes
### Sysexits-style codes agents must map before retry

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | General runtime error, remote command failure (the `exec` JSON stays on stdout with `exit_code`, and stderr carries `command_failed` with `remote_exit_code`) or partial multi-host failure |
| `2` | Usage error from the parser, such as two primary auth flags in one call |
| `64` | Usage or invalid arguments, including invalid host names and the `SSH_CLI_SECRETS_KEY` and `SSH_CLI_SECRETS_KEY_FILE` variables |
| `65` | Data error in JSON, TOML, schema or config |
| `66` | VPS or input file not found |
| `69` | Required service unavailable, such as a locked OS keyring or an expired config lock wait |
| `70` | Internal software failure with nothing the caller can fix |
| `73` | Cannot create the config or output |
| `74` | IO error or SSH connection or timeout |
| `77` | Authentication rejected, host key changed, sudo policy or secret decryption failure |
| `130` | SIGINT |
| `141` | Broken pipe on stdout or stderr |
| `143` | SIGTERM |

- Prefer `--json`, which is automatic when stdout is not a TTY (`--output-format` overrides)
- Retry on `74` and `69`, which the JSON error envelope marks with `retryable: true`
- Never retry on auth (`77`), usage (`64`, `2`), data (`65`) or not found (`66`)
- Default tracing is `error`, so exit handling and JSON stdout stay free of INFO noise
- `ssh-cli --help` already ends with an `Examples:` block and an `Exit status:` table, and all 48 leaves show `Examples:` in their own `--help`


## Performance
### Cold start and memory
- The cold start goal is under 100 ms on modern Linux hosts
- Measured on one Linux host, `ssh-cli vps list --json` on an empty registry ran in under 0.01 s with about 8 MB of peak memory
- Reproduce it with `/usr/bin/time -f '%e s %M KB' ssh-cli vps list --json`
- Die after each command so RAM returns to the OS immediately
- Avoid long-lived tunnels, since `--timeout-ms` is mandatory


## Memory Requirements
### Plan capacity for multi-host registries
- Config TOML size grows with host count and path lengths
- Output buffers respect `max_output_chars` per stream
- The `known_hosts` file grows slowly with unique host and port pairs
- Fan-out is capped by `--max-concurrency`, and SCP uses one channel unless `--scp-file-concurrency` is raised
- No embedding model and no Node heap are required


## Troubleshooting FAQ
### Install fails or the build drifts
- Rerun with `--locked` or use release 0.5.6 or newer, which pins russh 0.65.1 (see `scripts/verify_install_resolve.py`)
- Require Rust 1.89 or newer
- Upgrade from 0.3.9 to get the SCP wire fix, which exists since 0.4.0
- Upgrade from 0.4.0 to get the honest export of empty secrets and the post-bind tunnel exit, both since 0.4.1
- Upgrade to 0.5.0 or newer for the renamed EN API, and to 0.5.3 or newer for the SFTP integrity fix

### Authentication fails (exit 77)
- On a key-only host, set `--key` on `vps add` or pass `--key` or `--password-stdin` to `exec`
- With a passphrase key, use `--key-passphrase-stdin`
- When the host key changed, confirm it is legitimate and rerun with `--replace-host-key`
- When `sudo-exec` is disabled, remove `--disable-sudo` and run `vps edit <name> --enable-sudo`

### Usage errors (exit 64 or 2)
- A name is refused: use only ASCII letters, digits, `.`, `_` and `-`, starting with a letter or digit
- A lone positional on `exec` is refused: add the host, a selector or `--use-active`
- Two auth flags in one `vps edit` exit 2: pass one method per call
- A command is rejected as too long: raise `max_command_chars` or shorten the command
- `--dry-run` on an unsupported command exits 64: use it only on the eight previewed operations

### Config and secrets
- Encrypted secrets without a key: run `ssh-cli secrets init` or restore `secrets.key` or pass `--secrets-key-file`
- The call exits 69 on the config lock: repeat it or raise `--lock-wait-ms`
- `vps doctor` shows `config_dir_exposed: true`: tighten the directory yourself with `chmod 700`, because the CLI never changes a directory it does not own by default
- A legacy host name shows in `invalid_host_names`: repair it with `vps rename '<legacy name>' <new>`
- A config exits 65 with two hosts of the same name: rename one table in `config.toml`
- A TOML import exits 65: fix the parse error, and pass `--allow-incomplete` for a redacted skeleton
- Permission denied on the config: ensure `chmod 600` on `config.toml` and `secrets.key`

### Transfers, tunnels and logs
- SCP moves regular files only, so use `sftp upload --recursive` for trees
- A missing SCP remote exits 66 with `file not found: <path>`
- A failed SCP download leaves the destination absent or the previous file intact, and the partial file is `.ssh-cli.partial`
- A tunnel that exits 74 after `tunnel_listening` ran on 0.4.0, and the post-bind deadline exits 0 since 0.4.1
- Stderr noise in JSON pipelines does not exist by default, because the level is `error`
- Pass `-v`, `-vv` or `-vvv` to diagnose without leaking passwords, and `RUST_LOG=debug` has no effect
- On macOS, Gatekeeper blocks the binary: run `xattr -d com.apple.quarantine /path/to/ssh-cli`


## Documentation
### Where to read next
- Follow the first use in [docs/HOW_TO_USE.md](docs/HOW_TO_USE.md)
- Copy recipes from [docs/COOKBOOK.md](docs/COOKBOOK.md)
- Integrate agents via [docs/AGENTS.md](docs/AGENTS.md) and [INTEGRATIONS.md](INTEGRATIONS.md)
- Check platforms in [docs/CROSS_PLATFORM.md](docs/CROSS_PLATFORM.md)
- Migrate from 0.3.3 or newer in [docs/MIGRATION.md](docs/MIGRATION.md) (target line 0.5.6)
- Run tests via [docs/TESTING.md](docs/TESTING.md)
- Consume the JSON contracts under [docs/schemas/README.md](docs/schemas/README.md)
- Teach LLMs with [skills/ssh-cli-en/SKILL.md](skills/ssh-cli-en/SKILL.md)


## Contributing
- Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request
- Follow the bilingual documentation framework for every public doc change
- Follow the [Code of Conduct](CODE_OF_CONDUCT.md)


## Security
- Read [SECURITY.md](SECURITY.md) for private vulnerability reporting
- Prefer stdin secret flags and key files over argv passwords

### Declared limits
- The remote stdout and stderr of `exec` stay raw by design
- A legacy record with a control character in a tag, host or user still exits 65, and only the host NAME is tolerant
- The JSON keeps every character of the escaped set raw, such as U+202E and the zero-width space, because it is for machines
- A legacy host name raises no warning at load, and `vps doctor` lists it
- The global `--filter` and `--count-only` do not reach the `doctor` object, and they show up in `agent_shape.ignored`


## Changelog
- Read the version history in [CHANGELOG.md](CHANGELOG.md)


## Acknowledgments
- `russh` and `russh-sftp` carry the SSH and SFTP protocol
- `rustls`, `tokio-rustls` and `instant-acme` carry SSH-over-TLS, mTLS and ACME
- `tokio` runs the async runtime and `clap` parses the command line
- `serde`, `serde_json` and `toml` read and write every contract and the config
- `chacha20poly1305` and `keyring` protect the secrets at rest


## License
- Dual-licensed under MIT or Apache-2.0
- See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE)
