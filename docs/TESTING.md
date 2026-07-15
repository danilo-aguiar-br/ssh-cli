# Testing Guide

> Run the right ssh-cli test profile without hanging on remote networks.

- Read this document in [Portuguese (pt-BR)](TESTING.pt-BR.md).
- Product line: **0.3.6**.


## Why Categorized Tests
- Unit tests protect packing, schema, secrets AEAD, and pure logic without SSH servers.
- Integration tests protect CLI contracts, storage, and snapshots.
- Remote live tests are optional and must always use hard timeouts and never log credentials.
- Install resolve gates protect crates.io onboarding (GAP-014).


## Test Categories
- Unit tests inside `src/**` modules (includes `secrets` default encryption)
- CLI e2e under `tests/e2e_cli.rs`
- Gap/residual integration under `tests/gaps_v035_integration.rs` (fake secrets only)
- Storage integration under `tests/storage_integration.rs`
- Snapshot tests under `tests/snapshot_tests.rs`
- SCP and tunnel integration tests under `tests/`
- Property tests under `tests/proptest_tests.rs`
- i18n integration under `tests/i18n_integration.rs`
- Install resolve script `scripts/verify_install_resolve.sh`
- Real SSH E2E (optional, machine-local): `scripts/e2e_real_ssh.sh`
- Benchmarks under `benches/` (manual)


## How to Run
### Local developer loop

```bash
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
bash scripts/verify_install_resolve.sh
```

### Focused profiles

```bash
cargo test --locked --test e2e_cli
cargo test --locked --test gaps_v035_integration
cargo test --locked --test storage_integration
cargo test --locked --test snapshot_tests
cargo test --locked packing
cargo test --locked secrets::
```

### Real SSH E2E (never print secrets)

```bash
bash scripts/e2e_real_ssh.sh --from-grok-config
# or export SSH_CLI_E2E_HOST/USER/PASSWORD (and optional SUDO) then run without --from-grok-config
```


## CI Profiles
- This repository currently ships without GitHub Actions workflows.
- Maintainers run the local developer loop before every publish.
- Publish gates include package dry-run, install resolve verification, and bilingual docs parity.


## Environment Variables
- `SSH_CLI_HOME` isolates config during tests.
- `--config-dir` on CLI invocations is preferred for temporary registries.
- `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` opts out of default encryption for tests that assert plaintext TOML.
- Without that opt-out, first secret write auto-creates `secrets.key` and encrypts fields.
- `RUST_LOG` enables debug tracing on stderr when diagnosing failures.
- `NO_COLOR=1` stabilizes snapshot-sensitive output when needed.
- Never put live host passwords into env vars that tests print.


## Troubleshooting
- Snapshot drift: review `tests/snapshots/` and update only intentional UI changes (including version strings).
- Crypto resolve failures: re-check pins and rerun the install script without ignoring lock policy.
- Flaky timeout tests: ensure no real remote host is required unless explicitly configured.
- Permission failures: confirm temp dirs are writable and mode assertions match the OS.
- Encrypted fixture surprises: set `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` or provide a test master-key via env.
