# Auditoria Rules Rust (GraphRAG) — ssh-cli 0.4.2

> **Disclaimer (histórico):** auditoria de 2026-07-15 na era **0.4.2**. Para a linha de produto atual, use `gaps.md` (**0.5.1**) e a documentação vigente (`docs/TESTING.md`, `docs/RELEASE_CHECKLIST.md`, etc.). **Não** use este arquivo como checklist de publish.

**Data:** 2026-07-15  
**Escopo:** todos os `src/**/*.rs`, `tests/**/*.rs`, `build.rs`, `Cargo.toml`  
**Fontes de regra:** `docs_rules/rules_rust_codigo_ingles_internacionalizacao.md` + checklist crates.io  
**Tools usados:** context7 (clap/tracing), docs-rs (thiserror, russh 0.62.2), duckduckgo-search-cli, atomwrite  
**Telemetria:** ausente (OK)

## Veredito executivo

| Dimensão | Status | Severidade |
|----------|--------|------------|
| Identificadores em inglês | **NÃO CONFORME** | Crítica |
| Comentários/docs em inglês | **NÃO CONFORME** | Crítica |
| Mistura PT/EN no mesmo `.rs` | **NÃO CONFORME** | Crítica |
| thiserror / mensagens técnicas | **NÃO CONFORME** (hardcoded PT) | Alta |
| clap help/about | **NÃO CONFORME** (PT) | Alta |
| Vocabulário inclusivo (`master-key`) | **Parcial** | Média |
| SAFETY em `unsafe` | **Parcial** (comentários curtos) | Média |
| SPDX headers em `.rs` | **Ausente** | Baixa |
| Cargo.toml description | **OK** (inglês) | — |
| Telemetria | **OK** (ausente) | — |
| i18n bilíngue en/pt-BR | **Parcial** (enum existe; UI literals fora do enum) | Alta |
| Wire format TOML (`nome`,`senha`,…) | Estável; exige `serde(rename)` na migração EN | Bloqueante |

**Métrica de densidade PT (pré-migração):** ~2744 hits de tokens PT em 25 arquivos; ~155 funções com nomenclatura PT-like.  
**Tentativa de migração bulk 2026-07-15:** abortada e **revertida** — renomes curtos (`porta`, `nome`, `Tunnel`, `mod modelo`) + fuzzy corromperam sintaxe (`TunnelActive` sem `{`, `pub mod model` no lugar de `exit_codes`, funções de teste truncadas).  
**Política de correção:** renome ordenado longest-first, `-w` apenas, compile após cada lote, `serde(rename=…)` em campos TOML, **sem** renome de substrings genéricas.

## Inventário de gaps (acionáveis)

| ID | Gap | Evidência | Ação |
|----|-----|-----------|------|
| GAP-RUST-EN-001 | Módulos PT: `erros`, `mascaramento`, `ssh/cliente`, `vps/modelo` | paths | rename → `errors`, `masking`, `client`, `model` |
| GAP-RUST-EN-002 | Tipos PT: `ErroSshCli`, `ResultadoSshCli`, `VpsRegistro`, `ClienteSsh`, … | `src/**` | English types |
| GAP-RUST-EN-003 | Funções PT: `executar_*`, `imprimir_*`, `definir_*`, `obter_*`, … | ~155 fns | English verbs |
| GAP-RUST-EN-004 | Campos domínio PT: `nome`,`porta`,`usuario`,`senha`,`senha_su`,… | `vps/modelo.rs` | EN + `#[serde(rename)]` wire PT |
| GAP-RUST-EN-005 | Comentários/`//!`/`///` em PT | quase todos `.rs` | inglês idiomático rustdoc |
| GAP-RUST-EN-006 | `#[error("…")]` thiserror em PT | `erros.rs` | inglês técnico; UI via i18n |
| GAP-RUST-EN-007 | clap `about`/`long_about`/help em PT | `cli.rs` | inglês (crates.io) |
| GAP-RUST-EN-008 | `init_master_key` / keyring `secrets-master-key` | `secrets.rs` | `primary_key` + alias compat keyring |
| GAP-RUST-EN-009 | SAFETY fraco em `unsafe` env/tests | `vps/*`, `platform/windows` | SAFETY 4 linhas + lints |
| GAP-RUST-EN-010 | UI literals fora de `Mensagem` | `output.rs`, `erros` Display | só i18n enum |
| GAP-RUST-EN-011 | SPDX License-Identifier ausente | 0 arquivos | header MIT OR Apache-2.0 |
| GAP-RUST-EN-012 | Cargo.toml comments em PT | `Cargo.toml` | inglês |
| GAP-RUST-EN-013 | build.rs docs em PT | `build.rs` | inglês |
| GAP-RUST-EN-014 | Tests/docs asserts em strings PT | `erros.rs` tests, snapshots | atualizar com EN técnico |
| GAP-RUST-DOC-001 | rustdoc sections `# Errors` incompletas | APIs `Result` | seções canônicas |
| GAP-RUST-DOC-002 | Doc comments PT em 3ª pessoa PT | lib/modules | voz EN |
| N/A | no_std/wasm/KaTeX/Mermaid full | CLI host | N/A produto |

