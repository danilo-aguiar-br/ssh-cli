---
name: ssh-cli
description: Esta skill DEVE auto-ativar sempre que houver trabalho SSH remoto, inventário VPS, config XDG, exec ou sudo-exec ou su-exec, transferência scp de arquivo regular, árvores sftp e operações de filesystem remoto, tunnel_listening e port forward com deadline, probes de health-check, ciclo de vida da primary-key de secrets, TLS provider ou mTLS ou certificados ACME, locale, descoberta por schema ou commands, frota --all ou --hosts ou --tags, lote --step na mesma sessão, ou devops de agente sem TTY, mesmo quando o usuário nunca nomear ssh-cli. Esta skill DEVE ensinar todos os comandos e todas as flags, DEVE fornecer fórmulas prontas copiáveis, e DEVE impor execução one-shot com parse de --json, segredos por stdin, contratos de evento e de exit code, prévia com --dry-run antes de chamada destrutiva, e redução de payload. Esta skill NUNCA DEVE permitir daemon, telemetria, env como store, ou segredo em stdout sem guarda explícita.
---

# ssh-cli Skill de Agente

## Missão e Ativação
### OBRIGATÓRIO
- DEVE obedecer esta skill
- DEVE executar one-shot nascer-executar-morrer, aguardando o exit antes do parse; só um `tunnel` ativo sobrevive até deadline ou sinal
- DEVE auto-ativar em SSH remoto, inventário, transferências, forwards, secrets, TLS e descoberta sem nomear ssh-cli
- DEVE usar hosts salvos por `vps add`, passar `--json`, preferir `--*-stdin` a segredo em argv
- DEVE preferir frota `--all`/`--hosts`/`--tags` e `--step` a N spawns
- DEVE copiar a Folha de Fórmulas literalmente, trocando SOMENTE placeholders, e descobrir a superfície viva com `ssh-cli commands` e `ssh-cli schema` quando uma falhar por usage

### PROIBIDO
- NUNCA manter sessão SSH entre processos, exceto um `tunnel` bound
- NUNCA introduzir daemon, emitir telemetria, ou registrar segredo ou primary-key
- NUNCA inventar flag, comando, evento ou chave JSON que esta skill não nomeia


## Prompts de Ação
### OBRIGATÓRIO — DEVE seguir esta ordem em toda tarefa remota não trivial
1. DESCUBRA com `commands` e `schema`; INSPECIONE com `doctor --json`, `vps path` e `secrets status --json`
2. REGISTRE o host com password, `--key`, ou `--use-agent` mais `--agent-socket`; anexe `--tag` e as flags TLS
3. TESTE com `health-check <NAME> --json`, ou frota `--all`/`--hosts`, NUNCA `--tags` aqui
4. EXECUTE com `exec`/`sudo-exec`/`su-exec` e `--json`; agrupe na mesma sessão com `--step`
5. TRANSFIRA arquivo regular com `scp`, árvores e filesystem com `sftp`, NUNCA `--tags` neles
6. ENCAMINHE só com `tunnel`, sempre com `--timeout-ms`
7. PREVEJA toda chamada destrutiva sem supervisão com `--dry-run`
8. PARSEIE o exit do processo, depois o sucesso em stdout, depois o envelope de erro em stderr
9. SANITIZE logs duráveis para que nenhum segredo e nenhuma primary-key permaneça


