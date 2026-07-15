# Guia de migração

> Passe de ssh-cli 0.3.3 (ou posterior) para **0.3.9** sem perder o inventário multi-host.

- Leia este documento em [inglês](MIGRATION.md).


## O que muda

### Desde 0.3.4 (paridade de automação SSH central)
- Grafo crypto de install pinado: `cargo install --locked` funciona (GAP-014).
- Auth aceita chave privada via `--key` / `key_path` (GAP-002).
- `max_chars` vira `max_command_chars` e `max_output_chars` (GAP-004).
- `sudo-exec` empacota com `sh -c` seguro (GAP-005).
- `su-exec` consome a senha `su` gravada (GAP-003).
- Escrita de config atômica com flock e mode 0600 (GAP-007).
- Host keys com TOFU known_hosts (GAP-008).
- `tunnel` exige `--timeout-ms` (GAP-010).
- Schema de registros novos é 2.
- Licença dual MIT OR Apache-2.0.

### Desde 0.3.5
- `vps export` atômico, abort remoto TERM+KILL.
- Doctor reporta `secrets_at_rest`.
- JSON automático quando stdout não é TTY.

### Desde 0.3.6
- Cifragem at-rest por padrão dos segredos no `config.toml` (ChaCha20-Poly1305).
- Auto-cria `secrets.key` XDG (0o600) na primeira gravação de segredo.
- CLI `secrets status|init|reencrypt` (nunca imprime a master-key).
- Opt-out só para testes: `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`.
- Doctor: `secrets_key_file`, `secrets_plaintext_opt_out`.

### Desde 0.3.7
- Polimento de I/O para agentes: `--output-format` global em VPS CRUD, `health-check --json`, envelope de erro JSON, `--quiet` silencia sucesso humano.
- Tunnel `--timeout-ms` cobre connect SSH + loop.
- SCP valida arquivo local antes do connect; `vps remove` limpa `active` órfão.
- `su-exec --password-stdin`; conflitos clap para password/*_stdin.
- Exit de comando remoto não zero vira process exit `1` com `remote_exit_code` no envelope JSON.
- Segredos longos sempre mascaram como `***` (sem vazamento 12+4).
- Senha sudo/su no stdin do canal, não na argv remota.

### Desde 0.3.8
- russh atualizado para 0.62.2 (piso de segurança ≥0.60.3).
- Banners humanos de tunnel fora do stdout do agente (JSON/non-TTY/quiet).
- Sem VPS ativa retorna sysexits 66 (`EX_NOINPUT`) via erro tipado.
- `cargo deny`: `yanked=deny`, ignore vazio; `multiple-versions=warn` para duplicatas transitivas.
- String de versão reporta `-dirty` quando a tree está suja.
- Suite residual completa `tests/gaps_v038_integration.rs`.

### Desde 0.3.9 (atual)
- Nível de tracing padrão é error (não info); `-v` habilita debug; `RUST_LOG` sobrescreve — stderr de JSON/tunnel fica limpo por padrão.
- Senha vazia ou ausente em VPS só-chave serializa como JSON `null` (não `"***"`); não vazia ainda mascara como `***`; texto humano em show usa "(não definida)" para vazio.
- `health-check` aceita override `--timeout <ms>` (alinhado ao exec).
- Docs de linha de produto alinhados a 0.3.9; suite residual `tests/gaps_v039_integration.rs`.


## Migração passo a passo
### Atualize o binário

```bash
cargo install ssh-cli --locked --force
ssh-cli --version
```

### Valide inventário e modo de segredos

```bash
ssh-cli secrets status --json
ssh-cli vps doctor --json
ssh-cli vps list --json
```

### Se ainda houver secrets em claro no disco
- No primeiro save com 0.3.6+, um `secrets.key` é auto-criado e novas gravações cifram.
- Para re-cifrar inventário plaintext:

```bash
ssh-cli secrets init   # se secrets.key ainda não existir
ssh-cli secrets reencrypt
```

- Faça backup offline de `config.toml` e `secrets.key`; perder a chave torna blobs ilegíveis.

### Adicione chaves em hosts só-chave

```bash
ssh-cli vps edit prod --key ~/.ssh/id_ed25519
```

### Recheque elevação (prefira stdin)

```bash
printf '%s' '...' | ssh-cli vps edit prod --sudo-password-stdin
ssh-cli sudo-exec prod "id"
ssh-cli su-exec prod "id"
```

### Atualize wrappers de agentes
- Passe `--timeout-ms` em tunnels.
- Trate `--maxChars` como limite de entrada, não de saída.
- Prefira `--password-stdin` para segredos.
- Trate erro de host-key TOFU antes de forçar replace.
- Espere valores cifrados com prefixo `sshcli-enc:v1:` no TOML.
- Espere tracing padrão error; defina `RUST_LOG` ou `-v` só ao diagnosticar; não parseie stderr como JSON.
- Trate senha vazia em list/show JSON como `null` em hosts só-chave.
- Pode passar `health-check --timeout <ms>` quando o timeout padrão do host for longo ou curto demais.
- Espere process exit `1` (com `remote_exit_code` no envelope JSON) quando o comando remoto falhar.
- Espere ausência de VPS ativa como exit 66.
- Espere banners de tunnel só em caminhos humanos/TTY, não no stdout JSON do agente.


## Campos de host / schema (estáveis até 0.3.9)

### Após 0.3.4+
- `timeout_ms`
- `max_command_chars`
- `max_output_chars`
- `key_path`
- `key_passphrase` (mascarado)
- `disable_sudo`
- `schema_version` 2

### Segredos at-rest (era 0.3.6; ainda atuais)
- Campos de senha/sudo/su/passphrase podem guardar blobs `sshcli-enc:v1:…`
- Fontes de master-key: `SSH_CLI_SECRETS_KEY`, `SSH_CLI_SECRETS_KEY_FILE`, keyring ou XDG `secrets.key`.

### Mascaramento (0.3.9)
- Senha vazia → JSON `null`; não vazia → string `***`.
- Texto humano em show ainda usa "(não definida)" para senha vazia.


## Compatibilidade
- Hosts TOML existentes carregam e normalizam defaults no read/save.
- Alias legado `--maxChars` mapeia para limite de entrada.
- Timeout default 60000 ms.
- Always-trust de host key saiu dos builds de release.
- Cifragem default está ligada; plaintext exige opt-out explícito (testes).
- Tracing padrão é error; prosa INFO não é esperada no stderr do agente.


## Rollback
- Reinstale versão anterior com pin exato se necessário.
- Mantenha export redacted via `vps export` antes de experimentos grandes.
- Se voltar abaixo de 0.3.6, blobs cifrados exigem a master-key correspondente ou re-export em plaintext ainda na 0.3.6+.
