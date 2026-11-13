# Contribuindo para ssh-cli


- Leia este documento em [Inglês (en)](CONTRIBUTING.md)


## Boas-vindas
- Obrigado por contribuir com código, docs, testes ou bug reports
- Cada melhoria fortalece SSH multi-host one-shot para agentes de IA
- Este guia mira onboarding em menos de 10 minutos do clone ao primeiro teste


## Início rápido
- Clone o repositório e entre na raiz do workspace
- Valide uma árvore limpa com os comandos abaixo

```bash
timeout 3600 python3 scripts/check_all_gates.py
```

- Esse comando único roda a bateria obrigatória inteira
- Os gates abaixo servem para reexecução focada
- Os scripts exigem Python 3.9+ e só a biblioteca padrão
- No Windows digite `py -3` onde este guia diz `python3`, e ajuste orçamentos com flags como `--timeout-secs`, nunca com variáveis de ambiente

```bash
timeout 120 cargo check --all-targets --locked
timeout 300 cargo test --locked --all-features --no-fail-fast
timeout 60 python3 scripts/verify_install_resolve.py
timeout 900 python3 scripts/check_cross_targets.py
```

### Bateria completa de gates (obrigatória antes de declarar rodada fechada)
- Rode `scripts/check_all_gates.py` antes de afirmar que qualquer gate está verde
- Ele roda os catorze gates obrigatórios numa invocação e reporta todos os resultados
- Os catorze são `fmt`, `build-release`, `build-no-default`, `clippy`, `doc`, `test`, `exec-target`, `deny`, `cross-targets`, `advisory-freshness`, `advisory-hits`, `en-identifiers`, `markdown-format` e `install-resolve`
- O gate `test` roda `cargo test --locked --all-features --no-fail-fast`
- `--no-fail-fast` faz uma rodada vermelha mostrar todos os testes que falham em vez de parar no primeiro
- `scripts/check_advisory_freshness.py` não tem outro chamador, então pular a bateria pula esse gate inteiro
- O motivo de existir é estrutural, não estético
- `cargo clippy` e `cargo test` abortam no primeiro alvo que não compila
- Um único arquivo de teste quebrado esconde o estado de todo gate atrás dele
- Foi exatamente assim que o inventário local passou a declarar 835 verdes com quatro gates vermelhos
- Use `--only ID[,ID...]` para um subconjunto e `--list` para ver os ids
- A execução reporta o que pulou, então rodada parcial nunca lê como completa
- A bateria é sequencial por decisão, porque os gates de cargo compartilham um lock de `target/`

### Gate cross-target (B1 — obrigatório)
- Rode `scripts/check_cross_targets.py` após tocar qualquer coisa sob `#[cfg(target_os = ...)]`
- Todo outro gate roda somente para o triple do host
- Código atrás de um `cfg` estrangeiro é descartado antes do type-check, então nunca alcança `fmt`, `clippy`, `test` nem `deny`
- Um placar verde, portanto, não prova nada sobre Windows ou macOS
- Isso não é hipotético: o alvo Windows falhou com seis erros com todos os outros gates verdes
- O script faz type-check de `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` e `x86_64-apple-darwin`
- Windows é checado com `--no-default-features` porque a pilha TLS default puxa `aws-lc-sys`, que compila C
- Essa exceção é estrutural (A8) e ainda faz type-check de 100% do código `cfg(windows)` do produto
- Não declare em `docs/CROSS_PLATFORM.pt-BR.md` um alvo que esse script não checa


## Código de Conduta
- Este projeto segue o Contributor Covenant 2.1 em [CODE_OF_CONDUCT.pt-BR.md](CODE_OF_CONDUCT.pt-BR.md)
- Ao participar você concorda em respeitá-lo
- Reporte comportamento inaceitável em privado pelo canal descrito nesse arquivo


## Setup de desenvolvimento
### Requisitos de toolchain
- O MSRV (Minimum Supported Rust Version, versão mínima suportada do Rust) é Rust 1.89, declarado como `rust-version` em `Cargo.toml`
- Instale Rust via `rustup` e prefira o toolchain pinado quando existir
- Mantenha `Cargo.lock` commitado porque este crate entrega uma CLI (interface de linha de comando) binária

