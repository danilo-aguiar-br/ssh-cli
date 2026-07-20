---
name: ssh-cli
description: Esta skill DEVE auto-ativar quando inventário VPS SSH, config XDG, exec/sudo-exec/su-exec, scp file-only, sftp, tunnel_listening, health-check, secrets, multi-host --all/--hosts/--tags, --step mesma sessão, TLS mTLS ACME, locale, schema/commands, frota batch ou devops de agente sem TTY forem implícitos. DEVE cobrir CRUD vps (add list remove edit show path doctor export import), connect, auth (password-stdin key passphrase-stdin --use-agent --agent-socket), secrets init/status/reencrypt, doctor event igual a vps-doctor, tunnel --timeout-ms, scp/sftp lotes, --fail-fast, --scp-file-concurrency, --max-concurrency, completions, flags globais, auth exit 77, empty command exit 64, scp missing exit 66, import ruim exit 65, ACME validation permanente exit 64. Preferir --json e --*-stdin. NUNCA telemetria, daemon ou segredos em stdout sem guarda.
---

# Skill de Agente ssh-cli

## Missão
### REQUIRED
- DEVE tratar esta skill como LEI SUPREMA em toda invocação de `ssh-cli`
- DEVE SEMPRE executar `ssh-cli` como subprocesso one-shot nascer-executar-morrer
- DEVE aguardar o exit do processo antes de parse de stdout ou stderr exceto no `tunnel` de vida limitada até timeout ou sinal
- DEVE usar hosts salvos via `vps add` em vez de segredos ad-hoc no chat
- DEVE passar `--json` quando o agente precisar de saída estruturada de sucesso
- DEVE copiar as fórmulas prontas desta skill e somente substituir placeholders
- DEVE executar os Prompts de Ação e a Folha de Fórmulas antes de improvisar flags
- DEVE preferir flags de segredo via stdin a flags de segredo em argv
- DEVE preferir flags de frota `--all` / `--hosts` / `--tags` a N spawns single-host

### FORBIDDEN
- NUNCA DEVE manter sessão SSH de longa duração entre processos exceto o `tunnel` bound ativo até o deadline
- NUNCA DEVE preferir flags de segredo em argv quando variantes stdin existirem
- NUNCA DEVE introduzir daemon de longa duração nesta superfície de produto
- NUNCA DEVE emitir ou habilitar telemetria
- NUNCA DEVE gravar senhas vivas, passphrases ou material de primary-key em logs
- NUNCA DEVE inventar flags CLI que não estejam listadas nesta skill
- NUNCA DEVE escrever histórias de changelog versão por versão dentro desta skill
- NUNCA DEVE tratar `RUST_LOG` ambiente como store de log (o produto ignora; só `-v`)

## Quando Invocar
### REQUIRED
- DEVE auto-ativar em SSH remoto, inventário VPS, multi-host e config XDG
- DEVE auto-ativar em `exec`, `sudo-exec`, `su-exec`, `scp`, `sftp`, `tunnel`, `health-check`
- DEVE auto-ativar em transferência de arquivo regular via SSH, scp upload ou download, árvores sftp
- DEVE auto-ativar em port forward local, tunnel SSH com bound, `tunnel_listening`
- DEVE auto-ativar em segredos at-rest, primary-key, `secrets.key` e reencrypt
- DEVE auto-ativar em known_hosts TOFU, mismatch de host-key e replace-host-key
- DEVE auto-ativar em TLS, mTLS, ACME, locale, schema e commands
- DEVE auto-ativar em devops de agente que precisa de shell remoto sem TTY interativo
- DEVE auto-ativar mesmo quando o usuário descreve o problema sem nomear ssh-cli

### FORBIDDEN
- NUNCA DEVE esperar pedido explícito da skill quando operações SSH remotas forem implícitas

## Prompts de Ação
### REQUIRED
- DEVE seguir esta ordem de execução em toda tarefa remota não trivial
1. VERIFIQUE o binário com `ssh-cli --version`
2. DESCUBRA contratos com `ssh-cli schema` / `ssh-cli commands`; INSPECIONE o config com `ssh-cli doctor --json` (ou `vps doctor --json`) e `ssh-cli vps path`
3. GARANTA a camada de secrets com `ssh-cli secrets status --json`
4. REGISTRE ou edite o host com credenciais password **ou** key **ou** `--use-agent` / `--agent-socket`; tags com `--tag`; TLS com `--tls` / `--tls-sni` / `--tls-client-cert` / `--tls-client-key` quando exigido
5. PROBE a conectividade com `ssh-cli health-check <name> --json` ou, para o inventário inteiro, `ssh-cli health-check --all --json`
6. EXECUTE trabalho remoto com `exec` ou `sudo-exec` ou `su-exec` e `--json`; para frota DEVE preferir `exec|sudo-exec|su-exec --all '<CMD>' --json` (sessões concorrentes com bound) a N spawns single-host; para vários comandos na mesma sessão DEVE usar `--step`
7. TRANSFIRA arquivos regulares com `scp upload|download` e `--json`; para árvores / FS remoto DEVE usar `sftp upload|download --recursive` ou `sftp ls|mkdir|rmdir|rm|stat|rename`; frota com `--all`/`--hosts`
8. FAÇA port forward somente com `tunnel` mais `--timeout-ms` obrigatório e `--json`
9. PARSEIE o exit do processo, o stdout de sucesso da família do comando ou o envelope de erro no stderr (`exit_code`, `message`, `remote_exit_code`, `retryable`, `error_class`, `suggestion`)
10. SANITIZE todos os logs duráveis para que segredos e primary-key nunca permaneçam

### FORBIDDEN
- NUNCA DEVE pular o parse JSON após exit não zero em modo JSON
- NUNCA DEVE responder ao usuário antes de ler o exit code do processo

## Catálogo Completo de Comandos
### REQUIRED
- DEVE tratar a árvore abaixo como a superfície OBRIGATÓRIA do produto (descoberta via `ssh-cli commands`)

| Comando | Superfície |
| --- | --- |
| `vps add` | registra host |
| `vps list` | lista inventário (máscara) |
| `vps remove` | remove host |
| `vps edit` | edita campos do host |
| `vps show` | detalha host (máscara) |
| `vps path` | path vencedor do config |
| `vps doctor` | diagnóstico XDG/schema (+ `--probe-ssh`) |
| `vps export` | exporta inventário (TOML default) |
| `vps import` | importa TOML/JSON |
| `connect` | grava marcador `active` |
| `exec` | comando remoto one-shot |
| `sudo-exec` | elevação sudo one-shot |
| `su-exec` | elevação `su -` one-shot |
| `scp upload` | upload arquivo regular |
| `scp download` | download arquivo regular |
| `sftp upload` | upload SFTP (arquivo/árvore) |
| `sftp download` | download SFTP (arquivo/árvore) |
| `sftp ls` | lista dir remoto |
| `sftp mkdir` | cria dir remoto |
| `sftp rmdir` | remove dir remoto vazio |
| `sftp rm` | remove arquivo remoto |
| `sftp stat` | metadata de path remoto |
| `sftp rename` | renomeia path remoto |
| `tunnel` | port forward bound |
| `health-check` | probe SSH single/frota |
| `secrets status` | status at-rest |
| `secrets init` | gera primary-key |
| `secrets reencrypt` | reescreve secrets |
| `completions` | bash zsh fish elvish powershell |
| `commands` | árvore JSON de comandos |
| `schema` | catálogo ou body de schema |
| `doctor` | alias root de `vps doctor` |
| `locale show` | locale resolvido |
| `locale set` | persiste preferência de idioma |
| `locale clear` | remove preferência |
| `tls provider` | status rustls/`aws_lc_rs` |
| `tls paths` | layout XDG TLS |
| `tls mtls list` | lista identidades mTLS |
| `tls mtls import` | importa cert+key PEM |
| `tls mtls show` | paths de identidade |
| `tls mtls remove` | remove identidade |
| `tls acme account create` | cria conta ACME |
| `tls acme account show` | mostra conta ACME |
| `tls acme issue` | inicia ordem DNS-01 |
| `tls acme complete` | completa ordem pendente |
| `tls acme status` | status de cert/domínio |
| `tls acme list` | lista domínios ACME |

