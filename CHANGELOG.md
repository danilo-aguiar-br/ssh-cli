# Changelog

- Read this document in [Portuguese (pt-BR)](CHANGELOG.pt-BR.md).

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.2] - 2026-07-19

### Added
- **Global `--json`** (G-AUD-01): agent-friendly alias that forces JSON output (clap `from_global` on subcommands).
- **`exec` / `sudo-exec` / `su-exec` active VPS** (G-AUD-04): one positional = COMMAND against `connect` active host.
- **`vps path` JSON envelope** (G-AUD-02): `event: vps-path` when format is JSON.
- **`fs_perm` module** (G-AUD-24): single source for secret file/dir Unix modes.

### Fixed
- **Password argv warning false positive** (G-AUD-08): inspect real `Option` fields, not `Debug` strings.
- **TLS missing PEM files** (G-AUD-05): `FileNotFound` / permanent class (not exit 74 retryable).
- **`vps export` honors global JSON format** (G-AUD-03).
- **ACME account create requires `--contact mailto:…`** (G-AUD-06/28).
- **Primary auth mutual exclusion** on write (G-AUD-07): exactly one of password / key / agent.
- **Secrets error messages** no longer advertise env key stores (G-AUD-21).
- **Log filter CLI-only** (G-AUD-22): ambient `RUST_LOG` ignored; use `-v`.
- **Concurrency hard cap single source** (G-AUD-19/23): `constants::MAX_CONCURRENCY`.
- **Skill description ≤1024** chars (G-AUD-15).
- **ACME validation permanent** (G-E2E-01): `invalidContact` / 4xx problem types → exit 64 non-retryable (`tls/acme_error_map.rs`).
- **Single JSON on `vps add` auto-key** (G-E2E-04): fold `secrets_key_auto_created` into `vps-added` (one document).
- **Root `schema` + `doctor`** (G-E2E-02/03): agent discovery parity.
- **Version `-dirty` with `.commit_hash`** (G-E2E-06): honest provenance on dirty trees.
- **clap `env` feature removed** (G-E2E-08); help no longer teaches env stores (G-E2E-07).
- **Export redacted mask** (G-E2E-10): `***` via `FIXED_MASK`, not empty string.
- **`vps add --use-agent`** (G-E2E-19): registry auth triplo complete.
- **E2E harness offline SKIP** + release default bin (G-E2E-05); EN test idents (G-E2E-13).

### Removed
- **`.github/workflows`** (G-AUD-11): local gates only; no product CI/GH Actions in tree.
- **`src/erros.rs` PT shim** (G-AUD-14).
- **Product env config stores** `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` reads (G-AUD-12).

### Changed
- Version **0.5.1 → 0.5.2**.
- Integration tests aligned to CLI/XDG-only config (no env secrets/format store).

### G-SFTP residual harden R01–R15

### Security
- **Entry-name validation** + `ensure_local_under` on recursive/multi-file download (malicious SFTP server cannot escape local dest).
- **Partial cleanup** on every SFTP download error (parity with SCP).
- **Upload tree root** uses `symlink_metadata` (no-follow).

### Changed
- **Outer wall-clock timeout** (`under_timeout`) on multi-file and FS ops (`ls|mkdir|rm|stat|rename`).
- **`cli/scp_args.rs`** extracted (SRP; monólito `cli/mod` shrink).
- Docs/skills/llms: SCP = regular files; trees/FS = **`sftp`**.

### G-SFTP: SFTP subsystem

### Added
- **`russh-sftp` 2.3** (feature `ssh-real`) — SFTP v3 client subsystem.
- **`ssh-cli sftp`:** `upload|download` (optional `--recursive`), `ls`, `mkdir`, `rmdir`, `rm`, `stat`, `rename`.
- **Modules:** `src/ssh/sftp_session.rs`, `sftp_path.rs`, `sftp_types.rs`, `src/sftp/`, `src/cli/sftp_args.rs`.
- **JSON schemas:** `sftp-transfer`, `sftp-list`, `sftp-fs-op`, `sftp-batch`.
- **Gates:** `tests/gaps_v057_sftp.rs`.
- Multi-host `--all`/`--hosts` for SFTP upload/download (`map_bounded`).

