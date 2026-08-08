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
//! The modules themselves are private; every type below is re-exported here, so
//! the module column is plain text while the type column links to the public item.
//!
//! | Module | Types |
//! |--------|--------|
//! | `error` | [`crate::domain::DomainError`], [`crate::domain::secret_nonempty`] |
//! | `names` | [`crate::domain::VpsName`], [`crate::domain::SshHost`], [`crate::domain::SshUser`], [`crate::domain::HostTag`] |
//! | `ports` | [`crate::domain::SshPort`], [`crate::domain::BindPort`] |
//! | `limits` | [`crate::domain::TimeoutMs`], [`crate::domain::CharLimit`] |
//! | `command` | [`crate::domain::RemoteCommand`], [`crate::domain::KeyPath`] |
//! | `time` | [`crate::domain::Rfc3339Utc`], [`crate::domain::AddedAt`], [`crate::domain::CreatedAt`] |
//! | `ids` | [`crate::domain::CorrelationId`] (v4), [`crate::domain::BatchRunId`] (v7) |
//! | `http_url` | [`crate::domain::HttpsUrl`], [`crate::domain::AcmeOrderUrl`] |
//! | `money` | [`crate::domain::Money`] (**library-only**; no SSH/VPS CLI surface — G-E2E-14) |
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
