# Guia de agentes para ssh-cli


> Corte o desperdício de RAM e mantenha o SSH em vários hosts sob controle do agente.

- Leia este documento em [Inglês (en)](AGENTS.md)
- Combine com [../INTEGRATIONS.pt-BR.md](../INTEGRATIONS.pt-BR.md) e [../skills/ssh-cli-pt/SKILL.md](../skills/ssh-cli-pt/SKILL.md)
- Linha de produto: 0.5.6
- Este guia é um híbrido: uma abertura persuasiva que vende o modelo, depois um contrato imperativo a partir da seção Contrato do agente
- Descubra a superfície viva com `ssh-cli commands`, `ssh-cli schema` e `ssh-cli doctor` (alias raiz de `vps doctor`)
- `ssh-cli --help` imprime os exit codes em `Exit status:` e `Examples:` copiáveis sem custo de rede, e o `--help` das 48 folhas acrescenta ao menos um exemplo em cada enquanto o `-h` continua curto


## Por quê

### Troque processos SSH Node de longa duração por um binário que morre após rodar
- Processos SSH persistentes de longa duração queimam RAM enquanto seguram sockets ociosos
- Um host por processo daemon multiplica a contagem de processos para N servidores
- Um único binário Rust de interface de linha de comando (CLI) com armazenamento XDG Base Directory (XDG) para vários hosts elimina essa proliferação
- Agentes ganham JSON determinístico e sysexits sem custo extra de protocolo


## Economia

### Meça o ganho operacional
- Toda chamada inicia, responde e termina, então nenhum processo fica residente entre as chamadas, e `ssh-cli --version` confere a instalação sem nenhuma chamada de rede
- A memória do processo volta ao sistema após cada comando
- Não há custo de ambiente Node nem processo gerenciador SSH permanente
- Uma instalação serve Claude Code, Cursor, Windsurf, Codex e agentes de shell


## Soberania

### Mantenha credenciais e confiança de host locais
- Os hosts vivem no XDG sem proliferação de `.env`
- Prefira chaves privadas e segredos por stdin a senhas coladas no chat
- Os segredos são cifrados em repouso com ChaCha20-Poly1305 quando a chave primária existe, gerencie-a com `secrets status`, `secrets init` e `secrets reencrypt`
- O `known_hosts` de confiança no primeiro uso (TOFU) dificulta um ataque silencioso de intermediário
- A elevação pode ser desligada sempre que um fluxo precisa ficar sem privilégio
- O diretório de configuração nasce 0700 e o conjunto de caracteres do nome de host bloqueia truques de terminal e metacaracteres de shell
- PROIBIDO: registrar em log a chave primária, senhas de host ou segredos decifrados


## Superpoderes para agentes de IA
- Uma chamada, um objeto JSON, um exit code: sem raspar tela, sem prosa para adivinhar
- Prova de onde rodou: `target_resolved` e `target_source` voltam em toda chamada de um host
- Frota em um processo: `--all`, `--hosts` e `--tags` rodam sessões concorrentes limitadas
- Ensaio antes do risco: `--dry-run` pré-visualiza oito operações e ecoa o alvo
- Economia de tokens: oito flags de redução cortam o payload antes de ele ser serializado
- Mudança segura: troque a autenticação, renomeie e limpe campos em uma chamada, com `updated` como recibo
- Autodescritivo: `commands --json` publica todo argumento e `schema` publica todo formato de saída


## Agentes e orquestradores compatíveis
- Claude Code com o pacote de skill incluído
- Cursor com ferramentas de shell ou de agente
- Ferramenta de shell do Windsurf
- Ferramenta de shell do Codex CLI
- Ferramenta de shell do OpenCode
- Aider, Continue, Gemini CLI, OpenHands e bash ou zsh genéricos


## Inventário de comandos (árvore completa)
- As 48 folhas estão escritas por inteiro, não em notação de chaves
- Um agente que busca por `tls acme account create` precisa achar esse texto exato
- Descubra a mesma árvore em tempo de execução com `ssh-cli commands` (`event: "commands"`)

| Superfície | Comandos |
| --- | --- |
| `vps` | `ssh-cli vps add` `ssh-cli vps list` `ssh-cli vps remove` `ssh-cli vps rename` `ssh-cli vps edit` `ssh-cli vps show` `ssh-cli vps path` `ssh-cli vps doctor` `ssh-cli vps export` `ssh-cli vps import` |
| Sessão | `ssh-cli connect` |
| Exec | `ssh-cli exec` `ssh-cli sudo-exec` `ssh-cli su-exec` |
| `scp` | `ssh-cli scp upload` `ssh-cli scp download` (somente arquivos regulares) |
| `sftp` | `ssh-cli sftp upload` `ssh-cli sftp download` `ssh-cli sftp ls` `ssh-cli sftp mkdir` `ssh-cli sftp rmdir` `ssh-cli sftp rm` `ssh-cli sftp stat` `ssh-cli sftp rename` |
| Rede | `ssh-cli tunnel` (quatro modos: local, `--reverse`, `--socks5`, `--remote-socket`) `ssh-cli health-check` |
| `secrets` | `ssh-cli secrets status` `ssh-cli secrets init` `ssh-cli secrets reencrypt` |
| Descoberta | `ssh-cli completions` `ssh-cli commands` `ssh-cli schema` `ssh-cli doctor` (alias raiz de `ssh-cli vps doctor`) |
| `locale` | `ssh-cli locale show` `ssh-cli locale set` `ssh-cli locale clear` |
| `tls` | `ssh-cli tls provider` `ssh-cli tls paths` |
| `tls mtls` | `ssh-cli tls mtls list` `ssh-cli tls mtls import` `ssh-cli tls mtls show` `ssh-cli tls mtls remove` |
| `tls acme` | `ssh-cli tls acme account create` `ssh-cli tls acme account show` `ssh-cli tls acme issue` `ssh-cli tls acme complete` `ssh-cli tls acme status` `ssh-cli tls acme list` |

