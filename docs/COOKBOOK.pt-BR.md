# Cookbook


> Copie receitas executáveis que resolvem problemas reais de agentes SSH multi-host.

- Leia este documento em [Inglês (en)](COOKBOOK.md)
- Linha de produto: 0.5.6
- Toda receita segue as mesmas cinco partes: Problema, Solução, Explicação, Variantes e Veja também
- Toda saída vem de uma execução no binário 0.5.6, com nomes de host e caminhos trocados por marcadores genéricos
- `duration_ms`, `latency_ms` e `batch_run_id` variam entre execuções e `CONFIG_DIR` representa o seu diretório de configuração
- Siglas: SSH (Secure Shell, shell seguro), CLI (interface de linha de comando), LLM (modelo de linguagem grande), XDG (diretórios base do X Desktop Group), TLS (Transport Layer Security, segurança da camada de transporte), mTLS (TLS mútuo), ACME (Automatic Certificate Management Environment, ambiente de gestão automática de certificados), SCP (cópia segura), SFTP (protocolo de transferência de arquivos sobre SSH) e TOFU (confiar no primeiro uso)
- Molde a saída primeiro com `--select`, `--filter` e `--count-only` da própria CLI, as receitas usam `jaq` só para calcular ou testar um valor, e qualquer leitor de JSON serve
- As receitas leem só campos canônicos como `target_resolved` e `target_source`, a saída crua ainda traz os apelidos de leitura `host_resolved` e `host_source` e nenhuma receita os interpreta
- As mensagens do modo texto mostradas aqui vêm de `--lang en`, os corpos de erro JSON ficam em inglês em qualquer idioma


## Nota de latência
- Espere operações locais de cadastro em menos de um segundo e uma conexão SSH a frio dominada pelo tempo de ida e volta da rede
- Cinco execuções de `ssh-cli --version` levaram 25 ms no total no host de medição, então a partida do processo não é o custo
- Prefira comandos de execução única a túneis quando uma única ação remota basta


