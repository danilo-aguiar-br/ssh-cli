// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! TCP dial helpers (Rules Rust — rede).
//!
//! Product surface is SSH client + optional local tunnel listener. This module
//! owns dual-stack DNS resolution and Happy Eyeballs-style connect so callers
//! never stick on a blackholed first address (`addrs.next().unwrap()` antipattern).
//!
//! - **DNS:** async via [`tokio::net::lookup_host`] (cancelable with outer timeouts).
//! - **Dial:** try all resolved [`SocketAddr`]s; race families with a short delay
//!   (RFC 8305-inspired) and abort losers on first success.
//! - **Workload:** pure I/O; no CPU fan-out, no Rayon.

use std::io;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::task::JoinSet;

use crate::constants::HAPPY_EYEBALLS_ATTEMPT_DELAY_MS;

/// Resolve `host:port` and dial with multi-address Happy Eyeballs racing.
///
/// # Errors
///
/// Returns the last connect error when every candidate fails, or
/// [`io::ErrorKind::AddrNotAvailable`] when DNS yields no addresses.
pub async fn dial_tcp(host: &str, port: u16) -> io::Result<TcpStream> {
    let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host, port)).await?.collect();
    if addrs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            format!("no addresses resolved for {host}:{port}"),
        ));
    }
    let ordered = interleave_address_families(addrs);
    tracing::debug!(
        host,
        port,
        candidates = ordered.len(),
        "dialing TCP with Happy Eyeballs ordering"
    );
    race_connect(ordered).await
}

/// Interleave IPv6 and IPv4 candidates (IPv6 first within each pair).
///
/// Prefer dual-stack interleave over "all AAAA then all A" so a dead IPv6 path
/// does not delay every IPv4 attempt until the full v6 list is exhausted.
#[must_use]
pub fn interleave_address_families(addrs: Vec<SocketAddr>) -> Vec<SocketAddr> {
    let mut v6 = Vec::new();
    let mut v4 = Vec::new();
    for addr in addrs {
        match addr.ip() {
            IpAddr::V6(_) => v6.push(addr),
            IpAddr::V4(_) => v4.push(addr),
        }
    }
    let mut out = Vec::with_capacity(v6.len() + v4.len());
    let mut i6 = v6.into_iter();
    let mut i4 = v4.into_iter();
    loop {
        match (i6.next(), i4.next()) {
            (Some(a), Some(b)) => {
                out.push(a);
                out.push(b);
            }
            (Some(a), None) => {
                out.push(a);
                out.extend(i6);
                break;
            }
            (None, Some(b)) => {
                out.push(b);
                out.extend(i4);
                break;
            }
            (None, None) => break,
        }
    }
    out
}

/// Race connect attempts: first address immediately; further addresses start
/// after [`HAPPY_EYEBALLS_ATTEMPT_DELAY_MS`] or sooner if the previous attempt fails.
async fn race_connect(addrs: Vec<SocketAddr>) -> io::Result<TcpStream> {
    debug_assert!(!addrs.is_empty());

    let mut pending = addrs.into_iter();
    let mut set: JoinSet<(SocketAddr, io::Result<TcpStream>)> = JoinSet::new();
    let mut last_err: Option<io::Error> = None;
    let delay = Duration::from_millis(HAPPY_EYEBALLS_ATTEMPT_DELAY_MS);

    // Kick the first candidate immediately.
    if let Some(addr) = pending.next() {
        set.spawn(async move { (addr, TcpStream::connect(addr).await) });
    }

    let stagger = tokio::time::sleep(delay);
    tokio::pin!(stagger);
    let mut stagger_armed = true;

    loop {
        if set.is_empty() {
            // Start next if any remain (previous wave fully failed).
            if let Some(addr) = pending.next() {
                set.spawn(async move { (addr, TcpStream::connect(addr).await) });
                stagger.as_mut().reset(tokio::time::Instant::now() + delay);
                stagger_armed = true;
                continue;
            }
            break;
        }

        tokio::select! {
            biased;
            Some(joined) = set.join_next() => {
                match joined {
                    Ok((addr, Ok(stream))) => {
                        tracing::debug!(%addr, "TCP connect succeeded");
                        set.abort_all();
                        while set.join_next().await.is_some() {}
                        return Ok(stream);
                    }
                    Ok((addr, Err(e))) => {
                        tracing::debug!(%addr, err = %e, "TCP connect attempt failed");
                        last_err = Some(e);
                        // On failure, start the next candidate without waiting out the stagger.
                        if let Some(next) = pending.next() {
                            set.spawn(async move { (next, TcpStream::connect(next).await) });
                            stagger.as_mut().reset(tokio::time::Instant::now() + delay);
                            stagger_armed = true;
                        }
                    }
                    Err(join_err) if join_err.is_cancelled() => {
                        // Expected after abort_all on success path; ignore if we continue.
                    }
                    Err(join_err) => {
                        tracing::debug!(err = %join_err, "dial task join error");
                        last_err = Some(io::Error::other(join_err));
                    }
                }
            }
            _ = &mut stagger, if stagger_armed => {
                stagger_armed = false;
                if let Some(addr) = pending.next() {
                    set.spawn(async move { (addr, TcpStream::connect(addr).await) });
                    stagger.as_mut().reset(tokio::time::Instant::now() + delay);
                    stagger_armed = true;
                }
            }
        }
    }

    Err(last_err.unwrap_or_else(|| {
        io::Error::new(io::ErrorKind::ConnectionRefused, "all dial attempts failed")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

    #[test]
    fn interleave_prefers_v6_then_pairs_with_v4() {
        let addrs = vec![
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 22)),
            SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 22, 0, 0)),
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 2), 22)),
            SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 2), 22, 0, 0)),
        ];
        let ordered = interleave_address_families(addrs);
        assert_eq!(ordered.len(), 4);
        assert!(ordered[0].is_ipv6());
        assert!(ordered[1].is_ipv4());
        assert!(ordered[2].is_ipv6());
        assert!(ordered[3].is_ipv4());
    }

    #[test]
    fn interleave_v4_only_preserves_all() {
        let addrs = vec![
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1)),
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 2)),
        ];
        let ordered = interleave_address_families(addrs);
        assert_eq!(ordered.len(), 2);
        assert!(ordered.iter().all(|a| a.is_ipv4()));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn dial_unresolvable_host_errors() {
        // RFC 6761 `.invalid` TLD must not resolve.
        let err = dial_tcp("no-such-host.invalid", 22)
            .await
            .expect_err("must fail DNS or dial");
        // Either DNS failure or empty-result style errors are acceptable.
        let kind = err.kind();
        let msg = err.to_string();
        assert!(
            matches!(
                kind,
                io::ErrorKind::AddrNotAvailable
                    | io::ErrorKind::NotFound
                    | io::ErrorKind::Other
                    | io::ErrorKind::InvalidInput
                    | io::ErrorKind::TimedOut
                    | io::ErrorKind::ConnectionRefused
            ) || msg.contains("failed to lookup")
                || msg.contains("Name or service")
                || msg.contains("no addresses")
                || msg.contains("Temporary failure")
                || msg.contains("No address"),
            "unexpected dial error: {err:?}"
        );
    }
}
