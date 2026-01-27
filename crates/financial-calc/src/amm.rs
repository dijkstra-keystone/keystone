//! Automated Market Maker calculations including concentrated liquidity.
//!
//! This module provides AMM calculations for DEX protocols like Camelot
//! and Uniswap V3 on Arbitrum. Includes:
//!
//! - Constant product (x*y=k) calculations
//! - Concentrated liquidity (Uniswap V3-style) math
//! - Tick and sqrt price conversions
//! - Liquidity provision calculations
//! - Impermanent loss calculations
//!
//! # Example
//!
//! ```
//! use financial_calc::amm::{
//!     calculate_swap_output, calculate_liquidity_from_amounts,
//!     sqrt_price_to_tick, tick_to_sqrt_price,
//! };
//! use precision_core::Decimal;
//! use core::str::FromStr;
//!
//! // Constant product swap
//! let output = calculate_swap_output(
//!     Decimal::from(1000000i64),  // reserve_in: 1M
//!     Decimal::from(1000000i64),  // reserve_out: 1M
//!     Decimal::from(1000i64),     // amount_in: 1000
//!     Decimal::from(30i64),       // fee_bps: 0.3%
//! ).unwrap();
//! ```

use precision_core::{ArithmeticError, Decimal};

/// Tick spacing for 0.05% fee tier (Uniswap V3 convention).
pub const TICK_SPACING_LOW: i32 = 10;

/// Tick spacing for 0.30% fee tier (Uniswap V3 convention).
pub const TICK_SPACING_MEDIUM: i32 = 60;

/// Tick spacing for 1.00% fee tier (Uniswap V3 convention).
pub const TICK_SPACING_HIGH: i32 = 200;

/// Minimum tick value.
pub const MIN_TICK: i32 = -887272;

/// Maximum tick value.
pub const MAX_TICK: i32 = 887272;

/// Parameters for a concentrated liquidity position.
#[derive(Debug, Clone, Copy)]
pub struct ConcentratedPosition {
    /// Lower tick bound of the position.
    pub tick_lower: i32,
    /// Upper tick bound of the position.
    pub tick_upper: i32,
    /// Liquidity amount in the position.
    pub liquidity: Decimal,
}

/// Calculate output amount for a constant product AMM swap.
///
/// Formula: output = (reserve_out * amount_in * (10000 - fee_bps)) / (reserve_in * 10000 + amount_in * (10000 - fee_bps))
///
/// # Arguments
///
/// * `reserve_in` - Reserve of input token
/// * `reserve_out` - Reserve of output token
/// * `amount_in` - Amount being swapped in
/// * `fee_bps` - Fee in basis points (e.g., 30 for 0.3%)
pub fn calculate_swap_output(
    reserve_in: Decimal,
    reserve_out: Decimal,
    amount_in: Decimal,
    fee_bps: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let bps_base = Decimal::from(10000i64);
    let fee_factor = bps_base.try_sub(fee_bps)?;

    let amount_in_with_fee = amount_in.try_mul(fee_factor)?;
    let numerator = reserve_out.try_mul(amount_in_with_fee)?;
    let denominator = reserve_in.try_mul(bps_base)?.try_add(amount_in_with_fee)?;

    numerator.try_div(denominator)
}

/// Calculate required input for a desired output amount.
///
/// # Arguments
///
/// * `reserve_in` - Reserve of input token
/// * `reserve_out` - Reserve of output token
/// * `amount_out` - Desired output amount
/// * `fee_bps` - Fee in basis points
pub fn calculate_swap_input(
    reserve_in: Decimal,
    reserve_out: Decimal,
    amount_out: Decimal,
    fee_bps: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let bps_base = Decimal::from(10000i64);
    let fee_factor = bps_base.try_sub(fee_bps)?;

    let numerator = reserve_in.try_mul(amount_out)?.try_mul(bps_base)?;
    let denominator = reserve_out.try_sub(amount_out)?.try_mul(fee_factor)?;

    numerator.try_div(denominator)?.try_add(Decimal::ONE)
}

/// Calculate spot price (token1 per token0).
pub fn calculate_spot_price(
    reserve_0: Decimal,
    reserve_1: Decimal,
) -> Result<Decimal, ArithmeticError> {
    reserve_1.try_div(reserve_0)
}

