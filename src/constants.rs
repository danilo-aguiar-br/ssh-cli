// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Named domain constants (Rules Rust — no hardcoding).
//!
//! Compile-time values with semantic names live here. Runtime host credentials,
//! per-VPS timeouts, and primary-key material are **never** hardcoded: they are
//! loaded once from CLI / OS keyring / XDG files under
//! [`crate::constants::APP_NAME`] (`directories::ProjectDirs` + optional `--config-dir`).
//!
//! ## 12-Factor alignment (this product)
//!
//! | Factor | How ssh-cli applies it |
//! |--------|------------------------|
//! | III Config | Hosts in XDG `config.toml`; secrets via CLI flags / keyring / `secrets.key` (not `SSH_CLI_*` env stores) |
//! | Build/Release/Run | Version from `CARGO_PKG_VERSION`; commit via `build.rs`; no prod URL in binary |
//! | Backing services | SSH endpoints are user registry data, not compile-time constants |
//!
//! Modules keep *local* constants when the name is only meaningful in that
//! module (e.g. concurrency RAM budget). Cross-cutting identity, storage file
//! names, env var names, network defaults, and process timing are centralized.

// ── Identity / XDG ──────────────────────────────────────────────────────────

/// Binary and project name (`clap`, keyring service, XDG directory leaf).
pub const APP_NAME: &str = "ssh-cli";

/// `directories::ProjectDirs::from` qualifier (empty = no reverse-DNS prefix).
pub const PROJECT_QUALIFIER: &str = "";

/// `directories::ProjectDirs::from` organization (empty = app-only path).
pub const PROJECT_ORGANIZATION: &str = "";

/// Registry file name under the config directory.
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Primary-key file name (next to [`CONFIG_FILE_NAME`]; mode 0o600).
pub const SECRETS_KEY_FILE_NAME: &str = "secrets.key";

/// TOFU host-key store file name (sibling of [`CONFIG_FILE_NAME`]).
pub const KNOWN_HOSTS_FILE_NAME: &str = "known_hosts";

/// Active-VPS marker file name (sibling of [`CONFIG_FILE_NAME`]).
pub const ACTIVE_VPS_FILE_NAME: &str = "active";

/// Persisted UI language preference file name (sibling of [`CONFIG_FILE_NAME`]).
pub const LANG_PREFERENCE_FILE_NAME: &str = "lang";

/// XDG subdirectory for TLS material (mTLS client certs, ACME account/certs).
///
/// Layout (all under config dir, mode 0o600 for secrets):
/// - `tls/mtls/<name>/cert.pem` + `key.pem`
/// - `tls/acme/account.json`
/// - `tls/acme/<domain>/cert.pem` + `key.pem` + `order.json` (pending)
pub const TLS_DIR_NAME: &str = "tls";

/// Subdirectory of [`TLS_DIR_NAME`] for imported mTLS client identities.
pub const TLS_MTLS_DIR_NAME: &str = "mtls";

/// Subdirectory of [`TLS_DIR_NAME`] for ACME account + issued certificates.
pub const TLS_ACME_DIR_NAME: &str = "acme";

/// ACME account credentials file name (JSON, 0o600).
pub const TLS_ACME_ACCOUNT_FILE_NAME: &str = "account.json";

/// Certificate chain PEM file name under an ACME/mTLS identity directory.
pub const TLS_CERT_FILE_NAME: &str = "cert.pem";

/// Private key PEM file name under an ACME/mTLS identity directory.
pub const TLS_KEY_FILE_NAME: &str = "key.pem";

/// Pending ACME order state file name (agent two-step DNS-01 flow).
pub const TLS_ACME_ORDER_FILE_NAME: &str = "order.json";

