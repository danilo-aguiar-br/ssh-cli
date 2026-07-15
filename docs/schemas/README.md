# JSON Schemas Index

## English
- This directory versions machine-readable JSON contracts for ssh-cli stdout payloads (**0.3.6**).
- Validate agent parsers against these schemas before treating fields as stable.
- `vps-list.schema.json` contracts `ssh-cli vps list --json`.
- `vps-show.schema.json` contracts `ssh-cli vps show <name> --json`.
- `vps-doctor.schema.json` contracts `ssh-cli vps doctor --json` including `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out`, and `telemetry: false`.
- `exec.schema.json` contracts `ssh-cli exec ... --json`.
- `sudo-exec.schema.json` contracts `ssh-cli sudo-exec ... --json`.
- `su-exec.schema.json` contracts `ssh-cli su-exec ... --json`.
- `health-check.schema.json` contracts `ssh-cli health-check --json`.
- `error-envelope.schema.json` documents exit-code oriented failure semantics for agents.
- Secrets in list/show payloads are masked strings, never raw credentials.
- There is no schema for `secrets status` key material (command never emits the master key); treat status JSON as non-sensitive metadata only.
- `telemetry` in doctor output is always false.

## Português Brasileiro
- Este diretório versiona contratos JSON legíveis por máquina para payloads stdout do ssh-cli (**0.3.6**).
- Valide parsers de agentes contra estes schemas antes de tratar campos como estáveis.
- `vps-list.schema.json` cobre `ssh-cli vps list --json`.
- `vps-show.schema.json` cobre `ssh-cli vps show <name> --json`.
- `vps-doctor.schema.json` cobre `ssh-cli vps doctor --json` incluindo `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out` e `telemetry: false`.
- `exec.schema.json` cobre `ssh-cli exec ... --json`.
- `sudo-exec.schema.json` cobre `ssh-cli sudo-exec ... --json`.
- `su-exec.schema.json` cobre `ssh-cli su-exec ... --json`.
- `health-check.schema.json` cobre `ssh-cli health-check --json`.
- `error-envelope.schema.json` documenta semântica de falha orientada a exit codes para agentes.
- Segredos em list/show são strings mascaradas, nunca credenciais cruas.
- Não há schema de material de chave de `secrets status` (o comando nunca emite a master-key); trate o JSON de status como metadado não sensível.
- `telemetry` no doctor é sempre false.