### Changed
- **ScpOptions / SftpOptions:** `use_agent` + `agent_socket` (G-SFTP-17/18; CLI/XDG only).
- Stream transfers in 32 KiB chunks; partial download + atomic rename; symlink no-follow on trees.

### Security
- Remote path validation (control chars / empty segments).
- Recursive depth cap; listing entry cap; no full-file heap via `SftpSession::read/write`.

### G-SSH: SSH client rules / russh

### Added
- **`src/ssh/client_handler.rs`:** TOFU `check_server_key` + `HostKeyOutcome` typed recovery (G-SSH-01/09/14).
- **`src/ssh/client_connect.rs`:** dial + auth chain file key → agent → password (G-SSH-04/16).
- **`src/ssh/key_material.rs`:** Unix 0600 private-key perms + RSA ≥2048 / DSA reject (G-SSH-03/07).
- **CLI/XDG agent:** `--use-agent`, `--agent-socket`; `VpsRecord.use_agent` / `agent_socket` (no env store).
- **Gates:** `tests/gaps_v056_ssh.rs`.

### Changed
- **SSH client id:** `SSH-2.0-ssh-cli` (no russh version banner fingerprinting).
- **Config policy:** explicit rekey 1h/1GiB, window 2MiB, packet 32KiB, TCP SO_KEEPALIVE via `socket2`.
- **deny.toml:** ban `ssh2`, `thrussh`, `libssh-rs`.
- SRP split of connect/handler/key load from session/SCP path.

### Security
- Host-key change returns typed `HostKeyChanged` (exit EX_NOPERM) to agents.
- Fail-closed when `known_hosts_path` is missing outside tests.
- Password fallback remains opt-in via non-empty inventory secret (documented).

### G-UNSAFE: unsafe code e FFI

### Added
- **`src/test_util/env.rs`:** encapsulated `set_var`/`remove_var` with `// SAFETY:` (ed2024-ready).
- **`src/vps/config_io.rs`:** config path/load/save/permissions split (SRP / G-UNSAFE-10).
- **Gates:** `tests/gaps_v055_unsafe_ffi.rs` (allowlist, forbid, main order, no plaintext env).

### Changed
- **`main`:** `signals::register_handler()` **before** Tokio `new_multi_thread` (G-UNSAFE-13 / signal-hook first-hook).
- **SIGTERM SAFETY** multi-bullet (async-signal-safe atomics only; double-hit justified vs `flag::register`).
- **Windows console FFI** module safety docs; one op per `unsafe` block unchanged.
- VPS tests use `set_runtime_flags` for plaintext opt-out (no `SSH_CLI_ALLOW_PLAINTEXT_SECRETS` env).
- Secrets docs: fail-closed env key material; plaintext only via CLI flag.
- Concurrency docs: CLI `--max-concurrency` + auto formula only (no env store).
- `forbid(unsafe_code)` on `ssh/mod`, `vps/mod`, `vps/model`, `vps/config_io`.

### Security
- Product `unsafe` allowlist: `platform/windows.rs` + `signals.rs` only; test env mutation centralized.

### G-ERR: tratamento de erros

### Added
- **`SshCliError::Domain`**, **`Crypto`**, **`Config`** variants; TLS/channel structured errors with optional `source`.
- **Helpers:** `tls_msg` / `tls_src` / `channel_msg` / `channel_src` / `error_code()` (G-ERR-08/16).
- **JSON envelope `error_code`** field for agents (stable snake_case).
- **Gates:** `tests/gaps_v054_error_handling.rs`.
- **SSH client split (G-ERR-12):** `client.rs` facade + `client_real.rs` / `client_stub.rs` / `client_tests.rs`.

### Changed
- Display messages lowercase, no trailing period (G-ERR-01 / thiserror convention).
- `paths` validation returns `SshCliResult` (no `anyhow`/`bail!`).
- `VpsRecord::validate*` → `Result<(), DomainError>`.
- Secrets: env key material rejected (XDG + CLI only); concurrency `resolve_limit` ignores env store.
- `xdg_config_dir` uses `SshCliError::XdgDirectory`.

