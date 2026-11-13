# Como usar o ssh-cli


> Vá da instalação ao primeiro comando remoto.

- Leia este documento em [Inglês (en)](HOW_TO_USE.md)
- Volte ao [README.pt-BR.md](../README.pt-BR.md) para o mapa completo de comandos
- Linha de produto documentada aqui: 0.5.6
- Esta página cobre as 48 folhas de comando e as 89 flags distintas: as 27 flags globais e as 62 flags que existem só depois de um subcomando
- `-h, --help` e `-V, --version` ficam fora dessa conta, e `ssh-cli --version` imprime `ssh-cli 0.5.6` seguido do commit da compilação entre parênteses
- As saídas das seções "Saída esperada sem host" e "Comandos que exigem host" foram medidas no binário 0.5.6, os comandos offline contra um diretório de configuração descartável e os que exigem host contra um `sshd` local descartável
- A saída de sucesso do `su-exec` e os três comandos de rede do Automatic Certificate Management Environment (ACME) não foram executados, então listam campos de schema ou do código-fonte
- Nomes de host, endereços e `duration_ms` nos outros exemplos são ilustrativos, e a forma deles bate com as execuções medidas


## Pré-requisitos
- Instale o Rust 1.89 ou mais novo via rustup, a versão mínima de Rust suportada (MSRV)
- Tenha um compilador C na máquina de compilação, porque a crate `aws-lc-sys` por trás do provider `aws_lc_rs` de Transport Layer Security (TLS) compila C
- Garanta alcance de rede até o host SSH de destino
- Tenha uma senha ou uma chave privada OpenSSH para esse host
- Prefira um diretório de configuração XDG Base Directory (XDG) gravável para o armazenamento multi-host
- Rode em Linux, macOS ou Windows, as notas de plataforma estão em [CROSS_PLATFORM.pt-BR.md](CROSS_PLATFORM.pt-BR.md)
- Instale com `cargo install ssh-cli --locked` e use a 0.5.3 ou mais nova, esse é o piso de integridade de Secure Copy Protocol (SCP) e SSH File Transfer Protocol (SFTP) e esta página documenta a 0.5.6
- Não confie na 0.3.9 do crates.io para SCP, essa versão anunciava transferência mas o protocolo de fio estava quebrado
- Prefira a 0.5.3 ou mais nova para SFTP, versões anteriores podiam truncar arquivos remotos a zero byte no envio (G1)
- Verifique com `sha256sum` depois de toda transferência


## Novidades da 0.5.6
- O `russh` 0.65.1 fecha a RUSTSEC-2026-0492, a RUSTSEC-2026-0495 e dois advisories GHSA
- O `secrets init --keyring` agora grava no cofre do sistema operacional em vez de um mock no processo, então o `secrets status` seguinte responde `key_source: keyring` e `encryption_active: true`
- A entrada do cofre tem escopo do diretório de config resolvido, então um perfil criado com `--config-dir` nunca lê nem sobrescreve a chave do perfil real
- `--use-keyring` e `--keyring` chegam os dois ao caminho de gravação, e um cofre indisponível sai 69 em vez de reportar sucesso
- Um perfil tem exatamente uma chave primária, então o `secrets init` recusa com exit 64 quando já existe uma em qualquer um dos dois armazenamentos, e o `--force` aposenta a outra fonte para a rotação deixar uma chave só e o inventário legível
- A versão mínima de Rust é 1.89, que o russh 0.65 exige
- `--select` e `--truncate-content` agora reduzem um objeto único, então `ssh-cli doctor --select local.config_dir_exposed` devolve esse campo
- Um array nu reduzido escreve uma linha `{"event":"agent-shape",...}` em stderr com `input_count`, `output_count` e `dropped`, e o `--quiet` a suprime
- Uma flag de coleção passada em objeto único é nomeada em `agent_shape.ignored`
- Um item reduzido de `vps list` continua validando contra `ssh-cli schema vps-list`, que publica o item completo e o item reduzido
- `vps list` em modo texto dimensiona cada coluna pela célula mais larga, e `vps show` em modo texto alinha os seus 22 rótulos
- Toda folha mostra bloco `Examples:` no `--help`
- `vps rename <NAME> <NEW_NAME>` e `vps edit <NAME> --name <NEW>` renomeiam nativamente e toda credencial continua legível
- `vps edit` troca o método de autenticação em uma chamada com `--key`, `--password-stdin` ou `--use-agent`
- Oito flags de limpeza removem um campo em vez de sobrescrevê-lo
- `--dry-run` cobre oito operações e o plano de `vps edit`, `vps rename` e `vps remove` ecoa o alvo
- Nomes de host novos usam o charset estrito `[A-Za-z0-9._-]`, um `config.toml` antigo continua carregando
- `vps doctor` reporta `config_dir_exposed`, `config_dir_permissions` e `invalid_host_names` em `local`
- O modo texto escapa caracteres de controle e bidi como `\u{1b}`, o JSON não muda
- `--lock-wait-ms` limita o lock do config e um lock preso sai com 69 em vez de travar
- `commands --json` publica todo argumento com `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` e `hidden`


## Primeiro comando em 60 segundos

### Instalar, registrar, executar

```bash
cargo install ssh-cli --locked
ssh-cli vps add --name demo --host 203.0.113.10 --user ubuntu --key ~/.ssh/id_ed25519 --json
ssh-cli exec demo "echo hi" --json
```

- `vps add` imprime um documento e registra o host

```json
{"ok":true,"event":"vps-added","name":"demo","secrets_key_auto_created":false}
```

- `exec` imprime um objeto JSON, o `duration_ms` varia entre execuções

```json
{"target_resolved":"demo","target_source":"argv","host_resolved":"demo","host_source":"argv","active_fallback":false,"stdout":"hi\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":112}
```

- Confirme o exit code 0 e leia `stdout`, `stderr`, `exit_code` e `duration_ms`
- Leia `target_resolved`, `target_source` e `active_fallback` para saber qual host respondeu, `active_fallback` é true quando o host veio do marcador ativo
- `target_source` diz de onde o host veio: `argv`, `active_marker` (de `--use-active`) ou `selector` (de `--all`, `--hosts` ou `--tags`)
- `host_resolved` e `host_source` repetem os mesmos valores como aliases de leitura mantidos por compatibilidade, então ligue parsers novos a `target_resolved` e `target_source`
- Um sucesso sem `--step` é exatamente um objeto JSON no stdout (G8), `--step` imprime um objeto por passo
- Um comando remoto vazio falha com a mensagem técnica `empty command` (sempre em inglês) e exit 64
- Registre um host de senha com `--password-stdin` para o segredo nunca ir para o argv
- A primeira gravação de senha cria o `secrets.key` e imprime `secrets_key_auto_created: true` no mesmo documento `vps-added`, junto com `key_file` e `key_source`
- Saída esperada do primeiro host de senha, `CONFIG_DIR` representa o seu diretório de configuração

```bash
printf '%s' "$PASS" | ssh-cli vps add --name web1 --host 203.0.113.11 --user ubuntu --password-stdin --json
```

```json
{"ok":true,"event":"vps-added","key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","name":"web1","secrets_key_auto_created":true}
```

- Registre um host de agente com `vps add --use-agent`, acrescente `--agent-socket` para um socket próprio
- Rode `ssh-cli secrets status --json` e `ssh-cli doctor --json` quando caminhos ou modo de cifragem estiverem incertos
- Descubra contratos com `ssh-cli schema` e `ssh-cli commands`


## Comandos principais

### Inventário completo de comandos
- A tabela lista as 48 folhas com o formato do JSON que elas imprimem
- `objeto` significa um objeto simples sem campo `event`, `array` significa um array JSON de registros

| Comando | Finalidade | Imprime |
| --- | --- | --- |
| `ssh-cli vps add` | Registra um host com senha, chave ou agente | `vps-added` |
| `ssh-cli vps list` | Lista os hosts registrados com segredos mascarados | array |
| `ssh-cli vps remove` | Remove um host do registro | `vps-removed` |
| `ssh-cli vps rename` | Renomeia um host sem tocar nas credenciais | `vps-renamed` |
| `ssh-cli vps edit` | Altera, troca ou limpa campos do host | `vps-edited` |
| `ssh-cli vps show` | Mostra um host com segredos mascarados | objeto |
| `ssh-cli vps path` | Imprime o caminho resolvido da configuração | `vps-path` |
| `ssh-cli vps doctor` | Diagnostica caminhos, schema, modo de segredos e o diretório de configuração | `vps-doctor` |
| `ssh-cli vps export` | Exporta o inventário, redigido por padrão | `vps-export` |
| `ssh-cli vps import` | Importa um arquivo TOML ou um envelope JSON `vps-export` | `vps-import` |
| `ssh-cli connect` | Marca o host ativo | `vps-connected` |
| `ssh-cli exec` | Executa um comando remoto | objeto ou `exec-batch` |
| `ssh-cli sudo-exec` | Executa um comando remoto via `sudo` com empacotamento `sh -c` seguro | objeto ou `exec-batch` |
| `ssh-cli su-exec` | Executa um comando remoto via `su` quando a senha está guardada | objeto ou `exec-batch` |
| `ssh-cli scp upload` | Envia arquivos regulares, sem diretórios | `scp-transfer` |
| `ssh-cli scp download` | Baixa arquivos regulares com arquivo parcial e renomeação atômica | `scp-transfer` |
| `ssh-cli sftp upload` | Envia um arquivo ou uma árvore com `--recursive` | `sftp-transfer` |
| `ssh-cli sftp download` | Baixa um arquivo ou uma árvore com `--recursive` | `sftp-transfer` |
| `ssh-cli sftp ls` | Lista um diretório remoto | `sftp-list` |
| `ssh-cli sftp mkdir` | Cria um diretório remoto | `sftp-fs-op` |
| `ssh-cli sftp rmdir` | Remove um diretório remoto vazio | `sftp-fs-op` |
| `ssh-cli sftp rm` | Remove um arquivo remoto | `sftp-fs-op` |
| `ssh-cli sftp stat` | Consulta os metadados de um caminho remoto | `sftp-fs-op` |
| `ssh-cli sftp rename` | Renomeia ou move um caminho remoto | `sftp-fs-op` |
| `ssh-cli tunnel` | Encaminha porta com prazo obrigatório, quatro modos | `tunnel_listening` depois `tunnel_closed` |
| `ssh-cli health-check` | Sonda conectividade e latência | objeto ou `health-check-batch` |
| `ssh-cli secrets status` | Mostra o modo de cifragem sem a chave | objeto |
| `ssh-cli secrets init` | Cria a chave primária e nunca a imprime | `secrets-init` |
| `ssh-cli secrets reencrypt` | Recifra o inventário com a chave primária atual | `secrets-reencrypt` |
| `ssh-cli completions` | Imprime scripts de autocompletar do shell | texto de script |
| `ssh-cli commands` | Imprime a árvore de comandos com todos os argumentos | `commands` |
| `ssh-cli schema` | Lista o catálogo de schemas ou imprime um schema | `schema-catalog` ou corpo do schema |
| `ssh-cli doctor` | Alias raiz de `vps doctor` | `vps-doctor` |
| `ssh-cli locale show` | Mostra o idioma resolvido e a camada vencedora | objeto |
| `ssh-cli locale set` | Persiste a preferência de idioma no XDG | objeto |
| `ssh-cli locale clear` | Limpa a preferência de idioma guardada | objeto |
| `ssh-cli tls provider` | Mostra o status do `CryptoProvider` do rustls | `tls-provider` |
| `ssh-cli tls paths` | Mostra a estrutura TLS no XDG | `tls-paths` |
| `ssh-cli tls mtls list` | Lista as identidades de TLS mútuo (mTLS) importadas | `tls-mtls-list` |
| `ssh-cli tls mtls import` | Importa certificado e chave de cliente com um nome válido | `tls-mtls-import` |
| `ssh-cli tls mtls show` | Mostra uma identidade sem material de chave privada | `tls-mtls-show` |
| `ssh-cli tls mtls remove` | Remove uma identidade mTLS | `tls-mtls-remove` |
| `ssh-cli tls acme account create` | Cria uma conta ACME com `--contact` | `tls-acme-account-create` |
| `ssh-cli tls acme account show` | Mostra os metadados da conta ACME | `tls-acme-account-show` |
| `ssh-cli tls acme issue` | Inicia um pedido ACME com `--domain` e imprime o desafio | `tls-acme-challenge` |
| `ssh-cli tls acme complete` | Conclui o pedido ACME depois do desafio | `tls-acme-complete` |
| `ssh-cli tls acme status` | Mostra o status do pedido e do certificado | `tls-acme-status` |
| `ssh-cli tls acme list` | Lista os domínios ACME no XDG | `tls-acme-list` |

