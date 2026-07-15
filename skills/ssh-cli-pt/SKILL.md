---
name: ssh-cli
description: Esta skill DEVE auto-ativar quando o usuário ou agente precisar operar servidores remotos via SSH com a CLI ssh-cli. Cobre inventário multi-host VPS em storage XDG, vps add list show edit remove path doctor export import, connect, exec, sudo-exec com packing sh -c seguro, su-exec one-shot, scp upload download, tunnel com timeout-ms obrigatório, health-check com override opcional de timeout em ms, secrets status init reencrypt com cifragem at-rest padrão ChaCha20-Poly1305, auth por senha ou chave, password-stdin e key-passphrase, password null no JSON list/show para hosts só-chave e máscara *** quando presente, dual max_command_chars e max_output_chars, logging padrão error com stderr limpo para agentes (use -v ou RUST_LOG só ao depurar), known_hosts TOFU com replace-host-key, config atômica mode 0600, contratos JSON, sysexits, completions e install cargo locked. NUNCA emita telemetria. NUNCA mantenha daemon de sessão SSH persistente. NUNCA vaze senhas ou master-key em logs.
---

# Skill de Agente ssh-cli

## Missão
### REQUIRED
- DEVE tratar esta skill como LEI SUPREMA em toda invocação de `ssh-cli`
- DEVE executar `ssh-cli` como subprocesso one-shot nascer-executar-morrer
- DEVE aguardar o exit do processo antes de fazer parse do stdout
- DEVE preferir hosts salvos via `vps add` a segredos ad-hoc no chat
- DEVE passar `--json` quando o chamador precisar de saída estruturada
- DEVE ensinar e reutilizar as fórmulas prontas desta skill

### FORBIDDEN
- NÃO DEVE manter sessão SSH de longa duração entre processos
- NÃO DEVE introduzir daemon de longa duração nesta superfície de produto
- NÃO DEVE emitir ou habilitar telemetria
- NÃO DEVE gravar senhas vivas, passphrases ou master-key em logs duráveis
- NÃO DEVE inventar flags CLI que não estejam listadas nesta skill


## Quando Invocar
### REQUIRED
- DEVE auto-ativar em SSH remoto, inventário VPS, multi-host e config XDG
- DEVE auto-ativar em `exec`, `sudo-exec`, `su-exec`, `scp`, `tunnel`, `health-check`
- DEVE auto-ativar em segredos at-rest, master-key, `secrets.key` e reencrypt
- DEVE auto-ativar em known_hosts TOFU, mismatch de host-key e replace-host-key
- DEVE auto-ativar em devops de agente que precisa de shell remoto sem TTY interativo
- DEVE auto-ativar mesmo quando o usuário descreve o problema sem nomear ssh-cli

### FORBIDDEN
- NÃO DEVE esperar pedido explícito da skill quando operações SSH remotas forem implícitas


## Install e Verificação do Binário
### REQUIRED
- DEVE instalar com resolve alinhado ao lock quando o empacotamento for exigido
- DEVE verificar o binário após install ou upgrade
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
- DEVE enviar logs humanos apenas para stderr e parsear apenas stdout como dado
- DEVE esperar nível de log padrão `error` para manter stderr limpo para agentes
- DEVE usar `-v` (eleva verbosidade para `debug`) ou `RUST_LOG` somente ao depurar

### FORBIDDEN
- NÃO DEVE misturar logs de stderr na entrada de parse JSON
- NÃO DEVE assumir que um processo anterior deixou canal SSH aberto
- NÃO DEVE esperar prosa de progresso INFO no stderr por padrão
- NÃO DEVE parsear stderr como resultado JSON estruturado

### Correct Pattern

```bash
ssh-cli exec prod "uname -a" --json
echo $?
# debug somente ao diagnosticar
ssh-cli -v exec prod "true" --json
RUST_LOG=debug ssh-cli exec prod "true" --json
```


## CRUD do Inventário de Hosts
### REQUIRED
- DEVE registrar cada host com `--name` único
- DEVE fornecer password ou `--key` ou senha via stdin no add
- DEVE mascarar segredos ao exibir list ou show para humanos
- DEVE tratar password vazio ou ausente no JSON de list/show como JSON `null` (host só-chave)
- DEVE tratar password não vazio no JSON de list/show como máscara `***` nunca cru
- DEVE rodar `vps doctor --json` quando a localização do config for desconhecida
- DEVE usar `vps path` para imprimir o path vencedor do config
- DEVE usar `vps export` sem segredos por padrão
- DEVE exigir aprovação humana antes de `export --include-secrets`

### FORBIDDEN
- NÃO DEVE criar hosts com credencial vazia
- NÃO DEVE inventar senhas falsas para hosts só-chave
- NÃO DEVE tratar a máscara `***` como valor real de senha
- NÃO DEVE commitar inventários com segredos crus no git
- NÃO DEVE assumir que arquivos `.env` são lidos em runtime
- NÃO DEVE imprimir segredos decifrados em logs de chat