- Toda flag de toda folha está tabelada em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md), e esta página nomeia as que um agente busca por tarefa
- Registro e edição compartilham as flags de registro: `--host`, `--port`, `--user`, `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--sudo-password`, `--su-password`, `--tag`, `--timeout`, `--max-chars`, `--max-command-chars`, `--max-output-chars`
- O Transport Layer Security (TLS) no registro do host é definido com `--tls`, `--tls-sni`, `--tls-client-cert` e `--tls-client-key`, e `vps edit --no-tls` desliga de volta
- O ponto de acesso do agente é definido com `--use-agent` e `--agent-socket`, e limpo com `--no-agent-socket`
- `vps add --check` roda um `health-check` logo após registrar, então um registro quebrado aparece na escrita
- As flags de limpeza do `vps edit` são `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-key`, `--no-tls-sni`, `--no-tls-client-cert` e `--no-tls-client-key`
- `vps import --file <PATH>` lê arquivo TOML ou envelope `vps-export`, e `--allow-incomplete` aceita registros sem campos opcionais
- Uma identidade de TLS mútuo (mTLS) é importada com `tls mtls import --cert CERT --key KEY --name NAME`
- Uma conta do Automatic Certificate Management Environment (ACME) é criada com `tls acme account create --contact mailto:ops@example.com`, e `--staging` aponta para o diretório de testes
- `tls acme issue --domain NAME --print-challenge` abre o DNS-01 e imprime o desafio, e `--staging` mantém o pedido fora de produção
- O cofre do sistema é selecionado por `--use-keyring` ou por `secrets init --keyring`, e os dois resolvem a mesma entrada com escopo de perfil
- Esse cofre é o Keychain no macOS, o Credential Manager no Windows e o Secret Service por zbus no Linux, e um cofre que não responde a uma escrita sai 69 em vez de um sucesso falso
- A chave primária é resolvida nesta ordem: `--secrets-key-file`, o cofre do sistema quando `--use-keyring` está ativo, `secrets.key` no diretório de configuração e, por fim, uma entrada deixada por um `secrets init --keyring` anterior
- Uma leitura sob `--use-keyring` cujo cofre não responde registra um aviso e cai para `secrets.key`, então só o caminho de escrita sai 69
- Um perfil tem uma chave primária só, então o `secrets init` sai 64 quando já existe chave em qualquer um dos dois armazenamentos, e o `--force` rotaciona e aposenta a outra fonte
- `doctor --probe-ssh` prova a conexão SSH, e `--hosts <LIST>` limita o `doctor` ou o `health-check` a esse subconjunto
- `--description` anexa um comentário de shell ao comando remoto empacotado, então a trilha de auditoria mora no próprio comando
- `--verbose` é a forma longa de `-v`, e o nível é repetível até `-vvv`