/// Calculate price impact as a decimal.
///
/// # Returns
///
/// Price impact as decimal (e.g., 0.01 for 1% impact).
pub fn calculate_price_impact(
    reserve_in: Decimal,
    reserve_out: Decimal,
    amount_in: Decimal,
) -> Result<Decimal, ArithmeticError> {
    // Spot price before swap
    let spot_price = reserve_out.try_div(reserve_in)?;

    // Effective price after swap (no fees for impact calculation)
    let output = calculate_swap_output(reserve_in, reserve_out, amount_in, Decimal::ZERO)?;
    let effective_price = output.try_div(amount_in)?;

    // Impact = (spot - effective) / spot
    spot_price.try_sub(effective_price)?.try_div(spot_price)
}

/// Convert a tick to sqrt price (Q64.96 format conceptually).
///
/// sqrt_price = 1.0001^(tick/2)
///
/// # Arguments
///
/// * `tick` - The tick value
///
/// # Returns
///
/// The sqrt price as a Decimal.
pub fn tick_to_sqrt_price(tick: i32) -> Result<Decimal, ArithmeticError> {
    // sqrt(1.0001) ≈ 1.00004999875
    let sqrt_base = parse_const("1.0001").try_sqrt()?;

    if tick == 0 {
        return Ok(Decimal::ONE);
    }

    let abs_tick = tick.unsigned_abs();
    let mut result = Decimal::ONE;

    // Binary exponentiation
    let mut base = sqrt_base;
    let mut exp = abs_tick;

    while exp > 0 {
        if exp & 1 == 1 {
            result = result.try_mul(base)?;
        }
        base = base.try_mul(base)?;
        exp >>= 1;
    }

    if tick < 0 {
        Decimal::ONE.try_div(result)
    } else {
        Ok(result)
    }
}

/// Convert sqrt price to tick.
///
/// tick = 2 * log(sqrt_price) / log(1.0001)
pub fn sqrt_price_to_tick(sqrt_price: Decimal) -> Result<i32, ArithmeticError> {
    if sqrt_price <= Decimal::ZERO {
        return Err(ArithmeticError::LogOfNegative);
    }

    // log(1.0001) / 2 for tick calculation
    let log_base = parse_const("1.0001").try_ln()?.try_div(Decimal::from(2i64))?;

    let log_price = sqrt_price.try_ln()?;
    let tick_decimal = log_price.try_div(log_base)?;

    // Round towards zero and extract integer
    let truncated = tick_decimal.trunc(0);
    let (mantissa, _scale) = truncated.to_parts();

    // Safe conversion: ticks are bounded by MIN_TICK/MAX_TICK
    Ok(mantissa as i32)
}

/// Calculate liquidity from token amounts for a concentrated position.
///
/// # Arguments
///
/// * `sqrt_price_current` - Current sqrt price
/// * `sqrt_price_lower` - Lower bound sqrt price
/// * `sqrt_price_upper` - Upper bound sqrt price
/// * `amount_0` - Amount of token0 to deposit
/// * `amount_1` - Amount of token1 to deposit
pub fn calculate_liquidity_from_amounts(
    sqrt_price_current: Decimal,
    sqrt_price_lower: Decimal,
    sqrt_price_upper: Decimal,
    amount_0: Decimal,
    amount_1: Decimal,
) -> Result<Decimal, ArithmeticError> {
    if sqrt_price_current <= sqrt_price_lower {
        // Only token0
        calculate_liquidity_0(sqrt_price_lower, sqrt_price_upper, amount_0)
    } else if sqrt_price_current >= sqrt_price_upper {
        // Only token1
        calculate_liquidity_1(sqrt_price_lower, sqrt_price_upper, amount_1)
    } else {
        // Both tokens - take minimum liquidity
        let liq_0 = calculate_liquidity_0(sqrt_price_current, sqrt_price_upper, amount_0)?;
        let liq_1 = calculate_liquidity_1(sqrt_price_lower, sqrt_price_current, amount_1)?;
        Ok(liq_0.min(liq_1))
    }
}

