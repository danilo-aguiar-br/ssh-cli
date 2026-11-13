# Security Policy


[![Security Policy](https://img.shields.io/badge/security-policy-007AFF)](#reporting-a-vulnerability)

- Read this document in [Portuguese (pt-BR)](SECURITY.pt-BR.md)


## Supported Versions
- The current product line is 0.5.x and the latest release is 0.5.6
- The table below lists which ssh-cli versions currently receive security patches
- Users on unsupported lines must upgrade to the current release line

| Version | Status | Security Patches |
| --- | --- | --- |
| 0.5.6 | Supported | Yes, latest release and current line |
| 0.5.x before 0.5.6 | Supported | Yes, current line, but 0.5.0 to 0.5.4 carry GAP-SSH-EXEC-ARGC-001 and every earlier 0.5.x lacks the 0.5.6 fixes, so upgrade to 0.5.6 |
| 0.4.x | Supported | Critical / high when feasible, prefer upgrade to 0.5.x |
| 0.3.x | Limited | Critical only when feasible, crates.io 0.3.9 SCP wire was inoperant, so upgrade to 0.5.3+ for transfers |
| 0.2.x | Limited | Critical fixes only when feasible |
| 0.1.x | Unsupported | No patches |
| < 0.1 | Unsupported | No patches |


## Known Vulnerabilities
- Every ssh-cli release before 0.5.6 resolves dependencies that carry the advisories below, so upgrade to 0.5.6
- `russh` below 0.65.1 carries RUSTSEC-2026-0492, RUSTSEC-2026-0495, GHSA-fw5c-8223-6h42 and GHSA-3hm4-p7h4-rvwp
- `rustix` 1.1.4 carries RUSTSEC-2026-0486
- `webpki-roots` 1.0.9, `rustls-webpki` 0.103.15 and `webpki-root-certs` 1.0.9 carry RUSTSEC-2026-0381, RUSTSEC-2026-0382 and RUSTSEC-2026-0383 (CVE-2026-111943)
- The cause, the fixed version and the residual limit of each one are in [Security Fixes in 0.5.6](#security-fixes-in-056)
- Report a new vulnerability through [Reporting a Vulnerability](#reporting-a-vulnerability)


## Security Fixes in 0.5.6
### Dependency advisories — russh, rustix, async-executor and webpki-roots
- `russh` 0.62.5 is now 0.65.1, which closes RUSTSEC-2026-0492 (HIGH, `client::Handler` callbacks fired for a channel that was never opened) and RUSTSEC-2026-0495 (the agent server never called `Agent::confirm`)
- The same upgrade closes GHSA-fw5c-8223-6h42, where a zero maximum packet size stalled every channel write, and GHSA-3hm4-p7h4-rvwp, a stack overflow before authentication triggered by a nested OpenSSH certificate
- `Handler::check_server_key` now receives `&PublicKeyOrCertificate`, and the certificate branch is unreachable because `Preferred::DEFAULT` leaves `host_key_certificates` empty, so the embedded key goes through the same TOFU check
- `rustix` 1.1.4, flagged unsound by RUSTSEC-2026-0486, is now 1.1.6, and the yanked `async-executor` 1.14.0 is now 1.14.1
- `webpki-roots` 1.0.12 drops eight more distrusted roots: emSign Root CA - C1 and emSign ECC Root CA - C3 in 1.0.11, then Buypass Class 2, Buypass Class 3, DigiCert Trusted Root G4, QuoVadis Root CA 1 G3, QuoVadis Root CA 3 G3 and Certigna Root CA in 1.0.12
- The `advisory-hits` gate read `advisory.patched_versions` while `cargo audit` publishes `versions.patched`, so it reported no patched release for advisories that had one, and it now reads the published field

### Keyring backend
- `ssh-cli secrets init --keyring` reported success and persisted nothing
- The cause was the crate, not the machine: `keyring = "3"` resolved 3.6.3, which has no default feature and fell back to an in-process `mock` store
- The mock kept the key in the memory of the process that wrote it, so the next invocation answered `encryption_active: false` and the agent held a success it could not rely on
- The backend is now `keyring` 4.2.0 with the `v1` feature, which compiles a real store on all three systems: Keychain on macOS, Credential Manager on Windows and Secret Service over zbus on the remaining unix targets
- The store is consulted through `Entry::store_status()` before every write and every read, so an unavailable store exits 69 instead of answering `Ok`
- A second cause was measured: the leaf flag `secrets init --keyring` never reached the runtime state, because dispatch forwarded only the global `--use-keyring`
- A third cause was measured: a key stored by an earlier `secrets init --keyring` was invisible without repeating the flag
- The keyring entry is scoped by a digest of the resolved config directory, because the OS store is machine-wide while `secrets.key` and `config.toml` are per profile
- Without that scope, an isolated run with `--config-dir` overwrote the key of the real profile, which this audit measured happening
- A fourth cause was measured in an intermediate 0.5.6 build itself: `--use-keyring secrets init` answered `key_source: xdg_file`, because the subcommand received only the leaf flag and the global one never reached the write path
- A fifth cause was measured against a live profile: `secrets init --keyring` on a profile that already held `secrets.key` answered `key_source: keyring` and minted a second key, leaving the file to answer first on every later read
- The profile then held two different primary keys, `secrets status` reported `xdg_file`, and every secret already encrypted failed to decrypt with the key the command had just named as active
- The guard now asks the profile instead of the destination, so an existing key in either store refuses the write without `--force`, and `--force` removes the other source so exactly one key survives the rotation
- D2: `secrets init --force` no longer writes `secrets.key.bak`
- The backup copy left key material on disk after every rotation and violated the project rule against backup files
- The store is reached through a scoped OS thread, because the zbus blocking facade parks the calling thread on an `async-io` reactor while the process runs a multi-thread Tokio runtime

### Residual limits — keyring backend
- The keyring round-trip is proved on Linux with a session bus, and macOS and Windows only reach type-check in the `cross-targets` gate, so the Keychain and Credential Manager stores are not executed by this release's battery
- A machine without a session bus exits 69 and the tests report SKIP, never PASS
- The config directory digest is an identifier, not a secret, so it does not resist a local attacker who can already read the store's entry names

### TLS roots and advisory gate — A1 and A4
- A1: the TLS (Transport Layer Security) root store no longer trusts a distrusted authority: `webpki-roots` 1.0.9, `rustls-webpki` 0.103.15 and `webpki-root-certs` 1.0.9 still carried `emSign ECC Root CA - G3`, and the eMudhra intermediates under it have a public private key (Mozilla Bug 2083449, CVE-2026-111943, RUSTSEC-2026-0381/0382/0383)
- The client ACME (Automatic Certificate Management Environment) and every outbound TLS connection build their `RootCertStore` from that list, so the defect reached both
- The minimum is pinned in the manifest, `webpki-roots = "1.0.10"`, and the lock resolves `rustls-webpki` 0.103.17 and `webpki-root-certs` 1.0.10
- The manifest pin exists on purpose: a bare lock update could walk back to the compromised version silently, and now it cannot build
- A4: a new gate compares the advisory database with the resolved tree on every run: `scripts/check_advisory_hits.py` runs `cargo audit --json` and fails on any entry in `vulnerabilities.list` or on a yanked package
- The older `advisory-freshness` gate measured only the age of the clone, so a stale database could keep the battery green over a live CVE

### Residual limits — TLS roots and advisory gate
- The `advisory-hits` gate refuses to run when the advisory database is older than seven days, so a disconnected machine fails instead of passing on stale data
- The pin covers the three crates in the measured advisory: a future advisory against a different crate still depends on the next database refresh

### Text output, host names and config directory
- Terminal escape sequences no longer reach the operator's terminal in text mode: A host name, an error message or a stored field could carry ESC, BEL or bidi characters, and text mode printed them raw
- `src/output/sanitize.rs` escapes control, formatting and bidi characters as `\u{1b}`-style text in the one text funnel: success, error and warning messages, `vps list`, `vps show`, `health-check`, `doctor` and the batch outputs
- `sftp ls` and `sftp stat` also escape `\n` and `\t` in the remote name, so a hostile server cannot forge extra lines in a listing
- JSON output is unchanged, because it is for machines and `serde_json` already escapes ESC and BEL
- A new host name has a strict charset: `VpsName::try_new` accepts ASCII letters, digits, dot, hyphen and underscore, starting with a letter or digit, from 1 to 255 bytes
- It guards `vps add`, the destination of `vps rename` and `vps edit --name`, and `vps import`
- Reading stays lenient, so an old `config.toml` with a hostile name still loads, `doctor` lists the name escaped in `local.invalid_host_names` and `vps rename` repairs it
- Two host tables that normalize (NFC) to the same name make loading fail with exit 65 and `config has two hosts named ...`, so a host never disappears silently
- A name that becomes a target (`exec`, `health-check`, `--hosts`, `connect`) is refused with exit 64 when it carries control, formatting or bidi characters
- `tls mtls import --name` refuses the same characters with exit 64 and creates no directory, so an identity name cannot become a hostile directory name
- The config directory is born private: Every directory level the CLI creates gets mode `0700`, where it used to inherit `0755` from the umask next to a `0600` `config.toml`
- An existing directory is tightened to `0700` only when it is the default XDG directory, is not a symlink and belongs to the current user
- Any other existing directory is never chmodded: `doctor` reports `local.config_dir_exposed` and `local.config_dir_permissions`, and a log warning is visible with `-v`
- `vps export -o` creates missing directories `0700`, and `known_hosts.lock` and `lang` are born `0600` with no window before a chmod
- RUSTSEC-2026-0235 is resolved by removal: The advisory came from `rkyv`, which entered the lock only through the optional `rust_decimal`, so the unused money module and that dependency are gone
- The removal is a BREAKING change for library users and the changelog carries its migration path
- The published package no longer carries the nine `scripts/*.sh`, the stray root file `json` or pending `*.snap.new` files, and the `Cargo.toml` `exclude` keeps them out
- The rename path has no re-sealing step of its own, and the credential is never retyped: `VpsRecord` seals its secrets without associated data, so the blob is not bound to the host name
- The ciphertext token does change, because the ordinary save seals every secret again with a fresh nonce, and what survives is the credential, readable under the new name
- The export, edit and import detour is gone: `vps export` redacts secrets by design, so the round trip forced an operator to retype a credential into argv or a file, which is the exposure the stdin paths exist to avoid
- That export also carried no `use_agent` or `agent_socket`, so an agent-authenticated host came back unauthenticated and failed at the next connection instead of at the import
- Clearing a field cannot silently leave a host unusable: The eight clear flags are refused with exit 64 when the result would have no primary authentication method, or when `--no-agent-socket` would strip the only endpoint on a platform that has no default
- A malformed record at rest is worse than a refused edit, because every later command inherits it
- Contention is bounded, so a hostile or wedged lock holder cannot make the CLI hang: The config lock waits `--lock-wait-ms` (default 5000, max 300000) under full-jitter backoff and then exits 69 `unavailable`, which is retryable
- Before this, a peer process holding `config.toml.lock` kept every writer blocked indefinitely, with no output, no exit and no diagnosis

### Residual limits — text output, host names and config directory
- The stdout and stderr of a remote `exec` stay raw by design, so a hostile server can still write escape sequences into them
- JSON keeps every character of the escaped set raw, for example U+202E and ZWSP, because JSON is for machines
- Windows has no chmod, so the config directory relies on the profile ACL
- The global shaping flags reduce collections only, so they did not reach the `doctor` object: read `local.config_dir_exposed` with `ssh-cli doctor --json | jaq .local.config_dir_exposed` (a later 0.5.6 change added `--select` for single records, so this limit describes the first 0.5.6 fixes)
- macOS and Windows only pass through type-check in the `cross-targets` gate, so mode `0700` was not executed there
- A legacy record with a control character in a tag, host or user still exits 65, because only the host NAME is tolerant on load
- Error bodies stay in English by design, so agents receive stable text


## Security Fixes in 0.5.5
- GAP-SSH-EXEC-ARGC-001: the target host was chosen by positional count instead of by content (CWE-441). `exec`, `sudo-exec` and `su-exec` accepted one or two positionals: two meant `<VPS> <COMMAND>`, one meant "command against the host `connect` last marked"
- `--step` is an appending flag, not a positional, so `exec <HOST> --step <CMD>` collapsed into the one-positional branch
- The host name was then run as a remote binary (remote exit 127) and every `--step` after it executed on the marked host: a host the operator never typed
- Because the later steps succeeded, the process exited 0, so the operator read success while systemd units, packages and firewall rules landed somewhere else
- This is a security defect and not an ergonomics one: in a CLI for remote access the critical dimension is not what runs, it is where
- It required no attacker: ambiguous argv from a legitimate operator was enough, which is why it outranks a bug needing a hostile peer
- Fixed by making the ambiguity unrepresentable, not by guessing intent: A lone positional is now a usage error (exit 64) on every remote surface, `health-check` included
- Designate the target explicitly with `<VPS> <COMMAND>`, with a selector (`--all`, `--hosts a,b`, `--tags prod`), or with the marker on purpose via the new `--use-active`
- Opting in with no marker present is exit 66
- Failure now happens during parse, before any socket opens, so the blast radius of the mistake is a usage message instead of an audit across two hosts
- GAP-SSH-EXEC-ENVELOPE-002, the single-host envelope named no host, so the deflection above was undetectable from stdout: The fleet path already emitted a per-host `name` while the single-host path, the one that accepted the silent fallback, emitted none, so the asymmetry ran opposite to the risk
- Every envelope now carries `target_resolved`, `target_source` (`argv` | `active_marker` | `selector`) and `active_fallback`, on the error envelope too once the target resolved, where they are absent rather than empty
- `host_resolved` and `host_source` ship as 0.5.5 read aliases
- Bind new consumers to the canonical names
- `doctor` reports `local.active_vps`, so the marker is now legible by machine: before 0.5.5 no command exposed it and the only way to read it was opening the XDG file
- Upgrade to 0.5.5 for both
- Neither needs a remote attacker, and the first one silently rewrote the wrong machine


## Security Fixes in 0.5.4
- A1, remotely triggerable abort while logging the pre-auth banner: The SSH banner is remote input that arrives before authentication, and it is retained for diagnostic logging
- The `AUTH_BANNER_MAX_CHARS` (512) log cap already existed (G-SSH-14)
- The defect was that the cut was applied by byte index, which panics whenever it lands inside a multi-byte character
- The release profile sets `panic = "abort"`, so there is no unwind: an unauthenticated peer could kill the process outright with a banner containing a single well-placed non-ASCII character, taking a whole multi-host fan-out down with it
- Truncation now happens on character boundaries
- Note the residual limit of the fix: russh materialises the complete banner and hands it to the callback, so the client still holds the full string in memory, what A1 bounds is the panic, not the allocation
- This is the highest-severity class here because it needs no credentials
- A2, world-readable window on ACME and mTLS (mutual TLS) private keys: Key material under XDG `tls/acme/` and `tls/mtls/` was written with `std::fs::write`, which creates the file readable by any local user and only then tightens the mode, leaving a window a co-tenant could read
- Writes now go through the shared helper that creates at `0600`, so the window does not exist
- Fixing the mode after the fact only shortens exposure
- It never removes it
- A3, server-sent elevation bits landing on downloaded files: The SCP `C` header and SFTP (SSH File Transfer Protocol) attributes are produced by the server, so the mode they carry is untrusted
- Masking with `0o7777` let a hostile peer put setuid, setgid or sticky on the local file
- Inbound modes are now masked with `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`), so only plain `rwx` triples cross the network inbound
- The remote end has no authority to decide what is privileged on your machine
- Non-loopback bind now requires acknowledgement: `tunnel --bind` outside loopback requires `--i-accept-network-exposure` (G-TUN-R13)
- Before this, `--bind 0.0.0.0` silently published the forwarded remote service to the whole local network
- Under `--reverse` the same flag guards the server's bind address, because that is the exposed end in that direction
- Upgrade to 0.5.4 for all four
- A1 and A3 are reachable by a remote peer
- A2 is reachable by a local co-tenant


## Reporting a Vulnerability
- Report security issues through [GitHub Security Advisories](https://github.com/danilo-aguiar-br/ssh-cli/security/advisories/new) in the public `ssh-cli` repository as the preferred private channel
- Use email at daniloaguiarbr@proton.me only as fallback when GitHub private reporting is unavailable
- Never open a public GitHub issue, pull request, or discussion for security-related reports
- Include a minimal reproduction, affected versions, and expected versus actual behavior
- Include environment details such as OS, architecture, and rustc version
- Include a CVSS v4.0 severity estimate when possible (CVSS 3.1 accepted as fallback) to accelerate triage
- Redact live credentials from every attachment and log excerpt


## Response SLA
- Triage of every advisory starts within 72 business hours of submission
- Initial acknowledgment is sent within that same 72-business-hour window
- You receive a case identifier and an assigned maintainer contact
- Progress updates are shared at least every 7 days until resolution or public disclosure


## Fix SLA by CVSS Severity
- Critical severity (CVSS 9.0 to 10.0) receives a patch within 7 calendar days of validated triage
- High severity (CVSS 7.0 to 8.9) receives a patch within 14 calendar days of validated triage
- Medium severity (CVSS 4.0 to 6.9) receives a patch within 30 calendar days of validated triage
- Low severity (CVSS 0.1 to 3.9) receives a patch within 90 calendar days of validated triage
- Released fixes include a CHANGELOG entry and a GitHub Security Advisory when the affected line is still supported


## Disclosure Policy
- Coordinated disclosure is the default for validated vulnerabilities
- Public disclosure is delayed until a fixed release is available or the Fix SLA window expires with documented reason
- Researchers may publish after mutual agreement or after the coordinated window ends


## Security Update Policy
- Security fixes land on the current minor line first
- Backports to older minor lines happen only when impact and effort justify the work
- Users must upgrade with `cargo install ssh-cli --locked --force` after a security release
- `python3 scripts/generate_sbom.py` writes a CycloneDX SBOM (software bill of materials) to `target/sbom/ssh-cli.cdx.json` when `cargo-cyclonedx` is installed
- Without `cargo-cyclonedx` the same command writes only a `cargo tree` inventory, which is not an SBOM


## Hall of Fame
- Security researchers who request public credit are listed here after coordinated disclosure completes
- The list starts empty for the current ownership line under `danilo-aguiar-br`


## Threat Model (product: G-SEC-10)
- Trust boundary: CLI argv, env, XDG files (`config.toml`, `secrets.key`, `known_hosts`), and the remote SSH server are untrusted inputs
- Only the local process identity and OS keyring (when enabled) are treated as higher-trust material
- Assets: SSH passwords / key passphrases / sudo-su secrets (memory + at-rest), host-key fingerprints (TOFU), primary AEAD (Authenticated Encryption with Associated Data) key (`secrets.key` / keyring), the ability to run remote shell / SCP / local port forwards, and, as GAP-SSH-EXEC-ARGC-001 proved, the identity of the host reached, since the same argv running on the wrong machine is a breach even when every credential held
- Adversaries considered: (1) hostile CLI/agent argv and env
- (2) malicious or MITM SSH servers (host-key change)
- (3) co-tenant local users reading world-readable files or process listings
- (4) compromised transitive crates (supply chain)
- (5) ambiguous argv from a legitimate operator or agent: no adversary required
- GAP-SSH-EXEC-ARGC-001 was of this class: a well-formed command line that the product resolved to a host nobody named
- Treat "which machine does this reach" as an attacker-controlled question even when no attacker exists
- Out of scope: multi-tenant SaaS isolation, browser XSS/CSRF, container escape, kernel Spectre mitigations beyond normal process isolation, and long-lived daemon attack surface (this CLI is one-shot)
- Controls: typed validation at CLI/import/path boundaries
- `SecretString` + zeroize
- ChaCha20-Poly1305 at rest
- TOFU host keys with constant-time fingerprint compare
- Shell packing with single-quote escape and secrets on channel stdin
- No local shell-out for SSH
- Terminal-safe text output: control, formatting and bidi characters are escaped before any text reaches the terminal (0.5.6)
- Strict host-name and mTLS identity-name charset on creation (0.5.6)
- Config directory born `0700` (0.5.6)
- `deny.toml` + local `cargo deny check`
- Release `overflow-checks`
- Documented `unsafe` only at OS console/signal edges
- Review trigger: re-read this model on any change that adds network protocols, long-lived processes, new secret stores, or `unsafe` outside OS FFI wrappers


## Transport & crypto policy (G-TLS)
- Default path: product network transport is SSH-2 via `russh` on plain TCP
- Wire crypto uses the aws-lc-rs backend (`ssh-real`)
- Host authentication is TOFU (`known_hosts` under XDG)
- Optional SSH-over-TLS: when a VPS has `tls = true` (or `--tls` on `vps add`/`edit`), the client dials TCP, completes a rustls handshake (SNI + optional mTLS), then runs SSH on the TLS stream
- Floor: rustls ≥ 0.23.18 (never 0.23.13–0.23.17 Acceptor CVE window), the `min_rustls` value that `ssh-cli tls provider` prints
- The manifest asks for more than the floor: `Cargo.toml` declares `rustls = { version = "0.23.25" }`
- Provider: binary `main` calls `CryptoProvider::install_default` with `aws_lc_rs` only (once, before the Tokio runtime)
- Libraries use `ClientConfig::builder` / process default: they never reinstall
- Dual provider `ring` is banned in `deny.toml`
- Forbidden stacks: no `native-tls`, OpenSSL, `libssh2-sys`, or product use of `ring`
- PEM parse uses `rustls-pki-types` (not unmaintained `rustls-pemfile`)
- mTLS: client cert/key PEMs under XDG `tls/mtls/<name>/` (`ssh-cli tls mtls import`, `ssh-cli tls mtls list`, `ssh-cli tls mtls show`, `ssh-cli tls mtls remove`) or absolute paths on the VPS record
- ACME: Let's Encrypt via `instant-acme` (DNS-01, agent two-step: `tls acme issue --domain <DOMAIN> --print-challenge` → set TXT → `tls acme complete --domain <DOMAIN>`)
- Account + certs under XDG `tls/acme/` (0o600)
- Prefer staging with `--staging` for tests
- SSH channel compression is disabled (client prefers `none` only) to reduce compression side-channels when secrets cross the channel
- Release and supply-chain gates for this policy are local scripts (`cargo deny check`, `cargo tree`, residual tests) (G-TLS-11)


## STRIDE Map (critical components: G-SECDEV-03)

| Component | S | T | R | I | D | E | Primary controls |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CLI argv / env / stdin secrets | spoof agent args | tamper flags | deny which host ran it | info leak via argv | DoS huge stdin | elev via sudo-exec | typed clap + secret stdin → `SecretString` + size cap + explicit target designation with `target_resolved` / `target_source` on every envelope (0.5.5) |
| XDG `config.toml` / `secrets.key` | forge host entry | rewrite secrets | - | read co-tenant | truncate/corrupt | - | 0o600 file + 0o700 directory (0.5.6) + flock + AEAD at-rest |
| TOFU `known_hosts` | MITM host | swap fingerprint | - | fingerprint oracle | lock starve | - | flock + constant-time compare + `--replace-host-key` explicit |
| SSH session (russh) | fake server | MITM / key change | - | channel sniff | hang / flood | remote shell | TOFU + timeouts + no local shell-out |
| Remote packing (`sh -c` sudo/su) | - | inject metachar | - | secret on argv | - | unintended elev | single-quote escape + password on channel stdin |
| Multi-host fan-out | - | reorder jobs | - | cross-host leak in logs | resource exhaustion | - | `Semaphore` budget + cancel + redacted JSON |
| Terminal text output | hostile host name posing as another | ESC / bidi redraws the screen | - | hidden or forged lines | - | - | one escape funnel + strict name charset (0.5.6) |
| Supply chain (deps) | typosquat | malicious crate | - | backdoor | build DoS | - | `deny.toml` + local `cargo deny` + `--locked` + lockfile TLS bans |

### Accepted residual risks
- Password on argv is still parseable, with a warning, so prefer stdin
- The remote host after authentication is fully trusted for the one-shot command the operator requested, a premise that only holds once the target is designated explicitly, which is why 0.5.5 made designation mandatory
- A co-tenant with the same UID can read process memory, which is OS isolation and not this CLI
- Spectre and CPU side-channel threats are out of scope for a one-shot userland tool


## Logging and secret leakage (G2 / G14)
- Never: rely on ambient `RUST_LOG` for product debugging, the product ignores it (not a config store)
- Use graduated flags only: `-v` (info) / `-vv` (debug) / `-vvv` (trace) via `ArgAction::Count`
- The filter is always crate-scoped (`warn,ssh_cli=*`): never bare global `debug`/`trace`
- This keeps russh (and other deps) from dumping passwords or channel material onto stderr
- Even `-vvv` does not enable russh encrypted-channel dumps
- Only the `ssh_cli` crate (plus warn-level deps) is allowlisted
- Prefer 0.5.3+ so agents cannot accidentally enable global debug that leaked credentials in older lines


## Best Practices for Users
- Prefer private key authentication over password authentication when the host allows it
- Prefer `--password-stdin`, `--sudo-password-stdin`, and `--su-password-stdin` over argv secrets (password-on-argv emits a stderr warning on 0.5.2+)
- Prefer stdin password flags for agent runs
- Avoid embedding live secrets in shell history
- Default at-rest encryption (ChaCha20-Poly1305): on first secret write, auto-creates `secrets.key` (0o600) next to `config.toml` unless you opt out
- Secrets control is CLI/XDG only: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, or XDG `secrets.key`
- No `.env` product store
- Key resolution order: `--secrets-key-file` → keyring (when `--use-keyring`) → XDG `secrets.key` → keyring holding a key from an earlier `secrets init --keyring`
- `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` are rejected fail-closed (not a store)
- CLI: `ssh-cli secrets status|init|reencrypt` (never prints the master key)
- `--json` emits `secrets-init` / `secrets-reencrypt` without key material
- Opt-out for tests only: `--allow-plaintext-secrets` (no env store)
- Never: log the master key, passwords, or decrypted secrets
- `--include-secrets` to pipe/non-TTY requires `-o`/`--output` or `--i-understand-secrets-on-stdout` (guard against accidental secret dump on stdout)
- Keep `config.toml` mode `0600` and restrict backup locations
- Review TOFU host-key change errors before using `--replace-host-key`
- Never commit host registries that include live secrets
- Never commit local tooling sidecars (e.g. `.setting.cyber/`), the Grok config (`~/.grok/config.toml`), XDG `config.toml` / `secrets.key` / `known_hosts`, or the password files handed to the E2E script (`--password-file`, `--sudo-password-file`) into the repository
- Real-SSH E2E (G-E2E-05) must keep credentials outside the tree: prefer `python3 scripts/e2e_real_ssh.py --local-sshd` (unprivileged sshd on loopback, Unix only), or the lab flags `--host`, `--port`, `--user` and `--password-file` (plus `--sudo-password-file`, a path or `-` for stdin), or `$HOME/.grok/config.toml` via `--from-grok-config`
- The script takes flags only and reads no environment variables
- Offline / no lab → SKIP exit 0. The script refuses grok configs under the repo root
- Demo passwords in public docs are placeholders only (e.g. `demo-password-not-real`)
- Never reuse them on live hosts
- Disable elevation with `--disable-sudo` when a workflow must not escalate
- Keep `tunnel --bind` on loopback unless a second machine genuinely must reach the forward
- Any routable bind requires `--i-accept-network-exposure`, which is the acknowledgement of that exposure rather than a formality
- Treat `tunnel --remote-socket /var/run/docker.sock` as privilege delegation: forwarding a Docker or systemd socket hands the local port whatever authority that socket grants on the remote host
- Prefer `tunnel --socks5` bound to loopback over a routable fixed forward when an agent needs several destinations
- One acknowledged local bind beats N exposed ones
- Run one-shot commands only
- Never expect a long-lived SSH daemon from this CLI
- Install with `--locked` to avoid accidental crypto re-resolve drift
- Read machine output as JSON, because text mode escapes control characters for the human reader
- Keep the config directory private: The CLI never chmods an existing directory that is not the default XDG one, so read `local.config_dir_exposed` in `ssh-cli doctor --json` and run `chmod 700` yourself when it is `true`
- After upgrading to 0.5.6, read `local.invalid_host_names` in `ssh-cli doctor --json` and repair each legacy name with `ssh-cli vps rename`
- Switch a host between password, key and agent with `vps edit`, which clears the old secret, instead of the export, edit and import detour
- Prefer current 0.5.3+ for SFTP integrity (G1 closed 0-byte upload truncation), crate-scoped verbosity (G2/G14: no russh password dumps), the supply-chain floor (russh 0.65.1), working SCP/SFTP wire, ACME permanent classification (`invalidContact` → exit 64), and the three-asterisk mask (`FIXED_MASK`) shown by `vps list` and `vps show`
- Historical honesty: 0.4.1 fixed empty-secret redacted export (never `sshcli-enc:` of empty) and tunnel post-bind exit 0
- Default redacted `vps export` clears secrets: `password` is an empty string and the other secrets are null
- Empty secrets must serialize as empty strings, never encrypted `sshcli-enc:` blobs of empty values (0.4.2 EXP-001)
