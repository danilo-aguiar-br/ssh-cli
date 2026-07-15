# gaps.md — ssh-cli v0.4.1

## Metadados

| Campo | Valor |
|-------|--------|
| Versão de código | **0.4.1** (`Cargo.toml`) |
| Data | **2026-07-15** |
| Escopo | Fechamento AUD-POST (EXP-001, TUN-002, CLI-005, CLI-006, IO-009, REL-006) + ship 0.4.1 |
| Status deste inventário | **Fechado (0 Abertos)** |
| Suite de regressão | `gaps_v038` + `gaps_v039` + `gaps_v040` + **`gaps_v041`** + e2e E01–E14 |
| Supply-chain | **russh 0.62.2**; `yanked=deny`; `ignore=[]` |
| Telemetria | **Ausente** |
| Publish | Local ready; GitHub + crates.io **após OK** explícito do mantenedor |

## Inventário consolidado

### Histórico 0.3.7–0.4.0

Todos **Resolvidos** (LOG/JSON/CLI/DOC/DENY/REL/CHG + SEC + AUD-SCP 010–023 + DOC-004* + IO-007/008 + REL-004/005). Ver histórico em git tag `v0.4.0`.

### Bloco AUD-POST-0.4.0 → **0.4.1**

| ID | Título | Status | Teste / evidência |
|----|--------|--------|-------------------|
| GAP-SSH-EXP-001 | `vps export` redacted com ciphertext de senha vazia | **Resolvido (0.4.1)** | unit `empty_secret_never_encrypted_blob`; integration export sem `sshcli-enc:`; `gap_exp_001_*` |
| GAP-SSH-TUN-002 | Tunnel deadline pós-bind → exit 74 | **Resolvido (0.4.1)** | `AtomicBool` bound; `gap_tun_002_*`; e2e/ad-hoc EC=0 |
| GAP-SSH-CLI-005 | `tunnel` sem password-stdin / key-passphrase | **Resolvido (0.4.1)** | `gap_cli_005_*`; help + conflicts |
| GAP-SSH-CLI-006 | `health-check` sem password-stdin / key override | **Resolvido (0.4.1)** | `gap_cli_006_*`; `aplicar_overrides` key |
| GAP-SSH-IO-009 | JSON SCP sem `event` | **Resolvido (0.4.1)** | `event: scp-transfer`; schema; `gap_io_009_*` |
| GAP-SSH-PROC-001 | `cargo fmt --check` sujo | **Resolvido (0.4.1)** | fmt gate |
| GAP-SSH-REL-006 | Tag 0.4.0 sem EXP/TUN/CLI/IO patch | **Resolvido (0.4.1 local)**; remote c/ OK publish | commit+tag `v0.4.1` |

### Melhorias M1–M6 (dentro de 0.4.1)

| ID | Título | Status |
|----|--------|--------|
| M1 | health-check honra `--replace-host-key` | **Feito** |
| M2 | health-check `--json` → `definir_json_erros` | **Feito** |
| M3 | tunnel usa `aplicar_overrides` | **Feito** |
| M4 | revalidação EXP unit/integration | **Feito** |
| M5 | tabela gap→teste neste inventário | **Feito** |
| M6 | CHANGELOG honesty residual 0.4.0 | **Feito** |

## PA — contra-medidas

| PA | Contra-medida | Status |
|----|---------------|--------|
| PA-EXP-01..03 | empty secret + testes | **Feito** |
| PA-TUN-02 | bound flag pós-bind | **Feito** |
| PA-CLI-05 | flags tunnel | **Feito** |
| PA-CLI-06 | flags health-check | **Feito** |
| PA-IO-09 | event scp-transfer | **Feito** |
| PA-M1 / M2 | replace_host_key + json erros health | **Feito** |
| PA-TEST-041 | gaps_v041 | **Feito** |
| PA-DOC-041 | product line + CHANGELOG + honesty AUD-POST em checklists/root docs (DOC-041 follow-up: export vazio sem `sshcli-enc:`, tunnel exit 0 pós-bind, auth flags tunnel/health, `event` scp-transfer, gaps_v041) | **Feito** |
| PA-REL-06 | release 0.4.1 | **Feito local** (push/publish com OK) |

## Política

- **Resolvido** = código + teste + inventário
- Inventário **Fechado** com **0 Abertos**
- Sem telemetria
- Docs longos: atomwrite
- One-shot: nascer → executar → morrer
- PROIBIDO push/publish sem OK do mantenedor

## Tabela gap→teste (0.4.1)

| Gap | Teste |
|-----|-------|
| EXP-001 | `empty_secret_never_encrypted_blob`; `export_redacted_nao_contem_senha`; `gap_exp_001_serializar_empty_source` |
| TUN-002 | `gap_tun_002_bound_flag_source`; ad-hoc/e2e tunnel |
| CLI-005 | `gap_cli_005_tunnel_help_auth_flags`; `gap_cli_005_tunnel_password_stdin_conflict` |
| CLI-006 | `gap_cli_006_health_help_auth_flags`; `gap_cli_006_health_password_stdin_conflict` |
| IO-009 | `gap_io_009_scp_event_schema`; `gap_scp_021` (event required) |
| REL-006 | `gap_rel_006_changelog_041`; `gap_version_041` |

## Resumo quantitativo

| Métrica | Valor |
|---------|--------|
| Gaps AUD-POST resolvidos | **6** (EXP/TUN/CLI×2/IO/REL) |
| Melhorias M1–M6 | **6 Feito** |
| Abertos | **0** |
| russh | **0.62.2** |
| Telemetria | Ausente |
| Versão | **0.4.1** |

## Causa raiz (resumo)

### EXP-001
`serializar_segredo` cifrava string vazia → early-return `""`.

### TUN-002
Deadline pós-bind tratado como erro → bound flag distingue sucesso one-shot.

### CLI-005 / CLI-006
Subset de flags vs exec/scp → paridade clap + `aplicar_overrides`.

### IO-009
JSON SCP sem discriminador → `event: "scp-transfer"` + schema.

### REL-006
Fixes só no tree dirty → release **0.4.1**.
