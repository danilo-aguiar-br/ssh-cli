# Contribuindo para ssh-cli

- Read this document in [English](CONTRIBUTING.md).


## Boas-vindas
- Obrigado por contribuir com código, docs, testes ou bug reports.
- Cada melhoria fortalece SSH multi-host one-shot para agentes de IA.
- Este guia mira onboarding em menos de 10 minutos do clone ao primeiro teste.


## Início rápido
- Clone o repositório e entre na raiz do workspace.
- Valide uma árvore limpa com os comandos abaixo.

```bash
timeout 120 cargo check --all-targets --locked
timeout 300 cargo test --locked
timeout 60 bash scripts/verify_install_resolve.sh
```


## Setup de desenvolvimento
### Requisitos de toolchain
- Exija MSRV Rust 1.85.0 declarado em `Cargo.toml`.
- Instale Rust via `rustup` e prefira o toolchain pinado quando existir.
- Mantenha `Cargo.lock` commitado porque este crate entrega uma CLI binária.
- Nunca suba MSRV sem issue explícita de discussão.

### Pins de dependência
- A linha de produto **0.5.0** usa **russh 0.62.2** (desde 0.3.8) sem os pins COMPAT RC antigos; não reintroduza pins RC mortos sem issue.
- Nunca rode `cargo update` cego no grafo crypto.
- Rode `scripts/verify_install_resolve.sh` após qualquer mudança de dependência.


## Estratégia de branches
- Mantenha `main` como branch de integração.
- Use `feature/<short-kebab>` para features.
- Use `fix/<short-kebab>` para correções.
- Use `docs/<short-kebab>` para documentação.
- Use `chore/<short-kebab>` para tooling e manutenção.


## Convenção de commits
- Siga Conventional Commits 1.0.0 em branches compartilhadas.
- Use `feat` para features visíveis.
- Use `fix` para bug fixes.
- Use `docs` para mudanças só de documentação.
- Use `test` para mudanças só de testes.
- Use `chore` para manutenção.
- Nunca adicione linhas `Co-authored-by` para agentes de IA.


## Processo de Pull Request
- Abra PR com problema claro e comandos de validação.
- Inclua docs bilíngues quando documentos públicos mudarem.
- Preserve comportamento one-shot em todo comando de produto.
- Proíba introduzir packaging de daemon de longa duração ou telemetria.
- Peça review só após `cargo test --locked` e clippy passarem.


## Testes
- Leia [docs/TESTING.pt-BR.md](docs/TESTING.pt-BR.md) para categorias e perfis.
- Prefira unit tests determinísticos para packing e migração de schema.
- Use integration tests em `tests/` para contratos da CLI.
- Inclua as suites de regressão de gaps `tests/gaps_v038_integration.rs`, `tests/gaps_v039_integration.rs` e `tests/gaps_v040_integration.rs` / `tests/gaps_v041_integration.rs` (SCP/tunnel/IO 0.4.0 + AUD-POST 0.4.2) ao tocar superfície residual de auditoria.
- Para E2E SSH real local, prefira env `SSH_CLI_E2E_*`, ou `bash scripts/e2e_real_ssh.sh --from-grok-config` só em maintainer lendo `/.grok/config.toml`; a matriz oficial é **E01–E14** (E10–E14 cobrem SCP upload/download/`cmp`/ausente/preserve); nunca logue credenciais; nunca faça commit de config Grok/MCP ou inventário de hosts neste repositório.
- Testes que precisam de secrets em claro devem definir `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`.
- Nunca deixe testes flaky dependentes de rede sem timeout.


## Documentação
- Aplique o framework bilíngue em toda doc pública.
- Espelhe inglês e `.pt-BR` na mesma entrega.
- Abra todo documento público com link cruzado de idioma.
- Mantenha tom persuasivo fora de SKILL.md e schemas.
- Indexe todo schema JSON em `docs/schemas/README.md`.


## Reportar bugs
- Abra issue no GitHub com reprodução e esperado versus atual.
- Inclua OS, arquitetura, `ssh-cli --version` e exit code.
- Omita ou mascare segredos em logs e no histórico de comandos.


## Solicitar features
- Abra issue descrevendo o workflow do agente e o gap de paridade de automação SSH se houver.
- Prefira features que preservem one-shot e storage XDG multi-host.


## Processo de release
- Suba SemVer em `Cargo.toml` e atualize ambos os CHANGELOGs.
- Rode suite completa, clippy `-D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` e gate de install.
- Confirme docs bilíngues da raiz (README, SECURITY, INTEGRATIONS, llms*) alinhadas à superfície do release (inclui `secrets`, cifragem default, SCP file-only + honestidade 0.3.9, schema `scp-transfer` com `event`, `tunnel --json` / exit 0 pós-bind, honestidade export empty-secret, paridade auth tunnel/health e gaps_v041).
- Empacote com `cargo package --locked` e dry-run de publish quando necessário.
- Tag `vX.Y.Z` só após gates verdes e **autorização explícita do maintainer**.
- Prefira `cargo install ssh-cli --locked` na doc pública de install.
- Nunca publique segredos, inventários reais de hosts ou master-keys.


## Reconhecimento
- Contribuidores são creditados nas notas de release quando desejarem crédito público.
- Pesquisadores de segurança seguem [SECURITY.pt-BR.md](SECURITY.pt-BR.md) para crédito privado.


## Perguntas
- Abra discussion ou issue para dúvidas de processo.
- Contate o maintainer em daniloaguiarbr@proton.me para coordenação privada.
