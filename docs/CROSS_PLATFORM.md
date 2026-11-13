# Cross Platform


> Escape OS-specific SSH glue with one portable Rust binary.

- Read this document in [Portuguese (pt-BR)](CROSS_PLATFORM.pt-BR.md)
- Product line: 0.5.6


## The Pain You Already Know
- Node daemon wrappers differ by host package manager and runtime version
- Shell-only SSH scripts leak secrets into history and process lists
- Path conventions diverge between Linux, macOS, and Windows config homes
- Agents need one command surface that dies after each run everywhere


## Support Matrix
### Targets
- Six of the eight target triples below are named by `Cross.toml`, `scripts/dist_multiarch.py` or `scripts/check_cross_targets.py`, `x86_64-unknown-linux-gnu` is the host build and `aarch64-apple-darwin` is listed only to say that nothing covers it
- A row claims only what a script in this repository covers
- Re-check the type-check rows with `python3 scripts/check_cross_targets.py`

| Target triple | Platform | Status | Covered by | Release binary size |
| --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | Linux x86_64 gnu | Supported, primary target | Host build, full test suite and `scripts/check_all_gates.py` | 9048976 bytes |
| `x86_64-unknown-linux-musl` | Linux x86_64 musl | Supported (cross build) | `Cross.toml` image and `scripts/dist_multiarch.py`, plus `--features musl-allocator` when needed | Not measured |
| `aarch64-unknown-linux-musl` | Linux arm64 musl | Supported (cross build) | `Cross.toml` image and `scripts/dist_multiarch.py` | Not measured |
| `aarch64-unknown-linux-gnu` | Linux arm64 gnu | Supported (cross build) | `Cross.toml` image and `scripts/dist_multiarch.py` | Not measured |
| `x86_64-pc-windows-msvc` | Windows x86_64 | Supported (type-checked) | Gate `cross-targets` with `--no-default-features` | Not measured |
| `aarch64-pc-windows-msvc` | Windows arm64 | Supported (type-checked) | Gate `cross-targets` with `--no-default-features` | Not measured |
| `x86_64-apple-darwin` | macOS Intel | Supported (type-checked) | Gate `cross-targets` with `--no-default-features` | Not measured |
| `aarch64-apple-darwin` | macOS Apple Silicon | Not covered by any gate | Nothing in this repository builds or checks it | Not measured |

- Binary size: the 9048976 bytes were measured with `stat` on the installed binary of an intermediate 0.5.6 build of the primary target, built with the size-minimised `release` profile, with sha256 `a4c2079731cbbd8dc4a0d212baa87e904d5d507ff458f6726de96599cd728536`
- The measurement is valid only for this host and this build
- Nothing in this tree builds the other seven targets, so their sizes are not measured and none is claimed
- Type-checked means `cargo check` passes, and the Windows and macOS binaries were never executed by these gates
- The `--no-default-features` build leaves out the `ssh-real` and `tls` features, so it type-checks without `russh`, `rustls` and `aws-lc-sys`, the crate that compiles C and would need a cross C toolchain
- No native Windows or macOS run is recorded by these gates

### Environments

| Environment | Status | Notes |
| --- | --- | --- |
| WSL1 / WSL2 | Supported | Detected via `WSL_DISTRO_NAME`, `WSL_INTEROP` or `/proc/version`, so treat it as Linux |
| Containers | Supported | Mount the config dir or pass `--config-dir`, and `doctor` reports `local.runtime.is_container` |
| Termux (Android) | Best-effort | Detected via `TERMUX_VERSION` or `TERMUX_APK_RELEASE` as `local.runtime.is_termux`, and no gate builds an Android target |
| WASM / WASI | Not shipped | `russh` needs real sockets, so it is not a product target |
| Browser automation | N/A | No Chrome/chromedriver discovery, because this is an SSH-only product |


