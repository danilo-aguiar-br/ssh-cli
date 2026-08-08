# Cookbook

> **0.5.4** — release de segurança e agent-native. Corrige DoS remoto pré-auth no banner SSH (A1), impede que bits setuid enviados pelo servidor caiam no arquivo baixado (A3), fecha a janela de leitura pública em chaves privadas ACME/mTLS (A2) e adiciona flags de redução de payload (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) aplicadas antes da serialização. BREAKING: falha parcial multi-host agora sai com exit **1** (era 65); `--bind` fora do loopback exige `--i-accept-network-exposure`. Novo evento `tunnel_closed`.


> Copie receitas executáveis que resolvem problemas reais de SSH multi-host com agentes.

- Leia este documento em [inglês](COOKBOOK.md).
- Linha de produto: 0.5.4.


## Nota de latência
- Espere CRUD local em sub-segundo e cold connect SSH dominado pelo RTT de rede.
- Prefira comandos one-shot a tunnels quando uma única ação remota basta.


## Referência de valores padrão
- Porta padrão: 22
- Timeout padrão: 60000 ms
- max_command_chars padrão: 1000
- max_output_chars padrão: 100000
- Tracing padrão: error (`-v` → info, `-vv` → debug, `-vvv` → trace; escopo na crate; `RUST_LOG` ambiente é ignorado)
- Senha vazia em list/show JSON: `null` (hosts só-chave); não vazia mascara como `***`
- Telemetria: desligada
- Segredos at-rest: cifrados por padrão (auto `secrets.key`)
- Instalação: `cargo install ssh-cli --locked`
- Supply chain: russh 0.62.5; `cargo deny` com `yanked=deny`, `multiple-versions=deny`
- SCP: somente arquivos regulares (sem `-r` / sem diretórios). Árvores e FS remoto usam **`sftp`** (`upload|download --recursive`, `ls`, `mkdir`, …). Sufixo partial de download `.ssh-cli.partial`; JSON exige `event: "scp-transfer"`
- Wire SCP: use 0.4.0+ (prefira a linha de produto 0.5.4); nunca 0.3.9 (crates.io 0.3.9 anunciava SCP mas era inoperante)
- SFTP: prefira **0.5.3+** (correção de integridade de upload G1); verifique o destino com `sha256sum`; árvores via `--recursive`
- Export redacted: o corpo segue o formato resolvido, então é JSON em qualquer stdout non-TTY e TOML somente com `--output-format text`; secrets vazios como `""`; secrets não vazios redacted → `***` (`FIXED_MASK`, nunca `""` para não vazios); nunca blob `sshcli-enc:` no caminho redacted; JSON só com `vps export --json`
- Wire de hosts: schema v3 (serialização em inglês; dual-read de aliases legados em português)
- Tunnel pós-bind: deadline one-shot sai com exit 0 após `tunnel_listening` (TUN-002); timeout pré-bind permanece 74
- Tunnel `--bind` padrão: `127.0.0.1`
- Auth tunnel/health: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` (0.4.1+)
- Flags de secrets (só CLI/XDG; stores env de secrets rejeitados fail-closed): `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- Validação ACME permanente (ex.: `invalidContact`) → exit **64** (não faça retry como 74)
- Timeout abaixo de 1000 ms: aviso em stderr (unidade é milissegundos, não segundos)
- Senha em argv: aviso em stderr; prefira `--*-stdin`
- CRUD/connect/import com `--json`: eventos `vps-added` / `vps-edited` / `vps-removed` / `vps-connected` / `vps-import`
- A primeira gravação de segredo pode definir `secrets_key_auto_created: true` no mesmo documento `vps-added` quando a primary-key é provisionada
- Notas de produto (0.5.3 / G1–G19 voltadas ao usuário):
  - G1: integridade de upload SFTP (prefira 0.5.3+; verifique o destino com `sha256sum`)
  - G2/G14: `-v`/`-vv`/`-vvv` graduado com escopo na crate; `RUST_LOG` ambiente ignorado
  - G3: SETSTAT SFTP envia `atime`+`mtime` juntos
  - G4: `set_metadata` mutante SFTP é fail-closed
  - G5/G17: cancelamento multi-arquivo / batch mantém `results.len() == input.len()` (resto cancelled preenchido)
  - G8: `exec --json` de host único emite exatamente um objeto JSON
  - G9: download SCP propaga falha de `sync_data` antes do rename
  - G12/G19: bits de permissão mascarados por direção — `SFTP_PERM_MASK` (`0o7777`) no upload, `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) no download (A3)
  - G15: aceite prova de efeito no destino (checksum), não só contagem de bytes do cliente
  - G18: falhas de `set_permissions` local no download SFTP são sinalizadas
  - G7: matriz E2E real-SSH inclui checksum SFTP + árvore recursiva (**E17/E18**)
  - G6/G11: só isolamento de testes (veja E2E / TESTING) — não é contrato de CLI em runtime


## Como inicializar cifragem com primary-key

```bash
ssh-cli secrets init
ssh-cli secrets status --json
# nunca imprime o material da chave
# envelopes de agente:
ssh-cli secrets init --json
# → event: "secrets-init" (docs/schemas/secrets-init.schema.json)
ssh-cli secrets reencrypt --json
# → event: "secrets-reencrypt" (docs/schemas/secrets-reencrypt.schema.json)
# a primeira gravação de segredo pode auto-criar secrets.key e emitir:
# → event: "vps-added" com secrets_key_auto_created: true (um documento JSON)
# só CLI/XDG (stores env de secrets rejeitados fail-closed):
# ssh-cli --secrets-key-file /path/to/key secrets status --json
# ssh-cli --use-keyring secrets init --json
# ssh-cli --allow-plaintext-secrets vps add ...   # só testes
```


## Como descobrir contratos (schema / doctor / commands)

```bash
ssh-cli schema
ssh-cli schema vps-list
ssh-cli doctor --json
ssh-cli commands --json
```

- Root `schema` lista o catálogo de schemas de agente; `schema <name>` imprime um documento de schema.
- Root `doctor --json` (ou `vps doctor --json`) reporta paths, modo de secrets e runtime.
- Root `commands --json` emite a árvore completa de comandos CLI (`event: "commands"`) para descoberta por agentes.


## Como cadastrar host com senha (stdin, sem vazar em argv)

```bash
# prefira --password-stdin; senha em argv também funciona, mas avisa em stderr
printf '%s' 'demo-password-not-real' | ssh-cli vps add \
  --name prod \
  --host prod.example.com \
  --user deploy \
  --password-stdin
