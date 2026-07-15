# Guia de migração

> Passe de ssh-cli 0.3.3 (ou 0.3.4/0.3.5) para **0.3.6** sem perder o inventário multi-host.

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

### Desde 0.3.6 (atual)
- **Cifragem at-rest por padrão** dos segredos no `config.toml` (ChaCha20-Poly1305).
- Auto-cria `secrets.key` XDG (0o600) na primeira gravação de segredo.
- CLI `secrets status|init|reencrypt` (nunca imprime a master-key).
- Opt-out só para testes: `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`.
- Doctor: `secrets_key_file`, `secrets_plaintext_opt_out`.


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
- No primeiro save com 0.3.6, um `secrets.key` é auto-criado e novas gravações cifram.
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


## Campos de host / schema (0.3.6)

### Após 0.3.4+
- `timeout_ms`, `max_command_chars`, `max_output_chars`
- `key_path`, `key_passphrase` (mascarado)
- `disable_sudo`, `schema_version` 2

### Segredos at-rest (0.3.6)
- Campos de senha/sudo/su/passphrase podem guardar blobs `sshcli-enc:v1:…`
- Fontes de master-key: env, arquivo, keyring ou XDG `secrets.key`


## Compatibilidade
- Hosts TOML existentes carregam e normalizam defaults no read/save.
- Alias legado `--maxChars` mapeia para limite de entrada.
- Timeout default 60000 ms .
- Always-trust de host key saiu dos builds de release.
- Cifragem default está ligada; plaintext exige opt-out explícito (testes).


## Rollback
- Reinstale versão anterior com pin exato se necessário.
- Mantenha export redacted via `vps export` antes de experimentos grandes.
- Se voltar abaixo de 0.3.6, blobs cifrados exigem a master-key correspondente ou re-export em plaintext ainda na 0.3.6.
