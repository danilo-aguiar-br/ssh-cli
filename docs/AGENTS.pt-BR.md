# Guia de agentes para ssh-cli

> **G-E2E-16:** Prefira GraphRAG `list` / `read` pelo nome exato da memória a `hybrid-search` sob carga.
>
> **G-E2E-04 wire:** OBRIGATÓRIO um documento JSON por sucesso one-shot no data path.
> PROIBIDO: parsear dual-events NDJSON multi-linha como data path de sucesso.
> O campo `secrets_key_auto_created` (quando presente) vive no **mesmo** documento `vps-added` — nunca um segundo evento no stdout.
>
> **Descoberta:** `ssh-cli commands`, `ssh-cli schema`, `ssh-cli doctor` (alias root de `vps doctor`).

> Corte o desperdício de RAM de processos residentes e mantenha SSH multi-host sob controle do agente.

- Leia este documento em [inglês](AGENTS.md).
- Combine com [../INTEGRATIONS.pt-BR.md](../INTEGRATIONS.pt-BR.md) e [../skills/ssh-cli-pt/SKILL.md](../skills/ssh-cli-pt/SKILL.md).
- Linha de produto: 0.5.2.


## Por quê
### Substitua processos SSH Node de longa duração por binário nascer-executar-morrer
- Sessões daemon persistentes queimam RAM com sockets ociosos.
- Um host por processo daemon multiplica processos para N servidores.
- Um binário Rust com storage XDG multi-host colapsa essa proliferação.
- Agentes ganham JSON determinístico e sysexits sem overhead de protocolo extra.


## Economia
### Meça o ganho operacional
- Cold start alvo abaixo de 100 ms em hosts Linux típicos.
- Memória do processo volta ao SO após cada comando.
- Sem taxa de runtime Node e sem gerenciador SSH permanente.
- Uma instalação serve Claude Code, Cursor, Windsurf, Codex e agentes shell.


## Soberania
### Mantenha credenciais e confiança de host locais
- Guarde hosts sob XDG sem proliferação de `.env`.
- Prefira chaves privadas e segredos via stdin a senhas coladas no chat.
- Cifragem at-rest por padrão (ChaCha20-Poly1305 + auto `secrets.key`); gerencie com `secrets status|init|reencrypt`.
- Force known_hosts TOFU para dificultar MITM silencioso.
- Desabilite elevação quando o workflow deve permanecer sem privilégio.
- PROIBIDO: logar primary-key, senhas de host ou segredos decifrados.


## Agentes e orquestradores compatíveis
- Claude Code com o pacote de skill embarcado
- Cursor com shell ou agent tools
- Windsurf shell tool
- Codex CLI shell tool
- OpenCode shell tool
- Aider, Continue, Gemini CLI, OpenHands, bash/zsh genérico


