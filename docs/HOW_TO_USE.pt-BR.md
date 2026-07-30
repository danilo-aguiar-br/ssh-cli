# Como usar ssh-cli

> Vá da instalação ao primeiro comando remoto em menos de 60 segundos.

- Leia este documento em [inglês](HOW_TO_USE.md).
- Volte ao [README.pt-BR.md](../README.pt-BR.md) para o mapa completo de comandos.
- Linha de produto documentada aqui: 0.5.3.


## Pré-requisitos
- Instale Rust MSRV 1.85.0 ou superior via rustup.
- Garanta conectividade de rede até o host SSH alvo.
- Tenha senha ou chave privada OpenSSH para esse host.
- Prefira um XDG config home gravável para storage multi-host.
- Instale com `cargo install ssh-cli --locked` (0.5.3+ no crates.io; evite 0.3.9 para SCP).
- Não dependa do crates.io 0.3.9 para SCP: aquela release anunciava transferência, mas o protocolo wire estava quebrado (arquivos remotos de 0 bytes ou timeouts). Use 0.5.3+.
- Prefira **0.5.3+** para SFTP: builds anteriores podiam truncar arquivos remotos a zero bytes no upload (G1). Verifique com `sha256sum` após a transferência.


## Primeiro comando em 60 segundos
### Instale, cadastre, execute

```bash
cargo install ssh-cli --locked
# A primary-key é auto-criada na primeira gravação de segredo; init explícito é opcional:
ssh-cli secrets init
ssh-cli vps add --name demo --host 203.0.113.10 --user ubuntu --key ~/.ssh/id_ed25519
ssh-cli exec demo "uname -a" --json
```

- Confirme exit code 0 e inspecione campos JSON `stdout`, `stderr`, `exit_code`, `duration_ms`.
- Em sucesso com `--json`, parseie **exatamente um** objeto JSON no stdout (G8) — não dual-events multi-linha.
- Um comando remoto vazio falha com a mensagem técnica `empty command` (sempre em inglês) e exit de uso de domínio 64.
- Rode `ssh-cli secrets status --json` e `ssh-cli doctor --json` (ou `vps doctor --json`) quando path ou cifragem estiverem incertos.
- Descubra contratos: `ssh-cli schema` / `ssh-cli commands`.
- Cadastre hosts com agent-auth via `vps add --use-agent` (opcional `--agent-socket`).
- Prefira `--password-stdin` a `--password` ao cadastrar hosts com senha.


## Comandos centrais
### Inventário completo de comandos

