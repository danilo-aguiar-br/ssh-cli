// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! SSH tunnelling with a mandatory deadline (bounded one-shot).
//!
//! Four modes share this entry point, split across submodules because they share
//! a lifecycle but not a data path:
//!
//! | Mode | Who listens | Submodule |
//! |---|---|---|
//! | local forward | this process | `local` |
//! | SOCKS5 proxy | this process | `local` + `socks` |
//! | remote Unix socket | this process | `local` + `streamlocal` |
//! | reverse forward | the SSH server | `reverse` |
//!
//! What lives *here* is everything the modes genuinely share: the deadline
//! wrapper, the counters it reads after cancellation, the exposure guards, and
//! the two helpers (`pump`, `drain_forwards`) that every mode ends up calling.

mod local;
mod reverse;
mod socks;
mod stats;
mod streamlocal;

pub use local::ForwardKind;
pub use stats::TunnelStats;
pub use streamlocal::validate_remote_socket;

use crate::errors::SshCliError;
use crate::output;
use crate::ssh::client::{SshClient, SshClientTrait};
use crate::vps::find_by_name;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Which tunnel the caller asked for.
#[derive(Debug, Clone)]
pub enum TunnelMode {
    /// Local listener forwarding to a fixed remote `host:port`.
    Local {
        /// Remote host, resolved by the SSH server.
        remote_host: String,
        /// Remote port.
        remote_port: u16,
    },
    /// Local listener speaking SOCKS5; the client names a target per connection.
    Socks5,
    /// Local listener forwarding to a remote Unix domain socket.
    StreamLocal {
        /// Absolute path of the socket on the remote host.
        socket_path: String,
    },
    /// Server-side listener delivering connections back to a local port.
    Reverse {
        /// Address the server is asked to bind.
        remote_bind: String,
        /// Port the server is asked to bind (`0` = server allocates).
        remote_port: u16,
    },
}

impl TunnelMode {
    /// Wire label used in `tunnel_listening` / `tunnel_closed`.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Local { .. } => "local",
            Self::Socks5 => "socks5",
            Self::StreamLocal { .. } => "streamlocal",
            Self::Reverse { .. } => "reverse",
        }
    }
}

/// SSH credential overrides for one tunnel invocation.
#[derive(Default)]
pub struct TunnelAuth {
    /// Password override.
    pub password: Option<secrecy::SecretString>,
    /// Private key path override.
    pub key: Option<String>,
    /// Key passphrase override.
    pub key_passphrase: Option<secrecy::SecretString>,
    /// Authenticate through an SSH agent.
    pub use_agent: bool,
    /// Explicit agent socket path.
    pub agent_socket: Option<String>,
}

/// Everything one `tunnel` invocation needs.
///
/// Grouped into a struct rather than passed as fifteen positional arguments:
/// with that many `u16`/`bool`/`Option<String>` parameters in a row, a
/// transposition compiles cleanly and only shows up as a tunnel pointing
/// somewhere unintended.
pub struct TunnelRequest {
    /// Registry name of the host.
    pub vps_name: String,
    /// Local port to bind (`0` = ephemeral) — for reverse, the local target port.
    pub local_port: u16,
    /// Tunnel mode.
    pub mode: TunnelMode,
    /// Alternate config directory.
    pub config_override: Option<PathBuf>,
    /// Credential overrides.
    pub auth: TunnelAuth,
    /// Mandatory deadline in milliseconds.
    pub timeout_ms: u64,
    /// Replace a diverging host key in TOFU `known_hosts`.
    pub replace_host_key: bool,
    /// Agent-first JSON output.
    pub json: bool,
    /// Local bind address (ignored in reverse mode, where the server binds).
    pub bind_addr: String,
    /// Explicit acknowledgement that a routable bind exposes the service.
    pub accept_network_exposure: bool,
}