// ── Environment variable names (historical / fail-closed surface) ───────────
// Product config is CLI + XDG only. These names are kept for fail-closed
// rejection of secrets env stores and for test cleanup of legacy vars.
// G-AUD-12: do not reintroduce SSH_CLI_HOME / LANG / FORCE_TEXT as stores.
// G-UNSAFE-08: plaintext opt-out is CLI-only (`--allow-plaintext-secrets`).
// G-UNSAFE-14: concurrency is CLI-only (`--max-concurrency` + auto formula).

/// Historical name only — **not** read as product config store.
/// Use CLI `--config-dir` for isolated config roots.
pub const ENV_HOME: &str = "SSH_CLI_HOME";

/// Hex primary-key material. **Fail-closed:** presence is rejected (not a store).
/// Use XDG `secrets.key`, `--secrets-key-file`, or `--use-keyring`.
pub const ENV_SECRETS_KEY: &str = "SSH_CLI_SECRETS_KEY";

/// Path to a primary-key file (hex). **Fail-closed:** presence is rejected (not a store).
/// Use CLI `--secrets-key-file` or XDG `secrets.key`.
pub const ENV_SECRETS_KEY_FILE: &str = "SSH_CLI_SECRETS_KEY_FILE";

/// Historical name only — prefer CLI `--use-keyring` (not an env store).
pub const ENV_USE_KEYRING: &str = "SSH_CLI_USE_KEYRING";

/// Historical name only — **not** read as product store.
/// Use CLI `--lang` or `locale set` (XDG `lang`).
pub const ENV_LANG: &str = "SSH_CLI_LANG";

/// Historical name only — **not** read as product store.
/// Use `--json` / `--output-format text`.
pub const ENV_FORCE_TEXT: &str = "SSH_CLI_FORCE_TEXT";

// ── OS keyring identity ─────────────────────────────────────────────────────

/// Keyring service name (must match historical installs).
pub const KEYRING_SERVICE: &str = APP_NAME;

/// Canonical keyring user for the primary key.
pub const KEYRING_USER_PRIMARY: &str = "secrets-primary-key";

/// Legacy keyring user (read-only migration alias).
pub const KEYRING_USER_LEGACY: &str = "secrets-master-key";

// ── Network defaults (named; overridable via CLI / registry) ────────────────

/// Default SSH port when the user omits `--port` on `vps add`.
pub const DEFAULT_SSH_PORT: u16 = 22;

/// Default local bind for `tunnel --bind` (loopback-only for safety).
pub const DEFAULT_TUNNEL_BIND_ADDR: &str = "127.0.0.1";

/// Origin address advertised on SSH direct-tcpip channels for local forwards.
pub const TUNNEL_CHANNEL_ORIGIN_ADDR: &str = "127.0.0.1";

/// Origin port advertised on SSH direct-tcpip channels (`0` = ephemeral/unspecified).
pub const TUNNEL_CHANNEL_ORIGIN_PORT: u16 = 0;

/// Bind address requested from the server for `tunnel --reverse` (loopback default).
///
/// Empty would mean "all interfaces" per RFC 4254; loopback keeps the remote
/// listener private unless the operator asks for exposure, matching the local
/// `--bind` default rather than silently inverting it.
pub const DEFAULT_REVERSE_BIND_ADDR: &str = "127.0.0.1";

/// Queue depth for channels the server opens on an active reverse forward.
///
/// Bounded on purpose: the peer decides how fast these arrive, so an unbounded
/// queue would let a hostile server grow our memory without ever connecting.
pub const REVERSE_FORWARD_QUEUE_DEPTH: usize = 64;

/// Maximum bytes accepted while parsing one SOCKS5 handshake.
///
/// RFC 1928 bounds every handshake message: the greeting is at most 257 bytes
/// (2 + 255 methods) and the request at most 262 (4 + 1 length + 255 name + 2
/// port), so 519 is the largest legal handshake. The budget sits above that and
/// far below anything an attacker could use to make the proxy buffer at will.
pub const SOCKS5_HANDSHAKE_MAX_BYTES: usize = 1024;

