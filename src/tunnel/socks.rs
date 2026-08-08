// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe`.
#![forbid(unsafe_code)]
//! SOCKS5 (RFC 1928) front-end for `tunnel --socks5` (G-TUN-R02).
//!
//! The proxy speaks only the no-authentication method and only `CONNECT`; every
//! accepted connection becomes one SSH `direct-tcpip` channel. `BIND` and
//! `UDP ASSOCIATE` are refused with the RFC's own reply code rather than by
//! dropping the socket, so a client learns *why* instead of seeing a reset.
//!
//! # Why the parsing is split from the I/O
//!
//! Every function that interprets bytes here is pure and takes a slice. A SOCKS5
//! handshake is attacker-reachable the moment the proxy binds, so the code that
//! decides how many bytes to trust must be testable without a socket — including
//! the malformed cases, which are exactly the ones a live test never produces.

use crate::errors::{SshCliError, SshCliResult};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// `CONNECT` command byte (RFC 1928 §4).
const CMD_CONNECT: u8 = 0x01;
/// `BIND` command byte (RFC 1928 §4).
const CMD_BIND: u8 = 0x02;
/// `UDP ASSOCIATE` command byte (RFC 1928 §4).
const CMD_UDP_ASSOCIATE: u8 = 0x03;

/// Address type: IPv4 (RFC 1928 §5).
const ATYP_IPV4: u8 = 0x01;
/// Address type: domain name (RFC 1928 §5).
const ATYP_DOMAIN: u8 = 0x03;
/// Address type: IPv6 (RFC 1928 §5).
const ATYP_IPV6: u8 = 0x04;

/// Reply: succeeded (RFC 1928 §6).
pub const REP_SUCCEEDED: u8 = 0x00;
/// Reply: general SOCKS server failure.
pub const REP_GENERAL_FAILURE: u8 = 0x01;
/// Reply: host unreachable.
pub const REP_HOST_UNREACHABLE: u8 = 0x04;
/// Reply: command not supported.
pub const REP_COMMAND_NOT_SUPPORTED: u8 = 0x07;
/// Reply: address type not supported.
pub const REP_ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x08;

/// A parsed `CONNECT` target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Socks5Target {
    /// Host as the client expressed it: literal IP or domain name.
    ///
    /// Deliberately *not* resolved locally — the whole point of proxying through
    /// SSH is that the name is resolved on the remote side, where it may mean a
    /// different host than it does here.
    pub host: String,
    /// Destination port.
    pub port: u16,
}

/// Why a handshake was refused, carrying the reply code owed to the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Socks5Refusal {
    /// RFC 1928 reply code to send before closing.
    pub reply: u8,
    /// Human-readable reason for logs (never sent on the wire).
    pub reason: String,
}

impl Socks5Refusal {
    fn new(reply: u8, reason: impl Into<String>) -> Self {
        Self {
            reply,
            reason: reason.into(),
        }
    }
}

/// Validates the greeting header and reports how many method bytes follow.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] when the version byte is not SOCKS5 or the
/// client advertises zero methods (which RFC 1928 does not allow).
pub fn parse_greeting_header(header: &[u8; 2]) -> SshCliResult<usize> {
    if header[0] != crate::constants::SOCKS5_VERSION {
        return Err(SshCliError::InvalidArgument(format!(
            "not a SOCKS5 greeting: version byte 0x{:02x}",
            header[0]
        )));
    }
    if header[1] == 0 {
        return Err(SshCliError::InvalidArgument(
            "SOCKS5 greeting advertises zero methods".to_string(),
        ));
    }
    Ok(usize::from(header[1]))
}

/// Whether the client offered the no-authentication method.
#[must_use]
pub fn offers_no_auth(methods: &[u8]) -> bool {
    methods.contains(&crate::constants::SOCKS5_METHOD_NO_AUTH)
}

/// Number of address bytes that follow the `ATYP` byte, port excluded.
///
/// For a domain name the first byte is the length, so the caller must read that
/// byte before it can know the rest — expressed here as `None`.
#[must_use]
pub fn fixed_address_len(atyp: u8) -> Option<usize> {
    match atyp {
        ATYP_IPV4 => Some(4),
        ATYP_IPV6 => Some(16),
        _ => None,
    }
}

