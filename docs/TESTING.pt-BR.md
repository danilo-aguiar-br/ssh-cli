# Guia de testes

> Rode o perfil certo de testes do ssh-cli sem travar em redes remotas.

- Leia este documento em [inglês](TESTING.md).
- Linha de produto: **0.4.0**.


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
- Suite residual pós-0.3.9 / **0.4.0** em `tests/gaps_v040_integration.rs`; suite AUD-POST **0.4.1** em `tests/gaps_v041_integration.rs` (EXP-001, TUN-002, CLI-005/006, IO-009, REL-006)
- Storage integration em `tests/storage_integration.rs`
- Snapshot tests em `tests/snapshot_tests.rs`
- Superfície SCP em `tests/scp_integration.rs`
- Superfície tunnel em `tests/tunnel_integration.rs`
- Property tests em `tests/proptest_tests.rs`
- i18n integration em `tests/i18n_integration.rs`
- Script de install resolve `scripts/verify_install_resolve.sh`
- E2E SSH real (opcional, local da máquina): `scripts/e2e_real_ssh.sh` — matriz oficial **E01–E14** (E10–E14 cobrem SCP upload/download/cmp/missing/preserve)
- Benchmarks em `benches/` (manual)


## Como rodar
### Loop local do desenvolvedor

```bash
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
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

- Matriz oficial **E01–E14**; **E10–E14** = SCP upload, download, integridade (`cmp`), remoto ausente, preserve mode+mtime (SCP-023).
- O script imprime só rótulos PASS/FAIL — nunca host, user ou password.


## Perfis de CI
- Este repositório atualmente embarca sem workflows de GitHub Actions.
- Mantenedores rodam o loop local do desenvolvedor antes de cada publish.
- Gates de publish incluem package dry-run, verificação de install resolve, paridade bilíngue de docs e suites residuais `gaps_v040` + `gaps_v041`.


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
- Surpresas de fixture cifrada: defina `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` ou forneça master-key de teste via env.
- Stderr quiet inesperado: o padrão é tracing error; defina `RUST_LOG` ou `-v` se precisar de linhas debug.
- Falhas residuais de SCP: rode `cargo test --locked --test gaps_v040_integration` e leia o bloco AUD-SCP em `gaps.md`.
