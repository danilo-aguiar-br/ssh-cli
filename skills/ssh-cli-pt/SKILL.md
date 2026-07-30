---
name: ssh-cli
description: Esta skill DEVE auto-ativar quando inventário VPS SSH, config XDG, exec/sudo-exec/su-exec, scp file-only, sftp, tunnel_listening, health-check, secrets, multi-host --all/--hosts/--tags, --step mesma sessão, TLS mTLS ACME, locale, schema/commands, frota batch ou devops de agente sem TTY forem implícitos. DEVE cobrir CRUD vps (add list remove edit show path doctor export import), connect, auth (password-stdin key passphrase-stdin --use-agent --agent-socket), secrets init/status/reencrypt, doctor event igual a vps-doctor, tunnel --timeout-ms, scp/sftp lotes, --fail-fast, --scp-file-concurrency, --max-concurrency, completions, flags globais, auth exit 77, empty command exit 64, scp missing exit 66, import ruim exit 65, ACME validation permanente exit 64. DEVE preferir --json e --*-stdin. NUNCA telemetria, daemon ou segredos em stdout sem guarda.
---

# ssh-cli Skill de Agente

## Missão
### OBRIGATÓRIO
- DEVE tratar esta skill como LEI SUPREMA em toda invocação de `ssh-cli`
- DEVE SEMPRE executar `ssh-cli` como subprocesso one-shot nascer-executar-morrer
- DEVE aguardar o exit antes de parse de stdout/stderr, exceto no `tunnel` bound até timeout ou sinal
- DEVE usar hosts salvos via `vps add` e preferir frota `--all`/`--hosts`/`--tags` a N spawns
- DEVE passar `--json` para saída estruturada; copiar fórmulas desta skill e só substituir placeholders
- DEVE preferir `--*-stdin` a segredos em argv; descobrir a árvore com `ssh-cli commands` em dúvida

### PROIBIDO
- NUNCA manter sessão SSH entre processos exceto `tunnel` bound até o deadline
- NUNCA preferir segredos em argv quando stdin existir; NUNCA inventar flags fora desta skill
- NUNCA introduzir daemon, telemetria ou logs com senhas/passphrases/primary-key; NUNCA usar `RUST_LOG` (IGNORADO; só `-v`/`-vv`/`-vvv`)

## Quando Invocar
### OBRIGATÓRIO
- DEVE auto-ativar em SSH remoto, inventário VPS, multi-host, config XDG, exec/sudo-exec/su-exec, scp, sftp, tunnel, health-check
- DEVE auto-ativar em transferência de arquivo, árvores sftp, port forward, `tunnel_listening`, secrets at-rest, TOFU/host-key
- DEVE auto-ativar em TLS/mTLS/ACME, locale, schema/commands, devops de agente sem TTY, mesmo sem nomear ssh-cli

### PROIBIDO
- NUNCA esperar pedido explícito da skill quando operações SSH remotas forem implícitas

## Prompts de Ação
### OBRIGATÓRIO
- DEVE seguir esta ordem em toda tarefa remota não trivial
1. VERIFIQUE o binário com `ssh-cli --version`
2. DESCUBRA contratos com `ssh-cli schema` / `ssh-cli commands`; INSPECIONE config com `ssh-cli doctor --json` e `ssh-cli vps path`
3. GARANTA secrets com `ssh-cli secrets status --json`
4. REGISTRE ou edite host com password ou key ou `--use-agent`/`--agent-socket`; tags com `--tag`; TLS com `--tls`/`--tls-sni`/`--tls-client-cert`/`--tls-client-key` quando exigido
5. PROBE com `ssh-cli health-check <name> --json` ou `ssh-cli health-check --all --json`
6. EXECUTE com `exec`/`sudo-exec`/`su-exec` e `--json`; frota com `--all`; multi-comando na mesma sessão com `--step`
7. TRANSFIRA arquivo regular com `scp upload|download --json`; árvores/FS com `sftp` (`--recursive` ou `ls|mkdir|rmdir|rm|stat|rename`); frota com `--all`/`--hosts`
8. ENCAMINHE port forward só com `tunnel` + `--timeout-ms` obrigatório + `--json`
9. PARSEIE exit, stdout de sucesso e envelope de erro no stderr (`exit_code`, `message`, `remote_exit_code`, `retryable`, `error_class`, `suggestion`)
10. SANITIZE logs duráveis para que segredos e primary-key nunca permaneçam