### Política de MSRV
- Nunca suba o MSRV sem issue explícita de discussão
- Registre todo aumento de MSRV em `Alterado` nos dois idiomas do CHANGELOG
- Não use recurso de linguagem ou de biblioteca padrão mais novo que o MSRV
- O gate `cross-targets` mantém o código de Windows e macOS passando no type-check sob as mesmas regras de toolchain

### Pins de dependência
- A linha de produto 0.5.6 usa russh 0.65.1 (russh desde 0.3.8) sem os pins COMPAT RC antigos
- Não reintroduza pins RC mortos sem issue
- Nunca rode `cargo update` cego no grafo crypto
- Rode `scripts/verify_install_resolve.py` após qualquer mudança de dependência

### Inventário local de auditoria
- O registro local de auditoria (gitignored, não publicado) registra os gaps fechados na 0.5.6
- Cada gap lista problema, consequências, causa raiz, solução, benefícios e prova
- Não declare FIXED fazendo grep da prosa desse arquivo (G13/G15)
- FIXED exige prova de efeito no destino, como checksums, nome de teste ou comando medido


## Estratégia de branches
- Mantenha `main` como branch de integração
- Use `feature/<short-kebab>` para features
- Use `fix/<short-kebab>` para correções
- Use `docs/<short-kebab>` para documentação
- Use `chore/<short-kebab>` para tooling e manutenção


## Convenção de commits
- Siga Conventional Commits 1.0.0 em branches compartilhadas
- Use `feat` para features visíveis
- Use `fix` para bug fixes
- Use `docs` para mudanças só de documentação
- Use `test` para mudanças só de testes
- Use `chore` para manutenção
- Nunca adicione linhas `Co-authored-by` para agentes de IA


## Processo de Pull Request
- Passo 1: Abra PR com problema claro e os comandos de validação que você rodou
- Passo 2: Rode `python3 scripts/check_all_gates.py` e peça review só depois que os catorze gates estiverem verdes
- Passo 3: Adicione teste de regressão para toda mudança de comportamento, pelo binário real quando o contrato for a CLI
- Passo 4: Inclua docs bilíngues quando documentos públicos mudarem
- Passo 5: Adicione entrada no CHANGELOG sob `[Unreleased]` nos dois idiomas, com o mesmo número de entradas `BREAKING` em cada um
- Passo 6: Preserve comportamento one-shot em todo comando de produto
- Passo 7: Não introduza packaging de daemon de longa duração, telemetria nem configuração por variável de ambiente
- Passo 8: Mantenha segredos, inventários reais de hosts e master-keys fora do diff


## Testes
- Leia [docs/TESTING.pt-BR.md](docs/TESTING.pt-BR.md) para categorias, perfis e o catálogo completo de suítes
- Prefira unit tests determinísticos para packing e migração de schema
- Use integration tests em `tests/` para contratos da CLI
- Inclua as suítes de regressão de gaps ao tocar superfície residual
- Nomeie cada suíte explicitamente em vez de usar faixa, porque lista elidida (`v038 … v051`) não satisfaz checagem por `contains` e derruba em silêncio as suítes do meio
- A conformidade de documentação vive em `tests/docs_conformance.rs`, separada das suítes comportamentais: editar prosa nunca deve deixar o gate de build vermelho sem regressão funcional (G-QA-R03)
- Gates locais obrigatórios antes do PR: `cargo fmt --check` e `cargo test --locked --all-features --no-fail-fast`, mais clippy como no processo de release
- E2E SSH real é opcional quando não há host lab
- Para E2E SSH real local (G-E2E-05) prefira `python3 scripts/e2e_real_ssh.py --local-sshd` (só Unix)
- Caso contrário use `--host`/`--port`/`--user`/`--password-file` contra um host de lab
- Um maintainer pode usar `--from-grok-config`, que lê só `$HOME/.grok/config.toml` (Python 3.11+)
- O script aceita só flags, sem variáveis de ambiente (no Windows rode com `py -3`)
- Sem host lab o script sai 0 com SKIP (offline-safe)
- Binário default: `target/release/ssh-cli`
- Matriz oficial E01–E21 (E10–E14 SCP, E15 tunnel porta 0, E16 symlink, E17/E18 checksum SFTP, E19/E20 modelagem de payload, E21 rotação do keyring num profile vivo)
- Prefira sshd local / lab
- Sem storm de auth em hosts de produção com fail2ban
- Nunca logue credenciais
- Nunca commite config do Grok ou inventários neste repo
- Testes que precisam de secrets em claro devem passar `--allow-plaintext-secrets` (flag CLI, não é store env de produto)
- Nunca deixe testes flaky dependentes de rede sem timeout