/// SOCKS5 protocol version byte (RFC 1928).
pub const SOCKS5_VERSION: u8 = 0x05;

/// SOCKS5 "no authentication required" method (RFC 1928 §3).
pub const SOCKS5_METHOD_NO_AUTH: u8 = 0x00;

/// SOCKS5 "no acceptable methods" reply (RFC 1928 §3).
pub const SOCKS5_METHOD_NONE_ACCEPTABLE: u8 = 0xFF;

// ── Process / async timing (units in the name) ──────────────────────────────

/// Tokio runtime graceful shutdown budget after the one-shot command returns.
pub const RUNTIME_SHUTDOWN_TIMEOUT_SECS: u64 = 2;

/// Tunnel accept-loop cooperative signal poll interval.
pub const TUNNEL_SIGNAL_POLL_INTERVAL_MS: u64 = 200;

/// Grace period to drain in-flight tunnel forwards after stop.
pub const TUNNEL_FORWARD_DRAIN_TIMEOUT_SECS: u64 = 2;

/// Multi-host fan-out signal poll interval inside `map_bounded`.
pub const FAN_OUT_SIGNAL_POLL_INTERVAL_MS: u64 = 50;

// ── SSH transport / TCP dial (Rules Rust — rede) ────────────────────────────

/// Delay before starting the next Happy Eyeballs dial candidate (RFC 8305-inspired).
pub const HAPPY_EYEBALLS_ATTEMPT_DELAY_MS: u64 = 50;

/// SSH-level keepalive interval (seconds between keepalives when idle).
pub const SSH_KEEPALIVE_INTERVAL_SECS: u64 = 15;

/// Max unanswered SSH keepalives before russh closes the session.
pub const SSH_KEEPALIVE_MAX: usize = 3;

/// SSH identification string sent to the server (G-SSH-02).
///
/// Generic product id — must **not** embed the exact `russh` crate version
/// (server fingerprinting / version targeting).
pub const SSH_CLIENT_ID: &str = "SSH-2.0-ssh-cli";

/// Initial SSH channel window size (2 MiB — OpenSSH-class throughput).
pub const SSH_WINDOW_SIZE: u32 = 2 * 1024 * 1024;

/// Maximum SSH packet size (32 KiB — RFC 4253 common default).
pub const SSH_MAX_PACKET_SIZE: u32 = 32 * 1024;

/// Rekey after this many seconds of session lifetime (RFC 4253 §9).
pub const SSH_REKEY_TIME_SECS: u64 = 3600;

/// Rekey after this many bytes read or written (1 GiB — RFC 4253 §9).
pub const SSH_REKEY_BYTE_LIMIT: usize = 1 << 30;

/// Enable TCP-level `SO_KEEPALIVE` on the SSH dial socket (complements SSH KA).
pub const TCP_KEEPALIVE_ENABLED: bool = true;

/// Minimum RSA private-key size accepted for publickey auth (bits).
pub const SSH_RSA_MIN_BITS: usize = 2048;

/// Preferred RSA size; smaller (but ≥ min) keys log a warning (bits).
pub const SSH_RSA_PREFERRED_BITS: usize = 3072;

/// Windows OpenSSH agent named pipe (platform default when `--use-agent` has no path).
pub const WINDOWS_SSH_AGENT_PIPE: &str = r"\\.\pipe\openssh-ssh-agent";

// ── SFTP subsystem (G-SFTP) ─────────────────────────────────────────────────

/// SSH subsystem name for SFTP v3 (`request_subsystem`).
pub const SFTP_SUBSYSTEM: &str = "sftp";

/// Stream chunk size for SFTP file I/O (bytes). Never load whole files into RAM.
pub const SFTP_IO_CHUNK: usize = 32 * 1024;

// ── SCP wire (G-SCP) ────────────────────────────────────────────────────────

