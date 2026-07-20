# Cross Platform

> Escape OS-specific SSH glue with one portable Rust binary.

- Read this document in [Portuguese (pt-BR)](CROSS_PLATFORM.pt-BR.md).
- Product line: 0.5.2.


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
| macOS | Supported | arm64 + x86_64; may need Gatekeeper quarantine removal |
| Windows | Supported | UTF-8 CP 65001 + VT processing at boot; ProjectDirs config |
| WSL1 / WSL2 | Supported | Detected via `WSL_*` / `/proc/version`; treat as Linux |
| Containers | Supported | Mount config dir or pass `--config-dir`; doctor reports `runtime.is_container` |
| Termux (Android) | Best-effort | Detected via `TERMUX_*`; bionic libc when target available |
| WASM / WASI | Not shipped | `russh` needs real sockets; not a product target |
| Browser automation | N/A | No Chrome/chromedriver discovery (SSH-only product) |


## Crypto / transport (G-TLS)
- Same crypto stack on every supported OS: **SSH-2** via `russh` + **aws-lc-rs** (not TLS/HTTPS, not OpenSSL, not `native-tls`, not product `rustls`).
- Host keys: TOFU file under the platform config dir (`directories` / XDG on Linux).
- SSH channel compression is **`none` only** (no zlib) on all platforms.
- Product release gates for this policy are **local** (`cargo deny`, residual tests) — not a required cloud CI workflow.


## Linux
- Prefer `cargo install ssh-cli --locked` into `~/.cargo/bin`.
- Expect XDG config under `~/.config/ssh-cli/` by default.
- Ensure mode 0600 after first save of `config.toml` and `secrets.key`.
- Default at-rest encryption stores blobs in `config.toml`; keep `secrets.key` offline-backed up.


## macOS
- Same cargo install path as Linux.
- Clear quarantine with `xattr -d com.apple.quarantine` when Gatekeeper blocks the binary.
- Expect config under the macOS application support/project dirs resolved by `directories` 6.
- Keyring backend for the primary-key is optional via `--use-keyring` after `secrets init --keyring`.


## Windows
- Install through Rustup and cargo on a supported toolchain (MSRV 1.85.0).
- At process start the binary sets console code page **65001 (UTF-8)** and enables
  **virtual terminal processing** so ANSI colors work under conhost / PowerShell 5.1.
