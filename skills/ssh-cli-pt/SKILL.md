---
name: ssh-cli
description: Esta skill DEVE auto-ativar para SSH remoto via ssh-cli one-shot mesmo sem nomear ssh-cli. Entradas nome host IP usuário chave ou password-stdin comando paths portas tunnel timeout ms. Saídas sysexits, JSON exec (stdout stderr exit_code truncated_stdout truncated_stderr duration_ms), JSON scp (ok direction vps local remote bytes duration_ms), JSON tunnel (ok event tunnel_listening vps local_port remote_host remote_port timeout_ms), erro stderr (exit_code message remote_exit_code), inventário password null ou ***. Cobre vps CRUD path doctor export import connect exec sudo-exec packing su-exec scp upload download só-arquivo sem -r sem SFTP --json --timeout .ssh-cli.partial rename stream 32KiB preserve mtime mode tunnel --timeout-ms obrigatório --json aguardar tunnel_listening health-check --timeout secrets status init reencrypt --quiet logs error TOFU replace-host-key mode 0600 completions cargo install locked. NUNCA telemetria. NUNCA daemon SSH persistente. NUNCA vaze segredos. NUNCA scp recursivo de dirs.
---

# Skill de Agente ssh-cli

## Missão
### REQUIRED
- DEVE tratar esta skill como LEI SUPREMA em toda invocação de `ssh-cli`
- DEVE SEMPRE executar `ssh-cli` como subprocesso one-shot nascer-executar-morrer
- DEVE aguardar o exit do processo antes de fazer parse do stdout ou stderr exceto no `tunnel` de vida limitada até timeout ou sinal
- DEVE usar hosts salvos via `vps add` em vez de segredos ad-hoc no chat
- DEVE passar `--json` quando o agente precisar de saída estruturada de sucesso
- DEVE copiar as fórmulas prontas desta skill e somente substituir placeholders
- DEVE manter esta skill consolidada apenas como fórmulas operacionais

### FORBIDDEN
- NUNCA DEVE manter sessão SSH de longa duração entre processos exceto o `tunnel` ativo até o deadline
- NUNCA DEVE introduzir daemon de longa duração nesta superfície de produto
- NUNCA DEVE emitir ou habilitar telemetria
- NUNCA DEVE gravar senhas vivas, passphrases ou master-key em logs duráveis
- NUNCA DEVE inventar flags CLI que não estejam listadas nesta skill
- NUNCA DEVE escrever histórias de changelog versão por versão dentro desta skill


## Quando Invocar
### REQUIRED
- DEVE auto-ativar em SSH remoto, inventário VPS, multi-host e config XDG
- DEVE auto-ativar em `exec`, `sudo-exec`, `su-exec`, `scp`, `tunnel`, `health-check`
- DEVE auto-ativar em transferência de arquivo, cópia de arquivo regular via SSH, scp upload ou download
- DEVE auto-ativar em port forward local, tunnel SSH com bound, `tunnel_listening`
- DEVE auto-ativar em segredos at-rest, master-key, `secrets.key` e reencrypt
- DEVE auto-ativar em known_hosts TOFU, mismatch de host-key e replace-host-key
- DEVE auto-ativar em devops de agente que precisa de shell remoto sem TTY interativo
- DEVE auto-ativar mesmo quando o usuário descreve o problema sem nomear ssh-cli

### FORBIDDEN
- NUNCA DEVE esperar pedido explícito da skill quando operações SSH remotas forem implícitas


## Install e Verificação do Binário
### REQUIRED
- DEVE instalar com resolve alinhado ao lock quando o empacotamento for exigido
- DEVE verificar o binário após install ou upgrade antes de confiar em scp ou tunnel
- DEVE recusar orientar usuários a ignorar falhas de pin crypto sem release corrigida

### Correct Pattern

```bash
cargo install ssh-cli --locked --force
ssh-cli --version
ssh-cli --help
```


