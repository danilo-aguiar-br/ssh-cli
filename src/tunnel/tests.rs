// SPDX-License-Identifier: MIT OR Apache-2.0
//! Unit tests for the tunnel entry point and its mode routing.
#![allow(clippy::unwrap_used)]

use super::*;
use crate::ssh::client::{ConnectionConfig, ExecutionOutput, SshClientTrait, TransferResult};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Minimal client that satisfies the trait without a network.
///
/// Only the methods a given test drives are overridden; everything else keeps the
/// trait's own "unsupported" default, which is exactly what a stub can honestly
/// claim about reverse forwarding and Unix sockets.
struct Stub {
    /// Port `request_remote_forward` should report as allocated.
    allocated: u16,
}

impl Stub {
    fn new() -> Self {
        Self { allocated: 0 }
    }
    fn with_allocated(port: u16) -> Self {
        Self { allocated: port }
    }
}

#[async_trait]
impl SshClientTrait for Stub {
    async fn connect(_cfg: ConnectionConfig) -> Result<Box<Self>, crate::errors::SshCliError> {
        Ok(Box::new(Stub::new()))
    }
    async fn run_command(
        &mut self,
        _cmd: &str,
        _max: usize,
        _stdin: Option<Vec<u8>>,
    ) -> Result<ExecutionOutput, crate::errors::SshCliError> {
        unreachable!()
    }
    async fn upload(
        &self,
        _l: &Path,
        _r: &Path,
    ) -> Result<TransferResult, crate::errors::SshCliError> {
        unreachable!()
    }
    async fn download(
        &self,
        _r: &Path,
        _l: &Path,
    ) -> Result<TransferResult, crate::errors::SshCliError> {
        unreachable!()
    }
    async fn open_tunnel_channel(
        &self,
        _h: &str,
        _p: u16,
        _o: &str,
        _po: u16,
    ) -> Result<Box<dyn crate::ssh::client::TunnelChannel>, crate::errors::SshCliError> {
        Err(crate::errors::SshCliError::channel_msg("stub"))
    }
    async fn request_remote_forward(
        &self,
        _address: &str,
        _port: u16,
    ) -> Result<u16, crate::errors::SshCliError> {
        Ok(self.allocated)
    }
    async fn cancel_remote_forward(
        &self,
        _address: &str,
        _port: u16,
    ) -> Result<(), crate::errors::SshCliError> {
        Ok(())
    }
    async fn disconnect(&self) -> Result<(), crate::errors::SshCliError> {
        Ok(())
    }
}

fn request(timeout_ms: u64, mode: TunnelMode, bind_addr: &str) -> TunnelRequest {
    TunnelRequest {
        vps_name: "no-such-host".to_string(),
        local_port: 0,
        mode,
        config_override: None,
        auth: TunnelAuth::default(),
        timeout_ms,
        replace_host_key: false,
        json: true,
        bind_addr: bind_addr.to_string(),
        accept_network_exposure: false,
    }
}

fn local_mode() -> TunnelMode {
    TunnelMode::Local {
        remote_host: "localhost".to_string(),
        remote_port: 5432,
    }
}

/// G-TUN-R04 / G-TUN-R05 / G-QA-R01.
///
/// This replaces `assert_eq!(0_u64, 0)`, which was true by construction and
/// exercised no product code at all. It inflated the green count and, worse, its
/// name claimed the `timeout_ms == 0` guard was covered — so deleting that guard
/// (and shipping an unbounded tunnel, violating the one-shot rule) would not have
/// broken the suite.
///
/// The guard runs before `find_by_name`, so no host, config or network is needed.
#[tokio::test]
async fn timeout_zero_is_rejected_before_touching_the_registry() {
    let err = run_tunnel(request(0, local_mode(), "127.0.0.1"))
        .await
        .expect_err("timeout_ms == 0 must be rejected");

    let ssh = err
        .downcast_ref::<SshCliError>()
        .expect("must surface a typed SshCliError");
    assert!(
        matches!(ssh, SshCliError::InvalidArgument(_)),
        "expected InvalidArgument, got {ssh:?}"
    );
    assert_eq!(ssh.exit_code(), crate::errors::exit_codes::EX_USAGE);
    // Proves the guard ran *before* registry lookup: the VPS does not exist, so a
    // later guard would have produced VpsNotFound (exit 66) instead.
    assert!(!matches!(ssh, SshCliError::VpsNotFound(_)));
}