## Detalhes de integração do agente
### Contrato imperativo para autores
- OBRIGATÓRIO: invocar `ssh-cli` como subprocesso e aguardar o exit (one-shot).
- OBRIGATÓRIO: parsear JSON de stdout quando `--json` ou `--output-format json` (JSON auto se stdout não é TTY).
- OBRIGATÓRIO: tratar tracing em stderr como log fora de contrato; não parsear stderr como JSON de sucesso.
- OBRIGATÓRIO: quando o modo de erros JSON está ativo (`--json` / JSON efetivo em scp|tunnel|formato global), parsear envelopes de falha em **stderr** (`exit_code`, `message`, opcional `remote_exit_code`) via `docs/schemas/error-envelope.schema.json`.
- OBRIGATÓRIO: esperar tracing padrão no nível error; use `-v` só ao diagnosticar.
- PROIBIDO: tratar `RUST_LOG` ambiente como config de produto — é ignorado; só `-v` controla o tracing debug.
- OBRIGATÓRIO: cadastrar hosts com `vps add` antes de trabalho remoto repetido (auth: password **ou** key **ou** `--use-agent` / `--agent-socket`).
- OBRIGATÓRIO: fornecer senha ou chave; credencial vazia é rejeitada na gravação.
- OBRIGATÓRIO: tratar senha vazia em list/show JSON como `null` (hosts só-chave); não vazia mascara `***`.
- OBRIGATÓRIO: comando remoto vazio falha com mensagem técnica `empty command` (sempre em inglês) e exit de uso de domínio 64.
- OBRIGATÓRIO: passar `--timeout-ms` em toda invocação de `tunnel`.
- OBRIGATÓRIO: tratar `scp` como **somente arquivos regulares** (sem diretórios, sem `-r`). Para árvores / FS remoto use `sftp` (`upload|download --recursive`, `ls`, `mkdir`, `rm`, `stat`, `rename`).
- OBRIGATÓRIO: nunca depender do crates.io 0.3.9 para SCP; o wire estava quebrado — exija 0.5.2+.
- OBRIGATÓRIO: parsear sucesso SCP com `docs/schemas/scp-transfer.schema.json` (`ok`, `event` = `"scp-transfer"`, `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`) no **stdout**.
- OBRIGATÓRIO: arquivo local/remoto ausente no SCP sai com exit 66 e mensagem `file not found: <path>` (path canônico/normalizado; sem prefixos `SCP:` empilhados).
- OBRIGATÓRIO: corpo padrão de `vps export` é TOML mesmo em pipe/non-TTY; envelope JSON de agente só com `vps export --json` → `event: "vps-export"` (JSON auto non-TTY **não** se aplica ao export).
- OBRIGATÓRIO: tratar `vps export` redacted como sem segredos vivos; secret vazio serializa como `""` e nunca blob `sshcli-enc:` (EXP-001).
- OBRIGATÓRIO: `--include-secrets` exige `-o`/`--output` ou `--i-understand-secrets-on-stdout`.
- OBRIGATÓRIO: `vps import` aceita TOML (chaves EN + aliases PT no load) ou JSON `vps-export`; use `--allow-incomplete` para hosts redacted/skeleton.
- OBRIGATÓRIO: `added_at` / `adicionado_em` são opcionais no import (serde usa o instante atual quando omitidos).
- OBRIGATÓRIO: wire format schema v3 dual-read — serializa chaves EN; load ainda aceita aliases PT (`nome`/`porta`/`usuario`/`senha`/…).
- OBRIGATÓRIO: preferir flags de secrets `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` a variáveis de ambiente; preferir o termo primary-key; o keyring ainda pode aceitar o alias legado `secrets-master-key` na leitura.
- OBRIGATÓRIO: `secrets init --json` / `secrets reencrypt --json` emitem `secrets-init` / `secrets-reencrypt` (`docs/schemas/secrets-init.schema.json`, `docs/schemas/secrets-reencrypt.schema.json`); a 1ª gravação pode definir `secrets_key_auto_created: true` no mesmo JSON de sucesso (um documento). Catálogo: [docs/schemas/README.md](schemas/README.md).
- OBRIGATÓRIO: em `tunnel --json`, aguardar um objeto stdout com `event: "tunnel_listening"` (`docs/schemas/tunnel-listening.schema.json`) antes de usar a porta local; o processo permanece vivo até timeout ou sinal; após `tunnel_listening`, o deadline pós-bind sai com exit 0 (TUN-002); timeout pré-bind permanece 74.
- OBRIGATÓRIO: tunnel `--bind` tem padrão `127.0.0.1` (loopback).
- PERMITIDO: passar em `tunnel` / `health-check` as flags de auth `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` (paridade com exec/scp, CLI-005/006).
- PERMITIDO: passar `health-check --timeout <ms>` quando o timeout padrão do host for longo ou curto demais.
- OBRIGATÓRIO: preferir fan-out multi-host para frota — `exec|sudo-exec|su-exec|scp|health-check --all` **ou** `--hosts a,b,c` roda sessões **concorrentes com bound** (`Semaphore` + `JoinSet`), não um host por spawn de processo. JSON batch aplica-se a ambos os modos multi.
- OBRIGATÓRIO: parsear JSON multi-host via schemas batch: `health-check-batch` / `exec-batch` / `scp-batch` (`docs/schemas/*-batch.schema.json`); o envelope inclui `max_concurrency`.
- PERMITIDO: limitar fan-out com global `--max-concurrency N` (1..=64; auto = CPUs×4 vs RAM livre/2 / 16 MiB, clamp 1..=64). O mesmo gate limita accepts do tunnel.
- PROIBIDO: assumir multi-host sequencial por padrão quando `--all` existe — o wall-clock é dominado por RTT SSH; sessões concorrentes são o modus operandi do produto.
- OBRIGATÓRIO: valores de timeout abaixo de 1000 ms e valores de senha em argv emitem warn em stderr — não parsear essas linhas como envelope JSON de erro.
- OBRIGATÓRIO: preferir `--password-stdin` / `--key` a segredos em argv.
- OBRIGATÓRIO: instalar com `cargo install ssh-cli --locked` (ou path com pins).
- PROIBIDO: assumir conexão SSH longa entre runs de processo.
- PROIBIDO: reintroduzir packaging de daemon de longa duração neste repositório.
- PROIBIDO: habilitar ou emitir telemetria de produto.
- PROIBIDO: retry cego em exit 64, 65, 66 ou 77.
- PROIBIDO: parsear dual-events NDJSON multi-linha no data path de sucesso — um documento JSON por sucesso one-shot; `secrets_key_auto_created` (quando setado) fica no mesmo objeto `vps-added`.
- PROIBIDO: tratar `RUST_LOG` ambiente como config de produto (ignorado; só `-v`).
- PROIBIDO: imprimir ou armazenar material de primary-key dos comandos `secrets`.
- PROIBIDO: tratar árvores de diretório SCP ou `-r` recursivo como suportados.
- PROIBIDO: assumir que o host do agente executa binários OpenSSH para o produto —
  `ssh-cli` é Rust puro (`russh`); sem spawn local de `ssh`/`scp`/`ssh-keygen` em runtime.
