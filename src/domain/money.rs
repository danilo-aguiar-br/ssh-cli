// SPDX-License-Identifier: MIT OR Apache-2.0
//! Decimal money newtypes (G-DOM-08 / G-E2E-14) — **library compliance only**.
//!
//! Not exposed on any `ssh-cli` subcommand. Retained so domain-type rules stay
//! testable without inventing fake money fields on SSH hosts.
//!
//! ssh-cli has no monetary domain. This module satisfies coordinated
//! `rust_decimal` + newtype rules without injecting fake money into SSH config.
//!
//! Serialization uses **string only** (`serde-with-str`); never float.
#![forbid(unsafe_code)]

use super::error::DomainError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

/// Currency marker (zero-sized). Scale is fixed per currency.
pub trait Currency: Copy + Send + Sync + 'static {
    /// ISO-like code.
    fn code() -> &'static str;
    /// Display symbol.
    fn symbol() -> &'static str;
    /// Decimal scale (fiat 2, JPY 0, BTC 8).
    fn scale() -> u32;
}

/// Brazilian Real.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Brl;

impl Currency for Brl {
    fn code() -> &'static str {
        "BRL"
    }
    fn symbol() -> &'static str {
        "R$"
    }
    fn scale() -> u32 {
        2
    }
}

/// US Dollar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Usd;

impl Currency for Usd {
    fn code() -> &'static str {
        "USD"
    }
    fn symbol() -> &'static str {
        "$"
    }
    fn scale() -> u32 {
        2
    }
}

/// Amount of currency `C` with exact decimal arithmetic.
///
/// Cross-currency `Add` is a compile error (different `C`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Money<C: Currency> {
    #[serde(with = "rust_decimal::serde::str")]
    amount: Decimal,
    #[serde(skip)]
    _currency: PhantomData<C>,
}

impl<C: Currency> Money<C> {
    /// Builds money rounded to the currency scale.
    pub fn try_new(amount: Decimal) -> Result<Self, DomainError> {
        if amount.is_sign_negative() {
            return Err(DomainError::new(
                "money",
                format!("{} amount must not be negative", C::code()),
            ));
        }
        let amount = amount.round_dp(C::scale());
        Ok(Self {
            amount,
            _currency: PhantomData,
        })
    }

    /// Parses from a decimal string (no float).
    pub fn try_from_str(s: impl AsRef<str>) -> Result<Self, DomainError> {
        let d = Decimal::from_str(s.as_ref().trim())
            .map_err(|e| DomainError::new("money", e.to_string()))?;
        Self::try_new(d)
    }

    /// Zero amount.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            amount: Decimal::ZERO,
            _currency: PhantomData,
        }
    }

    /// Underlying decimal.
    #[must_use]
    pub const fn amount(&self) -> Decimal {
        self.amount
    }

    /// Currency code.
    #[must_use]
    pub fn code(&self) -> &'static str {
        C::code()
    }
}

impl<C: Currency> fmt::Display for Money<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, C::code())
    }
}

impl<C: Currency> std::ops::Add for Money<C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            amount: (self.amount + rhs.amount).round_dp(C::scale()),
            _currency: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::dec;

    #[test]
    fn brl_add_and_serde_str() {
        let a = Money::<Brl>::try_new(dec!(99.99)).unwrap();
        let b = Money::<Brl>::try_new(dec!(0.01)).unwrap();
        let t = a + b;
        assert_eq!(t.amount(), dec!(100.00));
        let j = serde_json::to_string(&t).unwrap();
        // string form, not float
        assert!(j.contains('\"') && j.contains("100"), "{j}");
        let back: Money<Brl> = serde_json::from_str(&j).unwrap();
        assert_eq!(back.amount(), dec!(100.00));
    }

    #[test]
    fn rejects_negative() {
        assert!(Money::<Usd>::try_new(dec!(-1)).is_err());
    }

    #[test]
    fn from_str() {
        let m = Money::<Usd>::try_from_str("12.34").unwrap();
        assert_eq!(m.amount(), dec!(12.34));
    }
}
