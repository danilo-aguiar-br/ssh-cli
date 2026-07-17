# Integrações

> Conecte 10+ agentes de coding a servidores remotos com ssh-cli one-shot.

- Read this document in [English](INTEGRATIONS.md).
- Combine este catálogo com [docs/AGENTS.pt-BR.md](docs/AGENTS.pt-BR.md) e [skills/ssh-cli-pt/SKILL.md](skills/ssh-cli-pt/SKILL.md).


## Aliases de flags
### Aliases camelCase implementados no clap (não invente outros)
- Use `--sudoPassword` como alias de `--sudo-password`.
- Use `--suPassword` como alias de `--su-password`.
- Use `--maxChars` como alias legado mapeado para `max_command_chars`.
- Use `--disableSudo` como alias de `--disable-sudo`.
- **Não** há alias camelCase para `--config-dir`, `--output-format` ou `--no-color` — use exatamente o kebab-case.

## Novas flags por versão
### Acompanhe o crescimento da superfície sem ler o código
- `0.5.1` **export/import agent-first + wire v3**: corpo de `vps export` é **TOML por padrão** (TTY e pipe); envelope JSON de agente só com `--json`; `vps import` aceita TOML (chaves EN + aliases PT) **ou** envelopes JSON `vps-export`; dual-read serialize EN + aliases PT; host **schema v3**; flags CLI `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (prefira ao env); eventos `secrets init|reencrypt --json` `secrets-init` / `secrets-reencrypt`; auto primary-key emite `secrets-key-auto-created`; caminhos de sucesso CRUD usam eventos JSON `emit_success`; `--include-secrets` em pipe/non-TTY exige `-o`/`--output` ou `--i-understand-secrets-on-stdout`; tunnel `--bind` (default `127.0.0.1`); import `TomlDe` → exit **65**; `SshAuthentication` → **77**; SCP missing `file not found: <path>` (exit **66**); warn de timeout se `<1000` ms; warn stderr de password em argv; doctor `secrets_plaintext_opt_out` é **bool**.
- `0.4.2` tunnel porta efêmera `local_port=0` reporta porta atribuída pelo SO após bind (TUN-003); SCP remoto ausente → exit **66** (IO-010); envelope `vps export --json` com `event: "vps-export"`; e2e E15/E16; suite `gaps_v042`.
- `0.4.1` AUD-POST + SCP **somente arquivos regulares** (herda wire 0.4.0; sem `-r` / sem SFTP): wire SCP sólido (evite crates.io **0.3.9** SCP quebrado); flags scp `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` → `docs/schemas/scp-transfer.schema.json` com `event: "scp-transfer"` obrigatório (IO-009); download grava `{path}.ssh-cli.partial` e faz rename; preserve mtime/mode bi-dir; upload em stream 32 KiB; `tunnel --json` emite `tunnel_listening` após bind e deadline pós-bind sai **0** (TUN-002); export redacted não emite `sshcli-enc:` para secret vazio (EXP-001); paridade auth `tunnel` (CLI-005) e `health-check` (CLI-006); envelope JSON de erro scp em stderr com `--json`.
- `0.4.0` wire SCP sólido (corrige crates.io **0.3.9**); transfers file-only; `tunnel --json` / `tunnel_listening`.
- `0.3.9` filtro de tracing default `error` (agent-first); senha vazia serializa como JSON `null` em hosts só-chave; `health-check --timeout <ms>`; auditoria de docs de product line.
- `0.3.8` russh 0.62.2; stdout de tunnel limpo para agentes; sem VPS ativa sai com `66` (`EX_NOINPUT`); `cargo deny` com `yanked=deny`.
- `0.3.7` `--output-format` no CRUD VPS; `health-check --json`; `--quiet`; envelope JSON de erro; timeout do tunnel cobre connect.
- `0.3.6` adiciona cifragem at-rest default, `secrets status|init|reencrypt`, `SSH_CLI_ALLOW_PLAINTEXT_SECRETS`, campos doctor de secrets, `scripts/e2e_real_ssh.sh`.
- `0.3.5` adiciona caminhos de passphrase stdin, JSON auto em non-TTY, doctor `secrets_at_rest`, export atômico residual.
- `0.3.4` adiciona `--key`, `--key-passphrase`, `--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--timeout-ms` (tunnel), `--disable-sudo`, `--description`, `--replace-host-key`, `max_command_chars`, `max_output_chars`, `vps doctor`, `vps export`, `vps import`, `su-exec`.
- `0.2.0` adiciona overrides runtime `--password`, `--sudo-password`, `--timeout` e aliases camelCase.
- Prefira **0.5.1+** para roundtrip export/import, wire schema v3, SCP funcional + `tunnel --json` / `--bind`, automação SSH completa, cifragem default e supply-chain limpa.


## Tabela resumo

| Agent / Platform | Integration style | JSON | Notes |
| --- | --- | --- | --- |
| Claude Code | subprocess CLI + skill | yes | Prefer skill package |
| Cursor | shell / agent tools | yes | Use `--json` |
| Windsurf | shell tool | yes | One-shot per task |
| Codex CLI | shell tool | yes | Map sysexits |
| OpenCode | shell tool | yes | One-shot only |
| Aider | shell commands | yes | Store hosts once |
| Continue | custom command | yes | XDG multi-host |
| Gemini CLI | shell tool | yes | Prefer stdin secrets |
| OpenHands | sandbox shell | yes | Bound tunnel timeouts |
| Generic bash/zsh | direct install | yes | Completions available |


## Claude Code
- Instale `ssh-cli` no PATH com `cargo install ssh-cli --locked`.
- Carregue [skills/ssh-cli-pt/SKILL.md](skills/ssh-cli-pt/SKILL.md) ou o pacote en.
- Cadastre hosts uma vez com `vps add` (prefira `--password-stdin`) e chame `exec` por tarefa.
- Prefira envelopes `--json` para resultados estruturados.
- Faça parse só do stdout; stderr default fica silencioso no nível de tracing `error` (defina `RUST_LOG` só ao depurar).
- Use `ssh-cli secrets status` / `vps doctor --json` como preflight de cifragem e paths.


## Cursor
- Adicione regra de projeto que prefere `ssh-cli` a processos Node SSH de longa duração.
- Mantenha credenciais fora do chat usando hosts salvos e flags stdin.
- Faça parse só do JSON em stdout; stderr default fica silencioso no nível de tracing `error` (ignore tracing salvo se definir `RUST_LOG`).


## Windsurf
- Invoque comandos one-shot após o cadastro de hosts.
- Nunca mantenha tunnel aberto sem `--timeout-ms`.


## Codex CLI
- Trate exits não zero como falhas tipadas usando a tabela de exit codes do README.
- Faça retry só em códigos transitórios de IO/timeout, nunca em auth ou usage.


## OpenCode
- Use modo shell tool com arrays de argv explícitos.
- Evite embutir senhas no texto do prompt; use registry ou stdin.


## Aider
- Documente nomes de hosts no repo sem segredos.
- Chame `ssh-cli exec <name> "..."` para ops remotas durante loops de edição.


## Continue
- Mapeie custom commands para subcomandos `ssh-cli` com `--json`.
- Use `vps doctor --json` como preflight de saúde para sessões de agente.


## Gemini CLI
- Prefira auth por chave e `vps show` mascarado para verificação.
- Mantenha elevação desabilitada salvo quando a tarefa exigir root.


## OpenHands
- Rode dentro do sandbox com policy de rede que permite só hosts alvo.
- Force tunnels limitados e timeouts curtos.


## Shell genérico
- Instale completions com `ssh-cli completions <shell>`.
- Exporte `SSH_CLI_HOME` apenas para sandboxes de teste isolados.