## Contrato de Ciclo de Vida
### REQUIRED
- DEVE invocar um processo CLI completo por ação de produto
- DEVE tratar stdout não-TTY como JSON por padrão quando `--output-format` for omitido
- DEVE forçar JSON com `--json` ou `--output-format json` para parse de agente
- DEVE enviar logs humanos apenas para stderr e parsear apenas stdout como dado de sucesso
- DEVE esperar nível de log padrão `error` para manter stderr limpo para agentes
- DEVE usar `-v` ou `RUST_LOG` somente ao depurar
- DEVE usar `-q` / `--quiet` para suprimir prosa humana não-JSON quando exigido
- DEVE tratar `scp --json`, `tunnel --json` e formato JSON global como ativadores de envelope de erro no stderr em falha
- DEVE parsear envelopes de falha no JSON de stderr quando o exit do processo for não zero e o modo JSON estiver ativo

### FORBIDDEN
- NUNCA DEVE misturar logs de stderr na entrada de parse JSON de sucesso
- NUNCA DEVE assumir que um processo anterior deixou canal SSH aberto
- NUNCA DEVE esperar prosa de progresso INFO no stderr por padrão
- NUNCA DEVE parsear stderr como JSON de sucesso

### Correct Pattern

```bash
ssh-cli exec prod "uname -a" --json
echo $?
ssh-cli -q exec prod "true" --json
ssh-cli -v exec prod "true" --json
RUST_LOG=debug ssh-cli exec prod "true" --json
```


## CRUD do Inventário de Hosts
### REQUIRED
- DEVE registrar cada host com `--name` único
- DEVE fornecer password ou `--key` ou senha via stdin no add
- DEVE passar `--port` quando a porta SSH não for 22
- DEVE passar `--check` no add quando um probe imediato de conectividade for exigido
- DEVE mascarar segredos ao exibir list ou show para humanos
- DEVE tratar password vazio ou ausente no JSON de list/show como JSON `null` (host só-chave)
- DEVE tratar password não vazio no JSON de list/show como máscara `***` nunca cru
- DEVE tratar `sudo_password`, `su_password` e `key_passphrase` da mesma forma (`null` quando ausente, `***` quando armazenado)
- DEVE rodar `vps doctor --json` quando a localização do config for desconhecida
- DEVE usar `vps path` para imprimir o path vencedor do config
- DEVE usar `vps export` sem segredos por padrão
- DEVE exigir aprovação humana antes de `export --include-secrets`

### FORBIDDEN
- NUNCA DEVE criar hosts com credencial vazia
- NUNCA DEVE inventar senhas falsas para hosts só-chave
- NUNCA DEVE tratar a máscara `***` como valor real de senha
- NUNCA DEVE commitar inventários com segredos crus no git
- NUNCA DEVE assumir que arquivos `.env` são lidos em runtime
- NUNCA DEVE imprimir segredos decifrados em logs de chat

### Correct Pattern

```bash
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519 --port 22 --check
ssh-cli vps list --json
ssh-cli vps show prod --json
ssh-cli vps edit prod --timeout 90000 --max-command-chars 2000 --max-output-chars 100000
ssh-cli vps path
ssh-cli vps doctor --json
ssh-cli vps export -o /tmp/hosts-redacted.toml
ssh-cli vps import --file /tmp/hosts-redacted.toml
ssh-cli vps remove prod
```


## Host Ativo com Connect
### REQUIRED
- DEVE usar `connect` somente para gravar o marcador irmão `active`
- DEVE ainda passar o nome explícito da VPS nos comandos da família exec quando a certeza for exigida
- DEVE permitir `health-check` sem nome somente depois que `connect` definir o host ativo

### Correct Pattern

```bash
ssh-cli connect prod
ssh-cli health-check --json
ssh-cli health-check prod --json
```


## Autenticação
### REQUIRED
- DEVE usar `--key` em hosts cloud só-chave
- DEVE usar `--password-stdin` quando o history de argv for compartilhado
- DEVE usar `--sudo-password-stdin` e `--su-password-stdin` em vez de segredos em argv
- DEVE usar `--key-passphrase-stdin` quando a chave privada for cifrada e o argv precisar permanecer limpo
- DEVE tratar exit 77 como falha de autenticação e mudar credenciais antes de retry
- DEVE esperar `password` no JSON de list/show como `null` em hosts só-chave e `***` quando houver senha armazenada

