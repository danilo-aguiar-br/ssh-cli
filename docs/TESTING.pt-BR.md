# Guia de testes

> Rode o perfil certo de testes do ssh-cli sem travar em redes remotas.

- Leia este documento em [inglês](TESTING.md).
- Linha de produto: **0.5.1** (suites históricas residuais incluem **0.4.1** AUD-POST / `gaps_v041`).


## Por que testes categorizados
- Unit tests protegem packing, schema, secrets AEAD e lógica pura sem servidores SSH.
- Integration tests protegem contratos da CLI, storage e snapshots.
- Testes live remotos são opcionais e devem sempre usar timeouts rígidos e nunca logar credenciais.
- Gates de install resolve protegem o onboarding no crates.io (GAP-014).
- Suites residuais de gaps travam I/O de agente, exit codes, supply chain, mascaramento, wire SCP e honestidade de docs.


## Categorias de teste
- Unit tests dentro de módulos `src/**` (inclui cifragem padrão de `secrets`)
- CLI e2e em `tests/e2e_cli.rs`
- Gap/residual integration em `tests/gaps_v035_integration.rs` (só secrets fake)
- Suite residual de I/O de agente em `tests/gaps_v037_integration.rs`
- Suite residual pós-0.3.7 em `tests/gaps_v038_integration.rs`
- Suite residual pós-0.3.8 em `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC/DENY/CHG)
- Suite residual pós-0.3.9 / **0.4.0** em `tests/gaps_v040_integration.rs`
- Suite AUD-POST em `tests/gaps_v041_integration.rs` (EXP-001 export empty, TUN-002 exit 0 pós-bind, CLI-005/006 paridade auth, IO-009 `event: scp-transfer`, REL-006, DOC-041 honesty)
- Suite AUD-E2E em `tests/gaps_v042_integration.rs` (TUN-003, IO-010, UX-001, REL-007, ENV-001, DOC-042, SCP-024)
- Suite **0.5.1** em `tests/gaps_v051_integration.rs` (export TOML padrão, JSON `vps-export`, dual-read schema v3, evento secrets-init, guarda include-secrets, CRUD `vps-added`, empty command, import exit 65)
- Storage integration em `tests/storage_integration.rs`
- Snapshot tests em `tests/snapshot_tests.rs`
- Superfície SCP em `tests/scp_integration.rs`
- Superfície tunnel em `tests/tunnel_integration.rs`
- Property tests em `tests/proptest_tests.rs`
- i18n integration em `tests/i18n_integration.rs`
- Script de install resolve `scripts/verify_install_resolve.sh`
- Gate de identificadores em inglês `scripts/check_en_identifiers.sh`
- E2E SSH real (opcional, local da máquina): `scripts/e2e_real_ssh.sh` — matriz oficial **E01–E16** (E10–E14 cobrem SCP upload/download/cmp/missing/preserve)
- Benchmarks em `benches/` (manual)


## Como rodar
### Loop local do desenvolvedor

```bash
cargo test --locked --all-targets
cargo clippy --all-targets --locked -- -D warnings
bash scripts/check_en_identifiers.sh
cargo build --release
bash scripts/verify_install_resolve.sh
```

### Perfis focados

```bash
cargo test --locked --test e2e_cli
cargo test --locked --test gaps_v035_integration
cargo test --locked --test gaps_v037_integration
cargo test --locked --test gaps_v038_integration
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo test --locked --test gaps_v042_integration
cargo test --locked --test gaps_v051_integration
cargo test --locked --test storage_integration
cargo test --locked --test snapshot_tests
cargo test --locked packing
cargo test --locked secrets::
```

### E2E SSH real (nunca imprimir segredos)

```bash
# Preferido em CI / máquinas compartilhadas: só env (nunca commite esses valores)
export SSH_CLI_E2E_HOST=… SSH_CLI_E2E_USER=… SSH_CLI_E2E_PASSWORD=…
bash scripts/e2e_real_ssh.sh

# Só do mantenedor local: parse de $HOME/.grok/config.toml (ssh-flowaiper MCP).
# Esse arquivo deve ficar em $HOME — nunca copie para este repositório.
bash scripts/e2e_real_ssh.sh --from-grok-config
```

- Matriz oficial **E01–E16**; **E10–E14** = SCP upload, download, integridade (`cmp`), remoto ausente, preserve mode+mtime (SCP-023).
- O script imprime só rótulos PASS/FAIL — nunca host, user ou password.
- **Política GAP-014 / fail2ban:** prefira `sshd` local ou VPS throwaway. **PROIBIDO:** tempestades de falha de autenticação em hosts de produção (ban do fail2ban). E2E em VPS de produção só com cuidado, whitelist de IP / `ignoreip`, e **sem** senhas erradas intencionais.


## Perfis de CI
- Este repositório atualmente embarca sem workflows de GitHub Actions.
- Mantenedores rodam o loop local do desenvolvedor antes de cada publish.
- Gates de publish incluem package dry-run, verificação de install resolve, paridade bilíngue de docs, checagem de identificadores em inglês (`bash scripts/check_en_identifiers.sh`), suites residuais `gaps_v040` + `gaps_v041` + `gaps_v042` + **`gaps_v051`**, mais o loop canônico: `cargo test --locked --all-targets`, clippy `-D warnings` e `cargo build --release`.


## Variáveis de ambiente
- `SSH_CLI_HOME` isola config durante testes.
- `--config-dir` nas invocações da CLI é preferido para registries temporários.
- `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` faz opt-out da cifragem padrão em testes que assertam TOML plaintext.
- Sem esse opt-out, a primeira gravação de segredo auto-cria `secrets.key` e cifra campos.
- Nível de tracing padrão é error; não espere prosa INFO em stderr por omissão.
- `RUST_LOG` sobrescreve o filtro padrão ao diagnosticar falhas.
- `-v` ativa tracing debug sem definir `RUST_LOG`.
- `NO_COLOR=1` estabiliza saída sensível a snapshot quando necessário.
- Nunca coloque senhas de hosts live em env vars que os testes imprimem.


## Troubleshooting
- Drift de snapshot: revise `tests/snapshots/` e atualize só mudanças intencionais de UI (incluindo strings de versão).
- Falhas de resolve de crypto: recheque pins e rode de novo o script de install sem ignorar a política do lock.
- Testes de timeout flaky: garanta que nenhum host remoto real seja necessário salvo configuração explícita.
- Falhas de permissão: confirme que dirs temporários são graváveis e que asserts de mode batem com o SO.
- Surpresas de fixture cifrada: defina `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` ou forneça primary-key de teste via env.
- Stderr quiet inesperado: o padrão é tracing error; defina `RUST_LOG` ou `-v` se precisar de linhas debug.
- Falhas residuais de SCP / AUD-POST / 0.5.1: rode `cargo test --locked --test gaps_v040_integration`, `cargo test --locked --test gaps_v041_integration`, `cargo test --locked --test gaps_v042_integration` e `cargo test --locked --test gaps_v051_integration`; leia os blocos AUD-SCP, AUD-POST e 0.5.1 em `gaps.md`.