/// Stream chunk size for SCP file I/O (bytes). Never load whole files into RAM.
///
/// B4: this value used to be declared twice as a function-local `const` inside
/// `src/ssh/client_real_scp.rs`, each carrying a deferral marker admitting it
/// belonged here. It is deliberately equal to [`SFTP_IO_CHUNK`] — both transfer
/// paths stream through the same window — but kept as a distinct name so a
/// future protocol-specific tuning of one does not silently retune the other.
pub const SCP_IO_CHUNK: usize = 32 * 1024;

/// Max bytes accepted for a single SCP protocol header line.
///
/// Headers are a few dozen bytes; the cap only exists so a hostile source cannot
/// make us grow an unbounded buffer by never sending a newline.
pub const SCP_HEADER_MAX_BYTES: usize = 16 * 1024;

/// SFTP v3 permission field mask: permission bits only (no `S_IFMT` file-type bits).
///
/// Unix `st_mode` carries type in the high bits (e.g. `0o100644` for a regular
/// file). The protocol `permissions` attribute must carry permission bits only
/// (`0o7777` = mode + setuid/setgid/sticky). See G12 / G19.
pub const SFTP_PERM_MASK: u32 = 0o7777;

/// Permission mask applied to modes that arrive **from the network** (A3).
///
/// [`SFTP_PERM_MASK`] keeps setuid/setgid/sticky, which is correct when reading a
/// mode from a **local** file we own and announcing it to the server. It is *not*
/// correct in the opposite direction: a malicious or compromised server can mark a
/// remote file setuid and have the download reproduce that bit on the local disk.
/// Elevation bits must never be attacker-controlled, so inbound modes are clamped
/// to plain `rwx` triples.
pub const SFTP_PERM_MASK_UNTRUSTED: u32 = 0o0777;

/// Max characters kept from a server-sent pre-auth banner before truncation (A1).
///
/// The banner is remote input that arrives *before* authentication. It is logged
/// for diagnostics only, so a small cap is enough and keeps hostile input bounded.
pub const AUTH_BANNER_MAX_CHARS: usize = 512;

/// Max recursion depth for `sftp upload|download --recursive`.
pub const SFTP_MAX_RECURSION_DEPTH: u32 = 64;

/// Fail-closed cap on directory listing entries (`sftp ls` / recursive walk).
pub const SFTP_LIST_MAX_ENTRIES: usize = 100_000;

/// Basename used only when a multi-file source has no file name component.
/// Prefer rejecting empty names; this is a last-resort label (G-SFTP-R13).
pub const SFTP_FALLBACK_BASENAME: &str = "ssh-cli-unnamed";

// ── Agent retry policy (Rules Rust — retry/backoff; re-invoke, not in-process) ─

/// Max retries **after** the first failure for agents re-invoking on exit 74.
///
/// Total attempts = `AGENT_RETRY_MAX_RETRIES + 1` (3). Matches `docs/AGENTS.md`.
pub const AGENT_RETRY_MAX_RETRIES: u32 = 2;

/// Base backoff delay (ms) for agent full-jitter formula.
pub const AGENT_RETRY_BASE_MS: u64 = 200;

/// Ceiling for a single agent backoff sleep (ms).
pub const AGENT_RETRY_MAX_DELAY_MS: u64 = 5_000;

/// Hard cap on configured retries (prevents accidental retry storms).
pub const HARD_RETRY_MAX_RETRIES: u32 = 10;

/// Hard ceiling for a single delay (ms).
pub const HARD_RETRY_MAX_DELAY_MS: u64 = 60_000;

// ── Config lock contention ───────────────────────────────────────────────────

/// Default total wait for the sibling `config.toml.lock` before giving up (ms).
///
/// `flock` blocks indefinitely, and an indefinite block is the worst outcome
/// for a one-shot agent: it neither fails nor reports. `--lock-wait-ms`
/// overrides this; the ceiling here is the default a caller gets for free.
pub const CONFIG_LOCK_WAIT_MS: u64 = 5_000;

