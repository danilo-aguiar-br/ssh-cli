# Changelog

- Leia este documento em [Inglês (en)](CHANGELOG.md).

Todas as mudanças notáveis deste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Segurança
- **A1** `russh` subiu de **0.62.2** para **0.62.5**. A CVE-2026-68930 (GHSA-m65r-rprj-r5rg, publicada em 2026-08-03) atinge `<= 0.62.4`: callbacks de canal são despachados para IDs de canal nunca abertos. O defeito é do lado **servidor** e esta CLI é cliente puro — não há uma única referência a `russh::server` em `src/` — então o impacto alcançável aqui era nulo. O pin subiu mesmo assim, porque `cargo deny check advisories` devolveu **zero achados** com o banco de advisories sincronizado no mesmo dia. Gate de supply chain verde não é prova de que nenhuma CVE publicada se aplica.
- **A3** A pilha de criptografia at-rest deixou de ser duplicada. `chacha20poly1305` 0.10 puxava `aead` 0.5 / `chacha20` 0.9 / `poly1305` 0.8 enquanto russh e aws-lc-rs resolviam a geração 0.6 / 0.10 / 0.9, ou seja, **duas implementações independentes de ChaCha20-Poly1305** compiladas num binário de segurança. Subir `chacha20poly1305` para 0.11, `getrandom` para 0.3, `toml` para 1 e `windows-sys` para 0.61 reduziu 41 versões duplicadas para 5, todas transitivas e justificadas uma a uma no `deny.toml`.
- `deny.toml` mudou `multiple-versions` de `warn` para **`deny`**. O aviso era relevante e ninguém lia; toda duplicata remanescente está enumerada com motivo, então uma duplicata *nova* — justamente a que dá para corrigir — reprova o gate.
- **A6** `tls mtls import` lia caminhos PEM fornecidos pelo operador com `fs::read` sem limite. As duas leituras passam a ser limitadas por `MAX_PEM_FILE_BYTES` (1 MiB) via o novo `paths::read_bytes_capped`.

### Mudado
- **BREAKING — G-ERR-R01** Novos exit codes **69** (`EX_UNAVAILABLE`) e **70** (`EX_SOFTWARE`), com as variantes `SshCliError::Unavailable` / `Software`. `SshCliError::Config` — exit 65, classificado permanente — virou o destino de todo `map_err` sem variante óbvia, 16 deles só em `src/secrets.rs`. Um keyring do SO bloqueado saía 65 com `retryable: false`, então a única falha do grupo que um retry simples resolve era exatamente a que o agente era instruído a abandonar. Falha de keyring agora sai **69** e é transiente; falha de CSPRNG sai **70**; I/O de `secrets.key` sai **74**. `Config` caiu de 16 para 3 ocorrências nesse módulo, todas erros de dado genuínos.
- **B2 — o texto de erro agora é localizado no modo humano; o envelope JSON segue em inglês por contrato.** 17 das 44 variantes de `i18n::Message` tinham tradução completa em inglês e português brasileiro e zero call site, então `--lang pt-BR` devolvia inglês byte a byte para toda falha. Seis eram ofuscadas pelos literais `#[error("…")]` do `thiserror`, que renderizam no ponto de emissão real; onze haviam sido substituídas por variantes por modo e nunca foram apagadas. Nada pega isso: `Message` é `pub`, então `dead_code` nunca dispara sobre variante que ninguém constrói, e o `match` exaustivo em `en()` e `pt()` obriga a existir TRADUÇÃO, nunca CHAMADOR. As variantes substituídas foram removidas, as de erro passaram a carregar o detalhe de origem, e `i18n::localized_error_text` as mapeia por `error_code` — o mesmo discriminador estável que o schema publica. O ramo `--json` mantém deliberadamente o `Display` inglês de `SshCliError`: o agente ramifica por `error_code`, e um `message` dependente de locale mudaria em silêncio o payload que ele parseia quando o host mudasse de idioma. Código sem tradução cai no inglês, então a localização é fail-open.
- **B3 / A7 — orçamento de componente.** `#[allow(clippy::too_many_arguments)]` aparecia 16 vezes em código de produção, concentrado exatamente nos arquivos que A7 apontava como grandes demais: o único lint que mede esse acoplamento estava calado justo onde importava, e é por isso que `-D warnings` ficava verde por cima. Removidas as 16, só 10 voltaram a disparar — seis eram atributos mortos cujas funções já haviam caído abaixo do limiar. `crate::vps::AuthOverrides`, `HealthCheckRequest`, `tunnel::ServeContext` e `output::TunnelClosedInput` substituem as listas posicionais; 16 supressões viraram 3, cada uma com comentário justificando por que a forma achatada é correta. `src/vps/exec_ops.rs` (748 linhas) foi partido em `exec_ops/{types,single,elevation,fleet}.rs` e `src/sftp/mod.rs` (630) em `sftp/{setup,dispatch,emit}.rs`, com todo caminho público preservado por reexport. `src/ssh/sftp_session.rs` segue inteiro por decisão declarada — é uma máquina de estados coesa do SFTP v3. Três suites novas (`gaps_v062_cross_platform`, `gaps_v062_i18n_reachability`, `gaps_v062_component_budget`) transformam tudo isso em medições que falham alto.
- **G-SCP-R01 / G-SCP-R02** O `scp-transfer` ganhou `mtime_preserved` e `durable`. Ambos são aditivos e têm default `true`, então eventos escritos por versões anteriores continuam válidos. O produto documentava preservação de mtime como garantia enquanto descartava a falha em dois níveis de aninhamento; o fsync do diretório pai após o rename atômico era invisível do mesmo jeito. As falhas passam a ser logadas e reportadas em vez de resolvidas por silêncio — transferências para filesystems que não representam timestamp (FAT32, exFAT, caminhos de interop do WSL) continuam bem-sucedidas, e agora dizem isso.

- **C3 — o gate de tamanho de componente foi calibrado para passar, não para uma meta.** A primeira versão de `gaps_v062_component_budget` usava teto único de 830 linhas, escolhido para caber `src/ssh/sftp_session.rs` (810), e o comentário do teto nomeava esse arquivo como "exceção deliberada". A constante concedia dez: nove outros arquivos estavam entre 601 e 810 e passavam sem jamais serem declarados, enquanto a mensagem de falha do próprio teste mandava não levantar o teto. É o modo de falha do B1 reaparecendo dentro da remediação do próprio B1 — um gate cujo limiar deriva da medição atual registra o presente e chama isso de conformidade. O orçamento agora é 600 duro mais a catraca `DECLARED_EXCEPTIONS`: todo arquivo acima é nomeado com teto congelado e justificativa obrigatória, uma entrada nunca pode crescer, e entrada cujo arquivo caiu ao orçamento reprova a suite, de modo que o ledger só encolhe. A catraca reprovou o próprio commit que a introduziu, ao pegar `src/i18n.rs` crescendo de 757 para 778 linhas. Em vez de editar o número, o arquivo foi partido — `en()` e `pt()` viraram `src/i18n/en.rs` e `src/i18n/pt.rs`, caindo de 778 para 577 linhas e saindo do ledger, com a exaustividade intacta porque cada `match` ainda precisa cobrir toda variante.
- **C3b — esse split desarmou três gates presos a caminho, agora presos a contrato.** `gaps_v062_i18n_reachability` passaria a contar as tabelas realocadas como call sites de produção, fazendo toda variante parecer alcançável — verde medindo nada. `gaps_v040_integration::gap_scp_020_i18n_mensagens` ficou vermelho procurando string traduzida no arquivo errado. `scripts/check_en_identifiers.sh` acusou falso positivo porque sua allowlist nomeava só `src/i18n.rs`. É a quarta ocorrência da classe, depois de `gaps_v057_sftp` e `gaps_v061_error_taxonomy`: asserção de fonte presa a caminho passa por vacuidade quando o código entra e falha por engano quando sai, porque testa layout e não contrato. A allowlist virou `src/i18n(\.rs|/)`, o teste de integração lê o subsistema concatenado, e a suite de alcançabilidade ganhou `TRANSLATION_TABLES` mais um teste que reprova se o caminho excluído deixar de existir ou se as tabelas voltarem para dentro. Ambos os gates foram reverificados por falsificação deliberada: variante-sonda sem uso deixou a alcançabilidade vermelha, e arquivo-sonda com literal em português fez o gate de identificadores sair com 1.

