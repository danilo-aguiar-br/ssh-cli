# Changelog

- Read this document in [Portuguese (pt-BR)](CHANGELOG.pt-BR.md).

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Full bilingual documentation framework (README, CONTRIBUTING, SECURITY, INTEGRATIONS, docs guides, schemas, skills)
- Dual license files `LICENSE-MIT` and `LICENSE-APACHE` with MIT OR Apache-2.0

## [0.3.6] - 2026-07-15

### Added
- Default at-rest encryption: auto-create XDG `secrets.key` (0o600) on first secret write
- CLI `secrets status|init|reencrypt` (never prints master key)
- `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` opt-out for tests
- Doctor fields: `secrets_key_file`, `secrets_plaintext_opt_out`
- Script `scripts/e2e_real_ssh.sh` for real SSH E2E without logging credentials
- Auth failure message teaches `--password-stdin` / `--key` / passphrase stdin

### Changed
- Version 0.3.5 → 0.3.6
- GAP-009 residual: encryption is default (not merely optional)
- Pin freeze documentation for russh 0.60.0 + crypto RC pins (R-PINS)

### Security
- Secrets in `config.toml` encrypted by default (`sshcli-enc:v1:…`)
- E2E protocol forbids printing host/user/password and uses `/tmp` + password-stdin

## [0.3.5] - 2026-07-15

### Fixed
- Residual GAP-007: atomic `vps export` (tempfile + fsync + 0o600)
- Residual GAP-006: remote abort uses TERM then KILL with longer sanitized pattern
- Residual GAP-009/012: optional at-rest secret encryption (ChaCha20-Poly1305) via env/file/keyring
- README no longer recommends install without `--locked`
- gaps.md parity matrix updated for 0.3.4/0.3.5 reality

### Added
- `--key-passphrase` / `--key-passphrase-stdin` runtime overrides on exec/sudo-exec/su-exec
- Auto JSON when stdout is not a TTY (unless `--output-format` set)
- `vps doctor` reports `secrets_at_rest` and `secrets_key_source` (never prints secrets)
- Integration tests `tests/gaps_v035_integration.rs` (fake secrets only)

### Changed
- Version 0.3.4 → 0.3.5

## [0.3.4] - 2026-07-15

### Fixed
- `cargo install` crypto graph: pin `primefield`, `primeorder`, `ecdsa`, `pkcs5`, exact `russh = 0.60.0` (GAP-014)
- `sudo-exec` packing with `sh -c`  (GAP-005)
- Atomic `config.toml` write with tempfile + fsync + flock (GAP-007)
- Host key TOFU via XDG `known_hosts` (GAP-008)
- Dual `max_command_chars` / `max_output_chars` (GAP-004)
- Timeout remote abort best-effort (GAP-006)
- Credential validation: password or key required (GAP-011)

### Added
- Auth by private key (`--key`, `key_path`) via russh `load_secret_key` (GAP-002)
- `su-exec` one-shot consuming `senha_su` (GAP-003)
- `--password-stdin` / sudo / su stdin secrets (GAP-009)
- `vps doctor`, `vps export`, `vps import` (GAP-012)
- Tunnel mandatory `--timeout-ms` (GAP-010)
- `--disable-sudo`, `--description`, `--replace-host-key`
- Schema v2 multi-host XDG fields
- Install resolve gate: `scripts/verify_install_resolve.sh`

### Changed
- Default timeout 60000 ms 
- `directories` 5 → 6 (GAP-013)
- Version bump 0.3.3 → 0.3.4
- Dual license MIT OR Apache-2.0

## [0.3.3] - 2026-07-15

### Changed
- Migrated crate ownership and repository to `danilo-aguiar-br` after previous GitHub account ban (crates.io owner was `ghost_*`).
- `repository` / `homepage` now point to `https://github.com/danilo-aguiar-br/ssh-cli`.
- Author metadata updated to `Danilo Aguiar <daniloaguiarbr@proton.me>`.
- Removed GitHub Actions CI/CD workflows and CI badges — new repository ships without Actions.

### Note
- crates.io already had versions through `0.3.2` from the previous owner account; this release is the first under the new owner and repository URL.

## [0.2.1] - 2026-04-16

### Fixed
- Pin `elliptic-curve = "=0.14.0-rc.30"` to fix `cargo install ssh-cli` failure caused by incompatible `elliptic-curve 0.14.0-rc.31+` being resolved for `p256/p384/p521 0.14.0-rc.8`

## [0.2.0] - 2026-04-15

### Added
- Fix sudo-exec stdin password piping with `printf '%s\n'`
- Runtime overrides: --password, --sudo-password, --timeout flags on exec/sudo-exec/scp/tunnel
- LLM-friendly camelCase aliases (--sudoPassword, --suPassword)

## [0.1.0] - 2026-04-14

Initial release.

[Unreleased]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.6...HEAD
[0.3.6]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/danilo-aguiar-br/ssh-cli/releases/tag/v0.3.3
[0.2.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/danilo-aguiar-br/ssh-cli/releases/tag/v0.1.0