/// Parses the four-byte request header into a command and address type.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] when the version byte is wrong. Unsupported
/// commands and address types are *not* errors here: they are returned as
/// [`Socks5Refusal`] so the caller can answer with the right reply code.
pub fn parse_request_header(header: &[u8; 4]) -> SshCliResult<Result<u8, Socks5Refusal>> {
    if header[0] != crate::constants::SOCKS5_VERSION {
        return Err(SshCliError::InvalidArgument(format!(
            "not a SOCKS5 request: version byte 0x{:02x}",
            header[0]
        )));
    }
    match header[1] {
        CMD_CONNECT => {}
        CMD_BIND => {
            return Ok(Err(Socks5Refusal::new(
                REP_COMMAND_NOT_SUPPORTED,
                "BIND is not supported: this proxy only opens outbound SSH channels",
            )))
        }
        CMD_UDP_ASSOCIATE => {
            return Ok(Err(Socks5Refusal::new(
                REP_COMMAND_NOT_SUPPORTED,
                "UDP ASSOCIATE is not supported: SSH forwarding is stream-only",
            )))
        }
        other => {
            return Ok(Err(Socks5Refusal::new(
                REP_COMMAND_NOT_SUPPORTED,
                format!("unknown SOCKS5 command 0x{other:02x}"),
            )))
        }
    }
    let atyp = header[3];
    if !matches!(atyp, ATYP_IPV4 | ATYP_IPV6 | ATYP_DOMAIN) {
        return Ok(Err(Socks5Refusal::new(
            REP_ADDRESS_TYPE_NOT_SUPPORTED,
            format!("unknown SOCKS5 address type 0x{atyp:02x}"),
        )));
    }
    Ok(Ok(atyp))
}

/// Renders the destination address bytes for a given address type.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] when the byte count does not match the type
/// or a domain name is not valid UTF-8.
pub fn decode_address(atyp: u8, bytes: &[u8]) -> SshCliResult<String> {
    match atyp {
        ATYP_IPV4 => {
            let octets: [u8; 4] = bytes.try_into().map_err(|_| {
                SshCliError::InvalidArgument(format!(
                    "SOCKS5 IPv4 address needs 4 bytes, got {}",
                    bytes.len()
                ))
            })?;
            Ok(std::net::Ipv4Addr::from(octets).to_string())
        }
        ATYP_IPV6 => {
            let octets: [u8; 16] = bytes.try_into().map_err(|_| {
                SshCliError::InvalidArgument(format!(
                    "SOCKS5 IPv6 address needs 16 bytes, got {}",
                    bytes.len()
                ))
            })?;
            Ok(std::net::Ipv6Addr::from(octets).to_string())
        }
        ATYP_DOMAIN => {
            // A zero-length name would make the request meaningless while still
            // being well-formed on the wire, so it is rejected explicitly rather
            // than forwarded to the server as an empty host.
            if bytes.is_empty() {
                return Err(SshCliError::InvalidArgument(
                    "SOCKS5 domain name is empty".to_string(),
                ));
            }
            std::str::from_utf8(bytes).map(str::to_owned).map_err(|_| {
                SshCliError::InvalidArgument("SOCKS5 domain name is not valid UTF-8".to_string())
            })
        }
        other => Err(SshCliError::InvalidArgument(format!(
            "unsupported SOCKS5 address type 0x{other:02x}"
        ))),
    }
}

/// Builds a reply frame with an all-zero IPv4 bound address.
///
/// The bound address is what the proxy would expose for `BIND`; for `CONNECT`
/// through an SSH channel there is no such address, and RFC 1928 permits the
/// unspecified value. Inventing the SSH server's address here would be a lie
/// the client could act on.
#[must_use]
pub fn encode_reply(reply_code: u8) -> [u8; 10] {
    let mut frame = [0_u8; 10];
    frame[0] = crate::constants::SOCKS5_VERSION;
    frame[1] = reply_code;
    frame[2] = 0x00; // RSV
    frame[3] = ATYP_IPV4;
    // frame[4..8] = 0.0.0.0, frame[8..10] = port 0 — already zeroed.
    frame
}