/// Runs the `tunnel` subcommand with a mandatory timeout.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] for a zero deadline, an unacknowledged
/// routable bind or an invalid remote socket; [`SshCliError::VpsNotFound`] for an
/// unknown host; [`SshCliError::SshTimeout`] when the deadline expires *before*
/// the listener is up.
pub async fn run_tunnel(request: TunnelRequest) -> Result<()> {
    let TunnelRequest {
        vps_name,
        local_port,
        mode,
        config_override,
        auth,
        timeout_ms,
        replace_host_key,
        json,
        bind_addr,
        accept_network_exposure,
    } = request;

    if timeout_ms == 0 {
        return Err(SshCliError::InvalidArgument(
            "tunnel requires --timeout-ms > 0 (bounded one-shot)".to_string(),
        )
        .into());
    }

    // G-TUN-R13: binding outside loopback publishes the forwarded remote service to
    // the whole local network with no additional authentication. The default is
    // loopback for exactly that reason, but any address used to be accepted in
    // silence — no prompt, no warning, not even a record in the JSON event. For an
    // agent-driven CLI a mis-inferred flag could expose a production database.
    // This mirrors the explicit-risk gate the project already applies to
    // `--replace-host-key`, and it fails before any network I/O is paid for.
    match &mode {
        TunnelMode::Reverse { remote_bind, .. } => {
            // In reverse mode the exposed surface is the *server's* listener, so
            // guarding the local bind would check the wrong end entirely.
            guard_remote_exposure(remote_bind, accept_network_exposure)?;
        }
        _ => guard_network_exposure(&bind_addr, accept_network_exposure)?,
    }
    if let TunnelMode::StreamLocal { socket_path } = &mode {
        validate_remote_socket(socket_path)?;
    }

    let vps = find_by_name(config_override.as_deref(), &vps_name)?
        .ok_or_else(|| SshCliError::VpsNotFound(vps_name.clone()))?;

    let path = crate::vps::resolve_config_path(config_override.as_deref())?;
    let cfg = resolve_tunnel_connection(vps, auth, Some(&path), replace_host_key);

    tracing::info!(
        vps = %vps_name,
        local_port,
        mode = mode.label(),
        timeout_ms,
        "starting SSH tunnel with deadline"
    );

    // GAP-SSH-IO-006: banners only on human TTY; agents/pipes do not pollute stdout.
    // GAP-SSH-IO-008: in JSON, zero prose — structured event after bind.
    // Banner with effective port is post-bind (TUN-003: port 0 is ephemeral).
    if !json {
        // `TunnelPressCtrlC` already existed and was bypassed by an English literal,
        // so its pt-BR translation was unreachable — a translated string nobody could
        // ever see. Routing through it is the fix; a second near-identical variant
        // would have preserved the duplication instead of removing it.
        output::print_human_banner(&crate::i18n::t(crate::i18n::Message::TunnelPressCtrlC));
    }

    // GAP-SSH-TUN-001: deadline covers connect + loop (not only the accept loop).
    // GAP-SSH-TUN-002: if the local listener is already up, deadline end is one-shot success
    // (not SshTimeout/exit 74). Timeout before bind (slow connect) remains an error.
    // Interior mutability: Arc<AtomicBool> shares the "listener up" bit between
    // the timeout wrapper and the accept loop (Release store / Acquire load).
    // Not RefCell/Mutex — a single independent flag is enough.
    let bound = Arc::new(AtomicBool::new(false));
    let bound_flag = Arc::clone(&bound);
    let stats = Arc::new(TunnelStats::default());
    let stats_loop = Arc::clone(&stats);
    let started = std::time::Instant::now();
    let mode_label = mode.label();
    let bind_for_event = match &mode {
        TunnelMode::Reverse { remote_bind, .. } => remote_bind.clone(),
        _ => bind_addr.clone(),
    };

    let result = tokio::time::timeout(Duration::from_millis(timeout_ms), async {
        let client: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
        serve_mode(
            ServeContext {
                // Cloned: `emit_closed` below still needs the name after this
                // coroutine takes ownership.
                vps_name: vps_name.clone(),
                local_port,
                timeout_ms,
                json,
                bind_addr,
                bound_flag: Some(bound_flag),
                stats: Some(stats_loop),
            },
            mode,
            client,
        )
        .await
    })
    .await;

    // G-TUN-R07: emitted on every ending. Placing this in the wrapper rather than in
    // the loop is what makes the deadline path work at all — there the loop future is
    // cancelled mid-poll and its own tail never runs.
    let emit_closed = |reason| {
        if json && bound.load(Ordering::Acquire) {
            // B3: the printer now takes the already-built DTO, so the event is
            // constructed exactly once and stays inspectable by tests.
            let event = output::build_tunnel_closed(output::TunnelClosedInput {
                vps: &vps_name,
                reason,
                bind: &bind_for_event,
                local_port: u16::try_from(stats.effective_port.load(Ordering::Acquire))
                    .unwrap_or(local_port),
                forwards_served: stats.forwards_served.load(Ordering::Relaxed),
                capacity_waits: stats.capacity_waits.load(Ordering::Relaxed),
                duration_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                mode: mode_label,
            });
            // R10 shape: never discard the Result. This event is the *only* thing that
            // distinguishes `deadline`, `signal` and `accept_error`, which all end with
            // exit 0. Swallowing the error would hand an agent a successful exit with no
            // way to learn which ending occurred, so the failure goes to stderr.
            if let Err(e) = output::print_tunnel_closed_json(&event) {
                tracing::warn!(err = %e, "failed to emit tunnel_closed event");
            }
        }
    };

    match result {
        Ok(inner) => {
            emit_closed(stats.close_reason());
            inner
        }
        Err(_) if bound.load(Ordering::Acquire) => {
            tracing::info!(timeout_ms, "tunnel ended by one-shot deadline (success)");
            emit_closed(crate::json_wire::TunnelCloseReason::Deadline);
            Ok(())
        }
        Err(_) => {
            tracing::warn!(timeout_ms, "tunnel timeout before local bind");
            Err(SshCliError::SshTimeout(timeout_ms).into())
        }
    }
}