## Catálogo de Comandos
### OBRIGATÓRIO — DEVE tratar estas 47 folhas como a superfície inteira
- `vps add`, `vps list`, `vps show`, `vps edit`, `vps remove`, `vps path`, `vps export`, `vps import` — inventário, sempre mascarado
- `vps doctor`, `doctor` — diagnóstico XDG e de schema; `doctor` é o ALIAS de raiz
- `connect` — grava SOMENTE o marcador de host ativo, NUNCA uma sessão
- `exec`, `sudo-exec`, `su-exec` — um comando remoto; elevação é one-shot com packing `sh -c` seguro
- `scp upload`, `scp download` — SOMENTE arquivo regular
- `sftp upload`, `sftp download`, `sftp ls`, `sftp mkdir`, `sftp rmdir`, `sftp rm`, `sftp stat`, `sftp rename` — árvores e filesystem remoto
- `tunnel` — forward com deadline, em quatro modos
- `health-check` — probe de conectividade
- `secrets status`, `secrets init`, `secrets reencrypt` — ciclo de vida da primary-key
- `commands`, `schema`, `completions` — árvore de comandos, catálogo de schemas, completions de shell
- `locale show`, `locale set`, `locale clear` — idioma da interface
- `tls provider`, `tls paths` — status do CryptoProvider e layout XDG de TLS
- `tls mtls list`, `tls mtls import`, `tls mtls show`, `tls mtls remove` — identidades de cliente
- `tls acme account create`, `tls acme account show`, `tls acme issue`, `tls acme complete`, `tls acme status`, `tls acme list` — conta e ciclo DNS-01


## Flags Globais
### OBRIGATÓRIO — DEVE colocar estas ANTES do subcomando
- `--json`, `-q`/`--quiet`, `--no-color`, `--config-dir <DIR>` — `--json` é OBRIGATÓRIA para parse de agente
- `--output-format text|json` — omitida, vira JSON sempre que o stdout não for TTY, INCLUSIVE o corpo de `vps export`
- `--lang <LOCALE>`, `-v`/`-vv`/`-vvv` — interface BCP47 negociada para `en` ou `pt-BR`; info, debug e trace no escopo do crate sobre o default `error`
- `--no-input` — recusa stdin, então toda flag `--*-stdin` falha com exit 64 em vez de bloquear esperando humano ausente
- `--dry-run` — imprime o plano e sai
- `--disable-sudo` — suprime elevação SOMENTE nesta invocação
- `--replace-host-key`, `--allow-plaintext-secrets` — exigem intenção humana; plaintext só em teste
- `--secrets-key-file <PATH>`, `--use-keyring` — primary-key de um arquivo 64 hex, ou do keyring do SO
- `--timeout <MS>` — timeout de operação SSH em MILISSEGUNDOS
- `--max-concurrency <N>` — teto de frota e de accepts, 1 a 64, automático se omitida
- `--fail-fast`, `--scp-file-concurrency <N>` — parar na primeira falha de frota; canais SCP paralelos na mesma sessão, default 1

### OBRIGATÓRIO — Redução de payload
- DEVE reduzir com estas flags em vez de canalizar por `jaq`, porque o corte precede a serialização
- `--select <CHAVES>` — mantém só estas chaves separadas por vírgula; `--fields` é a mesma flag; chave ausente é pulada, nunca emitida como null
- `--filter <EXPR>` — `chave=valor`, `chave!=valor`, `chave~substring`, repita para conjugar com AND
- `--limit <N>`, `--sort <CHAVE>`, `--dedupe-by <CHAVE>`, `--count-only` — limita, ordena ascendente com chaves ausentes no fim, descarta repetições, ou devolve só uma contagem
- `--truncate-content <N>` — encurta strings acima de N CARACTERES, nunca bytes, nunca partindo UTF-8
- `--max-output-bytes <N>` — descarta elementos do fim, nunca fatia o texto JSON
- DEVE saber que `vps list --json` emite um ARRAY PURO na raiz, então ali o relatório de redução vai para stderr

### OBRIGATÓRIO — Prévia com dry-run
- DEVE prever chamada destrutiva sem supervisão; o plano é JSON no stdout mesmo em modo texto
- DEVE ler o evento `dry-run` com `operation`, `dry_run` true, `executed` false; EXECUTE `ssh-cli schema dry-run` para o schema
- DEVE saber que a prévia existe SOMENTE em `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt`; exit 64 fora dessas é "sem prévia aqui", NUNCA plano defeituoso
- DEVE esperar pré-condições primeiro, então `--dry-run vps remove <ausente>` ainda sai 66
- DEVE ler `hosts[].replaces_existing` em `vps import`, porque é esse campo, não a contagem, que decide se o import é seguro
- DEVE ler `hosts_to_reencrypt` em `secrets init --force`, porque rotacionar sem recifrar esses hosts perde os segredos at-rest deles
- DEVE saber que as prévias de `sftp rm` e `sftp rmdir` NÃO conectam
- NUNCA ler ausência de evento de sucesso como inação; leia `executed` false

