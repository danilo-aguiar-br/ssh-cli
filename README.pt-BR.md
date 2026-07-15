# ssh-cli

[![crates.io](https://img.shields.io/crates/v/ssh-cli.svg)](https://crates.io/crates/ssh-cli)
[![docs.rs](https://docs.rs/ssh-cli/badge.svg)](https://docs.rs/ssh-cli)
[![MSRV](https://img.shields.io/badge/MSRV-1.85.0-blue)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

> Dê a qualquer LLM poder SSH remoto em um binário one-shot seguro.

- Read this document in [English](README.md).
- Instale com `cargo install ssh-cli --locked` para grafo alinhado ao lockfile.
- Atualize com `cargo install ssh-cli --locked --force`.
- Verifique com `ssh-cli --version`.
- Leia o histórico em [CHANGELOG.pt-BR.md](CHANGELOG.pt-BR.md).
- Integre agentes via [docs/AGENTS.pt-BR.md](docs/AGENTS.pt-BR.md) e [INTEGRATIONS.pt-BR.md](INTEGRATIONS.pt-BR.md).
- Siga o primeiro uso em [docs/HOW_TO_USE.pt-BR.md](docs/HOW_TO_USE.pt-BR.md).
- Copie receitas de [docs/COOKBOOK.pt-BR.md](docs/COOKBOOK.pt-BR.md).
- Confira plataformas em [docs/CROSS_PLATFORM.pt-BR.md](docs/CROSS_PLATFORM.pt-BR.md).
- Migre de 0.3.3+ em [docs/MIGRATION.pt-BR.md](docs/MIGRATION.pt-BR.md) (linha alvo **0.4.0**).
- Execute testes via [docs/TESTING.pt-BR.md](docs/TESTING.pt-BR.md).
- Consuma contratos JSON em [docs/schemas/README.md](docs/schemas/README.md).
- Ensine LLMs com [skills/ssh-cli-pt/SKILL.md](skills/ssh-cli-pt/SKILL.md).


## O que é?
### CLI SSH multi-host one-shot para agentes
- Entregue um binário Rust único sem runtime Node e sem daemon.
- Opere N hosts VPS a partir de storage XDG sem arquivos `.env`.
- Autentique com senha ou chave privada por host.
- Execute `exec`, `sudo-exec` e `su-exec` como processos one-shot.
- Capture stdout e stderr com JSON estruturado para orquestração.
- Detecte locale entre `en-US` e `pt-BR`.
- Desative telemetria completamente em todo build.


## Por que ssh-cli?
### Substitua processos SSH de longa duração por um binário que morre após rodar
- Evite processos Node residentes que mantêm sockets abertos entre tarefas.
- Corte RAM e CPU de sessões SSH de longa duração.
- Cadastre credenciais multi-host uma vez no XDG com escrita atômica.
- Alinhe packing de comando e semântica dual maxChars com o contrato one-shot para agentes.
- Confie em host keys via TOFU known_hosts em vez de always-trust.
- Encaminhe erros com códigos sysexits que agentes classificam com segurança.


## Superpoderes
### Capacidades que tornam agentes produtivos
- CRUD multi-host com `vps add|list|show|edit|remove|path|doctor|export|import`
- Execução remota one-shot com `exec`, `sudo-exec` e `su-exec`
- Packing seguro de `sudo` via `sh -c` e escape de shell
- Auth por chave privada com passphrase opcional
- Limites dual `max_command_chars` e `max_output_chars`
- Timeout com abort remoto best-effort
- Tunnel limitado via `--timeout-ms` obrigatório
- SCP upload e download de **arquivos regulares apenas** (sem diretórios recursivos / sem SFTP; wire sólido em **0.4.0** — evite SCP do crates.io 0.3.9)
- Paridade de flags scp com exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` (contrato `docs/schemas/scp-transfer.schema.json`)
- Download SCP grava `{path}.ssh-cli.partial` e rename atômico; preserve mtime/mode bi-direcional; upload em stream de 32 KiB
- `tunnel --json` emite `tunnel_listening` estruturado após bind local
- Health-check de latência com `--timeout` opcional
- Completions para bash zsh fish powershell
- Segredos via flags stdin para evitar leak em argv
- **Cifragem at-rest por padrão** (ChaCha20-Poly1305) com auto `secrets.key` XDG
- UX de master-key: `secrets status|init|reencrypt`
- known_hosts TOFU e escrita atômica do config com flock
- Hosts só-chave: senha vazia serializa como JSON `null` (não `"***"`) em `vps list` / `show`
- Filtro de tracing default é `error` (stderr limpo para agentes); override com `RUST_LOG` ou `-v` (debug)
- Install com russh 0.62.2 para `cargo install --locked` limpo


## Início rápido
### Instale e rode o primeiro comando remoto

```bash
cargo install ssh-cli --locked
ssh-cli secrets init   # opcional; auto-cria master-key na 1ª gravação de segredo
printf '%s' 'demo-password-not-real' | ssh-cli vps add \
  --name prod \
  --host prod.example.com \
  --port 22 \
  --user admin \
  --password-stdin
ssh-cli connect prod
ssh-cli exec prod "hostname" --json
```


## Instalação
### Escolha o caminho de install do seu ambiente
- Prefira crates.io com lockfile: `cargo install ssh-cli --locked` (**0.4.0+** no crates.io; evite **0.3.9** para SCP).
- Rebuild a partir do checkout: `cargo install --path . --locked`
- **Não** use install sem `--locked` salvo se validou o resolve crypto com os pins.
- Force upgrade após release: `cargo install ssh-cli --locked --force`
- Compile musl com feature de alocador no Alpine: `--features musl-allocator`
- Exija Rust MSRV 1.85.0 ou superior


## Uso
### Cadastre hosts e execute comandos one-shot
- **Cifragem at-rest por padrão** (ChaCha20-Poly1305): auto `secrets.key` na primeira gravação; override via `SSH_CLI_SECRETS_KEY` / `_FILE` / keyring; gerencie com `ssh-cli secrets status|init|reencrypt`. Opt-out: `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` (só testes).
- Prefira `--password-stdin` / `--key` a segredos em argv.
- Adicione hosts com senha via `vps add --password` ou `--password-stdin`.
- Adicione hosts com chave via `vps add --key ~/.ssh/id_ed25519`.
- Em hosts só-chave, campos de senha vazios serializam como JSON `null` em `vps list` / `show` (segredos não vazios mascaram como `"***"`).
- Marque o host ativo com `connect <name>`.
- Rode shells remotos com `exec <vps> "<cmd>"`.
- Eleve com `sudo-exec` ou `su-exec` quando configurado.
- Diagnostique paths XDG com `vps doctor --json`.
- Exporte inventário com segredos mascarados via `vps export`.


## Comandos
### Superfície de produto para humanos e agentes

| Comando | Propósito |
|---|---|
| `ssh-cli vps add` | Cadastra host (senha ou chave) |
| `ssh-cli vps list [--json]` | Lista hosts com segredos mascarados |
| `ssh-cli vps show <name> [--json]` | Mostra um host com segredos mascarados |
| `ssh-cli vps edit <name>` | Altera campos do host |
| `ssh-cli vps remove <name>` | Remove host |
| `ssh-cli vps path` | Imprime caminho do `config.toml` |
| `ssh-cli vps doctor [--json]` | Mostra camada XDG, schema e paths |
| `ssh-cli vps export` | Exporta hosts (segredos mascarados por padrão) |
| `ssh-cli vps import --file` | Importa hosts de TOML |
| `ssh-cli connect <name>` | Grava arquivo irmão `active` |
| `ssh-cli exec <vps> <cmd>` | Comando remoto one-shot |
| `ssh-cli sudo-exec <vps> <cmd>` | sudo one-shot com packing seguro |
| `ssh-cli su-exec <vps> <cmd>` | Elevação `su -` one-shot |
| `ssh-cli scp upload|download` | Somente arquivos regulares (sem `-r`/SFTP); flags `--timeout`, `--password-stdin`, `--key`, `--key-passphrase[-stdin]`, `--json` → schema `scp-transfer`; preserve mtime/mode |
| `ssh-cli tunnel ... --timeout-ms N [--json]` | Port-forward local com deadline; `--json` emite `tunnel_listening` após bind |
| `ssh-cli health-check [<vps>] [--timeout N]` | Sonda de conectividade (timeout opcional em ms) |
| `ssh-cli secrets status|init|reencrypt` | Master-key e cifragem at-rest (nunca imprime a chave) |
| `ssh-cli completions <shell>` | Scripts de completion de shell |


## Variáveis de ambiente
### Overrides permitidos para testes e locales

| Variável | Descrição | Exemplo |
|---|---|---|
| `SSH_CLI_HOME` | Sobrescreve diretório base de config | `/tmp/ssh-cli-test` |
| `SSH_CLI_LANG` | Sobrescreve locale | `pt-BR` |
| `SSH_CLI_SECRETS_KEY` | Master-key em 64 hex (cifra at-rest) | *(nunca logue)* |
| `SSH_CLI_SECRETS_KEY_FILE` | Arquivo com master-key hex 64 | `~/.config/ssh-cli/secrets.key` |
| `SSH_CLI_USE_KEYRING` | Usa keyring do SO para master-key | `1` |
| `SSH_CLI_ALLOW_PLAINTEXT_SECRETS` | Opt-out da cifragem default (**só testes**) | `1` |
| `NO_COLOR` | Desativa cores ANSI | `1` |
| `CLICOLOR_FORCE` | Força cores ANSI | `1` |
| `RUST_LOG` | Filtro de tracing (nível default é `error`) | `debug` |

- Prefira flags CLI a environment em runs de agentes em produção.
- O filtro de tracing default é `error` para manter stderr limpo; defina `RUST_LOG` só ao depurar (ou passe `-v` para debug).
- Nunca coloque senhas SSH em variáveis de ambiente; use inventário + stdin.
- Variáveis de master-key cifraram **segredos no disco**, não substituem senha SSH.


## Padrões de integração
### Conecte agentes só com subprocessos one-shot
- Invoque `ssh-cli` como subprocesso com argv explícito.
- Prefira `--json` ou `--output-format json` para parsing de máquina.
- Faça parse só do stdout; o nível de log default é `error`, então stderr fica silencioso em pipelines JSON — defina `RUST_LOG` só para debug quando precisar.
- Mapeie exits não zero com semântica sysexits antes de retry.
- Cadastre hosts uma vez via `vps add` e chame `exec` por tarefa.
- Passe segredos com `--password-stdin` quando history de argv for arriscado.
- Leia [INTEGRATIONS.pt-BR.md](INTEGRATIONS.pt-BR.md) para notas por agente.


## Exit codes
### Códigos no estilo sysexits que agentes devem mapear antes de retry

| Código | Significado |
|---|---|
| `0` | Sucesso |
| `1` | Erro genérico de runtime |
| `64` | Uso / argumentos inválidos |
| `65` | Erro de dados (JSON/TOML/schema) |
| `66` | VPS ou arquivo de entrada não encontrado |
| `73` | Não foi possível criar config/saída |
| `74` | IO ou conexão/timeout SSH |
| `77` | Autenticação rejeitada ou host-key / política sudo |
| `130` | SIGINT |
| `143` | SIGTERM |

- Prefira `--json` ou JSON automático quando stdout não é TTY (`--output-format` sobrescreve).
- Tracing default é `error`, então tratamento de exit e JSON em stdout ficam sem ruído INFO; use `RUST_LOG=debug` ou `-v` só ao diagnosticar.
- Faça retry só em IO/timeout transitório (`74`), nunca em auth (`77`) ou uso (`64`).


## Performance
### Metas de cold start e memória
- Busque cold start abaixo de 100 ms em hosts Linux modernos.
- Mantenha memória do processo bem abaixo de uma sessão daemon Node de longa duração residente.
- Morra após cada comando para devolver RAM ao SO imediatamente.
- Evite tunnels longos sem `--timeout-ms`.


## Requisitos de memória
### Planeje capacidade para inventários multi-host
- O TOML de config cresce com contagem de hosts e paths.
- Buffers de saída respeitam `max_output_chars` por stream.
- O arquivo known_hosts cresce devagar com pares host:port únicos.
- Nenhum modelo de embedding e nenhum heap Node são necessários.


## FAQ de troubleshooting
### Corrija falhas comuns de install e runtime
- Install falha em drift crypto RC: rode com `--locked` ou use a linha **0.4.0+** (russh 0.62.2) (`scripts/verify_install_resolve.sh`).
- Auth falha em hosts só-chave: defina `--key` em `vps add` ou passe `--key` / `--password-stdin` no `exec`.
- Auth falha com chave com passphrase: use `--key-passphrase-stdin`.
- Host key mudou: confirme legitimidade e rode com `--replace-host-key`.
- Comando rejeitado por tamanho: aumente `max_command_chars` ou encurte o comando.
- Config com secrets cifrados sem chave: rode `ssh-cli secrets init` ou restaure `secrets.key` / env.
- sudo-exec desabilitado: remova `--disable-sudo` e defina `disable_sudo=false` no host.
- Ruído inesperado em stderr em pipelines JSON: o nível default já é `error`; defina `RUST_LOG` só como `debug` (ou `-v`) ao diagnosticar.
- SCP do crates.io **0.3.9** falha ou grava remoto 0 bytes: atualize para **0.4.0+** (fix de wire); só arquivos regulares, não diretórios.
- Download SCP falha no meio: destino ausente ou arquivo anterior intacto (parcial usa `.ssh-cli.partial`).
- macOS Gatekeeper bloqueia o binário: rode `xattr -d com.apple.quarantine /path/to/ssh-cli`.
- Permissão negada no config: garanta `chmod 600` no `config.toml` e no `secrets.key` XDG.


## Contribuindo
- Leia [CONTRIBUTING.pt-BR.md](CONTRIBUTING.pt-BR.md) antes de abrir pull request.
- Siga o framework bilíngue de documentação em toda mudança de doc pública.


## Segurança
- Leia [SECURITY.pt-BR.md](SECURITY.pt-BR.md) para reporte privado de vulnerabilidades.
- Prefira flags stdin de segredo e arquivos de chave a senhas em argv.


## Changelog
- Leia o histórico em [CHANGELOG.pt-BR.md](CHANGELOG.pt-BR.md).
- Não cole notas de release neste README.


## Licença
- Dual-license sob MIT ou Apache-2.0.
- Veja [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT) e [LICENSE-APACHE](LICENSE-APACHE).
