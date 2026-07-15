# Guia de migração

> Passe de ssh-cli 0.3.3 (ou posterior) para **0.4.1** sem perder o inventário multi-host.

- Leia este documento em [inglês](MIGRATION.md).


## O que muda

### Desde 0.3.4 (paridade de automação SSH central)
- Grafo de crypto de install fixado para `cargo install --locked` funcionar (GAP-014).
- Auth aceita chaves privadas via `--key` / `key_path` (GAP-002).
- Semântica de `max_chars` dividida em `max_command_chars` e `max_output_chars` (GAP-004).
- `sudo-exec` empacota comandos com `sh -c` seguro (GAP-005).
- `su-exec` consome senha `su` armazenada (GAP-003).
- Escrita de config atômica com flock e mode 0600 (GAP-007).
- Host keys usam known_hosts TOFU (GAP-008).
- `tunnel` exige `--timeout-ms` (GAP-010).
- Schema version de registros novos é 2.
- Licença dual MIT OR Apache-2.0.

### Desde 0.3.5
- `vps export` atômico, abort remoto mais forte (TERM+KILL).
- Caminho AEAD opcional maduro; doctor reporta `secrets_at_rest`.
- JSON automático quando stdout não é TTY.

### Desde 0.3.6
- Cifragem at-rest padrão de segredos em `config.toml` (ChaCha20-Poly1305).
- Auto-cria XDG `secrets.key` (0o600) na primeira gravação de segredo.
- CLI `secrets status|init|reencrypt` (nunca imprime a master-key).
- Opt-out só para testes: `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`.
- Doctor: `secrets_key_file`, `secrets_plaintext_opt_out`.