### Suítes de regressão
- `tests/gaps_v035_integration.rs`, `tests/gaps_v037_integration.rs`, `tests/gaps_v038_integration.rs` e `tests/gaps_v039_integration.rs`: as suítes de integração anteriores
- `tests/gaps_v040_integration.rs`: contratos comportamentais de SCP e tunnel
- `tests/gaps_v041_integration.rs` e `tests/gaps_v042_integration.rs`: os fechamentos da 0.4.1 e da 0.4.2
- `tests/gaps_v051_integration.rs`: export, import, wire e secrets
- `tests/gaps_v053_domain_types.rs`, `tests/gaps_v054_error_handling.rs`, `tests/gaps_v055_unsafe_ffi.rs` e `tests/gaps_v057_sftp.rs`: os gates de domínio, erros, unsafe e SFTP da 0.5.x
- `tests/gaps_v058_e2e_residual.rs`: G-E2E residual (ACME permanente, um único `vps-added`, root `schema`/`doctor`, clap sem env, `-v`/`-vv`/`-vvv` graduados, FIXED_MASK, `--use-agent`)
- `tests/gaps_v059_agent_native.rs`: superfície agent-native da 0.5.4 (recusa de `--no-input` em `vps add` e `vps edit`, modelagem de payload com `--select`/`--filter`/`--limit`/`--count-only`, guarda de `--bind` fora de loopback)
- `tests/gaps_v060_tunnel_modes.rs`: LOTE E, contratos de argumento de `--reverse`/`--socks5`/`--remote-socket` e o preview de `--dry-run`, ambos pelo binário real
- `tests/gaps_v061_error_taxonomy.rs` e `tests/gaps_v061_scp_durability.rs`: exit codes 69 e 70 com o contrato do envelope, e relato de durabilidade do SCP
- `tests/gaps_v062_component_budget.rs` e `tests/gaps_v062_i18n_reachability.rs`: orçamentos de tamanho de arquivo e de lint e alcance das mensagens
- `tests/gaps_v063_e2e_harness.rs`, `tests/gaps_v063_locale_features.rs` e `tests/gaps_v063_secret_stdin.rs`: a matriz E01–E21 executável, features Cargo de locale e a superfície de entrada de segredos
- `tests/gaps_v064_gate_runner.rs`: contrato de cobertura da bateria, onde todo `scripts/check_*.py` é gate ou exclusão declarada
- `tests/gaps_v065_exec_target_designation.rs` e `tests/gaps_v066_transfer_target_designation.rs`: designação explícita de alvo para `exec` e para `scp` e `sftp`
- `tests/gaps_v067_packaging_reach.rs`, `tests/gaps_v068_source_language.rs` e `tests/gaps_v069_markdown_tables.rs`: alcance do `cargo package`, árvore de fonte em inglês e contagem de colunas de tabelas Markdown
- `tests/gaps_v070_doc_reference_reach.rs` e `tests/gaps_v071_vps_edit_rename.rs`: caminhos do repositório citados por documentos, e `vps rename`, `vps edit --name`, as flags de limpeza e o lock limitado
- `tests/gaps_v072_*.rs`: cinco suítes de veracidade do edit, roundtrip de import, contenção de lock, conformidade de schema e endurecimento de storage
- `tests/gaps_v073_*.rs`: as quatro suítes da 0.5.6 de troca de autenticação, charset de nome de host, escape de terminal e modo do diretório de config
- `tests/gaps_v074_flag_reach.rs`: a suíte da 0.5.6 de alcance de flag, onde toda flag publicada é documentada onde o inventário é prometido e exercitada por um teste de comportamento
- `tests/gaps_v075_markdown_format_gate.rs`: contrato de cobertura do gate de formato Markdown `scripts/check_markdown_format.py`, para que o critério de formatação dos documentos possa falhar
- Suítes de integração e de qualidade: `tests/e2e_cli.rs` roda a CLI de ponta a ponta num `--config-dir` isolado, `tests/i18n_integration.rs` confere a paridade de locale e de mensagens, `tests/proptest_tests.rs` reúne testes property-based, `tests/scp_integration.rs` e `tests/tunnel_integration.rs` conferem o help e os argumentos obrigatórios de `scp` e `tunnel`, `tests/snapshot_tests.rs` fotografa a saída estável com `insta`, `tests/storage_integration.rs` confere o CRUD do registro de VPS e `tests/test_quality.rs` barra testes que passam por construção