### Corrigido
- **C1 — o contrato de mensagem em inglês nunca chegou ao artefato legível por máquina.** O B2 estabeleceu que o `message` do envelope `--json` permanece em inglês independente de `--lang`, e documentou isso em `docs/AGENTS.pt-BR.md` e `docs/schemas/README.md` — mas não em `docs/schemas/error-envelope.schema.json`, cuja descrição de `message` seguia com o genérico "texto de erro legível por humano". O único artefato do qual um agente geraria cliente era justamente o que não declarava o contrato. A descrição agora declara o invariante, manda ramificar por `error_code` e delimita a localização ao modo texto humano.
- **C2 — metade do emissor de erro nunca foi localizada.** O B2 roteou todo `SshCliError` tipado pelo i18n e deixou intacto o último ramo de `resolve_exit_code`: uma cadeia `anyhow` que não faz downcast nem para `SshCliError` nem para `DomainError` imprimia inglês cru sob `--lang pt-BR`. A falha que o operador tem menos condições de interpretar era a única nunca traduzida, e parecia coberta porque o caminho tipado — o testado — estava correto. `localized_error_text` recebe `&SshCliError` por assinatura, então aquele ramo não tinha como chamá-la. `Message::ErrorUnexpected` e `i18n::localized_unexpected_text` fecham o buraco; o helper devolve `String` em vez de `Option` porque ali não existe renderização alternativa para falhar aberto. Só o rótulo é traduzido — a cadeia `anyhow` é preservada verbatim para não perder diagnóstico — e o envelope JSON mantém `error_code` `"unexpected"`.
- **B1 — o alvo Windows não compilava.** `cargo check --target x86_64-pc-windows-msvc --no-default-features` falhava com seis erros enquanto `fmt`, `clippy`, 818 testes, `deny` e `doc` estavam todos verdes, e `docs/CROSS_PLATFORM.pt-BR.md` declarava Windows como suportado. Cinco erros vinham do `#![forbid(unsafe_code)]` em `src/platform/mod.rs`: atributo interno em arquivo de módulo governa também os filhos, o filho `windows` é a única superfície FFI Win32 do produto, e `forbid` — diferente de `deny` — não admite `#[allow]` interno. O sexto era o `windows-sys` 0.61 redefinindo `HANDLE` de inteiro para `*mut c_void`, quebrando a guarda `handle == 0`. O módulo passou a `deny`, `windows.rs` carrega um `allow` com escopo de arquivo referenciando a allowlist auditada do G-UNSAFE, e a guarda virou `handle.is_null()`. A causa estrutural é que nenhum gate faz cross-compile: código sob `#[cfg(target_os = ...)]` de outro alvo é descartado antes do type-check, e fica invisível para todo gate rodado no host. `scripts/check_cross_targets.sh` agora faz type-check de `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` e `x86_64-apple-darwin`, e é obrigatório no `CONTRIBUTING.md` e na checklist de release.
- **B4** `SCP_IO_CHUNK = 32_768` era declarado duas vezes como `const` local de função em `src/ssh/client_real_scp.rs`, e `SCP_HEADER_MAX_BYTES` uma vez em `src/ssh/scp_wire.rs`, cada um com um marcador admitindo que pertencia a `crate::constants`. A justificativa escrita citava uma allowlist de edição de uma rodada já encerrada, então o motivo expirou e a duplicação ficou — e `src/constants.rs` já hospedava `SFTP_IO_CHUNK` com o valor idêntico. As duas constantes migraram, e um gate falha se o marcador reaparecer.
- **B5** O `gaps.md` declarava "OPEN residual: 0" no cabeçalho enquanto dezoito seções seguiam com título `**OPEN**` no corpo. As dezoito foram verificadas fechadas no código em duas auditorias; só os títulos nunca haviam sido atualizados. Rastreabilidade é a única coisa que aquele arquivo entrega, então a divergência é defeito de produto, não de forma.
- **A2** O `docs/schemas/error-envelope.schema.json` enumerava `error_class` como `transient|permanent|cancelled` enquanto a CLI emite `partial` desde a 0.5.4 (G-ERR-R02). Um validador estrito de agente rejeitava exatamente o envelope que o produto gera numa falha parcial de frota. O `error_code`, emitido em toda falha, também nunca foi contratado; os dois agora estão declarados.
- **A4** `sftp ls` e `sftp stat` escreviam em stdout com `println!`, furando a fachada `output` que o `src/lib.rs` documenta como obrigatória — então `--quiet` era ignorado e pipe para `head` abortava em EPIPE em vez de sair 141. As linhas de sucesso de `mkdir` / `rmdir` / `rm` / `rename` eram montadas com `format!` literal em inglês, então `--lang pt-BR` produzia inglês na saída SFTP mais lida. Ambas passam por `output` e `i18n`.
- **A6** O `cargo build --no-default-features` — a configuração de diagnóstico documentada no próprio `Cargo.toml` — falhava com **62 erros**. O subsistema SFTP nunca foi gateado por `ssh-real` apesar de chamar `russh_sftp` diretamente. O módulo, seus emissores e o braço de dispatch agora são gateados, e o subcomando responde com erro tipado em vez de falhar na linkagem.
- **G-QA-R02** A faixa de resolução do `run_tunnel` (busca no registro → overrides de credencial → `ConnectionConfig`) foi extraída para `resolve_tunnel_connection`, que recebe o registro já carregado e não toca disco. É exatamente a faixa onde o bug E3 de agent-auth viveu, e ela não tinha cobertura offline nenhuma.
- **D5 — dois schemas publicados eram inalcançáveis pela CLI.** `docs/schemas/` tinha 22 documentos enquanto o catálogo embutido `SCHEMAS` em `src/cli/schema_cmd.rs` listava 20, então `ssh-cli schema dry-run` e `ssh-cli schema tunnel-closed` saíam **64** com `unknown schema` — enquanto o `docs/schemas/README.md` manda o agente descobrir schemas rodando `ssh-cli schema` e documenta exatamente esses dois nomes. O contrato foi escrito onde era confortável escrever, não onde o consumidor lê. As duas entradas estão no catálogo e os dois comandos saem **0** com o documento. O novo `catalog_and_disk_agree` compara os dois conjuntos nas duas direções e valida que o nome é o leaf do arquivo menos o sufixo, então schema publicado em disco sem entrada no catálogo — ou entrada de catálogo sem arquivo — reprova a suite em vez de chegar ao agente como `unknown schema`.
- **D10 — o único desambiguador de um encerramento bem-sucedido de túnel era descartado.** O `src/tunnel.rs` emitia o evento de fechamento como `let _ = output::print_tunnel_closed_json(…)`. Os três finais que compartilham exit **0** — `deadline`, `signal` e `accept_error` — são distinguíveis *somente* por esse evento, então uma emissão falha entregava ao agente um exit de sucesso sem nenhuma forma de saber qual dos três havia ocorrido. A falha passa a ser reportada por `tracing::warn!`, o mesmo padrão que o R10 aplicou aos dois `Result` descartados em `copy_bidirectional`. O exit code é deliberadamente o mesmo — o encerramento é de fato sucesso — mas a falha de emissão fica visível no stderr, onde o contrato agent-native coloca diagnóstico.

