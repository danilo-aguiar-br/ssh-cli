# Changelog


- Leia este documento em [Inglês (en)](CHANGELOG.md)
- Todas as mudanças notáveis deste projeto estão documentadas neste arquivo
- O formato é baseado no [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/)
- Este projeto adere ao [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)
- Cada release lista suas seções nesta ordem: Segurança, Adicionado, Alterado, Depreciado, Removido, Corrigido, Interno


## [Unreleased]
- Nada ainda


## [0.5.6] - 2026-11-12

### Segurança
- O `russh` 0.62.5 agora é 0.65.1, o que fecha a RUSTSEC-2026-0492 (HIGH, callbacks do `client::Handler` disparados para um canal que nunca foi aberto, corrigida na 0.63.1) e a RUSTSEC-2026-0495 (o agent server nunca chamava `Agent::confirm`, corrigida na 0.65.0)
- O mesmo upgrade fecha a GHSA-fw5c-8223-6h42, em que um tamanho máximo de pacote zero travava toda escrita do canal, e a GHSA-3hm4-p7h4-rvwp, um estouro de pilha antes da autenticação disparado por certificado OpenSSH aninhado
- `Handler::check_server_key` agora recebe `&PublicKeyOrCertificate`, e o ramo de certificado é inalcançável porque `Preferred::DEFAULT` deixa `host_key_certificates` vazio, então a chave embutida passa pelo mesmo TOFU
- O `rustix` 1.1.4, marcado unsound pela RUSTSEC-2026-0486, agora é 1.1.6, e o `async-executor` 1.14.0 yanked agora é 1.14.1
- O `deny.toml` pula o `num-bigint@0.4.8` com o motivo escrito, porque o russh 0.65 passou ao num-bigint 0.5 e o secret-service 5.2.0, a versão mais nova, ainda chega ao 0.4
- O piso de segurança do russh estava escrito à mão em três lugares que ainda aprovavam a linha 0.62 vulnerável, e o `gap_dep_002_russh_patched_no_lock` e o gate `install-resolve` agora exigem `>= 0.65.0`, tirado da RUSTSEC-2026-0495
- O `webpki-roots` 1.0.12 remove mais oito raízes destituídas: emSign Root CA - C1 e emSign ECC Root CA - C3 na 1.0.11, depois Buypass Class 2, Buypass Class 3, DigiCert Trusted Root G4, QuoVadis Root CA 1 G3, QuoVadis Root CA 3 G3 e Certigna Root CA na 1.0.12
- Gap 50: `ssh-cli secrets init --keyring` respondia `{"ok":true,...,"key_source":"none"}` e não gravava nada, porque `keyring = "3"` resolvia 3.6.3, que não tem feature default e caía num store `mock` no próprio processo
- O mock guardava o valor na memória de um processo só, então o `secrets status` seguinte respondia `encryption_active:false` e o agente recebia um sucesso que não podia usar
- A `keyring` agora é `4.2.0` com a feature `v1`, que habilita o Keychain no macOS, o Credential Manager no Windows e o Secret Service por zbus nos unix restantes, e falha fechado com exit 69 quando nenhum store responde em vez de escrever num mock
- Segunda causa medida: a flag do leaf `secrets init --keyring` nunca chegava ao estado de runtime, porque o `dispatch` só repassava o global `--use-keyring`
- O `secrets status` então re-resolvia a chave só pelas flags e respondia `none` sobre uma chave recém-gravada
- Terceira causa: uma chave gravada por um `secrets init --keyring` anterior era invisível sem repetir a flag, então `load_primary_key` agora cai para o store quando não há arquivo nem flag
- A entrada do keyring é escopada por um digest do diretório de config resolvido, porque o store do SO é da máquina inteira enquanto `secrets.key` e `config.toml` são por perfil
- Sem o escopo, uma execução isolada com `--config-dir` sobrescrevia a chave do perfil real
- D2: `--force` não grava mais `secrets.key.bak`, que violava a regra do projeto contra cópia de segurança e deixava material de chave para trás a cada rotação
- O erro que mandava restaurar `secrets.key.bak` agora aponta `secrets reencrypt` com a chave anterior fornecida por `--secrets-key-file`
- Gap 53: `secrets init --keyring` num profile que já tinha `secrets.key` respondia `key_source: keyring` e cunhava uma segunda chave primária em vez de recusar
- O arquivo continuava respondendo primeiro, então o `secrets status` seguinte respondia `xdg_file`, e todo segredo já em repouso falhava ao decriptar com a chave que o comando acabara de reportar como ativa
- O mesmo defeito existia na direção inversa: rotacionar do cofre para o arquivo deixava a entrada do cofre viva e sombreando o arquivo
- A guarda agora pergunta pelo PROFILE, e não pelo destino, então qualquer chave existente recusa a escrita sem `--force`, e o `--force` aposenta a outra fonte para sobrar exatamente uma chave na rotação
- A1: a âncora TLS distribuía uma autoridade destituída: `webpki-roots` 1.0.9, `rustls-webpki` 0.103.15 e `webpki-root-certs` 1.0.9 ainda carregavam a `emSign ECC Root CA - G3`, cujas intermediárias eMudhra têm chave privada pública (Mozilla Bug 2083449, CVE-2026-111943, RUSTSEC-2026-0381/0382/0383)
- O `cargo deny check` estava vermelho em `advisories`, e o cliente ACME e o TLS de saída montavam o `RootCertStore` a partir dessa lista
- O mínimo agora é declarado no manifesto, `webpki-roots = "1.0.10"`, então um `cargo update` genérico não devolve a árvore para a versão comprometida sem quebrar o build
- O lock resolve `rustls-webpki` 0.103.17 e `webpki-root-certs` 1.0.10, e o `cargo deny check` devolve advisories, bans, licenses e sources todos ok
- A4: um gate novo compara o banco de advisory com a árvore resolvida: `scripts/check_advisory_hits.py` roda `cargo audit --json` e falha em qualquer achado de `vulnerabilities.list` ou em pacote yanked
- O gate `advisory-freshness` anterior só media a idade do clone, então um banco velho podia deixar o gate verde sobre uma CVE viva, que é exatamente o que a nota de 2026-08-07 no `deny.toml` registra
- O modo texto escapa sequências de controle do terminal: Um nome de host, uma mensagem de erro ou um campo guardado podia carregar ESC, BEL ou caracteres bidi, e o modo texto os escrevia crus, então um nome hostil podia redesenhar o terminal do operador
- `src/output/sanitize.rs` (`escape_untrusted`, `escape_untrusted_keep_whitespace`) agora transforma controle e formatação em texto no estilo `\u{1b}` no funil único de texto: mensagens de sucesso, erro e aviso, `vps list`, `vps show`, `health-check`, `doctor` e as saídas em lote
- Nome de host novo tem charset estrito: `VpsName::try_new` aceita letras ASCII, dígitos, ponto, hífen e sublinhado, começando por letra ou dígito, de 1 a 255 bytes, mais as regras de nome de arquivo (sem `..`, sem ponto ou espaço no fim, sem nomes reservados do Windows como `CON`)
- Ele guarda `vps add`, o destino de `vps rename` e `vps edit --name`, e `vps import`
- A leitura é leniente de propósito (ver `Alterado`), então um `config.toml` antigo continua funcionando
- O diretório de config nasce privado: O diretório que a CLI cria recebe modo `0700` em todo nível novo, onde antes herdava `0755` do umask e expunha nomes de host e tamanhos de arquivo ao lado de um `config.toml` `0600`
- Um diretório existente só é apertado a `0700` quando é o XDG padrão, não é symlink e pertence ao usuário atual
- Diretório existente que a CLI não possui por padrão nunca recebe chmod: Um `--config-dir` apontando para um diretório que já existe mantém o modo, grava um aviso de log (visível com `-v`, porque o nível padrão é `error`) e aparece no `doctor` como `local.config_dir_exposed` e `local.config_dir_permissions`
- Nomes que viram identidade ou campo remoto também são escapados ou recusados: `tls mtls import --name` recusa os mesmos caracteres de controle, de formatação e bidi dos nomes de host (os intervalos estão em `Alterado`) com exit 64 e não cria diretório, enquanto Unicode comum como `serviço-1` continua aceito
- `sftp ls` e `sftp stat` escapam `\n` e `\t` do nome remoto em modo texto
- Limites declarados, não escondidos: O stdout e o stderr remotos do `exec` seguem crus por desenho
- O Windows não tem chmod, então o diretório depende da ACL do perfil
- As flags globais de modelagem alcançam objeto único em parte: `--select` e `--truncate-content` reduzem um objeto único como o relatório do `doctor`, e as seis flags de coleção `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` e `--max-output-bytes` aparecem em `agent_shape.ignored`
- O JSON mantém crus todos os caracteres desse conjunto (U+202E e ZWSP, por exemplo), porque JSON é para máquina, enquanto ESC e BEL já saem escapados pelo `serde_json`
- Os corpos de erro ficam em inglês por desenho, para o agente receber texto estável
- Nome legado não gera aviso no carregamento: o `doctor` o lista

### Adicionado
- `scripts/check_markdown_format.py` transforma o contrato de formatação em gate executável, porque os Gaps 25 e 37 nomeavam "o linter de formatação" como critério de aceitação e esse linter não existia em lugar nenhum da árvore
- O gate roda como `markdown-format` em `scripts/check_all_gates.py`, e com o gate `doc` a bateria passa a rodar catorze gates
- Treze regras cobrem negrito, itálico, separador horizontal, bullet de asterisco, bullet terminado em ponto ou vírgula, H4 e mais fundo, contagem de H1, espaçamento de H1 e H2, a folga entre header e primeiro bullet, travessão fora de H3, blockquote decorativo e emoji
- `--selftest` prova que cada regra dispara no próprio fixture, porque um regex que não casa nada reporta árvore limpa pelo mesmo motivo que um alarme quebrado reporta ausência de fogo
- `docs_rules/` e `docs_prd/` ficam declarados fora do escopo como material de origem importado, em vez de pulados em silêncio
- `tests/gaps_v075_markdown_format_gate.rs` mantém o gate alcançável pela bateria, executável na árvore real e livre de import de terceiro
- `src/secrets/keyring_store.rs` grava e lê numa thread de SO dedicada, porque o store do Linux estaciona num reator `async-io` e o processo roda um runtime Tokio multi-thread
- `secrets::delete_key_from_keyring` remove a entrada do perfil atual, então um perfil apagado não deixa a chave primária no store da máquina para sempre
- `AcmeDirectory::from_staging_flag` é o ponto único de decisão do `--staging`, substituindo as duas cópias do mesmo condicional nas folhas de ACME
- Os casos E2E E19, E20 e E21 cobrem o item reduzido de `vps list --select` contra o schema publicado, o `--select` composto com `--limit` mais a contabilização no stderr, e o round-trip do keyring em dois processos
- `docs/schemas/vps-list.schema.json` publica duas formas de item com `anyOf`: o registro completo por `vps-show.schema.json`, e o registro reduzido que o `--select` produz
- A2: `--select` e `--truncate-content` agora reduzem um objeto único, que antes era inerte em `vps show`, `doctor`, `commands` e em todo payload sem coleção
- `ssh-cli commands --select tree.about` devolve só o nome do produto em vez de 52699 bytes
- O registro reduzido carrega `agent_shape` com `selected`, `content_truncated` e `ignored`, então o consumidor sabe qual das duas formas publicadas está segurando
- `docs/schemas/vps-show.schema.json` e `docs/schemas/vps-doctor.schema.json` publicam as duas formas por `anyOf`, no mesmo espírito de duas variantes que o `vps-export` usa desde o Gap 35, em vez de afrouxar o `additionalProperties` do schema, e usam `anyOf` e não `oneOf` porque `--truncate-content` e as flags só de coleção mantêm o registro inteiro, então um `oneOf` casaria as duas variantes
- A6: todas as 48 folhas agora mostram bloco `Examples:` no `--help` enquanto o `-h` continua curto, e um teste roda cada argv publicado com `--help` anexado para provar que ele parseia contra o conjunto real de flags
- `vps rename <NAME> <NEW_NAME>`, rename nativo: Antes disto, o único jeito de mudar o nome de um host era `vps export`, editar o arquivo e `vps import`
- Isso é armadilha de perda de dado disfarçada de procedimento: o export redige todo segredo, então o import ou recusava o arquivo com exit 64 ou, com `--allow-incomplete`, gravava de volta um host sem senha, e o operador tinha de redigitar uma credencial que a ferramenta já guardava
- Pior para host autenticado por agente, que perdia a autenticação por completo (ver a entrada em `Corrigido`)
- Nome é rótulo de registro, não motivo para tocar no segredo dele
- `vps edit <NAME> --name <NEW>` chega ao mesmo caminho a partir do verbo que o operador já usava, então as duas grafias existem e nenhuma é segunda implementação
- Renomear nunca pede o plaintext, e toda credencial continua legível sob o nome novo: `VpsRecord` serializa segredos pelas variantes sem AAD `sshcli-enc:v2:`, então o dado associado do AEAD não carrega nome de host e o rename não tem o que re-selar
- O token de ciphertext em si muda, porque toda gravação sela de novo com nonce novo: isso é o comportamento normal do AEAD e nunca toca o plaintext
- O teste afirma a propriedade real em vez da igualdade de bytes: depois do rename, `vps export --include-secrets` ainda devolve a credencial original sob a chave nova
- Se uma versão futura ligar o ciphertext ao host, esse teste falha primeiro
- Oito flags de limpeza, uma por campo: `--clear-tags`, `--no-key`, `--no-agent-socket`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert` e `--no-tls-client-key` removem um valor
- Até aqui todo campo opcional podia ser sobrescrito e nenhum podia ser removido, então um operador definia um SNI de TLS e nunca o desfazia
- As flags usam booleanos dedicados em vez de valor sentinela, porque `--tls-sni ""` não distingue "limpe" de "eu ia passar um nome e o shell comeu"
- `--no-key` também limpa a passphrase: Limpar só o caminho da chave deixaria em disco um segredo que ninguém lê e nada alcança: um órfão
- `vps edit --no-key` num host cuja única autenticação era aquela chave é recusado com exit 64, porque o resultado não autenticaria em lugar nenhum
- `--no-agent-socket` falha fechado no Unix: Remover o socket explícito de um host que autentica pelo agente o deixa sem endpoint que a plataforma forneça, então a edição é recusada com exit 64 e mensagem que nomeia as alternativas
- No Windows o endpoint padrão existe, então o mesmo comando é aceito
- O predicado mora ao lado de `resolved_agent_socket` em `ssh::connection`, onde esse conhecimento já estava
- `--dry-run` agora cobre `vps edit` e `vps rename`: As duas prévias rodam toda validação que a execução real roda, parse de nome, detecção de duplicata, pareamento de TLS, validação do registro, e param antes da escrita
- O plano de `vps-rename` carrega `previous_name` e `active_rewritten`, então o chamador vê se o marcador se moveria sem movê-lo
- `commands --json` publica todo argumento: A árvore carregava `name`, `about` e `subcommands`, o que dizia ao agente que `vps edit` existe e nada sobre o que ele aceita
- Cada argumento agora aparece com `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` e `hidden`, e cada comando carrega seus `aliases`
- Dez flags novas teriam ficado invisíveis na única superfície feita para descoberta
- `--lock-wait-ms` limita o lock do config: Padrão 5000 ms, teto 300000 ms, ambos em `crate::constants`
- Ver a entrada de contenção em `Corrigido`
- `use_agent` e `agent_socket` ficam visíveis em `vps show`, `vps list`, `vps export` e `vps import`: Os tipos de wire passam a carregar os dois campos, `print_details_text` imprime estado do agente, tags, passphrase da chave e os cinco campos de TLS, e o import os define antes de validar
- O evento `vps-renamed` existe como envelope próprio: Rename não é uma edição que por acaso trocou a chave: o operador precisa saber qual nome resolve agora, e o `vps-edited` sozinho não diria
- O envelope carrega `name` (depois), `previous_name` (antes), `active_rewritten`, `updated` e o echo de alvo (`target_resolved`, `target_source`)
- `vps edit <NAME> --name <NEW>` chega ao mesmo caminho e emite o mesmo evento `vps-renamed`, então o chamador recebe uma resposta só, qualquer que seja a grafia usada
- `vps-edited` é emitido só quando nada foi renomeado e nunca carrega `previous_name`
- `vps edit` e `vps rename` informam `updated`, os campos cujo valor guardado realmente mudou: Uma flag pode ser aceita e ainda assim deixar o valor idêntico, `--port 22` num host que já está na 22, e sem esse campo o envelope afirmava uma mudança que não aconteceu
- `updated` lista os campos na ordem de aplicação, cada um no máximo uma vez, com `tags` por último
- Ele fica vazio quando todo valor pedido já coincidia ou quando a invocação só renomeou o host, e uma invocação sem flag de campo, rename ou limpeza é recusada com exit 64
- O campo está sempre presente no JSON, então o consumidor testa lista vazia e não chave ausente
- `vps-import` informa `replaced`: O evento de sucesso lista os nomes de host que o import sobrescreveu, ordenados, e o campo está sempre presente, vazio quando nada foi sobrescrito
- O dry-run mantém `replaces_existing`, que é uma previsão
- `replaced` é o registro do que a execução real fez
- `docs/schemas/vps-edit.schema.json` e `docs/schemas/vps-rename.schema.json`: Os dois entram no catálogo de `schema`, e `dry-run.schema.json` passa a listar oito operações
- `vps doctor --json` informa o diretório de config e os nomes ruins: `local` ganha `config_dir_exposed` (booleano, sempre presente), `invalid_host_names` (array de strings escapadas, sempre presente, vazio quando todo nome é válido) e `config_dir_permissions` (string octal de quatro dígitos como `0700`, ausente fora do Unix ou quando o diretório não existe, e fora do Unix só esse campo falta enquanto `config_dir_exposed` vale `false`)
- O relatório em texto imprime `Config dir mode:`, `Config dir exposed:` e `Invalid host names:`
- `permissions` continua descrevendo só o `config.toml`
- `fs_perm::ensure_config_dir` é o único lugar que cria o diretório de config: Uma varredura de fonte em `gaps_v073_config_dir_mode` falha se outro módulo o criar
- Quatro suítes novas impedem a volta das correções da 0.5.6: `gaps_v073_vps_auth_switch`, `gaps_v073_host_name_charset`, `gaps_v073_terminal_escape` e `gaps_v073_config_dir_mode`
- O `--help` mostra os códigos de saída e exemplos: O `--help` raiz termina com a seção `Exit status:` para 0, 1, 2, 64, 65, 66, 69, 70, 73, 74, 77, 130, 141 e 143
- As 48 folhas trazem a seção `Examples:` por `after_long_help`, que o `--help` mostra e o `-h` deixa de fora
- O texto de ajuda mora em `src/cli/help_text.rs`
- O rustdoc diz quando uma função falha: Ele ganha 78 seções `# Errors`, uma seção `# Panics` e 7 doctests, então os doctests passam de 25 para 32
- Os seis schemas de lote, de listagem e de prévia ganham os campos opcionais `agent_shape` e `count`: `exec-batch`, `scp-batch`, `sftp-batch`, `health-check-batch`, `sftp-list` e `dry-run` agora descrevem a saída modelada que as flags globais produzem
- O `vps-rename.schema.json` lista os 19 nomes de campo que o `updated` pode carregar

