# gaps.md — ssh-cli v0.4.2 (AUD-E2E Fechado)

## Metadados

| Campo | Valor |
|-------|--------|
| Versão de código | **0.4.2** (`Cargo.toml`) |
| Data | **2026-07-15** |
| Escopo | Ship TUN-003 + IO-010 + UX-001 + REL-007 + ENV-001 + DOC-042 + SCP-024 |
| Suite | `gaps_v042` + suites históricas 038–041 |
| Telemetria | **Ausente** (`doctor.telemetry: false`) |
| Publish remoto | **STOP** — só com OK explícito do mantenedor |

## Inventário — Fechado (0 Abertos)

### Histórico 0.4.1 (ship)

| ID | Status |
|----|--------|
| GAP-SSH-EXP-001 | **Resolvido (0.4.1)** |
| GAP-SSH-TUN-002 | **Resolvido (0.4.1)** |
| GAP-SSH-CLI-005 | **Resolvido (0.4.1)** |
| GAP-SSH-CLI-006 | **Resolvido (0.4.1)** |
| GAP-SSH-IO-009 | **Resolvido (0.4.1)** |
| GAP-SSH-REL-006 | **Resolvido (0.4.1)** |

### AUD-E2E → 0.4.2

| ID | Título | Status |
|----|--------|--------|
| GAP-SSH-TUN-003 | Tunnel `porta_local=0` reporta porta OS via `local_addr()` | **Resolvido (0.4.2)** |
| GAP-SSH-IO-010 | SCP remote missing → exit **66** | **Resolvido (0.4.2)** |
| GAP-SSH-UX-001 | `vps export --json` envelope `event: vps-export` | **Resolvido (0.4.2)** |
| GAP-SSH-REL-007 | `.commit_hash` + build.rs precedence (crates.io) | **Resolvido (0.4.2)** |
| GAP-SSH-ENV-001 | e2e policy fail2ban / max 1 auth-neg | **Resolvido (0.4.2)** |
| GAP-SSH-DOC-042 | tunnel posicional + porta 0 efêmera | **Resolvido (0.4.2)** |
| GAP-SSH-SCP-024 | e2e symlink E16 | **Resolvido (0.4.2)** |
| GAP-SSH-INV-001 | gaps.md versionado | **Resolvido** |
| GAP-SSH-REL-008 | release 0.4.2 local (push/publish c/ OK) | **Resolvido local** |

## Tabela gap → teste (0.4.2)

| Gap | Teste |
|-----|--------|
| TUN-003 | unit tunnel + `gap_tun_003_*` + e2e E15 |
| IO-010 | unit `interpretar_status_scp_no_such_file` + `gap_io_010_*` + e2e E13 ec=66 |
| UX-001 | `gap_ux_001_export_json_*` + schema vps-export |
| REL-007 | `gap_rel_007_build_rs_precedence` + `.commit_hash` |
| ENV-001 | `gap_env_001_e2e_script_auth_policy` |
| DOC-042 | `gap_doc_042_tunnel_positional_skills` |
| SCP-024 | e2e E16 |
| REL-008 | `gap_rel_008_changelog_042` + version 0.4.2 |

## Honesty

- Ban TCP na VPS pós-auditoria = **fail2ban** (ENV-001 / senhas erradas em massa), **não** bug TUN-003.
- One-shot: nascer → executar → morrer. Zero telemetria.
- Publish GitHub + crates.io: **somente com OK**.

## PA status

Todos PA-TUN-03*, PA-IO-10, PA-UX-01, PA-REL-07, PA-ENV-01, PA-DOC-042, PA-SCP-24, PA-TEST-042, PA-GATE-042, PA-REL-08 (local): **Feito**.

## Resumo

| Métrica | Valor |
|---------|--------|
| Abertos | **0** |
| Versão | **0.4.2** |
| Telemetria | Ausente |