## Catálogo de Comandos
### OBRIGATÓRIO
- DEVE tratar a árvore abaixo como superfície OBRIGATÓRIA (descoberta via `ssh-cli commands`)
- `vps add` — registra host
- `vps list` — lista inventário (máscara)
- `vps remove` — remove host
- `vps edit` — edita campos do host
- `vps show` — detalha host (máscara)
- `vps path` — path vencedor do config
- `vps doctor` — diagnóstico XDG/schema (+ `--probe-ssh`)
- `vps export` — exporta inventário (TOML default)
- `vps import` — importa TOML/JSON
- `connect` — grava marcador `active`
- `exec` — comando remoto one-shot
- `sudo-exec` — elevação sudo one-shot
- `su-exec` — elevação `su -` one-shot
- `scp upload` — upload arquivo regular
- `scp download` — download arquivo regular
- `sftp upload` — upload SFTP arquivo/árvore
- `sftp download` — download SFTP arquivo/árvore
- `sftp ls` — lista dir remoto
- `sftp mkdir` — cria dir remoto
- `sftp rmdir` — remove dir remoto vazio
- `sftp rm` — remove arquivo remoto
- `sftp stat` — metadata de path remoto
- `sftp rename` — renomeia path remoto
- `tunnel` — port forward bound
- `health-check` — probe SSH single/frota
- `secrets status` — status at-rest
- `secrets init` — gera primary-key
- `secrets reencrypt` — reescreve secrets
- `completions` — bash zsh fish elvish powershell
- `commands` — árvore JSON de comandos
- `schema` — catálogo ou body de schema
- `doctor` — alias root de `vps doctor`
- `locale show` — locale resolvido
- `locale set` — persiste preferência de idioma
- `locale clear` — remove preferência
- `tls provider` — status rustls/`aws_lc_rs`
- `tls paths` — layout XDG TLS
- `tls mtls list` — lista identidades mTLS
- `tls mtls import` — importa cert+key PEM
- `tls mtls show` — paths de identidade
- `tls mtls remove` — remove identidade
- `tls acme account create` — cria conta ACME
- `tls acme account show` — mostra conta ACME
- `tls acme issue` — inicia ordem DNS-01
- `tls acme complete` — completa ordem pendente
- `tls acme status` — status de cert/domínio
- `tls acme list` — lista domínios ACME
- NUNCA inventar subcomandos fora deste catálogo

## Flags Globais
### OBRIGATÓRIO
- `--lang <LOCALE>` — força BCP47 (`en`, `en-US`, `pt-BR`, `pt`); negocia para `en` ou `pt-BR`
- `-v`/`-vv`/`-vvv` — info/debug/trace SEMPRE com escopo `warn,ssh_cli=*`; default `error`
- `-q` / `--quiet` — suprime prosa humana não-JSON
- `--config-dir <DIR>` — sobrescreve diretório de config
- `--no-color` — desliga cor
- `--output-format text|json` — formato global; non-TTY default JSON em comandos gerais
- `--json` — força JSON (alias de `--output-format json`)
- `--disable-sudo` — desliga elevação nesta invocação
- `--replace-host-key` — substitui host-key divergente no TOFU (após aprovação humana)
- `--allow-plaintext-secrets` — opt-out de cifragem at-rest (somente testes)
- `--secrets-key-file <PATH>` — primary-key em arquivo 64 hex
- `--use-keyring` — prefere OS keyring para primary-key
- `--timeout <MS>` — timeout SSH global (exec/scp/health-check); tunnel exige `--timeout-ms`
- `--max-concurrency <N>` — cap fan-out multi-host e accepts de tunnel (1..=64; auto se omitido)
- `--fail-fast` — para admissão multi-host no primeiro erro
- `--scp-file-concurrency <N>` — canais SCP paralelos na mesma sessão (default 1)

### PROIBIDO
- NUNCA usar env de produto `SSH_CLI_HOME`/`SSH_CLI_LANG`/`SSH_CLI_FORCE_TEXT`/`SSH_CLI_MAX_CONCURRENCY`/`SSH_CLI_SECRETS_KEY`/`SSH_CLI_SECRETS_KEY_FILE` como stores
- NUNCA confiar em `RUST_LOG`; NUNCA esperar dump de senha russh nos logs