### Alterado
- BREAKING para builds, o MSRV agora é Rust 1.89, porque o `russh` 0.65 declara `rust-version = "1.89"`, enquanto a `keyring` 4 sozinha pedia 1.88
- `scripts/check_all_gates.py` roda catorze gates, e o gate novo `doc` roda `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features` logo depois do `clippy`
- `scripts/check_advisory_freshness.py` declara o teto de 7 dias como exceção operacional, com o motivo escrito ao lado
- `skills/ssh-cli-pt/SKILL.md` e a cópia guardada em `~/Dropbox/ai/ai-llms/skills-nao-usadas/ssh-cli/SKILL.md` voltam a ser byte a byte iguais: 63 linhas que só existiam na cópia instalada foram medidas contra o binário e fundidas, e a única regra que ensinava o comportamento anterior a este release foi trocada pelo que `vps show a1 --select name,host` de fato devolve
- O MSRV subiu de 1.85.0 por causa da `keyring` 4.2.0, que declara `edition = "2024"` e `rust-version = "1.88.0"`, e o store zbus segue
- A matriz E2E é E01 a E21, antes E01 a E18
- `docs/CROSS_PLATFORM.md`, `docs/AGENTS.md`, `docs/COOKBOOK.md` e `INTEGRATIONS.md`, nos dois idiomas, agora nomeiam as 89 flags publicadas, onde nomeavam 39, 73, 85 e 68
- A3: um array nu reduzido relata a contagem em stderr em uma linha JSON com `"event":"agent-shape"`, carregando `input_count`, `output_count` e `dropped`
- Antes, a contagem ia só para `tracing::info!`, que o nível default `error` descarta, então `ssh-cli vps list --limit 2 --json` com 60 hosts devolvia dois registros e nada dizia dos outros 58, enquanto o módulo prometia "truncation is never silent"
- O stdout mantém o tipo do array, porque o consumidor o indexa, e o `--quiet` suprime a linha de stderr como todo diagnóstico
- A5: o gate `clippy` agora passa `-W clippy::doc_markdown`, então uma doc de flag que perde a crase falha a bateria em vez de rolar despercebida
- `vps edit` e `vps rename` publicam proveniência de alvo: Os dois chamam `set_resolved_target` depois da busca bem-sucedida e antes de qualquer mutação, e ecoam `target_resolved`, `target_source: argv` mais os aliases de leitura `host_*`
- Publicar antes da busca inverteria o contrato existente: `vps_not_found` precisa continuar reportando alvo nenhum, porque nenhum host foi escolhido
- O braço `vps edit` em `crud.rs` virou chamada de construtor: O bloco de 186 linhas que lia stdin, tomava o lock, mutava o registro, validava TLS e emitia o envelope mudou para `vps::edit`, onde `EditRequest` nomeia toda entrada e `apply_fields` é puro afora a checagem de existência do caminho da chave
- `cli/mod.rs` perde a política de dry-run e a árvore de comandos: As duas foram para `cli/dry_run.rs` e `cli/discovery.rs`
- O arquivo está num orçamento congelado, e arquivo de orçamento que só cresce deixa de significar algo
- BREAKING para quem usa a biblioteca, `SshCliError::Unavailable` carrega um enum, não uma string: A variante agora é `Unavailable { service: UnavailableService }` e o construtor é `SshCliError::unavailable(UnavailableService)`
- `UnavailableService` é `#[non_exhaustive]` com `Keyring` e `ConfigLock`
- Uma string livre deixava o keyring e o lock do config dividirem uma variante significando coisas diferentes, e deixou a falha de lock da 0.5.6 sair com dica sobre keyring
- O enum obriga cada chamador a nomear a própria causa e deixa a dica acompanhá-la
- O texto da wire não muda (`service unavailable: config_lock`), então consumidores de CLI e JSON não veem nada
- Migração: troque o argumento string pelo valor do enum e acrescente um braço curinga onde a variante é comparada
- BREAKING para wrappers de release e quem chama os gates locais, os nove `scripts/*.sh` agora são `scripts/*.py`: `check_all_gates`, `check_advisory_freshness`, `check_cross_targets`, `check_en_identifiers`, `verify_install_resolve`, `generate_sbom`, `dist_multiarch`, `release_attest` e `e2e_real_ssh` viraram programas Python 3.9+ que usam só a biblioteca padrão, com os auxiliares `_gatekit.py`, `_e2e_sshd.py` e `_e2e_cases.py` ao lado
- No Windows rodam como `py -3 scripts/<name>.py`
- Script de shell roda em Linux e macOS e em mais nenhum lugar, e este projeto entrega alvo Windows
- As variáveis de ambiente viraram flags: `CHECK_TIMEOUT_SECS` é `--timeout-secs` (aceita também pelos scripts que chamam cargo), `TARGETS`, `OUT_DIR` e `PARALLEL_JOBS` são `--targets`, `--out-dir` e `--jobs`, `REQUIRE_COSIGN` é `--require-cosign`, `CARGO_TARGET_DIR_INSTALL_GATE` é `--target-dir`, e `check_advisory_freshness` aceita `--db-root`
- `e2e_real_ssh` aceita `--host --port --user --password-file --sudo-password-file --from-grok-config [PATH] --bin --local-sshd` e não lê mais `SSH_CLI_E2E_*` nem `--config-dir`
- `--from-grok-config` exige Python 3.11+, e `--local-sshd` é só Unix e no Windows falha com `local_sshd=requested_but_unusable`
- Migração: `bash scripts/<name>.sh` vira `python3 scripts/<name>.py`
- Os antigos `.sh` foram apagados (ver `Removido`)
- A proveniência do `--version` lê um arquivo carimbado, e o `build.rs` para de reexecutar: `.commit_hash` agora guarda `<short hash> <version>`, e a ordem de resolução é env, depois git, depois o arquivo, depois `unknown`
- Sem env e sem git o arquivo só é exato se a versão for a do pacote, então um hash esquecido de uma release anterior não pode mais ser reportado como o atual
- Um arquivo gravado para outra versão sai como `<hash>-dirty`, porque a troca de versão prova que a árvore mudou depois daquele commit
- O `build.rs` não reexecuta mais a cada build quando não há `.git`
- Até o passo de refresh do `docs/RELEASE_CHECKLIST.pt-BR.md` rodar, a 0.5.6 imprime `08462e2-dirty`: o commit base desta árvore mais as mudanças locais, porque o commit de release ainda não existe
- BREAKING para scripts e agentes, `vps edit` troca o método de autenticação: `--key` limpa `password`, `use_agent` e `agent_socket`
- `--password` e `--password-stdin` limpam `key_path`, `key_passphrase`, `use_agent` e `agent_socket`
- `--use-agent` limpa `password`, `key_path` e `key_passphrase`
- Antes, as mesmas chamadas saíam 64 com `primary auth methods are mutually exclusive`, e o único caminho de senha para chave era passar antes por `--use-agent`
- Código que dependia dessa recusa agora recebe registro alterado e exit 0
- `updated` lista o que a troca de autenticação limpou: Só aparecem campos que tinham valor, depois do método novo, então `vps edit prod --key K` num host de senha informa `key_path` e `password`
- `--use-agent` segue a mesma regra
- Outra `--key` descarta a passphrase antiga: Quando a chamada não traz passphrase nova, `key_passphrase` é limpa e entra em `updated` se existia
- O mesmo caminho de chave não muda nada
- `--agent-socket` sozinho implica `--use-agent`: Ele limpa `password`, `key_path` e `key_passphrase`, e `updated` lista os que tinham valor
- Passphrase exige chave privada: `--key-passphrase` ou `--key-passphrase-stdin` num host que termina sem chave sai 64 (`a key passphrase needs a private key`) e deixa o registro idêntico byte a byte
- Flags de autenticação em conflito na mesma chamada saem 2 no parse: `--key` com `--password*`, `--agent-socket` com `--key`, `--password*` ou flag de passphrase, e flag de passphrase com `--use-agent` ou `--password*` são recusadas pelo clap antes de qualquer trabalho
- Antes chegavam ao validador e saíam 64 depois
- `vps edit X --use-agent --agent-socket S` continua válido, e `vps add` não mudou
- BREAKING para hosts que você cria, `VpsName::try_new` é estrito: `vps add`, o destino de `vps rename` e `vps edit --name`, e `vps import` recusam nomes fora de `[A-Za-z0-9._-]` (ver `Segurança`), onde espaço, acento e caractere de controle passavam
- A leitura continua leniente por `VpsName::from_stored`: NFC, não vazio, no máximo 255 bytes, sem caractere de controle (`char::is_control`) e sem os caracteres de formatação ou bidi U+00AD, U+061C, U+180E, U+200B a U+200F, U+202A a U+202E, U+2060 a U+2064, U+2066 a U+206F, U+FEFF e U+E0000 a U+E007F (U+2065 não é atribuído e fica de fora)
- Um host legado com espaço ou acento no nome ainda carrega, lista, mostra, resolve e pode ser renomeado para um nome estrito
- `vps import` lista todos os nomes ruins de uma vez: Ele valida todos os itens primeiro e sai 64 com `invalid VPS names in import (N): 'a', 'b'; a name is 1..=255 ASCII letters, digits, '.', '_' or '-' and starts with a letter or digit; nothing was imported`
- Nada é gravado, e o `--dry-run` dá o mesmo erro
- Um host legado com nome fora do charset estrito pode ser exportado, mas o export só restaura depois de `vps rename` dar a ele um nome estrito
- Um `config.toml` legado nunca trava a CLI: Um nome guardado com caractere de controle, de formatação ou bidi carrega
- O `doctor` o lista escapado em `local.invalid_host_names`, `list`, `show`, `remove` e `edit` o escapam em modo texto, e `vps rename '<legacy name>' <new>` o repara porque a origem não é validada
- Buscas que viram alvo recusam caractere de controle com exit 64: `exec`, `health-check`, `--hosts` e `connect` rejeitam esse nome como erro de uso, e o `connect` não grava marcador
- `show`, `remove`, `rename` e `edit` com origem inexistente continuam saindo 66
- `vps edit`, `vps rename` e `vps remove` ecoam o alvo no `--dry-run`: O plano carrega `target_resolved`, `target_source` (`argv`), `host_resolved` e `host_source` depois que a busca dá certo, e o `dry-run.schema.json` os lista
- Busca que falha continua sem alvo
- Registro inexistente sai 66 e não cria nada: `vps edit`, `vps rename` e `vps remove`, reais ou com `--dry-run`, num config que não existe não criam diretório nem arquivo de lock
- O mapa do config usa o nome NFC do registro como chave: Duas tabelas que normalizam para o mesmo nome saem 65 com `config has two hosts named ...` (nomes escapados), e nada se perde em silêncio
- `--config-dir something.toml` faz do pai o diretório de config para tudo: `secrets.key`, `known_hosts`, `active`, `lang` e os arquivos de TLS acompanham o `config.toml` ali
- `vps export -o new/deep/e.json` cria os diretórios que faltam com `0700`: Diretório existente nunca é alterado
- BREAKING para quem usa a biblioteca, `paths::validate_and_normalize` devolve `String`: Antes devolvia `VpsName`
- Nessa função fica só a regra de nome de arquivo (o caminho mTLS não mudou), e a regra de nome de host mora em `VpsName::try_new`
- Migração: use a `String` e chame `VpsName::try_new` onde o valor é nome de host
- O gate `test` roda `cargo test --locked --all-features --no-fail-fast`: Um binário de teste vermelho não esconde mais os outros (`scripts/check_all_gates.py`)
- Testes usam diretório temporário e três hosts: `apply_tests.rs` usa `tempfile`, `gap_doc_003` lê o schema com `serde_json`, o snapshot de `vps_not_found` foi regravado, e o roundtrip de import cobre três hosts (um com senha, um com chave, um com agente)

### Removido
- `serde_with` sai das dependências porque nada o usava, e 14 crates saíram do `Cargo.lock` nesta rodada de atualizações
- `package.homepage` sai do `Cargo.toml`, porque o cargo 1.100 o acusava como redundante
- Sem config de runner hospedado na tree, escrito sem nomear o fornecedor
- BREAKING para quem usa a biblioteca, `Money`, `Brl`, `Usd`, `Currency` e a dependência `rust_decimal` saíram: `src/domain/money.rs` não existe mais
- O produto nunca tratou dinheiro, então nada na CLI nem no JSON muda
- Migração: apague os imports, ou copie o módulo para o seu crate
- Os nove `scripts/*.sh`, 1236 linhas, foram apagados: Os programas `.py` de mesmo nome são os únicos pontos de entrada (ver `Alterado`)
- Script de shell roda em Linux e macOS e em mais nenhum lugar, e este projeto entrega alvo Windows
- O arquivo solto `json` na raiz do crate foi apagado: Era uma saída esquecida
- As suítes `tests/gaps_v052_tls_policy.rs`, `tests/gaps_v056_ssh.rs`, `tests/gaps_v062_cross_platform.rs` e `tests/gaps_v073_package_hygiene.rs` e o módulo de teste de `src/tls/provider.rs` foram apagados por decisão do dono

### Corrigido
- `scripts/check_advisory_hits.py` lia `advisory.patched_versions`, enquanto o `cargo audit` publica `versions.patched` no nível da entrada, então o gate dizia "no patched release" para advisory que tinha correção
- `use_keyring_is_wired_and_fails_closed` em `tests/gaps_v074_flag_reach.rs` e o caso E2E E21 em `scripts/_e2e_cases.py` deixavam uma entrada `secrets-primary-key:<digest>` no cofre do SO a cada execução
- O teste agora limpa com `set_config_dir` mais `delete_key_from_keyring`, e o E21 roda `secrets init --force` de volta ao arquivo, que apaga a entrada pelo `src/secrets/init_key.rs`
- As 7 entradas órfãs que essas execuções deixaram no cofre do SO foram apagadas
- `impl Display for SshPort` usava `write!` e ignorava a largura, então a coluna PORT do `vps list` em texto saía desalinhada, e agora delega a `u16`
- O `vps list` em texto mede cada coluna pela célula mais larga em vez das larguras fixas 20/30/6/15, que um usuário de 17 caracteres tirava do alinhamento, e nenhuma linha termina com espaço
- `vps show` em texto imprimia `Name:`, `Port:`, `User:` e `Added at:` com um espaço a mais, e os 22 rótulos agora alinham
- O cargo 1.100 acusava `human-panic` como sem uso em build debug, porque o `cfg(not(debug_assertions))` era redundante: o human-panic 2.0.8 `src/panic.rs:60` já é no-op em debug, então o cfg saiu
- O Gap 6 provava a própria contagem com um `43` escrito à mão que envelheceu para `53`, então a prova agora deriva o número em vez de afirmá-lo
- A prova do Gap 31 prometia zero termo proibido enquanto `CHANGELOG.md` e `CHANGELOG.pt-BR.md` somavam quatro, e os dois arquivos ficavam fora da varredura, mas os termos saíram e a prova agora os cobre
- O Gap 49 registrava um `sha256` de outro estado da fonte, então a prova agora deriva o hash do artefato instalado
- O `docs/COOKBOOK.md` ainda chamava um número intermediário não publicado de linha de produto corrente, e o binário que gerou as saídas é o 0.5.6, então a linha e a versão agora dizem o que são
- `CHANGELOG.md`, `CHANGELOG.pt-BR.md` e `skills/ssh-cli-pt/SKILL.md` traziam três desvios `h2-spacing` que o gate novo pegou na primeira execução, porque as seções de release ficaram a uma linha em branco
- Os Limites declarados do `gaps.md` contradiziam o Gap 39 sobre os exits de sinal do túnel, e os dois agora dizem os 130 e 143 medidos
- Os Gaps 39 e 42 registravam a matriz E2E como E01 a E18 enquanto ela roda E01 a E21 desde que a matriz cresceu
- O `.commit_hash` ficou parado em `a691fc7 0.5.5`, e o passo de refresh do `docs/RELEASE_CHECKLIST.pt-BR.md` o carimba com o release a que pertence
- `vps list --select name --json` devolvia `[{"name":"a"}]`, um item que não satisfazia nenhuma forma do schema publicado da lista, porque o relatório de redução de array puro vai para o stderr e nunca vira campo do item
- A suíte de conformidade de schema validava só a lista sem modelagem, e por isso o desvio era invisível
- A suíte agora valida as duas formas
- `a_document_without_the_inventory_points_at_one` aceitava qualquer menção a `HOW_TO_USE`, `README.md`, `llms-full.txt` ou `commands --json` como substituto do inventário de flags, e foi assim que os quatro documentos acima saíram incompletos e verdes
- Oito testes de flag afirmavam só que o parser não recusava: `--fail-fast`, `--use-keyring`, `--secrets-key-file`, `--replace-host-key`, `--scp-file-concurrency`, `--description`, `--contact` com `--staging` e `--keyring` no `secrets init`
- `--description` com quebra de linha podia encerrar o comentário de shell e transformar o resto do texto num segundo comando remoto
- A sanitização de quebra de linha agora é asseverada direto em `src/ssh/packing.rs`
- `--scp-file-concurrency` instalava o valor uma vez por `OnceLock`, então o valor que a janela de lote lia não podia ser medido
- Agora é um atômico, com clamp no instalador
- `ssh-cli --use-keyring secrets init --json` respondia `key_source: xdg_file`, porque o subcomando recebia só o `--keyring` do leaf e a flag global nunca chegava ao caminho de gravação
- O caminho de init agora lê o estado de runtime que a dispatch já instala, então as duas flags alcançam o mesmo cofre
- Um payload de objeto único que ignorava flag de coleção agora a nomeia em `agent_shape.ignored` em vez de aceitá-la em silêncio, inclusive quando essa flag é a única passada: `ssh-cli doctor --limit 2` devolvia o corpo idêntico ao de uma execução sem modelagem, porque o retorno antecipado disparava sempre que nem `--select` nem `--truncate-content` estavam presentes
- `vps-show` e `vps-doctor` casam as duas formas por `anyOf` e não por `oneOf`: o `--truncate-content` e as flags de coleção mantêm o registro inteiro, então as duas variantes descreviam o mesmo corpo e um `oneOf` teria reprovado a saída real
- `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` e `--max-output-bytes` seguem inertes em objeto único de propósito, porque não há coleção a reduzir, e o relatório diz isso flag por flag
- A autenticação por agente não desaparece mais no export e import: `MaskedVpsJson`, `ExportHostJson` e `ImportHostEntry` não tinham `use_agent` nem `agent_socket`, e `into_record` nunca os definia, então um host que autenticava pelo agente voltava do ciclo como host sem senha, sem chave e sem agente, um que sequer conecta
- Medido no wire antes da correção: o export de um host de agente não carregava vestígio do agente
- O lock do config não pendura mais: `flock` bloqueia sem limite por padrão, então um vizinho segurando `config.toml.lock` fazia todo outro one-shot esperar para sempre: sem saída, sem erro, sem exit
- Agora ele retenta com backoff full-jitter até `--lock-wait-ms` expirar e então devolve `Unavailable` (exit 69, retentável)
- Sinal recebido durante a espera vence o prazo e sai 130/143, porque o operador pediu para parar e um código retentável convidaria a retentar
- `vps import` toma o lock antes de carregar: Ele carregava o registro sem lock e salvava com lock, então um import que commitasse entre o load e o save de um vizinho descartava o host do vizinho
- O guard agora cobre load → merge → save, a mesma forma que o resto do CRUD segue
- `--dry-run` em host inexistente ainda sai 66: A pré-condição vence a prévia nos dois verbos novos, então um ensaio nunca reporta plano de operação que falharia
- Edição que não muda nada é no-op idempotente: Ela sai 0, emite `vps-edited` com `updated: []` e não regrava o `config.toml`, então não gasta arquivo temporário, fsync nem rename
- O exit 64 fica reservado para a invocação sem nenhuma flag de campo, rename ou limpeza, que é o único caso em que o chamador não pediu nada
- Um agente que repete a edição depois de uma resposta perdida agora recebe o mesmo sucesso, em vez de um erro que parece falha
- As pré-condições do rename têm ordem fixa: `vps rename ghost ghost` sai 66, porque "o host existe" é checado antes de "os nomes diferem"
- Nome idêntico de host que existe sai 64 e o erro carrega `target_resolved`, já que a essa altura um host foi escolhido
- O rename não deixa o marcador apontando para um nome que não existe mais: O marcador `active` é gravado primeiro e o `config.toml` depois
- Se o save falhar o marcador antigo é restaurado, e se a gravação do marcador falhar nada é renomeado
- Um crash entre os dois passos deixa marcador órfão, que é fail-closed: `--use-active` sai 66 em vez de agir no host errado, e `vps list` mostra o estado real
- O lock do config reage a sinal em até 100 ms, nasce `0600` e a dica dele não engana mais: A espera roda em fatias de 100 ms que checam o sinal, então SIGINT e SIGTERM saem 130 e 143 em até uma fatia
- Antes, um sleep bloqueado podia segurá-los por cerca de 5 s. `config.toml.lock` nasce com modo 0600 e a gravação do marcador faz fsync do diretório
- A dica do exit 69 `unavailable` agora manda repetir o argv ou subir `--lock-wait-ms` e não fala mais do keyring, que não tem relação com essa falha
- `VpsNotFound` (exit 66) diz que um rename ou remove anterior pode ter dado certo: A dica manda o chamador conferir `vps list`
- `vps rename` não é idempotente: repetir depois de sucesso sai 66, porque o nome antigo sumiu, e `vps show <NEW>` é como confirmar o resultado
- O modo texto não engole mais erro de escrita em stdout: Todo verbo agora o propaga: pipe quebrado sai 141 e disco cheio vira erro de I/O, onde antes os dois saíam 0 depois de a saída se perder
- Em `exec` com stdout fechado o erro de escrita vence a falha do comando remoto, porque o chamador não consegue ler o resultado de um comando cuja saída não pôde receber
- `vps import` valida TLS como o `vps edit`: Certificado sem chave sai 64 na hora do import, em vez de produzir um registro que falha depois, na conexão
- Os dois caminhos compartilhavam uma regra e só um a aplicava
- Um host não conseguia passar de senha para chave: `vps edit --key` num host de senha, e `--password` num host de chave, saíam 64 como mutuamente exclusivos
- A edição agora troca o método e limpa os segredos antigos (ver `Alterado`)
- Um nome de host com caracteres de controle podia ser criado e impresso cru: O charset estrito o recusa na criação, e o modo texto escapa o que já estiver guardado
- ESC e BEL chegavam crus ao terminal no modo texto: Mensagens, erros, `list`, `show`, `health-check`, `doctor` e as saídas em lote agora os escapam
- O diretório de config nascia `0755`: Agora nasce `0700`, e o `doctor` informa o diretório existente que estiver exposto
- `locale set --config-dir new/config.toml` criava um diretório chamado `config.toml`: Quando o caminho termina num arquivo `.toml` que não existe, o diretório criado é o pai
- Segredo cifrado atrás de um `--config-dir` `.toml` saía 74: O `secrets.key` era gravado num diretório com o nome do arquivo, e a execução deixava esse diretório para trás
- Agora o diretório é o pai
- `known_hosts.lock` e `lang` nasciam com o umask: Os dois nascem `0600`, sem janela
- Testes vermelhos ficaram verdes: Cinco testes estavam defasados em relação ao código: `snapshot_error_vps_not_found` (dica antiga), `gap_doc_003_residual_behaviors_documentados` (fatiava o texto do schema), `every_suite_on_disk_is_catalogued_in_testing` (o `docs/TESTING.md` não tinha cinco suítes), e `json_export_then_import_round_trips_every_field` mais `toml_export_then_import_round_trips_every_field` (editavam um host de senha com `--key`)
- O `clippy::manual_find` em `docs_conformance` foi corrigido, e o `cargo fmt --check` voltou a passar
- `apply_tests.rs` não escreve mais num caminho fixo em `/tmp`: Execuções em paralelo podiam colidir nele
- Agora usa `tempfile`
- O `cargo audit` não acusa mais o RUSTSEC-2026-0235: O `rkyv` só entrava no lock pelo `rust_decimal` opcional, e os dois saíram juntos
- `docs/TESTING.md` e `docs/TESTING.pt-BR.md` listam toda suíte do disco: Eles agora catalogam as cinco suítes `gaps_v072_*` que faltavam e as quatro suítes novas `gaps_v073_*`, e o `every_suite_on_disk_is_catalogued_in_testing` falha quando uma suíte fica de fora
- As duas skills do repositório recuperaram quatro regras de segurança que o corte de 4000 palavras tinha removido: O passo SANITIZE, a regra do `sudo` cru, a regra do TOFU e a forma persistente do `--disable-sudo` voltaram, pagas pelo corte de fórmuulas duplicadas
- A cópia instalada da skill de agente estava defasada: Era uma cópia antiga com 47 folhas que não conhecia `vps rename`, `vps edit --name`, `--lock-wait-ms`, as flags de limpeza nem a troca de autenticação, então foi substituída pelo texto atual com 48 folhas
- O `cargo clippy --lib --no-default-features -- -D warnings` volta a passar: Um `needless_return` em `src/cli/dispatch.rs` foi removido
- Documentos que ensinavam o comportamento errado foram corrigidos depois de medir o binário: O domínio do ACME é `--domain <DOMAIN>` e nunca posicional, então `tls acme issue example.com` sai 2
- O `vps export` redigido emite `password` vazio e null nos outros segredos: A máscara de três asteriscos aparece só em `vps list` e `vps show`
- Os resultados de `health-check --all` carregam `status` com `ok` ou `error`, então o filtro é `--filter status!=ok`
- Outras afirmações corrigidas: o `rustls` é permitido de propósito na lista de release, o macOS arm64 não é coberto por nenhum gate, e os textos sobre parse só do stdout, `--count-only` em lote, o formato do `exec --step`, `--no-input` com `--*-stdin` e o alcance do `--disable-sudo` agora batem com o binário
- Comentários de código que descreviam uma máscara no export foram corrigidos
- Os documentos publicados não carregam mais uma contagem de gaps velha: Eles dizem quais classes de gap a release fecha, e todo gap desta release está resolvido, exceto a tentativa de criptografia em Rust puro, que foi tentada e revertida
- Os documentos publicados não apontam mais para o `gaps.md` não publicado: `llms.txt`, `llms-full.txt`, `llms.pt-BR.txt` e os dois INTEGRATIONS pararam de linkar ou citar o arquivo
- Erros de uso estão documentados como texto puro: O exit 2 do parser de argumentos e o exit 64 de um `--filter` malformado imprimem texto puro no stderr mesmo com `--json`, e só erros de execução trazem o envelope
- O `vps-export.schema.json` descreve a saída real de `vps export -o`: Um `oneOf` aceita o corpo (`schema_version` e `hosts`) ou o resumo (`path` e `format`)
- O histórico do changelog está alinhado entre os dois idiomas: O cabeçalho em inglês colado dentro da seção 0.5.2 em português e uma seção vazia saíram, os títulos estão no idioma de cada arquivo, e toda versão tem o mesmo número de bullets nos dois arquivos, sem mudar fato de entrada publicada
- Os travessões fora de títulos H3 saíram dos documentos publicados: Viraram dois-pontos ou vírgulas com as palavras intactas, e o linter de formatação agora os conta
- O `vps import --file` com arquivo ausente sai 66 `file_not_found` como erro permanente: Ele saía 74 `io` com `retryable: true`, o que mandava o agente para um laço de retentativa de um caminho que não vai aparecer
- Catorze schemas foram realinhados com a saída que o binário emite: `dry-run`, `error-envelope`, `exec`, `sudo-exec`, `su-exec`, `exec-batch`, `vps-edit`, `vps-rename`, `secrets-init`, `secrets-reencrypt`, `vps-doctor`, `sftp-fs-op`, `tunnel-closed` e `health-check-batch`
- O `vps list` em modo texto mostra `(not set)` na coluna PASSWORD de host só com chave: Ele imprimia um valor mascarado para uma senha que o host não tem
- Os inventários de comandos em `docs/` citam a invocação completa: HOW_TO_USE, AGENTS, COOKBOOK e CROSS_PLATFORM listavam 28, 19, 20 e 43 folhas como um `vps add` solto, e o teste de cobertura agora exige `ssh-cli vps add` em um code span
- O comportamento documentado que o binário contradizia foi corrigido: um `--disable-sudo` global também persiste antes de `vps add` e `vps edit`, um `tunnel` parado por sinal sai 130 ou 143, só as duas variáveis `SSH_CLI_*` de segredo são recusadas, o exit 69 cobre também o lock do registro, e o prefixo gravado é `sshcli-enc:v2:`
- Uma variável de ambiente de chave de segredo é recusada como erro de uso em todo comando que lê ou grava um segredo guardado: `vps list`, `vps show`, `vps export`, `doctor` e `vps add` com `SSH_CLI_SECRETS_KEY` ou `SSH_CLI_SECRETS_KEY_FILE` saíam 65 `config` com uma mensagem que culpava o TOML, porque o serde achatava o erro tipado e o carregador o embrulhava de novo

