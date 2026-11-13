# Testing Guide


> Run the right ssh-cli test profile without hanging on remote networks.

- Read this document in [Portuguese (pt-BR)](TESTING.pt-BR.md)
- Product line: 0.5.6 (historical residual suite gates include 0.4.1 AUD-POST / `gaps_v041` and 0.5.2 wire / `gaps_v051`)


## Why Categorized Tests
- Unit tests protect packing, schema, secrets authenticated encryption (AEAD), and pure logic without SSH servers
- Integration tests protect command-line interface (CLI) contracts, storage, and snapshots
- Optional `ssh-keygen` fixtures (G-PROC-02) generate real OpenSSH keys for key-path tests
- Missing binary skips those cases: product runtime never spawns it
- Remote live tests are optional and must always use hard timeouts and never log credentials
- Install resolve gates protect crates.io onboarding (GAP-014)
- Residual gap suites lock agent I/O, exit codes, supply chain, masking, Secure Copy (SCP) and SSH File Transfer Protocol (SFTP) wire, and doc honesty contracts
- The audit record is a local maintainer file, gitignored, cargo-excluded and never published, and tests must not assert its FIXED text (G13/G15)
- G6: tests that touch signal/cancel global state (`CANCEL_FLAG`) use `#[serial_test::serial]` (dev-dep `serial_test`) so the suite is deterministic
- Do not remove serial markers from concurrency/signal tests
- G11: baseline must stay green on first run
- Re-running until pass is forbidden as a gate strategy