## Ciclo de Vida e JSON
### OBRIGATÓRIO
- DEVE invocar um processo CLI completo por ação; DEVE parsear só stdout como sucesso e stderr como logs/envelope de erro
- DEVE forçar `--json` para parse de agente; non-TTY default JSON em comandos gerais NÃO se aplica ao corpo de `vps export` (TOML salvo `vps export --json`)
- DEVE usar `-v`/`-vv`/`-vvv` só ao depurar; `RUST_LOG` é IGNORADO; NUNCA esperar dump de senha russh
- DEVE parsear CRUD com eventos `vps-added`/`vps-edited`/`vps-removed`/`vps-connected`/`vps-import`
- DEVE parsear `secrets_key_auto_created` no MESMO documento `vps-added` (nunca segundo evento)
- DEVE parsear doctor com `event` igual a `vps-doctor`; `secrets_plaintext_opt_out` é boolean
- DEVE tratar export redacted com secrets vazios como strings vazias e não vazios como `***` (`FIXED_MASK`); NUNCA `sshcli-enc` para vazios
- DEVE usar `vps export --json` só para envelope `event` igual a `vps-export`
- DEVE parsear exec single-step `--json` como exatamente UM objeto de sucesso; multi-step `--step` emite um objeto por step com índice `step` (0-based)
- DEVE ler exec `stdout`/`stderr`/`exit_code`/`truncated_stdout`/`truncated_stderr`/`duration_ms`
- DEVE ler scp sucesso com `event` igual a `scp-transfer`; tunnel ready com `event` igual a `tunnel_listening`
- DEVE ler SFTP com `sftp-transfer`/`sftp-list`/`sftp-fs-op`/`sftp-batch`; lotes com `health-check-batch`/`exec-batch`/`scp-batch`/`sftp-batch` e `max_concurrency`
- DEVE parsear envelope de erro `exit_code`/`message`/`remote_exit_code`/`retryable`/`error_class`/`suggestion`
- DEVE tratar `password`/`sudo_password`/`su_password`/`key_passphrase` em list/show como `null` (ausente) ou `***` (armazenado)
- DEVE tratar `added_at` presente em list/show/export; import aceita omissão e serde preenche default
- DEVE manter cancelamento de lote multi-host com cardinalidade correta; preferir `--fail-fast`/`--max-concurrency`

### PROIBIDO
- NUNCA misturar stderr no parse de sucesso nem assumir canal SSH residual
- NUNCA inventar chaves JSON ou senhas quando `password` for `null`
- NUNCA esperar segundo evento para `secrets_key_auto_created` nem múltiplos objetos em exec single-step; NUNCA tratar `secrets_plaintext_opt_out` como string

## Inventário Auth Secrets
### OBRIGATÓRIO
- DEVE registrar host com `--name` único e exatamente uma auth primária (password/`--password-stdin` ou `--key` ou `--use-agent`/`--agent-socket`)
- DEVE passar `--port` se não for 22; `--check` para probe imediato; `--tag` repetível; `--tls` e opcionais `--tls-sni`/`--tls-client-cert`/`--tls-client-key` quando exigido
- DEVE usar `vps doctor --json`/`doctor --json` e `vps path` para localizar config; timeouts de host em milissegundos (warning se menor que 1000)
- DEVE exportar sem segredos por padrão; exigir aprovação humana para `--include-secrets`; NUNCA pipe de segredos sem `-o`/`--output` ou `--i-understand-secrets-on-stdout`
- DEVE importar TOML EN/aliases PT e envelopes JSON `vps-export`; usar `--allow-incomplete` em skeleton redacted; TOML inválido = exit 65
- DEVE usar `connect` só para gravar marcador `active` (NÃO é sessão SSH); health-check sem nome só após connect
- DEVE preferir `--password-stdin`/`--key-passphrase-stdin`/`--sudo-password-stdin`/`--su-password-stdin`; password-like em argv emite warning
- DEVE aplicar overrides de auth em exec/scp/sftp/tunnel/health-check quando credenciais salvas forem insuficientes; auth falha = exit 77
- DEVE tratar cifragem at-rest como padrão; termo de produto primary-key; alias legado keyring `secrets-master-key` só leitura junto a `secrets-primary-key`
- DEVE rodar `secrets status --json` antes de diagnosticar decrypt; `secrets init`/`--json`/`--keyring`/`--force`; `secrets reencrypt`/`--json` após rotação
- DEVE resolver primary-key nesta ordem: (1) `--secrets-key-file` 64 hex; (2) `--use-keyring`; (3) `secrets.key` auto-criado com `secrets_key_auto_created` no mesmo `vps-added`
- DEVE restringir plaintext a testes com `--allow-plaintext-secrets`; usar `--config-dir` e `--lang`/`locale set` em vez de env
- DEVE esperar writes atômicos de `config.toml`/`secrets.key` e mode 0600 em Unix
- DEVE tratar mismatch de host-key como hard stop; `--replace-host-key` só após confirmação humana

