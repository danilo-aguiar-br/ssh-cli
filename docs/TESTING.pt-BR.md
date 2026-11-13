# Guia de testes


> Rode o perfil certo de testes do ssh-cli sem travar em redes remotas.

- Leia este documento em [Inglês (en)](TESTING.md)
- Linha de produto: 0.5.6 (suítes históricas residuais incluem 0.4.1 AUD-POST / `gaps_v041` e wire 0.5.2 / `gaps_v051`)


## Por que testes categorizados
- Testes unitários protegem o empacotamento, o schema, a cifragem autenticada (AEAD) dos segredos e a lógica pura sem servidores SSH
- Testes de integração protegem os contratos da interface de linha de comando (CLI), o armazenamento e os snapshots
- Fixtures opcionais `ssh-keygen` (G-PROC-02) geram chaves OpenSSH reais para os testes de caminho de chave
- Binário ausente pula esses casos: o produto em execução nunca o dispara
- Testes remotos ao vivo são opcionais e devem sempre usar timeouts rígidos e nunca logar credenciais
- Gates de install resolve protegem o onboarding no crates.io (GAP-014)
- Suítes residuais de gaps travam a entrada e saída do agente, os exit codes, a cadeia de suprimentos, o mascaramento, o formato de rede do Secure Copy (SCP) e do SSH File Transfer Protocol (SFTP) e a honestidade dos documentos
- O registro de auditoria é um arquivo local do mantenedor, ignorado pelo git, excluído do cargo e nunca publicado, e os testes não devem afirmar seu texto FIXED (G13/G15)
- G6: testes que tocam estado global de signal/cancel (`CANCEL_FLAG`) usam `#[serial_test::serial]` (dev-dep `serial_test`) para a suíte ser determinística
- Não remova marcadores serial de testes de concorrência/sinal
- G11: a linha de base deve ficar verde na primeira execução
- Re-rodar até passar é proibido como estratégia de gate


