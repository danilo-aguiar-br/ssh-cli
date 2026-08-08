# Contributing to ssh-cli

- Read this document in [Portuguese (pt-BR)](CONTRIBUTING.pt-BR.md).


## Welcome
- Thank you for contributing code, docs, tests, or bug reports.
- Every improvement strengthens one-shot multi-host SSH for AI agents.
- This guide targets onboarding under 10 minutes from clone to first test.


## Quick Start
- Clone the repository and enter the workspace root.
- Validate a clean tree with the commands below.

```bash
timeout 3600 bash scripts/check_all_gates.sh
```

- That single command runs the whole mandatory battery; the individual gates below are for focused reruns.

```bash
timeout 120 cargo check --all-targets --locked
timeout 300 cargo test --locked
timeout 60 bash scripts/verify_install_resolve.sh
timeout 900 bash scripts/check_cross_targets.sh
```

### Full gate battery (required before declaring a round closed)
- Run `scripts/check_all_gates.sh` before claiming any gate is green.
- It runs all ten mandatory gates in one invocation and reports every result.
- The ten are `fmt`, `build-release`, `build-no-default`, `clippy`, `test`, `deny`, `cross-targets`, `advisory-freshness`, `en-identifiers` and `install-resolve`.
- `scripts/check_advisory_freshness.sh` has no other caller, so skipping the battery skips that gate entirely.
- The reason it exists is structural, not stylistic.
- `cargo clippy` and `cargo test` both abort on the first unbuildable target.
- One broken test file therefore hides the state of every gate behind it.
- That is exactly how the local inventory came to declare 835 green while four gates were red.
- Use `--only ID[,ID...]` for a subset and `--list` to see the ids.
- The run reports what it skipped, so a partial run can never read as a full one.
- The battery is sequential by design because the cargo gates share one `target/` lock.

### Cross-target gate (B1 — required)
- Run `scripts/check_cross_targets.sh` after touching anything under `#[cfg(target_os = ...)]`.
- Every other gate runs for the host triple only.
- Code behind a foreign `cfg` is discarded before type-check, so it never reaches `fmt`, `clippy`, `test` or `deny`.
- A green board therefore proves nothing about Windows or macOS.
- This is not hypothetical: the Windows target failed with six errors while all other gates were green.
- The script type-checks `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` and `x86_64-apple-darwin`.
- Windows is checked with `--no-default-features` because the default TLS stack pulls `aws-lc-sys`, which compiles C.
- That exception is structural (A8) and still type-checks 100% of the product's own `cfg(windows)` code.
- Do not claim a target in `docs/CROSS_PLATFORM.md` that this script does not check.


## Development Setup
### Toolchain requirements
- Require MSRV Rust 1.85.0 declared in `Cargo.toml`.
- Install Rust via `rustup` and prefer the pinned toolchain file when present.
- Keep `Cargo.lock` committed because this crate ships a binary CLI.
- Never bump MSRV without an explicit issue discussion.

### Dependency pinning
- Product line **0.5.4** uses **russh 0.62.5** (since 0.3.8) without the older COMPAT RC crypto pins; do not reintroduce dead RC pins without an issue.
- Never run blind `cargo update` on the crypto graph.
- Run `scripts/verify_install_resolve.sh` after any dependency change.

### Local audit inventory
- `gaps.md` is a **local** audit inventory (gitignored; not published). Do not assert FIXED status by grepping its prose (G13/G15 — FIXED requires destination-effect proof such as checksums).


## Branching Strategy
- Keep `main` as the integration branch.
- Use `feature/<short-kebab>` for features.
- Use `fix/<short-kebab>` for bug fixes.
- Use `docs/<short-kebab>` for documentation-only work.
- Use `chore/<short-kebab>` for tooling and maintenance.


## Commit Convention
- Follow Conventional Commits 1.0.0 on shared branches.
- Use `feat` for user-visible features.
- Use `fix` for bug fixes.
- Use `docs` for documentation-only changes.
- Use `test` for test-only changes.
- Use `chore` for maintenance.
- Never add `Co-authored-by` lines for AI agents.


## Pull Request Process
- Open a PR with a clear problem statement and validation commands.
- Include bilingual docs when public documents change.
- Keep CLI one-shot behavior intact in every product command.
- Prohibit introducing long-lived daemon packaging or telemetry.
- Request review only after `cargo test --locked` and clippy pass.