### PROIBIDO
- NUNCA criar host com credencial vazia nem inventar senhas para hosts só-chave
- NUNCA commitar segredos crus; NUNCA imprimir primary-key ou secrets decifrados
- NUNCA combinar password/key com `--use-agent` no add; NUNCA plaintext em produção; NUNCA env de secrets como store
- NUNCA auto-substituir host-key sem aprovação; NUNCA desabilitar TOFU por conveniência

## Frota Exec Elevação
### OBRIGATÓRIO
- DEVE preferir frota a N spawns; `--all`/`--hosts` em exec/sudo-exec/su-exec/scp/sftp/health-check
- DEVE usar `--tags` SOMENTE em exec/sudo-exec/su-exec (NÃO em health-check/scp/sftp)
- DEVE tratar `--all`/`--hosts`/`--tags` como mutuamente exclusivos; tunnel é single-host
- DEVE usar `--fail-fast` e `--max-concurrency N` (1..=64; auto = CPUs×4 vs RAM livre/2 / 16 MiB)
- DEVE usar `--scp-file-concurrency N` para multi-arquivo na mesma sessão; multi-arquivo single-host `scp upload <VPS> f1 f2 … <REMOTE_DIR>`
- DEVE usar `vps doctor --probe-ssh [--hosts a,b] --json` para local + health em um `event` igual a `vps-doctor`
- DEVE tratar inventário vazio com frota como exit 64
- DEVE validar `max_command_chars` (default 1000) e `max_output_chars` (default 100000); elevar via `vps edit` se necessário
- DEVE tratar comando remoto vazio como exit 64 mensagem `empty command` (sempre inglês)
- DEVE usar `--step <CMD>` repetível na mesma sessão; step vazio = exit 64; multi-step JSON um objeto por step
- DEVE usar `sudo-exec` (packing `sh -c`) e `su-exec` one-shot; respeitar `--disable-sudo` e `disable_sudo` do host
- DEVE aceitar `--step` e frota em sudo-exec/su-exec igual a exec; anexar `--description` para auditoria remota

### PROIBIDO
- NUNCA inventar `--tags` em health-check/scp/sftp; NUNCA combinar `--all` com `--hosts`/`--tags`
- NUNCA spawn N processos quando frota cobre; NUNCA abrir N sessões quando `--step` basta
- NUNCA prefixar `sudo` cru em `exec` nem assumir shell elevado sticky
- NUNCA enviar comando remoto vazio; NUNCA retry 64/65/66/77 sem mudar inputs; NUNCA ignorar truncagem

## SCP SFTP Tunnel Health
### OBRIGATÓRIO
- DEVE usar scp só para arquivo regular; ordem `upload <vps> <local> <remote>` e `download <vps> <remote> <local>`; SEMPRE `--json` no agente
- DEVE parsear scp com `event` igual a `scp-transfer`; `direction` só `upload`/`download`
- DEVE esperar stream 32 KiB; download grava `.ssh-cli.partial` e rename atômico; preserve mtime/mode sem flag extra
- DEVE tratar SCP remoto ausente como exit 66 mensagem `file not found: <path>`
- DEVE preferir frota scp `--all`/`--hosts`; usar `--timeout <ms>` e overrides de auth quando necessário
- DEVE usar sftp para arquivo/árvore (`--recursive`) e FS (`ls|mkdir|rmdir|rm|stat|rename`); SEMPRE `--json` no agente
- DEVE gravar bytes reais no destino em SFTP upload; NUNCA confiar só no JSON `bytes` em uploads críticos — DEVE verificar tamanho ou checksum no destino
- DEVE tratar SFTP recursivo como NUNCA seguir symlink; máscara de permissões SFTP `0o7777` (sem bits S_IFMT)
- DEVE usar `sftp rmdir` só para dir remoto vazio; preferir frota sftp `--all`/`--hosts`
- DEVE passar `--timeout-ms` em todo `tunnel`; args posicionais `tunnel <vps> <local_port> <remote_host> <remote_port>`; bind padrão `127.0.0.1`
- DEVE AGUARDAR `event` igual a `tunnel_listening` antes de usar a porta local; porta `0` é efêmera — usar `local_port` do JSON (>=1)
- DEVE tratar deadline pós-bind como exit 0; timeout pré-bind como exit 74; processo vive até deadline ou sinal
- DEVE usar health-check com `--timeout <ms>` e overrides de auth; frota com `--all`/`--hosts`; `--replace-host-key` só após review humano
- DEVE usar `--timeout-ms` só no tunnel e `--timeout` em exec/scp/health-check