### Desde 0.3.7
- Polimento de I/O para agentes: `--output-format` global em VPS CRUD, `health-check --json`, envelope de erro JSON, `--quiet` silencia sucesso humano.
- Tunnel `--timeout-ms` cobre connect SSH + loop.
- SCP valida arquivo local antes do connect; `vps remove` limpa `active` órfão.
- `su-exec --password-stdin`; conflitos clap para password/*_stdin.
- Exit remoto não-zero mapeia para exit de processo `1` com `remote_exit_code` no envelope JSON.
- Segredos longos sempre mascaram como `***` (sem vazamento de prefixo 12+4).
- Senha sudo/su no stdin do canal, não em argv remoto.

### Desde 0.3.8
- russh atualizado para 0.62.2 (piso de segurança ≥0.60.3).
- Banners humanos de tunnel fora do stdout do agente (JSON/non-TTY/quiet).
- Sem VPS ativa retorna sysexits 66 (`EX_NOINPUT`) via erro tipado.
- `cargo deny`: `yanked=deny`, ignore list vazia; `multiple-versions=warn` para duplicatas transitivas.
- String de versão reporta `-dirty` quando a working tree está suja.
- Suite residual completa `tests/gaps_v038_integration.rs`.

### Desde 0.4.1 (atual)
- **Patch AUD-POST:** secrets vazios nunca viram blob `sshcli-enc` no export redacted (EXP-001); deadline do tunnel pós-bind sai **0** (TUN-002); paridade de flags auth em `tunnel`/`health-check` (CLI-005/006); JSON SCP com `event: "scp-transfer"` (IO-009). Só aditivo — sem breaking.
- **Correção wire SCP (0.4.0):** crates.io **0.3.9** SCP quebrado. Atualize para **0.4.0+** (prefira **0.4.1**) antes de depender de `scp`.
- SCP é **somente arquivos regulares** (sem `-r` / sem SFTP). Use `--timeout` para arquivos grandes (cobre connect + transfer). JSON de sucesso via `--json` / `--output-format json` (`docs/schemas/scp-transfer.schema.json`).
- Download SCP grava `{path}.ssh-cli.partial` e faz rename atômico; mode/times aplicados no **partial** antes do rename.
- Upload SCP faz stream em blocos de **32 KiB** (sem `fs::read` do arquivo inteiro na RAM).
- Preserve mtime/mode bidirecional (remoto `scp -tp` / `-fp`; parse de `T` + mode `C`).
- Paridade de flags SCP com exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json`.
- Falhas de `scp --json` emitem **envelope de erro JSON em stderr** (`exit_code`, `message`) — paridade com tunnel (IO-007b).
- `tunnel --json` emite um objeto stdout `event: "tunnel_listening"` após o bind local (`docs/schemas/tunnel-listening.schema.json`); ainda exige `--timeout-ms`.
- Tracing default error (não info); `-v` ativa debug; `RUST_LOG` sobrescreve — stderr JSON/tunnel limpo por omissão.
- Senha vazia ou ausente em VPS só-chave serializa como JSON `null` (não `"***"`); não vazia ainda mascara como `***`; texto humano em show usa "(não definida)" para vazio.
- `health-check` aceita override `--timeout <ms>` (alinhado ao exec).
- Docs de product line alinhados a **0.4.1** + inventário AUD-POST (`gaps.md` / suite `tests/gaps_v041_integration.rs`); suites `tests/gaps_v039_integration.rs` + `tests/gaps_v040_integration.rs` + **`gaps_v041`**; e2e oficial **E01–E14** (E10–E14 cobrem SCP).


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

### Se ainda houver segredos plaintext em disco
- No primeiro save com 0.3.6+, um `secrets.key` é auto-criado e novas gravações cifram.
- Para re-cifrar inventário plaintext existente:

```bash
ssh-cli secrets init   # se secrets.key ainda não existir
ssh-cli secrets reencrypt
```

- Faça backup offline de `config.toml` e `secrets.key`; perder a chave torna blobs cifrados ilegíveis.

### Adicione chaves a hosts só-chave

```bash
ssh-cli vps edit prod --key ~/.ssh/id_ed25519
```

### Revalide segredos de elevação (prefira stdin)

```bash
printf '%s' '...' | ssh-cli vps edit prod --sudo-password-stdin
ssh-cli sudo-exec prod "id"
ssh-cli su-exec prod "id"
```

### Atualize wrappers de agentes
- Passe `--timeout-ms` em tunnels.
- Em `tunnel --json`, aguarde `event == "tunnel_listening"` antes de usar a porta local.
- **TUN-002:** após `tunnel_listening`, o deadline one-shot pós-bind sai com exit **0** (não trate 74 como falha se o bind já foi sinalizado). Timeout pré-bind permanece 74.
- **EXP-001:** em `vps export` redacted, não espere nem parseie `sshcli-enc:` para secrets vazios — vazios serializam como `""`.
- **IO-009:** parseie sucesso SCP com `docs/schemas/scp-transfer.schema.json` incluindo `event: "scp-transfer"` obrigatório.
- **CLI-005:** `tunnel` aceita `--password-stdin`, `--key-passphrase` / `--key-passphrase-stdin` (além de `--key`).
- **CLI-006:** `health-check` aceita `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`.
- Em falha de `scp`/`tunnel` com `--json`, parseie o envelope de erro em stderr (não prosa humana).
- Trate SCP como somente arquivos regulares; não envie árvores de diretório.
- Re-teste transferências após sair do **0.3.9** (SCP daquela release não era confiável).
- Se veio de **0.4.0**: export redacted podia mostrar ciphertext falso de senha vazia; tunnel podia emitir `ok:true` e sair 74 — atualize wrappers e o binário para **0.4.1**.
- Trate `--maxChars` como limite de entrada, não de saída.
- Prefira `--password-stdin` para segredos.
- Trate erros de mismatch de host-key antes de forçar replace.
- Espere valores cifrados em `config.toml` com prefixo `sshcli-enc:v1:` (exceto export redacted de secret vazio).
- Espere tracing default error; defina `RUST_LOG` ou `-v` só ao diagnosticar; não parseie stderr como JSON de sucesso.
- Trate senha vazia em list/show JSON como `null` em hosts só-chave.
- Pode passar `health-check --timeout <ms>` quando o timeout padrão do host for longo ou curto demais.
- Espere exit de processo `1` (com `remote_exit_code` no envelope JSON) quando o comando remoto falhar.
- Espere sem VPS ativa como exit 66.
- Espere banners de tunnel só em caminhos humanos/TTY, não no stdout JSON do agente.


## Campos de host / schema (estáveis até 0.4.1)

### Após 0.3.4+
- `timeout_ms`
- `max_command_chars`
- `max_output_chars`
- `key_path`
- `key_passphrase` (mascarado)
- `disable_sudo`
- `schema_version` 2

### Segredos at-rest (era 0.3.6; ainda atuais)
- Campos password/sudo/su/passphrase podem armazenar blobs `sshcli-enc:v1:…`.
- Fontes da master-key: `SSH_CLI_SECRETS_KEY`, `SSH_CLI_SECRETS_KEY_FILE`, keyring ou XDG `secrets.key`.

### Mascaramento (0.4.0)
- Senha vazia → JSON `null`; não vazia → string `***`.
- Texto humano em show ainda usa "(não definida)" para senha vazia.

### Eventos de transfer / tunnel (0.4.0 / 0.4.1)
- JSON de sucesso SCP inclui `event: "scp-transfer"` obrigatório (IO-009, 0.4.1).
- Tunnel continua emitindo `event: "tunnel_listening"` após bind.
- Sucesso SCP: `docs/schemas/scp-transfer.schema.json`
- Tunnel listening: `docs/schemas/tunnel-listening.schema.json`
- Falhas em modo JSON: `docs/schemas/error-envelope.schema.json` em stderr


## Notas de compatibilidade
- Hosts TOML existentes carregam e migram defaults de campos nos caminhos de leitura/gravação.
- Alias legado `--maxChars` mapeia para limite de entrada de comando.
- Timeout padrão é 60000 ms para automação de agentes.
- Comportamento always-trust de host key sumiu em builds de release.
- Cifragem padrão ligada; plaintext exige env de opt-out explícito (testes).
- Tracing padrão é error; prosa INFO não é esperada no stderr do agente.
- SCP permanece file-only por design em 0.4.0+ (ainda verdade em 0.4.1; não é limitação temporária).


## Rollback
- Reinstale versão anterior com pin exato se necessário.
- Mantenha export mascarado via `vps export` antes de experimentos grandes.
- Se voltar abaixo de 0.3.6, blobs cifrados exigem a master-key correspondente ou re-export em plaintext ainda na 0.3.6+.
- Se voltar para **0.3.9**, não espere wire SCP funcional (atualize de novo para 0.4.0+ para transferências).