# com --json → um documento event: "vps-added" (secrets_key_auto_created true/false)
# alternativa agent auth:
# ssh-cli vps add --name lab --host 203.0.113.10 --user ubuntu --use-agent
# descoberta: ssh-cli schema | ssh-cli doctor --json
```


## Como cadastrar host só com chave

```bash
ssh-cli vps add --name edge --host edge.example.com --user ubuntu --key ~/.ssh/id_ed25519
# ssh-cli vps add ... --json → event: "vps-added"
# ssh-cli vps edit edge --user ubuntu --json → event: "vps-edited"
# ssh-cli vps remove edge --json → event: "vps-removed"
# ssh-cli vps connect edge --json → event: "vps-connected"
```


## Como rodar comando remoto com JSON

```bash
ssh-cli exec prod "hostname && uptime" --json
```


## Como rodar sudo seguro com comandos compostos

```bash
# packing usa `sh -c` seguro; metacaracteres ficam dentro do shell remoto
ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --description "bootstrap curl"
```


## Como elevar com su quando sudo não está disponível

```bash
printf '%s' 'root-secret' | ssh-cli vps edit prod --su-password-stdin
ssh-cli su-exec prod "whoami"
```


## Como rejeitar cedo comandos grandes de agente

```bash
ssh-cli vps edit prod --max-command-chars 1000
# comando longo é rejeitado antes do SSH quando passa do limite (max_command_chars)
```


## Como limitar saída para contexto de LLM

```bash
ssh-cli vps edit prod --max-output-chars 20000
ssh-cli exec prod "dmesg" --json
```


## Como sondar conectividade após o add

```bash
ssh-cli vps add --name lab --host lab.example.com --user lab --key ~/.ssh/id_ed25519 --check
ssh-cli health-check lab --json
# overrides opcionais de auth (paridade com exec/scp desde 0.4.1+):
# printf '%s' "$PASS" | ssh-cli health-check lab --json --password-stdin
# ssh-cli health-check lab --json --key ~/.ssh/id_ed25519
```


## Como rodar trabalho de frota em todos os hosts registrados

Prefira **um processo** com `--all` ou um subconjunto via `--hosts a,b` (SSH concorrente com bound) a N processos single-host. Limite o fan-out com `--max-concurrency` global (1..=64; fórmula auto quando omitido).

```bash
# sonda todos os hosts do inventário (JSON batch: health-check-batch)
ssh-cli --max-concurrency 8 health-check --all --json
# subconjunto do inventário (ainda JSON batch)
ssh-cli health-check --hosts web1,web2 --json

