# Release checklist: ssh-cli


> Mandatory gates before marking a release. The audit record is local and maintainer-only (not published).

- Read this document in [Portuguese (pt-BR)](RELEASE_CHECKLIST.pt-BR.md)
- Release target / product line: 0.5.6
- The `git` commands in this checklist are maintainer steps run by hand in the maintainer's own clone, because the source folder used to edit this document holds no git repository
- Historical gate: 0.4.1 DOC-041 / AUD-POST honesty (export empty, tunnel exit 0, auth parity, scp-transfer event)
- The audit inventory is local and not published, because it is gitignored and cargo-excluded
- Maintainers may keep it locally
- Tests must not assert its FIXED text (G13/G15)
- Residual suites: `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL/CHG)
- `tests/gaps_v040_integration.rs` (Secure Copy Protocol (SCP) 0.4.0)
- `tests/gaps_v041_integration.rs` (EXP-001, TUN-002, CLI-005/006, IO-009, REL-006)
- `tests/gaps_v042_integration.rs` (AUD-E2E)
- `tests/gaps_v051_integration.rs` (0.5.2 wire/export/secrets)
- `tests/gaps_v057_sftp.rs` (G-SFTP)
- `tests/gaps_v058_e2e_residual.rs` (G-E2E residual: schema/doctor root, single `vps-added`, `--use-agent`, help/clap env purge, ACME 64)
- `tests/gaps_v065_exec_target_designation.rs` (GAP-SSH-EXEC-ARGC-001: the exec family and `ssh-cli health-check` must be told their target in argv)
- `tests/gaps_v066_transfer_target_designation.rs` (the same rule extended to `scp` and `sftp`)
- `tests/gaps_v067_packaging_reach.rs` (every source file must survive `cargo package`, because a `.gitignore` pattern that swallows a source directory is invisible to every local build)
- `tests/gaps_v068_source_language.rs` (non-ASCII prose outside a citation, the half `scripts/check_en_identifiers.py` cannot read)
- `tests/gaps_v069_markdown_tables.rs` (a table row whose unescaped pipes disagree with its delimiter row renders with phantom columns, so no prose gate could see it)
- `tests/gaps_v070_doc_reference_reach.rs` (a document naming a suite the repository never had, which the forward sweep cannot see)
- `tests/gaps_v071_vps_edit_rename.rs` (native rename, eight clear flags, dry-run, lock wait)
- `tests/gaps_v072_edit_truthfulness.rs`, `tests/gaps_v072_import_roundtrip.rs`, `tests/gaps_v072_lock_contention.rs`, `tests/gaps_v072_schema_conformance.rs` and `tests/gaps_v072_storage_hardening.rs` (truthful `updated`, import round trip, lock exit 69, real envelopes against the schemas, file modes)
- `tests/gaps_v073_vps_auth_switch.rs`, `tests/gaps_v073_host_name_charset.rs`, `tests/gaps_v073_terminal_escape.rs` and `tests/gaps_v073_config_dir_mode.rs` (the four 0.5.6 suites: auth switch, strict host names, terminal escape, config directory mode)
- `tests/gaps_v074_flag_reach.rs` and `tests/gaps_v075_markdown_format_gate.rs` (each documented flag proven by behaviour, and the formatting gate kept in the battery)


## Purpose
- Prevent shipping with open gaps, stale product-line docs, or supply-chain waivers
- Keep release evidence honest (pre/post-fix notes in inventory, no secrets in logs)
- Align Cargo version, `--version`, docs product line, tags, and CHANGELOG anchors


## Gates (required)
- Gate 0: full gate battery with `python3 scripts/check_all_gates.py`, which exits 0 with all fourteen gates green and zero skipped
- `--list` prints the gate ids, so count them there rather than trusting this sentence
- Twelve of the fourteen ids map to the items below: `fmt` is Gate 6b, `build-release` is Gate 1, `build-no-default` is Gate 1b, `clippy` is Gate 2, `test` is Gate 6, `exec-target` is the dangerous-argv suite of Gate 7, `deny` is Gate 4, `cross-targets` is Gate 4c, `advisory-freshness` is Gate 4d, `advisory-hits` is Gate 4e, `en-identifiers` is Gate 3 and `install-resolve` is Gate 5, while `doc` and `markdown-format` run only inside the battery
- The `doc` gate runs `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features`, so a rustdoc warning or a broken intra-doc link fails the battery
- Its `test` gate runs `cargo test --locked --all-features --no-fail-fast`, so a red run lists every failing test instead of stopping at the first
- Its stdout has one tab-separated line per gate, with the id, `pass` or `fail`, the exit code and the milliseconds, then a `_summary` line with `total=`, `failed=` and `skipped=`
- Its exit is 0 when every selected gate passes, 1 when any gate fails and 2 on a usage error such as an unknown gate id
- Run it before ticking anything below, because `cargo clippy` and `cargo test` abort on the first unbuildable target and a single broken test file hides the state of every gate behind it
- `scripts/check_advisory_freshness.py` has no other caller, so skipping the battery skips that gate entirely
- Gate 1: release build with `cargo build --release` exits 0
- `cargo build --locked --release` and `cargo build --locked --all-features` print no `warning:` line, because the manifest no longer carries `package.homepage`, the unused `serde_with` or a redundant `human-panic` cfg
- Gate 1b: build without default features with `cargo build --no-default-features` exits 0 (the Windows path, A8)
- Gate 2: clippy clean with `cargo clippy --all-targets --all-features -- -D warnings -W clippy::doc_markdown` exits 0
- Gate 3: English identifiers with `python3 scripts/check_en_identifiers.py` exits 0
- Gate 4: supply chain deny (DENY-002) with `cargo deny check` exits 0, no russh CVE `ignore`, `yanked = "deny"` and an empty `ignore = []`
- `cargo audit` exits 0 when `cargo-audit` is installed, and `rg -c 'rust_decimal|rkyv' Cargo.lock Cargo.toml` finds nothing (RUSTSEC-2026-0235 left with the removed `rust_decimal`)
- Gate 4b: G-TLS crypto policy, with local gates only
  - `deny.toml` bans `openssl`, `openssl-sys`, `native-tls`, `libssh2-sys`, `ring`, `ssh2`, `thrussh` and `libssh-rs`
  - `rustls` is allowed on purpose, because the default `tls` feature (SSH-over-TLS, mTLS, ACME) uses it with the `aws_lc_rs` provider only, the manifest asks `rustls` 0.23.25 and `ssh-cli tls provider` prints the security floor `min_rustls` 0.23.18
  - `cargo tree -i openssl`, `-i native-tls`, `-i ring` and `-i flate2` report no package
  - SSH compression is `none` only, and the russh features do not enable `flate2`
  - SECURITY.md has the Transport & crypto policy (G-TLS), where SSH is not TLS and the crypto backend is aws-lc-rs
  - Every build host needs a C compiler, because `aws-lc-sys` compiles C, and the migration to pure Rust cryptography was tried and reverted, so no `[patch.crates-io]` and no `vendor-patches/` folder remain
- Gate 4c: cross-target build (B1, required) with `python3 scripts/check_cross_targets.py` exits 0 (`--timeout-secs N` bounds each cargo call)
  - Type-checks `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` and `x86_64-apple-darwin`
  - Every other gate in this list runs for the host triple only
  - Foreign `#[cfg(target_os = ...)]` code is dropped before type-check, so it is invisible to them
  - The Windows target once failed with six errors while this entire checklist was green
  - Windows uses `--no-default-features`, because the default TLS stack pulls `aws-lc-sys`, which compiles C (A8)
  - `docs/CROSS_PLATFORM.md` must not claim a target this script does not check, and it lists `aarch64-apple-darwin` as not covered
