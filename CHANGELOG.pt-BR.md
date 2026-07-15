# Changelog

- Read this document in [English](CHANGELOG.md).

Todas as mudanças notáveis deste projeto são documentadas neste arquivo.

O formato segue [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
e o versionamento segue [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.0...HEAD
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
