# gaps.md — ssh-cli v0.4.2 (+ residual EN closed 2026-07-15b)

## Metadados

| Campo | Valor |
|-------|--------|
| Versão de código | **0.4.2** (`Cargo.toml`) |
| Data | **2026-07-15** |
| Escopo | AUD-E2E + GraphRAG Rules Rust EN (core + residual) |
| Suite | `cargo test --all-targets` + `cargo clippy --all-targets -- -D warnings` + `scripts/check_en_identifiers.sh` |
| Telemetria | **Ausente** |
| Publish remoto | **STOP** — só com OK explícito |

## Inventário AUD-E2E 0.4.2 — Fechado

| ID | Status |
|----|--------|
| TUN-003, IO-010, UX-001, REL-007, ENV-001, DOC-042, SCP-024, REL-008 local | **Resolvido (0.4.2)** |

## Inventário AUD-RULES-RUST-2026-07-15

| ID | Severidade | Título | Status |
|----|------------|--------|--------|
| **GAP-RUST-EN-001** | Crítica | Módulos EN (`masking`, `client`, `model`, `errors`) | **Resolvido** |
| **GAP-RUST-EN-002** | Crítica | Funções/campos/identificadores EN + `serde(rename)` wire TOML | **Resolvido** |
| **GAP-RUST-EN-002b** | Alta | Locals/params/test helpers residual PT | **Resolvido** |
| **GAP-RUST-EN-003** | Alta | thiserror Display EN + tipos `SshCliError` | **Resolvido** |
| **GAP-RUST-EN-004** | Alta | clap about/help EN | **Resolvido** |
| **GAP-RUST-EN-005** | Alta | Doc comments / `//!` EN | **Resolvido** |
| **GAP-RUST-EN-005b** | Alta | Field help / residual rustdoc PT | **Resolvido** |
| **GAP-RUST-EN-006** | Média | `init_primary_key` + keyring primary (+ legacy) | **Resolvido** |
| **GAP-RUST-EN-007** | Média | SAFETY 4 linhas + lints unsafe | **Resolvido** |
| **GAP-RUST-EN-008** | Baixa | SPDX headers em `.rs` | **Resolvido** |
| **GAP-RUST-EN-009** | Alta | UI via `Message` (CRUD success) | **Resolvido** |
| **GAP-RUST-EN-009b** | Alta | details/list labels + technical errors EN; no PT product literals | **Resolvido** |
| **GAP-RUST-EN-MIX-001** | Crítica | Mistura PT/EN no mesmo `.rs` | **Resolvido** |
| **GAP-RUST-EN-010** | Processo | Bulk rename fuzzy proibido | **Mitigado** (string-aware + atomwrite) |
| **GAP-RUST-PROC-001** | Alta | CI/script anti-PT identifiers | **Resolvido** (`scripts/check_en_identifiers.sh` + workflow) |
| **GAP-RUST-META-001** | Alta | inventário 0 contradiz residual | **Resolvido** (reaberto e refechado honestamente) |
| **GAP-RUST-TEST-001** | Baixa | Nomes de testes PT | **Resolvido** (src + tests major) |
| **GAP-RUST-DOC-001** | Média | `# Errors` rustdoc canônico | **Parcial** (presente nas APIs críticas; não em 100% das fns) |
| **GAP-RUST-REL-001** | Média | Semver 0.5.0 por rename de API lib | **Aberto (processo)** — versão permanece 0.4.2; publish exige OK + decisão de bump |

### Política

1. Wire TOML permanece PT (`nome`, `porta`, `senha`, …) via `#[serde(rename = …)]`.
2. JSON agent-first permanece EN (`name`, `port`, `local_port`, …).
3. `Message::pt()` strings PT **obrigatórias** (i18n bilíngue) — não são gap.
4. `pub mod erros` re-export legado mantido para compat; preferir `crate::errors`.
5. Keyring legacy `secrets-master-key` read fallback mantido.
6. Publish GitHub/crates.io: **somente com OK**.
7. Zero telemetria.
8. Gate: `bash scripts/check_en_identifiers.sh` deve passar.

## Resumo

| Métrica | Valor |
|---------|--------|
| AUD-E2E abertos | **0** |
| AUD-RULES-RUST abertos (código) | **0** |
| Processo / semver | **GAP-RUST-REL-001** aberto (sem bump 0.5.0 / sem publish) |
| DOC-001 rustdoc sections | **Parcial** (não bloqueante para EN rule) |
| Versão | **0.4.2** |
| Telemetria | Ausente |
