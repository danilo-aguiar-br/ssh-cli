// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SSH-01/04/16: connect + auth chain (SRP extract from monólito client_real).
#![forbid(unsafe_code)]
//! Authenticated SSH dial: TCP/TLS → host-key TOFU → publickey/agent/password.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use secrecy::ExposeSecret;

use crate::errors::{SshCliError, SshCliResult};
use crate::ssh::client_handler::{
    new_host_key_outcome, take_host_key_error, ClientHandler, HostKeyOutcome,
};
use crate::ssh::connection::ConnectionConfig;

/// Result of a successful connect+auth (session handle + config + host-key outcome slot).
pub(crate) struct AuthenticatedSession {
    pub session: russh::client::Handle<ClientHandler>,
    pub cfg: ConnectionConfig,
    /// Kept so Drop/debug paths can inspect (usually empty after success).
    #[allow(dead_code)]
    pub host_key_outcome: HostKeyOutcome,
}

/// Connect, verify host key, authenticate. Full flow honors `timeout_ms`.
pub(crate) async fn connect_authenticated(
    cfg: ConnectionConfig,
) -> SshCliResult<AuthenticatedSession> {
    cfg.validate()?;

    let timeout = Duration::from_millis(cfg.timeout_ms.get());
    let host = cfg.host.clone();
    let port = cfg.port.get();
    let username = cfg.username.as_str().to_owned();
    let secure_password = cfg.password.clone();
    let key_path = cfg.key_path.clone();
    let key_passphrase = cfg.key_passphrase.clone();
    let use_agent = cfg.use_agent;
    let agent_socket = cfg.resolved_agent_socket();
    let host_key_outcome = new_host_key_outcome();
    let handler = ClientHandler::new(&cfg, Arc::clone(&host_key_outcome));
    let outcome_for_err = Arc::clone(&host_key_outcome);

    let client_config = crate::ssh::connect::build_ssh_client_config(timeout);

    tracing::info!(
        host = %host,
        port,
        username = %username,
        timeout_ms = cfg.timeout_ms.get(),
        has_key = key_path.is_some(),
        use_agent,
        "starting SSH connection"
    );

    let tls_opts = cfg.tls.clone();
    let connection_result = tokio::time::timeout(timeout, async move {
        let mut session = if let Some(ref tls) = tls_opts {
            #[cfg(feature = "tls")]
            {
                let tls_stream = crate::tls::dial_tls(host.as_str(), port, tls).await?;
                // Apply TCP policy on the underlying socket when possible is handled in dial_tls path;
                // SSH-over-TLS still gets SSH-level keepalive via client Config.
                russh::client::connect_stream(client_config, tls_stream, handler)
                    .await
                    .map_err(|e| map_connect_err(e, &outcome_for_err, "SSH-over-TLS handshake failed"))?
            }
            #[cfg(not(feature = "tls"))]
            {
                let _ = tls;
                return Err(SshCliError::tls_msg(
                    "TLS requested but feature `tls` is disabled; rebuild with default features"
                        .into(),
                ));
            }
        } else {
            let socket = crate::ssh::connect::dial_ssh(host.as_str(), port).await?;
            russh::client::connect_stream(client_config, socket, handler)
                .await
                .map_err(|e| map_connect_err(e, &outcome_for_err, "TCP/handshake failed"))?
        };

        // Auth chain: file key → agent → password (G-SSH-04/16).
        let mut authenticated = false;
        let mut auth_method = "none";

        if let Some(ref kp) = key_path {
            let key_path_owned = kp.as_path().to_path_buf();
            let mut pass = key_passphrase
                .as_ref()
                .map(|s| s.expose_secret().to_string());
            let pass_for_load = pass.take();
            let key_result = tokio::task::spawn_blocking(move || {
                use zeroize::Zeroize;
                let result =
                    crate::ssh::key_material::load_secret_key_checked(&key_path_owned, pass_for_load.as_deref());
                let mut pass_drop = pass_for_load;
                if let Some(ref mut p) = pass_drop {
                    p.zeroize();
                }
                result
            })
            .await
            .map_err(|e| {
                SshCliError::SshAuthentication(format!("key load task failed for {kp}: {e}"))
            })??;

            let hash = session
                .best_supported_rsa_hash()
                .await
                .map_err(|e| SshCliError::ConnectionFailed(format!("rsa hash: {e}")))?
                .flatten();
            let auth = session
                .authenticate_publickey(
                    username.clone(),
                    russh::keys::PrivateKeyWithHashAlg::new(Arc::new(key_result), hash),
                )
                .await
                .map_err(|e| {
                    SshCliError::ConnectionFailed(format!("publickey auth failed: {e}"))
                })?;
            authenticated = auth.success();
            if authenticated {
                auth_method = "publickey";
            } else {
                tracing::warn!(host = %host, "key auth rejected; trying agent/password if configured");
            }
            // G-SSH-18: PrivateKey Arc dropped with authenticate_publickey stack.
        }

        if !authenticated && use_agent {
            match try_agent_auth(&mut session, &username, agent_socket.as_ref()).await {
                Ok(true) => {
                    authenticated = true;
                    auth_method = "agent";
                }
                Ok(false) => {
                    tracing::warn!(host = %host, "agent auth did not succeed");
                }
                Err(e) => {
                    tracing::warn!(host = %host, err = %e, "agent auth error; continuing fallback");
                }
            }
        }

        if !authenticated && !secure_password.expose_secret().is_empty() {
            let auth = session
                .authenticate_password(username.clone(), secure_password.expose_secret())
                .await
                .map_err(|e| {
                    SshCliError::ConnectionFailed(format!("password auth failed: {e}"))
                })?;
            authenticated = auth.success();
            if authenticated {
                auth_method = "password";
            }
        }

        if !authenticated {
            tracing::warn!(host = %host, username = %username, "SSH authentication rejected");
            return Err(SshCliError::AuthenticationFailed);
        }

        tracing::info!(auth_method, "SSH authentication succeeded");
        Ok::<_, SshCliError>(session)
    })
    .await;

    let session = match connection_result {
        Ok(Ok(s)) => s,
        Ok(Err(err)) => return Err(err),
        Err(_) => return Err(SshCliError::SshTimeout(cfg.timeout_ms.get())),
    };

    tracing::info!("SSH connection authenticated successfully");
    Ok(AuthenticatedSession {
        session,
        cfg,
        host_key_outcome,
    })
}

