# ssh-cli: CLI SSH one-shot para agentes LLM


[![docs.rs](https://img.shields.io/docsrs/ssh-cli)](https://docs.rs/ssh-cli)
[![crates.io](https://img.shields.io/crates/v/ssh-cli)](https://crates.io/crates/ssh-cli)
[![License](https://img.shields.io/crates/l/ssh-cli)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.89-orange)](https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/)
[![Downloads](https://img.shields.io/crates/d/ssh-cli)](https://crates.io/crates/ssh-cli)
[![Rust](https://img.shields.io/badge/rust-1.89%2B-blue)](https://www.rust-lang.org)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.pt-BR.md)

> Dê a qualquer modelo de linguagem grande (LLM) poder SSH remoto por um binário one-shot de interface de linha de comando (CLI) seguro em memória.

- Leia este documento em [Inglês (en)](README.md)
- Instale com `cargo install ssh-cli --locked` e verifique com `ssh-cli --version`
- A versão atual é `0.5.6` com versão mínima suportada do Rust (MSRV) `1.89`
- Mapeie o produto para LLMs em [llms.pt-BR.txt](llms.pt-BR.txt) e [llms-full.txt](llms-full.txt)
- Pule para o [Início rápido](#início-rápido), a [Documentação](#documentação) ou o [Cookbook](docs/COOKBOOK.pt-BR.md)


## O que é?
### CLI SSH multi-host one-shot para agentes
- Entregue um binário Rust único sem runtime Node e sem daemon
- Opere N hosts VPS a partir de armazenamento XDG sem arquivos `.env`
- Autentique por host com senha, chave privada ou agente SSH
- Execute `exec`, `sudo-exec` e `su-exec` como processos one-shot puros que morrem depois do comando
- Devolva JSON estruturado no stdout para orquestração, com falhas de execução em um envelope estável no stderr
- Não colete telemetria em nenhum build


## Por que ssh-cli?
### Substitua processos SSH de longa duração por um binário que morre após rodar
- Evite processos residentes que seguram sockets e RAM entre tarefas
- Cadastre credenciais multi-host uma vez, com cifragem em repouso e escrita atômica
- Confie em host keys por confiança no primeiro uso (TOFU) `known_hosts` em vez de confiar sempre
- Encaminhe erros com códigos sysexits que agentes classificam com segurança


## Novidades da 0.5.6
- O `russh` 0.65.1 fecha a RUSTSEC-2026-0492, a RUSTSEC-2026-0495 e dois advisories GHSA, e o MSRV agora é Rust 1.89 porque o russh 0.65 o exige
- O `secrets init --keyring` grava no cofre do sistema operacional, e um cofre indisponível sai 69 em vez de relatar sucesso
- `--select` e `--truncate-content` agora reduzem um objeto único, então `commands --select tree.about` devolve um campo
- Um array nu reduzido escreve uma linha `{"event":"agent-shape",...}` em stderr, e o `--quiet` a suprime
- Uma flag de coleção passada em objeto único é nomeada em `agent_shape.ignored` em vez de ser descartada em silêncio
- Todas as 48 folhas mostram bloco `Examples:` no `--help`
- A âncora TLS não carrega mais autoridade destituída, e um gate novo compara o banco de advisory com a árvore resolvida
- Renomeie um host nativamente com `vps rename` ou `vps edit --name`, mantendo toda credencial
- Troque o método de autenticação de um host em uma chamada de `vps edit`
- Nomes de host seguem uma regra ASCII estrita, e nomes legados ainda carregam e podem ser reparados
- O modo texto escapa sequências de controle de terminal
- Um diretório de config novo nasce `0700`, e `vps doctor` informa `config_dir_exposed`, `config_dir_permissions` e `invalid_host_names`
- `--dry-run` cobre `vps edit` e `vps rename` e ecoa o alvo
- `--lock-wait-ms` limita a espera do lock do config e sai 69
- Leia a lista completa em [CHANGELOG.pt-BR.md](CHANGELOG.pt-BR.md)


## Superpoderes para agentes de IA
### Execução remota
- Rode `exec`, `sudo-exec` e `su-exec` one-shot, com empacotamento seguro de `sudo` via `sh -c` e escape de shell
- Nomeie o alvo de propósito com `<VPS> <COMMAND>`, um seletor (`--all`, `--hosts a,b`, `--tags prod`) ou `--use-active`
- Um posicional solitário sai com 64, então um nome de host nunca roda como binário remoto
- Confira o alvo depois com `target_resolved`, `target_source` e `active_fallback` no envelope de host único
- Leia `host_resolved` e `host_source` como os aliases de leitura dos mesmos campos
- Adicione comandos extras na mesma sessão SSH com `--step`
- `exec --json` com um passo emite exatamente um objeto NDJSON de sucesso
- Aplique os limites duais `max_command_chars` e `max_output_chars`
- Aborte no timeout com abort remoto best-effort
- O stdout e o stderr remotos do `exec` continuam crus por desenho

### Registro de hosts
- Gerencie hosts com `vps add`, `vps list`, `vps show`, `vps edit`, `vps remove`, `vps rename`, `vps path`, `vps doctor`, `vps export` e `vps import`
- Renomeie nativamente com `vps rename <NAME> <NEW_NAME>` ou `vps edit <NAME> --name <NEW>`, e toda credencial continua legível
- O rename emite `vps-renamed` com `previous_name` e `active_rewritten`
- Troque o método de autenticação em uma chamada: `vps edit --key` num host de senha, `--password-stdin` num host de chave ou `--use-agent`
- Cada método limpa os outros, e `updated` lista o método novo mais só os campos que tinham valor
- Duas flags de autenticação primária na mesma chamada saem 2, e uma passphrase sem chave sai 64 com o registro intacto
- Uma `--key` diferente descarta a passphrase antiga, salvo se a chamada trouxer uma nova
- `--agent-socket` sozinho implica `--use-agent`
- Remova um campo com as oito flags de limpeza `--no-key`, `--no-agent-socket`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key` e `--clear-tags`
- Aceite nomes de host só em ASCII `[A-Za-z0-9._-]`, começando por letra ou dígito, de 1 a 255 bytes
- A regra estrita vale em `vps add`, no destino de `vps rename` e em `vps import`
- `vps import` valida todos os itens, lista todos os nomes inválidos de uma vez, sai 64 e não grava nada, também sob `--dry-run`
- Um nome legado com caractere de controle ou bidi ainda carrega, e `vps rename '<legacy name>' <new>` o repara
- Duas tabelas do `config.toml` que normalizam para o mesmo nome NFC saem 65 com `config has two hosts named ...`
- `vps add --disable-sudo` e `vps edit --disable-sudo` persistem no registro do host, `vps edit --enable-sudo` desfaz, e `--disableSudo` é recusado nesses dois verbos
- Espere o lock do config até `--lock-wait-ms` (padrão 5000 ms, máximo 300000) e depois saia 69 em vez de pendurar
- Grave o wire schema v3 com chaves TOML em inglês e leia os aliases PT legados no carregamento

### Prévia com `--dry-run`
- Veja a prévia de oito operações: `vps remove`, `vps import`, `vps edit`, `vps rename`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt`
- Qualquer outro comando com `--dry-run` sai 64 em vez de ignorar a flag
- O plano de `vps edit`, `vps rename` e `vps remove` ecoa `target_resolved`, `target_source`, `host_resolved` e `host_source`
- Os três verbos saem 66 num config inexistente sem criar diretório nem lock

### Segurança da saída
- O modo texto escapa ESC, BEL, bidi e caracteres de formatação como `\u{1b}`, em mensagens, erros, `vps list`, `vps show`, `health-check`, `vps doctor` e saídas em lote
- `sftp ls` e `sftp stat` também escapam `\n` e `\t` em nomes remotos, então um nome hostil não forja linha de listagem
- O JSON mantém o valor original, porque é para máquina
- Reduza o payload antes da serialização com `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content` e `--max-output-bytes`
- As flags de coleção reduzem o array de `vps list`, o `results` dos envelopes em lote e o `entries` do `sftp ls`
- Um objeto único como `vps show`, um `exec` em um host ou `vps doctor` é reduzido por `--select` e `--truncate-content`, então `vps show prod --select name` devolve um campo só
- As flags de coleção (`--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--max-output-bytes`) não têm como valer para um registro só, e são nomeadas em `agent_shape.ignored` em vez de sumirem caladas
- Um envelope reduzido ganha `agent_shape` com `input_count`, `output_count` e `dropped`, e um registro único reduzido ganha `selected`, `content_truncated` e `ignored`
- Um array nu reduzido mantém o tipo e reporta o corte no stderr como `{"event":"agent-shape",…}`, que o `--quiet` suprime
- O filtro de tracing padrão é `error`, o que mantém o stderr limpo
- A verbosidade graduada `-v`, `-vv` e `-vvv` é sempre limitada ao crate (`warn,ssh_cli=*`), então o russh nunca despeja senhas
- O `RUST_LOG` do ambiente é ignorado

### Privado por padrão
- Um diretório de config novo nasce `0700`, e `vps doctor` informa `config_dir_permissions`, `config_dir_exposed` e `invalid_host_names` em `local`
- Um diretório XDG padrão já existente é apertado para `0700` quando não é symlink e pertence a você
- Qualquer outro diretório existente nunca recebe chmod: ele registra um aviso (visível com `-v`) e aparece no `doctor`
- `--config-dir algo.toml` usa o pai do caminho `.toml` como diretório de config para todos os arquivos
- Os diretórios criados por `vps export -o` nascem `0700`, e `known_hosts.lock` e `lang` nascem `0600`
- `tls mtls import --name` recusa caracteres de controle, formatação e bidi com exit 64 e não cria diretório, enquanto Unicode como `serviço-1` continua válido

### Transferências
- O Secure Copy Protocol (SCP) envia e baixa só arquivos regulares, sem diretórios recursivos
- As flags do SCP seguem o `exec`: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--json` (contrato `docs/schemas/scp-transfer.schema.json`)
- O sucesso JSON do SCP traz `event: "scp-transfer"`, e um arquivo remoto ausente sai 66 com `file not found: <path>`
- Um download SCP grava `{path}.ssh-cli.partial` e depois renomeia de forma atômica, e um `sync_data` com falha interrompe o rename
- O upload faz stream em blocos de 32 KiB
- A preservação de mtime e modo remotos é best-effort e reportada como `mtime_preserved` e `durable`
- O protocolo de transferência de arquivos do SSH (SFTP) usa o subsistema: `sftp upload`, `sftp download` (`--recursive` opcional, symlinks nunca seguidos), `sftp ls`, `sftp mkdir`, `sftp rmdir`, `sftp rm`, `sftp stat`, `sftp rename`
- Os eventos JSON do SFTP são `sftp-transfer`, `sftp-list`, `sftp-fs-op` e `sftp-batch`
- O upload SFTP preserva os bytes do payload e nunca trunca a zero
- `SFTP_PERM_MASK` (0o7777) vale no upload, e `SFTP_PERM_MASK_UNTRUSTED` (0o0777) remove bits setuid que o servidor envia no download
- O cancelamento de lote preenche o restante cancelado, então o tamanho de results coincide com a entrada

### Túneis e saúde
- Abra um túnel limitado com o `--timeout-ms` obrigatório e um `--bind` opcional (padrão `127.0.0.1`)
- Um `--bind` fora do loopback exige `--i-accept-network-exposure`
- `tunnel --json` emite `tunnel_listening` após o bind local e `tunnel_closed` no fim
- Um deadline pós-bind sai 0, enquanto um timeout pré-bind sai 74
- Um SIGINT sai 130 e um SIGTERM sai 143, e depois do bind o `tunnel_closed` traz `reason: signal`
- Use `--socks5` para um proxy SOCKS5, `--remote-socket PATH` para um socket Unix no host remoto e `--reverse` para um listener do lado do servidor
- Meça a latência com `health-check` e um `--timeout` opcional, para um host ou para a frota com `--all`

### Segredos e TLS
- Cifre segredos em repouso por padrão com ChaCha20-Poly1305 e um `secrets.key` XDG automático
- Escolha a chave com `--secrets-key-file`, `--use-keyring` ou o `secrets.key` XDG, e faça opt-out só em testes com `--allow-plaintext-secrets`
- Gerencie a master key com `secrets status`, `secrets init` e `secrets reencrypt`, com os eventos `secrets-init` e `secrets-reencrypt`
- A primeira gravação de segredo embute `secrets_key_auto_created: true` no mesmo JSON `vps-added`
- Passe segredos pelas flags de stdin para mantê-los fora do argv
- Mantenha o `known_hosts` TOFU e as gravações atômicas do config sob flock
- Use SSH-over-TLS, identidades de TLS mútuo (mTLS) e certificados DNS-01 do Automatic Certificate Management Environment (ACME) por `ssh-cli tls`

### Export e import
- O corpo de `vps export` segue o formato de saída resolvido: JSON sempre que o stdout não é TTY, mesmo quando `-o` nomeia um arquivo `.toml`
- Um corpo TOML exige `--output-format text`
- O export sai redigido por padrão: a senha vem como string vazia `""` e segredos ausentes como `null`, nunca como ciphertext
- A máscara fixa de três asteriscos `FIXED_MASK` existe só em `vps list` e `vps show`
- `--include-secrets` em pipe ou non-TTY exige `-o` ou `--i-understand-secrets-on-stdout`
- Um host só de chave mostra senha vazia como JSON `null` em `vps list` e `vps show`
- `vps import` aceita TOML (chaves EN mais aliases PT) ou um envelope JSON `vps-export`, e esqueletos mascarados exigem `--allow-incomplete`
- Um host exportado com nome legado fora do charset estrito só restaura depois de `vps rename`

### Descoberta
- Imprima todos os contratos com `ssh-cli schema` (25 JSON Schemas embarcados) e a árvore completa de comandos com `ssh-cli commands`
- Complete o shell com `ssh-cli completions` para bash, zsh, fish, powershell e elvish
- Escolha o idioma da UI entre `en-US` e `pt-BR` automaticamente


## Início rápido
### Instale e rode o primeiro comando remoto

```bash
cargo install ssh-cli --locked
printf %s 'demo-password-not-real' | ssh-cli vps add --name prod --host prod.example.com --port 22 --user admin --password-stdin
ssh-cli exec prod "hostname" --json
```

### Saída de exemplo do segundo comando (os caminhos variam)

```json
{"ok":true,"event":"vps-added","key_file":"~/.config/ssh-cli/secrets.key","key_source":"xdg_file","name":"prod","secrets_key_auto_created":true}
```


## Instalação
### Escolha o caminho de instalação do seu ambiente
- Prefira o crates.io com lockfile: `cargo install ssh-cli --locked`
- Atualize depois de um release com `cargo install ssh-cli --locked --force`
- Recompile a partir do checkout com `cargo install --path . --locked`
- Não instale sem `--locked` salvo se você validou que os pins de crypto resolvem limpos
- Compile musl com a feature de alocador no Alpine: `--features musl-allocator`
- Exija Rust `1.89` ou superior (MSRV, declarado como `rust-version` no `Cargo.toml`)

### Completions de shell
- Bash: `ssh-cli completions bash > ssh-cli.bash` e carregue o arquivo no seu profile
- Zsh: `ssh-cli completions zsh > _ssh-cli` e coloque em um diretório do seu `fpath`
- Fish: `ssh-cli completions fish > ~/.config/fish/completions/ssh-cli.fish`
- PowerShell: `ssh-cli completions powershell > ssh-cli.ps1` e carregue com dot-source no seu profile
- Elvish: `ssh-cli completions elvish > ssh-cli.elv`

### Idiomas
- A UI fala `en` e `pt-BR`
- A ordem de resolução é `--lang`, depois o arquivo XDG `lang` gravado por `ssh-cli locale set <code>`, depois o locale do sistema, depois `en`
- Os corpos de erro ficam em inglês por desenho, para o agente receber texto estável


## Features
### Flags de feature do Cargo

| Feature | Padrão | Descrição |
|---|---|---|
| `ssh-real` | sim | SSH real via `russh` + `aws-lc-rs` (compressão só `none`) |
| `tls` | sim | rustls + `aws_lc_rs`: SSH-over-TLS, mTLS, ACME, com o `Cargo.toml` pedindo 0.23.25 e o piso de segurança que o `tls provider` imprime como `min_rustls` em 0.23.18 |
| `musl-allocator` | não | `mimalloc` como global allocator (binário, útil em musl e Alpine) |

- O caminho de instalação sempre ativa os padrões (`ssh-real` e `tls`)
- Desative SSH real e TLS só para diagnóstico de dependências: `cargo build --no-default-features`
- O docs.rs compila com `all-features = true` e `--cfg docsrs` (veja `[package.metadata.docs.rs]` no `Cargo.toml`)

### Política de crypto
- O transporte padrão é SSH-2 em TCP puro (`russh` + aws-lc-rs)
- As host keys usam TOFU sob XDG (`known_hosts`)
- O SSH-over-TLS opcional (`vps add --tls`) faz um handshake rustls e depois SSH no stream TLS
- O binário instala o provider `aws_lc_rs` (`CryptoProvider`) uma vez no `main`
- O material mTLS e ACME fica sob o XDG `tls/`, sem env de produto para armazenar certificados
- Sem OpenSSL, sem `native-tls` e sem provider `ring` duplo
- A compressão do canal SSH é forçada a `none`
- Erros de validação ACME, `invalidContact` e problemas 4xx saem 64 e não são retryable, enquanto rate limits continuam transitórios com exit 74
- Detalhes em [SECURITY.pt-BR.md](SECURITY.pt-BR.md)


## Targets
### Plataformas cobertas pelo metadata do docs.rs
- `x86_64-unknown-linux-gnu` (padrão)
- `x86_64-apple-darwin` e `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `aarch64-unknown-linux-musl`
- Windows e macOS passam só por type-check, e o modo `0700` e o `getuid` não foram executados lá
- O Windows não tem chmod, então o diretório de config depende da ACL do perfil
- Veja [docs/CROSS_PLATFORM.pt-BR.md](docs/CROSS_PLATFORM.pt-BR.md) para notas de runtime


## Uso
### Cadastre hosts
- Prefira `--password-stdin` ou `--key` a segredos no argv
- Adicione um host de senha com `vps add --password-stdin` (ou `--password`)
- Adicione um host de chave com `vps add --key ~/.ssh/id_ed25519`
- Nomeie hosts com letras ASCII, dígitos, `.`, `_` e `-`, começando por letra ou dígito
- Marque o host ativo com `connect <name>`, que só grava um marcador e não abre sessão

### Altere hosts
- Troque de senha para chave em uma chamada com `vps edit prod --key ~/.ssh/id_ed25519`
- Troque de chave para senha com `vps edit prod --password-stdin`
- Renomeie com `vps rename prod prod-old`, onde repetir depois do sucesso sai 66 e `vps show prod-old` confirma
- Veja a prévia de qualquer uma com `--dry-run` e leia `target_resolved` antes da chamada real
- Repare um nome legado com `vps rename '<legacy name>' <new>`

### Execute
- Rode um comando de shell remoto com `exec <vps> "<cmd>"`
- Alcance a frota inteira com `exec --all '<cmd>'`, um subconjunto com `--hosts a,b` ou um rótulo com `--tags prod`
- Alcance o marcador do `connect` só de propósito com `exec --use-active '<cmd>'`, que sai 66 quando não há marcador
- Eleve com `sudo-exec` ou `su-exec` quando configurado
- Desative a elevação em uma chamada com `--disable-sudo`, ou reative num host com `vps edit --enable-sudo`

### Inspecione
- Diagnostique caminhos e o estado local com `doctor --json` (ou `vps doctor --json`)
- Leia `local.config_dir_exposed`, `local.config_dir_permissions`, `local.invalid_host_names` e `local.active_vps` desse objeto
- Sonde hosts por SSH com `vps doctor --probe-ssh` e um subconjunto em `--hosts`
- Descubra contratos com `ssh-cli schema` e `ssh-cli commands`
- Exporte o inventário com `vps export`, que grava um envelope JSON em qualquer stdout non-TTY


## Comandos
### Conjuntos de flags compartilhados
- Auth set: `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--use-agent`, `--agent-socket`
- Seletores de frota: `--all`, `--hosts <LIST>`, mais `--tags <LIST>` e `--use-active` onde a tabela os mostra
- Cada linha lista a folha com suas flags locais, e as flags globais estão nas próximas seções

### Grupos de comandos

| Grupo | Propósito | Folhas |
|---|---|---|
| `ssh-cli vps` | Gerencia os hosts VPS cadastrados | `add`, `list`, `remove`, `rename`, `edit`, `show`, `path`, `doctor`, `export`, `import` |
| `ssh-cli scp` | Transferência de arquivo por SCP | `upload`, `download` |
| `ssh-cli sftp` | Transferência SFTP e operações no sistema de arquivos remoto | `upload`, `download`, `ls`, `mkdir`, `rmdir`, `rm`, `stat`, `rename` |
| `ssh-cli secrets` | Gerencia a chave primária e a cifragem de segredos em repouso | `status`, `init`, `reencrypt` |
| `ssh-cli locale` | Diagnostica e gerencia o idioma da UI | `show`, `set`, `clear` |
| `ssh-cli tls` | Pilha TLS: status do provider, identidades mTLS e certificados ACME | `provider`, `paths`, `mtls`, `acme` |
| `ssh-cli tls mtls` | Gerencia identidades de cliente mTLS importadas sob o XDG `tls/mtls/` | `list`, `import`, `show`, `remove` |
| `ssh-cli tls acme` | Ciclo de vida da conta ACME e dos certificados DNS-01 | `account`, `issue`, `complete`, `status`, `list` |
| `ssh-cli tls acme account` | Gerencia a conta ACME | `create`, `show` |

### Registro de hosts

| Comando | Propósito | Flags locais |
|---|---|---|
| `ssh-cli vps add` | Cadastra um host | `--name`, `--host`, `--port`, `--user`, auth set, `--timeout`, `--max-command-chars`, `--max-output-chars`, `--max-chars` (alias legado de `--max-command-chars`), `--sudo-password`, `--sudo-password-stdin`, `--su-password`, `--su-password-stdin`, `--disable-sudo`, `--tag`, `--tls`, `--tls-sni`, `--tls-client-cert`, `--tls-client-key`, `--check` |
| `ssh-cli vps list` | Lista hosts com segredos mascarados | `--tag` |
| `ssh-cli vps show` | Mostra um host mascarado | `<NAME>` |
| `ssh-cli vps edit` | Altera campos, troca o método de autenticação e renomeia | `<NAME>`, `--name`, `--host`, `--port`, `--user`, auth set, `--timeout`, `--max-command-chars`, `--max-output-chars`, `--max-chars`, `--sudo-password`, `--sudo-password-stdin`, `--su-password`, `--su-password-stdin`, `--disable-sudo`, `--enable-sudo`, `--tls`, `--no-tls`, `--tls-sni`, `--tls-client-cert`, `--tls-client-key`, `--tag`, `--clear-tags`, `--no-key`, `--no-agent-socket`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key` |
| `ssh-cli vps remove` | Apaga um host | `<NAME>` |
| `ssh-cli vps rename` | Renomeia um host mantendo toda credencial | `<NAME> <NEW_NAME>` |
| `ssh-cli vps path` | Imprime o caminho do `config.toml` | nenhuma |
| `ssh-cli vps doctor` | Mostra camadas XDG, schema, caminhos e estado local | `--probe-ssh`, `--hosts` |
| `ssh-cli vps export` | Exporta hosts, mascarados por padrão | `--include-secrets`, `--output` (`-o`), `--i-understand-secrets-on-stdout` |
| `ssh-cli vps import` | Importa TOML ou um envelope JSON `vps-export` | `--file`, `--allow-incomplete` |

### Execução e sondas

| Comando | Propósito | Flags locais |
|---|---|---|
| `ssh-cli connect` | Grava o marcador irmão `active`, que não abre sessão | `<NAME>` |
| `ssh-cli exec` | Comando remoto one-shot, JSON `exec-batch` com seletor | `<VPS> <COMMAND>`, `--all`, `--hosts`, `--tags`, `--use-active`, `--step`, auth set, `--timeout`, `--description` |
| `ssh-cli sudo-exec` | `sudo` one-shot com empacotamento seguro `sh -c` | `<VPS> <COMMAND>`, `--all`, `--hosts`, `--tags`, `--use-active`, `--step`, auth set, `--sudo-password`, `--sudo-password-stdin`, `--timeout`, `--description` |
| `ssh-cli su-exec` | Elevação `su -` one-shot | `<VPS> <COMMAND>`, `--all`, `--hosts`, `--tags`, `--use-active`, `--step`, auth set, `--su-password`, `--su-password-stdin`, `--timeout`, `--description` |
| `ssh-cli health-check` | Sonda de conectividade, JSON `health-check-batch` para a frota | `<VPS_NAME>`, `--all`, `--hosts`, `--use-active`, auth set, `--timeout` |
| `ssh-cli tunnel` | Port-forward limitado com deadline obrigatório | `<VPS_NAME> <LOCAL_PORT> [REMOTE_HOST] [REMOTE_PORT]`, `--timeout-ms`, `--bind`, `--i-accept-network-exposure`, `--socks5`, `--remote-socket`, `--reverse`, auth set |

### Transferências

| Comando | Propósito | Flags locais |
|---|---|---|
| `ssh-cli scp upload` | Envia um arquivo regular, JSON `scp-transfer` | `--src`, `--dest`, `<VPS>`, `--all`, `--hosts`, auth set, `--timeout` |
| `ssh-cli scp download` | Baixa um arquivo regular, JSON `scp-transfer` | `--src`, `--dest`, `<VPS>`, `--all`, `--hosts`, auth set, `--timeout` |
| `ssh-cli sftp upload` | Envia um arquivo ou árvore por SFTP | `--src`, `--dest`, `<VPS>`, `--all`, `--hosts`, `--recursive`, auth set, `--timeout` |
| `ssh-cli sftp download` | Baixa um arquivo ou árvore por SFTP | `--src`, `--dest`, `<VPS>`, `--all`, `--hosts`, `--recursive`, auth set, `--timeout` |
| `ssh-cli sftp ls` | Lista um diretório remoto, JSON `sftp-list` | `<VPS_NAME> <REMOTE>`, auth set, `--timeout` |
| `ssh-cli sftp mkdir` | Cria um diretório remoto | `<VPS_NAME> <REMOTE>`, auth set, `--timeout` |
| `ssh-cli sftp rmdir` | Remove um diretório remoto vazio, prévia sob `--dry-run` | `<VPS_NAME> <REMOTE>`, auth set, `--timeout` |
| `ssh-cli sftp rm` | Remove um arquivo remoto, prévia sob `--dry-run` | `<VPS_NAME> <REMOTE>`, auth set, `--timeout` |
| `ssh-cli sftp stat` | Mostra metadados de um caminho remoto | `<VPS_NAME> <REMOTE>`, auth set, `--timeout` |
| `ssh-cli sftp rename` | Renomeia um caminho remoto | `<VPS_NAME> <FROM> <TO>`, auth set, `--timeout` |

### Segredos e TLS

| Comando | Propósito | Flags locais |
|---|---|---|
| `ssh-cli secrets status` | Mostra o status da cifragem, nunca a chave | nenhuma |
| `ssh-cli secrets init` | Gera e guarda a chave primária, prévia sob `--dry-run` | `--keyring`, `--force` |
| `ssh-cli secrets reencrypt` | Recifra o `config.toml` com a chave atual, prévia sob `--dry-run` | nenhuma |
| `ssh-cli tls provider` | Mostra o status do `CryptoProvider` rustls (`aws_lc_rs`) | nenhuma |
| `ssh-cli tls paths` | Imprime o layout de diretórios TLS do XDG | nenhuma |
| `ssh-cli tls mtls list` | Lista os nomes de identidade importados | nenhuma |
| `ssh-cli tls mtls import` | Importa cert e chave PEM como identidade nomeada | `--name`, `--cert`, `--key` |
| `ssh-cli tls mtls show` | Mostra os caminhos de uma identidade | `<NAME>` |
| `ssh-cli tls mtls remove` | Remove o diretório de uma identidade | `<NAME>` |
| `ssh-cli tls acme account create` | Cria uma conta ACME em `tls/acme/account.json` do XDG | `--staging`, `--contact`, `--force` |
| `ssh-cli tls acme account show` | Mostra se existe conta e o caminho dela | nenhuma |
| `ssh-cli tls acme issue` | Inicia um pedido DNS-01 e imprime o desafio TXT | `--domain`, `--staging`, `--print-challenge` |
| `ssh-cli tls acme complete` | Conclui o pedido depois de publicar o TXT no DNS | `--domain` |
| `ssh-cli tls acme status` | Mostra o status do certificado ou pendência | `--domain` |
| `ssh-cli tls acme list` | Lista os diretórios de domínio ACME | nenhuma |

### Descoberta e idioma

| Comando | Propósito | Flags locais |
|---|---|---|
| `ssh-cli completions` | Gera completions de shell | `<SHELL>` |
| `ssh-cli commands` | Emite a árvore completa de comandos em JSON | nenhuma |
| `ssh-cli schema` | Emite o catálogo de schemas ou o corpo de um schema | `<NAME>` |
| `ssh-cli doctor` | Alias raiz de `vps doctor` | `--probe-ssh`, `--hosts` |
| `ssh-cli locale show` | Mostra o idioma resolvido e a camada vencedora | nenhuma |
| `ssh-cli locale set` | Persiste o idioma no arquivo `lang` (`0600`) | `<LOCALE>` |
| `ssh-cli locale clear` | Remove a preferência persistida | nenhuma |

### Flags globais
- Toda flag global funciona antes ou depois do subcomando

| Flag | Propósito |
|---|---|
| `--lang <LOCALE>` | Força o idioma da UI (`en` ou `pt-BR`) |
| `-v`, `--verbose` | Verbosidade de log no stderr: `-v` info, `-vv` debug, `-vvv` trace |
| `-q`, `--quiet` | Suprime a saída que não é JSON |
| `--config-dir <DIR>` | Substitui o diretório de configuração |
| `--no-color` | Desativa a saída colorida |
| `--output-format <FORMAT>` | `json` ou `text` (JSON quando o stdout não é TTY) |
| `--json` | Força JSON no stdout |
| `--select <PATHS>` | Mantém só esses caminhos pontuados em cada registro, com o alias `--fields` |
| `--filter <EXPR>` | Mantém registros que casam `key=value`, `key!=value` ou `key~substring` |
| `--limit <N>` | Emite no máximo N registros |
| `--sort <PATH>` | Ordena os registros de forma crescente pelo caminho pontuado |
| `--dedupe-by <PATH>` | Descarta registros posteriores que repetem o valor do caminho |
| `--count-only` | Troca os registros por `{"count": N}` |
| `--truncate-content <CHARS>` | Encurta strings maiores que N caracteres |
| `--max-output-bytes <BYTES>` | Limita o tamanho do envelope descartando registros finais |
| `--no-input` | Recusa ler o stdin |
| `--dry-run` | Imprime o plano de uma operação destrutiva e sai sem executar |
| `--lock-wait-ms <MS>` | Espera o lock do config, padrão 5000, e depois sai 69 |
| `--disable-sudo` | Desativa `sudo-exec` e `su-exec` só nesta chamada, com o alias `--disableSudo` aceito onde a flag global é aceita, e antes de `vps add` ou `vps edit` também grava `disable_sudo: true` no registro do host |
| `--replace-host-key` | Substitui uma host key TOFU divergente |
| `--allow-plaintext-secrets` | Permite segredos em texto puro em repouso, só para testes |
| `--secrets-key-file <PATH>` | Usa um arquivo de chave primária de 64 hex nesta chamada |
| `--use-keyring` | Prefere o keyring do SO para a chave primária |
| `--timeout <MS>` | Timeout padrão das operações SSH, e um `--timeout` local vence |
| `--max-concurrency <N>` | Limita sessões multi-host e forwards de túnel, de 1 a 64 |
| `--fail-fast` | Para de admitir novas unidades multi-host após a primeira falha |
| `--scp-file-concurrency <N>` | Transferências SCP concorrentes em uma sessão, padrão 1 |

### Modos de túnel
- `ssh-cli tunnel <vps> <port> <host> <rport> --timeout-ms N` encaminha uma porta local, e accepts concorrentes são limitados por `--max-concurrency`
- `ssh-cli tunnel <vps> <port> --socks5 --timeout-ms N` serve SOCKS5 (RFC 1928, `CONNECT` sem autenticação), resolve nomes do lado remoto e responde `BIND` e `UDP ASSOCIATE` com `0x07`
- `ssh-cli tunnel <vps> <port> --remote-socket PATH --timeout-ms N` encaminha para um socket Unix, e `PATH` precisa ser absoluto
- `ssh-cli tunnel <vps> <port> <bind> <rport> --reverse --timeout-ms N` faz o servidor escutar em `<bind>:<rport>` e entregar de volta na `<port>` local
- Com `--reverse`, `<rport> 0` deixa o servidor alocar a porta e informar em `local_port`


## Variáveis de ambiente
### Os knobs do produto são flags e arquivos XDG, nunca stores `SSH_CLI_*`

| Variável | Comportamento | Exemplo |
|---|---|---|
| `SSH_CLI_SECRETS_KEY`, `SSH_CLI_SECRETS_KEY_FILE` | Fail-closed: um comando que resolve a primary-key sai 64 quando qualquer uma está presente | `SSH_CLI_SECRETS_KEY=x ssh-cli secrets status` |
| `SSH_CLI_HOME`, `SSH_CLI_LANG`, `SSH_CLI_FORCE_TEXT`, `SSH_CLI_USE_KEYRING` | Nunca lidas como stores de config | `ssh-cli --config-dir ./c vps list` |
| `RUST_LOG` | Ignorada pelo produto | `ssh-cli -vv exec h id --json` |
| `HOME` | Home do SO para resolver o caminho XDG | `ssh-cli --config-dir ./c vps path` |
| `USERPROFILE` | Home de reserva para expandir `~` quando `HOME` falta, como no Windows | `ssh-cli vps add --name w --host h --user u --key ~/.ssh/id_ed25519` |
| `TERM`, `NO_COLOR`, `CLICOLOR_FORCE` | Só detecção de terminal e cor | `ssh-cli --no-color vps list` |
| `CI`, `FLATPAK_ID`, `SNAP`, `WSL_DISTRO_NAME`, `KUBERNETES_SERVICE_HOST`, `container`, `TERMUX_VERSION`, `TERMUX_APK_RELEASE`, `PREFIX` | Só detecção de runtime, exibida como `local.runtime.*` no `vps doctor` | `ssh-cli vps doctor --json` |

- Nunca ponha senhas de host em variáveis de ambiente
- Use o registro e as flags de stdin

### Precedência de configuração

| Controle | Como | Exemplo |
|---|---|---|
| Diretório de config | `--config-dir`, senão o padrão XDG | `ssh-cli --config-dir /tmp/ssh-cli-test vps list` |
| Idioma | `--lang`, depois `ssh-cli locale set <code>`, depois o sistema, depois `en` | `ssh-cli --lang pt-BR vps list` |
| Formato de saída | `--json` ou `--output-format json\|text` | `ssh-cli exec h uptime --json` |
| Concorrência | `--max-concurrency N` (1 a 64, fórmula automática quando omitido) | `ssh-cli --max-concurrency 8 exec --all id --json` |
| Verbosidade | `-v`, `-vv`, `-vvv` (info, debug, trace, limitado ao crate) | `ssh-cli -vv exec h id --json` |
| Chave primária | `--secrets-key-file`, `--use-keyring` ou o `secrets.key` XDG | `ssh-cli --secrets-key-file ./k secrets status` |
| Opt-out de plaintext | `--allow-plaintext-secrets` (só testes) | `ssh-cli --allow-plaintext-secrets vps list` |


## Padrões de integração
### Conecte agentes só com subprocessos one-shot
- Faça parse de um documento por chamada: `ssh-cli exec prod "uptime" --json | jaq -r .stdout`
- Distribua por um rótulo: `ssh-cli exec --tags prod "id" --json | jaq -c '.results'`
- Reduza o payload antes de ele existir: `ssh-cli --select name,host vps list --json`
- Mantenha segredos fora do argv: `printf %s "$PASS" | ssh-cli exec prod "hostname" --password-stdin --json`
- Ensaie uma mutação e depois rode: `ssh-cli --dry-run vps rename prod prod-old` e depois `ssh-cli vps rename prod prod-old`
- Ramifique pelo exit code antes de fazer parse do stdout, e leia `retryable` no envelope de erro JSON que vai para o stderr
- O exit 2 do parser e o exit 64 de um `--filter` malformado imprimem texto puro no stderr mesmo com `--json`, então só erros de execução trazem o envelope
- Leia [INTEGRATIONS.pt-BR.md](INTEGRATIONS.pt-BR.md) para notas por agente


## Exit codes
### Códigos no estilo sysexits que agentes devem mapear antes de retry

| Código | Significado |
|---|---|
| `0` | Sucesso |
| `1` | Erro genérico de runtime, falha de comando remoto (o JSON do `exec` fica no stdout com `exit_code`, e o stderr traz `command_failed` com `remote_exit_code`) ou falha parcial multi-host |
| `2` | Erro de uso do parser, como duas flags de autenticação primária na mesma chamada |
| `64` | Uso ou argumentos inválidos, incluindo nomes de host inválidos e as variáveis `SSH_CLI_SECRETS_KEY` e `SSH_CLI_SECRETS_KEY_FILE` |
| `65` | Erro de dados em JSON, TOML, schema ou config |
| `66` | VPS ou arquivo de entrada não encontrado |
| `69` | Serviço obrigatório indisponível, como um keyring do SO travado ou a espera do lock do config esgotada |
| `70` | Falha interna de software sem nada que o chamador possa corrigir |
| `73` | Não foi possível criar o config ou a saída |
| `74` | Erro de IO ou conexão SSH ou timeout |
| `77` | Autenticação rejeitada, host key mudou, política de sudo ou falha ao decifrar segredo |
| `130` | SIGINT |
| `141` | Broken pipe no stdout ou stderr |
| `143` | SIGTERM |

- Prefira `--json`, que é automático quando o stdout não é TTY (`--output-format` sobrescreve)
- Faça retry em `74` e `69`, que o envelope de erro JSON marca com `retryable: true`
- Nunca faça retry em auth (`77`), uso (`64`, `2`), dados (`65`) ou não encontrado (`66`)
- O tracing padrão é `error`, então o tratamento de exit e o JSON em stdout ficam sem ruído INFO
- O `ssh-cli --help` já termina com um bloco `Examples:` e uma tabela `Exit status:`, e as 48 folhas mostram `Examples:` no próprio `--help`


## Performance
### Cold start e memória
- A meta de cold start é abaixo de 100 ms em hosts Linux modernos
- Medido em um host Linux, `ssh-cli vps list --json` num registro vazio rodou em menos de 0,01 s com cerca de 8 MB de memória de pico
- Reproduza com `/usr/bin/time -f '%e s %M KB' ssh-cli vps list --json`
- Morra após cada comando para devolver a RAM ao SO imediatamente
- Evite túneis de longa duração, já que `--timeout-ms` é obrigatório


## Requisitos de memória
### Planeje a capacidade para registros multi-host
- O TOML de config cresce com a contagem de hosts e o tamanho dos caminhos
- Os buffers de saída respeitam `max_output_chars` por stream
- O arquivo `known_hosts` cresce devagar com pares únicos de host e porta
- O fan-out é limitado por `--max-concurrency`, e o SCP usa um canal salvo se `--scp-file-concurrency` for aumentado
- Nenhum modelo de embedding e nenhum heap Node são necessários


## FAQ de troubleshooting
### A instalação falha ou o build diverge
- Rode de novo com `--locked` ou use o release 0.5.6 ou superior, que fixa o russh 0.65.1 (veja `scripts/verify_install_resolve.py`)
- Exija Rust 1.89 ou superior
- Atualize da 0.3.9 para ter a correção de wire do SCP, que existe desde a 0.4.0
- Atualize da 0.4.0 para ter o export honesto de segredos vazios e a saída pós-bind do túnel, ambos desde a 0.4.1
- Atualize para a 0.5.0 ou superior para a API EN renomeada, e para a 0.5.3 ou superior para a correção de integridade do SFTP

### A autenticação falha (exit 77)
- Num host só de chave, defina `--key` no `vps add` ou passe `--key` ou `--password-stdin` ao `exec`
- Com chave que tem passphrase, use `--key-passphrase-stdin`
- Quando a host key mudou, confirme que é legítima e rode de novo com `--replace-host-key`
- Quando o `sudo-exec` está desativado, remova `--disable-sudo` e rode `vps edit <name> --enable-sudo`

### Erros de uso (exit 64 ou 2)
- Um nome é recusado: use só letras ASCII, dígitos, `.`, `_` e `-`, começando por letra ou dígito
- Um posicional solitário no `exec` é recusado: adicione o host, um seletor ou `--use-active`
- Duas flags de autenticação num `vps edit` saem 2: passe um método por chamada
- Um comando é rejeitado por tamanho: aumente `max_command_chars` ou encurte o comando
- `--dry-run` num comando sem suporte sai 64: use só nas oito operações com prévia

### Config e segredos
- Segredos cifrados sem chave: rode `ssh-cli secrets init`, restaure o `secrets.key` ou passe `--secrets-key-file`
- A chamada sai 69 no lock do config: repita ou aumente `--lock-wait-ms`
- `vps doctor` mostra `config_dir_exposed: true`: aperte o diretório você mesmo com `chmod 700`, porque a CLI nunca altera um diretório que não é o padrão dela
- Um nome de host legado aparece em `invalid_host_names`: repare com `vps rename '<legacy name>' <new>`
- Um config sai 65 com dois hosts de mesmo nome: renomeie uma tabela no `config.toml`
- Um import de TOML sai 65: corrija o erro de parse, e passe `--allow-incomplete` para um esqueleto mascarado
- Permissão negada no config: garanta `chmod 600` no `config.toml` e no `secrets.key`

### Transferências, túneis e logs
- O SCP move só arquivos regulares, então use `sftp upload --recursive` para árvores
- Um remoto SCP ausente sai 66 com `file not found: <path>`
- Um download SCP que falha deixa o destino ausente ou o arquivo anterior intacto, e o arquivo parcial é `.ssh-cli.partial`
- Um túnel que sai 74 depois de `tunnel_listening` rodava na 0.4.0, e o deadline pós-bind sai 0 desde a 0.4.1
- Ruído de stderr em pipelines JSON não existe por padrão, porque o nível é `error`
- Passe `-v`, `-vv` ou `-vvv` para diagnosticar sem vazar senhas, e `RUST_LOG=debug` não tem efeito
- No macOS, o Gatekeeper bloqueia o binário: rode `xattr -d com.apple.quarantine /path/to/ssh-cli`


## Documentação
### Onde ler em seguida
- Siga o primeiro uso em [docs/HOW_TO_USE.pt-BR.md](docs/HOW_TO_USE.pt-BR.md)
- Copie receitas de [docs/COOKBOOK.pt-BR.md](docs/COOKBOOK.pt-BR.md)
- Integre agentes via [docs/AGENTS.pt-BR.md](docs/AGENTS.pt-BR.md) e [INTEGRATIONS.pt-BR.md](INTEGRATIONS.pt-BR.md)
- Confira plataformas em [docs/CROSS_PLATFORM.pt-BR.md](docs/CROSS_PLATFORM.pt-BR.md)
- Migre da 0.3.3 ou superior em [docs/MIGRATION.pt-BR.md](docs/MIGRATION.pt-BR.md) (linha alvo 0.5.6)
- Execute os testes via [docs/TESTING.pt-BR.md](docs/TESTING.pt-BR.md)
- Consuma os contratos JSON em [docs/schemas/README.md](docs/schemas/README.md)
- Ensine LLMs com [skills/ssh-cli-pt/SKILL.md](skills/ssh-cli-pt/SKILL.md)


## Contribuindo
- Leia [CONTRIBUTING.pt-BR.md](CONTRIBUTING.pt-BR.md) antes de abrir um pull request
- Siga o framework bilíngue de documentação em toda mudança de doc pública
- Siga o [Código de Conduta](CODE_OF_CONDUCT.pt-BR.md)


## Segurança
- Leia [SECURITY.pt-BR.md](SECURITY.pt-BR.md) para reporte privado de vulnerabilidades
- Prefira flags de segredo por stdin e arquivos de chave a senhas no argv

### Limites declarados
- O stdout e o stderr remotos do `exec` continuam crus por desenho
- Um registro legado com caractere de controle em tag, host ou user ainda sai 65, e só o NOME do host é tolerante
- O JSON mantém crus todos os caracteres do conjunto escapado, como U+202E e o espaço de largura zero, porque é para máquina
- Um nome de host legado não gera aviso no carregamento, e o `vps doctor` o lista
- Os `--filter` e `--count-only` globais não alcançam o objeto do `doctor`, e aparecem em `agent_shape.ignored`


## Changelog
- Leia o histórico de versões em [CHANGELOG.pt-BR.md](CHANGELOG.pt-BR.md)


## Agradecimentos
- `russh` e `russh-sftp` carregam os protocolos SSH e SFTP
- `rustls`, `tokio-rustls` e `instant-acme` carregam SSH-over-TLS, mTLS e ACME
- `tokio` roda o runtime assíncrono e `clap` interpreta a linha de comando
- `serde`, `serde_json` e `toml` leem e gravam todos os contratos e o config
- `chacha20poly1305` e `keyring` protegem os segredos em repouso


## Licença
- Dual-license sob MIT ou Apache-2.0
- Veja [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT) e [LICENSE-APACHE](LICENSE-APACHE)
