# gaps.md — ssh-cli v0.4.0

## Metadados

| Campo | Valor |
|-------|--------|
| Versão de código | **0.4.0** (`Cargo.toml`) |
| Data | **2026-07-15** |
| Escopo | Fechamento total AUD-SCP-2026-07-15 + SCP-023b + IO-008 |
| Status deste inventário | **Fechado (0 Abertos)** |
| Suite de regressão | `gaps_v038` + `gaps_v039` + `gaps_v040` + e2e E01–E14 |
| Supply-chain | **russh 0.62.2**; `yanked=deny`; `ignore=[]` |
| Telemetria | **Ausente** |

## Inventário consolidado

### Histórico 0.3.7–0.3.9

Todos **Resolvidos** (LOG/JSON/CLI/DOC/DENY/REL/CHG + SEC-001..003).

### Bloco AUD-SCP-2026-07-15 → **0.4.0** (18 + 023b + IO-008)

| ID | Título | Status | Teste / evidência |
|----|--------|--------|-------------------|
| GAP-SSH-SCP-010 | Header `\\n` literal | **Resolvido (0.4.0)** | unit `formatar_header_upload_scp_*` |
| GAP-SSH-SCP-011 | ACK/EOF sem `0x00` | **Resolvido (0.4.0)** | unit + e2e E10–E12 |
| GAP-SSH-SCP-012 | Upload sem status remoto | **Resolvido (0.4.0)** | `scp_aguardar_status` |
| GAP-SSH-SCP-013 | Download header/ACK | **Resolvido (0.4.0)** | e2e E11–E12 |
| GAP-SSH-SCP-014 | Path shell-escape | **Resolvido (0.4.0)** | unit + e2e path espaço |
| GAP-SSH-SCP-015 | Unit cristalizava bug | **Resolvido (0.4.0)** | unit proíbe `\`+`n` literal |
| GAP-SSH-SCP-016 | E2E oficial sem scp | **Resolvido (0.4.0)** | e2e E10–E14; `gap_e2e_script_e10_e12` |
| GAP-SSH-SCP-017 | Flags scp sem paridade | **Resolvido (0.4.0)** | `gap_scp_017_*` |
| GAP-SSH-SCP-018 | Upload `fs::read` total | **Resolvido (0.4.0)** | stream 32 KiB |
| GAP-SSH-SCP-019 | Sem `-r` / dirs | **Resolvido (0.4.0)** por design file-only | `gap_scp_019_*` + DOC-004 |
| GAP-SSH-SCP-020 | Sucesso hardcoded PT | **Resolvido (0.4.0)** | i18n |
| GAP-SSH-SCP-021 | Schema JSON transfer | **Resolvido (0.4.0)** | `scp-transfer.schema.json` |
| GAP-SSH-SCP-022 | Download parcial no disco | **Resolvido (0.4.0)** | `.ssh-cli.partial` + rename + fsync pai |
| GAP-SSH-SCP-023 | Preserve mtime/mode bi-dir | **Resolvido (0.4.0)** | `-tp`/`-fp` + mode + set_times; E14 |
| GAP-SSH-SCP-023b | Download sem -p/mode | **Resolvido (0.4.0)** | `comando_scp_remoto` + `aplicar_mode_local` |
| GAP-SSH-REL-004 | 0.3.9 anunciava SCP quebrado | **Resolvido (0.4.0)** | CHANGELOG honesty |
| GAP-SSH-DOC-004 | Docs sem file-only / alerta 0.3.9 | **Resolvido (0.4.0)** | product line |
| GAP-SSH-TEST-004 | Integração scp só surface | **Resolvido (0.4.0)** | `gaps_v040` |
| GAP-SSH-IO-007 | Sucesso scp sem JSON | **Resolvido (0.4.0)** | `imprimir_transferencia_json` |
| GAP-SSH-IO-008 | `tunnel` sem `--json` local | **Resolvido (0.4.0)** | `gap_io_008_tunnel_json_flag` + `imprimir_tunnel_listening_json` |
| GAP-SSH-HYG-001 | `rust_out` dirty | **Resolvido (0.4.0)** | `.gitignore` |
| GAP-SSH-SCP-001 | Valida local antes connect | **Resolvido** (pré) | suite gaps_v038/v040 |

## PA (todos Feitos)

| PA | Status |
|----|--------|
| PA-SCP-01..13 | **Feito** |
| PA-SCP-023b | **Feito** |
| PA-IO-008 | **Feito** |

## Processo (não é gap de código)

| ID | Título | Status |
|----|--------|--------|
| GAP-SSH-REL-005 | crates.io/GitHub ainda 0.3.9 até publish/push 0.4.0 | **Aguardando OK do mantenedor** (código local completo) |

## Política

- **Resolvido** = código + teste + nota de versão **0.4.0**
- Inventário **Fechado** somente com **0 Abertos** de produto
- Sem telemetria
- Docs longos: atomwrite
- One-shot: nascer → transferir → morrer
- PROIBIDO deixar bugs/gaps de produto para “versão futura”

## Resumo quantitativo 0.4.0

| Métrica | Valor |
|---------|--------|
| Gaps abertos de produto | **0** |
| Gaps AUD-SCP + pós-auditoria resolvidos | **22** |
| russh | **0.62.2** |
| Telemetria | Ausente |
| E2E | E01–E14 |
| Publish | Somente com OK explícito do mantenedor |

## Causa raiz (5 Porquês) — SCP-023b

| Nível | Pergunta | Resposta |
|-------|----------|----------|
| Sintoma | Download mode=644 mtime=now apesar de “SCP-023 resolvido” | E14/ad-hoc falhava no download |
| Por quê 1 | Por que mode/mtime não vinham? | Source remoto não emitia `T` e mode honesto |
| Por quê 2 | Por que o source não emitia? | `scp -f` **sem** `-p` (OpenSSH só envia T com pflag) |
| Por quê 3 | Por que o cliente não pedia -p? | `comando_scp_remoto` usava só `-t`/`-f` |
| Por quê 4 | Por que parse ignorava mode? | `parse_header_scp` retornava só size |
| Por quê 5 | Por que o inventário mentia? | Teste e2e não cobria preserve bi-dir; status “Resolvido” sem E14 |

**Causa raiz:** ausência de `-p` no source remoto + parse/apply incompletos + gate e2e sem preserve.

**Contra-medidas (Feitas):** `-tp`/`-fp` sempre; parse `(mode,size)`; `aplicar_mode_local`; E14; `gap_scp_023_*`.

## Causa raiz — IO-008

| Nível | Resposta |
|-------|----------|
| Sintoma | Agentes sem evento estruturado no tunnel |
| Por quê | Tunnel só banners humanos + sem flag `--json` local |
| Raiz | Paridade agent-first incompleta no subcomando tunnel |
| Contra-medida | `--json` + `event: tunnel_listening` após bind; erros via envelope JSON |

## Tools obrigatórios (sessão)

- GraphRAG: `rules_rust_ssh`, `cli_com_clap`, `cli_stdin_stdout`, `escrita_atomica`, testes/json
- `context7 library russh` (trust 9.7) + docs Channel/exec
- `mcp docs-rs` russh **0.62.2** `Channel` (exec implementa SCP tunnel)
- `duckduckgo-search-cli` OpenSSH scp `-p` / linha T
- `atomwrite` para este inventário

## Referências

- Código: `src/ssh/cliente.rs`, `src/scp.rs`, `src/cli.rs`, `src/tunnel.rs`, `src/output.rs`
- Testes: `tests/gaps_v040_integration.rs`, `tests/tunnel_integration.rs`, unit wire, `scripts/e2e_real_ssh.sh`
- Schemas: `docs/schemas/scp-transfer.schema.json`

## Revalidação (2026-07-15, sessão re-auditoria)

| Gate | Resultado |
|------|-----------|
| Agent team explore (SCP+CLI) | **22/22 PASS** código real |
| `cargo fmt --check` | **OK** (fmt residual em `parse_header_scp`) |
| `cargo clippy --locked --all-targets -D warnings` | **OK** |
| `cargo test --locked` | **OK** (188 unit + integrations; gaps_v040=16) |
| `cargo deny check` | **OK** (advisories/bans/licenses/sources) |
| `scripts/e2e_real_ssh.sh --from-grok-config` | **PASS E01–E14** fails=0 |
| context7 `/eugeny/russh` | trust **9.7** |
| docs-rs russh **0.62.2** `Channel::exec` | SCP via canal documentado |
| duckduckgo-search-cli | OpenSSH scp/`-p` consultado |
| atomwrite | `llms*.txt` DOC-004 residual file-only |
| Telemetria | Ausente (só `telemetry: false` no doctor) |
| Gaps de produto abertos | **0** |
| REL-005 push/publish | **Aguardando OK** do mantenedor |