/// Calculate liquidity from token0 amount.
///
/// L = amount0 * sqrt_pa * sqrt_pb / (sqrt_pb - sqrt_pa)
fn calculate_liquidity_0(
    sqrt_price_a: Decimal,
    sqrt_price_b: Decimal,
    amount_0: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let numerator = amount_0.try_mul(sqrt_price_a)?.try_mul(sqrt_price_b)?;
    let denominator = sqrt_price_b.try_sub(sqrt_price_a)?;
    numerator.try_div(denominator)
}

/// Calculate liquidity from token1 amount.
///
/// L = amount1 / (sqrt_pb - sqrt_pa)
fn calculate_liquidity_1(
    sqrt_price_a: Decimal,
    sqrt_price_b: Decimal,
    amount_1: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let denominator = sqrt_price_b.try_sub(sqrt_price_a)?;
    amount_1.try_div(denominator)
}

/// Calculate token amounts from liquidity for a concentrated position.
///
/// # Returns
///
/// Tuple of (amount_0, amount_1).
pub fn calculate_amounts_from_liquidity(
    sqrt_price_current: Decimal,
    sqrt_price_lower: Decimal,
    sqrt_price_upper: Decimal,
    liquidity: Decimal,
) -> Result<(Decimal, Decimal), ArithmeticError> {
    let (amount_0, amount_1) = if sqrt_price_current <= sqrt_price_lower {
        // Only token0
        let amount_0 = calculate_amount_0(sqrt_price_lower, sqrt_price_upper, liquidity)?;
        (amount_0, Decimal::ZERO)
    } else if sqrt_price_current >= sqrt_price_upper {
        // Only token1
        let amount_1 = calculate_amount_1(sqrt_price_lower, sqrt_price_upper, liquidity)?;
        (Decimal::ZERO, amount_1)
    } else {
        // Both tokens
        let amount_0 = calculate_amount_0(sqrt_price_current, sqrt_price_upper, liquidity)?;
        let amount_1 = calculate_amount_1(sqrt_price_lower, sqrt_price_current, liquidity)?;
        (amount_0, amount_1)
    };

    Ok((amount_0, amount_1))
}

/// Calculate token0 amount from liquidity.
///
/// amount0 = L * (sqrt_pb - sqrt_pa) / (sqrt_pa * sqrt_pb)
fn calculate_amount_0(
    sqrt_price_a: Decimal,
    sqrt_price_b: Decimal,
    liquidity: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let numerator = liquidity.try_mul(sqrt_price_b.try_sub(sqrt_price_a)?)?;
    let denominator = sqrt_price_a.try_mul(sqrt_price_b)?;
    numerator.try_div(denominator)
}

/// Calculate token1 amount from liquidity.
///
/// amount1 = L * (sqrt_pb - sqrt_pa)
fn calculate_amount_1(
    sqrt_price_a: Decimal,
    sqrt_price_b: Decimal,
    liquidity: Decimal,
) -> Result<Decimal, ArithmeticError> {
    liquidity.try_mul(sqrt_price_b.try_sub(sqrt_price_a)?)
}

/// Calculate position value in token1 terms.
pub fn calculate_position_value(
    sqrt_price_current: Decimal,
    sqrt_price_lower: Decimal,
    sqrt_price_upper: Decimal,
    liquidity: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let (amount_0, amount_1) = calculate_amounts_from_liquidity(
        sqrt_price_current,
        sqrt_price_lower,
        sqrt_price_upper,
        liquidity,
    )?;

    // Price = sqrt_price^2
    let price = sqrt_price_current.try_mul(sqrt_price_current)?;

    // Value = amount_0 * price + amount_1
    let value_0 = amount_0.try_mul(price)?;
    value_0.try_add(amount_1)
}

