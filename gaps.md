# gaps.md — ssh-cli v0.4.0

## Metadados

| Campo | Valor |
|-------|--------|
| Versão de código | **0.4.0** (`Cargo.toml`) |
| Data | **2026-07-15** |
| Escopo | Fechamento total AUD-SCP-2026-07-15 + ship honesto |
| Status deste inventário | **Fechado (0 Abertos)** |
| Suite de regressão | `gaps_v038` + `gaps_v039` + `gaps_v040` + e2e E01–E13 |
| Supply-chain | **russh 0.62.2**; `yanked=deny`; `ignore=[]` |
| Telemetria | Ausente |

## Inventário consolidado

### Histórico 0.3.7–0.3.9

Todos **Resolvidos** (ver seções históricas / commits anteriores). SEC-001..003 Resolvidos.

### Bloco AUD-SCP-2026-07-15 → **0.4.0** (18)

| ID | Título | Status | Teste / evidência |
|----|--------|--------|-------------------|
| GAP-SSH-SCP-010 | Header `\\n` literal | **Resolvido (0.4.0)** | unit `formatar_header_upload_scp_*` |
| GAP-SSH-SCP-011 | ACK/EOF sem `0x00` | **Resolvido (0.4.0)** | unit + e2e E10–E12 |
| GAP-SSH-SCP-012 | Upload sem status remoto | **Resolvido (0.4.0)** | `scp_aguardar_status` |
| GAP-SSH-SCP-013 | Download header/ACK | **Resolvido (0.4.0)** | e2e E11–E12 |
| GAP-SSH-SCP-014 | Path shell-escape | **Resolvido (0.4.0)** | unit `comando_scp_remoto_escapa_path` + e2e path espaço |
| GAP-SSH-SCP-015 | Unit cristalizava bug | **Resolvido (0.4.0)** | unit proíbe `\`+`n` literal |
| GAP-SSH-SCP-016 | E2E oficial sem scp | **Resolvido (0.4.0)** | `e2e_real_ssh.sh` E10–E13; `gap_e2e_script_e10_e12` |
| GAP-SSH-SCP-017 | Flags scp sem paridade | **Resolvido (0.4.0)** | `gap_scp_017_*` |
| GAP-SSH-SCP-018 | Upload `fs::read` total | **Resolvido (0.4.0)** | stream 32 KiB; `gap_scp_022_partial_suffix_na_fonte` |
| GAP-SSH-SCP-019 | Sem `-r` / dirs | **Resolvido (0.4.0)** por design: erro tipado + docs file-only | `gap_scp_019_*` + DOC-004 |
| GAP-SSH-SCP-020 | Sucesso hardcoded PT | **Resolvido (0.4.0)** | i18n `ScpUpload/DownloadConcluido` |
| GAP-SSH-SCP-021 | Schema JSON transfer | **Resolvido (0.4.0)** | `docs/schemas/scp-transfer.schema.json` |
| GAP-SSH-SCP-022 | Download parcial no disco | **Resolvido (0.4.0)** | `.ssh-cli.partial` + rename; cleanup on err |
| GAP-SSH-SCP-023 | Preserve mtime/mode | **Resolvido (0.4.0)** | linha `T` + mode `C0mmm` + `set_times` |
| GAP-SSH-REL-004 | 0.3.9 anunciava SCP quebrado | **Resolvido (0.4.0)** | CHANGELOG honesty + ship 0.4.0 |
| GAP-SSH-DOC-004 | Docs sem file-only / alerta 0.3.9 | **Resolvido (0.4.0)** | README/MIGRATION/product line |
| GAP-SSH-TEST-004 | Integração scp só surface | **Resolvido (0.4.0)** | `gaps_v040` + `scp_integration` flags |
| GAP-SSH-IO-007 | Sucesso scp sem JSON | **Resolvido (0.4.0)** | `imprimir_transferencia_json` |
| GAP-SSH-SCP-001 | Valida local antes connect | **Resolvido** (pré) | `gap_scp_001_*` |

## PA-SCP (todos Feitos em 0.4.0)

| PA | Status |
|----|--------|
| PA-SCP-01..13 | **Feito** |

## Política

- **Resolvido** = código + teste + nota de versão **0.4.0**
- Inventário **Fechado** somente com 0 Abertos
- Sem telemetria
- Escrita docs longos: atomwrite
- One-shot: nascer → transferir → morrer

## Resumo quantitativo 0.4.0

| Métrica | Valor |
|---------|--------|
| Gaps abertos | **0** |
| Gaps AUD-SCP resolvidos nesta release | **18** |
| russh | **0.62.2** |
| Telemetria | Ausente |
| E2E | E01–E13 |

## Referências

- Código: `src/ssh/cliente.rs`, `src/scp.rs`, `src/cli.rs`, `src/output.rs`, `src/i18n.rs`
- Testes: `tests/gaps_v040_integration.rs`, unit wire, `scripts/e2e_real_ssh.sh`
- Schemas: `docs/schemas/scp-transfer.schema.json`