## Conformidades positivas

- `Cargo.toml` description, keywords, categories, rust-version, docs.rs metadata presentes
- `#![warn(missing_docs)]` no lib
- Exit codes sysexits estáveis (contrato agent-first)
- JSON agent keys já em inglês (`name`, `port`, `event`, …)
- i18n tem `en`/`pt` match e `sys-locale`
- Sem telemetria; doctor `telemetry: false`
- atomwrite obrigatório para writes; sem publish sem OK

## Cadeia 5 Whys (EN-001…007)

1. Sintoma: crate não idiomatic crates.io/docs.rs (identificadores PT)  
2. Por quê: codebase nasceu agent-first com domínio PT  
3. Por quê: rules EN + i18n coexistiam sem gate CI de idioma  
4. Por quê: testes/gaps focaram comportamento SSH, não lint linguístico  
5. **Raiz:** ausência de enforcement (CI grepping PT identifiers / `cargo deny` custom) + debt estrutural de nomenclatura  

## Plano de ação (sem deferir gaps)

| PA | Entrega | Ordem |
|----|---------|-------|
| PA-EN-01 | `errors` module EN + thiserror EN + exit_codes | 1 |
| PA-EN-02 | module renames + path updates | 2 |
| PA-EN-03 | domain model EN + serde rename wire | 3 |
| PA-EN-04 | functions/verbs EN (longest-first) | 4 |
| PA-EN-05 | clap/docs/comments EN | 5 |
| PA-EN-06 | primary_key naming + SAFETY + SPDX | 6 |
| PA-EN-07 | i18n: Message variants EN; UI only via enum | 7 |
| PA-EN-08 | tests/snapshots/e2e script strings | 8 |
| PA-EN-09 | `cargo test` + clippy + inventário gaps=0 EN | 9 |
| PA-EN-10 | version bump policy: **0.5.0** se API lib pública muda (sem publish sem OK) | 10 |

## Riscos

- Renome de campos TOML sem `serde(rename)` quebra configs de usuários  
- Bulk fuzzy replace **proibido** (lição 2026-07-15)  
- Snapshots help PT → EN exigem `insta` accept  
- Keyring user id `secrets-master-key` precisa alias de leitura

## Status de execução nesta sessão

- [x] Auditoria completa + tools mandatórios  
- [x] Reversão de migração bulk corrompida  
- [x] PA-EN-01 errors module EN + thiserror EN (shipped in tree)
- [x] build.rs + Cargo.toml comments EN
- [x] call sites error variants EN
- [ ] PA-EN-02…10 remaining identifiers/docs (see gaps.md)

## Referências

- thiserror Display messages: inglês técnico (docs.rs/thiserror)  
- clap derive about/help: inglês para crates.io  
- russh 0.62.2 `client::connect` (docs-rs)  



## Execução de fechamento (mesma data)

- [x] EN-001 módulos `masking` / `ssh/client` / `vps/model`
- [x] EN-002 identificadores EN + `serde(rename)` wire TOML
- [x] EN-004 clap about/help EN (snapshot help atualizado)
- [x] EN-005 docs principais EN
- [x] EN-006 `init_primary_key` + keyring primary + legacy `secrets-master-key`
- [x] EN-007 SAFETY expandido + lints `clippy::undocumented_unsafe_blocks` etc.
- [x] EN-008 SPDX em `.rs`
- [x] EN-009 success UI via `Message` + labels output EN
- [x] `cargo check --all-targets` + `cargo test --all-targets` verdes (pós-snapshot)
- Publish: **STOP** sem OK


## Residual closure (2026-07-15b)

- Re-scan found residual PT after first "0 open" close (~346 hits).
- Fixed: identifiers (`load`/`save`/`is_cancelled`/`ScpOptions`/…), technical error strings EN,
  list/details labels EN, clap field docs EN, test names EN, CI gate `scripts/check_en_identifiers.sh`.
- Allowed residual: `Message::pt()` UI, TOML wire `serde(rename)`, `pub mod erros` shim, fixture password data.
- `cargo test --all-targets` + `cargo clippy --all-targets -- -D warnings` green.
- Publish still **STOP** without explicit OK. Semver 0.5.0 decision deferred (GAP-RUST-REL-001 process).