- Gate 4d: advisory freshness with `python3 scripts/check_advisory_freshness.py` exits 0
  - It fails when the RustSec advisory database clone is older than the accepted age, 7 days by default
  - `--db-root` points at the folder that holds the cargo-deny clones
- Gate 4e: advisory hits with `python3 scripts/check_advisory_hits.py` exits 0
  - It runs `cargo audit --json --no-fetch` against `Cargo.lock` and fails on any advisory, naming package, version, RUSTSEC id and the patched version
  - It refuses to run over a database older than 7 days, so a green result always means a fresh intersection and never a stale one
  - The fix it expects is a raised pin in `Cargo.toml`, never an entry in `deny.toml` `[advisories] ignore`
- Gate 5: install resolve with `python3 scripts/verify_install_resolve.py` exits 0, with russh at the security floor (0.65.0 or later, for RUSTSEC-2026-0492 and RUSTSEC-2026-0495) and `Cargo.lock` on 0.65.1
- Gate 6: full tests with `cargo test --locked --all-features --no-fail-fast` green: the lib plus every file in `tests/`
- Gate 6b: formatting (G10) with `cargo fmt --check` exits 0
- Gate 7: gap residual suites green, meaning every test in `tests/gaps_v038_integration.rs`, `tests/gaps_v039_integration.rs`, `tests/gaps_v040_integration.rs`, `tests/gaps_v041_integration.rs`, `tests/gaps_v042_integration.rs`, `tests/gaps_v051_integration.rs`, `tests/gaps_v057_sftp.rs`, `tests/gaps_v058_e2e_residual.rs`, `tests/gaps_v064_gate_runner.rs`, `tests/gaps_v065_exec_target_designation.rs`, `tests/gaps_v066_transfer_target_designation.rs`, `tests/gaps_v067_packaging_reach.rs`, `tests/gaps_v068_source_language.rs`, `tests/gaps_v069_markdown_tables.rs`, `tests/gaps_v070_doc_reference_reach.rs`, `tests/gaps_v071_vps_edit_rename.rs`, `tests/gaps_v072_edit_truthfulness.rs`, `tests/gaps_v072_import_roundtrip.rs`, `tests/gaps_v072_lock_contention.rs`, `tests/gaps_v072_schema_conformance.rs`, `tests/gaps_v072_storage_hardening.rs`, `tests/gaps_v073_config_dir_mode.rs`, `tests/gaps_v073_host_name_charset.rs`, `tests/gaps_v073_terminal_escape.rs`, `tests/gaps_v073_vps_auth_switch.rs`, `tests/gaps_v074_flag_reach.rs`, `tests/gaps_v075_markdown_format_gate.rs` and `tests/docs_conformance.rs`
- That includes tests not named `gap_*`
- This list is the minimum, because Gate 6 runs every other file in `tests/` too
- `the_test_gate_stays_locked` in `tests/gaps_v064_gate_runner.rs` fails when the `test` gate loses `--locked` or `--no-fail-fast`
- Gate 8: local e2e without a real VPS: help, fake VPS CRUD and completions behave as documented
- Gate 9: real VPS smoke when available: `ssh-cli health-check` / `ssh-cli exec` plus SCP matrix E10–E14 and SFTP E17/E18 (full matrix E01–E21) via `python3 scripts/e2e_real_ssh.py`
  - Use `--host --port --user --password-file`, or `--local-sshd` on Unix only, when credentials exist
  - Prefer local sshd or a throwaway VPS, with no auth-failure storms on production
  - Record the outcome in local maintainer notes without secrets (do not publish secrets, and the audit record is local-only)