/// Ceiling accepted by `--lock-wait-ms` (guards against a caller pinning a
/// one-shot forever by typo).
pub const CONFIG_LOCK_WAIT_MAX_MS: u64 = 300_000;

/// Longest single sleep of the config lock wait between two checks of the stop
/// signal (ms).
///
/// Bounds how late a SIGINT/SIGTERM is noticed while the lock is contended.
pub const LOCK_WAIT_SLICE_MS: u64 = 100;

// ── Error suggestions (agent-facing, wire text) ──────────────────────────────

/// Remedy for [`crate::errors::SshCliError::Unavailable`] when the OS keyring is down.
pub const SUGGESTION_UNAVAILABLE_KEYRING: &str =
    "unlock or start the OS keyring, or fall back to XDG `secrets.key` \
     (`--secrets-key-file`); the same argv succeeds once it answers (exit 69)";

/// Remedy for [`crate::errors::SshCliError::Unavailable`] when the registry lock is held.
pub const SUGGESTION_UNAVAILABLE_CONFIG_LOCK: &str =
    "another ssh-cli invocation holds the registry lock; repeat the same argv or \
     raise --lock-wait-ms (exit 69)";

/// Remedy for [`crate::errors::SshCliError::VpsNotFound`].
pub const SUGGESTION_VPS_NOT_FOUND: &str =
    "the host is not in the registry; if an earlier `vps rename` or `vps remove` may \
     have succeeded, run `vps list` before repeating";

// ── At-rest crypto sizes (ChaCha20-Poly1305) ─────────────────────────────────

/// Primary-key length in bytes.
pub const PRIMARY_KEY_LEN_BYTES: usize = 32;

/// Primary-key hex encoding length (`2 * PRIMARY_KEY_LEN_BYTES`).
pub const PRIMARY_KEY_HEX_LEN: usize = 64;

/// ChaCha20-Poly1305 nonce length in bytes.
pub const AEAD_NONCE_LEN_BYTES: usize = 12;

/// Poly1305 authentication tag length in bytes.
pub const AEAD_TAG_LEN_BYTES: usize = 16;

/// Unix permission mode for secret files (`config.toml`, `secrets.key`, `lang`).
pub const SECRET_FILE_MODE_UNIX: u32 = 0o600;

/// Unix permission mode for secret directories (TLS identity dirs, etc.).
pub const SECRET_DIR_MODE_UNIX: u32 = 0o700;

/// Unix permission bits that expose a path to its group and to other users.
///
/// A config directory whose mode intersects this mask is "exposed"; `doctor`
/// reports it and `fs_perm::ensure_config_dir` tightens it only when safe.
pub const GROUP_OTHER_MODE_MASK_UNIX: u32 = 0o077;

/// Extension (without the dot) that marks a `--config-dir` value as the registry FILE.
///
/// `--config-dir sub/c.toml` names the file; its parent is the configuration directory.
pub const CONFIG_FILE_EXTENSION: &str = "toml";

// ── Concurrency caps (single source; clap + fan-out) ─────────────────────────

/// Minimum concurrent multi-host / fan-out units (always ≥ 1).
pub const MIN_CONCURRENCY: usize = 1;

/// Hard upper bound on concurrent multi-host sessions and SCP file channels.
///
/// Clap `--max-concurrency` / `--scp-file-concurrency` ranges must use this value.
pub const MAX_CONCURRENCY: usize = 64;

/// Alias used by the concurrency budget module (same as [`MAX_CONCURRENCY`]).
pub const HARD_CAP: usize = MAX_CONCURRENCY;

/// Documented per multi-host session RAM budget (bytes) for auto concurrency formula.
pub const RAM_PER_TASK_BYTES: u64 = 16 * 1024 * 1024;

/// I/O oversubscribe factor vs CPU count for multi-host fan-out.
pub const IO_OVERSUBSCRIBE: usize = 4;