### FORBIDDEN
- NUNCA DEVE inventar senhas falsas para hosts só-chave
- NUNCA DEVE tratar `password` JSON `null` como bug ou campo ausente a fabricar
- NUNCA DEVE imprimir passphrases de chave ou senhas SSH
- NUNCA DEVE gravar segredos no history do shell quando stdin estiver disponível

### Correct Pattern

```bash
ssh-cli vps add --name edge --host edge.example.com --user ubuntu --key ~/.ssh/id_ed25519
printf '%s' "$SSH_PASSWORD" | ssh-cli vps add --name app --host app.example.com --user deploy --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli exec edge "id" --json --key ~/.ssh/id_ed25519_enc --key-passphrase-stdin
ssh-cli exec edge "id" --json
printf '%s' "$SSH_PASSWORD" | ssh-cli exec app "id" --json --password-stdin
```


## Segredos At-Rest
### REQUIRED
- DEVE tratar a cifragem at-rest como comportamento padrão do produto
- DEVE rodar `secrets status --json` antes de diagnosticar falhas de decrypt
- DEVE rodar `secrets init` quando um master-key explícito em arquivo ou keyring for exigido
- DEVE rodar `secrets reencrypt` após rotacionar o material da master-key
- DEVE restringir `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` apenas a testes automatizados
- NUNCA DEVE imprimir o valor da master-key

### FORBIDDEN
- NUNCA DEVE logar `SSH_CLI_SECRETS_KEY`, conteúdo de key file ou segredos de host decifrados
- NUNCA DEVE habilitar plaintext de segredos em fluxos de agente em produção

### Correct Pattern

```bash
ssh-cli secrets status --json
ssh-cli secrets init
ssh-cli secrets init --keyring
ssh-cli secrets reencrypt
```

### Fórmulas de Precedência de Env
- DEVE resolver master-key somente nesta ordem
- `SSH_CLI_SECRETS_KEY` como 64 caracteres hex
- `SSH_CLI_SECRETS_KEY_FILE` como path com 64 caracteres hex
- OS keyring quando `SSH_CLI_USE_KEYRING=1`
- XDG ou config-dir `secrets.key` auto-criado na primeira gravação de segredo
- Opt-out de plaintext somente com `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`


## Execução Remota
### REQUIRED
- DEVE validar o tamanho do comando contra `max_command_chars` antes de enviar comandos enormes do agente
- DEVE fazer parse de `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr` e `duration_ms` no JSON de sucesso
- DEVE anexar `--description` quando o history shell remoto se beneficiar de comentário de auditoria
- DEVE elevar `max_command_chars` do host via `vps edit` quando o agente precisar de comandos longos
- DEVE honrar default max_command_chars 1000 e max_output_chars 100000 salvo override
- DEVE passar `--timeout <ms>` na família exec quando o deadline padrão do host for curto demais

### FORBIDDEN
- NUNCA DEVE ignorar `truncated_stdout` ou `truncated_stderr` ao resumir saída para o usuário
- NUNCA DEVE fazer retry de exit 64 65 66 77 sem mudar inputs

### Correct Pattern

```bash
ssh-cli exec prod "hostname && uptime" --json --description "inventory"
ssh-cli exec prod "true" --json --timeout 120000
ssh-cli vps edit prod --max-command-chars 4000 --max-output-chars 200000
ssh-cli exec prod "long-agent-command-here" --json
```


## sudo-exec e su-exec
### REQUIRED
- DEVE usar `sudo-exec` para elevação sudo e confiar no packing seguro `sh -c`
- DEVE configurar senha sudo no host ou passar `--sudo-password` ou variante stdin
- DEVE usar `su-exec` somente quando a senha `su` estiver configurada
- DEVE respeitar `--disable-sudo` global e o `disable_sudo` do host
- DEVE tratar elevação como one-shot e NUNCA assumir shell elevado sticky

