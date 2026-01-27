//! Keystone DeFi SDK - Unified computation library for Arbitrum protocols.
//!
//! This crate provides a single integration point for DeFi protocols on Arbitrum,
//! combining precision arithmetic, financial calculations, and risk metrics.
//!
//! # Quick Start
//!
//! ```rust
//! use keystone_defi::prelude::*;
//! use core::str::FromStr;
//!
//! // Lending: Calculate health factor
//! let collateral = Decimal::from_str("10000").unwrap();
//! let debt = Decimal::from_str("5000").unwrap();
//! let threshold = Decimal::from_str("0.8").unwrap();
//! let health = health_factor(collateral, debt, threshold).unwrap();
//!
//! // AMM: Calculate swap output
//! let output = calculate_swap_output(
//!     Decimal::from(1000000i64),
//!     Decimal::from(1000000i64),
//!     Decimal::from(1000i64),
//!     Decimal::from(30i64),  // 0.3% fee
//! ).unwrap();
//!
//! // Vault: Calculate share price
//! let share_price = calculate_share_price(
//!     Decimal::from(1000000i64),
//!     Decimal::from(950000i64),
//! ).unwrap();
//!
//! // Derivatives: Calculate liquidation price
//! let position = PerpPosition {
//!     size: Decimal::from_str("1.5").unwrap(),
//!     entry_price: Decimal::from(2000i64),
//!     is_long: true,
//!     leverage: Decimal::from(10i64),
//!     collateral: Decimal::from(300i64),
//! };
//! let liq_price = calculate_liquidation_price(&position, Decimal::from_str("0.01").unwrap()).unwrap();
//! ```
//!
//! # Modules
//!
//! - [`precision`] - Core decimal arithmetic with 28-digit precision
//! - [`lending`] - Health factor, liquidation, and borrow calculations
//! - [`amm`] - Swap, liquidity, and price impact calculations
//! - [`vault`] - ERC4626 share/asset calculations and compounding
//! - [`derivatives`] - Perpetual futures, funding rates, and margin calculations
//! - [`options`] - Black-Scholes pricing and Greeks
//!
//! # Stylus Integration
//!
//! All types are `no_std` compatible and work in Arbitrum Stylus smart contracts:
//!
//! ```rust,ignore
//! #![cfg_attr(not(feature = "export-abi"), no_main, no_std)]
//! use keystone_defi::prelude::*;
//! use stylus_sdk::prelude::*;
//!
//! #[public]
//! impl MyContract {
//!     pub fn calculate_health(&self, collateral: U256, debt: U256) -> Result<U256, Vec<u8>> {
//!         let c = u256_to_decimal(collateral);
//!         let d = u256_to_decimal(debt);
//!         let threshold = Decimal::from_str("0.8").unwrap();
//!         let hf = health_factor(c, d, threshold).map_err(|_| b"calc error".to_vec())?;
//!         Ok(decimal_to_u256(hf))
//!     }
//! }
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Core precision arithmetic.
pub mod precision {
    pub use precision_core::{ArithmeticError, Decimal, ParseError, RoundingMode};
}

/// Lending protocol calculations.
pub mod lending {
    pub use risk_metrics::{
        collateral_ratio, health_factor, is_healthy, liquidation_price, liquidation_threshold,
        max_borrowable, available_liquidity, loan_to_value, utilization_rate,
    };
}

/// AMM and DEX calculations.
pub mod amm {
    pub use financial_calc::amm::{
        calculate_amounts_from_liquidity, calculate_impermanent_loss, calculate_liquidity_burn,
        calculate_liquidity_from_amounts, calculate_liquidity_mint, calculate_position_value,
        calculate_price_impact, calculate_spot_price, calculate_swap_input, calculate_swap_output,
        sqrt_price_to_tick, tick_spacing_to_fee_bps, tick_to_sqrt_price, ConcentratedPosition,
        MAX_TICK, MIN_TICK, TICK_SPACING_HIGH, TICK_SPACING_LOW, TICK_SPACING_MEDIUM,
    };
}

/// Vault and yield calculations.
pub mod vault {
    pub use financial_calc::{
        compound_interest, effective_annual_rate, future_value, net_present_value, present_value,
        simple_interest,
    };

    use precision_core::{ArithmeticError, Decimal};

    /// Calculate share price (assets per share).
    pub fn calculate_share_price(
        total_assets: Decimal,
        total_supply: Decimal,
    ) -> Result<Decimal, ArithmeticError> {
        if total_supply.is_zero() {
            return Ok(Decimal::ONE);
        }
        total_assets.try_div(total_supply)
    }

    /// Calculate shares to mint for a deposit (ERC4626).
    pub fn calculate_shares_for_deposit(
        assets: Decimal,
        total_assets: Decimal,
        total_supply: Decimal,
    ) -> Result<Decimal, ArithmeticError> {
        if total_supply.is_zero() {
            return Ok(assets);
        }
        assets.try_mul(total_supply)?.try_div(total_assets)
    }

    /// Calculate assets to return for redemption (ERC4626).
    pub fn calculate_assets_for_redeem(
        shares: Decimal,
        total_assets: Decimal,
        total_supply: Decimal,
    ) -> Result<Decimal, ArithmeticError> {
        shares.try_mul(total_assets)?.try_div(total_supply)
    }