- Gate 10: inventory not published
  - The audit record MUST NOT be published, because it is gitignored and cargo-excluded (one rule in `.gitignore` and one in the `Cargo.toml` exclude)
  - `rg -n -F '/gaps.md' .gitignore` must find the rule (exit 0)
  - FORBIDDEN: circular tests that assert FIXED text inside the audit record (G13/G15)
- Gate 11: honest pre/post-fix evidence for maintainers (DOC-002 / inventory integrity), with local notes only and no shipped secrets
- Gate 12: version string (REL-002), where `ssh-cli --version` matches the Cargo version plus the git hash and reports `-dirty` when the tree is dirty
  - The hash resolves env, then git, then `.commit_hash`, then `unknown`
  - Without env and git the file counts only when its version equals the package version
- Gate 13: local release commit and tag (REL-003), a maintainer step in the maintainer's own clone
  - Clean `git status` for the release commit
  - HEAD message is Release, and the local tag is `vX.Y.Z` (for 0.5.6: `v0.5.6`)
  - No remote push unless authorized
- Gate 13b: `.commit_hash` refresh (required after the release commit), because git lives outside this directory and the file cannot hold the release commit until that commit exists
  - After the release commit, write `<short hash> <version>` (for 0.5.6: the short hash of the release commit, one space, `0.5.6`) into `.commit_hash`
  - Rebuild with `cargo build --release`, then confirm `ssh-cli --version` shows that hash
  - Until this step runs, a build without git prints `unknown`, because the file only counts when its version equals the package version
- Gate 14: no telemetry, where `ssh-cli vps doctor --json` reports `"telemetry": false` under `local` and no metrics or telemetry SDKs are in the tree
- Gate 15: temporary probes removed, with no leftover `_probe_*` artifacts in the tree
- Gate 16: default tracing error (LOG-001)
  - Default level is error (not info), and tunnel/JSON mode stderr is envelope-only (no INFO progress banners such as "Tunnel SSH:" / "iniciando tunnel")
  - Graduated `-v`/`-vv`/`-vvv` is crate-scoped (G2/G14), and ambient `RUST_LOG` is ignored
- Gate 17: product-line docs match the Cargo version (DOC-003), so every product-line surface states 0.5.6
  - `llms.txt`, `llms.pt-BR.txt`, `llms-full.txt`
  - `README.md`, `README.pt-BR.md`
  - `INTEGRATIONS.md`, `INTEGRATIONS.pt-BR.md`
  - `docs/AGENTS.md`, `docs/AGENTS.pt-BR.md`
  - `docs/HOW_TO_USE.md`, `docs/HOW_TO_USE.pt-BR.md`
  - `docs/COOKBOOK.md`, `docs/COOKBOOK.pt-BR.md`
  - `docs/MIGRATION.md`, `docs/MIGRATION.pt-BR.md`
  - `docs/TESTING.md`, `docs/TESTING.pt-BR.md`
  - `docs/CROSS_PLATFORM.md`, `docs/CROSS_PLATFORM.pt-BR.md`
  - `docs/schemas/README.md`
  - `docs/RELEASE_CHECKLIST.md`, `docs/RELEASE_CHECKLIST.pt-BR.md`
- Gate 18: empty password is null in JSON (JSON-001)
  - Runtime: key-only `ssh-cli vps show --json` and `ssh-cli vps list --json` emit `"password": null` (not the mask string), and a non-empty password stays masked
  - Schema: `docs/schemas/vps-show.schema.json` (and list via `$ref`) declares `password` as `string` or `null`
- Gate 19: health-check timeout (CLI-004), where `ssh-cli health-check --timeout <ms>` is accepted (clap parse), aligned with exec overrides, and covered by gaps_v039
- Gate 20: CHANGELOG anchors (CHG-001)
  - `CHANGELOG.md` has the section `## [0.5.6]` and the compare/footer anchors for 0.5.6
  - The prior 0.5.5 / 0.5.4 / 0.5.3 / 0.5.2 / 0.4.x / 0.3.9 anchors stay as needed
- Gate 21: package dry-run with `cargo package --allow-dirty --list` succeeds, never auto-publish
  - The listing has no `*.sh` file, no file named `json`, no `__pycache__` and no `*.snap.new`
- Gate 22: DOC-004 / SCP honesty (0.4.0+), where product-line surfaces document:
  - SCP regular files only (no directories / no `-r`)
  - Trees via `sftp --recursive`
  - crates.io 0.3.9 advertised SCP but the wire was broken
  - Do not promise working SCP on 0.3.9
  - `docs/schemas/scp-transfer.schema.json` exists and is indexed (`docs/schemas/README.md`, `llms-full.txt`)
  - download partial suffix `.ssh-cli.partial`
  - `ssh-cli tunnel --json` / `tunnel_listening` and/or scp agent JSON surface in README/INTEGRATIONS/AGENTS
  - bilingual `skills/ssh-cli-en` and `skills/ssh-cli-pt` teach scp-transfer, tunnel_listening, file-only, partial, 32 KiB and the timeout matrix (DOC-004d)
  - SECURITY Supported Versions brands 0.5.x as the current line (not 0.3.x)
  - `cargo test --locked --test gaps_v040_integration` + `gaps_v041_integration` green
