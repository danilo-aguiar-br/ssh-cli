# Cookbook


> Copy executable recipes that solve real multi-host SSH agent problems.

- Read this document in [Portuguese (pt-BR)](COOKBOOK.pt-BR.md)
- Product line: 0.5.6
- Every recipe follows the same five parts: Problem, Solution, Explanation, Variants and See Also
- Every output comes from a run on the 0.5.6 binary, with host names and paths replaced by placeholders
- `duration_ms`, `latency_ms` and `batch_run_id` vary between runs and `CONFIG_DIR` stands for your configuration directory
- Acronyms: SSH (Secure Shell), CLI (command-line interface), LLM (large language model), XDG (X Desktop Group base directories), TLS (Transport Layer Security), mTLS (mutual TLS), ACME (Automatic Certificate Management Environment), SCP (secure copy), SFTP (SSH File Transfer Protocol) and TOFU (trust on first use)
- Shape output with the built-in `--select`, `--filter` and `--count-only` first, the recipes use `jaq` only to compute or test a value, and any JSON reader works
- Recipes read only canonical fields such as `target_resolved` and `target_source`, the raw output still carries the read aliases `host_resolved` and `host_source` and no recipe parses them
- Text-mode messages shown here come from `--lang en`, the JSON error bodies are English in every locale


## Latency Note
- Expect sub-second local CRUD and a cold SSH connect dominated by network RTT
- Five runs of `ssh-cli --version` took 25 ms in total on the measuring host, so process start is not the cost
- Prefer one-shot commands over tunnels when a single remote action is enough


