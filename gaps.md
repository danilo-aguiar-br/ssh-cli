# gaps.md — ssh-cli v0.5.0 (AUD-E2E-LOCAL 2026-07-15d)

## Metadados

| Campo | Valor |
|-------|--------|
| Versão de código | **0.5.0** (`Cargo.toml`) |
| Data | **2026-07-15** |
| Escopo | Fechamento total de gaps/bugs locais (sem GitHub / sem crates.io) |
| Suite | `cargo test --all-targets` + `cargo clippy --all-targets -- -D warnings` + `scripts/check_en_identifiers.sh` + e2e release |
| Telemetria | **Ausente** |
| Publish remoto | **STOP** — só com OK explícito |

## Inventário — todos fechados em código

| ID | Severidade | Título | Status |
|----|------------|--------|--------|
| GAP-AUD-SEC-001 | Crítica | `secrets init --force` sem reencrypt | **Resolvido** |
| GAP-AUD-I18N-001 | Alta | hardcode `primary-key pronta` | **Resolvido** |
| GAP-AUD-I18N-002 | Média | doctor `ausente` | **Resolvido** |
| GAP-AUD-I18N-003/004 | Média | erros técnicos PT | **Resolvido** |
| GAP-AUD-CLI-001 | Média | clap help PT residual | **Resolvido** |
| GAP-AUD-EN-001 | Baixa | IDs PT residuais (product) | **Resolvido** |
| GAP-AUD-EN-002 | Baixa | Residual PT names (`verify_tofu`, `remote_scp_command`, `use_json`, …) | **Resolvido** |
| GAP-AUD-VAL-001 | Média | VPS name com whitespace interno aceito | **Resolvido** (rejeitado) |
| GAP-AUD-TEST-001 | Baixa | regressão force-init | **Resolvido** |
| GAP-RUST-REL-001 | Processo | Semver 0.5.0 por rename de API | **Resolvido** (bump 0.5.0; **publish não feito**) |
| GAP-RUST-DOC-001 | Baixa | rustdoc `# Errors` em APIs críticas | **Resolvido** (load/save/secrets/paths/tofu) |

### Política

1. Wire TOML permanece PT via `serde(rename)`.
2. JSON agent-first permanece EN.
3. `Message::pt()` strings PT **obrigatórias** (i18n) — não são gap.
4. Erros técnicos / clap help / doctor machine fields → **EN**.
5. Publish GitHub/crates.io: **somente com OK**.
6. Zero telemetria.
7. Gate: `bash scripts/check_en_identifiers.sh` deve passar.

## Resumo

| Métrica | Valor |
|---------|--------|
| Gaps de código abertos | **0** |
| Publish remoto | **Não executado** (STOP) |
| Versão | **0.5.0** |
| Telemetria | Ausente |
