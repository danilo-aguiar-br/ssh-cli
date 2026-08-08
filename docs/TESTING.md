# Testing Guide

> **0.5.4** — security and agent-native release. Fixes a remote pre-auth DoS in the SSH banner path (A1), stops server-sent setuid bits landing on downloaded files (A3), closes the world-readable window on ACME/mTLS private keys (A2), and adds payload-shaping flags (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) applied before serialization. BREAKING: partial multi-host failure now exits **1** (was 65); a non-loopback `--bind` requires `--i-accept-network-exposure`. New `tunnel_closed` event.


> Run the right ssh-cli test profile without hanging on remote networks.

- Read this document in [Portuguese (pt-BR)](TESTING.pt-BR.md).
- Product line: **0.5.3** (historical residual suite gates include **0.4.1** AUD-POST / `gaps_v041` and **0.5.2** wire / `gaps_v051`).


## Why Categorized Tests
- Unit tests protect packing, schema, secrets AEAD, and pure logic without SSH servers.
- Integration tests protect CLI contracts, storage, and snapshots.
- Optional `ssh-keygen` fixtures (G-PROC-02) generate real OpenSSH keys for key-path
  tests; missing binary skips those cases — product runtime never spawns it.
- Remote live tests are optional and must always use hard timeouts and never log credentials.
- Install resolve gates protect crates.io onboarding (GAP-014).
- Residual gap suites lock agent I/O, exit codes, supply chain, masking, SCP/SFTP wire, and doc honesty contracts.
- Local `gaps.md` is a **gitignored** maintainer audit file (also cargo-excluded) — tests must **not** assert its FIXED text (G13/G15).
- **G6:** tests that touch signal/cancel global state (`CANCEL_FLAG`) use `#[serial_test::serial]` (dev-dep `serial_test`) so the suite is deterministic; do **not** remove serial markers from concurrency/signal tests.
- **G11:** baseline must stay green on first run; re-running until pass is **forbidden** as a gate strategy.


## Test Categories
- Unit tests inside `src/**` modules (includes `secrets` default encryption)
- CLI e2e under `tests/e2e_cli.rs`
- Gap/residual integration under `tests/gaps_v035_integration.rs` (fake secrets only)
- Agent I/O residual suite under `tests/gaps_v037_integration.rs`
- Post-0.3.7 residual suite under `tests/gaps_v038_integration.rs`
- Post-0.3.8 residual suite under `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC/DENY/CHG)
- Post-0.3.9 / **0.4.0** suite `tests/gaps_v040_integration.rs`
- AUD-POST suite `tests/gaps_v041_integration.rs` (EXP-001, TUN-002, CLI-005/006, IO-009, REL-006, DOC-041)
- AUD-E2E suite `tests/gaps_v042_integration.rs` (TUN-003, IO-010, UX-001, REL-007, ENV-001, DOC-042, SCP-024)
- **0.5.2** suite `tests/gaps_v051_integration.rs` (export redaction roundtrip, `vps-export` JSON, schema v3 dual-read, secrets-init event, include-secrets guard, CRUD `vps-added`, empty command, import exit 65)
- G-TLS residual suite `tests/gaps_v052_tls_policy.rs`
- Domain types suite `tests/gaps_v053_domain_types.rs`
- Error handling suite `tests/gaps_v054_error_handling.rs`
- Unsafe/FFI residual suite `tests/gaps_v055_unsafe_ffi.rs`
- G-SSH residual suite `tests/gaps_v056_ssh.rs`
- G-SFTP residual suite `tests/gaps_v057_sftp.rs` (SFTP surface; destination-effect proof preferred over inventory self-cert)
- G-E2E residual suite `tests/gaps_v058_e2e_residual.rs` (root `schema` / `doctor`, single `vps-added` with `secrets_key_auto_created`, `--use-agent`, help/clap env purge, ambient `RUST_LOG` ignored, export FIXED_MASK `***`, ACME exit 64, etc.)
- Storage integration under `tests/storage_integration.rs`
- Snapshot tests under `tests/snapshot_tests.rs`
- SCP surface under `tests/scp_integration.rs`
- Tunnel surface under `tests/tunnel_integration.rs`
- Tunnel mode surface under `tests/gaps_v060_tunnel_modes.rs` — the 0.5.4 modes `--reverse`, `--socks5` and `--remote-socket`, their mutual exclusion, the `mode` wire label (`local` / `reverse` / `socks5` / `streamlocal`), the `--i-accept-network-exposure` guard on both ends, and the `tunnel_closed` event
- Property tests under `tests/proptest_tests.rs`
- i18n integration under `tests/i18n_integration.rs`
- Gate battery runner `scripts/check_all_gates.sh` — runs all ten mandatory gates in one invocation
- Advisory freshness gate `scripts/check_advisory_freshness.sh` — only reachable through the battery runner
- Gate battery coverage contract `tests/gaps_v064_gate_runner.rs`
- Install resolve script `scripts/verify_install_resolve.sh`
- English identifier gate `scripts/check_en_identifiers.sh`
- Real SSH E2E (optional, machine-local): `scripts/e2e_real_ssh.sh` — official matrix **E01–E18** (E10–E14 cover SCP upload/download/cmp/missing/preserve; **E17/E18** cover SFTP checksum + recursive tree — G7)
- Benchmarks under `benches/` (manual)


## How to Run
### Full battery (required before declaring a gate green)
- One invocation runs every mandatory gate and reports every result.

```bash
bash scripts/check_all_gates.sh
```

- Add `--json` for NDJSON, `--only ID[,ID...]` for a subset, `--list` for the gate ids.
- `cargo clippy` and `cargo test` abort on the first unbuildable target, so a single broken test file hides every gate behind it.
- The run always reports how many gates it skipped, so a partial run cannot read as a full one.
- The battery is sequential on purpose: the cargo gates contend for one `target/` lock, so running them concurrently buys nothing.

### Local developer loop

```bash
cargo test --locked --all-targets
cargo clippy --all-targets --locked -- -D warnings
bash scripts/check_en_identifiers.sh
cargo build --release
bash scripts/verify_install_resolve.sh
```

### Focused profiles

```bash
cargo test --locked --test e2e_cli
cargo test --locked --test gaps_v035_integration
cargo test --locked --test gaps_v037_integration
cargo test --locked --test gaps_v038_integration
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo test --locked --test gaps_v042_integration
cargo test --locked --test gaps_v051_integration
cargo test --locked --test gaps_v052_tls_policy
cargo test --locked --test gaps_v056_ssh
cargo test --locked --test gaps_v057_sftp
cargo test --locked --test gaps_v058_e2e_residual
cargo test --locked --test storage_integration
cargo test --locked --test snapshot_tests
cargo test --locked packing
cargo test --locked secrets::
cargo fmt --check
```

### Real SSH E2E (never print secrets) — G-E2E-05

```bash
# Preferred (XDG / CLI first): isolated config-dir with hosts already registered
ssh-cli --config-dir /tmp/ssh-cli-e2e-lab vps add --name e2e --host … --user … --password-stdin
bash scripts/e2e_real_ssh.sh --config-dir /tmp/ssh-cli-e2e-lab