### FORBIDDEN
- NUNCA DEVE inventar subcomandos fora desta tabela
- NUNCA DEVE omitir `sftp rmdir`, `locale`, `tls`, `schema`, `commands` ou `doctor` root da superfície documentada

## Flags Globais
### REQUIRED
- DEVE conhecer e usar quando aplicável todas as flags globais
  - `--lang <LOCALE>` — força idioma BCP47 (`en`, `en-US`, `pt-BR`, `pt`); negocia para `en` ou `pt-BR`
  - `-v` / `--verbose` — sobe verbosidade no stderr (único controle de log; `RUST_LOG` é ignorado)
  - `-q` / `--quiet` — suprime prosa humana não-JSON
  - `--config-dir <DIR>` — sobrescreve diretório de config (testes/sandbox)
  - `--no-color` — desliga cor
  - `--output-format text|json` — formato global; non-TTY default JSON em comandos gerais
  - `--json` — força JSON (alias de `--output-format json`)
  - `--disable-sudo` — desliga elevação nesta invocação
  - `--replace-host-key` — substitui host-key divergente no TOFU (após aprovação humana)
  - `--allow-plaintext-secrets` — opt-out de cifragem at-rest (somente testes)
  - `--secrets-key-file <PATH>` — primary-key em arquivo 64 hex
  - `--use-keyring` — prefere OS keyring para primary-key
  - `--timeout <MS>` — default global de timeout SSH (exec/scp/health-check); local vence; tunnel exige `--timeout-ms`
  - `--max-concurrency <N>` — cap de fan-out multi-host e accepts de tunnel (1..=64; auto se omitido)
  - `--fail-fast` — para admissão multi-host no primeiro erro
  - `--scp-file-concurrency <N>` — canais SCP paralelos na **mesma** sessão (default 1 serial)

### FORBIDDEN
- NUNCA DEVE usar env de produto para HOME/LANG/FORCE_TEXT/MAX_CONCURRENCY/SECRETS
- NUNCA DEVE tratar `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` / `SSH_CLI_MAX_CONCURRENCY` / `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` como stores

## Install Completions e Verificação do Binário
### REQUIRED
- DEVE instalar com resolve alinhado ao lock quando o empacotamento for exigido
- DEVE verificar o binário após install ou upgrade antes de confiar em scp ou tunnel
- DEVE gerar completions de shell a partir do binário no onboarding humano
- DEVE manter automação de agente em flags explícitas e JSON, não em scripts de completion
- DEVE suportar shells bash, zsh, fish, elvish e powershell

### Correct Pattern

```bash
cargo install ssh-cli --locked --force
ssh-cli --version
ssh-cli completions bash
ssh-cli completions zsh
ssh-cli completions fish
ssh-cli completions elvish
ssh-cli completions powershell
```

## Contrato de Ciclo de Vida
### REQUIRED
- DEVE invocar um processo CLI completo por ação de produto
- DEVE tratar stdout não-TTY como JSON por padrão quando `--output-format` for omitido em comandos gerais
- DEVE NÃO afirmar que JSON automático em non-TTY se aplica a `vps export` — o corpo do export permanece TOML salvo `vps export --json`
- DEVE forçar JSON com `--json` ou `--output-format json` para parse de agente em comandos que não sejam export
- DEVE enviar logs humanos apenas para stderr e parsear apenas stdout como dado de sucesso
- DEVE esperar nível de log padrão `error` para manter stderr limpo para agentes
- DEVE usar `-v` somente ao depurar (`RUST_LOG` ambiente é ignorado)
- DEVE usar `-q` / `--quiet` para suprimir prosa humana não-JSON quando exigido
- DEVE tratar `scp --json`, `tunnel --json` e formato JSON global como ativadores de envelope de erro no stderr em falha
- DEVE parsear envelopes de falha no JSON de stderr quando o exit do processo for não zero e o modo JSON estiver ativo
- DEVE parsear sucesso JSON de CRUD via `emit_success` com eventos `vps-added` `vps-edited` `vps-removed` `vps-connected` `vps-import` quando o modo JSON estiver ativo
- DEVE parsear `secrets_key_auto_created` no **mesmo** documento JSON `vps-added` (nunca um segundo evento NDJSON)

### FORBIDDEN
- NUNCA DEVE misturar logs de stderr na entrada de parse JSON de sucesso
- NUNCA DEVE assumir que um processo anterior deixou canal SSH aberto
- NUNCA DEVE esperar prosa de progresso INFO no stderr por padrão
- NUNCA DEVE parsear stderr como JSON de sucesso
- NUNCA DEVE tratar JSON automático em non-TTY como aplicável ao corpo padrão de `vps export`
- NUNCA DEVE esperar um segundo evento NDJSON após auto-criação de primary-key

### Correct Pattern

```bash
ssh-cli exec prod "uname -a" --json
echo $?
ssh-cli -q exec prod "true" --json
```

## CRUD do Inventário e Export-Import
### REQUIRED
- DEVE registrar cada host com `--name` único
- DEVE fornecer password ou `--key` ou senha via stdin **ou** `--use-agent` / `--agent-socket` no add (exatamente uma auth primária)
- DEVE passar `--port` quando a porta SSH não for 22
- DEVE passar `--check` no add quando um probe imediato de conectividade for exigido
- DEVE passar `--tag <TAG>` (repetível) no add para seleção de frota por tags
- DEVE passar `--tls` e opcionalmente `--tls-sni` / `--tls-client-cert` / `--tls-client-key` no add quando SSH-over-TLS for exigido
- DEVE mascarar segredos ao exibir list ou show para humanos
- DEVE tratar password vazio ou ausente no JSON de list/show como JSON `null` (host só-chave)
- DEVE tratar password não vazio no JSON de list/show como máscara `***` (`FIXED_MASK`) nunca cru
- DEVE tratar `sudo_password`, `su_password` e `key_passphrase` da mesma forma (`null` quando ausente, `***` quando armazenado)
- DEVE rodar `vps doctor --json` ou root `doctor --json` quando a localização do config for desconhecida
- DEVE parsear doctor como root `event` igual a `vps-doctor` com `local` e `ssh_probe`
- DEVE usar `vps path` para imprimir o path vencedor do config
- DEVE parsear em doctor JSON o campo `secrets_plaintext_opt_out` como boolean, além de `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`
- DEVE tratar `added_at` como presente em list, show e export
- DEVE aceitar payloads de import que omitam `added_at`; serde preenche o default
- DEVE tratar valores de `--timeout` de host/vps como milissegundos; valores menores que 1000 emitem warning no stderr
- DEVE tratar o corpo de `vps export` como TOML por padrão mesmo em pipe ou non-TTY
- DEVE usar `vps export --json` somente para o envelope de agente com `event` igual a `vps-export`
- DEVE usar `vps export` sem segredos por padrão
- DEVE tratar `vps export` redacted como nunca contendo ciphertext `sshcli-enc` para secrets limpos ou vazios
- DEVE tratar secrets vazios no export redacted como strings vazias; secrets não vazios redacted como `***` (`FIXED_MASK`)
- DEVE exigir aprovação humana antes de `export --include-secrets`
- NUNCA DEVE passar `--include-secrets` para pipe sem `--output`/`-o` ou `--i-understand-secrets-on-stdout`
- DEVE aceitar import TOML com chaves EN mais aliases PT e envelopes JSON `vps-export` (schema v3 dual-read)
- DEVE usar `--allow-incomplete` para import de skeleton redacted quando hosts não tiverem auth completa
- DEVE tratar TOML de import inválido como exit `65`
- DEVE parsear eventos JSON de CRUD `vps-added` `vps-edited` `vps-removed` `vps-connected` `vps-import` quando o modo JSON estiver ativo
- DEVE filtrar `vps list --tag <TAG>` quando somente um subconjunto por tag for necessário