### Security
- No secret material in error Display for crypto ops; fail-closed on `SSH_CLI_SECRETS_KEY*` env store.

### G-DOM: tipos de domínio chrono/uuid/rust_decimal/url

### Added
- **Four domain crates (coordinated):** `chrono` 0.4.45 (`serde`+`clock`), `uuid` 1.24 (`v4`+`v7`+`serde`), `rust_decimal` 1.42 (`serde-with-str`+`macros`), `url` 2.5 (`serde`).
- **`src/domain/` split (SRP):** `error`, `names`, `ports`, `limits`, `command`, `time`, `ids`, `http_url`, `money`.
- **`Rfc3339Utc` / `AddedAt` / `CreatedAt`:** `DateTime<Utc>` newtype; VPS `added_at` + ACME timestamps.
- **`HttpsUrl` / `AcmeOrderUrl`:** HTTPS-only parse for ACME order resume (XDG).
- **`CorrelationId` (v4) / `BatchRunId` (v7):** multi-host batch JSON field `batch_run_id`.
- **`Money<C: Currency>`:** library-ready decimal money (not wired into VPS).
- **Gates:** `tests/gaps_v053_domain_types.rs` + proptest roundtrips (G-DOM-07/09).

### Changed
- Batch schemas (`health-check-batch`, `exec-batch`, `scp-batch`) require `batch_run_id` (UUID v7).
- Import `added_at` validates RFC 3339 (no length-only check).

### Security
- No `Local::now` in product sources; no `serde-float` on decimals; ACME URLs reject `http`/`data`/`javascript`.

### G-TLS product: rustls / SSH-over-TLS / mTLS / ACME

### Added
- **Feature `tls` (default):** `rustls` ≥ 0.23.18 + `aws_lc_rs`, `tokio-rustls`, `webpki-roots`, `rustls-pki-types`, `instant-acme`.
- **`CryptoProvider::install_default`** in binary `main` (aws_lc_rs only; libraries never reinstall).
- **`ClientConfig` builder** (`src/tls/client_config.rs`) with webpki-roots + optional mTLS client cert.
- **SSH-over-TLS:** `ConnectionConfig.tls` / VPS fields `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key`; dial TCP → rustls → `russh::connect_stream`.
- **CLI `tls`:** `provider`, `paths`, `mtls import|list|show|remove`, `acme account create|show`, `acme issue --print-challenge`, `acme complete`, `acme status|list`.
- **XDG layout:** `tls/mtls/<name>/`, `tls/acme/account.json`, `tls/acme/<domain>/` (0o600 secrets).
- Residual suite updates in `tests/gaps_v052_tls_policy.rs`.

### Changed
- `deny.toml`: allow product `rustls`; keep bans on `openssl*`, `native-tls`, `libssh2-sys`, `ring`; allow license `CDLA-Permissive-2.0` (webpki-roots).
- PEM load via `rustls-pki-types::PemObject` (no unmaintained `rustls-pemfile`).

### G-TLS / rustls policy — prior session

### Added
- **`src/ssh/connect.rs`:** single `build_ssh_client_config` + Happy Eyeballs dial helpers (G-TLS-07/09).
- **Residual suite** `tests/gaps_v052_tls_policy.rs` — lockfile/deny bans, no flate2, SECURITY/README policy (G-TLS-03).
- **SECURITY Transport & crypto policy (G-TLS)** — SSH ≠ TLS; aws-lc-rs; future rustls-only if HTTP; local gates only.

### Changed
- **SSH compression `none` only** — preferred algorithms never offer zlib (G-TLS-04).
- **russh features:** drop `flate2` (G-TLS-05); keep `aws-lc-rs` only.
- **`deny.toml`:** ban `openssl`, `ring`, `rustls` in addition to `openssl-sys` / `native-tls` / `libssh2-sys` (G-TLS-02).
- README / CROSS_PLATFORM / RELEASE_CHECKLIST / llms: crypto policy surfaces (G-TLS-01/06/08/11/12).

