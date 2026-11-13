# Migration Guide


> Move from ssh-cli 0.3.3 (or later) to 0.5.6 without losing multi-host inventory.

- Read this document in [Portuguese (pt-BR)](MIGRATION.pt-BR.md)


## What Changes

### Since 0.5.6 (upgrade from 0.5.5)
- 0.5.5 is the latest release on crates.io, and this section is the whole path from it to 0.5.6
- BREAKING: the minimum supported Rust version (MSRV) is Rust 1.89, so a toolchain older than 1.89 no longer builds this release, even if it built 0.5.5
- The reason is `russh` 0.65.1, which declares `rust-version = "1.89"` and closes RUSTSEC-2026-0492, RUSTSEC-2026-0495 and two GHSA advisories
- Building from source still needs a C compiler, because the Transport Layer Security (TLS) provider is `aws_lc_rs` and `aws-lc-sys` compiles C, and a migration to pure Rust cryptography was tried and reverted
- The keyring backend alone would need 1.88.0, because `keyring` 4.2.0 declares `edition = "2024"` and `rust-version = "1.88.0"`, and the zbus Secret Service store follows
- Run `rustup update stable` before `cargo install ssh-cli --locked`
- `secrets init --force` no longer writes `secrets.key.bak`
- If you relied on that copy to recover from a lost key, export the key beforehand or keep the previous value supplied through `--secrets-key-file`
- The error that told you to restore `secrets.key.bak` now points at `ssh-cli secrets reencrypt` with the previous key
- `secrets init --keyring` now writes to the OS store instead of an in-process mock, so the following `secrets status` reports `key_source: "keyring"` and `encryption_active: true`
- The store entry is scoped by the config directory, so a profile created with `--config-dir` no longer reads or overwrites the key of the real profile
- A key written by a previous `secrets init --keyring` is now found without repeating the flag, which is what makes it a store rather than a per-process variable
- The global `--use-keyring` now reaches `secrets init` too, so `--use-keyring secrets init` writes to the store instead of silently writing `secrets.key`
- On a machine with no answering credential store, `secrets init --keyring` exits 69 instead of reporting success
- BREAKING: a profile owns exactly one primary key, so `secrets init` now refuses with exit 64 when the profile already has a key in either store, and the message names `--force`
- Before this release the command accepted `--keyring` on a profile that held `secrets.key`, minted a second key and left the file answering first, which made every stored secret undecryptable
- `secrets init --force --keyring` now moves the key into the store and removes `secrets.key`, and `secrets init --force` on a store profile writes the file and removes the store entry
- If a profile already carries two keys from an earlier build, run `ssh-cli secrets init --force` once with the store you want to keep, then `ssh-cli secrets reencrypt`
- `vps list --select` now satisfies `docs/schemas/vps-list.schema.json`: the reduced item is the second published shape, and the reduction report stays on stderr
- Nothing changes for a consumer that reads the unshaped list, because that form is unchanged
- The global shaping flags changed behaviour for single-record payloads: `--select` and `--truncate-content` now reduce `vps show`, `doctor`, `commands` and every other object without a collection
- Read a field with `ssh-cli doctor --select local.config_dir_exposed` instead of piping to `jaq`
- A reduced record carries `agent_shape`, so a consumer that walks every key of `vps show --json --select` sees one extra object
- `docs/schemas/vps-show.schema.json` publishes both shapes through `anyOf`, and an unshaped run carries no `agent_shape`
- On a single record, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` and `--max-output-bytes` stay inert, and `agent_shape.ignored` now names each one that was passed instead of dropping it in silence
- A reduced bare array writes one JSON line to stderr with `"event":"agent-shape"`, carrying `input_count`, `output_count` and `dropped`, while stdout keeps the array's type and `--quiet` suppresses the line
- If your runner treats any stderr output as failure, pass `--quiet` or read the line and ignore it: the exit code and stdout are unchanged
- Rename is native: `ssh-cli vps rename <NAME> <NEW_NAME>` and `ssh-cli vps edit <NAME> --name <NEW_NAME>` move the registry key and keep every credential
- Stop using `ssh-cli vps export` → edit → `ssh-cli vps import` for this: the export redacts secrets, so the round trip forced you to retype the password
- Before 0.5.6 the round trip also dropped `use_agent` and `agent_socket`, so an agent-authenticated host silently lost its auth
- Since 0.5.6 `ssh-cli vps export` and `ssh-cli vps import` carry both fields, as measured in the export of an agent host
- BREAKING: `ssh-cli vps edit` switches the auth method, `--key` clears the password and the agent, `--password`/`--password-stdin` clear the key, its passphrase and the agent, and `--use-agent` clears the password and the key
- Before, a password host could not take a key (exit 64, `primary auth methods are mutually exclusive`) and the workaround was to pass through `--use-agent`
- Drop that workaround
- `updated` now lists what was cleared, but only fields that held a value
- Conflicting auth flags exit 2: `--key` with `--password*`, `--agent-socket` with `--key`, `--password*` or a passphrase flag, and a passphrase flag with `--use-agent` or `--password*` fail at parse time
- A passphrase with no key exits 64 and leaves the record untouched
- `ssh-cli vps edit X --use-agent --agent-socket S` stays valid
- A different `--key` path drops the old passphrase unless the same call brings a new one, and `updated` lists `key_passphrase`
- Before, a different key kept the passphrase of the old key on disk, an orphan that served no key
- `--agent-socket` alone now implies `--use-agent` and clears `password`, `key_path` and `key_passphrase`, so a password host given only a socket becomes an agent host
- Before, `--agent-socket` alone stored the socket on a host that kept authenticating by password, so it looked configured for the agent and was not
- BREAKING for hosts you create: names are strict, `ssh-cli vps add`, `ssh-cli vps rename`, `ssh-cli vps edit --name` and `ssh-cli vps import` accept only ASCII letters, digits, dot, hyphen and underscore, alphanumeric first, at most 255 bytes
- The strict rule also refuses `..`, a trailing dot or space, and Windows reserved names such as `CON`
- Older names with spaces or accents still load, list, show, resolve and can be renamed to a strict name
- `ssh-cli vps doctor` lists them in `local.invalid_host_names`
- A name with control or bidi characters stored in `config.toml` still loads: `ssh-cli doctor` lists it escaped, text output escapes it, and `ssh-cli vps rename '<old name>' <new name>` repairs it
- `ssh-cli vps import` validates every item and lists all bad names at once (`invalid VPS names in import (N): ...`, exit 64, nothing written, same in `--dry-run`)
- Lookups that become a target (`ssh-cli exec`, `ssh-cli health-check`, `--hosts` and `ssh-cli connect`) refuse a name with control or bidi characters as a usage error with exit 64, and `ssh-cli connect` writes no marker
- `ssh-cli tls mtls import --name` refuses control, formatting and bidi characters with exit 64 and creates no directory, while ordinary Unicode such as `serviço-1` is still accepted
- Two `[hosts]` tables whose names normalize to the same NFC name exit 65 with `config has two hosts named ...` (names escaped), and the loader keys the map by the NFC record name, so no host is lost silently
- An export of a legacy host with a non-strict name restores only after `ssh-cli vps rename`
- Text mode escapes control and bidi characters: as `\u{1b}` in messages, `ssh-cli vps list`, `ssh-cli vps show`, `ssh-cli health-check`, `ssh-cli doctor` and batch output
- JSON is unchanged, and remote `ssh-cli exec` output stays raw
- `ssh-cli sftp ls` and `ssh-cli sftp stat` also escape `\n` and `\t` in the remote name in text mode
- The config directory is born `0700`: An existing default XDG directory is tightened to `0700`
- Any other existing directory (a `--config-dir` you pointed at) is never chmodded, writes a log warning (visible with `-v`) and shows as `local.config_dir_exposed` in `ssh-cli doctor --json`
- Read it with `ssh-cli doctor --json --select local.config_dir_exposed`, because `--select` projects the `ssh-cli doctor` object directly
- `ssh-cli doctor --json` gains `local.config_dir_permissions` (a four-digit octal string such as `0700`, absent off Unix), `local.config_dir_exposed` (boolean, always present) and `local.invalid_host_names` (array, always present)
- `--config-dir something.toml` makes the parent folder the config directory for `secrets.key`, `known_hosts`, `active`, `lang` and the TLS files, where before it left a folder named `something.toml` behind
- `ssh-cli vps export -o new/deep/e.json` creates the missing folders `0700` and never changes an existing one, and `known_hosts.lock` and `lang` are born `0600`
- Windows has no chmod, so the folder relies on the profile ACL
- BREAKING for library users: `paths::validate_and_normalize` returns `String` and keeps only the file-name rule. Call `VpsName::try_new` for host names
- BREAKING for library users: `Money`, `Brl`, `Usd`, `Currency` and the `rust_decimal` dependency are removed, and RUSTSEC-2026-0235 (it came in through `rkyv`) leaves the lock with them
- The nine `scripts/*.sh` are deleted
- Only the `.py` programs remain
- The stray file `json` in the crate root is deleted, and the package `exclude` now lists `/json`, `*.sh`, `__pycache__/`, `*.pyc` and `*.snap.new`
- `ssh-cli vps edit` can clear a field now: Eight flags remove instead of overwrite: `--no-key` (also clears the passphrase), `--no-agent-socket`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key` and `--clear-tags`
- Each conflicts with the value flag of the same field, so `--no-key --key` is a parse error rather than an ambiguous instruction
- Clearing the only remaining auth method is refused with exit 64
- `--no-agent-socket` on a host that authenticates through the agent is also refused with exit 64 on Unix, because no default agent endpoint is left, while Windows has a default endpoint and allows it
- New events: `ssh-cli vps rename --json` and `ssh-cli vps edit --name --json` both emit `event: "vps-renamed"` with `name`, `previous_name`, `active_rewritten`, `updated`, `target_resolved` and `target_source`
- `vps-edited` is emitted only when nothing was renamed and carries no `previous_name`
- A parser that waited for `vps-edited` after `ssh-cli vps edit --name` must now accept `vps-renamed`
- `updated` is always present: the fields whose stored value changed, in application order, each at most once
- It is empty when every requested value already matched or when the invocation only renamed the host
- Target provenance is published only after the registry lookup, so `vps_not_found` still arrives without a target
- An edit that changes nothing is an idempotent no-op: It exits 0, emits `vps-edited` with `updated: []` and does not rewrite `config.toml`
- Exit 64 is reserved for an invocation with no field, rename or clear flag at all
- `ssh-cli vps rename` is not idempotent: `ssh-cli vps rename ghost ghost` exits 66 because the existing-host precondition wins, an identical name for an existing host exits 64 and the error carries `target_resolved`, and repeating a rename after a success exits 66 (the error hint says to check `ssh-cli vps list`)
- Confirm a rename with `ssh-cli vps show <NEW_NAME>`, never by repeating it
- Rename write order is fail-closed: The `active` marker is written first and `config.toml` second
- If the save fails the old marker is restored, and if the marker write fails nothing is renamed
- A crash between the two steps leaves an orphan marker, and `--use-active` then exits 66 instead of acting on the wrong host
- `--dry-run` grew from six to eight operations: `ssh-cli vps edit` and `ssh-cli vps rename` now preview
- The `operation` enum in `dry-run.schema.json` lists all eight
- The `--dry-run` plans of `ssh-cli vps edit`, `ssh-cli vps rename` and `ssh-cli vps remove` echo `target_resolved`, `target_source` (`argv`), `host_resolved` and `host_source` once the lookup succeeds
- `ssh-cli vps edit`, `ssh-cli vps rename` and `ssh-cli vps remove` on a config that does not exist exit 66 and create no directory and no lock file, with or without `--dry-run`
- The config lock is bounded: Contention used to block forever, which for a one-shot agent means no output, no exit, no diagnosis
- It now waits `--lock-wait-ms` (default 5000, accepted range 1 to 300000) and exits 69 with `unavailable`, which is retryable
- A signal during the wait still wins with 130 or 143
- `ssh-cli commands --json` publishes arguments: Each command node now carries `args` with `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` and `hidden`, so an agent can discover `--no-key` without running `--help`
- `ssh-cli --help` now gains an `Exit status:` section that lists the exit codes 0, 1, 2, 64, 65, 66, 69, 70, 73, 74, 77, 130, 141 and 143
- All 48 leaf commands carry an `Examples:` section in their `--help`, and `-h` stays short
- Both texts appear only in the long help, and `ssh-cli commands --json` and every JSON envelope are unchanged
- `--disable-sudo` has two forms: the global flag disables `ssh-cli sudo-exec` and `ssh-cli su-exec` for the invocation (alias `--disableSudo`, accepted only on the global flag) and, placed before `ssh-cli vps add` or `ssh-cli vps edit`, also saves `disable_sudo` in the record, while `ssh-cli vps add --disable-sudo` and `ssh-cli vps edit --disable-sudo` save the choice in the host record, and `ssh-cli vps edit --enable-sudo` undoes it
- The lock wait reacts to signals within 100 ms: It waits in 100 ms slices that check the signal, so SIGINT and SIGTERM exit 130 and 143 within one slice (before, up to about 5 s)
- `config.toml.lock` is created with mode 0600, the marker write fsyncs its directory, and the exit 69 hint says to repeat the argv or raise `--lock-wait-ms`
- It no longer mentions the keyring
- Text mode no longer swallows stdout write errors: In every verb a broken pipe now exits 141 and a full disk is an I/O error
- Before, both exited 0. On `ssh-cli exec` with stdout closed the write error wins over the remote command failure
- Wrappers that relied on exit 0 with a closed pipe must handle the failure
- `ssh-cli vps import` validates TLS like `ssh-cli vps edit`: A certificate without a key exits 64
- The `vps-import` success event gains `replaced` (the overwritten names, sorted, always present)
- The dry-run keeps `replaces_existing`
- `ssh-cli vps import` also takes the config lock before it loads the registry, so an import that commits between a peer's load and save no longer discards the peer's host
- `ssh-cli vps import --file` on a missing file exits 66 `file_not_found` as a permanent error, where it used to exit 74 `io` with `retryable: true`, so stop retrying that path
- `ssh-cli vps list` in text mode sizes each column from its widest cell, and the PASSWORD column shows `(not set)` for a key-only host instead of a mask
- `ssh-cli vps show` in text mode aligns all its labels, so a parser that cut text output at fixed widths must split on the first colon instead
- A `ssh-cli tunnel` stopped by SIGINT exits 130 and one stopped by SIGTERM exits 143, and the `tunnel_closed` event carries `reason: "signal"`
- The primary key is resolved in this order: `--secrets-key-file`, then the keyring with `--use-keyring`, then XDG `secrets.key`, then the keyring entry left by an earlier `secrets init --keyring`
- Only `SSH_CLI_SECRETS_KEY` and `SSH_CLI_SECRETS_KEY_FILE` are refused, so a command that reads or writes a registry secret exits 64 while either is set, and the other `SSH_CLI_*` names are ignored
- Fourteen schemas in `docs/schemas/` were realigned with the output the binary emits, so revalidate any consumer that pinned an older copy
- `--version` provenance: `.commit_hash` now holds `<short hash> <version>`
- Resolution order is env, git, the file, then `unknown`
- Without env and git the file counts only when its version is the package version
- `build.rs` no longer reruns on every build without `.git`, so 0.5.6 prints `unknown` until the release refresh step of the [release checklist](RELEASE_CHECKLIST.md)
- BREAKING for library users: `SshCliError::Unavailable` carries an enum. The variant is now `Unavailable { service: UnavailableService }` and the constructor is `SshCliError::unavailable(UnavailableService)`
- `UnavailableService` is `#[non_exhaustive]` with `Keyring` and `ConfigLock`
- Replace string arguments with the enum and add a wildcard arm when matching it
- The wire text is unchanged (`service unavailable: config_lock`), so CLI and JSON consumers are not affected
- BREAKING for release scripts and automation: the nine `scripts/*.sh` became `scripts/*.py`. They need Python 3.9+ and only the standard library
- On Windows run `py -3 scripts/<name>.py`
- Replace `bash scripts/<name>.sh` with `python3 scripts/<name>.py` for `check_all_gates`, `check_advisory_freshness`, `check_cross_targets`, `check_en_identifiers`, `verify_install_resolve`, `generate_sbom`, `dist_multiarch`, `release_attest` and `e2e_real_ssh`
- The helpers `_gatekit.py`, `_e2e_sshd.py` and `_e2e_cases.py` are not entry points
- Script environment variables became flags: `CHECK_TIMEOUT_SECS` is `--timeout-secs` (`check_cross_targets`, and the scripts that call cargo take it too), `TARGETS`, `OUT_DIR` and `PARALLEL_JOBS` are `--targets`, `--out-dir` and `--jobs` (`dist_multiarch`), `REQUIRE_COSIGN` is `--require-cosign` (`release_attest`), `CARGO_TARGET_DIR_INSTALL_GATE` is `--target-dir` (`verify_install_resolve`), and `check_advisory_freshness` takes `--db-root`
- `e2e_real_ssh` takes `--host --port --user --password-file --sudo-password-file --from-grok-config [PATH] --bin --local-sshd`
- The `SSH_CLI_E2E_*` variables and `--config-dir` are gone
- `--from-grok-config` needs Python 3.11+, and `--local-sshd` is Unix-only (on Windows it fails with `local_sshd=requested_but_unusable`)
- For maintainers: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean again, the five red tests are green, `src/vps/edit/apply_tests.rs` uses `tempfile` instead of a fixed path in `/tmp`, and the audit record of 0.5.6 stays local and unpublished
- The agent skills `skills/ssh-cli-en` and `skills/ssh-cli-pt` now teach the 48 leaf commands, `ssh-cli vps rename`, the auth switch, the SANITIZE step, the raw `sudo` rule, the trust on first use (TOFU) rule and the persistent `--disable-sudo` form, so replace any installed copy made before 0.5.6
- The `test` gate of `scripts/check_all_gates.py` runs `cargo test --locked --all-features --no-fail-fast`, so one red test binary no longer hides the others
- Four new suites, `gaps_v073_config_dir_mode`, `gaps_v073_host_name_charset`, `gaps_v073_terminal_escape` and `gaps_v073_vps_auth_switch`, keep these fixes from returning, and [TESTING](TESTING.md) catalogues every suite on disk
- Additional walk-through: [COOKBOOK](COOKBOOK.md) documents the rename recipe, including why it costs no credential

### Since 0.5.5
- BREAKING: a lone positional no longer designates a host (GAP-SSH-EXEC-ARGC-001, CWE-441), `ssh-cli exec`, `ssh-cli sudo-exec` and `ssh-cli su-exec` used to read the target from the count of positionals, so `ssh-cli exec prod --step uptime` parsed `prod` as the command and ran it on whichever host `ssh-cli connect` had last marked
- Same argv, different machine, no diagnostic
- One positional is now a usage error (exit 64)
- BREAKING: `ssh-cli health-check` requires a target too. A nameless `ssh-cli health-check` exits 64 instead of silently probing the active marker
- Reach the marker with `--use-active`: `ssh-cli health-check --use-active` and `ssh-cli exec --use-active <COMMAND>` are the only ways to inherit what `ssh-cli connect` wrote
- With no marker on disk they exit 66
- The split is stable across the whole CLI: 64 means no target was designated, 66 means the designated target does not exist
- Three legal forms, and nothing else: a name (`ssh-cli exec prod uptime`), a fleet selector (`--all`, `--hosts`, `--tags`), or the explicit `--use-active` opt-in
- Wrappers that relied on the implicit form must pick one
- `scp` and `sftp` follow the same rule: Transfers name their host in argv
- `--src`/`--dest` slots and positionals may not be mixed so that a leftover word silently becomes a hostname
- Every envelope now carries its provenance: `target_resolved`, `target_source` (`argv` | `active_marker` | `selector`) and `active_fallback` ship on success, and on the error envelope once the host was resolved, absent, never empty, when the failure predates resolution
- `host_resolved` and `host_source` are read aliases of the first two, written from the same value
- Bind new consumers to the canonical `target_*` spelling
- `ssh-cli doctor` reports `local.active_vps`
- BREAKING for library users: `ssh_cli::telemetry` is now `ssh_cli::tracing_setup`, with no deprecated alias, and the module never emitted telemetry

### Since 0.5.4
- BREAKING: partial multi-host failure now exits `1` (was `65`). Any wrapper that branched on `65` to mean "some hosts failed" must be updated
- `65` is `EX_DATAERR`, which claims the input was malformed
- A batch where three of ten hosts were unreachable had perfectly valid input, so the old code lied about the cause (G-ERR-R02)
- Check the batch envelope's per-host results, not just the process exit
- BREAKING: a non-loopback `ssh-cli tunnel --bind` now requires `--i-accept-network-exposure`. Any script passing `--bind 0.0.0.0` or a LAN address fails until the acknowledgement is added
- Before this, the flag silently published the forwarded remote service to the whole local network (G-TUN-R13)
- Under `--reverse` the same flag guards the server's bind address instead, because that is the exposed end in that direction
- `ssh-cli tunnel` gained three modes: The default local forward is unchanged, so existing invocations keep working
  - `--reverse` (G-TUN-R01) asks the server to listen and deliver connections back to your local port, the callback-webhook and inverted-bastion case that previously forced operators out of this CLI and into system `ssh`
  - `REMOTE_PORT` may be `0`, letting the server allocate and report the port
  - `--socks5` (G-TUN-R02) serves a local SOCKS5 proxy (RFC 1928 no-auth + CONNECT) choosing a destination per connection, so `REMOTE_HOST`/`REMOTE_PORT` are omitted
  - Replace N `ssh-cli tunnel` processes aimed at N hosts behind one bastion with one invocation: the handshake is paid once
  - `--remote-socket <PATH>` (G-TUN-R03) forwards to a remote Unix socket over `direct-streamlocal@openssh.com`, reaching container-daemon, PostgreSQL and systemd sockets that never listen on TCP
  - The path must be absolute or exit 64
- New `tunnel_closed` event: `ssh-cli tunnel --json` now emits it on shutdown with `reason`, `forwards_served` and `capacity_waits`
- Add it to any parser that treated `tunnel_listening` as the only tunnel event
- It is the only way to distinguish a clean deadline from a saturated semaphore, since all endings exit 0
- Eight payload-shaping flags are now global: `--select` (alias `--fields`), `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- They apply before serialization, so replacing an external JSON post-filter with these flags stops the oversized envelope from ever being written
- No existing behaviour changes when they are omitted
- `--no-input` and `--dry-run` are global: `--dry-run` is honoured only by `ssh-cli vps remove`, `ssh-cli vps import`, `ssh-cli vps edit`, `ssh-cli vps rename`, `ssh-cli sftp rm`, `ssh-cli sftp rmdir`, `ssh-cli secrets init` and `ssh-cli secrets reencrypt`
- Anywhere else it is refused with exit 64 rather than accepted and ignored
- Security fixes: upgrade for these alone: A1 truncates the server-sent pre-auth SSH banner on character boundaries rather than byte indices, the 512-character log cap already existed (G-SSH-14)
- What was broken is that slicing by byte index panics whenever the cut lands inside a multi-byte character, and the release profile sets `panic = "abort"`, so there is no unwind: the process dies and takes a whole multi-host fan-out with it, A2 creates ACME/mTLS private keys at `0600` instead of tightening the mode after the file already exists, A3 masks server-sent modes to plain `rwx` triples so setuid, setgid and sticky cannot ride a download onto a local file
- See `SECURITY.md` for the mechanism of each
- `ssh-cli schema dry-run` and `ssh-cli schema tunnel-closed` now emit their documents with exit 0 (previously exit 64), so schema discovery covers the full catalogue

### Since 0.5.3
- G1: SFTP upload no longer truncates destination to zero bytes, prefer 0.5.3+ for all SFTP
- Verify with destination `sha256sum`
- G2 / G14: Verbosity is graduated (`-v` info / `-vv` debug / `-vvv` trace) and always crate-scoped (`warn,ssh_cli=…`)
- Bare global debug removed (no password leak via russh)
- Ambient `RUST_LOG` still ignored
- G3: SFTP SETSTAT sends `atime`+`mtime` together (no epoch atime)
- G4: SFTP `set_metadata` Result is fail-closed (mutating SETSTAT is not best-effort)
- G5 / G17: Multi-file SCP/SFTP cancel fills cancelled remainder
- `results.len() == input.len()`
- G6: Signal/cancel tests that touch global state (`CANCEL_FLAG`) use `serial_test` isolation so the suite is deterministic (not agent runtime)
- G7: Official real-SSH E2E covers SFTP checksum matrix + recursive tree (E17/E18)
- Full matrix E01–E21
- G8: `ssh-cli exec --json` single-step emits exactly one JSON object (no dual events on the success path)
- G9: SCP download propagates `sync_data` failure before atomic rename
- G10: Release gate includes `cargo fmt --check`
- G11: Baseline suite is green without re-run lottery
- Re-running until pass is not a gate strategy
- G12 / G19: Permission bits masked with named `SFTP_PERM_MASK` (`0o7777`) on the outbound upload path
- Since 0.5.4 the inbound download path masks with `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) instead, because that mode comes from the server, see A3 above
- G13 / G15: No circular tests that assert FIXED text in the local audit record
- Acceptance requires destination effect proof (checksum)
- The audit record is maintainer-local (gitignored / cargo-excluded) and not a published contract
- G16: English identifiers and channel errors on the SCP client path (`client_real_scp.rs`)
- G18: SFTP download local `set_permissions` failures are surfaced

### Since 0.3.4 (core SSH automation parity)
- Install crypto graph is pinned so `cargo install --locked` succeeds (GAP-014)
- Auth accepts private keys through `--key` / `key_path` (GAP-002)
- `max_chars` semantics split into `max_command_chars` and `max_output_chars` (GAP-004)
- `ssh-cli sudo-exec` packs commands with secure `sh -c` (GAP-005)
- `ssh-cli su-exec` consumes stored `su` password (GAP-003)
- Config writes are atomic with flock and mode 0600 (GAP-007)
- Host keys use TOFU known_hosts (GAP-008)
- `ssh-cli tunnel` requires `--timeout-ms` (GAP-010)
- Schema version for new records was 2 at that time (historical, current wire is schema v3)
- License is dual MIT OR Apache-2.0

### Since 0.3.5
- Atomic `ssh-cli vps export`, stronger remote abort (TERM+KILL)
- Optional AEAD path matured
- Doctor reports `secrets_at_rest`
- Auto JSON when stdout is not a TTY

### Since 0.3.6
- Default at-rest encryption of secrets in `config.toml` (ChaCha20-Poly1305)
- Auto-creates XDG `secrets.key` (0o600) on first secret write
- CLI `ssh-cli secrets status`, `ssh-cli secrets init` and `ssh-cli secrets reencrypt` (never prints the primary key)
- Opt-out for tests only: `--allow-plaintext-secrets` (CLI-only, no env store)
- Doctor fields: `secrets_key_file`, `secrets_plaintext_opt_out`

### Since 0.3.7
- Agent I/O polish: global `--output-format` on VPS CRUD, `ssh-cli health-check --json`, JSON error envelope, `--quiet` silences human success
- Tunnel `--timeout-ms` covers SSH connect + loop
- SCP validates local file before connect
- `ssh-cli vps remove` clears orphan `active`
- `ssh-cli su-exec --password-stdin`
- Clap conflicts for password/*_stdin
- Remote non-zero exit maps to process exit `1` with `remote_exit_code` in JSON error envelope
- Long secrets always mask as the fixed mask (no 12+4 prefix leak)
- sudo/su password on channel stdin, not remote argv

### Since 0.3.8
- russh upgraded to 0.62.2 (security floor ≥0.60.3)
- Tunnel human banners stay off agent stdout (JSON/non-TTY/quiet)
- No active VPS returns sysexits 66 (`EX_NOINPUT`) via typed error
- `cargo deny`: `yanked=deny`, empty ignore list
- `multiple-versions=warn` for transitive duplicates
- Version string reports `-dirty` when the working tree is dirty
- Full residual suite `tests/gaps_v038_integration.rs`

### Since 0.4.1 (historical)
- AUD-POST patch: empty secrets never become `sshcli-enc` blobs on redacted export (EXP-001)
- Tunnel post-bind deadline exits 0 (TUN-002)
- `ssh-cli tunnel`/`ssh-cli health-check` auth flag parity with exec/scp (CLI-005/006)
- SCP JSON includes `event: "scp-transfer"` (IO-009)
- Additive only: no breaking CLI changes
- SCP wire fix (0.4.0): crates.io 0.3.9 advertised SCP but the protocol was broken
- Upgrade to 0.4.0+ (prefer product line 0.5.6) before relying on `scp`
- SCP is regular files only (no `-r`)
- Directory trees use `sftp --recursive`
- Use `--timeout` for large files (covers connect + transfer)
- Success JSON via `--json` / `--output-format json` (`docs/schemas/scp-transfer.schema.json`, and `sftp-transfer.schema.json` for SFTP)
- SCP download writes `{path}.ssh-cli.partial` then atomic rename
- Mode/times applied on the partial before rename
- SCP upload streams in 32 KiB chunks (no full-file `fs::read` into RAM)
- Preserve mtime/mode bi-directional (remote `scp -tp` / `-fp`, parse `T` + `C` mode)
- SCP flag parity with exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json`
- `scp --json` failures emit JSON error envelope on stderr (`exit_code`, `message`), parity with tunnel (IO-007b)
- `ssh-cli tunnel --json` emits one stdout object `event: "tunnel_listening"` after local bind (`docs/schemas/tunnel-listening.schema.json`)
- Still requires `--timeout-ms`
- Default tracing level is error (not info)
- `-v` enables debug
- Ambient `RUST_LOG` is ignored: JSON/tunnel stderr stays clean by default
- Empty or missing password on key-only VPS serializes as JSON `null` (not the mask string)
- Non-empty still masks as the fixed mask
- Human text show prints "(not set)" for an empty field, in English and in pt-BR, because the label is not translated
- `ssh-cli health-check` accepts `--timeout <ms>` override (aligned with exec)
- Product-line docs of that era aligned to 0.4.1
- Suites `tests/gaps_v039_integration.rs` + `tests/gaps_v040_integration.rs` + `tests/gaps_v041_integration.rs`
- Official e2e E01–E14 (E10–E14 cover SCP)

### Since 0.4.2 (historical, additive)
- Tunnel ephemeral local port 0: after bind, JSON/banner report the OS-assigned port (never 0 post-bind) (TUN-003)
- Formal `ssh-cli vps export --json` envelope (`event: "vps-export"`) matured
- Empty secrets stay `""` on redacted export
- Official e2e E15 (tunnel port 0) + E16 (symlink)
- Suite `tests/gaps_v042_integration.rs`


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

### Upgrade from 0.5.5 to 0.5.6
- Step 1: install with `cargo install ssh-cli --locked --force` and confirm that `ssh-cli --version` starts with `ssh-cli 0.5.6`
- Step 2: read the three new `ssh-cli doctor` fields

```bash
ssh-cli doctor --json --select local.invalid_host_names
ssh-cli doctor --json --select local.config_dir_exposed
ssh-cli doctor --json --select local.config_dir_permissions
```

- Step 3: rename every legacy host listed in `invalid_host_names` to a strict name with `ssh-cli vps rename 'old name' new-name`, because an export of a legacy host restores only after the rename
- Step 4: when `config_dir_exposed` is `true`, tighten the folder yourself on Unix (for example `chmod 700` on the folder), because the CLI never changes the mode of a directory it does not own by default
- Step 5: update wrappers that wait for `vps-edited` after `ssh-cli vps edit --name`, so they accept `vps-renamed`
- Step 6: update wrappers that depend on the old refusals, because `--key` on a password host and `--password` on a key host now succeed, two auth flags in one call exit 2, and a host name outside `[A-Za-z0-9._-]` exits 64
- Step 7: update wrappers that parse text mode, because control and bidi characters now appear as `\u{1b}`-style escapes while JSON is unchanged
- Step 8: replace `bash scripts/<name>.sh` with `python3 scripts/<name>.py` in release scripts, and use `py -3 scripts/<name>.py` on Windows
- Step 9: library users call `VpsName::try_new` for host names, drop `Money` and `rust_decimal`, and match `SshCliError::Unavailable { service }` with a wildcard arm
- Step 10: replace installed copies of the agent skills, then check the result with `ssh-cli vps list --json` and `ssh-cli schema vps-edit`

### If you still have plaintext secrets on disk
- On first save with 0.3.6+, a `secrets.key` is auto-created and new writes encrypt
- To re-cipher an existing plaintext inventory:

```bash
ssh-cli secrets init   # if secrets.key does not exist yet
ssh-cli secrets reencrypt
```

- Backup `config.toml` and `secrets.key` offline
- Losing the key makes encrypted blobs unreadable

### Switch a host from password to key

```bash
ssh-cli vps edit prod --key ~/.ssh/id_ed25519
printf '%s' '...' | ssh-cli vps edit prod --password-stdin
```

- Since 0.5.6 each call switches the method directly and clears the old secrets
- Before, the first one exited 64 and the detour went through `--use-agent`

### Re-check elevation secrets (prefer stdin)

```bash
printf '%s' '...' | ssh-cli vps edit prod --sudo-password-stdin
ssh-cli sudo-exec prod "id"
ssh-cli su-exec prod "id"
```

### Update agent wrappers
- Pass `--timeout-ms` for tunnels
- On `ssh-cli tunnel --json`, wait for `event == "tunnel_listening"` before using the local port
- TUN-002: after `tunnel_listening`, post-bind one-shot deadline exits 0 (do not treat 74 as failure if bind already signaled)
- Pre-bind timeout remains 74
- EXP-001: on redacted `ssh-cli vps export`, do not expect or parse `sshcli-enc:` for empty secrets, empties serialize as `""`
- IO-009: parse SCP success with `docs/schemas/scp-transfer.schema.json` including required `event: "scp-transfer"`
- CLI-005: `ssh-cli tunnel` accepts `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`
- CLI-006: `ssh-cli health-check` accepts `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`
- If you came from 0.4.0: redacted export could show fake empty-password ciphertext
- Tunnel could emit `ok:true` and still exit 74, upgrade wrappers and the binary to 0.5.3
- On SCP/tunnel `--json` failure, parse stderr error envelope (not human prose)
- Treat SCP as regular files only
- Do not send directory trees
- Prefer 0.5.3+ for SFTP
- Re-verify uploads with destination checksum after upgrade (G1)
- Re-test transfers after leaving 0.3.9 (that release SCP was not trustworthy)
- Treat `--maxChars` as input limit, not output limit
- Prefer `--password-stdin` for secrets
- Password on argv warns on stderr (0.5.2+)
- `ssh-cli vps add --timeout` with a value under 1000 ms warns on stderr (unit is milliseconds, not seconds), while `ssh-cli vps edit --timeout` accepts it silently
- Empty remote command fails with technical message `empty command` (any locale)
- Handle host-key mismatch errors before forcing replace
- Expect encrypted `config.toml` values prefixed with `sshcli-enc:v2:`, because every new write is `v2` and `v1` blobs are still read
- Expect default tracing error
- Use `-v`/`-vv`/`-vvv` when debugging (ambient `RUST_LOG` is ignored)
- Do not parse stderr as success JSON
- Parse single-host `ssh-cli exec --json` as one object (G8)
- ACME `invalidContact` / permanent validation → exit 64 (do not retry as 74) (G-E2E-01)
- First `ssh-cli vps add` with auto-key: one JSON document `event: "vps-added"` with field `secrets_key_auto_created` (G-E2E-04)
- Prefer root `ssh-cli schema` / `ssh-cli doctor` for agent discovery (G-E2E-02/03)
- Register agent-only hosts with `ssh-cli vps add --use-agent` / `--agent-socket` (G-E2E-19)
- Redacted export clears secrets instead of masking them: `password` is `""`, and `sudo_password`, `su_password` and `key_passphrase` are `null`
- The fixed mask appears only in `ssh-cli vps list` and `ssh-cli vps show`, where a key-only host shows `password` as `null` (G-E2E-10)
- clap feature `env` removed: no `#[arg(env=…)]` product config (G-E2E-08)
- Version stamp appends `-dirty` when the working tree is dirty even with `.commit_hash` (G-E2E-06)
- Treat empty password in list/show JSON as `null` for key-only hosts
- May pass `ssh-cli health-check --timeout <ms>` when host default timeout is too long or short
- Expect process exit `1` (with `remote_exit_code` in JSON envelope) when the remote command fails
- Expect no active VPS as exit 66
- Missing SCP file as exit 66 with `file not found: <path>`
- Expect tunnel banners only in human/TTY paths, not on agent JSON stdout
- Secrets control is CLI/XDG only (`--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, XDG `secrets.key`)
- Env secrets stores are rejected fail-closed
- Do assume auto JSON non-TTY applies to `ssh-cli vps export`, the body is JSON on any non-TTY stdout, and TOML needs `--output-format text`


## JSON Schema Changes
- Historical (0.3.4 era): new host records wrote `schema_version` 2 with the field set of that release
- Current (0.5.3): new writes use schema v3 and English TOML keys
- Loads dual-read legacy Portuguese key aliases
- Agent event schemas live under `docs/schemas/` (see [schemas/README.md](schemas/README.md))

### 0.5.6 — before and after
- `ssh-cli schema` catalog: before, 22 documents. After, 25, because `vps-edit.schema.json`, `vps-rename.schema.json` and `vps-import.schema.json` joined
- `ssh-cli vps doctor --json`, object `local`: before, no directory fields. After, three fields, shown here as measured on a fresh config directory

```json
{"local":{"config_dir_exposed":false,"config_dir_permissions":"0700","invalid_host_names":[]}}
```

- `--dry-run` plans of `ssh-cli vps edit`, `ssh-cli vps rename` and `ssh-cli vps remove`: before, no target echo. After, the four target fields, shown here for `ssh-cli vps remove` without its `config_path` field

```json
{"ok":true,"event":"dry-run","operation":"vps-remove","name":"h1","dry_run":true,"executed":false,"target_resolved":"h1","target_source":"argv","host_resolved":"h1","host_source":"argv"}
```

- `ssh-cli vps edit --json`: before, no `updated` list and no schema. After, `updated` is always present and `vps-edit.schema.json` types it
- The 19 names `updated` can hold are `host`, `port`, `user`, `password`, `key_path`, `key_passphrase`, `use_agent`, `agent_socket`, `timeout_ms`, `max_command_chars`, `max_output_chars`, `sudo_password`, `su_password`, `disable_sudo`, `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key` and `tags`
- `ssh-cli vps edit h1 --agent-socket S` on a password host, as measured

```json
{"ok":true,"event":"vps-edited","name":"h1","target_resolved":"h1","target_source":"argv","host_resolved":"h1","host_source":"argv","updated":["use_agent","password","agent_socket"]}
```

- `ssh-cli vps rename` and `ssh-cli vps edit --name`: both emit `vps-renamed` with `name`, `previous_name`, `active_rewritten`, `updated`, `target_resolved` and `target_source`, and `vps-rename.schema.json` types it
- `ssh-cli vps import --json`: before, no list of overwritten hosts. After, `replaced` is a sorted array, always present, and `vps-import.schema.json` types it

```json
{"ok":true,"event":"vps-import","imported":1,"replaced":["h1"]}
```

- `ssh-cli vps export`: before, the body carried no agent fields. After, each host carries `use_agent` and `agent_socket`, and `vps-export.schema.json` declares both
- `ssh-cli vps export -o FILE --json`: before, `vps-export.schema.json` required `schema_version` and `hosts`, so a validator rejected the summary that `-o` prints. After, `oneOf` accepts exactly one of two shapes, the body (`schema_version` and `hosts`) or the summary (`path` and `format`), as measured

```json
{"ok":true,"event":"vps-export","format":"json","include_secrets":false,"path":"rel.json"}
```

- `error-envelope` on stderr: the shape is unchanged, and a strict-name refusal arrives with exit 64 and `error_code` `invalid_argument`, such as `invalid VPS name 'bad;x'` on `ssh-cli vps rename` and `ssh-cli vps edit --name`, while `ssh-cli vps add` leaves the quoted name out of the message
- Two auth flags in one call exit 2 with the clap usage text and no JSON envelope, because the refusal happens at parse time
- JSON never escapes control or bidi characters beyond what `serde_json` does, because JSON is for machines

### After 0.3.4+ host fields
- `timeout_ms`
- `max_command_chars`
- `max_output_chars`
- `key_path`
- `key_passphrase` (masked)
- `disable_sudo`
- `schema_version` 2 (historical writes only, current wire is schema v3)

### At-rest secrets (0.3.6 era, still current)
- Password/sudo/su/passphrase fields may store `sshcli-enc:v2:…` blobs, and legacy `sshcli-enc:v1:…` blobs are still read
- Prefer CLI flags: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- Primary-key sources: CLI `--secrets-key-file` / `--use-keyring`, or XDG `secrets.key`
- `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` are rejected fail-closed (not a store)
- Every command that reads or writes a registry secret exits 64 with `are not supported` when either variable is set: `ssh-cli vps list`, `ssh-cli vps show`, `ssh-cli doctor`, `ssh-cli vps export`, `ssh-cli vps add`, `ssh-cli vps edit` and `ssh-cli secrets status`
- Commands that touch no secret exit 0 with the variable set: `ssh-cli vps path`, `ssh-cli locale show`, `ssh-cli schema` and `ssh-cli commands`

### Masking (0.4.0)
- Empty password → JSON `null`
- Non-empty → the masked string
- Human text show still prints "(not set)" for an empty password

### Transfer / tunnel events (0.4.0 / 0.4.1+)
- SCP success JSON includes required `event: "scp-transfer"` (IO-009)
- Tunnel still emits `event: "tunnel_listening"` after bind
- SCP success: `docs/schemas/scp-transfer.schema.json`
- Tunnel listening: `docs/schemas/tunnel-listening.schema.json`
- Failures in JSON mode: `docs/schemas/error-envelope.schema.json` on stderr


## Compatibility Notes
- Existing TOML hosts load and migrate field defaults on read/save paths
- Legacy alias `--maxChars` maps to command input limit
- Prefer `--max-command-chars` and `--max-output-chars`, because `--max-chars` is itself only a legacy alias of the command limit
- Default timeout is 60000 ms for agent automation
- Always-trust host key behavior is gone in release builds
- Default encryption is on
- Plaintext requires explicit CLI opt-out `--allow-plaintext-secrets` only (env secrets stores are rejected fail-closed)
- Default tracing is error
- INFO prose is not expected on agent stderr
- SCP remains file-only by design in 0.4.0+ (still true in 0.5.6, not a temporary limitation)
- SFTP integrity requires 0.5.3+ (G1)
- Do not rely on pre-0.5.3 SFTP upload without destination checksum proof
- Existing `config.toml` files keep loading in 0.5.6, and a stored host name with spaces, accents, control or bidi characters is accepted, listed by `ssh-cli doctor` and repairable with `ssh-cli vps rename`
- A legacy record with control characters in a tag, `host` or `user` still exits 65, because only the host NAME is tolerant
- A legacy name does not log a warning on load, so `ssh-cli doctor` is the place that lists it
- Parsers that matched raw bytes of text mode output must expect `\u{1b}`-style escapes, while JSON keeps every character raw
- Error bodies stay in English by design, and remote `ssh-cli exec` stdout and stderr stay raw by design
- The project follows Semantic Versioning 2.0.0 on a 0.x line, so a release may carry BREAKING changes, and 0.5.4, 0.5.5 and 0.5.6 each list some
- A deprecation is recorded in the `Deprecated` section of the CHANGELOG entry of the release that introduces it, and the 0.5.6 entry has none
- The repository sets no minimum number of minor releases for a deprecated API, and the 0.5.5 rename of `ssh_cli::telemetry` shipped without an alias
- Exit codes and JSON schemas are not frozen across 0.5.x releases, because 0.5.4 moved the partial multi-host failure from exit 65 to exit 1 and 0.5.6 added three schemas, so read the BREAKING lines before every upgrade
- The global `--select`, `--filter` and `--count-only` do not reach the `ssh-cli doctor` object, so read it with `jaq`
- Windows and macOS are only type-checked by the `cross-targets` gate, and the `0700` mode and the `getuid` check were never executed there


## Rollback
- Reinstall a previous version with an exact version pin if required
- Keep a redacted export via `ssh-cli vps export` before major experiments
- A redacted export restores the host skeleton only, so also keep `config.toml` and `secrets.key` offline
- If rolling back below 0.3.6, encrypted blobs need the matching primary-key or a plaintext re-export while still on 0.3.6+
- If rolling back to 0.3.9, do not expect working SCP wire (upgrade again to 0.4.0+ for transfers)
- If rolling back below 0.5.3, do not trust SFTP upload integrity without external checksum (G1)
- Rolling back from 0.5.6 to 0.5.5 keeps `schema_version` 3 in `config.toml`, but no test covers that rollback, so keep a redacted export first
- 0.5.5 has no `ssh-cli vps rename`, no clear flags and no `--lock-wait-ms`, and it exits 64 when you switch the auth method of a host
- Wrappers written for 0.5.6 find no `vps-renamed`, no `updated` and no `local.config_dir_exposed` on 0.5.5
- A host created with a strict 0.5.6 name loads on older versions, and a legacy name repaired with `ssh-cli vps rename` stays repaired


## Wire format: schema v3: introduced in 0.5.3, current through 0.5.6
- Current `schema_version` for new writes is 3 (not 2)
- New writes use English TOML keys: `name`, `port`, `username`, `password`, `added_at`, …
- Loads still accept legacy Portuguese keys (`nome`, `porta`, `usuario`, `senha`, `adicionado_em`), dual-read EN serialize / PT load aliases
- `added_at` is optional on import (defaults to now when missing)
- `ssh-cli vps export` body follows the resolved format: the agent envelope (`event: "vps-export"`) on any non-TTY stdout, even into a `.toml` filename
- `--output-format text` is the only route to a TOML body
- Auto JSON non-TTY applies to export like everywhere else
- `ssh-cli vps import` accepts TOML (EN + PT aliases) or JSON `vps-export` envelopes
- `--allow-incomplete` for redacted/skeleton hosts
- `--include-secrets` requires `-o`/`--output` or `--i-understand-secrets-on-stdout`
- Secrets control is CLI/XDG only: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, or XDG `secrets.key` (env secrets stores rejected fail-closed)
- Preferred term for the at-rest key is primary-key
- Legacy keyring entries labeled master-key are still readable
- `ssh-cli secrets init --json` → `event: "secrets-init"`
- `ssh-cli secrets reencrypt --json` → `event: "secrets-reencrypt"`
- First secret write may set `secrets_key_auto_created: true` on the same success JSON (one document)
- CRUD success JSON events when JSON is effective: `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import`
- Tunnel `--bind` defaults to `127.0.0.1` (loopback)
- Exit 65 covers `TomlDe` / bad import data
- Exit 77 is auth/host-key/permission
- Missing SCP file is exit 66 with `file not found: <path>`
- Graduated verbosity `-v`/`-vv`/`-vvv` (info/debug/trace), always crate-scoped (G2/G14)
- SFTP upload integrity fixed (G1)
- SETSTAT atime+mtime (G3)
- Fail-closed set_metadata (G4)
- Perm mask (G12)
- Batch cancel cardinality (G5/G17)
- Explicit Target Designation (0.5.5): the single-host `ssh-cli exec`/`ssh-cli sudo-exec`/`ssh-cli su-exec` envelope gained `target_resolved` and `target_source`, bind new consumers to those
- `host_resolved`/`host_source` are read aliases written from the same value, never an independent source of truth
- `active_fallback` is the boolean shortcut for `target_source == "active_marker"`, so a shell pipeline can branch without knowing how the enum is spelled
- The provenance trio is absent, never empty, on the error envelope when the failure predates target resolution: absence says no host was chosen, while `""` would assert one was chosen and is nameless
- A lone positional on the exec family is now a usage error (exit 64)
- Reach the marker `ssh-cli connect` wrote with the explicit `--use-active <COMMAND>`, which exits 66 when no marker exists
- Fleet selectors `--all`, `--hosts` and `--tags` report `target_source: "selector"`
- Transfers never read the marker, so they report `argv` or `selector` only
- `ssh-cli doctor` reports `local.active_vps`, so the marker is auditable without reading the file behind the CLI's back
- Suites: `tests/gaps_v042_integration.rs` + `tests/gaps_v051_integration.rs` + `tests/gaps_v057_sftp.rs` + `tests/gaps_v058_e2e_residual.rs`
- Target designation under `tests/gaps_v065_exec_target_designation.rs` and `tests/gaps_v066_transfer_target_designation.rs`
- Packaging reach under `tests/gaps_v067_packaging_reach.rs`
- Official e2e E01–E21
- Product line: 0.5.6


## See Also
- [HOW_TO_USE.md](HOW_TO_USE.md): end-user command surface
- [AGENTS.md](AGENTS.md): agent contracts and exit routing
- [COOKBOOK.md](COOKBOOK.md): copy-paste recipes
- [schemas/README.md](schemas/README.md): JSON schema index
- [TESTING.md](TESTING.md): test suites and gate battery
- [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md): release gates
- [CROSS_PLATFORM.md](CROSS_PLATFORM.md): target matrix and config directory modes
- [../CHANGELOG.md](../CHANGELOG.md): full release notes


## Release Notes
### 0.5.6 — edit and rename release
- `ssh-cli vps rename <NAME> <NEW_NAME>` and `ssh-cli vps edit <NAME> --name <NEW>` rename natively, so the export/edit/import detour is no longer needed
- The detour was never equivalent, because `ssh-cli vps export` redacts secrets and an import of that skeleton had to be re-filled by hand
- The export also carried no `use_agent`/`agent_socket`, so a host authenticated by agent came back unauthenticated
- Every credential stays readable under the new name: the record seals with an unbound context, so the associated data carries no host name and the rename has nothing to re-seal
- Eight clear flags remove a field instead of overwriting it
- `--dry-run` now covers eight operations
- The config lock waits a bounded `--lock-wait-ms` before exiting 69 instead of hanging forever

### 0.5.5 — target-designation security release
- Fixes GAP-SSH-EXEC-ARGC-001: `ssh-cli exec`/`ssh-cli sudo-exec`/`ssh-cli su-exec` decided the target host by positional count
- So `ssh-cli exec <HOST> --step <CMD>` executed the host name as a remote binary on whatever host `ssh-cli connect` last selected
- BREAKING: a lone positional is now a usage error
- Reach the active marker with the new `--use-active <COMMAND>`
- The single-host envelope gained `target_resolved` and `target_source`, so bind new consumers to those
- `host_resolved` and `host_source` are the 0.5.5 read aliases
- The envelope also gained `active_fallback`, and `ssh-cli doctor` now reports `local.active_vps`

### 0.5.4 — security and agent-native release
- Fixes a remote pre-auth DoS in the SSH banner path (A1)
- Stops server-sent setuid bits landing on downloaded files (A3)
- Closes the world-readable window on ACME/mTLS private keys (A2)
- Adds payload-shaping flags applied before serialization: `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- BREAKING: partial multi-host failure now exits 1 (was 65)
- BREAKING: a non-loopback `--bind` requires `--i-accept-network-exposure`
- New `tunnel_closed` event
