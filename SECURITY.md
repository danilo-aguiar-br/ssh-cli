# Security Policy

- Read this document in [Portuguese (pt-BR)](SECURITY.pt-BR.md).

## Supported Versions
- The table below lists which ssh-cli versions currently receive security patches.
- Users on unsupported lines must upgrade to the current release line.

| Version | Status | Security Patches |
| --- | --- | --- |
| 0.5.x | Supported | Yes, **current line** |
| 0.4.x | Supported | Critical / high when feasible; prefer upgrade to **0.5.x** |
| 0.3.x | Limited | Critical only when feasible; crates.io **0.3.9** SCP wire was inoperant — upgrade to **0.5.2+** for transfers |
| 0.2.x | Limited | Critical fixes only when feasible |
| 0.1.x | Unsupported | No patches |
| < 0.1 | Unsupported | No patches |

## Reporting a Vulnerability
- Report security issues through GitHub Security Advisories in the public `ssh-cli` repository as the preferred private channel.
- Use email at daniloaguiarbr@proton.me only as fallback when GitHub private reporting is unavailable.
- Never open a public GitHub issue, pull request, or discussion for security-related reports.
- Include a minimal reproduction, affected versions, and expected versus actual behavior.
- Include environment details such as OS, architecture, and rustc version.
- Include a CVSS **v4.0** severity estimate when possible (CVSS 3.1 accepted as fallback) to accelerate triage.
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


## Threat Model (product — G-SEC-10)
- **Trust boundary:** CLI argv, env, XDG files (`config.toml`, `secrets.key`, `known_hosts`), and the remote SSH server are **untrusted** inputs. Only the local process identity and OS keyring (when enabled) are treated as higher-trust material.
- **Assets:** SSH passwords / key passphrases / sudo-su secrets (memory + at-rest), host-key fingerprints (TOFU), primary AEAD key (`secrets.key` / keyring), and the ability to run remote shell / SCP / local port forwards.
- **Adversaries considered:** (1) hostile CLI/agent argv and env; (2) malicious or MITM SSH servers (host-key change); (3) co-tenant local users reading world-readable files or process listings; (4) compromised transitive crates (supply chain).
- **Out of scope:** multi-tenant SaaS isolation, browser XSS/CSRF, container escape, kernel Spectre mitigations beyond normal process isolation, and long-lived daemon attack surface (this CLI is **one-shot**).
- **Controls:** typed validation at CLI/import/path boundaries; `SecretString` + zeroize; ChaCha20-Poly1305 at rest; TOFU host keys with constant-time fingerprint compare; shell packing with single-quote escape and secrets on channel stdin; no local shell-out for SSH; `deny.toml` + **local** `cargo deny check` (product gates do not require GitHub Actions); release `overflow-checks`; documented `unsafe` only at OS console/signal edges.
- **Review trigger:** re-read this model on any change that adds network protocols, long-lived processes, new secret stores, or `unsafe` outside OS FFI wrappers.

## Transport & crypto policy (G-TLS)
- **Default path:** product network transport is **SSH-2** via `russh` on plain TCP. Wire crypto uses the **aws-lc-rs** backend (`ssh-real`). Host authentication is **TOFU** (`known_hosts` under XDG).
- **Optional SSH-over-TLS:** when a VPS has `tls = true` (or `--tls` on `vps add`/`edit`), the client dials TCP, completes a **rustls** handshake (SNI + optional mTLS), then runs SSH on the TLS stream. Floor: **rustls ≥ 0.23.18** (never 0.23.13–0.23.17 Acceptor CVE window).
- **Provider:** binary `main` calls `CryptoProvider::install_default` with **`aws_lc_rs` only** (once, before the Tokio runtime). Libraries use `ClientConfig::builder` / process default — they never reinstall. Dual provider **`ring` is banned** in `deny.toml`.
- **Forbidden stacks:** no `native-tls`, OpenSSL, `libssh2-sys`, or product use of `ring`. PEM parse uses `rustls-pki-types` (not unmaintained `rustls-pemfile`).
- **mTLS:** client cert/key PEMs under XDG `tls/mtls/<name>/` (`ssh-cli tls mtls import|list|show|remove`) or absolute paths on the VPS record.
- **ACME:** Let's Encrypt via `instant-acme` (DNS-01, agent two-step: `tls acme issue --print-challenge` → set TXT → `tls acme complete`). Account + certs under XDG `tls/acme/` (0o600). Prefer staging with `--staging` for tests.
- SSH channel compression is **disabled** (client prefers `none` only) to reduce compression side-channels when secrets cross the channel.
- Release/supply-chain gates for this policy are **local scripts** (`cargo deny check`, `cargo tree`, residual tests) — not a required GitHub Actions workflow (G-TLS-11).