# mesmo comando remoto em todos ou em um subconjunto (exec-batch; também sudo-exec / su-exec)
ssh-cli exec --all 'uptime' --json
ssh-cli exec --hosts web1,web2 'uptime' --json
ssh-cli --max-concurrency 4 sudo-exec --all 'systemctl is-active nginx' --json

# por tag, sem enumerar nomes — só na família exec (exec / sudo-exec / su-exec).
# As tags vêm de `vps add --tag prod --tag edge`; casa todo host que carregue qualquer tag listada.
ssh-cli exec --tags prod,edge 'uptime' --json
ssh-cli sudo-exec --tags prod 'systemctl restart nginx' --json
# --all, --hosts e --tags são mutuamente exclusivos: o clap rejeita qualquer par com exit 2.
# scp, sftp e health-check aceitam --all e --hosts, mas NÃO --tags.

# copia um arquivo local para o mesmo path remoto em todos (scp-batch)
ssh-cli scp upload --all ./app.tgz /tmp/app.tgz --json
ssh-cli scp upload --hosts web1,web2 ./app.tgz /tmp/app.tgz --json

# download: path local é prefixo → grava ./app.log.<vps>
ssh-cli scp download --all /var/log/app.log ./app.log --json

# doctor local + sonda SSH de frota opcional
ssh-cli vps doctor --probe-ssh --json
```

- Schemas batch: `docs/schemas/health-check-batch.schema.json`, `exec-batch.schema.json`, `scp-batch.schema.json` (envelope inclui `max_concurrency`).
- Limite o fan-out com `--max-concurrency` global (1..=64; fórmula auto quando omitido).
- Cancelamento multi-arquivo / batch mantém `results.len() == input.len()` com o resto cancelled preenchido (G5/G17).
- Inventário vazio + `--all` / `--hosts` → exit de uso **64**.
- Nome desconhecido em `--hosts` → exit de uso **64** (`unknown host(s) for --hosts`).
- Comandos single-host permanecem válidos quando o alvo é um nome posicional (JSON clássico).
- `tunnel` é single-host por contrato (um bind + uma sessão); multi-host = N invocações.


## Como sondar com timeout customizado

```bash
# --timeout é em milissegundos (não segundos); valores abaixo de 1000 avisam em stderr
# sobrescreva o timeout do host quando o padrão for longo ou curto demais para uma sonda rápida
ssh-cli health-check lab --timeout 15000 --json
# opcional: combine timeout com key ou password-stdin
# ssh-cli health-check lab --timeout 15000 --json --key ~/.ssh/id_ed25519
# evite sondas sub-segundo acidentais salvo intenção:
# ssh-cli health-check lab --timeout 500 --json   # funciona, mas avisa em stderr (<1000 ms)
```


## Como manter stderr do agente limpo

```bash
# tracing padrão é error: stderr de JSON/tunnel fica sem prosa INFO
ssh-cli exec lab "true" --json
# só ao diagnosticar (escopo na crate; sem vazamento de senha via russh — G2/G14):
# ssh-cli -v exec lab "true" --json    # info
# ssh-cli -vv exec lab "true" --json   # debug
# ssh-cli -vvv exec lab "true" --json  # trace
# RUST_LOG ambiente é ignorado
```


## Como diagnosticar paths XDG e modo de segredos

```bash
ssh-cli vps doctor --json
# espere secrets_at_rest, secrets_key_source, secrets_key_file, telemetry=false
ssh-cli vps path
ssh-cli secrets status --json
```


## Como re-cifrar inventário plaintext legado

```bash
ssh-cli secrets init
ssh-cli secrets reencrypt
# senhas em config.toml viram blobs sshcli-enc:v1:…
```


## Como exportar e importar inventário sem segredos

```bash
# o corpo segue o formato resolvido: JSON em non-TTY, TOML somente com --output-format text
ssh-cli vps export -o /tmp/hosts.mascarado.json
ssh-cli --output-format text vps export -o /tmp/hosts.mascarado.toml
# secrets vazios serializam como "" — nunca ciphertext sshcli-enc: (EXP-001)
# secrets não vazios redacted → "***" (FIXED_MASK; nunca "" para não vazios; G-E2E-10)
# envelope de agente só com --json → event: "vps-export"
ssh-cli vps export --json -o /tmp/hosts.mascarado.json
# import aceita TOML (chaves EN ou aliases PT legados) ou JSON vps-export
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.mascarado.toml
# hosts redacted/skeleton sem auth completa:
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.mascarado.toml \
  --allow-incomplete