### PROIBIDO
- NUNCA tratar `SSH_CLI_HOME`, `SSH_CLI_LANG`, `SSH_CLI_FORCE_TEXT`, `SSH_CLI_MAX_CONCURRENCY`, `SSH_CLI_SECRETS_KEY` ou `SSH_CLI_SECRETS_KEY_FILE` como store de configuração; são fail-closed
- NUNCA confiar em `RUST_LOG` do ambiente, que é IGNORADA, e NUNCA esperar dump de senha do russh


## Flags Locais Que Parecem Globais
### OBRIGATÓRIO — DEVE colocar estas DEPOIS do subcomando
- DEVE vigiar as quatro que parecem globais e NÃO são, porque antes do subcomando o clap sai com exit 2 — `tunnel --bind <ADDR>`, `tunnel --i-accept-network-exposure`, `tunnel --timeout-ms <MS>`, `vps doctor --probe-ssh`
- DEVE saber que `tunnel --bind` é loopback por padrão e parseada como IP, então typo falha no parse, e que `--i-accept-network-exposure` é OBRIGATÓRIA quando o lado exposto não for loopback
- DEVE preferir `--max-command-chars <N>` e `--max-output-chars <N>` em `vps add|edit`, porque `--max-chars <N>` é apenas ALIAS LEGADO do teto de comando

### OBRIGATÓRIO — A flag de elevação significa duas coisas diferentes
- DEVE saber que `--disable-sudo` ANTES do subcomando suprime elevação em UMA invocação e não muda nada em disco
- DEVE saber que `vps edit --disable-sudo` DEPOIS do subcomando grava `disable_sudo` no config e desabilita elevação naquele host de forma PERMANENTE
- DEVE usar `vps edit --enable-sudo` como ÚNICO desfazedor; o par conflita em `vps edit` e o clap o rejeita
- NUNCA recorrer à forma persistente quando o que se queria era supressão efêmera, porque a grafia é idêntica e só a posição revela o tempo de vida


## Ciclo de Vida e JSON
### OBRIGATÓRIO
- DEVE parsear sucesso SOMENTE do stdout; prosa e envelope de falha dura vão para stderr
- DEVE saber que o CORPO de `vps export` segue o formato resolvido, então o agente, cujo stdout nunca é TTY, recebe JSON até em arquivo `.toml`; TOML exige `--output-format text`
- DEVE parsear `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import`, `vps-export`, e ler `secrets_key_auto_created` no MESMO documento `vps-added`, NUNCA como segundo evento
- DEVE parsear `vps-doctor` com `local.secrets_plaintext_opt_out` BOOLEANO, e `ssh_probe` como null ou lote de health-check
- DEVE ler em exec `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr`, `duration_ms`, e reportar truncagem quando qualquer um dos dois for true
- DEVE tratar `exec --json` de passo único como exatamente UM objeto; `--step` emite um objeto por step, com `step` 0-based e o respectivo `command`
- DEVE ler `scp-transfer` com `ok`/`direction`/`bytes`/`duration_ms`, e `sftp-transfer`, `sftp-list`, `sftp-fs-op`, `sftp-batch`
- DEVE ler os lotes `health-check-batch`, `exec-batch`, `scp-batch`, `sftp-batch`, cada um com `max_concurrency`
- DEVE ler `tunnel_listening` com `local_port`, `remote_host`, `remote_port`, `timeout_ms`, `bind`, `mode`
- DEVE ler `tunnel_closed` com `reason`, `forwards_served`, `capacity_waits`, `duration_ms`, `mode`, distinguindo `reason` entre `deadline`, `signal` e `accept_error`, que compartilham o mesmo exit 0
- DEVE ler no envelope `exit_code`, `message`, `remote_exit_code`, `retryable`, `error_class`, `suggestion`
- DEVE EXECUTAR `ssh-cli schema` SEM nome para listar todo schema válido, e NUNCA adivinhar um nome