### PROIBIDO
- NUNCA passar diretórios no scp nem inventar `-r` (árvores = `sftp --recursive`); NUNCA tratar scp como SFTP
- NUNCA parsear scp/sftp/tunnel como campos da família exec; NUNCA deixar `.ssh-cli.partial` final
- NUNCA abrir tunnel sem bound ou sem `--timeout-ms`; NUNCA usar porta antes de `tunnel_listening`
- NUNCA bind em `0.0.0.0` sem decisão de segurança; NUNCA tunnel multi-host nativo
- NUNCA trocar `--timeout` e `--timeout-ms` entre subcomandos; NUNCA seguir symlink SFTP; NUNCA REPL SFTP
- NUNCA auto `--replace-host-key` sem aprovação humana

## Locale TLS Discovery Completions
### OBRIGATÓRIO
- DEVE usar `locale show`/`locale set <LOCALE>`/`locale clear`; `--lang` vence arquivo `lang` (mode 0o600); negociar BCP47 para `en` ou `pt-BR`
- DEVE usar `tls provider` e `tls paths` (layout XDG `tls/`/`tls/mtls/`/`tls/acme/`)
- DEVE usar `tls mtls import --name <NAME> --cert <PEM> --key <PEM>` e `list|show|remove`
- DEVE usar `tls acme account create --contact mailto:<EMAIL>` (obrigatório, repetível; `--staging`/`--force` quando aplicável) e `account show`
- DEVE usar `tls acme issue --domain <DOM> --print-challenge` (agente, sem wait); `complete`; `status`; `list`
- DEVE habilitar SSH-over-TLS com `vps add|edit --tls` e opcionais SNI/client cert/key
- DEVE tratar validação ACME permanente como exit 64 `retryable` false; NUNCA retry como 74
- DEVE rodar `commands` e `schema`/`schema <NAME>` para discovery; completions `bash|zsh|fish|elvish|powershell`
- DEVE manter automação em flags e JSON, não em scripts de completion

### PROIBIDO
- NUNCA tratar `SSH_CLI_LANG`/`LANG` como store; NUNCA inventar locales fora da negociação
- NUNCA armazenar certs fora do XDG; NUNCA omitir `--print-challenge` no issue; NUNCA inventar schemas

## Códigos de Saída e Retry
### OBRIGATÓRIO
- DEVE mapear 0 sucesso, 1 geral, 64 usage, 65 data, 66 not found, 73 cant create, 74 IO/SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- DEVE tratar empty command = 64; import TOML inválido = 65; SCP missing = 66; auth = 77; ACME validation permanente = 64
- DEVE tratar tunnel pós-bind deadline = 0; pré-bind timeout = 74
- DEVE fazer no máximo dois retries em 74 SOMENTE se `retryable` true; falhar rápido em 64/65/66/77
- DEVE ler `retryable`/`error_class`/`suggestion` antes de re-invocar; expor `exit_code` remoto e `remote_exit_code` separadamente do exit CLI

### PROIBIDO
- NUNCA engolir exits não zero nem confundir falha remota com usage local
- NUNCA retry exit 0 pós-bind do tunnel nem ACME permanente (64) como 74
- NUNCA responder antes de ler o exit; NUNCA pular parse JSON após exit não zero em modo JSON

