//! Perpetual futures and derivatives calculations.
//!
//! This module provides calculations for perpetual futures protocols
//! like GMX, Vertex, and Vela on Arbitrum. Includes funding rate,
//! liquidation price, and PnL calculations.
//!
//! # Example
//!
//! ```
//! use financial_calc::derivatives::{PerpPosition, calculate_pnl, calculate_liquidation_price};
//! use precision_core::Decimal;
//! use core::str::FromStr;
//!
//! let position = PerpPosition {
//!     size: Decimal::from_str("1.5").unwrap(),        // 1.5 ETH
//!     entry_price: Decimal::from(2000i64),            // $2000
//!     is_long: true,
//!     leverage: Decimal::from(10i64),                 // 10x
//!     collateral: Decimal::from(300i64),              // $300
//! };
//!
//! let current_price = Decimal::from(2200i64);
//! let pnl = calculate_pnl(&position, current_price).unwrap();
//! ```

use precision_core::{ArithmeticError, Decimal};

/// A perpetual futures position.
#[derive(Debug, Clone, Copy)]
pub struct PerpPosition {
    /// Position size in base asset units.
    pub size: Decimal,
    /// Entry price of the position.
    pub entry_price: Decimal,
    /// True for long, false for short.
    pub is_long: bool,
    /// Leverage multiplier.
    pub leverage: Decimal,
    /// Collateral amount in quote currency.
    pub collateral: Decimal,
}

/// Funding rate calculation parameters.
#[derive(Debug, Clone, Copy)]
pub struct FundingParams {
    /// Mark price (from oracle/TWAP).
    pub mark_price: Decimal,
    /// Index price (spot reference).
    pub index_price: Decimal,
    /// Interest rate component (annualized, as decimal).
    pub interest_rate: Decimal,
    /// Premium cap as decimal (e.g., 0.0005 for 0.05%).
    pub premium_cap: Decimal,
    /// Funding interval in hours.
    pub funding_interval_hours: Decimal,
}

/// Calculate profit/loss for a perpetual position.
///
/// # Arguments
///
/// * `position` - The perpetual position
/// * `current_price` - Current mark price
///
/// # Returns
///
/// PnL in quote currency (positive = profit, negative = loss).
pub fn calculate_pnl(
    position: &PerpPosition,
    current_price: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let price_diff = if position.is_long {
        current_price.try_sub(position.entry_price)?
    } else {
        position.entry_price.try_sub(current_price)?
    };

    position.size.try_mul(price_diff)
}

/// Calculate PnL as a percentage of collateral.
///
/// # Returns
///
/// PnL percentage as decimal (e.g., 0.15 for 15% profit).
pub fn calculate_pnl_percentage(
    position: &PerpPosition,
    current_price: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let pnl = calculate_pnl(position, current_price)?;
    pnl.try_div(position.collateral)
}

/// Calculate return on equity (ROE) considering leverage.
///
/// ROE = (PnL / Collateral) * 100
pub fn calculate_roe(
    position: &PerpPosition,
    current_price: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let pnl_pct = calculate_pnl_percentage(position, current_price)?;
    pnl_pct.try_mul(Decimal::from(100i64))
}

/// Calculate liquidation price for a perpetual position.
///
/// # Arguments
///
/// * `position` - The perpetual position
/// * `maintenance_margin_rate` - Maintenance margin as decimal (e.g., 0.01 for 1%)
///
/// # Returns
///
/// The price at which the position will be liquidated.
pub fn calculate_liquidation_price(
    position: &PerpPosition,
    maintenance_margin_rate: Decimal,
) -> Result<Decimal, ArithmeticError> {
    // Position notional = size * entry_price
    let notional = position.size.try_mul(position.entry_price)?;

    // Maintenance margin required
    let maintenance_margin = notional.try_mul(maintenance_margin_rate)?;

    // Loss that would trigger liquidation
    let max_loss = position.collateral.try_sub(maintenance_margin)?;

    // Price movement that would cause this loss
    let price_movement = max_loss.try_div(position.size)?;

    if position.is_long {
        // Long: liquidated when price drops
        position.entry_price.try_sub(price_movement)
    } else {
        // Short: liquidated when price rises
        position.entry_price.try_add(price_movement)
    }
}

