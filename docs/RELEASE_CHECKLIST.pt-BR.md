# Checklist de release — ssh-cli

> Gates obrigatórios antes de marcar release e inventário `gaps.md` como Fechado.

- Leia este documento em [inglês](RELEASE_CHECKLIST.md).
- Alvo de release / linha de produto: **0.4.0**.
- Inventário canônico: [../gaps.md](../gaps.md).
- Suites residuais: `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL/CHG); `tests/gaps_v040_integration.rs` (SCP-010..023, DOC-004, IO-007/007b/008, REL-004, TEST-004).


## Propósito
- Impedir ship com gaps abertos, docs de product line defasados ou waivers de supply-chain.
- Manter evidência de release honesta (notas pré/pós-fix no inventário, sem segredos em logs).
- Alinhar versão Cargo, `--version`, product line de docs, tags e âncoras do CHANGELOG.


## Gates (obrigatórios)

1. Build de release — `cargo build --release` exit 0.
2. Clippy limpo — `cargo clippy --all-targets -- -D warnings` exit 0.
3. Supply chain deny (DENY-002) — `cargo deny check` exit 0; sem `ignore` de CVE russh; `yanked=deny`; `ignore = []` vazio.
4. Install resolve — `bash scripts/verify_install_resolve.sh` exit 0; russh no piso de segurança (≥ 0.60.3; linha de produto usa 0.62.2).
5. Testes completos — `cargo test` verde (lib + integration + gaps_v037 + gaps_v038 + gaps_v039 + **gaps_v040**).
6. Suites de gaps 1:1 — todos os `gap_*` em `tests/gaps_v038_integration.rs`, `tests/gaps_v039_integration.rs` e `tests/gaps_v040_integration.rs` verdes.
7. E2e local (sem VPS real) — help, CRUD fake de VPS, completions se comportam como documentado.
8. Smoke VPS real (quando disponível) — `health-check` / `exec` mais matriz SCP **E10–E14** via `scripts/e2e_real_ssh.sh` quando houver credenciais; registrar resultado em `gaps.md` sem segredos.
9. Inventário versionado — `gaps.md` está tracked (não gitignored); `git check-ignore gaps.md` vazio.
10. Evidência pré/pós-fix honesta no inventário (DOC-002 / integridade do inventário).
11. String de versão (REL-002) — `ssh-cli --version` bate com versão Cargo + hash git; reporta `-dirty` quando a tree está suja.
12. Commit e tag locais de release (REL-003) — `git status` limpo no commit de release; mensagem HEAD de Release; tag local `vX.Y.Z` (para 0.4.0: `v0.4.0`); sem push remoto sem autorização.
13. Sem telemetria — `vps doctor --json` reporta `"telemetry": false`; sem SDKs de métricas/telemetria na tree.
14. Probes temporários removidos — sem artefatos `_probe_*` sobrando na tree.
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
18. Timeout de health-check (CLI-004) — `health-check --timeout <ms>` é aceito (parse clap), alinhado a overrides do exec; coberto por gaps_v039.
19. Âncoras do CHANGELOG (CHG-001) — `CHANGELOG.md` tem seção `## [0.4.0]` e âncora de compare/rodapé para 0.4.0 (e 0.3.9 anterior conforme necessário).
20. Package dry-run opcional — `cargo package --allow-dirty --list` ok; nunca auto-publish.
21. DOC-004 / honestidade SCP (0.4.0) — superfícies de product line documentam:
    - SCP **somente arquivos regulares** (sem diretórios / sem `-r` / sem SFTP)
    - crates.io **0.3.9** anunciava SCP mas o wire estava quebrado; não prometa SCP funcional em 0.3.9
    - `docs/schemas/scp-transfer.schema.json` existe e está indexado (`docs/schemas/README.md`, `llms-full.txt`)
    - sufixo partial de download **`.ssh-cli.partial`**
    - `tunnel --json` / `tunnel_listening` e/ou superfície JSON de scp em README/INTEGRATIONS/AGENTS
    - SECURITY Supported Versions marca **0.4.x** como linha atual (não 0.3.x)
    - `cargo test --locked --test gaps_v040_integration` verde (inclui gates DOC-004)


## Como verificar residuals rápido

```bash
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo deny check
bash scripts/verify_install_resolve.sh
ssh-cli --version
```

- LOG-001: tunnel com `--output-format json` falha sem conectar; stderr tem envelope JSON e sem prosa INFO.
- JSON-001: show JSON de host só-chave contém `"password": null`; arquivo de schema contém null no tipo de password.
- CLI-004: `health-check --timeout 50` não é "unexpected argument".
- DOC-003: arquivos de product line (incluindo este par de checklists) contêm `0.4.0`.
- DOC-004: README/INTEGRATIONS/AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION mencionam SCP file-only e aviso de wire 0.3.9; schema scp-transfer presente.
- DENY-002: `deny.toml` tem `yanked = "deny"`, `ignore = []`, política multiple-versions documentada.
- CHG-001 / REL: seção no CHANGELOG + tag local `v0.4.0` sem push não autorizado.
- TEST-004 / SCP: gaps_v040 cobre wire, schema, path partial, preserve, script e2e E10–E14.


## Política

- PROIBIDO: declarar inventário Fechado enquanto qualquer gap permanecer Aberto.
- PROIBIDO: waiver eterno de RUSTSEC / CVE sem tracking fechado na mesma release.
- PROIBIDO: `git push` ou publish no crates.io sem autorização explícita do mantenedor.
- PROIBIDO: logar ou colar segredos reais no inventário, notas do checklist ou logs de CI.
- OBRIGATÓRIO: escritas multi-linha de inventário / CHANGELOG usam atomwrite (ou escrita atômica equivalente).
- OBRIGATÓRIO: Status Resolvido só com código + teste + nota de versão em `gaps.md`.


## Referência

- [../gaps.md](../gaps.md) — inventário canônico de gaps
- [../deny.toml](../deny.toml) — política de supply-chain
- [../scripts/verify_install_resolve.sh](../scripts/verify_install_resolve.sh) — gate de re-resolve de install
- [../tests/gaps_v039_integration.rs](../tests/gaps_v039_integration.rs) — gates residuais LOG/JSON/CLI/DOC/DENY/CHG
- [../tests/gaps_v040_integration.rs](../tests/gaps_v040_integration.rs) — gates residuais SCP/IO/DOC-004/REL-004
- [schemas/vps-show.schema.json](schemas/vps-show.schema.json) — password `null` | mascarado `***`
- [schemas/scp-transfer.schema.json](schemas/scp-transfer.schema.json) — JSON de sucesso SCP (só arquivos)
- [schemas/tunnel-listening.schema.json](schemas/tunnel-listening.schema.json) — evento de bind do tunnel
- [schemas/README.md](schemas/README.md) — índice de schemas (linha de produto 0.4.0)
