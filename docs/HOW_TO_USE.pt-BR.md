# Como usar ssh-cli

> Vá da instalação ao primeiro comando remoto em menos de 60 segundos.

- Leia este documento em [inglês](HOW_TO_USE.md).
- Volte ao [README.pt-BR.md](../README.pt-BR.md) para o mapa completo de comandos.
- Linha de produto documentada aqui: **0.4.1** (GAP-001–014 fechados; residual LOG/JSON/CLI fechado; correção wire AUD-SCP + JSON de agente para scp/tunnel; AUD-POST EXP-001/TUN-002/CLI-005/006/IO-009 fechados).


## Pré-requisitos
- Instale Rust MSRV 1.85.0 ou superior via rustup.
- Garanta conectividade de rede até o host SSH alvo.
- Tenha senha ou chave privada OpenSSH para esse host.
- Prefira um XDG config home gravável para storage multi-host.
- Instale com `cargo install ssh-cli --locked` (**0.4.1+** no crates.io; evite **0.3.9** para SCP).
- Não dependa do crates.io **0.3.9** para SCP: aquela release anunciava transferência, mas o protocolo wire estava quebrado (arquivos remotos de 0 bytes ou timeouts). Use **0.4.1+**.


## Primeiro comando em 60 segundos
### Instale, cadastre, execute

```bash
cargo install ssh-cli --locked
# A master-key é auto-criada na primeira gravação de segredo; init explícito é opcional:
ssh-cli secrets init
ssh-cli vps add --name demo --host 203.0.113.10 --user ubuntu --key ~/.ssh/id_ed25519
ssh-cli exec demo "uname -a" --json
```

- Confirme exit code 0 e inspecione campos JSON `stdout`, `stderr`, `exit_code`, `duration_ms`.
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
- Download com falha não deixa o destino final corrompido: grava `{path}.ssh-cli.partial`, aplica mode/times no partial e faz rename atômico.
- Upload faz stream em blocos de 32 KiB (não carrega o arquivo inteiro na RAM).
- mtime/mode são preservados nos dois sentidos automaticamente (remoto `scp -tp` / `-fp`; sem flag extra do usuário).
- Gerencie master-key com `ssh-cli secrets status|init|reencrypt` (nunca imprime a chave).


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
- Exporte inventário com segredos mascarados: `ssh-cli vps export -o hosts.toml` (secret vazio serializa como `""`, nunca blob `sshcli-enc:`; EXP-001 / 0.4.1).
- Importe hosts com `ssh-cli vps import --file hosts.toml`.
- Re-cifre inventário plaintext após upgrade: `ssh-cli secrets reencrypt`.
- Espere JSON automático quando stdout não é TTY, salvo `--output-format`.
- Espere senha vazia em hosts só-chave como JSON `null` (não `"***"`); senhas não vazias mascaram como `***`; texto humano em show usa "(não definida)" para vazio.
- Em falha de `scp --json`, parseie o envelope de erro JSON em **stderr** (`exit_code`, `message`), não prosa humana.


## Configuração
### Inventário multi-host XDG
- Resolva o path de config com `ssh-cli vps path`.
- Espere gravações atômicas em `config.toml` mode 0600 (tempfile + fsync + flock).
- Espere arquivos irmãos `active`, `known_hosts` e `secrets.key` ao lado do config.
- Sobrescreva o diretório só em testes com `--config-dir` ou `SSH_CLI_HOME`.
- Armazene timeout, max_command_chars, max_output_chars, segredos sudo e su por host.
- Cifragem at-rest por padrão (ChaCha20-Poly1305): segredos viram blobs `sshcli-enc:v1:…`.
- Ordem da master-key: `SSH_CLI_SECRETS_KEY` → `SSH_CLI_SECRETS_KEY_FILE` → keyring (`SSH_CLI_USE_KEYRING=1`) → XDG `secrets.key`.
- Opt-out só para testes: `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`.


## Subcomandos não cobertos acima
- `health-check [--timeout <ms>]` sonda conectividade e imprime latência (`vps add --check` após cadastro); sobrescreva o timeout quando o padrão do host for longo ou curto demais.
- Paridade auth em `health-check` (0.4.1 / CLI-006): `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`.
- Nível de tracing padrão é error para manter stderr de JSON e tunnel limpos; use `RUST_LOG` ou `-v` (debug) ao diagnosticar.
- `tunnel` exige porta local, host remoto, porta remota e `--timeout-ms`.
- Opcional: `tunnel --json` emite `event: "tunnel_listening"` estruturado no stdout após o bind local (`docs/schemas/tunnel-listening.schema.json`); deadline pós-bind sai com exit **0** após o agente receber o evento (TUN-002 / 0.4.1); timeout pré-bind permanece **74**.
- Paridade auth em `tunnel` (0.4.1 / CLI-005): `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`.
- `completions` grava scripts de completion no stdout.
- `su-exec` exige senha `su` configurada no registro do host.
- `secrets` gerencia a master-key de cifragem sem nunca imprimi-la.


## Integração com agentes de IA
- Carregue o pacote de skill em `skills/ssh-cli-pt/`.
- Prefira saída JSON para parsing de tools.
- Siga roteamento de exit codes antes de retries (veja README ou [AGENTS.pt-BR.md](AGENTS.pt-BR.md)).
- Leia [AGENTS.pt-BR.md](AGENTS.pt-BR.md) e [../INTEGRATIONS.pt-BR.md](../INTEGRATIONS.pt-BR.md).
- Nunca logue master-key, senhas de host ou segredos decifrados.
