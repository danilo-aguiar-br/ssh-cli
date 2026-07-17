# Security Policy

- Read this document in [Portuguese (pt-BR)](SECURITY.pt-BR.md).

## Supported Versions
- The table below lists which ssh-cli versions currently receive security patches.
- Users on unsupported lines must upgrade to the current release line.

| Version | Status | Security Patches |
| --- | --- | --- |
| 0.5.x | Supported | Yes, **current line** |
| 0.4.x | Supported | Critical / high when feasible; prefer upgrade to **0.5.x** |
| 0.3.x | Limited | Critical only when feasible; crates.io **0.3.9** SCP wire was inoperant — upgrade to **0.5.1+** for transfers |
| 0.2.x | Limited | Critical fixes only when feasible |
| 0.1.x | Unsupported | No patches |
| < 0.1 | Unsupported | No patches |

## Reporting a Vulnerability
- Report security issues through GitHub Security Advisories in the public `ssh-cli` repository as the preferred private channel.
- Use email at daniloaguiarbr@proton.me only as fallback when GitHub private reporting is unavailable.
- Never open a public GitHub issue, pull request, or discussion for security-related reports.
- Include a minimal reproduction, affected versions, and expected versus actual behavior.
- Include environment details such as OS, architecture, and rustc version.
- Include a CVSS 3.1 severity estimate when possible to accelerate triage.
- Redact live credentials from every attachment and log excerpt.


## Response SLA
- Triage of every advisory starts within 72 business hours of submission.
- Initial acknowledgment is sent within that same 72-hour window.
- You receive a case identifier and an assigned maintainer contact.
- Progress updates are shared at least every 7 days until resolution or public disclosure.


## Fix SLA by CVSS Severity
- Critical severity (CVSS 9.0 to 10.0) receives a patch within 7 calendar days of validated triage.
- High severity (CVSS 7.0 to 8.9) receives a patch within 14 calendar days of validated triage.
- Medium severity (CVSS 4.0 to 6.9) receives a patch within 30 calendar days of validated triage.
- Low severity (CVSS 0.1 to 3.9) receives a patch within 90 calendar days of validated triage.
- Released fixes include a CHANGELOG entry and a GitHub Security Advisory when the affected line is still supported.


## Disclosure Policy
- Coordinated disclosure is the default for validated vulnerabilities.
- Public disclosure is delayed until a fixed release is available or the Fix SLA window expires with documented reason.
- Researchers may publish after mutual agreement or after the coordinated window ends.


## Security Update Policy
- Security fixes land on the current minor line first.
- Backports to older minor lines happen only when impact and effort justify the work.
- Users must upgrade with `cargo install ssh-cli --locked --force` after a security release.


## Hall of Fame
- Security researchers who request public credit are listed here after coordinated disclosure completes.
- The list starts empty for the current ownership line under `danilo-aguiar-br`.


## Best Practices for Users
- Prefer private key authentication over password authentication when the host allows it.
- Prefer `--password-stdin`, `--sudo-password-stdin`, and `--su-password-stdin` over argv secrets (password-on-argv emits a stderr warning on **0.5.1+**).
- Prefer stdin password flags for agent runs; avoid embedding live secrets in shell history.
- **Default at-rest encryption** (ChaCha20-Poly1305): on first secret write, auto-creates `secrets.key` (0o600) next to `config.toml` unless you opt out.
- Prefer CLI flags over env for secrets control: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (env layers still work when flags are unset).
- Key resolution order: CLI flags → `SSH_CLI_SECRETS_KEY` → `SSH_CLI_SECRETS_KEY_FILE` → keyring (`SSH_CLI_USE_KEYRING=1`) → XDG `secrets.key`.
- CLI: `ssh-cli secrets status|init|reencrypt` (never prints the master key); `--json` emits `secrets-init` / `secrets-reencrypt` without key material.
- Opt-out for tests only: `--allow-plaintext-secrets` or `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`.
- **Never** log the master key, passwords, or decrypted secrets.
- `--include-secrets` to pipe/non-TTY requires `-o`/`--output` or `--i-understand-secrets-on-stdout` (guard against accidental secret dump on stdout).
- Keep `config.toml` mode `0600` and restrict backup locations.
- Review TOFU host-key change errors before using `--replace-host-key`.
- Never commit host registries that include live secrets.
- Never commit local MCP sidecars (e.g. `.setting.cyber/`), Grok MCP config (`~/.grok/config.toml`), XDG `config.toml` / `secrets.key` / `known_hosts`, or E2E env files into the repository.
- Real-SSH E2E must keep credentials outside the tree (`SSH_CLI_E2E_*` env or `$HOME/.grok/config.toml`); the script refuses grok configs under the repo root.
- Demo passwords in public docs are placeholders only (e.g. `demo-password-not-real`); never reuse them on live hosts.
- Disable elevation with `--disable-sudo` when a workflow must not escalate.
- Run one-shot commands only; never expect a long-lived SSH daemon from this CLI.
- Install with `--locked` to avoid accidental crypto re-resolve drift.
- Prefer current **0.5.1+** for the supply-chain floor (russh 0.62.2) and for a working SCP wire (crates.io **0.3.9** SCP was inoperant).
- Historical honesty: **0.4.1** fixed empty-secret redacted export (never `sshcli-enc:` of empty) and tunnel post-bind exit 0.
- Default redacted `vps export` clears secrets; empty secrets must serialize as empty strings, never encrypted `sshcli-enc:` blobs of empty values (0.4.2 EXP-001).
