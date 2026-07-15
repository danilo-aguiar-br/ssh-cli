# Release checklist — ssh-cli

> Mandatory gates before marking a release and `gaps.md` inventory as closed (Fechado).

- Read this document in [Portuguese (pt-BR)](RELEASE_CHECKLIST.pt-BR.md).
- Release target / product line: **0.5.0**.
- Historical gate: **0.4.1** DOC-041 / AUD-POST honesty (export empty, tunnel exit 0, auth parity, scp-transfer event).
- Canonical inventory: [../gaps.md](../gaps.md).
- Residual suites: `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL/CHG); `tests/gaps_v040_integration.rs` (SCP 0.4.0); `tests/gaps_v041_integration.rs` (EXP-001, TUN-002, CLI-005/006, IO-009, REL-006).


## Purpose
- Prevent shipping with open gaps, stale product-line docs, or supply-chain waivers.
- Keep release evidence honest (pre/post-fix notes in inventory, no secrets in logs).
- Align Cargo version, `--version`, docs product line, tags, and CHANGELOG anchors.


## Gates (required)

1. Release build — `cargo build --release` exits 0.
2. Clippy clean — `cargo clippy --all-targets -- -D warnings` exits 0.
3. Supply chain deny (DENY-002) — `cargo deny check` exits 0; no russh CVE `ignore`; `yanked=deny`; empty `ignore = []`.
4. Install resolve — `bash scripts/verify_install_resolve.sh` exits 0; russh at security floor (≥ 0.60.3; product line uses 0.62.2).
5. Full tests — `cargo test` green (lib + integration + gaps_v037 + gaps_v038 + gaps_v039 + gaps_v040 + **gaps_v041**).
6. Gap suites 1:1 — all `gap_*` tests in `tests/gaps_v038_integration.rs`, `tests/gaps_v039_integration.rs`, and `tests/gaps_v040_integration.rs` / `tests/gaps_v041_integration.rs` green.
7. Local e2e (no real VPS) — help, fake VPS CRUD, completions behave as documented.
8. Real VPS smoke (when available) — `health-check` / `exec` plus SCP matrix **E10–E14** via `scripts/e2e_real_ssh.sh` when credentials exist; record outcome in `gaps.md` without secrets.
9. Inventory versioned — `gaps.md` is tracked (not gitignored); `git check-ignore gaps.md` is empty.
10. Honest pre/post-fix evidence in inventory (DOC-002 / inventory integrity).
11. Version string (REL-002) — `ssh-cli --version` matches Cargo version plus git hash; reports `-dirty` when the tree is dirty.
12. Local release commit and tag (REL-003) — clean `git status` for release commit; HEAD message is Release; local tag `vX.Y.Z` (for 0.4.2: `v0.4.2`); no remote push unless authorized.
13. No telemetry — `vps doctor --json` reports `"telemetry": false`; no metrics/telemetry SDKs in the tree.
14. Temporary probes removed — no leftover `_probe_*` artifacts in the tree.
15. Default tracing error (LOG-001) — default level is error (not info); tunnel/JSON mode stderr is envelope-only (no INFO progress banners such as "Tunnel SSH:" / "iniciando tunnel").
16. Product-line docs match Cargo version (DOC-003) — every product-line surface states **0.5.0**, including:
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
17. JSON empty password is null (JSON-001) — runtime: key-only `vps show|list --json` emits `"password": null` (not `"***"`); non-empty remains masked `***`. Schema: `docs/schemas/vps-show.schema.json` (and list via `$ref`) declares `password` type as `string` | `null`.
18. Health-check timeout (CLI-004) — `health-check --timeout <ms>` is accepted (clap parse), aligned with exec overrides; covered by gaps_v039.
19. CHANGELOG anchors (CHG-001) — `CHANGELOG.md` has section `## [0.4.2]` and compare/footer anchors for 0.4.2 (and prior 0.4.0 / 0.3.9 as needed).
20. Optional package dry-run — `cargo package --allow-dirty --list` succeeds; never auto-publish.
21. DOC-004 / SCP honesty (0.4.0+) — product-line surfaces document:
    - SCP **regular files only** (no directories / no `-r` / no SFTP)
    - crates.io **0.3.9** advertised SCP but wire was broken; do not promise working SCP on 0.3.9
    - `docs/schemas/scp-transfer.schema.json` exists and is indexed (`docs/schemas/README.md`, `llms-full.txt`)
    - download partial suffix **`.ssh-cli.partial`**
    - `tunnel --json` / `tunnel_listening` and/or scp agent JSON surface in README/INTEGRATIONS/AGENTS
    - bilingual `skills/ssh-cli-en` and `skills/ssh-cli-pt` teach scp-transfer, tunnel_listening, file-only, partial, 32 KiB, timeout matrix (DOC-004d)
    - SECURITY Supported Versions brands **0.4.x** as current line (not 0.3.x)
    - `cargo test --locked --test gaps_v040_integration` + `gaps_v041_integration` green