### Flags globais: 27 flags que vão antes do subcomando
- Escreva as flags globais antes do subcomando, como em `ssh-cli --config-dir DIR --json vps list`
- O binário também as aceita depois do subcomando, então exemplos antigos continuam funcionando
- O alias `--disableSudo` é aceito onde o global é aceito, antes ou depois do subcomando, e recusado por `vps add` e `vps edit`, que aceitam `--disable-sudo`
- `--timeout` e `--disable-sudo` também existem como flags locais e a local vence
- As flags de redução reduzem coleções de registros (o array de `vps list`, `results` dos envelopes em lote, `entries` do `sftp ls`)
- Em um objeto único como `vps show`, `exec` em um host, `commands` ou `doctor`, só `--select` e `--truncate-content` o reduzem, e o relatório fica em `agent_shape`
- As outras seis, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` e `--max-output-bytes`, deixam o objeto único inteiro e são nomeadas em `agent_shape.ignored`
- `--select` nomeia campos dentro de cada registro, então `--select results` devolve objetos vazios e a chamada certa é `--select name,stdout`

| Flag global | Efeito |
| --- | --- |
| `--lang LOCALE` | Idioma da interface só nesta chamada, `en` ou `pt-BR` |
| `-v, --verbose` | Nível de log `-v` info, `-vv` debug, `-vvv` trace |
| `-q, --quiet` | Silencia a saída humana de sucesso |
| `--config-dir DIR` | Isola o diretório de configuração, um caminho `.toml` usa o diretório pai |
| `--no-color` | Desliga as cores ANSI |
| `--output-format FORMAT` | Escolhe o formato de saída, JSON quando o stdout não é TTY |
| `--json` | Força JSON, alias de `--output-format json` |
| `--select PATHS` | Mantém só estes caminhos pontuados em cada registro, alias `--fields` |
| `--filter EXPR` | Mantém registros que casam com `key=value`, `key!=value` ou `key~substring` |
| `--limit N` | Emite no máximo N registros |
| `--sort PATH` | Ordena os registros de forma crescente por um caminho pontuado |
| `--dedupe-by PATH` | Descarta registros posteriores que repetem o valor de um caminho |
| `--count-only` | Troca os registros por uma contagem |
| `--truncate-content CHARS` | Encurta strings longas por caracteres |
| `--max-output-bytes BYTES` | Limita o envelope descartando registros finais |
| `--no-input` | Recusa ler o stdin e falha rápido, uma flag de stdin como `--password-stdin` sai então com 64 |
| `--dry-run` | Imprime o plano de uma operação destrutiva e para |
| `--lock-wait-ms MS` | Limita a espera pelo lock do config em milissegundos, padrão 5000, faixa de 1 a 300000, depois exit 69 |
| `--disable-sudo` | Bloqueia `sudo-exec` e `su-exec` nesta chamada, escrita antes de `vps add` ou `vps edit` ela fica gravada no registro e `--enable-sudo` desfaz |
| `--replace-host-key` | Troca uma chave de host divergente após revisão humana |
| `--allow-plaintext-secrets` | Só para testes: permite segredos em texto puro em repouso |
| `--secrets-key-file PATH` | Usa este arquivo de chave primária de 64 hex em uma chamada |
| `--use-keyring` | Prefere o keyring do SO para a chave primária, e o `secrets init` grava a chave nova nele |
| `--timeout MS` | Timeout SSH padrão em milissegundos, um `--timeout` local vence |
| `--max-concurrency N` | Limita a distribuição pela frota e os encaminhamentos de tunnel, de 1 a 64, automático quando omitida, qualquer outro valor sai com 2 |
| `--fail-fast` | Para de admitir hosts novos após a primeira falha |
| `--scp-file-concurrency N` | Arquivos SCP concorrentes em uma sessão |

### Flags locais: depois do subcomando
- Escreva as flags locais depois do subcomando, posicionais primeiro quando a linha de uso pedir
- `[X]` marca um posicional opcional e `<X>` um obrigatório
- Todo comando também aceita as 27 flags globais acima
- Uma flag marcada com `(obrigatória)` faz o comando sair com 2 quando falta, e os posicionais entre ângulos também são obrigatórios
- `--bind`, `--i-accept-network-exposure`, `--timeout-ms` e `--probe-ssh` parecem globais mas são locais, escritas antes do subcomando elas saem com 2
- `--max-chars` é o alias legado de `--max-command-chars`, prefira `--max-command-chars` e `--max-output-chars` em `vps add` e `vps edit`

| Comando | Posicionais e flags locais |
| --- | --- |
| `ssh-cli vps add` | `--name <NAME>` (obrigatória) `--host <HOST>` (obrigatória) `--port <PORT>` `--user <USER>` (obrigatória) `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` `--max-command-chars <N>` `--max-output-chars <N>` `--max-chars <N>` `--sudo-password <SUDO_PASSWORD>` `--sudo-password-stdin` `--su-password <SU_PASSWORD>` `--su-password-stdin` `--disable-sudo` `--tag <TAG>` `--tls` `--tls-sni <NAME>` `--tls-client-cert <PATH>` `--tls-client-key <PATH>` `--check` |
| `ssh-cli vps list` | `--tag <TAG>` |
| `ssh-cli vps remove` | `<NAME>` |
| `ssh-cli vps rename` | `<NAME>` `<NEW_NAME>` |
| `ssh-cli vps edit` | `<NAME>` `--name <NEW>` `--host <HOST>` `--port <PORT>` `--user <USER>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` `--max-command-chars <N>` `--max-output-chars <N>` `--max-chars <N>` `--sudo-password <SUDO_PASSWORD>` `--sudo-password-stdin` `--su-password <SU_PASSWORD>` `--su-password-stdin` `--disable-sudo` `--enable-sudo` `--tls` `--no-tls` `--tls-sni <NAME>` `--tls-client-cert <PATH>` `--tls-client-key <PATH>` `--tag <TAG>` `--clear-tags` `--no-key` `--no-agent-socket` `--clear-sudo-password` `--clear-su-password` `--no-tls-sni` `--no-tls-client-cert` `--no-tls-client-key` |
| `ssh-cli vps show` | `<NAME>` |
| `ssh-cli vps path` | nenhuma |
| `ssh-cli vps doctor` | `--probe-ssh` `--hosts <LIST>` |
| `ssh-cli vps export` | `--include-secrets` `--output <PATH>` `--i-understand-secrets-on-stdout` |
| `ssh-cli vps import` | `--file <PATH>` (obrigatória) `--allow-incomplete` |
| `ssh-cli connect` | `<NAME>` |
| `ssh-cli exec` | `<VPS> <COMMAND>` `--all` `--hosts <LIST>` `--tags <LIST>` `--use-active` `--step <CMD>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` `--description <DESCRIPTION>` |
| `ssh-cli sudo-exec` | `<VPS> <COMMAND>` `--all` `--hosts <LIST>` `--tags <LIST>` `--use-active` `--step <CMD>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--sudo-password <SUDO_PASSWORD>` `--sudo-password-stdin` `--timeout <MS>` `--description <DESCRIPTION>` |
| `ssh-cli su-exec` | `<VPS> <COMMAND>` `--all` `--hosts <LIST>` `--tags <LIST>` `--use-active` `--step <CMD>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--su-password <SU_PASSWORD>` `--su-password-stdin` `--timeout <MS>` `--description <DESCRIPTION>` |
| `ssh-cli scp upload` | `[VPS] [LOCAL] [REMOTE]...` `--all` `--hosts <LIST>` `--src <PATH>` `--dest <DIR>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli scp download` | `[VPS] [LOCAL] [REMOTE]...` `--all` `--hosts <LIST>` `--src <PATH>` `--dest <DIR>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp upload` | `[VPS] [LOCAL] [REMOTE]...` `--all` `--hosts <LIST>` `--src <PATH>` `--dest <DIR>` `--recursive` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp download` | `[VPS] [LOCAL] [REMOTE]...` `--all` `--hosts <LIST>` `--src <PATH>` `--dest <DIR>` `--recursive` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp ls` | `<VPS_NAME>` `<REMOTE>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp mkdir` | `<VPS_NAME>` `<REMOTE>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp rmdir` | `<VPS_NAME>` `<REMOTE>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp rm` | `<VPS_NAME>` `<REMOTE>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp stat` | `<VPS_NAME>` `<REMOTE>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli sftp rename` | `<VPS_NAME>` `<FROM>` `<TO>` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli tunnel` | `<VPS_NAME>` `<LOCAL_PORT>` `[REMOTE_HOST]` `[REMOTE_PORT]` `--socks5` `--remote-socket <PATH>` `--reverse` `--timeout-ms <MS>` (obrigatória) `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--bind <ADDR>` `--i-accept-network-exposure` |
| `ssh-cli health-check` | `[VPS_NAME]` `--all` `--hosts <LIST>` `--use-active` `--password <PASSWORD>` `--password-stdin` `--key <PATH>` `--key-passphrase <KEY_PASSPHRASE>` `--key-passphrase-stdin` `--use-agent` `--agent-socket <PATH>` `--timeout <MS>` |
| `ssh-cli secrets status` | nenhuma |
| `ssh-cli secrets init` | `--keyring` `--force` |
| `ssh-cli secrets reencrypt` | nenhuma |
| `ssh-cli completions` | `<SHELL>` |
| `ssh-cli commands` | nenhuma |
| `ssh-cli schema` | `[NAME]` |
| `ssh-cli doctor` | `--probe-ssh` `--hosts <LIST>` |
| `ssh-cli locale show` | nenhuma |
| `ssh-cli locale set` | `<LOCALE>` |
| `ssh-cli locale clear` | nenhuma |
| `ssh-cli tls provider` | nenhuma |
| `ssh-cli tls paths` | nenhuma |
| `ssh-cli tls mtls list` | nenhuma |
| `ssh-cli tls mtls import` | `--name <NAME>` (obrigatória) `--cert <PATH>` (obrigatória) `--key <PATH>` (obrigatória) |
| `ssh-cli tls mtls show` | `<NAME>` |
| `ssh-cli tls mtls remove` | `<NAME>` |
| `ssh-cli tls acme account create` | `--staging` `--contact <URL>` (obrigatória) `--force` |
| `ssh-cli tls acme account show` | nenhuma |
| `ssh-cli tls acme issue` | `--domain <DOMAIN>` (obrigatória) `--staging` `--print-challenge` |
| `ssh-cli tls acme complete` | `--domain <DOMAIN>` (obrigatória) |
| `ssh-cli tls acme status` | `--domain <DOMAIN>` |
| `ssh-cli tls acme list` | nenhuma |