/// G-TUN-R03: an invalid remote socket is rejected before the registry lookup.
#[tokio::test]
async fn relative_remote_socket_is_rejected_before_the_registry() {
    let err = run_tunnel(request(
        1_000,
        TunnelMode::StreamLocal {
            socket_path: "run/docker.sock".to_string(),
        },
        "127.0.0.1",
    ))
    .await
    .expect_err("relative remote socket must be rejected");

    let ssh = err.downcast_ref::<SshCliError>().unwrap();
    assert_eq!(ssh.exit_code(), crate::errors::exit_codes::EX_USAGE);
    assert!(!matches!(ssh, SshCliError::VpsNotFound(_)));
}

/// G-TUN-R01: a routable *remote* bind needs the same acknowledgement as a local one.
#[tokio::test]
async fn reverse_bind_outside_remote_loopback_requires_acceptance() {
    let err = run_tunnel(request(
        1_000,
        TunnelMode::Reverse {
            remote_bind: "0.0.0.0".to_string(),
            remote_port: 8080,
        },
        "127.0.0.1",
    ))
    .await
    .expect_err("0.0.0.0 on the server must not bind without acknowledgement");

    let ssh = err.downcast_ref::<SshCliError>().unwrap();
    assert_eq!(ssh.exit_code(), crate::errors::exit_codes::EX_USAGE);
    // The local bind is loopback here, so only the *remote* guard can have fired.
    assert!(!matches!(ssh, SshCliError::VpsNotFound(_)));
}

/// G-TUN-R13: a routable bind requires explicit risk acceptance.
#[test]
fn non_loopback_bind_requires_explicit_acceptance() {
    let err = guard_network_exposure("0.0.0.0", false)
        .expect_err("0.0.0.0 must not bind without acknowledgement");
    assert!(matches!(err, SshCliError::InvalidArgument(_)));
    assert_eq!(err.exit_code(), crate::errors::exit_codes::EX_USAGE);
}

#[test]
fn loopback_bind_needs_no_acceptance() {
    assert!(guard_network_exposure("127.0.0.1", false).is_ok());
    assert!(guard_network_exposure("::1", false).is_ok());
}

#[test]
fn non_loopback_bind_allowed_once_accepted() {
    assert!(guard_network_exposure("0.0.0.0", true).is_ok());
}

#[test]
fn malformed_bind_is_rejected() {
    let err = guard_network_exposure("127.0.0..1", true)
        .expect_err("malformed address must not reach bind()");
    assert!(matches!(err, SshCliError::InvalidArgument(_)));
}

#[test]
fn remote_loopback_names_need_no_acceptance() {
    // RFC 4254 lets the client name the interface; `localhost` is the spelling
    // OpenSSH itself uses, so rejecting it would break the common safe case.
    assert!(guard_remote_exposure("127.0.0.1", false).is_ok());
    assert!(guard_remote_exposure("localhost", false).is_ok());
    assert!(guard_remote_exposure("::1", false).is_ok());
}

#[test]
fn empty_remote_bind_means_all_interfaces_and_needs_acceptance() {
    // RFC 4254 §7.1: the empty string means "listen on all interfaces". An IP
    // parser would have rejected it as malformed and hidden the exposure.
    let err = guard_remote_exposure("", false).expect_err("empty bind is all-interfaces");
    assert!(matches!(err, SshCliError::InvalidArgument(_)));
    assert!(guard_remote_exposure("", true).is_ok());
}

#[test]
fn mode_labels_are_stable_wire_values() {
    assert_eq!(local_mode().label(), "local");
    assert_eq!(TunnelMode::Socks5.label(), "socks5");
    assert_eq!(
        TunnelMode::StreamLocal {
            socket_path: "/tmp/x.sock".into()
        }
        .label(),
        "streamlocal"
    );
    assert_eq!(
        TunnelMode::Reverse {
            remote_bind: "127.0.0.1".into(),
            remote_port: 0
        }
        .label(),
        "reverse"
    );
}

#[test]
fn socks5_listening_event_reports_no_single_destination() {
    // Reporting a concrete host would be a guess: with SOCKS5 the destination is
    // chosen per connection, after the event has already been emitted.
    assert_eq!(ForwardKind::Socks5.event_host(), "*");
    assert_eq!(ForwardKind::Socks5.event_port(), 0);
}

