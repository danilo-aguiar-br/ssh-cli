# Guia de migração


> Passe de ssh-cli 0.3.3 (ou posterior) para 0.5.6 sem perder o inventário multi-host.

- Leia este documento em [Inglês (en)](MIGRATION.md)


## O que muda

### Desde 0.5.6 (atualização a partir da 0.5.5)
- A 0.5.5 é a última release no crates.io, e esta seção é o caminho inteiro dela até a 0.5.6
- QUEBRA: a versão mínima suportada de Rust (MSRV) é 1.89, então uma toolchain mais velha que 1.89 não compila esta release, mesmo que compilasse a 0.5.5
- O motivo é o `russh` 0.65.1, que declara `rust-version = "1.89"` e fecha a RUSTSEC-2026-0492, a RUSTSEC-2026-0495 e dois advisories GHSA
- Compilar do código-fonte ainda exige um compilador C, porque o provider de Transport Layer Security (TLS) é `aws_lc_rs` e o `aws-lc-sys` compila C, e uma migração para criptografia em Rust puro foi tentada e revertida
- O backend de keyring sozinho pediria 1.88.0, porque a `keyring` 4.2.0 declara `edition = "2024"` e `rust-version = "1.88.0"`, e o cofre Secret Service por zbus segue
- Rode `rustup update stable` antes do `cargo install ssh-cli --locked`
- O `secrets init --force` não grava mais `secrets.key.bak`
- Se você dependia dessa cópia para recuperar de uma chave perdida, exporte a chave antes ou guarde o valor anterior fornecido por `--secrets-key-file`
- O erro que mandava restaurar `secrets.key.bak` agora aponta `ssh-cli secrets reencrypt` com a chave anterior
- O `secrets init --keyring` agora grava no cofre do SO em vez de um simulador dentro do processo, então o `secrets status` seguinte responde `key_source: "keyring"` e `encryption_active: true`
- A entrada do cofre é escopada pelo diretório de config, então um perfil criado com `--config-dir` não lê nem sobrescreve mais a chave do perfil real
- Uma chave gravada por um `secrets init --keyring` anterior agora é encontrada sem repetir a flag, que é o que faz dela um cofre e não uma variável de um processo
- A flag global `--use-keyring` agora chega também ao `secrets init`, então `--use-keyring secrets init` grava no cofre em vez de gravar `secrets.key` em silêncio
- Numa máquina sem cofre de credenciais respondendo, o `secrets init --keyring` sai 69 em vez de reportar sucesso
- QUEBRA: um perfil tem exatamente uma chave primária, então o `secrets init` agora recusa com exit 64 quando o perfil já tem chave em qualquer um dos dois armazenamentos, e a mensagem nomeia `--force`
- Antes desta release o comando aceitava `--keyring` num perfil que tinha `secrets.key`, cunhava uma segunda chave e deixava o arquivo responder primeiro, o que tornava todo segredo guardado indecifrável
- O `secrets init --force --keyring` agora move a chave para o cofre e remove `secrets.key`, e o `secrets init --force` num perfil de cofre grava o arquivo e remove a entrada do cofre
- Se um perfil já carrega duas chaves de uma build anterior, rode `ssh-cli secrets init --force` uma vez com o cofre que você quer manter, depois `ssh-cli secrets reencrypt`
- `vps list --select` agora satisfaz `docs/schemas/vps-list.schema.json`: o item reduzido é a segunda forma publicada, e o relatório de redução continua no stderr
- Nada muda para o consumidor que lê a lista sem modelagem, porque essa forma não mudou
- As flags globais de modelagem mudaram de comportamento em payload de objeto único: `--select` e `--truncate-content` agora reduzem `vps show`, `doctor`, `commands` e todo objeto sem coleção
- Leia um campo com `ssh-cli doctor --select local.config_dir_exposed` em vez de encanar para o `jaq`
- Um registro reduzido carrega `agent_shape`, então o consumidor que percorre toda chave de `vps show --json --select` vê um objeto a mais
- `docs/schemas/vps-show.schema.json` publica as duas formas por `anyOf`, e uma execução sem modelagem não traz `agent_shape`
- Em objeto único, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` e `--max-output-bytes` seguem inertes, e `agent_shape.ignored` agora nomeia cada uma que foi passada em vez de descartá-la em silêncio
- Um array nu reduzido escreve uma linha JSON em stderr com `"event":"agent-shape"`, carregando `input_count`, `output_count` e `dropped`, o stdout mantém o tipo do array e o `--quiet` suprime a linha
- Se o seu executor trata qualquer saída de stderr como falha, passe `--quiet` ou leia a linha e a ignore, porque o código de saída e o stdout não mudam
- A renomeação é nativa: `ssh-cli vps rename <NAME> <NEW_NAME>` e `ssh-cli vps edit <NAME> --name <NEW_NAME>` movem a chave do registro e mantêm toda credencial
- Pare de usar `ssh-cli vps export` → editar → `ssh-cli vps import` para isso: o export redige segredo, então o ciclo obrigava a redigitar a senha
- Antes da 0.5.6 o ciclo também descartava `use_agent` e `agent_socket`, então host autenticado por agente perdia a autenticação em silêncio
- Desde a 0.5.6 `ssh-cli vps export` e `ssh-cli vps import` carregam os dois campos, como medido no export de um host de agente
- BREAKING: `ssh-cli vps edit` troca o método de autenticação, `--key` limpa a senha e o agente, `--password`/`--password-stdin` limpam a chave, a passphrase dela e o agente, e `--use-agent` limpa a senha e a chave
- Antes, um host de senha não aceitava chave (exit 64, `primary auth methods are mutually exclusive`) e o contorno era passar por `--use-agent`
- Abandone esse contorno
- `updated` agora lista o que foi limpo, mas só campos que tinham valor
- Flags de autenticação em conflito saem 2: `--key` com `--password*`, `--agent-socket` com `--key`, `--password*` ou flag de passphrase, e flag de passphrase com `--use-agent` ou `--password*` falham no parse
- Passphrase sem chave sai 64 e deixa o registro intacto
- `ssh-cli vps edit X --use-agent --agent-socket S` continua válido
- Um caminho de `--key` diferente descarta a passphrase antiga, a menos que a mesma chamada traga uma nova, e `updated` lista `key_passphrase`
- Antes, uma chave diferente deixava a passphrase da chave antiga no disco, uma órfã que não servia a nenhuma chave
- `--agent-socket` sozinho agora implica `--use-agent` e limpa `password`, `key_path` e `key_passphrase`, então um host de senha que recebe só um socket vira host de agente
- Antes, `--agent-socket` sozinho gravava o socket num host que continuava autenticando por senha, então ele parecia configurado para o agente e não estava
- QUEBRA para hosts que você cria: nomes são estritos, `ssh-cli vps add`, `ssh-cli vps rename`, `ssh-cli vps edit --name` e `ssh-cli vps import` aceitam só letras ASCII, dígitos, ponto, hífen e sublinhado, começando por alfanumérico, até 255 bytes
- A regra estrita também recusa `..`, ponto ou espaço no fim e nomes reservados do Windows como `CON`
- Nomes antigos com espaço ou acento ainda carregam, listam, mostram, resolvem e podem ser renomeados para um nome estrito
- O `ssh-cli vps doctor` os lista em `local.invalid_host_names`
- Um nome com controle ou bidi guardado no `config.toml` ainda carrega: o `ssh-cli doctor` o lista escapado, a saída em texto o escapa, e `ssh-cli vps rename '<old name>' <new name>` o repara
- `ssh-cli vps import` valida todos os itens e lista todos os nomes ruins de uma vez (`invalid VPS names in import (N): ...`, exit 64, nada gravado, igual no `--dry-run`)
- Buscas que viram alvo (`ssh-cli exec`, `ssh-cli health-check`, `--hosts` e `ssh-cli connect`) recusam nome com caracteres de controle ou bidi como erro de uso com exit 64, e o `ssh-cli connect` não grava marcador
- `ssh-cli tls mtls import --name` recusa caracteres de controle, formatação e bidi com exit 64 e não cria diretório, enquanto Unicode comum como `serviço-1` continua aceito
- Duas tabelas `[hosts]` cujos nomes normalizam para o mesmo nome NFC saem 65 com `config has two hosts named ...` (nomes escapados), e o carregamento chaveia o mapa pelo nome NFC do registro, então nenhum host se perde em silêncio
- O export de um host legado com nome fora do charset estrito só restaura depois de `ssh-cli vps rename`
- O modo texto escapa caracteres de controle e bidi: como `\u{1b}` em mensagens, `ssh-cli vps list`, `ssh-cli vps show`, `ssh-cli health-check`, `ssh-cli doctor` e saídas em lote
- O JSON não muda, e a saída remota do `ssh-cli exec` segue crua
- `ssh-cli sftp ls` e `ssh-cli sftp stat` também escapam `\n` e `\t` no nome remoto no modo texto
- O diretório de config nasce `0700`: Um diretório XDG padrão existente é apertado a `0700`
- Qualquer outro diretório existente (um `--config-dir` que você apontou) nunca recebe chmod, grava aviso de log (visível com `-v`) e aparece como `local.config_dir_exposed` no `ssh-cli doctor --json`
- Leia com `ssh-cli doctor --json --select local.config_dir_exposed`, porque o `--select` projeta o objeto do `ssh-cli doctor` direto
- O `ssh-cli doctor --json` ganha `local.config_dir_permissions` (string octal de quatro dígitos como `0700`, ausente fora do Unix), `local.config_dir_exposed` (booleano, sempre presente) e `local.invalid_host_names` (array, sempre presente)
- `--config-dir something.toml` faz da pasta pai o diretório de config de `secrets.key`, `known_hosts`, `active`, `lang` e dos arquivos TLS, onde antes deixava uma pasta chamada `something.toml` para trás
- `ssh-cli vps export -o new/deep/e.json` cria as pastas que faltam `0700` e nunca altera uma existente, e `known_hosts.lock` e `lang` nascem `0600`
- O Windows não tem chmod, então a pasta depende da ACL do perfil
- QUEBRA para quem usa a biblioteca: `paths::validate_and_normalize` devolve `String` e guarda só a regra de nome de arquivo. Chame `VpsName::try_new` para nomes de host
- QUEBRA para quem usa a biblioteca: `Money`, `Brl`, `Usd`, `Currency` e a dependência `rust_decimal` foram removidos, e o RUSTSEC-2026-0235 (vinha pelo `rkyv`) sai do lock com eles
- Os nove `scripts/*.sh` foram apagados
- Ficam só os programas `.py`
- O arquivo solto `json` na raiz do crate foi apagado, e o `exclude` do pacote agora lista `/json`, `*.sh`, `__pycache__/`, `*.pyc` e `*.snap.new`
- `ssh-cli vps edit` agora limpa campo: Oito flags removem em vez de sobrescrever: `--no-key` (que também zera a passphrase), `--no-agent-socket`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key` e `--clear-tags`
- Cada uma conflita com a flag de valor do mesmo campo, então `--no-key --key` é erro de parse e não instrução ambígua
- Limpar o único método de auth restante é recusado com exit 64
- `--no-agent-socket` num host que autentica pelo agente também é recusado com exit 64 no Unix, porque não sobra endpoint de agente padrão, enquanto o Windows tem endpoint padrão e permite
- Eventos novos: `ssh-cli vps rename --json` e `ssh-cli vps edit --name --json` emitem `event: "vps-renamed"` com `name`, `previous_name`, `active_rewritten`, `updated`, `target_resolved` e `target_source`
- `vps-edited` é emitido só quando nada foi renomeado e não traz `previous_name`
- Um parser que esperava `vps-edited` depois de `ssh-cli vps edit --name` agora precisa aceitar `vps-renamed`
- `updated` está sempre presente: os campos cujo valor guardado mudou, na ordem de aplicação, cada um no máximo uma vez
- Fica vazio quando todo valor pedido já coincidia ou quando a invocação só renomeou o host
- A proveniência do alvo é publicada só depois da busca no registro, então `vps_not_found` continua chegando sem alvo
- Edição que não muda nada é uma operação nula idempotente: Ela sai 0, emite `vps-edited` com `updated: []` e não regrava o `config.toml`
- O exit 64 fica reservado para a invocação sem nenhuma flag de campo, de renomeação ou de limpeza
- `ssh-cli vps rename` não é idempotente: `ssh-cli vps rename ghost ghost` sai 66 porque a pré-condição de host existente vence, nome idêntico de host existente sai 64 e o erro carrega `target_resolved`, e repetir um rename depois de sucesso sai 66 (a dica do erro manda conferir `ssh-cli vps list`)
- Confirme uma renomeação com `ssh-cli vps show <NEW_NAME>`, nunca repetindo-o
- A ordem de gravação da renomeação falha fechada: O marcador `active` é gravado primeiro e o `config.toml` depois
- Se a gravação do registro falhar o marcador antigo é restaurado, e se a gravação do marcador falhar nada é renomeado
- Um crash entre os dois passos deixa marcador órfão, e `--use-active` então sai 66 em vez de agir no host errado
- `--dry-run` passou de seis para oito operações: `ssh-cli vps edit` e `ssh-cli vps rename` agora preveem
- O enum de `operation` em `dry-run.schema.json` lista as oito
- Os planos de `--dry-run` de `ssh-cli vps edit`, `ssh-cli vps rename` e `ssh-cli vps remove` ecoam `target_resolved`, `target_source` (`argv`), `host_resolved` e `host_source` assim que a busca dá certo
- `ssh-cli vps edit`, `ssh-cli vps rename` e `ssh-cli vps remove` num config que não existe saem 66 e não criam diretório nem arquivo de lock, com ou sem `--dry-run`
- O lock do config é limitado: A contenção travava para sempre, o que para um agente one-shot significa sem saída, sem exit e sem diagnóstico
- Agora ele espera `--lock-wait-ms` (padrão 5000, faixa aceita de 1 a 300000) e sai 69 com `unavailable`, que é retryable
- Um sinal durante a espera ainda vence com 130 ou 143
- `ssh-cli commands --json` publica argumentos: Cada nó de comando agora carrega `args` com `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` e `hidden`, então o agente descobre `--no-key` sem rodar `--help`
- O `ssh-cli --help` agora ganha uma seção `Exit status:` que lista os exit codes 0, 1, 2, 64, 65, 66, 69, 70, 73, 74, 77, 130, 141 e 143
- As 48 folhas de comando trazem uma seção `Examples:` no `--help`, e o `-h` continua curto
- Os dois textos aparecem só no help longo, e o `ssh-cli commands --json` e todo envelope JSON não mudam
- `--disable-sudo` tem duas formas: a flag global desativa `ssh-cli sudo-exec` e `ssh-cli su-exec` na invocação (alias `--disableSudo`, aceito só na flag global) e, posta antes de `ssh-cli vps add` ou `ssh-cli vps edit`, também grava `disable_sudo` no registro, enquanto `ssh-cli vps add --disable-sudo` e `ssh-cli vps edit --disable-sudo` gravam a escolha no registro do host, e `ssh-cli vps edit --enable-sudo` a desfaz
- A espera do lock reage a sinal em até 100 ms: Ela espera em fatias de 100 ms que checam o sinal, então SIGINT e SIGTERM saem 130 e 143 em até uma fatia (antes, até cerca de 5 s)
- `config.toml.lock` nasce com modo 0600, a gravação do marcador faz fsync do diretório, e a dica do exit 69 manda repetir o argv ou subir `--lock-wait-ms`
- Ela não fala mais de keyring
- O modo texto não engole mais erro de escrita em stdout: Em todo verbo, pipe quebrado agora sai 141 e disco cheio vira erro de I/O
- Antes os dois saíam 0. Em `ssh-cli exec` com stdout fechado o erro de escrita vence a falha do comando remoto
- Wrappers que dependiam de exit 0 com pipe fechado precisam tratar a falha
- `ssh-cli vps import` valida TLS como o `ssh-cli vps edit`: Certificado sem chave sai 64
- O evento de sucesso `vps-import` ganha `replaced` (os nomes sobrescritos, ordenado, sempre presente)
- O dry-run mantém `replaces_existing`
- O `ssh-cli vps import` também pega o lock do config antes de carregar o registro, então um import que grava entre a leitura e a gravação de outro processo não descarta mais o host dele
- O `ssh-cli vps import --file` com arquivo ausente sai 66 `file_not_found` como erro permanente, onde antes saía 74 `io` com `retryable: true`, então pare de tentar de novo esse caminho
- O `ssh-cli vps list` em modo texto dimensiona cada coluna pela célula mais larga, e a coluna PASSWORD mostra `(not set)` num host só com chave em vez de uma máscara
- O `ssh-cli vps show` em modo texto alinha todos os rótulos, então um parser que cortava a saída de texto em larguras fixas deve separar no primeiro dois-pontos
- Um `ssh-cli tunnel` parado por SIGINT sai 130 e um parado por SIGTERM sai 143, e o evento `tunnel_closed` carrega `reason: "signal"`
- A chave primária é resolvida nesta ordem: `--secrets-key-file`, depois o keyring com `--use-keyring`, depois o `secrets.key` do XDG, depois a entrada do keyring deixada por um `secrets init --keyring` anterior
- Só `SSH_CLI_SECRETS_KEY` e `SSH_CLI_SECRETS_KEY_FILE` são recusadas, então um comando que lê ou grava segredo do registro sai 64 enquanto uma delas estiver definida, e os outros nomes `SSH_CLI_*` são ignorados
- Catorze schemas em `docs/schemas/` foram realinhados com a saída que o binário emite, então revalide todo consumidor que fixou uma cópia antiga
- Proveniência do `--version`: `.commit_hash` agora guarda `<short hash> <version>`
- A ordem de resolução é env, git, o arquivo, depois `unknown`
- Sem env e sem git o arquivo só vale se a versão for a do pacote
- O `build.rs` não reexecuta mais a cada build sem `.git`, então a 0.5.6 imprime `unknown` até o passo de refresh do [checklist de release](RELEASE_CHECKLIST.pt-BR.md)
- QUEBRA para quem usa a biblioteca: `SshCliError::Unavailable` carrega um enum. A variante agora é `Unavailable { service: UnavailableService }` e o construtor é `SshCliError::unavailable(UnavailableService)`
- `UnavailableService` é `#[non_exhaustive]` com `Keyring` e `ConfigLock`
- Troque os argumentos string pelo enum e acrescente um braço curinga ao fazer match
- O texto no formato de rede não muda (`service unavailable: config_lock`), então consumidores de CLI e JSON não são afetados
- BREAKING para scripts de release e automação: os nove `scripts/*.sh` viraram `scripts/*.py`. Eles exigem Python 3.9+ e só a biblioteca padrão
- No Windows rode `py -3 scripts/<name>.py`
- Troque `bash scripts/<name>.sh` por `python3 scripts/<name>.py` em `check_all_gates`, `check_advisory_freshness`, `check_cross_targets`, `check_en_identifiers`, `verify_install_resolve`, `generate_sbom`, `dist_multiarch`, `release_attest` e `e2e_real_ssh`
- Os helpers `_gatekit.py`, `_e2e_sshd.py` e `_e2e_cases.py` não são pontos de entrada
- As variáveis de ambiente dos scripts viraram flags: `CHECK_TIMEOUT_SECS` é `--timeout-secs` (`check_cross_targets`, e os scripts que chamam cargo também o aceitam), `TARGETS`, `OUT_DIR` e `PARALLEL_JOBS` são `--targets`, `--out-dir` e `--jobs` (`dist_multiarch`), `REQUIRE_COSIGN` é `--require-cosign` (`release_attest`), `CARGO_TARGET_DIR_INSTALL_GATE` é `--target-dir` (`verify_install_resolve`), e `check_advisory_freshness` aceita `--db-root`
- `e2e_real_ssh` aceita `--host --port --user --password-file --sudo-password-file --from-grok-config [PATH] --bin --local-sshd`
- As variáveis `SSH_CLI_E2E_*` e `--config-dir` foram removidas
- `--from-grok-config` exige Python 3.11+, e `--local-sshd` é só Unix (no Windows falha com `local_sshd=requested_but_unusable`)
- Para mantenedores: `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` voltaram a passar limpos, os cinco testes vermelhos ficaram verdes, `src/vps/edit/apply_tests.rs` usa `tempfile` em vez de um caminho fixo em `/tmp`, e o registro de auditoria da 0.5.6 continua local e não publicado
- As skills de agente `skills/ssh-cli-en` e `skills/ssh-cli-pt` agora ensinam as 48 folhas de comando, `ssh-cli vps rename`, a troca de auth, o passo SANITIZE, a regra do `sudo` cru, a regra de confiança no primeiro uso (TOFU) e a forma persistente do `--disable-sudo`, então substitua qualquer cópia instalada antes da 0.5.6
- O gate `test` de `scripts/check_all_gates.py` roda `cargo test --locked --all-features --no-fail-fast`, então um binário de teste vermelho não esconde mais os outros
- Quatro suítes novas, `gaps_v073_config_dir_mode`, `gaps_v073_host_name_charset`, `gaps_v073_terminal_escape` e `gaps_v073_vps_auth_switch`, impedem que essas correções voltem, e o [TESTING](TESTING.pt-BR.md) cataloga toda suíte do disco
- Passo a passo adicional: o [COOKBOOK](COOKBOOK.pt-BR.md) documenta a receita de renomeação, inclusive por que ela não custa credencial

### Desde 0.5.5
- BREAKING: um posicional solitário não designa mais o host (GAP-SSH-EXEC-ARGC-001, CWE-441), `ssh-cli exec`, `ssh-cli sudo-exec` e `ssh-cli su-exec` liam o alvo pela contagem de posicionais, então `ssh-cli exec prod --step uptime` interpretava `prod` como o comando e o rodava no host que o `ssh-cli connect` marcara por último
- Mesmo argv, outra máquina, nenhum diagnóstico
- Um posicional agora é erro de uso (exit 64)
- BREAKING: `ssh-cli health-check` também exige alvo. Um `ssh-cli health-check` sem nome sai 64 em vez de sondar o marcador ativo em silêncio
- Alcance o marcador com `--use-active`: `ssh-cli health-check --use-active` e `ssh-cli exec --use-active <COMMAND>` são as únicas formas de herdar o que o `ssh-cli connect` escreveu
- Sem marcador em disco elas saem 66
- A divisão é estável em toda a CLI: 64 significa nenhum alvo designado, 66 significa alvo designado inexistente
- Três formas legais, e nenhuma outra: um nome (`ssh-cli exec prod uptime`), um seletor de frota (`--all`, `--hosts`, `--tags`), ou o opt-in explícito `--use-active`
- Wrappers que dependiam da forma implícita precisam escolher uma
- `scp` e `sftp` seguem a mesma regra: Transferências nomeiam o host no argv
- Slots `--src`/`--dest` e posicionais não podem ser misturados, para que uma palavra sobrando nunca vire nome de host em silêncio
- Todo envelope agora carrega a própria proveniência: `target_resolved`, `target_source` (`argv` | `active_marker` | `selector`) e `active_fallback` viajam no sucesso, e no envelope de erro assim que o host é resolvido, ausentes, nunca vazios, quando a falha antecede a resolução
- `host_resolved` e `host_source` são aliases de leitura dos dois primeiros, escritos a partir do mesmo valor
- Vincule consumidor novo à grafia canônica `target_*`
- O `ssh-cli doctor` reporta `local.active_vps`
- QUEBRA para quem usa a biblioteca: `ssh_cli::telemetry` agora é `ssh_cli::tracing_setup`, sem alias deprecado, e o módulo nunca emitiu telemetria

### Desde 0.5.4
- BREAKING: falha parcial multi-host agora sai com exit `1` (era `65`). Todo wrapper que ramificava em `65` para significar "alguns hosts falharam" precisa ser atualizado
- `65` é `EX_DATAERR`, que afirma que a entrada estava malformada
- Um lote em que três de dez hosts ficaram inalcançáveis tinha entrada perfeitamente válida, então o código antigo mentia sobre a causa (G-ERR-R02)
- Verifique os resultados por host no envelope do lote, não apenas o exit do processo
- BREAKING: `ssh-cli tunnel --bind` fora do loopback agora exige `--i-accept-network-exposure`. Todo script que passa `--bind 0.0.0.0` ou endereço de LAN falha até o reconhecimento ser acrescentado
- Antes disso, a flag publicava em silêncio o serviço remoto encaminhado para toda a rede local (G-TUN-R13)
- Sob `--reverse` a mesma flag protege o endereço de bind do servidor, porque é essa a ponta exposta naquele sentido
- `ssh-cli tunnel` ganhou três modos: O forward local padrão não mudou, então invocações existentes continuam funcionando
  - `--reverse` (G-TUN-R01) pede ao servidor que escute e entregue conexões de volta à sua porta local, o caso do webhook de callback e do bastion invertido, que antes forçava o operador a sair desta CLI e usar o `ssh` do sistema
  - `REMOTE_PORT` pode ser `0`, deixando o servidor alocar e informar a porta
  - `--socks5` (G-TUN-R02) serve um proxy SOCKS5 local (RFC 1928 no-auth + CONNECT) escolhendo o destino por conexão, então `REMOTE_HOST`/`REMOTE_PORT` são omitidos
  - Substitua N processos `ssh-cli tunnel` apontados a N hosts atrás de um bastion por uma invocação: o handshake é pago uma vez
  - `--remote-socket <PATH>` (G-TUN-R03) encaminha para socket Unix remoto via `direct-streamlocal@openssh.com`, alcançando sockets de daemon de contêiner, PostgreSQL e systemd que nunca escutam em TCP
  - O caminho deve ser absoluto ou sai com exit 64
- Novo evento `tunnel_closed`: `ssh-cli tunnel --json` agora o emite no encerramento com `reason`, `forwards_served` e `capacity_waits`
- Acrescente-o a qualquer parser que tratava `tunnel_listening` como o único evento de tunnel
- É a única forma de distinguir deadline limpo de semáforo saturado, já que todos os finais saem com exit 0
- Oito flags de redução de payload agora são globais: `--select` (apelido `--fields`), `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- Elas agem antes da serialização, então trocar um post-filtro JSON externo por essas flags impede que o envelope gigante seja escrito
- Nada muda quando elas são omitidas
- `--no-input` e `--dry-run` são globais: `--dry-run` é honrada somente por `ssh-cli vps remove`, `ssh-cli vps import`, `ssh-cli vps edit`, `ssh-cli vps rename`, `ssh-cli sftp rm`, `ssh-cli sftp rmdir`, `ssh-cli secrets init` e `ssh-cli secrets reencrypt`
- Em qualquer outro lugar é recusada com exit 64 em vez de aceita e ignorada
- Correções de segurança: atualize só por elas: A1 trunca o banner SSH pré-auth enviado pelo servidor por fronteira de caractere, e não por índice de byte, o teto de 512 caracteres no log já existia (G-SSH-14)
- O que estava quebrado é que cortar por índice de byte entra em pânico sempre que o corte cai no meio de um caractere multibyte, e o perfil de release usa `panic = "abort"`, então não há unwind: o processo morre e derruba um fan-out multi-host inteiro, A2 cria as chaves privadas ACME/mTLS em `0600` em vez de restringir o modo depois de o arquivo já existir, A3 mascara os modos enviados pelo servidor para triplas `rwx` simples, então setuid, setgid e sticky não pegam carona num download para o arquivo local
- Ver `SECURITY.pt-BR.md` para o mecanismo de cada uma
- `ssh-cli schema dry-run` e `ssh-cli schema tunnel-closed` agora emitem seus documentos com exit 0 (antes exit 64), então a descoberta de schema cobre o catálogo inteiro

### Desde 0.5.3
- G1: upload SFTP não trunca mais o destino a zero bytes, prefira 0.5.3+ para todo SFTP
- Verifique com `sha256sum` no destino
- G2 / G14: verbosidade graduada (`-v` info / `-vv` debug / `-vvv` trace) e sempre com escopo na crate (`warn,ssh_cli=…`)
- Debug global nu removido (sem vazamento de senha via russh)
- `RUST_LOG` ambiente continua ignorado
- G3: SETSTAT SFTP envia `atime`+`mtime` juntos (sem atime no epoch)
- G4: Result de `set_metadata` SFTP é fail-closed (SETSTAT mutante não é best-effort)
- G5 / G17: cancelamento multi-arquivo SCP/SFTP preenche o resto cancelled
- `results.len() == input.len()`
- G6: testes de signal/cancel que tocam estado global (`CANCEL_FLAG`) usam isolamento `serial_test` para a suíte ser determinística (não é runtime de agente)
- G7: E2E real oficial cobre matriz de checksum SFTP + árvore recursiva (E17/E18)
- Matriz completa E01–E21
- G8: `ssh-cli exec --json` de passo único emite exatamente um objeto JSON (sem dual-events no caminho de sucesso)
- G9: download SCP propaga falha de `sync_data` antes da renomeação atômica
- G10: gate de release inclui `cargo fmt --check`
- G11: suíte baseline fica verde sem loteria de re-execução
- Re-rodar até passar não é estratégia de gate
- G12 / G19: bits de permissão mascarados com `SFTP_PERM_MASK` nomeado (`0o7777`) no caminho de saída (upload)
- Desde a 0.5.4 o caminho de entrada (download) mascara com `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`), porque esse modo vem do servidor, veja A3 acima
- G13 / G15: sem testes circulares que afirmam texto FIXED no registro de auditoria local
- Aceite exige prova de efeito no destino (checksum)
- O registro de auditoria é local do mantenedor (ignorado pelo git / excluído do cargo) e não é contrato publicado
- G16: identificadores em inglês e erros de canal no caminho do cliente SCP (`client_real_scp.rs`)
- G18: falhas de `set_permissions` local no download SFTP são sinalizadas

### Desde 0.3.4 (paridade de automação SSH central)
- Grafo de crypto de install fixado para `cargo install --locked` funcionar (GAP-014)
- Auth aceita chaves privadas via `--key` / `key_path` (GAP-002)
- Semântica de `max_chars` dividida em `max_command_chars` e `max_output_chars` (GAP-004)
- `ssh-cli sudo-exec` empacota comandos com `sh -c` seguro (GAP-005)
- `ssh-cli su-exec` consome senha `su` armazenada (GAP-003)
- Escrita de config atômica com flock e mode 0600 (GAP-007)
- Host keys usam known_hosts TOFU (GAP-008)
- `ssh-cli tunnel` exige `--timeout-ms` (GAP-010)
- Schema version de registros novos era 2 na época (histórico, wire atual é schema v3)
- Licença dual MIT OR Apache-2.0

### Desde 0.3.5
- `ssh-cli vps export` atômico, abort remoto mais forte (TERM+KILL)
- Caminho AEAD opcional maduro
- Doctor reporta `secrets_at_rest`
- JSON automático quando stdout não é TTY

### Desde 0.3.6
- Cifragem em repouso padrão de segredos em `config.toml` (ChaCha20-Poly1305)
- Auto-cria XDG `secrets.key` (0o600) na primeira gravação de segredo
- CLI `ssh-cli secrets status`, `ssh-cli secrets init` e `ssh-cli secrets reencrypt` (nunca imprime a primary-key)
- Opt-out só para testes: `--allow-plaintext-secrets` (só CLI, sem cofre em variável de ambiente)
- Doctor: `secrets_key_file`, `secrets_plaintext_opt_out`

### Desde 0.3.7
- Polimento de I/O para agentes: `--output-format` global em VPS CRUD, `ssh-cli health-check --json`, envelope de erro JSON, `--quiet` silencia sucesso humano
- Tunnel `--timeout-ms` cobre connect SSH + loop
- SCP valida arquivo local antes do connect
- `ssh-cli vps remove` limpa `active` órfão
- `ssh-cli su-exec --password-stdin`
- Conflitos clap para password/*_stdin
- Exit remoto não-zero mapeia para exit de processo `1` com `remote_exit_code` no envelope JSON
- Segredos longos sempre mascaram como a máscara fixa (sem vazamento de prefixo 12+4)
- Senha sudo/su no stdin do canal, não em argv remoto

### Desde 0.3.8
- russh atualizado para 0.62.2 (piso de segurança ≥0.60.3)
- Banners humanos de tunnel fora do stdout do agente (JSON/non-TTY/quiet)
- Sem VPS ativa retorna sysexits 66 (`EX_NOINPUT`) via erro tipado
- `cargo deny`: `yanked=deny`, ignore list vazia
- `multiple-versions=warn` para duplicatas transitivas
- String de versão reporta `-dirty` quando a working tree está suja
- Suíte residual completa `tests/gaps_v038_integration.rs`

### Desde 0.4.1 (histórico)
- Patch AUD-POST: secrets vazios nunca viram blob `sshcli-enc` no export redigido (EXP-001)
- Deadline do tunnel pós-bind sai 0 (TUN-002)
- Paridade de flags auth em `ssh-cli tunnel`/`ssh-cli health-check` (CLI-005/006)
- JSON SCP com `event: "scp-transfer"` (IO-009)
- Só aditivo: sem breaking
- Correção do formato de rede SCP (0.4.0): crates.io 0.3.9 SCP quebrado
- Atualize para 0.4.0+ (prefira a linha de produto 0.5.6) antes de depender de `scp`
- SCP é somente arquivos regulares (sem `-r`)
- Árvores usam `sftp --recursive`
- Use `--timeout` para arquivos grandes (cobre connect + transfer)
- JSON de sucesso via `--json` / `--output-format json` (`docs/schemas/scp-transfer.schema.json`, e `sftp-transfer.schema.json` para SFTP)
- Download SCP grava `{path}.ssh-cli.partial` e faz renomeação atômica
- Mode/times aplicados no arquivo parcial antes da renomeação
- Upload SCP faz stream em blocos de 32 KiB (sem `fs::read` do arquivo inteiro na RAM)
- Preserve mtime/mode bidirecional (remoto `scp -tp` / `-fp`, parse de `T` + mode `C`)
- Paridade de flags SCP com exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json`
- Falhas de `scp --json` emitem envelope de erro JSON em stderr (`exit_code`, `message`), paridade com tunnel (IO-007b)
- `ssh-cli tunnel --json` emite um objeto stdout `event: "tunnel_listening"` após o bind local (`docs/schemas/tunnel-listening.schema.json`)
- Ainda exige `--timeout-ms`
- Tracing default error (não info)
- `-v` ativa debug
- `RUST_LOG` ambiente é ignorado: stderr JSON/tunnel limpo por omissão
- Senha vazia ou ausente em VPS só-chave serializa como JSON `null` (não a string da máscara)
- Não vazia ainda mascara como a máscara fixa
- O texto humano em show imprime "(not set)" para campo vazio, em inglês e em pt-BR, porque o rótulo não é traduzido
- `ssh-cli health-check` aceita override `--timeout <ms>` (alinhado ao exec)
- Docs de product line daquela era alinhados a 0.4.1
- Suítes `tests/gaps_v039_integration.rs` + `tests/gaps_v040_integration.rs` + `tests/gaps_v041_integration.rs`
- E2e oficial E01–E14 (E10–E14 cobrem SCP)

### Desde 0.4.2 (histórico, aditivo)
- Porta local efêmera de tunnel 0: após o bind, JSON/banner reportam a porta atribuída pelo SO (nunca 0 pós-bind) (TUN-003)
- Envelope formal de `ssh-cli vps export --json` (`event: "vps-export"`) amadurecido
- Secrets vazios permanecem `""` no export redigido
- e2e oficial E15 (tunnel porta 0) + E16 (symlink)
- Suíte `tests/gaps_v042_integration.rs`


## Migração passo a passo
### Atualize o binário

```bash
cargo install ssh-cli --locked --force
ssh-cli --version
```

### Valide inventário e modo de segredos

```bash
ssh-cli secrets status --json
ssh-cli vps doctor --json
ssh-cli vps list --json
```

### Atualize da 0.5.5 para a 0.5.6
- Passo 1: instale com `cargo install ssh-cli --locked --force` e confirme que `ssh-cli --version` começa com `ssh-cli 0.5.6`
- Passo 2: leia os três campos novos do `ssh-cli doctor`

```bash
ssh-cli doctor --json --select local.invalid_host_names
ssh-cli doctor --json --select local.config_dir_exposed
ssh-cli doctor --json --select local.config_dir_permissions
```

- Passo 3: renomeie todo host legado listado em `invalid_host_names` para um nome estrito com `ssh-cli vps rename 'old name' new-name`, porque o export de um host legado só restaura depois do rename
- Passo 4: quando `config_dir_exposed` for `true`, aperte a pasta você mesmo no Unix (por exemplo `chmod 700` na pasta), porque a CLI nunca altera o modo de um diretório que não é dela por padrão
- Passo 5: atualize wrappers que esperam `vps-edited` depois de `ssh-cli vps edit --name`, para que aceitem `vps-renamed`
- Passo 6: atualize wrappers que dependiam das recusas antigas, porque `--key` num host de senha e `--password` num host de chave agora funcionam, duas flags de auth na mesma chamada saem 2, e nome de host fora de `[A-Za-z0-9._-]` sai 64
- Passo 7: atualize wrappers que parseiam o modo texto, porque caracteres de controle e bidi agora aparecem como escapes no estilo `\u{1b}` enquanto o JSON não muda
- Passo 8: troque `bash scripts/<name>.sh` por `python3 scripts/<name>.py` nos scripts de release, e use `py -3 scripts/<name>.py` no Windows
- Passo 9: quem usa a biblioteca chama `VpsName::try_new` para nomes de host, abandona `Money` e `rust_decimal`, e faz match de `SshCliError::Unavailable { service }` com um braço curinga
- Passo 10: substitua as cópias instaladas das skills de agente e confira o resultado com `ssh-cli vps list --json` e `ssh-cli schema vps-edit`

### Se ainda houver segredos plaintext em disco
- Na primeira gravação com 0.3.6+, um `secrets.key` é auto-criado e novas gravações cifram
- Para re-cifrar inventário plaintext existente:

```bash
ssh-cli secrets init   # if secrets.key does not exist yet
ssh-cli secrets reencrypt
```

- Faça backup offline de `config.toml` e `secrets.key`
- Perder a chave torna blobs cifrados ilegíveis

### Troque um host de senha para chave

```bash
ssh-cli vps edit prod --key ~/.ssh/id_ed25519
printf '%s' '...' | ssh-cli vps edit prod --password-stdin
```

- Desde a 0.5.6 cada chamada troca o método direto e limpa os segredos antigos
- Antes, a primeira saía 64 e o desvio passava por `--use-agent`

### Revalide segredos de elevação (prefira stdin)

```bash
printf '%s' '...' | ssh-cli vps edit prod --sudo-password-stdin
ssh-cli sudo-exec prod "id"
ssh-cli su-exec prod "id"
```

### Atualize wrappers de agentes
- Passe `--timeout-ms` em tunnels
- Em `ssh-cli tunnel --json`, aguarde `event == "tunnel_listening"` antes de usar a porta local
- TUN-002: após `tunnel_listening`, o deadline one-shot pós-bind sai com exit 0 (não trate 74 como falha se o bind já foi sinalizado)
- Timeout pré-bind permanece 74
- EXP-001: em `ssh-cli vps export` redigido, não espere nem parseie `sshcli-enc:` para secrets vazios, vazios serializam como `""`
- IO-009: parseie sucesso SCP com `docs/schemas/scp-transfer.schema.json` incluindo `event: "scp-transfer"` obrigatório
- CLI-005: `ssh-cli tunnel` aceita `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`
- CLI-006: `ssh-cli health-check` aceita `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`
- Se veio de 0.4.0: export redigido podia mostrar texto cifrado falso de senha vazia
- Tunnel podia emitir `ok:true` e sair 74: atualize wrappers e o binário para 0.5.3
- Em falha de SCP/tunnel com `--json`, parseie o envelope de erro em stderr (não prosa humana)
- Trate SCP como somente arquivos regulares
- Não envie árvores de diretório
- Prefira 0.5.3+ para SFTP
- Re-verifique uploads com checksum no destino após a atualização (G1)
- Re-teste transferências após sair do 0.3.9 (SCP daquela release não era confiável)
- Trate `--maxChars` como limite de entrada, não de saída
- Prefira `--password-stdin` para segredos
- Senha em argv avisa em stderr (0.5.2+)
- O `ssh-cli vps add --timeout` com valor abaixo de 1000 ms avisa em stderr (unidade é milissegundos, não segundos), e o `ssh-cli vps edit --timeout` aceita em silêncio
- Comando remoto vazio falha com mensagem técnica `empty command` (qualquer locale)
- Trate erros de mismatch de host-key antes de forçar replace
- Espere valores cifrados em `config.toml` com prefixo `sshcli-enc:v2:`, porque toda gravação nova é `v2` e os blobs `v1` ainda são lidos
- Espere tracing default error
- Use `-v`/`-vv`/`-vvv` ao diagnosticar (`RUST_LOG` ambiente é ignorado)
- Não parseie stderr como JSON de sucesso
- Parseie `ssh-cli exec --json` de host único como um objeto (G8)
- ACME `invalidContact` / validação permanente → exit 64 (não faça retry como 74) (G-E2E-01)
- Primeiro `ssh-cli vps add` com auto-key: um documento JSON `event: "vps-added"` com campo `secrets_key_auto_created` (G-E2E-04)
- Prefira root `ssh-cli schema` / `ssh-cli doctor` para descoberta de agente (G-E2E-02/03)
- Cadastre hosts só-agent com `ssh-cli vps add --use-agent` / `--agent-socket` (G-E2E-19)
- O export redigido limpa os segredos em vez de mascará-los: `password` vale `""`, e `sudo_password`, `su_password` e `key_passphrase` valem `null`
- A máscara fixa aparece só em `ssh-cli vps list` e `ssh-cli vps show`, onde um host só-chave mostra `password` como `null` (G-E2E-10)
- Feature clap `env` removida: sem `#[arg(env=…)]` de config de produto (G-E2E-08)
- Stamp de versão anexa `-dirty` quando a working tree está suja mesmo com `.commit_hash` (G-E2E-06)
- Trate senha vazia em list/show JSON como `null` em hosts só-chave
- Pode passar `ssh-cli health-check --timeout <ms>` quando o timeout padrão do host for longo ou curto demais
- Espere exit de processo `1` (com `remote_exit_code` no envelope JSON) quando o comando remoto falhar
- Espere sem VPS ativa como exit 66
- Arquivo SCP ausente como exit 66 com `file not found: <path>`
- Espere banners de tunnel só em caminhos humanos/TTY, não no stdout JSON do agente
- Controle de secrets é só CLI/XDG (`--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, XDG `secrets.key`)
- Stores env de secrets são rejeitados fail-closed
- Assuma que JSON auto non-TTY se aplica a `ssh-cli vps export`, o corpo é JSON em qualquer stdout non-TTY, e TOML exige `--output-format text`


## Mudanças de JSON Schema
- Histórico (era 0.3.4): registros novos gravavam `schema_version` 2 com o conjunto de campos daquela release
- Atual (0.5.3): novas escritas usam schema v3 e chaves TOML em inglês
- A leitura faz leitura dupla de aliases legados em português
- Schemas de eventos de agente ficam em `docs/schemas/` (veja [schemas/README.md](schemas/README.md))

### 0.5.6 — antes e depois
- Catálogo de `ssh-cli schema`: antes, 22 documentos. Depois, 25, porque `vps-edit.schema.json`, `vps-rename.schema.json` e `vps-import.schema.json` entraram
- `ssh-cli vps doctor --json`, objeto `local`: antes, sem campos de diretório. Depois, três campos, mostrados aqui como medidos num diretório de config novo

```json
{"local":{"config_dir_exposed":false,"config_dir_permissions":"0700","invalid_host_names":[]}}
```

- Planos de `--dry-run` de `ssh-cli vps edit`, `ssh-cli vps rename` e `ssh-cli vps remove`: antes, sem eco do alvo. Depois, os quatro campos de alvo, mostrados aqui para `ssh-cli vps remove` sem o campo `config_path`

```json
{"ok":true,"event":"dry-run","operation":"vps-remove","name":"h1","dry_run":true,"executed":false,"target_resolved":"h1","target_source":"argv","host_resolved":"h1","host_source":"argv"}
```

- `ssh-cli vps edit --json`: antes, sem lista `updated` e sem schema. Depois, `updated` está sempre presente e o `vps-edit.schema.json` a tipa
- Os 19 nomes que `updated` pode conter são `host`, `port`, `user`, `password`, `key_path`, `key_passphrase`, `use_agent`, `agent_socket`, `timeout_ms`, `max_command_chars`, `max_output_chars`, `sudo_password`, `su_password`, `disable_sudo`, `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key` e `tags`
- `ssh-cli vps edit h1 --agent-socket S` num host de senha, como medido

```json
{"ok":true,"event":"vps-edited","name":"h1","target_resolved":"h1","target_source":"argv","host_resolved":"h1","host_source":"argv","updated":["use_agent","password","agent_socket"]}
```

- `ssh-cli vps rename` e `ssh-cli vps edit --name`: os dois emitem `vps-renamed` com `name`, `previous_name`, `active_rewritten`, `updated`, `target_resolved` e `target_source`, e o `vps-rename.schema.json` o tipa
- `ssh-cli vps import --json`: antes, sem lista dos hosts sobrescritos. Depois, `replaced` é um array ordenado, sempre presente, e o `vps-import.schema.json` o tipa

```json
{"ok":true,"event":"vps-import","imported":1,"replaced":["h1"]}
```

- `ssh-cli vps export`: antes, o corpo não trazia campos de agente. Depois, cada host traz `use_agent` e `agent_socket`, e o `vps-export.schema.json` declara os dois
- `ssh-cli vps export -o FILE --json`: antes, o `vps-export.schema.json` exigia `schema_version` e `hosts`, então um validador reprovava o resumo que o `-o` imprime. Depois, `oneOf` aceita exatamente um de dois formatos, o corpo (`schema_version` e `hosts`) ou o resumo (`path` e `format`), como medido

```json
{"ok":true,"event":"vps-export","format":"json","include_secrets":false,"path":"rel.json"}
```

- `error-envelope` em stderr: o formato não muda, e a recusa de nome estrito chega com exit 64 e `error_code` `invalid_argument`, como `invalid VPS name 'bad;x'` em `ssh-cli vps rename` e `ssh-cli vps edit --name`, enquanto `ssh-cli vps add` deixa o nome entre aspas fora da mensagem
- Duas flags de auth na mesma chamada saem 2 com o texto de uso do clap e sem envelope JSON, porque a recusa acontece no parse
- O JSON nunca escapa caracteres de controle ou bidi além do que o `serde_json` já faz, porque JSON é para máquina

### Após 0.3.4+
- `timeout_ms`
- `max_command_chars`
- `max_output_chars`
- `key_path`
- `key_passphrase` (mascarado)
- `disable_sudo`
- `schema_version` 2 (somente escritas históricas, wire atual é schema v3)

### Segredos em repouso (era 0.3.6, ainda atuais)
- Campos password/sudo/su/passphrase podem armazenar blobs `sshcli-enc:v2:…`, e os blobs legados `sshcli-enc:v1:…` ainda são lidos
- Prefira flags CLI: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- Fontes de primary-key: CLI `--secrets-key-file` / `--use-keyring`, ou XDG `secrets.key`
- `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` são rejeitadas com falha fechada (não são cofre)
- Todo comando que lê ou grava segredo do registro sai 64 com `are not supported` quando uma das duas variáveis está definida: `ssh-cli vps list`, `ssh-cli vps show`, `ssh-cli doctor`, `ssh-cli vps export`, `ssh-cli vps add`, `ssh-cli vps edit` e `ssh-cli secrets status`
- Comandos que não tocam segredo saem 0 com a variável definida: `ssh-cli vps path`, `ssh-cli locale show`, `ssh-cli schema` e `ssh-cli commands`

### Mascaramento (0.4.0)
- Senha vazia → JSON `null`
- Não vazia → a string mascarada
- O texto humano em show ainda imprime "(not set)" para senha vazia

### Eventos de transfer / tunnel (0.4.0 / 0.4.1+)
- JSON de sucesso SCP inclui `event: "scp-transfer"` obrigatório (IO-009)
- Tunnel continua emitindo `event: "tunnel_listening"` após bind
- Sucesso SCP: `docs/schemas/scp-transfer.schema.json`
- Tunnel listening: `docs/schemas/tunnel-listening.schema.json`
- Falhas em modo JSON: `docs/schemas/error-envelope.schema.json` em stderr


## Notas de compatibilidade
- Hosts TOML existentes carregam e migram defaults de campos nos caminhos de leitura/gravação
- Alias legado `--maxChars` mapeia para limite de entrada de comando
- Prefira `--max-command-chars` e `--max-output-chars`, porque `--max-chars` já é só um alias legado do limite de comando
- Timeout padrão é 60000 ms para automação de agentes
- Comportamento always-trust de host key sumiu em builds de release
- Cifragem padrão ligada
- Plaintext exige opt-out explícito só via CLI `--allow-plaintext-secrets` (stores env de secrets são rejeitados fail-closed)
- Tracing padrão é error
- Prosa INFO não é esperada no stderr do agente
- SCP permanece somente para arquivos por design em 0.4.0+ (ainda verdade em 0.5.6, não é limitação temporária)
- Integridade SFTP exige 0.5.3+ (G1)
- Não confie em upload SFTP pré-0.5.3 sem checksum externo
- Os `config.toml` existentes seguem carregando na 0.5.6, e um nome de host guardado com espaço, acento, controle ou bidi é aceito, listado pelo `ssh-cli doctor` e reparável com `ssh-cli vps rename`
- Um registro legado com caracteres de controle em tag, `host` ou `user` ainda sai 65, porque só o NOME do host é tolerante
- Um nome legado não grava aviso no carregamento, então o `ssh-cli doctor` é o lugar que o lista
- Parsers que casavam bytes crus da saída em modo texto devem esperar escapes no estilo `\u{1b}`, enquanto o JSON mantém todo caractere cru
- Os corpos de erro ficam em inglês por desenho, e o stdout e stderr remotos do `ssh-cli exec` ficam crus por desenho
- O projeto segue o Semantic Versioning 2.0.0 numa linha 0.x, então uma release pode trazer mudanças BREAKING, e a 0.5.4, a 0.5.5 e a 0.5.6 listam algumas
- Uma deprecação é registrada na seção `Deprecated` da entrada do CHANGELOG da release que a introduz, e a entrada da 0.5.6 não tem nenhuma
- O repositório não fixa um número mínimo de versões menores para uma API obsoleta, e a renomeação de `ssh_cli::telemetry` na 0.5.5 saiu sem alias
- Exit codes e JSON schemas não ficam congelados entre as releases 0.5.x, porque a 0.5.4 moveu a falha parcial multi-host do exit 65 para o exit 1 e a 0.5.6 acrescentou três schemas, então leia as linhas BREAKING antes de cada upgrade
- O `--select`, o `--filter` e o `--count-only` globais não alcançam o objeto do `ssh-cli doctor`, então leia-o com `jaq`
- Windows e macOS só passam por type-check no gate `cross-targets`, e o modo `0700` e a checagem de `getuid` nunca foram executados lá


## Reversão
- Reinstale versão anterior com pin exato se necessário
- Mantenha um export redacted via `ssh-cli vps export` antes de experimentos grandes
- Um export redacted restaura só o esqueleto dos hosts, então guarde também `config.toml` e `secrets.key` offline
- Se voltar abaixo de 0.3.6, blobs cifrados exigem a primary-key correspondente ou re-export em plaintext ainda na 0.3.6+
- Se voltar para 0.3.9, não espere wire SCP funcional (atualize de novo para 0.4.0+ para transferências)
- Se voltar abaixo de 0.5.3, não confie na integridade de upload SFTP sem checksum externo (G1)
- Voltar da 0.5.6 para a 0.5.5 mantém `schema_version` 3 no `config.toml`, mas nenhum teste cobre esse rollback, então guarde antes um export redacted
- A 0.5.5 não tem `ssh-cli vps rename`, nem flags de limpeza, nem `--lock-wait-ms`, e sai 64 quando você troca o método de auth de um host
- Wrappers escritos para a 0.5.6 não encontram `vps-renamed`, `updated` nem `local.config_dir_exposed` na 0.5.5
- Um host criado com nome estrito da 0.5.6 carrega em versões antigas, e um nome legado reparado com `ssh-cli vps rename` continua reparado


## Formato wire: schema v3: introduzido na 0.5.3, vigente na 0.5.6
- O `schema_version` atual para novas escritas é 3 (não 2)
- Novas escritas usam chaves TOML em inglês: `name`, `port`, `username`, `password`, `added_at`, …
- O load ainda aceita chaves legadas em português (`nome`, `porta`, `usuario`, `senha`, `adicionado_em`), dual-read serialize EN / aliases PT no load
- `added_at` é opcional no import (padrão: agora quando ausente)
- O corpo de `ssh-cli vps export` segue o formato resolvido: o envelope de agente (`event: "vps-export"`) em qualquer stdout non-TTY, inclusive num arquivo `.toml`
- `--output-format text` é o único caminho para corpo TOML
- JSON auto non-TTY se aplica ao export como em todo o resto
- `ssh-cli vps import` aceita TOML (EN + aliases PT) ou envelopes JSON `vps-export`
- `--allow-incomplete` para hosts redacted/skeleton
- `--include-secrets` exige `-o`/`--output` ou `--i-understand-secrets-on-stdout`
- Controle de secrets é só CLI/XDG: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, ou XDG `secrets.key` (stores env de secrets rejeitados fail-closed)
- Termo preferido para a chave at-rest é primary-key
- Entradas legadas de keyring rotuladas master-key ainda são legíveis
- `ssh-cli secrets init --json` → `event: "secrets-init"`
- `ssh-cli secrets reencrypt --json` → `event: "secrets-reencrypt"`
- A primeira gravação pode definir `secrets_key_auto_created: true` no mesmo JSON de sucesso (um documento)
- Eventos de sucesso CRUD em JSON efetivo: `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import`
- Tunnel `--bind` tem padrão `127.0.0.1` (loopback)
- Exit 65 cobre `TomlDe` / dados ruins de import
- Exit 77 é auth/host-key/permissão
- Arquivo SCP ausente é exit 66 com `file not found: <path>`
- Verbosidade graduada `-v`/`-vv`/`-vvv` (info/debug/trace), sempre com escopo na crate (G2/G14)
- Integridade de upload SFTP corrigida (G1)
- SETSTAT atime+mtime (G3)
- Set_metadata fail-closed (G4)
- Máscara de perms (G12)
- Cardinalidade de cancel em batch (G5/G17)
- Designação explícita de alvo (0.5.5): o envelope single-host de `ssh-cli exec`/`ssh-cli sudo-exec`/`ssh-cli su-exec` ganhou `target_resolved` e `target_source`, vincule consumidores novos a esses
- `host_resolved`/`host_source` são aliases de leitura escritos do mesmo valor, nunca fonte de verdade independente
- `active_fallback` é o atalho booleano para `target_source == "active_marker"`, então um pipeline de shell ramifica sem saber como o enum é grafado
- O trio de proveniência fica ausente, nunca vazio, no envelope de erro quando a falha antecede a resolução do alvo: a ausência diz que nenhum host foi escolhido, enquanto `""` afirmaria que um foi escolhido e não tem nome
- Um posicional solitário na família exec passa a ser erro de uso (exit 64)
- Alcance o marcador que o `ssh-cli connect` gravou com a `--use-active <COMMAND>` explícita, que sai 66 quando não há marcador
- Os seletores de frota `--all`, `--hosts` e `--tags` reportam `target_source: "selector"`
- Transferências nunca leem o marcador, então reportam apenas `argv` ou `selector`
- O `ssh-cli doctor` reporta `local.active_vps`, então o marcador é auditável sem ler o arquivo pelas costas da CLI
- Suítes: `tests/gaps_v042_integration.rs` + `tests/gaps_v051_integration.rs` + `tests/gaps_v057_sftp.rs` + `tests/gaps_v058_e2e_residual.rs`
- Designação de alvo em `tests/gaps_v065_exec_target_designation.rs` e `tests/gaps_v066_transfer_target_designation.rs`
- Alcance de empacotamento em `tests/gaps_v067_packaging_reach.rs`
- E2e oficial E01–E21
- Linha de produto: 0.5.6


## Veja também
- [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md): superfície de comandos do usuário
- [AGENTS.pt-BR.md](AGENTS.pt-BR.md): contratos de agente e roteamento de exit
- [COOKBOOK.pt-BR.md](COOKBOOK.pt-BR.md): receitas copy-paste
- [schemas/README.md](schemas/README.md): índice de schemas JSON
- [TESTING.pt-BR.md](TESTING.pt-BR.md): suítes de teste e bateria de gates
- [RELEASE_CHECKLIST.pt-BR.md](RELEASE_CHECKLIST.pt-BR.md): gates de release
- [CROSS_PLATFORM.pt-BR.md](CROSS_PLATFORM.pt-BR.md): matriz de alvos e modos do diretório de config
- [../CHANGELOG.pt-BR.md](../CHANGELOG.pt-BR.md): notas completas de release


## Notas de release
### 0.5.6 — release de edição e rename
- `ssh-cli vps rename <NAME> <NEW_NAME>` e `ssh-cli vps edit <NAME> --name <NEW>` renomeiam nativamente, então o desvio export/edit/import deixou de ser necessário
- O desvio nunca foi equivalente, porque `ssh-cli vps export` redige segredo e o import daquele esqueleto exigia redigitar a credencial à mão
- O export também não carregava `use_agent`/`agent_socket`, então host autenticado por agente voltava sem autenticação
- Toda credencial segue legível sob o nome novo: o registro sela com contexto não vinculado, então o dado associado não carrega nome de host e o rename não tem nada a selar de novo
- Oito flags de limpeza removem o campo em vez de sobrescrevê-lo
- `--dry-run` passa a cobrir oito operações
- O lock do config espera `--lock-wait-ms` limitado antes de sair 69 em vez de pendurar para sempre

### 0.5.5 — release de segurança de designação de alvo
- Corrige GAP-SSH-EXEC-ARGC-001: `ssh-cli exec`/`ssh-cli sudo-exec`/`ssh-cli su-exec` decidiam o host alvo pela contagem de posicionais
- Assim `ssh-cli exec <HOST> --step <CMD>` executava o nome do host como binário remoto no host que `ssh-cli connect` selecionara por último
- BREAKING: um único posicional agora é erro de uso
- Alcance o marcador ativo com a nova `--use-active <COMMAND>`
- O envelope single-host ganhou `target_resolved` e `target_source`, então vincule consumidores novos a esses
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