### Interno
- `Cargo.toml` declara `keyring = { version = "4.2.0", features = ["v1"] }` em vez de confiar num default que não existe, e o comentário de MSRV que ainda dizia 1.85 foi corrigido
- `tests/gaps_v074_flag_reach.rs` troca o teste de ponteiro por contagem: oito documentos em dois idiomas são lidos contra as 89 flags que `commands --json` publica, e a falta nomeia as flags ausentes por arquivo
- `tests/gaps_v063_e2e_harness.rs` fixa a matriz em 21 casos, e `scripts/_e2e_cases.py` agenda os três novos
- Os testes de keyring limpam por `delete_key_from_keyring`, e o único teste que ainda vazava uma entrada está listado em `Corrigido`
- O `.commit_hash` defasado (`a691fc7 0.5.5`) é declarado em vez de atualizado, porque não há metadados de git neste diretório e rodar git não foi pedido
- O `--version` portanto mostra o commit base com `-dirty`, e não um hash de release
- `src/agent_shape.rs` passou os caminhos de objeto único para `src/agent_shape_record.rs`, então os dois arquivos ficam sob o orçamento de 600 linhas por componente
- `tests/gaps_v074_flag_reach.rs` acrescenta 21 asserções sobre as 18 flags que não tinham teste de comportamento e sobre os documentos que afirmam o inventário completo
- `tests/docs_conformance.rs` acrescenta a cobertura de exemplos e o teste de parse das 48 folhas
- O validador de conformidade de schema agora exige `pattern`, keyword que cinco schemas publicados já carregavam sem checagem, e o `dev-dependencies` ganha a crate `regex` para isso
- Nova dependência direta `rustix` (features `process` e `std`, só Unix): Ela já estava no lock por causa do `tempfile`
- `src/output/sanitize.rs` e `src/vps/edit/auth.rs` são módulos novos: O primeiro cuida do escape de texto, o segundo cuida da troca de autenticação e da lista `updated`
- O `gaps.md` foi reescrito como o registro dos gaps da 0.5.6: Cada gap tem problema, consequências, causa raiz, solução, benefícios e prova, e o arquivo fica local porque não é publicado
- O `exclude` do `Cargo.toml` agora lista `/json`, `*.sh`, `__pycache__/`, `*.pyc` e `*.snap.new`, e o `cargo package --list` não mostra nenhum deles
- Arquivos foram divididos por responsabilidade: `src/cli/help_text.rs`, `src/cli/tunnel_mode.rs`, `src/cli/process_flags.rs`, `src/cli/locale_cmd.rs`, `src/i18n/error_text.rs`, `src/secrets/tests.rs`, `src/vps/model/tests.rs` e `src/vps/config_io/tests.rs` são novos
- O ledger de tamanho encolheu: `secrets.rs`, `cli/mod.rs`, `vps/model.rs` e `cli/dispatch.rs` agora medem menos que o orçamento de 600 linhas e saíram do ledger
- `FULL_INVENTORY_DOCS` em `tests/docs_conformance.rs` também lista `llms.txt`, `llms.pt-BR.txt`, os dois INTEGRATIONS e os dois COOKBOOK: o `every_command_appears_in_every_full_inventory_document` agora segura as 48 folhas em 15 arquivos, e achou `sftp download` faltando no COOKBOOK
- A migração para criptografia em Rust puro foi tentada e revertida: o provider TLS continua `aws_lc_rs`, e o `aws-lc-sys` ainda exige compilador C no build (a tentativa e o motivo medido ficam no registro local de auditoria)


## [0.5.5] - 2026-08-10

### Segurança
- Texto traduzido não chega mais ao JSON: Dezenove sites em SCP e SFTP construíam a falha a partir de `i18n::t(…)`, como corpo de `SshCliError::InvalidArgument` ou direto no campo `error` de uma entrada de lote, então sob `--lang pt-BR` o envelope carregava português
- O `docs/schemas/error-envelope.schema.json` declara que `message` é "sempre inglês, independente de `--lang`", porque um agente que ramifica sobre prosa que muda com o locale do operador não tem discriminador estável nenhum
- Dois desses sites escreviam em stdout: um `scp --all` cancelado punha `Operação cancelada pelo usuário.` no payload do lote
- As três frases agora vivem em `crate::constants` como `&str` simples, e `Message::OperationCancelled`, `ScpUploadFileOnly` e `ScpDownloadLocalNotDirectory` foram removidas: uma variante de `Message` é a promessa de que a string pode ser localizada com segurança, e para essas três isso nunca foi verdade
- O rótulo humano em volta de cada corpo continua traduzido
- O gate existente conferia o emissor, que não fazia nada errado
- O novo confere os construtores, porque um invariante que pode ser quebrado antes do posto de checagem precisa ser testado na origem
- BREAKING, `ssh_cli::telemetry` agora é `ssh_cli::tracing_setup`: O módulo nunca emitiu telemetria, a própria documentação dele descarta OTLP, backends de métrica e arquivos de log desde o primeiro commit, mas quem auditasse a árvore contra a proibição de telemetria deste projeto encontrava um módulo com esse nome e tinha de lê-lo para saber que era inocente
- Um nome que custa uma verificação a cada leitor está errado, por mais defensável que seja a história dele
- Nenhum alias depreciado foi criado: o alias preservaria justamente o nome que o renome existe para remover
- O campo de wire `"telemetry": false` do `vps doctor` mantém o nome, onde a palavra é a alegação que está sendo negada
- BREAKING, `health-check` agora exige alvo explícito: Ele testava o marcador ativo sempre que o nome era omitido, defendido como seguro porque um probe de conectividade é leitura idempotente
- A defesa é verdadeira sobre o probe e falsa sobre o hábito: um operador que aprende que o alvo é opcional na superfície de leitura carrega essa expectativa para o `exec`, que é onde o GAP-SSH-EXEC-ARGC-001 custou um dia de trabalho
- Assimetria de ergonomia entre leitura e escrita é armadilha, e a leitura é a metade mais barata de tornar estrita
- A taxonomia de exit passa a ser idêntica nas duas famílias: 64 significa que nenhum alvo foi designado, 66 significa que o alvo designado não existe
- Isso estreita o contrato antigo do `GAP-SSH-EXIT-002`, que mapeava `health-check` sem nome para 66 e assim confundia "você não nomeou nada" com "o que você nomeou não existe"
- Migração: `ssh-cli health-check` passa a ser `ssh-cli health-check --use-active` ou, preferencialmente, `ssh-cli health-check <VPS>`
- GAP-SSH-EXEC-ARGC-001 regra 6, exit 127 no passo zero agora aborta o lote: Esse código no primeiro passo é a impressão digital de um nome de host promovido a comando: o shell responde `command not found` e, antes disto, todos os `--step` restantes rodavam contra uma máquina que ninguém designou enquanto o exit agregado permanecia 0. O aborto é escopado ao índice zero
- Um 127 em qualquer índice posterior mantém o contrato de executar tudo, porque a essa altura a sessão já provou estar apontada onde o chamador acredita
- Defesa em profundidade: o argv que produziu isso agora é rejeitado no parse, e esta é a segunda tranca da mesma porta
- BREAKING, GAP-SSH-EXEC-ARGC-001: a família exec podia executar num host que o chamador nunca nomeou: `exec`, `sudo-exec` e `su-exec` declaravam `num_args = 1..=2` num único vetor posicional e usavam a contagem para decidir significado: dois posicionais eram `VPS COMANDO`, um era `COMANDO` contra o host que o marcador `active` em disco guardasse
- `--step` é `ArgAction::Append` e portanto nunca contou como posicional, então `exec <HOST> --step <CMD>` colapsava no ramo de um posicional: o nome do host era executado como binário remoto numa máquina diferente, o passo zero devolvia 127, e todos os `--step` seguintes rodavam ali enquanto o exit agregado do processo permanecia 0. Em 2026-08-09 isso colocou uma configuração completa de auto-clean, com units e timers systemd, no servidor errado
- O host pretendido nunca foi tocado e o operador foi informado de sucesso
- A classe é CWE-441 (Unintended Proxy or Intermediary, "Confused Deputy"): a CLI detinha uma autoridade que o chamador não via sendo exercida, e a gastou num alvo tirado do estado ambiente em vez da requisição
- Nomear a classe importa porque a correção generaliza a partir dela: toda superfície que age sobre um host deve tomar esse host do argv, nunca do ambiente
- A própria documentação de `Arg::num_args` do `clap` alerta para essa forma: um posicional com range continua consumindo valores "até encontrar outra flag ou opção"
- A aridade deixou de carregar significado
- O modo passa a ser declarado por flag: `<VPS> <COMANDO>` para um host, um seletor mais um posicional para frota, e a nova `--use-active <COMANDO>` para alcançar o marcador de forma deliberada
- Um único posicional sem flag agora é erro de uso (exit 64): era o atalho perigoso, e não resta argv que alcance um host que o chamador não designou
- Uma segunda guarda rejeita `--use-active <nome-registrado>`, que é o mesmo engano uma flag adiante
- O caminho de frota já falhava fechado
- Isto traz o caminho single-host à paridade
- Migração: `ssh-cli exec 'uptime'` passa a ser `ssh-cli exec --use-active 'uptime'` ou, preferencialmente, `ssh-cli exec <VPS> 'uptime'`
- A1: `russh` subiu de 0.62.2 para 0.62.5
- A CVE-2026-68930 (GHSA-m65r-rprj-r5rg, publicada em 2026-08-03) atinge `<= 0.62.4`: callbacks de canal são despachados para IDs de canal nunca abertos
- O defeito é do lado servidor e esta CLI é cliente puro: não há uma única referência a `russh::server` em `src/`, então o impacto alcançável aqui era nulo
- O pin subiu mesmo assim, porque `cargo deny check advisories` devolveu zero achados com o banco de advisories sincronizado no mesmo dia
- Gate de supply chain verde não é prova de que nenhuma CVE publicada se aplica
- A3: A pilha de criptografia at-rest deixou de ser duplicada
- `chacha20poly1305` 0.10 puxava `aead` 0.5 / `chacha20` 0.9 / `poly1305` 0.8 enquanto russh e aws-lc-rs resolviam a geração 0.6 / 0.10 / 0.9, ou seja, duas implementações independentes de ChaCha20-Poly1305 compiladas num binário de segurança
- Subir `chacha20poly1305` para 0.11, `getrandom` para 0.3, `toml` para 1 e `windows-sys` para 0.61 reduziu 41 versões duplicadas para 5, todas transitivas e justificadas uma a uma no `deny.toml`
- `deny.toml` mudou `multiple-versions` de `warn` para `deny`
- O aviso era relevante e ninguém lia
- Toda duplicata remanescente está enumerada com motivo, então uma duplicata nova: justamente a que dá para corrigir, reprova o gate
- A6: `tls mtls import` lia caminhos PEM fornecidos pelo operador com `fs::read` sem limite
- As duas leituras passam a ser limitadas por `MAX_PEM_FILE_BYTES` (1 MiB) via o novo `paths::read_bytes_capped`

### Adicionado
- GAP-SSH-EXEC-ENVELOPE-002 concluído, o envelope de erro também nomeia o host: O trabalho da 0.5.5 acrescentou os campos de auditoria ao envelope de sucesso
- O caminho de erro, onde a informação mais importa, não os carregava
- Medido: uma falha alcançada por argv e outra alcançada pelo marcador produziam envelopes byte-idênticos, então o desvio permanecia invisível no único canal que um agente de fato lê
- `target_resolved`, `target_source` e `active_fallback`: mais seus aliases de leitura `host_*`, agora aparecem também no envelope de erro, e ficam ausentes em vez de vazios quando a falha antecede a resolução do alvo, `vps_not_found` é esse caso
- Uma string vazia afirmaria que um host foi escolhido e não tem nome
- A ausência diz que nenhum host foi escolhido, e só uma dessas coisas é verdade
- GAP-SSH-EXEC-ENVELOPE-002, o envelope single-host agora nomeia o próprio alvo: `ExecutionJson` carregava `stdout`, `stderr`, `exit_code`, as flags de truncamento e `duration_ms`, e nada que identificasse o host
- `ExecHostJson` e `ExecBatchJson`, o caminho de frota, sempre carregaram `name`, então a assimetria corria ao contrário do risco: o único caminho que aceitava fallback silencioso era o que não podia ser auditado pelo stdout
- Detectar uma execução desviada exigia gastar um round-trip SSH extra num comando-crachá `hostname -f`, que é opcional e portanto é pulado justamente quando há pressa
- O envelope passa a carregar `target_resolved`, `target_source` (`argv` | `active_marker` | `selector`) e `active_fallback`, com `host_resolved` e `host_source` ao lado como aliases de leitura: sempre no caminho de sucesso, e no de erro sempre que a falha ocorrer depois de o host ter sido resolvido, então um desvio vira comparação de string em vez de investigação
- Os três são aditivos com defaults de serde, então eventos escritos por versões anteriores continuam desserializando
- `health-check` reporta `host_source` pelo mesmo motivo: nomear um host sem nomear a proveniência não distingue uma verificação deliberada de uma herdada
- Essa superfície foi defendida aqui como lugar legítimo para herdar o marcador, porque um probe é leitura idempotente
- A entrada BREAKING acima revoga essa defesa dentro desta mesma release, e o campo é o que torna o novo opt-in verificável em vez de apenas alegado
- GAP-SSH-DOCTOR-ACTIVE, `doctor` reporta o host ativo, não apenas o caminho até ele: `local.active_file` dava o caminho do marcador e nenhum comando da CLI expunha o conteúdo, então um agente tinha de ler o arquivo por conta própria, fora do contrato da ferramenta, para responder qual host um comando de alvo ambiente atingiria
- `local.active_vps` passa a carregar o nome, ou `null` quando não há marcador