```


## Como exportar com segredos (protegido)

```bash
# --include-secrets exige -o/--output (mode 0o600) ou ack explícito de stdout
ssh-cli vps export --include-secrets -o /tmp/hosts.secrets.toml
# pipe sem ack é recusado (exit 64):
# ssh-cli vps export --include-secrets | cat   # falha
# só se realmente precisar de stdout:
# ssh-cli vps export --include-secrets --i-understand-secrets-on-stdout
```


## Como abrir tunnel limitado

```bash
# --bind tem padrão 127.0.0.1 (loopback)
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000
# agentes: aguarde tunnel_listening antes de usar a porta local
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json
# stdout: {"ok":true,"event":"tunnel_listening","vps":"prod","local_port":18080,...}
# schema: docs/schemas/tunnel-listening.schema.json
# após tunnel_listening, deadline one-shot pós-bind sai com exit 0 (TUN-002); timeout pré-bind permanece 74
# override de bind opcional — bind roteável EXIGE o reconhecimento (G-TUN-R13):
# ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 \
#   --bind 0.0.0.0 --i-accept-network-exposure
# sem --i-accept-network-exposure o bind roteável é recusado, não publicado em silêncio
# tunnel --json também emite tunnel_closed no encerramento, com reason/forwards_served/capacity_waits
# auth opcional (paridade exec/scp, CLI-005):
printf '%s' "$PASS" | ssh-cli tunnel prod 18080 127.0.0.1 8080 \
  --timeout-ms 30000 --json --password-stdin
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json \
  --key ~/.ssh/id_ed25519
```


## Como alcançar muitos destinos com um só handshake (`--socks5`)

```bash
# G-TUN-R02: uma sessão SSH serve todo destino, escolhido por conexão
ssh-cli tunnel prod 1080 --socks5 --timeout-ms 300000 --json
# stdout: {"ok":true,"event":"tunnel_listening","mode":"socks5","local_port":1080,...}
# REMOTE_HOST / REMOTE_PORT são omitidos: o CONNECT do SOCKS5 carrega o alvo
# depois aponte qualquer cliente com suporte a SOCKS5 para 127.0.0.1:1080
curl --socks5-hostname 127.0.0.1:1080 http://10.0.0.5:8080/health
curl --socks5-hostname 127.0.0.1:1080 http://10.0.0.9:9200/_cluster/health
# prefira isso a N processos `ssh-cli tunnel`: o handshake é pago uma vez, não N
```


## Como encaminhar para socket Unix remoto (`--remote-socket`)

```bash
# G-TUN-R03: direct-streamlocal@openssh.com alcança alvos que nunca escutam em TCP
ssh-cli tunnel prod 2375 --remote-socket /var/run/docker.sock --timeout-ms 60000 --json
# stdout: {"ok":true,"event":"tunnel_listening","mode":"streamlocal",...}
DOCKER_HOST=tcp://127.0.0.1:2375 docker ps
# socket peer do PostgreSQL, mesma forma:
ssh-cli tunnel db 15432 --remote-socket /var/run/postgresql/.s.PGSQL.5432 \
  --timeout-ms 60000 --json
# o caminho deve ser ABSOLUTO — caminho relativo falha com exit 64 antes de qualquer SSH
# a existência local nunca é checada: o socket vive no filesystem do servidor
# trate isso como delegação de privilégio — a porta local herda a autoridade do socket
```


## Como deixar o servidor alcançar de volta (`--reverse`)

```bash
# G-TUN-R01: o servidor escuta e entrega conexões à sua porta local
# casos de uso: webhook de callback, debugger remoto apontando para IDE local, bastion invertido
ssh-cli tunnel prod 8080 127.0.0.1 9000 --reverse --timeout-ms 120000 --json
# stdout: {"ok":true,"event":"tunnel_listening","mode":"reverse",...}
# REMOTE_PORT 0 pede ao servidor que aloque e informe a porta que ligou:
ssh-cli tunnel prod 8080 127.0.0.1 0 --reverse --timeout-ms 120000 --json
# um forward local não aceita 0 — não haveria nada a que se conectar
# expor o bind do SERVIDOR exige o reconhecimento, já que é essa a ponta exposta aqui:
ssh-cli tunnel prod 8080 0.0.0.0 9000 --reverse \
  --i-accept-network-exposure --timeout-ms 120000 --json