### Rotina diária do operador
- Liste os hosts com `ssh-cli vps list --json`
- Mostre um host com `ssh-cli vps show demo --json`, os segredos continuam mascarados
- Altere campos com `ssh-cli vps edit demo --timeout 90000`
- Troque o método de autenticação com `ssh-cli vps edit demo --key ~/.ssh/id_ed25519`, `--password-stdin` ou `--use-agent`, cada uma limpa os outros métodos
- Marque o host ativo com `ssh-cli connect demo`, alcance-o somente com `--use-active`, como em `ssh-cli exec --use-active "uptime" --json`
- Rode trabalho privilegiado com `ssh-cli sudo-exec demo "systemctl status nginx" --json`
- Eleve com `ssh-cli su-exec` quando a senha de `su` está guardada no registro do host
- Transfira arquivos regulares com `ssh-cli scp upload demo ./app.tgz /tmp/app.tgz`, sem diretórios e sem `-r`
- Mova árvores de diretórios com `ssh-cli sftp upload --recursive demo ./dir /tmp/dir`
- Baixe com `ssh-cli scp download demo /var/log/app.log ./app.log`
- Interprete a transferência com `--json`, o evento é `scp-transfer` (`docs/schemas/scp-transfer.schema.json`)
- As flags do SCP espelham o `exec`: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` e `--key-passphrase-stdin`
- Um arquivo local ou remoto ausente sai com 66 e `file not found: <path>` sem prefixo `SCP:` empilhado
- Um download com falha deixa o caminho final intacto, grava `{path}.ssh-cli.partial` e renomeia de forma atômica
- O download SCP propaga uma falha de `sync_data` antes da renomeação (G9)
- O envio transmite em blocos de 32 KiB e nunca carrega o arquivo inteiro na RAM
- Preservar mtime e modo é melhor esforço, então leia `mtime_preserved` e `durable` no evento `scp-transfer` em vez de supor
- Gerencie a chave primária com `ssh-cli secrets status`, `ssh-cli secrets init` e `ssh-cli secrets reencrypt`, nenhum imprime a chave
- O keyring ainda pode aceitar o alias legado `secrets-master-key` na leitura
- Eventos de sucesso quando o JSON está efetivo: `vps-added`, `vps-edited` (ou `vps-renamed` quando `--name` renomeia), `vps-removed`, `vps-connected` e `vps-import`
- O catálogo de schemas está em [docs/schemas/README.md](schemas/README.md)

### Saída esperada sem host
- Estes comandos não precisam de servidor SSH, então toda saída abaixo foi medida no binário 0.5.6 com um `--config-dir` descartável
- `CONFIG_DIR` representa esse diretório
- As saídas são JSON porque o stdout não é TTY, acrescente `--json` em um terminal
- `vps list` imprime um array, `vps show` imprime um objeto, e `vps path`, `connect` e `vps remove` imprimem um documento de evento

```bash
ssh-cli vps list --json --select name,host,user
ssh-cli vps show web1 --json
ssh-cli vps path --json
ssh-cli connect demo --json
ssh-cli vps remove web1 --json
```

```json
[{"host":"203.0.113.10","name":"demo","user":"ubuntu"},{"host":"203.0.113.11","name":"web1","user":"ubuntu"}]
{"name":"web1","host":"203.0.113.11","port":22,"user":"ubuntu","password":"***","key_path":null,"key_passphrase":null,"sudo_password":null,"su_password":null,"timeout_ms":60000,"max_command_chars":1000,"max_output_chars":100000,"disable_sudo":false,"schema_version":3,"added_at":"2026-10-20T01:20:42.076576907+00:00","tags":["prod"]}
{"ok":true,"event":"vps-path","path":"CONFIG_DIR/config.toml"}
{"ok":true,"event":"vps-connected","name":"demo"}
{"ok":true,"event":"vps-removed","name":"web1"}
```

- `vps show` mascara um segredo guardado com três asteriscos e imprime um ausente como `null`, `added_at` varia
- Com `--output-format text`, `vps list` preenche a coluna `PASSWORD` com três asteriscos para um host que guarda senha e com `(not set)` para um host que usa só chave ou o agente
- `vps show` em modo texto imprime `(not set)` em todo campo vazio, como `Key path`, `Key passphrase` e `Agent socket`, e o rótulo não é traduzido
- `doctor` e `vps doctor` imprimem o mesmo documento `vps-doctor`, `ssh_probe` fica null até o `--probe-ssh` rodar, e o `--select` o reduz aos caminhos pedidos mais `agent_shape`, que é a segunda forma que o schema publica

```bash
ssh-cli doctor --json
```

```json
{"event":"vps-doctor","local":{"active_file":"CONFIG_DIR/active","active_vps":null,"config_dir_exposed":false,"config_dir_permissions":"0700","config_path":"CONFIG_DIR/config.toml","exists":true,"hosts":2,"invalid_host_names":[],"known_hosts":"CONFIG_DIR/known_hosts","layer":"--config-dir","permissions":"600","runtime":{"arch":"x86_64","is_ci":false,"is_container":false,"is_termux":false,"is_wsl":false,"os":"linux","sandbox":null},"schema_version":3,"secrets_at_rest":"encrypted","secrets_key_file":"CONFIG_DIR/secrets.key","secrets_key_source":"xdg_file","secrets_plaintext_opt_out":false,"telemetry":false},"ok":true,"ssh_probe":null}
```

- Um diretório novo mostra `config_dir_exposed` false, `config_dir_permissions` `0700` e `invalid_host_names` vazio
- `secrets status` imprime um objeto sem `event`, `secrets init` e `secrets reencrypt` imprimem um evento, e nenhum imprime a chave

```bash
ssh-cli secrets status --json
ssh-cli secrets init --json
ssh-cli secrets reencrypt --json
```

```json
{"at_rest":"encrypted","encryption_active":true,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","plaintext_opt_out":false}
{"ok":true,"event":"secrets-init","force":false,"key_file":"CONFIG_DIR/secrets.key","key_source":"xdg_file","reencrypted_hosts":0}
{"ok":true,"event":"secrets-reencrypt","hosts":2}
```

- `secrets init` rodou em um diretório novo sem hosts, então `reencrypted_hosts` é 0
- `secrets init` recusa sobrescrever uma chave existente com exit 64 até passar `--force`
- `commands` lista a árvore inteira, `schema` lista os 25 schemas embutidos e `locale` imprime objetos sem `event`
- `locale show` nomeia a camada vencedora em `source`, por exemplo `cli_flag`, `persisted` ou `system`
- `completions <SHELL>` imprime o script como texto puro para `bash`, `elvish`, `fish`, `powershell` ou `zsh`, qualquer outro shell sai com 2

```bash
ssh-cli commands | jaq -c '{bin,event,ok,version,top:(.tree.subcommands|map(.name))}'
ssh-cli schema | jaq -c '{event,ok,count:(.schemas|length)}'
ssh-cli locale show --json
ssh-cli locale set pt-BR --json
ssh-cli locale clear --json
```

```json
{"bin":"ssh-cli","event":"commands","ok":true,"version":"0.5.6","top":["vps","connect","exec","sudo-exec","su-exec","scp","sftp","tunnel","health-check","secrets","completions","commands","schema","doctor","locale","tls"]}
{"event":"schema-catalog","ok":true,"count":25}
{"available":["en","pt-BR"],"current":"en","direction":"ltr","persisted_raw":null,"preference_path":"CONFIG_DIR/lang","resolved":"en","script":"Latn","source":"cli_flag","system_raw":"pt-BR"}
{"applies":"next_invocation_unless_overridden","lang":"pt-BR","ok":true,"path":"CONFIG_DIR/lang"}
{"cleared":true,"ok":true}
```

- Os comandos TLS leem e gravam a árvore `tls/` sob o diretório de configuração e não tocam a rede, exceto `tls acme account create`, `tls acme issue` e `tls acme complete`

```bash
ssh-cli tls provider --json
ssh-cli tls paths --json
ssh-cli tls mtls list --json
ssh-cli tls mtls show edge --json
ssh-cli tls mtls remove edge --json
ssh-cli tls acme account show --json
ssh-cli tls acme list --json
ssh-cli tls acme status --json
```

```json
{"ok":true,"event":"tls-provider","installed":true,"min_rustls":"0.23.18","provider":"aws_lc_rs","stack":"rustls+aws_lc_rs"}
{"ok":true,"event":"tls-paths","acme_account":"CONFIG_DIR/tls/acme/account.json","acme_dir":"CONFIG_DIR/tls/acme","mtls_dir":"CONFIG_DIR/tls/mtls","tls_root":"CONFIG_DIR/tls"}
{"ok":true,"event":"tls-mtls-list","identities":[]}
{"ok":true,"event":"tls-mtls-show","cert_path":"CONFIG_DIR/tls/mtls/edge/cert.pem","key_path":"CONFIG_DIR/tls/mtls/edge/key.pem","name":"edge"}
{"ok":true,"event":"tls-mtls-remove","name":"edge"}
{"ok":true,"event":"tls-acme-account-show","directory":null,"path":"CONFIG_DIR/tls/acme/account.json","present":false}
{"ok":true,"event":"tls-acme-list","domains":[]}
{"ok":true,"event":"tls-acme-status","domains":[]}
```

- `tls mtls show` e `tls mtls remove` rodaram depois de `tls mtls import --name edge`, cuja saída está na seção TLS
- `tls acme status` sem `--domain` lista todos os domínios, e com `--domain example.com` imprime uma entrada com `has_cert`, `has_key` e `has_pending_order`

### Comandos que exigem host: o que imprimem
- Estes comandos precisam de um servidor SSH ou do servidor ACME, então a tabela lista os campos dos schemas publicados e os blocos depois dela mostram as execuções medidas
- `objeto` significa um objeto simples sem campo `event`, e `Campos` nomeia os campos que sempre aparecem
- Rode `ssh-cli schema <NAME>` para ler o contrato completo de qualquer evento

| Comando | Imprime | Campos |
| --- | --- | --- |
| `ssh-cli exec` | object ou `exec-batch` | `target_resolved`, `target_source`, `active_fallback`, `stdout`, `stderr`, `truncated_stdout`, `truncated_stderr`, `duration_ms` e `exit_code` |
| `ssh-cli sudo-exec` | object ou `exec-batch` | os mesmos campos do `exec` |
| `ssh-cli su-exec` | object ou `exec-batch` | os mesmos campos do `exec` |
| `ssh-cli scp upload` | `scp-transfer` ou `scp-batch` | `ok`, `event`, `target_resolved`, `target_source`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`, mais `mtime_preserved` e `durable` |
| `ssh-cli scp download` | `scp-transfer` ou `scp-batch` | os mesmos campos do `scp upload` |
| `ssh-cli sftp upload` | `sftp-transfer` ou `sftp-batch` | `ok`, `event`, `target_resolved`, `target_source`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`, mais `recursive` |
| `ssh-cli sftp download` | `sftp-transfer` ou `sftp-batch` | os mesmos campos do `sftp upload` |
| `ssh-cli sftp ls` | `sftp-list` | `ok`, `event`, `vps`, `path` e `entries` com `name`, `path`, `kind` e, quando existem, `size` e `mode` |
| `ssh-cli sftp mkdir` | `sftp-fs-op` | `ok`, `event`, `op`, `vps`, `path` e `duration_ms` |
| `ssh-cli sftp rmdir` | `sftp-fs-op` | os mesmos campos do `sftp mkdir` |
| `ssh-cli sftp rm` | `sftp-fs-op` | os mesmos campos do `sftp mkdir` |
| `ssh-cli sftp stat` | `sftp-fs-op` | os mesmos campos do `sftp mkdir`, mais `kind`, `size`, `mode` e `mtime` |
| `ssh-cli sftp rename` | `sftp-fs-op` | os mesmos campos do `sftp mkdir`, mais `to` |
| `ssh-cli tunnel` | `tunnel_listening` e depois `tunnel_closed` | `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`, `bind` e `mode`, depois `reason`, `forwards_served`, `capacity_waits` e `duration_ms` |
| `ssh-cli health-check` | object ou `health-check-batch` | `name`, `target_resolved`, `target_source`, `status` e `latency_ms` |
| `ssh-cli tls acme account create` | `tls-acme-account-create` | `present`, `path` e `directory` |
| `ssh-cli tls acme issue` | `tls-acme-challenge` | `domain`, `directory`, `order_url`, `dns_name`, `dns_value` e `created_at` |
| `ssh-cli tls acme complete` | `tls-acme-complete` | `domain`, `has_cert`, `has_key`, `has_pending_order`, `cert_path` e `key_path` |

- Os envelopes de lote trazem `batch_run_id`, `max_concurrency` e `results`, e cada resultado começa com `name` mais `ok` ou `status`
- Estas linhas rodaram contra um `sshd` descartável em 127.0.0.1 com um host chamado `lab`, os caminhos local e remoto dividem um disco sob `/tmp/lab` e `duration_ms` varia
- A conta de teste tem sudo sem senha, então o `sudo-exec` imprimiu uid 0, e o `su-exec` sem senha de su guardada sai com 64 e `su_password_missing`
- As linhas do ACME vêm das structs do código-fonte, porque nenhum servidor ACME foi contatado

```bash
ssh-cli sudo-exec lab "id -u" --json
ssh-cli exec lab "echo one" --step "echo two" --json
ssh-cli connect lab --json
ssh-cli exec --use-active "echo active" --json
ssh-cli health-check lab --json
ssh-cli health-check --use-active --json
```

```json
{"target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","active_fallback":false,"stdout":"0\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":195}
{"active_fallback":false,"command":"echo one","duration_ms":94,"exit_code":0,"host_resolved":"lab","host_source":"argv","stderr":"","stdout":"one\n","step":0,"target_resolved":"lab","target_source":"argv","truncated_stderr":false,"truncated_stdout":false}
{"active_fallback":false,"command":"echo two","duration_ms":96,"exit_code":0,"host_resolved":"lab","host_source":"argv","stderr":"","stdout":"two\n","step":1,"target_resolved":"lab","target_source":"argv","truncated_stderr":false,"truncated_stdout":false}
{"ok":true,"event":"vps-connected","name":"lab"}
{"target_resolved":"lab","target_source":"active_marker","host_resolved":"lab","host_source":"active_marker","active_fallback":true,"stdout":"active\n","stderr":"","exit_code":0,"truncated_stdout":false,"truncated_stderr":false,"duration_ms":103}
{"name":"lab","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","status":"ok","latency_ms":36}
{"name":"lab","target_resolved":"lab","target_source":"active_marker","host_resolved":"lab","host_source":"active_marker","status":"ok","latency_ms":37}
```

- `--step` imprime um objeto por passo com `step` (base 0) e `command`, e o comando posicional é o passo 0
- `--use-active` mostra `target_source` `active_marker` e, no `exec`, `active_fallback` true

```bash
ssh-cli scp upload lab /tmp/lab/payload.bin /tmp/lab/up.bin --json
ssh-cli scp download lab /tmp/lab/up.bin /tmp/lab/down.bin --json
ssh-cli scp upload --all /tmp/lab/payload.bin /tmp/lab/up_all.bin --json
ssh-cli --scp-file-concurrency 2 scp upload lab /tmp/lab/mm1 /tmp/lab/mm2 /tmp/lab/multi --json
```

```json
{"ok":true,"event":"scp-transfer","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","direction":"upload","vps":"lab","local":"/tmp/lab/payload.bin","remote":"/tmp/lab/up.bin","bytes":14,"duration_ms":97,"mtime_preserved":true,"durable":true}
{"ok":true,"event":"scp-transfer","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","direction":"download","vps":"lab","local":"/tmp/lab/down.bin","remote":"/tmp/lab/up.bin","bytes":14,"duration_ms":100,"mtime_preserved":true,"durable":true}
{"event":"scp-batch","target_source":"selector","host_source":"selector","batch_run_id":"01a156eb-42d4-7693-a849-b0b98d461e6e","direction":"upload","max_concurrency":64,"results":[{"name":"lab","ok":true,"bytes":14,"duration_ms":98,"local":"/tmp/lab/payload.bin"},{"name":"lab2","ok":true,"bytes":14,"duration_ms":99,"local":"/tmp/lab/payload.bin"}]}
{"event":"scp-batch","target_source":"argv","host_source":"argv","batch_run_id":"01a156eb-b918-7371-93eb-5f3104740b8a","direction":"upload","max_concurrency":64,"results":[{"name":"/tmp/lab/mm1","ok":true,"bytes":2,"duration_ms":100,"local":"/tmp/lab/mm1"},{"name":"/tmp/lab/mm2","ok":true,"bytes":2,"duration_ms":100,"local":"/tmp/lab/mm2"}]}
```

- Um resultado de lote nomeia o host em `name`, exceto no SCP de vários arquivos, onde `name` é o arquivo local e `target_source` é `argv`
- O diretório de destino de um SCP de vários arquivos precisa existir antes, senão cada resultado informa `file not found`
- `--scp-file-concurrency N` abre N canais SCP paralelos em uma sessão para uma transferência de vários arquivos, e o padrão é 1

```bash
ssh-cli sftp upload lab /tmp/lab/payload.bin /tmp/lab/sf.bin --json
ssh-cli sftp download lab /tmp/lab/sf.bin /tmp/lab/sf_down.bin --json
ssh-cli sftp upload --recursive lab /tmp/lab/tree /tmp/lab/tree2 --json
ssh-cli sftp ls lab /tmp/lab/tree2 --json
ssh-cli sftp mkdir lab /tmp/lab/newdir --json
ssh-cli sftp stat lab /tmp/lab/payload.bin --json
ssh-cli sftp rename lab /tmp/lab/newdir /tmp/lab/newdir2 --json
ssh-cli sftp rmdir lab /tmp/lab/newdir2 --json
ssh-cli sftp rm lab /tmp/lab/sf.bin --json
ssh-cli sftp upload --hosts lab,lab2 /tmp/lab/payload.bin /tmp/lab/hosts.bin --json
```

```json
{"ok":true,"event":"sftp-transfer","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","direction":"upload","vps":"lab","local":"/tmp/lab/payload.bin","remote":"/tmp/lab/sf.bin","bytes":14,"duration_ms":0}
{"ok":true,"event":"sftp-transfer","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","direction":"download","vps":"lab","local":"/tmp/lab/sf_down.bin","remote":"/tmp/lab/sf.bin","bytes":14,"duration_ms":0}
{"ok":true,"event":"sftp-transfer","target_resolved":"lab","target_source":"argv","host_resolved":"lab","host_source":"argv","direction":"upload","vps":"lab","local":"/tmp/lab/tree","remote":"/tmp/lab/tree2","bytes":4,"duration_ms":2,"recursive":true}
{"ok":true,"event":"sftp-list","vps":"lab","path":"/tmp/lab/tree2","entries":[{"name":"sub","path":"/tmp/lab/tree2/sub","kind":"dir","size":60,"mode":16877},{"name":"a.txt","path":"/tmp/lab/tree2/a.txt","kind":"file","size":2,"mode":33188}]}
{"ok":true,"event":"sftp-fs-op","op":"mkdir","vps":"lab","path":"/tmp/lab/newdir","duration_ms":137}
{"ok":true,"event":"sftp-fs-op","op":"stat","vps":"lab","path":"/tmp/lab/payload.bin","duration_ms":0,"kind":"file","size":14,"mode":33188,"mtime":1792459618}
{"ok":true,"event":"sftp-fs-op","op":"rename","vps":"lab","path":"/tmp/lab/newdir","to":"/tmp/lab/newdir2","duration_ms":135}
{"ok":true,"event":"sftp-fs-op","op":"rmdir","vps":"lab","path":"/tmp/lab/newdir2","duration_ms":140}
{"ok":true,"event":"sftp-fs-op","op":"rm","vps":"lab","path":"/tmp/lab/sf.bin","duration_ms":326}
{"event":"sftp-batch","target_source":"selector","host_source":"selector","batch_run_id":"01a156eb-4ad7-7af2-899a-f5dbc1b0a62a","direction":"upload","max_concurrency":64,"results":[{"name":"lab","ok":true,"bytes":14,"duration_ms":0,"local":"/tmp/lab/payload.bin"},{"name":"lab2","ok":true,"bytes":14,"duration_ms":0,"local":"/tmp/lab/payload.bin"}]}
```

- As entradas de `sftp ls` trazem `kind` (`dir` ou `file`), `size` e `mode` como número decimal, e `sftp stat` acrescenta `mtime` em segundos de época
- Uma transferência recursiva acrescenta `recursive` true e `bytes` conta os arquivos copiados

```bash
ssh-cli tunnel lab 18081 --socks5 --timeout-ms 1500 --json
ssh-cli tunnel lab 18082 --remote-socket /run/user/1000/bus --timeout-ms 1500 --json
ssh-cli tunnel lab 18083 127.0.0.1 0 --reverse --timeout-ms 1500 --json
```

```json
{"ok":true,"event":"tunnel_listening","vps":"lab","local_port":18081,"remote_host":"*","remote_port":0,"timeout_ms":1500,"bind":"127.0.0.1","mode":"socks5"}
{"ok":true,"event":"tunnel_listening","vps":"lab","local_port":18082,"remote_host":"/run/user/1000/bus","remote_port":0,"timeout_ms":1500,"bind":"127.0.0.1","mode":"streamlocal"}
{"ok":true,"event":"tunnel_listening","vps":"lab","local_port":34155,"remote_host":"127.0.0.1","remote_port":18083,"timeout_ms":1500,"bind":"127.0.0.1","mode":"reverse"}
```

- Cada tunnel também imprimiu o documento `tunnel_closed` com `reason` `deadline` e exit 0, só o primeiro documento de cada execução aparece
- Sob `--reverse` com `REMOTE_PORT` 0 o servidor alocou a porta e ela voltou em `local_port`, enquanto `remote_port` repete a porta local que você passou

```bash
ssh-cli health-check dead --json
ssh-cli exec badkey "id" --json
ssh-cli --fail-fast --max-concurrency 1 exec --hosts dead,lab "echo hi" --json
```

```json
{"exit_code":74,"error_code":"connection_failed","message":"ssh connection failed: TCP dial failed for 127.0.0.1:1: Connection refused (os error 111)","error_class":"transient","retryable":true,"suggestion":"retry at most twice with exponential full-jitter backoff (exit 74)"}
{"exit_code":77,"error_code":"authentication_failed","message":"ssh authentication failed; try --password-stdin, --key PATH, --key-passphrase-stdin, or verify the user","error_class":"permanent","retryable":false,"suggestion":"change credentials (--key / --password-stdin) or host-key policy; do not blind-retry","target_resolved":"badkey","target_source":"argv","host_resolved":"badkey","host_source":"argv","active_fallback":false}
{"event":"exec-batch","batch_run_id":"01a156eb-b7e7-7691-8e00-c7bdb3a6bc6c","max_concurrency":1,"results":[{"name":"dead","ok":false,"stdout":"","stderr":"ssh connection failed: TCP dial failed for 127.0.0.1:1: Connection refused (os error 111)","duration_ms":0,"error":"ssh connection failed: TCP dial failed for 127.0.0.1:1: Connection refused (os error 111)"},{"name":"lab","ok":false,"stdout":"","stderr":"not attempted (fail-fast stopped admission)","duration_ms":0,"error":"not attempted (fail-fast stopped admission)"}]}
{"exit_code":1,"error_code":"partial_failure","message":"2/2 hosts failed during exec","error_class":"partial","retryable":false}
```

- `dead` aponta para uma porta loopback fechada e `badkey` usa uma chave em que o servidor não confia, então os envelopes são exit 74 `connection_failed` (repetível) e exit 77 `authentication_failed` (permanente)
- Um erro de `health-check` não traz `target_resolved` nem `target_source`, enquanto um erro de `exec` traz os dois
- Com `--fail-fast` o host que nunca foi iniciado aparece em `results` com `ok` false e `not attempted (fail-fast stopped admission)`, e o processo sai com 1 e `partial_failure` no stderr


## SFTP

### Integridade, árvores e metadados
- Prefira a linha de produto 0.5.3 ou mais nova para todo trabalho SFTP
- A G1 corrigiu o truncamento no envio, versões anteriores podiam abrir o arquivo remoto com atributos que zeravam o conteúdo
- Verifique com checksum no destino e nunca confie só em contagens de bytes informadas pelo cliente (G15)
- Árvores recursivas usam `sftp upload --recursive` e `sftp download --recursive`, sem seguir link simbólico e com tetos de profundidade e listagem
- SETSTAT envia `atime` e `mtime` juntos (G3) e um `set_metadata` mutante falha fechado, sem aplicar nada pela metade (G4)
- Os bits de permissão são mascarados por direção: o upload usa `SFTP_PERM_MASK` (`0o7777`, G12) e o download usa `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`)
- Assim um servidor hostil não consegue pôr setuid, setgid ou sticky no arquivo que você acabou de baixar (A3)
- O cancelamento de multi-arquivo e lote mantém `results.len() == input.len()` com o restante cancelado preenchido (G5/G17)
- `sftp ls` e `sftp stat` escapam `\n` e `\t` de um nome remoto no modo texto, então um nome forjado não adiciona linhas
- O JSON de agente usa os schemas `sftp-transfer`, `sftp-list`, `sftp-fs-op` e `sftp-batch` em `docs/schemas/`

```bash
ssh-cli sftp upload demo ./payload.bin /tmp/payload.bin --json
ssh-cli exec demo "sha256sum /tmp/payload.bin" --json --select stdout
sha256sum ./payload.bin
```

- Saída esperada do `sftp upload`, `bytes` é o tamanho do arquivo e `duration_ms` varia

```json
{"ok":true,"event":"sftp-transfer","target_resolved":"demo","target_source":"argv","host_resolved":"demo","host_source":"argv","direction":"upload","vps":"demo","local":"./payload.bin","remote":"/tmp/payload.bin","bytes":8,"duration_ms":1}
```

- Compare os dois digests, o efeito no destino é o critério de aceite


## Verbosidade
- O nível de tracing padrão é error, então o stderr de JSON e de tunnel fica limpo
- A única exceção é array nu reduzido: `vps list --json --limit 2` escreve `{"event":"agent-shape","input_count":N,"output_count":N,"dropped":N}` no stderr, porque array nu não tem onde carregar o relatório sem mudar de tipo
- Essa linha nunca aparece numa execução sem redução, e `--quiet` a suprime como qualquer outra linha de stderr
- Verbosidade graduada (G14): `-v` é info, `-vv` é debug e `-vvv` é trace
- Os filtros são sempre restritos à crate (`warn,ssh_cli=…`) e nunca um `debug` global puro (G2)
- Esse escopo evita vazamento de senha pelos logs de `russh::client::encrypted`
- O `RUST_LOG` do ambiente é ignorado e só `-v`, `-vv` e `-vvv` controlam o tracing do produto
- `-q` silencia a saída humana de sucesso
- Diagnostique sem vazar senha com `ssh-cli -vv exec demo "true" --json`


## Idioma

```bash
ssh-cli locale show
ssh-cli locale set pt-BR
ssh-cli locale clear
ssh-cli --lang en vps list
```

- A preferência fica no XDG no arquivo `lang` e não existe `.env` nem armazenamento de idioma por variável de ambiente
- `locale show` informa o idioma resolvido e a camada vencedora
- `--lang en` sobrescreve em uma chamada e não persiste nada
- Os corpos de erro em JSON ficam em inglês por desenho, para o agente receber texto estável


## TLS

### SSH sobre TLS, mTLS e ACME

```bash
ssh-cli tls provider
ssh-cli tls paths
ssh-cli tls mtls list
ssh-cli tls mtls import --name edge --cert ./client.pem --key ./client-key.pem
ssh-cli tls mtls show edge
ssh-cli tls mtls remove edge
ssh-cli tls acme account create --contact mailto:ops@example.com
ssh-cli tls acme account show
ssh-cli tls acme issue --domain example.com --print-challenge
ssh-cli tls acme complete --domain example.com
ssh-cli tls acme status --domain example.com
ssh-cli tls acme list
```

- Os comandos ACME informam o domínio com `--domain`, um posicional solto sai com 2
- A stack é só rustls com `aws_lc_rs`, sem caminho OpenSSL ou native-tls
- Identidades mTLS e material ACME ficam no XDG `tls/` com segredos em modo 0600
- `tls mtls import --name` recusa caracteres de controle, formatação e bidi com exit 64 e não cria diretório
- Unicode comum como `serviço-1` continua aceito
- Validação permanente do ACME como `invalidContact` sai com 64, não repita como 74
- Saída esperada do `tls mtls import` para o nome `edge`, os caminhos seguem o diretório de configuração

```json
{"ok":true,"event":"tls-mtls-import","cert_path":"CONFIG_DIR/tls/mtls/edge/cert.pem","key_path":"CONFIG_DIR/tls/mtls/edge/key.pem","name":"edge"}
```


## Daemon

### Não existe daemon
- Trate toda invocação como nascer, executar, morrer (one-shot)
- Nunca espere um processo SSH em segundo plano deste projeto
- Limite os tunnels com o `--timeout-ms` obrigatório para o processo ainda terminar


## Padrões avançados

### Frota multi-host com concorrência limitada
- Prefira `exec`, `sudo-exec`, `su-exec`, `scp`, `sftp` e `health-check` com `--all` quando o registro tem mais de um host
- Um processo roda sessões concorrentes limitadas por `--max-concurrency N`, automático por CPUs e RAM, limitado a 1..=64
- Existem três seletores e eles são mutuamente exclusivos: `--all`, `--hosts a,b,c` e `--tags t1,t2`
- `--tags` é aceita só por `exec`, `sudo-exec` e `su-exec`, as tags vêm de `vps add --tag`
- `scp`, `sftp` e `health-check` aceitam `--all` e `--hosts`
- O marcador ativo é uma quarta rota, deliberada: `--use-active` em `exec`, `sudo-exec`, `su-exec` e `health-check`
- Interprete o JSON de lote com `health-check-batch`, `exec-batch`, `scp-batch` e `sftp-batch`, o envelope traz `max_concurrency`
- Não crie um processo da interface de linha de comando (CLI) por host quando `--all` está disponível

```bash
ssh-cli --max-concurrency 8 health-check --all --json
ssh-cli exec --tags prod,edge "echo hi" --json
```

- Saída esperada do `health-check --all`, `batch_run_id` e `latency_ms` variam

```json
{"event":"health-check-batch","batch_run_id":"00000000-0000-7000-8000-000000000000","max_concurrency":8,"results":[{"name":"demo","status":"ok","latency_ms":38},{"name":"demo2","status":"ok","latency_ms":41}]}
```

- Saída esperada do `exec --tags`, um envelope `exec-batch` e sem precisar enumerar nomes

```json
{"event":"exec-batch","batch_run_id":"00000000-0000-7000-8000-000000000000","max_concurrency":64,"results":[{"name":"demo2","ok":true,"exit_code":0,"stdout":"hi\n","stderr":"","duration_ms":147}]}
```

- Uma falha parcial sai com 1 e imprime o envelope de erro `partial_failure` no stderr

### Reduza o payload antes de ele ser escrito
- As oito flags de redução valem antes da serialização, então o envelope grande nunca é construído
- Passar o stdout por uma ferramenta JSON externa paga o custo total de tokens primeiro e reduz o payload depois
- O lote do health-check informa `status` (`ok` ou `error`), então filtre por `status` e não por `ok`
- Em um lote, `--truncate-content` marca `agent_shape.content_truncated` e `--count-only` mantém o envelope e troca `results` por `count`

```bash
ssh-cli health-check --all --json --select name,status --filter status!=ok
```

- Saída esperada, `agent_shape` diz quantos registros foram descartados e o exit code é 1 porque um host falhou

```json
{"agent_shape":{"dropped":2,"input_count":3,"output_count":1},"batch_run_id":"00000000-0000-7000-8000-000000000000","event":"health-check-batch","max_concurrency":64,"results":[{"name":"down1","status":"error"}]}
```

- O array puro de `vps list` não tem espaço para `agent_shape`, então o relatório da redução vai para o stderr em uma linha JSON com `"event":"agent-shape"` sempre que algo foi descartado ou truncado, e o `--quiet` a suprime
- `--count-only` em uma lista imprime só a contagem

```bash
ssh-cli vps list --json --count-only
```

```json
{"count":3}
```

### Troque o método de autenticação em uma chamada
- `--key` limpa `password`, `use_agent` e `agent_socket`
- `--password` e `--password-stdin` limpam `key_path`, `key_passphrase`, `use_agent` e `agent_socket`
- `--use-agent` limpa `password`, `key_path` e `key_passphrase`
- `updated` lista o método novo primeiro e depois só os campos limpos que tinham valor
- Uma `--key` diferente descarta a passphrase antiga a menos que a chamada traga uma nova, e o mesmo caminho de chave não muda nada
- `--agent-socket` sozinho implica `--use-agent` e limpa `password`, `key_path` e `key_passphrase`
- Duas flags primárias de autenticação na mesma chamada saem com 2, e uma passphrase sem chave sai com 64 e deixa o registro intacto

```bash
ssh-cli vps edit web1 --key ~/.ssh/id_ed25519 --json
```

```json
{"ok":true,"event":"vps-edited","host_resolved":"web1","host_source":"argv","name":"web1","target_resolved":"web1","target_source":"argv","updated":["key_path","password"]}
```

### Renomeie sem digitar a credencial de novo
- `ssh-cli vps rename <NAME> <NEW_NAME>` move a chave do registro, o nome guardado e o marcador `active` vizinho quando ele aponta para este host
- `ssh-cli vps edit <NAME> --name <NEW>` chega ao mesmo caminho de código e também emite `vps-renamed`, `vps-edited` só é emitido quando nada é renomeado
- Os segredos guardados continuam legíveis porque o registro os sela sem dados associados, então o nome novo não muda nada neles
- O antigo desvio export, edição, import não é mais necessário e ele descartava a autenticação de agente
- Um nome inválido, igual ao atual ou já usado sai com 64 (`vps_duplicate` para um nome já usado), uma origem desconhecida sai com 66

```bash
ssh-cli vps rename web1 web-1 --json
```

```json
{"ok":true,"event":"vps-renamed","active_rewritten":false,"host_resolved":"web1","host_source":"argv","name":"web-1","previous_name":"web1","target_resolved":"web1","target_source":"argv","updated":[]}
```

- As oito flags de limpeza são `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key`, `--no-key` e `--no-agent-socket`

### Campos do host, SSH sobre TLS e as flags que as tabelas só listam
- `vps add` exige `--name`, `--host` e `--user`, a porta é 22 a menos que `--port` diga outra
- `--sudo-password` e `--su-password` guardam as senhas de elevação por host, prefira `--sudo-password-stdin` e `--su-password-stdin` para nada ir ao argv
- `--check` roda um health-check logo depois do add, e uma sonda que falha sai com 74 depois de o host já estar registrado
- `--tls` liga o SSH sobre TLS (rustls) no host, e `vps edit --no-tls` desliga de novo
- `--tls-sni` define o nome do servidor TLS e assume `--host` quando omitida, `vps edit --no-tls-sni` a limpa
- `--tls-client-cert` e `--tls-client-key` apontam para os arquivos PEM do certificado e da chave de cliente mTLS, `vps edit --no-tls-client-cert` e `--no-tls-client-key` os limpam
- `--tls-sni` sem `--tls` é guardada sem erro

```bash
ssh-cli vps add --name t1 --host 127.0.0.1 --user u --key KEY_PATH --tls --tls-sni sni.example.com --tls-client-cert ./client.pem --tls-client-key ./client-key.pem --json
ssh-cli vps edit t1 --no-tls --json
```

```json
{"ok":true,"event":"vps-added","name":"t1","secrets_key_auto_created":false}
{"ok":true,"event":"vps-edited","host_resolved":"t1","host_source":"argv","name":"t1","target_resolved":"t1","target_source":"argv","updated":["tls"]}
```

- `vps show` passa a imprimir `tls` null e mantém `tls_sni`
- `scp` e `sftp` aceitam `--src` (repetível) com `--dest` (um diretório) quando uma transferência tem várias origens
- `secrets init --keyring` guarda a chave primária no keyring do SO em vez de `secrets.key`
- `tls mtls import` exige `--name`, `--cert` (PEM da cadeia de certificados) e `--key` (PEM da chave privada)
- `tls acme account create` exige uma ou mais URLs em `--contact` como `mailto:ops@example.com`, `--staging` escolhe o diretório de staging do Let's Encrypt e `--force` substitui uma conta existente
- `tls acme issue --staging` também usa o diretório de staging, e `--print-challenge` é obrigatória na prática porque sem ela a chamada sai com 64

### Ensaie com --dry-run
- `--dry-run` é aceita só por `vps remove`, `vps import`, `vps edit`, `vps rename`, `sftp rm`, `sftp rmdir`, `secrets init` e `secrets reencrypt`
- Em qualquer outro lugar ela é rejeitada com exit 64 em vez de aceita e ignorada, então um ensaio nunca é confundido com uma operação que já rodou
- O plano de `vps remove`, `vps edit` e `vps rename` ecoa `target_resolved`, `target_source` (`argv`), `host_resolved` e `host_source` depois que a busca dá certo
- Um host ausente ainda sai com 66 e não cria diretório nem arquivo de lock
- A prévia de `vps import` lista cada host em `hosts` com `name`, `replaces_existing` e `incomplete`, então leia `replaces_existing` antes do import real
- A prévia de `secrets init --force` informa `hosts_to_reencrypt`, e girar a chave sem recifrar esses hosts perde os segredos em repouso deles
- As prévias de `sftp rm` e `sftp rmdir` nunca conectam e nunca conferem se o caminho existe, elas ecoam `vps` e `remote`

```bash
ssh-cli vps remove web1 --dry-run --json
```

```json
{"ok":true,"event":"dry-run","config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"host_resolved":"web1","host_source":"argv","name":"web1","operation":"vps-remove","target_resolved":"web1","target_source":"argv"}
```

- Saída esperada de outras três prévias, `CONFIG_DIR` representa o seu diretório de configuração

```bash
ssh-cli --json --dry-run vps import --file hosts.json
ssh-cli --json --dry-run secrets init --force
ssh-cli --json --dry-run sftp rm demo /tmp/old.log
```

```json
{"ok":true,"event":"dry-run","config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"hosts":[{"incomplete":false,"name":"demo","replaces_existing":true},{"incomplete":false,"name":"edge","replaces_existing":true}],"imported":2,"operation":"vps-import","source":"CONFIG_DIR/hosts.json"}
{"ok":true,"event":"dry-run","config_path":"CONFIG_DIR/config.toml","dry_run":true,"executed":false,"force":true,"hosts_to_reencrypt":2,"keyring":false,"operation":"secrets-init"}
{"ok":true,"event":"dry-run","dry_run":true,"executed":false,"operation":"sftp-rm","remote":"/tmp/old.log","vps":"demo"}
```

### Repare um nome de host legado e confira o diretório de configuração
- Um `config.toml` antigo com um nome fora de `[A-Za-z0-9._-]` ainda carrega, lista e resolve
- O `doctor` lista esses nomes, escapados, em `local.invalid_host_names`
- `vps rename '<legacy name>' <new name>` repara um nome porque a origem não é validada
- Um nome com caractere de controle é recusado como alvo por `exec`, `health-check`, `--hosts` e `connect` com exit 64
- O `doctor` também reporta `config_dir_exposed` e `config_dir_permissions` do diretório de configuração
- O `--select` também alcança o objeto do `doctor`, como em `ssh-cli doctor --select local.invalid_host_names`, e os blocos abaixo usam `jaq` só para imprimir o valor puro

```bash
ssh-cli doctor --json | jaq -c '.local.invalid_host_names'
ssh-cli vps rename 'legacy host' legacy-host --json
ssh-cli doctor --json | jaq -c '.local | {invalid_host_names,config_dir_exposed,config_dir_permissions}'
```

```json
["legacy host"]
{"ok":true,"event":"vps-renamed","active_rewritten":false,"host_resolved":"legacy host","host_source":"argv","name":"legacy-host","previous_name":"legacy host","target_resolved":"legacy host","target_source":"argv","updated":[]}
{"invalid_host_names":[],"config_dir_exposed":false,"config_dir_permissions":"0700"}
```

### Automação de agente mais segura
- Passe segredos por flags de stdin (`--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--key-passphrase-stdin`) em vez do argv
- Anexe comentários de shell com `--description` para um histórico remoto fácil de auditar
- Desative a elevação em tarefas não confiáveis com `--disable-sudo`, por chamada como flag global, exceto antes de `vps add` ou `vps edit`, onde ela fica gravada no registro
- Persista o bloqueio no registro com `vps edit <NAME> --disable-sudo` e libere-o com `--enable-sudo`
- Troque uma chave de host legítima só após confirmação humana, com `--replace-host-key` (confiança no primeiro uso, TOFU)
- Passe `--no-input` em execuções sem operador para que um stdin ausente falhe rápido
- `--no-input` com uma flag de stdin como `--password-stdin` sai com 64, porque uma proíbe o que a outra lê
- O `stdout` e o `stderr` remotos do `exec` continuam crus por desenho, então remova os bytes de controle de terminal antes de mostrá-los a uma pessoa
- Só o `vps add --timeout` abaixo de 1000 ms avisa no stderr, enquanto `vps edit`, `exec` e `health-check` o aceitam em silêncio
- Um valor parecido com senha no argv (`--password` e as irmãs) avisa no stderr em `vps add`, `vps edit` e `exec`, então não interprete essas linhas como envelope de erro
- Em falha de `scp --json` ou `sftp --json`, interprete o envelope de erro JSON no stderr (`exit_code`, `message`)

### Export e import
- Exporte um inventário redigido com `ssh-cli --output-format text vps export -o hosts.toml`
- O corpo segue o formato resolvido, então sem `--output-format text` ele sai em JSON porque o stdout de um agente nunca é TTY
- Um export redigido limpa todo segredo: `password` é `""` e `key_passphrase`, `sudo_password` e `su_password` são `null`
- Um falso texto cifrado `sshcli-enc:` vazio nunca é gravado
- `vps list` e `vps show` mascaram segredos não vazios com `FIXED_MASK` (três asteriscos)
- O export JSON de agente é `ssh-cli vps export --json`, que imprime o envelope `vps-export`
- `--include-secrets` exige `-o` ou `--output`, ou `--i-understand-secrets-on-stdout`, senão sai com 64
- Importe com `ssh-cli vps import --file hosts.toml` (TOML com chaves em inglês ou aliases legados em português) ou um envelope JSON `vps-export`
- A chave do usuário é `username` no TOML (legado `usuario`) e `user` no JSON
- `vps import` confere todos os nomes primeiro e lista todos os inválidos de uma vez com exit 64 e nada gravado, também sob `--dry-run`
- O evento de sucesso traz `imported` e `replaced`, os nomes ordenados dos hosts que ele sobrescreveu
- Use `--allow-incomplete` para hosts redigidos ou esqueleto sem autenticação completa
- `vps import --file` com um arquivo que não existe sai com 66 e `file_not_found`, `error_class` permanent e `retryable` false, então confira o caminho em vez de repetir
- `added_at` é opcional no import e assume agora por padrão
- O inventário em fio é o schema v3, gravações novas usam chaves em inglês e as leituras ainda aceitam os aliases legados em português
- Recifre um inventário em texto puro após uma atualização com `ssh-cli secrets reencrypt`
- O export de um host legado cujo nome falha no charset estrito só restaura depois de `vps rename`
- O export cria os diretórios de saída ausentes com modo 0700 e nunca altera um existente
- `vps list` e `vps show` imprimem uma senha vazia como `null` em JSON, um caminho diferente do export
- Com `-o` o stdout é um resumo curto do arquivo gravado, e o corpo fica no arquivo

```bash
ssh-cli vps export -o hosts.json --json
ssh-cli vps import --file hosts.json --json
```

```json
{"ok":true,"event":"vps-export","format":"json","include_secrets":false,"path":"CONFIG_DIR/hosts.json"}
{"ok":true,"event":"vps-import","imported":2,"replaced":["demo","edge"]}
```

- Um export redigido de um host de senha é recusado pelo `vps import` com exit 64, complete o host com `vps edit` ou passe `--allow-incomplete`


## Modos de tunnel

### Um bind, uma sessão SSH, quatro formas
- `tunnel` abre um bind local e uma sessão SSH por invocação (G-PAR-30)
- Vários tunnels significam vários one-shots com portas distintas
- Todo modo exige `--timeout-ms`, um tunnel sem prazo é um daemon e esta CLI não entrega um
- Leia o campo JSON `mode` para saber a forma: `local`, `reverse`, `socks5` ou `streamlocal`
- O forward local padrão alcança um endereço remoto fixo

```bash
ssh-cli tunnel prod 15432 10.0.0.5 5432 --timeout-ms 60000 --json
```

- Saída esperada de um tunnel local de 3 segundos na porta 18080, dois documentos e exit 0

```json
{"ok":true,"event":"tunnel_listening","vps":"demo","local_port":18080,"remote_host":"127.0.0.1","remote_port":22922,"timeout_ms":3000,"bind":"127.0.0.1","mode":"local"}
{"ok":true,"event":"tunnel_closed","vps":"demo","reason":"deadline","bind":"127.0.0.1","local_port":18080,"forwards_served":0,"capacity_waits":0,"duration_ms":3001,"mode":"local"}
```

- Espere o `tunnel_listening` antes de usar a porta local
- `tunnel_closed` traz `reason`, `forwards_served` e `capacity_waits`, e distingue um prazo limpo de um semáforo saturado
- `reason` é `deadline` (exit 0), `signal` ou `accept_error`, então leia-o junto com o exit code
- Um SIGTERM depois do `tunnel_listening` imprimiu este documento e saiu com 143, e um SIGINT imprimiu o mesmo documento e saiu com 130

```json
{"ok":true,"event":"tunnel_closed","vps":"lab","reason":"signal","bind":"127.0.0.1","local_port":34439,"forwards_served":0,"capacity_waits":0,"duration_ms":239,"mode":"local"}
```

- Um tunnel sem `--timeout-ms` não vira daemon, é um erro de uso que sai com 2
- `--socks5` serve um proxy SOCKS5 local (RFC 1928, sem autenticação mais CONNECT) e escolhe o destino por conexão, então `REMOTE_HOST` e `REMOTE_PORT` são omitidos (G-TUN-R02)
- O proxy recusa BIND e UDP ASSOCIATE e envia nomes de host ao servidor sem resolver, então a resolução de nomes acontece no lado remoto
- Sob `--socks5` os eventos informam `remote_host` `*` e `remote_port` 0, e sob `--remote-socket` informam o caminho do socket em `remote_host` e `remote_port` 0

```bash
ssh-cli tunnel prod 1080 --socks5 --timeout-ms 300000 --json
```

- `--remote-socket` alcança um socket Unix no servidor por `direct-streamlocal@openssh.com`, que é como Docker, PostgreSQL e systemd ficam alcançáveis (G-TUN-R03)
- O caminho do socket precisa ser absoluto ou a chamada sai com 64, e a existência local nunca é consultada
- O cliente pode rodar no Windows porque localmente só fala TCP, o servidor é que precisa suportar a extensão

```bash
ssh-cli tunnel prod 2375 --remote-socket /var/run/docker.sock --timeout-ms 60000 --json
```

- `--reverse` deixa o servidor escutar e entregar de volta na sua porta local, para um webhook de retorno, um depurador remoto ou um bastião invertido (G-TUN-R01)
- `REMOTE_PORT` pode ser `0` sob `--reverse`, o servidor aloca a porta e a informa em `local_port`
- Um forward local não aceita `0` porque não haveria nada para conectar

```bash
ssh-cli tunnel prod 8080 0.0.0.0 9000 --reverse --i-accept-network-exposure --timeout-ms 120000 --json
```

### Segurança do bind e encerramento
- O `--bind` local assume `127.0.0.1` e o clap o valida como IP, então um erro de digitação como `127.0.0..1` sai com 2 na análise (G-TUN-R08)
- O bind remoto sob `--reverse` é o posicional `<remote_host>` e de propósito não é analisado como IP, porque a RFC 4254 dá sentido a nomes e à string vazia
- Um erro de digitação ali sai com 64 pelo guarda de exposição e não com 2 pelo clap
- Sob `--reverse` a flag `--bind` é aceita e depois ignorada, a entrega é forçada para o loopback e nada avisa
- Defina o endereço do lado do servidor por `<remote_host>`
- Qualquer bind roteável exige `--i-accept-network-exposure` (G-TUN-R13), sem ela o serviço encaminhado alcançaria toda a rede local em silêncio
- Sob `--reverse` essa flag protege o endereço de bind do servidor, já que essa é a ponta exposta
- Um timeout antes do bind ainda sai com 74 e um prazo depois do bind sai com 0 (TUN-002)
- `tunnel` aceita as mesmas flags de autenticação do `exec`: `--password-stdin`, `--key`, `--key-passphrase` e `--key-passphrase-stdin` (CLI-005)


## Configuração

### Registro multi-host no XDG
- Resolva o caminho da configuração com `ssh-cli vps path`
- As gravações em `config.toml` são atômicas com modo 0600 (tempfile, fsync e flock)
- Os arquivos vizinhos `active`, `known_hosts`, `lang` e `secrets.key` ficam ao lado da configuração
- `config.toml.lock` é o lock do registro pelo qual o `--lock-wait-ms` espera, e um escritor que perde a corrida sai com 69 e `retryable` true
- Um diretório que a CLI cria nasce 0700, um diretório XDG padrão existente é apertado para 0700 quando não é link simbólico e é seu
- Qualquer outro diretório existente nunca recebe chmod, o `doctor` mostra `config_dir_exposed` e `config_dir_permissions` no lugar
- `--config-dir something.toml` usa o diretório pai para tudo: `secrets.key`, `known_hosts`, `active`, `lang` e os arquivos TLS
- Duas tabelas de config que normalizam para o mesmo nome saem com 65 com `config has two hosts named ...`, então nenhum host se perde em silêncio
- `known_hosts.lock` e `lang` são criados 0600 sem janela antes do chmod, e `vps export -o new/deep/e.json` cria os níveis ausentes 0700
- O Windows não tem chmod, então o diretório depende da ACL do perfil
- Guarde timeout, `max_command_chars`, `max_output_chars`, segredos de sudo e su por host
- A cifragem em repouso padrão é ChaCha20-Poly1305 e os segredos viram blobs `sshcli-enc:v2:…`, e os blobs legados `v1` ainda são lidos
- O controle da chave primária é só CLI e XDG, resolvido nesta ordem: `--secrets-key-file <PATH>`, o keyring do SO sob `--use-keyring`, o `secrets.key` do XDG e, por fim, a entrada de keyring de um `secrets init --keyring` anterior
- Um keyring sem backend respondendo sai com 69 na gravação, e sob `--use-keyring` uma leitura cai para o `secrets.key`
- `SSH_CLI_SECRETS_KEY` e `SSH_CLI_SECRETS_KEY_FILE` são sempre recusadas, variável de ambiente não é um armazenamento
- Quando uma delas está definida, todo comando que carrega segredos do registro sai com 64 e `are not supported`: `doctor`, `secrets status`, `secrets reencrypt`, `vps add` com senha, e `vps list`, `vps show`, `vps export` e `vps edit` assim que o registro guarda um segredo
- Comandos que nunca tocam segredos, como `vps path`, `locale show`, `schema`, `commands` e `tls provider`, saem 0 com a variável definida, e as outras variáveis `SSH_CLI_*` são ignoradas
- A exceção de texto puro só para testes é `--allow-plaintext-secrets`
- `vps doctor --json` reporta caminhos, schema, contagem de hosts, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file` e `secrets_plaintext_opt_out`
- Também reporta `active_vps`, `config_dir_exposed`, `invalid_host_names` e `config_dir_permissions` em `local`
- `config_dir_permissions` é uma string octal de quatro dígitos como `0700` e fica ausente fora do Unix
- `permissions` continua descrevendo só o `config.toml`
- Não existe armazenamento `.env` de execução do produto


## Erros comuns
- Um erro de uso do clap (exit 2) e um `--filter` malformado (exit 64) imprimem texto puro no stderr mesmo com `--json`, só erros de execução trazem o envelope JSON
- Erros de execução imprimem um envelope JSON no stderr e deixam o stdout vazio
- O envelope sempre traz `exit_code`, `error_code`, `message`, `error_class` e `retryable`, e pode acrescentar `suggestion`, `remote_exit_code`, `target_resolved`, `target_source` e `active_fallback`
- Leia `retryable` antes de qualquer repetição, o exemplo abaixo é um erro permanente que o stderr imprimiu em uma linha

```bash
ssh-cli vps rename ghost new --json
```

```json
{"exit_code":66,"error_code":"vps_not_found","message":"vps 'ghost' not found in registry","error_class":"permanent","retryable":false,"suggestion":"the host is not in the registry; if an earlier `vps rename` or `vps remove` may have succeeded, run `vps list` before repeating"}
```

- Leia os exit codes e exemplos copiáveis offline com `ssh-cli --help`, eles ficam sob `Examples:` e `Exit status:` e não custam rede
- O `--help` de cada uma das 48 folhas acrescenta um bloco `Examples:`, e o `-h` continua curto
- Uma falha antes do despacho, como um `--filter` inválido, sai com 64 em prosa sem envelope, e a prosa segue o idioma do sistema, então passe `--lang en` quando um script compara o texto
- Um `--filter` sem operador e um com chave vazia como `=prod` saem com 64, enquanto `name~~prod` e um valor vazio como em `name=` são aceitos

| Sintoma | Exit | Correção |
| --- | --- | --- |
| `exec demo --step "cmd"` sem comando | 64 | Um posicional solto é erro de uso, passe `exec demo "cmd"` ou `exec --use-active "cmd"` |
| `vps edit --key K --password-stdin` | 2 | Escolha uma flag de autenticação, o clap recusa dois métodos primários |
| `vps edit --key K` com arquivo ausente | 66 | O caminho da chave precisa existir, confira com `ls` |
| `vps edit --no-key` em um host só com chave | 64 | Acrescente outro método antes, o host não autenticaria em lugar nenhum |
| `vps edit demo` sem campo | 64 | Passe ao menos um campo, `--name`, `--tag` ou uma flag de limpeza |
| `vps add --name 'bad name'` | 64 | Use letras ASCII, dígitos, ponto, hífen e sublinhado |
| `vps import` lista nomes inválidos | 64 | Corrija os nomes no arquivo, nada foi importado |
| `vps rename ghost new` | 66 | A origem não existe, rode `vps list` |
| `tls acme issue example.com` | 2 | Use `--domain example.com` |
| `health-check --all` com um host fora do ar | 1 | Leia `results[].status` e `error`, os irmãos não são abortados |
| `tls acme issue --domain example.com` sem `--print-challenge` | 64 | Acrescente `--print-challenge`, o fluxo tem dois passos e nunca espera |
| `tls acme complete --domain example.com` sem pedido pendente | 66 | Rode antes `tls acme issue --domain example.com --print-challenge` |
| `vps import` de um export redigido | 64 | Complete os hosts com `vps edit` ou passe `--allow-incomplete` |
| `--no-input` com `--password-stdin` | 64 | Tire o `--no-input` quando um segredo chega pelo stdin de propósito |
| `tunnel` sem `--timeout-ms` | 2 | Passe `--timeout-ms`, um tunnel sem prazo não existe |
| `--max-concurrency 0` ou acima de 64 | 2 | Escolha de 1 a 64 ou omita a flag para o valor automático |
| `--dry-run` em um comando sem prévia | 64 | Tire o `--dry-run`, só existem as oito prévias listadas acima |
| `sudo-exec` em um host com sudo desativado | 77 | Rode `vps edit NAME --enable-sudo` quando o bloqueio é persistente |
| `su-exec` sem senha de su guardada | 64 | Rode `vps edit NAME --su-password-stdin`, o código de erro é `su_password_missing` |
| `--filter nameprod` ou `--filter =prod` | 64 | Escreva `key=value`, `key!=value` ou `key~substring` com chave não vazia, a resposta é prosa e não um envelope |
| `exec --use-active demo "cmd"` com um nome de host | 64 | Passe só o comando, o `--use-active` já nomeia o host |
| `exec --use-active --all "cmd"` | 2 | Escolha uma rota só, o marcador ativo e um seletor se excluem |
| `exec --use-active` sem marcador ativo | 66 | Rode antes `ssh-cli connect NAME`, o código de erro é `no_active_vps` |
| `SSH_CLI_SECRETS_KEY` definida no ambiente, em um comando que carrega segredos | 64 | Remova a variável, a chave vem de `secrets.key`, `--secrets-key-file` ou `--use-keyring` |
| Lock do config preso por outro processo | 69 | Tente de novo, o erro é repetível, ajuste `--lock-wait-ms` |
| Conexão recusada ou timeout | 74 | Confira host, porta e rede, repita no máximo duas vezes |
| Autenticação falhou ou chave de host divergente | 77 | Mude as credenciais, nunca repita às cegas, confirme mudanças de chave com um humano |


## Códigos de saída (sysexits)
- Esta tabela bate com a seção `Exit status:` impressa por `ssh-cli --help`, as duas listam os mesmos 14 códigos

| Código | Significado |
| --- | --- |
| 0 | Sucesso |
| 1 | Falha geral de execução, por exemplo um exit remoto diferente de zero (o stdout mantém o objeto do `exec` e o stderr recebe `command_failed` com `remote_exit_code`) ou uma falha parcial multi-host |
| 2 | Erro de uso do clap (flags inválidas), o stderr traz prosa com linhas `error:`, `tip:` e `Usage:` e nenhum envelope JSON |
| 64 (`EX_USAGE`) | Argumento inválido ou uso de domínio, incluindo comando vazio, `--include-secrets` recusado sem `-o` ou ack, `--dry-run` recusado e validação permanente do ACME |
| 65 (`EX_DATAERR`) | Dado TOML ou JSON inválido, incompatibilidade de schema, dois hosts com o mesmo nome normalizado |
| 66 (`EX_NOINPUT`) | VPS não encontrado, nenhum VPS ativo ou arquivo ausente, como uma chave, uma origem local de transferência ou um `vps import --file` (`file not found: <path>`), permanente e nunca repetido |
| 69 (`EX_UNAVAILABLE`) | Um serviço do host não responde (keyring do SO ou lock do config), transitório e repetível |
| 70 (`EX_SOFTWARE`) | Falha interna sem entrada corrigível pelo usuário, permanente |
| 73 (`EX_CANTCREAT`) | Falha ao gravar ou criar a configuração |
| 74 (`EX_IOERR`) | Falha de conexão, IO ou timeout |
| 77 (`EX_NOPERM`) | Autenticação falhou, política de chave de host, permissão, sudo desativado ou segredo que falha ao decifrar |
| 130 | SIGINT |
| 141 | Pipe quebrado no stdout ou no stderr, o leitor fechou o pipe e a saída está incompleta, nunca repita às cegas |
| 143 | SIGTERM |

- Linha de produto: 0.5.6


## Próximos passos
- Copie receitas prontas do [COOKBOOK.pt-BR.md](COOKBOOK.pt-BR.md)
- Siga o contrato de agente em [AGENTS.pt-BR.md](AGENTS.pt-BR.md)
- Veja as notas de plataforma em [CROSS_PLATFORM.pt-BR.md](CROSS_PLATFORM.pt-BR.md)
- Atualize com [MIGRATION.pt-BR.md](MIGRATION.pt-BR.md)


## Integração com agentes de IA
- Carregue o pacote de skill em `skills/ssh-cli-pt/`
- Prefira a saída JSON para interpretação por ferramentas
- Siga o roteamento por exit code antes de repetir, veja [AGENTS.pt-BR.md](AGENTS.pt-BR.md)
- Leia [AGENTS.pt-BR.md](AGENTS.pt-BR.md) e [../INTEGRATIONS.pt-BR.md](../INTEGRATIONS.pt-BR.md)
- Os formatos de evento e payload estão em [docs/schemas/README.md](schemas/README.md)
- O binário embute os mesmos 25 schemas, liste-os com `ssh-cli schema` e imprima um com `ssh-cli schema <NAME>`
- Nunca registre em log a chave primária, senhas de host ou segredos decifrados