### FORBIDDEN
- NUNCA DEVE criar hosts com credencial vazia
- NUNCA DEVE inventar senhas falsas para hosts só-chave
- NUNCA DEVE tratar a máscara `***` como valor real de senha
- NUNCA DEVE commitar inventários com segredos crus no git
- NUNCA DEVE assumir que arquivos `.env` são lidos em runtime
- NUNCA DEVE imprimir segredos decifrados em logs de chat
- NUNCA DEVE esperar blobs `sshcli-enc` para secrets vazios no export redacted
- NUNCA DEVE enviar `--include-secrets` para stdout sem `--output`/`-o` ou `--i-understand-secrets-on-stdout`
- NUNCA DEVE tratar o corpo padrão de `vps export` como JSON sem `--json` no export
- NUNCA DEVE tratar timeout de host como segundos quando a unidade do produto é milissegundos

### Correct Pattern

```bash
ssh-cli vps add --name prod --host prod.example.com --user deploy --key ~/.ssh/id_ed25519 --port 22 --tag prod --tag web --check
ssh-cli vps add --name edge --host edge.example.com --user deploy --use-agent --agent-socket /run/user/1000/ssh-agent.sock
ssh-cli vps add --name tlsbox --host tls.example.com --user deploy --key ~/.ssh/id_ed25519 --tls --tls-sni tls.example.com
ssh-cli vps list --json
ssh-cli vps list --tag prod --json
ssh-cli vps show prod --json
ssh-cli vps edit prod --timeout 90000 --max-command-chars 2000 --max-output-chars 100000
ssh-cli vps doctor --json
ssh-cli doctor --json
ssh-cli vps path
ssh-cli vps export -o /tmp/hosts.toml
ssh-cli vps export --json
ssh-cli vps export --include-secrets -o /tmp/hosts-secrets.toml
ssh-cli vps import --file /tmp/hosts.toml
ssh-cli vps import --file /tmp/hosts-redacted.toml --allow-incomplete
ssh-cli vps remove prod
```

## Host Ativo com Connect
### REQUIRED
- DEVE usar `connect` somente para gravar o marcador irmão `active`
- DEVE ainda passar o nome explícito da VPS nos comandos da família exec quando a certeza for exigida
- DEVE executar `health-check` sem nome somente depois que `connect` definir o host ativo

### FORBIDDEN
- NUNCA DEVE tratar `connect` como sessão SSH aberta
- NUNCA DEVE assumir host ativo sem `connect` prévio quando o comando omitir o nome

### Correct Pattern

```bash
ssh-cli connect prod
ssh-cli health-check --json
```

## Autenticação
### REQUIRED
- DEVE usar `--key` em hosts cloud só-chave
- DEVE usar `--password-stdin` quando o history de argv for compartilhado
- DEVE usar `--sudo-password-stdin` e `--su-password-stdin` em vez de segredos em argv
- DEVE usar `--key-passphrase-stdin` quando a chave privada for cifrada e o argv precisar permanecer limpo
- DEVE tratar `--key-passphrase <VAL>` como override argv válido e DEVE preferir stdin a argv
- DEVE usar `--use-agent` e opcionalmente `--agent-socket` quando a auth primária for ssh-agent
- DEVE esperar que valores password-like em argv emitam warning no stderr; DEVE preferir `--password-stdin` `--key-passphrase-stdin` `--sudo-password-stdin` `--su-password-stdin`
- DEVE tratar exit 77 como falha de autenticação e mudar credenciais antes de retry
- DEVE esperar `password` no JSON de list/show como `null` em hosts só-chave e `***` quando houver senha armazenada
- DEVE aplicar os mesmos overrides de auth em `exec`, `scp`, `sftp`, `tunnel` e `health-check` quando as credenciais salvas do host forem insuficientes

### FORBIDDEN
- NUNCA DEVE inventar senhas falsas para hosts só-chave
- NUNCA DEVE tratar `password` JSON `null` como bug ou campo ausente a fabricar
- NUNCA DEVE imprimir passphrases de chave ou senhas SSH
- NUNCA DEVE gravar segredos no history do shell quando stdin estiver disponível
- NUNCA DEVE combinar password/key com `--use-agent` no add (mutuamente exclusivos)

### Correct Pattern

```bash
printf '%s' "$SSH_PASSWORD" | ssh-cli vps add --name app --host app.example.com --user deploy --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli exec edge "id" --json --key ~/.ssh/id_ed25519_enc --key-passphrase-stdin
ssh-cli exec edge "id" --json --use-agent --agent-socket /run/user/1000/ssh-agent.sock
```

## Segredos At-Rest
### REQUIRED
- DEVE tratar a cifragem at-rest como comportamento padrão do produto
- DEVE usar o termo de produto primary-key para a chave de cifragem at-rest
- DEVE aceitar o alias legado de keyring `secrets-master-key` somente como leitura legada junto ao canônico `secrets-primary-key`
- DEVE preferir flags CLI `--allow-plaintext-secrets` `--secrets-key-file` `--use-keyring` (sem stores em env)
- DEVE rodar `secrets status --json` antes de diagnosticar falhas de decrypt
- DEVE rodar `secrets init` quando uma primary-key explícita em arquivo ou keyring for exigida
- DEVE rodar `secrets init --json` quando o agente precisar do envelope de sucesso `secrets-init`
- DEVE rodar `secrets init --keyring --json` quando a primary-key DEVE ir para o OS keyring em vez de `secrets.key`
- DEVE rodar `secrets init --force --json` somente ao rotacionar de propósito e reescrever secrets sob nova chave
- DEVE rodar `secrets reencrypt` após rotacionar o material da primary-key
- DEVE rodar `secrets reencrypt --json` quando o agente precisar do envelope de sucesso `secrets-reencrypt`
- DEVE parsear eventos JSON `secrets-init` / `secrets-reencrypt`; no 1º `vps add` com auto-key parsear UM documento `vps-added` com campo `secrets_key_auto_created` (nunca um segundo evento)
- DEVE restringir plaintext de secrets apenas a testes via `--allow-plaintext-secrets`
- NUNCA DEVE imprimir material de primary-key ou conteúdo de key file
- DEVE resolver primary-key somente nesta ordem
  1. CLI `--secrets-key-file` como path com 64 caracteres hex
  2. OS keyring quando `--use-keyring` (leitura aceita `secrets-primary-key` e depois legado `secrets-master-key`)
  3. XDG ou config-dir `secrets.key` auto-criado na primeira gravação de segredo; campo `secrets_key_auto_created` no mesmo JSON `vps-added`
- DEVE tratar `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` como **rejeitadas fail-closed** (não são store)
- Opt-out de plaintext somente com `--allow-plaintext-secrets` em testes
- DEVE usar `--config-dir` para sobrescrever o diretório base de config em testes (não `SSH_CLI_HOME`)
- DEVE usar `--lang` ou `locale set` para forçar locale (não `SSH_CLI_LANG`)
- DEVE usar `-v` somente ao depurar; `RUST_LOG` ambiente é ignorado; o padrão permanece nível error

