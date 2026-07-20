# ssh-cli

- Nota histórica: **0.4.2** fechou TUN-003 / IO-010; **0.5.0** foi o rename EN/API + reencrypt de secrets force-init; linha de produto atual é **0.5.2** (roundtrip export/import agent-first, wire schema v3 dual-read, flags CLI de secrets, tunnel `--bind`).

[![docs.rs](https://img.shields.io/docsrs/ssh-cli)](https://docs.rs/ssh-cli)
[![crates.io](https://img.shields.io/crates/v/ssh-cli)](https://crates.io/crates/ssh-cli)
[![License](https://img.shields.io/crates/l/ssh-cli)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85.0-orange)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-blue)](https://www.rust-lang.org)
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
- Migre de 0.3.3+ em [docs/MIGRATION.pt-BR.md](docs/MIGRATION.pt-BR.md) (linha alvo **0.5.2**).
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
- Tunnel limitado via `--timeout-ms` obrigatório; `--bind` opcional (default `127.0.0.1`)
- Wire **schema v3**: serializa chaves TOML em inglês; dual-read EN + aliases PT legados no load
- `vps export` emite **TOML por padrão** (TTY e pipe); envelope JSON de agente só com `--json`; redacted por padrão; secrets vazios ficam `""`; secrets não vazios redacted mascaram como `***` (`FIXED_MASK`); `--include-secrets` em pipe/non-TTY exige `-o`/`--output` ou `--i-understand-secrets-on-stdout`
- `vps import` aceita TOML (chaves EN + aliases PT) **ou** envelopes JSON `vps-export`; skeletons redacted precisam de `--allow-incomplete`
- SCP upload e download de **arquivos regulares apenas** (sem `-r` no SCP; wire sólido em **0.4.0**; prefira **0.5.2+** — evite SCP do crates.io 0.3.9)
- **SFTP** (`ssh-cli sftp`): upload/download (opcional `--recursive`, sem seguir symlink), `ls`/`mkdir`/`rmdir`/`rm`/`stat`/`rename`; eventos JSON `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch`
- Paridade de flags scp com exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` (contrato `docs/schemas/scp-transfer.schema.json`; JSON de sucesso exige `event: "scp-transfer"`)
- Download SCP grava `{path}.ssh-cli.partial` e rename atômico; preserve mtime/mode bi-direcional; upload em stream de 32 KiB
- JSON SCP de sucesso exige `event: "scp-transfer"` (0.4.1 IO-009); remoto ausente → `file not found: <path>` exit **66**
- `tunnel --json` emite `tunnel_listening` estruturado após bind local; deadline pós-bind sai com exit **0** (não 74) após o agente receber `tunnel_listening`
- Paridade de flags auth em `tunnel` e `health-check` com exec/scp: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`
- Health-check de latência com `--timeout` opcional
- Export redacted (`vps export`) limpa segredos e **nunca** emite ciphertext `sshcli-enc:…` para senha vazia (`""` serializa como string vazia)
- Completions para bash zsh fish powershell
- Segredos via flags stdin para evitar leak em argv
- **Cifragem at-rest por padrão** (ChaCha20-Poly1305) com auto `secrets.key` XDG; flags CLI `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` preferidas ao env
- UX de master-key: `secrets status|init|reencrypt` com eventos `--json` `secrets-init` / `secrets-reencrypt`; a 1ª gravação de segredo embute `secrets_key_auto_created: true` no mesmo JSON `vps-added` (um documento)
- known_hosts TOFU e escrita atômica do config com flock
- Hosts só-chave: senha vazia serializa como JSON `null` (não `"***"`) em `vps list` / `show`
- Filtro de tracing default é `error` (stderr limpo para agentes); use `-v` para debug (`RUST_LOG` ambiente é ignorado)
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
- Prefira crates.io com lockfile: `cargo install ssh-cli --locked` (**0.5.2+** no crates.io; evite **0.3.9** para SCP).
- Rebuild a partir do checkout: `cargo install --path . --locked`
- **Não** use install sem `--locked` salvo se validou o resolve crypto com os pins.
- Force upgrade após release: `cargo install ssh-cli --locked --force`
- Compile musl com feature de alocador no Alpine: `--features musl-allocator`
- Exija Rust MSRV 1.85.0 ou superior


## Features
### Flags de feature do Cargo
| Feature | Padrão | Descrição |
|---------|--------|-----------|
| `ssh-real` | sim | SSH real via `russh` + `aws-lc-rs` (compressão só `none`) |
| `tls` | sim | rustls ≥0.23.18 + `aws_lc_rs`: SSH-over-TLS, mTLS, ACME |
| `musl-allocator` | não | `mimalloc` como global allocator (binário; útil em musl/Alpine) |

- O caminho de install sempre ativa defaults (`ssh-real` + `tls`).
- Desative SSH/TLS só para diagnóstico: `cargo build --no-default-features`.
- docs.rs compila com `all-features = true` e `--cfg docsrs` (ver `Cargo.toml` `[package.metadata.docs.rs]`).

### Política de crypto (G-TLS)
- Padrão: **SSH-2** em TCP puro (`russh` + **aws-lc-rs**). Host keys **TOFU** sob XDG.
- Opcional **SSH-over-TLS** (`vps add --tls`): handshake rustls + SSH no stream. `CryptoProvider` (`aws_lc_rs`) instalado uma vez no `main`.
- **mTLS / ACME:** `ssh-cli tls mtls …` e `ssh-cli tls acme …` sob XDG `tls/` (sem env de armazenamento).
- Sem OpenSSL / `native-tls` / dual `ring`. Compressão SSH só **`none`**.
- Validação ACME / `invalidContact` / problemas 4xx → exit **64** não-retryable (G-E2E-01); rate limits permanecem exit **74** transitório.
- Detalhes: [SECURITY.pt-BR.md](SECURITY.pt-BR.md).


## Targets
### Plataformas cobertas pelo metadata do docs.rs
- `x86_64-unknown-linux-gnu` (default)
- `x86_64-apple-darwin`, `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `aarch64-unknown-linux-musl`
- Veja [docs/CROSS_PLATFORM.pt-BR.md](docs/CROSS_PLATFORM.pt-BR.md) para notas de runtime.


## Uso
### Cadastre hosts e execute comandos one-shot
- **Cifragem at-rest por padrão** (ChaCha20-Poly1305): auto `secrets.key` na primeira gravação; prefira flags CLI `--secrets-key-file`, `--use-keyring`, `--allow-plaintext-secrets` (ou XDG `secrets.key`); gerencie com `ssh-cli secrets status|init|reencrypt`. `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` são **rejeitados fail-closed** (não são store). Opt-out só para testes via `--allow-plaintext-secrets`.
- Prefira `--password-stdin` / `--key` a segredos em argv.
- Adicione hosts com senha via `vps add --password` ou `--password-stdin`.
- Adicione hosts com chave via `vps add --key ~/.ssh/id_ed25519`.
- Em hosts só-chave, campos de senha vazios serializam como JSON `null` em `vps list` / `show` (segredos não vazios mascaram como `"***"`).
- Marque o host ativo com `connect <name>`.
- Rode shells remotos com `exec <vps> "<cmd>"`.
- Eleve com `sudo-exec` ou `su-exec` quando configurado.
- Diagnostique paths XDG com `doctor --json` (ou `vps doctor --json`).
- Descubra contratos com `ssh-cli schema` / `ssh-cli commands`.
- Exporte inventário com segredos mascarados via `vps export` (TOML por padrão; `--json` para envelope de agente).


## Comandos
### Superfície de produto para humanos e agentes

| Comando | Propósito |
|---|---|
| `ssh-cli vps add` | Cadastra host (senha **ou** chave **ou** `--use-agent` / `--agent-socket`) |
| `ssh-cli vps list [--json]` | Lista hosts com segredos mascarados |
| `ssh-cli vps show <name> [--json]` | Mostra um host com segredos mascarados |
| `ssh-cli vps edit <name>` | Altera campos do host |
| `ssh-cli vps remove <name>` | Remove host |
| `ssh-cli vps path` | Imprime caminho do `config.toml` |
| `ssh-cli vps doctor [--json]` | Mostra camada XDG, schema e paths |
| `ssh-cli doctor [--json]` | Alias root de `vps doctor` (G-E2E-03) |
| `ssh-cli schema [NAME]` | Emite catálogo de JSON Schema embarcado ou um schema (G-E2E-02) |
| `ssh-cli commands` | Emite a árvore completa de comandos em JSON (descoberta de agente) |
| `ssh-cli vps export` | Exporta hosts em **TOML por padrão** (TTY e pipe); envelope JSON de agente só com `--json`; segredos mascarados por padrão; vazios como `""` (nunca blob `sshcli-enc:`); `--include-secrets` em pipe/non-TTY exige `-o`/`--output` ou `--i-understand-secrets-on-stdout` |
| `ssh-cli vps import --file` | Importa hosts de **TOML** (chaves EN + aliases PT) **ou** envelope JSON `vps-export`; skeletons redacted precisam de `--allow-incomplete` |
| `ssh-cli connect <name>` | Grava arquivo irmão `active` |
| `ssh-cli exec <vps> <cmd>` | Comando remoto one-shot |
| `ssh-cli exec --all '<cmd>'` | Comando remoto **concorrente com bound** em todos os hosts (`exec-batch` JSON) |
| `ssh-cli sudo-exec <vps> <cmd>` / `--all` | sudo one-shot com packing seguro (frota com `--all`) |
| `ssh-cli su-exec <vps> <cmd>` / `--all` | Elevação `su -` one-shot (frota com `--all`) |
| `ssh-cli scp upload|download` | Somente arquivos regulares (sem `-r` no SCP); flags auth + `--use-agent` + `--json` → `scp-transfer`; remoto ausente → exit **66**; **`--all`** → `scp-batch` |
| `ssh-cli sftp upload\|download\|ls\|mkdir\|rmdir\|rm\|stat\|rename` | Subsistema SFTP v3; `--recursive` (sem seguir symlink); JSON `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch` |
| `ssh-cli tunnel ... --timeout-ms N [--bind ADDR] [--json]` | Port-forward local com deadline; `--bind` default `127.0.0.1`; `--json` emite `tunnel_listening` após bind; pós-bind exit **0**; auth: `--password-stdin`, `--key`, `--key-passphrase[-stdin]`; accepts concorrentes limitados por `--max-concurrency` |
| `ssh-cli health-check [<vps>] [--timeout N]` / `--all` | Sonda de conectividade (timeout opcional em ms); auth: `--password-stdin`, `--key`, `--key-passphrase[-stdin]`; **`--all`** → frota (`health-check-batch`) |
| `ssh-cli --max-concurrency N …` | Cap global (1..=64) de fan-out multi-host e forwards de tunnel (fórmula auto CPUs×RAM quando omitido) |
| `ssh-cli secrets status|init|reencrypt` | Master-key e cifragem at-rest (nunca imprime a chave); `--json` emite `secrets-init` / `secrets-reencrypt`; a 1ª gravação de segredo embute `secrets_key_auto_created: true` no mesmo JSON `vps-added` (um documento); flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` |
| `ssh-cli completions <shell>` | Scripts de completion de shell |


## Configuração (store de produto só via CLI)
### Knobs de produto são flags e XDG — não stores `SSH_CLI_*` em env

| Controle | Como | Exemplo |
|---|---|---|
| Diretório de config | `--config-dir` (senão XDG/`directories`) | `ssh-cli --config-dir /tmp/ssh-cli-test vps list` |
| Idioma | `--lang` ou `ssh-cli locale set <code>` (XDG `lang`) | `ssh-cli --lang pt-BR …` |
| Formato de saída | `--json` / `--output-format json\|text` | `ssh-cli exec h uptime --json` |
| Concorrência | `--max-concurrency N` (1..=64; fórmula auto quando omitido) | `ssh-cli --max-concurrency 8 exec --all id --json` |
| Primary-key | `--secrets-key-file`, `--use-keyring`, ou XDG `secrets.key` | `ssh-cli --secrets-key-file ./k secrets status` |
| Opt-out plaintext | `--allow-plaintext-secrets` (**só testes**) | `ssh-cli --allow-plaintext-secrets …` |

### Secrets env fail-closed (não é store)
| Variável | Comportamento |
|---|---|
| `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` | **Rejeitadas** se presentes — use XDG `secrets.key`, `--secrets-key-file` ou `--use-keyring` |

### Fronteira OS / host (só detecção — não store de config de produto)
| Variável | Papel |
|---|---|
| `HOME` | Home do SO para resolução XDG |
| `TERM` / `NO_COLOR` / `CLICOLOR_FORCE` | Capacidade de terminal / cor |
| `CI` / marcadores Flatpak | Detecção de runtime (`vps doctor` `runtime.*`) |
| `RUST_LOG` | **Ignorado** pelo produto (não é store de config); use `-v` para debug |

- O filtro de tracing default é `error` para manter stderr limpo; passe `-v` para debug (`RUST_LOG` ambiente é ignorado).
- Nunca coloque senhas SSH em variáveis de ambiente; use inventário + stdin.
- O produto **não** lê `SSH_CLI_HOME`, `SSH_CLI_LANG`, `SSH_CLI_FORCE_TEXT` nem `SSH_CLI_MAX_CONCURRENCY` como stores de config.


## Padrões de integração
### Conecte agentes só com subprocessos one-shot
- Invoque `ssh-cli` como subprocesso com argv explícito.
- Prefira `--json` ou `--output-format json` para parsing de máquina.
- Faça parse só do stdout; o nível de log default é `error`, então stderr fica silencioso em pipelines JSON — passe `-v` ao diagnosticar (`RUST_LOG` ambiente é ignorado).
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
- Tracing default é `error`, então tratamento de exit e JSON em stdout ficam sem ruído INFO; use `-v` só ao diagnosticar (`RUST_LOG` ambiente é ignorado).
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
- Install falha em drift crypto RC: rode com `--locked` ou use a linha **0.5.2+** (russh 0.62.2) (`scripts/verify_install_resolve.sh`).
- Auth falha em hosts só-chave: defina `--key` em `vps add` ou passe `--key` / `--password-stdin` no `exec` (auth rejeitada sai com **77**).
- Auth falha com chave com passphrase: use `--key-passphrase-stdin` (exit **77** na rejeição).
- Host key mudou: confirme legitimidade e rode com `--replace-host-key`.
- Comando rejeitado por tamanho: aumente `max_command_chars` ou encurte o comando.
- Config com secrets cifrados sem chave: rode `ssh-cli secrets init` ou restaure `secrets.key` / env / `--secrets-key-file`.
- sudo-exec desabilitado: remova `--disable-sudo` e defina `disable_sudo=false` no host.
- Ruído inesperado em stderr em pipelines JSON: o nível default já é `error`; passe `-v` ao diagnosticar (`RUST_LOG` ambiente é ignorado).
- SCP do crates.io **0.3.9** falha ou grava remoto 0 bytes: atualize para **0.5.2+** (fix de wire desde 0.4.0); só arquivos regulares, não diretórios.
- SCP remoto ausente: mensagem `file not found: <path>` e exit **66** (prefira **0.5.2+**; IO-010 em 0.4.2).
- Instalou **0.4.0** e `vps export` redacted mostra ciphertext `sshcli-enc:` para senha vazia, ou tunnel emite `ok:true` e depois exit **74**: atualize para **0.5.2+** (EXP-001 / TUN-002 desde 0.4.1).
- Download SCP falha no meio: destino ausente ou arquivo anterior intacto (parcial usa `.ssh-cli.partial`).
- Import com TOML inválido: erros de parse saem com exit **65** (`TomlDe` / erro de dados).
- Import de skeleton redacted sem segredos: passe `--allow-incomplete`.
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