## STRIDE Map (critical components — G-SECDEV-03)

| Component | S | T | R | I | D | E | Primary controls |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CLI argv / env / stdin secrets | spoof agent args | tamper flags | — | info leak via argv | DoS huge stdin | elev via sudo-exec | typed clap + secret stdin → `SecretString` + size cap |
| XDG `config.toml` / `secrets.key` | forge host entry | rewrite secrets | — | read co-tenant | truncate/corrupt | — | 0o600 + flock + AEAD at-rest |
| TOFU `known_hosts` | MITM host | swap fingerprint | — | fingerprint oracle | lock starve | — | flock + constant-time compare + `--replace-host-key` explicit |
| SSH session (russh) | fake server | MITM / key change | — | channel sniff | hang / flood | remote shell | TOFU + timeouts + no local shell-out |
| Remote packing (`sh -c` sudo/su) | — | inject metachar | — | secret on argv | — | unintended elev | single-quote escape + password on channel stdin |
| Multi-host fan-out | — | reorder jobs | — | cross-host leak in logs | resource exhaustion | — | `Semaphore` budget + cancel + redacted JSON |
| Supply chain (deps) | typosquat | malicious crate | — | backdoor | build DoS | — | `deny.toml` + local `cargo deny` + `--locked` + lockfile TLS bans |

**Accepted residual risks (explicit):** (1) password-on-argv still parseable (warned; prefer stdin); (2) remote host after auth is fully trusted for the one-shot command the operator requested; (3) co-tenant with same UID can read process memory — OS isolation, not this CLI; (4) Spectre/side-channel CPU class threats out of scope for a one-shot userland tool.

## Best Practices for Users
- Prefer private key authentication over password authentication when the host allows it.
- Prefer `--password-stdin`, `--sudo-password-stdin`, and `--su-password-stdin` over argv secrets (password-on-argv emits a stderr warning on **0.5.2+**).
- Prefer stdin password flags for agent runs; avoid embedding live secrets in shell history.
- **Default at-rest encryption** (ChaCha20-Poly1305): on first secret write, auto-creates `secrets.key` (0o600) next to `config.toml` unless you opt out.
- Secrets control is CLI/XDG only: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, or XDG `secrets.key`.
- Key resolution order: CLI flags → keyring (when `--use-keyring`) → XDG `secrets.key`. `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` are **rejected fail-closed** (not a store).
- CLI: `ssh-cli secrets status|init|reencrypt` (never prints the master key); `--json` emits `secrets-init` / `secrets-reencrypt` without key material.
- Opt-out for tests only: `--allow-plaintext-secrets` (no env store).
- **Never** log the master key, passwords, or decrypted secrets.
- `--include-secrets` to pipe/non-TTY requires `-o`/`--output` or `--i-understand-secrets-on-stdout` (guard against accidental secret dump on stdout).
- Keep `config.toml` mode `0600` and restrict backup locations.
- Review TOFU host-key change errors before using `--replace-host-key`.
- Never commit host registries that include live secrets.
- Never commit local MCP sidecars (e.g. `.setting.cyber/`), Grok MCP config (`~/.grok/config.toml`), XDG `config.toml` / `secrets.key` / `known_hosts`, or E2E env files into the repository.
- Real-SSH E2E (G-E2E-05) must keep credentials outside the tree: prefer `--config-dir` / pre-registered hosts, or `$HOME/.grok/config.toml` via `--from-grok-config`; harness-only `SSH_CLI_E2E_*` is accepted by the script only (not product runtime). Offline / no lab → **SKIP** exit 0. The script refuses grok configs under the repo root.
- Demo passwords in public docs are placeholders only (e.g. `demo-password-not-real`); never reuse them on live hosts.
- Disable elevation with `--disable-sudo` when a workflow must not escalate.
- Run one-shot commands only; never expect a long-lived SSH daemon from this CLI.
- Install with `--locked` to avoid accidental crypto re-resolve drift.
- Prefer current **0.5.2+** for the supply-chain floor (russh 0.62.2), working SCP/SFTP wire, ACME permanent classification (`invalidContact` → exit **64**), and redacted export mask `***` (`FIXED_MASK`).
- Historical honesty: **0.4.1** fixed empty-secret redacted export (never `sshcli-enc:` of empty) and tunnel post-bind exit 0.
- Default redacted `vps export` clears secrets; empty secrets must serialize as empty strings, never encrypted `sshcli-enc:` blobs of empty values (0.4.2 EXP-001).