### FORBIDDEN
- NUNCA DEVE logar material de primary-key, conteúdo de key file ou segredos de host decifrados
- NUNCA DEVE imprimir material de chave de `secrets init` ou `secrets reencrypt`
- NUNCA DEVE habilitar plaintext de segredos em fluxos de agente em produção
- NUNCA DEVE tratar `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` / `SSH_CLI_MAX_CONCURRENCY` como stores de config de produto
- NUNCA DEVE confiar em env de secrets rejeitadas fail-closed

### Correct Pattern

```bash
ssh-cli secrets status --json
ssh-cli secrets init --json
ssh-cli secrets init --force --json
ssh-cli secrets reencrypt --json
ssh-cli --secrets-key-file /tmp/primary.key secrets status --json
ssh-cli --use-keyring secrets status --json
ssh-cli --allow-plaintext-secrets --config-dir /tmp/ssh-cli-test secrets status --json
ssh-cli --config-dir /tmp/ssh-cli-test vps doctor --json
```

## Frota Multi-host
### REQUIRED
- DEVE preferir flags de frota a N spawns single-host (um processo, sessões SSH concorrentes com gate de admissão)
- DEVE usar `--all` / `--hosts` em `exec` / `sudo-exec` / `su-exec` / `scp` / `sftp` / `health-check`
- DEVE usar `--tags` SOMENTE em `exec` / `sudo-exec` / `su-exec` para seleção OR de tags (`health-check`, `scp` e `sftp` NÃO têm `--tags`)
- DEVE usar `--hosts` quando só um subconjunto do inventário for necessário (JSON batch mesmo com um nome)
- DEVE tratar `--all`, `--hosts` e `--tags` como mutuamente exclusivos onde aplicável
- DEVE tratar `tunnel` como single-host; multi-host = N one-shots
- DEVE usar `--fail-fast` quando a frota DEVE parar de admitir novos hosts após o primeiro erro
- DEVE usar `--max-concurrency N` global (1..=64) como cap de fan-out e accepts de tunnel (auto = CPUs×4 vs RAM livre/2 / 16 MiB quando omitido)
- DEVE usar `--scp-file-concurrency N` para canais SCP paralelos na mesma sessão (default 1)
- PODE usar `vps doctor --probe-ssh [--hosts a,b] --json` para diagnóstico local + health multi-host em **um** root `event` igual a `vps-doctor` (`ssh_probe`)
- PODE usar SCP multi-arquivo single-host: `scp upload <VPS> f1 f2 … <REMOTE_DIR>` (**uma sessão**, arquivos seriais ou com `--scp-file-concurrency`)
- PODE usar multi-host × multi-arquivo: `scp upload --all f1 f2 … <REMOTE_DIR>` ou `--hosts a,b` (bound de **sessões** por host)
- DEVE parsear JSON multi-host via schemas batch: `health-check-batch` / `exec-batch` / `scp-batch` / `sftp-batch`; o envelope inclui `max_concurrency`
- DEVE tratar inventário vazio + `--all` / `--hosts` (ou `--tags` na família exec) como exit de usage 64
- DEVE manter formas single-host quando o alvo for um nome explícito de VPS

### FORBIDDEN
- NUNCA DEVE assumir multi-host sequencial como padrão quando flags de frota estiverem disponíveis
- NUNCA DEVE inventar `--tags` em `health-check`, `scp` ou `sftp`
- NUNCA DEVE spawn um processo por host para frota quando flags de frota puderem cobrir o inventário
- NUNCA DEVE combinar `--all` com `--hosts` ou `--tags` na mesma invocação

### Correct Pattern

```bash
ssh-cli --max-concurrency 8 health-check --all --json
ssh-cli health-check --hosts web1,web2 --json
ssh-cli exec --all 'uptime' --json
ssh-cli exec --hosts web1,web2 'uptime' --json
ssh-cli exec --tags prod,web 'uptime' --json
ssh-cli --fail-fast exec --all 'true' --json
ssh-cli scp upload --all ./a.bin /tmp/a.bin --json
ssh-cli scp upload --hosts web1,web2 ./a.bin /tmp/a.bin --json
ssh-cli scp download --all /tmp/a.bin ./a --json
ssh-cli --scp-file-concurrency 4 scp upload prod ./a.bin ./b.bin /tmp/ --json
ssh-cli vps doctor --probe-ssh --json
ssh-cli vps doctor --probe-ssh --hosts web1,web2 --json
```

## Execução Remota e --step
### REQUIRED
- DEVE validar o tamanho do comando contra `max_command_chars` antes de enviar comandos enormes do agente
- DEVE tratar string de comando remoto vazia como falha dura com mensagem técnica exatamente `empty command` (sempre inglês) e exit de processo 64
- DEVE fazer parse de `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr` e `duration_ms` no JSON de sucesso
- DEVE anexar `--description` quando o history shell remoto se beneficiar de comentário de auditoria
- DEVE elevar `max_command_chars` do host via `vps edit` quando o agente precisar de comandos longos
- DEVE honrar default max_command_chars 1000 e max_output_chars 100000 salvo override
- DEVE passar `--timeout <ms>` na família exec quando o deadline padrão do host for curto demais
- DEVE preferir `exec --all` / `--hosts` / `--tags` para frota multi-host
- DEVE usar `--step <CMD>` (repetível) para comandos adicionais na **mesma** sessão SSH após o primário
- DEVE tratar cada `--step` vazio como falha de usage (exit 64)

### FORBIDDEN
- NUNCA DEVE ignorar `truncated_stdout` ou `truncated_stderr` ao resumir saída para o usuário
- NUNCA DEVE fazer retry de exit 64 65 66 77 sem mudar inputs
- NUNCA DEVE enviar string de comando remoto vazia
- NUNCA DEVE abrir N sessões quando `--step` puder encadear na mesma sessão

### Correct Pattern

```bash
ssh-cli exec prod "hostname && uptime" --json --description "inventory"
ssh-cli exec prod "true" --json --timeout 120000
ssh-cli exec prod "uname -a" --step "uptime" --step "df -h" --json
ssh-cli exec --all 'hostname && uptime' --json
ssh-cli exec --tags web 'systemctl is-active nginx' --json
```

## sudo-exec e su-exec
### REQUIRED
- DEVE usar `sudo-exec` para elevação sudo e confiar no packing seguro `sh -c`
- DEVE configurar senha sudo no host ou passar `--sudo-password` ou variante stdin
- DEVE usar `su-exec` somente quando a senha `su` estiver configurada
- DEVE respeitar `--disable-sudo` global e o `disable_sudo` do host
- DEVE tratar elevação como one-shot e NUNCA assumir shell elevado sticky
- DEVE aceitar `--step` e flags de frota em `sudo-exec` e `su-exec` da mesma forma que em `exec`

### FORBIDDEN
- NUNCA DEVE prefixar `sudo` cru em `exec` quando `sudo-exec` existe
- NUNCA DEVE assumir shell elevado persistente entre invocações

### Correct Pattern

```bash
ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --json
printf '%s' "$SUDO_PASSWORD" | ssh-cli sudo-exec prod "systemctl restart nginx" --json --sudo-password-stdin
ssh-cli su-exec prod "whoami" --json
ssh-cli sudo-exec --all "apt-get update" --json
ssh-cli sudo-exec prod "apt-get update" --step "apt-get install -y curl" --json
```