### Flags globais de destaque
- `--lang`, `-v`, `-vv`, `-vvv` (G14 graduado, G2 restrito à crate `warn,ssh_cli=*`), `-q`, `--config-dir`, `--no-color`, `--output-format`, `--json`
- `--disable-sudo`, `--replace-host-key`
- `--disable-sudo` tem dois sentidos: em `sudo-exec` e `su-exec` ele bloqueia a elevação naquela chamada, e em `vps add` e `vps edit` ele persiste `disable_sudo: true` no registro, `vps edit --enable-sudo` desfaz
- O sentido que persiste segue o subcomando e não a posição da flag, então `ssh-cli --disable-sudo vps edit <NAME> --port 22` também grava `disable_sudo` e o lista em `updated`
- O alias `--disableSudo` é aceito antes do subcomando e depois de todo subcomando exceto `vps add` e `vps edit`, que têm o seu próprio `--disable-sudo` e recusam o alias com exit 2 (conferido com `--help` nas 48 folhas)
- `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- `--timeout`, `--max-concurrency`, `--fail-fast`, `--scp-file-concurrency`, `--lock-wait-ms`
- Escreva as flags globais antes do subcomando, o binário também as aceita depois dele

### Redução de payload (0.5.4): não gaste tokens com dados que você vai descartar
- Oito flags globais moldam a resposta antes da serialização, então o envelope grande nunca é construído
- `--select <PATHS>` (alias `--fields`) mantém só estes caminhos pontuados em cada registro, ou no objeto único
- `--filter key=value`, `key!=value` ou `key~substring` mantém registros que casam, repetível e combinado com AND
- Um predicado malformado é rejeitado na análise com exit 64, então um erro de digitação nunca é confundido com resultado vazio
- `--limit N` emite no máximo N registros e é distinto dos limites de consulta de cada comando
- `--sort <PATH>` ordena de forma crescente por caminho pontuado, comparando números numericamente
- `--dedupe-by <PATH>` descarta registros posteriores que repetem o valor do caminho
- `--count-only` troca a coleção de registros por `{"count": N}`, contado depois de toda filtragem
- `--truncate-content <CHARS>` encurta textos longos por caracteres, nunca bytes, então o UTF-8 continua válido
- `--max-output-bytes <BYTES>` limita o envelope descartando registros finais, nunca cortando o texto JSON
- Prefira estas flags a passar o stdout por uma ferramenta JSON externa, porque o encadeamento paga o custo total de tokens primeiro
- O array puro de `vps list` não tem espaço para `agent_shape`, então o relatório da redução vai para o stderr em uma linha JSON com `"event":"agent-shape"` sempre que algo foi descartado ou truncado, e o `--quiet` a suprime
- Numa coleção a redução se descreve em `agent_shape` com `input_count`, `output_count` e `dropped`
- Toda flag alcança as coleções de registros: o array de `vps list`, `results` dos envelopes em lote e `entries` do `sftp ls`
- Só `--select` e `--truncate-content` reduzem um objeto único como `vps show`, `doctor`, `commands` ou `exec` em um host
- Num objeto único o `agent_shape` traz `selected`, `content_truncated` e `ignored`, e `ignored` nomeia cada flag de coleção que não tinha o que reduzir
- Um item de `vps list --select` é a forma reduzida que `ssh-cli schema vps-list` publica ao lado do registro completo, então ele valida contra o schema publicado
- `--select` nomeia campos dentro de cada registro, então `--select results` devolve objetos vazios e a chamada certa é `--select name,stdout`
- Os registros do lote de health-check trazem `name`, `status` (`ok` ou `error`), `latency_ms` e `error`, então filtre com `--filter status!=ok`

### Editando e renomeando um host (0.5.6)
- `vps rename <NAME> <NEW_NAME>` é a renomeação nativa e `vps edit <NAME> --name <NEW>` chega ao mesmo caminho
- Os dois verbos tomam o sujeito do argv, publicam `target_resolved` e `target_source: argv`, e nunca leem o marcador `active` como sujeito
- Os segredos guardados continuam legíveis porque o registro os sela sem dados associados, então o nome novo não muda nada neles
- Todo salvamento sela de novo com um nonce novo, o que é normal e nunca toca o texto em claro
- O desvio export, edição, import não é mais necessário para renomear, e ele descartava em silêncio a autenticação de agente porque o export não levava `use_agent` nem `agent_socket`
- O envelope `vps-renamed` traz `name`, `previous_name`, `active_rewritten` (true só quando o marcador apontava para o nome antigo), `updated`, `target_resolved` e `target_source`
- `vps edit --name` também emite `vps-renamed`, `vps-edited` só é emitido quando nada é renomeado e nunca traz `previous_name`
- `updated` lista os campos cujo valor guardado mudou, em ordem de aplicação, cada um no máximo uma vez, e está sempre presente
- Fica vazio quando todo valor pedido já coincidia ou quando a chamada só renomeou o host
- Uma edição que não muda nada é uma operação nula: exit 0, `vps-edited`, `updated: []` e o `config.toml` não é reescrito
- O exit 64 é reservado para uma chamada sem nenhum campo, renomeação ou flag de limpeza
- `vps rename` NÃO é idempotente: repeti-lo depois de um sucesso sai com 66 porque o nome antigo não existe mais, e `vps rename ghost ghost` também sai com 66
- Um destino já usado sai com 64 com `vps_duplicate`, um nome idêntico sai com 64 com `new name ... is identical to the current name` e um inválido também sai com 64
- Confirme o resultado com `vps show <NEW_NAME>` ou `vps list` em vez de repetir a chamada
- Flags de limpeza removem um campo: `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key`, `--no-key` e `--no-agent-socket`
- `--no-key` também limpa a passphrase, então um segredo que ninguém lê nunca fica para trás
- `--no-key` em um host cuja única autenticação é essa chave sai com 64 com `error_code` `domain_validation` e mensagem com prefixo `vps_auth:`
- `--no-agent-socket` sai com 64 quando o host ficaria sem ponto de acesso do agente em uma plataforma que não tem um padrão
- O lock da configuração espera um tempo limitado (`--lock-wait-ms`, padrão 5000, teto 300000) e depois sai com 69 com `unavailable`, que pode ser repetido
- `vps import` valida TLS como o `vps edit` (um certificado de cliente sem a sua chave sai com 64) e o envelope `vps-import` traz `replaced`, os nomes ordenados dos hosts que ele sobrescreveu, sempre presente
- Toda flag é descobrível: `ssh-cli commands --json` publica cada argumento com `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` e `hidden`

### Flags de recusa e de ensaio
- `--no-input` recusa ler o stdin e falha rápido em vez de bloquear esperando um humano ausente
- `--no-input` também proíbe segredos no stdin (`--password-stdin` e as outras flags `--*-stdin` saem com 64), então use-o em execuções que não passam segredo pelo stdin
- `--dry-run` imprime o plano de uma operação destrutiva e sai sem executá-la
- `--dry-run` é aceita só por `vps remove`, `vps import`, `vps edit`, `vps rename`, `sftp rm`, `sftp rmdir`, `secrets init` e `secrets reencrypt`
- Os dois ensaios de `sftp` nunca abrem conexão
- `vps import --dry-run` lista `hosts` com `name`, `incomplete` e `replaces_existing` mais `imported`, uma previsão, enquanto a execução real informa `replaced`
- `secrets init --dry-run` informa `hosts_to_reencrypt`, então leia-o antes de `--force`, e `secrets reencrypt --dry-run` informa `hosts`
- Para `vps remove`, `vps edit` e `vps rename` o plano também traz `target_resolved`, `target_source` (`argv`), `host_resolved` e `host_source` depois que a busca dá certo
- Um host ausente sai com 66 na execução real e no ensaio, e não cria diretório nem arquivo de lock
- Em qualquer outro lugar `--dry-run` é rejeitada com exit 64 em vez de aceita e ignorada, então um ensaio nunca é confundido com uma operação nula que já rodou


## Contrato do agente

### Obrigatório: processo e fluxos
- Invoque o `ssh-cli` como subprocesso e espere a saída (execução única)
- Leia o sucesso do stdout e a falha do envelope de erro no stderr
- Exceção: quando o próprio comando remoto falha, o stdout mantém o objeto do `exec` com o `exit_code` remoto, o stderr recebe um envelope `command_failed` com `remote_exit_code`, e a CLI sai com 1

### Obrigatório: saída e entrada de segredos
- Interprete o JSON do stdout quando `--json` ou `--output-format json` está definido, o JSON é automático quando o stdout não é TTY
- Interprete exatamente um objeto JSON em um caminho de sucesso (G8), as exceções são `exec --step`, que imprime um objeto por passo com `step` e `command`, e `tunnel --json`, que imprime `tunnel_listening` e depois `tunnel_closed`
- Trate o tracing do stderr como log, nunca como contrato
- Passe segredos por `--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin` e `--key-passphrase-stdin`

### Proibido: segredos no argv e com no-input
- Passar um segredo no argv, a CLI avisa no stderr quando um valor parecido com senha aparece ali
- Combinar `--no-input` com um segredo `--*-stdin`, essa chamada sai com 64
- `completions` imprime um script de shell e `vps export` em modo texto imprime um corpo TOML, todo outro sucesso é JSON

### JSON, erros e exit codes
- OBRIGATÓRIO: quando o modo de erros JSON está ativo, interprete o envelope de falha no stderr, ele tem sete campos: `exit_code`, `error_code`, `message`, `remote_exit_code`, `error_class`, `retryable` e `suggestion`
- Cinco estão sempre presentes, `remote_exit_code` e `suggestion` são opcionais, e o eco do alvo vem ao lado deles, veja `docs/schemas/error-envelope.schema.json`
- `error_class` é `transient`, `permanent`, `cancelled` ou `partial`, e `partial` marca uma distribuição por vários hosts em que alguns alvos tiveram sucesso
- Um erro de uso do clap (exit 2) e um `--filter` malformado (exit 64, uma falha antes do despacho) imprimem texto puro no stderr mesmo com `--json`, e esse texto segue o idioma do host, então passe `--lang en` para lê-lo em inglês, só erros de execução trazem o envelope, então não passe todo stderr para um leitor de JSON
- OBRIGATÓRIO: decida por `error_code`, o único discriminador estável, depois por `error_class` e `retryable`, nunca por `message`
- Uma falha parcial em vários hosts ainda imprime o envelope do lote no stdout, depois o stderr recebe `error_code` `partial_failure` com `error_class` `partial`, e o exit é 1
- OBRIGATÓRIO: espere o nível de tracing padrão error e use `-v`, `-vv` ou `-vvv` só para depurar
- PROIBIDO: depender do `RUST_LOG` do ambiente, ele é ignorado

### Idempotência

| Chamada | Comportamento ao repetir |
| --- | --- |
| `vps add` | Exit 64 `vps_duplicate`, o host existe |
| `vps edit` | Idempotente: exit 0 com `updated: []`, sem reescrita |
| `vps rename` | Não idempotente: exit 66 porque o nome antigo sumiu |
| `vps remove` | Não idempotente: exit 66 `vps_not_found` |
| `vps import` | Idempotente: exit 0 e `replaced` lista os nomes sobrescritos |
| `connect` | Idempotente: exit 0 |
| `locale set` e `locale clear` | Idempotente: exit 0 |
| `secrets init` | Exit 64 a menos que `--force`, a chave existe |
| `secrets reencrypt` | Seguro repetir, ele recifra com a chave atual |
| `exec`, `sudo-exec`, `su-exec` | Idempotente só até onde o comando remoto é, o binário nunca repete sozinho |
| Transferências `scp` e `sftp` | Sobrescrevem o destino, verifique o checksum depois |

### Limites de payload e de execução
- `max_command_chars` (padrão 1000) rejeita um comando grande demais com exit 64 `command_too_long` antes de qualquer trabalho SSH
- `max_output_chars` (padrão 100000) corta a saída remota e define `truncated_stdout` ou `truncated_stderr`
- `--max-output-bytes` limita o envelope e `--truncate-content` limita cada texto
- `--max-concurrency` aceita 1..=64 e sai com 2 fora dessa faixa, o valor automático segue CPUs e RAM livre
- `--fail-fast` para de admitir novas unidades da frota depois da primeira falha, as unidades em andamento terminam, e cada host nunca iniciado volta com um `error` que começa com `not attempted`: `not attempted (fail-fast stopped admission)` para `exec` e `not attempted (fan-out admission stopped)` para `scp` e `sftp`
- `--scp-file-concurrency` define quantos canais do Secure Copy Protocol (SCP) rodam em paralelo em uma sessão, e o padrão é 1
- `--timeout` é em milissegundos e o padrão é 60000
- Só `vps add --timeout` avisa quando o valor é menor que 1000, enquanto `vps edit`, `exec` e `health-check` o aceitam em silêncio
- `--lock-wait-ms` limita o lock da configuração, aceita 1..=300000 (exit 2 fora), padrão 5000
- Um `tunnel` sempre precisa de `--timeout-ms`, e omiti-lo é exit 2

### Idioma com --lang
- `--lang en` ou `--lang pt-BR` define o idioma do texto humano em uma chamada, `locale set` o persiste
- Decida por `error_code`: a B2 tornou a linha de erro humana localizável, então `--lang pt-BR` a renderiza em português no modo texto
- O `message` do envelope JSON fica em inglês estável por contrato, então a análise nunca depende do idioma do host
- A costura fica em `emit_resolved_ssh_error`: o ramo `--json` mantém o `Display` em inglês e o ramo humano consulta `i18n::localized_error_text`
- Códigos de erro sem tradução voltam ao `Display` em inglês, a localização é tolerante a falha e nunca deixa uma linha de erro em branco


## Detalhes de integração para agentes

### Obrigatório: registrar e trocar hosts
- Registre hosts com `vps add` antes de trabalho remoto repetido, com senha, chave ou `--use-agent` e `--agent-socket`
- Troque um host entre senha, chave e agente com uma chamada `vps edit`: `--key <PATH>`, `--password-stdin` ou `--use-agent`
- Cada uma limpa os outros métodos e `updated` lista o que mudou
- Uma `--key` diferente descarta a passphrase antiga a menos que a chamada traga uma nova, e `--agent-socket` sozinho implica `--use-agent`
- PROIBIDO: dois métodos primários de autenticação em uma chamada `vps edit` (o clap sai com 2) ou uma passphrase sem chave (exit 64, registro intacto)
- Um caminho de `--key` que não existe sai com 66 com `error_code` `file_not_found`
- OBRIGATÓRIO: leia cada entrada de `updated` como uma de `host`, `port`, `user`, `password`, `key_path`, `key_passphrase`, `use_agent`, `agent_socket`, `timeout_ms`, `max_command_chars`, `max_output_chars`, `sudo_password`, `su_password`, `disable_sudo`, `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key` e `tags`
- Esses 19 nomes também são o enum em `docs/schemas/vps-edit.schema.json`
- OBRIGATÓRIO: nomeie hosts com letras ASCII, dígitos, ponto, hífen e sublinhado, começando por letra ou dígito, de 1 a 255 bytes
- `vps add`, `vps rename`, `vps edit --name` e `vps import` recusam outros nomes com exit 64
- `vps import` valida todos os itens primeiro e lista todos os nomes inválidos de uma vez, sem gravar nada, também sob `--dry-run`
- Um `config.toml` antigo com outros nomes ainda carrega, mesmo com caracteres de controle
- O `doctor` os lista em `local.invalid_host_names` e `vps rename '<old name>' <new name>` os repara
- Um nome com caractere de controle é recusado como alvo por `exec`, `health-check`, `--hosts` e `connect` com exit 64, e o `connect` não grava marcador
- `tls mtls import --name` recusa caracteres de controle, formatação e bidi com exit 64 e não cria diretório

### Obrigatório: credenciais, comandos e escopo do SCP
- Informe uma senha ou uma chave, credenciais vazias são rejeitadas na gravação
- Trate uma senha vazia em list e show JSON como `null` (hosts só com chave) e uma não vazia como mascarada com três asteriscos
- Um comando remoto vazio falha com a mensagem técnica `empty command` (sempre em inglês) e exit 64
- Passe `--timeout-ms` em toda invocação de `tunnel`
- Trate o `scp` como somente arquivos regulares, sem diretórios e sem `-r`
- Para árvores e operações no sistema de arquivos remoto use `sftp` (`upload` e `download` com `--recursive`, `ls`, `mkdir`, `rm`, `stat`, `rename`)
- OBRIGATÓRIO: use a linha de produto 0.5.3 como piso para o SSH File Transfer Protocol (SFTP) (a linha atual é a 0.5.6), a G1 corrigiu o truncamento do upload a zero byte, e nunca dependa da 0.3.9 do crates.io para SCP porque esse protocolo estava quebrado (uma versão obsoleta, mantida aqui como história)
- Verifique transferências com um `sha256` no destino (G15), não só com tamanhos informados pelo cliente

### Obrigatório: resultados do SCP e formato do export
- Interprete um sucesso de SCP com `docs/schemas/scp-transfer.schema.json` (`ok`, `event` `scp-transfer`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`)
- Um arquivo local ou remoto de SCP ausente sai com 66 com `file not found: <path>`, caminho canônico e sem prefixo `SCP:` empilhado
- Leia `mtime_preserved` e `durable` no evento `scp-transfer`, preservar mtime e modo é melhor esforço (G-SCP-R01/R02)
- O corpo de `vps export` segue o formato de saída resolvido, então um agente não TTY recebe o envelope JSON `vps-export` mesmo quando `-o` nomeia um arquivo `.toml`
- Um corpo TOML exige `--output-format text`
- Com `-o`, o stdout traz só o resumo `ok`, `event`, `format`, `include_secrets` e `path`, e `docs/schemas/vps-export.schema.json` aceita os dois formatos

