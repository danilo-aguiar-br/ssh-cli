# Cross Platform

> Escape OS-specific SSH glue with one portable Rust binary.

- Read this document in [Portuguese (pt-BR)](CROSS_PLATFORM.pt-BR.md).
- Product line: **0.5.0**.


## The Pain You Already Know
- Node daemon wrappers differ by host package manager and runtime version.
- Shell-only SSH scripts leak secrets into history and process lists.
- Path conventions diverge between Linux, macOS, and Windows config homes.
- Agents need one command surface that dies after each run everywhere.


## Support Matrix

| Platform | Status | Notes |
| --- | --- | --- |
| Linux gnu | Supported | Primary development target |
| Linux musl | Supported | Use `--features musl-allocator` when needed |
| macOS | Supported | May need Gatekeeper quarantine removal |
| Windows | Supported | Config uses platform project dirs |
| Containers | Supported | Mount or set `SSH_CLI_HOME` for persistence |


## Linux
- Prefer `cargo install ssh-cli --locked` into `~/.cargo/bin`.
- Expect XDG config under `~/.config/ssh-cli/` by default.
- Ensure mode 0600 after first save of `config.toml` and `secrets.key`.
- Default at-rest encryption stores blobs in `config.toml`; keep `secrets.key` offline-backed up.


## macOS
- Same cargo install path as Linux.
- Clear quarantine with `xattr -d com.apple.quarantine` when Gatekeeper blocks the binary.
- Expect config under the macOS application support/project dirs resolved by `directories` 6.
- Keyring backend for master-key is optional via `SSH_CLI_USE_KEYRING=1` after `secrets init --keyring`.


## Windows
- Install through Rustup and cargo on a supported toolchain (MSRV 1.85.0).
- Use PowerShell completions from `ssh-cli completions powershell`.
- Prefer key files with explicit paths instead of relying on Unix home shortcuts.
- Config/project dirs come from `directories`; use `vps doctor --json` to see the winner.


## Containers
- Copy the binary into distroless or distro images without Node.
- Persist config dir or set `SSH_CLI_HOME` for multi-run host memory (`config.toml`, `known_hosts`, `secrets.key`, `active`).
- Keep one-shot semantics; do not wrap the CLI as a long-lived sidecar unless tunnel timeout is set.
- Never bake live secrets or `secrets.key` into image layers.


## Shell Support
- bash, zsh, fish, and PowerShell completions are generated on demand.
- Prefer explicit argv arrays in agent runtimes over shell string eval.
- Prefer stdin secret flags over embedding passwords in shell history.


## File Paths and XDG
- Resolve the winner with `ssh-cli vps doctor --json` (includes `secrets_*` fields).
- Override only in tests via `--config-dir` or `SSH_CLI_HOME`.
- Keep `known_hosts`, `active`, and `secrets.key` as sibling files of `config.toml`.
- Atomic writes + flock protect concurrent one-shot processes on the same config.


## SCP portability
- SCP is **regular files only** on every platform (no recursive directory transfer; no SFTP subsystem).
- Failed or in-progress downloads use sibling path ending in **`.ssh-cli.partial`**, then rename into place (platform-agnostic atomic pattern).
- Upload streams in 32 KiB chunks on every OS (avoids full-file RAM load).
- mtime/mode preserve follows OpenSSH-style remote `-p` / `T` line; on Unix local permissions APIs apply modes; on Windows permission bits may not match Unix octal semantics — do not assume full POSIX ACL fidelity.
- Real-SSH matrix E10–E14 in `scripts/e2e_real_ssh.sh` is primarily validated on Linux hosts.


## Performance by Target
- Linux cold start is the baseline under 100 ms target.
- musl builds may trade allocator characteristics; enable `musl-allocator` when needed.
- Network RTT dominates remote operations on every OS.


## Agents Validated per Platform
- Linux hosts are the primary validation surface for agent subprocess runs.
- macOS and Windows follow the same CLI contract and JSON schemas.
- JSON contracts (`scp-transfer` event, `tunnel_listening`, auth flags for tunnel/health) are identical on every OS; see AGENTS.md and docs/schemas/.
- Container agents must preserve exit codes and stdout/stderr separation.
- Default tracing is error-level so agent stderr stays free of INFO prose unless `RUST_LOG` or `-v` is set.
- Parse machine contracts from stdout only; treat stderr tracing as non-contract logs; JSON error envelopes use stderr when JSON mode is active.
- Real SSH E2E helpers live in `scripts/e2e_real_ssh.sh` (anti-leak; local only; E01–E14).