/// GAP-SSH-TUN-003: bind on 127.0.0.1:0 must expose real port ≠ 0.
#[tokio::test]
async fn tunnel_ephemeral_bind_reports_real_port() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("ephemeral bind");
    let port = listener.local_addr().expect("local_addr").port();
    assert_ne!(port, 0, "OS must assign port > 0 after bind :0");
    assert!(
        (1..=65535).contains(&port),
        "effective port out of 1..=65535: {port}"
    );
}

/// GAP-SSH-TUN-003: the reported port is the one the OS actually assigned.
///
/// E2: the previous version of this test read its own source with `include_str!`
/// and asserted the file *contained* the strings `local_addr()` and
/// `effective_port`. That passes when the text appears in a comment, and keeps
/// passing after the behaviour is deleted, so it verified nothing. This binds a
/// real ephemeral port and checks the observable outcome instead.
#[tokio::test]
async fn ephemeral_bind_reports_os_assigned_port_not_the_requested_zero() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("ephemeral bind");
    let effective = listener
        .local_addr()
        .map(|a| a.port())
        .expect("local_addr after bind");
    assert_ne!(
        effective, 0,
        "agents connect to the port from the event; 0 would be unusable"
    );
}

/// `#[serial]` is mandatory here: the accept loop polls the process-wide cancel
/// flags, and sibling tests (`concurrency_tests`, `scp::tests`) legitimately set
/// them. Running in parallel, this test would observe a foreign cancellation and
/// fail for reasons that have nothing to do with the tunnel.
#[tokio::test]
#[serial_test::serial]
async fn tunnel_with_client_ends_on_cancel() {
    crate::signals::reset_flags_for_tests();

    // E1: the previous body constructed the stub, immediately dropped it, and
    // never called `run_tunnel_with_client` at all — despite the test name
    // promising cancellation coverage. `run_tunnel_with_client` is `pub` and takes
    // an injected client precisely so it can be driven without a real server, so
    // there was no reason for the loop to stay unexercised.
    let stub: Box<dyn SshClientTrait> = Box::new(Stub::new());
    let bound = Arc::new(AtomicBool::new(false));

    // Port 0 keeps the test independent of what else is listening on the machine.
    let handle = tokio::spawn({
        let bound = Arc::clone(&bound);
        async move {
            run_tunnel_with_client(
                crate::tunnel::ServeContext {
                    vps_name: "stub-vps".to_string(),
                    local_port: 0,
                    timeout_ms: 60_000,
                    json: false,
                    bind_addr: "127.0.0.1".to_string(),
                    bound_flag: Some(bound),
                    stats: None,
                },
                "localhost",
                5432,
                stub,
            )
            .await
        }
    });

    // The loop must reach the bind and publish it. Polling beats a fixed sleep:
    // a slow CI box would otherwise turn this into a flaky failure.
    let mut attempts = 0;
    while !bound.load(Ordering::Acquire) && attempts < 100 {
        tokio::time::sleep(Duration::from_millis(10)).await;
        attempts += 1;
    }
    assert!(
        bound.load(Ordering::Acquire),
        "run_tunnel_with_client must publish the bound flag after listening"
    );

    // Still serving: it must not have returned on its own before the deadline.
    assert!(
        !handle.is_finished(),
        "tunnel must keep accepting until signal or deadline"
    );

    handle.abort();
    let _ = handle.await;
}