## Folha de Fórmulas Prontas
### OBRIGATÓRIO
- DEVE copiar as fórmulas exatamente e somente substituir placeholders
- DEVE EXECUTAR `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --port <PORT> --check`
- DEVE EXECUTAR `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --tag <TAG> --tag <TAG2>`
- DEVE EXECUTAR `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --use-agent --agent-socket <SOCKET>`
- DEVE EXECUTAR `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --tls --tls-sni <SNI>`
- DEVE EXECUTAR `ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --tls --tls-client-cert <CERT> --tls-client-key <KEY>`
- DEVE PASSAR senha com `printf '%s' "$PASS" | ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --password-stdin`
- DEVE PASSAR sudo com `printf '%s' "$SUDO" | ssh-cli vps edit <NAME> --sudo-password-stdin`
- DEVE EXECUTAR `ssh-cli vps list --json`
- DEVE EXECUTAR `ssh-cli vps list --tag <TAG> --json`
- DEVE EXECUTAR `ssh-cli vps show <NAME> --json`
- DEVE EXECUTAR `ssh-cli vps edit <NAME> --timeout <MS> --max-command-chars <N> --max-output-chars <N>`
- DEVE EXECUTAR `ssh-cli vps edit <NAME> --tls --tls-sni <SNI>` e `ssh-cli vps edit <NAME> --no-tls`
- DEVE EXECUTAR `ssh-cli vps doctor --json` e `ssh-cli doctor --json`
- DEVE EXECUTAR `ssh-cli vps doctor --probe-ssh --json` e com `--hosts <A>,<B>`
- DEVE EXECUTAR `ssh-cli vps path`
- DEVE EXECUTAR `ssh-cli vps export -o /tmp/hosts.toml`
- DEVE EXECUTAR `ssh-cli vps export --json`
- DEVE EXECUTAR `ssh-cli vps export --include-secrets -o /tmp/hosts-secrets.toml`
- NUNCA enviar `--include-secrets` em pipe sem `--output`/`-o` ou `--i-understand-secrets-on-stdout`
- DEVE EXECUTAR `ssh-cli vps import --file /tmp/hosts.toml`
- DEVE EXECUTAR `ssh-cli vps import --file /tmp/hosts.json`
- DEVE EXECUTAR `ssh-cli vps import --file /tmp/hosts-redacted.toml --allow-incomplete`
- DEVE EXECUTAR `ssh-cli connect <NAME>`
- DEVE EXECUTAR `ssh-cli vps remove <NAME>`
- DEVE EXECUTAR `ssh-cli exec <NAME> "<CMD>" --json`
- DEVE EXECUTAR `ssh-cli exec <NAME> "<CMD>" --json --timeout <MS> --description "<AUDIT>"`
- DEVE EXECUTAR `ssh-cli exec <NAME> "<CMD>" --step "<CMD2>" --step "<CMD3>" --json`
- DEVE EXECUTAR `ssh-cli -q exec <NAME> "<CMD>" --json`
- DEVE EXECUTAR `ssh-cli sudo-exec <NAME> "<CMD>" --json`
- DEVE PASSAR sudo com `printf '%s' "$SUDO" | ssh-cli sudo-exec <NAME> "<CMD>" --json --sudo-password-stdin`
- DEVE EXECUTAR `ssh-cli su-exec <NAME> "<CMD>" --json`
- DEVE EXECUTAR `ssh-cli --max-concurrency <N> exec --all "<CMD>" --json`
- DEVE EXECUTAR `ssh-cli exec --hosts <A>,<B> "<CMD>" --json`
- DEVE EXECUTAR `ssh-cli exec --tags <TAG1>,<TAG2> "<CMD>" --json`
- DEVE EXECUTAR `ssh-cli --fail-fast exec --all "<CMD>" --json`
- DEVE EXECUTAR `ssh-cli sudo-exec --all "<CMD>" --json`
- DEVE EXECUTAR `ssh-cli su-exec --all "<CMD>" --json`
- DEVE EXECUTAR `ssh-cli sudo-exec <NAME> "<CMD>" --step "<CMD2>" --json`
- DEVE EXECUTAR `ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json`
- DEVE EXECUTAR `ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json`
- DEVE EXECUTAR `ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --timeout <MS>`
- DEVE EXECUTAR `ssh-cli scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json`
- DEVE EXECUTAR `ssh-cli --scp-file-concurrency <N> scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json`
- DEVE EXECUTAR `ssh-cli scp upload --all <F1> <F2> <REMOTE_DIR> --json`
- DEVE EXECUTAR `ssh-cli scp download <NAME> <R1> <R2> <LOCAL_DIR> --json`
- DEVE PASSAR senha scp com `printf '%s' "$PASS" | ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json --password-stdin`
- DEVE PASSAR passphrase scp com `printf '%s' "$KEY_PASS" | ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --key <KEY_PATH> --key-passphrase-stdin`
- DEVE EXECUTAR `ssh-cli scp upload --all <LOCAL_FILE> <REMOTE_FILE> --json`
- DEVE EXECUTAR `ssh-cli scp download --all <REMOTE_FILE> <LOCAL_PREFIX> --json`
- DEVE EXECUTAR `ssh-cli scp upload --hosts <A>,<B> <LOCAL_FILE> <REMOTE_FILE> --json`
- DEVE EXECUTAR `ssh-cli sftp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json`
- DEVE EXECUTAR `ssh-cli sftp upload --recursive <NAME> <LOCAL_DIR> <REMOTE_DIR> --json`
- DEVE EXECUTAR `ssh-cli sftp download --recursive <NAME> <REMOTE_DIR> <LOCAL_DIR> --json`
- DEVE EXECUTAR `ssh-cli sftp ls <NAME> <REMOTE_DIR> --json`
- DEVE EXECUTAR `ssh-cli sftp mkdir <NAME> <REMOTE_DIR> --json`
- DEVE EXECUTAR `ssh-cli sftp rmdir <NAME> <REMOTE_DIR> --json`
- DEVE EXECUTAR `ssh-cli sftp rm <NAME> <REMOTE_FILE> --json`
- DEVE EXECUTAR `ssh-cli sftp stat <NAME> <REMOTE_PATH> --json`
- DEVE EXECUTAR `ssh-cli sftp rename <NAME> <FROM> <TO> --json`
- DEVE EXECUTAR `ssh-cli sftp upload --all <LOCAL_FILE> <REMOTE_FILE> --json`
- DEVE EXECUTAR `ssh-cli sftp upload --hosts <A>,<B> <LOCAL_FILE> <REMOTE_FILE> --json`
- DEVE EXECUTAR `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json`
- DEVE EXECUTAR `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --bind 127.0.0.1`
- DEVE EXECUTAR `ssh-cli tunnel <NAME> 0 <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json`
- DEVE AGUARDAR evento igual a `tunnel_listening` antes de usar a porta local
- DEVE PASSAR senha tunnel com `printf '%s' "$PASS" | ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --password-stdin`
- DEVE EXECUTAR `ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --key <KEY_PATH>`
- DEVE PASSAR passphrase tunnel com `printf '%s' "$KEY_PASS" | ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --key <KEY_PATH> --key-passphrase-stdin`
- DEVE EXECUTAR `ssh-cli health-check <NAME> --json` e com `--timeout <MS>`
- DEVE EXECUTAR `ssh-cli health-check --json` e `ssh-cli --max-concurrency <N> health-check --all --json`
- DEVE EXECUTAR `ssh-cli health-check --hosts <A>,<B> --json`
- DEVE PASSAR senha health com `printf '%s' "$PASS" | ssh-cli health-check <NAME> --json --password-stdin`
- DEVE EXECUTAR `ssh-cli health-check <NAME> --json --key <KEY_PATH>`
- DEVE PASSAR passphrase health com `printf '%s' "$KEY_PASS" | ssh-cli health-check <NAME> --json --key <KEY_PATH> --key-passphrase-stdin`
- DEVE EXECUTAR `ssh-cli health-check <NAME> --json --replace-host-key` somente após review humano
- DEVE EXECUTAR `ssh-cli secrets status --json`
- DEVE EXECUTAR `ssh-cli secrets init` e `ssh-cli secrets init --json`
- DEVE EXECUTAR `ssh-cli secrets init --force --json` e `ssh-cli secrets init --keyring --json`
- DEVE EXECUTAR `ssh-cli secrets reencrypt` e `ssh-cli secrets reencrypt --json`
- DEVE EXECUTAR `ssh-cli --allow-plaintext-secrets --config-dir <DIR> secrets status --json`
- DEVE EXECUTAR `ssh-cli --secrets-key-file <KEY_FILE> secrets status --json`
- DEVE EXECUTAR `ssh-cli --use-keyring secrets status --json`
- DEVE EXECUTAR `ssh-cli --replace-host-key exec <NAME> "true"`
- DEVE EXECUTAR `ssh-cli --config-dir <DIR> vps list --json`
- DEVE PASSAR passphrase exec com `printf '%s' "$KEY_PASS" | ssh-cli exec <NAME> "id" --json --key <KEY_PATH> --key-passphrase-stdin`
- DEVE EXECUTAR `ssh-cli locale show --json`
- DEVE EXECUTAR `ssh-cli locale set pt-BR` e `ssh-cli locale set en` e `ssh-cli locale clear`
- DEVE EXECUTAR `ssh-cli --lang pt-BR vps list --json`
- DEVE EXECUTAR `ssh-cli tls provider --json`
- DEVE EXECUTAR `ssh-cli tls paths --json`
- DEVE EXECUTAR `ssh-cli tls mtls import --name <NAME> --cert <CERT_PEM> --key <KEY_PEM> --json`
- DEVE EXECUTAR `ssh-cli tls mtls list --json`
- DEVE EXECUTAR `ssh-cli tls mtls show <NAME> --json`
- DEVE EXECUTAR `ssh-cli tls mtls remove <NAME>`
- DEVE EXECUTAR `ssh-cli tls acme account create --contact mailto:<EMAIL> --json`
- DEVE EXECUTAR `ssh-cli tls acme account create --contact mailto:<EMAIL> --staging --force --json`
- DEVE EXECUTAR `ssh-cli tls acme account show --json`
- DEVE EXECUTAR `ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --json`
- DEVE EXECUTAR `ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --staging --json`
- DEVE EXECUTAR `ssh-cli tls acme complete --domain <DOMAIN> --json`
- DEVE EXECUTAR `ssh-cli tls acme status --domain <DOMAIN> --json` e `ssh-cli tls acme status --json` e `ssh-cli tls acme list --json`
- DEVE EXECUTAR `ssh-cli commands` e `ssh-cli schema` e `ssh-cli schema <NAME>`
- DEVE EXECUTAR `ssh-cli -v exec <NAME> "true" --json` e `-vv` e `-vvv` para debug
- DEVE EXECUTAR `ssh-cli completions bash`
- DEVE EXECUTAR `ssh-cli completions zsh`
- DEVE EXECUTAR `ssh-cli completions fish`
- DEVE EXECUTAR `ssh-cli completions elvish`
- DEVE EXECUTAR `ssh-cli completions powershell`
- DEVE EXECUTAR `cargo install ssh-cli --locked --force`
- DEVE EXECUTAR `ssh-cli --version`
- DEVE PARSEAR exit, stdout de sucesso e envelope de erro no stderr após cada invocação