### Security
- No product TLS stack; no OpenSSL/`native-tls`/`ring`/`rustls` in the dependency graph.
- No product OTEL. Secrets remain `SecretString` + XDG AEAD.

### Sistema de Tipos

### Added
- **Domain newtypes (G-TYPE-01…20):** `src/domain/` with `VpsName`, `SshHost`, `SshUser`, `SshPort(NonZeroU16)`, `TimeoutMs`, `HostTag`, `CharLimit`, `RemoteCommand`, `KeyPath`, `BindPort` (ephemeral ≠ SSH).
- **`ssh/session_io.rs`:** extracted `truncate_utf8` / capped UTF-8 helpers (G-TYPE-14).
- Zero-cost layout tests for `SshPort` / `Option<SshPort>` niche.

### Changed
- **`VpsRecord` / `ConnectionConfig`:** fields are domain-proven; `try_new` replaces infallible `new`.
- **`HostSelection`:** `VpsName` / `HostTag` (parse at CLI boundary).
- **`ExecOptions` / `ScpOptions`:** `Option<TimeoutMs>`; steps are `Vec<RemoteCommand>`.
- **CLI ports:** `value_parser!(u16).range(1..=65535)` on VPS add/edit and tunnel remote port; local bind still allows `0`.
- **`paths::validate_and_normalize` → `Result<VpsName>`** (proof retained).
- **Import JSON:** empty host/user rejected; builds via `VpsRecord::try_new`.

### Security
- Auth material check centralized (`secret_nonempty`); no secrets in domain error messages.
- No product OTEL.

### Session notes (validation / serde / componentization)

### Added
- **Validation pipeline (G-SERDE-01…14):** `validator` 0.20 + `serde_with` 3 + `serde_path_to_error` + `serde_ignored`; module `src/validation.rs` (parse → serde → validate → domain).
- **Host tags on agent JSON wire (G-SERDE-06):** `tags` on list/export/import DTOs with round-trip tests.
- **Structural config validation on load (G-SERDE-04):** empty host/user, port 0, limit/char limits, tags charset.
- **Fuzz target** `import_envelope` for import deserialize (G-SERDE-12).
- **`ssh/connection.rs`:** ConnectionConfig extracted (G-COMP-R).
- **`cli/tests.rs`:** CLI unit tests extracted from monólito (G-COMP-R).

### Changed
- **deny_unknown_fields** on TOML `ConfigFile` / `VpsRecord` (G-SERDE-05); import JSON remains Must-Ignore with warn (G-SERDE-14).
- **SCP multi-host fan-out** uses `Arc<ScpOptions>` (G-MEM-SCP); `apply_scp_options` takes `&ScpOptions`.
- **CI Actions pinned by commit SHA** (G-PROC-PIN).
- Serde deps caret: `serde = "1"`, `serde_json = "1"`.

### Security
- No product telemetry (OTEL still forbidden). Secrets stay in `SecretString`; validation never logs secrets.


### Prior closeout / process notes

### Changed
- **O1–O6 + process (mandatory):** global `--fail-fast` (`map_bounded_with`); host `tags` + `--tags` / `vps --tag`; multi-cmd `--step` one session; SCP `--scp-file-concurrency` parallel channel windows; Arc `ExecOptions` fan-out; proptest packing + fuzz targets; CI miri/geiger/sbom/proptest; `scripts/release_attest.sh`. Zero product telemetry.
- **Deep componentization (G-COMP-05 / G-COMP-06a–d / G-CLOSE-09 / G-DRY-01 / G-EN-R01):** extract `vps/exec_ops.rs` (exec/sudo/su + DRY `finish_execution_output`); `ssh/scp_wire.rs`; `scp/{mod,batch}.rs`; `output/{mod,batch}.rs`; `cli/{mod,dispatch}.rs`; populate `commands/*` thin reexports. Residual EN renames (`metadata`, `config_path`). OPEN product security remains 0; monólitos inventoriáveis fatiados.
- **Componentization (G-COMP-02…04):** split `vps/doctor.rs`, `vps/import_export.rs`, and `vps/health.rs` out of the `vps` monolith (`mod.rs` ~2428 → ~1698 LOC); re-export `HostHealthResult` / `run_health_check` for existing callers.