## Referência de valores padrão
- Porta padrão: 22
- Tempo limite padrão: 60000 ms
- `max_command_chars` padrão: 1000
- `max_output_chars` padrão: 100000
- Rastreamento padrão: error, com `-v` info, `-vv` debug e `-vvv` trace, restrito à crate, e `RUST_LOG` do ambiente ignorado
- `--lock-wait-ms` padrão: 5000 ms, teto 300000 ms, e quem grava e espera mais que isso sai com 69
- Senha vazia em list e show JSON: `null` (hosts só com chave)
- Segredos não vazios em list e show mascaram como três asteriscos (`FIXED_MASK`)
- Um export redigido os limpa: `password` é `""` e os segredos opcionais são `null`
- Telemetria: desligada
- Segredos em repouso: cifrados quando existe uma primary-key, a primeira gravação de senha cria o `secrets.key`
- Instalação: `cargo install ssh-cli --locked`
- Cadeia de suprimentos: russh 0.65.1 e `cargo deny` com `yanked=deny` e `multiple-versions=deny`
- SCP: somente arquivos regulares, sem `-r` e sem diretórios
- Árvores de diretórios e operações no sistema de arquivos remoto usam `sftp` (`--recursive`, `ls`, `mkdir`)
- Sufixo parcial do download SCP: `.ssh-cli.partial`
- O JSON de sucesso do SCP traz `event: "scp-transfer"` e o campo `mtime_preserved`
- Protocolo do SCP: use a 0.4.0 ou mais nova e prefira a linha de produto 0.5.6, nunca a 0.3.9, cujo SCP era inoperante
- SFTP: prefira a 0.5.3 ou mais nova (integridade de upload G1) e verifique o destino com `sha256sum`
- O corpo de `ssh-cli vps export` segue o formato resolvido: JSON em qualquer stdout não TTY e TOML somente com `--output-format text`
- Formato gravado dos hosts: schema v3 com chaves em inglês, as leituras ainda aceitam os apelidos legados em português
- Nomes de host: letras ASCII, dígitos, ponto, hífen e sublinhado, começando por letra ou dígito, de 1 a 255 bytes
- O prazo do tunnel depois do bind sai com 0 após `tunnel_listening` (TUN-002) e um tempo limite estourado antes do bind sai com 74
- Um tunnel encerrado por SIGINT sai com 130 e por SIGTERM sai com 143, com `reason` igual a `signal` no `tunnel_closed`
- `--bind` padrão do tunnel: `127.0.0.1`
- As flags de segredos são só CLI e XDG: `--allow-plaintext-secrets`, `--secrets-key-file` e `--use-keyring`
- Validação permanente do ACME como `invalidContact` sai com 64, não repita como 74
- Só `ssh-cli vps add --timeout` abaixo de 1000 ms avisa no stderr, enquanto `ssh-cli vps edit`, `ssh-cli exec` e `ssh-cli health-check` o aceitam em silêncio
- Uma senha no argv avisa no stderr, prefira as flags `--*-stdin`
- `SSH_CLI_SECRETS_KEY` e `SSH_CLI_SECRETS_KEY_FILE` são recusadas com exit 64, e as outras variáveis `SSH_CLI_*` são ignoradas
- Eventos de sucesso: `vps-added`, `vps-edited`, `vps-renamed`, `vps-removed`, `vps-connected` e `vps-import`
- A primeira gravação de segredo pode definir `secrets_key_auto_created: true` no mesmo documento `vps-added`
- Os números G das próximas três linhas são os identificadores da seção 0.5.3 do `CHANGELOG.md`
- Notas de correção: G1 integridade do upload SFTP, G2 e G14 verbosidade graduada restrita à crate, G3 SETSTAT com `atime` e `mtime` juntos, G4 `set_metadata` que falha fechado
- G5 e G17 mantêm `results.len() == input.len()` no cancelamento, G8 um objeto JSON por `ssh-cli exec`, G9 `sync_data` antes do rename, G15 checksum como prova
- G12 e G19 mascaram bits de permissão por direção com `SFTP_PERM_MASK` (`0o7777`) no upload e `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) no download (A3)
- G18 expõe falhas de `set_permissions` no download SFTP, G6 e G11 tratam só de isolamento de testes


## Índice de comandos
- A tabela liga cada um dos 48 comandos folha à receita que o executa
- Procure o comando aqui e depois busque o título da receita neste arquivo

| Comando | Receita |
|---|---|
| `ssh-cli vps add` | Como registrar um host de senha (stdin, sem vazar no argv) |
| `ssh-cli vps list` | Como reduzir o payload de um agente antes de ele ser escrito |
| `ssh-cli vps remove` | Como ensaiar uma operação destrutiva e ler o eco do alvo |
| `ssh-cli vps rename` | Como renomear um host sem digitar a credencial de novo |
| `ssh-cli vps edit` | Como editar um host campo a campo e limpá-lo |
| `ssh-cli vps show` | Como registrar um host só com chave |
| `ssh-cli vps path` | Como diagnosticar caminhos XDG e o modo de segredos |
| `ssh-cli vps doctor` | Como diagnosticar caminhos XDG e o modo de segredos |
| `ssh-cli vps export` | Como exportar e importar o inventário sem segredos |
| `ssh-cli vps import` | Como exportar e importar o inventário sem segredos |
| `ssh-cli connect` | Como designar o host alvo e provar onde rodou |
| `ssh-cli exec` | Como executar um comando remoto com JSON |
| `ssh-cli sudo-exec` | Como executar sudo seguro com comandos compostos |
| `ssh-cli su-exec` | Como elevar com su quando o sudo não está disponível |
| `ssh-cli scp upload` | Como transferir um artefato de release com SCP |
| `ssh-cli scp download` | Como transferir um artefato de release com SCP |
| `ssh-cli sftp upload` | Como enviar por SFTP e verificar o checksum |
| `ssh-cli sftp download` | Como baixar um arquivo e uma árvore via SFTP |
| `ssh-cli sftp ls` | Como gerenciar caminhos remotos via SFTP |
| `ssh-cli sftp mkdir` | Como gerenciar caminhos remotos via SFTP |
| `ssh-cli sftp rmdir` | Como gerenciar caminhos remotos via SFTP |
| `ssh-cli sftp rm` | Como gerenciar caminhos remotos via SFTP |
| `ssh-cli sftp stat` | Como gerenciar caminhos remotos via SFTP |
| `ssh-cli sftp rename` | Como gerenciar caminhos remotos via SFTP |
| `ssh-cli tunnel` | Como abrir um tunnel com prazo |
| `ssh-cli health-check` | Como sondar conectividade com health-check |
| `ssh-cli secrets status` | Como inicializar a cifragem com primary-key |
| `ssh-cli secrets init` | Como inicializar a cifragem com primary-key |
| `ssh-cli secrets reencrypt` | Como recifrar um inventário legado em texto puro |
| `ssh-cli completions` | Como gerar completions de shell |
| `ssh-cli commands` | Como descobrir contratos (schema, doctor, commands) |
| `ssh-cli schema` | Como descobrir contratos (schema, doctor, commands) |
| `ssh-cli doctor` | Como descobrir contratos (schema, doctor, commands) |
| `ssh-cli locale show` | Como definir o idioma da interface |
| `ssh-cli locale set` | Como definir o idioma da interface |
| `ssh-cli locale clear` | Como definir o idioma da interface |
| `ssh-cli tls provider` | Como inspecionar provider TLS, caminhos e ACME |
| `ssh-cli tls paths` | Como inspecionar provider TLS, caminhos e ACME |
| `ssh-cli tls mtls list` | Como importar uma identidade mTLS com um nome válido |
| `ssh-cli tls mtls import` | Como importar uma identidade mTLS com um nome válido |
| `ssh-cli tls mtls show` | Como importar uma identidade mTLS com um nome válido |
| `ssh-cli tls mtls remove` | Como importar uma identidade mTLS com um nome válido |
| `ssh-cli tls acme account create` | Como inspecionar provider TLS, caminhos e ACME |
| `ssh-cli tls acme account show` | Como inspecionar provider TLS, caminhos e ACME |
| `ssh-cli tls acme issue` | Como inspecionar provider TLS, caminhos e ACME |
| `ssh-cli tls acme complete` | Como inspecionar provider TLS, caminhos e ACME |
| `ssh-cli tls acme status` | Como inspecionar provider TLS, caminhos e ACME |
| `ssh-cli tls acme list` | Como inspecionar provider TLS, caminhos e ACME |


## Como inicializar a cifragem com primary-key

### Problema
- Os segredos ficam em texto puro no disco até existir uma primary-key

### Solução

```bash
ssh-cli secrets status --json
ssh-cli secrets init --json
ssh-cli secrets status --json
```

- Saída esperada das três chamadas, em ordem, em um diretório de configuração novo

```json
{"at_rest":"plaintext","encryption_active":false,"key_file":"CONFIG_DIR/secrets.key","key_source":"none","plaintext_opt_out":false}
{"ok":true,"event":"secrets-init","force":false,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","reencrypted_hosts":0}
{"at_rest":"encrypted","encryption_active":true,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","plaintext_opt_out":false}
```

### Explicação
- A chave nunca é impressa, o JSON só nomeia o arquivo e a origem
- Um segundo `ssh-cli secrets init` sai com 64 com `profile already has a primary-key in CONFIG_DIR/secrets.key; use --force to rotate it`
- A primeira gravação de senha cria a chave sozinha e imprime `secrets_key_auto_created: true`

### Variantes
- Aponte para o seu arquivo de chave com `ssh-cli --secrets-key-file /path/to/key secrets status --json`, que informa `key_source` igual a `file`
- Use o cofre de chaves do SO com `ssh-cli --use-keyring secrets init --json` ou `ssh-cli secrets init --keyring --json`, os dois chegam ao mesmo cofre
- Um cofre que não responde faz o `ssh-cli secrets init --keyring` sair com 69 em vez de reportar sucesso
- A chave é resolvida nesta ordem: `--secrets-key-file`, o cofre sob `--use-keyring`, o `secrets.key` no diretório de configuração e por fim uma chave gravada por um `ssh-cli secrets init --keyring` anterior
- Com uma chave já no arquivo ou no cofre, `ssh-cli secrets init --keyring` sem `--force` sai com 64, e com `--force` aposenta a fonte antiga para o perfil manter uma só primary-key
- Mantenha texto puro só para testes com `--allow-plaintext-secrets`

### Veja também
- Recifrar um inventário legado abaixo e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como descobrir contratos (schema, doctor, commands)

### Problema
- Um agente precisa dos formatos de saída e dos argumentos aceitos sem ler o código-fonte

### Solução

```bash
ssh-cli schema | jaq '.schemas | length'
ssh-cli schema vps-list
ssh-cli commands --json | jaq -r '.tree.subcommands[].name'
ssh-cli doctor --json
```

- O catálogo tem 25 schemas e `ssh-cli commands` lista os 16 comandos de nível raiz

```json
25
vps connect exec sudo-exec su-exec scp sftp tunnel health-check secrets completions commands schema doctor locale tls
```

### Explicação
- `ssh-cli schema` imprime o catálogo como o evento `schema-catalog` e `ssh-cli schema <name>` imprime um documento de schema
- `ssh-cli commands --json` imprime a árvore inteira com `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` e `hidden` de cada argumento
- `ssh-cli doctor --json` é o alias raiz de `ssh-cli vps doctor` e reporta caminhos, modo de segredos e o diretório de configuração

### Variantes
- Leia a lista de argumentos de um comando com `ssh-cli commands --json | jaq '[.. | objects | select(.name? == "rename")] | .[0].args'`
- Imprima o schema de um lote com `ssh-cli schema exec-batch`
- Leia um campo de um documento grande com `ssh-cli commands --json --select tree.about`, que imprime `{"agent_shape":{"selected":["tree.about"]},"tree":{"about":"..."}}` em vez da árvore inteira
- Leia os exit codes e exemplos copiáveis com `ssh-cli --help`, e um exemplo por folha com `ssh-cli exec --help`

### Veja também
- O índice de schemas em [docs/schemas/README.md](schemas/README.md)


## Como registrar um host de senha (stdin, sem vazar no argv)

### Problema
- Uma senha no argv aparece na lista de processos e no histórico do shell

### Solução

```bash
printf '%s' 'demo-password-not-real' | ssh-cli vps add \
  --name prod \
  --host prod.example.com \
  --user deploy \
  --password-stdin --json
```

```json
{"ok":true,"event":"vps-added","key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","name":"prod","secrets_key_auto_created":true}
```

- Sem stdin a chamada falha com exit 64 e `vps_auth: must provide exactly one of --password, --key, or --use-agent`

### Explicação
- `--password-stdin` mantém o segredo fora do argv, uma senha no argv funciona mas avisa no stderr
- A primeira gravação de senha criou o `secrets.key`, então `secrets_key_auto_created` é `true` uma vez
- `ssh-cli vps show prod --json` imprime a senha como três asteriscos, nunca o texto puro

### Variantes
- Registre pelo agente com `ssh-cli vps add --name lab --host 203.0.113.10 --user ubuntu --use-agent`
- Leia a senha de um arquivo com `ssh-cli vps add ... --password-stdin < pw-file`

### Veja também
- Troque o método depois com a receita de autenticação abaixo, e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como registrar um host só com chave

### Problema
- Um host com chave nunca deveria guardar senha nenhuma

### Solução

```bash
ssh-cli vps add --name edge --host edge.example.com --user ubuntu --key ~/.ssh/id_ed25519 --json
ssh-cli vps show edge --json
```

```json
{"ok":true,"event":"vps-added","name":"edge","secrets_key_auto_created":false}
{"name":"edge","host":"edge.example.com","port":22,"user":"ubuntu","password":null,"key_path":"/home/you/.ssh/id_ed25519","key_passphrase":null,"sudo_password":null,"su_password":null,"timeout_ms":60000,"max_command_chars":1000,"max_output_chars":100000,"disable_sudo":false,"schema_version":3,"added_at":"2026-10-19T22:16:38.513594758+00:00"}
```

### Explicação
- O arquivo de chave precisa existir no registro, um caminho ausente sai com 66 e `file not found: private key not found`
- Um host só com chave imprime `password` como `null`, o que diz ao agente que não há senha para mascarar
- Nenhum `secrets.key` é criado porque nenhum segredo é guardado

### Variantes
- No modo texto `ssh-cli vps list --output-format text` imprime `(not set)` na coluna PASSWORD de um host sem senha e três asteriscos de um host que tem senha
- As colunas do texto se ajustam à célula mais larga, então um nome de usuário longo nunca desloca a coluna PASSWORD
- Sonde logo após registrar com `ssh-cli vps add ... --check`
- Agrupe hosts com `--tag prod --tag edge` e alcance-os depois com `--tags`

### Veja também
- A lista completa de flags de `ssh-cli vps add` em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como executar um comando remoto com JSON

### Problema
- Você quer uma resposta interpretável de um comando remoto, não prosa de terminal

### Solução

```bash
ssh-cli exec prod "echo hi" --json
```

```json
{"target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false,"stdout":"hi\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":112}
```

### Explicação
- Uma chamada sem `--step` imprime exatamente um objeto JSON (G8), nunca vários eventos
- `exit_code` é o exit code remoto, e quando o comando remoto falha a CLI sai com 1, mantém este objeto no stdout e imprime um envelope `command_failed` com `remote_exit_code` no stderr
- O stdout e o stderr remotos ficam crus por desenho, o escape do modo texto não os toca

### Variantes
- Acrescente passos na mesma sessão com `ssh-cli exec prod "echo one" --step "echo two" --json`, que imprime um objeto por passo com `step` e `command`
- Anote o histórico remoto com `--description "bootstrap"`

### Veja também
- Fan-out de frota abaixo e [AGENTS.pt-BR.md](AGENTS.pt-BR.md)


## Como designar o host alvo e provar onde rodou

### Problema
- Um comando que herda o host de um marcador em disco pode cair no servidor errado (GAP-SSH-EXEC-ARGC-001)

### Solução

```bash
ssh-cli exec prod "systemctl restart nginx" --json
ssh-cli exec --use-active "uptime" --json
```

- Sem marcador, `--use-active` sai com 66 e `no_active_vps`

```json
{"exit_code":66,"error_code":"no_active_vps","message":"no active vps; run 'ssh-cli connect <NAME>' first","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

- Depois de `ssh-cli connect prod`, o marcador só é alcançável por `--use-active` e o objeto diz isso

```json
{"target_resolved":"prod","target_source":"active_marker","host_resolved":"prod","host_source":"active_marker","active_fallback":true,"stdout":"hi\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":110}
```

```bash
out=$(ssh-cli exec prod "systemctl restart nginx" --json); ec=$?
echo "$out" | jaq -r '"\(.target_resolved) via \(.target_source)"'
echo "$out" | jaq -e '.target_source == "argv"' >/dev/null || echo "ambient target, aborting"
```

- Um posicional solto sai com 64 porque o nome do host viraria o comando

```bash
ssh-cli exec prod --step "systemctl restart nginx" --json
```

```json
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: expected exactly two positionals `<VPS> <COMMAND>`, got 1; designate the target explicitly: `<VPS> <COMMAND>`, or a selector with one positional (`--all`/`--hosts`/`--tags` `<COMMAND>`), or the active marker deliberately (`--use-active <COMMAND>`)","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

### Explicação
- `target_resolved` e `target_source` são os campos canônicos e `host_resolved` e `host_source` são seus aliases de leitura da 0.5.5
- `target_source` é `argv`, `active_marker` ou `selector`, então uma comparação de string prova onde a mutação caiu
- A divisão é estável na CLI: 64 significa nenhum alvo designado e 66 significa que o alvo designado não existe

### Variantes
- Pergunte qual host um comando ambiente atingiria com `ssh-cli doctor --json | jaq -r '.local.active_vps // "none"'`
- Escolha vários hosts de forma explícita com `--all`, `--hosts a,b` ou `--tags prod`

### Veja também
- As regras de seletores em [AGENTS.pt-BR.md](AGENTS.pt-BR.md)


## Como executar sudo seguro com comandos compostos

### Problema
- Um comando composto com `&&` ou aspas precisa chegar intacto ao shell remoto

### Solução

```bash
ssh-cli sudo-exec prod "echo one && echo 'two three'" --description "compound check" --json
```

```json
{"target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false,"stdout":"one\ntwo three\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":156}
```

- Uma chamada bloqueada imprime esta recusa com exit 77

```bash
ssh-cli --disable-sudo sudo-exec prod id --json
```

```json
{"exit_code":77,"error_code":"sudo_disabled","message":"sudo/su disabled for this host (disable_sudo)","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

### Explicação
- O comando é empacotado com um `sh -c` seguro, então metacaracteres ficam dentro do shell remoto
- A senha do sudo vem do registro do host, defina-a com `--sudo-password-stdin`, ou com `--sudo-password VALUE` e o aviso no stderr sobre o argv
- Prefira `ssh-cli sudo-exec` a um `sudo` cru dentro do `ssh-cli exec`, para o empacotamento e a política valerem

### Variantes
- Instale um pacote do mesmo jeito com `ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --description "bootstrap curl" --json`
- Rode em uma frota com `ssh-cli sudo-exec --tags prod 'systemctl restart nginx' --json`
- Bloqueie a elevação por host com `ssh-cli vps edit prod --disable-sudo` e libere com `--enable-sudo`
- Remova a senha do sudo guardada com `ssh-cli vps edit prod --clear-sudo-password --json`, `updated` é `["sudo_password"]`

### Veja também
- Desativar a elevação abaixo e [AGENTS.pt-BR.md](AGENTS.pt-BR.md)


## Como elevar com su quando o sudo não está disponível

### Problema
- A conta não tem regra de sudo mas você conhece a senha do `su`

### Solução

```bash
printf '%s' 'demo-password-not-real' | ssh-cli vps edit prod --su-password-stdin --json
ssh-cli su-exec prod "whoami" --json
```

```json
{"ok":true,"event":"vps-edited","host_resolved":"prod","host_source":"argv","name":"prod","target_resolved":"prod","target_source":"argv","updated":["su_password"]}
```

- Em caso de sucesso o `ssh-cli su-exec` imprime o mesmo objeto do `ssh-cli exec`, e antes de a senha ser guardada a chamada sai com 64 com este envelope

```json
{"exit_code":64,"error_code":"su_password_missing","message":"su_password not configured; use vps edit --su-password or --su-password-stdin","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

### Explicação
- A senha é selada em repouso como todo outro segredo
- `updated` nomeia `su_password`, então o envelope prova o que mudou
- Remova-a depois com `--clear-su-password`

### Variantes
- Defina-a direto com `--su-password` e aceite o aviso no stderr sobre o argv
- Rode em um subconjunto com `ssh-cli su-exec --hosts a,b 'whoami' --json`

### Veja também
- Sudo seguro acima e [AGENTS.pt-BR.md](AGENTS.pt-BR.md)


## Como rejeitar cedo comandos grandes demais de agentes

### Problema
- Um agente pode gerar um comando muito maior do que você quer rodar

### Solução

```bash
ssh-cli vps edit prod --max-command-chars 20 --json
ssh-cli exec prod "echo aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" --json
```

```json
{"ok":true,"event":"vps-edited","host_resolved":"prod","host_source":"argv","name":"prod","target_resolved":"prod","target_source":"argv","updated":["max_command_chars"]}
{"exit_code":64,"error_code":"command_too_long","message":"command exceeds max_command_chars (20): 35 characters","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

### Explicação
- O limite é checado antes de qualquer trabalho SSH, então um comando rejeitado não custa conexão
- A contagem é o tamanho da string do comando, aqui 35 caracteres
- O exit 64 é permanente, encurte ou divida o comando em vez de repetir

### Variantes
- Restaure o padrão com `ssh-cli vps edit prod --max-command-chars 1000`
- Defina-o no registro com `ssh-cli vps add --max-command-chars 200`

### Veja também
- Limitar a saída abaixo e [AGENTS.pt-BR.md](AGENTS.pt-BR.md)


## Como limitar a saída para o contexto de um LLM

### Problema
- Um comando ruidoso como `dmesg` pode inundar a janela de contexto de um agente

### Solução

```bash
ssh-cli vps edit prod --max-output-chars 5 --json
ssh-cli exec prod "echo 123456789" --json
```

```json
{"ok":true,"event":"vps-edited","host_resolved":"prod","host_source":"argv","name":"prod","target_resolved":"prod","target_source":"argv","updated":["max_output_chars"]}
{"target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false,"stdout":"12345","stderr":"","exit_code":0,"truncated_stdout":true,"truncated_stderr":false,"duration_ms":98}
```

### Explicação
- O corte acontece no limite do registro do host e `truncated_stdout` avisa que ele aconteceu
- Um valor realista é 20000, o padrão é 100000
- Corte o envelope também com o global `--truncate-content 200`

### Variantes
- Restaure o padrão com `ssh-cli vps edit prod --max-output-chars 100000`
- Limite a resposta inteira de uma frota com `ssh-cli exec --all 'dmesg' --json --max-output-bytes 65536`
- O alias legado `--max-chars` grava o mesmo campo de `--max-command-chars`, então `ssh-cli vps edit prod --max-chars 200` limita o tamanho do comando, não o da saída
- Corte campos de texto com `--truncate-content`, e cale a linha humana com `--quiet` ou `--no-color` para sobrar só o payload
- Suba o nível de diagnóstico com `--verbose` (`-v` info, `-vv` debug, `-vvv` trace) quando o payload não explicar um limite
- Anexe um nome TLS a um host com `ssh-cli vps edit web1 --tls-sni web1.internal`, e o handshake usa o nome do certificado em vez do endereço discado

### Veja também
- Reduzir o payload de um agente abaixo e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como sondar conectividade com health-check

### Problema
- Você precisa saber que um host responde antes de depender dele

### Solução

```bash
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 15000 --json
ssh-cli health-check down1 --json
```

```json
{"name":"prod","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","status":"ok","latency_ms":39}
{"name":"prod","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","status":"ok","latency_ms":41}
{"exit_code":74,"error_code":"connection_failed","message":"ssh connection failed: TCP dial failed for 127.0.0.1:22999: Connection refused (os error 111)","error_class":"transient","retryable":true,"suggestion":"retry at most twice with exponential full-jitter backoff (exit 74)"}
```

### Explicação
- `--timeout` é em milissegundos, e o `health-check` aceita um valor abaixo de 1000 sem aviso, só `ssh-cli vps add --timeout` avisa
- Um host morto sai com 74 com `retryable: true`, repita no máximo duas vezes com backoff
- `ssh-cli health-check` aceita as mesmas substituições de autenticação do `ssh-cli exec`

### Variantes
- Sobrescreva a autenticação com `printf '%s' "$PASS" | ssh-cli health-check prod --json --password-stdin`
- Use uma chave com `ssh-cli health-check prod --json --key ~/.ssh/id_ed25519` e `--key-passphrase-stdin` quando ela é cifrada

### Veja também
- Sondas de frota abaixo e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como rodar trabalho de frota em todos os hosts registrados

### Problema
- Criar um processo da CLI por host desperdiça tempo de relógio em idas e voltas SSH

### Solução

```bash
ssh-cli --max-concurrency 8 health-check --all --json
ssh-cli exec --hosts web1,web2 "uptime" --json
ssh-cli exec --tags prod,edge "uptime" --json
ssh-cli scp upload --all ./app.tgz /tmp/app.tgz --json
ssh-cli vps doctor --probe-ssh --json
```

- Saída esperada do `ssh-cli health-check --all`, o envelope de lote traz `max_concurrency` e uma entrada de `results` por host

```json
{"event":"health-check-batch","batch_run_id":"00000000-0000-7000-8000-000000000000","max_concurrency":8,"results":[{"name":"web1","status":"ok","latency_ms":38},{"name":"web2","status":"ok","latency_ms":41}]}
```

- Saída esperada do `ssh-cli exec --tags`, só aparecem os hosts que carregam uma tag listada

```json
{"event":"exec-batch","batch_run_id":"00000000-0000-7000-8000-000000000000","max_concurrency":64,"results":[{"name":"web2","ok":true,"exit_code":0,"stdout":"hi\n","stderr":"","duration_ms":147}]}
```

- Com um host fora do ar, o lote ainda chega ao stdout, depois a chamada sai com 1 e imprime um envelope `partial_failure` de classe `partial` no stderr

```json
{"event":"health-check-batch","batch_run_id":"00000000-0000-7000-8000-000000000000","max_concurrency":64,"results":[{"name":"down1","status":"error","latency_ms":0,"error":"ssh connection failed: TCP dial failed for 127.0.0.1:22999: Connection refused (os error 111)"},{"name":"web1","status":"ok","latency_ms":37}]}
{"exit_code":1,"error_code":"partial_failure","message":"1/2 hosts failed during health-check","error_class":"partial","retryable":false}
```

### Explicação
- `--all`, `--hosts` e `--tags` são mutuamente exclusivos e o clap rejeita qualquer par com exit 2
- `--tags` funciona só em `ssh-cli exec`, `ssh-cli sudo-exec` e `ssh-cli su-exec`, as tags vêm de `ssh-cli vps add --tag prod --tag edge`
- Um registro vazio com `--all` sai com 64, um nome desconhecido em `--hosts` sai com 64 com `unknown host(s) for --hosts`
- Um host com falha nunca aborta os irmãos e a chamada sai com 1 com um envelope `partial_failure` no stderr
- `ssh-cli tunnel` continua de um host só, vários hosts significam várias invocações

### Variantes
- Baixe de todos os hosts com `ssh-cli scp download --all /var/log/app.log ./app.log --json`, o caminho local vira prefixo e o arquivo cai como `./app.log.<vps>`
- Envie vários arquivos com `ssh-cli scp upload --all --src f1 --src f2 --dest /tmp/dir --json`, `--dest` é sempre um diretório que já precisa existir no lado remoto, e os resultados se chamam `host:file`
- Pare de admitir hosts novos depois da primeira falha com o global `--fail-fast`, as unidades já em execução terminam
- Um host que nunca começou continua em `results` com `ok` false e `not attempted (fail-fast stopped admission)` no `ssh-cli exec`, medido com `--max-concurrency 1`
- Os lotes de `ssh-cli scp` e `ssh-cli sftp` escrevem o mesmo estado como `not attempted (fan-out admission stopped)` no código-fonte, caso que este livro não reproduziu

### Veja também
- Schemas de lote em [docs/schemas/README.md](schemas/README.md)


## Como manter o stderr do agente limpo e diagnosticar sem vazar

### Problema
- Prosa informativa no stderr confunde leitores automáticos e logs verbosos podem vazar segredos

### Solução

```bash
ssh-cli exec lab "true" --json
ssh-cli -v exec lab "true" --json
ssh-cli -vv exec lab "true" --json
ssh-cli -vvv exec lab "true" --json
```

- O nível padrão é error, então a primeira chamada imprime só o objeto JSON no stdout e nada no stderr

```json
{"target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","active_fallback":false,"stdout":"","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":90}
```

- Com `-v` o stdout continua igual e o stderr ganha estas linhas

```text
2026-10-20T01:14:56.025174Z  INFO main ssh_cli::ssh::client_connect: starting SSH connection host=203.0.113.10 port=22 username=ubuntu timeout_ms=60000 has_key=true use_agent=false
2026-10-20T01:14:56.062337Z  INFO main ssh_cli::ssh::client_connect: SSH authentication succeeded auth_method="publickey"
2026-10-20T01:14:56.062357Z  INFO main ssh_cli::ssh::client_connect: SSH connection authenticated successfully
2026-10-20T01:14:56.153055Z  INFO main ssh_cli::ssh::client::real: SSH session closed
```

### Explicação
- Os filtros são restritos à crate (`warn,ssh_cli=…`), então uma execução de debug nunca liga os dumps de senha do `russh` (G2/G14)
- O `RUST_LOG` do ambiente é ignorado, só `-v`, `-vv` e `-vvv` mudam o nível
- Uma rejeição de chave de host também imprime uma linha de log no stderr antes do envelope JSON

### Variantes
- Silencie a saída humana de sucesso com `-q`
- Recuse esperar por um humano com `ssh-cli vps list --json --no-input`, toda flag `--*-stdin` então sai com 64 com `--no-input forbids reading secrets from stdin; pass the value via flag or drop --no-input`

### Veja também
- Verbosidade em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como diagnosticar caminhos XDG e o modo de segredos

### Problema
- Você precisa saber qual configuração vence e como os segredos são guardados

### Solução

```bash
ssh-cli doctor --json | jaq -c '.local | {config_path,exists,hosts,permissions,schema_version,secrets_at_rest,secrets_key_source,telemetry,active_vps}'
ssh-cli vps path --json
ssh-cli secrets status --json
```

```json
{"config_path":"CONFIG_DIR/config.toml","exists":true,"hosts":1,"permissions":"600","schema_version":3,"secrets_at_rest":"plaintext","secrets_key_source":"none","telemetry":false,"active_vps":"prod"}
{"ok":true,"event":"vps-path","path":"CONFIG_DIR/config.toml"}
{"at_rest":"plaintext","encryption_active":false,"key_file":"CONFIG_DIR/secrets.key","key_source":"none","plaintext_opt_out":false}
```

### Explicação
- `ssh-cli doctor --json` imprime um documento com o evento `vps-doctor`, o objeto `local` e `ssh_probe` igual a `null` a menos que você passe `--probe-ssh`
- `permissions` descreve só o `config.toml`, o diretório de configuração tem campos próprios
- O modo texto imprime os mesmos fatos, com as linhas `Config dir mode:`, `Config dir exposed:` e `Invalid host names:`

### Variantes
- Acrescente a sonda SSH com `ssh-cli vps doctor --probe-ssh --hosts web1 --json`
- `known_hosts.lock` e `lang` são criados 0600 sem janela antes do chmod
- Imprima só o caminho do arquivo de configuração com `ssh-cli vps path`, o evento é `vps-path`
- Isole um laboratório com `--config-dir /tmp/lab/config.toml`, o pai `/tmp/lab` vira o diretório de configuração de todo arquivo vizinho
- Leia os campos do diretório de configuração nas duas receitas sobre o diretório de configuração abaixo
- Leia os fatos do ambiente de execução com `ssh-cli doctor --json --select local.runtime`, os campos são `local.runtime.os`, `arch`, `is_ci`, `is_container`, `is_termux`, `is_wsl` e `sandbox`

### Veja também
- Configuração em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como isolar um diretório de configuração de laboratório

### Problema
- Uma execução de teste nunca pode tocar o registro, as chaves nem o idioma da sua configuração real

### Solução

```bash
printf '%s' 'demo-password-not-real' | ssh-cli --config-dir /tmp/lab/config.toml vps add --name a --host h.example.com --user deploy --password-stdin --json
ssh-cli --config-dir /tmp/lab/config.toml locale set pt-BR --json
ssh-cli --config-dir /tmp/lab/config.toml vps path --json
ls /tmp/lab
```

```text
{"ok":true,"event":"vps-added","key_file":"/tmp/lab/secrets.key","key_source":"xdg_file","name":"a","secrets_key_auto_created":true}
{"applies":"next_invocation_unless_overridden","lang":"pt-BR","ok":true,"path":"/tmp/lab/lang"}
{"ok":true,"event":"vps-path","path":"/tmp/lab/config.toml"}
config.toml
config.toml.lock
lang
secrets.key
```

### Explicação
- Um caminho que termina em `.toml` faz do pai o diretório de configuração de todo arquivo, então nenhum diretório chamado `config.toml` é criado
- Os níveis novos nascem 0700 e `known_hosts.lock` e `lang` nascem 0600
- O diretório real não é lido nem gravado, porque `--config-dir` o substitui
- O arquivo `lang` fica dentro do laboratório, então `ssh-cli locale set` nunca muda o seu idioma

### Variantes
- Passe um diretório simples como `--config-dir /tmp/lab` para o mesmo resultado, o arquivo é `config.toml` dentro dele
- `known_hosts`, `active` e a árvore `tls/` também ficam no diretório do laboratório quando um comando os usa
- Apague o laboratório com `rm -r /tmp/lab` quando terminar

### Veja também
- Diagnosticar caminhos XDG acima e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como recifrar um inventário legado em texto puro

### Problema
- Um inventário gravado antes de a primary-key existir ainda guarda senhas em texto puro

### Solução

```bash
ssh-cli secrets init --json
ssh-cli secrets reencrypt --json
```

```json
{"ok":true,"event":"secrets-init","force":false,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","reencrypted_hosts":0}
{"ok":true,"event":"secrets-reencrypt","hosts":1}
```

### Explicação
- `ssh-cli secrets init` sem `--force` só cria a chave e não recifra nada (`reencrypted_hosts` é 0)
- `ssh-cli secrets reencrypt` reescreve o `config.toml` para as senhas virarem blobs `sshcli-enc:`
- Ambos aceitam `--dry-run` e imprimem o plano sem gravar

### Variantes
- Pré-visualize com `ssh-cli secrets reencrypt --dry-run --json`, o plano nomeia `hosts` e `operation`
- Recifre com um arquivo de chave que você escolheu com `ssh-cli --secrets-key-file /path/to/key secrets reencrypt`

### Veja também
- A receita de inicialização acima, a receita de rotação de chave abaixo e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como rotacionar a primary-key sem perder segredos

### Problema
- Trocar a primary-key sem recifrar as senhas guardadas faz perdê-las em repouso

### Solução

```bash
ssh-cli secrets init --force --dry-run --json
ssh-cli secrets init --force --json
ssh-cli secrets status --json
```

```json
{"ok":true,"event":"dry-run","config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"force":true,"hosts_to_reencrypt":1,"keyring":false,"operation":"secrets-init"}
{"ok":true,"event":"secrets-init","force":true,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","reencrypted_hosts":1}
{"at_rest":"encrypted","encryption_active":true,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","plaintext_opt_out":false}
```

### Explicação
- Leia `hosts_to_reencrypt` no plano primeiro, porque é o número de hosts cujos segredos a rotação precisa recifrar
- A execução real imprime `reencrypted_hosts` com a contagem que recifrou, compare com o plano
- Sem `--force` uma chave existente faz o `ssh-cli secrets init` sair com 64 e não mudar nada
- O plano não grava nada, o arquivo de chave e o `config.toml` ficam como estavam

### Variantes
- Rotacione para o keyring do SO com `ssh-cli secrets init --force --keyring --json`, o plano imprime `keyring: true`
- Rotacione sob um arquivo de chave que você escolheu com `ssh-cli --secrets-key-file /path/to/key secrets init --force --json`

### Veja também
- A receita de recifragem acima e [AGENTS.pt-BR.md](AGENTS.pt-BR.md)


## Como exportar e importar o inventário sem segredos

### Problema
- Você quer copiar um registro para outra máquina sem carregar credenciais

### Solução

```bash
ssh-cli vps export -o /tmp/hosts.redacted.json --json
ssh-cli --output-format text vps export -o /tmp/hosts.redacted.toml
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.redacted.toml --allow-incomplete --dry-run --json
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.redacted.toml --allow-incomplete --json
```

```json
{"ok":true,"event":"vps-export","format":"json","include_secrets":false,"path":"/tmp/hosts.redacted.json"}
{"ok":true,"event":"dry-run","config_path":"/tmp/ssh-cli-copy/config.toml","dry_run":true,"executed":false,"hosts":[{"incomplete":false,"name":"demo","replaces_existing":false}],"imported":1,"operation":"vps-import","source":"/tmp/hosts.redacted.toml"}
{"ok":true,"event":"vps-import","imported":1,"replaced":[]}
```

### Explicação
- O corpo segue o formato resolvido: JSON em qualquer stdout não TTY, mesmo para um nome de arquivo `.toml`, e TOML somente com `--output-format text`
- Um export redigido limpa todo segredo, `password` vira `""` e os segredos opcionais viram `null`, e um falso `sshcli-enc:` vazio nunca é gravado
- Com `-o` a chamada imprime um envelope `vps-export` pequeno que nomeia o arquivo, sem `-o` ela imprime o envelope inteiro com os hosts no stdout
- `replaced` lista os nomes ordenados que o import sobrescreveu e fica vazio no primeiro import
- Um host de senha parece um export redigido, então o import sai com 64 a menos que você passe `--allow-incomplete` e depois o complete com `ssh-cli vps edit`

### Variantes
- O import aceita TOML com chaves em inglês ou apelidos legados em português, ou um envelope JSON `vps-export`, e a chave do usuário é `username` no TOML (legado `usuario`) e `user` no JSON
- Um host só com chave ou de agente faz a ida e volta completa, `use_agent` e `agent_socket` viajam no export
- Um `--file` ausente, com ou sem `--dry-run`, sai com 66 com `error_code` `file_not_found`, `error_class` `permanent` e `retryable` false, então não repita

### Veja também
- Nomes inválidos no import abaixo e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como pré-visualizar um import que sobrescreveria hosts

### Problema
- Um import que reaproveita um nome de host sobrescreve aquele registro, e a contagem de hosts importados não diz isso

### Solução

```bash
ssh-cli vps export -o hosts.json --json
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file hosts.json --json
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file hosts.json --dry-run --json
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file hosts.json --json
```

- Saída esperada das quatro chamadas, em ordem, com o registro de `/tmp/ssh-cli-copy` vazio no começo

```json
{"ok":true,"event":"vps-export","format":"json","include_secrets":false,"path":"hosts.json"}
{"ok":true,"event":"vps-import","imported":1,"replaced":[]}
{"ok":true,"event":"dry-run","config_path":"/tmp/ssh-cli-copy/config.toml","dry_run":true,"executed":false,"hosts":[{"incomplete":false,"name":"edge","replaces_existing":true}],"imported":1,"operation":"vps-import","source":"hosts.json"}
{"ok":true,"event":"vps-import","imported":1,"replaced":["edge"]}
```

### Explicação
- `replaces_existing` é true quando o nome já está no registro de destino, esse campo e não a contagem `imported` decide se o import é seguro
- `replaced` na execução real lista os nomes ordenados que foram sobrescritos, e fica vazio no primeiro import
- `incomplete` é true para um host que parece um export redigido, o import sai com 64 para ele a menos que você passe `--allow-incomplete`
- O plano roda a mesma validação da execução real, então um nome inválido falha do mesmo jeito

### Variantes
- Importe antes para um `--config-dir` descartável quando estiver em dúvida, o registro que importa nunca muda
- Guarde uma cópia do registro de destino antes de sobrescrever com `ssh-cli vps export -o backup.json --json` na máquina de destino

### Veja também
- A receita de export e import acima e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como exportar com segredos (protegido)

### Problema
- Um backup que mantém credenciais nunca pode chegar a um pipe por acidente

### Solução

```bash
ssh-cli vps export --include-secrets -o /tmp/hosts.secrets.json --json
ssh-cli vps export --include-secrets | cat
```

```json
{"ok":true,"event":"vps-export","format":"json","include_secrets":true,"path":"/tmp/hosts.secrets.json"}
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: refusing --include-secrets to a pipe/non-TTY stdout; use `--output <file>` (mode 0o600) or pass `--i-understand-secrets-on-stdout`","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

### Explicação
- O arquivo de saída é criado com modo 0600
- O pipe é recusado com exit 64 a menos que você o reconheça
- Diretórios pais ausentes de `-o new/deep/e.json` são criados com modo 0700 e os existentes nunca são alterados

### Variantes
- Reconheça o stdout só quando precisar mesmo dele com `--i-understand-secrets-on-stdout`
- Prefira um export redigido mais um `--password-stdin` novo na máquina de destino

### Veja também
- A receita redigida acima e [AGENTS.pt-BR.md](AGENTS.pt-BR.md)


## Como abrir um tunnel com prazo

### Problema
- Um forward sem prazo é um daemon e esta CLI não entrega um

### Solução

```bash
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 3000 --json
```

```json
{"ok":true,"event":"tunnel_listening","vps":"prod","local_port":18080,"remote_host":"127.0.0.1","remote_port":8080,"timeout_ms":3000,"bind":"127.0.0.1","mode":"local"}
{"ok":true,"event":"tunnel_closed","vps":"prod","reason":"deadline","bind":"127.0.0.1","local_port":18080,"forwards_served":0,"capacity_waits":0,"duration_ms":3001,"mode":"local"}
```

### Explicação
- Espere o objeto `tunnel_listening` antes de usar a porta local (`docs/schemas/tunnel-listening.schema.json`)
- `tunnel_closed` nomeia o `reason`, `forwards_served` e `capacity_waits`, e o fim por `deadline` sai com 0
- Um SIGINT sai com 130 e um SIGTERM sai com 143, os dois com `reason` igual a `signal`
- Um timeout antes do bind ainda sai com 74
- O processo fica vivo até o prazo ou um sinal

### Variantes
- Faça bind em outro endereço com `--bind 0.0.0.0 --i-accept-network-exposure`, um bind roteável é recusado sem o reconhecimento (G-TUN-R13) e a chamada sai com 64 com `--bind 0.0.0.0 exposes the forwarded service to the network; pass --i-accept-network-exposure to proceed`
- Autentique com `--password-stdin`, `--key`, `--key-passphrase` ou `--key-passphrase-stdin` como no `ssh-cli exec`

### Veja também
- Segurança do bind em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como alcançar muitos destinos com um só handshake (--socks5)

### Problema
- N túneis separados pagam N handshakes SSH para alcançar N hosts atrás de um bastião

### Solução

```bash
ssh-cli tunnel prod 1080 --socks5 --timeout-ms 300000 --json
curl --socks5-hostname 127.0.0.1:1080 http://10.0.0.5:8080/health
```

```json
{"ok":true,"event":"tunnel_listening","vps":"prod","local_port":1080,"remote_host":"*","remote_port":0,"timeout_ms":300000,"bind":"127.0.0.1","mode":"socks5"}
```

### Explicação
- Uma sessão SSH atende todo destino, escolhido por conexão (G-TUN-R02)
- `REMOTE_HOST` e `REMOTE_PORT` são omitidos porque o CONNECT do SOCKS5 carrega o alvo
- O proxy fala a RFC 1928 sem autenticação, então mantenha-o no loopback

### Variantes
- Aponte qualquer cliente com suporte a SOCKS5 para `127.0.0.1:1080`
- Combine com `--max-concurrency` para limitar forwards simultâneos

### Veja também
- Modos de tunnel em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como encaminhar para um socket Unix remoto (--remote-socket)

### Problema
- Docker, PostgreSQL e systemd escutam em um socket Unix, não em TCP

### Solução

```bash
ssh-cli tunnel prod 2375 --remote-socket /var/run/docker.sock --timeout-ms 60000 --json
docker -H tcp://127.0.0.1:2375 ps
```

```json
{"ok":true,"event":"tunnel_listening","vps":"prod","local_port":2375,"remote_host":"/var/run/docker.sock","remote_port":0,"timeout_ms":60000,"bind":"127.0.0.1","mode":"streamlocal"}
```

### Explicação
- A chamada abre um canal `direct-streamlocal@openssh.com` (G-TUN-R03)
- O caminho do socket precisa ser absoluto, um caminho relativo sai com 64 antes de qualquer trabalho SSH
- A existência local nunca é checada porque o socket vive no servidor
- Trate a porta local como delegação de privilégio, ela herda a autoridade do socket

### Variantes
- O PostgreSQL usa a mesma forma com `ssh-cli tunnel db 15432 --remote-socket /var/run/postgresql/.s.PGSQL.5432 --timeout-ms 60000 --json`
- Um cliente Windows funciona porque localmente só fala TCP, o servidor precisa suportar a extensão

### Veja também
- Modos de tunnel em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como deixar o servidor chamar de volta (--reverse)

### Problema
- Um webhook de retorno ou um depurador remoto precisa alcançar uma porta da sua máquina

### Solução

```bash
ssh-cli tunnel prod 8080 127.0.0.1 9000 --reverse --timeout-ms 120000 --json
```

```json
{"ok":true,"event":"tunnel_listening","vps":"prod","local_port":9000,"remote_host":"127.0.0.1","remote_port":8080,"timeout_ms":120000,"bind":"127.0.0.1","mode":"reverse"}
```

- Peça ao servidor para alocar a porta com `0` como último posicional, a porta alocada volta em `local_port` e varia a cada execução

```bash
ssh-cli tunnel prod 8080 127.0.0.1 0 --reverse --timeout-ms 120000 --json
```

```json
{"ok":true,"event":"tunnel_listening","vps":"prod","local_port":36423,"remote_host":"127.0.0.1","remote_port":8080,"timeout_ms":120000,"bind":"127.0.0.1","mode":"reverse"}
```

- Expor o lado do servidor exige o reconhecimento, porque essa é a ponta exposta

```bash
ssh-cli tunnel prod 8080 0.0.0.0 9000 --reverse --i-accept-network-exposure --timeout-ms 120000 --json
```

### Explicação
- O servidor escuta e entrega cada conexão na sua porta local (G-TUN-R01)
- Sob `--reverse` o evento é lido do ponto de vista do servidor: `bind` e `local_port` são onde o servidor escuta, `remote_host` e `remote_port` são o seu destino local de entrega
- `AllowTcpForwarding` e `GatewayPorts` do servidor ainda decidem se o bind é permitido
- `--bind` é aceita e depois ignorada sob `--reverse`, defina o endereço do servidor por `<remote_host>`

### Variantes
- Um forward local não aceita `REMOTE_PORT` 0, não haveria nada para conectar, enquanto `LOCAL_PORT` 0 pede ao sistema uma porta livre
- Um bind do lado do servidor fora do loopback sem o reconhecimento sai com 64 com uma mensagem que pede `--i-accept-network-exposure`

### Veja também
- Modos de tunnel em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como reduzir o payload de um agente antes de ele ser escrito

### Problema
- Passar um envelope grande por uma ferramenta externa paga o custo de tokens antes de ele encolher

### Solução

```bash
ssh-cli health-check --all --json --select name,status --filter status!=ok
ssh-cli vps list --json --select name,host --sort name --limit 2
ssh-cli vps list --json --count-only
ssh-cli vps list --json --filter nameprod
```

```text
{"agent_shape":{"dropped":2,"input_count":3,"output_count":1},"batch_run_id":"00000000-0000-7000-8000-000000000000","event":"health-check-batch","max_concurrency":64,"results":[{"name":"down1","status":"error"}]}
[{"host":"127.0.0.1","name":"demo"},{"host":"127.0.0.1","name":"demo2"}]
{"count":3}
Invalid argument: invalid --filter `nameprod`: expected key=value, key!=value or key~substring
```

- A primeira chamada sai com 1 porque um host falhou, as duas seguintes saem com 0 e a última sai com 64

### Explicação
- As oito flags de redução rodam antes da serialização, então o envelope grande nunca é construído
- O campo de resultado do lote é `status` com `ok` ou `error`, não existe campo `ok`, então `--filter ok=false` descartaria todos os registros
- Um predicado malformado é rejeitado na análise com exit 64, então um erro de digitação nunca parece ausência de resultados
- A redução alcança coleções de registros (`ssh-cli vps list`, `results`, `entries` do `ssh-cli sftp ls`) e também um objeto único como `ssh-cli vps show`, `ssh-cli doctor` ou um `ssh-cli exec` de um host
- Num objeto único só `--select` e `--truncate-content` reduzem, e `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` e `--max-output-bytes` são nomeadas em `agent_shape.ignored`
- `name~~prod` é lido como chave `name` com substring `~prod`, a recusa cobre operadores ausentes e chaves vazias

### Variantes
- Deduplique com `--dedupe-by host` e encurte strings com `--truncate-content 200`
- Escolha campos da resposta de uma frota com `ssh-cli exec --all 'echo a' --json --select name,stdout`, já que `--select results` devolveria objetos vazios
- `--fields` é apelido de `--select`, então `ssh-cli vps show prod --json --fields name,host --count-only` imprime `{"agent_shape":{"ignored":["--count-only"],"selected":["name","host"]},"host":"prod.example.com","name":"prod"}`
- O array puro de `ssh-cli vps list` não tem espaço para `agent_shape`, então `ssh-cli vps list --json --select name --limit 1` escreve `{"dropped":1,"event":"agent-shape","input_count":2,"output_count":1}` no stderr, e o `--quiet` suprime essa linha
- Um erro de uso do clap (exit 2) e um `--filter` malformado (exit 64) imprimem texto puro no stderr mesmo com `--json`, só erros de execução trazem o envelope JSON, e o texto puro segue o idioma da interface (`Argumento inválido:` em pt-BR)
- Limite o envelope com `--max-output-bytes 65536`, ele descarta registros finais e nunca corta o texto JSON
- Uma chave vazia como `--filter =prod` sai com 64 com `invalid --filter: empty key`, enquanto `--filter name~~prod` é válido, sai com 0 e aqui não casa nada
- Um leitor que fecha o pipe antes de uma saída grande terminar, como `ssh-cli completions bash | head -c 5`, faz a CLI sair com 141 com a saída incompleta, leia em `${PIPESTATUS[0]}` e nunca repita
- Um JSON pequeno como `ssh-cli vps list --json | head -c 5` termina de escrever antes de o leitor fechar e sai com 0, então não prova nada sobre o 141
- O exit 141 vale tanto para stdout fechado quanto para stderr fechado

### Veja também
- Redução de payload em [AGENTS.pt-BR.md](AGENTS.pt-BR.md)


## Como trocar um host de senha para chave e de volta

### Problema
- Mover um host entre métodos de autenticação saía com 64 e forçava um desvio pelo agente

### Solução

```bash
printf '%s' 'demo-password-not-real' | ssh-cli vps add --name web1 --host 203.0.113.10 --user deploy --password-stdin --json
ssh-cli vps edit web1 --key ~/.ssh/id_ed25519 --json
printf '%s' 'demo-password-not-real' | ssh-cli vps edit web1 --password-stdin --json
```

```json
{"ok":true,"event":"vps-added","key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","name":"web1","secrets_key_auto_created":true}
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":["key_path","password"]}
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":["password","key_path"]}
```

- Flags conflitantes são pegas pelo clap antes de qualquer trabalho, exit 2, e a primeira linha do erro em texto puro é mostrada

```bash
ssh-cli vps edit web1 --key ~/.ssh/id_ed25519 --password-stdin --json
```

```text
error: the argument '--key <PATH>' cannot be used with '--password-stdin'
```

### Explicação
- `--key` limpa `password`, `use_agent` e `agent_socket`, `--password*` limpa `key_path`, `key_passphrase`, `use_agent` e `agent_socket`
- `--use-agent` limpa `password`, `key_path` e `key_passphrase`, e `--agent-socket` sozinho implica `--use-agent`
- `updated` lista o método novo primeiro e depois só os campos que tinham valor
- Uma `--key` diferente descarta a passphrase antiga, o mesmo caminho de chave não muda nada
- Uma passphrase sem nenhuma chave sai com 64 e deixa o registro intacto

### Variantes
- Mude para o agente com `ssh-cli vps edit web1 --use-agent --json`, `updated` é `["use_agent","password"]` em um host de senha
- Acrescente uma passphrase a uma chave com `--key ~/.ssh/id_ed25519 --key-passphrase-stdin`

### Veja também
- Troca de autenticação em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como renomear um host sem digitar a credencial de novo

### Problema
- O desvio export, edição, import redigia segredos e descartava a autenticação de agente

### Solução

```bash
ssh-cli connect old-name
ssh-cli vps rename old-name new-name --json
ssh-cli vps edit new-name --name newer-name --json
```

```json
{"ok":true,"event":"vps-connected","name":"old-name"}
{"ok":true,"event":"vps-renamed","active_rewritten":true,"host_resolved":"old-name","host_source":"argv","name":"new-name","previous_name":"old-name","target_resolved":"old-name","target_source":"argv","updated":[]}
{"ok":true,"event":"vps-renamed","active_rewritten":true,"host_resolved":"new-name","host_source":"argv","name":"newer-name","previous_name":"new-name","target_resolved":"new-name","target_source":"argv","updated":[]}
```

### Explicação
- A credencial nunca é pedida, o registro sela os segredos sem dados associados e o nome novo não muda nada neles
- `ssh-cli vps edit --name` e `ssh-cli vps rename` emitem o mesmo evento `vps-renamed` com `previous_name`
- `active_rewritten` é `true` só quando o marcador `active` apontava para o nome antigo, e ele passa a seguir o novo
- Repetir um rename sai com 66 porque o nome antigo não existe mais, então confira `ssh-cli vps list` em vez de repetir

### Variantes
- Um destino já usado sai com 64 com `vps_duplicate` e um nome idêntico sai com 64 com `new name ... is identical to the current name`
- Um nome novo inválido também sai com 64
- `ssh-cli vps rename ghost ghost` sai com 66 porque o host precisa existir antes de os nomes serem comparados

### Veja também
- Renomear em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como editar um host campo a campo e limpá-lo

### Problema
- Todo campo opcional podia ser sobrescrito mas nenhum podia ser removido

### Solução

```bash
ssh-cli vps edit web1 --user deploy --port 2222 --tag prod --tag eu --json
ssh-cli vps edit web1 --clear-tags --tag staging --json
ssh-cli vps edit web1 --port 2222 --json
ssh-cli vps edit web1 --json
```

```json
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":["port","tags"]}
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":["tags"]}
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":[]}
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: nothing to edit for 'web1': pass at least one field, --name, --tag or a clear flag","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

- O host começou com `user` já `deploy`, então `updated` lista só `port` e `tags` na primeira chamada

### Explicação
- As oito flags de limpeza são `--clear-tags`, `--no-key`, `--no-agent-socket`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert` e `--no-tls-client-key`
- `--clear-tags` limpa primeiro e `--tag` depois acrescenta, então o resultado é `[staging]`
- `--no-key` também limpa a passphrase, e limpar o único método de autenticação restante sai com 64 com `error_code` `domain_validation` e mensagem com prefixo `vps_auth:`, e não grava nada
- Uma edição que não muda nada não faz nada: exit 0, `updated: []` e o `config.toml` não é reescrito
- Cada flag de limpeza conflita com a flag de valor do seu campo, `--no-key --key` sai com 2

### Variantes
- Remova as duas metades do mTLS com `--no-tls-client-cert --no-tls-client-key`
- Alterne o sudo com `--disable-sudo` e `--enable-sudo`, `updated` é `["disable_sudo"]`
- `--no-agent-socket` em um host de agente com socket explícito sai com 64 onde a plataforma não tem endpoint padrão, passe `--agent-socket PATH` ou troque de método

### Veja também
- A tabela de flags em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como reparar um nome de host legado

### Problema
- Um `config.toml` antigo guarda um nome com espaço, acento ou caractere de controle

### Solução

```bash
mkdir -p legacy && chmod 700 legacy
cat > legacy/config.toml <<'EOF'
schema_version = 3

[hosts."legacy host"]
name = "legacy host"
host = "legacy.example.com"
port = 22
username = "deploy"
use_agent = true
EOF
ssh-cli --config-dir legacy doctor --json | jaq -c '.local.invalid_host_names'
ssh-cli --config-dir legacy vps rename 'legacy host' legacy-host --json
ssh-cli --config-dir legacy doctor --json | jaq -c '.local.invalid_host_names'
```

```json
["legacy host"]
{"ok":true,"event":"vps-renamed","active_rewritten":false,"host_resolved":"legacy host","host_source":"argv","name":"legacy-host","previous_name":"legacy host","target_resolved":"legacy host","target_source":"argv","updated":[]}
[]
```

### Explicação
- A CLI nunca trava no próprio arquivo, o nome legado carrega, lista, mostra e resolve
- O `ssh-cli doctor` lista os nomes fora do conjunto de caracteres estrito, escapados, em `local.invalid_host_names`
- `ssh-cli vps rename` aceita a origem legada porque só o nome novo é validado
- Um nome com caractere de controle é recusado como alvo por `ssh-cli exec`, `ssh-cli health-check`, `--hosts` e `ssh-cli connect` com exit 64
- Um host legado exporta bem mas só restaura depois de `ssh-cli vps rename` dar a ele um nome estrito

### Variantes
- Um nome com um byte ESC é impresso como `a\u{1b}[2Jb` no modo texto e em `invalid_host_names`
- Duas tabelas que normalizam para um nome saem com 65 com `config has two hosts named ...`

### Veja também
- Reparar um nome de host legado em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como ler nomes hostis com segurança no modo texto

### Problema
- Um nome de host guardado com um byte ESC pode redesenhar o seu terminal quando uma ferramenta o imprime cru

### Solução

```bash
mkdir -p hostile && chmod 700 hostile
cat > hostile/config.toml <<'EOF'
schema_version = 3

[hosts."a\u001b[2Jb"]
name = "a\u001b[2Jb"
host = "h.example.com"
port = 22
username = "deploy"
use_agent = true
EOF
ssh-cli --lang en --config-dir hostile --output-format text vps show "$(printf 'a\033[2Jb')" | sed -n 1p
ssh-cli --lang en --config-dir hostile --output-format text vps show "$(printf 'a\033[2Jz')"
ssh-cli --config-dir hostile exec "$(printf 'a\033[2Jb')" id --json
ssh-cli --config-dir hostile vps list --json | jaq -c '.[].name'
```

- A primeira linha abaixo é o stdout, e a segunda é o stderr da chamada que sai com 66

```text
Name:           a\u{1b}[2Jb
VPS 'a\u{1b}[2Jz' not found.
```

- A terceira chamada é recusada com exit 64 e a quarta imprime o escape do JSON

```json
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: vps_name: name must not contain control or invisible formatting characters","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
"a\u001b[2Jb"
```

### Explicação
- O modo texto escreve caracteres de controle, de formatação e bidi como `\u{1b}` nas mensagens de sucesso, erro e aviso, em `list` e `show`, e nas saídas em lote
- O JSON mantém o valor original, porque é para máquina e o escape do `serde_json` já esconde o byte ESC
- Uma busca que viraria alvo recusa caracteres de controle e bidi com exit 64, `ssh-cli exec`, `ssh-cli health-check`, `--hosts` e `ssh-cli connect` inclusive, e o `ssh-cli connect` não grava marcador
- O erro de um nome que não existe repete o nome digitado já escapado

### Variantes
- Nomes remotos recebem o mesmo cuidado, `ssh-cli sftp ls` e `ssh-cli sftp stat` também escapam `\n` e `\t` no modo texto
- O stdout e o stderr remotos do `ssh-cli exec` ficam crus por desenho, filtre-os antes de imprimir em um terminal

### Veja também
- Reparar um nome de host legado acima e [AGENTS.pt-BR.md](AGENTS.pt-BR.md)


## Como conferir que o diretório de configuração não está exposto

### Problema
- Um diretório de configuração com modo 0755 mostra nomes de host e tamanhos de arquivo a outros usuários

### Solução

```bash
ssh-cli doctor --json --select local.config_dir_exposed
mkdir shared && chmod 755 shared
ssh-cli --config-dir shared doctor --json --select local.config_dir_permissions,local.config_dir_exposed
chmod 700 shared
ssh-cli --config-dir shared doctor --json --select local.config_dir_permissions,local.config_dir_exposed
```

```json
{"agent_shape":{"selected":["local.config_dir_exposed"]},"local":{"config_dir_exposed":false}}
{"agent_shape":{"selected":["local.config_dir_permissions","local.config_dir_exposed"]},"local":{"config_dir_exposed":true,"config_dir_permissions":"0755"}}
{"agent_shape":{"selected":["local.config_dir_permissions","local.config_dir_exposed"]},"local":{"config_dir_exposed":false,"config_dir_permissions":"0700"}}
```

### Explicação
- Um diretório que a CLI cria nasce 0700 em cada nível novo
- Um diretório XDG padrão existente é apertado para 0700 quando não é link simbólico e é seu
- Qualquer outro diretório existente nunca recebe chmod, o `ssh-cli doctor` reporta a exposição e um aviso de log aparece com `-v`
- O `--select` alcança o objeto do `ssh-cli doctor` e nomeia os caminhos mantidos em `agent_shape.selected`, enquanto `--filter` e `--count-only` não têm como valer para um registro só e são nomeados em `agent_shape.ignored`
- A correção é sua: rode `chmod 700` no diretório e a mesma chamada imprime `config_dir_exposed` igual a `false`

### Variantes
- `config_dir_permissions` fica ausente fora do Unix e `config_dir_exposed` lê `false` lá
- No Windows o diretório depende da ACL do perfil porque não existe chmod

### Veja também
- Configuração em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como ver todos os nomes inválidos em um só import

### Problema
- Um arquivo com vários nomes de host ruins deveria falhar uma vez, com a lista toda

### Solução

```bash
cat > inv.toml <<'EOF'
schema_version = 3

[hosts."bad name"]
name = "bad name"
host = "h.example.com"
port = 22
username = "u"
use_agent = true

[hosts."semi;colon"]
name = "semi;colon"
host = "h.example.com"
port = 22
username = "u"
use_agent = true
EOF
ssh-cli vps import --file inv.toml --json
ssh-cli vps import --file inv.toml --dry-run --json
```

```json
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: invalid VPS names in import (2): 'bad name', 'semi;colon'; a name is 1..=255 ASCII letters, digits, '.', '_' or '-' and starts with a letter or digit; nothing was imported","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: invalid VPS names in import (2): 'bad name', 'semi;colon'; a name is 1..=255 ASCII letters, digits, '.', '_' or '-' and starts with a letter or digit; nothing was imported","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

### Explicação
- O import valida todos os itens primeiro e não grava nada quando um nome falha
- `--dry-run` dá o mesmo erro, então o ensaio é fiel
- Corrija os nomes no arquivo e rode de novo, repetir sem mudar não ajuda

### Variantes
- Use letras ASCII, dígitos, ponto, hífen e sublinhado, começando por letra ou dígito, até 255 bytes
- `ssh-cli vps add --name 'bad name'` e o destino de `ssh-cli vps rename` seguem a mesma regra estrita e saem com 64

### Veja também
- Export e import em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como importar uma identidade mTLS com um nome válido

### Problema
- O nome de uma identidade vira nome de diretório, então um nome hostil não pode criar um

### Solução

```bash
openssl req -x509 -newkey ed25519 -nodes -keyout client-key.pem -out client.pem -subj /CN=demo -days 2
ssh-cli tls mtls import --name edge --cert ./client.pem --key ./client-key.pem --json
ssh-cli tls mtls import --name 'serviço-1' --cert ./client.pem --key ./client-key.pem --json
ssh-cli tls mtls list --json
ssh-cli tls mtls show edge --json
```

```json
{"ok":true,"event":"tls-mtls-import","cert_path":"CONFIG_DIR/tls/mtls/edge/cert.pem","key_path":"CONFIG_DIR/tls/mtls/edge/key.pem","name":"edge"}
{"ok":true,"event":"tls-mtls-import","cert_path":"CONFIG_DIR/tls/mtls/serviço-1/cert.pem","key_path":"CONFIG_DIR/tls/mtls/serviço-1/key.pem","name":"serviço-1"}
{"ok":true,"event":"tls-mtls-list","identities":["edge","serviço-1"]}
{"ok":true,"event":"tls-mtls-show","cert_path":"CONFIG_DIR/tls/mtls/edge/cert.pem","key_path":"CONFIG_DIR/tls/mtls/edge/key.pem","name":"edge"}
```

- Um nome com caractere de inversão bidi como U+202E sai com 64 e não cria diretório

```json
{"exit_code":64,"error_code":"invalid_argument","message":"invalid argument: invalid mTLS identity name: invalid argument: file name must not contain control or invisible formatting characters","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

### Explicação
- `ssh-cli tls mtls import` recusa caracteres de controle, formatação e bidi, Unicode comum continua aceito
- `ssh-cli tls mtls list` mostra os nomes de identidade guardados
- Chaves e certificados ficam em `tls/mtls/<name>/` no diretório de configuração com modos privados
- `show` nunca imprime material de chave privada

### Variantes
- Remova uma identidade com `ssh-cli tls mtls remove edge --json`, ele imprime `{"ok":true,"event":"tls-mtls-remove","name":"edge"}`
- Anexe-a a um host com `ssh-cli vps edit web1 --tls --tls-client-cert CERT --tls-client-key KEY`
- Desligue de novo o SSH sobre TLS com `ssh-cli vps edit web1 --no-tls --json`, `updated` é `["tls"]` e o SNI e os caminhos mTLS guardados ficam até você passar as flags de limpeza deles

### Veja também
- TLS em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como ensaiar uma operação destrutiva e ler o eco do alvo

### Problema
- Você quer ver qual host uma chamada destrutiva tocaria antes de ela tocar em qualquer coisa

### Solução

```bash
ssh-cli vps remove web1 --dry-run --json
ssh-cli vps rename web1 web-1 --dry-run --json
ssh-cli vps edit web1 --name web-1 --dry-run --json
```

```json
{"ok":true,"event":"dry-run","config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"host_resolved":"web1","host_source":"argv","name":"web1","operation":"vps-remove","target_resolved":"web1","target_source":"argv"}
{"ok":true,"event":"dry-run","active_rewritten":false,"config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"host_resolved":"web1","host_source":"argv","name":"web-1","operation":"vps-rename","previous_name":"web1","target_resolved":"web1","target_source":"argv","updated":[]}
{"ok":true,"event":"dry-run","active_rewritten":false,"config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"host_resolved":"web1","host_source":"argv","name":"web-1","operation":"vps-rename","previous_name":"web1","target_resolved":"web1","target_source":"argv","updated":[]}
```

- Contra um registro que não existe, os três verbos saem com 66 e não criam diretório nem arquivo de lock

```json
{"exit_code":66,"error_code":"vps_not_found","message":"vps 'ghost' not found in registry","error_class":"permanent","retryable":false,"suggestion":"the host is not in the registry; if an earlier `vps rename` or `vps remove` may have succeeded, run `vps list` before repeating"}
```

### Explicação
- `--dry-run` é aceita só por `ssh-cli vps remove`, `ssh-cli vps import`, `ssh-cli vps edit`, `ssh-cli vps rename`, `ssh-cli sftp rm`, `ssh-cli sftp rmdir`, `ssh-cli secrets init` e `ssh-cli secrets reencrypt`
- Em qualquer outro lugar ela sai com 64, então uma flag que não faz nada nunca passa por ensaio
- O plano roda toda validação da execução real e traz `target_resolved`, `target_source` (`argv`), `host_resolved` e `host_source`
- `active_rewritten` diz se o marcador `active` se moveria, sem movê-lo
- Uma busca que falha não traz alvo, porque nenhum host chegou a ser escolhido

### Variantes
- As operações remotas imprimem `vps` e `remote` no lugar, por exemplo `ssh-cli sftp rm prod /srv/app/stale.log --dry-run --json`
- Uma chamada recusada é como `ssh-cli exec prod 'rm -rf /tmp/x' --dry-run --json` e sai com 64

### Veja também
- O schema do dry-run em [docs/schemas/README.md](schemas/README.md)


## Como transferir um artefato de release com SCP

### Problema
- Você precisa de um arquivo regular no host, com prova do que foi movido

### Solução

```bash
ssh-cli scp upload prod ./payload.bin /tmp/payload.bin --timeout 120000 --json
ssh-cli scp download prod /tmp/payload.bin ./payload.local --json
ssh-cli scp download prod /tmp/missing.bin ./missing --json
```

```json
{"ok":true,"event":"scp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"upload","vps":"prod","local":"./payload.bin","remote":"/tmp/payload.bin","bytes":8,"duration_ms":111,"mtime_preserved":true,"durable":true}
{"ok":true,"event":"scp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"download","vps":"prod","local":"./payload.local","remote":"/tmp/payload.bin","bytes":8,"duration_ms":209,"mtime_preserved":true,"durable":true}
{"exit_code":66,"error_code":"file_not_found","message":"file not found: /tmp/missing.bin","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

### Explicação
- O SCP move só arquivos regulares, use `sftp --recursive` para árvores
- Um download grava `<path>.ssh-cli.partial` e o renomeia de forma atômica, então uma falha deixa o caminho final intacto
- Preservar mtime e modo é melhor esforço e `mtime_preserved` e `durable` informam o resultado real
- Falhas com `--json` imprimem o envelope de erro no stderr

### Variantes
- Envie vários arquivos para um diretório remoto que já existe com `ssh-cli scp upload prod f1 f2 /tmp/dir --json`, uma sessão e transferências seriais, e o evento é `scp-batch` com uma entrada de `results` por arquivo
- Confira o resultado com `ssh-cli exec prod "ls -l /tmp/dir" --json`
- Abra canais paralelos na mesma sessão com o global `--scp-file-concurrency 2` antes do subcomando, o padrão é 1

### Veja também
- Rotina diária do operador em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como enviar por SFTP e verificar o checksum

### Problema
- Uma contagem de bytes informada pelo cliente não prova que o destino tem os bytes certos

### Solução

```bash
ssh-cli sftp upload prod ./payload.bin /tmp/payload.bin --json
ssh-cli exec prod "sha256sum /tmp/payload.bin" --json | jaq -r .stdout
sha256sum ./payload.bin
```

```json
{"ok":true,"event":"sftp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"upload","vps":"prod","local":"./payload.bin","remote":"/tmp/payload.bin","bytes":8,"duration_ms":1}
d4e4877bac978b7952f0d544fc52ebff5411d351d129f1f056fa43f11da9af2b  /tmp/payload.bin
d4e4877bac978b7952f0d544fc52ebff5411d351d129f1f056fa43f11da9af2b  ./payload.bin
```

- Os dois digests são idênticos, a segunda linha do `sha256sum` é o arquivo local

### Explicação
- Os digests iguais são o critério de aceite, efeito no destino em vez de afirmações do cliente (G15)
- A G1 corrigiu o truncamento no upload na 0.5.3, então use a 0.5.3 ou mais nova
- Downloads gravam um arquivo parcial e o renomeiam, e falhas de `set_permissions` local são expostas (G18)

### Variantes
- Mova uma árvore com `ssh-cli sftp upload --recursive prod ./dist/tree /opt/app/tree --json`, links simbólicos não são seguidos
- Distribua para todos os hosts com `ssh-cli sftp upload --all ./payload.bin /tmp/payload.bin --json`

### Veja também
- SFTP em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como baixar um arquivo e uma árvore via SFTP

### Problema
- Você precisa de um arquivo remoto ou de uma árvore de diretórios inteira na sua máquina, com prova de que chegou

### Solução

```bash
ssh-cli sftp download prod /srv/app/f1 ./out/f1.local --json
ssh-cli sftp download --recursive prod /srv/app/tree ./out/tree --json
ssh-cli sftp download prod /srv/app/missing ./out/m --json
sha256sum ./out/f1.local
```

```json
{"ok":true,"event":"sftp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"download","vps":"prod","local":"./out/f1.local","remote":"/srv/app/f1","bytes":2,"duration_ms":1}
{"ok":true,"event":"sftp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"download","vps":"prod","local":"./out/tree","remote":"/srv/app/tree","bytes":2,"duration_ms":6,"recursive":true}
{"exit_code":66,"error_code":"file_not_found","message":"file not found: /srv/app/missing","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

- O digest local bate com o remoto, aqui os dois arquivos guardam os mesmos dois bytes

### Explicação
- A chamada da árvore imprime um documento `sftp-transfer` com `recursive: true` e `bytes`
- Uma árvore mantém a estrutura, `./out/tree/sub/f` aparece para um remoto `tree/sub/f`, e links simbólicos não são seguidos
- Um caminho remoto ausente sai com 66 com `file not found` e não deixa arquivo local para trás
- Um download único grava um arquivo parcial e o renomeia, então o caminho final nunca fica pela metade
- Os bits de permissão são mascarados com `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`), então um servidor não consegue te entregar um arquivo setuid (A3)

### Variantes
- Baixe um arquivo de todos os hosts com `ssh-cli sftp download --all /var/log/app.log ./app.log --json`, cada arquivo cai como `./app.log.<vps>` e o evento é `sftp-batch`
- Baixe várias origens com `--hosts web1,web2 --src f1 --src f2 --dest ./out`, os arquivos caem em `./out/<host>/`

### Veja também
- A receita de upload acima e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como verificar uma árvore SFTP recursiva

### Problema
- Uma transferência recursiva imprime um total de bytes só, então não mostra que todo arquivo chegou íntegro

### Solução

```bash
mkdir -p tree/sub && echo a > tree/a && echo b > tree/sub/f && ln -s a tree/link
ssh-cli sftp upload --recursive prod ./tree /srv/app/tree --json
(cd tree && sha256sum a sub/f)
ssh-cli exec prod "cd /srv/app/tree && sha256sum a sub/f" --json | jaq -r .stdout
ssh-cli exec prod "ls /srv/app/tree" --json | jaq -r .stdout
```

```text
{"ok":true,"event":"sftp-transfer","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","direction":"upload","vps":"prod","local":"./tree","remote":"/srv/app/tree","bytes":4,"duration_ms":2,"recursive":true}
87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7  a
0263829989b6fd954f72baaf2fc64bc2e2f01d692d4de72986ea808f6e99813f  sub/f
87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7  a
0263829989b6fd954f72baaf2fc64bc2e2f01d692d4de72986ea808f6e99813f  sub/f

a
sub
```

- Os dois pares de digests são idênticos, o primeiro par é a árvore local, e a listagem não tem `link`

### Explicação
- `bytes` é a soma dos bytes dos arquivos que a chamada moveu, 4 aqui para dois arquivos de 2 bytes, e não prova conteúdo nenhum
- Digests iguais por arquivo são o critério de aceite, efeito no destino em vez de afirmações do cliente
- O link simbólico não é seguido, então não chega ao lado remoto
- Uma chamada recursiva imprime um documento `sftp-transfer` com `recursive: true`

### Variantes
- Confira um download do mesmo jeito com `ssh-cli sftp download --recursive prod /srv/app/tree ./out/tree --json` e `(cd out/tree && sha256sum a sub/f)`
- Verifique todos os hosts em uma chamada com `ssh-cli exec --all "cd /srv/app/tree && sha256sum a sub/f" --json --select name,stdout`

### Veja também
- A receita de checksum acima e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como gerenciar caminhos remotos via SFTP

### Problema
- Você precisa listar, criar, renomear e remover caminhos remotos sem um shell

### Solução

```bash
ssh-cli sftp ls prod /tmp/rem --json
ssh-cli sftp stat prod /tmp/rem/payload.bin --json
ssh-cli sftp mkdir prod /tmp/rem/app --json
ssh-cli sftp rename prod /tmp/rem/app /tmp/rem/app.bak --json
ssh-cli sftp rmdir prod /tmp/rem/app.bak --dry-run --json
ssh-cli sftp rm prod /tmp/rem/payload.bin --json
```

```json
{"ok":true,"event":"sftp-list","vps":"prod","path":"/tmp/rem","entries":[{"name":"payload.bin","path":"/tmp/rem/payload.bin","kind":"file","size":8,"mode":33188}]}
{"ok":true,"event":"sftp-fs-op","op":"stat","vps":"prod","path":"/tmp/rem/payload.bin","duration_ms":0,"kind":"file","size":8,"mode":33188,"mtime":1792458836}
{"ok":true,"event":"sftp-fs-op","op":"mkdir","vps":"prod","path":"/tmp/rem/app","duration_ms":318}
{"ok":true,"event":"sftp-fs-op","op":"rename","vps":"prod","path":"/tmp/rem/app","to":"/tmp/rem/app.bak","duration_ms":311}
{"ok":true,"event":"dry-run","dry_run":true,"executed":false,"operation":"sftp-rmdir","remote":"/tmp/rem/app.bak","vps":"prod"}
{"ok":true,"event":"sftp-fs-op","op":"rm","vps":"prod","path":"/tmp/rem/payload.bin","duration_ms":210}
```

### Explicação
- `ssh-cli sftp ls` imprime o evento `sftp-list`, e `ssh-cli sftp mkdir`, `ssh-cli sftp rename`, `ssh-cli sftp rm` e `ssh-cli sftp rmdir` imprimem `sftp-fs-op` com o `op` que rodaram
- `ssh-cli sftp stat` imprime `sftp-fs-op` com `op` igual a `stat` e acrescenta `kind`, `size`, `mode` e `mtime`
- `ssh-cli sftp rm` e `ssh-cli sftp rmdir` aceitam `--dry-run`, os outros verbos não
- O modo texto escapa `\n` e `\t` em nomes remotos, então um nome forjado não adiciona linhas falsas
- Os bits de permissão do download são mascarados com `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) e o upload usa `SFTP_PERM_MASK` (`0o7777`)

### Variantes
- Um caminho ausente sai com 66 com `file not found`
- Schemas: `sftp-list`, `sftp-fs-op`, `sftp-transfer` e `sftp-batch`

### Veja também
- SFTP em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como gerar completions de shell

### Problema
- Você quer completar com Tab todo comando e flag

### Solução

```bash
mkdir -p ~/.local/share/bash-completion/completions
ssh-cli completions bash > ~/.local/share/bash-completion/completions/ssh-cli
ssh-cli completions zsh
ssh-cli completions fish
ssh-cli completions powershell
ssh-cli completions elvish
```

- A primeira linha do script bash, para uma conferência rápida

```text
_ssh-cli() {
```

### Explicação
- O script vai para o stdout e nada é instalado por você
- Os scripts são gerados da mesma árvore de comandos do `ssh-cli commands`

### Variantes
- No Windows redirecione `ssh-cli completions powershell` para o seu perfil do PowerShell
- Gere de novo a cada atualização para as flags novas completarem

### Veja também
- Descobrir contratos acima e [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como definir o idioma da interface

### Problema
- Você quer mensagens em um idioma, uma vez, ou para uma única chamada

### Solução

```bash
ssh-cli --lang en locale show --json
ssh-cli locale set pt-BR --json
ssh-cli locale clear --json
```

```json
{"available":["en","pt-BR"],"current":"en","direction":"ltr","persisted_raw":null,"preference_path":"CONFIG_DIR/lang","resolved":"en","script":"Latn","source":"cli_flag","system_raw":"pt-BR"}
{"applies":"next_invocation_unless_overridden","lang":"pt-BR","ok":true,"path":"CONFIG_DIR/lang"}
{"cleared":true,"ok":true}
```

### Explicação
- A preferência é o arquivo `lang` no diretório de configuração com modo 0600, nunca uma variável de ambiente
- `--lang` sobrescreve em uma chamada e não persiste nada, por isso `source` é `cli_flag` na primeira saída
- Os corpos de erro em JSON ficam em inglês para o agente nunca depender do idioma do host, decida por `error_code`

### Variantes
- Tags aceitas negociam para `en` ou `pt-BR`, por exemplo `en-US` ou `pt`, como em `ssh-cli --lang en vps list`
- `ssh-cli locale show` informa o idioma resolvido e `ssh-cli locale set` persiste uma preferência para as próximas chamadas
- `ssh-cli locale clear` remove a preferência guardada e repeti-lo é inofensivo
- A camada vencedora é `cli_flag`, depois a preferência guardada, depois o idioma do sistema

### Veja também
- Idioma em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como inspecionar provider TLS, caminhos e ACME

### Problema
- Você precisa saber a pilha TLS, onde o material fica e o estado de um pedido ACME

### Solução

```bash
ssh-cli tls provider --json
ssh-cli tls paths --json
ssh-cli tls acme account show --json
ssh-cli tls acme list --json
ssh-cli tls acme status --domain example.com --json
```

```json
{"ok":true,"event":"tls-provider","installed":true,"min_rustls":"0.23.18","provider":"aws_lc_rs","stack":"rustls+aws_lc_rs"}
{"ok":true,"event":"tls-paths","acme_account":"CONFIG_DIR/tls/acme/account.json","acme_dir":"CONFIG_DIR/tls/acme","mtls_dir":"CONFIG_DIR/tls/mtls","tls_root":"CONFIG_DIR/tls"}
{"ok":true,"event":"tls-acme-account-show","directory":null,"path":"CONFIG_DIR/tls/acme/account.json","present":false}
{"ok":true,"event":"tls-acme-list","domains":[]}
{"ok":true,"event":"tls-acme-status","domains":[{"cert_path":null,"domain":"example.com","has_cert":false,"has_key":false,"has_pending_order":false,"key_path":null}]}
```

- Antes de existir uma conta, `ssh-cli tls acme issue` e `ssh-cli tls acme complete` falham com exit 66 e estes envelopes

```bash
ssh-cli tls acme issue --domain example.com --print-challenge --json
ssh-cli tls acme complete --domain example.com --json
```

```json
{"exit_code":66,"error_code":"file_not_found","message":"file not found: ACME account missing at CONFIG_DIR/tls/acme/account.json; run `ssh-cli tls acme account create`","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
{"exit_code":66,"error_code":"file_not_found","message":"file not found: no pending ACME order for example.com; run `tls acme issue --domain example.com --print-challenge` first","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged"}
```

### Explicação
- `ssh-cli tls provider` informa a pilha: só rustls com `aws_lc_rs`, não há OpenSSL em tempo de execução
- Compilar do código-fonte exige um compilador C, porque o provider `aws_lc_rs` compila o `aws-lc-sys`
- `min_rustls` é o menor rustls que o binário aceita, e o `Cargo.toml` pede o rustls 0.23.25
- `ssh-cli tls acme account show` diz se existe uma conta, `ssh-cli tls acme list` lista os domínios e `ssh-cli tls acme status` mostra o estado do pedido e do certificado
- Os comandos ACME informam o domínio com `--domain`, um posicional solto como `ssh-cli tls acme issue example.com` sai com 2
- O material fica no diretório de configuração em `tls/`, e `ssh-cli tls paths` imprime cada raiz

### Variantes
- Crie a conta com `ssh-cli tls acme account create --contact mailto:ops@<YOUR_DOMAIN>`, acrescente `--staging` para ensaiar
- A chamada de criação contata o diretório ACME, um contato em domínio reservado como `example.com` sai com 64 com `invalidContact` e repetir não adianta
- Peça e conclua com `ssh-cli tls acme issue --domain example.com --print-challenge` e `ssh-cli tls acme complete --domain example.com`

### Veja também
- TLS em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como tratar a rotação de chave de host com segurança (TOFU)

### Problema
- Uma chave de host alterada pode ser uma rotação legítima ou uma interceptação

### Solução

```bash
ssh-cli exec prod "echo hi" --json
ssh-cli --replace-host-key exec prod "echo hi" --json
```

```json
{"exit_code":77,"error_code":"host_key_changed","message":"host key changed for prod.example.com:22: expected SHA256:STORED, got SHA256:OFFERED (use --replace-host-key if legitimate)","error_class":"permanent","retryable":false,"suggestion":"change credentials (--key / --password-stdin) or host-key policy; do not blind-retry","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
{"target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false,"stdout":"hi\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":92}
```

### Explicação
- A primeira conexão registra a chave em `known_hosts` e uma divergência depois sai com 77
- Confirme a rotação com um humano antes de passar `--replace-host-key`
- A flag troca a chave guardada naquela única chamada e as chamadas seguintes passam sem ela

### Variantes
- Mantenha `--replace-host-key` fora de scripts padrão, ele remove a proteção contra interceptação
- Não repita o exit 77 às cegas, mude as credenciais ou a política de chave de host antes

### Veja também
- Exit codes em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como desativar a elevação para automação não confiável

### Problema
- Um fluxo não confiável precisa ficar sem privilégio seja o que for que o registro do host guarda

### Solução

```bash
ssh-cli --disable-sudo exec prod "id -u" --json
ssh-cli --disable-sudo sudo-exec prod "id -u" --json
```

```json
{"target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false,"stdout":"1000\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":94}
{"exit_code":77,"error_code":"sudo_disabled","message":"sudo/su disabled for this host (disable_sudo)","error_class":"permanent","retryable":false,"suggestion":"fix CLI arguments, registry state, or schema; do not retry unchanged","target_resolved":"prod","target_source":"argv","host_resolved":"prod","host_source":"argv","active_fallback":false}
```

### Explicação
- O `ssh-cli exec` simples ainda roda porque não pede elevação
- `ssh-cli sudo-exec` e `ssh-cli su-exec` ficam bloqueados nesta invocação com exit 77
- A forma persistente é `ssh-cli vps edit prod --disable-sudo`, que guarda `disable_sudo: true`
- A `--disable-sudo` global antes de `ssh-cli vps add` ou `ssh-cli vps edit` também grava `disable_sudo: true`, então esse bloqueio sobrevive à chamada

### Variantes
- Libere o bloqueio persistente com `ssh-cli vps edit prod --enable-sudo`
- Registre um host já bloqueado com `ssh-cli vps add ... --disable-sudo`
- `ssh-cli --disable-sudo vps edit prod --json` imprime `updated` igual a `["disable_sudo"]`, e `--disableSudo` é um apelido aceito da flag global

### Veja também
- Automação de agente mais segura em [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md)


## Como rodar E2E de SSH real sem registrar segredos

### Problema
- Você quer prova contra um sshd real sem pôr um segredo em um log

### Solução

```bash
python3 scripts/e2e_real_ssh.py --local-sshd
python3 scripts/e2e_real_ssh.py --host 203.0.113.10 --port 22 --user deploy --password-file /path/to/pw-file
```

- Saída esperada do primeiro comando, um rótulo por caso e um resumo final

```text
PASS E01
PASS E02
PASS E03
PASS E04
PASS E05
PASS E06
PASS E07
PASS E08
PASS E09
PASS E10
PASS E11
PASS E12
PASS E13
PASS E14
PASS E15
PASS E16
PASS E17
PASS E18
PASS E19
PASS E20
PASS E21
---
fails=0 soft_sudo=0 tmp_destroyed=yes
```

### Explicação
- O script imprime só rótulos PASS, FAIL e SKIP, nunca um host, um usuário ou uma senha
- `--local-sshd` sobe um sshd sem privilégio no loopback, é só Unix
- Sem um host de laboratório o script sai com 0 com SKIP, seguro sem rede
- A matriz oficial é E01 a E21, E10 a E14 cobrem SCP, E17 e E18 cobrem checksum e árvores SFTP, E19 cobre o item reduzido de `vps list --select` contra o schema publicado, E20 cobre `--select` com `--limit` e a contabilização no stderr, e E21 cobre a rotação de um perfil vivo para o cofre com `--force`, a ida e volta em dois processos e a legibilidade do inventário que já existia

### Variantes
- No Windows rode `py -3 scripts/e2e_real_ssh.py ...`
- Aponte para outro binário com `--bin`, o padrão é `target/release/ssh-cli`
- Prefira um sshd local ou um VPS descartável e evite tempestades de falha de autenticação em produção por causa do fail2ban

### Veja também
- O catálogo de suítes em [TESTING.pt-BR.md](TESTING.pt-BR.md)