## Test Categories
- Unit tests inside the `src` modules (includes `secrets` default encryption)
- Unit tests of `ssh-cli vps edit` in `src/vps/edit/apply_tests.rs` use `tempfile::tempdir()`, never a fixed path in the system temp folder, so parallel runs cannot collide
- CLI e2e under `tests/e2e_cli.rs`
- Gap/residual integration under `tests/gaps_v035_integration.rs` (fake secrets only)
- Agent I/O residual suite under `tests/gaps_v037_integration.rs`
- Post-0.3.7 residual suite under `tests/gaps_v038_integration.rs`
- Post-0.3.8 residual suite under `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC/DENY/CHG)
- Post-0.3.9 / 0.4.0 suite `tests/gaps_v040_integration.rs`
- AUD-POST suite `tests/gaps_v041_integration.rs` (EXP-001, TUN-002, CLI-005/006, IO-009, REL-006, DOC-041)
- AUD-E2E suite `tests/gaps_v042_integration.rs` (TUN-003, IO-010, UX-001, REL-007, ENV-001, DOC-042, SCP-024)
- 0.5.2: suite `tests/gaps_v051_integration.rs` (export redaction roundtrip, `vps-export` JSON, schema v3 dual-read, secrets-init event, include-secrets guard, CRUD `vps-added`, empty command, import exit 65)
- Domain types suite `tests/gaps_v053_domain_types.rs`
- Error handling suite `tests/gaps_v054_error_handling.rs`
- Unsafe/FFI residual suite `tests/gaps_v055_unsafe_ffi.rs`
- G-SFTP residual suite `tests/gaps_v057_sftp.rs` (SFTP surface, with destination-effect proof preferred over inventory self-cert)
- G-E2E residual suite `tests/gaps_v058_e2e_residual.rs` (root `ssh-cli schema` / `ssh-cli doctor`, single `vps-added` with `secrets_key_auto_created`, `--use-agent`, help/clap env purge, ambient `RUST_LOG` ignored, ACME exit 64, etc.)
- Storage integration under `tests/storage_integration.rs`
- Snapshot tests under `tests/snapshot_tests.rs`
- SCP surface under `tests/scp_integration.rs`
- Tunnel surface under `tests/tunnel_integration.rs`
- Tunnel mode surface under `tests/gaps_v060_tunnel_modes.rs`: the 0.5.4 modes `--reverse`, `--socks5` and `--remote-socket`, their mutual exclusion, the `mode` wire label (`local` / `reverse` / `socks5` / `streamlocal`), the `--i-accept-network-exposure` guard on both ends, and the `tunnel_closed` event
- Agent-native output surface under `tests/gaps_v059_agent_native.rs`: `--select`, `--filter`, `--limit`, `--count-only` and the truncation report
- Error taxonomy under `tests/gaps_v061_error_taxonomy.rs`: `error_class`, `retryable` and `suggestion` on every envelope
- SCP durability under `tests/gaps_v061_scp_durability.rs`: `mtime_preserved` and `durable` are reported, never assumed
- Component budget under `tests/gaps_v062_component_budget.rs`: the ratchet fails both on growth past a frozen cap and on a stale entry whose file already shrank, so the ledger records repayment as loudly as debt
- i18n reachability under `tests/gaps_v062_i18n_reachability.rs`: every message key is reachable in both languages
- Gate falsification for `scripts/check_en_identifiers.py`, driven by `scripts/check_all_gates.py`: each check must be provably able to fail, and two of them were dead from introduction
- The standing proof now lives in `tests/gaps_v068_source_language.rs`
- E2E harness, locale features and secret stdin under `tests/gaps_v063_e2e_harness.rs`, `tests/gaps_v063_locale_features.rs`, `tests/gaps_v063_secret_stdin.rs`
- Explicit target designation under `tests/gaps_v065_exec_target_designation.rs`: the argv of GAP-SSH-EXEC-ARGC-001 stays rejected, and the provenance trio reaches success and error envelopes alike
- Transfer target designation under `tests/gaps_v066_transfer_target_designation.rs`: the same rule on `scp` and `sftp`
- Packaging reach under `tests/gaps_v067_packaging_reach.rs`: every source file survives `cargo package`
- A `.gitignore` pattern that swallows a source directory is invisible to every local build
- Source language under `tests/gaps_v068_source_language.rs`: code and comments are English, exempting citations rather than whole files
- Markdown table rendering under `tests/gaps_v069_markdown_tables.rs`: a row whose unescaped pipes disagree with its delimiter row is valid Markdown that renders with phantom columns
- Documentation reference reach under `tests/gaps_v070_doc_reference_reach.rs`: every repository path a document names exists on disk, and every suite on disk is catalogued here
- The phantom suite this closed was invisible to the forward sweep
- Edit and rename under `tests/gaps_v071_vps_edit_rename.rs`: the secret stays readable under the new name while the file keeps only ciphertext, the `ssh-cli connect` marker follows only when it named the source, neither verb retargets from the marker, the eight clear flags empty exactly their field and fail closed when they would leave zero auth, `--dry-run` writes nothing on either verb, and the config lock exits 69 instead of hanging
- Edit truthfulness under `tests/gaps_v072_edit_truthfulness.rs`, `ssh-cli vps edit` reports only what changed: a flag whose value already matches leaves `updated` empty (and the key stays in the envelope), a no-op exits 0 and leaves `config.toml` byte-identical, and a rename fails in a fixed order (unknown host 66, identical name 64, taken name 64)
- Import round trip under `tests/gaps_v072_import_roundtrip.rs`: a host exported with its secrets and imported into an empty registry compares equal field by field through JSON and TOML, Transport Layer Security (TLS) fields are validated on import, the `replaced` field is reported, a closed stdout has its own exit code, and a pinned field count fails the moment `VpsRecord` grows without export and import learning the new field
- Lock contention under `tests/gaps_v072_lock_contention.rs`: exit 69 on a held registry lock carries the lock remedy and not the keyring one, proven with a real second process holding `config.toml.lock`
- Schema conformance under `tests/gaps_v072_schema_conformance.rs`: real envelopes printed by the binary validate against the published schemas in `docs/schemas/`, through a small hand-written validator so no new dependency enters the supply chain
- Storage hardening under `tests/gaps_v072_storage_hardening.rs`: the lock and marker files keep their expected modes on disk after a real invocation, asserted on Unix only because Windows has no POSIX mode bits
- Auth method switch under `tests/gaps_v073_vps_auth_switch.rs`, `ssh-cli vps edit` switches a host from one authentication method to another in one call: `--key` clears the password and the agent, `--password` and `--password-stdin` clear the key, its passphrase and the agent, and `--use-agent` clears the password and the key
- The same suite proves that two primary auth flags in one call exit 2 with an empty stdout, a passphrase with no key exits 64 and leaves the file untouched, a different key drops the old passphrase, `--agent-socket` alone switches to the agent, and the `--dry-run` plans echo `target_resolved` and `target_source`
- Host name charset under `tests/gaps_v073_host_name_charset.rs`: a new host name is limited to ASCII letters, digits, dot, hyphen and underscore on `ssh-cli vps add`, `ssh-cli vps rename`, `ssh-cli vps edit --name` and `ssh-cli vps import`
- The same suite proves that `ssh-cli vps import` lists every bad name at once and writes nothing, legacy names with spaces or accents still load, and `ssh-cli vps rename` repairs a legacy name with control characters
- Lookups that become a target (`ssh-cli exec`, `ssh-cli health-check`, `--hosts`, `ssh-cli connect`) refuse control characters with exit 64, and `ssh-cli tls mtls import --name` refuses control, formatting and bidi characters without creating a directory
- Terminal escape under `tests/gaps_v073_terminal_escape.rs`: control and bidi characters are escaped as `\u{1b}`-style text in text mode (`ssh-cli vps list`, `ssh-cli vps show`, not-found errors and lookups) before they reach the terminal, while JSON keeps the original value
- Config directory mode under `tests/gaps_v073_config_dir_mode.rs`: the config directory is created with mode `0700`, an existing default XDG directory is tightened, and any other existing directory is never chmodded while `ssh-cli doctor` reports `config_dir_exposed`
- The same suite proves that a path ending in `.toml` makes the parent the config directory, `ssh-cli vps export` creates missing folders private, two tables that normalize to one name exit 65, and a source scan fails when another module creates the directory
- Flag reach under `tests/gaps_v074_flag_reach.rs`: each documented flag is proven by observable behaviour instead of parser acceptance, and each document that claims the full inventory fails when it loses a flag
- Markdown format gate under `tests/gaps_v075_markdown_format_gate.rs`: `scripts/check_markdown_format.py` stays in the battery, runs against the real tree and imports only the standard library
- Documentation conformance under `tests/docs_conformance.rs`: the 48-leaf command inventory, schema indexing in both languages, bilingual release parity, the eight `--dry-run` operations, directional permission masks and the 7000-word skill budget
- Test quality under `tests/test_quality.rs`
- Property tests under `tests/proptest_tests.rs`
- i18n integration under `tests/i18n_integration.rs`
- Gate battery runner `scripts/check_all_gates.py`: runs all fourteen mandatory gates in one invocation
- The suites above run under `cargo test`, which the battery invokes as its `test` gate
- Advisory freshness gate `scripts/check_advisory_freshness.py`: only reachable through the battery runner
- Advisory hits gate `scripts/check_advisory_hits.py`: runs `cargo audit --json --no-fetch` and fails on any advisory against `Cargo.lock`, because a fresh database and a flat `cargo deny` still never stated which advisories hit the tree
- Gate battery coverage contract `tests/gaps_v064_gate_runner.rs`: every public `scripts/*.py` is a gate or a declared non-gate, and `the_test_gate_stays_locked` fails when the `test` gate loses `--locked` or `--no-fail-fast`
- Install resolve script `scripts/verify_install_resolve.py`
- English identifier gate `scripts/check_en_identifiers.py`
- Real SSH E2E (optional, machine-local): `scripts/e2e_real_ssh.py`: official matrix E01–E21 (E10–E14 cover SCP upload/download/cmp/missing/preserve, E17/E18 cover SFTP checksum + recursive tree, G7, and E19–E21 cover the reduced `vps list --select` item, the `--limit` accounting, and the keyring round-trip)
- Benchmarks under `benches/` (manual)


## How to Run
### Full battery (required before declaring a gate green)
- One invocation runs every mandatory gate and reports every result

```bash
python3 scripts/check_all_gates.py
```

- Add `--json` for NDJSON, `--only ID[,ID...]` for a subset, `--list` for the gate ids
- The fourteen gates are `fmt`, `build-release`, `build-no-default`, `clippy`, `doc`, `test`, `exec-target`, `deny`, `cross-targets`, `advisory-freshness`, `advisory-hits`, `en-identifiers`, `markdown-format` and `install-resolve`
- The `test` gate runs `cargo test --locked --all-features --no-fail-fast`
- `--no-fail-fast` makes a red run show every failing test instead of stopping at the first
- Other flags are `--fail-fast` to stop at the first red gate, `--log-dir DIR` to keep the per-gate logs and `--timeout-secs N` for the per-gate budget (default 3600, `0` for none)
- `--quiet` hides the progress lines on stderr, and the tail of a failed gate still prints
- The stdout has one tab-separated line per gate, with the id, `pass` or `fail`, the exit code and the milliseconds, then a `_summary` line with `total=`, `failed=` and `skipped=`
- The exit is 0 when every selected gate passes, 1 when any gate fails and 2 on a usage error such as an unknown gate id
- The `exec-target` gate runs only `tests/gaps_v065_exec_target_designation.rs`, so a regression of the dangerous exec argv is named in the report
- `cargo clippy` and `cargo test` abort on the first unbuildable target, so a single broken test file hides every gate behind it
- The run always reports how many gates it skipped, so a partial run cannot read as a full one
- The battery is sequential on purpose: the cargo gates contend for one `target/` lock, so running them concurrently buys nothing

### Local developer loop

```bash
cargo fmt --check
cargo test --locked --all-features --no-fail-fast
cargo clippy --all-targets --all-features --locked -- -D warnings
python3 scripts/check_en_identifiers.py
cargo build --release
python3 scripts/verify_install_resolve.py
```

### Focused profiles
- Every suite in `tests/` runs alone with `cargo test --locked --test <suite>`
- The block lists all of them

```bash
cargo test --locked --test docs_conformance
cargo test --locked --test e2e_cli
cargo test --locked --test gaps_v035_integration
cargo test --locked --test gaps_v037_integration
cargo test --locked --test gaps_v038_integration
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo test --locked --test gaps_v042_integration
cargo test --locked --test gaps_v051_integration
cargo test --locked --test gaps_v053_domain_types
cargo test --locked --test gaps_v054_error_handling
cargo test --locked --test gaps_v055_unsafe_ffi
cargo test --locked --test gaps_v057_sftp
cargo test --locked --test gaps_v058_e2e_residual
cargo test --locked --test gaps_v059_agent_native
cargo test --locked --test gaps_v060_tunnel_modes
cargo test --locked --test gaps_v061_error_taxonomy
cargo test --locked --test gaps_v061_scp_durability
cargo test --locked --test gaps_v062_component_budget
cargo test --locked --test gaps_v062_i18n_reachability
cargo test --locked --test gaps_v063_e2e_harness
cargo test --locked --test gaps_v063_locale_features
cargo test --locked --test gaps_v063_secret_stdin
cargo test --locked --test gaps_v064_gate_runner
cargo test --locked --test gaps_v065_exec_target_designation
cargo test --locked --test gaps_v066_transfer_target_designation
cargo test --locked --test gaps_v067_packaging_reach
cargo test --locked --test gaps_v068_source_language
cargo test --locked --test gaps_v069_markdown_tables
cargo test --locked --test gaps_v070_doc_reference_reach
cargo test --locked --test gaps_v071_vps_edit_rename
cargo test --locked --test gaps_v072_edit_truthfulness
cargo test --locked --test gaps_v072_import_roundtrip
cargo test --locked --test gaps_v072_lock_contention
cargo test --locked --test gaps_v072_schema_conformance
cargo test --locked --test gaps_v072_storage_hardening
cargo test --locked --test gaps_v073_config_dir_mode
cargo test --locked --test gaps_v073_host_name_charset
cargo test --locked --test gaps_v073_terminal_escape
cargo test --locked --test gaps_v073_vps_auth_switch
cargo test --locked --test gaps_v074_flag_reach
cargo test --locked --test gaps_v075_markdown_format_gate
cargo test --locked --test i18n_integration
cargo test --locked --test proptest_tests
cargo test --locked --test scp_integration
cargo test --locked --test snapshot_tests
cargo test --locked --test storage_integration
cargo test --locked --test test_quality
cargo test --locked --test tunnel_integration
cargo test --locked --lib
cargo test --locked packing
cargo test --locked secrets::
cargo fmt --check
```

### Documentation profile
- Run these suites after any change to a document, a schema or a skill
- They cover command inventory, schema indexing, Markdown tables, repository paths, suite catalog, packaging reach, flag reach, Markdown format and the gate runner

```bash
cargo test --locked --test docs_conformance --test gaps_v070_doc_reference_reach --test gaps_v072_schema_conformance --test gaps_v039_integration --test gaps_v064_gate_runner --test gaps_v067_packaging_reach --test gaps_v069_markdown_tables --test gaps_v068_source_language --test gaps_v074_flag_reach --test gaps_v075_markdown_format_gate
```

### Real SSH E2E (never print secrets) — G-E2E-05

```bash
# Preferred: unprivileged sshd on loopback, nothing to configure (Unix only)
python3 scripts/e2e_real_ssh.py --local-sshd

# Lab host: flags only, no environment variables; the password comes from a file or stdin
python3 scripts/e2e_real_ssh.py --host … --port 22 --user … --password-file /path/to/pw-file --sudo-password-file /path/to/sudo-pw-file

# Maintainer-local only: parse $HOME/.grok/config.toml
# That file must stay under $HOME: never copy it into this repository. Needs Python 3.11+.
python3 scripts/e2e_real_ssh.py --from-grok-config
```

- Default binary: `target/release/ssh-cli` (override with `--bin`)
- On Windows use `py -3 scripts/e2e_real_ssh.py ...`
- `--local-sshd` is Unix-only
- `--from-grok-config [PATH]` is optional and maintainer-only, so other contributors use `--local-sshd` or the lab flags
- Without a lab host / credentials, the script exits 0 with SKIP, which is offline-safe and not a red gate
- Official matrix E01–E21
- E10–E14 = SCP upload, download, integrity (`cmp`), missing remote, preserve mode+mtime (SCP-023)
- E17 = SFTP upload/download checksum
- E18 = SFTP recursive tree (G7)
- E19 = `vps list --select` returns a reduced item that satisfies `vps-list.schema.json` (D6)
- E20 = `vps list --select --limit` drops records, reports the count on stderr, and `--quiet` silences it (D7)
- E21 = `secrets init --keyring` round-trips through the OS store across two processes, and skips with SOFT when no store answers (Gap 50)
- Script prints only PASS/FAIL/SKIP labels: never host, user, or password
- Residual gate suite: `cargo test --locked --test gaps_v058_e2e_residual`
- GAP-014 / fail2ban policy: prefer local `sshd` or a throwaway VPS
- FORBIDDEN: auth-failure storms against production hosts (fail2ban bans)
- Production VPS e2e only with care, IP whitelist / `ignoreip`, and no intentional wrong passwords


## Publish Gates
- Maintainers run the local developer loop before every publish
- Publish gates include package dry-run, install resolve verification, bilingual docs parity and the English identifier check (`python3 scripts/check_en_identifiers.py`)
- They also include the residual suites `gaps_v040`, `gaps_v041`, `gaps_v042`, `gaps_v051`, `gaps_v057` and `gaps_v058`, plus `cargo fmt --check` (G10)
- The canonical loop is `cargo test --locked --all-features --no-fail-fast`, clippy `--all-targets --all-features -- -D warnings` and `cargo build --release`
- `python3 scripts/check_all_gates.py` runs all of them, plus the cross-target type-check, in one invocation


## Environment Variables
- Use `--config-dir` on CLI invocations to isolate config during tests (product does not read `SSH_CLI_HOME`)
- `--allow-plaintext-secrets` opts out of default encryption for tests that assert plaintext TOML
- Without that opt-out, first secret write auto-creates `secrets.key` and encrypts fields
- Default tracing level is error
- Do not expect INFO prose on stderr by default
- Ambient `RUST_LOG` is ignored
- Use `-v`/`-vv`/`-vvv` when diagnosing failures (G2/G14 crate-scoped)
- `-v` → info, `-vv` → debug, `-vvv` → trace (CLI-only log filter)
- Use `--no-color` when snapshot-sensitive output needs stable text, because the standard color variables only drive terminal detection
- The product ignores `SSH_CLI_HOME`, `SSH_CLI_LANG`, `SSH_CLI_FORCE_TEXT` and `SSH_CLI_USE_KEYRING` as configuration, and it rejects `SSH_CLI_SECRETS_KEY` and `SSH_CLI_SECRETS_KEY_FILE` fail-closed
- Every command that reads or writes a registry secret exits 64 when either secrets variable is set, while `ssh-cli vps path`, `ssh-cli locale show`, `ssh-cli schema` and `ssh-cli commands` exit 0
- Never put live host passwords into env vars that tests print


## Troubleshooting
- Snapshot drift: review `tests/snapshots/` and update only intentional UI changes (including version strings)
- Crypto resolve failures: re-check pins and rerun the install script without ignoring lock policy
- Flaky timeout tests: ensure no real remote host is required unless explicitly configured
- Permission failures: confirm temp dirs are writable and mode assertions match the OS
- Encrypted fixture surprises: pass `--allow-plaintext-secrets` or provide a test primary-key via `--secrets-key-file` / XDG `secrets.key`
- Unexpected quiet stderr: default is error-level tracing
- Pass `-v`/`-vv`/`-vvv` if you need more lines (ambient `RUST_LOG` is ignored)
- Intermittent cancel/signal failures: confirm `#[serial_test::serial]` remains on tests that touch global cancel state (G6)
- Do not parallelize those cases
- Baseline red without code change: fix the suite, re-run lottery is not a gate (G11)
- Publish gates include `cargo fmt --check` (G10)
- SCP / AUD-POST / 0.5.2 / SFTP / G-E2E residual failures: run `cargo test --locked --test gaps_v040_integration`, `gaps_v041_integration`, `gaps_v042_integration`, `gaps_v051_integration`, `gaps_v057_sftp`, and `gaps_v058_e2e_residual`
- The local audit record may hold maintainer notes but is not a published gate artifact and tests must not assert its text (G13/G15)
- A pending `*.snap.new` file fails `no_pending_snapshot_files_exist`: accept the snapshot or delete the file
- A red documentation suite after a document edit: run the documentation profile above and read which path, schema or table row it names
- Never leave `__pycache__` or `.pyc` files in the tree: run Python with `python3 -B`


## Release Notes
### 0.5.6 — formatting gate
- `scripts/check_markdown_format.py` makes the formatting contract executable, closing what Gaps 25 and 37 named as their acceptance criterion while no such linter existed
- The battery runs fourteen gates, and `markdown-format` and `doc` are the two newest
- Thirteen rules cover bold, italic, horizontal rules, star bullets, trailing period or comma on a bullet, H4 and deeper, H1 count, H1 and H2 spacing, the header-to-bullet gap, em dashes outside an H3, decorative blockquotes and emoji
- `--selftest` carries one fixture per rule, so a regex that matches nothing cannot report a clean tree
- `docs_rules/` and `docs_prd/` stay out of scope as imported source material, declared rather than silently skipped
- `tests/gaps_v075_markdown_format_gate.rs` requires the gate to stay in the battery, to execute against the real tree over at least thirty documents, and to import nothing outside the standard library

### 0.5.6 — keyring and contract
- `secrets init --keyring` writes to the operating system store through `keyring` 4.2.0 with the `v1` feature, and `Entry::store_status()` makes an unavailable store exit 69 instead of answering success
- The store entry is scoped by a digest of the resolved config directory, so `--config-dir` profiles no longer share one machine-wide key
- `--use-keyring` also reaches the init path, and the two flags can no longer disagree about the store
- The minimum Rust version is 1.89, required by russh 0.65
- `tests/gaps_v074_flag_reach.rs` proves each of the eight flags by observable behaviour instead of parser acceptance, and fails per document when one loses a flag from its published inventory
- The E2E matrix is E01–E21: E19 and E20 cover the reduced `vps list` payload and E21 the keyring round-trip
- E21 rotates a live profile into the store with `--force`, so it executes the transition an operator actually makes and fails when a leftover `secrets.key` would shadow the new key
- E20 provisions its own second registry entry, because a second host is what the case needs and `vps list` never dials, so a SOFT skip there was a named coverage hole, not a passing run
- `secrets_init_refuses_to_fork_the_primary_key` covers the defect E21 found: a second `secrets init` on a live profile must exit 64 and leave the key untouched
- `docs/schemas/vps-list.schema.json` declares both item shapes, so a `--select` item validates
- `deny.toml` carries three `unmaintained` ignores with written reasons, and the suite asserts the ignore set is exactly those three

### 0.5.6 — audit closure
- `--select` and `--truncate-content` reduce a single record, and a collection-only flag on one record is named under `agent_shape.ignored` even when it is the only flag passed, covered by `src/agent_shape_tests.rs`, `tests/gaps_v059_agent_native.rs` and `tests/gaps_v072_schema_conformance.rs` over both published shapes
- A reduced bare array writes `{"event":"agent-shape",...}` to stderr, covered by `tests/gaps_v059_agent_native.rs`
- `--quiet` suppresses that stderr line like every other one
- `tests/gaps_v074_flag_reach.rs` asserts behaviour for the 18 flags that had none and checks the documents that claim the full inventory
- `tests/docs_conformance.rs` requires an `Examples:` block on all 48 leaves and runs every published argv with `--help` appended
- `scripts/check_advisory_hits.py` fails the battery when an advisory reaches the lockfile, and the `clippy` gate now carries `-W clippy::doc_markdown`
- The battery runs fourteen gates
- The pure-Rust crypto migration was tried and reverted, so the TLS provider stays `aws_lc_rs` and the suites written for that migration were removed with it

### 0.5.6 — edit and rename
- `ssh-cli vps rename <NAME> <NEW_NAME>` and `ssh-cli vps edit <NAME> --name <NEW>` change a host name natively, with the credentials untouched
- The record seals each secret with an unbound context, so the associated data carries no host name and the secret stays readable under the new name
- Every save re-seals with a fresh nonce, which is normal and never touches the plaintext
- This replaces the export/edit/import detour, which silently dropped agent authentication because the export carried no agent fields
- Eight clear flags remove a field instead of overwriting it
- `--dry-run` covers both verbs
- The config lock waits a bounded `--lock-wait-ms` and then exits 69 instead of hanging
- `ssh-cli commands --json` publishes every argument

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