/// Conservative CPU×IO cap when free RAM cannot be read (non-Linux).
pub const NON_LINUX_CPU_CAP: usize = 8;

// ── Wire error details (English by contract) ────────────────────────────────
//
// These are the *bodies* of failures that reach a JSON envelope, and the
// envelope schema pins them to English: see the `message` description in
// `docs/schemas/error-envelope.schema.json`, which states it stays English
// regardless of `--lang`. Agents branch on `error_code`; prose that changes
// with the operator's locale turns a stable discriminator into a moving one.
//
// They live here rather than inline because five modules raise the same three
// failures, and a phrase copied five times drifts in four of them. The human
// text mode still localizes the *label* around these bodies — that path runs
// through `i18n::localized_error_text` and is untouched.

/// Rejection when `scp upload` is pointed at anything but a regular file.
pub const SCP_UPLOAD_FILE_ONLY_MSG: &str =
    "upload only supports regular files (no directories / no -r)";

/// Rejection when `scp download` is handed an existing directory as its target.
pub const SCP_DOWNLOAD_LOCAL_NOT_DIRECTORY_MSG: &str =
    "download local path must be a file path, not an existing directory";

/// Abort raised when a signal arrives mid-transfer and the loop stops early.
pub const OPERATION_CANCELLED_MSG: &str = "operation cancelled by user";

// Compile-time invariants (const/static rules).
// ── Host-name charset and untrusted-text escaping ───────────────────────────

/// Punctuation bytes allowed in a new VPS name besides ASCII letters and digits.
///
/// The first byte of a name must be an ASCII letter or digit, so a name can
/// never be read as a flag (`-x`) or as a hidden entry (`.x`).
pub const VPS_NAME_EXTRA_BYTES: &[u8] = b"._-";

/// Unicode format and bidirectional code points that are invisible or reorder
/// the text around them, as inclusive `(first, last)` pairs.
///
/// Together with `char::is_control` this is the one set that is rejected in a
/// looked-up host name, refused in a file name, and escaped as `\u{..}` before
/// text output. Members: soft hyphen (U+00AD), Arabic letter mark (U+061C),
/// Mongolian vowel separator (U+180E), zero-width and directional marks
/// (U+200B..U+200F), embeddings and overrides (U+202A..U+202E), word joiner and
/// invisible operators (U+2060..U+2064), isolates and the deprecated format
/// controls (U+2066..U+206F), the byte order mark (U+FEFF) and the tag
/// characters (U+E0000..U+E007F) that can smuggle invisible text.
/// U+2065 is unassigned and stays out on purpose, so the range around it is split.
pub const UNTRUSTED_FORMAT_CHAR_RANGES: &[(char, char)] = &[
    ('\u{00AD}', '\u{00AD}'),
    ('\u{061C}', '\u{061C}'),
    ('\u{180E}', '\u{180E}'),
    ('\u{200B}', '\u{200F}'),
    ('\u{202A}', '\u{202E}'),
    ('\u{2060}', '\u{2064}'),
    ('\u{2066}', '\u{206F}'),
    ('\u{FEFF}', '\u{FEFF}'),
    ('\u{E0000}', '\u{E007F}'),
];

const _: () = assert!(!VPS_NAME_EXTRA_BYTES.is_empty());
const _: () = assert!(!UNTRUSTED_FORMAT_CHAR_RANGES.is_empty());