/// Calculate impermanent loss for a concentrated liquidity position.
///
/// # Arguments
///
/// * `entry_sqrt_price` - Sqrt price when position was opened
/// * `current_sqrt_price` - Current sqrt price
/// * `sqrt_price_lower` - Lower bound of position
/// * `sqrt_price_upper` - Upper bound of position
/// * `liquidity` - Position liquidity
///
/// # Returns
///
/// Impermanent loss as a decimal (e.g., -0.05 for 5% loss).
pub fn calculate_impermanent_loss(
    entry_sqrt_price: Decimal,
    current_sqrt_price: Decimal,
    sqrt_price_lower: Decimal,
    sqrt_price_upper: Decimal,
    liquidity: Decimal,
) -> Result<Decimal, ArithmeticError> {
    // Value if held
    let (entry_amount_0, entry_amount_1) = calculate_amounts_from_liquidity(
        entry_sqrt_price,
        sqrt_price_lower,
        sqrt_price_upper,
        liquidity,
    )?;

    let current_price = current_sqrt_price.try_mul(current_sqrt_price)?;
    let held_value = entry_amount_0.try_mul(current_price)?.try_add(entry_amount_1)?;

    // Value as LP
    let lp_value = calculate_position_value(
        current_sqrt_price,
        sqrt_price_lower,
        sqrt_price_upper,
        liquidity,
    )?;

    // IL = (LP value - held value) / held value
    if held_value.is_zero() {
        return Ok(Decimal::ZERO);
    }

    lp_value.try_sub(held_value)?.try_div(held_value)
}

/// Calculate fee tier in basis points from tick spacing.
pub fn tick_spacing_to_fee_bps(tick_spacing: i32) -> Decimal {
    match tick_spacing {
        10 => Decimal::from(5i64),   // 0.05%
        60 => Decimal::from(30i64),  // 0.30%
        200 => Decimal::from(100i64), // 1.00%
        _ => Decimal::ZERO,
    }
}

/// Calculate liquidity shares to mint for a proportional deposit.
///
/// For full-range liquidity similar to Uniswap V2.
pub fn calculate_liquidity_mint(
    amount_0: Decimal,
    amount_1: Decimal,
    reserve_0: Decimal,
    reserve_1: Decimal,
    total_supply: Decimal,
) -> Result<Decimal, ArithmeticError> {
    if total_supply.is_zero() {
        // Initial liquidity: sqrt(amount_0 * amount_1)
        amount_0.try_mul(amount_1)?.try_sqrt()
    } else {
        // Proportional: min(amount_0/reserve_0, amount_1/reserve_1) * total_supply
        let ratio_0 = amount_0.try_div(reserve_0)?;
        let ratio_1 = amount_1.try_div(reserve_1)?;
        let min_ratio = ratio_0.min(ratio_1);
        min_ratio.try_mul(total_supply)
    }
}

/// Calculate tokens to return for burning liquidity shares.
///
/// # Returns
///
/// Tuple of (amount_0, amount_1).
pub fn calculate_liquidity_burn(
    shares: Decimal,
    reserve_0: Decimal,
    reserve_1: Decimal,
    total_supply: Decimal,
) -> Result<(Decimal, Decimal), ArithmeticError> {
    let ratio = shares.try_div(total_supply)?;
    let amount_0 = reserve_0.try_mul(ratio)?;
    let amount_1 = reserve_1.try_mul(ratio)?;
    Ok((amount_0, amount_1))
}

