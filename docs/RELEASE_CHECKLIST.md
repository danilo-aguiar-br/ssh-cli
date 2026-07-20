# Release checklist — ssh-cli

> Mandatory gates before marking a release and `gaps.md` inventory as closed (Fechado).

- Read this document in [Portuguese (pt-BR)](RELEASE_CHECKLIST.pt-BR.md).
- Release target / product line: **0.5.2**.
- Historical gate: **0.4.1** DOC-041 / AUD-POST honesty (export empty, tunnel exit 0, auth parity, scp-transfer event).
- Canonical inventory: [../gaps.md](../gaps.md).
- Residual suites: `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL/CHG); `tests/gaps_v040_integration.rs` (SCP 0.4.0); `tests/gaps_v041_integration.rs` (EXP-001, TUN-002, CLI-005/006, IO-009, REL-006); `tests/gaps_v042_integration.rs` (AUD-E2E); `tests/gaps_v051_integration.rs` (0.5.2 wire/export/secrets); `tests/gaps_v058_e2e_residual.rs` (G-E2E residual: schema/doctor root, single `vps-added`, `--use-agent`, help/clap env purge, FIXED_MASK, ACME 64).


## Purpose
- Prevent shipping with open gaps, stale product-line docs, or supply-chain waivers.
- Keep release evidence honest (pre/post-fix notes in inventory, no secrets in logs).
- Align Cargo version, `--version`, docs product line, tags, and CHANGELOG anchors.


## Gates (required)

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
5. Install resolve — `bash scripts/verify_install_resolve.sh` exits 0; russh at security floor (≥ 0.60.3; product line uses 0.62.2).
6. Full tests — `cargo test --locked --all-targets` green (lib + integration + gaps_v037 + gaps_v038 + gaps_v039 + gaps_v040 + gaps_v041 + gaps_v042 + **gaps_v051** + **gaps_v052** + **gaps_v058**).
7. Gap residual suites green — every test in `tests/gaps_v038_integration.rs`, `tests/gaps_v039_integration.rs`, `tests/gaps_v040_integration.rs`, `tests/gaps_v041_integration.rs`, `tests/gaps_v042_integration.rs`, `tests/gaps_v051_integration.rs`, `tests/gaps_v052_tls_policy.rs`, and `tests/gaps_v058_e2e_residual.rs` passes (including tests not named `gap_*`).
8. Local e2e (no real VPS) — help, fake VPS CRUD, completions behave as documented.
9. Real VPS smoke (when available) — `health-check` / `exec` plus SCP matrix **E10–E14** (full matrix **E01–E16**) via `scripts/e2e_real_ssh.sh` when credentials exist; prefer local sshd / throwaway VPS; no auth-failure storms on production; record outcome in `gaps.md` without secrets.
10. Inventory versioned — `gaps.md` is tracked (not gitignored); `git check-ignore gaps.md` is empty.
11. Honest pre/post-fix evidence in inventory (DOC-002 / inventory integrity).
12. Version string (REL-002) — `ssh-cli --version` matches Cargo version plus git hash; reports `-dirty` when the tree is dirty.
13. Local release commit and tag (REL-003) — clean `git status` for release commit; HEAD message is Release; local tag `vX.Y.Z` (for 0.5.2: `v0.5.2`); no remote push unless authorized.
14. No telemetry — `vps doctor --json` reports `"telemetry": false`; no metrics/telemetry SDKs in the tree.
15. Temporary probes removed — no leftover `_probe_*` artifacts in the tree.
16. Default tracing error (LOG-001) — default level is error (not info); tunnel/JSON mode stderr is envelope-only (no INFO progress banners such as "Tunnel SSH:" / "iniciando tunnel").
17. Product-line docs match Cargo version (DOC-003) — every product-line surface states **0.5.2**, including:
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
20. CHANGELOG anchors (CHG-001) — `CHANGELOG.md` has section `## [0.5.2]` and compare/footer anchors for 0.5.2 (and prior 0.4.x / 0.3.9 as needed).
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
24. DOC-051 / 0.5.2 honesty — product-line surfaces document:
    - `vps export` default body is **TOML** (even pipes); JSON agent envelope only with `--json` → `event: "vps-export"`
    - wire **schema v3** dual-read (EN serialize / PT load aliases)
    - secrets schemas `secrets-init.schema.json` / `secrets-reencrypt.schema.json` indexed
    - tunnel `--bind` defaults to `127.0.0.1`
    - exit **77** for auth; exit **65** for `TomlDe` / bad import
    - secrets flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` preferred over env
    - `--include-secrets` requires `-o` or `--i-understand-secrets-on-stdout`
    - `cargo test --locked --test gaps_v042_integration` + `gaps_v051_integration` green


## How to verify residuals quickly

```bash
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo test --locked --test gaps_v042_integration
cargo test --locked --test gaps_v051_integration
cargo test --locked --test gaps_v058_e2e_residual
cargo deny check
bash scripts/check_en_identifiers.sh
bash scripts/verify_install_resolve.sh
ssh-cli --version
```

- LOG-001: tunnel with `--output-format json` fails without connecting; stderr has JSON envelope and no INFO prose.
- JSON-001: key-only host show JSON contains `"password": null`; schema file contains null in password type.
- CLI-004: `health-check --timeout 50` is not "unexpected argument".
- DOC-003: product-line files (including this checklist pair) contain `0.5.2`.
- DOC-004: README/INTEGRATIONS/AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION mention file-only SCP and 0.3.9 wire warning; scp-transfer schema present.
- DOC-004d: `skills/ssh-cli-en` and `skills/ssh-cli-pt` teach scp-transfer, tunnel_listening, file-only, partial, 32 KiB stream, and timeout matrix; evals cover the surface.
- DOC-041: export redacted empty secrets have no `sshcli-enc:`; tunnel post-bind deadline exit 0 after `tunnel_listening`; tunnel/health auth flag parity documented; scp-transfer schema requires `event`; gaps_v041 green.
- DOC-051: export TOML default; schema v3; secrets schemas; `--bind` loopback; exit 77; secrets flags; include-secrets; gaps_v042 + gaps_v051 green.
- DENY-002: `deny.toml` has `yanked = "deny"`, `ignore = []`, multiple-versions policy documented.
- CHG-001 / REL: CHANGELOG section + local tag `v0.5.2` without unauthorized push.
- TEST-004 / SCP: gaps_v040 covers wire, schema, partial path, preserve, e2e script E10–E14.
- AUD-POST / gaps_v041: EXP-001, TUN-002, CLI-005/006, IO-009, REL-006 residual suite green.
- 0.5.2 / gaps_v051: export TOML default, `vps-export` JSON, dual-read, secrets-init event, include-secrets guard, CRUD `vps-added`, empty command, import exit 65.
- G-E2E / gaps_v058: root `schema` / `doctor`, single `vps-added` + `secrets_key_auto_created`, `--use-agent`, ambient `RUST_LOG` ignored, FIXED_MASK `***`, ACME exit 64.
- EN identifiers: `scripts/check_en_identifiers.sh` exits 0.


## Policy

- FORBIDDEN: declare inventory Fechado (closed) while any gap remains Aberto (open).
- FORBIDDEN: eternal RUSTSEC / CVE waive without closed tracking in the same release.
- FORBIDDEN: `git push` or crates.io publish without explicit maintainer authorization.
- FORBIDDEN: log or paste real secrets into inventory, checklist notes, or CI logs.
- REQUIRED: multi-line inventory / CHANGELOG writes use atomwrite (or equivalent atomic write).
- REQUIRED: Status Resolvido only with code + test + version note in `gaps.md`.


## G-22 — Distribution, SBOM, multi-arch (process)

> Product identity remains **one-shot CLI** (crates.io + local tag). Binary multi-arch
> and signed SBOM are **release-process** gates, not runtime features. Scripts live under
> `scripts/`; they never push or publish.

### 25. Multi-arch release binaries (G-22)

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

### 26. SBOM generation (G-22)

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

### 27. G-22 acceptance criteria (close inventory)

- [ ] `bash scripts/dist_multiarch.sh` produces at least musl x86_64 artifact + sha256 (or documented skip when Docker unavailable).
- [ ] `bash scripts/generate_sbom.sh` produces CycloneDX JSON **or** documented fallback inventory.
- [ ] Release notes / GitHub Release (when authorized) list binary triples + SBOM path.
- [ ] No auto-push / no auto-publish from scripts.


## Reference

- [../gaps.md](../gaps.md) — canonical gap inventory
- [../Cross.toml](../Cross.toml) — cross-rs target images
- [../scripts/dist_multiarch.sh](../scripts/dist_multiarch.sh) — multi-arch build
- [../scripts/generate_sbom.sh](../scripts/generate_sbom.sh) — SBOM / inventory
- [../deny.toml](../deny.toml) — supply-chain policy
- [../scripts/verify_install_resolve.sh](../scripts/verify_install_resolve.sh) — install re-resolve gate
- [../scripts/check_en_identifiers.sh](../scripts/check_en_identifiers.sh) — English identifier residual gate
- [../tests/gaps_v039_integration.rs](../tests/gaps_v039_integration.rs) — residual gates LOG/JSON/CLI/DOC/DENY/CHG
- [../tests/gaps_v040_integration.rs](../tests/gaps_v040_integration.rs) — residual gates SCP/IO/DOC-004/REL-004
- [../tests/gaps_v041_integration.rs](../tests/gaps_v041_integration.rs) — residual gates EXP-001/TUN-002/CLI-005/006/IO-009/REL-006 (DOC-041)
- [../tests/gaps_v042_integration.rs](../tests/gaps_v042_integration.rs) — residual gates AUD-E2E (TUN-003, IO-010, ENV-001, SCP-024, …)
- [../tests/gaps_v051_integration.rs](../tests/gaps_v051_integration.rs) — residual gates 0.5.2 export/schema v3/secrets
- [../tests/gaps_v058_e2e_residual.rs](../tests/gaps_v058_e2e_residual.rs) — residual gates G-E2E (schema/doctor root, single `vps-added`, `--use-agent`, FIXED_MASK, ACME 64)
- [schemas/vps-show.schema.json](schemas/vps-show.schema.json) — password `null` | masked `***`
- [schemas/scp-transfer.schema.json](schemas/scp-transfer.schema.json) — SCP success JSON (files only; requires `event`)
- [schemas/tunnel-listening.schema.json](schemas/tunnel-listening.schema.json) — tunnel bind event
- [schemas/vps-export.schema.json](schemas/vps-export.schema.json) — `vps export --json` only (`event: "vps-export"`)
- [schemas/secrets-init.schema.json](schemas/secrets-init.schema.json) — `secrets init --json`
- [schemas/secrets-reencrypt.schema.json](schemas/secrets-reencrypt.schema.json) — `secrets reencrypt --json`
- [schemas/README.md](schemas/README.md) — schema index (product line 0.5.2)

## Residual G-E2E docs gate (v0.5.2)

- [ ] Ambient `RUST_LOG` ignored documented; only `-v` for debug
- [ ] Single JSON `vps-added` with field `secrets_key_auto_created`
- [ ] Root `schema` / `doctor` documented
- [ ] `vps add --use-agent` documented
- [ ] ACME `invalidContact` → exit 64 permanent documented
- [ ] Export redacted `***` (`FIXED_MASK`) documented
- [ ] E2E XDG-first + SKIP offline documented in TESTING
- [ ] Suite `tests/gaps_v058_e2e_residual.rs` in publish gates

## Multi-OS local matrix (G-E2E-18)

- Product code: `src/platform/{linux,macos,windows}.rs` — no GitHub Actions cloud matrix.
- Local multi-arch: `scripts/dist_multiarch.sh` when cross toolchains are installed.
- Validate path length / agent socket notes on macOS and Windows before tagging a release.
- Do **not** reintroduce `.github/workflows` for CI (policy: one-shot CLI, no cloud CI product).