22. DOC-041 / AUD-POST honesty (0.4.2) — product-line and agent surfaces document:
    - redacted `vps export` **never** documents or expects `sshcli-enc:` for empty secrets
    - tunnel post-bind deadline exits **0** after `tunnel_listening` (one-shot success; not 74)
    - `tunnel` / `health-check` auth flags parity documented (`--password-stdin`, key / passphrase overrides as applicable)
    - `scp-transfer` schema **requires** `event: "scp-transfer"`
    - `cargo test --locked --test gaps_v041_integration` green


## How to verify residuals quickly

```bash
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo deny check
bash scripts/verify_install_resolve.sh
ssh-cli --version
```

- LOG-001: tunnel with `--output-format json` fails without connecting; stderr has JSON envelope and no INFO prose.
- JSON-001: key-only host show JSON contains `"password": null`; schema file contains null in password type.
- CLI-004: `health-check --timeout 50` is not "unexpected argument".
- DOC-003: product-line files (including this checklist pair) contain `0.4.2`.
- DOC-004: README/INTEGRATIONS/AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION mention file-only SCP and 0.3.9 wire warning; scp-transfer schema present.
- DOC-004d: `skills/ssh-cli-en` and `skills/ssh-cli-pt` teach scp-transfer, tunnel_listening, file-only, partial, 32 KiB stream, and timeout matrix; evals cover the surface.
- DOC-041: export redacted empty secrets have no `sshcli-enc:`; tunnel post-bind deadline exit 0 after `tunnel_listening`; tunnel/health auth flag parity documented; scp-transfer schema requires `event`; gaps_v041 green.
- DENY-002: `deny.toml` has `yanked = "deny"`, `ignore = []`, multiple-versions policy documented.
- CHG-001 / REL: CHANGELOG section + local tag `v0.4.2` without unauthorized push.
- TEST-004 / SCP: gaps_v040 covers wire, schema, partial path, preserve, e2e script E10–E14.
- AUD-POST / gaps_v041: EXP-001, TUN-002, CLI-005/006, IO-009, REL-006 residual suite green.


## Policy

- FORBIDDEN: declare inventory Fechado (closed) while any gap remains Aberto (open).
- FORBIDDEN: eternal RUSTSEC / CVE waive without closed tracking in the same release.
- FORBIDDEN: `git push` or crates.io publish without explicit maintainer authorization.
- FORBIDDEN: log or paste real secrets into inventory, checklist notes, or CI logs.
- REQUIRED: multi-line inventory / CHANGELOG writes use atomwrite (or equivalent atomic write).
- REQUIRED: Status Resolvido only with code + test + version note in `gaps.md`.


## Reference

- [../gaps.md](../gaps.md) — canonical gap inventory
- [../deny.toml](../deny.toml) — supply-chain policy
- [../scripts/verify_install_resolve.sh](../scripts/verify_install_resolve.sh) — install re-resolve gate
- [../tests/gaps_v039_integration.rs](../tests/gaps_v039_integration.rs) — residual gates LOG/JSON/CLI/DOC/DENY/CHG
- [../tests/gaps_v040_integration.rs](../tests/gaps_v040_integration.rs) — residual gates SCP/IO/DOC-004/REL-004
- [../tests/gaps_v041_integration.rs](../tests/gaps_v041_integration.rs) — residual gates EXP-001/TUN-002/CLI-005/006/IO-009/REL-006 (DOC-041)
- [schemas/vps-show.schema.json](schemas/vps-show.schema.json) — password `null` | masked `***`
- [schemas/scp-transfer.schema.json](schemas/scp-transfer.schema.json) — SCP success JSON (files only; requires `event`)
- [schemas/tunnel-listening.schema.json](schemas/tunnel-listening.schema.json) — tunnel bind event
- [schemas/README.md](schemas/README.md) — schema index (product line 0.4.2)
