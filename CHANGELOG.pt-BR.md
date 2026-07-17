# Changelog

- Read this document in [English](CHANGELOG.md).

Todas as mudanças notáveis deste projeto são documentadas neste arquivo.

O formato segue [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
e o versionamento segue [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2026-07-17

### Corrigido
- **Roundtrip export/import agent-first**: corpo default de `vps export` é **TOML** mesmo em non-TTY; JSON só com `--json`. Import aceita TOML (chaves EN+PT) e envelopes JSON `vps-export` (GAP-AUD-001/022).
- **Wire dual-read**: deserializa EN + aliases PT legados; serializa chaves em inglês; schema **v3**; default `added_at` quando ausente (GAP-AUD-002/021). Substitui a nota de wire 0.5.0 (chaves PT só via `serde(rename)`).
- **JSON de `secrets init` / `reencrypt`** (`event: secrets-init|secrets-reencrypt`) via `--json` ou `--output-format json` (GAP-AUD-003).
- Erro de comando vazio é técnico em inglês (`empty command`) em qualquer locale (GAP-AUD-004).
- Caminhos de sucesso CRUD/connect/import emitem JSON estruturado quando o formato é JSON (GAP-AUD-008).
- Mensagem SCP remoto ausente normalizada para `file not found: <path>` (GAP-AUD-025); EC 66 mantido.
- Erros de parse TOML no import mapeiam para sysexits **65** (`TomlDe`) (GAP-AUD-012).
- Exit de `SshAuthentication` alinhado a **77** (GAP-AUD-020).
- Timeouts `< 1000` ms emitem warning em stderr (GAP-AUD-009).
- `--include-secrets` em pipe/non-TTY exige `--output` ou `--i-understand-secrets-on-stdout` (GAP-AUD-011).
- Doctor `secrets_plaintext_opt_out` é JSON **bool** (GAP-AUD-013).
- Hardcodes/tracing residuais em inglês técnico (GAP-AUD-005).

### Adicionado
- Flags CLI: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (camadas env depreciadas, ainda funcionam) (GAP-AUD-006).
- Evento `secrets-key-auto-created` quando a primary-key é provisionada na primeira gravação (GAP-AUD-007).
- Tunnel `--bind` (default `127.0.0.1`) (GAP-AUD-018).
- Warning em stderr de password em argv (GAP-AUD-010).

### Alterado
- Versão **0.5.0 → 0.5.1**.
- Tracing / identificadores residuais padronizados em inglês (GAP-AUD-005).
- Aliases de tipo em português no módulo `erros` marcados como deprecated (GAP-AUD-017).

### Notas
- Sem publish crates.io/GitHub sem OK explícito do maintainer.
- Contratos reais de transferência SCP de 0.5.0 §1.1 não devem regredir.

## [0.5.0] - 2026-07-15

### Corrigido
- **CRÍTICO**: `secrets init --force` reencripta hosts existentes e grava `secrets.key.bak` (GAP-AUD-SEC-001).
- Doctor `permissions` em inglês (`"missing"`).
- Mensagens técnicas, help clap e identificadores residualmente em EN.
- Nomes de VPS com whitespace interno rejeitados (GAP-AUD-VAL-001).

### Alterado
- Semver **0.5.0** por renomeações de API em inglês. Wire TOML ainda usava chaves PT via `serde(rename)` nesta release (**supersedido em 0.5.1** por serialize EN + dual-read EN/PT, schema v3).
- `secrets init` / `reencrypt` via `Message` i18n.

### Notas
- Sem publish crates.io/GitHub sem OK explícito.

## [0.4.2] - 2026-07-15

### Corrigido
- **Tunnel porta efêmera** (`local_port=0`): após bind, JSON/banner reportam a porta **atribuída pelo SO** via `local_addr()` (nunca `0` pós-bind) (GAP-SSH-TUN-003). Schema `local_port.minimum` = 1.
- **SCP remote missing** agora sai com **66** `ArquivoNaoEncontrado` (paridade com missing local) em vez de **74** `CanalFalhou` quando o OpenSSH reporta `No such file` / `not found` (GAP-SSH-IO-010). Erros de protocolo/permissão permanecem 74.

### Adicionado
- `vps export --json` envelope agent-first: `event: "vps-export"`, hosts redacted por padrão, sem `sshcli-enc:` para secrets vazios (GAP-SSH-UX-001 / paridade EXP-001); schema `docs/schemas/vps-export.schema.json`
- Embed de commit hash no pack crates.io: `build.rs` com precedência env → `.commit_hash` → git → `unknown` (GAP-SSH-REL-007)
- e2e oficial **E15** (tunnel porta 0) + **E16** (symlink) + E13 exige exit **66**; política ENV-001/fail2ban no header do script
- Suite `tests/gaps_v042_integration.rs`

### Alterado
- Versão 0.4.1 → **0.4.2**
- Docs/skills: tunnel continua com args **posicionais**; porta `0` = efêmera; confiar em `local_port` do JSON; nunca inventar `--local-port` (GAP-SSH-DOC-042)

### Segurança / honestidade
- Ban TCP na VPS após e2e de auditoria foi **fail2ban** por senhas erradas intencionais (ENV-001), **não** TUN-003.
- Sem telemetria

### Notas
- CLI one-shot: nascer → executar → morrer
- Contratos agent aditivos (PATCH)


## [0.4.1] - 2026-07-15

### Corrigido
- **Export redacted com secret vazio** não emite mais ciphertext `sshcli-enc:v1:…` para senha `""` (GAP-SSH-EXP-001).
- **Deadline do tunnel** após bind local não retorna mais exit **74** quando o agente já recebeu `tunnel_listening` (GAP-SSH-TUN-002). Timeout pré-bind permanece 74.

### Adicionado
- Paridade de flags auth em `tunnel`: `--password-stdin`, `--key-passphrase`, `--key-passphrase-stdin` (GAP-SSH-CLI-005)
- Paridade de flags auth em `health-check`: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` (GAP-SSH-CLI-006)
- Campo JSON SCP `event: \"scp-transfer\"` + schema obrigatório (GAP-SSH-IO-009)
- Suite `tests/gaps_v041_integration.rs`
- `health-check` honra `--replace-host-key` global e envelope JSON de erro com `--json`

### Alterado
- Versão 0.4.0 → **0.4.1**
- Docs/skills de product line com paridade auth e event scp-transfer

### Segurança / honesty
- **Se instalou 0.4.0 do crates.io:** export redacted podia mostrar ciphertext falso de senha vazia; tunnel podia emitir `ok:true` e sair 74. Atualize para **0.4.1**.
- Sem telemetria

### Notas
- CLI one-shot: nascer → executar → morrer
- Contratos agent aditivos apenas (PATCH)

## [0.4.0] - 2026-07-15

### Corrigido
- **Protocolo wire SCP** quebrado no crates.io **0.3.9** (header com `\\n` literal em vez de newline real `0x0a`; ACK/EOF com data vazia em vez do byte `0x00`; status remoto não validado; download com header/terminador incorretos) — SCP-010..013
- Escape shell do path remoto SCP para espaços e meta-caracteres (SCP-014)
- Unit tests não cristalizam mais o header quebrado (SCP-015)
- Download não deixa arquivo final parcial em falha: grava `{path}.ssh-cli.partial` e faz rename atômico (SCP-022); mode/times aplicados no **partial** antes do rename (SCP-022b)
- Upload não carrega o arquivo inteiro em RAM (`fs::read`); stream em chunks de 32 KiB (SCP-018)
- `scp --json` habilita envelope JSON de erro em stderr (paridade com tunnel; IO-007b)
- Mensagens de validação file-only do SCP em i18n EN/PT (SCP-020b)

### Adicionado
- E2E oficial E10–E14 SCP em `scripts/e2e_real_ssh.sh` (upload, download, `cmp`, remoto ausente, preserve mode/mtime) (SCP-016, SCP-023)
- Paridade de flags scp com exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` (SCP-017)
- JSON estruturado de sucesso SCP + `docs/schemas/scp-transfer.schema.json` (IO-007, SCP-021)
- Preserve mtime/mode bi-direcional: remoto `scp -tp`/`-fp`, linha `T` + parse mode `C`, set_permissions + set_times (SCP-023/023b; e2e E14)
- `tunnel --json` emite evento estruturado `tunnel_listening` após bind local (IO-008)
- Mensagens i18n EN/PT de sucesso SCP (SCP-020)
- Suite `tests/gaps_v040_integration.rs` (TEST-004)

### Alterado
- Versão 0.3.9 → **0.4.0**
- Docs de product line documentam **somente arquivos regulares** (sem `-r` / sem SFTP) e a regressão wire SCP de 0.3.9 (DOC-004, SCP-019, REL-004)
- Honestidade da raiz (SECURITY 0.4.x atual, INTEGRATIONS superfície real 0.4.0, CONTRIBUTING gaps_v040) (DOC-004b)
- Honestidade de `docs/*`: AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION/TESTING/RELEASE_CHECKLIST/CROSS_PLATFORM + índice de schemas cobrem SCP file-only, partial, stream 32 KiB, preserve, `scp --json`, `tunnel --json` / `tunnel_listening` e aviso wire 0.3.9 (DOC-004c)
- Honestidade de `skills/*`: skills bilíngues + evals ensinam SCP file-only, JSON scp-transfer, `.ssh-cli.partial`, stream 32 KiB, preserve mtime/mode, tunnel `--json` / `tunnel_listening`, matriz de flags de timeout (DOC-004d)
- Adicionado `docs/schemas/tunnel-listening.schema.json` para o contrato de agente IO-008
- `scp` honra `--replace-host-key` global e `--output-format json` global

### Segurança / honestidade
- **Se você instalou 0.3.9 do crates.io e usou `scp`:** essa release anunciava SCP, mas o wire era inoperante (upload frequentemente gerava arquivo remoto 0 bytes ou timeout). Atualize para **0.4.0**.
- Sem telemetria

### Notas
- CLI one-shot: conectar → transferir → desconectar → sair
- Arquivos grandes: aumente `--timeout` (cobre connect + transferência completa)

## [0.3.9] - 2026-07-15

### Corrigido
- Residuais da auditoria pós-0.3.8: LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL-003, CHG-001
- Tracing default **error** (agent-first); `-v` ativa debug (LOG-001)
- stderr JSON sem prosa INFO por omissão (LOG-001)
- VPS só-chave: `password: null` no JSON (não `"***"`) (JSON-001)
- `health-check --timeout <ms>` alinhado ao exec (CLI-004)
- Docs de product line em **0.3.9** e comportamentos residuais documentados em README, `llms*.txt`, INTEGRATIONS, `docs/*` e skills (auditoria profunda DOC-003)
- Âncoras de compare do CHANGELOG para 0.3.8/0.3.9 (CHG-001)
- `deny.toml` documenta warns multi-version esperados sem ignore de CVE (DENY-002)
- `docs/schemas/vps-show.schema.json` permite `password` com tipo `string | null` (paridade JSON-001)
- Higiene de exposição SEC-001..003: ignore `.setting.cyber/`, E2E recusa grok config no repo, docs usam `demo-password-not-real`

### Adicionado
- Suite `tests/gaps_v039_integration.rs` para gaps residuais de auditoria (incl. SEC-001..003)

### Alterado
- Versão 0.3.8 → 0.3.9
- `exclude` do Cargo inclui `.setting.cyber/` e sidecars sqlite do enrich-queue

### Notas
- Sem telemetria
- Credenciais reais ficam fora da árvore (`~/.config/ssh-cli/`, `$HOME/.grok/config.toml`)

## [0.3.8] - 2026-07-15

### Corrigido
- Gaps residuais pós-auditoria 0.3.7 (IO-006, EXIT-002, VAL-004, TEST-004, DOC-001, REL-001/002, DENY-001, PROC-001, E2E-001)
- Banners do tunnel não poluem stdout de agentes (IO-006)
- Sem VPS ativa retorna exit 66 tipado (EXIT-002)
- Parse OpenSSH de key_path no write-path (VAL-004)
- Suite `gaps_v038_integration` 1:1 (TEST-004)
- Version string com `-dirty` se tree suja (REL-002)
- Inventário `gaps.md` versionado; checklist `docs/RELEASE_CHECKLIST.md`

### Segurança
- Upgrade **russh 0.62.2** (piso ≥0.60.3); remove pins COMPAT RC (DEP-002)
- `cargo deny` sem waivers CVE/yanked; remove license morta Unicode-DFS-2016
- Gate install exige russh patched; permite primefield estável
- crossbeam-epoch ≥0.9.20 (RUSTSEC-2026-0204)

### Alterado
- Versão 0.3.7 → 0.3.8
- Política de `verify_install_resolve.sh` invertida

### Notas
- Sem telemetria
- Fixes de produto 0.3.7 não commitados entram neste commit de release


### Adicionado
- Framework completo de documentação bilíngue (README, CONTRIBUTING, SECURITY, INTEGRATIONS, guias docs, schemas, skills)
- Arquivos de licença dual `LICENSE-MIT` e `LICENSE-APACHE` com MIT OR Apache-2.0

## [0.3.7] - 2026-07-15

### Corrigido
- Todos os 23 gaps de `gaps.md` (VAL/IO/TUN/SCP/STATE/PERM/CLI/TEST/EXIT/SEC/DEP/IMP)
- Write-path de domínio: `validar_e_normalizar`, porta 1..=65535, chave existente (VAL-001..003)
- I/O: `--output-format` no CRUD VPS, `health-check --json`, envelope JSON de erro, `--quiet` silencia sucesso humano, `println!` só em `output` (IO-001..005)
- Tunnel: `--timeout-ms` cobre connect + loop (TUN-001)
- SCP valida arquivo local antes do connect (SCP-001)
- `vps remove` limpa `active` órfão; lock `0o600` (STATE-001, PERM-001)
- `su-exec --password-stdin`; conflitos clap password/*_stdin; completions EPIPE seguro (CLI-001..003)
- Testes de sinais `#[serial]`; snapshot help; assert real de abort (TEST-001..003)
- Falha de comando remoto → exit do processo `1` (não o código remoto) (EXIT-001)
- Senha sudo/su no stdin do canal, não na argv; máscara sempre `***` (SEC-001, SEC-002)
- Import redacted com UX + `--allow-incomplete` (IMP-001)
- `cargo deny` verde com política de pins datada (DEP-001)

### Alterado
- Versão 0.3.6 → 0.3.7
- **Quebra de contrato (agentes):** senhas longas não expõem 12+4; exit remoto ≠0 vira processo `1` com `remote_exit_code` no envelope
- `SSH_CLI_FORCE_TEXT=1` força formato texto

### Segurança
- Sem senha sudo/su em `ps` remoto
- Sem vazamento de prefixo de senha em list/show

## [0.3.6] - 2026-07-15

### Adicionado
- Cifragem at-rest por padrão: auto `secrets.key` (0o600) na primeira gravação
- CLI `secrets status|init|reencrypt` (nunca imprime master-key)
- Opt-out `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` para testes
- Doctor: `secrets_key_file`, `secrets_plaintext_opt_out`
- Script `scripts/e2e_real_ssh.sh` para E2E real sem logar credenciais
- Mensagem de auth falha orienta stdin/key

### Alterado
- Versão 0.3.5 → 0.3.6
- GAP-009 residual: cifragem default (não só opcional)
- Documentação de pin freeze russh/crypto (R-PINS)

### Segurança
- Segredos no TOML cifrados por padrão
- Protocolo E2E proíbe vazar host/user/password

## [0.3.5] - 2026-07-15

### Corrigido
- Residual GAP-007: `vps export` atômico
- Residual GAP-006: abort remoto TERM+KILL
- Residual GAP-009/012: cifragem opcional at-rest (env/file/keyring)
- README sem install sem `--locked`
- Matriz de paridade do gaps.md atualizada

### Adicionado
- Overrides `--key-passphrase` em exec/sudo-exec/su-exec
- JSON automático fora de TTY
- Doctor com `secrets_at_rest` / `secrets_key_source`
- Testes `tests/gaps_v035_integration.rs`

### Alterado
- Versão 0.3.4 → 0.3.5

## [0.3.4] - 2026-07-15

### Fixed
- Grafo crypto de `cargo install`: pin `primefield`, `primeorder`, `ecdsa`, `pkcs5`, `russh = 0.60.0` exato (GAP-014)
- Packing de `sudo-exec` com `sh -c`  (GAP-005)
- Escrita atômica de `config.toml` com tempfile + fsync + flock (GAP-007)
- Host key TOFU via `known_hosts` XDG (GAP-008)
- Dual `max_command_chars` / `max_output_chars` (GAP-004)
- Abort remoto best-effort no timeout (GAP-006)
- Validação de credencial: password ou key obrigatório (GAP-011)

### Added
- Auth por chave privada (`--key`, `key_path`) via russh `load_secret_key` (GAP-002)
- `su-exec` one-shot consumindo `senha_su` (GAP-003)
- Segredos via stdin (`--password-stdin` e pares sudo/su) (GAP-009)
- `vps doctor`, `vps export`, `vps import` (GAP-012)
- Tunnel com `--timeout-ms` obrigatório (GAP-010)
- `--disable-sudo`, `--description`, `--replace-host-key`
- Schema v2 multi-host XDG
- Gate de install: `scripts/verify_install_resolve.sh`

### Changed
- Timeout default 60000 ms 
- `directories` 5 → 6 (GAP-013)
- Versão 0.3.3 → 0.3.4
- Dual license MIT OR Apache-2.0

## [0.3.3] - 2026-07-15

### Changed
- Migração de ownership e repositório para `danilo-aguiar-br` após ban da conta GitHub anterior.
- `repository` / `homepage` apontam para `https://github.com/danilo-aguiar-br/ssh-cli`.
- Metadados de autor atualizados para `Danilo Aguiar <daniloaguiarbr@proton.me>`.
- Workflows GitHub Actions e badges de CI removidos.

### Note
- crates.io já tinha versões até `0.3.2` da conta anterior; este release é o primeiro sob o novo owner.

## [0.2.1] - 2026-04-16

### Fixed
- Pin `elliptic-curve = "=0.14.0-rc.30"` para corrigir falha de `cargo install ssh-cli`

## [0.2.0] - 2026-04-15

### Added
- Fix de piping de senha sudo-exec com `printf '%s\n'`
- Overrides de runtime em exec/sudo-exec/scp/tunnel
- Aliases camelCase para LLMs

## [0.1.0] - 2026-04-14

Release inicial.

[Unreleased]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.9...v0.4.0
[0.3.9]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.8...v0.3.9
[0.3.8]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.7...v0.3.8
[0.3.7]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/danilo-aguiar-br/ssh-cli/releases/tag/v0.3.3
[0.2.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/danilo-aguiar-br/ssh-cli/releases/tag/v0.1.0