## Categorias de teste
- Testes unitários dentro dos módulos de `src` (inclui cifragem padrão de `secrets`)
- Os testes unitários de `ssh-cli vps edit` em `src/vps/edit/apply_tests.rs` usam `tempfile::tempdir()`, nunca um caminho fixo na pasta temporária do sistema, então execuções em paralelo não colidem
- CLI e2e em `tests/e2e_cli.rs`
- Integração residual de gaps em `tests/gaps_v035_integration.rs` (só segredos falsos)
- Suíte residual de I/O de agente em `tests/gaps_v037_integration.rs`
- Suíte residual pós-0.3.7 em `tests/gaps_v038_integration.rs`
- Suíte residual pós-0.3.8 em `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC/DENY/CHG)
- Suíte residual pós-0.3.9 / 0.4.0 em `tests/gaps_v040_integration.rs`
- Suíte AUD-POST em `tests/gaps_v041_integration.rs` (EXP-001, TUN-002, CLI-005/006, IO-009, REL-006, DOC-041)
- Suíte AUD-E2E em `tests/gaps_v042_integration.rs` (TUN-003, IO-010, UX-001, REL-007, ENV-001, DOC-042, SCP-024)
- Suíte 0.5.2 em `tests/gaps_v051_integration.rs` (roundtrip de redaction do export, JSON `vps-export`, dual-read schema v3, evento secrets-init, guarda include-secrets, CRUD `vps-added`, empty command, import exit 65)
- Suíte de tipos de domínio em `tests/gaps_v053_domain_types.rs`
- Suíte de tratamento de erros em `tests/gaps_v054_error_handling.rs`
- Suíte residual unsafe/FFI em `tests/gaps_v055_unsafe_ffi.rs`
- Suíte residual G-SFTP em `tests/gaps_v057_sftp.rs` (superfície SFTP, com prova de efeito no destino preferida à auto-certificação de inventário)
- Suíte residual G-E2E em `tests/gaps_v058_e2e_residual.rs` (root `ssh-cli schema` / `ssh-cli doctor`, um único `vps-added` com `secrets_key_auto_created`, `--use-agent`, purge de env help/clap, `RUST_LOG` ambiente ignorado, ACME exit 64, etc.)
- Integração do armazenamento em `tests/storage_integration.rs`
- Snapshot tests em `tests/snapshot_tests.rs`
- Superfície SCP em `tests/scp_integration.rs`
- Superfície tunnel em `tests/tunnel_integration.rs`
- Superfície dos modos de tunnel em `tests/gaps_v060_tunnel_modes.rs`: os modos `--reverse`, `--socks5` e `--remote-socket` da 0.5.4, a exclusão mútua entre eles, o rótulo `mode` no formato de rede (`local` / `reverse` / `socks5` / `streamlocal`), o guard `--i-accept-network-exposure` nas duas pontas e o evento `tunnel_closed`
- Superfície de saída agent-native em `tests/gaps_v059_agent_native.rs`: `--select`, `--filter`, `--limit`, `--count-only` e o relatório de truncagem
- Taxonomia de erro em `tests/gaps_v061_error_taxonomy.rs`: `error_class`, `retryable` e `suggestion` em todo envelope
- Durabilidade de SCP em `tests/gaps_v061_scp_durability.rs`: `mtime_preserved` e `durable` são reportados, nunca presumidos
- Orçamento de componente em `tests/gaps_v062_component_budget.rs`: a catraca falha tanto ao crescer além de um teto congelado quanto em entrada obsoleta cujo arquivo já encolheu, então o livro-razão registra quitação com o mesmo volume da dívida
- Alcançabilidade de i18n em `tests/gaps_v062_i18n_reachability.rs`: toda chave de mensagem é alcançável nos dois idiomas
- Falsificação de gate para `scripts/check_en_identifiers.py`, conduzida por `scripts/check_all_gates.py`, cada check precisa ser comprovadamente capaz de falhar, e dois deles nasceram mortos
- A prova vigente vive em `tests/gaps_v068_source_language.rs`
- Arcabouço de ponta a ponta (E2E), features de locale e stdin de segredo em `tests/gaps_v063_e2e_harness.rs`, `tests/gaps_v063_locale_features.rs`, `tests/gaps_v063_secret_stdin.rs`
- Designação explícita de alvo em `tests/gaps_v065_exec_target_designation.rs`: o argv do GAP-SSH-EXEC-ARGC-001 segue rejeitado, e o trio de proveniência alcança envelopes de sucesso e de erro igualmente
- Designação de alvo em transferências em `tests/gaps_v066_transfer_target_designation.rs`: a mesma regra em `scp` e `sftp`
- Alcance do empacotamento em `tests/gaps_v067_packaging_reach.rs`: todo arquivo-fonte sobrevive ao `cargo package`
- Um padrão de `.gitignore` que engole diretório de código-fonte é invisível a todo build local
- Idioma do código-fonte em `tests/gaps_v068_source_language.rs`: código e comentário em inglês, isentando citações em vez de arquivos inteiros
- Renderização de tabela Markdown em `tests/gaps_v069_markdown_tables.rs`: linha cujos pipes não escapados discordam da linha delimitadora é Markdown válido que renderiza com colunas fantasma
- Alcance de referência da documentação em `tests/gaps_v070_doc_reference_reach.rs`: todo caminho do repositório que um documento nomeia existe no disco, e toda suíte do disco está catalogada aqui
- A suíte fantasma que isso fechou era invisível à varredura direta
- Edição e rename em `tests/gaps_v071_vps_edit_rename.rs`: o segredo continua legível sob o nome novo e o arquivo só guarda texto cifrado, o marcador do `ssh-cli connect` segue só quando apontava a origem, nenhum dos dois verbos troca de alvo a partir do marcador, as oito flags de limpeza esvaziam exatamente o próprio campo e falham fechado quando deixariam zero auth, `--dry-run` não grava nada nos dois verbos, e o lock do config sai 69 em vez de pendurar
- Veracidade da edição em `tests/gaps_v072_edit_truthfulness.rs`, `ssh-cli vps edit` informa só o que mudou: uma flag cujo valor já confere deixa `updated` vazio (e a chave continua no envelope), um no-op sai 0 e deixa o `config.toml` idêntico byte a byte, e o rename falha numa ordem fixa (host desconhecido 66, nome idêntico 64, nome já em uso 64)
- Round trip de import em `tests/gaps_v072_import_roundtrip.rs`: um host exportado com seus segredos e importado num registro vazio compara igual campo a campo por JSON e por TOML, os campos de Transport Layer Security (TLS) são validados no import, o campo `replaced` é reportado, stdout fechado tem exit code próprio, e uma contagem de campos fixada falha assim que o `VpsRecord` cresce sem que export e import aprendam o campo novo
- Contenção de lock em `tests/gaps_v072_lock_contention.rs`: o exit 69 com o lock do registro ocupado traz o remédio do lock e não o do keyring, provado com um segundo processo real segurando o `config.toml.lock`
- Conformidade de schema em `tests/gaps_v072_schema_conformance.rs`: envelopes reais impressos pelo binário validam contra os schemas publicados em `docs/schemas/`, por um validador pequeno escrito à mão para não trazer dependência nova à cadeia de suprimentos
- Endurecimento do armazenamento em `tests/gaps_v072_storage_hardening.rs`: os arquivos de lock e de marcador mantêm os modos esperados em disco após uma invocação real, assertados só em Unix porque o Windows não tem bits de modo POSIX
- Troca de método de auth em `tests/gaps_v073_vps_auth_switch.rs`, `ssh-cli vps edit` troca um host de um método de autenticação para outro numa chamada só: `--key` limpa a senha e o agente, `--password` e `--password-stdin` limpam a chave, a passphrase dela e o agente, e `--use-agent` limpa a senha e a chave
- A mesma suíte prova que duas flags de auth primária na mesma chamada saem 2 com stdout vazio, passphrase sem chave sai 64 e deixa o arquivo intacto, uma chave diferente descarta a passphrase antiga, `--agent-socket` sozinho troca para o agente, e os planos de `--dry-run` ecoam `target_resolved` e `target_source`
- Charset do nome de host em `tests/gaps_v073_host_name_charset.rs`: um nome de host novo é limitado a letras ASCII, dígitos, ponto, hífen e sublinhado em `ssh-cli vps add`, `ssh-cli vps rename`, `ssh-cli vps edit --name` e `ssh-cli vps import`
- A mesma suíte prova que `ssh-cli vps import` lista todos os nomes ruins de uma vez e não grava nada, nomes antigos com espaço ou acento ainda carregam, e `ssh-cli vps rename` repara um nome antigo com caracteres de controle
- Buscas que viram alvo (`ssh-cli exec`, `ssh-cli health-check`, `--hosts`, `ssh-cli connect`) recusam caracteres de controle com exit 64, e `ssh-cli tls mtls import --name` recusa caracteres de controle, formatação e bidi sem criar diretório
- Escape de terminal em `tests/gaps_v073_terminal_escape.rs`: caracteres de controle e bidi são escapados como texto no estilo `\u{1b}` no modo texto (`ssh-cli vps list`, `ssh-cli vps show`, erros de não encontrado e buscas) antes de chegar ao terminal, enquanto o JSON mantém o valor original
- Modo do diretório de config em `tests/gaps_v073_config_dir_mode.rs`: o diretório de config é criado com modo `0700`, um diretório XDG padrão existente é apertado, e qualquer outro diretório existente nunca recebe chmod enquanto o `ssh-cli doctor` reporta `config_dir_exposed`
- A mesma suíte prova que um caminho terminado em `.toml` faz da pasta pai o diretório de config, `ssh-cli vps export` cria pastas que faltam privadas, duas tabelas que normalizam para o mesmo nome saem 65, e uma varredura do código falha quando outro módulo cria o diretório
- Alcance das flags em `tests/gaps_v074_flag_reach.rs`: cada flag documentada é provada por comportamento observável e não pela aceitação do parser, e cada documento que afirma o inventário completo falha quando perde uma flag
- Gate de formato Markdown em `tests/gaps_v075_markdown_format_gate.rs`: o `scripts/check_markdown_format.py` continua na bateria, roda contra a árvore real e importa só a biblioteca padrão
- Conformidade de documentação em `tests/docs_conformance.rs`: inventário de 48 folhas de comando, indexação de schemas nos dois idiomas, paridade bilíngue de release, as oito operações de `--dry-run`, máscaras de permissão direcionais e o orçamento de 7000 palavras das skills
- Qualidade de teste em `tests/test_quality.rs`
- Testes de propriedade em `tests/proptest_tests.rs`
- Integração de i18n em `tests/i18n_integration.rs`
- Executor da bateria de gates `scripts/check_all_gates.py`: roda os catorze gates obrigatórios numa invocação
- As suítes acima rodam sob `cargo test`, que a bateria invoca como seu gate `test`
- Gate de frescor de advisories `scripts/check_advisory_freshness.py`: alcançável somente pelo executor da bateria
- Gate de hits de advisory `scripts/check_advisory_hits.py`: roda `cargo audit --json --no-fetch` e falha em qualquer advisory contra o `Cargo.lock`, porque a base fresca e o `cargo deny` verde nunca diziam quais advisories atingiam a árvore
- Contrato de cobertura da bateria `tests/gaps_v064_gate_runner.rs`: todo `scripts/*.py` público é um gate ou um não-gate declarado, e `the_test_gate_stays_locked` falha quando o gate `test` perde `--locked` ou `--no-fail-fast`
- Script de install resolve `scripts/verify_install_resolve.py`
- Gate de identificadores em inglês `scripts/check_en_identifiers.py`
- E2E SSH real (opcional, local da máquina): `scripts/e2e_real_ssh.py`: matriz oficial E01–E21 (E10–E14 cobrem SCP upload/download/cmp/missing/preserve, E17/E18 cobrem SFTP checksum + árvore recursiva, G7, e E19–E21 cobrem o item reduzido de `vps list --select`, a contabilização de `--limit` e o round-trip do keyring)
- Medições de desempenho em `benches/` (manual)


## Como rodar
### Bateria completa (obrigatória antes de declarar gate verde)
- Uma invocação roda todo gate obrigatório e reporta todo resultado

```bash
python3 scripts/check_all_gates.py
```

- Acrescente `--json` para NDJSON, `--only ID[,ID...]` para subconjunto e `--list` para ver os ids
- Os catorze gates são `fmt`, `build-release`, `build-no-default`, `clippy`, `doc`, `test`, `exec-target`, `deny`, `cross-targets`, `advisory-freshness`, `advisory-hits`, `en-identifiers`, `markdown-format` e `install-resolve`
- O gate `test` roda `cargo test --locked --all-features --no-fail-fast`
- `--no-fail-fast` faz uma rodada vermelha mostrar todos os testes que falham em vez de parar no primeiro
- Outras flags são `--fail-fast` para parar no primeiro gate vermelho, `--log-dir DIR` para guardar os logs de cada gate e `--timeout-secs N` para o orçamento por gate (padrão 3600, `0` para sem limite)
- `--quiet` esconde as linhas de progresso no stderr, e o final de um gate que falhou ainda é impresso
- O stdout traz uma linha por gate separada por tabs, com o id, `pass` ou `fail`, o exit code e os milissegundos, depois uma linha `_summary` com `total=`, `failed=` e `skipped=`
- O exit é 0 quando todo gate escolhido passa, 1 quando algum falha e 2 em erro de uso, como um id de gate desconhecido
- O gate `exec-target` roda só `tests/gaps_v065_exec_target_designation.rs`, então uma regressão do argv perigoso de exec aparece com nome no relatório
- `cargo clippy` e `cargo test` abortam no primeiro alvo que não compila, então um único arquivo de teste quebrado esconde todo gate atrás dele
- A execução sempre reporta quantos gates pulou, então rodada parcial não pode ler como completa
- A bateria é sequencial de propósito: os gates de cargo disputam um lock de `target/`, então rodá-los em paralelo não ganha nada

### Loop local do desenvolvedor

```bash
cargo fmt --check
cargo test --locked --all-features --no-fail-fast
cargo clippy --all-targets --all-features --locked -- -D warnings
python3 scripts/check_en_identifiers.py
cargo build --release
python3 scripts/verify_install_resolve.py
```

### Perfis focados
- Toda suíte de `tests/` roda sozinha com `cargo test --locked --test <suite>`
- O bloco lista todas elas

```bash
cargo test --locked --test docs_conformance
cargo test --locked --test e2e_cli
cargo test --locked --test gaps_v035_integration
cargo test --locked --test gaps_v037_integration
cargo test --locked --test gaps_v038_integration
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo test --locked --test gaps_v042_integration
cargo test --locked --test gaps_v051_integration
cargo test --locked --test gaps_v053_domain_types
cargo test --locked --test gaps_v054_error_handling
cargo test --locked --test gaps_v055_unsafe_ffi
cargo test --locked --test gaps_v057_sftp
cargo test --locked --test gaps_v058_e2e_residual
cargo test --locked --test gaps_v059_agent_native
cargo test --locked --test gaps_v060_tunnel_modes
cargo test --locked --test gaps_v061_error_taxonomy
cargo test --locked --test gaps_v061_scp_durability
cargo test --locked --test gaps_v062_component_budget
cargo test --locked --test gaps_v062_i18n_reachability
cargo test --locked --test gaps_v063_e2e_harness
cargo test --locked --test gaps_v063_locale_features
cargo test --locked --test gaps_v063_secret_stdin
cargo test --locked --test gaps_v064_gate_runner
cargo test --locked --test gaps_v065_exec_target_designation
cargo test --locked --test gaps_v066_transfer_target_designation
cargo test --locked --test gaps_v067_packaging_reach
cargo test --locked --test gaps_v068_source_language
cargo test --locked --test gaps_v069_markdown_tables
cargo test --locked --test gaps_v070_doc_reference_reach
cargo test --locked --test gaps_v071_vps_edit_rename
cargo test --locked --test gaps_v072_edit_truthfulness
cargo test --locked --test gaps_v072_import_roundtrip
cargo test --locked --test gaps_v072_lock_contention
cargo test --locked --test gaps_v072_schema_conformance
cargo test --locked --test gaps_v072_storage_hardening
cargo test --locked --test gaps_v073_config_dir_mode
cargo test --locked --test gaps_v073_host_name_charset
cargo test --locked --test gaps_v073_terminal_escape
cargo test --locked --test gaps_v073_vps_auth_switch
cargo test --locked --test gaps_v074_flag_reach
cargo test --locked --test gaps_v075_markdown_format_gate
cargo test --locked --test i18n_integration
cargo test --locked --test proptest_tests
cargo test --locked --test scp_integration
cargo test --locked --test snapshot_tests
cargo test --locked --test storage_integration
cargo test --locked --test test_quality
cargo test --locked --test tunnel_integration
cargo test --locked --lib
cargo test --locked packing
cargo test --locked secrets::
cargo fmt --check
```

### Perfil de documentação
- Rode estas suítes depois de qualquer mudança em documento, schema ou skill
- Elas cobrem inventário de comandos, indexação de schemas, tabelas Markdown, caminhos do repositório, catálogo de suítes, alcance do empacotamento, alcance das flags, formato Markdown e o executor dos gates

```bash
cargo test --locked --test docs_conformance --test gaps_v070_doc_reference_reach --test gaps_v072_schema_conformance --test gaps_v039_integration --test gaps_v064_gate_runner --test gaps_v067_packaging_reach --test gaps_v069_markdown_tables --test gaps_v068_source_language --test gaps_v074_flag_reach --test gaps_v075_markdown_format_gate
```

### E2E SSH real (nunca imprimir segredos) — G-E2E-05

```bash
# Preferred: unprivileged sshd on loopback, nothing to configure (Unix only)
python3 scripts/e2e_real_ssh.py --local-sshd

# Lab host: flags only, no environment variables; the password comes from a file or stdin
python3 scripts/e2e_real_ssh.py --host … --port 22 --user … --password-file /path/to/pw-file --sudo-password-file /path/to/sudo-pw-file

# Maintainer-local only: parse $HOME/.grok/config.toml
# That file must stay under $HOME: never copy it into this repository. Needs Python 3.11+.
python3 scripts/e2e_real_ssh.py --from-grok-config
```

- Binário padrão: `target/release/ssh-cli` (troque com `--bin`)
- No Windows use `py -3 scripts/e2e_real_ssh.py ...`
- `--local-sshd` é só Unix
- `--from-grok-config [PATH]` é opcional e só do mantenedor, então os outros contribuidores usam `--local-sshd` ou as flags de lab
- Sem host de laboratório ou credenciais, o script sai 0 com SKIP, que é seguro offline e não é gate vermelho
- Matriz oficial E01–E21
- E10–E14 = SCP upload, download, integridade (`cmp`), remoto ausente, preserve mode+mtime (SCP-023)
- E17 = SFTP upload/download checksum
- E18 = SFTP árvore recursiva (G7)
- E19 = `vps list --select` devolve item reduzido que satisfaz `vps-list.schema.json` (D6)
- E20 = `vps list --select --limit` descarta registros, informa a contagem no stderr e `--quiet` silencia (D7)
- E21 = `secrets init --keyring` faz round-trip pelo store do SO em dois processos, e dá SOFT quando nenhum store responde (Gap 50)
- O script imprime só rótulos PASS/FAIL/SKIP: nunca host, usuário ou senha
- Suíte residual: `cargo test --locked --test gaps_v058_e2e_residual`
- Política GAP-014 / fail2ban: prefira `sshd` local ou VPS throwaway
- PROIBIDO: tempestades de falha de autenticação em hosts de produção (ban do fail2ban)
- E2E em VPS de produção só com cuidado, whitelist de IP / `ignoreip`, e sem senhas erradas intencionais


## Gates de publicação
- Mantenedores rodam o ciclo local do desenvolvedor antes de cada publicação
- Gates de publicação incluem o package dry-run, verificação de install resolve, paridade bilíngue de docs e a checagem de identificadores em inglês (`python3 scripts/check_en_identifiers.py`)
- Eles também incluem as suítes residuais `gaps_v040`, `gaps_v041`, `gaps_v042`, `gaps_v051`, `gaps_v057` e `gaps_v058`, mais `cargo fmt --check` (G10)
- O ciclo canônico é `cargo test --locked --all-features --no-fail-fast`, clippy `--all-targets --all-features -- -D warnings` e `cargo build --release`
- `python3 scripts/check_all_gates.py` roda todos eles, mais o type-check cross-target, numa invocação


## Variáveis de ambiente
- Use `--config-dir` nas invocações CLI para isolar config em testes (o produto não lê `SSH_CLI_HOME`)
- `--allow-plaintext-secrets` faz opt-out da cifragem padrão em testes que assertam TOML plaintext
- Sem esse opt-out, a primeira gravação de segredo auto-cria `secrets.key` e cifra campos
- Nível de tracing padrão é error
- Não espere prosa INFO em stderr por omissão
- `RUST_LOG` ambiente é ignorado
- Use `-v`/`-vv`/`-vvv` ao diagnosticar falhas (G2/G14 com escopo na crate)
- `-v` → info, `-vv` → debug, `-vvv` → trace (filtro de log só via CLI)
- Use `--no-color` quando a saída sensível a snapshot precisar de texto estável, porque as variáveis de cor padrão só guiam a detecção do terminal
- O produto ignora `SSH_CLI_HOME`, `SSH_CLI_LANG`, `SSH_CLI_FORCE_TEXT` e `SSH_CLI_USE_KEYRING` como configuração, e rejeita `SSH_CLI_SECRETS_KEY` e `SSH_CLI_SECRETS_KEY_FILE` com fail-closed
- Todo comando que lê ou grava segredo do registro sai 64 quando uma das duas variáveis de segredo está definida, enquanto `ssh-cli vps path`, `ssh-cli locale show`, `ssh-cli schema` e `ssh-cli commands` saem 0
- Nunca coloque senhas de hosts reais em variáveis de ambiente que os testes imprimem


## Solução de problemas
- Deriva de snapshot: revise `tests/snapshots/` e atualize só mudanças intencionais de UI (incluindo strings de versão)
- Falhas na resolução de criptografia: confira de novo as versões fixadas e rode de novo o script de instalação sem ignorar a política do lock
- Testes de timeout instáveis: garanta que nenhum host remoto real seja necessário salvo configuração explícita
- Falhas de permissão: confirme que diretórios temporários são graváveis e que asserções de modo batem com o SO
- Surpresas de fixture cifrada: passe `--allow-plaintext-secrets` ou forneça uma chave primária de teste via `--secrets-key-file` / XDG `secrets.key`
- Stderr quiet inesperado: o padrão é tracing error
- Passe `-v`/`-vv`/`-vvv` se precisar de mais linhas (`RUST_LOG` ambiente é ignorado)
- Falhas intermitentes de cancel/signal: confirme que `#[serial_test::serial]` permanece nos testes que tocam estado global de cancel (G6)
- Não paralelize esses casos
- Linha de base vermelha sem mudança de código: corrija a suíte, loteria de re-execução não é gate (G11)
- Gates de publicação incluem `cargo fmt --check` (G10)
- Falhas residuais de SCP / AUD-POST / 0.5.2 / SFTP / G-E2E: rode `cargo test --locked --test gaps_v040_integration`, `gaps_v041_integration`, `gaps_v042_integration`, `gaps_v051_integration`, `gaps_v057_sftp` e `gaps_v058_e2e_residual`
- O registro de auditoria local pode ter notas do mantenedor, mas não é artefato de gate publicado e os testes não devem afirmar seu texto (G13/G15)
- Um arquivo `*.snap.new` pendente derruba `no_pending_snapshot_files_exist`: aceite o snapshot ou apague o arquivo
- Suíte de documentação vermelha depois de editar um documento: rode o perfil de documentação acima e leia qual caminho, schema ou linha de tabela ela nomeia
- Nunca deixe `__pycache__` ou arquivos `.pyc` na árvore: rode Python com `python3 -B`


## Notas de release
### 0.5.6 — gate de formatação
- `scripts/check_markdown_format.py` transforma o contrato de formatação em algo executável, fechando o que os Gaps 25 e 37 nomeavam como critério de aceitação enquanto esse linter não existia
- A bateria roda catorze gates, e `markdown-format` e `doc` são os dois mais novos
- Treze regras cobrem negrito, itálico, separador horizontal, bullet de asterisco, ponto ou vírgula final em bullet, H4 e mais fundo, contagem de H1, espaçamento de H1 e H2, folga entre header e bullet, travessão fora de H3, blockquote decorativo e emoji
- `--selftest` traz um fixture por regra, então um regex que não casa nada não consegue reportar árvore limpa
- `docs_rules/` e `docs_prd/` ficam fora do escopo como material de origem importado, declarados em vez de pulados em silêncio
- `tests/gaps_v075_markdown_format_gate.rs` exige que o gate fique na bateria, execute na árvore real sobre pelo menos trinta documentos e não importe nada fora da biblioteca padrão

### 0.5.6 — keyring e contrato
- O `secrets init --keyring` grava no cofre do sistema operacional por `keyring` 4.2.0 com a feature `v1`, e `Entry::store_status()` faz um cofre indisponível sair 69 em vez de responder sucesso
- A entrada do cofre tem escopo por digest do diretório de config resolvido, então perfis com `--config-dir` não dividem mais uma chave única da máquina
- O `--use-keyring` também chega ao caminho de init, e as duas flags não podem mais discordar sobre o cofre
- A versão mínima de Rust é 1.89, exigida pelo russh 0.65
- `tests/gaps_v074_flag_reach.rs` prova cada uma das oito flags por comportamento observável em vez de aceitação do parser, e falha por documento quando um perde uma flag do inventário publicado
- A matriz E2E é E01–E21: E19 e E20 cobrem o payload reduzido do `vps list` e E21 o round-trip do keyring
- O E21 rotaciona um profile vivo para o cofre com `--force`, então executa a transição que o operador de fato faz, e falha quando um `secrets.key` remanescente sombrearia a chave nova
- O E20 provisiona a própria segunda entrada no registro, porque uma segunda entrada é o que o caso precisa e o `vps list` nunca disca, e o SKIP SOFT ali era lacuna de cobertura nomeada, não execução aprovada
- `secrets_init_refuses_to_fork_the_primary_key` cobre o defeito que o E21 achou: um segundo `secrets init` num profile vivo deve sair 64 e deixar a chave intacta
- `docs/schemas/vps-list.schema.json` declara as duas formas de item, então um item de `--select` valida
- O `deny.toml` carrega três ignores de `unmaintained` com motivo escrito, e a suíte cobra que o conjunto de ignores seja exatamente esses três

### 0.5.6 — fechamento da auditoria
- `--select` e `--truncate-content` reduzem um objeto único, e uma flag de coleção num registro só é nomeada em `agent_shape.ignored` mesmo quando é a única passada, coberto por `src/agent_shape_tests.rs`, `tests/gaps_v059_agent_native.rs` e `tests/gaps_v072_schema_conformance.rs` nas duas formas publicadas
- Um array nu reduzido escreve `{"event":"agent-shape",...}` em stderr, coberto por `tests/gaps_v059_agent_native.rs`
- O `--quiet` suprime essa linha de stderr como qualquer outra
- `tests/gaps_v074_flag_reach.rs` afirma comportamento para as 18 flags que não tinham e confere os documentos que afirmam o inventário completo
- `tests/docs_conformance.rs` exige bloco `Examples:` nas 48 folhas e roda cada argv publicado com `--help` anexado
- `scripts/check_advisory_hits.py` reprova a bateria quando um advisory alcança o lockfile, e o gate `clippy` agora carrega `-W clippy::doc_markdown`
- A bateria roda catorze gates
- A migração para criptografia em Rust puro foi tentada e revertida, então o provider TLS continua `aws_lc_rs` e as suítes escritas para essa migração saíram com ela

### 0.5.6 — edição e rename
- `ssh-cli vps rename <NAME> <NEW_NAME>` e `ssh-cli vps edit <NAME> --name <NEW>` mudam o nome do host nativamente, com as credenciais intactas
- O registro sela cada segredo com contexto não vinculado, então o dado associado não carrega nome de host e o segredo continua legível sob o nome novo
- Cada gravação re-sela com nonce novo, o que é normal e nunca toca o plaintext
- Isso substitui o desvio export/edit/import, que descartava auth de agente em silêncio porque o export não carregava campo de agente
- Oito flags de limpeza removem o campo em vez de sobrescrevê-lo
- `--dry-run` cobre os dois verbos
- O lock do config espera `--lock-wait-ms` limitado e depois sai 69 em vez de pendurar
- `ssh-cli commands --json` publica todo argumento

### 0.5.5 — release de segurança de designação de alvo
- Corrige GAP-SSH-EXEC-ARGC-001: `ssh-cli exec`/`ssh-cli sudo-exec`/`ssh-cli su-exec` decidiam o host alvo pela contagem de posicionais
- Assim `ssh-cli exec <HOST> --step <CMD>` executava o nome do host como binário remoto no host que `ssh-cli connect` selecionara por último
- BREAKING: um único posicional agora é erro de uso
- Alcance o marcador ativo com a nova `--use-active <COMMAND>`
- O envelope de host único ganhou `target_resolved` e `target_source`, então vincule consumidores novos a esses
- `host_resolved` e `host_source` são os aliases de leitura da 0.5.5
- O envelope também ganhou `active_fallback`, e o `ssh-cli doctor` passa a reportar `local.active_vps`

### 0.5.4 — release de segurança e agent-native
- Corrige DoS remoto pré-auth no banner SSH (A1)
- Impede que bits setuid enviados pelo servidor caiam no arquivo baixado (A3)
- Fecha a janela de leitura pública em chaves privadas ACME/mTLS (A2)
- Adiciona flags de redução de payload aplicadas antes da serialização: `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- BREAKING: falha parcial multi-host agora sai com exit 1 (era 65)
- BREAKING: `--bind` fora do loopback exige `--i-accept-network-exposure`
- Novo evento `tunnel_closed`