### PROIBIDO
- NUNCA inventar chave ausente, esperar vários objetos de `exec --json` de passo único, ou supor canal aberto de um processo anterior


## Inventário Auth Secrets
### OBRIGATÓRIO
- DEVE dar a cada host um `--name` único e EXATAMENTE UMA auth primária — password, `--password-stdin`, `--key`, ou `--use-agent` com `--agent-socket`, trocável por `vps edit`
- DEVE passar `--port` quando não for 22, `--check` para probe imediato, `--tag` repetível para frota, e `--tls` mais os opcionais `--tls-sni`, `--tls-client-cert`, `--tls-client-key`, desfeitos por `--no-tls`
- DEVE ler as máscaras — ausente é JSON `null`, armazenado é a máscara fixa, para `password`, `sudo_password`, `su_password` e `key_passphrase`
- DEVE tratar timeouts de host como MILISSEGUNDOS; abaixo de 1000 emite warning
- DEVE exportar sem segredos por padrão, com segredo vazio redigido como string VAZIA e nunca blob cifrado, e exigir aprovação humana mais `--output` ou `--i-understand-secrets-on-stdout` em `--include-secrets`
- DEVE aceitar import de TOML com chaves em inglês mais aliases legados em português, ou de `vps-export` JSON onde `added_at` pode faltar; TOML inválido é exit 65
- DEVE saber que o TOML diz `username` onde o envelope JSON diz `user`, então import TOML com `user` dá exit 65
- DEVE preferir `--password-stdin`, `--key-passphrase-stdin`, `--sudo-password-stdin`, `--su-password-stdin` a argv, que emite warning, e aplicá-las como override em `exec`, `scp`, `sftp`, `tunnel` e `health-check` quando a credencial salva não bastar
- DEVE passar o nome explícito do host sempre que certeza importar, porque `health-check` sem nome mira o que `connect` marcou como ativo
- DEVE resolver a primary-key nesta ordem — `--secrets-key-file`, depois o keyring do SO com `--use-keyring`, depois um `secrets.key` auto-criado no diretório de config; o keyring aceita o alias legado na leitura
- DEVE esperar escrita atômica, mode 0600 no Unix
- DEVE tratar divergência de host-key como PARADA DURA, usando `--replace-host-key` só após confirmação humana

### PROIBIDO
- NUNCA criar host sem credencial, inventar senha quando o JSON mostra `null`, tratar a máscara como segredo real, commitar segredo cru, imprimir primary-key, ou habilitar plaintext fora de teste
- NUNCA combinar password ou key com `--use-agent` no `vps add`, e NUNCA desabilitar TOFU por conveniência


## Frota Exec Elevação
### OBRIGATÓRIO
- DEVE usar `--all` ou `--hosts <A>,<B>` em `exec`, `sudo-exec`, `su-exec`, `scp`, `sftp` e `health-check`
- DEVE usar `--tags <LISTA>` SOMENTE em `exec`, `sudo-exec` e `su-exec`, casando todo host que tenha QUALQUER tag da lista
- DEVE tratar `--all`, `--hosts` e `--tags` como exclusivas; o clap rejeita qualquer par com exit 2, e inventário vazio mais seletor é exit 64
- DEVE tratar `tunnel` como single-host, então forward multi-host exige N one-shots
- DEVE enviar comando remoto não vazio; comando vazio ou `--step` vazio é exit 64 com a mensagem `empty command`, sempre em inglês
- DEVE respeitar os tetos de comando e de saída, elevando-os com `vps edit` quando o payload exceder
- DEVE usar `--step <CMD>` repetível para vários comandos em UMA sessão
- DEVE respeitar `--disable-sudo` e o ajuste do host, tratar elevação como one-shot, e anexar `--description` quando a auditoria remota importar