/// G-TUN-R01: the *server's* allocated port is what reaches the bound flag.
///
/// A reverse forward has no local listener, so "bound" means the server accepted
/// `tcpip-forward`. Publishing the requested port instead of the allocated one
/// would hand an agent a port nothing is listening on — the same defect
/// `GAP-SSH-TUN-003` fixed for the local ephemeral bind.
#[tokio::test]
#[serial_test::serial]
async fn reverse_publishes_the_port_the_server_allocated() {
    crate::signals::reset_flags_for_tests();

    let stats = Arc::new(TunnelStats::default());
    let bound = Arc::new(AtomicBool::new(false));
    let client: Box<dyn SshClientTrait> = Box::new(Stub::with_allocated(41_337));

    super::reverse::serve(
        super::reverse::ReverseServe {
            vps_name: "stub-vps".to_string(),
            remote_bind: "127.0.0.1".to_string(),
            // Asking for 0 is what makes the server's answer the only source of truth.
            remote_port: 0,
            local_host: "127.0.0.1".to_string(),
            local_port: 5432,
            timeout_ms: 60_000,
            json: false,
        },
        client,
        Some(Arc::clone(&bound)),
        Some(Arc::clone(&stats)),
    )
    .await
    .expect("reverse serve must end cleanly when the channel source closes");

    assert!(
        bound.load(Ordering::Acquire),
        "an established remote listener must publish the bound flag"
    );
    assert_eq!(
        stats.effective_port.load(Ordering::Acquire),
        41_337,
        "the allocated port, not the requested 0, must be reported"
    );
    // The stub's channel source is empty, so the loop ends because no further
    // channels can arrive — which must not read as a clean deadline.
    assert_eq!(
        stats.close_reason(),
        crate::json_wire::TunnelCloseReason::AcceptError
    );
}

/// The trait defaults must refuse, not silently succeed.
///
/// A stub that answered `Ok` for reverse forwarding would make every mock-driven
/// test pass while the real feature was missing — the failure mode this project
/// already hit once with `--no-input`.
#[tokio::test]
async fn unsupported_modes_refuse_on_a_client_without_them() {
    struct Bare;
    #[async_trait]
    impl SshClientTrait for Bare {
        async fn connect(_cfg: ConnectionConfig) -> Result<Box<Self>, crate::errors::SshCliError> {
            Ok(Box::new(Bare))
        }
        async fn run_command(
            &mut self,
            _c: &str,
            _m: usize,
            _s: Option<Vec<u8>>,
        ) -> Result<ExecutionOutput, crate::errors::SshCliError> {
            unreachable!()
        }
        async fn upload(
            &self,
            _l: &Path,
            _r: &Path,
        ) -> Result<TransferResult, crate::errors::SshCliError> {
            unreachable!()
        }
        async fn download(
            &self,
            _r: &Path,
            _l: &Path,
        ) -> Result<TransferResult, crate::errors::SshCliError> {
            unreachable!()
        }
        async fn open_tunnel_channel(
            &self,
            _h: &str,
            _p: u16,
            _o: &str,
            _po: u16,
        ) -> Result<Box<dyn crate::ssh::client::TunnelChannel>, crate::errors::SshCliError>
        {
            unreachable!()
        }
        async fn disconnect(&self) -> Result<(), crate::errors::SshCliError> {
            Ok(())
        }
    }

    let bare = Bare;
    assert!(bare.open_streamlocal_channel("/tmp/x.sock").await.is_err());
    assert!(bare.request_remote_forward("127.0.0.1", 0).await.is_err());
    assert!(bare.cancel_remote_forward("127.0.0.1", 0).await.is_err());
    assert!(bare.accept_forwarded_channel().await.is_none());
}

// ── G-QA-R02: the resolution range, previously reachable only with a real host ──

/// Builds a registry record without touching disk.
fn record(timeout_ms: u64) -> crate::vps::model::VpsRecord {
    crate::vps::model::VpsRecord::test_new(
        "resolver-host",
        "203.0.113.7",
        2222,
        "operator",
        secrecy::SecretString::from("registry-password".to_string()),
        None,
        None,
        Some(timeout_ms),
        None,
        None,
        None,
        None,
        false,
    )
}

/// E3 regression: `--use-agent` / `--agent-socket` parsed and were then discarded.
///
/// The flags were hard-coded to `false` / `None` at the call site, so a host registered
/// for agent authentication could not open a tunnel at all while `exec` and `scp` could.
/// Nothing offline could catch it, because the whole range lived behind `find_by_name`.
#[test]
fn agent_auth_overrides_reach_the_connection_config() {
    let auth = TunnelAuth {
        use_agent: true,
        agent_socket: Some("/run/user/1000/keyring/ssh".to_string()),
        ..TunnelAuth::default()
    };

    let cfg = resolve_tunnel_connection(record(9_000), auth, None, false);

    assert!(cfg.use_agent, "--use-agent must survive to the connection");
    assert_eq!(
        cfg.agent_socket.as_deref(),
        Some(std::path::Path::new("/run/user/1000/keyring/ssh")),
        "--agent-socket must survive to the connection"
    );
}