fn map_connect_err(
    e: russh::Error,
    outcome: &HostKeyOutcome,
    prefix: &str,
) -> SshCliError {
    // G-SSH-01: prefer typed host-key errors over generic handshake text.
    if let Some(product) = take_host_key_error(outcome) {
        return product;
    }
    SshCliError::ConnectionFailed(format!("{prefix}: {e}"))
}

async fn try_agent_auth(
    session: &mut russh::client::Handle<ClientHandler>,
    username: &str,
    socket: Option<&PathBuf>,
) -> SshCliResult<bool> {
    let Some(path) = socket else {
        return Err(SshCliError::InvalidArgument(
            "agent socket path not resolved".into(),
        ));
    };

    #[cfg(unix)]
    {
        use russh::keys::agent::client::AgentClient;
        use russh::keys::agent::AgentIdentity;

        let mut agent = AgentClient::connect_uds(path).await.map_err(|e| {
            SshCliError::SshAuthentication(format!(
                "ssh-agent connect failed ({}): {e}",
                path.display()
            ))
        })?;
        let identities = agent.request_identities().await.map_err(|e| {
            SshCliError::SshAuthentication(format!("ssh-agent list identities failed: {e}"))
        })?;
        if identities.is_empty() {
            tracing::warn!("ssh-agent has no identities");
            return Ok(false);
        }
        let hash = session
            .best_supported_rsa_hash()
            .await
            .map_err(|e| SshCliError::ConnectionFailed(format!("rsa hash: {e}")))?
            .flatten();
        for id in identities {
            let AgentIdentity::PublicKey { key, .. } = id else {
                continue;
            };
            let auth = session
                .authenticate_publickey_with(username.to_owned(), key, hash, &mut agent)
                .await
                .map_err(|e| {
                    SshCliError::ConnectionFailed(format!("agent publickey auth failed: {e}"))
                })?;
            if auth.success() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[cfg(windows)]
    {
        use russh::keys::agent::client::AgentClient;
        use russh::keys::agent::AgentIdentity;

        let mut agent = AgentClient::connect_named_pipe(path.as_os_str())
            .await
            .map_err(|e| {
                SshCliError::SshAuthentication(format!(
                    "ssh-agent named pipe connect failed ({}): {e}",
                    path.display()
                ))
            })?;
        let identities = agent.request_identities().await.map_err(|e| {
            SshCliError::SshAuthentication(format!("ssh-agent list identities failed: {e}"))
        })?;
        if identities.is_empty() {
            tracing::warn!("ssh-agent has no identities");
            return Ok(false);
        }
        let hash = session
            .best_supported_rsa_hash()
            .await
            .map_err(|e| SshCliError::ConnectionFailed(format!("rsa hash: {e}")))?
            .flatten();
        for id in identities {
            let AgentIdentity::PublicKey { key, .. } = id else {
                continue;
            };
            let auth = session
                .authenticate_publickey_with(username.to_owned(), key, hash, &mut agent)
                .await
                .map_err(|e| {
                    SshCliError::ConnectionFailed(format!("agent publickey auth failed: {e}"))
                })?;
            if auth.success() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = (session, username, path);
        Err(SshCliError::InvalidArgument(
            "ssh-agent is not supported on this platform".into(),
        ))
    }
}
