# Como usar ssh-cli

> Vá da instalação ao primeiro comando remoto em menos de 60 segundos.

- Leia este documento em [inglês](HOW_TO_USE.md).
- Volte ao [README.pt-BR.md](../README.pt-BR.md) para o mapa completo de comandos.
- Linha de produto documentada aqui: 0.5.1.


## Pré-requisitos
- Instale Rust MSRV 1.85.0 ou superior via rustup.
- Garanta conectividade de rede até o host SSH alvo.
- Tenha senha ou chave privada OpenSSH para esse host.
- Prefira um XDG config home gravável para storage multi-host.
- Instale com `cargo install ssh-cli --locked` (0.5.1+ no crates.io; evite 0.3.9 para SCP).
- Não dependa do crates.io 0.3.9 para SCP: aquela release anunciava transferência, mas o protocolo wire estava quebrado (arquivos remotos de 0 bytes ou timeouts). Use 0.5.1+.


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
- Um comando remoto vazio falha com a mensagem técnica `empty command` (sempre em inglês) e exit de uso de domínio 64.
- Rode `ssh-cli secrets status --json` e `ssh-cli vps doctor --json` quando path ou cifragem estiverem incertos.
- Prefira `--password-stdin` a `--password` ao cadastrar hosts com senha.


## Comandos centrais
### Loop diário do operador
- Liste hosts com `ssh-cli vps list --json`.
- Mostre um host com `ssh-cli vps show demo --json` (segredos mascarados).
- Altere campos com `ssh-cli vps edit demo --timeout 90000`.
- Marque host ativo com `ssh-cli connect demo`.
- Rode trabalho privilegiado com `ssh-cli sudo-exec demo "systemctl status nginx" --json` (packing seguro `sh -c`).
- Eleve com `ssh-cli su-exec` quando a senha `su` estiver no registro do host.
- Transfira **somente arquivos regulares** (sem diretórios, sem `-r`, sem SFTP) com `ssh-cli scp upload demo ./app.tgz /tmp/app.tgz`.
- Baixe com `ssh-cli scp download demo /var/log/app.log ./app.log`.
- Prefira JSON de agente: `ssh-cli scp upload demo ./app.tgz /tmp/app.tgz --json` (schema `docs/schemas/scp-transfer.schema.json`; sucesso exige `event: "scp-transfer"`).
- Flags SCP com paridade ao exec: `--timeout` (connect + transfer), `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json`.
- Arquivo local/remoto ausente no SCP sai com exit 66 e mensagem `file not found: <path>` (path canônico/normalizado; sem prefixos `SCP:` empilhados).
- Download com falha não deixa o destino final corrompido: grava `{path}.ssh-cli.partial`, aplica mode/times no partial e faz rename atômico.
- Upload faz stream em blocos de 32 KiB (não carrega o arquivo inteiro na RAM).
- mtime/mode são preservados nos dois sentidos automaticamente (remoto `scp -tp` / `-fp`; sem flag extra do usuário).
- Gerencie a primary-key com `ssh-cli secrets status|init|reencrypt` (nunca imprime a chave). O keyring ainda pode aceitar o alias legado `secrets-master-key` na leitura.
- `secrets init --json` / `secrets reencrypt --json` emitem eventos de sucesso (`secrets-init`, `secrets-reencrypt`; schemas `docs/schemas/secrets-init.schema.json`, `docs/schemas/secrets-reencrypt.schema.json`); criação automática de chave pode emitir `secrets-key-auto-created`. Veja [docs/schemas/README.md](schemas/README.md).
- Eventos JSON de sucesso CRUD quando JSON está efetivo: `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import` (e `secrets-key-auto-created` quando uma chave é auto-criada). Catálogo: [docs/schemas/README.md](schemas/README.md).


## Daemon
### Não existe daemon
- Trate cada invocação como nascer-executar-morrer (one-shot).
- Nunca espere um worker SSH em background neste projeto.
- Limite tunnels com `--timeout-ms` obrigatório para o processo ainda encerrar.