/// Builds the connection config for a tunnel from an already-loaded registry record.
///
/// G-QA-R02: `run_tunnel` had dependency injection only for the accept loop, which was
/// the part that needed a socket. Everything between the argument guards and that loop
/// — override application and config assembly — stayed welded to `find_by_name`, so it
/// could only be exercised against a real host. That is exactly the range where the E3
/// bug lived: `--use-agent` and `--agent-socket` parsed, were dropped on the floor, and
/// a host registered for agent auth simply could not open a tunnel while exec and scp
/// could. Taking the record as an argument makes the whole range testable offline.
///
/// The registry's own `timeout` is deliberately *not* consulted: a tunnel is bounded by
/// `--timeout-ms`, and letting the host record shorten or extend that would make the
/// one-shot deadline depend on state the caller never mentioned.
#[must_use]
pub fn resolve_tunnel_connection(
    mut vps: crate::vps::model::VpsRecord,
    auth: TunnelAuth,
    config_toml: Option<&std::path::Path>,
    replace_host_key: bool,
) -> crate::ssh::client::ConnectionConfig {
    // GAP-SSH-CLI-005 / M3: parity with exec/scp via apply_overrides (password/key/passphrase).
    crate::vps::apply_overrides(
        &mut vps,
        crate::vps::AuthOverrides {
            password: auth.password,
            key_path: auth.key,
            key_passphrase: auth.key_passphrase,
            use_agent: auth.use_agent,
            agent_socket: auth.agent_socket,
            // E3: the tunnel deliberately does NOT override the registry timeout.
            // `--timeout-ms` is the tunnel's own deadline, not the SSH connect
            // budget; conflating them silently shortened long-lived forwards.
            ..Default::default()
        },
    );
    crate::vps::build_connection_config(&vps, config_toml, replace_host_key)
}

