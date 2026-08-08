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
timeout 3600 bash scripts/check_all_gates.sh
```

- Esse comando único roda a bateria obrigatória inteira; os gates abaixo servem para reexecução focada.

```bash
timeout 120 cargo check --all-targets --locked
timeout 300 cargo test --locked
timeout 60 bash scripts/verify_install_resolve.sh
timeout 900 bash scripts/check_cross_targets.sh
```

### Bateria completa de gates (obrigatória antes de declarar rodada fechada)
- Rode `scripts/check_all_gates.sh` antes de afirmar que qualquer gate está verde.
- Ele roda os dez gates obrigatórios numa invocação e reporta todos os resultados.
- Os dez são `fmt`, `build-release`, `build-no-default`, `clippy`, `test`, `deny`, `cross-targets`, `advisory-freshness`, `en-identifiers` e `install-resolve`.
- `scripts/check_advisory_freshness.sh` não tem outro chamador, então pular a bateria pula esse gate inteiro.
- O motivo de existir é estrutural, não estético.
- `cargo clippy` e `cargo test` abortam no primeiro alvo que não compila.
- Um único arquivo de teste quebrado esconde o estado de todo gate atrás dele.
- Foi exatamente assim que o inventário local passou a declarar 835 verdes com quatro gates vermelhos.
- Use `--only ID[,ID...]` para um subconjunto e `--list` para ver os ids.
- A execução reporta o que pulou, então rodada parcial nunca lê como completa.
- A bateria é sequencial por decisão, porque os gates de cargo compartilham um lock de `target/`.

### Gate cross-target (B1 — obrigatório)
- Rode `scripts/check_cross_targets.sh` após tocar qualquer coisa sob `#[cfg(target_os = ...)]`.
- Todo outro gate roda somente para o triple do host.
- Código atrás de um `cfg` estrangeiro é descartado antes do type-check.
- Ele nunca alcança `fmt`, `clippy`, `test` nem `deny`.
- Um placar verde, portanto, não prova nada sobre Windows ou macOS.
- Isso não é hipotético: o alvo Windows falhou com seis erros com todos os outros gates verdes.
- O script faz type-check de `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` e `x86_64-apple-darwin`.
- Windows é checado com `--no-default-features` porque a pilha TLS default puxa `aws-lc-sys`, que compila C.
- Essa exceção é estrutural (A8) e ainda faz type-check de 100% do código `cfg(windows)` do produto.
- Não declare em `docs/CROSS_PLATFORM.pt-BR.md` um alvo que esse script não checa.


## Setup de desenvolvimento
### Requisitos de toolchain
- Exija MSRV Rust 1.85.0 declarado em `Cargo.toml`.
- Instale Rust via `rustup` e prefira o toolchain pinado quando existir.
- Mantenha `Cargo.lock` commitado porque este crate entrega uma CLI binária.
- Nunca suba MSRV sem issue explícita de discussão.

### Pins de dependência
- A linha de produto **0.5.4** usa **russh 0.62.5** (desde 0.3.8) sem os pins COMPAT RC antigos; não reintroduza pins RC mortos sem issue.
- Nunca rode `cargo update` cego no grafo crypto.
- Rode `scripts/verify_install_resolve.sh` após qualquer mudança de dependência.

### Inventário local de auditoria
- `gaps.md` é inventário de auditoria **local** (gitignored; não publicado). Não declare FIXED fazendo grep da prosa desse arquivo (G13/G15 — FIXED exige prova de efeito no destino, ex.: checksums).


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
- Inclua as suites de regressão de gaps ao tocar superfície residual. Nomeie cada uma explicitamente em vez de usar faixa — lista elidida (`v038 … v051`) não satisfaz checagem por `contains` e derruba em silêncio as suites do meio: `tests/gaps_v035_integration.rs`, `tests/gaps_v037_integration.rs`, `tests/gaps_v038_integration.rs`, `tests/gaps_v039_integration.rs`, **`tests/gaps_v040_integration.rs`** (contratos comportamentais de SCP e tunnel), `tests/gaps_v041_integration.rs`, `tests/gaps_v042_integration.rs`, `tests/gaps_v051_integration.rs` (export/import/wire/secrets), `tests/gaps_v052_tls_policy.rs`, `tests/gaps_v053_domain_types.rs`, `tests/gaps_v054_error_handling.rs`, `tests/gaps_v055_unsafe_ffi.rs`, `tests/gaps_v056_ssh.rs`, `tests/gaps_v057_sftp.rs`, `tests/gaps_v058_e2e_residual.rs` (G-E2E residual: ACME permanente, um único `vps-added`, root `schema`/`doctor`, clap sem env, `-v`/`-vv`/`-vvv` graduados, FIXED_MASK, `--use-agent`), **`tests/gaps_v059_agent_native.rs`** (superfície agent-native da 0.5.4: recusa de `--no-input` em `vps add` e `vps edit`, modelagem de payload com `--select`/`--filter`/`--limit`/`--count-only`, guarda de `--bind` fora de loopback), **`tests/gaps_v060_tunnel_modes.rs`** (LOTE E: contratos de argumento de `--reverse`/`--socks5`/`--remote-socket` e o preview de `--dry-run`, ambos pelo binário real) e **`tests/gaps_v064_gate_runner.rs`** (contrato de cobertura da bateria: todo `scripts/check_*.sh` é gate ou exclusão declarada).
- **Gates locais (obrigatórios antes do PR):** `cargo fmt --check`, `cargo test --locked` (e clippy como no processo de release). E2E SSH real é **opcional** quando não há host lab.
- Para E2E SSH real local (G-E2E-05): prefira **`--config-dir`** com hosts já cadastrados via `vps add`, ou `bash scripts/e2e_real_ssh.sh --from-grok-config` só em maintainer lendo `$HOME/.grok/config.toml`. Env harness-only `SSH_CLI_E2E_*` é aceito pelo script (não é store de produto). Sem host lab o script sai **0** com **SKIP** (offline-safe). Binário default: `target/release/ssh-cli`. Matriz oficial **E01–E18** (E10–E14 SCP; E15 tunnel porta 0; E16 symlink; E17/E18 checksum SFTP). Prefira **sshd local** / lab; **sem storm de auth** em hosts de produção com fail2ban; nunca logue credenciais; nunca commite config Grok/MCP ou inventários neste repo.
- Testes que precisam de secrets em claro devem passar **`--allow-plaintext-secrets`** (flag CLI; não é store env de produto).
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
- Confirme docs bilíngues da raiz (README, SECURITY, INTEGRATIONS, llms*) alinhadas à superfície de release corrente **0.5.4**, que mantém todo o conjunto da 0.5.3 fechado e acrescenta as entradas da 0.5.4 registradas no changelog (`--reverse`/`--socks5`/`--remote-socket`, `mtime_preserved`/`durable` no `scp-transfer`, exit codes 69 e 70): G1–G19 fechados (integridade SFTP, `-v`/`-vv`/`-vvv` crate-scoped, cardinalidade de cancel em batch), root `schema`/`doctor`/`commands`/`locale`/`tls`, um único JSON `vps-added` + `secrets_key_auto_created`, `RUST_LOG` ambiente ignorado, ACME `invalidContact`→64, export redacted `***` (`FIXED_MASK`), `vps add --use-agent`, sem GH Actions de produto, `secrets` + cifragem default, wire schema v3 dual-read, SFTP preferindo 0.5.3+, e suites `gaps_v042` + `gaps_v051` + **`gaps_v058`**. Só gates locais: `cargo fmt --check`, `cargo test`, E2E opcional (sem workflows cloud de CI de produto).
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
