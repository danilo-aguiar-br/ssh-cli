# Guia de testes

> Rode o perfil certo de testes do ssh-cli sem travar em redes remotas.

- Leia este documento em [inglês](TESTING.md).
- Linha de produto: **0.3.9**.


## Por que testes categorizados
- Unit tests protegem packing, schema, AEAD de secrets e lógica pura sem servidores SSH.
- Integration tests protegem contratos da CLI, storage e snapshots.
- Testes live remotos são opcionais e devem sempre usar timeout rígido e nunca logar credenciais.
- Gates de install resolve protegem o onboarding no crates.io (GAP-014).
- Suites de gaps residuais travam contratos de I/O de agente, exit codes, supply chain e mascaramento.


## Categorias de teste
- Unit tests em módulos `src/**` (inclui cifragem default em `secrets`)
- CLI e2e em `tests/e2e_cli.rs`
- Integração de gaps residuais em `tests/gaps_v035_integration.rs` (só secrets fake)
- Suite residual de I/O de agente em `tests/gaps_v037_integration.rs`
- Suite residual pós-0.3.7 em `tests/gaps_v038_integration.rs`
- Suite residual pós-0.3.8 em `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC/DENY/CHG)
- Storage integration em `tests/storage_integration.rs`
- Snapshots em `tests/snapshot_tests.rs`
- SCP e tunnel em `tests/`
- Property tests em `tests/proptest_tests.rs`
- i18n em `tests/i18n_integration.rs`
- Script de install resolve `scripts/verify_install_resolve.sh`
- E2E SSH real (opcional, máquina local): `scripts/e2e_real_ssh.sh`
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
cargo test --locked --test storage_integration
cargo test --locked --test snapshot_tests
cargo test --locked packing
cargo test --locked secrets::
```

### E2E SSH real (nunca imprimir segredos)

```bash
# Preferido em CI / máquinas compartilhadas: só env (nunca committe esses valores)
export SSH_CLI_E2E_HOST=… SSH_CLI_E2E_USER=… SSH_CLI_E2E_PASSWORD=…
bash scripts/e2e_real_ssh.sh

# Só em máquina do maintainer: parse de $HOME/.grok/config.toml (MCP ssh-flowaiper).
# Esse arquivo deve ficar em $HOME — nunca copie para este repositório.
bash scripts/e2e_real_ssh.sh --from-grok-config
```


## Perfis de CI
- Este repositório ainda não embarca workflows GitHub Actions.
- Maintainers rodam o loop local antes de cada publish.
- Gates de publish incluem dry-run de package, install resolve e paridade de docs bilíngues.


## Variáveis de ambiente
- `SSH_CLI_HOME` isola config durante testes.
- `--config-dir` nas invocações da CLI é preferido para inventários temporários.
- `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` desliga cifragem default em testes que assertam TOML em claro.
- Sem esse opt-out, a primeira gravação de segredo cria `secrets.key` e cifra campos.
- Nível de tracing padrão é error; não espere prosa INFO em stderr por omissão.
- `RUST_LOG` sobrescreve o filtro padrão ao diagnosticar falhas.
- `-v` habilita tracing debug sem definir `RUST_LOG`.
- `NO_COLOR=1` estabiliza saída sensível a snapshot quando necessário.
- Nunca coloque senhas reais de host em env vars que os testes imprimem.


## Troubleshooting
- Drift de snapshot: revise `tests/snapshots/` e atualize só mudanças intencionais de UI (incluindo versão).
- Falhas de resolve crypto: recheque pins e rode o script de install sem ignorar a política de lock.
- Timeouts flaky: garanta que nenhum host remoto real é exigido salvo configuração explícita.
- Falhas de permissão: confirme dirs temp graváveis e asserções de mode no SO.
- Surpresa de fixture cifrada: defina `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` ou forneça master-key de teste via env.
- Stderr inesperadamente quieto: o padrão é tracing error; defina `RUST_LOG` ou `-v` se precisar de linhas debug.