| Comando | Propósito |
| --- | --- |
| `vps add` | Cadastrar um host (senha **ou** chave **ou** `--use-agent`) |
| `vps list` | Listar hosts (segredos mascarados) |
| `vps remove` | Remover host do inventário |
| `vps edit` | Alterar campos do host (timeout, chaves, elevação, …) |
| `vps show` | Mostrar um host (segredos mascarados) |
| `vps path` | Imprimir path resolvido de config |
| `vps doctor` | Diagnosticar paths, schema, modo de secrets, probe SSH opcional |
| `vps export` | Exportar inventário (corpo padrão **TOML**; redacted por omissão) |
| `vps import` | Importar TOML ou envelope JSON `vps-export` |
| `connect` | Marcar o host ativo |
| `exec` | Rodar comando remoto (VPS ativa se o nome for omitido) |
| `sudo-exec` | Rodar via `sudo` remoto + packing seguro `sh -c` |
| `su-exec` | Rodar via `su` remoto quando a senha su estiver armazenada |
| `scp upload` | Upload de **arquivo(s) regular(es)** (sem diretórios / sem `-r`) |
| `scp download` | Download de **arquivo(s) regular(es)** (partial + rename atômico) |
| `sftp upload` | Upload SFTP (opcional `--recursive` para árvores) — prefira **0.5.3+** |
| `sftp download` | Download SFTP (opcional `--recursive`) |
| `sftp ls` | Listar diretório remoto |
| `sftp mkdir` | Criar diretório remoto |
| `sftp rmdir` | Remover diretório remoto vazio |
| `sftp rm` | Remover arquivo remoto |
| `sftp stat` | Stat de path remoto |
| `sftp rename` | Renomear/mover path remoto |
| `tunnel` | Port-forward local com `--timeout-ms` obrigatório |
| `health-check` | Sondar conectividade / latência |
| `secrets status` | Modo de cifragem sem imprimir a chave |
| `secrets init` | Criar primary-key (nunca imprime) |
| `secrets reencrypt` | Re-cifrar inventário sob a primary-key atual |
| `completions` | Scripts de completion no stdout |
| `commands` | Listar superfície de comandos para agentes |
| `schema [NAME]` | Listar ou emitir um schema JSON embarcado |
| `doctor` | Alias root de `vps doctor` |
| `locale show` | Mostrar idioma de UI resolvido e camada vencedora |
| `locale set` | Persistir preferência de idioma (XDG) |
| `locale clear` | Limpar preferência de locale armazenada |
| `tls provider` | Status do `CryptoProvider` rustls (`aws_lc_rs`) |
| `tls paths` | Paths do layout TLS em XDG |
| `tls mtls list` | Listar identidades mTLS importadas |
| `tls mtls import` | Importar cert/chave mTLS sob XDG |
| `tls mtls show` | Mostrar uma identidade mTLS (sem material de chave privada) |
| `tls mtls remove` | Remover identidade mTLS |
| `tls acme account create` | Criar conta ACME (exige `--contact mailto:…`) |
| `tls acme account show` | Mostrar metadados da conta ACME |
| `tls acme issue` | Iniciar order ACME (`--print-challenge` para DNS/HTTP) |
| `tls acme complete` | Completar order ACME após challenge |
| `tls acme status` | Status de order/cert ACME |
| `tls acme list` | Listar domínios ACME sob XDG |

### Loop diário do operador
- Liste hosts com `ssh-cli vps list --json`.
- Mostre um host com `ssh-cli vps show demo --json` (segredos mascarados).
- Altere campos com `ssh-cli vps edit demo --timeout 90000`.
- Marque host ativo com `ssh-cli connect demo`.
- Rode trabalho privilegiado com `ssh-cli sudo-exec demo "systemctl status nginx" --json` (packing seguro `sh -c`).
- Eleve com `ssh-cli su-exec` quando a senha `su` estiver no registro do host.
- Transfira **arquivos regulares** com `ssh-cli scp upload demo ./app.tgz /tmp/app.tgz` (sem diretórios / sem `-r`). Para árvores use `ssh-cli sftp upload --recursive demo ./dir /tmp/dir`.
- Baixe com `ssh-cli scp download demo /var/log/app.log ./app.log`.
- Prefira JSON de agente: `ssh-cli scp upload demo ./app.tgz /tmp/app.tgz --json` (schema `docs/schemas/scp-transfer.schema.json`; sucesso exige `event: "scp-transfer"`).
- Flags SCP com paridade ao exec: `--timeout` (connect + transfer), `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json`.
- Arquivo local/remoto ausente no SCP sai com exit 66 e mensagem `file not found: <path>` (path canônico/normalizado; sem prefixos `SCP:` empilhados).
- Download com falha não deixa o destino final corrompido: grava `{path}.ssh-cli.partial`, aplica mode/times no partial e faz rename atômico. Download SCP propaga falha de `sync_data` antes do rename (G9).
- Upload faz stream em blocos de 32 KiB (não carrega o arquivo inteiro na RAM).
- mtime/mode são preservados nos dois sentidos automaticamente (remoto `scp -tp` / `-fp`; sem flag extra do usuário).
- Gerencie a primary-key com `ssh-cli secrets status|init|reencrypt` (nunca imprime a chave). O keyring ainda pode aceitar o alias legado `secrets-master-key` na leitura.
- `secrets init --json` / `secrets reencrypt --json` emitem eventos de sucesso (`secrets-init`, `secrets-reencrypt`; schemas `docs/schemas/secrets-init.schema.json`, `docs/schemas/secrets-reencrypt.schema.json`); a 1ª gravação de segredo pode definir o campo `secrets_key_auto_created: true` no mesmo documento JSON `vps-added` (nunca um segundo evento no stdout). Veja [docs/schemas/README.md](schemas/README.md).
- Eventos JSON de sucesso CRUD quando JSON está efetivo: `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import` (com campo opcional `secrets_key_auto_created` quando uma chave é auto-criada — ainda um documento). Catálogo: [docs/schemas/README.md](schemas/README.md).