/// Calculate distance to liquidation as percentage.
///
/// # Returns
///
/// Percentage distance (e.g., 0.15 means 15% price move to liquidation).
pub fn calculate_liquidation_distance(
    position: &PerpPosition,
    current_price: Decimal,
    maintenance_margin_rate: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let liq_price = calculate_liquidation_price(position, maintenance_margin_rate)?;

    let distance = if position.is_long {
        current_price.try_sub(liq_price)?
    } else {
        liq_price.try_sub(current_price)?
    };

    distance.try_div(current_price)
}

/// Calculate funding rate using mark-index premium model.
///
/// funding_rate = clamp(premium + interest_rate, -cap, +cap)
/// premium = (mark_price - index_price) / index_price
///
/// # Arguments
///
/// * `params` - Funding calculation parameters
///
/// # Returns
///
/// Funding rate as decimal for the funding interval.
pub fn calculate_funding_rate(params: &FundingParams) -> Result<Decimal, ArithmeticError> {
    // Premium = (mark - index) / index
    let premium = params
        .mark_price
        .try_sub(params.index_price)?
        .try_div(params.index_price)?;

    // Convert annual interest rate to funding interval rate
    let hours_per_year = Decimal::from(8760i64); // 365 * 24
    let interval_interest = params
        .interest_rate
        .try_mul(params.funding_interval_hours)?
        .try_div(hours_per_year)?;

    // Raw funding rate
    let raw_rate = premium.try_add(interval_interest)?;

    // Clamp to premium cap
    let capped_rate = if raw_rate > params.premium_cap {
        params.premium_cap
    } else if raw_rate < -params.premium_cap {
        -params.premium_cap
    } else {
        raw_rate
    };

    Ok(capped_rate)
}

/// Calculate funding payment for a position.
///
/// Longs pay shorts when funding rate is positive.
/// Shorts pay longs when funding rate is negative.
///
/// # Arguments
///
/// * `position` - The perpetual position
/// * `mark_price` - Current mark price
/// * `funding_rate` - Current funding rate as decimal
///
/// # Returns
///
/// Funding payment (positive = receive, negative = pay).
pub fn calculate_funding_payment(
    position: &PerpPosition,
    mark_price: Decimal,
    funding_rate: Decimal,
) -> Result<Decimal, ArithmeticError> {
    // Position value at mark
    let position_value = position.size.try_mul(mark_price)?;

    // Funding amount
    let funding_amount = position_value.try_mul(funding_rate)?;

    // Longs pay positive funding, receive negative
    // Shorts receive positive funding, pay negative
    if position.is_long {
        Ok(-funding_amount)
    } else {
        Ok(funding_amount)
    }
}

/// Calculate effective leverage based on current price.
///
/// effective_leverage = notional_value / (collateral + unrealized_pnl)
pub fn calculate_effective_leverage(
    position: &PerpPosition,
    current_price: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let notional = position.size.try_mul(current_price)?;
    let pnl = calculate_pnl(position, current_price)?;
    let equity = position.collateral.try_add(pnl)?;

    if equity <= Decimal::ZERO {
        return Err(ArithmeticError::DivisionByZero);
    }

    notional.try_div(equity)
}

/// Calculate margin ratio (inverse of effective leverage).
///
/// margin_ratio = equity / notional_value
pub fn calculate_margin_ratio(
    position: &PerpPosition,
    current_price: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let notional = position.size.try_mul(current_price)?;
    let pnl = calculate_pnl(position, current_price)?;
    let equity = position.collateral.try_add(pnl)?;

    equity.try_div(notional)
}

/// Calculate the maximum position size for given collateral and leverage.
///
/// max_size = (collateral * leverage) / entry_price
pub fn calculate_max_position_size(
    collateral: Decimal,
    leverage: Decimal,
    entry_price: Decimal,
) -> Result<Decimal, ArithmeticError> {
    collateral.try_mul(leverage)?.try_div(entry_price)
}

/// Calculate required collateral for a position.
///
/// required_collateral = (size * entry_price) / leverage
pub fn calculate_required_collateral(
    size: Decimal,
    entry_price: Decimal,
    leverage: Decimal,
) -> Result<Decimal, ArithmeticError> {
    size.try_mul(entry_price)?.try_div(leverage)
}