### PROIBIDO
- NUNCA inventar `--tags` em `health-check`, `scp` ou `sftp`, spawnar um processo por host quando um seletor de frota cobre o conjunto, prefixar `sudo` cru em `exec`, ou supor shell elevado sticky


## SCP SFTP Tunnel Health
### OBRIGATÓRIO
- DEVE usar `scp` SOMENTE para arquivo regular, na ordem upload local depois remoto, download remoto depois local
- DEVE esperar stream de 32 KiB, com download via `.ssh-cli.partial` e rename atômico
- DEVE tratar mtime e mode como BEST-EFFORT e ler `mtime_preserved` e `durable` no `scp-transfer`; o exit code NÃO diz que o timestamp foi aplicado nem que a entrada é durável
- DEVE tratar arquivo remoto ausente no SCP como exit 66 com a mensagem `file not found`, NUNCA como 74
- DEVE usar `sftp` para árvores com `--recursive`, e `sftp rmdir` só em diretório VAZIO
- DEVE saber que o SFTP recursivo NUNCA segue symlink, e que as máscaras são DIRECIONAIS — a de saída, no upload, mantém setuid, setgid e sticky num arquivo que já é seu; a de entrada, no download, `SFTP_PERM_MASK_UNTRUSTED`, os remove, então elevação do servidor nunca chega ao arquivo local
- DEVE verificar tamanho ou checksum no destino após upload SFTP crítico, NUNCA confiando só na contagem de bytes
- DEVE passar `--timeout` em `exec`, `scp`, `sftp` e `health-check`, e `--timeout-ms` SOMENTE no `tunnel`, em TODO tunnel
- DEVE usar a forma posicional `tunnel <VPS> <LOCAL_PORT> [REMOTE_HOST] [REMOTE_PORT]`, tratando porta local `0` como efêmera e lendo o `local_port` real após o bind
- DEVE ler `mode` em `tunnel_listening` e `tunnel_closed` — `local`, `socks5`, `streamlocal` ou `reverse` — porque é ele que diz como ler os campos vizinhos
- DEVE OMITIR `REMOTE_HOST` e `REMOTE_PORT` com `--socks5` e `--remote-socket`, que não têm destino único, e DEVE PASSÁ-LOS com `--reverse`, onde são o que o SERVIDOR liga; `REMOTE_PORT 0` é legal SOMENTE sob `--reverse`, voltando em `local_port`. Qualquer outra combinação é exit 64
- DEVE passar `--i-accept-network-exposure` quando o lado exposto sair do loopback; sob `--reverse` esse lado é o host remoto posicional, comparado como TEXTO porque nome e string vazia carregam significado, então typo ali é exit 64, não erro de parse
- DEVE saber que `--bind` é aceita e então DESCARTADA sob `--reverse`; defina o listener do servidor pelo host remoto posicional
- DEVE esperar que `--socks5` fale só CONNECT sem autenticação, recusando BIND e UDP ASSOCIATE, e encaminhando nomes de host sem resolver
- DEVE passar caminho POSIX ABSOLUTO em `--remote-socket`, que nomeia um socket no SERVIDOR; caminho relativo é exit 64
- DEVE tratar forward recusado como POLÍTICA, nunca erro transitório
- DEVE AGUARDAR `tunnel_listening` antes de usar a porta local, com o processo vivo até deadline ou sinal

### PROIBIDO
- NUNCA passar diretório no scp nem inventar flag recursiva no scp; árvores DEVEM usar `sftp --recursive`
- NUNCA trocar `--timeout` por `--timeout-ms`, nem tratar arquivo parcial remanescente como artefato final
- NUNCA ligar tunnel em todas as interfaces sem decisão explícita de segurança, e NUNCA substituir host-key automaticamente


## Locale TLS Descoberta
### OBRIGATÓRIO
- DEVE resolver locale — `--lang`, depois `locale set`, depois o sistema, depois inglês
- DEVE gerenciar identidades com `tls mtls import --name --cert --key`, depois `list`/`show`/`remove`
- DEVE criar conta ACME com `--contact` repetível na forma mailto
- DEVE emitir em DOIS passos — `tls acme issue --domain --print-challenge`, publicar o registro DNS TXT, depois `tls acme complete --domain` — e NUNCA inventar loop de espera interativo
- DEVE tratar falha permanente de validação ACME como exit 64 com `retryable` false, e uma transitória como exit 74 só quando o envelope a marcar como retryable