### Security
- **Closeout meta-audit (G-CLOSE):** doctor/concurrency/SCP integer casts use `TryFrom` (no truncating `as`); remaining pure modules `forbid(unsafe_code)`; extract `vps/selection.rs` (SRP); re-ran context7 + docsrs-cli + duckduckgo for skill compliance.
- **Security development audit (G-SECDEV):** secrets cross the CLI boundary as `SecretString` (`read_secret_stdin` + `ExecOptions`/`ScpOptions`/tunnel/health overrides); pure modules `#![forbid(unsafe_code)]`; deny `clippy::mem_forget` + undocumented/multi-op unsafe; STRIDE map + CVSS v4 preference in `SECURITY.md` (+ pt-BR).
- **Defensive security audit (G-SEC):** deny `unsafe_op_in_unsafe_fn`; release
  `overflow-checks`; TOFU fingerprint constant-time compare; product paths
  free of `.unwrap`/`.expect`/`unreachable!` in CLI parsers, concurrency admit,
  and single-host selection arms; import port via `u16::try_from`;
  `SshCliError` is `#[non_exhaustive]`; threat model in `SECURITY.md` (+ pt-BR);
  CI job `cargo deny check` (`deny.toml`).

### Added
- **Retry audit (G-RETRY):** typed error classification (`ErrorClass` / `ErrorLayer` /
  `RetryKind`, `is_retryable` / `is_permanent` / `suggestion`) on `SshCliError`;
  named `retry::RetryConfig` with full-jitter backoff and agent defaults (max 2
  retries on exit 74); JSON error envelope fields `error_class`, `retryable`,
  `suggestion` + schema update. In-process auto-retry of non-idempotent remote
  ops remains **off** (agent re-invokes the process).

### Fixed
- **Network audit (G-NET):** SSH dial uses async DNS + Happy Eyeballs multi-address race
  (`net::dial_tcp` + `russh::client::connect_stream`); enable `TCP_NODELAY` and SSH
  keepalives (`15s` / max `3`); private-key load and known_hosts TOFU run in
  `spawn_blocking`; tunnel accept continues on transient errors and sets nodelay on
  local forwards.


### Changed
- **Hardcode audit (G-HC):** central `constants` module for XDG file names, env keys,
  app identity, network defaults (`DEFAULT_SSH_PORT`, tunnel bind/origin), process
  timing, AEAD/keyring sizes; single `paths::xdg_config_dir()` helper. No product
  secrets/URLs were present; hosts remain registry/CLI data.

### Fixed
- **External process audit (G-PROC):** `build.rs` git probes set explicit `Stdio`
  (null/piped); remote commands reject NUL before SSH exec packing; test
  `ssh-keygen` fixtures use direct argv + explicit stdio and skip if missing.
- Docs: CROSS_PLATFORM / AGENTS process-boundary policy (no local OpenSSH spawn;
  MSRV ≥ 1.77.2 BatBadBut; remote `sh -c` packing only on target host).

### Added
- **Multi-host bounded concurrency (modus operandi):** `health-check|exec|sudo-exec|su-exec|scp --all` fans out with `Semaphore` + `JoinSet` (cap from `--max-concurrency` / `SSH_CLI_MAX_CONCURRENCY` / auto CPUs×RAM formula, clamp 1..=64). Batch JSON: `health-check-batch` / `exec-batch` / `scp-batch` (`docs/schemas/*-batch.schema.json`). Tunnel accept forwards use the same admission gate.
- **Selective multi-host `--hosts a,b,c`:** same bounded fan-out and batch JSON as `--all` (even for one name); unified via `HostSelection` + `resolve_host_jobs`.
- **Multi-file SCP (single-host, G-PAR-47):** `scp upload VPS f1 f2 … REMOTE_DIR` / download symmetric — **one SSH session**, serial transfers (auth once).
- **Multi-host × multi-file SCP (G-PAR-48):** `scp upload --all f1 f2 … REMOTE_DIR` / `--hosts a,b` — outer bounded fan-out per host session; files serial on each session; download writes under `LOCAL_DIR/<host>/`.
- **TOFU flock (G-PAR-49):** `known_hosts` mutations exclusive-lock + reload-merge under multi-host first-connect.
- **`vps doctor --probe-ssh [--hosts a,b]`:** single JSON root `event: vps-doctor` with `local` + optional `ssh_probe` (no dual roots).
- **`map_bounded` cancel:** stops admission on SIGINT/SIGTERM; `force_exit` aborts in-flight JoinSet (timer poll); debug logs `available_permits`; span `fan_out_unit` per task (G-PAR-52).
- Agent docs/skills: multi-host fleet + multi-file / cartesian SCP + doctor envelope contract.

