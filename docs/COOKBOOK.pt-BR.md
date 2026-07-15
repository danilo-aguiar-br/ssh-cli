# Cookbook

> Copie receitas executáveis que resolvem problemas reais de SSH multi-host com agentes.

- Leia este documento em [inglês](COOKBOOK.md).
- Linha de produto: **0.3.9**.


## Nota de latência
- Espere CRUD local sub-segundo e connect SSH dominado pelo RTT de rede.
- Prefira comandos one-shot a tunnels quando uma única ação remota basta.


## Referência de defaults
- Porta padrão: 22
- Timeout padrão: 60000 ms
- max_command_chars padrão: 1000
- max_output_chars padrão: 100000
- Tracing padrão: error (`-v` → debug; `RUST_LOG` sobrescreve)
- Senha vazia em list/show JSON: `null` (hosts só-chave); não vazia mascara como `***`
- Telemetria: desligada
- Segredos at-rest: cifrados por padrão (auto `secrets.key`)
- Install: `cargo install ssh-cli --locked`
- Supply chain: russh 0.62.2; `cargo deny` com `yanked=deny`, `multiple-versions=warn`


## Como inicializar cifragem com master-key

```bash
ssh-cli secrets init
ssh-cli secrets status --json
# nunca imprime o material da chave
```


## Como cadastrar host com senha (stdin, sem leak em argv)

```bash
printf '%s' 's3cret' | ssh-cli vps add \
  --name prod \
  --host prod.example.com \
  --user deploy \
  --password-stdin
```


## Como cadastrar host só com chave

```bash
ssh-cli vps add --name edge --host edge.example.com --user ubuntu --key ~/.ssh/id_ed25519
```


## Como rodar comando remoto com JSON

```bash
ssh-cli exec prod "hostname && uptime" --json
```


## Como rodar sudo seguro com comandos compostos

```bash
# packing usa `sh -c` seguro; metacaracteres ficam no shell remoto
ssh-cli sudo-exec prod "apt-get update && apt-get install -y curl" --description "bootstrap curl"
```


## Como elevar com su quando sudo não está disponível

```bash
printf '%s' 'root-secret' | ssh-cli vps edit prod --su-password-stdin
ssh-cli su-exec prod "whoami"
```


## Como rejeitar cedo comandos grandes do agente

```bash
ssh-cli vps edit prod --max-command-chars 1000
# comando longo é rejeitado antes do SSH se exceder o limite
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
```


## Como sondar com timeout customizado

```bash
# sobrescreva o timeout do host quando o padrão for longo ou curto demais para um probe
ssh-cli health-check lab --timeout 15000 --json
```


## Como manter o stderr do agente limpo

```bash
# tracing padrão é error: stderr de JSON/tunnel fica sem prosa INFO
ssh-cli exec lab "true" --json
# só ao diagnosticar:
# RUST_LOG=debug ssh-cli exec lab "true" --json
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
# senhas no config.toml viram blobs sshcli-enc:v1:…
```


## Como exportar e importar inventário sem segredos

```bash
ssh-cli vps export -o /tmp/hosts.mascarado.toml
ssh-cli --config-dir /tmp/ssh-cli-copy vps import --file /tmp/hosts.mascarado.toml
```


## Como abrir tunnel limitado

```bash
ssh-cli tunnel prod 18080 127.0.0.1 8080 --timeout-ms 30000
```


## Como transferir artefato de release

```bash
ssh-cli scp upload prod ./dist/app.tar.gz /opt/app/app.tar.gz
ssh-cli exec prod "tar -tzf /opt/app/app.tar.gz | head"
```


## Como tratar rotação de host key com segurança (TOFU)

```bash
# a primeira falha reporta divergência; só após revisão humana:
ssh-cli --replace-host-key exec prod "true"
```


## Como desabilitar elevação em automação não confiável

```bash
ssh-cli --disable-sudo exec prod "id"
# sudo-exec/su-exec permanecem bloqueados nesta invocação
```


## Como rodar E2E SSH real sem logar segredos

```bash
# usa env estilo daemon ou --from-grok-config; imprime só PASS/FAIL
bash scripts/e2e_real_ssh.sh --from-grok-config
```