### PROIBIDO
- NUNCA guardar certificado fora do layout XDG, omitir `--print-challenge` no issue, ou inventar nome de schema


## Códigos de Saída e Retry
### OBRIGATÓRIO
- DEVE mapear 0 sucesso, 1 geral, 64 usage, 65 dados, 66 not found, 69 indisponível, 70 erro interno, 73 cant create, 74 IO ou SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- DEVE tratar 69 como RETENTÁVEL, porque keyring do SO bloqueado ou ausente responde ao mesmo argv assim que o serviço sobe
- DEVE tratar 70 como PERMANENTE, porque um CSPRNG que não produz bytes não se resolve esperando nem mudando argumento
- DEVE tratar comando vazio como 64, TOML de import inválido como 65, arquivo SCP remoto ausente como 66, falha de auth como 77, validação ACME permanente como 64
- DEVE tratar deadline de tunnel alcançado DEPOIS do bind como exit 0, e timeout ANTES do bind como exit 74
- DEVE retentar no máximo duas vezes em 74 com backoff e SOMENTE se o envelope disser `retryable` true, e falhar rápido em 64, 65, 66 e 77, mudando os inputs primeiro
- DEVE expor `exit_code` remoto do JSON de sucesso e `remote_exit_code` do envelope de erro, separados do exit do processo


## Folha de Fórmulas Prontas
### OBRIGATÓRIO — DEVE EXECUTAR estas LITERALMENTE, trocando SOMENTE placeholders
- Descoberta e identidade
  - `ssh-cli --version`
  - `ssh-cli commands`
  - `ssh-cli schema`
  - `ssh-cli schema <NAME>`
  - `ssh-cli completions bash|zsh|fish|elvish|powershell`
  - `ssh-cli locale show --json`
  - `ssh-cli locale set <LOCALE>`
  - `ssh-cli locale clear`
  - `ssh-cli --lang <LOCALE> vps list --json`
  - `ssh-cli -v exec <NAME> "true" --json`, também `-vv` e `-vvv`
- Inventário
  - `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --port <PORT> --tag <TAG> --tag <TAG2> --check`
  - `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --use-agent --agent-socket <SOCK> --tag <TAG>`
  - `printf '%s' "$PASS" | ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --password-stdin`
  - `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --tls --tls-sni <SNI> --tls-client-cert <CERT> --tls-client-key <KEY>`
  - `printf '%s' "$SUDO" | ssh-cli vps edit <NAME> --sudo-password-stdin`
  - `printf '%s' "$SU" | ssh-cli vps edit <NAME> --su-password-stdin`
  - `ssh-cli vps edit <NAME> --timeout <MS> --max-command-chars <N> --max-output-chars <N>`
  - `ssh-cli vps edit <NAME> --use-agent --agent-socket <SOCK>`
  - `ssh-cli vps edit <NAME> --tls --tls-sni <SNI>`, e `ssh-cli vps edit <NAME> --no-tls`
  - `ssh-cli vps edit <NAME> --disable-sudo` PERSISTE o bloqueio; `ssh-cli vps edit <NAME> --enable-sudo` é o ÚNICO desfazedor
  - `ssh-cli vps list --json`, e `ssh-cli vps list --tag <TAG> --json`
  - `ssh-cli vps show <NAME> --json`
  - `ssh-cli vps path`
  - `ssh-cli doctor --json`, e o idêntico `ssh-cli vps doctor --json`
  - `ssh-cli doctor --probe-ssh --json`, e `ssh-cli vps doctor --probe-ssh --hosts <A>,<B> --json`
  - `ssh-cli vps export -o /tmp/hosts.json`, e `ssh-cli --output-format text vps export -o /tmp/hosts.toml`
  - `ssh-cli vps export --json`
  - `ssh-cli vps export --include-secrets -o /tmp/hosts-secrets.toml`
  - `ssh-cli vps import --file /tmp/hosts.toml`
  - `ssh-cli vps import --file /tmp/hosts-redacted.toml --allow-incomplete`
  - `ssh-cli connect <NAME>`
  - `ssh-cli vps remove <NAME>`