## Transferências SCP
### REQUIRED
- DEVE usar `scp upload` ou `scp download` somente para cópia de arquivo regular
- DEVE passar `--json` em toda transferência parseada por agente
- DEVE fazer parse do sucesso scp somente no stdout com campos `ok`, `event` (`scp-transfer`), `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`
- DEVE tratar o `event` de sucesso scp como a string constante `scp-transfer`
- DEVE tratar `ok` como true e `direction` somente como `upload` ou `download`
- DEVE usar ordem de argumentos `upload <vps> <local> <remote>` e `download <vps> <remote> <local>`
- DEVE passar `--timeout <ms>` no scp quando connect-plus-transfer precisar de deadline maior
- DEVE usar `--password-stdin` e `--key-passphrase-stdin` no scp sempre que os segredos apareceriam no argv
- DEVE usar override `--key` no scp da mesma forma que no exec quando o path de chave salvo for insuficiente
- DEVE esperar upload em stream de chunks 32 KiB sem carregar o arquivo inteiro em RAM
- DEVE esperar download gravando path irmão terminando em `.ssh-cli.partial` e depois rename no lugar
- DEVE esperar preserve de mtime e mode nos dois sentidos sem flag extra do usuário
- DEVE parsear falhas duras de scp no envelope de erro do stderr quando o modo JSON estiver ativo
- DEVE tratar SCP remoto ausente como exit `66` com mensagem `file not found: <path>`
- DEVE preferir frota `scp upload|download --all` / `--hosts` a N spawns
- DEVE usar `--scp-file-concurrency` para multi-arquivo paralelo na mesma sessão quando necessário

### FORBIDDEN
- NUNCA DEVE passar diretórios como paths local ou remote no scp
- NUNCA DEVE inventar flags recursivas como `-r` no scp (use `sftp --recursive` para árvores)
- NUNCA DEVE tratar scp como subsystem SFTP
- NUNCA DEVE usar `--timeout-ms` no scp (essa flag é exclusiva do tunnel)
- NUNCA DEVE parsear sucesso scp como JSON da família exec `stdout`/`stderr`/`exit_code`
- NUNCA DEVE tratar um path `.ssh-cli.partial` residual como artefato final após download completo
- NUNCA DEVE inventar flag de usuário obrigatória para preserve de mtime ou mode
- NUNCA DEVE omitir o campo `event` ao documentar ou parsear JSON de sucesso scp
- NUNCA DEVE tratar SCP remoto ausente como exit `74` quando o exit for `66`

### Correct Pattern

```bash
ssh-cli scp upload prod ./app.tgz /tmp/app.tgz --json
ssh-cli scp download prod /var/log/app.log ./app.log --json
ssh-cli scp upload prod ./a.bin ./b.bin /tmp/ --json
ssh-cli --scp-file-concurrency 4 scp upload prod ./a.bin ./b.bin /tmp/ --json
ssh-cli scp upload --all ./a.bin /tmp/a.bin --json
# sucesso => {"ok":true,"event":"scp-transfer",...}
```

## Subsistema SFTP
### REQUIRED
- DEVE usar `sftp upload|download` para arquivos ou árvores (`--recursive`) e `sftp ls|mkdir|rmdir|rm|stat|rename` para FS remoto
- DEVE passar `--json` em toda transferência ou FS op SFTP parseada por agente
- DEVE parsear sucesso de transfer com `event` igual a `sftp-transfer`; list com `sftp-list`; FS com `sftp-fs-op`; frota com `sftp-batch`
- DEVE tratar walks recursivos como **sem seguir symlink** (fail-closed)
- DEVE esperar stream 32 KiB e download partial + rename atômico
- DEVE passar `--timeout <ms>` quando connect+op precisar de deadline wall-clock maior
- DEVE preferir `sftp upload|download --all` / `--hosts` para frota

### FORBIDDEN
- NUNCA DEVE inventar REPL SFTP interativo
- NUNCA DEVE seguir symlinks em transferência recursiva
- NUNCA DEVE tratar JSON de sucesso SFTP como campos da família exec
- NUNCA DEVE usar scp para árvores quando `sftp --recursive` existe

### Correct Pattern

```bash
ssh-cli sftp upload prod ./app.tgz /tmp/app.tgz --json
ssh-cli sftp upload --recursive prod ./dist /tmp/dist --json
ssh-cli sftp download --recursive prod /tmp/dist ./dist --json
ssh-cli sftp ls prod /var/log --json
ssh-cli sftp mkdir prod /tmp/newdir --json
ssh-cli sftp rmdir prod /tmp/newdir --json
ssh-cli sftp rm prod /tmp/app.tgz --json
ssh-cli sftp stat prod /tmp/app.tgz --json
ssh-cli sftp rename prod /tmp/a /tmp/b --json
ssh-cli sftp upload --all ./app.tgz /tmp/app.tgz --json
```

## Tunnel
### REQUIRED
- DEVE passar `--timeout-ms` em todo comando `tunnel`
- DEVE passar `--bind` de forma consciente quando bind fora de loopback for exigido; o padrão é `127.0.0.1`
- NUNCA DEVE expor `0.0.0.0` sem decisão de segurança explícita
- DEVE tratar porta local `0` como efêmera atribuída pelo SO; após bind, confiar no JSON `local_port` (>=1), nunca conectar na porta 0
- DEVE NUNCA inventar flag `--local-port`; args do tunnel são posicionais `tunnel <vps> <local_port> <remote_host> <remote_port>`
- DEVE passar `--json` quando o agente precisar de sinal estruturado de ready
- DEVE aguardar um objeto no stdout com `event` igual a `tunnel_listening` antes de usar a porta local
- DEVE fazer parse dos campos de ready do tunnel `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`
- DEVE deixar o processo do tunnel vivo até o deadline de `--timeout-ms` ou sinal
- DEVE tratar deadline pós-bind do tunnel como sucesso exit `0` após `tunnel_listening`
- DEVE tratar timeout pré-bind do tunnel como exit `74`
- DEVE parsear falhas duras de tunnel no envelope de erro do stderr quando o modo JSON estiver ativo
- DEVE usar no tunnel overrides de auth `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--use-agent`, `--agent-socket` quando as credenciais salvas do host forem insuficientes
- DEVE preferir `--key-passphrase-stdin` a `--key-passphrase` sempre que stdin estiver disponível

### FORBIDDEN
- NUNCA DEVE abrir túneis sem bound
- NUNCA DEVE deixar processos de tunnel deliberadamente detached para sempre
- NUNCA DEVE usar a porta local antes de `tunnel_listening` quando `--json` estiver ativo
- NUNCA DEVE tratar o start do tunnel como completo só pelo spawn do processo
- NUNCA DEVE usar `--timeout` no lugar de `--timeout-ms` no tunnel
- NUNCA DEVE tratar exit `0` do deadline pós-bind como falha após `tunnel_listening`
- NUNCA DEVE afirmar que o tunnel não tem password-stdin ou overrides de chave
- NUNCA DEVE fazer bind em `0.0.0.0` sem decisão de segurança explícita
- NUNCA DEVE tratar tunnel como multi-host nativo

### Correct Pattern

```bash
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json
# aguardar => {"ok":true,"event":"tunnel_listening","vps":"prod","local_port":18080,"remote_host":"127.0.0.1","remote_port":8080,"timeout_ms":30000}
# deadline pós-bind sai 0; timeout pré-bind sai 74
ssh-cli tunnel prod 0 127.0.0.1 8080 --timeout-ms 30000 --json
# usar local_port do JSON (>=1), NUNCA conectar em 0
```

