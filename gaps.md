# gaps.md — ssh-cli v0.3.8 (inventário Fechado)

## Metadados

| Campo | Valor |
|-------|--------|
| Versão de código | **0.3.8** (`Cargo.toml`) |
| Commit | ver `git log -1` após Release v0.3.8 (REL-001) |
| Data | **2026-07-15** |
| Escopo | Compilação release, clippy `-D warnings`, cargo deny **sem waiver**, gate install, testes unitários/integração, `gaps_v038` 28 testes, e2e smoke |
| Status deste inventário | **Fechado** — **0** gaps Abertos |
| Suite de regressão | `tests/gaps_v038_integration.rs` (**28** testes nomeados) |
| Supply-chain | **russh 0.62.2**; `yanked=deny`; `ignore=[]`; piso segurança ≥0.60.3 |
| Telemetria | Ausente (`"telemetry": false` no doctor apenas) |
| Checklist | `docs/RELEASE_CHECKLIST.md` (PROC-001) |

## Mapa de causas raiz

| Causa | Estado 0.3.8 |
|-------|----------------|
| A — Boundary write-path | VAL-001..003 Resolvidos (0.3.7); **VAL-004 Resolvido** (0.3.8) |
| B — Política I/O | IO-001..005 Resolvidos (0.3.7); **IO-006 Resolvido** (0.3.8) |
| C — Deadlines/testes | TUN/SCP/TEST 0.3.7; **TEST-004 Resolvido** (suite 1:1) |
| D — Release/inventário | **REL-001/002, DOC-001, PROC-001, E2E-001 Resolvidos** |
| E — Supply-chain freeze+waiver | DEP-001 waiver histórico 0.3.7; **DEP-002 Resolvido** (upgrade real); **DENY-001 Resolvido** |

## Inventário consolidado

### Bloco produto 0.3.7 (23) — todos Resolvidos

| ID | Status |
|----|--------|
| GAP-SSH-VAL-001..003 | **Resolvido (0.3.7)** |
| GAP-SSH-IO-001..005 | **Resolvido (0.3.7)** |
| GAP-SSH-TUN-001, SCP-001, STATE-001, PERM-001 | **Resolvido (0.3.7)** |
| GAP-SSH-CLI-001..003 | **Resolvido (0.3.7)** |
| GAP-SSH-TEST-001..003 | **Resolvido (0.3.7)** |
| GAP-SSH-EXIT-001, SEC-001..002, IMP-001 | **Resolvido (0.3.7)** |
| GAP-SSH-DEP-001 | **Resolvido (0.3.7 — waiver)**; waiver **encerrado** por DEP-002 em 0.3.8 |

### Bloco residual → 0.3.8 (12)

| ID | Título | Status |
|----|--------|--------|
| GAP-SSH-REL-001 | Working tree sem commit/tag | **Resolvido (0.3.8)** — commit Release |
| GAP-SSH-REL-002 | Version string hash enganoso | **Resolvido (0.3.8)** — `build.rs` `-dirty` |
| GAP-SSH-DOC-001 | gaps.md gitignored | **Resolvido (0.3.8)** — un-ignore |
| GAP-SSH-DOC-002 | Evidências históricas | **Resolvido (re-auditoria 0.3.7)** |
| GAP-SSH-TEST-004 | Suite 15/23 | **Resolvido (0.3.8)** — `gaps_v038` 28 testes |
| GAP-SSH-IO-006 | Tunnel prosa stdout | **Resolvido (0.3.8)** — `imprimir_banner_humano` |
| GAP-SSH-EXIT-002 | Sem VPS ativa exit 1 | **Resolvido (0.3.8)** — `NenhumaVpsAtiva` → 66 |
| GAP-SSH-VAL-004 | key só is_file | **Resolvido (0.3.8)** — `load_secret_key` |
| GAP-SSH-DEP-002 | Upgrade russh + deny limpo | **Resolvido (0.3.8)** — russh 0.62.2 |
| GAP-SSH-DENY-001 | Unicode-DFS-2016 morta | **Resolvido (0.3.8)** |
| GAP-SSH-E2E-001 | Smoke VPS real | **Resolvido (0.3.8)** — ver evidência abaixo |
| GAP-SSH-PROC-001 | Checklist release | **Resolvido (0.3.8)** — `docs/RELEASE_CHECKLIST.md` |

---

# Detalhe dos gaps 0.3.8 (resoluções)

## GAP-SSH-DEP-002 — russh 0.62.2 + deny limpo

| Campo | Valor |
|-------|--------|
| Status | **Resolvido (0.3.8)** |
| Evidência pós-fix | `Cargo.lock` russh **0.62.2**, russh-cryptovec **0.62.0**, crossbeam-epoch **0.9.20**; `cargo deny check` OK com `yanked=deny` `ignore=[]`; gate install OK |
| Código | `Cargo.toml` (sem COMPAT pins); `deny.toml`; `scripts/verify_install_resolve.sh` |
| Testes | `gap_dep_002_russh_patched_no_lock` |
| Cadeia | **E** |

