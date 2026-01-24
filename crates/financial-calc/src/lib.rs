#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Financial calculation functions built on precision-core.
//!
//! This crate provides common financial calculations with deterministic results.

mod interest;
mod percentage;
mod time_value;

pub use interest::{compound_interest, effective_annual_rate, simple_interest};
pub use percentage::{basis_points_to_decimal, percentage_change, percentage_of};
pub use precision_core::{ArithmeticError, Decimal, RoundingMode};
pub use time_value::{future_value, net_present_value, present_value};