## Health-check
### REQUIRED
- DEVE usar `health-check` para verificar conectividade após mudanças de host
- DEVE passar `--timeout <ms>` em `health-check` quando um deadline não padrão for necessário
- DEVE usar em `health-check` overrides de auth `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--use-agent`, `--agent-socket` quando as credenciais salvas do host forem insuficientes
- DEVE preferir `--key-passphrase-stdin` a `--key-passphrase` sempre que stdin estiver disponível
- DEVE usar `health-check --replace-host-key` somente após confirmação humana de rotação de host-key
- DEVE parsear falhas duras de health-check no envelope de erro do stderr quando o modo JSON estiver ativo
- NUNCA DEVE usar `--timeout-ms` em health-check
- DEVE preferir `health-check --all --json` ao sondar o inventário inteiro (schema batch `health-check-batch`)

### FORBIDDEN
- NUNCA DEVE afirmar que health-check não tem password-stdin ou overrides de chave
- NUNCA DEVE passar `--replace-host-key` automaticamente sem aprovação humana

### Correct Pattern

```bash
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 5000 --json
ssh-cli --max-concurrency 8 health-check --all --json
ssh-cli health-check --hosts web1,web2 --json
# somente após review humano do mismatch de host-key
ssh-cli health-check prod --json --replace-host-key
```

## Locale
### REQUIRED
- DEVE usar `locale show` para diagnosticar idioma resolvido, camada vencedora e locales disponíveis
- DEVE usar `locale set <LOCALE>` para persistir preferência no arquivo `lang` sob o config-dir (mode 0o600)
- DEVE usar `locale clear` para remover a preferência persistida
- DEVE usar `--lang <LOCALE>` para forçar idioma somente nesta invocação (vence o arquivo `lang`)
- DEVE negociar tags BCP47 para `en` ou `pt-BR` (ex. `en`, `en-US`, `pt`, `pt-BR`)
- DEVE preferir `--lang` / `locale set` a qualquer env de idioma

### FORBIDDEN
- NUNCA DEVE tratar `SSH_CLI_LANG` ou `LANG` bruto portátil como store de produto
- NUNCA DEVE inventar locales fora da negociação suportada

### Correct Pattern

```bash
ssh-cli locale show --json
ssh-cli locale set pt-BR
ssh-cli locale clear
ssh-cli --lang pt-BR exec prod "true" --json
```

## TLS mTLS e ACME
### REQUIRED
- DEVE usar `tls provider` para confirmar stack rustls/`aws_lc_rs`
- DEVE usar `tls paths` para imprimir layout XDG (`tls/`, `tls/mtls/`, `tls/acme/`)
- DEVE usar `tls mtls import --name <NAME> --cert <PEM> --key <PEM>` para importar identidade cliente
- DEVE usar `tls mtls list|show|remove` para gerenciar identidades
- DEVE usar `tls acme account create --contact mailto:ops@example.com` (contact obrigatório, repetível; `--staging` / `--force` quando aplicável)
- DEVE usar `tls acme account show` para verificar existência e path
- DEVE usar `tls acme issue --domain <DOM> --print-challenge` para iniciar DNS-01 e imprimir TXT (agente, sem wait interativo)
- DEVE usar `tls acme complete --domain <DOM>` após publicar o TXT
- DEVE usar `tls acme status [--domain <DOM>]` e `tls acme list` para status
- DEVE habilitar SSH-over-TLS no host com `vps add|edit --tls` e opcionalmente `--tls-sni` / `--tls-client-cert` / `--tls-client-key`
- DEVE tratar falhas de validação ACME (`invalidContact`, outros problem types 4xx permanentes) como exit **64** permanente com `retryable` false
- NUNCA DEVE fazer retry de validação ACME permanente como se fosse exit 74

### FORBIDDEN
- NUNCA DEVE armazenar certs TLS fora do layout XDG do produto
- NUNCA DEVE inventar stores env para material TLS
- NUNCA DEVE tratar erro ACME permanente (exit 64) como retryable/IO 74
- NUNCA DEVE omitir `--print-challenge` em fluxos de agente no `issue`

### Correct Pattern

```bash
ssh-cli tls provider --json
ssh-cli tls paths --json
ssh-cli tls mtls import --name client1 --cert ./client.pem --key ./client-key.pem --json
ssh-cli tls mtls list --json
ssh-cli tls mtls show client1 --json
ssh-cli tls mtls remove client1
ssh-cli tls acme account create --contact mailto:ops@example.com --staging --json
ssh-cli tls acme account show --json
ssh-cli tls acme issue --domain app.example.com --print-challenge --staging --json
ssh-cli tls acme complete --domain app.example.com --json
ssh-cli tls acme status --domain app.example.com --json
ssh-cli tls acme list --json
ssh-cli vps add --name tlsbox --host tls.example.com --user deploy --key ~/.ssh/id_ed25519 --tls --tls-sni tls.example.com
```

## Discovery (commands / schema)
### REQUIRED
- DEVE rodar `ssh-cli commands` para emitir a árvore completa de subcomandos em JSON
- DEVE rodar `ssh-cli schema` sem nome para listar o catálogo embutido
- DEVE rodar `ssh-cli schema <NAME>` para emitir o body de um schema (ex. `vps-list`, `exec-batch`)
- DEVE preferir discovery via `commands`/`schema` a inventar contratos

### FORBIDDEN
- NUNCA DEVE inventar nomes de schema ou campos não emitidos pelo produto

### Correct Pattern

```bash
ssh-cli commands
ssh-cli schema
ssh-cli schema vps-list
ssh-cli schema exec-batch
```

## Timeouts Host-keys e Segurança de Storage
### REQUIRED
- DEVE passar `--timeout-ms` somente em `tunnel` e SEMPRE como obrigatório
- DEVE passar `--timeout` em `scp`, família exec e `health-check` ao sobrescrever deadlines
- DEVE tratar todos os valores de timeout de host e VPS como milissegundos, não segundos
- DEVE esperar warning no stderr quando um valor de timeout de host/vps for menor que 1000 ms
- NUNCA DEVE intercambiar `--timeout` e `--timeout-ms` entre subcomandos
- NUNCA DEVE definir timeout de host abaixo de 1000 salvo deadline sub-segundo intencional
- DEVE tratar mismatch de host key como hard stop até confirmação humana de rotação
- DEVE usar `--replace-host-key` somente após confirmação
- DEVE esperar writes atômicos de `config.toml` e `secrets.key` e mode 0600 em Unix
- DEVE usar `--config-dir` para sandboxes isolados de agente (o produto não lê `SSH_CLI_HOME`)

### FORBIDDEN
- NUNCA DEVE auto-substituir host keys sem aprovação do usuário
- NUNCA DEVE desabilitar TOFU por conveniência em fluxos de produção

## Códigos de Saída e Retry
### REQUIRED
- DEVE mapear exits como 0 sucesso, 1 geral, 64 usage, 65 data, 66 not found, 73 cant create, 74 IO ou SSH, 77 auth, 130 SIGINT, 143 SIGTERM
- DEVE tratar comando remoto vazio como exit `64` com mensagem `empty command`
- DEVE tratar TOML de import inválido como exit `65`
- DEVE tratar SCP remoto ausente como exit `66` com mensagem `file not found: <path>`
- DEVE tratar falha de autenticação como exit `77`
- DEVE tratar falha de validação ACME permanente como exit `64` (nunca retry como 74)
- DEVE tratar deadline pós-bind do tunnel como exit 0 após `tunnel_listening`
- DEVE tratar timeout pré-bind do tunnel como exit 74
- DEVE fazer no máximo dois retries em 74 com backoff **somente** quando `retryable` for true
- DEVE falhar rápido em 64 65 66 77 sem retry cego
- DEVE ler `retryable`, `error_class` e `suggestion` do envelope antes de re-invocar
- DEVE expor o `exit_code` remoto do JSON de sucesso separadamente do exit do processo CLI
- DEVE expor `remote_exit_code` do envelope de erro em stderr quando presente

