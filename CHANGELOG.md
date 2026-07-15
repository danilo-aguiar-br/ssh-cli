# Changelog

- Read this document in [Portuguese (pt-BR)](CHANGELOG.pt-BR.md).

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- `docs/schemas/vps-show.schema.json` allows `password` type `string | null` (JSON-001 contract parity with runtime)
- Portuguese cross-language openers in `docs/*.pt-BR.md` use Portuguese narrative ("Leia este documento em inglês")
- Bilingual `docs/RELEASE_CHECKLIST.md` + `docs/RELEASE_CHECKLIST.pt-BR.md` with residual gates LOG/JSON/CLI/DOC/DENY/REL/CHG
- DOC-003 tests cover checklists and schema password null window

## [0.3.9] - 2026-07-15

### Fixed
- Post-0.3.8 audit residuals: LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL-003, CHG-001
- Default tracing level is **error** (agent-first); `-v` enables debug (LOG-001)
- Tunnel/JSON stderr no longer emits INFO progress banners by default (LOG-001)
- Key-only VPS JSON: empty password serializes as `null` instead of `"***"` (JSON-001)
- `health-check --timeout <ms>` override aligned with exec (CLI-004)
- Product-line docs bumped to **0.3.9** and residual behaviors documented across README, `llms*.txt`, INTEGRATIONS, `docs/*` (HOW_TO_USE, COOKBOOK, MIGRATION, TESTING, CROSS_PLATFORM, AGENTS, schemas), and skills (DOC-003 deep audit)
- CHANGELOG compare anchors for 0.3.8/0.3.9 (CHG-001)
- `deny.toml` documents expected multi-version warns without CVE ignore (DENY-002)

### Added
- Suite `tests/gaps_v039_integration.rs` for residual audit gaps

### Changed
- Version 0.3.8 → 0.3.9

### Notes
- No telemetry
- Local tags `v0.3.8` / `v0.3.9` (no push unless authorized)

## [0.3.8] - 2026-07-15

### Fixed
- Residual gaps post-0.3.7 audit (IO-006, EXIT-002, VAL-004, TEST-004, DOC-001, REL-001/002, DENY-001, PROC-001, E2E-001)
- Tunnel human banners no longer pollute agent stdout (JSON/non-TTY/quiet) (IO-006)
- No active VPS returns sysexits 66 via typed `ErroSshCli::NenhumaVpsAtiva` (EXIT-002)
- OpenSSH private key parse on VPS write-path after `is_file` (VAL-004)
- Full named regression suite `tests/gaps_v038_integration.rs` (TEST-004)
- Version string reports `-dirty` when working tree is dirty (REL-002)
- Inventory `gaps.md` versioned (DOC-001); release checklist `docs/RELEASE_CHECKLIST.md` (PROC-001)

### Security
- Upgrade **russh 0.62.2** (security floor ≥0.60.3); remove crypto COMPAT RC pins (DEP-002)
- `cargo deny`: `yanked=deny`, empty ignore list; drop dead `Unicode-DFS-2016` allow (DENY-001)
- Install resolve gate requires patched russh; allows stable primefield
- crossbeam-epoch ≥0.9.20 (RUSTSEC-2026-0204 / criterion dev)

### Changed
- Version 0.3.7 → 0.3.8
- `scripts/verify_install_resolve.sh` policy inverted (stable crypto allowed; patched russh required)

### Notes
- No telemetry (doctor reports `telemetry: false` only)
- Product fixes from uncommitted 0.3.7 ship in this release commit


### Added
- Full bilingual documentation framework (README, CONTRIBUTING, SECURITY, INTEGRATIONS, docs guides, schemas, skills)
- Dual license files `LICENSE-MIT` and `LICENSE-APACHE` with MIT OR Apache-2.0

## [0.3.7] - 2026-07-15

### Fixed
- All 23 gaps from `gaps.md` (VAL/IO/TUN/SCP/STATE/PERM/CLI/TEST/EXIT/SEC/DEP/IMP)
- Domain write-path: `validar_e_normalizar`, port 1..=65535, key file exists (VAL-001..003)
- I/O: global `--output-format` on VPS CRUD, `health-check --json`, JSON error envelope, `--quiet` silences human success, `println!` only in `output` (IO-001..005)
- Tunnel `--timeout-ms` covers SSH connect + loop (TUN-001)
- SCP validates local file before connect (SCP-001)
- `vps remove` clears orphan `active`; lock file `0o600` (STATE-001, PERM-001)
- `su-exec --password-stdin`; clap conflicts for password/*_stdin; completions broken-pipe safe (CLI-001..003)
- Signals tests `#[serial]`; help snapshot; non-tautological abort pattern test (TEST-001..003)
- Remote command failure uses process exit `EX_GENERAL` (not remote code) (EXIT-001)
- sudo/su password on channel stdin, not remote argv; mask always `***` (SEC-001, SEC-002)
- Import redacted UX + `--allow-incomplete` (IMP-001)
- `cargo deny` green with dated russh/crypto pin policy (DEP-001)

### Changed
- Version 0.3.6 → 0.3.7
- **Breaking (agent contracts):** long secrets no longer show 12+4 chars (always `***`); remote non-zero exit maps to process exit `1` with `remote_exit_code` in JSON error envelope
- `SSH_CLI_FORCE_TEXT=1` forces text output format (test/scripts)

### Security
- No sudo/su password in remote process list (`ps`)
- No password prefix leak in `vps list`/`show`

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

[Unreleased]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.9...HEAD
[0.3.9]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.8...v0.3.9
[0.3.8]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.7...v0.3.8
[0.3.7]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/danilo-aguiar-br/ssh-cli/releases/tag/v0.3.3
[0.2.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/danilo-aguiar-br/ssh-cli/releases/tag/v0.1.0