- OBRIGATÓRIO: tratar strings de comando remoto como input hostil; bytes NUL são
  rejeitados com invalid-argument antes do exec no canal SSH (G-PROC-03).


## Integrações de crate
- Consumidores publicados dependem do contrato da CLI, não de API de lib instável.
- Fixe experimentos de lib em versão exata se linkar `ssh_cli` como lib.
- Prefira integração via binário no PATH para agentes.


## Contrato CRUD e JSON
### Operações legíveis por máquina
- Listar hosts: `ssh-cli vps list --json` retorna array de objetos mascarados.
- Mostrar host: `ssh-cli vps show <name> --json` retorna um objeto mascarado.
- Descoberta: `ssh-cli commands`, `ssh-cli schema [NAME]`, `ssh-cli doctor` (alias de `vps doctor`).
- Doctor: `ssh-cli vps doctor --json` (ou `ssh-cli doctor --json`) retorna camada, paths, schema, contagem de hosts, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out` (booleano JSON), telemetry false.
- Secrets: `ssh-cli secrets status --json` retorna modo de cifragem sem material de chave; `secrets init --json` → `event: "secrets-init"`; `secrets reencrypt --json` → `event: "secrets-reencrypt"`.
- Eventos de sucesso CRUD quando JSON está efetivo (`--json` / `--output-format json` / JSON auto non-TTY): `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import` (com campo opcional `secrets_key_auto_created` quando uma chave é auto-criada — ainda um documento).
- Família exec (host único): `ssh-cli exec|sudo-exec|su-exec <vps> <cmd> --json` retorna stdout, stderr, exit_code, flags de truncagem, duration_ms.
- Família exec (frota): `ssh-cli exec|sudo-exec|su-exec --all '<cmd>' --json` ou `--hosts a,b '<cmd>'` → `event: "exec-batch"` (`exec-batch.schema.json`); falha parcial por host não aborta irmãos.
- Tunnel: **single-host** (um bind + uma sessão por one-shot). Multi-host = N invocações com ports/`--bind` distintos.
- Doctor: `vps doctor [--json]` emite um único root `event: vps-doctor` (`local` + `ssh_probe: null`). Use `--probe-ssh` para health multi-host embutido em `ssh_probe` (opcional `--hosts a,b`). Nunca dois roots JSON.
- SCP multi-arquivo (single-host): `scp upload <VPS> f1 f2 … <REMOTE_DIR>` / download simétrico — **uma sessão SSH** e transfers seriais (auth uma vez; G-PAR-47).
- SCP multi-host × multi-arquivo: `scp upload --all f1 f2 … <REMOTE_DIR>` ou `--hosts a,b` — bound de **sessões** por host; arquivos seriais na sessão (G-PAR-48). Download multi-arquivo frota grava em `<LOCAL_DIR>/<host>/`.
- Health (único): `ssh-cli health-check [<vps>] [--timeout <ms>] [--password-stdin|--key|--key-passphrase[-stdin]] --json` retorna name, status, latency_ms.
- Health (frota): `ssh-cli health-check --all --json` → `event: "health-check-batch"` (`health-check-batch.schema.json`).
- SCP (único): `ssh-cli scp upload|download <vps> <local> <remote> --json` retorna sucesso de transferência no stdout (`scp-transfer.schema.json` com `event: "scp-transfer"`); falhas usam envelope de erro em stderr; arquivo ausente → exit 66 `file not found: <path>` (path canônico/normalizado).
- SCP (frota / multi-arquivo batch): `event: "scp-batch"` (`scp-batch.schema.json`); download um arquivo frota grava `local.<vps>`; multi-arquivo frota usa subdirs por host; `name` pode ser `host:path`.
- Fatos operacionais SCP: exija 0.5.2+; upload faz stream de 32 KiB; download grava `{path}.ssh-cli.partial` e renomeia; mtime/mode preservados nos dois sentidos.
- Tunnel: `ssh-cli tunnel <vps> <porta_local> <host_remoto> <porta_remota> --timeout-ms <ms> [--bind 127.0.0.1] [--password-stdin|--key|--key-passphrase[-stdin]] --json` emite `tunnel_listening` no stdout após o bind; `--bind` padrão `127.0.0.1`; deadline pós-bind → exit 0; timeout pré-bind permanece 74.
- Export: corpo padrão de `ssh-cli vps export` é TOML (mesmo em pipes); vazios como `""` (nunca `sshcli-enc:`). Use `vps export --json` para envelope de agente `event: "vps-export"`. `--include-secrets` exige `-o` ou `--i-understand-secrets-on-stdout`.
- Import: `ssh-cli vps import --file <path> [--allow-incomplete]` aceita TOML (serialize EN / aliases PT no load) ou JSON `vps-export`; `added_at` / `adicionado_em` opcionais (padrão: agora).
- Campos de senha vazios serializam como JSON `null`; segredos não vazios mascaram como `***` (`FIXED_MASK`). Em `vps export` redacted, não vazios → `***`; vazios → `""`.
- Valide payloads contra schemas em `docs/schemas/`; índice: [docs/schemas/README.md](schemas/README.md).


## Roteamento de exit codes
- Exit 0 significa sucesso.
- Exit 1 significa falha genérica de runtime; inspecione stderr.
- Exit 64 significa erro de uso/argumento (incluindo comando vazio) **ou** validação ACME permanente (`invalidContact` / 4xx); corrija argv/contact, não faça retry.
- Exit 65 (`TomlDe` / JSON / schema) significa erro de parse/dados; corrija o payload.
- Exit 66 significa VPS ou arquivo ausente (`file not found: <path>` no SCP); cadastre ou corrija o nome/path.
- Exit 73 significa falha de escrita de config; cheque permissões e disco.
- Exit 74 significa falha de IO/conexão SSH; retry de rede pode ajudar.
- Exit 77 significa falha de auth ou política de host-key; tente `--key` / `--password-stdin` / passphrase stdin; sem retry cego.
- Exit 130/143 significa término por sinal.


## Estratégia de retry
- Prefira os campos do envelope JSON `retryable` + `error_class` a heurísticas só de exit (`docs/schemas/error-envelope.schema.json`).
- Retry no máximo duas vezes quando `retryable: true` / exit 74 com backoff **exponencial full-jitter** (base 200ms, teto 5s; ver `ssh_cli::retry::RetryConfig::agent_default`).
- Nunca faça retry em `retryable: false` ou exits 64, 65, 66, 77, 1 (comando remoto), 130/143/141 sem mudar as entradas.
- Validação ACME permanente (`invalidContact` / 4xx) é exit **64**, não exit 74 — **não** trate como IO de rede retentável.
- O binário **não** faz auto-retry de `exec`/`scp` não idempotentes in-process (one-shot / menor privilégio); o agente reinvoca o processo.
- Encurte ou divida comandos quando o exit indicar rejeição por max_command_chars.
- Confirme mudanças de host key com humano antes de `--replace-host-key`.
