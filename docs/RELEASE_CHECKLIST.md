# Release checklist — ssh-cli

> **0.5.4** — security and agent-native release. Fixes a remote pre-auth DoS in the SSH banner path (A1), stops server-sent setuid bits landing on downloaded files (A3), closes the world-readable window on ACME/mTLS private keys (A2), and adds payload-shaping flags (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) applied before serialization. BREAKING: partial multi-host failure now exits **1** (was 65); a non-loopback `--bind` requires `--i-accept-network-exposure`. New `tunnel_closed` event.


> Mandatory gates before marking a release. Local `gaps.md` is maintainer-only (not published).

- Read this document in [Portuguese (pt-BR)](RELEASE_CHECKLIST.pt-BR.md).
- Release target / product line: **0.5.4**.
- Historical gate: **0.4.1** DOC-041 / AUD-POST honesty (export empty, tunnel exit 0, auth parity, scp-transfer event).
- Local audit inventory (not published): `gaps.md` — **gitignored** and cargo-excluded; maintainers may keep it locally. Tests must not assert its FIXED text (G13/G15).
- Residual suites: `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL/CHG); `tests/gaps_v040_integration.rs` (SCP 0.4.0); `tests/gaps_v041_integration.rs` (EXP-001, TUN-002, CLI-005/006, IO-009, REL-006); `tests/gaps_v042_integration.rs` (AUD-E2E); `tests/gaps_v051_integration.rs` (0.5.2 wire/export/secrets); `tests/gaps_v056_ssh.rs` + `tests/gaps_v057_sftp.rs` (G-SSH / G-SFTP); `tests/gaps_v058_e2e_residual.rs` (G-E2E residual: schema/doctor root, single `vps-added`, `--use-agent`, help/clap env purge, FIXED_MASK, ACME 64).


## Purpose
- Prevent shipping with open gaps, stale product-line docs, or supply-chain waivers.
- Keep release evidence honest (pre/post-fix notes in inventory, no secrets in logs).
- Align Cargo version, `--version`, docs product line, tags, and CHANGELOG anchors.


## Gates (required)

0. Full gate battery — `bash scripts/check_all_gates.sh` exits 0 with all ten gates green and zero skipped. It covers items 1 to 5 plus the cross-target check, the advisory freshness gate and the `--no-default-features` build in one invocation. Run this before ticking anything below: `cargo clippy` and `cargo test` abort on the first unbuildable target, so a single broken test file hides the state of every gate behind it. `scripts/check_advisory_freshness.sh` has no other caller, so skipping the battery skips that gate entirely.
1. Release build — `cargo build --release` exits 0.
2. Clippy clean — `cargo clippy --all-targets -- -D warnings` exits 0.
3. English identifiers — `bash scripts/check_en_identifiers.sh` exits 0.
4. Supply chain deny (DENY-002) — `cargo deny check` exits 0; no russh CVE `ignore`; `yanked=deny`; empty `ignore = []`.
4b. **G-TLS crypto policy (local gates only — no required GitHub Actions):**
    - `deny.toml` bans `openssl`, `openssl-sys`, `native-tls`, `libssh2-sys`, `ring`, `rustls`.
    - `cargo tree -i rustls`, `-i openssl`, `-i ring`, `-i native-tls` report no package (empty tree).
    - `cargo tree -i flate2` reports no package (SSH compression `none` only; no russh flate2 feature).
    - `cargo test --locked --test gaps_v052_tls_policy` green.
    - SECURITY.md has **Transport & crypto policy (G-TLS)** (SSH ≠ TLS; aws-lc-rs; compression none).
4c. **Cross-target build (B1 — required):** `bash scripts/check_cross_targets.sh` exits 0.
    - Type-checks `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` and `x86_64-apple-darwin`.
    - Every other gate in this list runs for the host triple only.
    - Foreign `#[cfg(target_os = ...)]` code is dropped before type-check, so it is invisible to them.
    - The Windows target once failed with six errors while this entire checklist was green.
    - Windows uses `--no-default-features`: the default TLS stack pulls `aws-lc-sys`, which compiles C (A8).
    - `docs/CROSS_PLATFORM.md` must not claim a target this script does not check.