## SFTP (prefira 0.5.3+)
### Integridade, árvores e metadados
- Prefira a linha de produto **0.5.3+** para todo trabalho SFTP. **G1** corrigiu truncamento no upload: builds anteriores podiam abrir o arquivo remoto com atributos que zeravam o conteúdo do destino. Sempre verifique com checksum no destino (`sha256sum` / `sha256sum` remoto) — não confie só em contagem de bytes do cliente (G15).
- Árvores recursivas: `ssh-cli sftp upload --recursive demo ./tree /tmp/tree` e `sftp download --recursive …` (sem seguir symlink; caps de profundidade e listagem).
- SETSTAT envia `atime`+`mtime` juntos (G3); `set_metadata` mutante é fail-closed (G4); permissões usam `SFTP_PERM_MASK` `0o7777` (G12).
- Cancelamento multi-arquivo / batch mantém `results.len() == input.len()` com o restante marcado cancelled (G5/G17).
- JSON de agente: schemas `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch` em `docs/schemas/`.
- Exemplo de verificação de integridade:

```bash
ssh-cli sftp upload demo ./payload.bin /tmp/payload.bin --json
ssh-cli exec demo "sha256sum /tmp/payload.bin" --json
sha256sum ./payload.bin
# compare os digests — o efeito no destino é o critério de aceite
```


## Verbosidade (-v / -vv / -vvv)
- Nível de tracing padrão é **error** para manter stderr de JSON e tunnel limpos.
- Verbosidade graduada (G14): `-v` → **info**, `-vv` → **debug**, `-vvv` → **trace**.
- Filtros são sempre **com escopo na crate** (`warn,ssh_cli=…`) — nunca `debug` global nu (G2). Isso evita vazamento de senha via logs `russh::client::encrypted`.
- `RUST_LOG` ambiente é **ignorado**; só `-v`/`-vv`/`-vvv` da CLI controlam o tracing do produto.
- Quiet: `-q` silencia sucesso humano.
- Exemplo de diagnóstico sem vazamento de senha: `ssh-cli -vv exec demo "true" --json`.


## Locale
```bash
ssh-cli locale show
ssh-cli locale set pt-BR
ssh-cli locale clear
# override one-shot (não persiste):
ssh-cli --lang en vps list
```
- Preferência armazenada sob XDG (sem `.env` / sem store de idioma em env de produto).
- `locale show` reporta idioma resolvido e camada vencedora.


## TLS (SSH-over-TLS / mTLS / ACME)
```bash
ssh-cli tls provider
ssh-cli tls paths
ssh-cli tls mtls list
ssh-cli tls mtls import --name edge --cert ./client.pem --key ./client-key.pem
ssh-cli tls mtls show edge
ssh-cli tls mtls remove edge
ssh-cli tls acme account create --contact mailto:ops@example.com
ssh-cli tls acme account show
ssh-cli tls acme issue example.com --print-challenge
ssh-cli tls acme complete example.com
ssh-cli tls acme status example.com
ssh-cli tls acme list
```
- Stack é **rustls** + **aws_lc_rs** apenas (sem OpenSSL / native-tls no produto).
- Identidades mTLS e material ACME vivem sob XDG `tls/` (secrets mode 0o600).
- Validação ACME permanente (ex.: `invalidContact`) → exit **64** (não faça retry como 74).


## Daemon
### Não existe daemon
- Trate cada invocação como nascer-executar-morrer (one-shot).
- Nunca espere um worker SSH em background neste projeto.
- Limite tunnels com `--timeout-ms` obrigatório para o processo ainda encerrar.