### Interno
- `.atomwrite/` passa a ser barrado do crate publicado no `.gitignore`, no `exclude` do manifesto e no `.cargoignore`. O gate de empacotamento pre-publish mediu o `cargo package --list` embarcando `.atomwrite/scratch/old.txt` e `.atomwrite/scratch/new.txt`, staging deixado por uma rodada anterior de edição. Não estar rastreado pelo git não protegia nada: o Cargo empacota todo arquivo que não é rastreado nem ignorado. O `.gitignore` já cobria `.serena/`, `.claude/`, `.setting.cyber/` e `.cursor/`, e esquecia justamente o sidecar que a ferramenta de edição do próprio agente cria.
- Acrescentado `gap_sec_001b_todo_sidecar_pontuado_esta_barrado_do_pacote`, que descobre os diretórios pontuados na raiz em vez de enumerá-los. O irmão `gap_sec_001` fixa um sidecar por nome, e nomeá-los um a um é exatamente como `.atomwrite/` passou ao lado de quatro vizinhos listados. O que aparecer na raiz precisa agora ser publicado de propósito, que é o caso de `.git` e `.cargo`, ou barrado nas três superfícies, então um sidecar novo deixa a suíte vermelha no dia em que nasce. Provado negativamente: remover a linha do `.gitignore` reprova o gate nomeando esse arquivo exato.
- Corrigida uma afirmação falsa sobre `vps export` nas duas skills e nos dois conjuntos de evals. Elas diziam que o corpo do export fica TOML salvo com `--json`. Não fica: o corpo segue o formato de saída resolvido, e esse resolve para JSON sempre que o stdout não é TTY, o que é toda invocação de agente. Medido, `vps export -o /tmp/hosts.toml` grava um envelope JSON num arquivo chamado `.toml`; corpo TOML exige `--output-format text`. O `export_import_toml_roundtrip` dava aparência de cobertura afirmando apenas que `sshcli-enc:` está ausente e que o import aceita o arquivo — e como o import aceita TOML *e* envelopes JSON, o roundtrip fecha com o corpo errado. A mesma afirmação foi então corrigida nas superfícies longas e agora é sustentada por um gate.
- A afirmação falsa do TOML no `vps export` foi corrigida nas vinte superfícies longas que ainda a carregavam: `README` e `llms`/`llms-full` nos dois idiomas, `INTEGRATIONS`, `docs/AGENTS`, `docs/COOKBOOK`, `docs/MIGRATION`, `docs/HOW_TO_USE`, `docs/TESTING`, `docs/RELEASE_CHECKLIST` e `docs/schemas/README`. Várias afirmavam o inverso do comportamento medido, dizendo ao leitor que o JSON automático em non-TTY *não* se aplica ao export. Os exemplos de `docs/COOKBOOK` e `docs/HOW_TO_USE` passam a mostrar os dois caminhos, porque um comando copiável que grava JSON num arquivo `.toml` ensina a lição errada duas vezes.
- Novo `no_document_claims_export_defaults_to_toml` em `tests/docs_conformance.rs`, baseado em frases sobre as skills, os documentos de inventário completo e as quinze superfícies que descrevem o corpo do export. Ele pegou uma linha em `docs/HOW_TO_USE.md` que uma varredura manual por regex perdeu, já na primeira execução. O histórico do CHANGELOG fica isento de propósito: aquelas entradas registram o que uma versão passada afirmou, e reescrevê-las apagaria a evidência de que a afirmação foi feita.
- O `export_import_toml_roundtrip` foi renomeado para `export_body_follows_resolved_format_and_both_import` e agora afirma o que o nome antigo prometia. Ele fixa o envelope JSON do caminho non-TTY padrão, fixa um corpo TOML com `--output-format text`, fixa que o TOML diz `username` onde o envelope diz `user`, e importa os dois. A versão antiga afirmava apenas que `sshcli-enc:` está ausente e que o import aceita o arquivo, e como o import aceita TOML *e* envelopes JSON ele fechava com o corpo errado parecendo verde. O `tests/gaps_v051_integration.rs` tinha `export_pipe_defaults_to_json_when_non_tty` esse tempo todo: a suíte provava a verdade num teste e anunciava a mentira no nome do vizinho. Um teste cujo nome afirma o que ele nunca verifica é pior que um teste ausente, porque a lacuna fica invisível exatamente onde alguém iria procurar.
- As duas skills foram reconstruídas contra o binário instalado e cortadas para o teto de produto de 4000 palavras, vindo de 4906 e 5008. Nada saiu do contrato: os 47 comandos, todas as flags e todos os tokens de wire sobrevivem. As palavras saíram de prosa duplicada — bullets `PROIBIDO` que eram só o espelho negativo de um `OBRIGATÓRIO` da mesma seção, e uma seção `Proibições Absolutas` que repetia por inteiro as proibições que cada seção já havia dito.
- Novo `skills_stay_within_the_word_budget` em `tests/docs_conformance.rs`. O teto era regra escrita que nada contava, então os dois arquivos rodaram um quinto acima dele com 13 gates verdes. Regra com número e sem gate é indistinguível de regra ausente.
- Novo `skills_name_every_command`: `LEAF_COMMANDS` passa a ser afirmado sobre as duas skills. O `FULL_INVENTORY_DOCS` lista de propósito os sete documentos longos e nunca incluiu as skills, embora cada skill abra o catálogo declarando ser a superfície inteira — verdadeiro quando medido, mantido verdadeiro por nada.
- As duas skills passam a separar flags globais das locais de subcomando. A `--i-accept-network-exposure` estava documentada como global e é local do `tunnel`; colocada antes do subcomando sai com exit 2 no parse. A seção nova nomeia as quatro que parecem globais e não são.
- As duas skills passam a documentar os dois significados de `--disable-sudo`. Antes do subcomando ela suprime elevação numa invocação e não toca o disco; em `vps edit` grava `disable_sudo` no config e desabilita elevação naquele host de forma permanente. A `--enable-sudo`, único jeito de desfazer a forma persistente, não aparecia em nenhuma das skills, então o agente podia entrar num estado que a skill não ensinava a sair. A `--max-chars` também não era nomeada e passa a constar como alias legado do teto de comando.
- Removida narrativa de versão das duas skills, que as regras de produto proíbem ali: dois títulos com `0.5.4+`, uma cláusula `desde a 0.5.4/A3`, os identificadores de gap `G-SCP-R01/R02` e `G-ERR-R01`, e uma frase descrevendo como uma release anterior se comportava.
- A documentação publicada passou a cobrir a superfície 0.5.4 que ela apenas anunciava. `tunnel --reverse`, `--socks5` e `--remote-socket` embarcaram nesta release e apareciam em zero de `docs/HOW_TO_USE`, `docs/COOKBOOK`, `INTEGRATIONS`, `docs/MIGRATION`, `docs/CROSS_PLATFORM`, `SECURITY` e `llms-full.txt`, em nenhum dos dois idiomas: as três flags existiam só no banner de release e no pacote de skills. O `HOW_TO_USE` ganhou seção de modos de tunnel com exemplo executável por modo, o `COOKBOOK` ganhou quatro receitas, o `MIGRATION` ganhou a seção `Desde 0.5.4` cobrindo os dois BREAKING, o `SECURITY` ganhou seção de correções para A1/A2/A3 mais o guard de exposição, e o `CROSS_PLATFORM` ganhou a regra de portabilidade de que o cliente pode ser Windows e o socket não.
- Novo `the_054_surface_reaches_every_user_facing_document` em `tests/docs_conformance.rs`. Essa suíte já afirmava esses tokens, mas somente sobre as duas skills e os dois changelogs, então ficou verde durante tudo o que está acima — gate apontado para o subconjunto errado é indistinguível de gate ausente no placar. O contrato é por documento em vez de uma lista única: o cookbook deve uma receita, o guia de migração deve uma nota de upgrade, a política de segurança deve o reconhecimento.
- Novo `every_command_appears_in_every_full_inventory_document`: os 47 comandos folha devem aparecer como caminho completo literal em todo arquivo que declara inventário completo. O `llms-full.txt` e as tabelas de `docs/AGENTS` usavam notação de chaves (`tls acme {account create,…}`), então um recuperador que busca `tls acme account create` não achava nada e concluiria que o comando não existe. Notação compacta serve a um índice curto que um humano folheia; não serve a um arquivo cuja função declarada é ingestão por máquina.
- Novo `every_schema_is_indexed_in_the_full_llm_map`: existem 22 schemas em disco e o `llms-full.txt` indexava 14, omitindo `dry-run` e todos os contratos `sftp-*` e `*-batch` — exatamente os envelopes necessários para parsear trabalho de frota e de SFTP. O `docs/schemas/README.md` tinha gate que varre o diretório; o mapa de descoberta não tinha, então ficou atrás do disco.
- Corrigido um exemplo de cookbook que a CLI recusaria hoje: `--bind 0.0.0.0` aparecia sem `--i-accept-network-exposure`, que o G-TUN-R13 tornou obrigatória. Nenhum gate executa exemplo de cookbook, então a receita envelheceu para inválida sem sinal algum.
- Doze declarações de linha de produto passaram de 0.5.3 para 0.5.4. A deriva era simétrica nos dois idiomas, então nenhum gate de paridade podia vê-la; cerca de sessenta menções restantes a `0.5.3` são piso de versão legítimo (`prefira 0.5.3+` para SFTP) ou seções históricas de migração, e foram classificadas antes de qualquer edição em vez de substituídas às cegas.
- A máscara de permissão SFTP passa a estar documentada como o que ela é — direcional. `SFTP_PERM_MASK` (`0o7777`) vale na saída, no upload, e preserva setuid/setgid/sticky de propósito num arquivo que o chamador já controla; `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) vale na entrada, no download, onde o modo vem do servidor. Doze linhas em dez arquivos nomeavam a primeira como se fosse a única máscara, e `docs/MIGRATION` anunciava A3 quinze linhas acima contradizendo-a — o leitor conclui que a correção não existe. O novo `permission_mask_claims_are_directional` varre `docs/` e reprova qualquer arquivo que cite `0o7777` sem a constante de entrada, então a classe não pode reincidir, e não só esta instância.
- Esse gate de máscara nasceu varrendo só `docs/` — o diretório que eu estava editando — e ficou verde enquanto a afirmação idêntica seguia incorreta em `llms.txt`, `llms.pt-BR.txt`, `llms-full.txt` e nos dois `SKILL.md`. Um gate escrito para pegar alvo errado foi ele mesmo escopado ao diff do autor, e não à afirmação. Ampliá-lo para uma lista escrita à mão de quatro diretórios então perdeu `docs/schemas/README.md`, porque `read_dir` não recursa — lista de lugares onde olhar é, ela mesma, coisa que fica atrás do disco. A varredura agora é recursiva a partir da raiz sobre `.md` e `.txt`, pulando `target/` e `.git`, então documento criado em subdiretório novo entra sem ninguém lembrar de cadastrá-lo; seis arquivos a mais foram corrigidos.
- `--tags` está documentada. O seletor é real em `exec`, `sudo-exec` e `su-exec`, e tinha zero ocorrências nos quinze arquivos de `docs/` enquanto `--all` e `--hosts` estavam documentados em todo lugar — um agente que lê esses arquivos conclui que frota por rótulo não existe e abre um processo por host. O novo `fleet_documents_name_every_selector` afirma que todo seletor que o clap aceita aparece onde a frota é descrita; ele também pegou `--hosts` ausente dos dois `docs/HOW_TO_USE`, que nenhuma busca manual havia procurado.
- Descrição de A1 corrigida em seis pontos. O teto de log de 512 caracteres do banner é anterior a esta release (G-SSH-14); o que A1 corrigiu foi o corte aplicado por índice de byte, que entra em pânico dentro de caractere multibyte, e `panic = "abort"` transforma isso em morte do processo sem unwind — um par remoto conseguia matar um fan-out multi-host com um único caractere não-ASCII. O `SECURITY.md` titulava a falha como banner pré-auth "sem teto" e afirmava que a correção limita o que o par faz o cliente reter; o russh materializa o banner inteiro antes do callback, então o limite residual agora está explícito: A1 limita o pânico, não a alocação.
- Corrigida a afirmação de que `tunnel --bind` é validado como IP pelo clap em todos os casos. Isso vale para o bind local. Sob `--reverse` a ponta exposta é o posicional `<remote_host>`, guardado por `guard_remote_exposure`, que compara texto em vez de parsear IP porque a RFC 4254 dá significado a nomes e à string vazia — um typo ali é exit 64 do guard, não exit 2 do clap. Quatro linhas diziam o contrário.
- Documentado que `--bind` é aceito e então descartado em silêncio sob `--reverse`: a entrega reversa é forçada para loopback e o `ReverseServe` nunca recebe o endereço. Os docs diziam corretamente que o reconhecimento guarda o bind do servidor, mas nenhum avisava que a própria flag não faz nada ali, então `--reverse --bind 192.168.1.10` não mudava nada e não avisava nada.
- `docs/RELEASE_CHECKLIST` ganhou o item 26, o bloco de honestidade da 0.5.4. Medido antes da correção: os dois checklists e os dois `docs/TESTING` mencionavam `tunnel_closed`, `--select` e `--count-only` somente na linha 3 — o banner de release — e não nomeavam nenhum dos três modos de tunnel em lugar algum, com zero menções a `--dry-run` ou `--tags`. Um checklist existe para ser contrato, e este anunciava a release no cabeçalho enquanto não verificava nada dela no corpo. Os quatro arquivos entraram em `SURFACE_054`, e `gaps_v060_tunnel_modes.rs` está nomeado no inventário de suítes.
- Uma linha de terminologia corrigida, e não nove. A linha 63 de `docs/MIGRATION` usava master-key como termo de produto nos dois idiomas; as outras oito menções em `docs/` descrevem explicitamente o alias legado de keyring aceito na leitura, o que é verdade, então substituição cega teria destruído oito afirmações corretas.
- `TransferResult` carrega os dois flags de durabilidade; `print_transfer_json` recebe por referência em vez de oito argumentos posicionais.
- `build_tunnel_closed` / `build_tunnel_listening` separam a construção do payload da emissão. `tunnel_closed`, `forwards_served` e `capacity_waits` apareciam fora do emissor em exatamente um lugar: um teste que verifica se o CHANGELOG os menciona. Apagar a emissão não deixaria a suíte vermelha — prosa validando prosa, a um passo da tautologia que o G-QA-R01 foi escrito para impedir.
- Novos `tests/gaps_v061_error_taxonomy.rs` e `tests/gaps_v061_scp_durability.rs`; novos testes de tunnel cobrindo repasse de agent-auth, o timeout de registro deliberadamente não sobrescrito, cada braço de `close_reason`, o campo `bind` e o `forwards_served` medido num accept real de loopback com cliente injetado.
- O `gap_deny_002_deny_toml_sem_ignore_cve` travava `multiple-versions = "warn"` literalmente, então endurecer a política quebrava o teste enquanto afrouxar para `allow` passaria batido. Agora ele rejeita `allow` e aceita qualquer coisa mais estrita.
- A persistência do `known_hosts` dimensiona o buffer pela contagem de entradas em vez de crescer do zero a cada escrita TOFU.
- Dois doc-comments em português em `src/ssh/client_real_scp.rs` e `src/retry.rs` traduzidos para inglês, conforme a regra de código-fonte em inglês.
- **`scripts/check_all_gates.sh`** roda a bateria obrigatória inteira — `fmt`, `build-release`, `build-no-default`, `clippy`, `test`, `deny`, `cross-targets`, `advisory-freshness`, `en-identifiers`, `install-resolve` — numa invocação, com um registro TSV ou NDJSON por gate no stdout, log por gate no stderr e exit não-zero se qualquer gate estiver vermelho. Ele existe porque `cargo clippy` e `cargo test` abortam no primeiro alvo que não compila, então um único arquivo de teste quebrado escondia o estado de todo gate atrás dele — que foi exatamente como o inventário local passou a declarar 835 verdes com quatro gates vermelhos e um alvo inconstruível. Ele pegou uma violação de rustfmt na primeira execução. Não é CI: sem workflow, sem runner, sem rede. A bateria é sequencial por decisão, porque os gates de cargo disputam um lock de `target/`. O `scripts/check_advisory_freshness.sh` não tinha outro chamador, então só era alcançável por um script que nada documentava; o runner e esse gate agora estão declarados em `CONTRIBUTING`, `docs/TESTING` e na checklist de release, nos dois idiomas.
- **`tests/gaps_v064_gate_runner.rs`** transforma a cobertura do runner em contrato. Todo `scripts/check_*.sh` precisa ser um gate, e todo `scripts/*.sh` precisa ser gate ou estar nomeado — com motivo — num bloco `Declared non-gates` no cabeçalho do runner. A cobertura antes era completa só por coincidência de nomenclatura: um `scripts/check_foo.sh` futuro passaria batido, e bateria que omite script em silêncio lê como cobertura total. A suite também trava `--locked` no gate de teste e afirma que o runner está documentado nas duas versões de idioma dos três documentos de mantenedor. As listagens de diretório são ordenadas antes de qualquer asserção, porque o `std::fs::read_dir` documenta que sua ordem depende de plataforma e pode mudar entre chamadas.
- O `tests/docs_conformance.rs` passa a afirmar os tokens de `CONTRIBUTING`, `docs/TESTING` e `docs/RELEASE_CHECKLIST` sobre os **dois** arquivos de idioma. Checar só o inglês é como o `CONTRIBUTING.pt-BR.md` passou a omitir `gaps_v040`, a lista explícita de suites e a seção inteira do gate cross-target com a suite verde — o leitor que não lê inglês recebia estritamente menos, e o gate de paridade não tinha como ver. O arquivo pt-BR também trazia faixa elidida de suites (`v038 … v051`), exatamente o padrão contra o qual o par em inglês adverte, porque checagem por `contains` derruba em silêncio as suites do meio.
- **D1** o `tests/gaps_v040_integration.rs` chamava `tunnel_subsystem()` e `output_subsystem()`, nenhuma das duas existente, então o alvo não compilava e `clippy` e `test` abortavam antes de um único teste rodar. As duas são wrappers finos sobre um helper compartilhado `concat_subsystem`, ao lado do `i18n_subsystem()` que o autor original havia escrito. Código de teste fica fora da concatenação de propósito: incluí-lo deixaria a asserção passar por casar com o texto do próprio teste.
- **D4** o `tests/gaps_v063_secret_stdin.rs` exigia exit **64** de um erro de parse do clap. Confirmado na fonte no clap 4.6.6, cujo `error::Error::exit` documenta "exits with a status of `2`": os códigos sysexits de `src/errors.rs` valem somente para erro de produto levantado *depois* do parse bem-sucedido. O `assert_ne!` companheiro passava por vacuidade, porque 64 nunca é o que o parser emite; agora ele exclui 2, que é o código que a forma suportada de fato não pode produzir.
- **D7** o `tunnel::tests::accepting_a_connection_increments_forwards_served` passava isolado e falhava na suite numa taxa que acompanhava `--test-threads`. O `serial_test::serial` serializa apenas os testes que o carregam, então um leitor não-serial da flag global de parada continuava correndo contra escritores marcados. Corrigido com as duas metades — `#[serial_test::serial]` mais `crate::signals::reset_flags_for_tests()` — e verificado por execuções repetidas em 8, 32 e 72 threads, não por uma execução verde, já que uma passada só não distingue correção de corrida.
- A catraca de componente então mordeu a própria correção: o `src/tunnel.rs` passou do orçamento duro de 600 linhas, e entradas de `DECLARED_EXCEPTIONS` só podem encolher. `TunnelStats` e seu teste unitário de `close_reason` migraram para o novo `src/tunnel/stats.rs`, derrubando `tunnel.rs` para 570 e removendo a entrada dele do ledger. Levantar o teto seria a desonestidade que a catraca existe para bloquear.
- **D3** o `scripts/e2e_real_ssh.sh` substituía um `--bin` não executável pelo default e seguia, imprimindo `PASS E01` até `PASS E16` para um binário que o operador nunca nomeou — num harness cujo próprio texto de uso promete que ambiente inutilizável é falha e nunca skip silencioso. Uma flag de intenção `BIN_EXPLICIT`, marcada por `--bin` e pelo `SSH_CLI_E2E_BIN` harness-only, faz esse caso sair **2** com `FAIL E00`; o auto-build sobrevive somente quando nada foi nomeado. Verificado negativamente.
- **D9** o `tests/gaps_v062_i18n_reachability.rs` procurava `fn en(msg: &Message)` em `src/i18n.rs`, que o C3 havia movido para `src/i18n/en.rs`, então a contagem era incondicionalmente o arquivo inteiro e o ramo guardado nunca podia ser tomado. A busca morta e sua condição saíram e o doc comment agora descreve o que o código faz.
- **D6** a lista de campos do `scp-transfer` em `skills/ssh-cli-en/SKILL.md` ganhou `ok`, alinhando à skill pt-BR que já tinha os quatro. A lista de tokens do gate de paridade ganhou `ok`/`direction`, `mtime_preserved` e `durable` — os três que de fato discriminam, já que os nomes de campo isolados já aparecem nas duas skills por outros contextos e a checagem é `contains`.

## [0.5.4] - 2026-08-06

### Segurança
- **A1** Corrigida negação de serviço remota pré-autenticação. `auth_banner` fatiava o banner enviado pelo servidor no byte 512 (`&banner[..512]`); um caractere multibyte nessa fronteira causava panic e, como o perfil de release define `panic = "abort"`, o processo inteiro morria — derrubando junto qualquer fan-out `--all`. A truncagem agora respeita fronteiras de caractere.
- **A2** Chaves privadas ACME/mTLS não ficam mais legíveis por todos numa janela transitória. `write_secret_file` usava `std::fs::write` (criando em `0644` sob a umask padrão) e restringia depois, com o erro descartado. Agora delega ao helper compartilhado que cria em `0600` via `O_EXCL` e propaga falha de permissão.
- **A3** Bits `setuid`/`setgid`/`sticky` enviados pelo servidor não são mais reproduzidos no arquivo baixado. Modos vindos da rede são mascarados com `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) em SFTP e SCP; modos de arquivos locais enviados ao servidor mantêm a máscara completa.
- **A6** `parse_hex_key` rejeita entrada não-ASCII antes de fatiar, então um `secrets.key` com UTF-8 multibyte somando 64 bytes devolve erro tratado em vez de panic.

### Adicionado
- **C1** Camada agent-native de redução de payload como flags globais: `--select`/`--fields`, `--filter` (repetível, AND), `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`. Aplicada no funil único de serialização JSON ANTES de o envelope existir, então o payload gigante nunca é construído. Medido: `vps list` cai de 943 para 19 bytes com `--select name --limit 1`. Um `--filter` malformado falha no parse (exit 64) em vez de casar nada em silêncio.
- **C2** `--no-input` recusa stdin de forma declarativa em vez de bloquear para sempre esperando um humano ausente.
- **C2** `--dry-run` imprime o plano de uma operação destrutiva e sai sem executá-la: `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt`. O plano é JSON mesmo no modo texto (`docs/schemas/dry-run.schema.json`), porque prévia existe para ser comparada. Em qualquer outro comando a flag é **recusada com exit 64** em vez de aceita e ignorada — exatamente o defeito com que `--no-input` foi entregue. As pré-condições rodam antes, então prever a remoção de um host ausente ainda sai 66, em vez de prometer um sucesso que a execução real não entregaria.
- **G-TUN-R01** `tunnel --reverse` pede ao servidor que escute e entrega as conexões de volta numa porta local. `REMOTE_PORT 0` é aceito só nesse modo: o servidor aloca e informa a porta que ligou, e é ela que chega em `tunnel_listening.local_port`. Bind remoto fora do loopback exige `--i-accept-network-exposure` — nessa direção quem fica exposto é o listener do servidor, então guardar o `--bind` local checaria o lado errado.
- **G-TUN-R02** `tunnel --socks5` serve um proxy SOCKS5 (RFC 1928, sem autenticação + `CONNECT`); cada conexão aceita vira um canal `direct-tcpip`. Nomes de host são encaminhados sem resolução local, para que signifiquem o que significam do lado *remoto*. `BIND` e `UDP ASSOCIATE` recebem o código de resposta `0x07` em vez de um fechamento seco. O handshake tem teto de 1024 bytes, acima dos 519 que a RFC permite e muito abaixo de qualquer valor útil para fazer o proxy bufferizar à vontade.
- **G-TUN-R03** `tunnel --remote-socket <PATH>` encaminha uma porta local para um socket Unix no host remoto (`direct-streamlocal@openssh.com`). O caminho é validado como absoluto e sem byte NUL; NÃO é conferido contra o filesystem local, que nada tem a ver com o do servidor.
- **Contrato do tunnel** `tunnel_listening` e `tunnel_closed` ganharam `mode` (`local` / `socks5` / `streamlocal` / `reverse`). É o discriminador que diz como ler os campos vizinhos: sob `reverse` quem escuta é o servidor, e sob `socks5` não existe destino único. O default é `local`, então eventos anteriores à 0.5.4 continuam válidos.
- **G-TUN-R07** Novo evento `tunnel_closed` (`docs/schemas/tunnel-closed.schema.json`) com `reason` (`deadline` / `signal` / `accept_error`), `forwards_served`, `capacity_waits` e `duration_ms`. Esses três finais compartilhavam exit 0 e eram indistinguíveis.
- **G-TUN-R06** `tunnel_listening` ganhou o campo `bind`, permitindo auditar pelo próprio contrato se um serviço foi publicado além do loopback (aditivo; consumidores existentes não são afetados).

### Alterado
- **BREAKING — G-ERR-R02** Falha parcial multi-host agora sai com exit **1**, `error_code: "partial_failure"` e `error_class: "partial"`, carregando `failed`/`total`. Antes saía **65**, o mesmo código de TOML malformado, então o agente não distinguia "1 de 10 hosts falhou" de "a configuração está corrompida". O exit 65 volta a significar erro de dado real.
- **BREAKING — G-TUN-R13** Um `--bind` fora do loopback agora exige `--i-accept-network-exposure`; sem a flag o tunnel sai com exit **64** antes de qualquer I/O de rede. `--bind 0.0.0.0` antes publicava o serviço remoto encaminhado na rede local em silêncio.
- **G-TUN-R08** `--bind` é validado como endereço IP pelo clap, então um typo falha no parse (exit 2) em vez de após um handshake SSH completo.
- **G-TUN-R09** Falhas de bind mapeiam por `ErrorKind`: `AddrInUse`/`PermissionDenied` → 74 (retryable), `AddrNotAvailable`/`InvalidInput` → 64. Todas colapsavam antes em 65 classificado como permanente.

### Corrigido
- **C2** `--no-input` agora recusa stdin em `vps add` e `vps edit`. A recusa estava implementada só no caminho de override de exec/scp/tunnel, então os dois comandos de registro — justamente os mais prováveis de rodar sem supervisão — aceitavam a flag e liam a senha do mesmo jeito. O guard foi movido para dentro de `read_secret_stdin`, de modo que todo chamador o herda. Coberto por `tests/gaps_v059_agent_native.rs`, que não existia quando a flag foi entregue.
- **Gate de docs** `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` voltou a ficar verde: 56 links intra-doc quebrados em 14 sites foram qualificados por completo, e os links para itens `pub(crate)` (`take_utf8_capped`, `EXEC_CAPTURE_HARD_MAX_BYTES`, `lock_config`) viraram código inline, já que item privado não tem página pública para apontar.
- **E3** `tunnel` respeita `--use-agent` e `--agent-socket`. O clap aceitava ambos e o dispatch os descartava, então um host registrado para autenticação por agente não conseguia abrir tunnel algum.
- **G-TUN-R10/R11** A cópia do forward usa `tokio::io::copy_bidirectional`, que reporta bytes por direção e o erro. O `join!` anterior de dois `copy` descartava os dois `Result`, tornando uma falha no meio da transferência indistinguível de um término limpo em qualquer verbosidade.
- **G-TUN-R12** A saturação de concorrência é anunciada uma vez por execução e reportada como `capacity_waits`; antes era invisível e aparecia só como latência sem causa declarada.
- **E4** O banner humano imprime o endereço de bind efetivo em vez de `localhost:` fixo.
- **E7** `open_tunnel_channel` preserva o erro de origem via `channel_src` em vez de achatá-lo em texto.
- **E6** `tunnel-listening.schema.json` está alinhado ao range do clap para `remote_port`. Com os modos acrescentados adiante nesta release o mínimo compartilhado passa a ser 0, já que `socks5` e `streamlocal` não têm porta remota única e `reverse` usa 0 para dizer "o servidor aloca"; `local_port` mantém `minimum: 1`, que é o campo ao qual o agente se conecta.
- **B2** SFTP verifica contagem de bytes nas **duas** direções. O SCP já conferia `sent != size` e `received != size` desde a 0.5.2 enquanto o SFTP não conferia nenhuma, então uma transferência SFTP truncada reportava `ok` com um campo `bytes` plausível — o laço contava o que escreveu, nunca o que sobreviveu. Uploads agora comparam com o tamanho local anunciado E refazem `stat` no remoto (a prova de efeito no destino: cota estourada ou filesystem cheio são invisíveis de outro modo); downloads verificam antes do rename atômico, então leitura curta nunca chega ao caminho final. Servidores que omitem `size` são aceitos em vez de reprovados, porque o atributo é opcional no protocolo.

### Interno
- **G-QA-R01** Nova suite `tests/test_quality.rs` reprova o build em asserções entre dois literais ou `assert!(true)`. `src/tunnel.rs` mantinha `assert_eq!(0_u64, 0)` sob o nome `timeout_zero_conceptually_rejected`, que não exercitava código de produto algum enquanto sugeria que a guarda one-shot estava coberta.
- **G-QA-R03** Asserções de documentação movidas para `tests/docs_conformance.rs`, então editar prosa não deixa mais o gate de comportamento vermelho sem regressão funcional.
- **G-DOC-R01/G-DOC-R02** Fechados junto com a separação acima: o `CONTRIBUTING.md` passa a listar cada suíte de teste nominalmente em vez de citar `gaps_v040` como se fosse a superfície inteira, e a skill pt-BR documenta os campos do `scp-transfer` (`ok`, `direction`, `bytes`, `duration_ms`) que a versão em inglês já trazia. Ambos foram corrigidos neste release sem serem citados nele.
- **G-TUN-R04/R05, E1, E2** Os testes de tunnel passam a exercitar comportamento real: a guarda `timeout_ms == 0` é chamada, `run_tunnel_with_client` é de fato invocada, e a asserção de porta efêmera confere a porta ligada em vez de casar strings do próprio fonte com `include_str!`.
- **C3** O bloco de oito linhas de "N/M falharam", duplicado em cinco caminhos de lote, foi substituído por `errors::finish_batch`.
- **Divisão G-TUN** `src/tunnel.rs` virou raiz de módulo sobre `local`, `reverse`, `socks` e `streamlocal`. Os três modos de listener local compartilham um único laço de accept parametrizado pelo destino: copiá-lo significaria manter o tratamento de sinal, o portão de admissão, o drain e a contabilidade de saturação em triplicata — justamente as partes que dão errado de forma sutil em uma cópia e não nas outras. `run_tunnel` recebe um `TunnelRequest` em vez de quinze argumentos posicionais, onde uma transposição compilava limpa e só aparecia como um tunnel apontando para onde ninguém pediu.
- **G-QA-R01** `gap_tun_003_source_local_addr` não faz mais grep de `local_addr()` e `effective_port` no texto de `src/tunnel.rs`. Essa asserção passava com a string dentro de um comentário, continuava passando depois de o comportamento ser apagado, e falhava pelo único motivo que não é regressão: a divisão de módulos moveu o código. Agora ela dirige o laço de accept real com um cliente injetado e lê a porta que o produto publicou.
- Canais `forwarded-tcpip` e `forwarded-streamlocal` não solicitados são recusados com `AdministrativelyProhibited` em vez do default do russh, que aceita e descarta. A fila de entrada é limitada, então um servidor não consegue crescer a memória do cliente abrindo canais mais rápido do que são drenados; fila cheia aplica backpressure no fio.

## [0.5.3] - 2026-07-30

### Corrigido
- **G1** upload SFTP não trunca mais o destino a zero bytes (`FileAttributes::empty()` em vez de `Default`).
- **G2** filtro `-v` com escopo na crate (`warn,ssh_cli=…`); nunca `debug` global (sem vazamento de senha via `russh::client::encrypted`).
- **G3** SETSTAT SFTP envia `atime`+`mtime` juntos (sem atime no epoch).
- **G4** Result de `set_metadata` SFTP é propagado (SETSTAT mutante não é best-effort).
- **G5/G17** cancelamento multi-arquivo preenche resto como cancelled; `results.len() == input.len()`.
- **G6/G11** testes de sinal serializados; `reset_flags_for_tests`; cardinalidade sob cancel.
- **G7** E2E real cobre SFTP com matriz de checksum + árvore recursiva (E17/E18).
- **G8** `exec --json` de passo único emite exatamente um objeto NDJSON.
- **G9** download SCP propaga falha de `sync_data` antes do rename atômico.
- **G10** débito de formatação fechado com gate `cargo fmt --check`.
- **G12** permissões mascaradas com `SFTP_PERM_MASK` (`0o7777`).
- **G13** removido teste circular que assertava texto FIXED em `gaps.md`.
- **G14** verbosidade graduada `-v`/`-vv`/`-vvv` (`ArgAction::Count`).
- **G15** inventário exige prova de efeito no destino (checksum), não auto-certificação.
- **G16** identificadores e erros de canal em inglês em `client_real_scp.rs`.
- **G18** falhas de `set_permissions` no download SFTP são sinalizadas.
- **G19** constante nomeada `SFTP_PERM_MASK`.

### Alterado
- Versão **0.5.3**.

## [0.5.2] - 2026-07-19

### Adicionado
- **`--json` global** (G-AUD-01): alias agent-friendly que força JSON (clap `from_global` nos subcomandos).
- **`exec` / `sudo-exec` / `su-exec` com VPS ativo** (G-AUD-04): um posicional = COMMAND no host ativo de `connect`.
- **Envelope JSON de `vps path`** (G-AUD-02): `event: vps-path` quando o formato é JSON.
- **Módulo `fs_perm`** (G-AUD-24): fonte única para modos Unix de arquivos/dirs de segredo.
- **Comandos root `schema` + `doctor`** (G-E2E-02/03): descoberta de contrato por agentes; `doctor` é alias de `vps doctor`.
- **`vps add --use-agent` / `--agent-socket`** (G-E2E-19): triplo de auth no inventário (senha / chave / agent).

### Corrigido
- **Warning falso de password em argv** (G-AUD-08): inspeciona `Option` real, não strings `Debug`.
- **TLS PEM ausente** (G-AUD-05): `FileNotFound` / classe permanente (não exit 74 retryable).
- **`vps export` honra formato JSON global** (G-AUD-03).
- **ACME account create exige `--contact mailto:…`** (G-AUD-06/28).
- **Exclusão mútua de auth primária** na gravação (G-AUD-07): exatamente um de password / key / agent.
- **Mensagens de secrets** não anunciam mais stores env (G-AUD-21).
- **Filtro de log só via CLI** (G-AUD-22 / G-E2E-09): `RUST_LOG` ambiente é **ignorado**; use `-v`.
- **Cap de concorrência** com fonte única (G-AUD-19/23): `constants::MAX_CONCURRENCY`.
- **Skill description ≤1024** chars (G-AUD-15).
- **ACME validação permanente** (G-E2E-01): `invalidContact` / tipos de problema 4xx → exit **64** não-retryable (`tls/acme_error_map.rs`).
- **Um único JSON em `vps add` com auto-key** (G-E2E-04): campo `secrets_key_auto_created` embutido em `vps-added` (um documento; nunca dois eventos).
- **Stamp de versão `-dirty` com `.commit_hash`** (G-E2E-06): proveniência honesta em trees dirty.
- **Feature clap `env` removida** (G-E2E-08); help não ensina mais stores env (G-E2E-07).
- **Máscara de export redacted** (G-E2E-10): `***` via `FIXED_MASK`, não string vazia.
- **Harness E2E offline SKIP** + bin release default (G-E2E-05); identificadores de teste em EN (G-E2E-13).

### Removido
- **`.github/workflows`** (G-AUD-11 / G-E2E-11): só gates locais; sem CI/GH Actions de produto na tree.
- **Shim PT `src/erros.rs`** (G-AUD-14).
- **Leituras env de config de produto** `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` (G-AUD-12).

### Alterado
- Versão **0.5.1 → 0.5.2**.
- Testes de integração alinhados a config só CLI/XDG (sem store env de secrets/formato).
- Gate residual: `tests/gaps_v058_e2e_residual.rs` (G-E2E-01…15,17,19 FIXED; 16/18 MITIGATED).

### G-SFTP residual harden R01–R15

### Segurança
- Validação de **basename de entry** + `ensure_local_under` em download recursivo/multi-file (servidor SFTP malicioso não escapa destino local).
- **Cleanup de partial** em qualquer erro de download SFTP (paridade SCP).
- **Root de upload tree** com `symlink_metadata` (no-follow).

### Alterado
- **Timeout wall-clock** (`under_timeout`) em multi-file e FS ops.
- **`cli/scp_args.rs`** extraído (SRP).
- Docs/skills: SCP = arquivos regulares; árvores/FS = **`sftp`**.

### G-SFTP: subsistema SFTP

### Adicionado
- **`russh-sftp` 2.3** + `ssh-cli sftp` (upload/download/`--recursive`, ls/mkdir/rmdir/rm/stat/rename).
- Schemas JSON `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch`.
- Gate `tests/gaps_v057_sftp.rs`.
- Agent em `ScpOptions`/`SftpOptions` (CLI/XDG).

### Segurança
- Stream 32 KiB (sem heap full-file); paths validados; recursive depth cap; symlink no-follow.

### G-SSH: regras SSH / russh

### Adicionado
- **`client_handler` / `client_connect` / `key_material`:** TOFU tipado, cadeia de auth, perms de chave.
- **Agent CLI/XDG:** `--use-agent`, `--agent-socket` (sem env como store).
- **Gates:** `tests/gaps_v056_ssh.rs`.

### Alterado
- client_id genérico `SSH-2.0-ssh-cli`; rekey/window/TCP keepalive explícitos; deny ban ssh2/thrussh.

### Segurança
- `HostKeyChanged` tipado; fail-closed known_hosts; RSA ≥2048; password só se secret non-empty no inventário.

### G-UNSAFE: unsafe code e FFI

### Adicionado
- **`test_util::env`:** encapsula `set_var`/`remove_var` com `// SAFETY:`.
- **`vps/config_io.rs`:** split de path/load/save (SRP).
- **Gates:** `tests/gaps_v055_unsafe_ffi.rs`.

### Alterado
- **`main`:** `register_handler` **antes** do Tokio multi_thread (G-UNSAFE-13).
- SAFETY de SIGTERM expandido; docs Windows FFI; testes plaintext via `set_runtime_flags`.
- Docs secrets/concurrency sem env-as-store; `forbid(unsafe_code)` em módulos puros.

### Segurança
- Allowlist de `unsafe` de produto: windows console + signals; env de teste encapsulado.

### G-ERR: tratamento de erros

### Adicionado
- Variantes `Domain`/`Crypto`/`Config`; TLS/canal com `source`; `error_code` no envelope JSON; gates `gaps_v054`; split do client SSH.

### Alterado
- Display minúsculo; `paths` tipado; validate de VPS com `DomainError`; secrets sem env-as-store; concurrency sem env store.

### G-DOM: tipos de domínio chrono/uuid/rust_decimal/url

### Adicionado
- **Quatro crates de domínio (coordenadas):** `chrono` 0.4.45, `uuid` 1.24 (v4+v7+serde), `rust_decimal` 1.42 (serde-with-str), `url` 2.5 (serde).
- **`src/domain/` dividido (SRP):** time, ids, http_url, money, names, ports, limits, command, error.
- **`Rfc3339Utc`:** timestamps VPS/ACME como `DateTime<Utc>` (wire RFC 3339).
- **`HttpsUrl` / `AcmeOrderUrl`:** parse HTTPS para resume ACME no XDG.
- **`BatchRunId` (v7):** campo `batch_run_id` nos JSON batch multi-host.
- **`Money<C>`:** biblioteca decimal (sem superfície monetária no VPS).
- **Gates:** `tests/gaps_v053_domain_types.rs` + proptest.

### Alterado
- Schemas batch exigem `batch_run_id`; import valida RFC 3339 em `added_at`.

### Segurança
- Sem `Local::now`; sem `serde-float`; URLs ACME só `https`.

### G-TLS produto: rustls / SSH-over-TLS / mTLS / ACME

### Adicionado
- **Feature `tls` (padrão):** `rustls` ≥ 0.23.18 + `aws_lc_rs`, `tokio-rustls`, `webpki-roots`, `rustls-pki-types`, `instant-acme`.
- **`CryptoProvider::install_default`** no `main` do binário (somente aws_lc_rs).
- **SSH-over-TLS**, **mTLS** (XDG) e **ACME** DNS-01 em dois passos (agent-friendly).
- Subcomando `ssh-cli tls …` e campos VPS `tls` / `tls_sni` / cert+key.

### Alterado
- `deny.toml` permite rustls de produto; ban mantém OpenSSL/native-tls/ring.
- PEM via `rustls-pki-types` (sem `rustls-pemfile`).

### G-TLS / política rustls — sessão anterior

### Adicionado
- **`src/ssh/connect.rs`:** helper único de Config + dial Happy Eyeballs (G-TLS-07/09).
- **Suite residual** `tests/gaps_v052_tls_policy.rs` (G-TLS-03).
- **SECURITY Política de transporte e crypto (G-TLS)** — SSH ≠ TLS; aws-lc-rs; rustls futuro só com ADR.

### Alterado
- **Compressão SSH só `none`** (G-TLS-04).
- **russh:** remove feature `flate2` (G-TLS-05); mantém `aws-lc-rs`.
- **`deny.toml`:** ban `openssl`, `ring`, `rustls` além de `openssl-sys` / `native-tls` / `libssh2-sys` (G-TLS-02).
- README / CROSS_PLATFORM / RELEASE_CHECKLIST / llms: superfícies de política crypto (G-TLS-01/06/08/11/12).

### Segurança
- Sem stack TLS de produto; sem OpenSSL/`native-tls`/`ring`/`rustls` no grafo.
- Sem OTEL de produto.

### Sistema de Tipos

### Adicionado
- **Newtypes de domínio (G-TYPE-01…20):** `src/domain/` com `VpsName`, `SshHost`, `SshUser`, `SshPort(NonZeroU16)`, `TimeoutMs`, `HostTag`, `CharLimit`, `RemoteCommand`, `KeyPath`, `BindPort`.
- **`ssh/session_io.rs`:** extração de helpers UTF-8 (G-TYPE-14).
- Testes de layout zero-cost para `SshPort`.

### Alterado
- **`VpsRecord` / `ConnectionConfig`:** campos com prova de tipo; `try_new` no lugar de `new` infalível.
- **`HostSelection`:** tipado com `VpsName` / `HostTag`.
- **`ExecOptions` / `ScpOptions`:** `TimeoutMs` e `RemoteCommand`.
- **CLI:** portas SSH com range 1..=65535; bind local ainda aceita 0.
- **`validate_and_normalize` → `VpsName`**.
- **Import JSON:** host/user vazios rejeitados na fronteira.

### Segurança
- Helper único `secret_nonempty`; sem OTEL de produto.

### Notas de sessão (validação / serde)


### Adicionado
- **Pipeline de validação (G-SERDE-01…14):** `validator` 0.20 + `serde_with` 3 + `serde_path_to_error` + `serde_ignored`; módulo `src/validation.rs`.
- **Tags no JSON agent (G-SERDE-06):** list/export/import com round-trip.
- **Validação estrutural no load (G-SERDE-04).**
- **Fuzz** `import_envelope` (G-SERDE-12).
- **`ssh/connection.rs`** e **`cli/tests.rs`** (G-COMP-R).

### Alterado
- **deny_unknown_fields** no TOML crítico; import JSON Must-Ignore com warn.
- **Arc\<ScpOptions\>** no fan-out multi-host SCP (G-MEM-SCP).
- **Actions CI pinados por SHA** (G-PROC-PIN).

### Segurança
- Sem telemetria de produto. Secrets em `SecretString`.

- Read this document in [English](CHANGELOG.md).

Todas as mudanças notáveis deste projeto são documentadas neste arquivo.

O formato segue [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
e o versionamento segue [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### Prior closeout / process notes

### Changed
- **O1–O6 + processo (obrigatório):** `--fail-fast`; tags de host; multi-cmd `--step` na mesma sessão; SCP `--scp-file-concurrency`; Arc de options no fan-out; proptest/fuzz; CI miri/geiger/sbom; `scripts/release_attest.sh`. Zero telemetria de produto.
- **Componentização profunda (G-COMP-05 / G-COMP-06a–d / G-CLOSE-09 / G-DRY-01 / G-EN-R01):** extraídos `vps/exec_ops.rs` (exec/sudo/su + DRY `finish_execution_output`); `ssh/scp_wire.rs`; `scp/{mod,batch}.rs`; `output/{mod,batch}.rs`; `cli/{mod,dispatch}.rs`; `commands/*` com reexports reais. Renomes EN residuais. OPEN de segurança de produto permanece 0; monólitos inventoriáveis fatiados.
- **Componentização (G-COMP-02…04):** extraídos `vps/doctor.rs`, `vps/import_export.rs` e `vps/health.rs` do monólito `vps` (`mod.rs` ~2428 → ~1698 LOC); reexport de `HostHealthResult` / `run_health_check`.

### Segurança
- **Meta-auditoria de fechamento (G-CLOSE):** casts de doctor/concurrency/SCP via `TryFrom`; `forbid(unsafe_code)` nos módulos puros restantes; extração `vps/selection.rs` (SRP); reexecução context7-cli + docsrs-cli + duckduckgo para conformidade da skill.
- **Auditoria de segurança de desenvolvimento (G-SECDEV):** secrets atravessam a fronteira CLI como `SecretString` (`read_secret_stdin` + overrides de exec/scp/tunnel/health); módulos puros com `#![forbid(unsafe_code)]`; deny de `clippy::mem_forget` + unsafe sem SAFETY / multi-op; mapa STRIDE + preferência CVSS v4 em `SECURITY.md` (+ pt-BR).
- **Auditoria de segurança defensiva (G-SEC):** `deny(unsafe_op_in_unsafe_fn)`;
  `overflow-checks` em release; comparação constant-time de fingerprint TOFU;
  caminhos de produto sem `.unwrap`/`.expect`/`unreachable!` em parsers CLI,
  admissão de concorrência e ramos single-host; porta de import via
  `u16::try_from`; `SshCliError` `#[non_exhaustive]`; modelo de ameaça em
  `SECURITY.md` (+ pt-BR); job CI `cargo deny check` (`deny.toml`).

### Added
- **Auditoria de retry (G-RETRY):** classificação tipada de erros (`ErrorClass` /
  `ErrorLayer` / `RetryKind`, `is_retryable` / `is_permanent` / `suggestion`) em
  `SshCliError`; `retry::RetryConfig` nomeado com backoff full-jitter e defaults
  de agente (máx. 2 retries no exit 74); envelope JSON com `error_class`,
  `retryable`, `suggestion` + schema. Auto-retry in-process de ops remotas não
  idempotentes permanece **desligado** (agente reinvoca o processo).

### Corrigido
- **Auditoria de rede (G-NET):** dial SSH com DNS assíncrono + corrida multi-endereço
  Happy Eyeballs (`net::dial_tcp` + `russh::client::connect_stream`); `TCP_NODELAY` e
  keepalives SSH (`15s` / máx. `3`); carga de chave privada e TOFU de known_hosts em
  `spawn_blocking`; accept do tunnel resiste a erros transitórios e aplica nodelay nos
  forwards locais.


### Alterado
- **Auditoria de hardcode (G-HC):** módulo central `constants` (nomes XDG, env keys,
  identidade do app, defaults de rede, timing de processo, AEAD/keyring); helper
  único `paths::xdg_config_dir()`. Sem segredos/URLs de produto no binário; hosts
  continuam no registry/CLI.

### Corrigido
- **Auditoria de processos externos (G-PROC):** probes `git` em `build.rs` com
  `Stdio` explícito (null/piped); comandos remotos rejeitam NUL antes do packing
  de exec SSH; fixtures de teste `ssh-keygen` usam argv direto + stdio explícito
  e fazem skip se o binário estiver ausente.
- Docs: política de fronteira de processo em CROSS_PLATFORM / AGENTS (sem spawn
  local OpenSSH; MSRV ≥ 1.77.2 BatBadBut; packing remoto `sh -c` só no host alvo).

### Adicionado
- **Concorrência multi-host com bound (modus operandi):** `health-check|exec|sudo-exec|su-exec|scp --all` faz fan-out com `Semaphore` + `JoinSet` (cap de `--max-concurrency` / `SSH_CLI_MAX_CONCURRENCY` / fórmula auto CPUs×RAM, clamp 1..=64). JSON batch: `health-check-batch` / `exec-batch` / `scp-batch` (`docs/schemas/*-batch.schema.json`). Forwards de accept do tunnel usam o mesmo gate.
- **Seleção seletiva `--hosts a,b,c`:** mesmo fan-out e JSON batch que `--all` (mesmo com um nome); unificado via `HostSelection` + `resolve_host_jobs`.
- **SCP multi-arquivo (single-host, G-PAR-47):** uma **sessão SSH** e transfers seriais (auth uma vez).
- **SCP multi-host × multi-arquivo (G-PAR-48):** `scp upload --all f1 f2 … REMOTE_DIR` — bound por sessão host; arquivos seriais na sessão.
- **TOFU flock (G-PAR-49):** mutações de `known_hosts` com lock exclusivo + reload-merge.
- **`vps doctor --probe-ssh [--hosts a,b]`:** um único root JSON `event: vps-doctor` com `local` + `ssh_probe` opcional (sem dual roots).
- **`map_bounded` cancel:** para admissão em SIGINT/SIGTERM; `force_exit` aborta JoinSet; span `fan_out_unit` + `available_permits`.
- Docs/skills de agente: frota multi-host + multi-arquivo / cartesiano SCP + envelope doctor.

### Alterado
- Path SCP (validação e pós-download) usa `tokio::fs` / `spawn_blocking` (não bloqueia workers sob fan-out).
- `scripts/dist_multiarch.sh` suporta `PARALLEL_JOBS` (default 2) via `xargs -P`.

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

[Unreleased]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.3...HEAD
[0.5.3]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.1...v0.5.2
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