- Execução e frota
  - `ssh-cli exec <NAME> "<CMD>" --json`
  - `ssh-cli exec <NAME> "<CMD>" --json --timeout <MS> --description "<AUDIT>"`
  - `ssh-cli -q exec <NAME> "<CMD>" --json`
  - `ssh-cli exec <NAME> "<CMD>" --step "<CMD2>" --step "<CMD3>" --json`
  - `ssh-cli exec <NAME> "id" --json --use-agent --agent-socket <SOCK>`
  - `printf '%s' "$KEY_PASS" | ssh-cli exec <NAME> "id" --json --key <KEY_PATH> --key-passphrase-stdin`
  - `ssh-cli sudo-exec <NAME> "<CMD>" --json`, e com `--step "<CMD2>"`
  - `printf '%s' "$SUDO" | ssh-cli sudo-exec <NAME> "<CMD>" --json --sudo-password-stdin`
  - `ssh-cli su-exec <NAME> "<CMD>" --json`
  - `printf '%s' "$SU" | ssh-cli su-exec <NAME> "<CMD>" --json --su-password-stdin`
  - `ssh-cli --max-concurrency <N> exec --all "<CMD>" --json`
  - `ssh-cli --fail-fast exec --all "<CMD>" --json`
  - `ssh-cli exec --hosts <A>,<B> "<CMD>" --json`
  - `ssh-cli exec --tags <TAG1>,<TAG2> "<CMD>" --json`
  - `ssh-cli sudo-exec --all "<CMD>" --json`, e `ssh-cli sudo-exec --tags <TAG> "<CMD>" --json`
  - `ssh-cli su-exec --all "<CMD>" --json`
- Transferência
  - `ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json`
  - `ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json`
  - `ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --timeout <MS>`
  - `ssh-cli scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json`
  - `ssh-cli scp download <NAME> <R1> <R2> <LOCAL_DIR> --json`
  - `ssh-cli --scp-file-concurrency <N> scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json`
  - `ssh-cli scp upload --all <LOCAL_FILE> <REMOTE_FILE> --json`
  - `ssh-cli scp upload --all <F1> <F2> <REMOTE_DIR> --json`
  - `ssh-cli scp download --all <REMOTE_FILE> <LOCAL_PREFIX> --json`
  - `printf '%s' "$PASS" | ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json --password-stdin`
  - `printf '%s' "$KEY_PASS" | ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --key <KEY_PATH> --key-passphrase-stdin`
  - `ssh-cli sftp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json`
  - `ssh-cli sftp upload --recursive <NAME> <LOCAL_DIR> <REMOTE_DIR> --json`
  - `ssh-cli sftp download --recursive <NAME> <REMOTE_DIR> <LOCAL_DIR> --json`
  - `ssh-cli sftp ls <NAME> <REMOTE_DIR> --json`
  - `ssh-cli sftp mkdir <NAME> <REMOTE_DIR> --json`
  - `ssh-cli sftp rmdir <NAME> <REMOTE_DIR> --json`
  - `ssh-cli sftp rm <NAME> <REMOTE_FILE> --json`
  - `ssh-cli sftp stat <NAME> <REMOTE_PATH> --json`
  - `ssh-cli sftp rename <NAME> <FROM> <TO> --json`
  - `ssh-cli sftp upload --hosts <A>,<B> <LOCAL_FILE> <REMOTE_FILE> --json`