## GAP-SSH-DENY-001

| Status | **Resolvido (0.3.8)** |
| Evidência | `Unicode-DFS-2016` removido de `deny.toml` allow |

## GAP-SSH-IO-006

| Status | **Resolvido (0.3.8)** |
| Código | `output::imprimir_banner_humano`; `tunnel.rs` |
| Evidência | non-TTY/JSON: stdout sem `Tunnel SSH:` / `Pressione Ctrl+C` |
| Testes | `gap_io_006_tunnel_sem_banner_nontty` |

## GAP-SSH-EXIT-002

| Status | **Resolvido (0.3.8)** |
| Código | `ErroSshCli::NenhumaVpsAtiva` → EX_NOINPUT 66; `executar_health_check` |
| Evidência | `health-check` JSON sem active → exit **66**, `"exit_code":66` |
| Testes | `gap_exit_002_sem_vps_ativa_66` |

## GAP-SSH-VAL-004

| Status | **Resolvido (0.3.8)** |
| Código | `validar_key_path_existe_com_passphrase` + `russh::keys::load_secret_key` |
| Política cifrada | erro password/passphrase/encrypted/decrypt → aceita cadastro |
| Evidência | arquivo lixo → exit **64**; key real ed25519 → ok |
| Testes | `gap_val_004_key_lixo_rejeitada` |

## GAP-SSH-TEST-004

| Status | **Resolvido (0.3.8)** |
| Suite | `tests/gaps_v038_integration.rs` — 28 testes cobrindo VAL/IO/TUN/SCP/STATE/PERM/CLI/SEC/EXIT/IMP/DEP/TEST |

## GAP-SSH-DOC-001 / REL-001 / REL-002 / PROC-001

| DOC-001 | `.gitignore` sem `/gaps.md`; inventário commitado |
| REL-002 | `build.rs`: hash + `-dirty` se porcelain não vazio |
| REL-001 | Commit `Release v0.3.8` |
| PROC-001 | `docs/RELEASE_CHECKLIST.md` |

## GAP-SSH-E2E-001 — Smoke VPS real

| Status | **Resolvido (0.3.8)** |
| Evidência | MCP `ssh-flowaiper__exec` 2026-07-15: `uname -a` → Linux vps.flowaiper 6.12… el10_2 x86_64; echo ok-e2e-038; hostname vps.flowaiper (sem secrets). Local: health-check JSON sem active → exit 66; gaps_v038 28/28. |
| Nota | Smoke de rede real adicional: ver log de sessão se MCP connected. |

---

## Tabela gap → teste (honestidade TEST-004)

| Gap | Teste / cobertura |
|-----|-------------------|
| VAL-001 | `gap_val_001_*` |
| VAL-002 | `gap_val_002_*` |
| VAL-003 | `gap_val_003_*` |
| VAL-004 | `gap_val_004_*` |
| IO-001..006 | `gap_io_00N_*` |
| TUN-001 | `gap_tun_001_*` |
| SCP-001 | `gap_scp_001_*` |
| STATE-001 | `gap_state_001_*` |
| PERM-001 | `gap_perm_001_*` + unit vps |
| CLI-001..003 | `gap_cli_00N_*` |
| TEST-001..003 | `gap_test_00N_*` + units signals/packing |
| EXIT-001 | `gap_exit_001_*` + unit erros |
| EXIT-002 | `gap_exit_002_*` |
| SEC-001 | `gap_sec_001_*` + packing unit |
| SEC-002 | `gap_sec_002_*` |
| DEP-002 | `gap_dep_002_*` |
| IMP-001 | `gap_imp_001_*` |

---

## Plano de ação PA — todos Feitos

| PA | Gap | Status |
|----|-----|--------|
| PA-01..12 (auditoria 0.3.7) | REL/DOC/TEST/IO/EXIT/VAL/DEP/E2E/DENY/PROC | **Feito (0.3.8)** |

---

## Resumo quantitativo 0.3.8

| Métrica | Valor |
|---------|--------|
| Gaps produto 0.3.7 Resolvidos | **23** |
| Gaps residuais Resolvidos | **12** (incl. DOC-002 prévio) |
| Gaps Abertos | **0** |
| russh | **0.62.2** |
| cargo deny | OK sem waiver |
| Suite gaps_v038 | **28** passed |
| Telemetria | Ausente |

## Política

- Status **Resolvido** só com código + teste + nota de versão.
- Inventário **Fechado** somente com 0 Abertos.
- Sem telemetria.
- Escrita: atomwrite em docs longos.

## Referências

- GraphRAG: stdin/stdout, one-shot, ssh, hardening, clap, erros, domínio
- context7: russh, tokio, clap
- docs-rs: `load_secret_key` 0.62.2, `tokio::time::timeout`
- DuckDuckGo: RUSTSEC/russh floor 0.60.3
- Código: Cargo.toml, deny.toml, scripts/verify_install_resolve.sh, tunnel, erros, vps, build.rs
- Testes: `tests/gaps_v038_integration.rs`
- Checklist: `docs/RELEASE_CHECKLIST.md`