/// Reads and answers the greeting, then parses the `CONNECT` request.
///
/// Every read is `read_exact` with a length the protocol itself dictates, and the
/// running total is checked against [`crate::constants::SOCKS5_HANDSHAKE_MAX_BYTES`].
/// A peer therefore cannot make the proxy buffer more than one bounded handshake
/// before it has committed to anything.
///
/// # Errors
/// [`SshCliError::InvalidArgument`] on malformed input, [`SshCliError::Io`] on
/// socket failures. A well-formed but unsupported request yields
/// `Ok(Err(refusal))` after the reply has already been written.
pub async fn handshake<S>(stream: &mut S) -> SshCliResult<Result<Socks5Target, Socks5Refusal>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut budget = crate::constants::SOCKS5_HANDSHAKE_MAX_BYTES;

    let mut greeting = [0_u8; 2];
    read_exact_capped(stream, &mut greeting, &mut budget).await?;
    let n_methods = parse_greeting_header(&greeting)?;

    let mut methods = vec![0_u8; n_methods];
    read_exact_capped(stream, &mut methods, &mut budget).await?;

    if !offers_no_auth(&methods) {
        // Answering before closing is required: a client that sees the socket die
        // cannot tell "no shared method" from "proxy is broken".
        write_all(
            stream,
            &[
                crate::constants::SOCKS5_VERSION,
                crate::constants::SOCKS5_METHOD_NONE_ACCEPTABLE,
            ],
        )
        .await?;
        return Ok(Err(Socks5Refusal::new(
            REP_GENERAL_FAILURE,
            "client offered no acceptable authentication method",
        )));
    }
    write_all(
        stream,
        &[
            crate::constants::SOCKS5_VERSION,
            crate::constants::SOCKS5_METHOD_NO_AUTH,
        ],
    )
    .await?;

    let mut header = [0_u8; 4];
    read_exact_capped(stream, &mut header, &mut budget).await?;
    let atyp = match parse_request_header(&header)? {
        Ok(atyp) => atyp,
        Err(refusal) => {
            write_all(stream, &encode_reply(refusal.reply)).await?;
            return Ok(Err(refusal));
        }
    };

    let addr_len = match fixed_address_len(atyp) {
        Some(len) => len,
        None => {
            let mut len_byte = [0_u8; 1];
            read_exact_capped(stream, &mut len_byte, &mut budget).await?;
            usize::from(len_byte[0])
        }
    };
    let mut addr = vec![0_u8; addr_len];
    read_exact_capped(stream, &mut addr, &mut budget).await?;
    let host = decode_address(atyp, &addr)?;

    let mut port_bytes = [0_u8; 2];
    read_exact_capped(stream, &mut port_bytes, &mut budget).await?;
    let port = u16::from_be_bytes(port_bytes);

    Ok(Ok(Socks5Target { host, port }))
}

/// Writes a reply frame to the client.
///
/// # Errors
/// [`SshCliError::Io`] on socket failure.
pub async fn write_reply<S>(stream: &mut S, reply_code: u8) -> SshCliResult<()>
where
    S: AsyncWrite + Unpin,
{
    write_all(stream, &encode_reply(reply_code)).await
}

async fn write_all<S: AsyncWrite + Unpin>(stream: &mut S, bytes: &[u8]) -> SshCliResult<()> {
    stream.write_all(bytes).await.map_err(SshCliError::Io)?;
    stream.flush().await.map_err(SshCliError::Io)
}