## Padrões avançados
### Automação mais segura para agentes
- Alimente segredos por flags stdin (`--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--key-passphrase-stdin`) em vez de argv.
- Anexe comentários shell com `--description` para histórico remoto auditável.
- Desabilite elevação em tarefas não confiáveis com `--disable-sudo`.
- Substitua host key legítima só após confirmação humana com `--replace-host-key` (TOFU).
- Exporte inventário com segredos mascarados: `ssh-cli vps export -o hosts.toml` (corpo padrão é TOML, inclusive em pipe/non-TTY; secret vazio serializa como `""`, nunca blob `sshcli-enc:`; EXP-001). O texto de help reflete esse comportamento TOML por padrão.
- Export JSON de agente só com `ssh-cli vps export --json` → envelope `event: "vps-export"` (JSON auto em non-TTY **não** se aplica a `vps export`).
- `--include-secrets` exige `-o`/`--output` ou `--i-understand-secrets-on-stdout` (pipe/stdout sem ack é recusado, exit 64).
- Importe hosts com `ssh-cli vps import --file hosts.toml` (TOML com chaves EN ou aliases PT legados) ou envelope JSON `vps-export`; use `--allow-incomplete` para hosts redacted/skeleton sem auth completa.
- `added_at` / `adicionado_em` são opcionais no import (serde usa o instante atual quando omitidos).
- Inventário wire usa schema v3: novas escritas serializam chaves em inglês (`name`, `port`, `username`, `password`, `added_at`, …); a leitura ainda aceita aliases legados em português (`nome`, `porta`, `usuario`, `senha`, `adicionado_em`).
- Re-cifre inventário plaintext após upgrade: `ssh-cli secrets reencrypt`.
- Espere JSON automático quando stdout não é TTY, salvo `--output-format` (exceto `vps export`, que permanece TOML sem `--json`).
- Espere senha vazia em hosts só-chave como JSON `null` (não `"***"`); senhas não vazias mascaram como `***`; texto humano em show usa "(não definida)" para vazio.
- Em falha de `scp --json`, parseie o envelope de erro JSON em **stderr** (`exit_code`, `message`), não prosa humana.
- Valores de timeout abaixo de 1000 ms avisam em stderr (milissegundos, não segundos); valores de senha em argv também avisam — prefira `--*-stdin`.


## Configuração
### Inventário multi-host XDG
- Resolva o path de config com `ssh-cli vps path`.
- Espere gravações atômicas em `config.toml` mode 0600 (tempfile + fsync + flock).
- Espere arquivos irmãos `active`, `known_hosts` e `secrets.key` ao lado do config.
- Sobrescreva o diretório só em testes com `--config-dir` ou `SSH_CLI_HOME`.
- Armazene timeout, max_command_chars, max_output_chars, segredos sudo e su por host.
- Cifragem at-rest por padrão (ChaCha20-Poly1305): segredos viram blobs `sshcli-enc:v1:…`.
- Prefira flags CLI ao env para controle da primary-key: `--allow-plaintext-secrets`, `--secrets-key-file <PATH>`, `--use-keyring` (globais). O keyring ainda pode aceitar o alias legado `secrets-master-key` na leitura.
- A resolução da primary-key ainda aceita fallbacks de env (`SSH_CLI_SECRETS_KEY`, `SSH_CLI_SECRETS_KEY_FILE`, `SSH_CLI_USE_KEYRING=1`) e depois XDG `secrets.key`; flags são preferidas.
- Opt-out de plaintext só para testes: `--allow-plaintext-secrets` (ou `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` deprecado).
- `vps doctor --json` reporta paths, schema, contagem de hosts, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file` e `secrets_plaintext_opt_out` (booleano JSON).


## Subcomandos não cobertos acima
- `health-check [--timeout <ms>]` sonda conectividade e imprime latência (`vps add --check` após cadastro); sobrescreva o timeout quando o padrão do host for longo ou curto demais.
- Paridade auth em `health-check` (0.4.1+ / CLI-006): `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`.
- Nível de tracing padrão é error para manter stderr de JSON e tunnel limpos; use `RUST_LOG` ou `-v` (debug) ao diagnosticar.
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
| 64 (`EX_USAGE`) | Argumento inválido / uso de domínio (inclui comando vazio, recusa de `--include-secrets` sem `-o` ou ack) |
| 65 (`EX_DATAERR`) | Dados TOML/JSON de entrada inválidos (`TomlDe` / parse JSON / schema incompatível) |
| 66 (`EX_NOINPUT`) | VPS não encontrada, sem VPS ativa, ou arquivo ausente (`file not found: <path>` no SCP) |
| 73 (`EX_CANTCREAT`) | Falha de escrita / criação de config |
| 74 (`EX_IOERR`) | Conexão/IO/timeout |
| 77 (`EX_NOPERM`) | Falha de autenticação / política de host-key / permissão / sudo desabilitado |
| 130 | SIGINT |
| 143 | SIGTERM |

Linha de produto: 0.5.1.


## Integração com agentes de IA
- Carregue o pacote de skill em `skills/ssh-cli-pt/`.
- Prefira saída JSON para parsing de tools.
- Siga roteamento de exit codes antes de retries (veja README ou [AGENTS.pt-BR.md](AGENTS.pt-BR.md)).
- Leia [AGENTS.pt-BR.md](AGENTS.pt-BR.md) e [../INTEGRATIONS.pt-BR.md](../INTEGRATIONS.pt-BR.md).
- Formas de eventos e payloads: [docs/schemas/README.md](schemas/README.md).
- Nunca logue primary-key, senhas de host ou segredos decifrados.
