//! Health factor calculations for DeFi lending positions.

use precision_core::{ArithmeticError, Decimal};

/// Calculates the health factor of a lending position.
///
/// Formula: `(collateral_value * liquidation_threshold) / debt_value`
///
/// - Health factor > 1.0: Position is healthy
/// - Health factor = 1.0: Position is at liquidation threshold
/// - Health factor < 1.0: Position can be liquidated
///
/// Returns `DivisionByZero` if `debt_value` is zero.
pub fn health_factor(
    collateral_value: Decimal,
    debt_value: Decimal,
    liquidation_threshold: Decimal,
) -> Result<Decimal, ArithmeticError> {
    if debt_value.is_zero() {
        return Err(ArithmeticError::DivisionByZero);
    }

    let weighted_collateral = collateral_value.try_mul(liquidation_threshold)?;
    weighted_collateral.try_div(debt_value)
}

/// Checks if a position is healthy (health factor >= minimum).
///
/// A position is typically considered healthy if health factor >= 1.0,
/// but protocols may use different minimum thresholds.
pub fn is_healthy(
    collateral_value: Decimal,
    debt_value: Decimal,
    liquidation_threshold: Decimal,
    min_health_factor: Decimal,
) -> Result<bool, ArithmeticError> {
    if debt_value.is_zero() {
        return Ok(true); // No debt means healthy
    }

    let hf = health_factor(collateral_value, debt_value, liquidation_threshold)?;
    Ok(hf >= min_health_factor)
}

/// Calculates the collateral ratio (collateral / debt).
///
/// Returns `DivisionByZero` if `debt_value` is zero.
pub fn collateral_ratio(
    collateral_value: Decimal,
    debt_value: Decimal,
) -> Result<Decimal, ArithmeticError> {
    if debt_value.is_zero() {
        return Err(ArithmeticError::DivisionByZero);
    }
    collateral_value.try_div(debt_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_factor_healthy() {
        let collateral = Decimal::from(1000i64);
        let debt = Decimal::from(500i64);
        let threshold = Decimal::new(80, 2); // 0.80 = 80%

        let hf = health_factor(collateral, debt, threshold).unwrap();
        // (1000 * 0.80) / 500 = 800 / 500 = 1.6
        assert_eq!(hf, Decimal::new(16, 1));
    }

    #[test]
    fn health_factor_at_threshold() {
        let collateral = Decimal::from(1000i64);
        let debt = Decimal::from(800i64);
        let threshold = Decimal::new(80, 2); // 80%

        let hf = health_factor(collateral, debt, threshold).unwrap();
        // (1000 * 0.80) / 800 = 800 / 800 = 1.0
        assert_eq!(hf, Decimal::ONE);
    }

    #[test]
    fn health_factor_unhealthy() {
        let collateral = Decimal::from(1000i64);
        let debt = Decimal::from(900i64);
        let threshold = Decimal::new(80, 2); // 80%

        let hf = health_factor(collateral, debt, threshold).unwrap();
        // (1000 * 0.80) / 900 = 800 / 900 ≈ 0.889
        assert!(hf < Decimal::ONE);
    }

    #[test]
    fn health_factor_zero_debt() {
        let collateral = Decimal::from(1000i64);
        let debt = Decimal::ZERO;
        let threshold = Decimal::new(80, 2);

        assert!(matches!(
            health_factor(collateral, debt, threshold),
            Err(ArithmeticError::DivisionByZero)
        ));
    }

    #[test]
    fn is_healthy_no_debt() {
        let collateral = Decimal::from(1000i64);
        let debt = Decimal::ZERO;
        let threshold = Decimal::new(80, 2);
        let min_hf = Decimal::ONE;

        assert!(is_healthy(collateral, debt, threshold, min_hf).unwrap());
    }

    #[test]
    fn is_healthy_with_debt() {
        let collateral = Decimal::from(1000i64);
        let debt = Decimal::from(500i64);
        let threshold = Decimal::new(80, 2);
        let min_hf = Decimal::ONE;

        assert!(is_healthy(collateral, debt, threshold, min_hf).unwrap());
    }

    #[test]
    fn is_healthy_unhealthy() {
        let collateral = Decimal::from(1000i64);
        let debt = Decimal::from(900i64);
        let threshold = Decimal::new(80, 2);
        let min_hf = Decimal::ONE;

        assert!(!is_healthy(collateral, debt, threshold, min_hf).unwrap());
    }

    #[test]
    fn collateral_ratio_basic() {
        let collateral = Decimal::from(1500i64);
        let debt = Decimal::from(1000i64);

        let ratio = collateral_ratio(collateral, debt).unwrap();
        assert_eq!(ratio, Decimal::new(15, 1)); // 1.5
    }
}