### Correct Pattern

```bash
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519
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

### Correct Pattern

```bash
ssh-cli connect prod
ssh-cli health-check prod --json
```


## Autenticação
### REQUIRED
- DEVE usar `--key` em hosts cloud só-chave
- DEVE preferir `--password-stdin` quando o history de argv for compartilhado
- DEVE preferir `--sudo-password-stdin` e `--su-password-stdin` a segredos em argv
- DEVE tratar exit 77 como falha de autenticação e mudar credenciais antes de retry
- DEVE passar `--key-passphrase` ou passphrase via stdin somente quando a chave for cifrada
- DEVE esperar `password` no JSON de list/show como `null` em hosts só-chave e `***` quando houver senha armazenada

### FORBIDDEN
- NÃO DEVE inventar senhas falsas para hosts só-chave
- NÃO DEVE tratar `password` JSON `null` como bug ou campo ausente a fabricar
- NÃO DEVE imprimir passphrases de chave ou senhas SSH
- NÃO DEVE gravar segredos no history do shell quando stdin estiver disponível

### Correct Pattern

```bash
ssh-cli vps add --name edge --host edge.example.com --user ubuntu --key ~/.ssh/id_ed25519
printf '%s' "$SSH_PASSWORD" | ssh-cli vps add --name app --host app.example.com --user deploy --password-stdin
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
- DEVE nunca imprimir o valor da master-key

### FORBIDDEN
- NÃO DEVE logar `SSH_CLI_SECRETS_KEY`, conteúdo de key file ou segredos de host decifrados
- NÃO DEVE habilitar plaintext de segredos em fluxos de agente em produção

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
- DEVE fazer parse de `stdout`, `stderr`, `exit_code`, flags de truncagem e `duration_ms` no JSON
- DEVE anexar `--description` quando o history shell remoto se beneficiar de comentário de auditoria
- DEVE elevar `max_command_chars` do host via `vps edit` quando o agente precisar de comandos longos
- DEVE honrar default max_command_chars 1000 e max_output_chars 100000 salvo override

### FORBIDDEN
- NÃO DEVE ignorar flags de truncagem ao resumir saída para o usuário
- NÃO DEVE fazer retry de exit 64 65 66 77 sem mudar inputs

### Correct Pattern

```bash
ssh-cli exec prod "hostname && uptime" --json --description "inventory"
ssh-cli vps edit prod --max-command-chars 4000 --max-output-chars 200000
ssh-cli exec prod "long-agent-command-here" --json
```


## sudo-exec e su-exec
### REQUIRED
- DEVE usar `sudo-exec` para elevação sudo e confiar no packing seguro `sh -c`
- DEVE configurar senha sudo no host ou passar `--sudo-password` ou variante stdin
- DEVE usar `su-exec` somente quando a senha `su` estiver configurada
- DEVE respeitar `--disable-sudo` global e o `disable_sudo` do host
- DEVE tratar elevação como one-shot e nunca assumir shell elevado sticky

### FORBIDDEN
- NÃO DEVE prefixar `sudo` cru em `exec` quando `sudo-exec` existe
- NÃO DEVE assumir shell elevado persistente entre invocações

### Correct Pattern

```bash
ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --json
printf '%s' "$SUDO_PASSWORD" | ssh-cli sudo-exec prod "systemctl restart nginx" --json --sudo-password-stdin
ssh-cli su-exec prod "whoami" --json
ssh-cli --disable-sudo exec prod "id" --json
```


## Transferências Túneis Health
### REQUIRED
- DEVE usar `scp upload` ou `scp download` para cópia de arquivos
- DEVE passar `--timeout-ms` em todo comando `tunnel`
- DEVE usar `health-check` para verificar conectividade após mudanças de host
- DEVE permitir override opcional `--timeout <ms>` em `health-check` quando um deadline não padrão for necessário
- DEVE limitar túneis e encerrar quando o deadline terminar

### FORBIDDEN
- NÃO DEVE abrir túneis sem bound
- NÃO DEVE deixar processos de tunnel deliberadamente detached para sempre
- NÃO DEVE inventar outro nome de flag de timeout para `health-check` (use `--timeout`, não `--timeout-ms`)

### Correct Pattern

```bash
ssh-cli scp upload prod ./app.tgz /tmp/app.tgz
ssh-cli scp download prod /var/log/app.log ./app.log
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 5000 --json
```


## Host Keys e Segurança de Storage
### REQUIRED
- DEVE tratar mismatch de host key como hard stop até confirmação humana de rotação
- DEVE usar `--replace-host-key` somente após confirmação
- DEVE esperar writes atômicos de `config.toml` e mode 0600 em Unix
- DEVE esperar writes atômicos de `secrets.key` e mode 0600 em Unix
- DEVE usar `--config-dir` ou `SSH_CLI_HOME` para sandboxes isolados de agente