const _: () = assert!(!APP_NAME.is_empty());
const _: () = assert!(!CONFIG_FILE_NAME.is_empty());
const _: () = assert!(!SECRETS_KEY_FILE_NAME.is_empty());
const _: () = assert!(!KNOWN_HOSTS_FILE_NAME.is_empty());
const _: () = assert!(!ACTIVE_VPS_FILE_NAME.is_empty());
const _: () = assert!(!ENV_HOME.is_empty());
const _: () = assert!(DEFAULT_SSH_PORT > 0);
const _: () = assert!(!DEFAULT_TUNNEL_BIND_ADDR.is_empty());
const _: () = assert!(RUNTIME_SHUTDOWN_TIMEOUT_SECS > 0);
const _: () = assert!(TUNNEL_SIGNAL_POLL_INTERVAL_MS > 0);
const _: () = assert!(TUNNEL_FORWARD_DRAIN_TIMEOUT_SECS > 0);
const _: () = assert!(REVERSE_FORWARD_QUEUE_DEPTH > 0);
// 519 is the largest handshake RFC 1928 permits; a smaller cap would reject
// valid clients, which is a bug that only shows up against unusual peers.
const _: () = assert!(SOCKS5_HANDSHAKE_MAX_BYTES >= 519);
const _: () = assert!(SOCKS5_VERSION == 0x05);
const _: () = assert!(FAN_OUT_SIGNAL_POLL_INTERVAL_MS > 0);
const _: () = assert!(HAPPY_EYEBALLS_ATTEMPT_DELAY_MS > 0);
const _: () = assert!(SSH_KEEPALIVE_INTERVAL_SECS > 0);
const _: () = assert!(SSH_KEEPALIVE_MAX > 0);
const _: () = assert!(!SSH_CLIENT_ID.is_empty());
const _: () = assert!(SSH_WINDOW_SIZE > 0);
const _: () = assert!(SSH_MAX_PACKET_SIZE > 0);
const _: () = assert!(SSH_REKEY_TIME_SECS > 0);
const _: () = assert!(SSH_REKEY_BYTE_LIMIT > 0);
const _: () = assert!(SSH_RSA_MIN_BITS >= 2048);
const _: () = assert!(SSH_RSA_PREFERRED_BITS >= SSH_RSA_MIN_BITS);
const _: () = assert!(!WINDOWS_SSH_AGENT_PIPE.is_empty());
const _: () = assert!(!SFTP_SUBSYSTEM.is_empty());
const _: () = assert!(SFTP_IO_CHUNK > 0);
const _: () = assert!(SFTP_PERM_MASK == 0o7777);
const _: () = assert!(SFTP_MAX_RECURSION_DEPTH > 0);
const _: () = assert!(SFTP_LIST_MAX_ENTRIES > 0);
const _: () = assert!(!SFTP_FALLBACK_BASENAME.is_empty());
const _: () = assert!(AGENT_RETRY_MAX_RETRIES > 0);
const _: () = assert!(AGENT_RETRY_BASE_MS >= 50);
const _: () = assert!(AGENT_RETRY_MAX_DELAY_MS >= AGENT_RETRY_BASE_MS);
const _: () = assert!(HARD_RETRY_MAX_RETRIES >= AGENT_RETRY_MAX_RETRIES);
const _: () = assert!(HARD_RETRY_MAX_DELAY_MS >= AGENT_RETRY_MAX_DELAY_MS);
const _: () = assert!(PRIMARY_KEY_HEX_LEN == PRIMARY_KEY_LEN_BYTES * 2);
const _: () = assert!(AEAD_NONCE_LEN_BYTES == 12);
const _: () = assert!(AEAD_TAG_LEN_BYTES == 16);
const _: () = assert!(SECRET_FILE_MODE_UNIX == 0o600);
const _: () = assert!(!SCP_UPLOAD_FILE_ONLY_MSG.is_empty());
const _: () = assert!(!SCP_DOWNLOAD_LOCAL_NOT_DIRECTORY_MSG.is_empty());
const _: () = assert!(!OPERATION_CANCELLED_MSG.is_empty());
const _: () = assert!(LOCK_WAIT_SLICE_MS > 0);
const _: () = assert!(!SUGGESTION_UNAVAILABLE_KEYRING.is_empty());
const _: () = assert!(!SUGGESTION_UNAVAILABLE_CONFIG_LOCK.is_empty());
const _: () = assert!(!SUGGESTION_VPS_NOT_FOUND.is_empty());