### FORBIDDEN
- NUNCA DEVE prefixar `sudo` cru em `exec` quando `sudo-exec` existe
- NUNCA DEVE assumir shell elevado persistente entre invocações

### Correct Pattern

```bash
ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --json
printf '%s' "$SUDO_PASSWORD" | ssh-cli sudo-exec prod "systemctl restart nginx" --json --sudo-password-stdin
ssh-cli su-exec prod "whoami" --json
ssh-cli --disable-sudo exec prod "id" --json
```


## Transferências SCP
### REQUIRED
- DEVE usar `scp upload` ou `scp download` somente para cópia de arquivo regular
- DEVE passar `--json` em toda transferência parseada por agente
- DEVE fazer parse do sucesso scp somente no stdout com campos `ok`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`
- DEVE tratar `ok` como true e `direction` somente como `upload` ou `download`
- DEVE usar ordem de argumentos `upload <vps> <local> <remote>` e `download <vps> <remote> <local>`
- DEVE passar `--timeout <ms>` opcional no scp quando connect-plus-transfer precisar de deadline maior
- DEVE preferir `--password-stdin` e `--key-passphrase-stdin` a segredos em argv no scp
- DEVE permitir override `--key` no scp da mesma forma que no exec
- DEVE esperar upload em stream de chunks 32 KiB sem carregar o arquivo inteiro em RAM
- DEVE esperar download gravando path irmão terminando em `.ssh-cli.partial` e depois rename no lugar
- DEVE esperar preserve de mtime e mode nos dois sentidos sem flag extra do usuário
- DEVE parsear falhas duras de scp no envelope de erro do stderr quando o modo JSON estiver ativo

### FORBIDDEN
- NUNCA DEVE passar diretórios como paths local ou remote no scp
- NUNCA DEVE inventar flags recursivas como `-r`
- NUNCA DEVE tratar scp como subsystem SFTP
- NUNCA DEVE usar `--timeout-ms` no scp (essa flag é exclusiva do tunnel)
- NUNCA DEVE parsear sucesso scp como JSON da família exec `stdout`/`stderr`/`exit_code`
- NUNCA DEVE tratar um path `.ssh-cli.partial` residual como artefato final após download completo
- NUNCA DEVE inventar flag de usuário obrigatória para preserve de mtime ou mode

### Correct Pattern

```bash
ssh-cli scp upload prod ./app.tgz /tmp/app.tgz --json
ssh-cli scp download prod /var/log/app.log ./app.log --json
ssh-cli scp upload prod ./big.bin /tmp/big.bin --json --timeout 300000
printf '%s' "$PASS" | ssh-cli scp download prod /etc/app.env ./app.env --json --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli scp upload prod ./payload.bin /tmp/payload.bin --json --key ~/.ssh/id_ed25519_enc --key-passphrase-stdin
# sucesso stdout => {"ok":true,"direction":"upload|download","vps":"...","local":"...","remote":"...","bytes":N,"duration_ms":N}
# exit não zero => stderr {"exit_code":N,"message":"..."} remote_exit_code opcional
```


## Tunnel
### REQUIRED
- DEVE passar `--timeout-ms` em todo comando `tunnel`
- DEVE passar `--json` quando o agente precisar de sinal estruturado de ready
- DEVE aguardar um objeto no stdout com `event` igual a `tunnel_listening` antes de usar a porta local
- DEVE fazer parse dos campos de ready do tunnel `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`
- DEVE deixar o processo do tunnel vivo até o deadline de `--timeout-ms` ou sinal
- DEVE parsear falhas duras de tunnel no envelope de erro do stderr quando o modo JSON estiver ativo
- DEVE permitir override `--key` no tunnel quando exigido

### FORBIDDEN
- NUNCA DEVE abrir túneis sem bound
- NUNCA DEVE deixar processos de tunnel deliberadamente detached para sempre
- NUNCA DEVE usar a porta local antes de `tunnel_listening` quando `--json` estiver ativo
- NUNCA DEVE tratar o start do tunnel como completo só pelo spawn do processo
- NUNCA DEVE usar `--timeout` no lugar de `--timeout-ms` no tunnel
- NUNCA DEVE inventar `--password-stdin` no tunnel se não estiver listado aqui

### Correct Pattern

```bash
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json
# aguardar stdout => {"ok":true,"event":"tunnel_listening","vps":"prod","local_port":18080,"remote_host":"127.0.0.1","remote_port":8080,"timeout_ms":30000}
# depois usar 127.0.0.1:18080; processo permanece vivo até deadline ou SIGINT/SIGTERM
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json --key ~/.ssh/id_ed25519
```


## Health-check
### REQUIRED
- DEVE usar `health-check` para verificar conectividade após mudanças de host
- DEVE passar override opcional `--timeout <ms>` em `health-check` quando um deadline não padrão for necessário
- NUNCA DEVE usar `--timeout-ms` em health-check

### Correct Pattern

```bash
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 5000 --json
ssh-cli health-check --json
```


## Matriz de Flags de Timeout
### REQUIRED
- DEVE passar `--timeout-ms` somente em `tunnel` e SEMPRE como obrigatório
- DEVE passar `--timeout` em `scp`, família exec e `health-check` ao sobrescrever deadlines
- NUNCA DEVE intercambiar `--timeout` e `--timeout-ms` entre subcomandos


## Host Keys e Segurança de Storage
### REQUIRED
- DEVE tratar mismatch de host key como hard stop até confirmação humana de rotação
- DEVE usar `--replace-host-key` somente após confirmação
- DEVE esperar writes atômicos de `config.toml` e mode 0600 em Unix
- DEVE esperar writes atômicos de `secrets.key` e mode 0600 em Unix
- DEVE usar `--config-dir` ou `SSH_CLI_HOME` para sandboxes isolados de agente

### FORBIDDEN
- NUNCA DEVE auto-substituir host keys sem aprovação do usuário
- NUNCA DEVE desabilitar TOFU por conveniência em fluxos de produção

### Correct Pattern

```bash
ssh-cli vps doctor --json
# somente após review humano dos detalhes do mismatch
ssh-cli --replace-host-key exec prod "true"
ssh-cli --config-dir /tmp/ssh-cli-sandbox vps list --json
```


## Completions
### REQUIRED
- DEVE gerar completions de shell a partir do binário no onboarding humano
- DEVE manter automação de agente em flags explícitas e JSON, não em scripts de completion
- DEVE suportar shells bash, zsh, fish, elvish e powershell

### Correct Pattern

```bash
ssh-cli completions bash
ssh-cli completions zsh
ssh-cli completions fish
ssh-cli completions elvish
ssh-cli completions powershell
```


## Códigos de Saída e Retry
### REQUIRED
- DEVE mapear exits como 0 sucesso, 1 geral, 64 usage, 65 data, 66 not found, 73 cant create, 74 IO ou SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- DEVE fazer no máximo dois retries em 74 com backoff
- DEVE falhar rápido em 64 65 66 77 sem retry cego
- DEVE expor o `exit_code` remoto do JSON de sucesso separadamente do exit do processo CLI
- DEVE expor `remote_exit_code` do envelope de erro em stderr quando presente

### FORBIDDEN
- NUNCA DEVE engolir exits não zero
- NUNCA DEVE confundir falha do comando remoto com falha de usage local da CLI

### Correct Pattern

```bash
ssh-cli exec prod "true" --json
echo $?
ssh-cli exec missing-host "true" --json; echo $?
```


## Contrato de Parse JSON
### REQUIRED
- DEVE parsear somente stdout como JSON de sucesso quando o modo JSON estiver ativo e o exit for caminho de sucesso
- DEVE ler campos da família exec `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr`, `duration_ms`
- DEVE ler campos de sucesso scp `ok`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`
- DEVE ler campos de ready do tunnel `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`
- DEVE tratar o `event` do tunnel como a string constante `tunnel_listening`
- DEVE parsear campos do envelope de erro em stderr `exit_code`, `message` e `remote_exit_code` opcional em falhas duras no modo JSON incluindo scp e tunnel
- DEVE tratar payloads de list show doctor secrets status como objetos tipados e usar só campos documentados
- DEVE tratar `password` em list/show como JSON `null` quando vazio ou ausente e como `***` quando armazenado
- DEVE tratar `sudo_password`, `su_password` e `key_passphrase` em list/show como `null` ou `***` da mesma forma
- DEVE reportar truncagem ao usuário quando `truncated_stdout` ou `truncated_stderr` for true