5. Install resolve — `bash scripts/verify_install_resolve.sh` exits 0; russh at security floor (≥ 0.60.3; product line uses 0.62.5).
6. Full tests — `cargo test --locked --all-targets` green (lib + integration + gaps_v037…v042 + **gaps_v051** + **gaps_v052** + **gaps_v056** + **gaps_v057** + **gaps_v058**).
6b. Formatting (G10) — `cargo fmt --check` exits 0.
7. Gap residual suites green — every test in `tests/gaps_v038_integration.rs`, `tests/gaps_v039_integration.rs`, `tests/gaps_v040_integration.rs`, `tests/gaps_v041_integration.rs`, `tests/gaps_v042_integration.rs`, `tests/gaps_v051_integration.rs`, `tests/gaps_v052_tls_policy.rs`, `tests/gaps_v056_ssh.rs`, `tests/gaps_v057_sftp.rs`, `tests/gaps_v058_e2e_residual.rs`, and `tests/gaps_v064_gate_runner.rs` passes (including tests not named `gap_*`).
8. Local e2e (no real VPS) — help, fake VPS CRUD, completions behave as documented.
9. Real VPS smoke (when available) — `health-check` / `exec` plus SCP matrix **E10–E14** and SFTP **E17/E18** (full matrix **E01–E18**) via `scripts/e2e_real_ssh.sh` when credentials exist; prefer local sshd / throwaway VPS; no auth-failure storms on production; record outcome in local maintainer notes without secrets (do not publish secrets; `gaps.md` is local-only).
10. Inventory not published — `gaps.md` **MUST NOT** be published; it is **gitignored** and cargo-excluded (`/gaps.md` in `.gitignore` and `Cargo.toml` exclude). `git check-ignore gaps.md` **must match** (exit 0). FORBIDDEN: circular tests that assert FIXED text inside `gaps.md` (G13/G15).
11. Honest pre/post-fix evidence for maintainers (DOC-002 / inventory integrity) — keep local notes only; do not ship secrets.
12. Version string (REL-002) — `ssh-cli --version` matches Cargo version plus git hash; reports `-dirty` when the tree is dirty.
13. Local release commit and tag (REL-003) — clean `git status` for release commit; HEAD message is Release; local tag `vX.Y.Z` (for 0.5.4: `v0.5.4`); no remote push unless authorized.
14. No telemetry — `vps doctor --json` reports `"telemetry": false`; no metrics/telemetry SDKs in the tree.
15. Temporary probes removed — no leftover `_probe_*` artifacts in the tree.
16. Default tracing error (LOG-001) — default level is error (not info); tunnel/JSON mode stderr is envelope-only (no INFO progress banners such as "Tunnel SSH:" / "iniciando tunnel"). Graduated `-v`/`-vv`/`-vvv` is crate-scoped (G2/G14); ambient `RUST_LOG` ignored.
17. Product-line docs match Cargo version (DOC-003) — every product-line surface states **0.5.4**, including:
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
18. JSON empty password is null (JSON-001) — runtime: key-only `vps show|list --json` emits `"password": null` (not `"***"`); non-empty remains masked `***`. Schema: `docs/schemas/vps-show.schema.json` (and list via `$ref`) declares `password` type as `string` | `null`.
19. Health-check timeout (CLI-004) — `health-check --timeout <ms>` is accepted (clap parse), aligned with exec overrides; covered by gaps_v039.
20. CHANGELOG anchors (CHG-001) — `CHANGELOG.md` has section `## [0.5.4]` and compare/footer anchors for 0.5.4 (and prior 0.5.3 / 0.5.2 / 0.4.x / 0.3.9 as needed).
21. Optional package dry-run — `cargo package --allow-dirty --list` succeeds; never auto-publish.
22. DOC-004 / SCP honesty (0.4.0+) — product-line surfaces document:
    - SCP **regular files only** (no directories / no `-r`); trees via `sftp --recursive`
    - crates.io **0.3.9** advertised SCP but wire was broken; do not promise working SCP on 0.3.9
    - `docs/schemas/scp-transfer.schema.json` exists and is indexed (`docs/schemas/README.md`, `llms-full.txt`)
    - download partial suffix **`.ssh-cli.partial`**
    - `tunnel --json` / `tunnel_listening` and/or scp agent JSON surface in README/INTEGRATIONS/AGENTS
    - bilingual `skills/ssh-cli-en` and `skills/ssh-cli-pt` teach scp-transfer, tunnel_listening, file-only, partial, 32 KiB, timeout matrix (DOC-004d)
    - SECURITY Supported Versions brands **0.5.x** as current line (not 0.3.x)
    - `cargo test --locked --test gaps_v040_integration` + `gaps_v041_integration` green