/// Everything an already-connected client needs to serve one tunnel, minus the
/// mode-specific destination.
///
/// # Why a struct (B3)
///
/// The three serve entry points below took nine and ten positional parameters,
/// with `u16`, `u64` and `bool` sitting next to each other. `local_port` and
/// `remote_port` are both `u16`; swapping them compiles and binds the wrong
/// side of the tunnel. The one lint that measures this — `too_many_arguments` —
/// was suppressed on all three, so nothing reported it.
pub struct ServeContext {
    /// Registry name of the relay host.
    pub vps_name: String,
    /// Local port to bind (`0` = OS-assigned).
    pub local_port: u16,
    /// Mandatory deadline in milliseconds.
    pub timeout_ms: u64,
    /// Agent-first JSON output.
    pub json: bool,
    /// Local bind address.
    pub bind_addr: String,
    /// Set once the listener is bound (readiness handshake for callers).
    pub bound_flag: Option<Arc<AtomicBool>>,
    /// Lifetime counters published in the `tunnel_closed` event.
    pub stats: Option<Arc<TunnelStats>>,
}

/// Routes an already-connected client into the loop its mode requires.
async fn serve_mode(
    ctx: ServeContext,
    mode: TunnelMode,
    client: Box<dyn SshClientTrait>,
) -> Result<()> {
    let ServeContext {
        vps_name,
        local_port,
        timeout_ms,
        json,
        bind_addr,
        bound_flag,
        stats,
    } = ctx;
    let (vps_name, bind_addr) = (vps_name.as_str(), bind_addr.as_str());
    match mode {
        TunnelMode::Reverse {
            remote_bind,
            remote_port,
        } => {
            reverse::serve(
                reverse::ReverseServe {
                    vps_name: vps_name.to_string(),
                    remote_bind,
                    remote_port,
                    // The delivery target is always loopback: a reverse tunnel exists
                    // to reach a service on *this* machine, and letting the remote
                    // side steer us at an arbitrary local address would turn the
                    // tunnel into an outbound port scanner.
                    local_host: crate::constants::DEFAULT_TUNNEL_BIND_ADDR.to_string(),
                    local_port,
                    timeout_ms,
                    json,
                },
                client,
                bound_flag,
                stats,
            )
            .await
        }
        other => {
            let kind = match other {
                TunnelMode::Local {
                    remote_host,
                    remote_port,
                } => ForwardKind::Tcp {
                    host: remote_host,
                    port: remote_port,
                },
                TunnelMode::Socks5 => ForwardKind::Socks5,
                TunnelMode::StreamLocal { socket_path } => ForwardKind::StreamLocal { socket_path },
                TunnelMode::Reverse { .. } => unreachable!("handled by the arm above"),
            };
            local::serve(
                local::LocalServe {
                    vps_name: vps_name.to_string(),
                    local_port,
                    bind_addr: bind_addr.to_string(),
                    timeout_ms,
                    json,
                    kind,
                },
                client,
                bound_flag,
                stats,
            )
            .await
        }
    }
}

/// Rejects a non-loopback bind unless the caller explicitly accepted the risk.
///
/// Pure and side-effect free so the policy is unit-testable without a socket.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] (exit 64) when the address is routable and
/// `accepted` is false, or when the address cannot be parsed.
pub fn guard_network_exposure(bind_addr: &str, accepted: bool) -> Result<(), SshCliError> {
    let parsed: std::net::IpAddr = bind_addr.parse().map_err(|_| {
        SshCliError::InvalidArgument(format!("invalid --bind address `{bind_addr}`"))
    })?;
    if parsed.is_loopback() || accepted {
        if !parsed.is_loopback() {
            tracing::warn!(
                bind = %bind_addr,
                "tunnel bound outside loopback: the forwarded remote service is reachable from the local network"
            );
        }
        return Ok(());
    }
    Err(SshCliError::InvalidArgument(format!(
        "--bind {bind_addr} exposes the forwarded service to the network; \
         pass --i-accept-network-exposure to proceed"
    )))
}

/// Rejects a reverse forward that would publish a *remote* listener.
///
/// The mirror of [`guard_network_exposure`] for the inverted direction. The
/// address is not parsed as an IP because RFC 4254 also assigns meaning to names
/// and to the empty string (all interfaces), so an IP parser would reject the
/// very forms that matter most.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] (exit 64) when the remote bind is routable
/// and `accepted` is false.
pub fn guard_remote_exposure(remote_bind: &str, accepted: bool) -> Result<(), SshCliError> {
    let loopback = matches!(remote_bind, "127.0.0.1" | "::1" | "localhost");
    if loopback || accepted {
        if !loopback {
            tracing::warn!(
                bind = %remote_bind,
                "reverse tunnel bound outside remote loopback: the local service is reachable from the remote network"
            );
        }
        return Ok(());
    }
    Err(SshCliError::InvalidArgument(format!(
        "--reverse binding `{remote_bind}` on the server exposes your local service to \
         the remote network; pass --i-accept-network-exposure to proceed"
    )))
}

