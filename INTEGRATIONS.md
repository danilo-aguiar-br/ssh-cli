# Integrations

> Connect 10+ AI coding agents to remote servers with one-shot ssh-cli.

- Read this document in [Portuguese (pt-BR)](INTEGRATIONS.pt-BR.md).
- Pair this catalog with [docs/AGENTS.md](docs/AGENTS.md) and [skills/ssh-cli-en/SKILL.md](skills/ssh-cli-en/SKILL.md).


## Flag Aliases
### camelCase aliases implemented in clap (do not invent others)
- Use `--sudoPassword` as alias of `--sudo-password`.
- Use `--suPassword` as alias of `--su-password`.
- Use `--maxChars` as legacy alias mapping to `max_command_chars`.
- Use `--disableSudo` as alias of `--disable-sudo`.
- **No** camelCase aliases for `--config-dir`, `--output-format`, or `--no-color` — use the kebab-case forms exactly.


## New Flags by Version
### Track surface growth without reading source
- `0.5.1` **agent-first export/import + wire v3**: `vps export` body is **TOML by default** (TTY and pipe); JSON agent envelope only with `--json`; `vps import` accepts TOML (EN keys + PT aliases) **or** JSON `vps-export` envelopes; dual-read EN serialize + PT aliases; host **schema v3**; CLI flags `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (prefer over env); `secrets init|reencrypt --json` events `secrets-init` / `secrets-reencrypt`; auto primary-key emits `secrets-key-auto-created`; CRUD success paths use `emit_success` JSON events; `--include-secrets` to pipe/non-TTY requires `-o`/`--output` or `--i-understand-secrets-on-stdout`; tunnel `--bind` (default `127.0.0.1`); import `TomlDe` → exit **65**; `SshAuthentication` → **77**; SCP missing message `file not found: <path>` (exit **66**); timeout warn if `<1000` ms; password-on-argv stderr warn; doctor `secrets_plaintext_opt_out` is **bool**.
- `0.4.2` tunnel ephemeral port `local_port=0` reports OS-assigned port after bind (TUN-003); SCP remote missing → exit **66** (IO-010); `vps export --json` envelope `event: "vps-export"`; e2e E15/E16; suite `gaps_v042`.
- `0.4.1` SCP **regular files only** auth parity + scp event + export/tunnel fixes (inherits 0.4.0 wire; no `-r` / no SFTP); solid SCP wire (avoid crates.io **0.3.9** broken SCP); scp flags `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` → `docs/schemas/scp-transfer.schema.json` with required `event: "scp-transfer"` (IO-009); download writes `{path}.ssh-cli.partial` then rename; preserve mtime/mode bi-dir; upload streams 32 KiB; `tunnel --json` emits `tunnel_listening` after bind; scp JSON error envelope on stderr when `--json`; **EXP-001** redacted export empty secrets stay `""` (never `sshcli-enc:` of empty); **TUN-002** tunnel post-bind deadline exits **0** after `tunnel_listening` (pre-bind timeout still **74**); **CLI-005** tunnel auth stdin/passphrase parity; **CLI-006** health-check auth parity.
- `0.4.0` solid SCP wire protocol (fixes crates.io **0.3.9** broken SCP); file-only transfers; `tunnel --json` / `tunnel_listening`.
- `0.3.9` default tracing filter `error` (agent-first); empty password serializes as JSON `null` on key-only hosts; `health-check --timeout <ms>`; product-line docs audit.
- `0.3.8` russh 0.62.2; tunnel agent stdout clean; no-active-VPS exits `66` (`EX_NOINPUT`); `cargo deny` with `yanked=deny`.
- `0.3.7` `--output-format` on VPS CRUD; `health-check --json`; `--quiet`; JSON error envelope; tunnel timeout covers connect.
- `0.3.6` adds default at-rest encryption, `secrets status|init|reencrypt`, `SSH_CLI_ALLOW_PLAINTEXT_SECRETS`, doctor fields `secrets_key_file` / `secrets_plaintext_opt_out`, `scripts/e2e_real_ssh.sh`.
- `0.3.5` adds `--key-passphrase-stdin` runtime paths, auto JSON on non-TTY, doctor `secrets_at_rest`, residual atomic export and AEAD (then optional).
- `0.3.4` adds `--key`, `--key-passphrase`, `--password-stdin`, `--sudo-password-stdin`, `--su-password-stdin`, `--timeout-ms` (tunnel), `--disable-sudo`, `--description`, `--replace-host-key`, `max_command_chars`, `max_output_chars`, `vps doctor`, `vps export`, `vps import`, `su-exec`.
- `0.2.0` adds runtime `--password`, `--sudo-password`, `--timeout` overrides and camelCase aliases.
- Prefer **0.5.1+** for export/import roundtrip, wire schema v3, working SCP + `tunnel --json` / `--bind`, full SSH automation, default secret encryption, and clean supply-chain.


## Summary Table

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
- Install `ssh-cli` on the host PATH with `cargo install ssh-cli --locked`.
- Load [skills/ssh-cli-en/SKILL.md](skills/ssh-cli-en/SKILL.md) or the pt package.
- Register hosts once with `vps add` (prefer `--password-stdin`) then call `exec` per task.
- Prefer `--json` envelopes for structured tool results.
- Parse stdout only; default stderr is silent at tracing level `error` (set `RUST_LOG` only when debugging).
- Use `ssh-cli secrets status` / `vps doctor --json` as preflight for encryption and paths.


## Cursor
- Add a project rule that prefers `ssh-cli` over long-lived Node SSH processes.
- Keep credentials out of chat by using stored hosts and stdin secret flags.
- Parse JSON stdout only; default stderr is silent at tracing level `error` (ignore tracing unless you set `RUST_LOG`).


## Windsurf
- Invoke one-shot commands after host registration.
- Never keep a tunnel open without `--timeout-ms`.


## Codex CLI
- Treat non-zero exits as typed failures using the exit code table in README.
- Retry only on transient IO/timeout codes, never on auth or usage errors.


## OpenCode
- Use shell tool mode with explicit argv arrays.
- Avoid embedding passwords in prompt text; use registry or stdin.


## Aider
- Document host names in the repo without secrets.
- Call `ssh-cli exec <name> "..."` for remote ops during edit loops.


## Continue
- Map custom commands to `ssh-cli` subcommands with `--json`.
- Use `vps doctor --json` as a health preflight for agent sessions.


## Gemini CLI
- Prefer key auth and masked `vps show` for verification.
- Keep elevation disabled unless the task requires root.


## OpenHands
- Run inside the sandbox with network policy that allows only target hosts.
- Force bounded tunnels and short timeouts.


## Generic Shell
- Install completions with `ssh-cli completions <shell>`.
- Export `SSH_CLI_HOME` only for isolated test sandboxes.