### Obrigatório: export e import de segredos
- Um `vps export` redigido limpa todo segredo: `password` é texto vazio e `key_passphrase`, `sudo_password` e `su_password` são `null`, nunca texto cifrado `sshcli-enc:` de vazio (EXP-001)
- `--include-secrets` exige `-o` ou `--output`, ou `--i-understand-secrets-on-stdout`
- `vps import` aceita TOML (chaves em inglês e aliases legados em português na leitura) ou um `vps-export` JSON, e `--allow-incomplete` aceita hosts redigidos ou esqueleto
- A chave do usuário é `username` no TOML (legado `usuario`) e `user` no JSON

### Obrigatório: formato em fio e flags de segredos
- `added_at` é opcional no import (o serde assume agora) e o formato em fio é o schema v3, que serializa chaves em inglês
- Prefira as flags de CLI de segredos `--allow-plaintext-secrets`, `--secrets-key-file` e `--use-keyring` a variáveis de ambiente, porque `SSH_CLI_SECRETS_KEY` e `SSH_CLI_SECRETS_KEY_FILE` fazem todo comando que lê ou grava segredos do registro falhar fechado com 64
- Medido com `SSH_CLI_SECRETS_KEY` definida num registro que guarda um segredo: `vps list`, `vps show`, `vps add`, `vps rename`, `vps remove`, `vps export`, `doctor`, `secrets status`, `secrets reencrypt` e `connect` saem com 64, enquanto `vps path`, `locale show`, `schema`, `commands` e `tls provider` nunca tocam segredos e saem com 0
- `secrets init --json` e `secrets reencrypt --json` emitem `secrets-init` e `secrets-reencrypt` (`docs/schemas/secrets-init.schema.json`, `docs/schemas/secrets-reencrypt.schema.json`)
- A primeira gravação de segredo pode definir `secrets_key_auto_created: true` no mesmo JSON de sucesso, um documento
- O catálogo de schemas está em [docs/schemas/README.md](schemas/README.md)
- OBRIGATÓRIO: em `tunnel --json`, espere o objeto do stdout com `event: "tunnel_listening"` (`docs/schemas/tunnel-listening.schema.json`) antes de usar a porta local
- O processo fica vivo até o timeout ou um sinal, o prazo depois do bind termina com exit 0 (TUN-002), um SIGINT ou SIGTERM imprime `tunnel_closed` com `reason` `signal` e sai com 130 ou 143, e um timeout antes do bind continua 74
- OBRIGATÓRIO: o `--bind` do tunnel assume `127.0.0.1` (loopback)