# AllowTcpForwarding / GatewayPorts do servidor ainda governam se o bind é permitido
```


## Como encolher o payload do agente antes de ele ser escrito

```bash
# as oito flags de redução agem ANTES da serialização: o envelope grande nunca é construído
ssh-cli health-check --all --json --select name,ok --filter ok=false
# só as falhas, dois campos cada
ssh-cli vps list --json --select name,host --sort name --limit 10
ssh-cli exec --all 'uptime' --json --truncate-content 200 --max-output-bytes 65536
# quantos hosts estão inalcançáveis, sem transferir registro de host algum:
ssh-cli health-check --all --json --filter ok=false --count-only
# stdout: {"count":2}
# predicado malformado é rejeitado no parse, então typo nunca parece "nenhum resultado"
ssh-cli vps list --json --filter 'nameprod'     # exit 64: nenhum operador
ssh-cli vps list --json --filter '=prod'        # exit 64: chave vazia
# note o que É válido: `name~~prod` parseia como chave `name`, substring `~prod` — exit 0, `[]`.
# A recusa cobre operador ausente, não toda string de aparência estranha.
```


## Como ensaiar uma operação destrutiva (`--dry-run`)

```bash
# aceita SOMENTE por estes seis; em qualquer outro lugar --dry-run é recusada com exit 64
ssh-cli vps remove old-host --dry-run --json
ssh-cli vps import --file hosts.toml --dry-run --json
ssh-cli sftp rm prod /srv/app/stale.log --dry-run --json
ssh-cli sftp rmdir prod /srv/app/empty-dir --dry-run --json
ssh-cli secrets init --dry-run --json
ssh-cli secrets reencrypt --dry-run --json
# a recusa é o ponto: flag que não faz nada em silêncio em metade da superfície
# é pior que flag nenhuma, porque o operador acredita que o ensaio aconteceu
ssh-cli exec prod 'rm -rf /tmp/x' --dry-run --json   # exit 64, não um no-op silencioso
# execução sem operador também deve recusar esperar por um humano que não está lá:
ssh-cli vps list --json --no-input
```


## Como fazer health-check com auth agent-safe

```bash
ssh-cli health-check prod --json
ssh-cli health-check prod --timeout 5000 --json
# paridade auth 0.4.1+ (CLI-006):
printf '%s' "$PASS" | ssh-cli health-check prod --json --password-stdin
ssh-cli health-check prod --json --key ~/.ssh/id_ed25519
printf '%s' "$KEY_PASS" | ssh-cli health-check prod --json \
  --key ~/.ssh/id_ed25519_enc --key-passphrase-stdin
```


## Como transferir artefato de release (somente arquivo regular)

```bash
# Use 0.4.0+ (prefira a linha de produto 0.5.4); nunca 0.3.9 — o wire SCP daquela release estava quebrado
# SCP: sem diretórios / sem -r (use `sftp --recursive` para árvores)
ssh-cli scp upload prod ./dist/app.tar.gz /opt/app/app.tar.gz \
  --timeout 120000 --json
# sucesso em stdout → docs/schemas/scp-transfer.schema.json
# inclui event: "scp-transfer" obrigatório (IO-009)
# falhas com --json → envelope de erro em stderr
ssh-cli exec prod "tar -tzf /opt/app/app.tar.gz | head"
```


## Como baixar arquivo remoto com segurança

```bash
ssh-cli scp download prod /var/log/app.log ./app.log --json
# em falha o path final fica intacto; intermediário é ./app.log.ssh-cli.partial
# mtime/mode preservados nos dois sentidos (remoto scp -tp/-fp)
# falha de sync_data é propagada antes do rename (G9)
```


## Como fazer upload SFTP e verificar checksum (prefira 0.5.3+)

```bash
# G1 corrigiu truncamento de upload SFTP em 0.5.3 — sempre prove o efeito no destino
ssh-cli sftp upload prod ./payload.bin /tmp/payload.bin --json
ssh-cli exec prod "sha256sum /tmp/payload.bin" --json
sha256sum ./payload.bin
# download (partial + rename atômico; G18 sinaliza falhas de set_permissions local):
ssh-cli sftp download prod /tmp/payload.bin ./payload.remote.bin --json
# árvore recursiva (sem seguir symlink):
ssh-cli sftp upload --recursive prod ./dist/tree /opt/app/tree --json
# frota multi-host:
ssh-cli sftp upload --all ./payload.bin /tmp/payload.bin --json
```


## Como gerenciar paths remotos via SFTP

```bash
ssh-cli sftp ls prod /tmp --json
ssh-cli sftp mkdir prod /tmp/app --json
ssh-cli sftp stat prod /tmp/app --json
ssh-cli sftp rename prod /tmp/app /tmp/app.bak --json
ssh-cli sftp rm prod /tmp/file.bin --json
ssh-cli sftp rmdir prod /tmp/empty --json
# schemas: sftp-list / sftp-fs-op
```

- Prefira a linha de produto **0.5.3+** para todo trabalho SFTP (integridade de upload G1).
- SETSTAT envia `atime`+`mtime` juntos (G3); metadata mutante é fail-closed (G4).
- Bits de permissão são mascarados por direção: `SFTP_PERM_MASK` (`0o7777`; G12/G19) no upload, `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) no download, então setuid/setgid/sticky enviados pelo servidor nunca chegam ao arquivo local (A3).
- Falhas de `set_permissions` local no download são sinalizadas (G18).