async fn read_exact_capped<S: AsyncRead + Unpin>(
    stream: &mut S,
    buf: &mut [u8],
    budget: &mut usize,
) -> SshCliResult<()> {
    let want = buf.len();
    if want > *budget {
        return Err(SshCliError::InvalidArgument(format!(
            "SOCKS5 handshake exceeds {} bytes",
            crate::constants::SOCKS5_HANDSHAKE_MAX_BYTES
        )));
    }
    stream.read_exact(buf).await.map_err(SshCliError::Io)?;
    *budget -= want;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_rejects_socks4() {
        let err = parse_greeting_header(&[0x04, 0x01]).expect_err("SOCKS4 must be rejected");
        assert!(matches!(err, SshCliError::InvalidArgument(_)));
    }

    #[test]
    fn greeting_rejects_zero_methods() {
        assert!(parse_greeting_header(&[0x05, 0x00]).is_err());
    }

    #[test]
    fn greeting_reports_method_count() {
        assert_eq!(parse_greeting_header(&[0x05, 0x03]).unwrap(), 3);
    }

    #[test]
    fn no_auth_is_detected_anywhere_in_the_list() {
        assert!(offers_no_auth(&[0x02, 0x00]));
        assert!(!offers_no_auth(&[0x01, 0x02]));
    }

    #[test]
    fn bind_is_refused_with_the_rfc_code() {
        let refusal = parse_request_header(&[0x05, CMD_BIND, 0x00, ATYP_IPV4])
            .unwrap()
            .expect_err("BIND must be refused");
        assert_eq!(refusal.reply, REP_COMMAND_NOT_SUPPORTED);
    }

    #[test]
    fn udp_associate_is_refused_with_the_rfc_code() {
        let refusal = parse_request_header(&[0x05, CMD_UDP_ASSOCIATE, 0x00, ATYP_IPV4])
            .unwrap()
            .expect_err("UDP ASSOCIATE must be refused");
        assert_eq!(refusal.reply, REP_COMMAND_NOT_SUPPORTED);
    }

    #[test]
    fn unknown_address_type_is_refused_not_errored() {
        let refusal = parse_request_header(&[0x05, CMD_CONNECT, 0x00, 0x09])
            .unwrap()
            .expect_err("unknown ATYP must be refused");
        assert_eq!(refusal.reply, REP_ADDRESS_TYPE_NOT_SUPPORTED);
    }

    #[test]
    fn connect_with_ipv4_is_accepted() {
        assert_eq!(
            parse_request_header(&[0x05, CMD_CONNECT, 0x00, ATYP_IPV4])
                .unwrap()
                .unwrap(),
            ATYP_IPV4
        );
    }

    #[test]
    fn address_lengths_follow_the_rfc() {
        assert_eq!(fixed_address_len(ATYP_IPV4), Some(4));
        assert_eq!(fixed_address_len(ATYP_IPV6), Some(16));
        // Domain length is data, not a constant — the caller must read it.
        assert_eq!(fixed_address_len(ATYP_DOMAIN), None);
    }

    #[test]
    fn ipv4_decodes_to_dotted_quad() {
        assert_eq!(
            decode_address(ATYP_IPV4, &[127, 0, 0, 1]).unwrap(),
            "127.0.0.1"
        );
    }

    #[test]
    fn ipv6_decodes_to_canonical_form() {
        let mut bytes = [0_u8; 16];
        bytes[15] = 1;
        assert_eq!(decode_address(ATYP_IPV6, &bytes).unwrap(), "::1");
    }

    #[test]
    fn domain_is_passed_through_unresolved() {
        // Resolving locally would defeat the purpose: the name must mean whatever
        // it means on the *remote* side of the SSH session.
        assert_eq!(
            decode_address(ATYP_DOMAIN, b"internal.db.lan").unwrap(),
            "internal.db.lan"
        );
    }

    #[test]
    fn empty_domain_is_rejected() {
        assert!(decode_address(ATYP_DOMAIN, b"").is_err());
    }

    #[test]
    fn non_utf8_domain_is_rejected() {
        assert!(decode_address(ATYP_DOMAIN, &[0xFF, 0xFE]).is_err());
    }

    #[test]
    fn short_ipv4_is_rejected_rather_than_padded() {
        assert!(decode_address(ATYP_IPV4, &[127, 0, 0]).is_err());
    }

    #[test]
    fn reply_frame_is_well_formed() {
        let frame = encode_reply(REP_SUCCEEDED);
        assert_eq!(frame[0], crate::constants::SOCKS5_VERSION);
        assert_eq!(frame[1], REP_SUCCEEDED);
        assert_eq!(frame[3], ATYP_IPV4);
        assert_eq!(&frame[4..], &[0, 0, 0, 0, 0, 0]);
    }

    /// Drives `handshake` against an in-memory peer.
    ///
    /// `tokio::io::duplex` rather than a mock-stream crate: the whole payload can
    /// be written up front because the pipe buffers it, so the test exercises the
    /// real read/write ordering without adding a dependency for one helper.
    async fn drive(payload: &[u8]) -> (SshCliResult<Result<Socks5Target, Socks5Refusal>>, Vec<u8>) {
        let (mut client, mut server) = tokio::io::duplex(4096);
        client.write_all(payload).await.expect("feed handshake");
        let outcome = super::handshake(&mut server).await;
        drop(server);
        let mut written = Vec::new();
        client
            .read_to_end(&mut written)
            .await
            .expect("drain replies");
        (outcome, written)
    }

    #[tokio::test]
    async fn handshake_parses_a_domain_connect() {
        let mut payload = vec![0x05, 0x01, 0x00]; // greeting: one method, no-auth
        payload.extend_from_slice(&[0x05, CMD_CONNECT, 0x00, ATYP_DOMAIN]);
        payload.push(9);
        payload.extend_from_slice(b"localhost");
        payload.extend_from_slice(&443_u16.to_be_bytes());

        let (outcome, written) = drive(&payload).await;
        let target = outcome
            .expect("handshake must parse")
            .expect("CONNECT must be accepted");
        assert_eq!(target.host, "localhost");
        assert_eq!(target.port, 443);
        // The method reply must precede the request parse, or a strict client hangs.
        assert_eq!(written, vec![0x05, 0x00]);
    }

    #[tokio::test]
    async fn handshake_accepts_ipv4_connect() {
        let mut payload = vec![0x05, 0x02, 0x00, 0x02];
        payload.extend_from_slice(&[0x05, CMD_CONNECT, 0x00, ATYP_IPV4]);
        payload.extend_from_slice(&[10, 0, 0, 7]);
        payload.extend_from_slice(&5432_u16.to_be_bytes());

        let (outcome, _) = drive(&payload).await;
        let target = outcome.unwrap().unwrap();
        assert_eq!(target.host, "10.0.0.7");
        assert_eq!(target.port, 5432);
    }

    #[tokio::test]
    async fn handshake_refuses_a_client_without_no_auth() {
        let (outcome, written) = drive(&[0x05, 0x01, 0x02]).await; // only GSSAPI
        let refusal = outcome
            .expect("handshake must not error")
            .expect_err("client without no-auth must be refused");
        assert_eq!(refusal.reply, REP_GENERAL_FAILURE);
        // RFC 1928 §3: answer 0xFF instead of closing, so the client can say why.
        assert_eq!(written, vec![0x05, 0xFF]);
    }

    #[tokio::test]
    async fn handshake_answers_bind_with_command_not_supported() {
        let mut payload = vec![0x05, 0x01, 0x00];
        payload.extend_from_slice(&[0x05, CMD_BIND, 0x00, ATYP_IPV4]);

        let (outcome, written) = drive(&payload).await;
        let refusal = outcome.unwrap().expect_err("BIND must be refused");
        assert_eq!(refusal.reply, REP_COMMAND_NOT_SUPPORTED);
        // Method reply, then a full 10-byte refusal frame — not a bare close.
        assert_eq!(written.len(), 2 + 10);
        assert_eq!(written[2], crate::constants::SOCKS5_VERSION);
        assert_eq!(written[3], REP_COMMAND_NOT_SUPPORTED);
    }

    #[tokio::test]
    async fn handshake_rejects_a_socks4_client() {
        let (outcome, _) = drive(&[0x04, 0x01, 0x00]).await;
        let err = outcome.expect_err("SOCKS4 must not be parsed as SOCKS5");
        assert!(matches!(err, SshCliError::InvalidArgument(_)));
    }

    #[test]
    fn handshake_budget_covers_the_largest_legal_request() {
        // Greeting 2 + 255 methods, then request 4 + 1 + 255 + 2: the cap must not
        // reject input the RFC allows. This is the assertion that would have caught
        // the first version of the constant, which was 512 and rejected 519.
        let largest = 2 + 255 + 4 + 1 + 255 + 2;
        assert!(crate::constants::SOCKS5_HANDSHAKE_MAX_BYTES >= largest);
    }
}
