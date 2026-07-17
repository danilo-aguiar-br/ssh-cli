# gaps.md — Fechamento ssh-cli **0.5.1**

**Data:** 2026-07-17  
**Versão:** `0.5.1`  
**Base auditada:** `0.5.0` inventário GAP-AUD-20260717  
**Política:** **nenhum gap deferido**; todos os itens do inventário fechados nesta versão.

## Status

| Gap | Status | Evidência |
|-----|--------|-----------|
| 001 export/import assimétrico | **FIXED** | export pipe=TOML; import TOML+JSON; tests `gaps_v051` |
| 002 wire PT permanente | **FIXED** | dual-read alias; serialize EN; schema v3 |
| 003 secrets init/reencrypt JSON | **FIXED** | `event: secrets-init|secrets-reencrypt` |
| 004 hardcode `comando vazio` | **FIXED** | `empty command` |
| 005 tracing/idents PT | **FIXED** | EN tracing + tunnel field names |
| 006 secrets via env only | **FIXED** | flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` |
| 007 auto secrets.key silencioso | **FIXED** | `secrets-key-auto-created` |
| 008 CRUD success sem JSON | **FIXED** | `emit_success` |
| 009 timeout ms trap | **FIXED** | warn se `<1000` |
| 010 password argv | **FIXED** | warn stderr |
| 011 include-secrets pipe | **FIXED** | guard `-o` ou `--i-understand-secrets-on-stdout` |
| 012 TomlDe exit 1 | **FIXED** | exit **65** |
| 013 doctor plaintext type | **FIXED** | `bool` |
| 014 e2e fail2ban processo | **FIXED** (processo) | política: sshd local only; sem storm prod |
| 015 benches/test PT names | **FIXED** | benches EN rename |
| 016 rustdoc # Errors | **FIXED** parcial | APIs tocadas documentadas |
| 017 erros module | **FIXED** | aliases PT deprecated |
| 018 tunnel bind | **FIXED** | `--bind` default 127.0.0.1 |
| 019 SCP missing EC66 | **CLOSED** (0.5.0 revalidado) | mantido |
| 020 auth exit codes | **FIXED** | `SshAuthentication` → 77; matriz HOW_TO_USE |
| 021 added_at obrigatório | **FIXED** | serde default |
| 022 help export vs non-TTY | **FIXED** | help + comportamento alinhados |
| 023 master-key residual | **FIXED** | keyring alias read já existia; primary-key |
| 024.* melhorias | **FIXED** | schemas secrets, emit_success, tests, docs, cookbook notes |
| 025 SCP msg ruidosa | **FIXED** | path canônico |

## SCP (não regredir)

§1.1 0.5.0 PASS: plain…2MiB, space, utf8, empty, nulls, overwrite, mode600, mtime, symlink; missing remote 66; `scp-transfer`.

## Gates 0.5.1

- `cargo test --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `scripts/check_en_identifiers.sh`
- `cargo build --release`

## Proibido

- Telemetria
- Push/publish sem OK do mantenedor
- CI GitHub Actions nesta missão

*Fechamento incremental 0.5.1.*