## Crypto / transport (G-TLS)
- Same crypto stack on every supported OS: SSH-2 via `russh` 0.65 on plain TCP with the `aws-lc-rs` backend
- The optional SSH-over-TLS path, where TLS is Transport Layer Security, covers mutual TLS (mTLS) and the Automatic Certificate Management Environment (ACME), and uses `rustls` with the `aws_lc_rs` provider only
- `Cargo.toml` asks for `rustls` 0.23.25, and `ssh-cli tls provider` prints the security floor as `min_rustls` 0.23.18
- Building from source needs a C compiler on every OS, because `aws-lc-sys` 0.47 compiles the AWS-LC C and assembly sources
- On Windows x86 and x86_64 that build also needs NASM, unless `AWS_LC_SYS_PREBUILT_NASM=1` selects the prebuilt objects shipped inside `aws-lc-sys`
- A move to a pure-Rust crypto backend was tried in 0.5.6 and reverted, because crates.io has no pure-Rust backend for `russh` 0.65 or `instant-acme` 0.8
- Forbidden stacks: OpenSSL, `native-tls`, `libssh2-sys` and `ring`
- Host keys: trust on first use (TOFU) file under the platform config dir (`directories`, and XDG Base Directory or XDG on Linux)
- SSH channel compression is `none` only (no zlib) on all platforms
- Release gates for this policy are local scripts: `cargo deny` and `scripts/check_advisory_hits.py`


## Linux
- Prefer `cargo install ssh-cli --locked` into `~/.cargo/bin`
- Expect XDG config under `~/.config/ssh-cli/` by default
- The command-line interface (CLI) creates a new config directory with mode `0700` and writes `config.toml` and `secrets.key` with mode `0600`
- Default at-rest encryption stores blobs in `config.toml`
- Keep `secrets.key` offline-backed up
- With `--use-keyring` or `secrets init --keyring` the primary key goes to Secret Service over zbus, which needs a session bus, and a store that does not answer a write exits 69


## macOS
- Same cargo install path as Linux
- Clear quarantine with `xattr -d com.apple.quarantine` when Gatekeeper blocks the binary
- Expect config under the macOS application support/project dirs resolved by `directories` 6
- The keyring backend for the primary key is Keychain, selected by `--use-keyring` or by `secrets init --keyring`, and both write to the same profile-scoped entry
- Only `x86_64-apple-darwin` is type-checked by a gate, and `aarch64-apple-darwin` is not covered (see the target table)


