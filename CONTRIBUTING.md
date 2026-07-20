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
timeout 120 cargo check --all-targets --locked
timeout 300 cargo test --locked
timeout 60 bash scripts/verify_install_resolve.sh
```


## Development Setup
### Toolchain requirements
- Require MSRV Rust 1.85.0 declared in `Cargo.toml`.
- Install Rust via `rustup` and prefer the pinned toolchain file when present.
- Keep `Cargo.lock` committed because this crate ships a binary CLI.
- Never bump MSRV without an explicit issue discussion.

### Dependency pinning
- Product line **0.5.2** uses **russh 0.62.2** (since 0.3.8) without the older COMPAT RC crypto pins; do not reintroduce dead RC pins without an issue.
- Never run blind `cargo update` on the crypto graph.
- Run `scripts/verify_install_resolve.sh` after any dependency change.


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
- Include gap regression suites when touching residual audit surface: `tests/gaps_v038_integration.rs` … `tests/gaps_v051_integration.rs` (export/import/wire/secrets), `tests/gaps_v057_sftp.rs`, and **`tests/gaps_v058_e2e_residual.rs`** (G-E2E residual: ACME permanent, single `vps-added`, root `schema`/`doctor`, clap no env, `-v` only, FIXED_MASK, `--use-agent`).
- For local real-SSH E2E (G-E2E-05): prefer **`--config-dir`** with hosts already registered via `vps add`, or maintainer-local `bash scripts/e2e_real_ssh.sh --from-grok-config` reading `$HOME/.grok/config.toml` only. Harness-only `SSH_CLI_E2E_*` env is accepted by the script (not product runtime). Without a lab host the script exits **0** with **SKIP** (offline-safe). Default binary is `target/release/ssh-cli`. Official matrix **E01–E16** (E10–E14 SCP; E15 tunnel port 0; E16 symlink). Prefer **local sshd** / lab hosts; **no intentional auth storm** on production fail2ban targets; never log credentials; never commit Grok/MCP config or host inventories into this repo.
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
- Confirm root bilingual docs (README, SECURITY, INTEGRATIONS, llms*) match the **0.5.2** release surface: root `schema`/`doctor`, single-JSON `vps-added` + `secrets_key_auto_created`, ambient `RUST_LOG` ignored (`-v` only), ACME `invalidContact`→64, export redacted `***` (`FIXED_MASK`), `vps add --use-agent`, no product GH Actions, `secrets` + default encryption, wire schema v3 dual-read, SFTP surface, and suites `gaps_v042` + `gaps_v051` + **`gaps_v058`**. Local gates only (no cloud CI product workflows).
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