### Changed
- SCP upload/download path validation and post-transfer metadata use `tokio::fs` / `spawn_blocking` (non-blocking Tokio workers under multi-host fan-out).
- `scripts/dist_multiarch.sh` supports `PARALLEL_JOBS` (default 2) via `xargs -P`.

## [0.5.1] - 2026-07-17

### Fixed
- **Import/export agent roundtrip**: `vps export` default body is **TOML** even on non-TTY; JSON only with `--json`. Import accepts TOML (EN+PT keys) and JSON `vps-export` envelopes (GAP-AUD-001/022).
- **Wire dual-read**: deserialize EN + legacy PT aliases; serialize English keys; schema **v3**; default `added_at` when missing (GAP-AUD-002/021). Supersedes 0.5.0 wire note (PT keys via `serde(rename)` only).
- **`secrets init` / `reencrypt` JSON** envelopes (`event: secrets-init|secrets-reencrypt`) via `--json` or `--output-format json` (GAP-AUD-003).
- Empty command error is English technical (`empty command`) under any locale (GAP-AUD-004).
- CRUD/connect/import success paths emit structured JSON when format is JSON (GAP-AUD-008).
- SCP remote-missing message normalized to `file not found: <path>` (GAP-AUD-025); EC 66 retained.
- Import TOML parse errors map to sysexits **65** (`TomlDe`) (GAP-AUD-012).
- `SshAuthentication` exit code aligned to **77** (GAP-AUD-020).
- Timeout values `< 1000` ms emit a stderr warning (GAP-AUD-009).
- `--include-secrets` to pipe/non-TTY requires `--output` or `--i-understand-secrets-on-stdout` (GAP-AUD-011).
- Doctor `secrets_plaintext_opt_out` is JSON **bool** (GAP-AUD-013).

### Added
- CLI flags: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (env layers deprecated, still work) (GAP-AUD-006).
- Event `secrets-key-auto-created` when primary key is provisioned on first write (GAP-AUD-007).
- Tunnel `--bind` (default `127.0.0.1`) (GAP-AUD-018).
- Password-on-argv stderr warning (GAP-AUD-010).

### Changed
- Version **0.5.0 → 0.5.1**.
- Tracing / residual identifiers standardized to English (GAP-AUD-005).
- Portuguese type aliases in `erros` module marked deprecated (GAP-AUD-017).

### Notes
- No crates.io/GitHub publish without explicit maintainer OK.
- SCP real transfer contracts from 0.5.0 §1.1 must not regress.

## [0.5.0] - 2026-07-15

### Fixed
- **CRITICAL**: `secrets init --force` now re-encrypts existing host secrets under the new primary key and writes `secrets.key.bak` (GAP-AUD-SEC-001).
- Doctor `permissions` field uses English `"missing"` instead of Portuguese `"ausente"`.
- Technical error messages, clap help residual PT, and product identifiers standardized to English.
- VPS names with internal whitespace are rejected (GAP-AUD-VAL-001).

### Changed
- Semver **0.5.0**: English renames of public/lib identifiers and residual API surface (`generate_completions`, `plaintext_allowed`, `ENC_PREFIX`, `verify_tofu`, etc.). Wire TOML keys were still Portuguese via `serde(rename)` in this release (**superseded in 0.5.1** by English serialize + dual-read EN/PT aliases, schema v3).
- `secrets init` / `secrets reencrypt` success lines go through `Message` i18n.

