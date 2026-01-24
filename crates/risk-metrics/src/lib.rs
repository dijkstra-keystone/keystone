#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Risk metrics and calculations for DeFi applications.
//!
//! This crate provides risk measurement functions including health factors,
//! liquidation thresholds, and position metrics.

mod health;
mod liquidation;
mod position;

pub use health::{collateral_ratio, health_factor, is_healthy};
pub use liquidation::{liquidation_price, liquidation_threshold, max_borrowable};
pub use position::{available_liquidity, loan_to_value, utilization_rate};
pub use precision_core::{ArithmeticError, Decimal, RoundingMode};