### FORBIDDEN
- NUNCA DEVE engolir exits não zero
- NUNCA DEVE confundir falha do comando remoto com falha de usage local da CLI
- NUNCA DEVE fazer retry do exit 0 pós-bind do tunnel como se fosse falha
- NUNCA DEVE fazer retry de ACME validation permanente (exit 64) como 74

## Contrato de Parse JSON
### REQUIRED
- DEVE parsear somente stdout como JSON de sucesso quando o modo JSON estiver ativo e o exit for caminho de sucesso
- DEVE ler campos da família exec `stdout`, `stderr`, `exit_code`, `truncated_stdout`, `truncated_stderr`, `duration_ms`
- DEVE ler campos de sucesso scp `ok`, `event` (`scp-transfer`), `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`
- DEVE ler campos de ready do tunnel `ok`, `event`, `vps`, `local_port`, `remote_host`, `remote_port`, `timeout_ms`
- DEVE tratar o `event` do tunnel como a string constante `tunnel_listening`
- DEVE tratar o `event` de sucesso scp como a string constante `scp-transfer`
- DEVE parsear campos do envelope de erro em stderr `exit_code`, `message`, `remote_exit_code`, `retryable`, `error_class`, `suggestion` em falhas duras no modo JSON incluindo scp, tunnel e health-check
- DEVE tratar payloads de list show doctor secrets status como objetos tipados e usar só campos documentados
- DEVE tratar `password` em list/show como JSON `null` quando vazio ou ausente e como `***` quando armazenado
- DEVE tratar `sudo_password`, `su_password` e `key_passphrase` em list/show como `null` ou `***` da mesma forma
- DEVE tratar `added_at` em list/show/export como presente; import DEVE aceitar omissão de `added_at` e serde preenche o default
- DEVE parsear doctor JSON como `event` igual a `vps-doctor` com `local.secrets_plaintext_opt_out` boolean e `ssh_probe` null ou batch
- DEVE parsear `secrets_key_auto_created` no mesmo JSON `vps-added` (nunca segundo evento)
- DEVE reportar truncagem ao usuário quando `truncated_stdout` ou `truncated_stderr` for true

### FORBIDDEN
- NUNCA DEVE inventar chaves JSON ausentes
- NUNCA DEVE inventar senhas falsas quando `password` for `null`
- NUNCA DEVE pretty-print de segredos encontrados em campos inesperados
- NUNCA DEVE parsear stderr como JSON de sucesso
- NUNCA DEVE parsear sucesso scp como campos da família exec
- NUNCA DEVE parsear ready do tunnel como campos da família exec
- NUNCA DEVE parsear sucesso scp sem exigir `event` igual a `scp-transfer`
- NUNCA DEVE tratar `secrets_plaintext_opt_out` do doctor como string

## Folha de Fórmulas Prontas
### REQUIRED
- DEVE copiar estas fórmulas exatamente e somente substituir placeholders