## Testing
- Read [docs/TESTING.md](docs/TESTING.md) for categories and profiles.
- Prefer deterministic unit tests for packing and schema migration.
- Use integration tests under `tests/` for CLI contracts.
- Include gap regression suites when touching residual audit surface. Name them explicitly rather than as a range — an elided list (`v038 … v051`) does not satisfy a `contains` check and silently drops the suites in the middle: `tests/gaps_v035_integration.rs`, `tests/gaps_v037_integration.rs`, `tests/gaps_v038_integration.rs`, `tests/gaps_v039_integration.rs`, **`tests/gaps_v040_integration.rs`** (SCP/tunnel behavioural contracts), `tests/gaps_v041_integration.rs`, `tests/gaps_v042_integration.rs`, `tests/gaps_v051_integration.rs` (export/import/wire/secrets), `tests/gaps_v052_tls_policy.rs`, `tests/gaps_v053_domain_types.rs`, `tests/gaps_v054_error_handling.rs`, `tests/gaps_v055_unsafe_ffi.rs`, `tests/gaps_v056_ssh.rs`, `tests/gaps_v057_sftp.rs`, `tests/gaps_v058_e2e_residual.rs` (G-E2E residual: ACME permanent, single `vps-added`, root `schema`/`doctor`, clap no env, graduated `-v`/`-vv`/`-vvv`, FIXED_MASK, `--use-agent`), **`tests/gaps_v059_agent_native.rs`** (0.5.4 agent-native surface: `--no-input` refusal on `vps add`/`vps edit`, payload shaping with `--select`/`--filter`/`--limit`/`--count-only`, non-loopback `--bind` guard), and **`tests/gaps_v060_tunnel_modes.rs`** (LOTE E: `--reverse`/`--socks5`/`--remote-socket` argument contracts and the `--dry-run` preview, both driven through the real binary — the v0.5.4 audit found `--no-input` shipped inert precisely because nothing ever executed it), and **`tests/gaps_v064_gate_runner.rs`** (gate battery coverage contract: every `scripts/check_*.sh` is either a gate or a declared exclusion).
- Documentation conformance lives in **`tests/docs_conformance.rs`**, separate from behavioural suites: editing prose must never turn the build gate red without a functional regression (G-QA-R03).
- **Local gates (required before PR):** `cargo fmt --check`, `cargo test --locked` (and clippy as in release process). Real-SSH E2E is **optional** when no lab host is available.
- For local real-SSH E2E (G-E2E-05): prefer **`--config-dir`** with hosts already registered via `vps add`, or maintainer-local `bash scripts/e2e_real_ssh.sh --from-grok-config` reading `$HOME/.grok/config.toml` only. Harness-only `SSH_CLI_E2E_*` env is accepted by the script (not product runtime). Without a lab host the script exits **0** with **SKIP** (offline-safe). Default binary is `target/release/ssh-cli`. Official matrix **E01–E18** (E10–E14 SCP; E15 tunnel port 0; E16 symlink; E17/E18 SFTP checksum). Prefer **local sshd** / lab hosts; **no intentional auth storm** on production fail2ban targets; never log credentials; never commit Grok/MCP config or host inventories into this repo.
- Unit/integration tests that need plaintext secrets must pass **`--allow-plaintext-secrets`** (CLI flag; not an env product store).
- Never leave flaky remote-dependent tests without timeouts.


## Documentation
- Apply the bilingual documentation framework on every public doc.
- Mirror English and `.pt-BR` content in the same delivery.
- Open every public document with a cross-language link.
- Keep persuasive tone out of SKILL.md and schemas.
- Index every JSON schema in `docs/schemas/README.md`.


## Report Bugs
- Open a GitHub issue with reproduction steps and expected versus actual output.
- Include OS, architecture, `ssh-cli --version`, and exit code.
- Redact secrets from logs and command history.


## Request Features
- Open an issue describing the agent workflow and the SSH automation parity gap if any.
- Prefer features that preserve one-shot lifecycle and XDG multi-host storage.


## Release Process
- Bump SemVer in `Cargo.toml` and update both CHANGELOG languages.
- Run full test suite, clippy `-D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`, and install resolve gate.
- Confirm root bilingual docs (README, SECURITY, INTEGRATIONS, llms*) match the current **0.5.4** release surface, which keeps the whole 0.5.3 set closed and adds the 0.5.4 entries recorded in the changelog (`--reverse`/`--socks5`/`--remote-socket`, `scp-transfer` `mtime_preserved`/`durable`, exit codes 69 and 70): G1–G19 closed (SFTP integrity, graduated crate-scoped `-v`/`-vv`/`-vvv`, batch cancel cardinality), root `schema`/`doctor`/`commands`/`locale`/`tls`, single-JSON `vps-added` + `secrets_key_auto_created`, ambient `RUST_LOG` ignored, ACME `invalidContact`→64, export redacted `***` (`FIXED_MASK`), `vps add --use-agent`, no product GH Actions, `secrets` + default encryption, wire schema v3 dual-read, SFTP surface prefer 0.5.3+, and suites `gaps_v042` + `gaps_v051` + **`gaps_v058`**. Local gates only: `cargo fmt --check`, `cargo test`, E2E optional (no cloud CI product workflows).
- Package with `cargo package --locked` and dry-run publish when needed.
- Tag `vX.Y.Z` only after publish gates pass and **explicit maintainer authorization**.
- Prefer `cargo install ssh-cli --locked` in public install docs.
- Never publish secrets, real host inventories, or master keys.


## Recognition
- Contributors are credited in release notes when they choose public credit.
- Security researchers follow [SECURITY.md](SECURITY.md) for private credit.


## Questions
- Open a discussion or issue for process questions.
- Contact the maintainer at daniloaguiarbr@proton.me for private coordination.