### FORBIDDEN
- NUNCA DEVE inventar chaves JSON ausentes
- NUNCA DEVE inventar senhas falsas quando `password` for `null`
- NUNCA DEVE pretty-print de segredos encontrados em campos inesperados
- NUNCA DEVE parsear stderr como JSON de sucesso
- NUNCA DEVE parsear sucesso scp como campos da família exec
- NUNCA DEVE parsear ready do tunnel como campos da família exec

### Correct Pattern

```bash
ssh-cli vps list --json
ssh-cli vps show prod --json
# host só-chave => "password": null
# host com senha => "password": "***"
ssh-cli exec prod "uname -a" --json
# sucesso exec => stdout/stderr/exit_code/truncated_*/duration_ms
ssh-cli scp upload prod ./f.bin /tmp/f.bin --json
# sucesso scp => ok/direction/vps/local/remote/bytes/duration_ms
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 10000 --json
# ready tunnel => ok/event/vps/local_port/remote_host/remote_port/timeout_ms
```


## Variáveis de Ambiente
### REQUIRED
- DEVE usar `SSH_CLI_HOME` para sobrescrever o diretório base de config em testes
- DEVE usar `SSH_CLI_LANG` ou `--lang` para forçar locale
- DEVE usar `SSH_CLI_SECRETS_KEY` somente como master-key de 64 hex e nunca logá-la
- DEVE usar `SSH_CLI_SECRETS_KEY_FILE` quando a master-key estiver em arquivo
- DEVE usar `SSH_CLI_USE_KEYRING=1` quando storage no OS keyring for exigido
- DEVE reservar `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` apenas para testes
- DEVE usar `RUST_LOG` somente ao depurar; o padrão permanece nível error sem ela