## Proibições Absolutas
### PROIBIDO
- NUNCA manter sessões SSH entre turnos exceto tunnel bound até deadline; NUNCA reintroduzir daemons ou telemetria
- NUNCA vazar segredos em argv quando stdin existir; NUNCA imprimir primary-key ou credenciais vivas
- NUNCA abrir tunnel sem `--timeout-ms` nem usar porta antes de `tunnel_listening`
- NUNCA scp de diretórios; árvores = `sftp --recursive` sem symlink; NUNCA inventar `-r` no scp
- NUNCA deixar `.ssh-cli.partial` final; NUNCA inventar senhas para `password` null
- NUNCA `sshcli-enc` para secrets vazios no export; NUNCA pipe `--include-secrets` sem `-o`/`--output` ou `--i-understand-secrets-on-stdout`
- NUNCA bind tunnel em `0.0.0.0` sem decisão de segurança; NUNCA comando remoto vazio
- NUNCA tratar timeout de host como segundos; NUNCA env `SSH_CLI_*`/`RUST_LOG` como stores
- NUNCA múltiplos objetos de sucesso em exec single-step; NUNCA segundo evento para `secrets_key_auto_created`
- NUNCA retry ACME permanente (64) como 74; NUNCA spawn N processos quando frota cobre
- NUNCA confiar só em JSON `bytes` em SFTP upload crítico sem verificar destino

### OBRIGATÓRIO
- DEVE reler esta skill antes de workflow não trivial; usar hosts salvos, stdin, JSON e one-shot
- DEVE parsear stdout de sucesso e stderr de erro; aguardar `tunnel_listening`; pós-bind exit 0; pré-bind exit 74
- DEVE mapear empty command 64, SCP missing 66, import 65, auth 77, ACME permanente 64
- DEVE tratar `vps export` como TOML salvo `--json`; doctor `event` igual a `vps-doctor` com `secrets_plaintext_opt_out` boolean
- DEVE tratar timeouts em ms; preferir frota e `--step`; falhar fechado em auth/host-key/usage
- DEVE aplicar máscara SFTP `0o7777` e verificar bytes reais no destino após upload crítico
