# Cookbook

> Copie receitas executáveis que resolvem problemas reais de SSH multi-host com agentes.

- Leia este documento em [inglês](COOKBOOK.md).
- Linha de produto: 0.5.2.


## Nota de latência
- Espere CRUD local em sub-segundo e cold connect SSH dominado pelo RTT de rede.
- Prefira comandos one-shot a tunnels quando uma única ação remota basta.


## Referência de valores padrão
- Porta padrão: 22
- Timeout padrão: 60000 ms
- max_command_chars padrão: 1000
- max_output_chars padrão: 100000
- Tracing padrão: error (`-v` → debug; `RUST_LOG` ambiente é ignorado)
- Senha vazia em list/show JSON: `null` (hosts só-chave); não vazia mascara como `***`
- Telemetria: desligada
- Segredos at-rest: cifrados por padrão (auto `secrets.key`)
- Instalação: `cargo install ssh-cli --locked`
- Supply chain: russh 0.62.2; `cargo deny` com `yanked=deny`, `multiple-versions=warn`
- SCP: somente arquivos regulares (sem `-r` / sem diretórios). Árvores e FS remoto usam **`sftp`** (`upload|download --recursive`, `ls`, `mkdir`, …). Sufixo partial de download `.ssh-cli.partial`; JSON exige `event: "scp-transfer"`
- Wire SCP: use 0.4.0+ (prefira a linha de produto 0.5.2); nunca 0.3.9 (crates.io 0.3.9 anunciava SCP mas era inoperante)
- Export redacted: corpo padrão é TOML (mesmo em pipes); secrets vazios como `""`; secrets não vazios redacted → `***` (`FIXED_MASK`, nunca `""` para não vazios); nunca blob `sshcli-enc:` no caminho redacted; JSON só com `vps export --json`
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


## Como descobrir contratos (schema / doctor)

```bash
ssh-cli schema
ssh-cli schema vps-list
ssh-cli doctor --json
```

- Root `schema` lista contratos de agente; `schema <name>` imprime um documento de schema.
- Root `doctor --json` (ou `vps doctor --json`) reporta paths, modo de secrets e runtime.


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

Prefira **um processo** com `--all` (SSH concorrente com bound) a N processos single-host. Limite o fan-out com `--max-concurrency` global (1..=64; fórmula auto quando omitido).

```bash
# sonda todos os hosts do inventário (JSON batch: health-check-batch)
ssh-cli --max-concurrency 8 health-check --all --json

# mesmo comando remoto em todos (exec-batch; também sudo-exec / su-exec --all)
ssh-cli exec --all 'uptime' --json
ssh-cli --max-concurrency 4 sudo-exec --all 'systemctl is-active nginx' --json

# copia um arquivo local para o mesmo path remoto em todos (scp-batch)
ssh-cli scp upload --all ./app.tgz /tmp/app.tgz --json

# download: path local é prefixo → grava ./app.log.<vps>
ssh-cli scp download --all /var/log/app.log ./app.log --json
```

- Schemas batch: `docs/schemas/health-check-batch.schema.json`, `exec-batch.schema.json`, `scp-batch.schema.json` (envelope inclui `max_concurrency`).
- Inventário vazio + `--all` → exit de uso **64** (`no hosts registered for --all`).
- Comandos single-host permanecem válidos quando o alvo é um nome.


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
# só ao diagnosticar:
# ssh-cli -v exec lab "true" --json
# ssh-cli -v exec lab "true" --json
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
# corpo padrão do export é TOML mesmo em pipe/non-TTY (não auto-JSON)
ssh-cli vps export -o /tmp/hosts.mascarado.toml
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
# override de bind opcional (só quando intencional):
# ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --bind 0.0.0.0
# auth opcional (paridade exec/scp, CLI-005):
printf '%s' "$PASS" | ssh-cli tunnel prod 18080 127.0.0.1 8080 \
  --timeout-ms 30000 --json --password-stdin
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000 --json \
  --key ~/.ssh/id_ed25519
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
# Use 0.4.0+ (prefira a linha de produto 0.5.2); nunca 0.3.9 — o wire SCP daquela release estava quebrado
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
- Matriz oficial **E01–E16**; **E10–E14** = SCP upload, download, integridade (`cmp`), remoto ausente, preserve mode+mtime.
- O script imprime só rótulos PASS/FAIL/SKIP — nunca host, user ou password.
- Prefira `sshd` local / VPS throwaway; nunca tempestade de auth falha em produção (fail2ban).