### Correct Pattern

```bash
SSH_CLI_HOME=/tmp/ssh-cli-test ssh-cli vps doctor --json
SSH_CLI_LANG=pt-BR ssh-cli --help
RUST_LOG=debug ssh-cli -v exec prod "true" --json
```


## Workflow do Agente
### REQUIRED
1. PRIMEIRO verifique o binário com `ssh-cli --version`
2. DEPOIS inspecione o config com `ssh-cli vps doctor --json` e `ssh-cli vps path`
3. DEPOIS garanta a camada de secrets com `ssh-cli secrets status --json`
4. DEPOIS registre ou edite o host com credenciais password-ou-key
5. DEPOIS rode `ssh-cli health-check <name> --json` (adicione `--timeout <ms>` quando necessário)
6. DEPOIS rode `exec` ou `sudo-exec` ou `su-exec` com `--json`
7. DEPOIS para transferência de arquivo rode `scp upload|download` com `--json` e parseie campos scp-transfer
8. DEPOIS para port forward rode `tunnel` com `--timeout-ms` obrigatório e `--json`; aguarde `tunnel_listening` antes de usar
9. DEPOIS faça parse do exit do processo, do schema de sucesso no stdout da família do comando ou do envelope de erro no stderr antes de responder ao usuário
10. POR FIM NUNCA deixe segredos ou master-key em logs duráveis

### Correct Pattern

```bash
ssh-cli --version
ssh-cli vps doctor --json
ssh-cli secrets status --json
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519 --check
ssh-cli health-check prod --json
ssh-cli exec prod "uname -a && df -h" --json --description "baseline"
ssh-cli scp upload prod ./artifact.tgz /tmp/artifact.tgz --json
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json
```


