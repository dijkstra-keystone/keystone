#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Deterministic fixed-point arithmetic for financial computation.
//!
//! This crate provides [`Decimal`], a 128-bit decimal type with configurable
//! rounding modes designed for financial calculations that must produce
//! identical results across all platforms.
//!
//! # Oracle Integration
//!
//! The [`oracle`] module provides utilities for working with different oracle
//! decimal formats (Chainlink, Pyth, etc.) commonly used in DeFi applications.

mod decimal;
mod error;
pub mod oracle;
mod rounding;
mod tolerance;

pub use decimal::Decimal;
pub use error::{ArithmeticError, ParseError};
pub use rounding::RoundingMode;
pub use tolerance::{
    approx_eq, approx_eq_relative, approx_eq_ulps, within_basis_points, within_percentage,
};

#[cfg(feature = "proptest")]
mod proptest_impl;