# Harness-only env (NOT product runtime store) — never commit these values
export SSH_CLI_E2E_HOST=… SSH_CLI_E2E_USER=… SSH_CLI_E2E_PASSWORD=…
bash scripts/e2e_real_ssh.sh

# Maintainer-local only: parse $HOME/.grok/config.toml
# That file must stay under $HOME — never copy it into this repository.
bash scripts/e2e_real_ssh.sh --from-grok-config
```

- Default binary: `target/release/ssh-cli` (override with harness `SSH_CLI_E2E_BIN` only).
- Without a lab host / credentials, the script exits **0** with **SKIP** (offline-safe; do not treat SKIP as red gate).
- Official matrix **E01–E18**; **E10–E14** = SCP upload, download, integrity (`cmp`), missing remote, preserve mode+mtime (SCP-023); **E17** = SFTP upload/download checksum; **E18** = SFTP recursive tree (G7).
- Script prints only PASS/FAIL/SKIP labels — never host, user, or password.
- Residual gate suite: `cargo test --locked --test gaps_v058_e2e_residual`.
- **GAP-014 / fail2ban policy:** prefer local `sshd` or a throwaway VPS. **FORBIDDEN:** auth-failure storms against production hosts (fail2ban bans). Production VPS e2e only with care, IP whitelist / `ignoreip`, and **no** intentional wrong passwords.


## CI Profiles
- This repository currently ships without GitHub Actions workflows.
- Maintainers run the local developer loop before every publish.
- Publish gates include package dry-run, install resolve verification, bilingual docs parity, English identifier check (`bash scripts/check_en_identifiers.sh`), residual suites `gaps_v040` + `gaps_v041` + `gaps_v042` + **`gaps_v051`** + **`gaps_v056`** + **`gaps_v057`** + **`gaps_v058`**, `cargo fmt --check` (G10), plus the canonical loop: `cargo test --locked --all-targets`, clippy `-D warnings`, and `cargo build --release`.


## Environment Variables
- Use `--config-dir` on CLI invocations to isolate config during tests (product does not read `SSH_CLI_HOME`).
- `--allow-plaintext-secrets` opts out of default encryption for tests that assert plaintext TOML.
- Without that opt-out, first secret write auto-creates `secrets.key` and encrypts fields.
- Default tracing level is error; do not expect INFO prose on stderr by default.
- Ambient `RUST_LOG` is ignored; use `-v`/`-vv`/`-vvv` when diagnosing failures (G2/G14 crate-scoped).
- `-v` → info, `-vv` → debug, `-vvv` → trace (CLI-only log filter).
- `NO_COLOR=1` stabilizes snapshot-sensitive output when needed.
- Never put live host passwords into env vars that tests print.


## Troubleshooting
- Snapshot drift: review `tests/snapshots/` and update only intentional UI changes (including version strings).
- Crypto resolve failures: re-check pins and rerun the install script without ignoring lock policy.
- Flaky timeout tests: ensure no real remote host is required unless explicitly configured.
- Permission failures: confirm temp dirs are writable and mode assertions match the OS.
- Encrypted fixture surprises: pass `--allow-plaintext-secrets` or provide a test primary-key via `--secrets-key-file` / XDG `secrets.key`.
- Unexpected quiet stderr: default is error-level tracing; pass `-v`/`-vv`/`-vvv` if you need more lines (ambient `RUST_LOG` is ignored).
- Intermittent cancel/signal failures: confirm `#[serial_test::serial]` remains on tests that touch global cancel state (G6); do not parallelize those cases.
- Baseline red without code change: fix the suite — re-run lottery is not a gate (G11). Publish gates include `cargo fmt --check` (G10).
- SCP / AUD-POST / 0.5.2 / SFTP / G-E2E residual failures: run `cargo test --locked --test gaps_v040_integration`, `gaps_v041_integration`, `gaps_v042_integration`, `gaps_v051_integration`, `gaps_v056_ssh`, `gaps_v057_sftp`, and `gaps_v058_e2e_residual`. Local `gaps.md` may hold maintainer notes but is **not** a published gate artifact and tests must not assert its text (G13/G15).