## Proibições Absolutas
### FORBIDDEN
- NUNCA DEVE manter sessões SSH abertas entre turnos do agente exceto tunnel ativo até o deadline
- NUNCA DEVE reintroduzir daemons Node ou de protocolo de longa duração para esta superfície de produto
- NUNCA DEVE vazar segredos em argv quando variantes stdin existirem
- NUNCA DEVE ignorar mismatch de host-key
- NUNCA DEVE abrir tunnels sem `--timeout-ms`
- NUNCA DEVE usar a porta local do tunnel antes de `tunnel_listening` quando o modo JSON estiver ativo
- NUNCA DEVE fazer scp de diretórios ou inventar transferência recursiva
- NUNCA DEVE tratar JSON de sucesso scp como campos da família exec
- NUNCA DEVE deixar paths `.ssh-cli.partial` de download como entregável final após sucesso
- NUNCA DEVE esperar prosa de progresso INFO no stderr por padrão
- NUNCA DEVE inventar senhas falsas para hosts só-chave quando o JSON mostrar `null`
- NUNCA DEVE documentar changelogs históricos de versão dentro desta skill
- NUNCA DEVE inventar histórias de feature versão por versão
- NUNCA DEVE colar credenciais vivas em exemplos ou logs


## Folha de Fórmulas Prontas
### REQUIRED
- DEVE copiar estas fórmulas exatamente e somente substituir placeholders

```bash
# inventário
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --port <PORT> --check
printf '%s' "$PASS" | ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --password-stdin
printf '%s' "$SUDO" | ssh-cli vps edit <NAME> --sudo-password-stdin
ssh-cli vps list --json
ssh-cli vps show <NAME> --json
ssh-cli vps edit <NAME> --timeout <MS> --max-command-chars <N> --max-output-chars <N>
ssh-cli vps doctor --json
ssh-cli vps path
ssh-cli vps export -o <FILE>
ssh-cli vps import --file <FILE>
ssh-cli connect <NAME>

# ops remotas
ssh-cli exec <NAME> "<CMD>" --json
ssh-cli exec <NAME> "<CMD>" --json --timeout <MS> --description "<AUDIT>"
ssh-cli -q exec <NAME> "<CMD>" --json
ssh-cli sudo-exec <NAME> "<CMD>" --json
printf '%s' "$SUDO" | ssh-cli sudo-exec <NAME> "<CMD>" --json --sudo-password-stdin
ssh-cli su-exec <NAME> "<CMD>" --json

# transferências scp (somente arquivos regulares; agente DEVE usar --json)
ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json
ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json
ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --timeout <MS>
printf '%s' "$PASS" | ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --key <KEY_PATH> --key-passphrase-stdin

# tunnel (--timeout-ms obrigatório; aguardar tunnel_listening antes de usar)
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json

# health
ssh-cli health-check <NAME> --json
ssh-cli health-check <NAME> --timeout <MS> --json
ssh-cli health-check --json

# secrets e segurança
ssh-cli secrets status --json
ssh-cli secrets init
ssh-cli secrets reencrypt
ssh-cli --replace-host-key exec <NAME> "true"
ssh-cli --config-dir <DIR> vps list --json
printf '%s' "$KEY_PASS" | ssh-cli exec <NAME> "id" --json --key <KEY_PATH> --key-passphrase-stdin

# debug (opcional; nível de log padrão é error)
ssh-cli -v exec <NAME> "true" --json
RUST_LOG=debug ssh-cli exec <NAME> "true" --json

# completions
ssh-cli completions bash
ssh-cli completions zsh
ssh-cli completions fish
ssh-cli completions elvish
ssh-cli completions powershell

# install
cargo install ssh-cli --locked --force
ssh-cli --version
```


## Lembrete Final
### REQUIRED
- DEVE reler esta skill antes de todo workflow não trivial de ssh-cli
- DEVE usar hosts salvos, segredos via stdin, saída JSON e execução one-shot
- DEVE parsear somente stdout como JSON de sucesso e manter stderr quieto por padrão
- DEVE parsear envelopes de erro no stderr em falhas duras incluindo scp e tunnel
- DEVE aguardar `tunnel_listening` antes de usar a porta local do tunnel
- DEVE tratar scp como somente arquivos regulares com download partial-then-rename
- DEVE falhar fechado em erros de auth, host-key e usage
- DEVE manter esta skill consolidada apenas como fórmulas operacionais