/// Calculate breakeven price accounting for fees.
///
/// # Arguments
///
/// * `position` - The perpetual position
/// * `open_fee_rate` - Opening fee as decimal
/// * `close_fee_rate` - Closing fee as decimal
pub fn calculate_breakeven_price(
    position: &PerpPosition,
    open_fee_rate: Decimal,
    close_fee_rate: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let notional = position.size.try_mul(position.entry_price)?;

    // Total fees as percentage of notional
    let total_fee_rate = open_fee_rate.try_add(close_fee_rate)?;
    let total_fees = notional.try_mul(total_fee_rate)?;

    // Price movement needed to cover fees
    let price_movement_for_fees = total_fees.try_div(position.size)?;

    if position.is_long {
        position.entry_price.try_add(price_movement_for_fees)
    } else {
        position.entry_price.try_sub(price_movement_for_fees)
    }
}

/// Calculate average entry price after adding to position.
///
/// # Arguments
///
/// * `existing_size` - Current position size
/// * `existing_avg_price` - Current average entry price
/// * `additional_size` - Size being added
/// * `additional_price` - Price of additional position
pub fn calculate_average_entry_price(
    existing_size: Decimal,
    existing_avg_price: Decimal,
    additional_size: Decimal,
    additional_price: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let existing_cost = existing_size.try_mul(existing_avg_price)?;
    let additional_cost = additional_size.try_mul(additional_price)?;
    let total_cost = existing_cost.try_add(additional_cost)?;
    let total_size = existing_size.try_add(additional_size)?;

    total_cost.try_div(total_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    fn decimal(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn sample_long_position() -> PerpPosition {
        PerpPosition {
            size: decimal("1.5"),          // 1.5 ETH
            entry_price: decimal("2000"),  // $2000
            is_long: true,
            leverage: decimal("10"),
            collateral: decimal("300"),    // $300
        }
    }

    fn sample_short_position() -> PerpPosition {
        PerpPosition {
            size: decimal("1.5"),
            entry_price: decimal("2000"),
            is_long: false,
            leverage: decimal("10"),
            collateral: decimal("300"),
        }
    }

    #[test]
    fn test_long_pnl_profit() {
        let position = sample_long_position();
        let current_price = decimal("2200"); // Price up $200

        let pnl = calculate_pnl(&position, current_price).unwrap();

        // PnL = 1.5 * (2200 - 2000) = 1.5 * 200 = 300
        assert_eq!(pnl, decimal("300"));
    }

    #[test]
    fn test_long_pnl_loss() {
        let position = sample_long_position();
        let current_price = decimal("1800"); // Price down $200

        let pnl = calculate_pnl(&position, current_price).unwrap();

        // PnL = 1.5 * (1800 - 2000) = 1.5 * -200 = -300
        assert_eq!(pnl, decimal("-300"));
    }

    #[test]
    fn test_short_pnl_profit() {
        let position = sample_short_position();
        let current_price = decimal("1800"); // Price down $200

        let pnl = calculate_pnl(&position, current_price).unwrap();

        // Short profits when price goes down
        // PnL = 1.5 * (2000 - 1800) = 1.5 * 200 = 300
        assert_eq!(pnl, decimal("300"));
    }

    #[test]
    fn test_short_pnl_loss() {
        let position = sample_short_position();
        let current_price = decimal("2200"); // Price up $200

        let pnl = calculate_pnl(&position, current_price).unwrap();

        // Short loses when price goes up
        // PnL = 1.5 * (2000 - 2200) = 1.5 * -200 = -300
        assert_eq!(pnl, decimal("-300"));
    }

    #[test]
    fn test_pnl_percentage() {
        let position = sample_long_position();
        let current_price = decimal("2200");

        let pnl_pct = calculate_pnl_percentage(&position, current_price).unwrap();

        // 300 PnL / 300 collateral = 100%
        assert_eq!(pnl_pct, decimal("1"));
    }

    #[test]
    fn test_liquidation_price_long() {
        let position = sample_long_position();
        let maintenance_rate = decimal("0.01"); // 1%

        let liq_price = calculate_liquidation_price(&position, maintenance_rate).unwrap();

        // Notional = 1.5 * 2000 = 3000
        // Maintenance = 3000 * 0.01 = 30
        // Max loss = 300 - 30 = 270
        // Price movement = 270 / 1.5 = 180
        // Liq price = 2000 - 180 = 1820
        assert_eq!(liq_price, decimal("1820"));
    }

    #[test]
    fn test_liquidation_price_short() {
        let position = sample_short_position();
        let maintenance_rate = decimal("0.01");

        let liq_price = calculate_liquidation_price(&position, maintenance_rate).unwrap();

        // Short: liquidated when price rises
        // Liq price = 2000 + 180 = 2180
        assert_eq!(liq_price, decimal("2180"));
    }

    #[test]
    fn test_funding_rate_positive_premium() {
        let params = FundingParams {
            mark_price: decimal("2020"),   // Mark above index
            index_price: decimal("2000"),
            interest_rate: decimal("0.0"),
            premium_cap: decimal("0.01"),
            funding_interval_hours: decimal("8"),
        };

        let rate = calculate_funding_rate(&params).unwrap();

        // Premium = (2020 - 2000) / 2000 = 0.01
        assert_eq!(rate, decimal("0.01"));
    }

    #[test]
    fn test_funding_rate_capped() {
        let params = FundingParams {
            mark_price: decimal("2100"),   // Large premium
            index_price: decimal("2000"),
            interest_rate: decimal("0.0"),
            premium_cap: decimal("0.01"),  // 1% cap
            funding_interval_hours: decimal("8"),
        };

        let rate = calculate_funding_rate(&params).unwrap();

        // Premium = 5% but capped at 1%
        assert_eq!(rate, decimal("0.01"));
    }

    #[test]
    fn test_funding_payment_long_positive_rate() {
        let position = sample_long_position();
        let mark_price = decimal("2000");
        let funding_rate = decimal("0.001"); // 0.1%

        let payment = calculate_funding_payment(&position, mark_price, funding_rate).unwrap();

        // Position value = 1.5 * 2000 = 3000
        // Funding = 3000 * 0.001 = 3
        // Long pays positive funding
        assert_eq!(payment, decimal("-3"));
    }

    #[test]
    fn test_funding_payment_short_positive_rate() {
        let position = sample_short_position();
        let mark_price = decimal("2000");
        let funding_rate = decimal("0.001");

        let payment = calculate_funding_payment(&position, mark_price, funding_rate).unwrap();

        // Short receives positive funding
        assert_eq!(payment, decimal("3"));
    }

    #[test]
    fn test_effective_leverage() {
        let position = sample_long_position();
        let current_price = decimal("2200"); // In profit

        let eff_leverage = calculate_effective_leverage(&position, current_price).unwrap();

        // Notional = 1.5 * 2200 = 3300
        // PnL = 300, equity = 300 + 300 = 600
        // Effective leverage = 3300 / 600 = 5.5
        assert_eq!(eff_leverage, decimal("5.5"));
    }

    #[test]
    fn test_max_position_size() {
        let collateral = decimal("1000");
        let leverage = decimal("10");
        let entry_price = decimal("2000");

        let max_size = calculate_max_position_size(collateral, leverage, entry_price).unwrap();

        // Max size = (1000 * 10) / 2000 = 5 ETH
        assert_eq!(max_size, decimal("5"));
    }

    #[test]
    fn test_required_collateral() {
        let size = decimal("5");
        let entry_price = decimal("2000");
        let leverage = decimal("10");

        let required = calculate_required_collateral(size, entry_price, leverage).unwrap();

        // Required = (5 * 2000) / 10 = 1000
        assert_eq!(required, decimal("1000"));
    }

    #[test]
    fn test_breakeven_price() {
        let position = sample_long_position();
        let open_fee = decimal("0.001");  // 0.1%
        let close_fee = decimal("0.001"); // 0.1%

        let breakeven = calculate_breakeven_price(&position, open_fee, close_fee).unwrap();

        // Notional = 1.5 * 2000 = 3000
        // Total fees = 3000 * 0.002 = 6
        // Price movement = 6 / 1.5 = 4
        // Breakeven = 2000 + 4 = 2004
        assert_eq!(breakeven, decimal("2004"));
    }

    #[test]
    fn test_average_entry_price() {
        let existing_size = decimal("1.0");
        let existing_price = decimal("2000");
        let additional_size = decimal("0.5");
        let additional_price = decimal("2100");

        let avg = calculate_average_entry_price(
            existing_size,
            existing_price,
            additional_size,
            additional_price,
        )
        .unwrap();

        // Total cost = 1.0 * 2000 + 0.5 * 2100 = 2000 + 1050 = 3050
        // Total size = 1.5
        // Avg = 3050 / 1.5 = 2033.333...
        let expected = decimal("3050").try_div(decimal("1.5")).unwrap();
        assert_eq!(avg, expected);
    }
}