fn parse_const(s: &str) -> Decimal {
    s.parse().expect("Invalid constant")
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    fn decimal(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_swap_output_no_fee() {
        let output = calculate_swap_output(
            decimal("1000000"),  // 1M reserve_in
            decimal("1000000"),  // 1M reserve_out
            decimal("1000"),     // 1000 in
            Decimal::ZERO,       // no fee
        )
        .unwrap();

        // Expected: 1000000 * 1000 / (1000000 + 1000) ≈ 999.001
        assert!(output > decimal("999"));
        assert!(output < decimal("1000"));
    }

    #[test]
    fn test_swap_output_with_fee() {
        let output_no_fee = calculate_swap_output(
            decimal("1000000"),
            decimal("1000000"),
            decimal("1000"),
            Decimal::ZERO,
        )
        .unwrap();

        let output_with_fee = calculate_swap_output(
            decimal("1000000"),
            decimal("1000000"),
            decimal("1000"),
            decimal("30"), // 0.3% fee
        )
        .unwrap();

        // Output with fee should be less
        assert!(output_with_fee < output_no_fee);
    }

    #[test]
    fn test_price_impact() {
        let impact = calculate_price_impact(
            decimal("1000000"),
            decimal("1000000"),
            decimal("10000"), // 1% of reserves
        )
        .unwrap();

        // Should have ~1% impact for 1% of reserves
        assert!(impact > decimal("0.009"));
        assert!(impact < decimal("0.011"));
    }

    #[test]
    fn test_tick_conversion_roundtrip() {
        let original_tick = 100;
        let sqrt_price = tick_to_sqrt_price(original_tick).unwrap();
        let recovered_tick = sqrt_price_to_tick(sqrt_price).unwrap();

        // Should be close (may have small rounding)
        assert!((original_tick - recovered_tick).abs() <= 1);
    }

    #[test]
    fn test_tick_zero() {
        let sqrt_price = tick_to_sqrt_price(0).unwrap();
        assert_eq!(sqrt_price, Decimal::ONE);
    }

    #[test]
    fn test_tick_positive() {
        let sqrt_price = tick_to_sqrt_price(1000).unwrap();
        // sqrt(1.0001^1000) > 1
        assert!(sqrt_price > Decimal::ONE);
    }

    #[test]
    fn test_tick_negative() {
        let sqrt_price = tick_to_sqrt_price(-1000).unwrap();
        // sqrt(1.0001^-1000) < 1
        assert!(sqrt_price < Decimal::ONE);
    }

    #[test]
    fn test_liquidity_from_amounts_in_range() {
        let sqrt_current = decimal("1.0");
        let sqrt_lower = decimal("0.9");
        let sqrt_upper = decimal("1.1");

        let liquidity = calculate_liquidity_from_amounts(
            sqrt_current,
            sqrt_lower,
            sqrt_upper,
            decimal("1000"),
            decimal("1000"),
        )
        .unwrap();

        assert!(liquidity > Decimal::ZERO);
    }

    #[test]
    fn test_amounts_from_liquidity_roundtrip() {
        let sqrt_current = decimal("1.0");
        let sqrt_lower = decimal("0.9");
        let sqrt_upper = decimal("1.1");
        let initial_liquidity = decimal("10000");

        let (amount_0, amount_1) = calculate_amounts_from_liquidity(
            sqrt_current,
            sqrt_lower,
            sqrt_upper,
            initial_liquidity,
        )
        .unwrap();

        let recovered_liquidity = calculate_liquidity_from_amounts(
            sqrt_current,
            sqrt_lower,
            sqrt_upper,
            amount_0,
            amount_1,
        )
        .unwrap();

        // Should be approximately equal
        let diff = (recovered_liquidity - initial_liquidity).abs();
        let tolerance = initial_liquidity.try_mul(decimal("0.0001")).unwrap();
        assert!(diff < tolerance);
    }

    #[test]
    fn test_impermanent_loss_no_price_change() {
        let sqrt_price = decimal("1.0");

        let il = calculate_impermanent_loss(
            sqrt_price,
            sqrt_price,
            decimal("0.9"),
            decimal("1.1"),
            decimal("10000"),
        )
        .unwrap();

        // No price change = no IL
        assert!(il.abs() < decimal("0.0001"));
    }

    #[test]
    fn test_impermanent_loss_price_increase() {
        let entry_sqrt = decimal("1.0");
        let current_sqrt = decimal("1.05"); // ~10% price increase

        let il = calculate_impermanent_loss(
            entry_sqrt,
            current_sqrt,
            decimal("0.8"),
            decimal("1.2"),
            decimal("10000"),
        )
        .unwrap();

        // IL should be negative (LP underperforms HODL)
        assert!(il < Decimal::ZERO);
    }

    #[test]
    fn test_liquidity_mint_initial() {
        let shares = calculate_liquidity_mint(
            decimal("1000"),
            decimal("1000"),
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        )
        .unwrap();

        // sqrt(1000 * 1000) = 1000
        assert_eq!(shares, decimal("1000"));
    }

    #[test]
    fn test_liquidity_mint_proportional() {
        let shares = calculate_liquidity_mint(
            decimal("100"),
            decimal("100"),
            decimal("1000"),
            decimal("1000"),
            decimal("1000"),
        )
        .unwrap();

        // 10% deposit should give 10% of supply
        assert_eq!(shares, decimal("100"));
    }

    #[test]
    fn test_liquidity_burn() {
        let (amount_0, amount_1) = calculate_liquidity_burn(
            decimal("100"),   // 10% of supply
            decimal("1000"),  // reserve_0
            decimal("2000"),  // reserve_1
            decimal("1000"),  // total_supply
        )
        .unwrap();

        assert_eq!(amount_0, decimal("100"));
        assert_eq!(amount_1, decimal("200"));
    }
}