### FORBIDDEN
- NÃO DEVE auto-substituir host keys sem aprovação do usuário
- NÃO DEVE desabilitar TOFU por conveniência em fluxos de produção

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

### Correct Pattern

```bash
ssh-cli completions bash
ssh-cli completions zsh
ssh-cli completions fish
```


## Códigos de Saída e Retry
### REQUIRED
- DEVE mapear exits como 0 sucesso, 1 geral, 64 usage, 65 data, 66 not found, 73 cant create, 74 IO ou SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- DEVE fazer no máximo dois retries em 74 com backoff
- DEVE falhar rápido em 64 65 66 77 sem retry cego
- DEVE expor o `exit_code` remoto do JSON separadamente do exit do processo CLI

### FORBIDDEN
- NÃO DEVE engolir exits não zero
- NÃO DEVE confundir falha do comando remoto com falha de usage local da CLI

### Correct Pattern

```bash
ssh-cli exec prod "true" --json
echo $?
ssh-cli exec missing-host "true" --json; echo $?
```


## Contrato de Parse JSON
### REQUIRED
- DEVE parsear somente stdout como JSON quando `--json` estiver ativo
- DEVE ler campos `stdout`, `stderr`, `exit_code`, flags de truncagem e `duration_ms` em resultados da família exec
- DEVE tratar payloads de list show doctor secrets status como objetos tipados e usar só campos documentados
- DEVE tratar `password` em list/show como JSON `null` quando vazio ou ausente (host só-chave)
- DEVE tratar `password` em list/show como string mascarada `***` quando houver senha armazenada
- DEVE reportar truncagem ao usuário quando a saída for cortada por `max_output_chars`

### FORBIDDEN
- NÃO DEVE inventar chaves JSON ausentes
- NÃO DEVE inventar senhas falsas quando `password` for `null`
- NÃO DEVE pretty-print de segredos encontrados em campos inesperados
- NÃO DEVE parsear stderr como dado JSON

### Correct Pattern

```bash
ssh-cli vps list --json
ssh-cli vps show prod --json
# host só-chave => "password": null
# host com senha => "password": "***"
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
7. DEPOIS faça parse do exit code e dos campos JSON somente do stdout antes de responder ao usuário
8. POR FIM nunca deixe segredos ou master-key em logs duráveis

### Correct Pattern

```bash
ssh-cli --version
ssh-cli vps doctor --json
ssh-cli secrets status --json
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519 --check
ssh-cli health-check prod --json
ssh-cli exec prod "uname -a && df -h" --json --description "baseline"
```


## Proibições Absolutas
### FORBIDDEN
- NÃO DEVE manter sessões SSH abertas entre turnos do agente
- NÃO DEVE reintroduzir daemons de longa duração para esta superfície de produto
- NÃO DEVE vazar segredos em argv quando variantes stdin existirem
- NÃO DEVE ignorar mismatch de host-key
- NÃO DEVE abrir tunnels sem `--timeout-ms`
- NÃO DEVE esperar prosa de progresso INFO no stderr por padrão
- NÃO DEVE inventar senhas falsas para hosts só-chave quando o JSON mostrar `null`
- NÃO DEVE documentar changelogs históricos de versão dentro desta skill
- NÃO DEVE inventar histórias de feature versão por versão
- NÃO DEVE colar credenciais vivas em exemplos ou logs


## Folha de Fórmulas Prontas
### REQUIRED
- DEVE copiar estas fórmulas exatamente e somente substituir placeholders

```bash
# inventário
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH>
printf '%s' "$PASS" | ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --password-stdin
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
ssh-cli sudo-exec <NAME> "<CMD>" --json
ssh-cli su-exec <NAME> "<CMD>" --json
ssh-cli scp upload <NAME> <LOCAL> <REMOTE>
ssh-cli scp download <NAME> <REMOTE> <LOCAL>
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS>
ssh-cli health-check <NAME> --json
ssh-cli health-check <NAME> --timeout <MS> --json

# secrets e segurança
ssh-cli secrets status --json
ssh-cli secrets init
ssh-cli secrets reencrypt
ssh-cli --replace-host-key exec <NAME> "true"
ssh-cli --config-dir <DIR> vps list --json

# debug (opcional; nível de log padrão é error)
ssh-cli -v exec <NAME> "true" --json
RUST_LOG=debug ssh-cli exec <NAME> "true" --json

# install
cargo install ssh-cli --locked --force
ssh-cli --version
```


## Lembrete Final
### REQUIRED
- DEVE reler esta skill antes de todo workflow não trivial de ssh-cli
- DEVE preferir hosts salvos, segredos via stdin, saída JSON e execução one-shot
- DEVE parsear somente stdout como JSON e manter stderr quieto por padrão
- DEVE falhar fechado em erros de auth, host-key e usage
- DEVE manter esta skill consolidada apenas como fórmulas operacionais