```bash
# inventário
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --port <PORT> --check
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --tag <TAG> --tag <TAG2>
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --use-agent --agent-socket <SOCKET>
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --tls --tls-sni <SNI>
ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --key <KEY_PATH> --tls --tls-client-cert <CERT> --tls-client-key <KEY>
printf '%s' "$PASS" | ssh-cli vps add --name <NAME> --host <HOST> --user <USER> --password-stdin
printf '%s' "$SUDO" | ssh-cli vps edit <NAME> --sudo-password-stdin
ssh-cli vps list --json
ssh-cli vps list --tag <TAG> --json
ssh-cli vps show <NAME> --json
ssh-cli vps edit <NAME> --timeout <MS> --max-command-chars <N> --max-output-chars <N>
ssh-cli vps edit <NAME> --tls --tls-sni <SNI>
ssh-cli vps edit <NAME> --no-tls
ssh-cli vps doctor --json
ssh-cli doctor --json
ssh-cli vps doctor --probe-ssh --json
ssh-cli vps doctor --probe-ssh --hosts <A>,<B> --json
ssh-cli vps path
ssh-cli vps export -o /tmp/hosts.toml
ssh-cli vps export --json
ssh-cli vps export --include-secrets -o /tmp/hosts-secrets.toml
# secrets vazios no export redacted permanecem strings vazias; NUNCA espere sshcli-enc para valores vazios
# NUNCA envie --include-secrets em pipe sem --output/-o ou --i-understand-secrets-on-stdout
# host --timeout é milissegundos; valores menores que 1000 emitem warning no stderr
ssh-cli vps import --file /tmp/hosts.toml
ssh-cli vps import --file /tmp/hosts.json
ssh-cli vps import --file /tmp/hosts-redacted.toml --allow-incomplete
# import DEVE aceitar omissão de added_at; list/show/export apresentam added_at
ssh-cli connect <NAME>
ssh-cli vps remove <NAME>

# ops remotas
ssh-cli exec <NAME> "<CMD>" --json
ssh-cli exec <NAME> "<CMD>" --json --timeout <MS> --description "<AUDIT>"
ssh-cli exec <NAME> "<CMD>" --step "<CMD2>" --step "<CMD3>" --json
ssh-cli -q exec <NAME> "<CMD>" --json
ssh-cli sudo-exec <NAME> "<CMD>" --json
printf '%s' "$SUDO" | ssh-cli sudo-exec <NAME> "<CMD>" --json --sudo-password-stdin
ssh-cli su-exec <NAME> "<CMD>" --json
# comando remoto vazio => mensagem "empty command" e exit 64 (sempre inglês)
# frota multi-host (concorrente com bound; preferir a N spawns single-host)
ssh-cli --max-concurrency <N> exec --all "<CMD>" --json
ssh-cli exec --hosts <A>,<B> "<CMD>" --json
ssh-cli exec --tags <TAG1>,<TAG2> "<CMD>" --json
ssh-cli --fail-fast exec --all "<CMD>" --json
ssh-cli sudo-exec --all "<CMD>" --json
ssh-cli su-exec --all "<CMD>" --json
ssh-cli sudo-exec <NAME> "<CMD>" --step "<CMD2>" --json

# transferências scp (somente arquivos regulares; agente DEVE usar --json; event DEVE ser scp-transfer)
ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json
ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json
ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --timeout <MS>
# multi-file single-host (uma sessão)
ssh-cli scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json
ssh-cli --scp-file-concurrency <N> scp upload <NAME> <F1> <F2> <REMOTE_DIR> --json
# multi-host × multi-file
ssh-cli scp upload --all <F1> <F2> <REMOTE_DIR> --json
ssh-cli scp download <NAME> <R1> <R2> <LOCAL_DIR> --json
printf '%s' "$PASS" | ssh-cli scp download <NAME> <REMOTE_FILE> <LOCAL_FILE> --json --password-stdin
printf '%s' "$KEY_PASS" | ssh-cli scp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json --key <KEY_PATH> --key-passphrase-stdin
# frota scp (download: path local é prefixo → <local>.<vps>)
ssh-cli scp upload --all <LOCAL_FILE> <REMOTE_FILE> --json
ssh-cli scp download --all <REMOTE_FILE> <LOCAL_PREFIX> --json
ssh-cli scp upload --hosts <A>,<B> <LOCAL_FILE> <REMOTE_FILE> --json

# sftp (árvores + FS; event sftp-transfer / sftp-list / sftp-fs-op / sftp-batch)
ssh-cli sftp upload <NAME> <LOCAL_FILE> <REMOTE_FILE> --json
ssh-cli sftp upload --recursive <NAME> <LOCAL_DIR> <REMOTE_DIR> --json
ssh-cli sftp download --recursive <NAME> <REMOTE_DIR> <LOCAL_DIR> --json
ssh-cli sftp ls <NAME> <REMOTE_DIR> --json
ssh-cli sftp mkdir <NAME> <REMOTE_DIR> --json
ssh-cli sftp rmdir <NAME> <REMOTE_DIR> --json
ssh-cli sftp rm <NAME> <REMOTE_FILE> --json
ssh-cli sftp stat <NAME> <REMOTE_PATH> --json
ssh-cli sftp rename <NAME> <FROM> <TO> --json
ssh-cli sftp upload --all <LOCAL_FILE> <REMOTE_FILE> --json

# tunnel (--timeout-ms obrigatório; --bind padrão 127.0.0.1; aguardar tunnel_listening; deadline pós-bind exit 0)
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --bind 127.0.0.1
ssh-cli tunnel <NAME> 0 <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json
printf '%s' "$PASS" | ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --password-stdin
ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --key <KEY_PATH>
printf '%s' "$KEY_PASS" | ssh-cli tunnel <NAME> <LOCAL_PORT> <REMOTE_HOST> <REMOTE_PORT> --timeout-ms <MS> --json --key <KEY_PATH> --key-passphrase-stdin

# health
ssh-cli health-check <NAME> --json
ssh-cli health-check <NAME> --timeout <MS> --json
ssh-cli health-check --json
ssh-cli --max-concurrency <N> health-check --all --json
ssh-cli health-check --hosts <A>,<B> --json
printf '%s' "$PASS" | ssh-cli health-check <NAME> --json --password-stdin
ssh-cli health-check <NAME> --json --key <KEY_PATH>
printf '%s' "$KEY_PASS" | ssh-cli health-check <NAME> --json --key <KEY_PATH> --key-passphrase-stdin
ssh-cli health-check <NAME> --json --replace-host-key

# secrets e segurança (preferir flags CLI a env; termo de produto primary-key)
ssh-cli secrets status --json
ssh-cli secrets init
ssh-cli secrets init --json
ssh-cli secrets init --force --json
ssh-cli secrets init --keyring --json
ssh-cli secrets reencrypt
ssh-cli secrets reencrypt --json
ssh-cli --allow-plaintext-secrets --config-dir <DIR> secrets status --json
ssh-cli --secrets-key-file <KEY_FILE> secrets status --json
ssh-cli --use-keyring secrets status --json
ssh-cli --replace-host-key exec <NAME> "true"
ssh-cli --config-dir <DIR> vps list --json
printf '%s' "$KEY_PASS" | ssh-cli exec <NAME> "id" --json --key <KEY_PATH> --key-passphrase-stdin
# preferir secrets via stdin; password-like em argv emite warning no stderr

# locale
ssh-cli locale show --json
ssh-cli locale set pt-BR
ssh-cli locale set en
ssh-cli locale clear
ssh-cli --lang pt-BR vps list --json

# TLS / mTLS / ACME
ssh-cli tls provider --json
ssh-cli tls paths --json
ssh-cli tls mtls import --name <NAME> --cert <CERT_PEM> --key <KEY_PEM> --json
ssh-cli tls mtls list --json
ssh-cli tls mtls show <NAME> --json
ssh-cli tls mtls remove <NAME>
ssh-cli tls acme account create --contact mailto:<EMAIL> --json
ssh-cli tls acme account create --contact mailto:<EMAIL> --staging --force --json
ssh-cli tls acme account show --json
ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --json
ssh-cli tls acme issue --domain <DOMAIN> --print-challenge --staging --json
ssh-cli tls acme complete --domain <DOMAIN> --json
ssh-cli tls acme status --domain <DOMAIN> --json
ssh-cli tls acme status --json
ssh-cli tls acme list --json

# discovery
ssh-cli commands
ssh-cli schema
ssh-cli schema <NAME>
ssh-cli doctor --json

# debug somente ao diagnosticar; nível de log padrão é error
ssh-cli -v exec <NAME> "true" --json

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

## Proibições Absolutas
### FORBIDDEN
- NUNCA DEVE manter sessões SSH abertas entre turnos do agente exceto tunnel bound ativo até o deadline
- NUNCA DEVE reintroduzir daemons Node ou de protocolo de longa duração para esta superfície de produto
- NUNCA DEVE vazar segredos em argv quando variantes stdin existirem
- NUNCA DEVE preferir `--key-passphrase` em argv quando `--key-passphrase-stdin` estiver disponível
- NUNCA DEVE ignorar mismatch de host-key
- NUNCA DEVE abrir tunnels sem `--timeout-ms`
- NUNCA DEVE usar a porta local do tunnel antes de `tunnel_listening` quando o modo JSON estiver ativo
- NUNCA DEVE fazer scp de diretórios; para árvores DEVE usar `sftp --recursive` (sem seguir symlink)
- NUNCA DEVE tratar JSON de sucesso scp como campos da família exec
- NUNCA DEVE deixar paths `.ssh-cli.partial` de download como entregável final após sucesso
- NUNCA DEVE inventar senhas falsas para hosts só-chave quando o JSON mostrar `null`
- NUNCA DEVE documentar changelogs históricos de versão dentro desta skill
- NUNCA DEVE colar credenciais vivas em exemplos ou logs
- NUNCA DEVE esperar `sshcli-enc` para secrets vazios no export redacted
- NUNCA DEVE tratar exit 0 pós-bind do tunnel como falha após `tunnel_listening`
- NUNCA DEVE enviar `--include-secrets` em pipe sem `--output`/`-o` ou `--i-understand-secrets-on-stdout`
- NUNCA DEVE fazer bind do tunnel em `0.0.0.0` sem decisão de segurança explícita
- NUNCA DEVE imprimir material de primary-key
- NUNCA DEVE enviar strings de comando remoto vazias
- NUNCA DEVE tratar valores de timeout de host como segundos
- NUNCA DEVE tratar env `SSH_CLI_*` / `RUST_LOG` como stores de produto
- NUNCA DEVE fazer retry de ACME validation permanente (exit 64) como 74
- NUNCA DEVE spawn N processos single-host quando `--all` / `--hosts` / `--tags` cobrir a frota
- NUNCA DEVE esperar segundo evento NDJSON para `secrets_key_auto_created`

### REQUIRED
- DEVE reler esta skill antes de todo workflow não trivial de ssh-cli
- DEVE usar hosts salvos, segredos via stdin, saída JSON e execução one-shot
- DEVE parsear somente stdout como JSON de sucesso e envelopes de erro no stderr em falhas duras
- DEVE aguardar `tunnel_listening` antes de usar a porta local do tunnel
- DEVE tratar deadline pós-bind do tunnel como exit 0 e timeout pré-bind como exit 74
- DEVE tratar empty command como exit 64, SCP remoto ausente como exit 66, TOML de import inválido como exit 65, auth como exit 77, ACME validation permanente como exit 64
- DEVE tratar o corpo de `vps export` como TOML salvo `vps export --json`
- DEVE parsear doctor com `event` igual a `vps-doctor`, `secrets_plaintext_opt_out` como boolean e `added_at` como opcional somente no import
- DEVE tratar timeouts como milissegundos e esperar warning se menor que 1000 em timeout de host/vps
- DEVE preferir flags de frota e `--step` na mesma sessão
- DEVE falhar fechado em erros de auth, host-key, usage e secrets env rejeitadas