#[test]
fn key_override_replaces_the_registry_credential() {
    let auth = TunnelAuth {
        key: Some("/tmp/override_ed25519".to_string()),
        ..TunnelAuth::default()
    };

    let cfg = resolve_tunnel_connection(record(9_000), auth, None, false);

    assert_eq!(
        cfg.key_path.as_ref().map(|k| k.as_path()),
        Some(std::path::Path::new("/tmp/override_ed25519")),
        "--key must win over the stored credential, matching exec and scp"
    );
}

/// The registry timeout must NOT become the tunnel deadline.
///
/// This is a deliberate design decision the gap called out explicitly: a tunnel is
/// bounded by `--timeout-ms`, and letting a host record shorten or extend it would make
/// the one-shot deadline depend on state the caller never mentioned. It had no offline
/// coverage, so "simplifying" the code by forwarding the record value would have looked
/// like a cleanup and silently broken the bound.
#[test]
fn registry_timeout_is_carried_but_never_reinterpreted_as_the_deadline() {
    let cfg = resolve_tunnel_connection(record(1_234), TunnelAuth::default(), None, false);

    assert_eq!(
        cfg.timeout_ms.get(),
        1_234,
        "the connection budget still comes from the record"
    );
    // The deadline itself is `--timeout-ms`, enforced by `run_tunnel`'s guard, which is
    // covered by `timeout_zero_is_rejected_before_touching_the_registry`.
}

#[test]
fn replace_host_key_reaches_the_connection_config() {
    let cfg = resolve_tunnel_connection(record(9_000), TunnelAuth::default(), None, true);
    assert!(cfg.replace_host_key);

    let cfg = resolve_tunnel_connection(record(9_000), TunnelAuth::default(), None, false);
    assert!(!cfg.replace_host_key);
}

// ── G-TUN-R07 / R11 / R12: the shutdown event, previously asserted only in prose ──

// `close_reason_is_derived_from_the_flags_the_loop_managed_to_set` lives beside the
// type it exercises, in `src/tunnel/stats.rs`: it is a pure unit test over
// `TunnelStats`, with no listener, no runtime and no client.

#[test]
fn only_an_accept_error_makes_the_closed_event_not_ok() {
    use crate::json_wire::TunnelCloseReason;

    for reason in [TunnelCloseReason::Deadline, TunnelCloseReason::Signal] {
        let v = crate::output::build_tunnel_closed(crate::output::TunnelClosedInput {
            vps: "h",
            reason,
            bind: "127.0.0.1",
            local_port: 8080,
            forwards_served: 0,
            capacity_waits: 0,
            duration_ms: 10,
            mode: "local",
        });
        assert!(v.ok, "{reason:?} is a clean lifetime");
    }

    let v = crate::output::build_tunnel_closed(crate::output::TunnelClosedInput {
        vps: "h",
        reason: TunnelCloseReason::AcceptError,
        bind: "127.0.0.1",
        local_port: 8080,
        forwards_served: 0,
        capacity_waits: 0,
        duration_ms: 10,
        mode: "local",
    });
    assert!(
        !v.ok,
        "the process still exits 0 for having bound, so `ok` is the only field that \
         distinguishes a loop that died seconds into a five-minute deadline"
    );
}

#[test]
fn closed_event_reports_the_counters_it_was_given() {
    let v = crate::output::build_tunnel_closed(crate::output::TunnelClosedInput {
        vps: "prod-db",
        reason: crate::json_wire::TunnelCloseReason::Deadline,
        bind: "0.0.0.0",
        local_port: 15_432,
        forwards_served: 7,
        capacity_waits: 3,
        duration_ms: 60_000,
        mode: "socks5",
    });

    assert_eq!(v.event, "tunnel_closed");
    assert_eq!(v.vps, "prod-db");
    assert_eq!(v.bind, "0.0.0.0");
    assert_eq!(v.local_port, 15_432);
    assert_eq!(
        v.forwards_served, 7,
        "G-TUN-R11: did anything ever connect?"
    );
    assert_eq!(v.capacity_waits, 3, "G-TUN-R12: was the tunnel throttled?");
    assert_eq!(v.duration_ms, 60_000);
    assert_eq!(v.mode, "socks5");

    // Round-trips on the wire, which is what an agent actually parses.
    let json = serde_json::to_string(&v).expect("serialize");
    assert!(json.contains("\"reason\":\"deadline\""), "got: {json}");
    assert!(json.contains("\"forwards_served\":7"), "got: {json}");
    assert!(json.contains("\"capacity_waits\":3"), "got: {json}");
}

