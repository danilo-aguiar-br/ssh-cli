# Contributing to ssh-cli


- Read this document in [Portuguese (pt-BR)](CONTRIBUTING.pt-BR.md)


## Welcome
- Thank you for contributing code, docs, tests, or bug reports
- Every improvement strengthens one-shot multi-host SSH for AI agents
- This guide targets onboarding under 10 minutes from clone to first test


## Quick Start
- Clone the repository and enter the workspace root
- Validate a clean tree with the commands below

```bash
timeout 3600 python3 scripts/check_all_gates.py
```

- That single command runs the whole mandatory battery
- The individual gates below are for focused reruns
- The scripts need Python 3.9+ and only its standard library
- On Windows type `py -3` where this guide says `python3`, and tune budgets with flags such as `--timeout-secs`, never with environment variables

```bash
timeout 120 cargo check --all-targets --locked
timeout 300 cargo test --locked --all-features --no-fail-fast
timeout 60 python3 scripts/verify_install_resolve.py
timeout 900 python3 scripts/check_cross_targets.py
```

### Full gate battery (required before declaring a round closed)
- Run `scripts/check_all_gates.py` before claiming any gate is green
- It runs all fourteen mandatory gates in one invocation and reports every result
- The fourteen are `fmt`, `build-release`, `build-no-default`, `clippy`, `doc`, `test`, `exec-target`, `deny`, `cross-targets`, `advisory-freshness`, `advisory-hits`, `en-identifiers`, `markdown-format` and `install-resolve`
- The `test` gate runs `cargo test --locked --all-features --no-fail-fast`
- `--no-fail-fast` makes a red run show every failing test instead of stopping at the first
- `scripts/check_advisory_freshness.py` has no other caller, so skipping the battery skips that gate entirely
- The reason it exists is structural, not stylistic
- `cargo clippy` and `cargo test` both abort on the first unbuildable target
- One broken test file therefore hides the state of every gate behind it
- That is exactly how the local inventory came to declare 835 green while four gates were red
- Use `--only ID[,ID...]` for a subset and `--list` to see the ids
- The run reports what it skipped, so a partial run can never read as a full one
- The battery is sequential by design because the cargo gates share one `target/` lock

### Cross-target gate (B1 — required)
- Run `scripts/check_cross_targets.py` after touching anything under `#[cfg(target_os = ...)]`
- Every other gate runs for the host triple only
- Code behind a foreign `cfg` is discarded before type-check, so it never reaches `fmt`, `clippy`, `test` or `deny`
- A green board therefore proves nothing about Windows or macOS
- This is not hypothetical: the Windows target failed with six errors while all other gates were green
- The script type-checks `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` and `x86_64-apple-darwin`
- Windows is checked with `--no-default-features` because the default TLS stack pulls `aws-lc-sys`, which compiles C
- That exception is structural (A8) and still type-checks 100% of the product's own `cfg(windows)` code
- Do not claim a target in `docs/CROSS_PLATFORM.md` that this script does not check