## Windows
- Install through Rustup and cargo on a supported toolchain (minimum supported Rust version or MSRV 1.89, which `russh` 0.65 requires)
- The keyring backend for the primary key is Credential Manager, reached by the same `--use-keyring` and `secrets init --keyring`
- At process start the binary sets console code page 65001 (UTF-8) and enables `ENABLE_VIRTUAL_TERMINAL_PROCESSING` so ANSI colors work under conhost / PowerShell 5.1
- Use PowerShell completions from `ssh-cli completions powershell`
- A leading `~/` in a key path expands from `HOME` and falls back to `USERPROFILE`, but explicit paths are safer
- Config/project dirs come from `directories`
- Use `ssh-cli vps doctor --json` to see which config directory won (`local.config_path` and `local.layer`)
- Local path components are capped at 255 bytes
- Total path approaching legacy `MAX_PATH` (260) is rejected unless the `\\?\` extended prefix is used
- New VPS registry names are strict ASCII: letters, digits, dot, hyphen and underscore, alphanumeric first, at most 255 bytes
- Windows reserved devices (`CON`, `NUL`, `COM1`, …) are refused too, while reading an older `config.toml` stays lenient
- Windows has no chmod, so the `0700` mode does not exist there and the config directory relies on the profile access control list (ACL)
- `doctor` omits `local.config_dir_permissions` on Windows and reports `local.config_dir_exposed` as `false`
- The `0700` mode and the `getuid` check were only type-checked for Windows and macOS and never executed there
- Run scripts as `py -3 scripts/<name>.py`, because the `.sh` scripts no longer exist


## Config Directory Modes
- On Unix one single helper creates the config directory, and each new level is born with mode `0700`
- An existing default XDG directory is tightened to `0700` only when it is not a symlink and belongs to the current user
- Any other existing directory, such as a `--config-dir` you created, is never chmodded
- That directory writes a log warning (visible with `-v`) and shows as `local.config_dir_exposed` in `ssh-cli doctor --json`
- Read it with `ssh-cli doctor --json --select local.config_dir_exposed`, because `--select` also reduces the single `doctor` object
- `local.config_dir_permissions` holds the four-digit octal mode, such as `0700`, and is absent off Unix
- `--config-dir something.toml` makes the parent folder the config directory for every file: `secrets.key`, `known_hosts`, `active`, `lang` and the TLS files sit next to `config.toml`
- When that parent does not exist, the CLI creates the parent and never a folder named `something.toml`
- `vps export -o new/deep/e.json` creates the missing folders with mode `0700` and never changes an existing one
- `known_hosts.lock`, `config.toml.lock` and `lang` are born with mode `0600`, with no window before a chmod
- ACME and mTLS private keys are created with mode `0600`


## Text, Names and UTF-8
- Text mode escapes control and bidi characters as `\u{1b}`-style text on every OS: messages, errors, `list`, `show`, `health-check`, `doctor` and batch output
- `sftp ls` and `sftp stat` also escape `\n` and `\t` in remote names
- JSON keeps the original characters because it is for machines, and remote `exec` output stays raw by design
- New host names accept only ASCII letters, digits, dot, hyphen and underscore on every OS
- A legacy name with spaces, accents or control characters still loads, and `vps rename '<old name>' <new name>` repairs it
- `doctor` lists legacy names that fail the strict charset in `local.invalid_host_names`
- `tls mtls import --name` refuses control, formatting and bidi characters with exit 64, while `serviço-1` is still accepted
- Files and config use UTF-8, and the Windows console is switched to code page 65001 at boot
- Turn colors off with `--no-color` on every OS, because it wins over any terminal detection


## Containers
- Copy the binary into distroless or distro images without Node
- Persist config dir (or pass `--config-dir`) for multi-run host memory (`config.toml`, `known_hosts`, `secrets.key`, `active`)
- Keep one-shot semantics
- Do not wrap the CLI as a long-lived sidecar unless tunnel timeout is set
- Never bake live secrets or `secrets.key` into image layers
- Runtime markers (`/.dockerenv`, `/run/.containerenv`, `KUBERNETES_SERVICE_HOST`, `container=`) surface as `local.runtime.is_container` in `vps doctor --json`


## Runtime diagnostics
- `ssh-cli vps doctor --json` embeds `local.runtime`: `os`, `arch`, `is_wsl`, `is_container`, `is_ci`, `is_termux`, `sandbox` (`flatpak` | `snap` | null)
- Flatpak/Snap host installs emit a warning at boot, because filesystem and keyring may differ
- Detection never shells out (`uname` and `systemd-detect-virt` are not used)


## External processes (G-PROC)
- Runtime product code never spawns local children: SSH, Secure Copy Protocol (SCP) and tunnels run in-process through `russh`, so the agent host needs no OpenSSH `ssh`, `scp` or `ssh-keygen`
- Remote elevation packs `sudo`/`su` + `sh -c` on the target host via the SSH channel only, quoted and with passwords on channel stdin
- That is not local `Command`
- Build-time optional: `git` in `build.rs` for short HEAD
- The hash resolves in the order env, git, `.commit_hash`, `unknown`
- The file holds `<short hash> <version>` and counts only when that version is the package's, so a stale file yields `unknown`
- Explicit `Stdio` null/piped
- No shell
- Test-time optional: `ssh-keygen` fixtures for real OpenSSH keys
- Skip if missing
- Toolchain MSRV 1.89 ≥ 1.77.2 (CVE-2024-24576 BatBadBut)
- Product does not invoke `.bat`/`.cmd`
- Job Objects / process groups for local trees: N/A
- Remote commands reject NUL bytes before exec packing
- Multi-line CR/LF allowed


## Shell Support
- Completions via `clap_complete`: Bash, Zsh, Fish, PowerShell, Elvish (`ssh-cli completions <shell>`)
- Nushell is not in the default `clap_complete::Shell` enum
- Generate via external tooling if needed
- Prefer explicit argv arrays in agent runtimes over shell string eval
- Prefer stdin secret flags over embedding passwords in shell history


## File Paths and XDG
- Resolve the winner with `ssh-cli vps doctor --json` (includes `secrets_*` fields)
- Override with `--config-dir` (the product does not read `SSH_CLI_HOME`)
- Keep `known_hosts`, `active`, and `secrets.key` as sibling files of `config.toml`
- Atomic writes + flock protect concurrent one-shot processes on the same config
- A peer holding `config.toml.lock` makes the CLI wait `--lock-wait-ms` (default 5000) and then exit 69
- No product `.env` runtime store for secrets, language, or config home
- `locale show|set|clear` works on all platforms (XDG / ProjectDirs via `directories`)
- `tls mtls/*` and `tls acme/*` material lives under the platform config dir `tls/` (XDG on Linux)


## Full CLI surface (OS-agnostic)
- Command inventory is identical on every supported OS
- Discover at runtime with `ssh-cli commands` / `ssh-cli schema`

| Surface | Commands |
| --- | --- |
| VPS registry | `ssh-cli vps add` `ssh-cli vps list` `ssh-cli vps remove` `ssh-cli vps rename` `ssh-cli vps edit` `ssh-cli vps show` `ssh-cli vps path` `ssh-cli vps doctor` `ssh-cli vps export` `ssh-cli vps import` |
| Session | `ssh-cli connect` |
| Remote exec | `ssh-cli exec` `ssh-cli sudo-exec` `ssh-cli su-exec` |
| SCP | `ssh-cli scp upload` `ssh-cli scp download` (regular files only) |
| SFTP | `ssh-cli sftp upload` `ssh-cli sftp download` `ssh-cli sftp ls` `ssh-cli sftp mkdir` `ssh-cli sftp rmdir` `ssh-cli sftp rm` `ssh-cli sftp stat` `ssh-cli sftp rename` |
| Network | `ssh-cli tunnel` (local, `--reverse`, `--socks5`, `--remote-socket`) `ssh-cli health-check` |
| Secrets | `ssh-cli secrets status` `ssh-cli secrets init` `ssh-cli secrets reencrypt` |
| Discovery | `ssh-cli completions` `ssh-cli commands` `ssh-cli schema` `ssh-cli doctor` (root alias of `ssh-cli vps doctor`) |
| Locale | `ssh-cli locale show` `ssh-cli locale set` `ssh-cli locale clear` |
| TLS | `ssh-cli tls provider` `ssh-cli tls paths` |
| TLS mTLS | `ssh-cli tls mtls list` `ssh-cli tls mtls import` `ssh-cli tls mtls show` `ssh-cli tls mtls remove` |
| TLS ACME | `ssh-cli tls acme account create` `ssh-cli tls acme account show` `ssh-cli tls acme issue` `ssh-cli tls acme complete` `ssh-cli tls acme status` `ssh-cli tls acme list` |

### Global flags (all platforms)
- Output and logging: `--lang`, `-v`/`-vv`/`-vvv` (crate-scoped G2/G14, ambient `RUST_LOG` ignored), `-q`, `--no-color`, `--output-format`, `--json`
- Payload shaping: `--select` (alias `--fields`), `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- Run control: `--no-input`, `--dry-run`, `--lock-wait-ms`, `--timeout`, `--max-concurrency` (fleet fan-out 1..=64), `--fail-fast`, `--scp-file-concurrency`
- Config and secrets: `--config-dir`, `--disable-sudo`, `--replace-host-key`, `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- `--disable-sudo` disables `sudo-exec` and `su-exec` for that one invocation, and its alias `--disableSudo` is accepted before the subcommand and after every subcommand except `vps add` and `vps edit`
- On `vps add` and `vps edit` the same flag saves the choice in the host record whether it comes before or after the subcommand, and `vps edit --enable-sudo` undoes it
- `vps add` and `vps edit` have their own `--disable-sudo`, so clap refuses the alias `--disableSudo` after them with exit 2
- Use product line 0.5.3 as the floor for SSH File Transfer Protocol (SFTP) on every platform (G1 upload integrity)
- Verify destination with checksum
- No product `.env`: XDG / ProjectDirs / `--config-dir` / CLI flags only

### Per-leaf flags (identical on every platform)
- This table is the same list `ssh-cli commands --json` publishes, so a flag that exists here exists everywhere
- `vps add` and `vps edit` share the auth and record flags: `--host`, `--port`, `--user`, `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--sudo-password`, `--sudo-password-stdin`, `--su-password`, `--su-password-stdin`, `--tag`, `--timeout`
- `vps add` and `vps edit` also carry the command-bound and TLS flags: `--max-chars`, `--max-command-chars`, `--max-output-chars`, `--tls`, `--tls-sni`, `--tls-client-cert`, `--tls-client-key`, `--use-agent`, `--agent-socket`
- `vps add` alone has `--check`, which runs `health-check` once right after registering the host
- `vps edit` alone has the clear flags `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-key`, `--no-agent-socket`, `--no-tls`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key` and `--enable-sudo`
- `vps list` takes `--tag`, `vps import` takes `--file` and `--allow-incomplete`, and `vps export` takes `--output`, `--include-secrets` and `--i-understand-secrets-on-stdout`
- `exec`, `sudo-exec` and `su-exec` take `--all`, `--hosts`, `--tags`, `--use-active`, `--use-agent`, `--agent-socket`, `--step`, `--description` and `--timeout`
- `sudo-exec` adds `--sudo-password` and `--sudo-password-stdin`, and `su-exec` adds `--su-password` and `--su-password-stdin`
- `health-check` takes `--all`, `--hosts`, `--use-active`, `--use-agent`, `--agent-socket` and `--timeout`
- `scp upload` and `scp download` take `--src` and `--dest`, plus `--all`, `--hosts` and the auth trio
- `sftp upload` and `sftp download` take `--recursive`, `--src` and `--dest`, plus `--all` and `--hosts`
- `sftp ls`, `sftp mkdir`, `sftp rmdir`, `sftp rm`, `sftp stat` and `sftp rename` take the auth trio and `--timeout` only
- `tunnel` takes `--reverse`, `--socks5`, `--remote-socket`, `--bind`, `--i-accept-network-exposure` and `--timeout-ms`
- `secrets init` takes `--keyring` and `--force`, and `--keyring` selects the same store as the global `--use-keyring`
- `vps doctor` and the root `doctor` take `--hosts` and `--probe-ssh`
- `tls mtls import` takes `--cert`, `--key` and `--name`
- `tls acme account create` takes `--contact`, `--staging` and `--force`, and `tls acme issue` takes `--domain`, `--staging` and `--print-challenge`
- The long spellings `--verbose` and `--quiet` are accepted everywhere their short forms `-v` and `-q` are, and `--help` works on every leaf


## Tunnel mode portability
- All four `tunnel` modes are available on every supported OS: default local forward, `--reverse`, `--socks5` and `--remote-socket`
- `--remote-socket <PATH>` names a Unix socket on the server, not on this machine, so the client may run on Windows: locally it only ever speaks TCP and asks the server to reach the socket
- The gate that matters for `--remote-socket` is the server's support for the `direct-streamlocal@openssh.com` extension, detected on the wire rather than guessed from the local platform
- A Windows client against an OpenSSH Linux server works
- A Linux client against a server without the extension does not
- `--remote-socket` validates only what cannot be valid on any POSIX host: empty path, relative path, or embedded NUL, each exit 64
- Checking local existence would be actively wrong, because the path belongs to a filesystem this machine cannot see
- `--socks5` binds a local TCP listener and speaks RFC 1928 no-auth CONNECT, so it has no platform-specific dependency at all
- `--reverse` asks the server to listen
- Whether the server permits that bind is governed by its `GatewayPorts` and `AllowTcpForwarding`, which are server-side settings and not a client platform trait
- The local `--bind` is parsed as an IP address by clap on every OS, so a typo fails at exit 2 before any SSH handshake
- Any routable bind requires `--i-accept-network-exposure` on every platform
- The remote bind under `--reverse` is compared as text rather than parsed as an IP (RFC 4254 allows names and the empty string), so a typo there is exit 64 from the exposure guard
- Both behaviours are identical on every platform, because neither check consults the host OS
- Note also that `--bind` is accepted and then ignored under `--reverse` on every platform


## SCP portability
- SCP is regular files only on every platform (no recursive directory transfer)
- Use `sftp upload|download --recursive` for directory trees (no symlink follow)
- Failed or in-progress downloads use sibling path ending in `.ssh-cli.partial`, then rename into place (platform-agnostic atomic pattern)
- Upload streams in 32 KiB chunks on every OS (avoids full-file RAM load)
- mtime/mode preserve follows OpenSSH-style remote `-p` / `T` line
- On Unix local permissions APIs apply modes
- On Windows permission bits may not match Unix octal semantics, so do not assume full POSIX ACL fidelity
- Use product line 0.5.3 as the floor for SFTP on every platform (G1 upload integrity)
- Verify destination with checksum
- Real-SSH matrix E01–E21 (E10–E14 SCP, E17/E18 SFTP checksum/tree, E19/E20 payload shaping, E21 keyring rotation of a live profile) in `scripts/e2e_real_ssh.py` is primarily validated on Linux hosts
- `--local-sshd` is Unix-only, and on Windows the script stops with `FAIL E00: --local-sshd is Unix-only`
- Prefer local `sshd` / throwaway VPS
- Never run auth-failure storms on production hosts (fail2ban bans)


## Performance by Target
- Every call starts, answers and exits on every OS, so no process stays resident between calls
- musl builds may trade allocator characteristics
- Enable `musl-allocator` when needed
- Network round-trip time (RTT) dominates remote operations on every OS


## Agents Validated per Platform
- Linux hosts are the primary validation surface for agent subprocess runs
- macOS and Windows follow the same CLI contract and JSON schemas
- Root discovery aliases work on every OS: `ssh-cli doctor` (alias of `vps doctor`) and `ssh-cli schema` (embedded catalog / one schema body)
- JSON contracts (`scp-transfer` event, `tunnel_listening`, auth flags for tunnel/health) are identical on every OS
- See AGENTS.md and docs/schemas/
- Tunnel `--bind` defaults to `127.0.0.1` (loopback) on every platform
- Override only when intentionally exposing the listener
- Container agents must preserve exit codes and stdout/stderr separation
- Default tracing is error-level so agent stderr stays free of INFO prose unless `-v`/`-vv`/`-vvv` is set (crate-scoped, ambient `RUST_LOG` is ignored)
- Parse machine contracts from stdout only
- Treat stderr tracing as non-contract logs
- JSON error envelopes use stderr when JSON mode is active
- Real SSH E2E helpers live in `scripts/e2e_real_ssh.py` (anti-leak, local only, E01–E21, never production auth-failure storms)
- `ssh-cli --version` stamp (`Cargo` version + git hash + optional `-dirty`) is OS-agnostic
- No product `.env` runtime store: XDG / `--config-dir` / CLI flags only


## Multi-OS local matrix (G-E2E-18)
- Product code: `src/platform/{linux,macos,windows}.rs` modules, multi-OS behavior is validated locally
- Local multi-arch binaries via `scripts/dist_multiarch.py` (flags `--targets`, `--out-dir`, `--jobs`, `py -3` on Windows, and `Cross.toml`) when `cross` or a local cross linker and sysroot are installed
- Every gate is a local script run by hand, and maintainers validate on Linux primarily
- Check path length / agent socket notes on macOS and Windows before tagging a release


## Release Notes
- Only the 0.5.6 entries describe the current behavior, the older entries are history
### 0.5.6 — keyring and contract
- The keyring backend compiles a real store on all three systems: Keychain on macOS, Credential Manager on Windows and Secret Service over zbus on the other unix targets
- An unavailable store exits 69 on every platform instead of answering success, so a machine without a session bus reports the truth
- The minimum Rust version is 1.89 on all three targets, which russh 0.65 requires
- The installed Linux x86_64 release binary measures 9048976 bytes with sha256 `a4c2079731cbbd8dc4a0d212baa87e904d5d507ff458f6726de96599cd728536`
- macOS and Windows remain type-checked only, because the battery runs on Linux
### 0.5.6 — audit closure
- The TLS root store drops the distrusted `emSign ECC Root CA - G3` chain, and `scripts/check_advisory_hits.py` fails the battery when an advisory reaches the lockfile
- Single-record payloads accept `--select` and `--truncate-content`, and a reduced bare array reports its count on stderr
- No platform-specific change: the same three targets build and pass the same gates
- The pure-Rust crypto migration was reverted, so the TLS provider stays `aws_lc_rs` and a source build still needs a C compiler
### 0.5.6 — edit and rename
- `vps rename <NAME> <NEW_NAME>` and `vps edit <NAME> --name <NEW>` change a host name natively, with the credentials untouched
- The record seals each secret without associated data, so it stays readable under the new name
- Every save re-seals with a fresh nonce, which is normal and never touches the plaintext
- This replaces the export/edit/import detour, which silently dropped agent authentication because the export carried no agent fields
- Eight clear flags remove a field instead of overwriting it
- `--dry-run` covers both verbs
- The config lock waits a bounded `--lock-wait-ms` and then exits 69 instead of hanging
- `commands --json` publishes every argument
- Text mode escapes control, formatting and bidi characters on every OS, and JSON keeps them raw
- A new host name accepts only ASCII letters, digits, dot, hyphen and underscore, and an older `config.toml` with other names still loads
- A config directory the CLI creates is born `0700`, and an existing one is tightened only when it is the default XDG directory
- `--config-dir something.toml` makes the parent folder the config directory for every file

### 0.5.5 — target-designation security release
- Fixes GAP-SSH-EXEC-ARGC-001: `exec`/`sudo-exec`/`su-exec` decided the target host by positional count
- So `exec <HOST> --step <CMD>` executed the host name as a remote binary on whatever host `connect` last selected
- BREAKING: a lone positional is now a usage error
- Reach the active marker with the new `--use-active <COMMAND>`
- The single-host envelope gained `target_resolved` and `target_source`, so bind new consumers to those
- `host_resolved` and `host_source` are the 0.5.5 read aliases
- The envelope also gained `active_fallback`, and `doctor` now reports `local.active_vps`

### 0.5.4 — security and agent-native release
- Fixes a remote pre-auth denial of service (DoS) in the SSH banner path (A1)
- Stops server-sent setuid bits landing on downloaded files (A3)
- Closes the world-readable window on ACME/mTLS private keys (A2)
- Adds payload-shaping flags applied before serialization: `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- BREAKING: partial multi-host failure now exits 1 (was 65)
- BREAKING: a non-loopback `--bind` requires `--i-accept-network-exposure`
- New `tunnel_closed` event