### Notes
- No crates.io/GitHub publish in this change set without explicit maintainer OK.

## [0.4.2] - 2026-07-15

### Fixed
- **Tunnel ephemeral port** (`local_port=0`): after bind, JSON/banner report the **OS-assigned** port via `local_addr()` (never `0` post-bind) (GAP-SSH-TUN-003). Schema `local_port.minimum` is 1.
- **SCP remote missing** now exits **66** `ArquivoNaoEncontrado` (parity with local missing) instead of **74** `CanalFalhou` when OpenSSH reports `No such file` / `not found` (GAP-SSH-IO-010). Protocol/permission errors remain 74.

### Added
- `vps export --json` agent-first envelope: `event: "vps-export"`, redacted hosts by default, no `sshcli-enc:` for empty secrets (GAP-SSH-UX-001 / EXP-001 parity); schema `docs/schemas/vps-export.schema.json`
- Commit hash embed for crates.io: `build.rs` precedence `SSH_CLI_COMMIT_HASH` → `.commit_hash` pack file → git → `unknown` (GAP-SSH-REL-007)
- Official e2e **E15** (tunnel port 0) + **E16** (symlink) + E13 asserts exit **66**; ENV-001 fail2ban policy in script header (GAP-SSH-ENV-001, SCP-024)
- Suite `tests/gaps_v042_integration.rs`

### Changed
- Version 0.4.1 → **0.4.2**
- Product-line docs + skills: tunnel remains **positional** args; port `0` = ephemeral; trust JSON `local_port` after bind; never invent `--local-port` (GAP-SSH-DOC-042)

### Security / honesty
- VPS TCP ban after audit e2e was **fail2ban** from intentional wrong-password tests (ENV-001), **not** TUN-003. Whitelist/ignoreip is ops; CLI one-shot does not open sshd permanently.
- No telemetry

### Notes
- One-shot CLI: birth → execute → die
- Additive agent contracts only (PATCH)


## [0.4.1] - 2026-07-15

### Fixed
- **Export redacted empty secret** no longer emits `sshcli-enc:v1:…` ciphertext for password `""` (GAP-SSH-EXP-001). Empty secrets serialize as empty strings so cross-machine import of skeletons stays honest.
- **Tunnel one-shot deadline** after local bind no longer returns exit **74** `TimeoutSsh` when the agent already received `tunnel_listening` (GAP-SSH-TUN-002). Pre-bind timeout remains exit 74.

### Added
- `tunnel` auth flag parity with exec/scp: `--password-stdin`, `--key-passphrase`, `--key-passphrase-stdin` (GAP-SSH-CLI-005)
- `health-check` auth flag parity: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` (GAP-SSH-CLI-006)
- SCP success JSON field `event: \"scp-transfer\"` + schema required (GAP-SSH-IO-009)
- Suite `tests/gaps_v041_integration.rs` (AUD-POST regression)
- `health-check` honors global `--replace-host-key` and enables JSON error envelope with `--json`

### Changed
- Version 0.4.0 → **0.4.1**
- Product-line docs + skills document tunnel/health auth parity and scp-transfer event

### Security / honesty
- **If you installed 0.4.0 from crates.io:** redacted `vps export` could show fake empty-password ciphertext; tunnel agents could see `ok:true` then exit 74. Upgrade to **0.4.1**.
- No telemetry

### Notes
- One-shot CLI: birth → execute → die
- Additive agent contracts only (PATCH)

## [0.4.0] - 2026-07-15

### Fixed
- **SCP wire protocol** was broken on crates.io **0.3.9** (header used literal `\\n` instead of real newline `0x0a`; ACK/EOF sent empty data instead of byte `0x00`; status not validated; download header/terminator incorrect) — SCP-010..013
- Remote SCP path shell-escape for spaces and meta-characters (SCP-014)
- Unit tests no longer crystalize the broken header form (SCP-015)
- Download no longer leaves a partial final file on failure: write `{path}.ssh-cli.partial` then atomic rename (SCP-022); mode/times applied on the **partial** before rename (SCP-022b)
- Upload no longer loads the entire file into RAM (`fs::read`); streams in 32 KiB chunks (SCP-018)
- `scp --json` enables the JSON error envelope on stderr (parity with tunnel; IO-007b)
- SCP file-only validation messages are i18n EN/PT (SCP-020b)