### Permitido: sobrescrever autenticação e timeout
- `tunnel` e `health-check` podem usar `--password-stdin`, `--key`, `--key-passphrase` e `--key-passphrase-stdin` (paridade CLI-005/006 com exec e scp)
- `health-check --timeout <ms>` quando o padrão do host é longo ou curto demais
- OBRIGATÓRIO: prefira a distribuição por vários hosts para trabalho de frota: `exec`, `sudo-exec`, `su-exec`, `scp`, `sftp` e `health-check` com `--all` ou `--hosts a,b,c` rodam sessões concorrentes limitadas (`Semaphore` e `JoinSet`), não um processo por host
- O JSON de lote vale para os dois modos de vários hosts, mesmo quando `--hosts` lista um nome
- OBRIGATÓRIO: existe um terceiro seletor só na família exec: `exec`, `sudo-exec` e `su-exec` com `--tags t1,t2` endereçam todo host que carrega qualquer uma dessas tags (`vps add --tag`)
- `--all`, `--hosts` e `--tags` são mutuamente exclusivos e o clap rejeita qualquer par
- `scp`, `sftp` e `health-check` aceitam `--all` e `--hosts` mas não `--tags`
- OBRIGATÓRIO: interprete o JSON de vários hosts com `health-check-batch`, `exec-batch`, `scp-batch` e `sftp-batch` (`docs/schemas/*-batch.schema.json`), todo envelope traz `batch_run_id` e `max_concurrency`
- `scp-batch` e `sftp-batch` também trazem `target_source` `selector` e seu alias `host_source`, e `exec-batch` não
- PERMITIDO: limite a distribuição com o global `--max-concurrency N` (1..=64), o mesmo portão limita os encaminhamentos aceitos pelo tunnel
- PROIBIDO: assumir vários hosts em sequência por padrão quando `--all` está disponível, o tempo de relógio é dominado pelo tempo de ida e volta (RTT) do SSH e sessões concorrentes são o modus operandi

