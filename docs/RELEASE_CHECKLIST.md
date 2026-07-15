# Release checklist — ssh-cli

Use this list before marking any release (and `gaps.md`) as **Fechado**.

## Gates (obrigatórios)

| # | Critério | Como verificar |
|---|----------|----------------|
| 1 | `cargo build --release` | exit 0 |
| 2 | `cargo clippy --all-targets -- -D warnings` | exit 0 |
| 3 | `cargo deny check` | exit 0; **sem** `ignore` de CVE russh; `yanked=deny` |
| 4 | `bash scripts/verify_install_resolve.sh` | exit 0; russh ≥ 0.60.3 |
| 5 | `cargo test --release` (lib + integration + gaps_v038) | exit 0 |
| 6 | Suite gaps 1:1 (`tests/gaps_v038_integration.rs`) | todos `gap_*` verdes |
| 7 | e2e manual subcomandos locais (help, vps CRUD fake, completions) | OK |
| 8 | e2e VPS real (smoke) | health-check / exec; registrar em `gaps.md` sem secrets |
| 9 | Inventário `gaps.md` versionado (não gitignored) | `git check-ignore gaps.md` vazio |
| 10 | Evidências pré/pós-fix honestas no inventário | DOC-002 |
| 11 | Version string coerente (`--version` = Cargo version + hash; `-dirty` se tree suja) | REL-002 |
| 12 | Commit/tag local da release | `git status` clean; HEAD mensagem Release |
| 13 | Sem telemetria | doctor `"telemetry": false`; sem SDKs de métricas |
| 14 | Probes temporários removidos (`_probe_*`) | ausentes no tree |
| 15 | (Opcional) `cargo package --allow-dirty --list` dry-run | sem publish automático |

## Política

- **PROIBIDO** declarar inventário Fechado com gaps Abertos.
- **PROIBIDO** waiver eterno de RUSTSEC sem tracking fechado na mesma release.
- **PROIBIDO** push/publish sem autorização explícita do maintainer.
- Escrita multi-linha de inventário/CHANGELOG: **atomwrite**.

## Referência

- `gaps.md` — inventário canônico
- `deny.toml` — supply-chain
- `scripts/verify_install_resolve.sh` — install re-resolve