## Code of Conduct
- This project follows the Contributor Covenant 2.1 in [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- By taking part you agree to uphold it
- Report unacceptable behavior privately through the channel described in that file


## Development Setup
### Toolchain requirements
- The MSRV (Minimum Supported Rust Version) is Rust 1.89, declared as `rust-version` in `Cargo.toml`
- Install Rust via `rustup` and prefer the pinned toolchain file when present
- Keep `Cargo.lock` committed because this crate ships a binary CLI (command-line interface)

### MSRV policy
- Never bump the MSRV without an explicit issue discussion
- Record every MSRV bump under `Changed` in both CHANGELOG languages
- Do not use a language or standard library feature newer than the MSRV
- The `cross-targets` gate keeps the Windows and macOS code type-checking under the same toolchain rules

### Dependency pinning
- Product line 0.5.6 uses russh 0.65.1 (russh since 0.3.8) without the older COMPAT RC crypto pins
- Do not reintroduce dead RC pins without an issue
- Never run blind `cargo update` on the crypto graph
- Run `scripts/verify_install_resolve.py` after any dependency change

### Local audit inventory
- The local audit record (gitignored, not published) records the gaps closed in 0.5.6
- Each gap lists problem, consequences, root cause, solution, benefits and proof
- Do not assert FIXED status by grepping its prose (G13/G15)
- FIXED requires proof of the effect at the destination, such as checksums, a test name or a measured command


## Branching Strategy
- Keep `main` as the integration branch
- Use `feature/<short-kebab>` for features
- Use `fix/<short-kebab>` for bug fixes
- Use `docs/<short-kebab>` for documentation-only work
- Use `chore/<short-kebab>` for tooling and maintenance


## Commit Convention
- Follow Conventional Commits 1.0.0 on shared branches
- Use `feat` for user-visible features
- Use `fix` for bug fixes
- Use `docs` for documentation-only changes
- Use `test` for test-only changes
- Use `chore` for maintenance
- Never add `Co-authored-by` lines for AI agents


## Pull Request Process
- Step 1: Open a PR with a clear problem statement and the validation commands you ran
- Step 2: Run `python3 scripts/check_all_gates.py` and request review only after all fourteen gates are green
- Step 3: Add a regression test for every behavior change, driven through the real binary when the contract is the CLI
- Step 4: Include bilingual docs when public documents change
- Step 5: Add a CHANGELOG entry under `[Unreleased]` in both languages, with the same number of `BREAKING` entries in each
- Step 6: Keep CLI one-shot behavior intact in every product command
- Step 7: Do not introduce long-lived daemon packaging, telemetry or environment-variable configuration
- Step 8: Keep secrets, real host inventories and master keys out of the diff


## Testing
- Read [docs/TESTING.md](docs/TESTING.md) for categories, profiles and the complete suite catalog
- Prefer deterministic unit tests for packing and schema migration
- Use integration tests under `tests/` for CLI contracts
- Include gap regression suites when touching residual audit surface
- Name each suite explicitly rather than as a range, because an elided list (`v038 … v051`) does not satisfy a `contains` check and silently drops the suites in the middle
- Documentation conformance lives in `tests/docs_conformance.rs`, separate from behavioural suites: editing prose must never turn the build gate red without a functional regression (G-QA-R03)
- Local gates required before a PR: `cargo fmt --check` and `cargo test --locked --all-features --no-fail-fast`, plus clippy as in the release process
- Real-SSH E2E is optional when no lab host is available
- For local real-SSH E2E (G-E2E-05) prefer `python3 scripts/e2e_real_ssh.py --local-sshd` (Unix only)
- Otherwise use `--host`/`--port`/`--user`/`--password-file` against a lab host
- A maintainer may use `--from-grok-config`, which reads `$HOME/.grok/config.toml` only (Python 3.11+)
- The script takes flags only, no environment variables (on Windows run it with `py -3`)
- Without a lab host the script exits 0 with SKIP (offline-safe)
- Default binary is `target/release/ssh-cli`
- Official matrix E01–E21 (E10–E14 SCP, E15 tunnel port 0, E16 symlink, E17/E18 SFTP checksum, E19/E20 payload shaping, E21 keyring rotation of a live profile)
- Prefer local sshd / lab hosts
- No intentional auth storm on production fail2ban targets
- Never log credentials
- Never commit the Grok config or host inventories into this repo
- Unit/integration tests that need plaintext secrets must pass `--allow-plaintext-secrets` (CLI flag, not an env product store)
- Never leave flaky remote-dependent tests without timeouts

### Regression suites
- `tests/gaps_v035_integration.rs`, `tests/gaps_v037_integration.rs`, `tests/gaps_v038_integration.rs` and `tests/gaps_v039_integration.rs`: the earlier integration suites
- `tests/gaps_v040_integration.rs`: SCP and tunnel behavioural contracts
- `tests/gaps_v041_integration.rs` and `tests/gaps_v042_integration.rs`: the 0.4.1 and 0.4.2 closures
- `tests/gaps_v051_integration.rs`: export, import, wire and secrets
- `tests/gaps_v053_domain_types.rs`, `tests/gaps_v054_error_handling.rs`, `tests/gaps_v055_unsafe_ffi.rs` and `tests/gaps_v057_sftp.rs`: the 0.5.x domain, error, unsafe and SFTP gates
- `tests/gaps_v058_e2e_residual.rs`: G-E2E residual (ACME permanent, single `vps-added`, root `schema`/`doctor`, clap no env, graduated `-v`/`-vv`/`-vvv`, FIXED_MASK, `--use-agent`)
- `tests/gaps_v059_agent_native.rs`: the 0.5.4 agent-native surface (`--no-input` refusal on `vps add`/`vps edit`, payload shaping with `--select`/`--filter`/`--limit`/`--count-only`, non-loopback `--bind` guard)
- `tests/gaps_v060_tunnel_modes.rs`: LOTE E, the `--reverse`/`--socks5`/`--remote-socket` argument contracts and the `--dry-run` preview, both driven through the real binary
- `tests/gaps_v061_error_taxonomy.rs` and `tests/gaps_v061_scp_durability.rs`: exit codes 69 and 70 with the envelope contract, and SCP durability reporting
- `tests/gaps_v062_component_budget.rs` and `tests/gaps_v062_i18n_reachability.rs`: file-size and lint budgets and message reachability
- `tests/gaps_v063_e2e_harness.rs`, `tests/gaps_v063_locale_features.rs` and `tests/gaps_v063_secret_stdin.rs`: the runnable E01–E21 matrix, locale Cargo features and the secret-input surface
- `tests/gaps_v064_gate_runner.rs`: gate battery coverage contract, where every `scripts/check_*.py` is either a gate or a declared exclusion
- `tests/gaps_v065_exec_target_designation.rs` and `tests/gaps_v066_transfer_target_designation.rs`: explicit target designation for `exec` and for `scp` and `sftp`
- `tests/gaps_v067_packaging_reach.rs`, `tests/gaps_v068_source_language.rs` and `tests/gaps_v069_markdown_tables.rs`: `cargo package` reach, English source tree and Markdown table column counts
- `tests/gaps_v070_doc_reference_reach.rs` and `tests/gaps_v071_vps_edit_rename.rs`: repository paths named by documents, and `vps rename`, `vps edit --name`, the clear flags and the bounded lock
- `tests/gaps_v072_*.rs`: five suites for edit truthfulness, import round trip, lock contention, schema conformance and storage hardening
- `tests/gaps_v073_*.rs`: the four 0.5.6 suites for the auth switch, host-name charset, terminal escape and config directory mode
- `tests/gaps_v074_flag_reach.rs`: the 0.5.6 suite for flag reach, where every published flag is documented where the inventory is claimed and exercised by a behavioural test
- `tests/gaps_v075_markdown_format_gate.rs`: coverage contract for the Markdown format gate `scripts/check_markdown_format.py`, so the formatting criterion of the documents can fail
- Integration and quality suites: `tests/e2e_cli.rs` drives the CLI end to end in an isolated `--config-dir`, `tests/i18n_integration.rs` checks locale and message parity, `tests/proptest_tests.rs` holds property-based tests, `tests/scp_integration.rs` and `tests/tunnel_integration.rs` check the `scp` and `tunnel` help and required arguments, `tests/snapshot_tests.rs` snapshots stable output with `insta`, `tests/storage_integration.rs` checks the VPS registry CRUD and `tests/test_quality.rs` guards against tests that pass by construction


## Documentation
- Apply the bilingual documentation framework on every public doc
- Mirror English and `.pt-BR` content in the same delivery
- Open every public document with a cross-language link
- Keep persuasive tone out of SKILL.md and schemas
- Index every JSON schema in `docs/schemas/README.md`

### Translation policy
- English is the canonical text and the `.pt-BR` file is its mirror
- Both files have the same sections and the same bullets in the same order
- Commands, flags, field names, exit codes and code identifiers stay untouched in the translation
- Brazilian Portuguese text keeps every accent and diacritical mark
- A change to one language is not done until the other language carries it


## Report Bugs
- Open a GitHub issue with reproduction steps and expected versus actual output
- Include OS, architecture, `ssh-cli --version`, and exit code
- Redact secrets from logs and command history
- Report security vulnerabilities privately, following [SECURITY.md](SECURITY.md), and never in a public issue


## Request Features
- Open an issue describing the agent workflow and the SSH automation parity gap if any
- Prefer features that preserve one-shot lifecycle and XDG multi-host storage


## Release Process
- Move the `[Unreleased]` entries into a new `[X.Y.Z] - YYYY-MM-DD` section in both CHANGELOG languages
- Keep the section order Security, Added, Changed, Deprecated, Removed, Fixed, Internal
- Mark each breaking change with the prefix `BREAKING` and a `Migration:` line, with the same count in both languages
- Update the footer links: `[Unreleased]` compares the new tag with `HEAD`, and each version has its own compare link
- Bump SemVer in `Cargo.toml`
- Run `python3 scripts/check_all_gates.py` and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`
- Follow the item list in `docs/RELEASE_CHECKLIST.md`
- Confirm the root bilingual docs (README, SECURITY, INTEGRATIONS, llms*) describe the release surface recorded in the CHANGELOG
- The gates run locally, and the E2E matrix is optional
- Package with `cargo package --locked` and dry-run publish when needed
- Tag `vX.Y.Z` only after publish gates pass and explicit maintainer authorization
- Prefer `cargo install ssh-cli --locked` in public install docs
- Never publish secrets, real host inventories, or master keys


## Recognition
- Contributors are credited in release notes when they choose public credit
- Security researchers follow [SECURITY.md](SECURITY.md) for private credit


## Questions
- Open a discussion or issue for process questions
- Contact the maintainer at daniloaguiarbr@proton.me for private coordination
