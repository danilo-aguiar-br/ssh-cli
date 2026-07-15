# gaps.md — ssh-cli v0.4.2 (+ AUD-RULES-RUST-2026-07-15 closed)

## Metadados

| Campo | Valor |
|-------|--------|
| Versão de código | **0.4.2** (`Cargo.toml`) |
| Data | **2026-07-15** |
| Escopo | AUD-E2E + auditoria GraphRAG Rules Rust (idioma EN) |
| Suite | `cargo test --all-targets` + `cargo check` |
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
| **GAP-RUST-EN-003** | Alta | thiserror Display EN + tipos `SshCliError` | **Resolvido** |
| **GAP-RUST-EN-004** | Alta | clap about/help em inglês | **Resolvido** |
| **GAP-RUST-EN-005** | Alta | Doc comments / `//!` em inglês (módulos principais + clap) | **Resolvido** (residual PT só em i18n `pt()` e strings UI pt-BR) |
| **GAP-RUST-EN-006** | Média | `init_primary_key` + keyring `secrets-primary-key` (+ legacy alias) | **Resolvido** |
| **GAP-RUST-EN-007** | Média | SAFETY 4 linhas + lints unsafe no `lib.rs` | **Resolvido** |
| **GAP-RUST-EN-008** | Baixa | SPDX headers em `.rs` | **Resolvido** |
| **GAP-RUST-EN-009** | Alta | UI humana via enum `Message` (CRUD success paths) | **Resolvido** (labels doctor/details em EN técnico) |
| **GAP-RUST-EN-010** | Processo | Bulk rename fuzzy proibido | **Mitigado** (string-aware /tmp + atomwrite write) |

### Política

1. Wire TOML permanece PT (`nome`, `porta`, `senha`, …) via `#[serde(rename = …)]`.
2. JSON agent-first permanece EN (`name`, `port`, `local_port`, …).
3. Publish GitHub/crates.io: **somente com OK**.
4. Zero telemetria.

## Resumo

| Métrica | Valor |
|---------|--------|
| AUD-E2E abertos | **0** |
| AUD-RULES-RUST abertos | **0** (checklist product-N/A: wasm/no_std/KaTeX) |
| Versão | **0.4.2** |
| Telemetria | Ausente |
