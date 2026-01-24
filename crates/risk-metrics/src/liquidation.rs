//! Liquidation calculations for DeFi lending.

use precision_core::{ArithmeticError, Decimal};

/// Calculates the price at which a position becomes liquidatable.
///
/// Formula: `(debt_value * liquidation_threshold) / collateral_amount`
///
/// This returns the collateral price below which the health factor drops to 1.0.
pub fn liquidation_price(
    collateral_amount: Decimal,
    debt_value: Decimal,
    liquidation_threshold: Decimal,
) -> Result<Decimal, ArithmeticError> {
    if collateral_amount.is_zero() {
        return Err(ArithmeticError::DivisionByZero);
    }
    if liquidation_threshold.is_zero() {
        return Err(ArithmeticError::DivisionByZero);
    }

    debt_value.try_div(collateral_amount.try_mul(liquidation_threshold)?)
}

/// Calculates the liquidation threshold value.
///
/// Returns the maximum debt allowed before liquidation for a given collateral value.
///
/// Formula: `collateral_value * liquidation_threshold`
pub fn liquidation_threshold(
    collateral_value: Decimal,
    threshold_percentage: Decimal,
) -> Result<Decimal, ArithmeticError> {
    collateral_value.try_mul(threshold_percentage)
}

/// Calculates maximum borrowable amount given collateral.
///
/// Formula: `(collateral_value * max_ltv) - current_debt`
///
/// Returns zero if result would be negative (already over-borrowed).
pub fn max_borrowable(
    collateral_value: Decimal,
    max_ltv: Decimal,
    current_debt: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let max_total = collateral_value.try_mul(max_ltv)?;
    let available = max_total.try_sub(current_debt)?;

    if available.is_negative() {
        Ok(Decimal::ZERO)
    } else {
        Ok(available)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liquidation_price_basic() {
        let collateral_amount = Decimal::from(10i64); // 10 ETH
        let debt_value = Decimal::from(12000i64); // $12,000 debt
        let threshold = Decimal::new(80, 2); // 80%

        let liq_price = liquidation_price(collateral_amount, debt_value, threshold).unwrap();
        // $12,000 / (10 * 0.80) = $12,000 / 8 = $1,500
        assert_eq!(liq_price, Decimal::from(1500i64));
    }

    #[test]
    fn liquidation_price_zero_collateral() {
        let collateral_amount = Decimal::ZERO;
        let debt_value = Decimal::from(1000i64);
        let threshold = Decimal::new(80, 2);

        assert!(matches!(
            liquidation_price(collateral_amount, debt_value, threshold),
            Err(ArithmeticError::DivisionByZero)
        ));
    }

    #[test]
    fn liquidation_threshold_basic() {
        let collateral_value = Decimal::from(10000i64);
        let threshold = Decimal::new(75, 2); // 75%

        let max_debt = liquidation_threshold(collateral_value, threshold).unwrap();
        assert_eq!(max_debt, Decimal::from(7500i64));
    }

    #[test]
    fn max_borrowable_basic() {
        let collateral_value = Decimal::from(10000i64);
        let max_ltv = Decimal::new(50, 2); // 50%
        let current_debt = Decimal::from(2000i64);

        let available = max_borrowable(collateral_value, max_ltv, current_debt).unwrap();
        // (10000 * 0.50) - 2000 = 5000 - 2000 = 3000
        assert_eq!(available, Decimal::from(3000i64));
    }

    #[test]
    fn max_borrowable_over_limit() {
        let collateral_value = Decimal::from(10000i64);
        let max_ltv = Decimal::new(50, 2); // 50%
        let current_debt = Decimal::from(6000i64); // Already over 50% LTV

        let available = max_borrowable(collateral_value, max_ltv, current_debt).unwrap();
        assert_eq!(available, Decimal::ZERO);
    }

    #[test]
    fn max_borrowable_no_debt() {
        let collateral_value = Decimal::from(10000i64);
        let max_ltv = Decimal::new(75, 2); // 75%
        let current_debt = Decimal::ZERO;

        let available = max_borrowable(collateral_value, max_ltv, current_debt).unwrap();
        assert_eq!(available, Decimal::from(7500i64));
    }

    #[test]
    fn liquidation_price_realistic() {
        // ETH at $2,000, user deposits 5 ETH ($10,000 collateral)
        // Borrows $6,000 USDC with 80% liquidation threshold
        let collateral_amount = Decimal::from(5i64);
        let debt_value = Decimal::from(6000i64);
        let threshold = Decimal::new(80, 2);

        let liq_price = liquidation_price(collateral_amount, debt_value, threshold).unwrap();
        // $6,000 / (5 * 0.80) = $6,000 / 4 = $1,500
        // If ETH drops to $1,500, position gets liquidated
        assert_eq!(liq_price, Decimal::from(1500i64));
    }
}
