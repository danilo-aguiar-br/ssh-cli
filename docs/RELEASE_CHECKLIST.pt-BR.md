# Checklist de release — ssh-cli

> Gates obrigatórios antes de marcar uma release e o inventário `gaps.md` como Fechado.

- Leia este documento em [inglês](RELEASE_CHECKLIST.md).
- Alvo de release / linha de produto: **0.4.0**.
- Inventário canônico: [../gaps.md](../gaps.md).
- Suite residual: `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL/CHG).


## Propósito
- Impedir ship com gaps abertos, docs de product line defasados ou waivers de supply-chain.
- Manter evidências de release honestas (notas pré/pós-fix no inventário, sem secrets em logs).
- Alinhar versão Cargo, `--version`, product line nos docs, tags e âncoras do CHANGELOG.


## Gates (obrigatórios)

1. Build de release — `cargo build --release` exit 0.
2. Clippy limpo — `cargo clippy --all-targets -- -D warnings` exit 0.
3. Supply chain deny (DENY-002) — `cargo deny check` exit 0; sem `ignore` de CVE russh; `yanked=deny`; `ignore = []` vazio.
4. Install resolve — `bash scripts/verify_install_resolve.sh` exit 0; russh no piso de segurança (≥ 0.60.3; linha de produto usa 0.62.2).
5. Testes completos — `cargo test` verde (lib + integration + gaps_v037 + gaps_v038 + gaps_v039).
6. Suites de gaps 1:1 — todos os testes `gap_*` em `tests/gaps_v038_integration.rs` e `tests/gaps_v039_integration.rs` verdes; suite residual **gaps_v039** verde (LOG/JSON/CLI/DOC/DENY/CHG).
7. e2e local (sem VPS real) — help, CRUD VPS fake, completions conforme documentado.
8. Smoke VPS real (quando disponível) — `health-check` / `exec`; registrar resultado em `gaps.md` sem secrets.
9. Inventário versionado — `gaps.md` rastreado (não gitignored); `git check-ignore gaps.md` vazio.
10. Evidências pré/pós-fix honestas no inventário (DOC-002 / integridade do inventário).
11. String de versão (REL-002) — `ssh-cli --version` bate com versão Cargo + hash git; reporta `-dirty` se a tree estiver suja.
12. Commit e tag locais de release (REL-003) — `git status` limpo no commit de release; mensagem HEAD de Release; tag local `vX.Y.Z` (para 0.4.0: `v0.4.0`); sem push remoto sem autorização.
13. Sem telemetria — `vps doctor --json` reporta `"telemetry": false`; sem SDKs de métricas/telemetria na tree.
14. Probes temporários removidos — sem artefatos `_probe_*` restantes na tree.
15. Tracing default error (LOG-001) — nível default é error (não info); stderr em modo tunnel/JSON é só envelope (sem banners INFO como "Tunnel SSH:" / "iniciando tunnel").
16. Docs de product line = versão Cargo (DOC-003) — toda superfície de product line declara **0.4.0**, incluindo:
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
17. Senha vazia em JSON é null (JSON-001) — runtime: `vps show|list --json` em host só-chave emite `"password": null` (não `"***"`); não vazia permanece mascarada `***`. Schema: `docs/schemas/vps-show.schema.json` (e list via `$ref`) declara tipo de `password` como `string` | `null`.
18. Timeout do health-check (CLI-004) — `health-check --timeout <ms>` é aceito (parse clap), alinhado aos overrides de exec; coberto por gaps_v039.
19. Âncoras do CHANGELOG (CHG-001) — `CHANGELOG.md` tem seção `## [0.4.0]` e âncora de compare/rodapé para 0.4.0 (e 0.3.8 anterior conforme necessário).
20. Dry-run opcional de package — `cargo package --allow-dirty --list` ok; nunca publish automático.


## Como verificar residuais rapidamente

```bash
cargo test --locked --test gaps_v039_integration
cargo deny check
bash scripts/verify_install_resolve.sh
ssh-cli --version
```

- LOG-001: tunnel com `--output-format json` falha sem conectar; stderr tem envelope JSON e sem prosa INFO.
- JSON-001: show JSON de host só-chave contém `"password": null`; arquivo de schema contém null no tipo de password.
- CLI-004: `health-check --timeout 50` não é "unexpected argument".
- DOC-003: arquivos de product line (incluindo este par de checklists) contêm `0.4.0`.
- DENY-002: `deny.toml` tem `yanked = "deny"`, `ignore = []`, política multiple-versions documentada.
- CHG-001 / REL: seção no CHANGELOG + tag local `v0.4.0` sem push não autorizado.


## Política

- PROIBIDO: declarar inventário Fechado com qualquer gap Aberto.
- PROIBIDO: waiver eterno de RUSTSEC / CVE sem tracking fechado na mesma release.
- PROIBIDO: `git push` ou publish no crates.io sem autorização explícita do maintainer.
- PROIBIDO: logar ou colar secrets reais no inventário, notas do checklist ou logs de CI.
- OBRIGATÓRIO: escrita multi-linha de inventário / CHANGELOG usa atomwrite (ou escrita atômica equivalente).
- OBRIGATÓRIO: status Resolvido só com código + teste + nota de versão em `gaps.md`.


## Referência

- [../gaps.md](../gaps.md) — inventário canônico de gaps
- [../deny.toml](../deny.toml) — política de supply-chain
- [../scripts/verify_install_resolve.sh](../scripts/verify_install_resolve.sh) — gate de install re-resolve
- [../tests/gaps_v039_integration.rs](../tests/gaps_v039_integration.rs) — gates residuais LOG/JSON/CLI/DOC/DENY/CHG
- [schemas/vps-show.schema.json](schemas/vps-show.schema.json) — password `null` | mascarado `***`
- [schemas/README.md](schemas/README.md) — índice de schemas (linha de produto 0.4.0)
