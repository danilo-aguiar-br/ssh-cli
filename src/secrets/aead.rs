// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted.
#![forbid(unsafe_code)]
//! ChaCha20-Poly1305 sealing of at-rest secrets, and the context that binds them.
//!
//! Split out of `secrets.rs` so the cryptographic core of the product is one
//! auditable unit rather than a middle third of a 740-line file that also handles
//! keyring lookup, XDG paths, CLI flag state and status reporting. In a security
//! CLI the question "what exactly does the encryption do" should be answerable by
//! reading one module.
//!
//! # Blob versions
//!
//! See [`crate::secrets`] for the `v1` / `v2` compatibility rules. In short: `v1`
//! was sealed without associated data, so its tag proved only "encrypted with this
//! key" and a ciphertext could be relocated between hosts or between fields
//! undetected. `v2` binds the tag to a [`SecretContext`].

use super::{PrimaryKey, ENC_PREFIX, ENC_PREFIX_V2};
use crate::constants::{AEAD_NONCE_LEN_BYTES, AEAD_TAG_LEN_BYTES};
use crate::errors::{SshCliError, SshCliResult};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use zeroize::Zeroize;

/// Domain separator of the associated-data encoding (A7).
///
/// Included so a `v2` AAD can never collide with any other AEAD usage that a
/// future version might add under the same key.
const AAD_DOMAIN: &[u8] = b"ssh-cli:secret-aad:v2";

/// Host name used when the call site cannot name the owning host yet.
///
/// See the module docs: blobs sealed under this context stay relocatable, which
/// is exactly the `v1` guarantee and therefore not a regression.
const UNBOUND_NAME: &str = "";

/// Where a secret belongs: the host that owns it and the field that holds it.
///
/// A7: this pair is fed to the AEAD as associated data, so the tag proves not
/// only "encrypted with this key" but also "sealed for *this* slot". Moving a
/// ciphertext between hosts or between `password` and `su_password` then fails
/// verification instead of silently decrypting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecretContext<'a> {
    host: &'a str,
    field: &'a str,
}

impl<'a> SecretContext<'a> {
    /// Binds a secret to `host` (record name) and `field` (e.g. `password`).
    #[must_use]
    pub const fn new(host: &'a str, field: &'a str) -> Self {
        Self { host, field }
    }

    /// Context for call sites that do not carry host/field yet.
    #[must_use]
    pub const fn unbound() -> Self {
        Self {
            host: UNBOUND_NAME,
            field: UNBOUND_NAME,
        }
    }

    /// True when this context carries no binding at all.
    #[must_use]
    pub const fn is_unbound(&self) -> bool {
        self.host.is_empty() && self.field.is_empty()
    }

    /// Encodes the context as unambiguous associated data.
    ///
    /// Length-prefixed rather than delimiter-joined: a host literally named
    /// `a:b` must not be able to impersonate the pair `(a, b)`.
    pub(super) fn aad(&self) -> Vec<u8> {
        let host = self.host.as_bytes();
        let field = self.field.as_bytes();
        let mut out = Vec::with_capacity(AAD_DOMAIN.len() + 8 + host.len() + field.len());
        out.extend_from_slice(AAD_DOMAIN);
        // Lengths are bounded by validated record names; the cast cannot wrap in
        // practice and saturating keeps the encoding total either way.
        out.extend_from_slice(&u32::try_from(host.len()).unwrap_or(u32::MAX).to_be_bytes());
        out.extend_from_slice(host);
        out.extend_from_slice(&u32::try_from(field.len()).unwrap_or(u32::MAX).to_be_bytes());
        out.extend_from_slice(field);
        out
    }
}

/// Seals `plaintext` as a `v2` blob bound to `ctx` (A7).
pub fn encrypt_secret(
    key: &PrimaryKey,
    plaintext: &str,
    ctx: SecretContext<'_>,
) -> SshCliResult<String> {
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_slice())
        .map_err(|_| SshCliError::crypto("aead_key"))?;
    let mut nonce_bytes = [0u8; AEAD_NONCE_LEN_BYTES];
    // G-ERR-R01: same reasoning as `generate_hex_key` — RNG failure is exit 70.
    getrandom::fill(&mut nonce_bytes).map_err(|_| SshCliError::software("rng"))?;
    // A3: `aead` 0.6 replaced the deprecated `Nonce::from_slice` with infallible
    // `From<[u8; N]>` for a fixed-size array, which is exactly what we have here.
    let nonce = Nonce::from(nonce_bytes);
    let aad = ctx.aad();
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext.as_bytes(),
                aad: &aad,
            },
        )
        .map_err(|_| SshCliError::crypto("encrypt"))?;
    let mut packed = Vec::with_capacity(AEAD_NONCE_LEN_BYTES + ciphertext.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);
    Ok(format!(
        "{ENC_PREFIX_V2}{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &packed)
    ))
}

/// Opens a `v1` (no AAD) or `v2` (AAD) blob.
///
/// For `v2` the bound context is tried first and the unbound context second:
/// the fallback only helps blobs that were themselves written without a binding
/// (see module docs), and never lets a bound blob be read from another slot.
pub fn decrypt_secret(
    key: &PrimaryKey,
    blob: &str,
    ctx: SecretContext<'_>,
) -> SshCliResult<String> {
    let (b64, versioned) = match blob.strip_prefix(ENC_PREFIX_V2) {
        Some(rest) => (rest, true),
        None => (
            blob.strip_prefix(ENC_PREFIX)
                .ok_or_else(|| SshCliError::crypto("blob_parse"))?,
            false,
        ),
    };
    let packed = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
        .map_err(|_| SshCliError::crypto("blob_b64"))?;
    if packed.len() < AEAD_NONCE_LEN_BYTES + AEAD_TAG_LEN_BYTES {
        return Err(SshCliError::Config("encrypted blob too short".to_string()));
    }
    let (nonce_bytes, ct) = packed.split_at(AEAD_NONCE_LEN_BYTES);
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_slice())
        .map_err(|_| SshCliError::crypto("aead_key"))?;
    // The length was already checked above, so this conversion cannot fail; keeping it
    // fallible rather than panicking means a future edit to that check degrades into a
    // typed error instead of aborting the process.
    let nonce = Nonce::try_from(nonce_bytes).map_err(|_| SshCliError::crypto("blob_nonce"))?;

    let plain = if versioned {
        let aad = ctx.aad();
        match cipher.decrypt(&nonce, Payload { msg: ct, aad: &aad }) {
            Ok(p) => p,
            Err(_) if !ctx.is_unbound() => {
                // Migration path only: a blob written before the call site knew
                // its host/field carries the unbound AAD.
                let legacy = SecretContext::unbound().aad();
                cipher
                    .decrypt(
                        &nonce,
                        Payload {
                            msg: ct,
                            aad: &legacy,
                        },
                    )
                    .map_err(|_| SshCliError::crypto("decrypt"))?
            }
            Err(_) => return Err(SshCliError::crypto("decrypt")),
        }
    } else {
        // `v1`: sealed without associated data, hence relocatable. Read-only.
        cipher
            .decrypt(&nonce, ct)
            .map_err(|_| SshCliError::crypto("decrypt"))?
    };

    match String::from_utf8(plain) {
        Ok(s) => Ok(s),
        Err(e) => {
            // from_utf8 failure keeps bytes in the error — scrub before drop.
            let mut bad = e.into_bytes();
            bad.zeroize();
            Err(SshCliError::Config(
                "decrypted secret is not valid UTF-8".to_string(),
            ))
        }
    }
}
