// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Named domain constants (Rules Rust — proibição de hardcode).
//!
//! Compile-time values with semantic names live here. Runtime host credentials,
//! per-VPS timeouts, and primary-key material are **never** hardcoded: they are
//! loaded once from CLI / OS keyring / XDG files under
//! [`APP_NAME`] (`directories::ProjectDirs` + optional `--config-dir`).
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

// Compile-time invariants (const/static rules).
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