## Padrões avançados
### Frota multi-host (concorrência limitada)
- Prefira `exec|sudo-exec|su-exec|scp|sftp|health-check --all` quando o inventário tiver mais de um host — um processo, sessões concorrentes limitadas por `--max-concurrency N` (auto CPUs×RAM quando omitido, clamp 1..=64).
- Parseie JSON batch via `docs/schemas/*-batch.schema.json` (`health-check-batch`, `exec-batch`, `scp-batch`, `sftp-batch`); o envelope inclui `max_concurrency`.
- Exemplo: `ssh-cli --max-concurrency 8 health-check --all --json` e depois `ssh-cli exec --all 'hostname' --json`.
- **Não** spawn um processo CLI por host para frota quando `--all` estiver disponível.
- Em cancelamento, resultados multi-arquivo SCP/SFTP mantêm a cardinalidade de entrada (G5/G17).

### Automação mais segura para agentes
- Alimente segredos por flags stdin (`--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--key-passphrase-stdin`) em vez de argv.
- Anexe comentários shell com `--description` para histórico remoto auditável.
- Desabilite elevação em tarefas não confiáveis com `--disable-sudo`.
- Substitua host key legítima só após confirmação humana com `--replace-host-key` (TOFU).
- Exporte inventário com segredos mascarados: `ssh-cli vps export -o hosts.toml` (corpo padrão é TOML, inclusive em pipe/non-TTY; segredos não vazios mascaram como `***` (`FIXED_MASK`); vazios ficam `""`; nunca blob `sshcli-enc:` de vazio; EXP-001 / G-E2E-10). Em list/show, senha vazia é JSON `null` — path diferente do export. O texto de help reflete esse comportamento TOML por padrão.
- Export JSON de agente só com `ssh-cli vps export --json` → envelope `event: "vps-export"` (JSON auto em non-TTY **não** se aplica a `vps export`).
- `--include-secrets` exige `-o`/`--output` ou `--i-understand-secrets-on-stdout` (pipe/stdout sem ack é recusado, exit 64).
- Importe hosts com `ssh-cli vps import --file hosts.toml` (TOML com chaves EN ou aliases PT legados) ou envelope JSON `vps-export`; use `--allow-incomplete` para hosts redacted/skeleton sem auth completa.
- `added_at` / `adicionado_em` são opcionais no import (serde usa o instante atual quando omitidos).
- Inventário wire usa schema v3: novas escritas serializam chaves em inglês (`name`, `port`, `username`, `password`, `added_at`, …); a leitura ainda aceita aliases legados em português (`nome`, `porta`, `usuario`, `senha`, `adicionado_em`).
- Re-cifre inventário plaintext após upgrade: `ssh-cli secrets reencrypt`.
- Espere JSON automático quando stdout não é TTY, salvo `--output-format` (exceto `vps export`, que permanece TOML sem `--json`).
- Espere senha vazia em hosts só-chave como JSON `null` (não `"***"`); senhas não vazias mascaram como `***`; texto humano em show usa "(não definida)" para vazio.
- Em falha de `scp --json` / `sftp --json`, parseie o envelope de erro JSON em **stderr** (`exit_code`, `message`), não prosa humana.
- Valores de timeout abaixo de 1000 ms avisam em stderr (milissegundos, não segundos); valores de senha em argv também avisam — prefira `--*-stdin`.


## Flags globais relevantes
- `--lang` — override one-shot de idioma de UI
- `-v` / `-vv` / `-vvv` — verbosidade graduada (info/debug/trace; escopo na crate; G2/G14)
- `-q` — silencia sucesso humano
- `--config-dir` — isola config XDG (testes / labs paralelos)
- `--no-color` — desliga cores ANSI
- `--output-format` / `--json` — força JSON de máquina
- `--disable-sudo` — bloqueia elevação nesta invocação
- `--replace-host-key` — troca de host-key TOFU após revisão humana
- `--allow-plaintext-secrets` / `--secrets-key-file` / `--use-keyring` — controle de secrets (só CLI/XDG)
- `--timeout` — override de connect/transfer (ms)
- `--max-concurrency` — clamp de fan-out de frota 1..=64
- `--fail-fast` — aborta o restante multi-host após a primeira falha
- `--scp-file-concurrency` — limite de concorrência multi-arquivo


