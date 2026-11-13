//! Tunnel mode selection from the mutually exclusive `tunnel` mode flags.
#![forbid(unsafe_code)]

/// Resolves the tunnel mode from the mutually exclusive mode flags.
///
/// Pure: no registry, no socket, no clock. The positional arguments mean
/// different things per mode — `remote_host` is a destination for a local
/// forward and a *server bind address* under `--reverse` — so getting this wrong
/// points the tunnel somewhere the caller never asked for. Keeping the decision
/// in one testable function is what lets every combination be covered without a
/// server.
///
/// # Errors
/// [`crate::errors::SshCliError::InvalidArgument`] (exit 64) when a mode is given
/// arguments it cannot use, or is missing arguments it requires.
pub fn resolve_tunnel_mode(
    socks5: bool,
    remote_socket: Option<String>,
    reverse: bool,
    remote_host: Option<String>,
    remote_port: Option<u16>,
) -> Result<crate::tunnel::TunnelMode, crate::errors::SshCliError> {
    use crate::errors::SshCliError::InvalidArgument;
    use crate::tunnel::TunnelMode;

    let positional_given = remote_host.is_some() || remote_port.is_some();

    if socks5 {
        if positional_given {
            return Err(InvalidArgument(
                "--socks5 chooses a destination per connection; remove REMOTE_HOST and REMOTE_PORT"
                    .to_string(),
            ));
        }
        return Ok(TunnelMode::Socks5);
    }

    if let Some(socket_path) = remote_socket {
        if positional_given {
            return Err(InvalidArgument(
                "--remote-socket replaces the destination; remove REMOTE_HOST and REMOTE_PORT"
                    .to_string(),
            ));
        }
        return Ok(TunnelMode::StreamLocal { socket_path });
    }

    let (Some(host), Some(port)) = (remote_host, remote_port) else {
        return Err(InvalidArgument(
            "tunnel requires REMOTE_HOST and REMOTE_PORT unless --socks5 or --remote-socket \
             is used"
                .to_string(),
        ));
    };

    if reverse {
        // Port 0 is meaningful here and only here: the server allocates and reports
        // back, exactly like a local ephemeral bind.
        return Ok(TunnelMode::Reverse {
            remote_bind: host,
            remote_port: port,
        });
    }

    if port == 0 {
        return Err(InvalidArgument(
            "REMOTE_PORT 0 is only valid with --reverse, where the server allocates the port"
                .to_string(),
        ));
    }
    Ok(TunnelMode::Local {
        remote_host: host,
        remote_port: port,
    })
}
