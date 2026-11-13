# Integrações


> Conecte 9 agentes de IA e qualquer shell a servidores com 48 comandos one-shot

- Leia este documento em [Inglês (en)](INTEGRATIONS.md)
- Combine este catálogo com [docs/AGENTS.pt-BR.md](docs/AGENTS.pt-BR.md) e [skills/ssh-cli-pt/SKILL.md](skills/ssh-cli-pt/SKILL.md)
- Contrato de integração: a ssh-cli roda como subprocesso, então o contrato inteiro é argv, stdin, stdout, stderr e o exit code
- Todo comando ssh-cli dos exemplos abaixo foi rodado contra a build da 0.5.6 que já traz o backend de keyring, enquanto a configuração do lado do agente não é testada por este projeto e segue a documentação oficial de cada um
- Versão atual: 0.5.6, com as mudanças mais novas no topo da seção de novas flags


## Aliases de flags
### Aliases camelCase implementados no clap (não invente outros)
- Use `--sudoPassword` como alias de `--sudo-password`
- Use `--suPassword` como alias de `--su-password`
- Use `--maxChars` ou `--max-chars` como aliases legados que mapeiam para `max_command_chars`, e prefira `--max-command-chars`
- Use `--disableSudo` como alias do global `--disable-sudo`, aceito antes ou depois do subcomando onde a flag global é aceita, enquanto `vps add` e `vps edit` o rejeitam e usam `--disable-sudo`, que persiste o bloqueio no registro
- Use `--fields` como alias de `--select`
- Use `--json` como alias de `--output-format json`
- Não há aliases camelCase para `--config-dir`, `--output-format` ou `--no-color`: use exatamente o kebab-case