## Documentação
- Aplique o framework bilíngue em toda doc pública
- Espelhe inglês e `.pt-BR` na mesma entrega
- Abra todo documento público com link cruzado de idioma
- Mantenha tom persuasivo fora de SKILL.md e schemas
- Indexe todo schema JSON em `docs/schemas/README.md`

### Política de tradução
- O inglês é o texto canônico e o arquivo `.pt-BR` é o espelho
- Os dois arquivos têm as mesmas seções e os mesmos bullets na mesma ordem
- Comandos, flags, nomes de campo, exit codes e identificadores de código ficam intocados na tradução
- O texto em português do Brasil mantém todos os acentos e sinais diacríticos
- Uma mudança em um idioma não está pronta até o outro idioma recebê-la


## Reportar bugs
- Abra issue no GitHub com reprodução e esperado versus atual
- Inclua OS, arquitetura, `ssh-cli --version` e exit code
- Omita ou mascare segredos em logs e no histórico de comandos
- Reporte vulnerabilidades de segurança em privado, seguindo [SECURITY.pt-BR.md](SECURITY.pt-BR.md), e nunca em issue pública


## Solicitar features
- Abra issue descrevendo o workflow do agente e o gap de paridade de automação SSH se houver
- Prefira features que preservem one-shot e storage XDG multi-host


## Processo de release
- Mova as entradas de `[Unreleased]` para uma seção nova `[X.Y.Z] - YYYY-MM-DD` nos dois idiomas do CHANGELOG
- Mantenha a ordem de seções Segurança, Adicionado, Alterado, Depreciado, Removido, Corrigido, Interno
- Marque cada breaking change com o prefixo `BREAKING` e uma linha `Migração:`, com a mesma contagem nos dois idiomas
- Atualize os links de rodapé: `[Unreleased]` compara a tag nova com `HEAD`, e cada versão tem seu próprio link de comparação
- Suba SemVer em `Cargo.toml`
- Rode `python3 scripts/check_all_gates.py` e `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`
- Siga a lista de itens de `docs/RELEASE_CHECKLIST.pt-BR.md`
- Confirme que os docs bilíngues da raiz (README, SECURITY, INTEGRATIONS, llms*) descrevem a superfície de release registrada no CHANGELOG
- Os gates rodam localmente, e a matriz E2E é opcional
- Empacote com `cargo package --locked` e dry-run de publish quando necessário
- Tag `vX.Y.Z` só após gates verdes e autorização explícita do maintainer
- Prefira `cargo install ssh-cli --locked` na doc pública de install
- Nunca publique segredos, inventários reais de hosts ou master-keys


## Reconhecimento
- Contribuidores são creditados nas notas de release quando desejarem crédito público
- Pesquisadores de segurança seguem [SECURITY.pt-BR.md](SECURITY.pt-BR.md) para crédito privado


## Perguntas
- Abra discussion ou issue para dúvidas de processo
- Contate o maintainer em daniloaguiarbr@proton.me para coordenação privada