### Obrigatório: cancelamento, avisos e instalação
- O cancelamento de SCP e SFTP com vários arquivos preenche o restante cancelado para que `results.len() == input.len()` (G5/G17)
- Um `vps add --timeout` abaixo de 1000 e um valor parecido com senha no argv emitem um aviso no stderr, não interprete essas linhas como envelope de erro JSON
- Instale com `cargo install ssh-cli --locked`

### Proibido: suposições inseguras
- Assumir uma conexão SSH de longa duração entre execuções do processo
- Reintroduzir empacotamento de daemon de longa duração neste repositório
- Habilitar ou emitir telemetria do produto
- Repetir às cegas em exit 64, 65, 66 ou 77
- Imprimir ou guardar material da chave primária vindo dos comandos `secrets`
- Tratar árvores de diretórios no SCP ou um `-r` recursivo como suportados
- Instalar ou chamar binários cliente do OpenSSH no host do agente para o `ssh-cli`, porque ele fala SSH dentro do processo pelo `russh` e não cria nenhum `ssh`, `scp` ou `ssh-keygen` local
- OBRIGATÓRIO: trate textos de comando remoto como entrada hostil, bytes NUL são rejeitados com erro de argumento inválido antes do exec no canal SSH (G-PROC-03)
- `sftp ls` e `sftp stat` escapam `\n` e `\t` de um nome remoto no modo texto, então um nome forjado não adiciona linhas