## Como gerar completions de shell

```bash
ssh-cli completions bash
ssh-cli completions zsh
ssh-cli completions fish
ssh-cli completions powershell
ssh-cli completions elvish
```


## Como diagnosticar com -vv sem vazamento de senha

```bash
# filtros com escopo na crate (warn,ssh_cli=…) — nunca debug global nu (G2)
ssh-cli -vv exec prod "true" --json
# NÃO defina RUST_LOG; RUST_LOG ambiente é ignorado
```


## Como definir locale de UI

```bash
ssh-cli locale show
ssh-cli locale set pt-BR
ssh-cli --lang en vps list
ssh-cli locale clear
```


## Como inspecionar provider e paths TLS

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

- Stack TLS é só **rustls** + **aws_lc_rs** (sem runtime OpenSSL).
- Material fica sob XDG `tls/` (veja `ssh-cli tls paths`).
- Falhas permanentes de validação ACME (ex.: `invalidContact`) → exit **64** (não faça retry como 74).


## Como fazer CRUD completo de VPS

```bash
ssh-cli vps add --name web1 --host 203.0.113.10 --user deploy --key ~/.ssh/id_ed25519 --json
ssh-cli vps list --json
ssh-cli vps show web1 --json
ssh-cli vps edit web1 --timeout 90000 --json
ssh-cli connect web1
ssh-cli vps path
ssh-cli doctor --json
ssh-cli vps remove web1 --json
```


## Como tratar rotação de host key com segurança (TOFU)

```bash
# a primeira falha reporta mismatch; só após revisão humana:
ssh-cli --replace-host-key exec prod "true"
```


## Como desabilitar elevação em automação não confiável

```bash
ssh-cli --disable-sudo exec prod "id"
# sudo-exec/su-exec permanecem bloqueados nesta invocação
```


## Como rodar E2E SSH real sem logar segredos

```bash
# Preferido (XDG / --config-dir primeiro): config-dir isolado com hosts já cadastrados
ssh-cli --config-dir /tmp/ssh-cli-e2e-lab vps add --name e2e --host … --user … --password-stdin
bash scripts/e2e_real_ssh.sh --config-dir /tmp/ssh-cli-e2e-lab

# Env só do harness (NÃO é store de runtime do produto) — nunca commite esses valores
# export SSH_CLI_E2E_HOST=… SSH_CLI_E2E_USER=… SSH_CLI_E2E_PASSWORD=…
# bash scripts/e2e_real_ssh.sh

# Só mantenedor local: parse $HOME/.grok/config.toml ($HOME only; nunca copie para o repositório)
# bash scripts/e2e_real_ssh.sh --from-grok-config
```

- Binário padrão: `target/release/ssh-cli` (override só com harness `SSH_CLI_E2E_BIN`).
- Sem host de lab / credenciais, o script sai **0** com **SKIP** (seguro offline; não trate SKIP como gate vermelho).
- Matriz oficial **E01–E18**; **E10–E14** = SCP upload, download, integridade (`cmp`), remoto ausente, preserve mode+mtime; **E17/E18** = SFTP checksum + árvore recursiva (G7).
- G6/G11 são só isolamento de testes (testes de sinal serializados / `reset_flags_for_tests`) — não são superfície de CLI do produto.
- O script imprime só rótulos PASS/FAIL/SKIP — nunca host, user ou password.
- Prefira `sshd` local / VPS throwaway; nunca tempestade de auth falha em produção (fail2ban).