## Default Values Reference
- Port default: 22
- Timeout default: 60000 ms
- `max_command_chars` default: 1000
- `max_output_chars` default: 100000
- Tracing default: error, with `-v` info, `-vv` debug and `-vvv` trace, crate-scoped, and ambient `RUST_LOG` ignored
- `--lock-wait-ms` default: 5000 ms, ceiling 300000 ms, and a writer that waits longer exits 69
- Empty password in list and show JSON: `null` (key-only hosts)
- Non-empty secrets in list and show mask as three asterisks (`FIXED_MASK`)
- A redacted export clears them instead: `password` is `""` and the optional secrets are `null`
- Telemetry: disabled
- Secrets at rest: encrypted once a primary-key exists, a first password write creates `secrets.key`
- Install: `cargo install ssh-cli --locked`
- Supply chain: russh 0.65.1 and `cargo deny` with `yanked=deny` and `multiple-versions=deny`
- SCP: regular files only, no `-r` and no directories
- Directory trees and remote filesystem operations use `sftp` (`--recursive`, `ls`, `mkdir`)
- SCP download partial suffix: `.ssh-cli.partial`
- SCP success JSON carries `event: "scp-transfer"` and the field `mtime_preserved`
- SCP wire: use 0.4.0 or newer and prefer product line 0.5.6, never 0.3.9, whose SCP was inoperant
- SFTP: prefer 0.5.3 or newer (G1 upload integrity) and verify the destination with `sha256sum`
- The `ssh-cli vps export` body follows the resolved format: JSON on any non-TTY stdout and TOML only with `--output-format text`
- Host wire: schema v3 with English keys, loads still accept the legacy Portuguese aliases
- Host names: ASCII letters, digits, dot, hyphen and underscore, starting with a letter or digit, 1 to 255 bytes
- Tunnel post-bind deadline exits 0 after `tunnel_listening` (TUN-002) and a pre-bind timeout exits 74
- A tunnel ended by SIGINT exits 130 and by SIGTERM exits 143, with `reason` set to `signal` in `tunnel_closed`
- Tunnel `--bind` default: `127.0.0.1`
- Secrets flags are CLI and XDG only: `--allow-plaintext-secrets`, `--secrets-key-file` and `--use-keyring`
- ACME permanent validation such as `invalidContact` exits 64, do not retry it as 74
- Only `ssh-cli vps add --timeout` under 1000 ms warns on stderr, while `ssh-cli vps edit`, `ssh-cli exec` and `ssh-cli health-check` accept it silently
- A password on argv warns on stderr, prefer the `--*-stdin` flags
- `SSH_CLI_SECRETS_KEY` and `SSH_CLI_SECRETS_KEY_FILE` are refused with exit 64, and the other `SSH_CLI_*` variables are ignored
- Success events: `vps-added`, `vps-edited`, `vps-renamed`, `vps-removed`, `vps-connected` and `vps-import`
- A first secret write may set `secrets_key_auto_created: true` on the same `vps-added` document
- The G numbers in the next three lines are the identifiers of the 0.5.3 section in `CHANGELOG.md`
- Gap notes: G1 SFTP upload integrity, G2 and G14 graduated crate-scoped verbosity, G3 SETSTAT with `atime` and `mtime` together, G4 fail-closed `set_metadata`
- G5 and G17 keep `results.len() == input.len()` on cancel, G8 one JSON object per `ssh-cli exec`, G9 `sync_data` before the rename, G15 checksum as proof
- G12 and G19 mask permission bits by direction with `SFTP_PERM_MASK` (`0o7777`) on upload and `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) on download (A3)
- G18 surfaces SFTP download `set_permissions` failures, G6 and G11 concern test isolation only


## Command Index
- The table maps each of the 48 leaf commands to the recipe that runs it
- Look the command up here, then search the recipe title in this file

| Command | Recipe |
|---|---|
| `ssh-cli vps add` | How To Register a Password Host (stdin, no argv leak) |
| `ssh-cli vps list` | How To Shrink an Agent Payload Before It Is Written |
| `ssh-cli vps remove` | How To Rehearse a Destructive Operation and Read the Target Echo |
| `ssh-cli vps rename` | How To Rename a Host Without Retyping the Credential |
| `ssh-cli vps edit` | How To Edit a Host Field by Field and Clear It |
| `ssh-cli vps show` | How To Register a Key-Only Host |
| `ssh-cli vps path` | How To Diagnose XDG Paths and Secrets Mode |
| `ssh-cli vps doctor` | How To Diagnose XDG Paths and Secrets Mode |
| `ssh-cli vps export` | How To Export and Import Inventory Without Secrets |
| `ssh-cli vps import` | How To Export and Import Inventory Without Secrets |
| `ssh-cli connect` | How To Designate the Target Host and Prove Where It Ran |
| `ssh-cli exec` | How To Run a Remote Command With JSON |
| `ssh-cli sudo-exec` | How To Run Safe sudo With Compound Commands |
| `ssh-cli su-exec` | How To Elevate With su When sudo Is Unavailable |
| `ssh-cli scp upload` | How To Transfer a Release Artifact With SCP |
| `ssh-cli scp download` | How To Transfer a Release Artifact With SCP |
| `ssh-cli sftp upload` | How To Upload via SFTP and Verify the Checksum |
| `ssh-cli sftp download` | How To Download a File and a Tree via SFTP |
| `ssh-cli sftp ls` | How To Manage Remote Paths via SFTP |
| `ssh-cli sftp mkdir` | How To Manage Remote Paths via SFTP |
| `ssh-cli sftp rmdir` | How To Manage Remote Paths via SFTP |
| `ssh-cli sftp rm` | How To Manage Remote Paths via SFTP |
| `ssh-cli sftp stat` | How To Manage Remote Paths via SFTP |
| `ssh-cli sftp rename` | How To Manage Remote Paths via SFTP |
| `ssh-cli tunnel` | How To Open a Bounded Tunnel |
| `ssh-cli health-check` | How To Probe Connectivity With Health-Check |
| `ssh-cli secrets status` | How To Initialize Primary-Key Encryption |
| `ssh-cli secrets init` | How To Initialize Primary-Key Encryption |
| `ssh-cli secrets reencrypt` | How To Re-encrypt a Legacy Plaintext Inventory |
| `ssh-cli completions` | How To Generate Shell Completions |
| `ssh-cli commands` | How To Discover Contracts (schema, doctor, commands) |
| `ssh-cli schema` | How To Discover Contracts (schema, doctor, commands) |
| `ssh-cli doctor` | How To Discover Contracts (schema, doctor, commands) |
| `ssh-cli locale show` | How To Set the UI Locale |
| `ssh-cli locale set` | How To Set the UI Locale |
| `ssh-cli locale clear` | How To Set the UI Locale |
| `ssh-cli tls provider` | How To Inspect TLS Provider, Paths and ACME |
| `ssh-cli tls paths` | How To Inspect TLS Provider, Paths and ACME |
| `ssh-cli tls mtls list` | How To Import an mTLS Identity With a Valid Name |
| `ssh-cli tls mtls import` | How To Import an mTLS Identity With a Valid Name |
| `ssh-cli tls mtls show` | How To Import an mTLS Identity With a Valid Name |
| `ssh-cli tls mtls remove` | How To Import an mTLS Identity With a Valid Name |
| `ssh-cli tls acme account create` | How To Inspect TLS Provider, Paths and ACME |
| `ssh-cli tls acme account show` | How To Inspect TLS Provider, Paths and ACME |
| `ssh-cli tls acme issue` | How To Inspect TLS Provider, Paths and ACME |
| `ssh-cli tls acme complete` | How To Inspect TLS Provider, Paths and ACME |
| `ssh-cli tls acme status` | How To Inspect TLS Provider, Paths and ACME |
| `ssh-cli tls acme list` | How To Inspect TLS Provider, Paths and ACME |


## How To Initialize Primary-Key Encryption

### Problem
- Secrets sit in plaintext on disk until a primary-key exists

### Solution

```bash
ssh-cli secrets status --json
ssh-cli secrets init --json
ssh-cli secrets status --json
```

- Expected output of the three calls, in order, on a fresh configuration directory

```json
{"at_rest":"plaintext","encryption_active":false,"key_file":"CONFIG_DIR/secrets.key","key_source":"none","plaintext_opt_out":false}
{"ok":true,"event":"secrets-init","force":false,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","reencrypted_hosts":0}
{"at_rest":"encrypted","encryption_active":true,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","plaintext_opt_out":false}
```

### Explanation
- The key is never printed, the JSON only names the file and the source
- A second `ssh-cli secrets init` exits 64 with `profile already has a primary-key in CONFIG_DIR/secrets.key; use --force to rotate it`
- A first password write creates the key by itself and prints `secrets_key_auto_created: true`

### Variants
- Point at your own key file with `ssh-cli --secrets-key-file /path/to/key secrets status --json`, which reports `key_source` as `file`
- Use the OS keyring with `ssh-cli --use-keyring secrets init --json` or `ssh-cli secrets init --keyring --json`, both reach the same store
- A keyring store that does not answer makes `ssh-cli secrets init --keyring` exit 69 instead of reporting success
- The key resolves in this order: `--secrets-key-file`, the keyring under `--use-keyring`, `secrets.key` in the config directory, then a key stored by an earlier `ssh-cli secrets init --keyring`
- With a key already in the file or in the store, `ssh-cli secrets init --keyring` without `--force` exits 64, and with `--force` it retires the old source so the profile keeps one primary-key
- Keep plaintext for tests only with `--allow-plaintext-secrets`

### See Also
- Re-encrypt a legacy inventory below and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Discover Contracts (schema, doctor, commands)

### Problem
- An agent needs the output shapes and the accepted arguments without reading the source

### Solution

```bash
ssh-cli schema | jaq '.schemas | length'
ssh-cli schema vps-list
ssh-cli commands --json | jaq -r '.tree.subcommands[].name'
ssh-cli doctor --json
```

- The catalog holds 25 schemas and `ssh-cli commands` lists the 16 top-level commands

```json
25
vps connect exec sudo-exec su-exec scp sftp tunnel health-check secrets completions commands schema doctor locale tls
```

### Explanation
- `ssh-cli schema` prints the catalog as the event `schema-catalog` and `ssh-cli schema <name>` prints one schema document
- `ssh-cli commands --json` prints the whole tree with `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` and `hidden` for every argument
- `ssh-cli doctor --json` is the root alias of `ssh-cli vps doctor` and reports paths, secrets mode and the config directory

### Variants
- Read one argument list with `ssh-cli commands --json | jaq '[.. | objects | select(.name? == "rename")] | .[0].args'`
- Print the schema of a batch with `ssh-cli schema exec-batch`
- Read one field of a large document with `ssh-cli commands --json --select tree.about`, which prints `{"agent_shape":{"selected":["tree.about"]},"tree":{"about":"..."}}` instead of the whole tree
- Read the exit codes and copyable examples with `ssh-cli --help`, and one example per leaf with `ssh-cli exec --help`

### See Also
- The schema index in [docs/schemas/README.md](schemas/README.md)


## How To Register a Password Host (stdin, no argv leak)

### Problem
- A password on argv shows up in process lists and shell history

### Solution

```bash
printf '%s' 'demo-password-not-real' | ssh-cli vps add \
  --name prod \
  --host prod.example.com \
  --user deploy \
  --password-stdin --json
```

```json
{"ok":true,"event":"vps-added","key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","name":"prod","secrets_key_auto_created":true}
```

- Without stdin the call fails with exit 64 and `vps_auth: must provide exactly one of --password, --key, or --use-agent`

### Explanation
- `--password-stdin` keeps the secret out of argv, a password on argv works but warns on stderr
- The first password write created `secrets.key`, so `secrets_key_auto_created` is `true` once
- `ssh-cli vps show prod --json` prints the password as three asterisks, never the plaintext

### Variants
- Register through the agent with `ssh-cli vps add --name lab --host 203.0.113.10 --user ubuntu --use-agent`
- Read the password from a file with `ssh-cli vps add ... --password-stdin < pw-file`

### See Also
- Switch the method later with the recipe on authentication below, and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Register a Key-Only Host

### Problem
- A key-based host should never store a password at all

### Solution

```bash
ssh-cli vps add --name edge --host edge.example.com --user ubuntu --key ~/.ssh/id_ed25519 --json
ssh-cli vps show edge --json
```

```json
{"ok":true,"event":"vps-added","name":"edge","secrets_key_auto_created":false}
{"name":"edge","host":"edge.example.com","port":22,"user":"ubuntu","password":null,"key_path":"/home/you/.ssh/id_ed25519","key_passphrase":null,"sudo_password":null,"su_password":null,"timeout_ms":60000,"max_command_chars":1000,"max_output_chars":100000,"disable_sudo":false,"schema_version":3,"added_at":"2026-10-19T22:16:38.513594758+00:00"}
```

### Explanation
- The key file must exist when you register, a missing path exits 66 with `file not found: private key not found`
- A key-only host prints `password` as `null`, which tells an agent there is no password to mask
- No `secrets.key` is created because no secret is stored

### Variants
- In text mode `ssh-cli vps list --output-format text` prints `(not set)` in the PASSWORD column for a host without a password and three asterisks for a host that has one
- The text columns size themselves to the widest cell, so a long user name never shifts the PASSWORD column
- Probe right after registering with `ssh-cli vps add ... --check`
- Group hosts with `--tag prod --tag edge` and reach them later with `--tags`

### See Also
- The full flag list of `ssh-cli vps add` in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Run a Remote Command With JSON

### Problem
- You want one parseable answer from a remote command, not terminal prose

### Solution

```bash
ssh-cli exec prod "echo hi" --json
```

```json
{"target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false,"stdout":"hi\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":112}
```

### Explanation
- A call without `--step` prints exactly one JSON object (G8), never several events
- `exit_code` is the remote exit code, and when the remote command fails the CLI exits 1, keeps this object on stdout and prints a `command_failed` envelope with `remote_exit_code` on stderr
- Remote stdout and stderr stay raw by design, the text mode escape does not touch them

### Variants
- Add steps on the same session with `ssh-cli exec prod "echo one" --step "echo two" --json`, which prints one object per step with `step` and `command`
- Annotate remote history with `--description "bootstrap"`

### See Also
- Fleet fan-out below and [AGENTS.md](AGENTS.md)


## How To Designate the Target Host and Prove Where It Ran

### Problem
- A command that inherits its host from a marker on disk can land on the wrong server (GAP-SSH-EXEC-ARGC-001)

### Solution

```bash
ssh-cli exec prod "systemctl restart nginx" --json
ssh-cli exec --use-active "uptime" --json
```

- Without a marker, `--use-active` exits 66 with `no_active_vps`

```json
{"exit_code":66,"error_code":"no_active_vps","message":"no active vps; run 'ssh-cli connect <NAME>' first","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

- After `ssh-cli connect prod`, the marker is reachable only through `--use-active` and the object says so

```json
{"target_resolved":"prod","target_source":"active_marker","host_resolved":"prod","host_source":"active_marker","active_fallback":true,"stdout":"hi\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":110}
```

```bash
out=$(ssh-cli exec prod "systemctl restart nginx" --json); ec=$?
echo "$out" | jaq -r '"\(.target_resolved) via \(.target_source)"'
echo "$out" | jaq -e '.target_source == "argv"' >/dev/null || echo "ambient target, aborting"
```

- A lone positional exits 64 because the host name would have become the command

```bash
ssh-cli exec prod --step "systemctl restart nginx" --json
```

```json
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: expected exactly two positionals `<VPS> <COMMAND>`, got 1; designate the target explicitly: `<VPS> <COMMAND>`, or a selector with one positional (`--all`/`--hosts`/`--tags` `<COMMAND>`), or the active marker deliberately (`--use-active <COMMAND>`)","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

### Explanation
- `target_resolved` and `target_source` are the canonical fields and `host_resolved` and `host_source` are their 0.5.5 read aliases
- `target_source` is `argv`, `active_marker` or `selector`, so one string comparison proves where a mutation landed
- The split is stable across the CLI: 64 means no target designated and 66 means the designated target does not exist

### Variants
- Ask which host an ambient command would hit with `ssh-cli doctor --json | jaq -r '.local.active_vps // "none"'`
- Pick several hosts explicitly with `--all`, `--hosts a,b` or `--tags prod`

### See Also
- The selector rules in [AGENTS.md](AGENTS.md)


## How To Run Safe sudo With Compound Commands

### Problem
- A compound command with `&&` or quotes must reach the remote shell intact

### Solution

```bash
ssh-cli sudo-exec prod "echo one && echo 'two three'" --description "compound check" --json
```

```json
{"target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false,"stdout":"one\ntwo three\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":156}
```

- A blocked call prints this refusal with exit 77

```bash
ssh-cli --disable-sudo sudo-exec prod id --json
```

```json
{"exit_code":77,"error_code":"sudo_disabled","message":"sudo/su disabled for this host (disable_sudo)","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

### Explanation
- The command is packed with a secure `sh -c`, so metacharacters stay inside the remote shell
- The sudo password comes from the host record, set it with `--sudo-password-stdin`, or with `--sudo-password VALUE` and the stderr warning about argv
- Prefer `ssh-cli sudo-exec` over a raw `sudo` inside `ssh-cli exec`, so the packing and the policy apply

### Variants
- Install a package the same way with `ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --description "bootstrap curl" --json`
- Run on a fleet with `ssh-cli sudo-exec --tags prod 'systemctl restart nginx' --json`
- Block elevation per host with `ssh-cli vps edit prod --disable-sudo` and lift it with `--enable-sudo`
- Remove the stored sudo password with `ssh-cli vps edit prod --clear-sudo-password --json`, `updated` is `["sudo_password"]`

### See Also
- Disable elevation below and [AGENTS.md](AGENTS.md)


## How To Elevate With su When sudo Is Unavailable

### Problem
- The account has no sudo rule but you know the `su` password

### Solution

```bash
printf '%s' 'demo-password-not-real' | ssh-cli vps edit prod --su-password-stdin --json
ssh-cli su-exec prod "whoami" --json
```

```json
{"ok":true,"event":"vps-edited","host_resolved":"prod","host_source":"argv","name":"prod","target_resolved":"prod","target_source":"argv","updated":["su_password"]}
```

- On success `ssh-cli su-exec` prints the same object as `ssh-cli exec`, and before the password is stored the call exits 64 with this envelope

```json
{"exit_code":64,"error_code":"su_password_missing","message":"su_password not configured; use vps edit --su-password or --su-password-stdin","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

### Explanation
- The password is sealed at rest like every other secret
- `updated` names `su_password`, so the envelope proves what changed
- Remove it later with `--clear-su-password`

### Variants
- Set it inline with `--su-password` and accept the stderr warning about argv
- Run on a subset with `ssh-cli su-exec --hosts a,b 'whoami' --json`

### See Also
- Safe sudo above and [AGENTS.md](AGENTS.md)


## How To Reject Oversized Agent Commands Early

### Problem
- An agent may generate a command far longer than you want to run

### Solution

```bash
ssh-cli vps edit prod --max-command-chars 20 --json
ssh-cli exec prod "echo aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" --json
```

```json
{"ok":true,"event":"vps-edited","host_resolved":"prod","host_source":"argv","name":"prod","target_resolved":"prod","target_source":"argv","updated":["max_command_chars"]}
{"exit_code":64,"error_code":"command_too_long","message":"command exceeds max_command_chars (20): 35 characters","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

### Explanation
- The limit is checked before any SSH work, so a rejected command costs no connection
- The count is the length of the command string, here 35 characters
- Exit 64 is permanent, shorten or split the command instead of retrying

### Variants
- Restore the default with `ssh-cli vps edit prod --max-command-chars 1000`
- Set it at registration with `ssh-cli vps add --max-command-chars 200`

### See Also
- Bound output below and [AGENTS.md](AGENTS.md)


## How To Bound Output for LLM Context

### Problem
- A noisy command such as `dmesg` can flood the context window of an agent

### Solution

```bash
ssh-cli vps edit prod --max-output-chars 5 --json
ssh-cli exec prod "echo 123456789" --json
```

```json
{"ok":true,"event":"vps-edited","host_resolved":"prod","host_source":"argv","name":"prod","target_resolved":"prod","target_source":"argv","updated":["max_output_chars"]}
{"target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false,"stdout":"12345","stderr":"","exit_code":0,"truncated_stdout":true,"truncated_stderr":false,"duration_ms":98}
```

### Explanation
- The cut happens at the host record limit and `truncated_stdout` tells you it happened
- A realistic value is 20000, the default is 100000
- Cut the envelope too with the global `--truncate-content 200`

### Variants
- Restore the default with `ssh-cli vps edit prod --max-output-chars 100000`
- Cap a whole fleet answer with `ssh-cli exec --all 'dmesg' --json --max-output-bytes 65536`
- The legacy alias `--max-chars` writes the same field as `--max-command-chars`, so `ssh-cli vps edit prod --max-chars 200` bounds the command length, not the output
- Cut string fields with `--truncate-content`, and silence the human line with `--quiet` or `--no-color` so only the payload is left
- Raise the diagnostic level with `--verbose` (`-v` info, `-vv` debug, `-vvv` trace) when the payload is not enough to explain a bound
- Attach a TLS server name to a host with `ssh-cli vps edit web1 --tls-sni web1.internal` so the handshake uses the certificate name instead of the dial address

### See Also
- Shrink an agent payload below and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Probe Connectivity With Health-Check

### Problem
- You need to know that a host answers before you rely on it

### Solution

```bash
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 15000 --json
ssh-cli health-check down1 --json
```

```json
{"name":"prod","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","status":"ok","latency_ms":39}
{"name":"prod","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","status":"ok","latency_ms":41}
{"exit_code":74,"error_code":"connection_failed","message":"ssh connection failed: TCP dial failed for 127.0.0.1:22999: Connection refused (os error 111)","error_class":"transient","retryable":true,"suggestion":"retry at most twice with exponential full-jitter backoff (exit 74)"}
```

### Explanation
- `--timeout` is in milliseconds, and `health-check` accepts a value under 1000 with no warning, only `ssh-cli vps add --timeout` warns
- A dead host exits 74 with `retryable: true`, retry at most twice with backoff
- `ssh-cli health-check` takes the same auth overrides as `ssh-cli exec`

### Variants
- Override auth with `printf '%s' "$PASS" | ssh-cli health-check prod --json --password-stdin`
- Use a key with `ssh-cli health-check prod --json --key ~/.ssh/id_ed25519` and `--key-passphrase-stdin` when it is encrypted

### See Also
- Fleet probes below and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Run Fleet Work Across All Registered Hosts

### Problem
- Spawning one CLI process per host wastes wall-clock time on SSH round trips

### Solution

```bash
ssh-cli --max-concurrency 8 health-check --all --json
ssh-cli exec --hosts web1,web2 "uptime" --json
ssh-cli exec --tags prod,edge "uptime" --json
ssh-cli scp upload --all ./app.tgz /tmp/app.tgz --json
ssh-cli vps doctor --probe-ssh --json
```

- Expected `ssh-cli health-check --all` output, the batch envelope carries `max_concurrency` and one `results` entry per host

```json
{"event":"health-check-batch","batch_run_id":"00000000-0000-7000-8000-000000000000","max_concurrency":8,"results":[{"name":"web1","status":"ok","latency_ms":38},{"name":"web2","status":"ok","latency_ms":41}]}
```

- Expected `ssh-cli exec --tags` output, only the hosts that carry a listed tag appear

```json
{"event":"exec-batch","batch_run_id":"00000000-0000-7000-8000-000000000000","max_concurrency":64,"results":[{"name":"web2","ok":true,"exit_code":0,"stdout":"hi\n","stderr":"","duration_ms":147}]}
```

- With one host down, the batch still reaches stdout, then the call exits 1 and prints a `partial_failure` envelope of class `partial` on stderr

```json
{"event":"health-check-batch","batch_run_id":"00000000-0000-7000-8000-000000000000","max_concurrency":64,"results":[{"name":"down1","status":"error","latency_ms":0,"error":"ssh connection failed: TCP dial failed for 127.0.0.1:22999: Connection refused (os error 111)"},{"name":"web1","status":"ok","latency_ms":37}]}
{"exit_code":1,"error_code":"partial_failure","message":"1/2 hosts failed during health-check","error_class":"partial","retryable":false}
```

### Explanation
- `--all`, `--hosts` and `--tags` are mutually exclusive and clap rejects any pair with exit 2
- `--tags` works on `ssh-cli exec`, `ssh-cli sudo-exec` and `ssh-cli su-exec` only, tags come from `ssh-cli vps add --tag prod --tag edge`
- An empty registry with `--all` exits 64, an unknown name in `--hosts` exits 64 with `unknown host(s) for --hosts`
- One failed host never aborts its siblings and the call exits 1 with a `partial_failure` envelope on stderr
- `ssh-cli tunnel` stays single-host, several hosts mean several invocations

### Variants
- Download from every host with `ssh-cli scp download --all /var/log/app.log ./app.log --json`, the local path becomes a prefix and the file lands as `./app.log.<vps>`
- Send several files with `ssh-cli scp upload --all --src f1 --src f2 --dest /tmp/dir --json`, `--dest` is always a directory that must already exist on the remote side, and results are named `host:file`
- Stop admitting new hosts after the first failure with the global `--fail-fast`, units already running still finish
- A host that never started stays in `results` with `ok` false and `not attempted (fail-fast stopped admission)` for `ssh-cli exec`, measured with `--max-concurrency 1`
- The `ssh-cli scp` and `ssh-cli sftp` batches word the same state `not attempted (fan-out admission stopped)` in the source, a case this book did not reproduce

### See Also
- Batch schemas in [docs/schemas/README.md](schemas/README.md)


## How To Keep Agent stderr Clean and Diagnose Without a Leak

### Problem
- Info prose on stderr confuses parsers and verbose logs can leak secrets

### Solution

```bash
ssh-cli exec lab "true" --json
ssh-cli -v exec lab "true" --json
ssh-cli -vv exec lab "true" --json
ssh-cli -vvv exec lab "true" --json
```

- The default level is error, so the first call prints only the JSON object on stdout and nothing on stderr

```json
{"target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","active_fallback":false,"stdout":"","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":90}
```

- With `-v` stdout stays the same and stderr gains these lines

```text
2026-10-20T01:14:56.025174Z  INFO main ssh_cli::ssh::client_connect: starting SSH connection host=203.0.113.10 port=22 username=ubuntu timeout_ms=60000 has_key=true use_agent=false
2026-10-20T01:14:56.062337Z  INFO main ssh_cli::ssh::client_connect: SSH authentication succeeded auth_method="publickey"
2026-10-20T01:14:56.062357Z  INFO main ssh_cli::ssh::client_connect: SSH connection authenticated successfully
2026-10-20T01:14:56.153055Z  INFO main ssh_cli::ssh::client::real: SSH session closed
```

### Explanation
- Filters are crate-scoped (`warn,ssh_cli=…`), so a debug run never turns on `russh` password dumps (G2/G14)
- Ambient `RUST_LOG` is ignored, only `-v`, `-vv` and `-vvv` change the level
- A host key rejection also prints one log line on stderr before the JSON envelope

### Variants
- Silence human success output with `-q`
- Refuse to wait on a human with `ssh-cli vps list --json --no-input`, every `--*-stdin` flag then exits 64 with `--no-input forbids reading secrets from stdin; pass the value via flag or drop --no-input`

### See Also
- Verbosity in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Diagnose XDG Paths and Secrets Mode

### Problem
- You need to know which configuration wins and how secrets are stored

### Solution

```bash
ssh-cli doctor --json | jaq -c '.local | {config_path,exists,hosts,permissions,schema_version,secrets_at_rest,secrets_key_source,telemetry,active_vps}'
ssh-cli vps path --json
ssh-cli secrets status --json
```

```json
{"config_path":"CONFIG_DIR/config.toml","exists":true,"hosts":1,"permissions":"600","schema_version":3,"secrets_at_rest":"plaintext","secrets_key_source":"none","telemetry":false,"active_vps":"prod"}
{"ok":true,"event":"vps-path","path":"CONFIG_DIR/config.toml"}
{"at_rest":"plaintext","encryption_active":false,"key_file":"CONFIG_DIR/secrets.key","key_source":"none","plaintext_opt_out":false}
```

### Explanation
- `ssh-cli doctor --json` prints one document with the event `vps-doctor`, the `local` object and `ssh_probe` set to `null` unless you pass `--probe-ssh`
- `permissions` describes `config.toml` only, the config directory has its own fields
- Text mode prints the same facts, with `Config dir mode:`, `Config dir exposed:` and `Invalid host names:` lines

### Variants
- Add the SSH probe with `ssh-cli vps doctor --probe-ssh --hosts web1 --json`
- `known_hosts.lock` and `lang` are created 0600 with no window before the chmod
- Print only the config file path with `ssh-cli vps path`, the event is `vps-path`
- Isolate a lab with `--config-dir /tmp/lab/config.toml`, the parent `/tmp/lab` becomes the config directory for every sibling file
- Read the config directory fields in the two recipes on the config directory below
- Read the runtime facts with `ssh-cli doctor --json --select local.runtime`, the fields are `local.runtime.os`, `arch`, `is_ci`, `is_container`, `is_termux`, `is_wsl` and `sandbox`

### See Also
- Configuration in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Isolate a Lab Configuration Directory

### Problem
- A test run must never touch the registry, the keys or the language of your real configuration

### Solution

```bash
printf '%s' 'demo-password-not-real' | ssh-cli --config-dir /tmp/lab/config.toml vps add --name a --host h.example.com --user deploy --password-stdin --json
ssh-cli --config-dir /tmp/lab/config.toml locale set pt-BR --json
ssh-cli --config-dir /tmp/lab/config.toml vps path --json
ls /tmp/lab
```

```text
{"ok":true,"event":"vps-added","key_file":"/tmp/lab/secrets.key","key_source":"xdg_file","name":"a","secrets_key_auto_created":true}
{"applies":"next_invocation_unless_overridden","lang":"pt-BR","ok":true,"path":"/tmp/lab/lang"}
{"ok":true,"event":"vps-path","path":"/tmp/lab/config.toml"}
config.toml
config.toml.lock
lang
secrets.key
```

### Explanation
- A path that ends in `.toml` makes its parent the configuration directory of every file, so no directory named `config.toml` is created
- The new levels are born 0700 and `known_hosts.lock` and `lang` are born 0600
- The real directory is neither read nor written, because `--config-dir` replaces it
- The `lang` file stays inside the lab, so `ssh-cli locale set` never changes your own language

### Variants
- Pass a plain directory such as `--config-dir /tmp/lab` for the same result, the file is `config.toml` inside it
- `known_hosts`, `active` and the `tls/` tree also live in the lab directory once a command uses them
- Delete the lab with `rm -r /tmp/lab` when you are done

### See Also
- Diagnose XDG paths above and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Re-encrypt a Legacy Plaintext Inventory

### Problem
- An inventory written before the primary-key existed still holds plaintext passwords

### Solution

```bash
ssh-cli secrets init --json
ssh-cli secrets reencrypt --json
```

```json
{"ok":true,"event":"secrets-init","force":false,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","reencrypted_hosts":0}
{"ok":true,"event":"secrets-reencrypt","hosts":1}
```

### Explanation
- `ssh-cli secrets init` without `--force` only creates the key and re-ciphers nothing (`reencrypted_hosts` is 0)
- `ssh-cli secrets reencrypt` rewrites `config.toml` so passwords become `sshcli-enc:` blobs
- Both accept `--dry-run` and print the plan without writing

### Variants
- Preview with `ssh-cli secrets reencrypt --dry-run --json`, the plan names `hosts` and `operation`
- Re-encrypt under a key file you chose with `ssh-cli --secrets-key-file /path/to/key secrets reencrypt`

### See Also
- The initialization recipe above, the key rotation recipe below and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Rotate the Primary-Key Without Losing Secrets

### Problem
- Replacing the primary-key without re-ciphering the stored passwords loses them at rest

### Solution

```bash
ssh-cli secrets init --force --dry-run --json
ssh-cli secrets init --force --json
ssh-cli secrets status --json
```

```json
{"ok":true,"event":"dry-run","config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"force":true,"hosts_to_reencrypt":1,"keyring":false,"operation":"secrets-init"}
{"ok":true,"event":"secrets-init","force":true,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","reencrypted_hosts":1}
{"at_rest":"encrypted","encryption_active":true,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","plaintext_opt_out":false}
```

### Explanation
- Read `hosts_to_reencrypt` in the plan first, because it is the number of hosts whose secrets the rotation must re-cipher
- The real run prints `reencrypted_hosts` with the count it re-ciphered, compare it with the plan
- Without `--force` an existing key makes `ssh-cli secrets init` exit 64 and change nothing
- The plan writes nothing, the key file and `config.toml` stay as they were

### Variants
- Rotate into the OS keyring with `ssh-cli secrets init --force --keyring --json`, the plan prints `keyring: true`
- Rotate under a key file you chose with `ssh-cli --secrets-key-file /path/to/key secrets init --force --json`

### See Also
- The re-encryption recipe above and [AGENTS.md](AGENTS.md)


## How To Export and Import Inventory Without Secrets

### Problem
- You want to copy a registry to another machine without carrying credentials

### Solution

```bash
ssh-cli vps export -o /tmp/hosts.redacted.json --json
ssh-cli --output-format text vps export -o /tmp/hosts.redacted.toml
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.redacted.toml --allow-incomplete --dry-run --json
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.redacted.toml --allow-incomplete --json
```

```json
{"ok":true,"event":"vps-export","format":"json","include_secrets":false,"path":"/tmp/hosts.redacted.json"}
{"ok":true,"event":"dry-run","config_path":"/tmp/ssh-cli-copy/config.toml","dry_run":true,"executed":false,"hosts":[{"incomplete":false,"name":"demo","replaces_existing":false}],"imported":1,"operation":"vps-import","source":"/tmp/hosts.redacted.toml"}
{"ok":true,"event":"vps-import","imported":1,"replaced":[]}
```

### Explanation
- The body follows the resolved format: JSON on any non-TTY stdout, even for a `.toml` file name, and TOML only with `--output-format text`
- A redacted export clears every secret, `password` becomes `""` and the optional secrets become `null`, and a fake empty `sshcli-enc:` is never written
- With `-o` the call prints a small `vps-export` envelope naming the file, without `-o` it prints the whole envelope with the hosts on stdout
- `replaced` lists the sorted names the import overwrote and is empty on a first import
- A password host looks like a redacted export, so the import exits 64 unless you pass `--allow-incomplete` and then complete it with `ssh-cli vps edit`

### Variants
- Import accepts TOML with English keys or legacy Portuguese aliases, or a JSON `vps-export` envelope, and the user key is `username` in TOML (legacy `usuario`) and `user` in JSON
- A key-only or agent host round-trips complete, `use_agent` and `agent_socket` travel in the export
- A missing `--file`, with or without `--dry-run`, exits 66 with `error_code` `file_not_found`, `error_class` `permanent` and `retryable` false, so do not retry it

### See Also
- Invalid names on import below and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Preview an Import That Would Overwrite Hosts

### Problem
- An import that reuses a host name overwrites that record, and the count of imported hosts does not say so

### Solution

```bash
ssh-cli vps export -o hosts.json --json
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file hosts.json --json
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file hosts.json --dry-run --json
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file hosts.json --json
```

- Expected output of the four calls, in order, with the registry of `/tmp/ssh-cli-copy` empty at the start

```json
{"ok":true,"event":"vps-export","format":"json","include_secrets":false,"path":"hosts.json"}
{"ok":true,"event":"vps-import","imported":1,"replaced":[]}
{"ok":true,"event":"dry-run","config_path":"/tmp/ssh-cli-copy/config.toml","dry_run":true,"executed":false,"hosts":[{"incomplete":false,"name":"edge","replaces_existing":true}],"imported":1,"operation":"vps-import","source":"hosts.json"}
{"ok":true,"event":"vps-import","imported":1,"replaced":["edge"]}
```

### Explanation
- `replaces_existing` is true when the name is already in the target registry, that field and not the `imported` count decides whether the import is safe
- `replaced` in the real run lists the sorted names that were overwritten, and it is empty on a first import
- `incomplete` is true for a host that looks like a redacted export, the import exits 64 for it unless you pass `--allow-incomplete`
- The plan runs the same validation as the real run, so an invalid name fails the same way

### Variants
- Import into a throwaway `--config-dir` first when you are unsure, the registry you care about never changes
- Keep a copy of the target registry before an overwrite with `ssh-cli vps export -o backup.json --json` on the target machine

### See Also
- The export and import recipe above and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Export With Secrets (guarded)

### Problem
- A backup that keeps credentials must never reach a pipe by accident

### Solution

```bash
ssh-cli vps export --include-secrets -o /tmp/hosts.secrets.json --json
ssh-cli vps export --include-secrets | cat
```

```json
{"ok":true,"event":"vps-export","format":"json","include_secrets":true,"path":"/tmp/hosts.secrets.json"}
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: refusing --include-secrets to a pipe/non-TTY stdout; use `--output <file>` (mode 0o600) or pass `--i-understand-secrets-on-stdout`","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

### Explanation
- The output file is created with mode 0600
- The pipe is refused with exit 64 unless you acknowledge it
- Missing parent directories of `-o new/deep/e.json` are created with mode 0700 and existing ones are never changed

### Variants
- Acknowledge stdout only when you truly need it with `--i-understand-secrets-on-stdout`
- Prefer a redacted export plus a fresh `--password-stdin` on the target machine

### See Also
- The redacted recipe above and [AGENTS.md](AGENTS.md)


## How To Open a Bounded Tunnel

### Problem
- A forward without a deadline is a daemon and this CLI does not ship one

### Solution

```bash
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 3000 --json
```

```json
{"ok":true,"event":"tunnel_listening","vps":"prod","local_port":18080,"remote_host":"127.0.0.1","remote_port":8080,"timeout_ms":3000,"bind":"127.0.0.1","mode":"local"}
{"ok":true,"event":"tunnel_closed","vps":"prod","reason":"deadline","bind":"127.0.0.1","local_port":18080,"forwards_served":0,"capacity_waits":0,"duration_ms":3001,"mode":"local"}
```

### Explanation
- Wait for the `tunnel_listening` object before you use the local port (`docs/schemas/tunnel-listening.schema.json`)
- `tunnel_closed` names the `reason`, `forwards_served` and `capacity_waits`, and a `deadline` ending exits 0
- A SIGINT exits 130 and a SIGTERM exits 143, both with `reason` set to `signal`
- A pre-bind timeout still exits 74
- The process stays alive until the deadline or a signal

### Variants
- Bind another address with `--bind 0.0.0.0 --i-accept-network-exposure`, a routable bind is refused without the acknowledgement (G-TUN-R13) and the call exits 64 with `--bind 0.0.0.0 exposes the forwarded service to the network; pass --i-accept-network-exposure to proceed`
- Authenticate with `--password-stdin`, `--key`, `--key-passphrase` or `--key-passphrase-stdin` as in `ssh-cli exec`

### See Also
- Bind safety in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Reach Many Destinations Through One Handshake (--socks5)

### Problem
- N separate tunnels pay N SSH handshakes to reach N hosts behind one bastion

### Solution

```bash
ssh-cli tunnel prod 1080 --socks5 --timeout-ms 300000 --json
curl --socks5-hostname 127.0.0.1:1080 http://10.0.0.5:8080/health
```

```json
{"ok":true,"event":"tunnel_listening","vps":"prod","local_port":1080,"remote_host":"*","remote_port":0,"timeout_ms":300000,"bind":"127.0.0.1","mode":"socks5"}
```

### Explanation
- One SSH session serves every destination, chosen per connection (G-TUN-R02)
- `REMOTE_HOST` and `REMOTE_PORT` are omitted because SOCKS5 CONNECT carries the target
- The proxy speaks RFC 1928 with no auth, so keep it on loopback

### Variants
- Point any SOCKS5-aware client at `127.0.0.1:1080`
- Mix it with `--max-concurrency` to cap simultaneous forwards

### See Also
- Tunnel modes in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Forward to a Remote Unix Socket (--remote-socket)

### Problem
- Docker, PostgreSQL and systemd listen on a Unix socket, not on TCP

### Solution

```bash
ssh-cli tunnel prod 2375 --remote-socket /var/run/docker.sock --timeout-ms 60000 --json
docker -H tcp://127.0.0.1:2375 ps
```

```json
{"ok":true,"event":"tunnel_listening","vps":"prod","local_port":2375,"remote_host":"/var/run/docker.sock","remote_port":0,"timeout_ms":60000,"bind":"127.0.0.1","mode":"streamlocal"}
```

### Explanation
- The call opens a `direct-streamlocal@openssh.com` channel (G-TUN-R03)
- The socket path must be absolute, a relative path exits 64 before any SSH work
- Local existence is never checked because the socket lives on the server
- Treat the local port as privilege delegation, it inherits the socket's authority

### Variants
- PostgreSQL uses the same shape with `ssh-cli tunnel db 15432 --remote-socket /var/run/postgresql/.s.PGSQL.5432 --timeout-ms 60000 --json`
- A Windows client works because locally it only speaks TCP, the server must support the extension

### See Also
- Tunnel modes in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Let the Server Reach Back (--reverse)

### Problem
- A callback webhook or a remote debugger must reach a port on your machine

### Solution

```bash
ssh-cli tunnel prod 8080 127.0.0.1 9000 --reverse --timeout-ms 120000 --json
```

```json
{"ok":true,"event":"tunnel_listening","vps":"prod","local_port":9000,"remote_host":"127.0.0.1","remote_port":8080,"timeout_ms":120000,"bind":"127.0.0.1","mode":"reverse"}
```

- Ask the server to allocate the port with `0` as the last positional, the allocated port comes back in `local_port` and varies per run

```bash
ssh-cli tunnel prod 8080 127.0.0.1 0 --reverse --timeout-ms 120000 --json
```

```json
{"ok":true,"event":"tunnel_listening","vps":"prod","local_port":36423,"remote_host":"127.0.0.1","remote_port":8080,"timeout_ms":120000,"bind":"127.0.0.1","mode":"reverse"}
```

- Exposing the server side needs the acknowledgement, because that is the exposed end

```bash
ssh-cli tunnel prod 8080 0.0.0.0 9000 --reverse --i-accept-network-exposure --timeout-ms 120000 --json
```

### Explanation
- The server listens and delivers each connection to your local port (G-TUN-R01)
- Under `--reverse` the event is read from the server's point of view: `bind` and `local_port` are where the server listens, `remote_host` and `remote_port` are your local delivery target
- Server-side `AllowTcpForwarding` and `GatewayPorts` still decide whether the bind is permitted
- `--bind` is accepted and then ignored under `--reverse`, set the server address through `<remote_host>`

### Variants
- A local forward cannot take `REMOTE_PORT` 0, there would be nothing to connect to, while `LOCAL_PORT` 0 asks the OS for a free port
- A server-side bind outside loopback without the acknowledgement exits 64 with a message that asks you to pass `--i-accept-network-exposure`

### See Also
- Tunnel modes in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Shrink an Agent Payload Before It Is Written

### Problem
- Piping a large envelope through an external tool pays the token cost before it shrinks

### Solution

```bash
ssh-cli health-check --all --json --select name,status --filter status!=ok
ssh-cli vps list --json --select name,host --sort name --limit 2
ssh-cli vps list --json --count-only
ssh-cli vps list --json --filter nameprod
```

```text
{"agent_shape":{"dropped":2,"input_count":3,"output_count":1},"batch_run_id":"00000000-0000-7000-8000-000000000000","event":"health-check-batch","max_concurrency":64,"results":[{"name":"down1","status":"error"}]}
[{"host":"127.0.0.1","name":"demo"},{"host":"127.0.0.1","name":"demo2"}]
{"count":3}
Invalid argument: invalid --filter `nameprod`: expected key=value, key!=value or key~substring
```

- The first call exits 1 because one host failed, the next two exit 0 and the last one exits 64

### Explanation
- The eight shaping flags run before serialization, so the big envelope is never built
- The batch result field is `status` with `ok` or `error`, there is no `ok` field, so `--filter ok=false` would drop every record
- A malformed predicate is rejected at parse time with exit 64, so a typo never looks like no matches
- Shaping reaches record collections (`ssh-cli vps list`, `results`, `entries` of `ssh-cli sftp ls`) and also a single object such as `ssh-cli vps show`, `ssh-cli doctor` or a single-host `ssh-cli exec`
- On a single object only `--select` and `--truncate-content` reduce, and `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` and `--max-output-bytes` are named under `agent_shape.ignored`
- `name~~prod` parses as key `name` with substring `~prod`, the refusal covers missing operators and empty keys

### Variants
- Dedupe with `--dedupe-by host` and shorten strings with `--truncate-content 200`
- Pick fields of a fleet answer with `ssh-cli exec --all 'echo a' --json --select name,stdout`, since `--select results` would return empty objects
- `--fields` is an alias of `--select`, so `ssh-cli vps show prod --json --fields name,host --count-only` prints `{"agent_shape":{"ignored":["--count-only"],"selected":["name","host"]},"host":"prod.example.com","name":"prod"}`
- The bare array of `ssh-cli vps list` has no room for `agent_shape`, so `ssh-cli vps list --json --select name --limit 1` writes `{"dropped":1,"event":"agent-shape","input_count":2,"output_count":1}` on stderr, and `--quiet` suppresses that line
- A clap usage error (exit 2) and a malformed `--filter` (exit 64) print plain text on stderr even with `--json`, only execution errors carry the JSON envelope, and the plain text follows the UI locale (`Argumento inválido:` under pt-BR)
- Cap the envelope with `--max-output-bytes 65536`, it drops trailing records and never slices the JSON text
- An empty key such as `--filter =prod` exits 64 with `invalid --filter: empty key`, while `--filter name~~prod` is valid, exits 0 and matches nothing here
- A reader that closes the pipe before a large output ends, such as `ssh-cli completions bash | head -c 5`, makes the CLI exit 141 with the output incomplete, read it from `${PIPESTATUS[0]}` and never retry
- A small JSON such as `ssh-cli vps list --json | head -c 5` finishes writing before the reader closes and exits 0, so it proves nothing about 141
- Exit 141 covers a closed stdout and a closed stderr alike

### See Also
- Payload shaping in [AGENTS.md](AGENTS.md)


## How To Switch a Host From Password to Key and Back

### Problem
- Moving a host between authentication methods used to exit 64 and force a detour through the agent

### Solution

```bash
printf '%s' 'demo-password-not-real' | ssh-cli vps add --name web1 --host 203.0.113.10 --user deploy --password-stdin --json
ssh-cli vps edit web1 --key ~/.ssh/id_ed25519 --json
printf '%s' 'demo-password-not-real' | ssh-cli vps edit web1 --password-stdin --json
```

```json
{"ok":true,"event":"vps-added","key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","name":"web1","secrets_key_auto_created":true}
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":["key_path","password"]}
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":["password","key_path"]}
```

- Conflicting flags are caught by clap before any work, exit 2, and the first line of the plain-text error is shown

```bash
ssh-cli vps edit web1 --key ~/.ssh/id_ed25519 --password-stdin --json
```

```text
error: the argument '--key <PATH>' cannot be used with '--password-stdin'
```

### Explanation
- `--key` clears `password`, `use_agent` and `agent_socket`, `--password*` clears `key_path`, `key_passphrase`, `use_agent` and `agent_socket`
- `--use-agent` clears `password`, `key_path` and `key_passphrase`, and `--agent-socket` alone implies `--use-agent`
- `updated` lists the new method first and then only the fields that held a value
- A different `--key` drops the old passphrase, the same key path changes nothing
- A passphrase without any key exits 64 and leaves the record untouched

### Variants
- Move to the agent with `ssh-cli vps edit web1 --use-agent --json`, `updated` is `["use_agent","password"]` on a password host
- Add a passphrase to a key with `--key ~/.ssh/id_ed25519 --key-passphrase-stdin`

### See Also
- Authentication switch in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Rename a Host Without Retyping the Credential

### Problem
- The export, edit, import detour redacted secrets and dropped agent authentication

### Solution

```bash
ssh-cli connect old-name
ssh-cli vps rename old-name new-name --json
ssh-cli vps edit new-name --name newer-name --json
```

```json
{"ok":true,"event":"vps-connected","name":"old-name"}
{"ok":true,"event":"vps-renamed","active_rewritten":true,"host_resolved":"old-name","host_source":"argv","name":"new-name","previous_name":"old-name","target_resolved":"old-name","target_source":"argv","updated":[]}
{"ok":true,"event":"vps-renamed","active_rewritten":true,"host_resolved":"new-name","host_source":"argv","name":"newer-name","previous_name":"new-name","target_resolved":"new-name","target_source":"argv","updated":[]}
```

### Explanation
- The credential is never asked for, the record seals secrets without associated data and the new name changes nothing about them
- `ssh-cli vps edit --name` and `ssh-cli vps rename` emit the same `vps-renamed` event with `previous_name`
- `active_rewritten` is `true` only when the `active` marker pointed at the old name, and it follows the new one
- Repeating a rename exits 66 because the old name no longer exists, so check `ssh-cli vps list` instead of repeating

### Variants
- A taken destination exits 64 with `vps_duplicate` and an identical name exits 64 with `new name ... is identical to the current name`
- An invalid new name exits 64 as well
- `ssh-cli vps rename ghost ghost` exits 66 because the host must exist before the names are compared

### See Also
- Rename in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Edit a Host Field by Field and Clear It

### Problem
- Every optional field could be overwritten but none could be removed

### Solution

```bash
ssh-cli vps edit web1 --user deploy --port 2222 --tag prod --tag eu --json
ssh-cli vps edit web1 --clear-tags --tag staging --json
ssh-cli vps edit web1 --port 2222 --json
ssh-cli vps edit web1 --json
```

```json
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":["port","tags"]}
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":["tags"]}
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":[]}
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: nothing to edit for 'web1': pass at least one field, --name, --tag or a clear flag","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

- The host started with `user` already `deploy`, so `updated` lists only `port` and `tags` in the first call

### Explanation
- The eight clear flags are `--clear-tags`, `--no-key`, `--no-agent-socket`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert` and `--no-tls-client-key`
- `--clear-tags` clears first and `--tag` then appends, so the result is `[staging]`
- `--no-key` also clears the passphrase, and clearing the only remaining auth method exits 64 with `error_code` `domain_validation` and a message prefixed `vps_auth:`, and writes nothing
- An edit that changes nothing is a no-op: exit 0, `updated: []` and `config.toml` is not rewritten
- Each clear flag conflicts with the value flag of its own field, `--no-key --key` exits 2

### Variants
- Drop both mTLS halves with `--no-tls-client-cert --no-tls-client-key`
- Toggle sudo with `--disable-sudo` and `--enable-sudo`, `updated` is `["disable_sudo"]`
- `--no-agent-socket` on an agent host with an explicit socket exits 64 where the platform has no default endpoint, pass `--agent-socket PATH` or switch method

### See Also
- The flag table in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Repair a Legacy Host Name

### Problem
- An older `config.toml` holds a name with a space, an accent or a control character

### Solution

```bash
mkdir -p legacy && chmod 700 legacy
cat > legacy/config.toml <<'EOF'
schema_version = 3

[hosts."legacy host"]
name = "legacy host"
host = "legacy.example.com"
port = 22
username = "deploy"
use_agent = true
EOF
ssh-cli --config-dir legacy doctor --json | jaq -c '.local.invalid_host_names'
ssh-cli --config-dir legacy vps rename 'legacy host' legacy-host --json
ssh-cli --config-dir legacy doctor --json | jaq -c '.local.invalid_host_names'
```

```json
["legacy host"]
{"ok":true,"event":"vps-renamed","active_rewritten":false,"host_resolved":"legacy host","host_source":"argv","name":"legacy-host","previous_name":"legacy host","target_resolved":"legacy host","target_source":"argv","updated":[]}
[]
```

### Explanation
- The CLI never blocks on its own file, the legacy name loads, lists, shows and resolves
- `ssh-cli doctor` lists the names outside the strict charset, escaped, in `local.invalid_host_names`
- `ssh-cli vps rename` accepts the legacy source because only the new name is validated
- A name with a control character is refused as a target by `ssh-cli exec`, `ssh-cli health-check`, `--hosts` and `ssh-cli connect` with exit 64
- A legacy host exports fine but restores only after `ssh-cli vps rename` gives it a strict name

### Variants
- A name with an ESC byte prints as `a\u{1b}[2Jb` in text mode and in `invalid_host_names`
- Two tables that normalize to one name exit 65 with `config has two hosts named ...`

### See Also
- Repair a legacy host name in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Read Hostile Names Safely in Text Mode

### Problem
- A stored host name with an ESC byte can redraw your terminal when a tool prints it raw

### Solution

```bash
mkdir -p hostile && chmod 700 hostile
cat > hostile/config.toml <<'EOF'
schema_version = 3

[hosts."a\u001b[2Jb"]
name = "a\u001b[2Jb"
host = "h.example.com"
port = 22
username = "deploy"
use_agent = true
EOF
ssh-cli --lang en --config-dir hostile --output-format text vps show "$(printf 'a\033[2Jb')" | sed -n 1p
ssh-cli --lang en --config-dir hostile --output-format text vps show "$(printf 'a\033[2Jz')"
ssh-cli --config-dir hostile exec "$(printf 'a\033[2Jb')" id --json
ssh-cli --config-dir hostile vps list --json | jaq -c '.[].name'
```

- The first line below is stdout, and the second is the stderr of the call that exits 66

```text
Name:           a\u{1b}[2Jb
VPS 'a\u{1b}[2Jz' not found.
```

- The third call is refused with exit 64 and the fourth prints the JSON escape

```json
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: vps_name: name must not contain control or invisible formatting characters","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
"a\u001b[2Jb"
```

### Explanation
- Text mode writes control, formatting and bidi characters as `\u{1b}` in success, error and warning messages, in `list` and `show`, and in batch output
- The JSON keeps the original value, because it is for machines and the `serde_json` escape already hides the ESC byte
- A lookup that would become a target refuses control and bidi characters with exit 64, `ssh-cli exec`, `ssh-cli health-check`, `--hosts` and `ssh-cli connect` included, and `ssh-cli connect` writes no marker
- The error for a name that does not exist echoes the typed name escaped

### Variants
- Remote names get the same care, `ssh-cli sftp ls` and `ssh-cli sftp stat` also escape `\n` and `\t` in text mode
- The remote stdout and stderr of `ssh-cli exec` stay raw by design, filter them before you print them to a terminal

### See Also
- Repair a legacy host name above and [AGENTS.md](AGENTS.md)


## How To Check That the Config Directory Is Not Exposed

### Problem
- A config directory with mode 0755 shows host names and file sizes to other users

### Solution

```bash
ssh-cli doctor --json --select local.config_dir_exposed
mkdir shared && chmod 755 shared
ssh-cli --config-dir shared doctor --json --select local.config_dir_permissions,local.config_dir_exposed
chmod 700 shared
ssh-cli --config-dir shared doctor --json --select local.config_dir_permissions,local.config_dir_exposed
```

```json
{"agent_shape":{"selected":["local.config_dir_exposed"]},"local":{"config_dir_exposed":false}}
{"agent_shape":{"selected":["local.config_dir_permissions","local.config_dir_exposed"]},"local":{"config_dir_exposed":true,"config_dir_permissions":"0755"}}
{"agent_shape":{"selected":["local.config_dir_permissions","local.config_dir_exposed"]},"local":{"config_dir_exposed":false,"config_dir_permissions":"0700"}}
```

### Explanation
- A directory the CLI creates is born 0700 at every new level
- An existing default XDG directory is tightened to 0700 when it is not a symlink and belongs to you
- Any other existing directory is never chmodded, `ssh-cli doctor` reports the exposure instead and a log warning shows with `-v`
- `--select` reaches the `ssh-cli doctor` object directly and names the kept paths in `agent_shape.selected`, while `--filter` and `--count-only` cannot apply to one record and are named under `agent_shape.ignored`
- The fix is yours: run `chmod 700` on the directory and the same call prints `config_dir_exposed` as `false`

### Variants
- `config_dir_permissions` is absent off Unix and `config_dir_exposed` reads `false` there
- On Windows the directory relies on the profile ACL because there is no chmod

### See Also
- Configuration in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To See Every Invalid Name in One Import

### Problem
- A file with several bad host names should fail once, with the whole list

### Solution

```bash
cat > inv.toml <<'EOF'
schema_version = 3

[hosts."bad name"]
name = "bad name"
host = "h.example.com"
port = 22
username = "u"
use_agent = true

[hosts."semi;colon"]
name = "semi;colon"
host = "h.example.com"
port = 22
username = "u"
use_agent = true
EOF
ssh-cli vps import --file inv.toml --json
ssh-cli vps import --file inv.toml --dry-run --json
```

```json
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: invalid VPS names in import (2): 'bad name', 'semi;colon'; a name is 1..=255 ASCII letters, digits, '.', '_' or '-' and starts with a letter or digit; nothing was imported","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: invalid VPS names in import (2): 'bad name', 'semi;colon'; a name is 1..=255 ASCII letters, digits, '.', '_' or '-' and starts with a letter or digit; nothing was imported","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

### Explanation
- The import validates every item first and writes nothing when one name fails
- `--dry-run` gives the same error, so the rehearsal is faithful
- Fix the names in the file and run again, retrying unchanged cannot help

### Variants
- Use ASCII letters, digits, dot, hyphen and underscore, starting with a letter or digit, up to 255 bytes
- `ssh-cli vps add --name 'bad name'` and the target of `ssh-cli vps rename` follow the same strict rule and exit 64

### See Also
- Export and import in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Import an mTLS Identity With a Valid Name

### Problem
- An identity name becomes a directory name, so a hostile name must not create one

### Solution

```bash
openssl req -x509 -newkey ed25519 -nodes -keyout client-key.pem -out client.pem -subj /CN=demo -days 2
ssh-cli tls mtls import --name edge --cert ./client.pem --key ./client-key.pem --json
ssh-cli tls mtls import --name 'serviço-1' --cert ./client.pem --key ./client-key.pem --json
ssh-cli tls mtls list --json
ssh-cli tls mtls show edge --json
```

```json
{"ok":true,"event":"tls-mtls-import","cert_path":"CONFIG_DIR/tls/mtls/edge/cert.pem","key_path":"CONFIG_DIR/tls/mtls/edge/key.pem","name":"edge"}
{"ok":true,"event":"tls-mtls-import","cert_path":"CONFIG_DIR/tls/mtls/serviço-1/cert.pem","key_path":"CONFIG_DIR/tls/mtls/serviço-1/key.pem","name":"serviço-1"}
{"ok":true,"event":"tls-mtls-list","identities":["edge","serviço-1"]}
{"ok":true,"event":"tls-mtls-show","cert_path":"CONFIG_DIR/tls/mtls/edge/cert.pem","key_path":"CONFIG_DIR/tls/mtls/edge/key.pem","name":"edge"}
```

- A name with a bidi override such as U+202E exits 64 and creates no directory

```json
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: invalid mTLS identity name: invalid argument: file name must not contain control or invisible formatting characters","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

### Explanation
- `ssh-cli tls mtls import` refuses control, formatting and bidi characters, ordinary Unicode stays accepted
- `ssh-cli tls mtls list` shows the stored identity names
- Keys and certificates land under `tls/mtls/<name>/` in the config directory with private modes
- `show` never prints private key material

### Variants
- Remove an identity with `ssh-cli tls mtls remove edge --json`, it prints `{"ok":true,"event":"tls-mtls-remove","name":"edge"}`
- Attach it to a host with `ssh-cli vps edit web1 --tls --tls-client-cert CERT --tls-client-key KEY`
- Turn SSH-over-TLS off again with `ssh-cli vps edit web1 --no-tls --json`, `updated` is `["tls"]` and the stored SNI and mTLS paths stay until you pass their clear flags

### See Also
- TLS in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Rehearse a Destructive Operation and Read the Target Echo

### Problem
- You want to see which host a destructive call would touch before it touches anything

### Solution

```bash
ssh-cli vps remove web1 --dry-run --json
ssh-cli vps rename web1 web-1 --dry-run --json
ssh-cli vps edit web1 --name web-1 --dry-run --json
```

```json
{"ok":true,"event":"dry-run","config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"host_resolved":"web1","host_source":"argv","name":"web1","operation":"vps-remove","target_resolved":"web1","target_source":"argv"}
{"ok":true,"event":"dry-run","active_rewritten":false,"config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"host_resolved":"web1","host_source":"argv","name":"web-1","operation":"vps-rename","previous_name":"web1","target_resolved":"web1","target_source":"argv","updated":[]}
{"ok":true,"event":"dry-run","active_rewritten":false,"config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"host_resolved":"web1","host_source":"argv","name":"web-1","operation":"vps-rename","previous_name":"web1","target_resolved":"web1","target_source":"argv","updated":[]}
```

- Against a registry that does not exist, the three verbs exit 66 and create no directory and no lock file

```json
{"exit_code":66,"error_code":"vps_not_found","message":"vps 'ghost' not found in registry","error_class":"permanent","retryable":false,"suggestion":"the host is not in the registry; if an earlier `vps rename` or `vps remove` may have succeeded, run `vps list` before repeating"}
```

### Explanation
- `--dry-run` is accepted only by `ssh-cli vps remove`, `ssh-cli vps import`, `ssh-cli vps edit`, `ssh-cli vps rename`, `ssh-cli sftp rm`, `ssh-cli sftp rmdir`, `ssh-cli secrets init` and `ssh-cli secrets reencrypt`
- Anywhere else it exits 64, so a flag that does nothing never passes for a rehearsal
- The plan runs every validation of the real run and carries `target_resolved`, `target_source` (`argv`), `host_resolved` and `host_source`
- `active_rewritten` tells whether the `active` marker would move, without moving it
- A failed lookup carries no target, because no host was ever chosen

### Variants
- The remote operations print `vps` and `remote` instead, for example `ssh-cli sftp rm prod /srv/app/stale.log --dry-run --json`
- A refused call looks like `ssh-cli exec prod 'rm -rf /tmp/x' --dry-run --json` and exits 64

### See Also
- The dry-run schema in [docs/schemas/README.md](schemas/README.md)


## How To Transfer a Release Artifact With SCP

### Problem
- You need a regular file on the host, with proof of what moved

### Solution

```bash
ssh-cli scp upload prod ./payload.bin /tmp/payload.bin --timeout 120000 --json
ssh-cli scp download prod /tmp/payload.bin ./payload.local --json
ssh-cli scp download prod /tmp/missing.bin ./missing --json
```

```json
{"ok":true,"event":"scp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"upload","vps":"prod","local":"./payload.bin","remote":"/tmp/payload.bin","bytes":8,"duration_ms":111,"mtime_preserved":true,"durable":true}
{"ok":true,"event":"scp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"download","vps":"prod","local":"./payload.local","remote":"/tmp/payload.bin","bytes":8,"duration_ms":209,"mtime_preserved":true,"durable":true}
{"exit_code":66,"error_code":"file_not_found","message":"file not found: /tmp/missing.bin","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

### Explanation
- SCP moves regular files only, use `sftp --recursive` for trees
- A download writes `<path>.ssh-cli.partial` and renames it atomically, so a failure leaves the final path untouched
- Preserving mtime and mode is best effort and `mtime_preserved` and `durable` report the real outcome
- Failures with `--json` print the error envelope on stderr

### Variants
- Send several files to one existing remote directory with `ssh-cli scp upload prod f1 f2 /tmp/dir --json`, one session and serial transfers, and the event is `scp-batch` with one `results` entry per file
- Check the result with `ssh-cli exec prod "ls -l /tmp/dir" --json`
- Open parallel channels on the same session with the global `--scp-file-concurrency 2` placed before the subcommand, the default is 1

### See Also
- Daily operator loop in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Upload via SFTP and Verify the Checksum

### Problem
- A client-reported byte count does not prove the destination holds the right bytes

### Solution

```bash
ssh-cli sftp upload prod ./payload.bin /tmp/payload.bin --json
ssh-cli exec prod "sha256sum /tmp/payload.bin" --json | jaq -r .stdout
sha256sum ./payload.bin
```

```json
{"ok":true,"event":"sftp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"upload","vps":"prod","local":"./payload.bin","remote":"/tmp/payload.bin","bytes":8,"duration_ms":1}
d4e4877bac978b7952f0d544fc52ebff5411d351d129f1f056fa43f11da9af2b  /tmp/payload.bin
d4e4877bac978b7952f0d544fc52ebff5411d351d129f1f056fa43f11da9af2b  ./payload.bin
```

- Both digests are identical, the second `sha256sum` line is the local file

### Explanation
- The matching digests are the acceptance criterion, destination effect over client claims (G15)
- G1 fixed upload truncation in 0.5.3, so use 0.5.3 or newer
- Downloads write a partial file and rename it, and local `set_permissions` failures are surfaced (G18)

### Variants
- Move a tree with `ssh-cli sftp upload --recursive prod ./dist/tree /opt/app/tree --json`, symlinks are not followed
- Fan out with `ssh-cli sftp upload --all ./payload.bin /tmp/payload.bin --json`

### See Also
- SFTP in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Download a File and a Tree via SFTP

### Problem
- You need a remote file or a whole directory tree on your machine, with proof that it arrived

### Solution

```bash
ssh-cli sftp download prod /srv/app/f1 ./out/f1.local --json
ssh-cli sftp download --recursive prod /srv/app/tree ./out/tree --json
ssh-cli sftp download prod /srv/app/missing ./out/m --json
sha256sum ./out/f1.local
```

```json
{"ok":true,"event":"sftp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"download","vps":"prod","local":"./out/f1.local","remote":"/srv/app/f1","bytes":2,"duration_ms":1}
{"ok":true,"event":"sftp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"download","vps":"prod","local":"./out/tree","remote":"/srv/app/tree","bytes":2,"duration_ms":6,"recursive":true}
{"exit_code":66,"error_code":"file_not_found","message":"file not found: /srv/app/missing","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

- The local digest matches the remote one, here both files hold the same two bytes

### Explanation
- The tree call prints one `sftp-transfer` document with `recursive: true` and `bytes`
- A tree keeps its layout, `./out/tree/sub/f` appears for a remote `tree/sub/f`, and symlinks are not followed
- A missing remote path exits 66 with `file not found` and leaves no local file behind
- A single download writes a partial file and renames it, so the final path is never half written
- Permission bits are masked with `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`), so a server cannot hand you a setuid file (A3)

### Variants
- Pull one file from every host with `ssh-cli sftp download --all /var/log/app.log ./app.log --json`, each file lands as `./app.log.<vps>` and the event is `sftp-batch`
- Pull several sources with `--hosts web1,web2 --src f1 --src f2 --dest ./out`, the files land under `./out/<host>/`

### See Also
- The upload recipe above and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Verify a Recursive SFTP Tree

### Problem
- A recursive transfer prints one byte total, so it cannot show that every file arrived intact

### Solution

```bash
mkdir -p tree/sub && echo a > tree/a && echo b > tree/sub/f && ln -s a tree/link
ssh-cli sftp upload --recursive prod ./tree /srv/app/tree --json
(cd tree && sha256sum a sub/f)
ssh-cli exec prod "cd /srv/app/tree && sha256sum a sub/f" --json | jaq -r .stdout
ssh-cli exec prod "ls /srv/app/tree" --json | jaq -r .stdout
```

```text
{"ok":true,"event":"sftp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"upload","vps":"prod","local":"./tree","remote":"/srv/app/tree","bytes":4,"duration_ms":2,"recursive":true}
87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7  a
0263829989b6fd954f72baaf2fc64bc2e2f01d692d4de72986ea808f6e99813f  sub/f
87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7  a
0263829989b6fd954f72baaf2fc64bc2e2f01d692d4de72986ea808f6e99813f  sub/f

a
sub
```

- The two digest pairs are identical, the first pair is the local tree, and the listing has no `link`

### Explanation
- `bytes` is the sum of the file bytes the call moved, 4 here for two files of 2 bytes, and it proves no content
- Equal digests per file are the acceptance criterion, destination effect over client claims
- The symlink is not followed, so it does not reach the remote side
- A recursive call prints one `sftp-transfer` document with `recursive: true`

### Variants
- Check a download the same way with `ssh-cli sftp download --recursive prod /srv/app/tree ./out/tree --json` and `(cd out/tree && sha256sum a sub/f)`
- Verify every host in one call with `ssh-cli exec --all "cd /srv/app/tree && sha256sum a sub/f" --json --select name,stdout`

### See Also
- The checksum recipe above and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Manage Remote Paths via SFTP

### Problem
- You need to list, create, rename and remove remote paths without a shell

### Solution

```bash
ssh-cli sftp ls prod /tmp/rem --json
ssh-cli sftp stat prod /tmp/rem/payload.bin --json
ssh-cli sftp mkdir prod /tmp/rem/app --json
ssh-cli sftp rename prod /tmp/rem/app /tmp/rem/app.bak --json
ssh-cli sftp rmdir prod /tmp/rem/app.bak --dry-run --json
ssh-cli sftp rm prod /tmp/rem/payload.bin --json
```

```json
{"ok":true,"event":"sftp-list","vps":"prod","path":"/tmp/rem","entries":[{"name":"payload.bin","path":"/tmp/rem/payload.bin","kind":"file","size":8,"mode":33188}]}
{"ok":true,"event":"sftp-fs-op","op":"stat","vps":"prod","path":"/tmp/rem/payload.bin","duration_ms":0,"kind":"file","size":8,"mode":33188,"mtime":1792458836}
{"ok":true,"event":"sftp-fs-op","op":"mkdir","vps":"prod","path":"/tmp/rem/app","duration_ms":318}
{"ok":true,"event":"sftp-fs-op","op":"rename","vps":"prod","path":"/tmp/rem/app","to":"/tmp/rem/app.bak","duration_ms":311}
{"ok":true,"event":"dry-run","dry_run":true,"executed":false,"operation":"sftp-rmdir","remote":"/tmp/rem/app.bak","vps":"prod"}
{"ok":true,"event":"sftp-fs-op","op":"rm","vps":"prod","path":"/tmp/rem/payload.bin","duration_ms":210}
```

### Explanation
- `ssh-cli sftp ls` prints the event `sftp-list`, and `ssh-cli sftp mkdir`, `ssh-cli sftp rename`, `ssh-cli sftp rm` and `ssh-cli sftp rmdir` print `sftp-fs-op` with the `op` they ran
- `ssh-cli sftp stat` prints `sftp-fs-op` with `op` set to `stat` and adds `kind`, `size`, `mode` and `mtime`
- `ssh-cli sftp rm` and `ssh-cli sftp rmdir` accept `--dry-run`, the other verbs do not
- Text mode escapes `\n` and `\t` in remote names, so a forged name cannot add fake lines
- Download permission bits are masked with `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) and upload uses `SFTP_PERM_MASK` (`0o7777`)

### Variants
- A missing path exits 66 with `file not found`
- Schemas: `sftp-list`, `sftp-fs-op`, `sftp-transfer` and `sftp-batch`

### See Also
- SFTP in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Generate Shell Completions

### Problem
- You want tab completion for every command and flag

### Solution

```bash
mkdir -p ~/.local/share/bash-completion/completions
ssh-cli completions bash > ~/.local/share/bash-completion/completions/ssh-cli
ssh-cli completions zsh
ssh-cli completions fish
ssh-cli completions powershell
ssh-cli completions elvish
```

- The first line of the bash script, for a quick check

```text
_ssh-cli() {
```

### Explanation
- The script goes to stdout and nothing is installed for you
- The scripts are generated from the same command tree as `ssh-cli commands`

### Variants
- On Windows redirect `ssh-cli completions powershell` into your profile
- Regenerate after every upgrade so new flags complete

### See Also
- Discover contracts above and [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Set the UI Locale

### Problem
- You want messages in one language, once, or for a single call

### Solution

```bash
ssh-cli --lang en locale show --json
ssh-cli locale set pt-BR --json
ssh-cli locale clear --json
```

```json
{"available":["en","pt-BR"],"current":"en","direction":"ltr","persisted_raw":null,"preference_path":"CONFIG_DIR/lang","resolved":"en","script":"Latn","source":"cli_flag","system_raw":"pt-BR"}
{"applies":"next_invocation_unless_overridden","lang":"pt-BR","ok":true,"path":"CONFIG_DIR/lang"}
{"cleared":true,"ok":true}
```

### Explanation
- The preference is the `lang` file in the config directory with mode 0600, never an environment variable
- `--lang` overrides for one call and persists nothing, so `source` is `cli_flag` in the first output
- Error bodies in JSON stay in English so an agent never depends on the host locale, branch on `error_code`

### Variants
- Accepted tags negotiate to `en` or `pt-BR`, for example `en-US` or `pt`, as in `ssh-cli --lang en vps list`
- `ssh-cli locale show` reports the resolved language and `ssh-cli locale set` persists a preference for the next calls
- `ssh-cli locale clear` removes the stored preference and repeating it is harmless
- The winning layer is `cli_flag`, then the stored preference, then the system locale

### See Also
- Locale in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Inspect TLS Provider, Paths and ACME

### Problem
- You need to know the TLS stack, where its material lives and the state of an ACME order

### Solution

```bash
ssh-cli tls provider --json
ssh-cli tls paths --json
ssh-cli tls acme account show --json
ssh-cli tls acme list --json
ssh-cli tls acme status --domain example.com --json
```

```json
{"ok":true,"event":"tls-provider","installed":true,"min_rustls":"0.23.18","provider":"aws_lc_rs","stack":"rustls+aws_lc_rs"}
{"ok":true,"event":"tls-paths","acme_account":"CONFIG_DIR/tls/acme/account.json","acme_dir":"CONFIG_DIR/tls/acme","mtls_dir":"CONFIG_DIR/tls/mtls","tls_root":"CONFIG_DIR/tls"}
{"ok":true,"event":"tls-acme-account-show","directory":null,"path":"CONFIG_DIR/tls/acme/account.json","present":false}
{"ok":true,"event":"tls-acme-list","domains":[]}
{"ok":true,"event":"tls-acme-status","domains":[{"cert_path":null,"domain":"example.com","has_cert":false,"has_key":false,"has_pending_order":false,"key_path":null}]}
```

- Before an account exists, `ssh-cli tls acme issue` and `ssh-cli tls acme complete` fail with exit 66 and these envelopes

```bash
ssh-cli tls acme issue --domain example.com --print-challenge --json
ssh-cli tls acme complete --domain example.com --json
```

```json
{"exit_code":66,"error_code":"file_not_found","message":"file not found: ACME account missing at CONFIG_DIR/tls/acme/account.json; run `ssh-cli tls acme account create`","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
{"exit_code":66,"error_code":"file_not_found","message":"file not found: no pending ACME order for example.com; run `tls acme issue --domain example.com --print-challenge` first","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

### Explanation
- `ssh-cli tls provider` reports the stack: rustls with `aws_lc_rs` only, there is no OpenSSL at runtime
- Building from source needs a C compiler, because the `aws_lc_rs` provider compiles `aws-lc-sys`
- `min_rustls` is the lowest rustls the binary accepts, and `Cargo.toml` asks for rustls 0.23.25
- `ssh-cli tls acme account show` tells whether an account exists, `ssh-cli tls acme list` lists the domains and `ssh-cli tls acme status` shows order and certificate state
- The ACME commands name the domain with `--domain`, a bare positional such as `ssh-cli tls acme issue example.com` exits 2
- Material lives under the config directory in `tls/`, and `ssh-cli tls paths` prints every root

### Variants
- Create the account with `ssh-cli tls acme account create --contact mailto:ops@<YOUR_DOMAIN>`, add `--staging` to rehearse
- The create call contacts the ACME directory, a contact on a reserved domain such as `example.com` exits 64 with `invalidContact` and retrying cannot help
- Order and finish with `ssh-cli tls acme issue --domain example.com --print-challenge` and `ssh-cli tls acme complete --domain example.com`

### See Also
- TLS in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Handle Host Key Rotation Safely (TOFU)

### Problem
- A changed host key may be a legitimate rotation or an interception

### Solution

```bash
ssh-cli exec prod "echo hi" --json
ssh-cli --replace-host-key exec prod "echo hi" --json
```

```json
{"exit_code":77,"error_code":"host_key_changed","message":"host key changed for prod.example.com:22: expected SHA256:STORED, got SHA256:OFFERED (use --replace-host-key if legitimate)","error_class":"permanent","retryable":false,"suggestion":"change credentials (--key / --password-stdin) or host-key policy; do not blind-retry","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
{"target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false,"stdout":"hi\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":92}
```

### Explanation
- The first connection records the key in `known_hosts` and a mismatch later exits 77
- Confirm the rotation with a human before you pass `--replace-host-key`
- The flag replaces the stored key for that one call and later calls pass without it

### Variants
- Keep `--replace-host-key` out of default scripts, it removes the protection against interception
- Do not retry exit 77 blindly, change credentials or the host-key policy first

### See Also
- Exit codes in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Disable Elevation for Untrusted Automation

### Problem
- An untrusted workflow must stay unprivileged whatever the host record stores

### Solution

```bash
ssh-cli --disable-sudo exec prod "id -u" --json
ssh-cli --disable-sudo sudo-exec prod "id -u" --json
```

```json
{"target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false,"stdout":"1000\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":94}
{"exit_code":77,"error_code":"sudo_disabled","message":"sudo/su disabled for this host (disable_sudo)","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

### Explanation
- Plain `ssh-cli exec` still runs because it asks for no elevation
- `ssh-cli sudo-exec` and `ssh-cli su-exec` are blocked for this invocation with exit 77
- The persistent form is `ssh-cli vps edit prod --disable-sudo`, which stores `disable_sudo: true`
- The global `--disable-sudo` placed before `ssh-cli vps add` or `ssh-cli vps edit` also writes `disable_sudo: true`, so that block outlives the call

### Variants
- Lift the persistent block with `ssh-cli vps edit prod --enable-sudo`
- Register a host already blocked with `ssh-cli vps add ... --disable-sudo`
- `ssh-cli --disable-sudo vps edit prod --json` prints `updated` as `["disable_sudo"]`, and `--disableSudo` is an accepted alias of the global flag

### See Also
- Safer agent automation in [HOW_TO_USE.md](HOW_TO_USE.md)


## How To Run Real SSH E2E Without Logging Secrets

### Problem
- You want proof against a real sshd without putting a secret in a log

### Solution

```bash
python3 scripts/e2e_real_ssh.py --local-sshd
python3 scripts/e2e_real_ssh.py --host 203.0.113.10 --port 22 --user deploy --password-file /path/to/pw-file
```

- Expected output of the first command, one label per case and a final summary

```text
PASS E01
PASS E02
PASS E03
PASS E04
PASS E05
PASS E06
PASS E07
PASS E08
PASS E09
PASS E10
PASS E11
PASS E12
PASS E13
PASS E14
PASS E15
PASS E16
PASS E17
PASS E18
PASS E19
PASS E20
PASS E21
---
fails=0 soft_sudo=0 tmp_destroyed=yes
```

### Explanation
- The script prints only PASS, FAIL and SKIP labels, never a host, a user or a password
- `--local-sshd` starts an unprivileged sshd on loopback, it is Unix only
- Without a lab host the script exits 0 with SKIP, offline-safe
- The official matrix is E01 to E21, E10 to E14 cover SCP, E17 and E18 cover SFTP checksum and trees, E19 covers the reduced `vps list --select` item against the published schema, E20 covers `--select` with `--limit` and its stderr accounting, and E21 rotates a live profile into the store with `--force`, proves the round-trip across two processes and proves the inventory written before the rotation stays readable

### Variants
- On Windows run `py -3 scripts/e2e_real_ssh.py ...`
- Point at another binary with `--bin`, the default is `target/release/ssh-cli`
- Prefer a local sshd or a throwaway VPS and avoid auth-failure storms on production because of fail2ban

### See Also
- The catalog of suites in [TESTING.md](TESTING.md)
