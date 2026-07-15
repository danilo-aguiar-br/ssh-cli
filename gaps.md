# gaps.md — ssh-cli v0.3.9 (inventário Fechado)

## Metadados

| Campo | Valor |
|-------|--------|
| Versão de código | **0.3.9** (`Cargo.toml`) |
| Commit | ver `git log -1` após Release v0.3.9 |
| Data | **2026-07-15** |
| Escopo | Residuais da auditoria pós-0.3.8 + gates release; suite `gaps_v039` |
| Status deste inventário | **Fechado** — **0** gaps Abertos |
| Suite de regressão | `tests/gaps_v038_integration.rs` + `tests/gaps_v039_integration.rs` |
| Supply-chain | **russh 0.62.2**; `yanked=deny`; `ignore=[]` |
| Telemetria | Ausente |

## Inventário consolidado

### Histórico 0.3.7 (23) + 0.3.8 (12)

Todos **Resolvidos** — ver seções anteriores e commit `94941e4` (0.3.8).

### Bloco auditoria pós-0.3.8 → 0.3.9 (7)

| ID | Título | Status |
|----|--------|--------|
| GAP-SSH-LOG-001 | Tracing default INFO polui stderr (JSON/tunnel) | **Resolvido (0.3.9)** — default **error**; `-v` → debug |
| GAP-SSH-JSON-001 | password `"***"` em VPS key-only (vazio) | **Resolvido (0.3.9)** — `null` se vazio |
| GAP-SSH-CLI-004 | `health-check` sem `--timeout` | **Resolvido (0.3.9)** |
| GAP-SSH-DOC-003 | Docs product line ainda 0.3.6 | **Resolvido (0.3.9)** |
| GAP-SSH-DENY-002 | Warnings duplicate crates no deny | **Resolvido (0.3.9)** — política `multiple-versions=warn` documentada; sem ignore CVE |
| GAP-SSH-REL-003 | Tag local `v0.3.8` ausente | **Resolvido (0.3.9)** — tags locais `v0.3.8` + `v0.3.9` (sem push) |
| GAP-SSH-CHG-001 | Âncoras CHANGELOG 0.3.8/0.3.9 | **Resolvido (0.3.9)** |

---

# Detalhe 0.3.9

## GAP-SSH-LOG-001

| Campo | Valor |
|-------|--------|
| Status | **Resolvido (0.3.9)** |
| Causa raiz | `inicializar_logs` default `info` → prosa em stderr com envelope JSON |
| Código | `src/cli.rs` `inicializar_logs` → default **error** |
| Teste | `gap_log_001_tunnel_json_stderr_sem_info_prosa` |
| Cadeia | B (I/O agent-first) estendida a stderr |

## GAP-SSH-JSON-001

| Status | **Resolvido (0.3.9)** |
| Código | `output::registro_para_json_mascarado` + texto show |
| Política | senha vazia → JSON `null` / texto "(não definida)"; não-vazia → `***` |
| Testes | `gap_json_001_*` + unit `registro_para_json_mascarado_password_null_quando_vazio` |

## GAP-SSH-CLI-004

| Status | **Resolvido (0.3.9)** |
| Código | `Comando::HealthCheck { timeout }`; `aplicar_overrides` em `executar_health_check` |
| Teste | `gap_cli_004_health_check_aceita_timeout` |

## GAP-SSH-DOC-003

| Status | **Resolvido (0.3.9)** |
| Arquivos | `llms.txt`, `llms.pt-BR.txt`, `llms-full.txt`, `README*.md`, `docs/AGENTS*.md`, `INTEGRATIONS*.md` |
| Teste | `gap_doc_003_version_contem_039` |

## GAP-SSH-DENY-002

| Status | **Resolvido (0.3.9)** |
| Código | `deny.toml` comentário + `multiple-versions = "warn"` (sem rebaixar yanked/CVE) |
| Nota | Duplicatas aead/chacha20 são transitivas russh + chacha20poly1305 — warn aceitável |
| Teste | `gap_deny_002_deny_toml_sem_ignore_cve` |

## GAP-SSH-REL-003 / CHG-001

| REL-003 | tags locais `v0.3.8` (commit 94941e4) e `v0.3.9` (HEAD release) |
| CHG-001 | seções e âncoras `[0.3.9]` / `[0.3.8]` no CHANGELOG |
| Teste | `gap_chg_001_changelog_tem_039` |

---

## Plano de ação PA (auditoria 0.3.8) — todos Feitos

| PA | Gap | Status |
|----|-----|--------|
| PA-A1/A2 | LOG-001 | **Feito** |
| PA-A3 | DOC-003 | **Feito** |
| PA-A4 | REL-003 | **Feito** |
| PA-A5 | JSON-001 | **Feito** |
| PA-A6 | CHG/DENY | **Feito** |
| PA-A7 | CLI-004 | **Feito** |
| PA-A8 | inventário honestidade | **Feito** (este arquivo) |

---

## Resumo quantitativo 0.3.9

| Métrica | Valor |
|---------|--------|
| Gaps abertos | **0** |
| Gaps 0.3.9 resolvidos | **7** |
| Gaps legados 0.3.7+0.3.8 | **35** Resolvidos |
| russh | **0.62.2** |
| Telemetria | Ausente |

## Política

- Status **Resolvido** só com código + teste + nota de versão.
- Inventário **Fechado** somente com 0 Abertos.
- Sem telemetria.
- Escrita: atomwrite em docs longos.

## Referências

- Auditoria profunda 2026-07-15 (pós-0.3.8)
- GraphRAG: stdin/stdout, one-shot, ssh, hardening
- context7 / docs-rs / duckduckgo-search-cli (sessão de auditoria)
- Código: `cli.rs` logs, `output.rs` mask, `vps` health-check timeout, `deny.toml`
- Testes: `tests/gaps_v039_integration.rs`