### Obrigatório: entrada hostil e integridade do SFTP
- O SETSTAT do SFTP envia `atime` e `mtime` juntos (G3) e `set_metadata` falha fechado (G4)
- Bits de permissão são mascarados por direção, o upload usa `SFTP_PERM_MASK` (`0o7777`) e mantém setuid, setgid e sticky em um arquivo que você já possui (G12/G19), o download usa `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) para bits de elevação enviados pelo servidor nunca caírem localmente (A3)
- Falhas de `set_permissions` local no download SFTP são erros, nunca silenciosas (G18)
- O caminho de protocolo do cliente SCP (`client_real_scp.rs`) usa identificadores em inglês e erros de canal em inglês (G16)


## CRUD e contrato JSON

### Operações legíveis por máquina
- Listar hosts: `ssh-cli vps list --json` devolve um array de objetos de host mascarados
- Mostrar um host: `ssh-cli vps show <name> --json` devolve um objeto de host mascarado
- Descoberta: `ssh-cli commands`, `ssh-cli schema [NAME]` e `ssh-cli doctor`
- Segredos: `ssh-cli secrets status --json` devolve o modo de cifragem sem material de chave
- `secrets init --json` emite `event: "secrets-init"` e `secrets reencrypt --json` emite `event: "secrets-reencrypt"`
- Eventos de sucesso do CRUD quando o JSON está efetivo: `vps-added`, `vps-edited` (ou `vps-renamed` quando `vps edit --name` renomeia), `vps-removed`, `vps-connected` e `vps-import`, com `secrets_key_auto_created` no mesmo documento quando uma chave é criada sozinha
- Família exec em um host: `exec`, `sudo-exec` e `su-exec` com `--json` devolvem `target_resolved`, `target_source`, seus aliases de leitura da 0.5.5 `host_resolved` e `host_source`, `active_fallback`, `stdout`, `stderr`, `exit_code`, as flags de truncamento e `duration_ms` como um objeto (G8)
- OBRIGATÓRIO: designe o host alvo no argv em toda superfície, incluindo `health-check`
- Um posicional solto é erro de uso (exit 64)
- O marcador gravado por `connect` só é alcançável por `--use-active` (GAP-SSH-EXEC-ARGC-001)
- A divisão é estável na CLI: 64 significa nenhum alvo designado e 66 significa que o alvo designado não existe
- Leia `target_source` (`argv`, `active_marker` ou `selector`), ou seu alias `host_source`, antes de confiar onde uma mutação caiu
- OBRIGATÓRIO: leia `target_resolved`, `target_source` e `active_fallback` também no envelope de erro
- Eles estão presentes sempre que a falha aconteceu depois de o host ser resolvido e ausentes, nunca vazios, quando não, `vps_not_found` é o caso ausente
- `host_resolved` e `host_source` vão ao lado deles como aliases de leitura da 0.5.5, escritos a partir do mesmo valor, então vincule consumidores novos à grafia canônica `target_*`
- OBRIGATÓRIO: `exec --step` aborta o lote quando o passo zero sai com 127, esse código no primeiro passo é a assinatura de um nome de host lido como comando
- Um 127 em qualquer índice posterior ainda roda os passos restantes
- Exec de frota: `ssh-cli exec --all '<cmd>' --json`, `sudo-exec --all`, `su-exec --all` ou `--hosts a,b '<cmd>'` emitem `event: "exec-batch"` (`exec-batch.schema.json`), e uma falha parcial por host não aborta os irmãos
- Tunnel é de um host só, um bind e uma sessão por execução, tunnels em vários hosts são N invocações com portas ou `--bind` distintos
- Doctor: `ssh-cli vps doctor [--json]` emite um único `event: vps-doctor` raiz com `local` e `ssh_probe: null`, nunca duas raízes JSON
- Acrescente `--probe-ssh` para uma checagem de saúde em vários hosts embutida em `ssh_probe`, com um subconjunto opcional `--hosts a,b`
- Quando um host sondado falha, o stdout ainda traz o único documento `vps-doctor`, o stderr recebe um envelope `partial_failure` e o exit é 1
- SCP com vários arquivos em um host: `ssh-cli scp upload <VPS> f1 f2 … <REMOTE_DIR>` e `download <VPS> r1 r2 … <LOCAL_DIR>` usam uma sessão SSH e transferências em série (G-PAR-47)
- SCP com vários hosts e vários arquivos: `ssh-cli scp upload --all --src f1 --src f2 --dest <REMOTE_DIR>` ou `--hosts a,b`, arquivos em série em cada sessão (G-PAR-48)
- Um download de frota de um arquivo grava `<local>.<vps>` e um download de frota com vários arquivos grava em `<LOCAL_DIR>/<host>/`
- OBRIGATÓRIO: com `--all` ou `--hosts`, mais de uma origem viaja pelas posições nomeadas `--src` (repetível) e `--dest`
- Um seletor mais três ou mais posicionais é exit 64, porque uma flag não posicional não pode remapear um posicional
- `--dest` é sempre um diretório, qualquer que seja a contagem de origens, e as posições nomeadas também funcionam em um host só
- Saúde em um host: `ssh-cli health-check <vps> [--timeout <ms>] --json` devolve `name`, `status` e `latency_ms` mais o eco do alvo
- Saúde em uma frota: `--all` ou `--hosts a,b` emite `event: "health-check-batch"` (`health-check-batch.schema.json`)
- SCP em um host: `ssh-cli scp upload|download <vps> <local> <remote> --json` devolve `scp-transfer` no stdout e as falhas usam o envelope de erro no stderr
- Lotes SCP de frota e com vários arquivos emitem `event: "scp-batch"`, o `name` de um resultado com vários hosts e vários arquivos pode ser `host:path`
- Fatos operacionais do SCP: use a 0.5.3 ou mais nova, o upload envia blocos de 32 KiB, o download grava `{path}.ssh-cli.partial` e depois renomeia, e uma falha de `sync_data` propaga antes da renomeação (G9)
- SFTP: `ssh-cli sftp upload`, `ssh-cli sftp download`, `ssh-cli sftp ls`, `ssh-cli sftp mkdir`, `ssh-cli sftp rmdir`, `ssh-cli sftp rm`, `ssh-cli sftp stat` e `ssh-cli sftp rename` com os schemas `sftp-transfer`, `sftp-list`, `sftp-fs-op` e `sftp-batch`
- Idioma: `ssh-cli locale show`, `ssh-cli locale set` e `ssh-cli locale clear`, com o `--lang` de uma chamada
- TLS: `ssh-cli tls provider`, `ssh-cli tls paths`, `ssh-cli tls mtls list`, `ssh-cli tls mtls import`, `ssh-cli tls mtls show`, `ssh-cli tls mtls remove`, `ssh-cli tls acme account create`, `ssh-cli tls acme account show`, `ssh-cli tls acme issue`, `ssh-cli tls acme complete`, `ssh-cli tls acme status` e `ssh-cli tls acme list`
- OBRIGATÓRIO: os comandos ACME recebem o domínio por `--domain`, um posicional solto sai com 2
- Tunnel: `ssh-cli tunnel <vps> <local_port> [remote_host] [remote_port] --timeout-ms <ms> [--bind 127.0.0.1] --json` emite `tunnel_listening` no stdout depois do bind e `tunnel_closed` no encerramento
- Modos de tunnel (0.5.4): `--socks5` serve um proxy `CONNECT` RFC 1928 sem autenticação, `--remote-socket <PATH>` encaminha para um socket Unix remoto e `--reverse` pede ao servidor que escute e entregue de volta em `<local_port>`
- Os três são mutuamente exclusivos
- `remote_host` e `remote_port` são omitidos sob `--socks5` e `--remote-socket` (passá-los é exit 64) e significam o bind do servidor sob `--reverse`, onde `remote_port 0` deixa o servidor alocar a porta
- Os dois eventos de tunnel trazem `mode` (`local`, `socks5`, `streamlocal` ou `reverse`), leia-o antes de interpretar os campos irmãos porque sob `--reverse` `local_port` é a porta que o servidor alocou, enquanto `remote_host` e `remote_port` trazem o host e a porta de entrega locais
- `--i-accept-network-exposure` protege o `--bind` local nos três modos locais e o bind remoto sob `--reverse`, a ponta exposta nessa direção
- O bind local é analisado como IP pelo clap (um erro de digitação sai com 2) e o bind remoto é comparado como texto porque a RFC 4254 admite nomes e o texto vazio (um erro de digitação sai com 64 pela guarda)
- PROIBIDO: passar `--bind` junto com `--reverse` e esperar que importe, a flag é aceita pelo clap e depois descartada em silêncio porque a entrega reversa é forçada para o loopback
- Enderece a escuta do lado do servidor pelo posicional `<remote_host>`
- Export: o corpo de `ssh-cli vps export` é o envelope JSON `event: "vps-export"` em qualquer stdout não TTY e `--output-format text` é a única rota para TOML
- Import: `ssh-cli vps import --file <path> [--allow-incomplete]`, um host que parece um export redigido sai com 64 a menos que `--allow-incomplete` seja passado
- Um `--file` ausente sai com 66 com `error_code` `file_not_found`, `error_class` `permanent` e `retryable` false, na execução real e sob `--dry-run`, então corrija o caminho e nunca repita o mesmo argv
- List e show imprimem uma senha vazia como `null` em JSON e um segredo não vazio mascarado com `FIXED_MASK` (três asteriscos)
- Valide payloads contra os schemas em `docs/schemas/`, o índice é [docs/schemas/README.md](schemas/README.md)

### Campos do doctor e o diretório de configuração
- OBRIGATÓRIO: leia `ssh-cli doctor --json` como um documento com `event: "vps-doctor"`, `ok`, `local` e `ssh_probe`
- `local` traz `layer`, `config_path`, `exists`, `permissions`, `config_dir_exposed`, `config_dir_permissions`, `invalid_host_names`, `schema_version`, `hosts`, `known_hosts`, `active_file`, `active_vps`, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out`, `telemetry` e `runtime`
- `local.runtime` traz `os`, `arch`, `is_wsl`, `is_container`, `is_ci`, `is_termux` e `sandbox`

