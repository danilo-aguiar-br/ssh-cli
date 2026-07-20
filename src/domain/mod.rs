// SPDX-License-Identifier: MIT OR Apache-2.0
//! Domain newtypes — **parse, don't validate** (G-TYPE / G-DOM).
//!
//! Workload: pure local construction (sequential). Parallelism starts only after
//! typed values reach SSH fan-out. Zero-cost: wrappers are `#[repr(transparent)]`
//! or niche-optimized enums.
//!
//! Rules: private fields, `try_new` only, no `Deref`, no infallible `From` with
//! invariants. Wire DTOs stay primitives; domain carries the proof.
//!
//! ## Modules
//!
//! | Module | Types |
//! |--------|--------|
//! | [`error`] | [`DomainError`], [`secret_nonempty`] |
//! | [`names`] | [`VpsName`], [`SshHost`], [`SshUser`], [`HostTag`] |
//! | [`ports`] | [`SshPort`], [`BindPort`] |
//! | [`limits`] | [`TimeoutMs`], [`CharLimit`] |
//! | [`command`] | [`RemoteCommand`], [`KeyPath`] |
//! | [`time`] | [`Rfc3339Utc`], [`AddedAt`], [`CreatedAt`] |
//! | [`ids`] | [`CorrelationId`] (v4), [`BatchRunId`] (v7) |
//! | [`http_url`] | [`HttpsUrl`], [`AcmeOrderUrl`] |
//! | [`money`] | [`Money`] (**library-only**; no SSH/VPS CLI surface — G-E2E-14) |
#![forbid(unsafe_code)]

mod command;
mod error;
mod http_url;
mod ids;
mod limits;
mod money;
mod names;
mod ports;
mod time;

pub use command::{KeyPath, RemoteCommand};
pub use error::{domain_err, secret_nonempty, DomainError};
pub use http_url::{AcmeOrderUrl, HttpsUrl};
pub use ids::{BatchRunId, CorrelationId};
pub use limits::{CharLimit, TimeoutMs};
pub use money::{Brl, Currency, Money, Usd};
pub use names::{try_tags, HostTag, SshHost, SshUser, VpsName};
pub use ports::{BindPort, SshPort};
pub use time::{AddedAt, CreatedAt, Rfc3339Utc};