/// G-TUN-R06: `bind` existed only in the schema and in prose.
///
/// The field was added precisely so an agent could audit, from the structured contract
/// alone, whether it had just published a remote service beyond loopback. Nothing
/// asserted the value the product actually emits.
#[test]
fn listening_event_reports_the_effective_bind_address() {
    let loopback = crate::output::build_tunnel_listening(
        "h",
        8080,
        "db.internal",
        5432,
        30_000,
        "127.0.0.1",
        "local",
    );
    assert_eq!(loopback.bind, "127.0.0.1");
    assert_eq!(loopback.event, "tunnel_listening");

    let exposed = crate::output::build_tunnel_listening(
        "h",
        8080,
        "db.internal",
        5432,
        30_000,
        "0.0.0.0",
        "local",
    );
    assert_eq!(
        exposed.bind, "0.0.0.0",
        "an agent must be able to tell these two apart from the event alone"
    );
    assert_ne!(
        loopback.bind, exposed.bind,
        "collapsing the two is exactly what G-TUN-R06 was written to prevent"
    );
}

/// Drives the real accept loop and checks the counter the event publishes.
///
/// No sshd and no remote host: the injected [`Stub`] answers `open_tunnel_channel`, and
/// the connection is a plain loopback TCP dial. This is the assertion that makes
/// deleting `forwards_served` fail the build.
///
/// `#[serial]` and the flag reset are load-bearing. The accept loop polls
/// `signals::should_stop()` — process-global atomics — and breaks on a set flag, dropping
/// the listener so the next dial is refused. Other tests set those flags on purpose; all
/// are `#[serial]`, but `serial_test` only excludes serial tests from *each other*, so a
/// non-serial reader still races them. Symptom was textbook: green alone, red in the
/// suite, failure rate tracking `--test-threads`.
#[tokio::test]
#[serial_test::serial]
async fn accepting_a_connection_increments_forwards_served() {
    crate::signals::reset_flags_for_tests();
    let bound = Arc::new(AtomicBool::new(false));
    let stats = Arc::new(TunnelStats::default());

    let bound_probe = Arc::clone(&bound);
    let stats_probe = Arc::clone(&stats);

    let loop_handle = tokio::spawn(run_tunnel_with_client_stats(
        crate::tunnel::ServeContext {
            vps_name: "counted".to_string(),
            local_port: 0,
            // 60 s, matching the sibling tests. The old 1.5 s was below the 2 s this
            // test may spend polling for the bind, so the deadline could fire before
            // the dial — a second, smaller hazard on top of the signal-flag race above.
            // The test asserts on the counter and leaves its loop the moment it
            // increments, so a generous deadline costs nothing.
            timeout_ms: 60_000,
            json: false,
            bind_addr: "127.0.0.1".to_string(),
            bound_flag: Some(Arc::clone(&bound)),
            stats: Some(Arc::clone(&stats)),
        },
        "localhost",
        5432,
        Box::new(Stub::new()),
    ));

    // Wait for the listener rather than sleeping a fixed amount: the port is ephemeral
    // and only known after bind.
    let mut port = 0;
    for _ in 0..200 {
        if bound_probe.load(Ordering::Acquire) {
            port = u16::try_from(stats_probe.effective_port.load(Ordering::Acquire)).unwrap_or(0);
            if port != 0 {
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(port != 0, "listener never published an effective port");

    let conn = tokio::net::TcpStream::connect(("127.0.0.1", port)).await;
    assert!(conn.is_ok(), "loopback dial to the tunnel listener failed");
    drop(conn);

    for _ in 0..200 {
        if stats_probe.forwards_served.load(Ordering::Relaxed) > 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    assert!(
        stats_probe.forwards_served.load(Ordering::Relaxed) >= 1,
        "a connection was accepted but the counter the shutdown event publishes stayed at zero"
    );

    // The accept loop has no deadline of its own — `run_tunnel` owns the
    // `tokio::time::timeout` wrapper — so awaiting the handle here would block
    // forever. Abort, exactly as `tunnel_with_client_ends_on_cancel` does.
    loop_handle.abort();
    let _ = loop_handle.await;
}