    /// Calculate APY from APR given compounding frequency.
    pub fn calculate_apy_from_apr(
        apr: Decimal,
        compounds_per_year: u32,
    ) -> Result<Decimal, ArithmeticError> {
        let n = Decimal::from(compounds_per_year as i64);
        let rate_per_period = apr.try_div(n)?;
        let base = Decimal::ONE.try_add(rate_per_period)?;

        let mut result = Decimal::ONE;
        for _ in 0..compounds_per_year {
            result = result.try_mul(base)?;
        }

        result.try_sub(Decimal::ONE)
    }

    /// Calculate performance fee on gains.
    pub fn calculate_performance_fee(
        gains: Decimal,
        fee_bps: Decimal,
    ) -> Result<Decimal, ArithmeticError> {
        let bps_base = Decimal::from(10000i64);
        gains.try_mul(fee_bps)?.try_div(bps_base)
    }
}

/// Derivatives and perpetual futures calculations.
pub mod derivatives {
    pub use financial_calc::derivatives::{
        calculate_average_entry_price, calculate_breakeven_price, calculate_effective_leverage,
        calculate_funding_payment, calculate_funding_rate, calculate_liquidation_distance,
        calculate_liquidation_price, calculate_margin_ratio, calculate_max_position_size,
        calculate_pnl, calculate_pnl_percentage, calculate_required_collateral, calculate_roe,
        FundingParams, PerpPosition,
    };
}

/// Options pricing and Greeks.
pub mod options {
    pub use financial_calc::options::{
        black_scholes_call, black_scholes_put, call_greeks, implied_volatility, normal_cdf,
        normal_pdf, put_greeks, Greeks, OptionParams,
    };
}

/// Interest and time value calculations.
pub mod interest {
    pub use financial_calc::{
        compound_interest, effective_annual_rate, future_value, net_present_value, present_value,
        simple_interest,
    };
}

/// Day count conventions for interest calculations.
pub mod day_count {
    pub use financial_calc::day_count::{Date, DayCountConvention, YearFraction};
}

/// Yield curve and term structure.
pub mod term_structure {
    pub use financial_calc::term_structure::{
        CurveNode, FlatTermStructure, PiecewiseTermStructure, TermStructure, MAX_CURVE_NODES,
    };
}

/// Numerical solvers.
pub mod solver {
    pub use financial_calc::solver::{
        bisection, brent, default_tolerance, newton_raphson, newton_raphson_numerical, secant,
        SolverResult, DEFAULT_MAX_ITER,
    };
}

/// Interpolation methods.
pub mod interpolation {
    pub use financial_calc::interpolation::{
        CubicSpline, DataPoint, Interpolator, Linear, LogLinear,
    };
}

/// Commonly used imports for DeFi calculations.
pub mod prelude {
    pub use crate::precision::{ArithmeticError, Decimal, RoundingMode};

    // Lending
    pub use crate::lending::{
        collateral_ratio, health_factor, is_healthy, liquidation_price, max_borrowable,
    };

    // AMM
    pub use crate::amm::{
        calculate_impermanent_loss, calculate_liquidity_mint, calculate_price_impact,
        calculate_spot_price, calculate_swap_output, sqrt_price_to_tick, tick_to_sqrt_price,
    };

    // Vault
    pub use crate::vault::{
        calculate_apy_from_apr, calculate_assets_for_redeem, calculate_performance_fee,
        calculate_share_price, calculate_shares_for_deposit, compound_interest,
    };

    // Derivatives
    pub use crate::derivatives::{
        calculate_funding_rate, calculate_liquidation_price, calculate_pnl, FundingParams,
        PerpPosition,
    };

    // Options
    pub use crate::options::{black_scholes_call, black_scholes_put, call_greeks, OptionParams};

    // Interest
    pub use crate::interest::{future_value, net_present_value, present_value, simple_interest};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use core::str::FromStr;

    fn decimal(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_lending_health_factor() {
        let hf = health_factor(
            decimal("10000"),
            decimal("5000"),
            decimal("0.8"),
        )
        .unwrap();

        // (10000 * 0.8) / 5000 = 1.6
        assert_eq!(hf, decimal("1.6"));
    }

    #[test]
    fn test_amm_swap() {
        let output = calculate_swap_output(
            decimal("1000000"),
            decimal("1000000"),
            decimal("1000"),
            decimal("30"),
        )
        .unwrap();

        assert!(output > decimal("996"));
        assert!(output < decimal("1000"));
    }

    #[test]
    fn test_vault_share_price() {
        let price = crate::vault::calculate_share_price(
            decimal("1050000"),
            decimal("1000000"),
        )
        .unwrap();

        assert_eq!(price, decimal("1.05"));
    }

    #[test]
    fn test_derivatives_pnl() {
        let position = PerpPosition {
            size: decimal("1.0"),
            entry_price: decimal("2000"),
            is_long: true,
            leverage: decimal("10"),
            collateral: decimal("200"),
        };

        let pnl = calculate_pnl(&position, decimal("2100")).unwrap();
        assert_eq!(pnl, decimal("100"));
    }

    #[test]
    fn test_options_pricing() {
        let params = OptionParams {
            spot: decimal("100"),
            strike: decimal("100"),
            rate: decimal("0.05"),
            time: decimal("1.0"),
            volatility: decimal("0.2"),
        };

        let call = black_scholes_call(&params).unwrap();
        assert!(call > decimal("9"));
        assert!(call < decimal("12"));
    }
}
