# Guia de testes

> Rode o perfil certo de testes do ssh-cli sem travar em redes remotas.

- Read this document in [English](TESTING.md).
- Linha de produto: **0.3.6**.


## Por que testes categorizados
- Unit tests protegem packing, schema, AEAD de secrets e lógica pura sem servidores SSH.
- Integration tests protegem contratos da CLI, storage e snapshots.
- Testes live remotos são opcionais e devem sempre usar timeout rígido e nunca logar credenciais.
- Gates de install resolve protegem o onboarding no crates.io (GAP-014).


## Categorias de teste
- Unit tests em módulos `src/**` (inclui cifragem default em `secrets`)
- CLI e2e em `tests/e2e_cli.rs`
- Integração de gaps residuais em `tests/gaps_v035_integration.rs` (só secrets fake)
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
cargo test --locked --test storage_integration
cargo test --locked --test snapshot_tests
cargo test --locked packing
cargo test --locked secrets::
```

### E2E SSH real (nunca imprimir segredos)

```bash
bash scripts/e2e_real_ssh.sh --from-grok-config
# ou exporte SSH_CLI_E2E_HOST/USER/PASSWORD (e SUDO opcional) e rode sem --from-grok-config
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
- `RUST_LOG` habilita tracing debug em stderr ao diagnosticar falhas.
- `NO_COLOR=1` estabiliza saída sensível a snapshot quando necessário.
- Nunca coloque senhas reais de host em env vars que os testes imprimem.


## Troubleshooting
- Drift de snapshot: revise `tests/snapshots/` e atualize só mudanças intencionais de UI (incluindo versão).
- Falhas de resolve crypto: recheque pins e rode o script de install sem ignorar a política de lock.
- Timeouts flaky: garanta que nenhum host remoto real é exigido salvo configuração explícita.
- Falhas de permissão: confirme dirs temp graváveis e asserções de mode no SO.
- Surpresa de fixture cifrada: defina `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` ou forneça master-key de teste via env.