## Novas flags por versão
### Acompanhe o crescimento da superfície sem ler o código
- `0.5.6` rename e edição completa (GAP-RENAME-001): `vps rename <NAME> <NEW_NAME>` e `vps edit <NAME> --name <NEW>` movem a chave do registro e mantêm toda credencial, o registro sela sem dado associado, então o blob não fica preso ao nome do host e a credencial continua legível sob o nome novo
- Oito flags de limpeza removem o campo em vez de sobrescrevê-lo: `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key`, `--no-key` (caminho da chave mais passphrase), `--no-agent-socket`
- Cada uma conflita com a flag de valor do próprio campo, e limpar o último método de auth sai 64
- `0.5.6` troca de autenticação: `vps edit --key`, `--password`, `--password-stdin` e `--use-agent` trocam o método guardado e limpam os campos opostos em uma chamada
- `updated` lista o método novo e só os campos limpos que tinham valor
- Duas flags de autenticação primária na mesma chamada saem 2 no parse, e passphrase sem nenhuma chave sai 64 com o registro intacto
- Um `--key` diferente descarta a passphrase antiga, salvo se vier uma nova, e `--agent-socket` sozinho implica `--use-agent`
- `0.5.6` nomes de host e segurança do terminal: nomes novos são letras ASCII, dígitos, ponto, hífen e sublinhado, de 1 a 255 bytes, começando por letra ou dígito
- `vps import` lista todos os nomes inválidos de uma vez com exit 64 e não grava nada, também sob `--dry-run`
- Nomes guardados antes da 0.5.6 continuam carregando, o `doctor` os lista em `local.invalid_host_names`, e `vps rename` os repara
- O modo texto escreve caracteres de controle e bidi como escapes tipo `\u{1b}`, e `sftp ls` e `sftp stat` também escapam `\n` e `\t` em nomes remotos
- `tls mtls import --name` recusa caracteres de controle, formatação e bidi com exit 64 e não cria diretório
- Duas tabelas `[hosts]` que normalizam para o mesmo nome NFC saem 65 em vez de perder um host em silêncio
- `0.5.6` diretório de configuração: um diretório criado pela interface de linha de comando (CLI) nasce 0700, um diretório existente fora do padrão nunca recebe chmod, e o `doctor` reporta `local.config_dir_permissions` e `local.config_dir_exposed`
- `--config-dir <path>.toml` usa o pai como diretório de todos os arquivos, `vps export -o` cria diretórios que faltam com 0700, e `known_hosts.lock` e `lang` nascem 0600
- `0.5.6` eco do dry-run: os planos de `vps edit`, `vps rename` e `vps remove` trazem `target_resolved`, `target_source`, `host_resolved` e `host_source`
- `vps edit`, `vps rename` e `vps remove` num registro ausente saem 66 e não criam diretório nem lock
- `0.5.6` eventos e prévias: `vps edit <NAME> --name <NEW>` e `vps rename <NAME> <NEW_NAME>` emitem `vps-renamed` com `name`, `previous_name`, `active_rewritten`, `updated`, `target_resolved` e `target_source`
- `vps-edited` é emitido só quando nada foi renomeado e não traz `previous_name`
- `updated` está sempre presente: os campos cujo valor guardado mudou, na ordem de aplicação, cada um no máximo uma vez
- Vazio quando todo valor pedido já coincidia ou quando a invocação só renomeou o host
- Uma edição que não muda nada é no-op idempotente (exit 0, `vps-edited`, `updated: []`, `config.toml` não regravado), e uma invocação sem flag de campo, rename ou limpeza sai 64
- `--dry-run` passa a cobrir oito operações
- O lock do config honra o global `--lock-wait-ms` (padrão 5000, máximo 300000) e sai 69 `unavailable` em vez de bloquear para sempre, e `commands --json` publica os `args` de cada comando com `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` e `hidden`
- `0.5.6` semântica do rename e comportamento do lock: `vps rename ghost ghost` sai 66 (a pré-condição de host existente vence)
- Nome idêntico de host existente sai 64 e o erro carrega `target_resolved`
- Um destino que já existe sai 64 com `vps_duplicate`
- O rename grava o marcador `active` primeiro e o `config.toml` depois, restaura o marcador antigo se o save falhar e não renomeia nada se a gravação do marcador falhar
- Um crash entre os dois passos deixa marcador órfão, que é fail-closed (`--use-active` sai 66)
- `vps rename` não é idempotente: repetir depois de sucesso sai 66, e `vps show <NEW>` confirma o resultado
- A espera do lock roda em fatias de 100 ms que checam o sinal, então SIGINT/SIGTERM saem 130/143 em até uma fatia
- `config.toml.lock` nasce com modo 0600
- A gravação do marcador faz fsync do diretório
- A dica do exit 69 agora manda repetir o argv ou subir `--lock-wait-ms`
- `0.5.6` modo texto, import e versão: erro de escrita em stdout agora propaga em todo verbo no modo texto (pipe quebrado sai 141, disco cheio vira erro de I/O, antes o exit era 0), e em `exec` com stdout fechado o erro de escrita vence a falha do comando remoto
- `vps import` valida Transport Layer Security (TLS) como o `vps edit` (certificado sem chave sai 64), e o evento de sucesso `vps-import` ganha `replaced` (nomes sobrescritos, ordenado, sempre presente, o dry-run mantém `replaces_existing`)
- `--version` lê `.commit_hash` como `<short hash> <version>` e resolve env, depois git, depois o arquivo, depois `unknown`
- Sem env e sem git o arquivo só vale se a versão for a do pacote
- `0.5.6` BREAKING (biblioteca) e scripts: `SshCliError::Unavailable { service: UnavailableService }` e `SshCliError::unavailable(UnavailableService)` substituem o serviço em string (`UnavailableService` é `#[non_exhaustive]` com `Keyring` e `ConfigLock`)
- O texto da wire não muda (`service unavailable: config_lock`)
- Os nove scripts de release viraram `scripts/*.py` (Python 3.9+, só stdlib, no Windows `py -3 scripts/<name>.py`) e suas variáveis de ambiente viraram flags
- Ver [docs/MIGRATION.pt-BR.md](docs/MIGRATION.pt-BR.md)
- `0.5.5` BREAKING + segurança: designação explícita de alvo (GAP-SSH-EXEC-ARGC-001, CWE-441): a família exec inferia o host pela contagem de posicionais, então `exec <HOST> --step <CMD>` rodava o nome do host como binário remoto no host que o `connect` marcou por último
- Um posicional solitário agora é erro de uso e `health-check` sem nome sai 64
- Alcance o marcador com `--use-active`, que sai 66 quando não há marcador
- `0.5.5` proveniência de alvo em todo envelope (GAP-SSH-EXEC-ENVELOPE-002): `target_resolved`, `target_source` (`argv` | `active_marker` | `selector`) e `active_fallback`, sempre no sucesso, e no envelope de erro assim que o alvo é resolvido, onde ficam ausentes em vez de vazios
- Vincule consumidores novos a esses nomes canônicos
- `host_resolved` e `host_source` acompanham como aliases de leitura da 0.5.5
- `doctor` reporta `local.active_vps`
- `0.5.4` modos de tunnel (G-TUN-R01/R02/R03): `tunnel --reverse` (servidor escuta e entrega de volta, `REMOTE_PORT 0` = servidor aloca), `tunnel --socks5` (proxy SOCKS5 local, RFC 1928 no-auth + CONNECT, destino por conexão, `REMOTE_HOST`/`REMOTE_PORT` omitidos), `tunnel --remote-socket <PATH>` (socket Unix remoto via `direct-streamlocal@openssh.com`, caminho absoluto ou exit 64)
- O `mode` no JSON é `local` | `reverse` | `socks5` | `streamlocal`
- `0.5.4` redução de payload agent-native: globais `--select` (apelido `--fields`), `--filter` (`key=value` | `key!=value` | `key~substring`, repetível com AND), `--limit`, `--sort`, `--dedupe-by`, `--count-only` (→ `{"count": N}`), `--truncate-content` (caracteres, nunca bytes), `--max-output-bytes` (descarta registros do fim, nunca fatia o JSON), todas aplicadas antes da serialização
- Mais as globais `--no-input` e `--dry-run` (honrada pelas oito operações listadas na 0.5.6, e nas demais sai 64)
- `0.5.4` BREAKING + segurança: falha parcial multi-host sai com exit 1 (era 65, G-ERR-R02)
- `tunnel --bind` fora do loopback exige `--i-accept-network-exposure` (G-TUN-R13)
- Novo evento `tunnel_closed` (`reason`, `forwards_served`, `capacity_waits`)
- `schema dry-run` / `schema tunnel-closed` agora saem com exit 0
- A1 aplica o teto de log `AUTH_BANNER_MAX_CHARS` (512), que já existia, em fronteira de caractere e não por índice de byte, removendo um abort disparável remotamente, A2 cria chaves Automatic Certificate Management Environment (ACME) e TLS mútuo (mTLS) em `0600` em vez de restringir depois de criar, A3 mascara os modos de entrada com `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) para que setuid/setgid/sticky não peguem carona num download
- `0.5.3` integridade do SSH File Transfer Protocol (SFTP) + segurança para agentes (G1–G19): upload SFTP não trunca mais destinos a 0 bytes (`FileAttributes::empty`)
- SETSTAT envia atime+mtime juntos
- `set_metadata` fail-closed
- Bits de permissão via `SFTP_PERM_MASK` (`0o7777`)
- Download sinaliza falhas de `set_permissions` local
- Download SCP faz `sync_data` antes do rename
- Cancel de batch preenche resto cancelled (`results.len() == input.len()`)
- `exec --json` um único objeto NDJSON
- E2E SFTP com checksums E17/E18
- Prefira 0.5.3+ para qualquer transferência SFTP
- `0.5.3` verbosidade graduada (G2/G14): `-v` info / `-vv` debug / `-vvv` trace via `ArgAction::Count`, sempre com allowlist crate-scoped `warn,ssh_cli=*`, nunca debug global bare (sem dump de senha do russh)
- `RUST_LOG` ambiente continua ignorado
- Mesmo `-vvv` não habilita dumps de canal cifrado do russh
- `0.5.2+` SFTP (G-SFTP): `ssh-cli sftp upload|download|ls|mkdir|rmdir|rm|stat|rename` via `russh-sftp` 2.3
- Upload/download com `--recursive` (sem seguir symlink), multi-host `--all`/`--hosts`, eventos JSON `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch`
- Somente I/O em stream (sem heap do arquivo inteiro)
- SCP continua só arquivo regular
- Use 0.5.3+ para as correções de integridade SFTP acima
- `0.5.2+` fan-out multi-host (concorrência limitada): `exec|sudo-exec|su-exec|scp|sftp|health-check --all` roda sessões SSH concorrentes limitadas por `--max-concurrency N` (1..=64, auto CPUs×4 vs RAM livre/2 / 16 MiB)
- Envelopes batch: `health-check-batch` / `exec-batch` / `scp-batch` / `sftp-batch` (`docs/schemas/*-batch.schema.json`, campo `max_concurrency`)
- Prefira um processo com `--all` a N spawns single-host para frota
- Accepts de tunnel compartilham o mesmo gate
- `0.5.2` E2E residual + export/import agent-first + wire v3: root `schema`/`doctor`
- Um único `vps-added` + `secrets_key_auto_created`
- `RUST_LOG` ambiente ignorado
- ACME permanente 64
- `vps add --use-agent`
- O export redigido grava `password` como string vazia e os demais segredos como `null`, enquanto `FIXED_MASK` mascara só `vps list` e `vps show`
- Mais o corpo de `vps export` segue o formato de saída resolvido (JSON em qualquer stdout non-TTY, TOML somente com `--output-format text`)
- `vps import` aceita TOML (chaves EN + aliases PT) ou envelopes JSON `vps-export`
- Dual-read serialize EN + aliases PT
- Host schema v3
- Flags CLI `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (nunca variáveis de ambiente)
- Eventos `secrets init|reencrypt --json` `secrets-init` / `secrets-reencrypt`
- A 1ª gravação define `secrets_key_auto_created` no mesmo documento `vps-added`
- Caminhos de sucesso CRUD usam eventos JSON `emit_success`
- `--include-secrets` em pipe/non-TTY exige `-o`/`--output` ou `--i-understand-secrets-on-stdout`
- Tunnel `--bind` (default `127.0.0.1`)
- Import `TomlDe` → exit 65
- `SshAuthentication` → 77
- SCP missing `file not found: <path>` (exit 66)
- Warn de timeout em `vps add` se `<1000` ms
- Warn stderr de password em argv
- Doctor `secrets_plaintext_opt_out` é bool
- `0.4.2` tunnel porta efêmera `local_port=0` reporta porta atribuída pelo SO após bind (TUN-003)
- SCP remoto ausente → exit 66 (IO-010)
- Envelope `vps export --json` com `event: "vps-export"`
- E2e E15/E16
- Suíte `gaps_v042`
- `0.4.1` AUD-POST + SCP somente arquivos regulares (herda wire 0.4.0, sem `-r` / sem SFTP)
- Wire SCP sólido (evite crates.io 0.3.9 SCP quebrado)
- Flags scp `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` → `docs/schemas/scp-transfer.schema.json` com `event: "scp-transfer"` obrigatório (IO-009)
- Download grava `{path}.ssh-cli.partial` e faz rename
- Preserve mtime/mode bi-dir
- Upload em stream 32 KiB
- `tunnel --json` emite `tunnel_listening` após bind
- CLI-005 paridade de auth stdin/passphrase do tunnel
- CLI-006 paridade de auth do health-check
- Envelope JSON de erro scp em stderr com `--json`
- EXP-001 export redigido: segredos vazios ficam `""` (nunca `sshcli-enc:` de vazio)
- TUN-002 deadline pós-bind do tunnel sai 0 após `tunnel_listening` (timeout pré-bind continua 74)
- `0.4.0` wire SCP sólido (corrige crates.io 0.3.9)
- Transferências só de arquivo
- `tunnel --json` / `tunnel_listening`
- `0.3.9` filtro de tracing default `error` (agent-first)
- Senha vazia serializa como JSON `null` em hosts só-chave
- `health-check --timeout <ms>`
- Auditoria de docs de product line
- `0.3.8` russh 0.62.2
- Stdout de tunnel limpo para agentes
- Sem VPS ativa sai com `66` (`EX_NOINPUT`)
- `cargo deny` com `yanked=deny`
- `0.3.7` `--output-format` no CRUD VPS
- `health-check --json`
- `--quiet`
- Envelope JSON de erro
- Timeout do tunnel cobre connect
- `0.3.6` adiciona cifragem at-rest default, `secrets status|init|reencrypt`, campos doctor `secrets_key_file` / `secrets_plaintext_opt_out`, `scripts/e2e_real_ssh.py`
- `0.3.5` adiciona caminhos `--key-passphrase-stdin`, JSON auto em non-TTY, doctor `secrets_at_rest`, export atômico residual
- `0.3.4` adiciona `--key`, `--key-passphrase`, `--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--timeout-ms` (tunnel), `--disable-sudo`, `--description`, `--replace-host-key`, `max_command_chars`, `max_output_chars`, `vps doctor`, `vps export`, `vps import`, `su-exec`
- `0.2.0` adiciona overrides runtime `--password`, `--sudo-password`, `--timeout` e aliases camelCase
- Prefira 0.5.3+ para integridade SFTP (G1–G19), verbosidade graduada crate-scoped, roundtrip export/import, wire schema v3, SCP funcional + `tunnel --json` / `--bind`, automação SSH completa, cifragem default e supply-chain limpa


## Descoberta para agentes
- `ssh-cli commands`: árvore completa com os 48 comandos folha, e `--json` publica todo argumento com `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` e `hidden`
- `ssh-cli --help`: termina com uma seção `Exit status:` que lista os 14 exit codes, e todas as 48 folhas trazem uma seção `Examples:` no próprio `--help`
- `ssh-cli schema [NAME]`: catálogo de JSON Schema embarcado com 25 schemas, ou o corpo de um schema
- `ssh-cli doctor`: alias raiz de `vps doctor`, com o relatório local e um probe SSH opcional
- `ssh-cli locale show|set|clear`: preferência de locale da UI sob XDG, nunca uma variável de ambiente de produto
- `ssh-cli tls provider|paths|mtls|acme`: material opcional de SSH-over-TLS sob XDG `tls/`


## Tabela resumo

| Agente / Plataforma | Estilo de integração | JSON | Notas | Documentação oficial |
| --- | --- | --- | --- | --- |
| Claude Code | CLI em subprocesso + skill | sim | Prefira o pacote de skill | [docs](https://code.claude.com/docs/en/overview) |
| Cursor | ferramentas de shell e de agente | sim | Use `--json` | [docs](https://cursor.com/docs) |
| Windsurf | ferramenta de shell | sim | One-shot por tarefa | [docs](https://docs.windsurf.com) |
| Codex CLI | ferramenta de shell | sim | Mapeie os sysexits | [docs](https://github.com/openai/codex) |
| OpenCode | ferramenta de shell | sim | Somente one-shot | [docs](https://opencode.ai/docs) |
| Aider | comandos de shell | sim | Cadastre os hosts uma vez | [docs](https://aider.chat/docs/) |
| Continue | comando customizado | sim | Multi-host XDG | [docs](https://docs.continue.dev) |
| Gemini CLI | ferramenta de shell | sim | Prefira segredos por stdin | [docs](https://github.com/google-gemini/gemini-cli) |
| OpenHands | shell em sandbox | sim | Limite os timeouts de tunnel | [docs](https://docs.openhands.dev/overview/introduction) |
| Generic Shell | instalação direta em bash, zsh, fish, elvish ou PowerShell | sim | Completions disponíveis | [docs](https://www.gnu.org/software/bash/manual/) |


## Claude Code
- Documentação oficial: [Claude Code](https://code.claude.com/docs/en/overview)
- Estilo de integração: a ferramenta Bash roda `ssh-cli` como subprocesso, e o pacote de skill ensina a superfície inteira
- Instale o binário no PATH do host com `cargo install ssh-cli --locked`
- Carregue [skills/ssh-cli-pt/SKILL.md](skills/ssh-cli-pt/SKILL.md) ou o pacote en
- Cadastre hosts uma vez com `vps add --password-stdin` e chame `exec` por tarefa
- Prefira envelopes `--json`, faça parse do stdout e leia o envelope de falha no stderr
- O stderr padrão fica silencioso no nível de tracing `error`, e `-v`, `-vv` ou `-vvv` o elevam restrito ao crate enquanto o `RUST_LOG` do ambiente é ignorado
- Rode `ssh-cli secrets status`, `ssh-cli doctor --json` e `ssh-cli commands --json` como preflight de cifragem, caminhos e descoberta de superfície

```bash
printf '%s' "$PASS" | ssh-cli vps add --name prod --host h.example --user deploy --password-stdin --json
ssh-cli exec prod 'hostname' --json
ssh-cli doctor --json
```


## Cursor
- Documentação oficial: [Cursor](https://cursor.com/docs)
- Estilo de integração: a ferramenta de terminal do agente roda `ssh-cli`, com `--json` para resultados estruturados
- Adicione uma regra de projeto que prefere `ssh-cli` a processos Node SSH de longa duração
- Mantenha credenciais fora do chat usando hosts salvos e flags de segredo por stdin
- Faça parse só do JSON em stdout e leia o exit code primeiro
- O stderr padrão fica silencioso no nível de tracing `error`, então ignore o tracing salvo se passar `-v`, `-vv` ou `-vvv`
- Use `--select name,stdout,exit_code` em execuções de frota para manter cada resultado pequeno

```bash
ssh-cli exec --hosts web1,web2 'uptime' --json --select name,stdout,exit_code
```


## Windsurf
- Documentação oficial: [Windsurf](https://docs.windsurf.com)
- Estilo de integração: a ferramenta de shell roda um comando `ssh-cli` por tarefa
- Invoque comandos one-shot depois do cadastro de hosts
- Nunca mantenha um tunnel aberto sem `--timeout-ms`, que o `tunnel` exige de qualquer forma
- Espere o evento `tunnel_listening` antes de usar a porta local
- Leia `tunnel_closed` para saber por que o tunnel terminou
- Um prazo vencido depois do bind sai 0, então trate o exit 0 como o fim normal de um tunnel

```bash
ssh-cli tunnel prod 8080 127.0.0.1 80 --timeout-ms 60000 --json
```


## Codex CLI
- Documentação oficial: [Codex CLI](https://github.com/openai/codex)
- Estilo de integração: a ferramenta de shell roda `ssh-cli` e o agente lê o exit code
- Trate exits diferentes de zero como falhas tipadas usando a tabela `Exit status:` no fim do `ssh-cli --help`
- Mapeie o exit code antes de qualquer retry: 77 é auth, 64 é uso, 65 é dados e 66 é não encontrado
- Retente só quando `retryable` for true no envelope de falha, nunca em erros de auth ou de uso
- Passe o comando como um único argumento entre aspas, para o shell remoto recebê-lo intacto
- Um comando remoto que falhou sai com 1, e o envelope no stderr traz `remote_exit_code` enquanto o JSON no stdout tem `exit_code`

```bash
ssh-cli exec prod 'systemctl is-active nginx' --json
echo "exit=$?"
```


## OpenCode
- Documentação oficial: [OpenCode](https://opencode.ai/docs)
- Estilo de integração: a ferramenta de shell roda `ssh-cli` com argv explícito
- Use o modo de ferramenta de shell com arrays de argv explícitos
- Evite embutir senhas no texto do prompt
- Use o registro ou segredos por stdin
- Somente one-shot: não existe sessão para manter entre chamadas
- Rode `ssh-cli commands --json` uma vez para descobrir toda flag de todo comando

```bash
ssh-cli exec prod 'df -h /' --json
```


## Aider
- Documentação oficial: [Aider](https://aider.chat/docs/)
- Estilo de integração: comandos de shell rodados da sessão
- Documente nomes de host no repositório sem segredos
- Chame `ssh-cli exec <name> "..."` para operações remotas durante os loops de edição
- Cadastre o host uma vez com `vps add`, para o repositório nunca carregar credencial
- Reduza `max_output_chars` no host com `vps edit` para limitar a saída remota, e leia `truncated_stdout` para saber do corte
- Use `vps list --select name,host` para mostrar o registro sem nenhum segredo

```bash
ssh-cli vps list --select name,host --json
ssh-cli exec prod 'journalctl -u app -n 20 --no-pager' --json
```


## Continue
- Documentação oficial: [Continue](https://docs.continue.dev)
- Estilo de integração: um comando customizado mapeia para um subcomando de `ssh-cli`
- Mapeie comandos customizados para subcomandos de `ssh-cli` com `--json`
- Use `vps doctor --json` como preflight de saúde para sessões de agente
- Leia `local.config_dir_exposed` no preflight e avise quando for true
- Sonde o registro inteiro com `health-check --all --json` e liste as falhas com `--filter status!=ok`
- O registro XDG é compartilhado por toda sessão do mesmo usuário

```bash
ssh-cli doctor --json --select local.config_dir_exposed
ssh-cli health-check --all --json
```


## Gemini CLI
- Documentação oficial: [Gemini CLI](https://github.com/google-gemini/gemini-cli)
- Estilo de integração: a ferramenta de shell roda `ssh-cli` e lê o JSON
- Prefira autenticação por chave e o `vps show` mascarado para verificação
- Mantenha a elevação desligada com `vps edit <name> --disable-sudo` salvo se a tarefa exigir root
- Religue a elevação depois com `vps edit <name> --enable-sudo`
- Prefira segredos por stdin, porque senha no argv aparece na lista de processos
- Use `--use-agent` para autenticar por um agente SSH sem arquivo de chave

```bash
ssh-cli vps show prod --json
ssh-cli vps edit prod --disable-sudo --json
```


## OpenHands
- Documentação oficial: [OpenHands](https://docs.openhands.dev/overview/introduction)
- Estilo de integração: o shell do sandbox roda `ssh-cli` dentro do container
- Instale o binário na imagem com `cargo install ssh-cli --locked`
- Rode dentro do sandbox com uma política de rede que permita só os hosts alvo
- Force tunnels limitados e timeouts curtos
- Limite o fan-out com `--max-concurrency`, de 1 a 64, quando o sandbox for pequeno
- Use `--fail-fast` para parar de admitir hosts depois da primeira falha

```bash
ssh-cli tunnel prod 8080 127.0.0.1 80 --timeout-ms 30000 --json
ssh-cli health-check prod --timeout 5000 --json
```


## Shell genérico
- Documentação oficial: [Shell genérico](https://www.gnu.org/software/bash/manual/)
- Estilo de integração: instalação direta, sem agente no meio
- Instale completions com `ssh-cli completions <SHELL>` para bash, zsh, fish, elvish ou powershell
- Use `--config-dir` só em sandboxes de teste isolados, porque o produto não lê variável de ambiente para isso
- Encadeie o JSON do stdout em `jaq` para escolher um campo
- Leia o exit code com `$?` antes de fazer parse de qualquer coisa
- Redirecione o stderr para um arquivo quando quiser guardar o envelope de falha

```bash
ssh-cli completions bash > ssh-cli.bash
ssh-cli commands --json | jaq '.tree.subcommands | length'
```


## Referência de comandos
- Os 48 comandos folha de `ssh-cli commands`, um bullet para cada
- `ssh-cli vps add`: Adiciona um VPS novo ao registro
- `ssh-cli vps list`: Lista todos os hosts VPS (senhas mascaradas)
- `ssh-cli vps remove`: Remove um VPS do registro
- `ssh-cli vps rename`: Renomeia um VPS registrado sem tocar nas credenciais
- `ssh-cli vps edit`: Edita campos de um VPS existente
- `ssh-cli vps show`: Mostra os detalhes de um VPS (senhas mascaradas)
- `ssh-cli vps path`: Mostra o caminho do arquivo de configuração
- `ssh-cli vps doctor`: Diagnóstico das camadas XDG, do caminho e do schema
- `ssh-cli vps export`: Exporta hosts (senhas redigidas por padrão)
- `ssh-cli vps import`: Importa hosts de um arquivo TOML ou de um envelope JSON `vps-export` (chaves EN e PT legadas)
- `ssh-cli connect`: Define o VPS ativo (grava o arquivo irmão `active` no diretório de configuração)
- `ssh-cli exec`: Roda um comando no VPS por SSH (stdout e stderr capturados)
- `ssh-cli sudo-exec`: Roda um comando com `sudo` (empacotamento seguro com `sh -c`)
- `ssh-cli su-exec`: Roda um comando com elevação `su -` de uso único
- `ssh-cli scp upload`: Envia um arquivo local ao host remoto (somente arquivos regulares)
- `ssh-cli scp download`: Baixa um arquivo remoto para o host local (somente arquivos regulares)
- `ssh-cli sftp upload`: Envia um arquivo ou árvore local ao host remoto por SFTP
- `ssh-cli sftp download`: Baixa um arquivo ou árvore remota ao host local por SFTP
- `ssh-cli sftp ls`: Lista um diretório remoto
- `ssh-cli sftp mkdir`: Cria um diretório remoto
- `ssh-cli sftp rmdir`: Remove um diretório remoto vazio
- `ssh-cli sftp rm`: Remove um arquivo remoto
- `ssh-cli sftp stat`: Mostra os metadados de um caminho remoto
- `ssh-cli sftp rename`: Renomeia um caminho remoto
- `ssh-cli tunnel`: Túnel SSH com prazo obrigatório (one-shot limitado)
- `ssh-cli health-check`: Verifica a conectividade SSH com um VPS (`--all`, `--hosts` ou `--use-active`)
- `ssh-cli secrets status`: Mostra o estado da cifragem (sem material sensível)
- `ssh-cli secrets init`: Gera e guarda a chave primária (`secrets.key` ou keyring) e nunca a imprime
- `ssh-cli secrets reencrypt`: Regrava o `config.toml` recifrando os segredos com a chave atual
- `ssh-cli completions`: Gera scripts de completion para o shell
- `ssh-cli commands`: Emite a árvore completa de comandos em JSON (descoberta por agente, regras `mycli commands`)
- `ssh-cli schema`: Emite o catálogo de JSON Schema embarcado ou o corpo de um schema
- `ssh-cli doctor`: Alias raiz de `vps doctor` (diagnóstico de XDG e schema)
- `ssh-cli locale show`: Mostra o idioma resolvido, a camada vencedora e os idiomas disponíveis (padrão)
- `ssh-cli locale set`: Grava o idioma preferido no diretório de configuração (arquivo `lang`, modo 0600)
- `ssh-cli locale clear`: Remove a preferência de idioma gravada
- `ssh-cli tls provider`: Mostra o estado do CryptoProvider do rustls (`aws_lc_rs`)
- `ssh-cli tls paths`: Imprime os caminhos do layout XDG de TLS
- `ssh-cli tls mtls list`: Lista os nomes das identidades importadas
- `ssh-cli tls mtls import`: Importa certificado e chave PEM como identidade nomeada
- `ssh-cli tls mtls show`: Mostra os caminhos de uma identidade
- `ssh-cli tls mtls remove`: Remove o diretório de uma identidade
- `ssh-cli tls acme account create`: Cria uma conta ACME (credenciais em `tls/acme/account.json` no XDG, modo 0600)
- `ssh-cli tls acme account show`: Mostra se existe conta e o caminho dela
- `ssh-cli tls acme issue`: Inicia o pedido DNS-01 e imprime o desafio TXT (grava a URL do pedido no XDG)
- `ssh-cli tls acme complete`: Conclui um pedido pendente depois que o TXT de DNS é publicado
- `ssh-cli tls acme status`: Mostra o estado do certificado ou do pedido de um domínio ou de todos
- `ssh-cli tls acme list`: Lista os diretórios de domínios ACME no XDG


## Flags por folha
- As flags que uma integração precisa passar em cada folha, no mesmo conjunto que `ssh-cli commands --json` publica
- `ssh-cli vps add`: `--name`, `--host`, `--port`, `--user`, `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--sudo-password`, `--sudo-password-stdin`, `--su-password`, `--su-password-stdin`, `--tag`, `--timeout`, `--max-chars`, `--max-command-chars`, `--max-output-chars`, `--tls`, `--tls-sni`, `--tls-client-cert`, `--tls-client-key`, `--use-agent`, `--agent-socket`, `--check`, `--disable-sudo`
- `ssh-cli vps edit`: as mesmas flags de registro mais `--enable-sudo`, `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-key`, `--no-agent-socket`, `--no-tls`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key`
- `ssh-cli vps list`: `--tag`
- `ssh-cli vps import`: `--file` e `--allow-incomplete`
- `ssh-cli vps export`: `--output`, `--include-secrets`, `--i-understand-secrets-on-stdout`
- `ssh-cli vps doctor` e `ssh-cli doctor`: `--hosts` e `--probe-ssh`
- `ssh-cli exec`, `ssh-cli sudo-exec`, `ssh-cli su-exec`: `--all`, `--hosts`, `--tags`, `--use-active`, `--use-agent`, `--agent-socket`, `--step`, `--description`, `--timeout`, e nas folhas de elevação `--sudo-password` ou `--su-password` com as gêmeas `-stdin`
- `ssh-cli health-check`: `--all`, `--hosts`, `--use-active`, `--use-agent`, `--agent-socket`, `--timeout`
- `ssh-cli scp upload` e `ssh-cli scp download`: `--src` e `--dest`, mais `--all`, `--hosts` e o trio de autenticação
- `ssh-cli sftp upload` e `ssh-cli sftp download`: `--recursive`, `--src` e `--dest`, mais `--all` e `--hosts`
- As outras folhas de SFTP (`ls`, `mkdir`, `rmdir`, `rm`, `stat`, `rename`) usam o trio de autenticação e `--timeout`
- `ssh-cli tunnel`: `--reverse`, `--socks5`, `--remote-socket`, `--bind`, `--i-accept-network-exposure`, `--timeout-ms`
- `ssh-cli secrets init`: `--keyring` e `--force`
- `ssh-cli tls mtls import`: `--cert`, `--key` e `--name`
- `ssh-cli tls acme account create`: `--contact`, `--staging` e `--force`
- `ssh-cli tls acme issue`: `--domain`, `--staging` e `--print-challenge`
- `ssh-cli tls acme complete` e `ssh-cli tls acme status`: `--domain`


## Flags globais
- As 27 flags globais, válidas antes ou depois do subcomando
- `--lang <LOCALE>`: Força o idioma da CLI (`en` ou `pt-BR`, e tags BCP47 negociam para eles)
- `-v, --verbose`: Aumenta a verbosidade dos logs no stderr (`-v` info, `-vv` debug, `-vvv` trace), sempre restrita a este crate
- `-q, --quiet`: Suprime a saída que não é JSON
- `--config-dir <DIR>`: Substitui o diretório de configuração, e um caminho terminado em `.toml` faz do pai dele o diretório
- `--no-color`: Desliga a saída colorida
- `--output-format <text|json>`: Escolhe `text` ou `json`, e o padrão é JSON quando o stdout não é um TTY
- `--json`: Força a saída JSON, alias de `--output-format json`
- `--select <PATHS>`: Mantém só estes caminhos com pontos em cada registro (CSV, alias `--fields`)
- `--filter <EXPR>`: Mantém registros que casam com `key=value`, `key!=value` ou `key~substring`, repetível com AND
- `--limit <N>`: Emite no máximo N registros
- `--sort <PATH>`: Ordena os registros de forma crescente pelo caminho com pontos, comparando números como números
- `--dedupe-by <PATH>`: Descarta os registros seguintes que repetem o valor deste caminho com pontos
- `--count-only`: Troca a coleção de registros por `{"count": N}`, contado depois do filtro
- `--truncate-content <CHARS>`: Encurta strings maiores que N caracteres, nunca em bytes
- `--max-output-bytes <BYTES>`: Limita o tamanho do envelope descartando registros do fim, nunca cortando o texto JSON
- `--no-input`: Recusa ler o stdin e falha na hora em vez de bloquear
- `--dry-run`: Imprime o plano e sai sem executar, aceita só por oito operações e rejeitada com exit 64 nas demais
- `--lock-wait-ms <MS>`: Limita a espera pelo lock do config (padrão 5000, máximo 300000) antes do exit 69
- `--disable-sudo`: Desliga `sudo-exec` e `su-exec` nesta invocação (alias `--disableSudo`), e antes de `vps add` ou `vps edit` também grava `disable_sudo` no host
- `--replace-host-key`: Substitui uma chave de host divergente no `known_hosts` TOFU
- `--allow-plaintext-secrets`: Permite segredos em texto puro em repouso, só para testes
- `--secrets-key-file <PATH>`: Lê a chave primária de 64 hex deste arquivo em uma invocação
- `--use-keyring`: Prefere o keyring do SO para a chave primária
- `--timeout <MS>`: Define o timeout SSH global em milissegundos, e um `--timeout` local vence, enquanto o `tunnel` ainda exige `--timeout-ms`
- `--max-concurrency <N>`: Limita as sessões multi-host e os forwards de tunnel concorrentes, de 1 a 64
- `--fail-fast`: Para de admitir unidades multi-host novas depois da primeira falha
- `--scp-file-concurrency <N>`: Define as transferências SCP concorrentes em uma sessão SSH (padrão 1)
