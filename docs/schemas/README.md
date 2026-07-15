# JSON Schemas Index

## English
- This directory versions machine-readable JSON contracts for ssh-cli stdout/stderr payloads (**0.4.2**).
- Validate agent parsers against these schemas before treating fields as stable.
- `vps-list.schema.json` contracts `ssh-cli vps list --json`.
- `vps-show.schema.json` contracts `ssh-cli vps show <name> --json`.
- `vps-list.schema.json` reuses the vps-show item schema via `$ref` (`items.$ref` → `vps-show.schema.json`).
- `vps-show` / `vps-list` schemas allow `password` as JSON `null` or the masked string `***` (GAP-SSH-JSON-001 / **0.4.0**): empty/key-only hosts serialize as `null`; non-empty is never a raw credential.
- `vps-doctor.schema.json` contracts `ssh-cli vps doctor --json` including `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out`, and `telemetry: false`.
- `exec.schema.json` contracts `ssh-cli exec ... --json`.
- `sudo-exec.schema.json` contracts `ssh-cli sudo-exec ... --json`.
- `su-exec.schema.json` contracts `ssh-cli su-exec ... --json`.
- `health-check.schema.json` contracts `ssh-cli health-check --json`.
- Optional CLI `--timeout` on `health-check` does not change response schema fields.
- `scp-transfer.schema.json` contracts `ssh-cli scp upload|download --json` **success on stdout** (regular files only; no directories / no `-r` / no SFTP); required field `event: "scp-transfer"` (IO-009 / **0.4.2**).
- `tunnel-listening.schema.json` contracts the post-bind stdout event for `ssh-cli tunnel ... --json` (`event: "tunnel_listening"`). After `tunnel_listening`, one-shot post-bind deadline is process exit **0** (not a schema field; see AGENTS / TUN-002). Pre-bind timeout remains **74**.
- `vps-export.schema.json` contracts `ssh-cli vps export --json` (`event: "vps-export"`, redacted by default; GAP-SSH-UX-001 / **0.4.2**).
- `error-envelope.schema.json` contracts **stderr** failure payloads when JSON errors mode is active (`--json` / global `--output-format json` / effective JSON on scp and tunnel): fields `exit_code`, `message`, optional `remote_exit_code`.
- Secrets in list/show payloads: empty password is JSON `null` (key-only hosts); non-empty password is the masked string `***`, never raw credentials.
- There is no schema for `secrets status` key material (command never emits the master key); treat status JSON as non-sensitive metadata only.
- `telemetry` in doctor output is always false.

## Português Brasileiro
- Este diretório versiona contratos JSON legíveis por máquina para payloads stdout/stderr do ssh-cli (**0.4.2**).
- Valide parsers de agentes contra estes schemas antes de tratar campos como estáveis.
- `vps-list.schema.json` cobre `ssh-cli vps list --json`.
- `vps-show.schema.json` cobre `ssh-cli vps show <name> --json`.
- `vps-list.schema.json` reutiliza o schema de item de vps-show via `$ref` (`items.$ref` → `vps-show.schema.json`).
- Os schemas `vps-show` / `vps-list` permitem `password` como JSON `null` ou a string mascarada `***` (GAP-SSH-JSON-001 / **0.4.0**): hosts vazios/só-chave serializam como `null`; não vazio nunca é credencial crua.
- `vps-doctor.schema.json` cobre `ssh-cli vps doctor --json` incluindo `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out` e `telemetry: false`.
- `exec.schema.json` cobre `ssh-cli exec ... --json`.
- `sudo-exec.schema.json` cobre `ssh-cli sudo-exec ... --json`.
- `su-exec.schema.json` cobre `ssh-cli su-exec ... --json`.
- `health-check.schema.json` cobre `ssh-cli health-check --json`.
- O CLI opcional `--timeout` em `health-check` não altera os campos do schema de resposta.
- `scp-transfer.schema.json` cobre sucesso em **stdout** de `ssh-cli scp upload|download --json` (somente arquivos regulares; sem diretórios / sem `-r` / sem SFTP); o campo `event: "scp-transfer"` é **obrigatório** (GAP-SSH-IO-009 / **0.4.2**).
- `tunnel-listening.schema.json` cobre o evento pós-bind em stdout de `ssh-cli tunnel ... --json` (`event: "tunnel_listening"`). Após `tunnel_listening`, o deadline one-shot pós-bind é exit **0** do processo (não é campo de schema; ver AGENTS / TUN-002). Timeout pré-bind permanece **74**.
- `error-envelope.schema.json` cobre payloads de falha em **stderr** quando o modo de erros JSON está ativo (`--json` / `--output-format json` global / JSON efetivo em scp e tunnel): campos `exit_code`, `message`, opcional `remote_exit_code`.
- Segredos em list/show: senha vazia é JSON `null` (hosts só-chave); senha não vazia é a string mascarada `***`, nunca credenciais cruas.
- Não há schema de material de chave de `secrets status` (o comando nunca emite a master-key); trate o JSON de status como metadado não sensível.
- `telemetry` no doctor é sempre false.