- Gate 23: DOC-041 / AUD-POST honesty (historical 0.4.x), where product-line and agent surfaces document:
  - redacted `ssh-cli vps export` never documents or expects `sshcli-enc:` for empty secrets
  - tunnel post-bind deadline exits 0 after `tunnel_listening` (one-shot success, not 74)
  - `ssh-cli tunnel` / `ssh-cli health-check` auth flags parity (`--password-stdin`, key / passphrase overrides as applicable)
  - `scp-transfer` schema requires `event: "scp-transfer"`
  - `cargo test --locked --test gaps_v041_integration` green
- Gate 24: DOC-051 / 0.5.2 honesty (historical), where product-line surfaces document:
  - `ssh-cli vps export` body follows the resolved format: JSON `event: "vps-export"` on any non-TTY stdout
  - TOML only with `--output-format text`
  - wire schema v3 dual-read (EN serialize / PT load aliases)
  - secrets schemas `secrets-init.schema.json` / `secrets-reencrypt.schema.json` indexed
  - tunnel `--bind` defaults to `127.0.0.1`
  - exit 77 for auth
  - exit 65 for `TomlDe` / bad import
  - secrets flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` preferred over env
  - `--include-secrets` requires `-o` or `--i-understand-secrets-on-stdout`
  - `cargo test --locked --test gaps_v042_integration` + `gaps_v051_integration` green
- Gate 25: DOC-053 / 0.5.3 honesty, where product-line surfaces document:
  - SFTP upload integrity fixed (G1)
  - Prefer 0.5.3+
  - Verify with destination `sha256`
  - SETSTAT atime+mtime (G3), fail-closed set_metadata (G4), perm mask (G12)
  - Graduated verbosity `-v`/`-vv`/`-vvv` crate-scoped (G2/G14)
  - `RUST_LOG` ignored
  - Batch cancel cardinality (G5/G17)
  - `ssh-cli exec --json` single object (G8)
  - SCP `sync_data` (G9)
  - E2E E17/E18 SFTP checksum/tree (G7)
  - No circular tests over the audit record (G13/G15)
  - Audit record gitignored
  - `cargo fmt --check` green (G10)
  - `cargo test --locked --test gaps_v057_sftp` + `gaps_v058_e2e_residual` green
  - Full CLI command tree documented in HOW_TO_USE + AGENTS + COOKBOOK (including `ssh-cli locale show`, `ssh-cli locale set` and `ssh-cli locale clear` + full `tls mtls/*` / `tls acme/*`)
  - G6 `serial_test` / deterministic residual suite isolation
  - G11 baseline green (`cargo test --locked --all-features --no-fail-fast` + residual suites)
  - G16 English SCP identifiers (wire/schema/docs use English field names)
  - G18 `set_permissions` fail-closed on SFTP download
  - G19 `SFTP_PERM_MASK` named constant (`0o7777`, outbound upload) and `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`, inbound download, A3)
  - `ssh-cli commands` discovery documented (agent inventory surface)
  - Product-line docs list `locale` + full `tls` tree (mtls + acme account/issue/complete/status/list)
- Gate 26: DOC-054 / 0.5.4 honesty, where product-line surfaces document:
  - all four `ssh-cli tunnel` modes: default local forward, `--reverse` (G-TUN-R01), `--socks5` (G-TUN-R02), `--remote-socket <PATH>` (G-TUN-R03)
  - The three flags are mutually exclusive
  - JSON `mode` label is one of `local` / `reverse` / `socks5` / `streamlocal`
  - `--reverse` with `REMOTE_PORT 0` lets the server allocate and report the port
  - A local forward cannot accept 0
  - `--socks5` and `--remote-socket` omit `REMOTE_HOST`/`REMOTE_PORT` (passing them is exit 64)
  - `--remote-socket` requires an absolute path
  - a routable `--bind` requires `--i-accept-network-exposure` (G-TUN-R13), including a local forward and not only `--reverse`
  - under `--reverse` the guard covers the server's bind (the positional `<remote_host>`), which is compared as text and not IP-parsed (typo gives exit 64, not exit 2)
  - `--bind` itself is accepted and silently discarded
  - `tunnel_closed` event with `reason`, `forwards_served`, `capacity_waits` is the only discriminator between the three exit-0 endings
  - payload shaping applied before serialization: `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
  - `--dry-run` is honoured by `ssh-cli vps remove`, `ssh-cli vps import`, `ssh-cli vps edit`, `ssh-cli vps rename`, `ssh-cli sftp rm`, `ssh-cli sftp rmdir`, `ssh-cli secrets init`, `ssh-cli secrets reencrypt` and only by those
  - Elsewhere it is exit 64
  - `--tags` fleet selector on `ssh-cli exec` / `ssh-cli sudo-exec` / `ssh-cli su-exec` only, mutually exclusive with `--all` and `--hosts`
  - BREAKING: partial multi-host failure exits 1 (was 65, G-ERR-R02)
  - A1 applies the pre-existing 512-character banner log cap on character boundaries (removes a remotely triggerable abort under `panic = "abort"`)
  - A2 creates ACME/mTLS keys at `0600`
  - A3 masks inbound modes with `SFTP_PERM_MASK_UNTRUSTED`
  - `cargo test --locked --test docs_conformance` green (including the 0.5.4 surface, fleet-selector and directional-mask gates)
- Gate 27: DOC-056 / 0.5.6 honesty, where product-line surfaces document:
  - `ssh-cli vps rename <NAME> <NEW_NAME>` and `ssh-cli vps edit <NAME> --name <NEW_NAME>` exist, keep every credential readable under the new name, and never retype a secret
  - the export/edit/import detour is obsolete: export redacts secrets and carried no `use_agent`/`agent_socket`, so an agent-authenticated host came back unauthenticated
  - eight clear flags: `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key`, `--no-key` (key path plus passphrase), `--no-agent-socket`, each conflicting with the value flag of its own field
  - `ssh-cli vps edit <NAME> --name <NEW_NAME>` and `ssh-cli vps rename` both emit `vps-renamed` (`name`, `previous_name`, `active_rewritten`, `updated`, `target_resolved`, `target_source`)
  - `vps-edited` is emitted only when nothing was renamed and carries no `previous_name`
  - Both publish `target_resolved`/`target_source` only after the lookup
  - `updated` is always present: fields whose stored value changed, in application order, each at most once, drawn from 19 field names
  - Empty when every requested value already matched or when the invocation only renamed the host
  - An invocation with no field, rename or clear flag exits 64
  - an edit that changes nothing exits 0 with `vps-edited` and `updated: []` and does not rewrite `config.toml`
  - `ssh-cli vps rename` is not idempotent (repeating it after a success exits 66, and `ssh-cli vps show <NEW_NAME>` confirms it)
  - `ssh-cli vps edit` switches the auth method: `--key` clears the password and the agent, `--password` and `--password-stdin` clear the key, its passphrase and the agent, `--use-agent` clears the password and the key, and `updated` lists what was cleared
  - a different `--key` drops the old passphrase, `--agent-socket` alone implies `--use-agent`, and a passphrase with no key exits 64
  - two primary auth flags in one call exit 2 at parse time
  - host names are strict: ASCII `[A-Za-z0-9._-]`, alphanumeric first, 1 to 255 bytes, on `ssh-cli vps add`, `ssh-cli vps rename`, `ssh-cli vps edit --name` and `ssh-cli vps import`, while older names still load
  - `ssh-cli vps import` lists every bad name at once with exit 64 and writes nothing, also under `--dry-run`
  - lookups that become a target (`ssh-cli exec`, `ssh-cli health-check`, `--hosts`, `ssh-cli connect`) refuse control and bidi characters with exit 64, and `ssh-cli tls mtls import --name` does the same without creating a directory
  - text mode escapes control and bidi characters as `\u{1b}` (including `ssh-cli sftp ls` and `ssh-cli sftp stat` remote names), while JSON and remote `ssh-cli exec` output stay raw
  - the config directory is born `0700`, an existing non-default directory is never chmodded, and `ssh-cli doctor --json` reports `local.config_dir_permissions`, `local.config_dir_exposed` and `local.invalid_host_names`
  - `--config-dir something.toml` makes the parent the config directory, and `ssh-cli vps export -o new/deep/e.json` creates the missing folders `0700`
  - two config tables that normalize to one name exit 65 with `config has two hosts named ...`
  - `--dry-run` plans of `ssh-cli vps edit`, `ssh-cli vps rename` and `ssh-cli vps remove` echo `target_resolved`, `target_source`, `host_resolved` and `host_source`
  - the nine `scripts/*.sh` are now `scripts/*.py` (Python 3.9+, stdlib only, Windows `py -3 scripts/<name>.py`) with flags instead of environment variables
  - `SshCliError::Unavailable` carries `UnavailableService`, `paths::validate_and_normalize` returns `String`, and `Money`, `Brl`, `Usd`, `Currency` and `rust_decimal` are gone (BREAKING for library users only, and the wire text is unchanged)
  - `--dry-run` covers the eight operations, and `--lock-wait-ms` bounds the config lock into exit 69 `unavailable` instead of an indefinite hang
  - `ssh-cli commands --json` publishes every argument with `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` and `hidden`
  - `ssh-cli --help` has an `Exit status:` section (0, 1, 2, 64, 65, 66, 69, 70, 73, 74, 77, 130, 141 and 143), and all 48 leaf commands show `Examples:` in `--help` while `-h` stays short
  - both skills teach the 48 leaf commands, `ssh-cli vps rename`, the auth switch, the SANITIZE step, the raw `sudo` rule, the TOFU rule and the persistent `--disable-sudo` form, within the 7000-word budget
  - the minimum supported Rust version (MSRV) is 1.89 because `russh` 0.65 declares it, and every build host needs a C compiler for `aws-lc-sys`
  - `secrets init --keyring` writes to the OS store through `keyring` 4.2.0 with the `v1` feature, exits 69 when no store answers, is scoped per config directory, and exits 64 instead of minting a second primary key on a profile that already has one
  - the global `--use-keyring` reaches `secrets init`, and the primary key resolves `--secrets-key-file`, then the keyring with `--use-keyring`, then XDG `secrets.key`, then the keyring entry of an earlier `secrets init --keyring`
  - `--select` and `--truncate-content` reduce a single record, the collection flags `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` and `--max-output-bytes` are named under `agent_shape.ignored` there, and a reduced bare array writes one `agent-shape` line to stderr that `--quiet` suppresses
  - `ssh-cli vps list --select` items satisfy `vps-list.schema.json`, text `ssh-cli vps list` sizes each column from its widest cell and shows `(not set)` for a key-only password, and text `ssh-cli vps show` aligns all its labels
  - `ssh-cli vps import --file` on a missing file exits 66 `file_not_found` as a permanent error, and a `ssh-cli tunnel` stopped by SIGINT or SIGTERM exits 130 or 143 with `reason: "signal"`
  - only `SSH_CLI_SECRETS_KEY` and `SSH_CLI_SECRETS_KEY_FILE` are refused, and the global `--disable-sudo` placed before `ssh-cli vps add` or `ssh-cli vps edit` saves `disable_sudo` in the record
  - `cargo test --locked --test gaps_v071_vps_edit_rename` and `cargo test --locked --test docs_conformance` green


## How to verify residuals quickly

```bash
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo test --locked --test gaps_v042_integration
cargo test --locked --test gaps_v051_integration
cargo test --locked --test gaps_v057_sftp
cargo test --locked --test gaps_v058_e2e_residual
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
cargo test --locked --test gaps_v073_vps_auth_switch
cargo test --locked --test gaps_v074_flag_reach
cargo test --locked --test gaps_v075_markdown_format_gate
cargo test --locked --test gaps_v073_host_name_charset
cargo test --locked --test gaps_v073_terminal_escape
cargo test --locked --test gaps_v073_config_dir_mode
cargo test --locked --test docs_conformance
cargo fmt --check
cargo deny check
python3 scripts/check_en_identifiers.py
python3 scripts/check_markdown_format.py
python3 scripts/verify_install_resolve.py
ssh-cli --version
rg -n -F '/gaps.md' .gitignore   # must find the rule (exit 0)
```

- LOG-001: tunnel with `--output-format json` fails without connecting
- Stderr has JSON envelope and no INFO prose
- JSON-001: key-only host show JSON contains `"password": null`
- Schema file contains null in password type
- CLI-004: `ssh-cli health-check --timeout 50` is not "unexpected argument"
- DOC-003: product-line files (including this checklist pair) contain `0.5.6`
- DOC-004: README/INTEGRATIONS/AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION mention file-only SCP and 0.3.9 wire warning
- Scp-transfer schema present
- DOC-004d: `skills/ssh-cli-en` and `skills/ssh-cli-pt` teach scp-transfer, tunnel_listening, file-only, partial, 32 KiB stream, and timeout matrix
- Evals cover the surface
- DOC-041: export redacted empty secrets have no `sshcli-enc:`
- Tunnel post-bind deadline exit 0 after `tunnel_listening`
- Tunnel/health auth flag parity documented
- Scp-transfer schema requires `event`
- Gaps_v041 green
- DOC-051: export redaction
- Format follows --output-format
- Schema v3
- Secrets schemas
- `--bind` loopback
- Exit 77
- Secrets flags
- Include-secrets
- Gaps_v042 + gaps_v051 green
- DOC-053 / 0.5.3: SFTP G1 integrity
- Verbosity G2/G14
- E17/E18
- Local audit record gitignored
- `cargo fmt --check`
- Gaps_v057 green
- Full CLI tree (locale + tls mtls/acme) in HOW_TO_USE/AGENTS/COOKBOOK
- G6 serial_test
- G11 baseline
- G16 EN SCP ids
- G18 set_permissions
- G19 `SFTP_PERM_MASK`
- `ssh-cli commands` discovery
- DENY-002: `deny.toml` has `yanked = "deny"`, `ignore = []`, multiple-versions policy documented
- CHG-001 / REL: CHANGELOG section `## [0.5.6]` + local tag `v0.5.6` without unauthorized push
- TEST-004 / SCP: gaps_v040 covers wire, schema, partial path, preserve, e2e script E10–E14
- AUD-POST / gaps_v041: EXP-001, TUN-002, CLI-005/006, IO-009, REL-006 residual suite green
- 0.5.2 / gaps_v051: export redaction roundtrip, `vps-export` JSON, dual-read, secrets-init event, include-secrets guard, CRUD `vps-added`, empty command, import exit 65
- G-E2E / gaps_v058: root `ssh-cli schema` / `ssh-cli doctor`, single `vps-added` + `secrets_key_auto_created`, `--use-agent`, ambient `RUST_LOG` ignored, ACME exit 64
- 0.5.6 / `gaps_v071_vps_edit_rename`, `gaps_v072_edit_truthfulness`, `gaps_v072_import_roundtrip`, `gaps_v072_lock_contention`, `gaps_v072_schema_conformance`, `gaps_v072_storage_hardening`, `gaps_v073_config_dir_mode`, `gaps_v073_host_name_charset`, `gaps_v073_terminal_escape` and `gaps_v073_vps_auth_switch`: rename, auth switch, strict names, terminal escape, config directory mode, schema conformance
- G10: `cargo fmt --check` green
- EN identifiers: `scripts/check_en_identifiers.py` exits 0
- Markdown format: `scripts/check_markdown_format.py` exits 0, closing what Gaps 25 and 37 named as their acceptance criterion


## Suite inventory
- Gate 6 runs the lib and these 50 files of `tests/`, one name per line so that no range is elided

```text
docs_conformance
e2e_cli
gaps_v035_integration
gaps_v037_integration
gaps_v038_integration
gaps_v039_integration
gaps_v040_integration
gaps_v041_integration
gaps_v042_integration
gaps_v051_integration
gaps_v053_domain_types
gaps_v054_error_handling
gaps_v055_unsafe_ffi
gaps_v057_sftp
gaps_v058_e2e_residual
gaps_v059_agent_native
gaps_v060_tunnel_modes
gaps_v061_error_taxonomy
gaps_v061_scp_durability
gaps_v062_component_budget
gaps_v062_i18n_reachability
gaps_v063_e2e_harness
gaps_v063_locale_features
gaps_v063_secret_stdin
gaps_v064_gate_runner
gaps_v065_exec_target_designation
gaps_v066_transfer_target_designation
gaps_v067_packaging_reach
gaps_v068_source_language
gaps_v069_markdown_tables
gaps_v070_doc_reference_reach
gaps_v071_vps_edit_rename
gaps_v072_edit_truthfulness
gaps_v072_import_roundtrip
gaps_v072_lock_contention
gaps_v072_schema_conformance
gaps_v072_storage_hardening
gaps_v073_config_dir_mode
gaps_v073_host_name_charset
gaps_v073_terminal_escape
gaps_v073_vps_auth_switch
gaps_v074_flag_reach
gaps_v075_markdown_format_gate
i18n_integration
proptest_tests
scp_integration
snapshot_tests
storage_integration
test_quality
tunnel_integration
```


## Policy
- FORBIDDEN: declare inventory Fechado (closed) while any gap remains Aberto (open)
- FORBIDDEN: eternal RUSTSEC / CVE waive without closed tracking in the same release
- FORBIDDEN: `git push` or crates.io publish without explicit maintainer authorization
- FORBIDDEN: log or paste real secrets into inventory, checklist notes, or gate logs
- FORBIDDEN: publish the audit record or assert its FIXED text from product tests (G13/G15)
- REQUIRED: multi-line inventory / CHANGELOG writes use atomwrite (or equivalent atomic write)
- REQUIRED: Status Resolvido only with code + test + version note (destination-effect evidence, not inventory FIXED text)


## G-22: Distribution, SBOM, multi-arch (process)
- Product identity remains one-shot CLI (crates.io + local tag)
- Binary multi-arch and signed SBOM are release-process gates, not runtime features
- Scripts live under `scripts/` and never push or publish

### 28. Multi-arch release binaries (G-22)
- Config: root [`Cross.toml`](../Cross.toml) targets:
  - `x86_64-unknown-linux-musl`
  - `aarch64-unknown-linux-musl`
  - `aarch64-unknown-linux-gnu`
- Tooling: install [`cross`](https://github.com/cross-rs/cross) and the container engine it needs
- Build matrix (local, no push):

```bash
python3 scripts/dist_multiarch.py
# or a single target:
python3 scripts/dist_multiarch.py --targets x86_64-unknown-linux-musl
```

- Artifacts: `target/dist/ssh-cli-<triple>` + `.sha256` sidecars
- `--targets` takes a space- or comma-separated list, `--jobs` bounds the parallel builds (default 2) and `--timeout-secs` bounds each target (default 3600, `0` for none)
- The stdout has one tab-separated line per target with the target, `ok` or `fail` and the artifact path, and the exit is 0 when every target built, 1 otherwise and 2 on a usage error
- `cross` + `dist_multiarch.py` is the supported path
- Without `cross` the script tries `cargo build --release --locked --target <triple>`, which needs a local linker and sysroot
- FORBIDDEN: attach unstripped debug builds as “release” without documenting

### 29. SBOM generation (G-22)

```bash
# Preferred: CycloneDX JSON
cargo install cargo-cyclonedx
python3 scripts/generate_sbom.py
# → target/sbom/ssh-cli.cdx.json (or path argument)
```

- Fallback without cargo-cyclonedx: the script writes a `cargo tree` inventory to a `.tree.txt` file named after the output path and warns that it is not a CycloneDX SBOM
- Signing (maintainer, offline):

```bash
# Example with a local cosign key or with gpg: choose one standard
# cosign sign-blob --key <path-to-cosign.key> --bundle target/sbom/ssh-cli.cdx.json.bundle target/sbom/ssh-cli.cdx.json
# gpg --detach-sign --armor target/sbom/ssh-cli.cdx.json
```

- Attach SBOM + signature + multi-arch binaries to the release page of tag `vX.Y.Z` only after explicit maintainer authorization (same rule as `git push`)
- crates.io publish remains separate (`cargo publish`)
- SBOM is release evidence, not a crate file

### 30. G-22 acceptance criteria (close inventory)
- [ ] `python3 scripts/dist_multiarch.py` produces at least the musl x86_64 artifact + sha256 (or a documented skip when no container engine is available)
- [ ] `python3 scripts/generate_sbom.py` produces CycloneDX JSON or the documented fallback inventory
- [ ] `python3 scripts/release_attest.py` prints the attestation checklist and exits 0, and `--require-cosign` exits 1 when `cosign` is not on the PATH
- [ ] Release notes (when authorized) list binary triples + SBOM path
- [ ] No auto-push / no auto-publish from scripts


## Reference
- Audit record: local-only maintainer inventory (gitignored / cargo-excluded, not published)
- [../Cross.toml](../Cross.toml): cross-rs target images
- [../scripts/dist_multiarch.py](../scripts/dist_multiarch.py): multi-arch build
- [../scripts/generate_sbom.py](../scripts/generate_sbom.py): SBOM / inventory
- [../deny.toml](../deny.toml): supply-chain policy
- [../scripts/check_all_gates.py](../scripts/check_all_gates.py): mandatory gate battery runner (all fourteen gates, one invocation)
- [../scripts/check_cross_targets.py](../scripts/check_cross_targets.py): Windows and macOS type-check gate
- [../scripts/check_advisory_freshness.py](../scripts/check_advisory_freshness.py): advisory database freshness gate
- [../scripts/verify_install_resolve.py](../scripts/verify_install_resolve.py): install re-resolve gate
- [../scripts/check_en_identifiers.py](../scripts/check_en_identifiers.py): English identifier residual gate
- [../scripts/check_markdown_format.py](../scripts/check_markdown_format.py): published Markdown formatting gate
- [../tests/gaps_v039_integration.rs](../tests/gaps_v039_integration.rs): residual gates LOG/JSON/CLI/DOC/DENY/CHG
- [../tests/gaps_v040_integration.rs](../tests/gaps_v040_integration.rs): residual gates SCP/IO/DOC-004/REL-004
- [../tests/gaps_v041_integration.rs](../tests/gaps_v041_integration.rs): residual gates EXP-001/TUN-002/CLI-005/006/IO-009/REL-006 (DOC-041)
- [../tests/gaps_v042_integration.rs](../tests/gaps_v042_integration.rs): residual gates AUD-E2E (TUN-003, IO-010, ENV-001, SCP-024, …)
- [../tests/gaps_v051_integration.rs](../tests/gaps_v051_integration.rs): residual gates 0.5.2 export/schema v3/secrets
- [../tests/gaps_v057_sftp.rs](../tests/gaps_v057_sftp.rs): residual gates G-SFTP
- [../tests/gaps_v058_e2e_residual.rs](../tests/gaps_v058_e2e_residual.rs): residual gates G-E2E (schema/doctor root, single `vps-added`, `--use-agent`, ACME 64)
- [schemas/vps-show.schema.json](schemas/vps-show.schema.json): password `null` or the masked string
- [schemas/scp-transfer.schema.json](schemas/scp-transfer.schema.json): SCP success JSON (files only, requires `event`)
- [schemas/tunnel-listening.schema.json](schemas/tunnel-listening.schema.json): tunnel bind event
- [schemas/vps-export.schema.json](schemas/vps-export.schema.json): `ssh-cli vps export --json` only (`event: "vps-export"`)
- [schemas/secrets-init.schema.json](schemas/secrets-init.schema.json): `ssh-cli secrets init --json`
- [schemas/secrets-reencrypt.schema.json](schemas/secrets-reencrypt.schema.json): `ssh-cli secrets reencrypt --json`
- [schemas/README.md](schemas/README.md): schema index (product line 0.5.6)


## Residual G-E2E docs gate (v0.5.3)
- [ ] Ambient `RUST_LOG` ignored documented
- Only `-v`/`-vv`/`-vvv` for verbosity (G2/G14)
- [ ] Single JSON `vps-added` with field `secrets_key_auto_created` (G8 family)
- [ ] Root `ssh-cli schema` / `ssh-cli doctor` documented
- [ ] `ssh-cli vps add --use-agent` documented
- [ ] ACME `invalidContact` → exit 64 permanent documented
- [ ] Redacted export clears secrets (`password` is an empty string, the others are `null`) documented
- [ ] E2E XDG-first + SKIP offline documented in TESTING
- [ ] E2E E17/E18 SFTP checksum/tree documented (G7)
- [ ] SFTP G1 integrity + prefer 0.5.3+ documented
- [ ] Full CLI command tree (including `locale` + full `tls mtls` / `tls acme`) documented in HOW_TO_USE + AGENTS + COOKBOOK
- [ ] `ssh-cli commands` discovery documented
- [ ] G6 `serial_test` / deterministic suite
- [ ] G11 baseline green
- [ ] G16 English SCP identifiers
- [ ] G18 `set_permissions` fail-closed on SFTP download
- [ ] G19 `SFTP_PERM_MASK` named constant
- [ ] Product-line docs list `locale` + full `tls` tree
- [ ] Audit record gitignored (`rg -n -F '/gaps.md' .gitignore` finds the rule)
- [ ] No circular FIXED-text tests (G13/G15)
- [ ] `cargo fmt --check` in publish gates (G10)
- [ ] Suite `tests/gaps_v057_sftp.rs` + `tests/gaps_v058_e2e_residual.rs` in publish gates


## Multi-OS local matrix (G-E2E-18)
- Product code: `src/platform/{linux,macos,windows}.rs`, validated by local scripts
- Local multi-arch: `scripts/dist_multiarch.py` when cross toolchains are installed
- Validate path length / agent socket notes on macOS and Windows before tagging a release
- Every gate is a local script run by hand, with no cloud service in the loop


## Release Notes
### 0.5.6 — edit and rename release
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