### Alterado
- As linhas de uso pararam de ensinar um argv que o parser rejeita: O `clap` gerava `Usage: ssh-cli exec --use-active <VPS> [COMMAND]...`, anunciando uma forma de dois posicionais que `--use-active` proíbe, impressa exatamente no instante em que o chamador já tinha errado
- O posicional continua sendo um vetor, porque dois slots nomeados ligariam `exec --all uptime` a um slot `VPS`
- A correção é um `override_usage` explícito nas três superfícies exec e no `health-check`, soletrando as três formas legais
- O doc-comment do opt-in do `su-exec` também deixou de descrever `sudo`, de onde havia sido copiado
- A proveniência de commit não congela mais dentro de um checkout: O `.commit_hash` é rastreado no repositório, então vencia o git em todo build local: o binário reportava `f3b98b7` enquanto o `HEAD` estava seis commits à frente
- O git passa a vencer sempre que existir um checkout, e o arquivo mantém exatamente seu propósito declarado: um tarball do crates.io não traz `.git`, então a sonda do git nada encontra ali e o arquivo responde, como projetado
- BREAKING, G-ERR-R01: Novos exit codes 69 (`EX_UNAVAILABLE`) e 70 (`EX_SOFTWARE`), com as variantes `SshCliError::Unavailable` / `Software`
- `SshCliError::Config`: exit 65, classificado permanente, virou o destino de todo `map_err` sem variante óbvia, 16 deles só em `src/secrets.rs`
- Um keyring do SO bloqueado saía 65 com `retryable: false`, então a única falha do grupo que um retry simples resolve era exatamente a que o agente era instruído a abandonar
- Falha de keyring agora sai 69 e é transiente
- Falha de CSPRNG sai 70
- I/O de `secrets.key` sai 74
- `Config` caiu de 16 para 3 ocorrências nesse módulo, todas erros de dado genuínos
- B2: o texto de erro agora é localizado no modo humano
- O envelope JSON segue em inglês por contrato: 17 das 44 variantes de `i18n::Message` tinham tradução completa em inglês e português brasileiro e zero call site, então `--lang pt-BR` devolvia inglês byte a byte para toda falha
- Seis eram ofuscadas pelos literais `#[error("…")]` do `thiserror`, que renderizam no ponto de emissão real
- Onze haviam sido substituídas por variantes por modo e nunca foram apagadas
- Nada pega isso: `Message` é `pub`, então `dead_code` nunca dispara sobre variante que ninguém constrói, e o `match` exaustivo em `en()` e `pt()` obriga a existir TRADUÇÃO, nunca CHAMADOR
- As variantes substituídas foram removidas, as de erro passaram a carregar o detalhe de origem, e `i18n::localized_error_text` as mapeia por `error_code`: o mesmo discriminador estável que o schema publica
- O ramo `--json` mantém deliberadamente o `Display` inglês de `SshCliError`: o agente ramifica por `error_code`, e um `message` dependente de locale mudaria em silêncio o payload que ele parseia quando o host mudasse de idioma
- Código sem tradução cai no inglês, então a localização é fail-open
- B3 / A7, orçamento de componente: `#[allow(clippy::too_many_arguments)]` aparecia 16 vezes em código de produção, concentrado exatamente nos arquivos que A7 apontava como grandes demais: o único lint que mede esse acoplamento estava calado justo onde importava, e é por isso que `-D warnings` ficava verde por cima
- Removidas as 16, só 10 voltaram a disparar: seis eram atributos mortos cujas funções já haviam caído abaixo do limiar
- `crate::vps::AuthOverrides`, `HealthCheckRequest`, `tunnel::ServeContext` e `output::TunnelClosedInput` substituem as listas posicionais
- 16 supressões viraram 3, cada uma com comentário justificando por que a forma achatada é correta
- `src/vps/exec_ops.rs` (748 linhas) foi partido em `exec_ops/{types,single,elevation,fleet}.rs` e `src/sftp/mod.rs` (630) em `sftp/{setup,dispatch,emit}.rs`, com todo caminho público preservado por reexport
- `src/ssh/sftp_session.rs` segue inteiro por decisão declarada: é uma máquina de estados coesa do SFTP v3
- Três suítes novas (`gaps_v062_cross_platform`, `gaps_v062_i18n_reachability`, `gaps_v062_component_budget`) transformam tudo isso em medições que falham alto
- G-SCP-R01 / G-SCP-R02: O `scp-transfer` ganhou `mtime_preserved` e `durable`
- Ambos são aditivos e têm default `true`, então eventos escritos por versões anteriores continuam válidos
- O produto documentava preservação de mtime como garantia enquanto descartava a falha em dois níveis de aninhamento
- O fsync do diretório pai após o rename atômico era invisível do mesmo jeito
- As falhas passam a ser logadas e reportadas em vez de resolvidas por silêncio: transferências para filesystems que não representam timestamp (FAT32, exFAT, caminhos de interop do WSL) continuam bem-sucedidas, e agora dizem isso
- C3, o gate de tamanho de componente foi calibrado para passar, não para uma meta: A primeira versão de `gaps_v062_component_budget` usava teto único de 830 linhas, escolhido para caber `src/ssh/sftp_session.rs` (810), e o comentário do teto nomeava esse arquivo como "exceção deliberada"
- A constante concedia dez: nove outros arquivos estavam entre 601 e 810 e passavam sem jamais serem declarados, enquanto a mensagem de falha do próprio teste mandava não levantar o teto
- É o modo de falha do B1 reaparecendo dentro da remediação do próprio B1: um gate cujo limiar deriva da medição atual registra o presente e chama isso de conformidade
- O orçamento agora é 600 duro mais a catraca `DECLARED_EXCEPTIONS`: todo arquivo acima é nomeado com teto congelado e justificativa obrigatória, uma entrada nunca pode crescer, e entrada cujo arquivo caiu ao orçamento reprova a suíte, de modo que o ledger só encolhe
- A catraca reprovou o próprio commit que a introduziu, ao pegar `src/i18n.rs` crescendo de 757 para 778 linhas
- Em vez de editar o número, o arquivo foi partido: `en()` e `pt()` viraram `src/i18n/en.rs` e `src/i18n/pt.rs`, caindo de 778 para 577 linhas e saindo do ledger, com a exaustividade intacta porque cada `match` ainda precisa cobrir toda variante
- C3b, esse split desarmou três gates presos a caminho, agora presos a contrato: `gaps_v062_i18n_reachability` passaria a contar as tabelas realocadas como call sites de produção, fazendo toda variante parecer alcançável, verde medindo nada
- `gaps_v040_integration::gap_scp_020_i18n_mensagens` ficou vermelho procurando string traduzida no arquivo errado
- `scripts/check_en_identifiers.sh` acusou falso positivo porque sua allowlist nomeava só `src/i18n.rs`
- É a quarta ocorrência da classe, depois de `gaps_v057_sftp` e `gaps_v061_error_taxonomy`: asserção de fonte presa a caminho passa por vacuidade quando o código entra e falha por engano quando sai, porque testa layout e não contrato
- A allowlist virou `src/i18n(\.rs|/)`, o teste de integração lê o subsistema concatenado, e a suíte de alcançabilidade ganhou `TRANSLATION_TABLES` mais um teste que reprova se o caminho excluído deixar de existir ou se as tabelas voltarem para dentro
- Ambos os gates foram reverificados por falsificação deliberada: variante-sonda sem uso deixou a alcançabilidade vermelha, e arquivo-sonda com literal em português fez o gate de identificadores sair com 1

### Corrigido
- O crate publicado não compilava, e nenhum comando local era capaz de dizer isso: Renomear o módulo de resolução de alvo para `src/cli/target/` colidiu com o `target/` sem barra do `.gitignore`
- Um padrão de gitignore sem barra inicial casa em qualquer profundidade, então o git ignorou um diretório de código-fonte, e o `cargo package`: que monta a lista de arquivos pelo git dentro de um checkout, descartou os dois arquivos daquele módulo do tarball
- `cargo build`, `cargo clippy`, 902 testes e os onze scripts de gate seguiram verdes, porque todos eles leem o disco
- Só o `cargo publish --dry-run` falhou, com `E0583: file not found for module `target``
- A regra de ignore agora está ancorada em `/target/`, que é o que ela sempre quis dizer: este crate tem um único membro e a saída de build mora na raiz, e o módulo foi renomeado de novo, para `targeting`, de modo que a correção não dependa mais de uma linha do `.gitignore` continuar certa
- O texto localizado de `no_active_vps` ensinava um argv que a superfície rejeita: `health-check --use-active` também alcança `SshCliError::NoActiveVps`, e a mensagem respondia `execute \`connect <NOME>\` ou informe \`<VPS> <COMANDO>\``, mas `health-check` não recebe comando, então quem seguisse a instrução colhia um segundo erro de uso
- A redação passa a ser agnóstica de superfície nos dois idiomas
- É a localização chegando depois da mudança de superfície: o opt-in `--use-active` deu a uma mensagem antiga um segundo chamador, e nada emparelha mensagem com o conjunto de superfícies capaz de emiti-la
- Dois doc-comments envelheceram do mesmo jeito e foram corrigidos: `HealthCheckJson::host_source` e `HealthOptions::host_source` ainda explicavam por que um probe sem nome pode herdar o marcador, o que a mesma release transformara em erro de uso
- Justificativas envelhecem pior que descrições, porque o comportamento que defendem pode ser removido sem que o parágrafo perceba
- C1, o contrato de mensagem em inglês nunca chegou ao artefato legível por máquina: O B2 estabeleceu que o `message` do envelope `--json` permanece em inglês independente de `--lang`, e documentou isso em `docs/AGENTS.pt-BR.md` e `docs/schemas/README.md`, mas não em `docs/schemas/error-envelope.schema.json`, cuja descrição de `message` seguia com o genérico "texto de erro legível por humano"
- O único artefato do qual um agente geraria cliente era justamente o que não declarava o contrato
- A descrição agora declara o invariante, manda ramificar por `error_code` e delimita a localização ao modo texto humano
- C2, metade do emissor de erro nunca foi localizada: O B2 roteou todo `SshCliError` tipado pelo i18n e deixou intacto o último ramo de `resolve_exit_code`: uma cadeia `anyhow` que não faz downcast nem para `SshCliError` nem para `DomainError` imprimia inglês cru sob `--lang pt-BR`
- A falha que o operador tem menos condições de interpretar era a única nunca traduzida, e parecia coberta porque o caminho tipado: o testado, estava correto
- `localized_error_text` recebe `&SshCliError` por assinatura, então aquele ramo não tinha como chamá-la
- `Message::ErrorUnexpected` e `i18n::localized_unexpected_text` fecham o buraco
- O helper devolve `String` em vez de `Option` porque ali não existe renderização alternativa para falhar aberto
- Só o rótulo é traduzido: a cadeia `anyhow` é preservada verbatim para não perder diagnóstico, e o envelope JSON mantém `error_code` `"unexpected"`
- B1, o alvo Windows não compilava: `cargo check --target x86_64-pc-windows-msvc --no-default-features` falhava com seis erros enquanto `fmt`, `clippy`, 818 testes, `deny` e `doc` estavam todos verdes, e `docs/CROSS_PLATFORM.pt-BR.md` declarava Windows como suportado
- Cinco erros vinham do `#![forbid(unsafe_code)]` em `src/platform/mod.rs`: atributo interno em arquivo de módulo governa também os filhos, o filho `windows` é a única superfície FFI Win32 do produto, e `forbid`, diferente de `deny`, não admite `#[allow]` interno
- O sexto era o `windows-sys` 0.61 redefinindo `HANDLE` de inteiro para `*mut c_void`, quebrando a guarda `handle == 0`
- O módulo passou a `deny`, `windows.rs` carrega um `allow` com escopo de arquivo referenciando a allowlist auditada do G-UNSAFE, e a guarda virou `handle.is_null()`
- A causa estrutural é que nenhum gate faz cross-compile: código sob `#[cfg(target_os = ...)]` de outro alvo é descartado antes do type-check, e fica invisível para todo gate rodado no host
- `scripts/check_cross_targets.sh` agora faz type-check de `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` e `x86_64-apple-darwin`, e é obrigatório no `CONTRIBUTING.md` e na checklist de release
- B4: `SCP_IO_CHUNK = 32_768` era declarado duas vezes como `const` local de função em `src/ssh/client_real_scp.rs`, e `SCP_HEADER_MAX_BYTES` uma vez em `src/ssh/scp_wire.rs`, cada um com um marcador admitindo que pertencia a `crate::constants`
- A justificativa escrita citava uma allowlist de edição de uma rodada já encerrada, então o motivo expirou e a duplicação ficou: e `src/constants.rs` já hospedava `SFTP_IO_CHUNK` com o valor idêntico
- As duas constantes migraram, e um gate falha se o marcador reaparecer
- B5: O `gaps.md` declarava "OPEN residual: 0" no cabeçalho enquanto dezoito seções seguiam com título `OPEN` em negrito no corpo
- As dezoito foram verificadas fechadas no código em duas auditorias
- Só os títulos nunca haviam sido atualizados
- Rastreabilidade é a única coisa que aquele arquivo entrega, então a divergência é defeito de produto, não de forma
- A2: O `docs/schemas/error-envelope.schema.json` enumerava `error_class` como `transient|permanent|cancelled` enquanto a CLI emite `partial` desde a 0.5.4 (G-ERR-R02)
- Um validador estrito de agente rejeitava exatamente o envelope que o produto gera numa falha parcial de frota
- O `error_code`, emitido em toda falha, também nunca foi contratado
- Os dois agora estão declarados
- A4: `sftp ls` e `sftp stat` escreviam em stdout com `println!`, furando a fachada `output` que o `src/lib.rs` documenta como obrigatória, então `--quiet` era ignorado e pipe para `head` abortava em EPIPE em vez de sair 141
- As linhas de sucesso de `mkdir` / `rmdir` / `rm` / `rename` eram montadas com `format!` literal em inglês, então `--lang pt-BR` produzia inglês na saída SFTP mais lida
- Ambas passam por `output` e `i18n`
- A6: O `cargo build --no-default-features`, a configuração de diagnóstico documentada no próprio `Cargo.toml`, falhava com 62 erros
- O subsistema SFTP nunca foi gateado por `ssh-real` apesar de chamar `russh_sftp` diretamente
- O módulo, seus emissores e o braço de dispatch agora são gateados, e o subcomando responde com erro tipado em vez de falhar na linkagem
- G-QA-R02: A faixa de resolução do `run_tunnel` (busca no registro → overrides de credencial → `ConnectionConfig`) foi extraída para `resolve_tunnel_connection`, que recebe o registro já carregado e não toca disco
- É exatamente a faixa onde o bug E3 de agent-auth viveu, e ela não tinha cobertura offline nenhuma
- D5, dois schemas publicados eram inalcançáveis pela CLI: `docs/schemas/` tinha 22 documentos enquanto o catálogo embutido `SCHEMAS` em `src/cli/schema_cmd.rs` listava 20, então `ssh-cli schema dry-run` e `ssh-cli schema tunnel-closed` saíam 64 com `unknown schema`, enquanto o `docs/schemas/README.md` manda o agente descobrir schemas rodando `ssh-cli schema` e documenta exatamente esses dois nomes
- O contrato foi escrito onde era confortável escrever, não onde o consumidor lê
- As duas entradas estão no catálogo e os dois comandos saem 0 com o documento
- O novo `catalog_and_disk_agree` compara os dois conjuntos nas duas direções e valida que o nome é o leaf do arquivo menos o sufixo, então schema publicado em disco sem entrada no catálogo: ou entrada de catálogo sem arquivo, reprova a suíte em vez de chegar ao agente como `unknown schema`
- D10, o único desambiguador de um encerramento bem-sucedido de túnel era descartado: O `src/tunnel.rs` emitia o evento de fechamento como `let _ = output::print_tunnel_closed_json(…)`
- Os três finais `deadline`, `signal` e `accept_error` são distinguíveis somente por esse evento, então uma emissão falha entregava ao agente um exit de sucesso sem nenhuma forma de saber qual dos três havia ocorrido
- A falha passa a ser reportada por `tracing::warn!`, o mesmo padrão que o R10 aplicou aos dois `Result` descartados em `copy_bidirectional`
- O exit code é deliberadamente o mesmo: o encerramento é de fato sucesso, mas a falha de emissão fica visível no stderr, onde o contrato agent-native coloca diagnóstico
- K4, a proveniência de commit degradava em silêncio para `unknown` no crate publicado: Uma ferramenta externa de higiene executou `git rm --cached .commit_hash`, e o `cargo package --list` caiu de 245 entradas para 244 com o arquivo ausente
- Dentro de um checkout o Cargo monta a lista do pacote a partir do índice, então arquivo não rastreado sai do tarball por mais que siga em disco
- O `.commit_hash` é o passo 2 da precedência do `build.rs` (`SSH_CLI_COMMIT_HASH` → `.commit_hash` → `git` → `unknown`) e a única fonte de proveniência que sobrevive num build de fonte do crates.io, onde não existe checkout: então a remoção fez todo build publicado reportar `unknown` sem que nenhum gate reclamasse
- O `gap_rel_007_build_rs_precedence` afirmava que o arquivo existe em disco, o que é verdade e insuficiente, então ficou verde o tempo todo
- Ele passa a exigir também o arquivo no índice do git, pulando essa checagem quando `.git` está ausente: que é exatamente o build do crates.io, onde o arquivo é a evidência sobrevivente e não algo a reverificar
- A primeira tentativa afirmava que a última regra de `.gitignore` que casa deve ser uma negação
- Isso foi medido errado e revertido, porque re-adicionar o arquivo com `git add -f` com a regra de ignore ainda vencendo por last-match restaurou as 245 entradas, provando que o rastreamento é o que sustenta e a regra de ignore não