## Configuração
### Inventário multi-host XDG
- Resolva o path de config com `ssh-cli vps path`.
- Espere gravações atômicas em `config.toml` mode 0600 (tempfile + fsync + flock).
- Espere arquivos irmãos `active`, `known_hosts` e `secrets.key` ao lado do config.
- Sobrescreva o diretório só em testes com `--config-dir`.
- Armazene timeout, max_command_chars, max_output_chars, segredos sudo e su por host.
- Cifragem at-rest por padrão (ChaCha20-Poly1305): segredos viram blobs `sshcli-enc:v1:…`.
- Controle de primary-key é só CLI/XDG: `--secrets-key-file`, `--use-keyring`, ou XDG `secrets.key`. O keyring ainda pode aceitar o alias legado `secrets-master-key` na leitura.
- `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` são **rejeitadas fail-closed** (não são store).
- Opt-out de plaintext só para testes: `--allow-plaintext-secrets` (sem store em env).
- `vps doctor --json` reporta paths, schema, contagem de hosts, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file` e `secrets_plaintext_opt_out` (booleano JSON).
- Sem store runtime de produto em `.env`.


## Subcomandos não cobertos acima
- `health-check [--timeout <ms>]` sonda conectividade e imprime latência (`vps add --check` após cadastro); sobrescreva o timeout quando o padrão do host for longo ou curto demais.
- Paridade auth em `health-check` (0.4.1+ / CLI-006): `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`.
- Nível de tracing padrão é error; use `-v`/`-vv`/`-vvv` ao diagnosticar (`RUST_LOG` ambiente é ignorado).
- `tunnel` exige porta local, host remoto, porta remota e `--timeout-ms`.
- Tunnel `--bind` tem padrão `127.0.0.1` (loopback); sobrescreva só quando pretender expor o listener de propósito.
- Opcional: `tunnel --json` emite `event: "tunnel_listening"` estruturado no stdout após o bind local (`docs/schemas/tunnel-listening.schema.json`); após o agente receber o evento, o deadline pós-bind sai com exit 0 (TUN-002); timeout pré-bind permanece 74.
- Paridade auth em `tunnel` (CLI-005): `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`.
- `completions` grava scripts de completion no stdout.
- `su-exec` exige senha `su` configurada no registro do host.
- `secrets` gerencia a primary-key de cifragem sem nunca imprimi-la.


## Exit codes (sysexits)

| Code | Meaning |
|------|---------|
| 0 | Sucesso |
| 1 | Falha genérica de runtime (ex.: exit remoto não-zero com `remote_exit_code` no envelope JSON) |
| 2 | Uso clap (flags inválidas) |
| 64 (`EX_USAGE`) | Argumento inválido / uso de domínio (inclui comando vazio, recusa de `--include-secrets` sem `-o` ou ack, validação ACME permanente ex. `invalidContact`) |
| 65 (`EX_DATAERR`) | Dados TOML/JSON de entrada inválidos (`TomlDe` / parse JSON / schema incompatível) |
| 66 (`EX_NOINPUT`) | VPS não encontrada, sem VPS ativa, ou arquivo ausente (`file not found: <path>` no SCP) |
| 73 (`EX_CANTCREAT`) | Falha de escrita / criação de config |
| 74 (`EX_IOERR`) | Conexão/IO/timeout |
| 77 (`EX_NOPERM`) | Falha de autenticação / política de host-key / permissão / sudo desabilitado |
| 130 | SIGINT |
| 143 | SIGTERM |

Linha de produto: 0.5.3.


## Integração com agentes de IA
- Carregue o pacote de skill em `skills/ssh-cli-pt/`.
- Prefira saída JSON para parsing de tools.
- Siga roteamento de exit codes antes de retries (veja README ou [AGENTS.pt-BR.md](AGENTS.pt-BR.md)).
- Leia [AGENTS.pt-BR.md](AGENTS.pt-BR.md) e [../INTEGRATIONS.pt-BR.md](../INTEGRATIONS.pt-BR.md).
- Formas de eventos e payloads: [docs/schemas/README.md](schemas/README.md).
- Nunca logue primary-key, senhas de host ou segredos decifrados.