23. DOC-041 / AUD-POST honesty (historical 0.4.x) — product-line and agent surfaces document:
    - redacted `vps export` **never** documents or expects `sshcli-enc:` for empty secrets
    - tunnel post-bind deadline exits **0** after `tunnel_listening` (one-shot success; not 74)
    - `tunnel` / `health-check` auth flags parity documented (`--password-stdin`, key / passphrase overrides as applicable)
    - `scp-transfer` schema **requires** `event: "scp-transfer"`
    - `cargo test --locked --test gaps_v041_integration` green
24. DOC-051 / 0.5.2 honesty (historical) — product-line surfaces document:
    - `vps export` body follows the resolved format: JSON `event: "vps-export"` on any non-TTY stdout; TOML only with `--output-format text`
    - wire **schema v3** dual-read (EN serialize / PT load aliases)
    - secrets schemas `secrets-init.schema.json` / `secrets-reencrypt.schema.json` indexed
    - tunnel `--bind` defaults to `127.0.0.1`
    - exit **77** for auth; exit **65** for `TomlDe` / bad import
    - secrets flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` preferred over env
    - `--include-secrets` requires `-o` or `--i-understand-secrets-on-stdout`
    - `cargo test --locked --test gaps_v042_integration` + `gaps_v051_integration` green
25. DOC-053 / 0.5.3 honesty — product-line surfaces document:
    - SFTP upload integrity fixed (G1); prefer 0.5.3+; verify with destination `sha256`
    - SETSTAT atime+mtime (G3), fail-closed set_metadata (G4), perm mask (G12)
    - Graduated verbosity `-v`/`-vv`/`-vvv` crate-scoped (G2/G14); `RUST_LOG` ignored
    - Batch cancel cardinality (G5/G17); `exec --json` single object (G8); SCP `sync_data` (G9)
    - E2E **E17/E18** SFTP checksum/tree (G7)
    - No circular `gaps.md` tests (G13/G15); `gaps.md` gitignored
    - `cargo fmt --check` green (G10)
    - `cargo test --locked --test gaps_v056_ssh` + `gaps_v057_sftp` + `gaps_v058_e2e_residual` green
    - Full CLI command tree documented in HOW_TO_USE + AGENTS + COOKBOOK (including `locale show|set|clear` + full `tls mtls/*` / `tls acme/*`)
    - G6 `serial_test` / deterministic residual suite isolation
    - G11 baseline green (`cargo test --locked --all-targets` + residual suites)
    - G16 English SCP identifiers (wire/schema/docs use English field names)
    - G18 `set_permissions` fail-closed on SFTP download
    - G19 `SFTP_PERM_MASK` named constant (`0o7777`, outbound upload) and `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`, inbound download; A3)
    - `ssh-cli commands` discovery documented (agent inventory surface)
    - Product-line docs list `locale` + full `tls` tree (mtls + acme account/issue/complete/status/list)
26. DOC-054 / 0.5.4 honesty — product-line surfaces document:
    - all four `tunnel` modes: default local forward, `--reverse` (G-TUN-R01), `--socks5` (G-TUN-R02), `--remote-socket <PATH>` (G-TUN-R03); the three flags are mutually exclusive
    - JSON `mode` label is one of `local` / `reverse` / `socks5` / `streamlocal`
    - `--reverse` with `REMOTE_PORT 0` lets the server allocate and report the port; a local forward cannot accept 0
    - `--socks5` and `--remote-socket` omit `REMOTE_HOST`/`REMOTE_PORT` (passing them is exit **64**); `--remote-socket` requires an absolute path
    - a routable `--bind` requires `--i-accept-network-exposure` (G-TUN-R13), **including** a local forward — not only `--reverse`
    - under `--reverse` the guard covers the **server's** bind (the positional `<remote_host>`), which is compared as text, not IP-parsed (typo → exit 64, not exit 2); `--bind` itself is accepted and silently discarded
    - `tunnel_closed` event with `reason`, `forwards_served`, `capacity_waits` — the only discriminator between the three exit-0 endings
    - payload shaping applied before serialization: `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
    - `--dry-run` is honoured by `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt` — and **only** by those; elsewhere it is exit 64
    - `--tags` fleet selector on `exec` / `sudo-exec` / `su-exec` only, mutually exclusive with `--all` and `--hosts`
    - BREAKING: partial multi-host failure exits **1** (was 65; G-ERR-R02)
    - A1 applies the pre-existing 512-character banner log cap on character boundaries (removes a remotely triggerable abort under `panic = "abort"`); A2 creates ACME/mTLS keys at `0600`; A3 masks **inbound** modes with `SFTP_PERM_MASK_UNTRUSTED`
    - `cargo test --locked --test docs_conformance` green (including the 0.5.4 surface, fleet-selector and directional-mask gates)


## How to verify residuals quickly

```bash
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo test --locked --test gaps_v042_integration
cargo test --locked --test gaps_v051_integration
cargo test --locked --test gaps_v056_ssh
cargo test --locked --test gaps_v057_sftp
cargo test --locked --test gaps_v058_e2e_residual
cargo fmt --check
cargo deny check
bash scripts/check_en_identifiers.sh
bash scripts/verify_install_resolve.sh
ssh-cli --version
git check-ignore gaps.md   # must match (exit 0)
```

- LOG-001: tunnel with `--output-format json` fails without connecting; stderr has JSON envelope and no INFO prose.
- JSON-001: key-only host show JSON contains `"password": null`; schema file contains null in password type.
- CLI-004: `health-check --timeout 50` is not "unexpected argument".
- DOC-003: product-line files (including this checklist pair) contain `0.5.4`.
- DOC-004: README/INTEGRATIONS/AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION mention file-only SCP and 0.3.9 wire warning; scp-transfer schema present.
- DOC-004d: `skills/ssh-cli-en` and `skills/ssh-cli-pt` teach scp-transfer, tunnel_listening, file-only, partial, 32 KiB stream, and timeout matrix; evals cover the surface.
- DOC-041: export redacted empty secrets have no `sshcli-enc:`; tunnel post-bind deadline exit 0 after `tunnel_listening`; tunnel/health auth flag parity documented; scp-transfer schema requires `event`; gaps_v041 green.
- DOC-051: export redaction; format follows --output-format; schema v3; secrets schemas; `--bind` loopback; exit 77; secrets flags; include-secrets; gaps_v042 + gaps_v051 green.
- DOC-053 / 0.5.3: SFTP G1 integrity; verbosity G2/G14; E17/E18; gaps.md gitignored; `cargo fmt --check`; gaps_v056/v057 green; full CLI tree (locale + tls mtls/acme) in HOW_TO_USE/AGENTS/COOKBOOK; G6 serial_test; G11 baseline; G16 EN SCP ids; G18 set_permissions; G19 `SFTP_PERM_MASK`; `ssh-cli commands` discovery.
- DENY-002: `deny.toml` has `yanked = "deny"`, `ignore = []`, multiple-versions policy documented.
- CHG-001 / REL: CHANGELOG section `## [0.5.4]` + local tag `v0.5.4` without unauthorized push.
- TEST-004 / SCP: gaps_v040 covers wire, schema, partial path, preserve, e2e script E10–E14.
- AUD-POST / gaps_v041: EXP-001, TUN-002, CLI-005/006, IO-009, REL-006 residual suite green.
- 0.5.2 / gaps_v051: export redaction roundtrip, `vps-export` JSON, dual-read, secrets-init event, include-secrets guard, CRUD `vps-added`, empty command, import exit 65.
- G-E2E / gaps_v058: root `schema` / `doctor`, single `vps-added` + `secrets_key_auto_created`, `--use-agent`, ambient `RUST_LOG` ignored, FIXED_MASK `***`, ACME exit 64.
- G10: `cargo fmt --check` green.
- EN identifiers: `scripts/check_en_identifiers.sh` exits 0.


## Policy

- FORBIDDEN: declare inventory Fechado (closed) while any gap remains Aberto (open).
- FORBIDDEN: eternal RUSTSEC / CVE waive without closed tracking in the same release.
- FORBIDDEN: `git push` or crates.io publish without explicit maintainer authorization.
- FORBIDDEN: log or paste real secrets into inventory, checklist notes, or CI logs.
- REQUIRED: multi-line inventory / CHANGELOG writes use atomwrite (or equivalent atomic write).
- REQUIRED: Status Resolvido only with code + test + version note (destination-effect evidence; not inventory FIXED text).


## G-22 — Distribution, SBOM, multi-arch (process)

> Product identity remains **one-shot CLI** (crates.io + local tag). Binary multi-arch
> and signed SBOM are **release-process** gates, not runtime features. Scripts live under
> `scripts/`; they never push or publish.

### 28. Multi-arch release binaries (G-22)

- Config: root [`Cross.toml`](../Cross.toml) targets:
  - `x86_64-unknown-linux-musl`
  - `aarch64-unknown-linux-musl`
  - `aarch64-unknown-linux-gnu`
- Tooling: install [`cross`](https://github.com/cross-rs/cross) + Docker.
- Build matrix (local, no push):

```bash
bash scripts/dist_multiarch.sh
# or a single target:
TARGETS="x86_64-unknown-linux-musl" bash scripts/dist_multiarch.sh
```

- Artifacts: `target/dist/ssh-cli-<triple>` + `.sha256` sidecars.
- Optional cargo-dist: maintainers **may** add a `dist-workspace.toml` / GitHub
  `release.yml` later; until then `cross` + `dist_multiarch.sh` is the supported path.
- FORBIDDEN: attach unstripped debug builds as “release” without documenting.

### 29. SBOM generation (G-22)

```bash
# Preferred: CycloneDX JSON
cargo install cargo-cyclonedx
bash scripts/generate_sbom.sh
# → target/sbom/ssh-cli.cdx.json (or path argument)
```

- Fallback without cargo-cyclonedx: script writes `cargo tree` inventory and warns
  that it is **not** a CycloneDX SBOM.
- Signing (maintainer, offline):

```bash
# Example with cosign keyless (needs OIDC) or local key — choose one org standard:
# cosign sign-blob --bundle target/sbom/ssh-cli.cdx.json.bundle target/sbom/ssh-cli.cdx.json
# gpg --detach-sign --armor target/sbom/ssh-cli.cdx.json
```

- Attach SBOM + signature + multi-arch binaries to the GitHub Release for tag `vX.Y.Z`
  **only** after explicit maintainer authorization (same rule as `git push`).
- crates.io publish remains separate (`cargo publish`); SBOM is release evidence, not a crate file.

### 30. G-22 acceptance criteria (close inventory)

- [ ] `bash scripts/dist_multiarch.sh` produces at least musl x86_64 artifact + sha256 (or documented skip when Docker unavailable).
- [ ] `bash scripts/generate_sbom.sh` produces CycloneDX JSON **or** documented fallback inventory.
- [ ] Release notes / GitHub Release (when authorized) list binary triples + SBOM path.
- [ ] No auto-push / no auto-publish from scripts.


## Reference

- `gaps.md` — **local-only** maintainer audit inventory (gitignored / cargo-excluded; not published)
- [../Cross.toml](../Cross.toml) — cross-rs target images
- [../scripts/dist_multiarch.sh](../scripts/dist_multiarch.sh) — multi-arch build
- [../scripts/generate_sbom.sh](../scripts/generate_sbom.sh) — SBOM / inventory
- [../deny.toml](../deny.toml) — supply-chain policy
- [../scripts/check_all_gates.sh](../scripts/check_all_gates.sh) — mandatory gate battery runner (all ten gates, one invocation)
- [../scripts/check_advisory_freshness.sh](../scripts/check_advisory_freshness.sh) — advisory database freshness gate
- [../scripts/verify_install_resolve.sh](../scripts/verify_install_resolve.sh) — install re-resolve gate
- [../scripts/check_en_identifiers.sh](../scripts/check_en_identifiers.sh) — English identifier residual gate
- [../tests/gaps_v039_integration.rs](../tests/gaps_v039_integration.rs) — residual gates LOG/JSON/CLI/DOC/DENY/CHG
- [../tests/gaps_v040_integration.rs](../tests/gaps_v040_integration.rs) — residual gates SCP/IO/DOC-004/REL-004
- [../tests/gaps_v041_integration.rs](../tests/gaps_v041_integration.rs) — residual gates EXP-001/TUN-002/CLI-005/006/IO-009/REL-006 (DOC-041)
- [../tests/gaps_v042_integration.rs](../tests/gaps_v042_integration.rs) — residual gates AUD-E2E (TUN-003, IO-010, ENV-001, SCP-024, …)
- [../tests/gaps_v051_integration.rs](../tests/gaps_v051_integration.rs) — residual gates 0.5.2 export/schema v3/secrets
- [../tests/gaps_v056_ssh.rs](../tests/gaps_v056_ssh.rs) — residual gates G-SSH
- [../tests/gaps_v057_sftp.rs](../tests/gaps_v057_sftp.rs) — residual gates G-SFTP
- [../tests/gaps_v058_e2e_residual.rs](../tests/gaps_v058_e2e_residual.rs) — residual gates G-E2E (schema/doctor root, single `vps-added`, `--use-agent`, FIXED_MASK, ACME 64)
- [schemas/vps-show.schema.json](schemas/vps-show.schema.json) — password `null` | masked `***`
- [schemas/scp-transfer.schema.json](schemas/scp-transfer.schema.json) — SCP success JSON (files only; requires `event`)
- [schemas/tunnel-listening.schema.json](schemas/tunnel-listening.schema.json) — tunnel bind event
- [schemas/vps-export.schema.json](schemas/vps-export.schema.json) — `vps export --json` only (`event: "vps-export"`)
- [schemas/secrets-init.schema.json](schemas/secrets-init.schema.json) — `secrets init --json`
- [schemas/secrets-reencrypt.schema.json](schemas/secrets-reencrypt.schema.json) — `secrets reencrypt --json`
- [schemas/README.md](schemas/README.md) — schema index (product line 0.5.4)

## Residual G-E2E docs gate (v0.5.3)

- [ ] Ambient `RUST_LOG` ignored documented; only `-v`/`-vv`/`-vvv` for verbosity (G2/G14)
- [ ] Single JSON `vps-added` with field `secrets_key_auto_created` (G8 family)
- [ ] Root `schema` / `doctor` documented
- [ ] `vps add --use-agent` documented
- [ ] ACME `invalidContact` → exit 64 permanent documented
- [ ] Export redacted `***` (`FIXED_MASK`) documented
- [ ] E2E XDG-first + SKIP offline documented in TESTING
- [ ] E2E **E17/E18** SFTP checksum/tree documented (G7)
- [ ] SFTP G1 integrity + prefer 0.5.3+ documented
- [ ] Full CLI command tree (including `locale` + full `tls mtls` / `tls acme`) documented in HOW_TO_USE + AGENTS + COOKBOOK
- [ ] `ssh-cli commands` discovery documented
- [ ] G6 `serial_test` / deterministic suite; G11 baseline green; G16 English SCP identifiers
- [ ] G18 `set_permissions` fail-closed on SFTP download; G19 `SFTP_PERM_MASK` named constant
- [ ] Product-line docs list `locale` + full `tls` tree
- [ ] `gaps.md` gitignored (`git check-ignore gaps.md` matches); no circular FIXED-text tests (G13/G15)
- [ ] `cargo fmt --check` in publish gates (G10)
- [ ] Suite `tests/gaps_v056_ssh.rs` + `tests/gaps_v057_sftp.rs` + `tests/gaps_v058_e2e_residual.rs` in publish gates


## Multi-OS local matrix (G-E2E-18)

- Product code: `src/platform/{linux,macos,windows}.rs` — no GitHub Actions cloud matrix.
- Local multi-arch: `scripts/dist_multiarch.sh` when cross toolchains are installed.
- Validate path length / agent socket notes on macOS and Windows before tagging a release.
- Do **not** reintroduce `.github/workflows` for CI (policy: one-shot CLI, no cloud CI product).

