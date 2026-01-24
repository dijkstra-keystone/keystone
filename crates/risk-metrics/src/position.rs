//! Position metrics for DeFi protocols.

use precision_core::{ArithmeticError, Decimal};

/// Calculates loan-to-value ratio.
///
/// Formula: `debt_value / collateral_value`
///
/// Returns as a decimal (0.5 = 50% LTV).
pub fn loan_to_value(
    debt_value: Decimal,
    collateral_value: Decimal,
) -> Result<Decimal, ArithmeticError> {
    if collateral_value.is_zero() {
        return Err(ArithmeticError::DivisionByZero);
    }
    debt_value.try_div(collateral_value)
}

/// Calculates utilization rate of a lending pool.
///
/// Formula: `total_borrows / total_supply`
///
/// Returns as a decimal (0.75 = 75% utilization).
pub fn utilization_rate(
    total_borrows: Decimal,
    total_supply: Decimal,
) -> Result<Decimal, ArithmeticError> {
    if total_supply.is_zero() {
        if total_borrows.is_zero() {
            return Ok(Decimal::ZERO);
        }
        return Err(ArithmeticError::DivisionByZero);
    }
    total_borrows.try_div(total_supply)
}

/// Calculates available liquidity in a pool.
///
/// Formula: `total_supply - total_borrows`
///
/// Returns zero if borrows exceed supply (shouldn't happen in normal operation).
pub fn available_liquidity(
    total_supply: Decimal,
    total_borrows: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let liquidity = total_supply.try_sub(total_borrows)?;
    if liquidity.is_negative() {
        Ok(Decimal::ZERO)
    } else {
        Ok(liquidity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ltv_basic() {
        let debt = Decimal::from(5000i64);
        let collateral = Decimal::from(10000i64);

        let ltv = loan_to_value(debt, collateral).unwrap();
        assert_eq!(ltv, Decimal::new(5, 1)); // 0.5 = 50%
    }

    #[test]
    fn ltv_zero_collateral() {
        let debt = Decimal::from(1000i64);
        let collateral = Decimal::ZERO;

        assert!(matches!(
            loan_to_value(debt, collateral),
            Err(ArithmeticError::DivisionByZero)
        ));
    }

    #[test]
    fn ltv_no_debt() {
        let debt = Decimal::ZERO;
        let collateral = Decimal::from(10000i64);

        let ltv = loan_to_value(debt, collateral).unwrap();
        assert_eq!(ltv, Decimal::ZERO);
    }

    #[test]
    fn utilization_basic() {
        let borrows = Decimal::from(75000i64);
        let supply = Decimal::from(100000i64);

        let util = utilization_rate(borrows, supply).unwrap();
        assert_eq!(util, Decimal::new(75, 2)); // 0.75 = 75%
    }

    #[test]
    fn utilization_empty_pool() {
        let borrows = Decimal::ZERO;
        let supply = Decimal::ZERO;

        let util = utilization_rate(borrows, supply).unwrap();
        assert_eq!(util, Decimal::ZERO);
    }

    #[test]
    fn utilization_full() {
        let borrows = Decimal::from(100000i64);
        let supply = Decimal::from(100000i64);

        let util = utilization_rate(borrows, supply).unwrap();
        assert_eq!(util, Decimal::ONE);
    }

    #[test]
    fn available_liquidity_basic() {
        let supply = Decimal::from(100000i64);
        let borrows = Decimal::from(75000i64);

        let liquidity = available_liquidity(supply, borrows).unwrap();
        assert_eq!(liquidity, Decimal::from(25000i64));
    }

    #[test]
    fn available_liquidity_full_utilization() {
        let supply = Decimal::from(100000i64);
        let borrows = Decimal::from(100000i64);

        let liquidity = available_liquidity(supply, borrows).unwrap();
        assert_eq!(liquidity, Decimal::ZERO);
    }

    #[test]
    fn available_liquidity_no_borrows() {
        let supply = Decimal::from(100000i64);
        let borrows = Decimal::ZERO;

        let liquidity = available_liquidity(supply, borrows).unwrap();
        assert_eq!(liquidity, supply);
    }
}