/// Copies bytes both ways until either side closes.
///
/// G-TUN-R10 / G-TUN-R11: the previous implementation ran two `tokio::io::copy`
/// futures under `join!` and discarded both `Result`s with `let _ =`, so a
/// connection that died mid-transfer was indistinguishable from one that finished
/// cleanly — at any verbosity. `copy_bidirectional` returns the byte counts for
/// both directions *and* the error, and already performs the EOF-triggered
/// `shutdown()` on the opposing side that the manual version open-coded.
///
/// # Errors
/// [`SshCliError::Io`] when the copy fails mid-stream.
pub(crate) async fn pump<L>(
    mut local: L,
    mut channel: Box<dyn crate::ssh::client::TunnelChannel>,
    peer: &str,
    peer_port: u16,
) -> Result<()>
where
    L: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    match tokio::io::copy_bidirectional(&mut local, &mut *channel).await {
        Ok((to_remote, to_local)) => {
            tracing::debug!(
                bytes_to_remote = to_remote,
                bytes_to_local = to_local,
                "tunnel forward completed"
            );
            Ok(())
        }
        Err(e) => {
            // Surfaced as a warning rather than swallowed: "the tunnel connects but no
            // data arrives" is undiagnosable when this path is silent.
            tracing::warn!(err = %e, %peer, peer_port, "tunnel forward copy failed");
            Err(SshCliError::Io(e).into())
        }
    }
}

/// Drains in-flight forwards with a bounded grace, aborting what will not finish.
pub(crate) async fn drain_forwards(forwards: &mut tokio::task::JoinSet<()>) {
    if crate::signals::is_force_exit() {
        tracing::info!("force-exit: aborting tunnel forwards");
        forwards.abort_all();
    }
    // Bounded drain: cooperative cancel gets a short grace; force already aborted.
    let drain = tokio::time::timeout(
        Duration::from_secs(crate::constants::TUNNEL_FORWARD_DRAIN_TIMEOUT_SECS),
        async { while forwards.join_next().await.is_some() {} },
    )
    .await;
    if drain.is_err() {
        tracing::warn!("tunnel forward drain timed out; aborting remainder");
        forwards.abort_all();
        while forwards.join_next().await.is_some() {}
    }
}

/// Testable local-forward loop (see [`run_tunnel_with_client_stats`] for counters).
///
/// # Errors
/// Propagates bind and forwarding failures from the local accept loop.
pub async fn run_tunnel_with_client(
    mut ctx: ServeContext,
    remote_host: &str,
    remote_port: u16,
    client: Box<dyn SshClientTrait>,
) -> Result<()> {
    ctx.stats = None;
    run_tunnel_with_client_stats(ctx, remote_host, remote_port, client).await
}

/// Testable local-forward loop that publishes lifetime counters into
/// [`ServeContext::stats`].
///
/// # Errors
/// Propagates bind and forwarding failures from the local accept loop.
pub async fn run_tunnel_with_client_stats(
    ctx: ServeContext,
    remote_host: &str,
    remote_port: u16,
    client: Box<dyn SshClientTrait>,
) -> Result<()> {
    local::serve(
        local::LocalServe {
            vps_name: ctx.vps_name,
            local_port: ctx.local_port,
            bind_addr: ctx.bind_addr,
            timeout_ms: ctx.timeout_ms,
            json: ctx.json,
            kind: ForwardKind::Tcp {
                host: remote_host.to_string(),
                port: remote_port,
            },
        },
        client,
        ctx.bound_flag,
        ctx.stats,
    )
    .await
}

#[cfg(test)]
mod tests;
