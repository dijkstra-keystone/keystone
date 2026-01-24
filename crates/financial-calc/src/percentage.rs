//! Percentage calculations.

use precision_core::{ArithmeticError, Decimal};

const ONE_HUNDRED: Decimal = Decimal::ONE_HUNDRED;

/// Calculates the percentage of a value.
///
/// Returns `value * (percentage / 100)`.
pub fn percentage_of(value: Decimal, percentage: Decimal) -> Result<Decimal, ArithmeticError> {
    value.try_mul(percentage)?.try_div(ONE_HUNDRED)
}

/// Calculates the percentage change between two values.
///
/// Returns `((new - old) / old) * 100`.
///
/// Returns `DivisionByZero` if `old_value` is zero.
pub fn percentage_change(
    old_value: Decimal,
    new_value: Decimal,
) -> Result<Decimal, ArithmeticError> {
    let diff = new_value.try_sub(old_value)?;
    let ratio = diff.try_div(old_value)?;
    ratio.try_mul(ONE_HUNDRED)
}

/// Converts basis points to a decimal rate.
///
/// One basis point equals 0.01% or 0.0001 in decimal form.
/// 100 basis points = 1% = 0.01
pub fn basis_points_to_decimal(bps: Decimal) -> Result<Decimal, ArithmeticError> {
    bps.try_div(Decimal::new(10000, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentage_of_basic() {
        let value = Decimal::from(200i64);
        let pct = Decimal::from(25i64);
        assert_eq!(percentage_of(value, pct).unwrap(), Decimal::from(50i64));
    }

    #[test]
    fn percentage_of_decimal() {
        let value = Decimal::new(1000, 0);
        let pct = Decimal::new(75, 1); // 7.5%
        let result = percentage_of(value, pct).unwrap();
        assert_eq!(result, Decimal::from(75i64));
    }

    #[test]
    fn percentage_change_increase() {
        let old = Decimal::from(100i64);
        let new = Decimal::from(125i64);
        let result = percentage_change(old, new).unwrap();
        assert_eq!(result, Decimal::from(25i64));
    }

    #[test]
    fn percentage_change_decrease() {
        let old = Decimal::from(100i64);
        let new = Decimal::from(80i64);
        let result = percentage_change(old, new).unwrap();
        assert_eq!(result, Decimal::from(-20i64));
    }

    #[test]
    fn percentage_change_zero_old() {
        let old = Decimal::ZERO;
        let new = Decimal::from(100i64);
        assert!(matches!(
            percentage_change(old, new),
            Err(ArithmeticError::DivisionByZero)
        ));
    }

    #[test]
    fn basis_points_conversion() {
        let bps = Decimal::from(100i64); // 100 bps = 1%
        let decimal = basis_points_to_decimal(bps).unwrap();
        assert_eq!(decimal, Decimal::new(1, 2)); // 0.01

        let bps = Decimal::from(50i64); // 50 bps = 0.5%
        let decimal = basis_points_to_decimal(bps).unwrap();
        assert_eq!(decimal, Decimal::new(5, 3)); // 0.005
    }
}