- Use PowerShell completions from `ssh-cli completions powershell`.
- Prefer key files with explicit paths instead of relying on Unix home shortcuts.
- Config/project dirs come from `directories`; use `vps doctor --json` to see the winner.
- Local path components are capped at 255 bytes; total path approaching legacy
  `MAX_PATH` (260) is rejected unless the `\\?\` extended prefix is used.
- VPS registry names reject Windows reserved devices (`CON`, `NUL`, `COM1`, …).


## Containers
- Copy the binary into distroless or distro images without Node.
- Persist config dir (or pass `--config-dir`) for multi-run host memory (`config.toml`, `known_hosts`, `secrets.key`, `active`).
- Keep one-shot semantics; do not wrap the CLI as a long-lived sidecar unless tunnel timeout is set.
- Never bake live secrets or `secrets.key` into image layers.
- Runtime markers (`/.dockerenv`, `/run/.containerenv`, `KUBERNETES_SERVICE_HOST`,
  `container=`) surface as `runtime.is_container` in `vps doctor --json`.


## Runtime diagnostics
- `ssh-cli vps doctor --json` embeds a `runtime` object:
  `os`, `arch`, `is_wsl`, `is_container`, `is_ci`, `is_termux`, `sandbox`
  (`flatpak` | `snap` | null).
- Flatpak/Snap host installs emit a **warning** at boot (filesystem/keyring may differ).
- Detection never shells out (`uname`, `systemd-detect-virt` are not used).


## External processes (G-PROC)
- **Runtime product code never spawns local children.** SSH, SCP, and tunnels use
  pure Rust (`russh`) — no OpenSSH `ssh`/`scp`/`ssh-keygen` on the agent host.
- Remote elevation packs `sudo`/`su` + `sh -c` **on the target host** via the SSH
  channel only (quoted; passwords on channel stdin). That is not local `Command`.
- Build-time optional: `git` in `build.rs` for short HEAD (falls back to env /
  `.commit_hash` / `unknown`). Explicit `Stdio` null/piped; no shell.
- Test-time optional: `ssh-keygen` fixtures for real OpenSSH keys; skip if missing.
- Toolchain MSRV **1.85.0** ≥ **1.77.2** (CVE-2024-24576 BatBadBut). Product does
  not invoke `.bat`/`.cmd`. Job Objects / process groups for local trees: **N/A**.
- Remote commands reject **NUL** bytes before exec packing; multi-line CR/LF allowed.


## Shell Support
- Completions via `clap_complete`: **Bash, Zsh, Fish, PowerShell, Elvish**
  (`ssh-cli completions <shell>`).
- Nushell is not in the default `clap_complete::Shell` enum; generate via external
  tooling if needed.
- Prefer explicit argv arrays in agent runtimes over shell string eval.
- Prefer stdin secret flags over embedding passwords in shell history.


## File Paths and XDG
- Resolve the winner with `ssh-cli vps doctor --json` (includes `secrets_*` fields).
- Override only in tests via `--config-dir` (product does not read `SSH_CLI_HOME`).
- Keep `known_hosts`, `active`, and `secrets.key` as sibling files of `config.toml`.
- Atomic writes + flock protect concurrent one-shot processes on the same config.


## SCP portability
- SCP is regular files only on every platform (no recursive directory transfer). Use `sftp upload|download --recursive` for directory trees (no symlink follow).
- Failed or in-progress downloads use sibling path ending in `.ssh-cli.partial`, then rename into place (platform-agnostic atomic pattern).
- Upload streams in 32 KiB chunks on every OS (avoids full-file RAM load).
- mtime/mode preserve follows OpenSSH-style remote `-p` / `T` line; on Unix local permissions APIs apply modes; on Windows permission bits may not match Unix octal semantics — do not assume full POSIX ACL fidelity.
- Real-SSH matrix E01–E16 (E10–E14 SCP) in `scripts/e2e_real_ssh.sh` is primarily validated on Linux hosts; prefer local `sshd` / throwaway VPS. Never run auth-failure storms on production hosts (fail2ban bans).


## Performance by Target
- Linux cold start is the baseline under 100 ms target.
- musl builds may trade allocator characteristics; enable `musl-allocator` when needed.
- Network RTT dominates remote operations on every OS.


## Agents Validated per Platform
- Linux hosts are the primary validation surface for agent subprocess runs.
- macOS and Windows follow the same CLI contract and JSON schemas.
- Root discovery aliases work on every OS: `ssh-cli doctor` (alias of `vps doctor`) and `ssh-cli schema` (embedded catalog / one schema body).
- JSON contracts (`scp-transfer` event, `tunnel_listening`, auth flags for tunnel/health) are identical on every OS; see AGENTS.md and docs/schemas/.
- Tunnel `--bind` defaults to `127.0.0.1` (loopback) on every platform; override only when intentionally exposing the listener.
- Container agents must preserve exit codes and stdout/stderr separation.
- Default tracing is error-level so agent stderr stays free of INFO prose unless `-v` is set (ambient `RUST_LOG` is ignored).
- Parse machine contracts from stdout only; treat stderr tracing as non-contract logs; JSON error envelopes use stderr when JSON mode is active.
- Real SSH E2E helpers live in `scripts/e2e_real_ssh.sh` (anti-leak; local only; E01–E16; never production auth-failure storms / fail2ban policy).
- `ssh-cli --version` stamp (`Cargo` version + git hash + optional `-dirty`) is OS-agnostic.


## Multi-OS local matrix (G-E2E-18)
- Product code: `src/platform/{linux,macos,windows}.rs` modules — multi-OS behavior is local-only.
- Local multi-arch binaries via `scripts/dist_multiarch.sh` (and `Cross.toml`) when cross toolchains / Docker are installed.
- **No required cloud GitHub Actions product CI** — maintainers validate on Linux primarily; check path length / agent socket notes on macOS and Windows before tagging a release.
- Do **not** reintroduce `.github/workflows` for product CI (policy: one-shot CLI, no cloud CI product).
