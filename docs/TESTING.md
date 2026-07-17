# Testing Guide

> Run the right ssh-cli test profile without hanging on remote networks.

- Read this document in [Portuguese (pt-BR)](TESTING.pt-BR.md).
- Product line: **0.5.1** (historical residual suite gates include **0.4.1** AUD-POST / `gaps_v041`).


## Why Categorized Tests
- Unit tests protect packing, schema, secrets AEAD, and pure logic without SSH servers.
- Integration tests protect CLI contracts, storage, and snapshots.
- Remote live tests are optional and must always use hard timeouts and never log credentials.
- Install resolve gates protect crates.io onboarding (GAP-014).
- Residual gap suites lock agent I/O, exit codes, supply chain, masking, SCP wire, and doc honesty contracts.


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
- **0.5.1** suite `tests/gaps_v051_integration.rs` (export TOML default, `vps-export` JSON, schema v3 dual-read, secrets-init event, include-secrets guard, CRUD `vps-added`, empty command, import exit 65)
- Storage integration under `tests/storage_integration.rs`
- Snapshot tests under `tests/snapshot_tests.rs`
- SCP surface under `tests/scp_integration.rs`
- Tunnel surface under `tests/tunnel_integration.rs`
- Property tests under `tests/proptest_tests.rs`
- i18n integration under `tests/i18n_integration.rs`
- Install resolve script `scripts/verify_install_resolve.sh`
- English identifier gate `scripts/check_en_identifiers.sh`
- Real SSH E2E (optional, machine-local): `scripts/e2e_real_ssh.sh` — official matrix **E01–E16** (E10–E14 cover SCP upload/download/cmp/missing/preserve)
- Benchmarks under `benches/` (manual)


## How to Run
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
cargo test --locked --test storage_integration
cargo test --locked --test snapshot_tests
cargo test --locked packing
cargo test --locked secrets::
```

### Real SSH E2E (never print secrets)

```bash
# Preferred in CI / shared machines: env only (never commit these values)
export SSH_CLI_E2E_HOST=… SSH_CLI_E2E_USER=… SSH_CLI_E2E_PASSWORD=…
bash scripts/e2e_real_ssh.sh

# Maintainer-local only: parse $HOME/.grok/config.toml (ssh-flowaiper MCP).
# That file must stay under $HOME — never copy it into this repository.
bash scripts/e2e_real_ssh.sh --from-grok-config
```

- Official matrix **E01–E16**; **E10–E14** = SCP upload, download, integrity (`cmp`), missing remote, preserve mode+mtime (SCP-023).
- Script prints only PASS/FAIL labels — never host, user, or password.
- **GAP-014 / fail2ban policy:** prefer local `sshd` or a throwaway VPS. **FORBIDDEN:** auth-failure storms against production hosts (fail2ban bans). Production VPS e2e only with care, IP whitelist / `ignoreip`, and **no** intentional wrong passwords.


## CI Profiles
- This repository currently ships without GitHub Actions workflows.
- Maintainers run the local developer loop before every publish.
- Publish gates include package dry-run, install resolve verification, bilingual docs parity, English identifier check (`bash scripts/check_en_identifiers.sh`), residual suites `gaps_v040` + `gaps_v041` + `gaps_v042` + **`gaps_v051`**, plus the canonical loop: `cargo test --locked --all-targets`, clippy `-D warnings`, and `cargo build --release`.


## Environment Variables
- `SSH_CLI_HOME` isolates config during tests.
- `--config-dir` on CLI invocations is preferred for temporary registries.
- `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` opts out of default encryption for tests that assert plaintext TOML.
- Without that opt-out, first secret write auto-creates `secrets.key` and encrypts fields.
- Default tracing level is error; do not expect INFO prose on stderr by default.
- `RUST_LOG` overrides the default filter when diagnosing failures.
- `-v` enables debug tracing without setting `RUST_LOG`.
- `NO_COLOR=1` stabilizes snapshot-sensitive output when needed.
- Never put live host passwords into env vars that tests print.


## Troubleshooting
- Snapshot drift: review `tests/snapshots/` and update only intentional UI changes (including version strings).
- Crypto resolve failures: re-check pins and rerun the install script without ignoring lock policy.
- Flaky timeout tests: ensure no real remote host is required unless explicitly configured.
- Permission failures: confirm temp dirs are writable and mode assertions match the OS.
- Encrypted fixture surprises: set `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` or provide a test primary-key via env.
- Unexpected quiet stderr: default is error-level tracing; set `RUST_LOG` or `-v` if you need debug lines.
- SCP / AUD-POST / 0.5.1 residual failures: run `cargo test --locked --test gaps_v040_integration`, `cargo test --locked --test gaps_v041_integration`, `cargo test --locked --test gaps_v042_integration`, and `cargo test --locked --test gaps_v051_integration`; read `gaps.md` AUD-SCP, AUD-POST, and 0.5.1 blocks.