- Modos de tunnel, e então DEVE AGUARDAR `tunnel_listening`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json`
  - `ssh-cli tunnel <NAME> 0 <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json`, e então leia o `local_port` efêmero
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --bind 127.0.0.1`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> --socks5 --timeout-ms <MS> --json`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> --remote-socket /var/run/docker.sock --timeout-ms <MS> --json`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> 127.0.0.1 <REMOTE_PORT> --reverse --timeout-ms <MS> --json`
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> 127.0.0.1 0 --reverse --timeout-ms <MS> --json`, e então leia o `local_port` alocado pelo servidor
  - `ssh-cli tunnel <NAME> <LOCAL_PORT> 0.0.0.0 <REMOTE_PORT> --reverse --i-accept-network-exposure --timeout-ms <MS> --json`
  - Anexe `--key <KEY_PATH>`, `--use-agent --agent-socket <SOCK>`, `--password-stdin` ou `--key-passphrase-stdin` a qualquer fórmula de tunnel
- Prévia, health e secrets
  - `ssh-cli --json --dry-run vps remove <NAME>`
  - `ssh-cli --json --dry-run vps import --file <PATH>`
  - `ssh-cli --json --dry-run sftp rm <NAME> <REMOTE_FILE>`
  - `ssh-cli --json --dry-run sftp rmdir <NAME> <REMOTE_DIR>`
  - `ssh-cli --json --dry-run secrets init --force`
  - `ssh-cli --json --dry-run secrets reencrypt`
  - `ssh-cli health-check <NAME> --json`, e com `--timeout <MS>`
  - `ssh-cli health-check --json`
  - `ssh-cli health-check --all --json`, e `ssh-cli --max-concurrency <N> health-check --all --json`
  - `ssh-cli health-check --hosts <A>,<B> --json`
  - `ssh-cli health-check <NAME> --json --use-agent --agent-socket <SOCK>`
  - `printf '%s' "$PASS" | ssh-cli health-check <NAME> --json --password-stdin`
  - `printf '%s' "$KEY_PASS" | ssh-cli health-check <NAME> --json --key <KEY_PATH> --key-passphrase-stdin`
  - `ssh-cli health-check <NAME> --json --replace-host-key`
  - `ssh-cli secrets status --json`
  - `ssh-cli secrets init --json`, `ssh-cli secrets init --force --json`, `ssh-cli secrets init --keyring --json`
  - `ssh-cli secrets reencrypt --json`
  - `ssh-cli --secrets-key-file <KEY_FILE> secrets status --json`
  - `ssh-cli --use-keyring secrets status --json`
  - `ssh-cli --allow-plaintext-secrets --config-dir <DIR> secrets status --json`
  - `ssh-cli --config-dir <DIR> vps list --json`
  - `ssh-cli --no-input vps add --name <NAME> --host <HOST> --user <USER> --password-stdin`
- Redução de payload
  - `ssh-cli --select name,host,user vps list --json`
  - `ssh-cli --filter user=root --limit 5 vps list --json`
  - `ssh-cli --sort name --dedupe-by host vps list --json`
  - `ssh-cli --count-only vps list --json`
  - `ssh-cli --truncate-content 500 --max-output-bytes 65536 exec --all "<CMD>" --json`
- TLS
  - `ssh-cli tls provider --json`
  - `ssh-cli tls paths --json`
  - `ssh-cli tls mtls list --json`
  - `ssh-cli tls mtls import --name <NAME> --cert <CERT_PEM> --key <KEY_PEM> --json`
  - `ssh-cli tls mtls show <NAME> --json`
  - `ssh-cli tls mtls remove <NAME> --json`
  - `ssh-cli tls acme account create --contact mailto:<EMAIL> --json`
  - `ssh-cli tls acme account create --contact mailto:<EMAIL> --staging --force --json`
  - `ssh-cli tls acme account show --json`
  - `ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --json`
  - `ssh-cli tls acme complete --domain <DOMAIN> --json`
  - `ssh-cli tls acme status --json`, e `ssh-cli tls acme status --domain <DOMAIN> --json`
  - `ssh-cli tls acme list --json`

### PROIBIDO
- NUNCA canalizar `--include-secrets` sem `--output` ou `--i-understand-secrets-on-stdout`
- NUNCA inventar `--local-port`; a porta local é POSICIONAL