### Obrigatório: campos do doctor
- Leia `local.active_vps` (o nome do host, `null` quando não definido) ao lado de `local.active_file` (o seu caminho)
- Leia `local.config_dir_exposed` (booleano, sempre presente), `local.invalid_host_names` (array de nomes escapados, sempre presente, vazio quando todo nome é válido) e `local.config_dir_permissions` (octal de quatro dígitos como `0700`, ausente fora do Unix)
- Fora do Unix só `config_dir_permissions` falta e `config_dir_exposed` lê `false`
- `permissions` continua descrevendo só o `config.toml`
- Leia-os com `ssh-cli doctor --json --select local.config_dir_exposed`, porque o `--select` projeta o objeto do `doctor` direto e `--filter` e `--count-only` não valem para um registro só
- Um diretório que a CLI cria nasce 0700 em cada nível, um diretório XDG padrão existente é apertado para 0700 quando não é link simbólico e pertence ao usuário
- Qualquer outro diretório existente nunca recebe chmod: `config_dir_exposed` é `true` e `config_dir_permissions` mostra o modo
- `--config-dir something.toml` faz do diretório pai o diretório de configuração de `secrets.key`, `known_hosts`, `active`, `lang` e dos arquivos TLS
- `known_hosts.lock` e `lang` são criados 0600 e o export cria diretórios ausentes 0700
- OBRIGATÓRIO: espere que o modo texto escape caracteres de controle, de formatação e bidirecionais como `\u{1b}`
- O conjunto escapado é todo caractere `char::is_control` mais U+00AD, U+061C, U+180E, U+200B a U+200F, U+202A a U+202E, U+2060 a U+2064, U+2066 a U+206F, U+FEFF e U+E0000 a U+E007F
- U+2065 não é atribuído e fica de fora, e o mesmo conjunto é recusado em nomes de host e de identidade mTLS
- O JSON não muda e mantém esses caracteres crus, e a saída remota do `exec` fica crua
- Uma entrada de registro com caractere de controle em tag, host ou user (campos já validados desde antes) ainda sai com 65, só o NOME do host é tolerante
- Duas tabelas da configuração que normalizam para o mesmo nome saem com 65 com `config has two hosts named ...`


## Roteamento por exit code
- Exit 0 significa sucesso
- Exit 1 significa uma falha geral de execução ou uma falha parcial em vários hosts, inspecione o stderr e `remote_exit_code`
- Exit 2 significa erro de uso do clap, impresso como texto puro no stderr sem envelope, corrija as flags
- Exit 64 significa erro de uso ou de argumento (incluindo comando vazio, nome inválido ou `--dry-run` recusado) ou validação permanente do ACME (`invalidContact`), corrija o argv e não repita
- Exit 65 (`TomlDe`, JSON, schema) significa erro de leitura ou de dados, corrija o payload de entrada
- Exit 66 significa VPS ou arquivo ausente (`file not found: <path>` no SCP e no `vps import --file`), registre-o ou corrija o nome ou o caminho
- Exit 69 significa que um serviço do host não responde (uma escrita no keyring ou o lock da configuração), é transitório e pode ser repetido
- Exit 70 significa uma falha interna sem entrada corrigível pelo usuário, repetir sem mudar não ajuda
- Exit 73 significa falha de gravação da configuração, confira permissões e disco
- Exit 74 significa falha de entrada e saída ou de conexão SSH, uma nova tentativa de rede pode ajudar
- Exit 77 significa falha de autenticação, política de chave de host, sudo desativado ou um segredo guardado que não decifra, tente `--key`, `--password-stdin` ou uma passphrase no stdin e não repita às cegas
- Exit 141 significa cano quebrado: o leitor fechou o stdout ou o stderr, a saída está incompleta, o estado precisa ser reconferido e a chamada nunca é repetida às cegas
- Exit 130 e 143 significam término por sinal (SIGINT e SIGTERM), e um `tunnel` imprime antes `tunnel_closed` com `reason` `signal`
- Esses códigos batem com a seção `Exit status:` de `ssh-cli --help`


## Estratégia de nova tentativa
- Prefira os campos `retryable` e `error_class` do envelope de erro JSON a heurísticas só por exit (`docs/schemas/error-envelope.schema.json`)
- Repita no máximo duas vezes em `retryable: true` ou exit 74, com espera exponencial e variação aleatória completa (base 200 ms, teto 5 s, veja `ssh_cli::retry::RetryConfig::agent_default`)
- Nunca repita em `retryable: false` ou em exit 64, 65, 66, 77, 1 (comando remoto), 130, 143 ou 141 sem mudar as entradas
- Validação permanente do ACME (`invalidContact` ou 4xx) é exit 64, não exit 74, então não a trate como falha de rede que pode ser repetida
- Depois de uma resposta perdida, consulte o estado em vez de repetir: a dica do 66 de `vps rename` e `vps remove` manda rodar `vps list` antes
- O binário não repete sozinho `exec`, `scp` ou `sftp` não idempotentes no processo (menor privilégio de execução única)
- O agente reinvoca o processo
- Encurte ou divida comandos quando um exit sinalizar uma rejeição de `max_command_chars`
- Confirme mudanças de chave de host com um humano antes de `--replace-host-key`


## Limites declarados
- O stdout e o stderr remotos do `exec` ficam crus por desenho, o escape do modo texto não os alcança
- Em objeto único, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` e `--max-output-bytes` seguem inertes e são nomeadas em `agent_shape.ignored`, enquanto `--select` e `--truncate-content` reduzem o registro
- O nome aparece mesmo quando a flag de coleção é a única passada, então `ssh-cli doctor --limit 2` responde com `agent_shape.ignored` em vez de um corpo sem modelagem
- O JSON mantém cru todo caractere do conjunto escapado (por exemplo U+202E e o espaço de largura zero) porque é para máquinas
- Os corpos de erro ficam em inglês por desenho para os agentes receberem texto estável
- Um nome legado não gera aviso no carregamento, o `doctor` o lista
- macOS e Windows só passam pela checagem de tipos no gate `cross-targets`, então o modo 0700 e o `getuid` não foram executados lá
- O Windows não tem chmod, então o diretório de configuração depende da lista de controle de acesso (ACL) do perfil
- O keyring do sistema só roda de verdade no Linux neste repositório, enquanto o Keychain e o Credential Manager passam só pela checagem de tipos do gate `cross-targets`


## Integrações de crate
- Consumidores publicados dependem do contrato da CLI, não de uma API de biblioteca instável
- Fixe experimentos de biblioteca em uma versão exata da crate ao ligar `ssh_cli` como biblioteca
- Prefira a integração por binário instalado no PATH para agentes