### Added
- Official e2e E10–E14 SCP coverage in `scripts/e2e_real_ssh.sh` (upload, download, `cmp`, missing remote, mode/mtime preserve) (SCP-016, SCP-023)
- SCP flag parity with exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` (SCP-017)
- Structured SCP success JSON + `docs/schemas/scp-transfer.schema.json` (IO-007, SCP-021)
- Preserve mtime/mode bi-directional: remote `scp -tp`/`-fp`, `T` line + `C` mode parse, set_permissions + set_times (SCP-023/023b; e2e E14)
- `tunnel --json` emits structured `tunnel_listening` event after local bind (IO-008)
- i18n EN/PT success messages for SCP (SCP-020)
- Suite `tests/gaps_v040_integration.rs` (TEST-004)

### Changed
- Version 0.3.9 → **0.4.0**
- Product-line docs document **regular files only** (no `-r` / no SFTP subsystem) and the 0.3.9 SCP wire regression (DOC-004, SCP-019, REL-004)
- Root docs honesty (SECURITY 0.4.x current, INTEGRATIONS real 0.4.0 surface, CONTRIBUTING gaps_v040) (DOC-004b)
- `docs/*` honesty: AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION/TESTING/RELEASE_CHECKLIST/CROSS_PLATFORM + schemas index cover SCP file-only, partial, 32 KiB stream, preserve, `scp --json`, `tunnel --json` / `tunnel_listening`, and 0.3.9 wire warning (DOC-004c)
- `skills/*` honesty: bilingual agent skills + evals teach SCP file-only, scp-transfer JSON, `.ssh-cli.partial`, 32 KiB stream, mtime/mode preserve, tunnel `--json` / `tunnel_listening`, timeout flag matrix (DOC-004d)
- Added `docs/schemas/tunnel-listening.schema.json` for IO-008 agent contract
- `scp` honors global `--replace-host-key` and global `--output-format json`

### Security / honesty
- **If you installed 0.3.9 from crates.io and used `scp`:** that release advertised SCP but the wire implementation was inoperant (upload often produced 0-byte remote files or timed out). Upgrade to **0.4.0**.
- No telemetry

### Notes
- One-shot CLI: connect → transfer → disconnect → exit
- Large files: raise `--timeout` (covers connect + full transfer)

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
- `docs/schemas/vps-show.schema.json` allows `password` type `string | null` (JSON-001 contract parity with runtime)
- Portuguese cross-language openers in `docs/*.pt-BR.md` use Portuguese narrative ("Leia este documento em inglês")
- Bilingual `docs/RELEASE_CHECKLIST.md` + `docs/RELEASE_CHECKLIST.pt-BR.md` with residual gates LOG/JSON/CLI/DOC/DENY/REL/CHG
- DOC-003 tests cover checklists and schema password null window
- Skills EN/PT consolidated as imperative operational formulas (LOG/JSON/CLI, error envelope, quiet, key-passphrase-stdin, port, full completions) without version changelog stories
- Workspace secret-hygiene residuals SEC-001..003: ignore `.setting.cyber/` fully, E2E refuses grok config inside the repo, docs use `demo-password-not-real` (not `s3cret`)

### Added
- Suite `tests/gaps_v039_integration.rs` for residual audit gaps (incl. SEC-001..003)

### Changed
- Version 0.3.8 → 0.3.9
- Cargo package `exclude` adds `.setting.cyber/` and enrich-queue sqlite sidecars

### Notes
- No telemetry
- Real credentials stay outside the tree (`~/.config/ssh-cli/`, `$HOME/.grok/config.toml`)

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

[Unreleased]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.2...HEAD
[0.5.2]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.9...v0.4.0
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