### Interno
- `src/cli/exec_target/` agora é `src/cli/targeting/`: Ele nasceu servindo as três superfícies exec e foi batizado por elas
- Hoje também resolve o `health-check`
- Um módulo com nome mais estreito que a função dele convida o próximo chamador a copiá-lo em vez de estendê-lo, que é como a duplicação que ele foi extraído para remover volta
- O módulo é privado e todos os itens são `pub(crate)`, então nada fora do crate percebe
- As funções mantêm o sufixo `_exec_target`: elas nomeiam uma operação, não um lugar
- A primeira tentativa deste renome usou o nome mais curto `target` e produziu um crate impublicável: veja a entrada de empacotamento em Corrigido
- O `tests/gaps_v067_packaging_reach.rs` exige que todo arquivo-fonte sobreviva ao `cargo package`: O gate tem dois braços: uma leitura literal do `.gitignore`, que nunca pula, e uma sonda por `git check-ignore`, que é exata por ser o mesmo matcher que o Cargo consulta e que anuncia o próprio skip em voz alta quando o git falta
- Uma denylist de nomes como `target` foi recusada: com a regra de ignore ancorada o nome é inofensivo, então a denylist seria superstição guardando o último defeito em vez da classe
- O cabeçalho `### Corrigido` da 0.5.4 em português dizia `### Segurança`: Os quatro bullets são idênticos aos do inglês sob `### Fixed`, então o gêmeo inglês prova a intenção
- Quem lesse em português procurando correções da 0.5.4 não achava nenhuma, e achava correções arquivadas como segurança
- Corrigir um rótulo errado restaura o registro
- Não é reescrever história
- Os changelogs agora são gateados por estrutura, não só por token: A release 0.5.5 tinha `### Security` três vezes, `### Added` duas e `### Changed` duas, e o inglês tinha nove cabeçalhos contra oito do português
- O que faltava importava: quatro entradas de segurança, incluindo uma CVE do `russh`, estavam sob `### Adicionado`, então quem lesse em português procurando trabalho de segurança não as encontraria
- Todos os gates existentes comparavam tokens, todas as palavras estavam presentes, e por isso todos ficaram verdes: a propriedade que falhou foi a colocação
- O `the_newest_release_has_one_section_per_name_in_both_languages` agora exige nomes permitidos, unicidade, ordem canônica e paridade EN↔pt-BR por uma tabela de pares explícita, escopado à release do topo para não reescrever história já publicada por causa de uma regra criada depois dela
- Comentários de código agora são gateados por idioma, e o gate passa a isentar citação em vez de arquivo: Havia quarenta e cinco linhas de comentário em português espalhadas por dezenove arquivos, a palavra `desvio` em quatro trechos do changelog inglês e cerca de trinta funções de teste com identificadores em português
- Nada disso foi pego porque nada olhava
- A primeira versão, `source_code_is_written_in_english`, varria diacríticos latinos contra uma allowlist por arquivo
- Ela cresceu para treze entradas, cada uma honestamente merecida, porque ali os acentos eram mesmo a carga útil
- O custo era estrutural: uma vez que `tests/docs_conformance.rs` está isento, as mil e duzentas linhas dele estão isentas
- O `tests/gaps_v068_source_language.rs` o substitui e apaga spans de string, char e crase por linha, então comentário pode citar dado localizado e não pode ser português: o que mantém esses mesmos treze arquivos sob a regra
- A allowlist agora está vazia, e isso é medição: todo acento nos treze, `src/i18n/pt.rs` inclusive, já vive dentro de literal
- Escolher a unidade certa de isenção eliminou a necessidade de isentar qualquer coisa
- O scanner de idioma precisou aprender que `\` não escapa nada dentro de raw string: O primeiro rascunho leu `r"\\?\"` em `src/paths.rs` como aspa escapada, engoliu o terminador e dessincronizou o resto do arquivo
- O sintoma foi camuflagem, não silêncio: três linhas de `assert!` cheias de dado de teste português legítimo foram acusadas, enquanto um `// é precomposed` genuinamente nu onze linhas abaixo foi engolido e nunca reportado
- Um scanner que faz parse errado acusa o inocente e protege o culpado na mesma execução, então raw strings passam a ser lidas como raw strings e os dois tipos de literal atravessam linhas
- Dois nomes de teste em português sobreviveram a todos os gates, e a extensão diz por quê: `gap_sec_001b_todo_sidecar_pontuado_esta_barrado_do_pacote` e `gap_sec_002_e2e_recusa_grok_config_dentro_do_repo` não carregavam conectivo listado nem acento, então nem o gate de shell nem o de diacrítico os enxergavam
- Foram achados por leitura
- O `check_en_identifiers.sh` ganha `_da_`, `_dos_`, `_das_`, `_dentro_` e `_esta_`
- Ele deliberadamente não ganha `_do_` nem `_no_`: medido nesta árvore, `_do_` aparece quatro vezes, duas delas em inglês real (`rejections_do_not`), e `_no_` trinta e nove vezes, quase todas em inglês
- Sinal que dispara em código correto faz desligarem o gate inteiro, então os homógrafos ficam de fora e a exclusão fica escrita onde o próximo autor vai buscá-los
- Dezessete mensagens de `assert!` em português no `tests/gaps_v039_integration.rs` agora estão em inglês: Elas viviam dentro de literais de string, que é exatamente onde a regra de citação se recusa a olhar, porque é ali que mora o dado de teste
- Mensagem de asserção é prosa vestida de dado
- A regra não separa as duas mecanicamente, então essa classe segue sendo tarefa de leitura e o limite fica declarado aqui em vez de subentendido
- O gate `install-resolve` ficou vermelho por causa do host, não do código: Ele é o único gate que compila a árvore de dependências inteira do zero, e o fazia dentro de `mktemp -d`, que segue `TMPDIR` para um `tmpfs` sob quota por usuário
- Executado depois de o gate `test` já ter enchido esse `tmpfs`, morreu com `Disk quota exceeded (os error 122)` compilando `aws-lc-sys`
- Um gate que grita lobo passa a ser pulado, e gate pulado não protege nada
- Só o `CARGO_TARGET_DIR` mudou, para um diretório aninhado sob `target/`: o que o gate prova é que o grafo re-resolve sem o lockfile do repositório, e essa prova vive inteiramente no `Cargo.lock` gerado no diretório temporário, então onde os objetos caem é irrelevante para ela
- Reaproveitar artefatos entre execuções cortou o gate de 53s para 34s como efeito colateral
- A catraca de orçamento de componente não tinha opinião sobre teto que ultrapassa o próprio arquivo: O `DECLARED_EXCEPTIONS` recusa crescimento e recusa linhas cujo arquivo caiu ao orçamento, mas um teto deixado para trás por arquivo que encolheu seguia autorizando a diferença: três entradas guardavam 13, 25 e 31 linhas de crédito que ninguém defendeu, espaço para uma função inteira com o gate ainda verde
- É a falha que o próprio doc-comment deste ledger nomeia, chegando pela outra direção, e duas das três foram abertas pelas extrações desta mesma release
- Os tetos passam a estar nas medidas reais e o `assert_ledger_is_current` falha acima de `MAX_LEDGER_SLACK` (8 linhas)
- A igualdade foi rejeitada de propósito: ela deixaria vermelho o ato de apagar uma linha, tornando o jeito mais barato de ficar verde não arrumar nada, e gate que pune faxina treina o oposto do que pretende
- O ledger de orçamento de componente quitou uma entrada: O `json_wire/vps_export.rs` estava congelado em 606 linhas e agora mede 586, dentro do `HARD_BUDGET`, então o `the_exception_ledger_contains_no_stale_entries` falhou até a linha ser apagada
- É a catraca funcionando no sentido para o qual foi feita: ela registra a quitação com o mesmo volume com que registra a dívida, e subir o teto para manter a linha viva é justamente o movimento que ela existe para bloquear
- O ledger caiu para seis entradas
- A prosa voltada a agente ensinava o alias em vez do campo canônico: O `docs/schemas/exec.schema.json` manda vincular consumidor novo a `target_resolved` e `target_source`, com `host_resolved` e `host_source` como aliases de leitura da 0.5.5 escritos a partir do mesmo valor
- Os dois `SKILL.md` e os dois `AGENTS.md` nomeavam só os aliases, então um agente que seguisse a skill vinculava à grafia de compatibilidade da qual o schema afasta
- Os quatro passam a ensinar o par canônico primeiro e a nomear os aliases pelo que são
- A bateria tem onze gates, e cinco documentos diziam dez: O `check_all_gates.sh` ganhou o `exec-target` sem que nenhum dos seus leitores percebesse
- Os dois `RELEASE_CHECKLIST`, os dois `CONTRIBUTING` e os dois `TESTING` estão corrigidos, e o checklist agora manda contar com `--list` em vez de confiar na frase: número em prosa é fato sem gate atrás dele
- O `docs/MIGRATION.md` não tinha seção da 0.5.5: A release com duas quebras no modo como todo verbo acha o host não documentava nenhuma delas onde quem atualiza vai olhar
- Os dois idiomas passam a trazer as três formas legais, a divisão 64/66 e o trio de proveniência
- O `docs/schemas/README.md` listava `telemetry`, `active_vps` e os campos de segredo soltos, quando o schema os aninha sob `local`: Um agente que vincula a `.telemetry` lê `null`
- A metade portuguesa estava ainda mais atrás, descrevendo o envelope de erro como três campos quando ele tem doze
- As duas metades agora batem com o schema, incluindo o trio de proveniência e a regra de ausente-nunca-vazio
- O `SECURITY.md` documentava as correções da 0.5.4 e parava ali, enquanto a 0.5.5 é ela própria uma release de segurança: A política que o leitor consulta primeiro nada dizia sobre o GAP-SSH-EXEC-ARGC-001, o defeito de maior severidade desta linha, um comando que reescrevia uma máquina que ninguém nomeou
- As duas metades ganharam a seção `Correções de segurança na 0.5.5`, o modelo de ameaça ganhou um quinto adversário (argv ambíguo de operador legítimo, que não precisa de atacante nenhum), e a linha STRIDE de argv deixou de escrever `—` sob Repúdio: antes do trio de proveniência existir, uma execução genuinamente não podia ser amarrada a um host, que é exatamente o que Repúdio nomeia
- O blockquote da 0.5.5 ensinava o alias em vinte e dois lugares: Ele anunciava que o envelope "ganhou `host_resolved`, `host_source` e `active_fallback`" enquanto o `docs/schemas/exec.schema.json` manda vincular consumidores novos a `target_resolved` e `target_source` e chama o outro par de alias de leitura da 0.5.5
- A mesma frase está replicada em doze documentos ingleses e dez portugueses, então um único substantivo errado propagou para todo ponto de entrada de uma vez
- O `docs/COOKBOOK.md` era pior que prosa errada: sua receita de copiar e colar fazia parse de `.host_source`, então todo leitor que a seguisse vinculava automação nova ao campo de compatibilidade
- Quatro linhas de tabela renderizavam com colunas fantasma no crates.io: Um pipe dentro de uma célula encerra essa célula mesmo dentro de um code span, então ``| `scp upload|download` |`` é uma linha de três colunas numa tabela de duas
- A linha de `secrets` carregava dois desses pipes
- Linhas vizinhas escapavam os seus corretamente, então isto era deriva entre linhas e não convenção
- O `tests/gaps_v069_markdown_tables.rs` agora conta pipes não escapados por linha contra a linha delimitadora que abre cada tabela, nos vinte e cinco documentos renderizados: estrutural em vez de lexical, então também pega célula perdida por pipe solto em prosa
- Os dois changelogs declaravam Keep a Changelog 1.1.0 e abriam sem `[Unreleased]`, e um gate impunha a omissão: O formato exige essa seção
- O `top_release_sections` do `tests/docs_conformance.rs` pegava o primeiro cabeçalho `## [` incondicionalmente, então acrescentá-la faria o parser devolver vazio e disparar a própria asserção "the parser is broken, not the changelog"
- O gate proibia a conformidade com o formato que o cabeçalho do changelog diz seguir
- O parser agora pula `[Unreleased]` explicitamente, o `the_release_parser_skips_the_unreleased_section` fixa esse comportamento, e os dois arquivos abrem com a seção
- O rodapé estava obsoleto do mesmo jeito: `[Unreleased]` comparava contra `v0.5.3`, duas releases atrás, e nem `[0.5.5]:` nem `[0.5.4]:` existiam
- O `README.md` proibia numa seção exatamente o que fazia em outra: A seção `Changelog` dizia "Não cole notas de release neste README" enquanto a linha 8 colava uma nota histórica de várias releases, e o `gap_doc_004_product_line_040_e_file_only` exige essa nota, então as duas regras não podiam ser obedecidas como estavam escritas
- A regra passa a ser enunciada como o projeto de fato pratica: o banner e a nota de uma linha sobre a linha de produto são o histórico que este README carrega, e tudo por release vive no changelog
- Os três arquivos `llms.*` abriam com nota de release onde o formato põe a descrição do projeto: Quem consome a convenção `llms.txt` lê o primeiro blockquote após o H1 como aquilo que o projeto é
- Recebia a nota de segurança da 0.5.5
- As taglines agora são o primeiro blockquote nos três, com os banners de release abaixo delas
- O `docs/TESTING.md` catalogava 24 das 40 suítes de teste que existiam então: O documento existe para dizer ao contribuidor para que serve cada suíte, e parava em `gaps_v064`, omitindo o próprio `docs_conformance.rs`, o `test_quality.rs`, toda a faixa `gaps_v061`–`v063`, e cada suíte que esta release acrescentou, inclusive `gaps_v065_exec_target_designation.rs`, que guarda justamente o defeito que dá nome à release
- Um catálogo que cobre 60% é pior que nenhum: o leitor para de procurar depois de consultá-lo e conclui que a cobertura não existe
- As duas metades agora listam as quarenta e uma, e o `tests/gaps_v070_doc_reference_reach.rs` passa a reprovar o build se isso voltar a divergir: em qualquer direção
- Os dois READMEs nomeavam os 47 comandos e nenhum deles era localizável: A tabela `Commands` os dobrava, `sftp upload|download|ls|…` numa célula só, o que se lê bem e derrota a recuperação: `rg -F 'sftp ls' README.md` não devolvia nada enquanto o comando estava bem ali
- Desdobrar em spans separados corrigiu as colunas fantasma ao mesmo tempo, e `README.md` / `README.pt-BR.md` entraram em `FULL_INVENTORY_DOCS` para que a dobra não volte
- Acrescentada a linha `Designação de alvo (0.5.5)` que faltava, mais linhas para `--hosts` / `--tags` e `--use-active`: a mudança breaking desta release não tinha linha alguma na tabela que descreve a superfície do produto
- `.atomwrite/` passa a ser barrado do crate publicado no `.gitignore`, no `exclude` do manifesto e no `.cargoignore`
- O gate de empacotamento pre-publish mediu o `cargo package --list` embarcando `.atomwrite/scratch/old.txt` e `.atomwrite/scratch/new.txt`, staging deixado por uma rodada anterior de edição
- Não estar rastreado pelo git não protegia nada: o Cargo empacota todo arquivo que não é rastreado nem ignorado
- O `.gitignore` já cobria `.serena/`, `.claude/`, `.setting.cyber/` e `.cursor/`, e esquecia justamente o sidecar que a ferramenta de edição do próprio agente cria
- Acrescentado `gap_sec_001b_every_dotted_sidecar_is_barred_from_the_package`, que descobre os diretórios pontuados na raiz em vez de enumerá-los
- O irmão `gap_sec_001` fixa um sidecar por nome, e nomeá-los um a um é exatamente como `.atomwrite/` passou ao lado de quatro vizinhos listados
- O que aparecer na raiz precisa agora ser publicado de propósito, que é o caso de `.git` e `.cargo`, ou barrado nas três superfícies, então um sidecar novo deixa a suíte vermelha no dia em que nasce
- Provado negativamente: remover a linha do `.gitignore` reprova o gate nomeando esse arquivo exato
- Corrigida uma afirmação falsa sobre `vps export` nas duas skills e nos dois conjuntos de evals
- Elas diziam que o corpo do export fica TOML salvo com `--json`
- Não fica: o corpo segue o formato de saída resolvido, e esse resolve para JSON sempre que o stdout não é TTY, o que é toda invocação de agente
- Medido, `vps export -o /tmp/hosts.toml` grava um envelope JSON num arquivo chamado `.toml`
- Corpo TOML exige `--output-format text`
- O `export_import_toml_roundtrip` dava aparência de cobertura afirmando apenas que `sshcli-enc:` está ausente e que o import aceita o arquivo: e como o import aceita TOML e envelopes JSON, o roundtrip fecha com o corpo errado
- A mesma afirmação foi então corrigida nas superfícies longas e agora é sustentada por um gate
- A afirmação falsa do TOML no `vps export` foi corrigida nas vinte superfícies longas que ainda a carregavam: `README` e `llms`/`llms-full` nos dois idiomas, `INTEGRATIONS`, `docs/AGENTS`, `docs/COOKBOOK`, `docs/MIGRATION`, `docs/HOW_TO_USE`, `docs/TESTING`, `docs/RELEASE_CHECKLIST` e `docs/schemas/README`
- Várias afirmavam o inverso do comportamento medido, dizendo ao leitor que o JSON automático em non-TTY não se aplica ao export
- Os exemplos de `docs/COOKBOOK` e `docs/HOW_TO_USE` passam a mostrar os dois caminhos, porque um comando copiável que grava JSON num arquivo `.toml` ensina a lição errada duas vezes
- Novo `no_document_claims_export_defaults_to_toml` em `tests/docs_conformance.rs`, baseado em frases sobre as skills, os documentos de inventário completo e as quinze superfícies que descrevem o corpo do export
- Ele pegou uma linha em `docs/HOW_TO_USE.md` que uma varredura manual por regex perdeu, já na primeira execução
- O histórico do CHANGELOG fica isento de propósito: aquelas entradas registram o que uma versão passada afirmou, e reescrevê-las apagaria a evidência de que a afirmação foi feita
- O `export_import_toml_roundtrip` foi renomeado para `export_body_follows_resolved_format_and_both_import` e agora afirma o que o nome antigo prometia
- Ele fixa o envelope JSON do caminho non-TTY padrão, fixa um corpo TOML com `--output-format text`, fixa que o TOML diz `username` onde o envelope diz `user`, e importa os dois
- A versão antiga afirmava apenas que `sshcli-enc:` está ausente e que o import aceita o arquivo, e como o import aceita TOML e envelopes JSON ele fechava com o corpo errado parecendo verde
- O `tests/gaps_v051_integration.rs` tinha `export_pipe_defaults_to_json_when_non_tty` esse tempo todo: a suíte provava a verdade num teste e anunciava a mentira no nome do vizinho
- Um teste cujo nome afirma o que ele nunca verifica é pior que um teste ausente, porque a lacuna fica invisível exatamente onde alguém iria procurar
- As duas skills foram reconstruídas contra o binário instalado e cortadas para o teto de produto de 4000 palavras, vindo de 4906 e 5008
- Nada saiu do contrato: os 47 comandos, todas as flags e todos os tokens de wire sobrevivem
- As palavras saíram de prosa duplicada: bullets `PROIBIDO` que eram só o espelho negativo de um `OBRIGATÓRIO` da mesma seção, e uma seção `Proibições Absolutas` que repetia por inteiro as proibições que cada seção já havia dito
- Novo `skills_stay_within_the_word_budget` em `tests/docs_conformance.rs`
- O teto era regra escrita que nada contava, então os dois arquivos rodaram um quinto acima dele com 13 gates verdes
- Regra com número e sem gate é indistinguível de regra ausente
- Novo `skills_name_every_command`: `LEAF_COMMANDS` passa a ser afirmado sobre as duas skills
- O `FULL_INVENTORY_DOCS` lista de propósito os sete documentos longos e nunca incluiu as skills, embora cada skill abra o catálogo declarando ser a superfície inteira: verdadeiro quando medido, mantido verdadeiro por nada
- As duas skills passam a separar flags globais das locais de subcomando
- A `--i-accept-network-exposure` estava documentada como global e é local do `tunnel`
- Colocada antes do subcomando sai com exit 2 no parse
- A seção nova nomeia as quatro que parecem globais e não são
- As duas skills passam a documentar os dois significados de `--disable-sudo`
- Antes do subcomando ela suprime elevação numa invocação e não toca o disco
- Em `vps edit` grava `disable_sudo` no config e desabilita elevação naquele host de forma permanente
- A `--enable-sudo`, único jeito de desfazer a forma persistente, não aparecia em nenhuma das skills, então o agente podia entrar num estado que a skill não ensinava a sair
- A `--max-chars` também não era nomeada e passa a constar como alias legado do teto de comando
- Removida narrativa de versão das duas skills, que as regras de produto proíbem ali: dois títulos com `0.5.4+`, uma cláusula `desde a 0.5.4/A3`, os identificadores de gap `G-SCP-R01/R02` e `G-ERR-R01`, e uma frase descrevendo como uma release anterior se comportava
- A documentação publicada passou a cobrir a superfície 0.5.4 que ela apenas anunciava
- `tunnel --reverse`, `--socks5` e `--remote-socket` embarcaram nesta release e apareciam em zero de `docs/HOW_TO_USE`, `docs/COOKBOOK`, `INTEGRATIONS`, `docs/MIGRATION`, `docs/CROSS_PLATFORM`, `SECURITY` e `llms-full.txt`, em nenhum dos dois idiomas: as três flags existiam só no banner de release e no pacote de skills
- O `HOW_TO_USE` ganhou seção de modos de tunnel com exemplo executável por modo, o `COOKBOOK` ganhou quatro receitas, o `MIGRATION` ganhou a seção `Desde 0.5.4` cobrindo os dois BREAKING, o `SECURITY` ganhou seção de correções para A1/A2/A3 mais o guard de exposição, e o `CROSS_PLATFORM` ganhou a regra de portabilidade de que o cliente pode ser Windows e o socket não
- Novo `the_054_surface_reaches_every_user_facing_document` em `tests/docs_conformance.rs`
- Essa suíte já afirmava esses tokens, mas somente sobre as duas skills e os dois changelogs, então ficou verde durante tudo o que está acima: gate apontado para o subconjunto errado é indistinguível de gate ausente no placar
- O contrato é por documento em vez de uma lista única: o cookbook deve uma receita, o guia de migração deve uma nota de upgrade, a política de segurança deve o reconhecimento
- Novo `every_command_appears_in_every_full_inventory_document`: os 47 comandos folha devem aparecer como caminho completo literal em todo arquivo que declara inventário completo
- O `llms-full.txt` e as tabelas de `docs/AGENTS` usavam notação de chaves (`tls acme {account create,…}`), então um recuperador que busca `tls acme account create` não achava nada e concluiria que o comando não existe
- Notação compacta serve a um índice curto que um humano folheia
- Não serve a um arquivo cuja função declarada é ingestão por máquina
- Novo `every_schema_is_indexed_in_the_full_llm_map`: existem 22 schemas em disco e o `llms-full.txt` indexava 14, omitindo `dry-run` e todos os contratos `sftp-*` e `*-batch`, exatamente os envelopes necessários para parsear trabalho de frota e de SFTP
- O `docs/schemas/README.md` tinha gate que varre o diretório
- O mapa de descoberta não tinha, então ficou atrás do disco
- Corrigido um exemplo de cookbook que a CLI recusaria hoje: `--bind 0.0.0.0` aparecia sem `--i-accept-network-exposure`, que o G-TUN-R13 tornou obrigatória
- Nenhum gate executa exemplo de cookbook, então a receita envelheceu para inválida sem sinal algum
- Doze declarações de linha de produto passaram de 0.5.3 para 0.5.4
- A deriva era simétrica nos dois idiomas, então nenhum gate de paridade podia vê-la
- Cerca de sessenta menções restantes a `0.5.3` são piso de versão legítimo (`prefira 0.5.3+` para SFTP) ou seções históricas de migração, e foram classificadas antes de qualquer edição em vez de substituídas às cegas
- A máscara de permissão SFTP passa a estar documentada como o que ela é: direcional
- `SFTP_PERM_MASK` (`0o7777`) vale na saída, no upload, e preserva setuid/setgid/sticky de propósito num arquivo que o chamador já controla
- `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) vale na entrada, no download, onde o modo vem do servidor
- Doze linhas em dez arquivos nomeavam a primeira como se fosse a única máscara, e `docs/MIGRATION` anunciava A3 quinze linhas acima contradizendo-a: o leitor conclui que a correção não existe
- O novo `permission_mask_claims_are_directional` varre `docs/` e reprova qualquer arquivo que cite `0o7777` sem a constante de entrada, então a classe não pode reincidir, e não só esta instância
- Esse gate de máscara nasceu varrendo só `docs/`: o diretório que eu estava editando, e ficou verde enquanto a afirmação idêntica seguia incorreta em `llms.txt`, `llms.pt-BR.txt`, `llms-full.txt` e nos dois `SKILL.md`
- Um gate escrito para pegar alvo errado foi ele mesmo escopado ao diff do autor, e não à afirmação
- Ampliá-lo para uma lista escrita à mão de quatro diretórios então perdeu `docs/schemas/README.md`, porque `read_dir` não recursa: lista de lugares onde olhar é, ela mesma, coisa que fica atrás do disco
- A varredura agora é recursiva a partir da raiz sobre `.md` e `.txt`, pulando `target/` e `.git`, então documento criado em subdiretório novo entra sem ninguém lembrar de cadastrá-lo
- Seis arquivos a mais foram corrigidos
- `--tags` está documentada
- O seletor é real em `exec`, `sudo-exec` e `su-exec`, e tinha zero ocorrências nos quinze arquivos de `docs/` enquanto `--all` e `--hosts` estavam documentados em todo lugar: um agente que lê esses arquivos conclui que frota por rótulo não existe e abre um processo por host
- O novo `fleet_documents_name_every_selector` afirma que todo seletor que o clap aceita aparece onde a frota é descrita
- Ele também pegou `--hosts` ausente dos dois `docs/HOW_TO_USE`, que nenhuma busca manual havia procurado
- Descrição de A1 corrigida em seis pontos
- O teto de log de 512 caracteres do banner é anterior a esta release (G-SSH-14)
- O que A1 corrigiu foi o corte aplicado por índice de byte, que entra em pânico dentro de caractere multibyte, e `panic = "abort"` transforma isso em morte do processo sem unwind: um par remoto conseguia matar um fan-out multi-host com um único caractere não-ASCII
- O `SECURITY.md` titulava a falha como banner pré-auth "sem teto" e afirmava que a correção limita o que o par faz o cliente reter
- O russh materializa o banner inteiro antes do callback, então o limite residual agora está explícito: A1 limita o pânico, não a alocação
- Corrigida a afirmação de que `tunnel --bind` é validado como IP pelo clap em todos os casos
- Isso vale para o bind local
- Sob `--reverse` a ponta exposta é o posicional `<remote_host>`, guardado por `guard_remote_exposure`, que compara texto em vez de parsear IP porque a RFC 4254 dá significado a nomes e à string vazia: um typo ali é exit 64 do guard, não exit 2 do clap
- Quatro linhas diziam o contrário
- Documentado que `--bind` é aceito e então descartado em silêncio sob `--reverse`: a entrega reversa é forçada para loopback e o `ReverseServe` nunca recebe o endereço
- Os docs diziam corretamente que o reconhecimento guarda o bind do servidor, mas nenhum avisava que a própria flag não faz nada ali, então `--reverse --bind 192.168.1.10` não mudava nada e não avisava nada
- `docs/RELEASE_CHECKLIST` ganhou o item 26, o bloco de honestidade da 0.5.4
- Medido antes da correção: os dois checklists e os dois `docs/TESTING` mencionavam `tunnel_closed`, `--select` e `--count-only` somente na linha 3, o banner de release, e não nomeavam nenhum dos três modos de tunnel em lugar algum, com zero menções a `--dry-run` ou `--tags`
- Um checklist existe para ser contrato, e este anunciava a release no cabeçalho enquanto não verificava nada dela no corpo
- Os quatro arquivos entraram em `SURFACE_054`, e `gaps_v060_tunnel_modes.rs` está nomeado no inventário de suítes
- Uma linha de terminologia corrigida, e não nove
- A linha 63 de `docs/MIGRATION` usava master-key como termo de produto nos dois idiomas
- As outras oito menções em `docs/` descrevem explicitamente o alias legado de keyring aceito na leitura, o que é verdade, então substituição cega teria destruído oito afirmações corretas
- `TransferResult` carrega os dois flags de durabilidade
- `print_transfer_json` recebe por referência em vez de oito argumentos posicionais
- `build_tunnel_closed` / `build_tunnel_listening` separam a construção do payload da emissão
- `tunnel_closed`, `forwards_served` e `capacity_waits` apareciam fora do emissor em exatamente um lugar: um teste que verifica se o CHANGELOG os menciona
- Apagar a emissão não deixaria a suíte vermelha: prosa validando prosa, a um passo da tautologia que o G-QA-R01 foi escrito para impedir
- Novos `tests/gaps_v061_error_taxonomy.rs` e `tests/gaps_v061_scp_durability.rs`
- Novos testes de tunnel cobrindo repasse de agent-auth, o timeout de registro deliberadamente não sobrescrito, cada braço de `close_reason`, o campo `bind` e o `forwards_served` medido num accept real de loopback com cliente injetado
- O `gap_deny_002_deny_toml_sem_ignore_cve` travava `multiple-versions = "warn"` literalmente, então endurecer a política quebrava o teste enquanto afrouxar para `allow` passaria batido
- Agora ele rejeita `allow` e aceita qualquer coisa mais estrita
- A persistência do `known_hosts` dimensiona o buffer pela contagem de entradas em vez de crescer do zero a cada escrita TOFU
- Dois doc-comments em português em `src/ssh/client_real_scp.rs` e `src/retry.rs` traduzidos para inglês, conforme a regra de código-fonte em inglês
- `scripts/check_all_gates.sh`: roda a bateria obrigatória inteira, `fmt`, `build-release`, `build-no-default`, `clippy`, `test`, `deny`, `cross-targets`, `advisory-freshness`, `en-identifiers`, `install-resolve`, numa invocação, com um registro TSV ou NDJSON por gate no stdout, log por gate no stderr e exit não-zero se qualquer gate estiver vermelho
- Ele existe porque `cargo clippy` e `cargo test` abortam no primeiro alvo que não compila, então um único arquivo de teste quebrado escondia o estado de todo gate atrás dele: que foi exatamente como o inventário local passou a declarar 835 verdes com quatro gates vermelhos e um alvo inconstruível
- Ele pegou uma violação de rustfmt na primeira execução
- Não é CI: sem workflow, sem runner, sem rede
- A bateria é sequencial por decisão, porque os gates de cargo disputam um lock de `target/`
- O `scripts/check_advisory_freshness.sh` não tinha outro chamador, então só era alcançável por um script que nada documentava
- O runner e esse gate agora estão declarados em `CONTRIBUTING`, `docs/TESTING` e na checklist de release, nos dois idiomas
- `tests/gaps_v064_gate_runner.rs`: transforma a cobertura do runner em contrato
- Todo `scripts/check_*.sh` precisa ser um gate, e todo `scripts/*.sh` precisa ser gate ou estar nomeado: com motivo, num bloco `Declared non-gates` no cabeçalho do runner
- A cobertura antes era completa só por coincidência de nomenclatura: um `scripts/check_foo.sh` futuro passaria batido, e bateria que omite script em silêncio lê como cobertura total
- A suíte também trava `--locked` no gate de teste e afirma que o runner está documentado nas duas versões de idioma dos três documentos de mantenedor
- As listagens de diretório são ordenadas antes de qualquer asserção, porque o `std::fs::read_dir` documenta que sua ordem depende de plataforma e pode mudar entre chamadas
- O `tests/docs_conformance.rs` passa a afirmar os tokens de `CONTRIBUTING`, `docs/TESTING` e `docs/RELEASE_CHECKLIST` sobre os dois arquivos de idioma
- Checar só o inglês é como o `CONTRIBUTING.pt-BR.md` passou a omitir `gaps_v040`, a lista explícita de suítes e a seção inteira do gate cross-target com a suíte verde: o leitor que não lê inglês recebia estritamente menos, e o gate de paridade não tinha como ver
- O arquivo pt-BR também trazia faixa elidida de suítes (`v038 … v051`), exatamente o padrão contra o qual o par em inglês adverte, porque checagem por `contains` derruba em silêncio as suítes do meio
- D1: o `tests/gaps_v040_integration.rs` chamava `tunnel_subsystem()` e `output_subsystem()`, nenhuma das duas existente, então o alvo não compilava e `clippy` e `test` abortavam antes de um único teste rodar
- As duas são wrappers finos sobre um helper compartilhado `concat_subsystem`, ao lado do `i18n_subsystem()` que o autor original havia escrito
- Código de teste fica fora da concatenação de propósito: incluí-lo deixaria a asserção passar por casar com o texto do próprio teste
- D4: o `tests/gaps_v063_secret_stdin.rs` exigia exit 64 de um erro de parse do clap
- Confirmado na fonte no clap 4.6.6, cujo `error::Error::exit` documenta "exits with a status of `2`": os códigos sysexits de `src/errors.rs` valem somente para erro de produto levantado depois do parse bem-sucedido
- O `assert_ne!` companheiro passava por vacuidade, porque 64 nunca é o que o parser emite
- Agora ele exclui 2, que é o código que a forma suportada de fato não pode produzir
- D7: o `tunnel::tests::accepting_a_connection_increments_forwards_served` passava isolado e falhava na suíte numa taxa que acompanhava `--test-threads`
- O `serial_test::serial` serializa apenas os testes que o carregam, então um leitor não-serial da flag global de parada continuava correndo contra escritores marcados
- Corrigido com as duas metades: `#[serial_test::serial]` mais `crate::signals::reset_flags_for_tests()`, e verificado por execuções repetidas em 8, 32 e 72 threads, não por uma execução verde, já que uma passada só não distingue correção de corrida
- A catraca de componente então mordeu a própria correção: o `src/tunnel.rs` passou do orçamento duro de 600 linhas, e entradas de `DECLARED_EXCEPTIONS` só podem encolher
- `TunnelStats` e seu teste unitário de `close_reason` migraram para o novo `src/tunnel/stats.rs`, derrubando `tunnel.rs` para 570 e removendo a entrada dele do ledger
- Levantar o teto seria a desonestidade que a catraca existe para bloquear
- D3: o `scripts/e2e_real_ssh.sh` substituía um `--bin` não executável pelo default e seguia, imprimindo `PASS E01` até `PASS E16` para um binário que o operador nunca nomeou, num harness cujo próprio texto de uso promete que ambiente inutilizável é falha e nunca skip silencioso
- Uma flag de intenção `BIN_EXPLICIT`, marcada por `--bin` e pelo `SSH_CLI_E2E_BIN` harness-only, faz esse caso sair 2 com `FAIL E00`
- O auto-build sobrevive somente quando nada foi nomeado
- Verificado negativamente
- D9: o `tests/gaps_v062_i18n_reachability.rs` procurava `fn en(msg: &Message)` em `src/i18n.rs`, que o C3 havia movido para `src/i18n/en.rs`, então a contagem era incondicionalmente o arquivo inteiro e o ramo guardado nunca podia ser tomado
- A busca morta e sua condição saíram e o doc comment agora descreve o que o código faz
- D6: a lista de campos do `scp-transfer` em `skills/ssh-cli-en/SKILL.md` ganhou `ok`, alinhando à skill pt-BR que já tinha os quatro
- A lista de tokens do gate de paridade ganhou `ok`/`direction`, `mtime_preserved` e `durable`: os três que de fato discriminam, já que os nomes de campo isolados já aparecem nas duas skills por outros contextos e a checagem é `contains`
- E1: o `docs/TESTING.md` e o gêmeo pt-BR catalogavam `tests/gaps_v063_gate_falsification.rs`, uma suíte que este repositório nunca teve
- O bullet prometia que todo check do `scripts/check_en_identifiers.sh` tinha prova de falsificabilidade vigente, e a trilha terminava num nome de arquivo
- A varredura direta: toda suíte do disco está listada?, devolvia zero limpo, porque fantasma é invisível desse lado
- As duas metades agora nomeiam os artefatos que existem, e o `tests/gaps_v070_doc_reference_reach.rs` varre nos dois sentidos para que a classe não volte
- E2: os schemas de `exec`, `sudo-exec` e `su-exec` descreviam `host_resolved` como campo próprio, "VPS the command ran on", enquanto `scp-transfer`, `sftp-transfer` e `health-check` no mesmo diretório o rotulavam corretamente como alias de compatibilidade de `target_resolved`
- Três schemas de uma mesma release ensinavam duas respostas diferentes para "qual campo é canônico"
- Pior: prometiam que o campo fica `empty` em eventos antigos, enquanto o `error-envelope.schema.json` promete `Absent — not empty` e argumenta por quê, string vazia afirma que um host foi escolhido e não tem nome
- Consumidor escrito contra o lado errado lê ausência como hostname, ou hostname como ausência
- E3: o `active_fallback` do `error-envelope.schema.json` ancorava o próprio sentido em `host_resolved` e `host_source`, sem nunca nomear o par canônico, o alias explicando o campo canônico em vez do contrário
- E4: o `docs/MIGRATION.md` intitulava a seção de wire como "Formato wire 0.5.3, atual" enquanto a linha logo abaixo dizia "Linha de produto: 0.5.5", e não listava nenhuma mudança de wire desta release
- O schema version continua 3, então o título era defensável e ainda assim lido como "0.5.3 é a release atual"
- O título agora data o formato e declara seu alcance, e a seção carrega as adições da 0.5.5: o trio de proveniência, sua regra de ausência, a divisão entre exit 64 e 66 e os seletores de frota
- E5: catorze agrupamentos de comando dobravam várias folhas num único token, `ssh-cli sftp upload|download|ls|mkdir|rmdir|rm|stat|rename` nos dois `docs/AGENTS.md`, `ssh-cli exec|sudo-exec|su-exec` nos docs de agente e no `docs/schemas/README.md`, mais as famílias `locale`, `tls` e `secrets`
- A forma dobrada satisfaz o leitor humano e devolve nada para uma busca literal por `ssh-cli sftp download`, que é como um agente procura comando: a substring simplesmente não está no arquivo
- Desdobrada, a contagem pesquisável com prefixo no `docs/AGENTS.md` foi de 16 de 47 para 38 de 47, com o restante carregado pela tabela de inventário que o `every_command_appears_in_every_full_inventory_document` já gateia
- E6: o `SECURITY.pt-BR.md` havia perdido cinco itens técnicos que o gêmeo inglês carregava, numa seção sobre crypto de transporte: a janela de CVE do rustls (`0.23.13–0.23.17`) que o leitor precisa evitar, os quatro subcomandos `tls mtls`, a alternativa de caminho absoluto no registro do VPS, o crate `instant-acme` e o modo `0o600` sob `tls/acme/`. mTLS e ACME estavam fundidos num bullet só, 8 bullets em inglês contra 7 em português, e o motivo de a compressão estar desabilitada (redução de side-channel) sumira
- Quem lesse em português não tinha como saber quais versões do rustls evitar
- E7: o documento de segurança se contradizia sobre a release que documenta
- A tabela de versões suportadas marcava toda a `0.5.x` como atual sem ressalva, enquanto a seção 60 linhas abaixo explica que `0.5.0`–`0.5.4` carregam o GAP-SSH-EXEC-ARGC-001
- A lista de ativos nomeava "a capacidade de executar shell remoto / SCP / port forwards locais" sem a identidade do host alcançado, embora a seção nova argumente que a dimensão crítica não é o que executa e sim onde
- E o risco residual aceito (2) ainda dizia que o host remoto é confiado "para o comando one-shot pedido pelo operador", o que pressupõe que o host é o pedido: exatamente a premissa que esta release quebrou
- Os três agora declaram a ressalva
- E8: o `docs/RELEASE_CHECKLIST.md` estava sem uma regra de política que o gêmeo pt-BR carregava: nunca publicar o `gaps.md` nem assertar seu texto FIXED em testes de produto (G13/G15)
- Noventa bullets contra noventa e um: o tipo de lacuna que uma leitura corrida não expõe e uma contagem por seção expõe na hora
- Os sete pares bilíngues sob `docs/` agora batem em H2, H3, bullets e blocos cercados
- E9: os schemas da família exec listavam `host_resolved` e `host_source` em `required`, e o `health-check.schema.json` listava `host_source` mas não `host_resolved`, assimetria que marca drift, não decisão
- Um alias de compatibilidade dentro de `required` deixa de ser alias: a razão de existir um é que uma major futura possa removê-lo sem quebrar validação, e exigi-lo fecha essa porta
- O `scp-transfer` e o `sftp-transfer` da mesma release já exigiam apenas o par canônico, e o `tests/gaps_v061_scp_durability.rs` enuncia a regra de fundo para outro campo: um nome em `required` invalida todo evento que não o tenha
- Os seis agora exigem `target_resolved`/`target_source` e deixam os aliases opcionais
- A CLI segue emitindo ambos, então nenhum consumidor existente vê mudança
- E10: o `health-check.schema.json` escapou inteiro da correção de alias, e o motivo é instrutivo: aquela passada foi guiada pela busca da palavra `empty`, a promessa falsa que a família exec carregava, e este arquivo nunca a usou
- O defeito dele era estrutural
- O `HealthCheckJson` faz flatten de `TargetEcho`, cujos quatro campos são todos incondicionais, então o envelope de sucesso sempre carrega `host_resolved`: e o schema nunca o declarou
- O canônico `target_source` apontava para o alias `host_source` como se o alias detivesse o significado, o inverso do padrão que todo outro schema segue
- Nada reprovava validação, porque `additionalProperties` é `true`: o defeito era silencioso
- A lição é que auditar alias significa comparar os campos que um DTO serializa com o `properties` do schema, e não caçar frases suspeitas
- E11: mais três divergências entre schema e emissor, todas silenciosas pelo mesmo motivo: o `error_code` era descrito como "Present on every failure envelope" estando fora de `required`, embora o `print_error_envelope` o grave como `&str` incondicional
- O `vps-show.schema.json` não declarava nenhum dos cinco campos condicionais que o `MaskedVpsJson` emite (`tags`, `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key`), e o `vps-list.schema.json` faz `$ref` desse item, então a lacuna se propagava para a listagem
- E onze dos vinte e dois schemas não tinham `description` de topo, o que pesa porque `ssh-cli schema <name>` devolve o corpo bruto e metade deles não dizia a que comando pertencia
- E12: o `docs/schemas/README.md` mandava o parser ramificar no `event` sem listar os payloads que não carregam nenhum
- Medido contra o binário: `exec`, `sudo-exec`, `su-exec`, `vps-show` e o `health-check` de host único emitem objeto sem chave `event`, e o `vps list` emite array puro, sem envelope algum
- Um parser que trate `event` ausente como malformado rejeita seis payloads válidos
- A nota sobre eventos CRUD sem schema também escondia quatro deles atrás de reticências: o `vps-import` de forma mais grave, já que ele aparece no enum `operation` do `dry-run.schema.json`, o que manda o leitor caçar o schema do evento real
- E13: o `docs/schemas/README.md` carregava o único agrupamento de comando do repositório sem tabela individual no mesmo documento para compensar: `ssh-cli sftp mkdir|rmdir|rm|rename|stat` colapsava cinco folhas num token que não casa com nenhuma delas, e as linhas de batch faziam o mesmo com `scp`/`sftp upload|download`
- Em outros pontos o agrupamento fica ao lado de uma tabela que lista cada folha, então a busca ainda encontra
- Aqui nada encontrava
- As nove folhas de transferência agora são literalmente pesquisáveis com o prefixo `ssh-cli`
- O `docs/RELEASE_CHECKLIST.md` também terminava em linha em branco, sozinho entre os catorze arquivos dos sete pares, o tipo de deriva que uma checagem por contagem de linhas reporta como paridade porque dois erros se cancelam: a linha em branco final do inglês compensava exatamente o bullet a mais que o português carregava


## [0.5.4] - 2026-08-06

### Segurança
- A1: Corrigida negação de serviço remota pré-autenticação
- `auth_banner` fatiava o banner enviado pelo servidor no byte 512 (`&banner[..512]`)
- Um caractere multibyte nessa fronteira causava panic e, como o perfil de release define `panic = "abort"`, o processo inteiro morria: derrubando junto qualquer fan-out `--all`
- A truncagem agora respeita fronteiras de caractere
- A2: Chaves privadas ACME/mTLS não ficam mais legíveis por todos numa janela transitória
- `write_secret_file` usava `std::fs::write` (criando em `0644` sob a umask padrão) e restringia depois, com o erro descartado
- Agora delega ao helper compartilhado que cria em `0600` via `O_EXCL` e propaga falha de permissão
- A3: Bits `setuid`/`setgid`/`sticky` enviados pelo servidor não são mais reproduzidos no arquivo baixado
- Modos vindos da rede são mascarados com `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) em SFTP e SCP
- Modos de arquivos locais enviados ao servidor mantêm a máscara completa
- A6: `parse_hex_key` rejeita entrada não-ASCII antes de fatiar, então um `secrets.key` com UTF-8 multibyte somando 64 bytes devolve erro tratado em vez de panic

### Adicionado
- C1: Camada agent-native de redução de payload como flags globais: `--select`/`--fields`, `--filter` (repetível, AND), `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- Aplicada no funil único de serialização JSON ANTES de o envelope existir, então o payload gigante nunca é construído
- Medido: `vps list` cai de 943 para 19 bytes com `--select name --limit 1`
- Um `--filter` malformado falha no parse (exit 64) em vez de casar nada em silêncio
- C2: `--no-input` recusa stdin de forma declarativa em vez de bloquear para sempre esperando um humano ausente
- C2: `--dry-run` imprime o plano de uma operação destrutiva e sai sem executá-la: `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt`
- O plano é JSON mesmo no modo texto (`docs/schemas/dry-run.schema.json`), porque prévia existe para ser comparada
- Em qualquer outro comando a flag é recusada com exit 64 em vez de aceita e ignorada: exatamente o defeito com que `--no-input` foi entregue
- As pré-condições rodam antes, então prever a remoção de um host ausente ainda sai 66, em vez de prometer um sucesso que a execução real não entregaria
- G-TUN-R01: `tunnel --reverse` pede ao servidor que escute e entrega as conexões de volta numa porta local
- `REMOTE_PORT 0` é aceito só nesse modo: o servidor aloca e informa a porta que ligou, e é ela que chega em `tunnel_listening.local_port`
- Bind remoto fora do loopback exige `--i-accept-network-exposure`: nessa direção quem fica exposto é o listener do servidor, então guardar o `--bind` local checaria o lado errado
- G-TUN-R02: `tunnel --socks5` serve um proxy SOCKS5 (RFC 1928, sem autenticação + `CONNECT`)
- Cada conexão aceita vira um canal `direct-tcpip`
- Nomes de host são encaminhados sem resolução local, para que signifiquem o que significam do lado remoto
- `BIND` e `UDP ASSOCIATE` recebem o código de resposta `0x07` em vez de um fechamento seco
- O handshake tem teto de 1024 bytes, acima dos 519 que a RFC permite e muito abaixo de qualquer valor útil para fazer o proxy bufferizar à vontade
- G-TUN-R03: `tunnel --remote-socket <PATH>` encaminha uma porta local para um socket Unix no host remoto (`direct-streamlocal@openssh.com`)
- O caminho é validado como absoluto e sem byte NUL
- NÃO é conferido contra o filesystem local, que nada tem a ver com o do servidor
- Contrato do tunnel: `tunnel_listening` e `tunnel_closed` ganharam `mode` (`local` / `socks5` / `streamlocal` / `reverse`)
- É o discriminador que diz como ler os campos vizinhos: sob `reverse` quem escuta é o servidor, e sob `socks5` não existe destino único
- O default é `local`, então eventos anteriores à 0.5.4 continuam válidos
- G-TUN-R07: Novo evento `tunnel_closed` (`docs/schemas/tunnel-closed.schema.json`) com `reason` (`deadline` / `signal` / `accept_error`), `forwards_served`, `capacity_waits` e `duration_ms`
- Esses três finais compartilhavam exit 0 e eram indistinguíveis
- G-TUN-R06: `tunnel_listening` ganhou o campo `bind`, permitindo auditar pelo próprio contrato se um serviço foi publicado além do loopback (aditivo, consumidores existentes não são afetados)

### Alterado
- BREAKING, G-ERR-R02: Falha parcial multi-host agora sai com exit 1, `error_code: "partial_failure"` e `error_class: "partial"`, carregando `failed`/`total`
- Antes saía 65, o mesmo código de TOML malformado, então o agente não distinguia "1 de 10 hosts falhou" de "a configuração está corrompida"
- O exit 65 volta a significar erro de dado real
- BREAKING, G-TUN-R13: Um `--bind` fora do loopback agora exige `--i-accept-network-exposure`
- Sem a flag o tunnel sai com exit 64 antes de qualquer I/O de rede
- `--bind 0.0.0.0` antes publicava o serviço remoto encaminhado na rede local em silêncio
- G-TUN-R08: `--bind` é validado como endereço IP pelo clap, então um typo falha no parse (exit 2) em vez de após um handshake SSH completo
- G-TUN-R09: Falhas de bind mapeiam por `ErrorKind`: `AddrInUse`/`PermissionDenied` → 74 (retryable), `AddrNotAvailable`/`InvalidInput` → 64
- Todas colapsavam antes em 65 classificado como permanente

### Corrigido
- C2: `--no-input` agora recusa stdin em `vps add` e `vps edit`
- A recusa estava implementada só no caminho de override de exec/scp/tunnel, então os dois comandos de registro: justamente os mais prováveis de rodar sem supervisão, aceitavam a flag e liam a senha do mesmo jeito
- O guard foi movido para dentro de `read_secret_stdin`, de modo que todo chamador o herda
- Coberto por `tests/gaps_v059_agent_native.rs`, que não existia quando a flag foi entregue
- Gate de docs: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` voltou a ficar verde: 56 links intra-doc quebrados em 14 sites foram qualificados por completo, e os links para itens `pub(crate)` (`take_utf8_capped`, `EXEC_CAPTURE_HARD_MAX_BYTES`, `lock_config`) viraram código inline, já que item privado não tem página pública para apontar
- E3: `tunnel` respeita `--use-agent` e `--agent-socket`
- O clap aceitava ambos e o dispatch os descartava, então um host registrado para autenticação por agente não conseguia abrir tunnel algum
- G-TUN-R10/R11: A cópia do forward usa `tokio::io::copy_bidirectional`, que reporta bytes por direção e o erro
- O `join!` anterior de dois `copy` descartava os dois `Result`, tornando uma falha no meio da transferência indistinguível de um término limpo em qualquer verbosidade
- G-TUN-R12: A saturação de concorrência é anunciada uma vez por execução e reportada como `capacity_waits`
- Antes era invisível e aparecia só como latência sem causa declarada
- E4: O banner humano imprime o endereço de bind efetivo em vez de `localhost:` fixo
- E7: `open_tunnel_channel` preserva o erro de origem via `channel_src` em vez de achatá-lo em texto
- E6: `tunnel-listening.schema.json` está alinhado ao range do clap para `remote_port`
- Com os modos acrescentados adiante nesta release o mínimo compartilhado passa a ser 0, já que `socks5` e `streamlocal` não têm porta remota única e `reverse` usa 0 para dizer "o servidor aloca"
- `local_port` mantém `minimum: 1`, que é o campo ao qual o agente se conecta
- B2: SFTP verifica contagem de bytes nas duas direções
- O SCP já conferia `sent != size` e `received != size` desde a 0.5.2 enquanto o SFTP não conferia nenhuma, então uma transferência SFTP truncada reportava `ok` com um campo `bytes` plausível: o laço contava o que escreveu, nunca o que sobreviveu
- Uploads agora comparam com o tamanho local anunciado E refazem `stat` no remoto (a prova de efeito no destino: cota estourada ou filesystem cheio são invisíveis de outro modo)
- Downloads verificam antes do rename atômico, então leitura curta nunca chega ao caminho final
- Servidores que omitem `size` são aceitos em vez de reprovados, porque o atributo é opcional no protocolo

### Interno
- G-QA-R01: Nova suíte `tests/test_quality.rs` reprova o build em asserções entre dois literais ou `assert!(true)`
- `src/tunnel.rs` mantinha `assert_eq!(0_u64, 0)` sob o nome `timeout_zero_conceptually_rejected`, que não exercitava código de produto algum enquanto sugeria que a guarda one-shot estava coberta
- G-QA-R03: Asserções de documentação movidas para `tests/docs_conformance.rs`, então editar prosa não deixa mais o gate de comportamento vermelho sem regressão funcional
- G-DOC-R01/G-DOC-R02: Fechados junto com a separação acima: o `CONTRIBUTING.md` passa a listar cada suíte de teste nominalmente em vez de citar `gaps_v040` como se fosse a superfície inteira, e a skill pt-BR documenta os campos do `scp-transfer` (`ok`, `direction`, `bytes`, `duration_ms`) que a versão em inglês já trazia
- Ambos foram corrigidos neste release sem serem citados nele
- G-TUN-R04/R05, E1, E2: Os testes de tunnel passam a exercitar comportamento real: a guarda `timeout_ms == 0` é chamada, `run_tunnel_with_client` é de fato invocada, e a asserção de porta efêmera confere a porta ligada em vez de casar strings do próprio fonte com `include_str!`
- C3: O bloco de oito linhas de "N/M falharam", duplicado em cinco caminhos de lote, foi substituído por `errors::finish_batch`
- Divisão G-TUN: `src/tunnel.rs` virou raiz de módulo sobre `local`, `reverse`, `socks` e `streamlocal`
- Os três modos de listener local compartilham um único laço de accept parametrizado pelo destino: copiá-lo significaria manter o tratamento de sinal, o portão de admissão, o drain e a contabilidade de saturação em triplicata, justamente as partes que dão errado de forma sutil em uma cópia e não nas outras
- `run_tunnel` recebe um `TunnelRequest` em vez de quinze argumentos posicionais, onde uma transposição compilava limpa e só aparecia como um tunnel apontando para onde ninguém pediu
- G-QA-R01: `gap_tun_003_source_local_addr` não faz mais grep de `local_addr()` e `effective_port` no texto de `src/tunnel.rs`
- Essa asserção passava com a string dentro de um comentário, continuava passando depois de o comportamento ser apagado, e falhava pelo único motivo que não é regressão: a divisão de módulos moveu o código
- Agora ela dirige o laço de accept real com um cliente injetado e lê a porta que o produto publicou
- Canais `forwarded-tcpip` e `forwarded-streamlocal` não solicitados são recusados com `AdministrativelyProhibited` em vez do default do russh, que aceita e descarta
- A fila de entrada é limitada, então um servidor não consegue crescer a memória do cliente abrindo canais mais rápido do que são drenados
- Fila cheia aplica backpressure no fio


## [0.5.3] - 2026-07-30

### Corrigido
- G1: upload SFTP não trunca mais o destino a zero bytes (`FileAttributes::empty()` em vez de `Default`)
- G2: filtro `-v` com escopo na crate (`warn,ssh_cli=…`)
- Nunca `debug` global (sem vazamento de senha via `russh::client::encrypted`)
- G3: SETSTAT SFTP envia `atime`+`mtime` juntos (sem atime no epoch)
- G4: Result de `set_metadata` SFTP é propagado (SETSTAT mutante não é best-effort)
- G5/G17: cancelamento multi-arquivo preenche resto como cancelled
- `results.len() == input.len()`
- G6/G11: testes de sinal serializados
- `reset_flags_for_tests`
- Cardinalidade sob cancel
- G7: E2E real cobre SFTP com matriz de checksum + árvore recursiva (E17/E18)
- G8: `exec --json` de passo único emite exatamente um objeto NDJSON
- G9: download SCP propaga falha de `sync_data` antes do rename atômico
- G10: débito de formatação fechado com gate `cargo fmt --check`
- G12: permissões mascaradas com `SFTP_PERM_MASK` (`0o7777`)
- G13: removido teste circular que assertava texto FIXED em `gaps.md`
- G14: verbosidade graduada `-v`/`-vv`/`-vvv` (`ArgAction::Count`)
- G15: inventário exige prova de efeito no destino (checksum), não auto-certificação
- G16: identificadores e erros de canal em inglês em `client_real_scp.rs`
- G18: falhas de `set_permissions` no download SFTP são sinalizadas
- G19: constante nomeada `SFTP_PERM_MASK`

### Alterado
- Versão 0.5.3


## [0.5.2] - 2026-07-19

### Adicionado
- `--json` global: (G-AUD-01): alias agent-friendly que força JSON (clap `from_global` nos subcomandos)
- `exec` / `sudo-exec` / `su-exec` com VPS ativo: (G-AUD-04): um posicional = COMMAND no host ativo de `connect`
- Envelope JSON de `vps path`: (G-AUD-02): `event: vps-path` quando o formato é JSON
- Módulo `fs_perm`: (G-AUD-24): fonte única para modos Unix de arquivos/dirs de segredo

### Corrigido
- Warning falso de password em argv: (G-AUD-08): inspeciona `Option` real, não strings `Debug`
- TLS PEM ausente: (G-AUD-05): `FileNotFound` / classe permanente (não exit 74 retryable)
- `vps export` honra formato JSON global: (G-AUD-03)
- ACME account create exige `--contact mailto:…`: (G-AUD-06/28)
- Exclusão mútua de auth primária: na gravação (G-AUD-07): exatamente um de password / key / agent
- Mensagens de secrets: não anunciam mais stores env (G-AUD-21)
- Filtro de log só via CLI: (G-AUD-22 / G-E2E-09): `RUST_LOG` ambiente é ignorado
- Use `-v`
- Cap de concorrência: com fonte única (G-AUD-19/23): `constants::MAX_CONCURRENCY`
- Skill description ≤1024: chars (G-AUD-15)
- ACME validação permanente: (G-E2E-01): `invalidContact` / tipos de problema 4xx → exit 64 não-retryable (`tls/acme_error_map.rs`)
- Um único JSON em `vps add` com auto-key: (G-E2E-04): campo `secrets_key_auto_created` embutido em `vps-added` (um documento, nunca dois eventos)
- Comandos root `schema` + `doctor`: (G-E2E-02/03): descoberta de contrato por agentes
- `doctor` é alias de `vps doctor`
- Stamp de versão `-dirty` com `.commit_hash`: (G-E2E-06): proveniência honesta em trees dirty
- Feature clap `env` removida: (G-E2E-08)
- Help não ensina mais stores env (G-E2E-07)
- Máscara de export redacted: (G-E2E-10): `FIXED_MASK` (três asteriscos), não string vazia (superado: o `vps export` redacted agora emite `password` vazio e null nos outros segredos, e a máscara aparece só em `vps list` e `vps show`)
- `vps add --use-agent` / `--agent-socket`: (G-E2E-19): triplo de auth no inventário (senha / chave / agent)
- Harness E2E offline SKIP: + bin release default (G-E2E-05)
- Identificadores de teste em EN (G-E2E-13)

### Removido
- `.github/workflows`: (G-AUD-11 / G-E2E-11): só gates locais
- Sem runner hospedado de produto na tree
- Shim PT `src/erros.rs`: (G-AUD-14)
- Leituras env de config de produto: `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` (G-AUD-12)

### Alterado
- Versão 0.5.1 → 0.5.2
- Testes de integração alinhados a config só CLI/XDG (sem store env de secrets/formato)
- Gate residual: `tests/gaps_v058_e2e_residual.rs` (G-E2E-01…15,17,19 FIXED, 16/18 MITIGATED)

### G-SFTP residual harden R01–R15

### Segurança
- Validação de basename de entry + `ensure_local_under` em download recursivo/multi-file (servidor SFTP malicioso não escapa destino local)
- Cleanup de partial: em qualquer erro de download SFTP (paridade SCP)
- Root de upload tree: com `symlink_metadata` (no-follow)

### Alterado
- Timeout wall-clock: (`under_timeout`) em multi-file e FS ops
- `cli/scp_args.rs`: extraído (SRP)
- Docs/skills: SCP = arquivos regulares
- Árvores/FS = `sftp`

### G-SFTP: subsistema SFTP

### Adicionado
- `russh-sftp` 2.3 (feature `ssh-real`): subsistema cliente SFTP v3
- `ssh-cli sftp`: `upload|download` (`--recursive` opcional), `ls`, `mkdir`, `rmdir`, `rm`, `stat`, `rename`
- Módulos: `src/ssh/sftp_session.rs`, `sftp_path.rs`, `sftp_types.rs`, `src/sftp/`, `src/cli/sftp_args.rs`
- Schemas JSON: `sftp-transfer`, `sftp-list`, `sftp-fs-op`, `sftp-batch`
- Gates: `tests/gaps_v057_sftp.rs`
- Multi-host `--all`/`--hosts` para upload/download SFTP (`map_bounded`)

### Alterado
- ScpOptions / SftpOptions: `use_agent` + `agent_socket` (G-SFTP-17/18, só CLI/XDG)
- Transferências em stream de chunks de 32 KiB
- Download parcial + rename atômico
- Symlink sem seguir em árvores

### Segurança
- Validação de path remoto (caracteres de controle / segmentos vazios)
- Teto de profundidade recursiva
- Teto de entradas de listagem
- Sem heap de arquivo inteiro via `SftpSession::read/write`

### G-SSH: regras SSH / russh

### Adicionado
- `src/ssh/client_handler.rs`: TOFU `check_server_key` + recuperação tipada `HostKeyOutcome` (G-SSH-01/09/14)
- `src/ssh/client_connect.rs`: dial + cadeia de auth chave de arquivo → agent → senha (G-SSH-04/16)
- `src/ssh/key_material.rs`: perms Unix 0600 de chave privada + rejeição de RSA <2048 e de DSA (G-SSH-03/07)
- Agent CLI/XDG: `--use-agent`, `--agent-socket`
- `VpsRecord.use_agent` / `agent_socket` (sem env como store)
- Gates: `tests/gaps_v056_ssh.rs`

### Alterado
- client id do SSH: `SSH-2.0-ssh-cli` (sem fingerprint por banner de versão do russh)
- Política de config: rekey explícito 1h/1GiB, janela 2MiB, pacote 32KiB, TCP SO_KEEPALIVE via `socket2`
- deny.toml: ban de `ssh2`, `thrussh`, `libssh-rs`
- Split SRP de connect/handler/carga de chave fora do caminho de sessão/SCP

### Segurança
- Troca de host key devolve `HostKeyChanged` tipado (exit EX_NOPERM) para agentes
- Fail-closed quando `known_hosts_path` falta fora de testes
- Fallback de senha continua opt-in por secret não vazio no inventário (documentado)

### G-UNSAFE: unsafe code e FFI

### Adicionado
- `src/test_util/env.rs`: `set_var`/`remove_var` encapsulados com `// SAFETY:` (pronto para ed2024)
- `src/vps/config_io.rs`: split de path/load/save/permissões do config (SRP / G-UNSAFE-10)
- Gates: `tests/gaps_v055_unsafe_ffi.rs` (allowlist, forbid, ordem do main, sem env de plaintext)

### Alterado
- `main`: `signals::register_handler()` antes do Tokio `new_multi_thread` (G-UNSAFE-13 / first-hook do signal-hook)
- SAFETY de SIGTERM: multi-bullet (só atômicos async-signal-safe, duplo hit justificado vs `flag::register`)
- FFI de console do Windows: docs de segurança do módulo
- Uma op por bloco `unsafe` inalterada
- Testes de VPS usam `set_runtime_flags` para o opt-out de plaintext (sem env `SSH_CLI_ALLOW_PLAINTEXT_SECRETS`)
- Docs de secrets: material de chave por env é fail-closed
- Plaintext só por flag de CLI
- Docs de concorrência: só `--max-concurrency` da CLI + fórmula auto (sem store env)
- `forbid(unsafe_code)` em `ssh/mod`, `vps/mod`, `vps/model`, `vps/config_io`

### Segurança
- Allowlist de `unsafe` de produto: só `platform/windows.rs` + `signals.rs`
- Mutação de env de teste centralizada

### G-ERR: tratamento de erros

### Adicionado
- `SshCliError::Domain`, variantes `Crypto` e `Config`
- Erros estruturados de TLS/canal com `source` opcional
- Helpers: `tls_msg` / `tls_src` / `channel_msg` / `channel_src` / `error_code()` (G-ERR-08/16)
- Campo `error_code` do envelope JSON para agentes (snake_case estável)
- Gates: `tests/gaps_v054_error_handling.rs`
- Split do client SSH (G-ERR-12): fachada `client.rs` + `client_real.rs` / `client_stub.rs` / `client_tests.rs`

### Alterado
- Mensagens de Display em minúsculas, sem ponto final (G-ERR-01 / convenção thiserror)
- Validação de `paths` devolve `SshCliResult` (sem `anyhow`/`bail!`)
- `VpsRecord::validate*` → `Result<(), DomainError>`
- Secrets: material de chave por env é rejeitado (só XDG + CLI)
- `resolve_limit` de concorrência ignora store env
- `xdg_config_dir` usa `SshCliError::XdgDirectory`

### Segurança
- Nenhum material secreto no Display de erro das ops de crypto
- Fail-closed no store env `SSH_CLI_SECRETS_KEY*`

### G-DOM: tipos de domínio chrono/uuid/rust_decimal/url

### Adicionado
- Quatro crates de domínio (coordenadas): `chrono` 0.4.45 (`serde`+`clock`), `uuid` 1.24 (`v4`+`v7`+`serde`), `rust_decimal` 1.42 (`serde-with-str`+`macros`), `url` 2.5 (`serde`)
- `src/domain/` dividido (SRP): `error`, `names`, `ports`, `limits`, `command`, `time`, `ids`, `http_url`, `money`
- `Rfc3339Utc` / `AddedAt` / `CreatedAt`: newtype de `DateTime<Utc>`
- `added_at` do VPS + timestamps ACME
- `HttpsUrl` / `AcmeOrderUrl`: parse só HTTPS para retomar ordem ACME (XDG)
- `CorrelationId` (v4) / `BatchRunId` (v7): campo `batch_run_id` no JSON batch multi-host
- `Money<C: Currency>`: dinheiro decimal pronto para biblioteca (não ligado ao VPS)
- Gates: `tests/gaps_v053_domain_types.rs` + roundtrips de proptest (G-DOM-07/09)

### Alterado
- Schemas batch (`health-check-batch`, `exec-batch`, `scp-batch`) exigem `batch_run_id` (UUID v7)
- Import valida `added_at` como RFC 3339 (sem checagem só de tamanho)

### Segurança
- Sem `Local::now` nas fontes de produto
- Sem `serde-float` em decimais
- URLs ACME rejeitam `http`/`data`/`javascript`

### G-TLS produto: rustls / SSH-over-TLS / mTLS / ACME

### Adicionado
- Feature `tls` (padrão): `rustls` ≥ 0.23.18 + `aws_lc_rs`, `tokio-rustls`, `webpki-roots`, `rustls-pki-types`, `instant-acme`
- `CryptoProvider::install_default`: no `main` do binário (só aws_lc_rs, bibliotecas nunca reinstalam)
- Builder de `ClientConfig` (`src/tls/client_config.rs`) com webpki-roots + certificado de cliente mTLS opcional
- SSH-over-TLS: `ConnectionConfig.tls` / campos VPS `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key`
- Dial TCP → rustls → `russh::connect_stream`
- CLI `tls`: `provider`, `paths`, `mtls import|list|show|remove`, `acme account create|show`, `acme issue --print-challenge`, `acme complete`, `acme status|list`
- Layout XDG: `tls/mtls/<name>/`, `tls/acme/account.json`, `tls/acme/<domain>/` (segredos 0o600)
- Atualizações da suíte residual em `tests/gaps_v052_tls_policy.rs`

### Alterado
- `deny.toml`: permite `rustls` de produto
- Mantém os bans de `openssl*`, `native-tls`, `libssh2-sys`, `ring`
- Permite a licença `CDLA-Permissive-2.0` (webpki-roots)
- PEM via `rustls-pki-types::PemObject` (sem `rustls-pemfile` sem manutenção)

### G-TLS / política rustls — sessão anterior

### Adicionado
- `src/ssh/connect.rs`: `build_ssh_client_config` único + helpers de dial Happy Eyeballs (G-TLS-07/09)
- Suíte residual: `tests/gaps_v052_tls_policy.rs`, bans de lockfile/deny, sem flate2, política em SECURITY/README (G-TLS-03)
- SECURITY Política de transporte e crypto (G-TLS): SSH ≠ TLS
- Aws-lc-rs
- Rustls-only futuro se houver HTTP
- Só gates locais

### Alterado
- Compressão SSH só `none`: os algoritmos preferidos nunca oferecem zlib (G-TLS-04)
- Features do russh: remove `flate2` (G-TLS-05)
- Mantém só `aws-lc-rs`
- `deny.toml`: ban de `openssl`, `ring`, `rustls` além de `openssl-sys` / `native-tls` / `libssh2-sys` (G-TLS-02)
- README / CROSS_PLATFORM / RELEASE_CHECKLIST / llms: superfícies da política de crypto (G-TLS-01/06/08/11/12)

### Segurança
- Sem stack TLS de produto
- Sem OpenSSL/`native-tls`/`ring`/`rustls` no grafo de dependências
- Sem OTEL de produto
- Secrets continuam em `SecretString` + AEAD XDG

### Sistema de Tipos

### Adicionado
- Newtypes de domínio (G-TYPE-01…20): `src/domain/` com `VpsName`, `SshHost`, `SshUser`, `SshPort(NonZeroU16)`, `TimeoutMs`, `HostTag`, `CharLimit`, `RemoteCommand`, `KeyPath`, `BindPort` (efêmera ≠ SSH)
- `ssh/session_io.rs`: helpers `truncate_utf8` / UTF-8 com teto extraídos (G-TYPE-14)
- Testes de layout zero-cost para o nicho de `SshPort` / `Option<SshPort>`

### Alterado
- `VpsRecord` / `ConnectionConfig`: campos com prova de domínio
- `try_new` substitui o `new` infalível
- `HostSelection`: `VpsName` / `HostTag` (parse na fronteira da CLI)
- `ExecOptions` / `ScpOptions`: `Option<TimeoutMs>`
- Steps são `Vec<RemoteCommand>`
- Portas da CLI: `value_parser!(u16).range(1..=65535)` em VPS add/edit e na porta remota do tunnel
- Bind local ainda aceita `0`
- `paths::validate_and_normalize` → `Result<VpsName>`: (prova mantida)
- Import JSON: host/user vazios rejeitados
- Constrói via `VpsRecord::try_new`

### Segurança
- Checagem de material de auth centralizada (`secret_nonempty`)
- Sem secrets em mensagens de erro de domínio
- Sem OTEL de produto

### Notas de sessão (validação / serde / componentização)

### Adicionado
- Pipeline de validação (G-SERDE-01…14): `validator` 0.20 + `serde_with` 3 + `serde_path_to_error` + `serde_ignored`
- Módulo `src/validation.rs` (parse → serde → validate → domínio)
- Tags de host no wire JSON de agente (G-SERDE-06): `tags` nos DTOs de list/export/import com testes de round-trip
- Validação estrutural do config no load (G-SERDE-04): host/user vazios, porta 0, limites de caracteres, charset de tags
- Alvo de fuzz: `import_envelope` para a desserialização do import (G-SERDE-12)
- `ssh/connection.rs`: ConnectionConfig extraído (G-COMP-R)
- `cli/tests.rs`: testes unitários da CLI extraídos do monólito (G-COMP-R)

### Alterado
- deny_unknown_fields: no TOML `ConfigFile` / `VpsRecord` (G-SERDE-05)
- Import JSON continua Must-Ignore com warn (G-SERDE-14)
- Fan-out SCP multi-host: usa `Arc<ScpOptions>` (G-MEM-SCP)
- `apply_scp_options` recebe `&ScpOptions`
- Actions de CI pinadas por SHA de commit: (G-PROC-PIN)
- Deps de serde com caret: `serde = "1"`, `serde_json = "1"`

### Segurança
- Sem telemetria de produto (OTEL continua proibido)
- Secrets ficam em `SecretString`
- A validação nunca registra secrets

### Alterado
- O1–O6 + processo (obrigatório): `--fail-fast`
- Tags de host
- Multi-cmd `--step` na mesma sessão
- SCP `--scp-file-concurrency`
- Arc de options no fan-out
- Proptest/fuzz
- CI miri/geiger/sbom
- `scripts/release_attest.sh`
- Zero telemetria de produto
- Componentização profunda (G-COMP-05 / G-COMP-06a–d / G-CLOSE-09 / G-DRY-01 / G-EN-R01): extraídos `vps/exec_ops.rs` (exec/sudo/su + DRY `finish_execution_output`)
- `ssh/scp_wire.rs`
- `scp/{mod,batch}.rs`
- `output/{mod,batch}.rs`
- `cli/{mod,dispatch}.rs`
- `commands/*` com reexports reais
- Renomes EN residuais
- OPEN de segurança de produto permanece 0
- Monólitos inventoriáveis fatiados
- Componentização (G-COMP-02…04): extraídos `vps/doctor.rs`, `vps/import_export.rs` e `vps/health.rs` do monólito `vps` (`mod.rs` ~2428 → ~1698 LOC)
- Reexport de `HostHealthResult` / `run_health_check`

### Segurança
- Meta-auditoria de fechamento (G-CLOSE): casts de doctor/concurrency/SCP via `TryFrom`
- `forbid(unsafe_code)` nos módulos puros restantes
- Extração `vps/selection.rs` (SRP)
- Reexecução context7-cli + docsrs-cli + duckduckgo para conformidade da skill
- Auditoria de segurança de desenvolvimento (G-SECDEV): secrets atravessam a fronteira CLI como `SecretString` (`read_secret_stdin` + overrides de exec/scp/tunnel/health)
- Módulos puros com `#![forbid(unsafe_code)]`
- Deny de `clippy::mem_forget` + unsafe sem SAFETY / multi-op
- Mapa STRIDE + preferência CVSS v4 em `SECURITY.md` (+ pt-BR)
- Auditoria de segurança defensiva (G-SEC): `deny(unsafe_op_in_unsafe_fn)`
- `overflow-checks` em release
- Comparação constant-time de fingerprint TOFU
- Caminhos de produto sem `.unwrap`/`.expect`/`unreachable!` em parsers CLI, admissão de concorrência e ramos single-host
- Porta de import via `u16::try_from`
- `SshCliError` `#[non_exhaustive]`
- Modelo de ameaça em `SECURITY.md` (+ pt-BR)
- Job CI `cargo deny check` (`deny.toml`)

### Adicionado
- Auditoria de retry (G-RETRY): classificação tipada de erros (`ErrorClass` / `ErrorLayer` / `RetryKind`, `is_retryable` / `is_permanent` / `suggestion`) em `SshCliError`
- `retry::RetryConfig` nomeado com backoff full-jitter e defaults de agente (máx. 2 retries no exit 74)
- Envelope JSON com `error_class`, `retryable`, `suggestion` + schema
- Auto-retry in-process de ops remotas não idempotentes permanece desligado (agente reinvoca o processo)

### Corrigido
- Auditoria de rede (G-NET): dial SSH com DNS assíncrono + corrida multi-endereço Happy Eyeballs (`net::dial_tcp` + `russh::client::connect_stream`)
- `TCP_NODELAY` e keepalives SSH (`15s` / máx. `3`)
- Carga de chave privada e TOFU de known_hosts em `spawn_blocking`
- Accept do tunnel resiste a erros transitórios e aplica nodelay nos forwards locais

### Alterado
- Auditoria de hardcode (G-HC): módulo central `constants` (nomes XDG, env keys, identidade do app, defaults de rede, timing de processo, AEAD/keyring)
- Helper único `paths::xdg_config_dir()`
- Sem segredos/URLs de produto no binário
- Hosts continuam no registry/CLI

### Corrigido
- Auditoria de processos externos (G-PROC): probes `git` em `build.rs` com `Stdio` explícito (null/piped)
- Comandos remotos rejeitam NUL antes do packing de exec SSH
- Fixtures de teste `ssh-keygen` usam argv direto + stdio explícito e fazem skip se o binário estiver ausente
- Docs: política de fronteira de processo em CROSS_PLATFORM / AGENTS (sem spawn local OpenSSH, MSRV ≥ 1.77.2 BatBadBut, packing remoto `sh -c` só no host alvo)

### Adicionado
- Concorrência multi-host com bound (modus operandi): `health-check|exec|sudo-exec|su-exec|scp --all` faz fan-out com `Semaphore` + `JoinSet` (cap de `--max-concurrency` / `SSH_CLI_MAX_CONCURRENCY` / fórmula auto CPUs×RAM, clamp 1..=64)
- JSON batch: `health-check-batch` / `exec-batch` / `scp-batch` (`docs/schemas/*-batch.schema.json`)
- Forwards de accept do tunnel usam o mesmo gate
- Seleção seletiva `--hosts a,b,c`: mesmo fan-out e JSON batch que `--all` (mesmo com um nome)
- Unificado via `HostSelection` + `resolve_host_jobs`
- SCP multi-arquivo (single-host, G-PAR-47): uma sessão SSH e transfers seriais (auth uma vez)
- SCP multi-host × multi-arquivo (G-PAR-48): `scp upload --all f1 f2 … REMOTE_DIR`, bound por sessão host
- Arquivos seriais na sessão
- O download grava sob `LOCAL_DIR/<host>/`
- TOFU flock (G-PAR-49): mutações de `known_hosts` com lock exclusivo + reload-merge
- `vps doctor --probe-ssh [--hosts a,b]`: um único root JSON `event: vps-doctor` com `local` + `ssh_probe` opcional (sem dual roots)
- `map_bounded` cancel: para admissão em SIGINT/SIGTERM
- `force_exit` aborta JoinSet
- Logs de debug de `available_permits`
- Span `fan_out_unit` por task (G-PAR-52)
- Docs/skills de agente: frota multi-host + multi-arquivo / cartesiano SCP + envelope doctor

### Alterado
- Path SCP (validação e pós-download) usa `tokio::fs` / `spawn_blocking` (não bloqueia workers sob fan-out)
- `scripts/dist_multiarch.sh` suporta `PARALLEL_JOBS` (default 2) via `xargs -P`


## [0.5.1] - 2026-07-17

### Corrigido
- Roundtrip export/import agent-first: corpo default de `vps export` é TOML mesmo em non-TTY
- JSON só com `--json`
- Import aceita TOML (chaves EN+PT) e envelopes JSON `vps-export` (GAP-AUD-001/022)
- Wire dual-read: deserializa EN + aliases PT legados
- Serializa chaves em inglês
- Schema v3
- Default `added_at` quando ausente (GAP-AUD-002/021)
- Substitui a nota de wire 0.5.0 (chaves PT só via `serde(rename)`)
- JSON de `secrets init` / `reencrypt`: (`event: secrets-init|secrets-reencrypt`) via `--json` ou `--output-format json` (GAP-AUD-003)
- Erro de comando vazio é técnico em inglês (`empty command`) em qualquer locale (GAP-AUD-004)
- Caminhos de sucesso CRUD/connect/import emitem JSON estruturado quando o formato é JSON (GAP-AUD-008)
- Mensagem SCP remoto ausente normalizada para `file not found: <path>` (GAP-AUD-025)
- EC 66 mantido
- Erros de parse TOML no import mapeiam para sysexits 65 (`TomlDe`) (GAP-AUD-012)
- Exit de `SshAuthentication` alinhado a 77 (GAP-AUD-020)
- Timeouts `< 1000` ms emitem warning em stderr (GAP-AUD-009)
- `--include-secrets` em pipe/non-TTY exige `--output` ou `--i-understand-secrets-on-stdout` (GAP-AUD-011)
- Doctor `secrets_plaintext_opt_out` é JSON bool (GAP-AUD-013)

### Adicionado
- Flags CLI: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (camadas env depreciadas, ainda funcionam) (GAP-AUD-006)
- Evento `secrets-key-auto-created` quando a primary-key é provisionada na primeira gravação (GAP-AUD-007)
- Tunnel `--bind` (default `127.0.0.1`) (GAP-AUD-018)
- Warning em stderr de password em argv (GAP-AUD-010)

### Alterado
- Versão 0.5.0 → 0.5.1
- Tracing / identificadores residuais padronizados em inglês (GAP-AUD-005)
- Aliases de tipo em português no módulo `erros` marcados como deprecated (GAP-AUD-017)

### Notas
- Sem publish crates.io/GitHub sem OK explícito do maintainer
- Contratos reais de transferência SCP de 0.5.0 §1.1 não devem regredir


## [0.5.0] - 2026-07-15

### Corrigido
- CRÍTICO: `secrets init --force` reencripta hosts existentes e grava `secrets.key.bak` (GAP-AUD-SEC-001)
- Doctor `permissions` em inglês (`"missing"`)
- Mensagens técnicas, help clap e identificadores residualmente em EN
- Nomes de VPS com whitespace interno rejeitados (GAP-AUD-VAL-001)

### Alterado
- Semver 0.5.0 por renomeações de API em inglês
- Wire TOML ainda usava chaves PT via `serde(rename)` nesta release (supersedido em 0.5.1 por serialize EN + dual-read EN/PT, schema v3)
- `secrets init` / `reencrypt` via `Message` i18n

### Notas
- Sem publish crates.io/GitHub sem OK explícito


## [0.4.2] - 2026-07-15

### Corrigido
- Tunnel porta efêmera: (`local_port=0`): após bind, JSON/banner reportam a porta atribuída pelo SO via `local_addr()` (nunca `0` pós-bind) (GAP-SSH-TUN-003)
- Schema `local_port.minimum` = 1
- SCP remote missing: agora sai com 66 `ArquivoNaoEncontrado` (paridade com missing local) em vez de 74 `CanalFalhou` quando o OpenSSH reporta `No such file` / `not found` (GAP-SSH-IO-010)
- Erros de protocolo/permissão permanecem 74

### Adicionado
- `vps export --json` envelope agent-first: `event: "vps-export"`, hosts redacted por padrão, sem `sshcli-enc:` para secrets vazios (GAP-SSH-UX-001 / paridade EXP-001)
- Schema `docs/schemas/vps-export.schema.json`
- Embed de commit hash no pack crates.io: `build.rs` com precedência env → `.commit_hash` → git → `unknown` (GAP-SSH-REL-007)
- e2e oficial E15 (tunnel porta 0) + E16 (symlink) + E13 exige exit 66
- Política ENV-001/fail2ban no header do script
- Suíte `tests/gaps_v042_integration.rs`

### Alterado
- Versão 0.4.1 → 0.4.2
- Docs/skills: tunnel continua com args posicionais
- Porta `0` = efêmera
- Confiar em `local_port` do JSON
- Nunca inventar `--local-port` (GAP-SSH-DOC-042)

### Segurança
- Ban TCP na VPS após e2e de auditoria foi fail2ban por senhas erradas intencionais (ENV-001), não TUN-003
- Whitelist/ignoreip é operação
- A CLI one-shot não abre sshd permanentemente
- Sem telemetria

### Notas
- CLI one-shot: nascer → executar → morrer
- Contratos agent aditivos (PATCH)


## [0.4.1] - 2026-07-15

### Corrigido
- Export redacted com secret vazio: não emite mais ciphertext `sshcli-enc:v1:…` para senha `""` (GAP-SSH-EXP-001)
- Secrets vazios serializam como strings vazias, então o import entre máquinas de esqueletos continua honesto
- Deadline do tunnel: após bind local não retorna mais exit 74 quando o agente já recebeu `tunnel_listening` (GAP-SSH-TUN-002)
- Timeout pré-bind permanece 74

### Adicionado
- Paridade de flags auth em `tunnel`: `--password-stdin`, `--key-passphrase`, `--key-passphrase-stdin` (GAP-SSH-CLI-005)
- Paridade de flags auth em `health-check`: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` (GAP-SSH-CLI-006)
- Campo JSON SCP `event: \"scp-transfer\"` + schema obrigatório (GAP-SSH-IO-009)
- Suíte `tests/gaps_v041_integration.rs`
- `health-check` honra `--replace-host-key` global e envelope JSON de erro com `--json`

### Alterado
- Versão 0.4.0 → 0.4.1
- Docs/skills de product line com paridade auth e event scp-transfer

### Segurança
- Se instalou 0.4.0 do crates.io: export redacted podia mostrar ciphertext falso de senha vazia
- Tunnel podia emitir `ok:true` e sair 74
- Atualize para 0.4.1
- Sem telemetria

### Notas
- CLI one-shot: nascer → executar → morrer
- Contratos agent aditivos apenas (PATCH)


## [0.4.0] - 2026-07-15

### Corrigido
- Protocolo wire SCP: quebrado no crates.io 0.3.9 (header com `\\n` literal em vez de newline real `0x0a`, ACK/EOF com data vazia em vez do byte `0x00`, status remoto não validado, download com header/terminador incorretos), SCP-010..013
- Escape shell do path remoto SCP para espaços e meta-caracteres (SCP-014)
- Unit tests não cristalizam mais o header quebrado (SCP-015)
- Download não deixa arquivo final parcial em falha: grava `{path}.ssh-cli.partial` e faz rename atômico (SCP-022)
- Mode/times aplicados no partial antes do rename (SCP-022b)
- Upload não carrega o arquivo inteiro em RAM (`fs::read`)
- Stream em chunks de 32 KiB (SCP-018)
- `scp --json` habilita envelope JSON de erro em stderr (paridade com tunnel, IO-007b)
- Mensagens de validação file-only do SCP em i18n EN/PT (SCP-020b)

### Adicionado
- E2E oficial E10–E14 SCP em `scripts/e2e_real_ssh.sh` (upload, download, `cmp`, remoto ausente, preserve mode/mtime) (SCP-016, SCP-023)
- Paridade de flags scp com exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` (SCP-017)
- JSON estruturado de sucesso SCP + `docs/schemas/scp-transfer.schema.json` (IO-007, SCP-021)
- Preserve mtime/mode bi-direcional: remoto `scp -tp`/`-fp`, linha `T` + parse mode `C`, set_permissions + set_times (SCP-023/023b, e2e E14)
- `tunnel --json` emite evento estruturado `tunnel_listening` após bind local (IO-008)
- Mensagens i18n EN/PT de sucesso SCP (SCP-020)
- Suíte `tests/gaps_v040_integration.rs` (TEST-004)

### Alterado
- Versão 0.3.9 → 0.4.0
- Docs de product line documentam somente arquivos regulares (sem `-r` / sem SFTP) e a regressão wire SCP de 0.3.9 (DOC-004, SCP-019, REL-004)
- Honestidade da raiz (SECURITY 0.4.x atual, INTEGRATIONS superfície real 0.4.0, CONTRIBUTING gaps_v040) (DOC-004b)
- Honestidade de `docs/*`: AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION/TESTING/RELEASE_CHECKLIST/CROSS_PLATFORM + índice de schemas cobrem SCP file-only, partial, stream 32 KiB, preserve, `scp --json`, `tunnel --json` / `tunnel_listening` e aviso wire 0.3.9 (DOC-004c)
- Honestidade de `skills/*`: skills bilíngues + evals ensinam SCP file-only, JSON scp-transfer, `.ssh-cli.partial`, stream 32 KiB, preserve mtime/mode, tunnel `--json` / `tunnel_listening`, matriz de flags de timeout (DOC-004d)
- Adicionado `docs/schemas/tunnel-listening.schema.json` para o contrato de agente IO-008
- `scp` honra `--replace-host-key` global e `--output-format json` global

### Segurança
- Se você instalou 0.3.9 do crates.io e usou `scp`: essa release anunciava SCP, mas o wire era inoperante (upload frequentemente gerava arquivo remoto 0 bytes ou timeout)
- Atualize para 0.4.0
- Sem telemetria

### Notas
- CLI one-shot: conectar → transferir → desconectar → sair
- Arquivos grandes: aumente `--timeout` (cobre connect + transferência completa)


## [0.3.9] - 2026-07-15

### Corrigido
- Residuais da auditoria pós-0.3.8: LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL-003, CHG-001
- Tracing default error (agent-first)
- `-v` ativa debug (LOG-001)
- stderr JSON sem prosa INFO por omissão (LOG-001)
- VPS só-chave: `password: null` no JSON (não a máscara de três asteriscos) (JSON-001)
- `health-check --timeout <ms>` alinhado ao exec (CLI-004)
- Docs de product line em 0.3.9 e comportamentos residuais documentados em README, `llms*.txt`, INTEGRATIONS, `docs/*` (HOW_TO_USE, COOKBOOK, MIGRATION, TESTING, CROSS_PLATFORM, AGENTS, schemas) e skills (auditoria profunda DOC-003)
- Âncoras de compare do CHANGELOG para 0.3.8/0.3.9 (CHG-001)
- `deny.toml` documenta warns multi-version esperados sem ignore de CVE (DENY-002)
- `docs/schemas/vps-show.schema.json` permite `password` com tipo `string | null` (paridade JSON-001)
- As aberturas de link cruzado em `docs/*.pt-BR.md` usam narrativa em português ("Leia este documento em inglês")
- `docs/RELEASE_CHECKLIST.md` + `docs/RELEASE_CHECKLIST.pt-BR.md` bilíngues com gates residuais LOG/JSON/CLI/DOC/DENY/REL/CHG
- Os testes DOC-003 cobrem as checklists e a janela de `password` nulo no schema
- Skills EN/PT consolidadas como fórmulas operacionais imperativas (LOG/JSON/CLI, envelope de erro, quiet, key-passphrase-stdin, port, completions completos) sem histórias de changelog de versão
- Higiene de exposição SEC-001..003: ignore `.setting.cyber/`, E2E recusa grok config no repo, docs usam `demo-password-not-real`

### Adicionado
- Suíte `tests/gaps_v039_integration.rs` para gaps residuais de auditoria (incl. SEC-001..003)

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
- Suíte `gaps_v038_integration` 1:1 (TEST-004)
- Version string com `-dirty` se tree suja (REL-002)
- Inventário `gaps.md` versionado
- Checklist `docs/RELEASE_CHECKLIST.md`

### Segurança
- Upgrade russh 0.62.2 (piso ≥0.60.3)
- Remove pins COMPAT RC (DEP-002)
- `cargo deny` sem waivers CVE/yanked
- Remove license morta Unicode-DFS-2016
- Gate install exige russh patched
- Permite primefield estável
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
- Todos os gaps listados no `gaps.md` da época (VAL/IO/TUN/SCP/STATE/PERM/CLI/TEST/EXIT/SEC/DEP/IMP)
- Write-path de domínio: `validar_e_normalizar`, porta 1..=65535, chave existente (VAL-001..003)
- I/O: `--output-format` no CRUD VPS, `health-check --json`, envelope JSON de erro, `--quiet` silencia sucesso humano, `println!` só em `output` (IO-001..005)
- Tunnel: `--timeout-ms` cobre connect + loop (TUN-001)
- SCP valida arquivo local antes do connect (SCP-001)
- `vps remove` limpa `active` órfão
- Lock `0o600` (STATE-001, PERM-001)
- `su-exec --password-stdin`
- Conflitos clap password/*_stdin
- Completions EPIPE seguro (CLI-001..003)
- Testes de sinais `#[serial]`
- Snapshot help
- Assert real de abort (TEST-001..003)
- Falha de comando remoto → exit do processo `1` (não o código remoto) (EXIT-001)
- Senha sudo/su no stdin do canal, não na argv
- Máscara sempre de três asteriscos (SEC-001, SEC-002)
- Import redacted com UX + `--allow-incomplete` (IMP-001)
- `cargo deny` verde com política de pins datada (DEP-001)

### Alterado
- Versão 0.3.6 → 0.3.7
- Quebra de contrato (agentes): senhas longas não expõem 12+4
- Exit remoto ≠0 vira processo `1` com `remote_exit_code` no envelope
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

### Corrigido
- Grafo crypto de `cargo install`: pin `primefield`, `primeorder`, `ecdsa`, `pkcs5`, `russh = 0.60.0` exato (GAP-014)
- Packing de `sudo-exec` com `sh -c`  (GAP-005)
- Escrita atômica de `config.toml` com tempfile + fsync + flock (GAP-007)
- Host key TOFU via `known_hosts` XDG (GAP-008)
- Dual `max_command_chars` / `max_output_chars` (GAP-004)
- Abort remoto best-effort no timeout (GAP-006)
- Validação de credencial: password ou key obrigatório (GAP-011)

### Adicionado
- Auth por chave privada (`--key`, `key_path`) via russh `load_secret_key` (GAP-002)
- `su-exec` one-shot consumindo `senha_su` (GAP-003)
- Segredos via stdin (`--password-stdin` e pares sudo/su) (GAP-009)
- `vps doctor`, `vps export`, `vps import` (GAP-012)
- Tunnel com `--timeout-ms` obrigatório (GAP-010)
- `--disable-sudo`, `--description`, `--replace-host-key`
- Schema v2 multi-host XDG
- Gate de install: `scripts/verify_install_resolve.sh`

### Alterado
- Timeout default 60000 ms
- `directories` 5 → 6 (GAP-013)
- Versão 0.3.3 → 0.3.4
- Dual license MIT OR Apache-2.0


## [0.3.3] - 2026-07-15

### Alterado
- Migração de ownership e repositório para `danilo-aguiar-br` após ban da conta GitHub anterior
- `repository` / `homepage` apontam para `https://github.com/danilo-aguiar-br/ssh-cli`
- Metadados de autor atualizados para `Danilo Aguiar <daniloaguiarbr@proton.me>`
- Workflows de runner hospedado e badges de pipeline removidos

### Nota
- crates.io já tinha versões até `0.3.2` da conta anterior
- Este release é o primeiro sob o novo owner


## [0.2.1] - 2026-04-16

### Corrigido
- Pin `elliptic-curve = "=0.14.0-rc.30"` para corrigir falha de `cargo install ssh-cli`


## [0.2.0] - 2026-04-15

### Adicionado
- Fix de piping de senha sudo-exec com `printf '%s\n'`
- Overrides de runtime em exec/sudo-exec/scp/tunnel
- Aliases camelCase para LLMs


## [0.1.0] - 2026-04-14
- Release inicial

[Unreleased]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.6...HEAD
[0.5.6]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.5...v0.5.6
[0.5.5]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.4...v0.5.5
[0.5.4]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.3...v0.5.4
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
