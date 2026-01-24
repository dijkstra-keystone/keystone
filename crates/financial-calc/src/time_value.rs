//! Time value of money calculations.

use precision_core::{ArithmeticError, Decimal};

/// Calculates future value of a present amount.
///
/// Formula: `present_value * (1 + rate)^periods`
///
/// - `pv`: Present value
/// - `rate`: Interest rate per period as decimal
/// - `periods`: Number of periods
pub fn future_value(pv: Decimal, rate: Decimal, periods: u32) -> Result<Decimal, ArithmeticError> {
    let factor = compound_factor(rate, periods)?;
    pv.try_mul(factor)
}

/// Calculates present value of a future amount.
///
/// Formula: `future_value / (1 + rate)^periods`
///
/// - `fv`: Future value
/// - `rate`: Discount rate per period as decimal
/// - `periods`: Number of periods
pub fn present_value(fv: Decimal, rate: Decimal, periods: u32) -> Result<Decimal, ArithmeticError> {
    let factor = compound_factor(rate, periods)?;
    fv.try_div(factor)
}

/// Calculates net present value of a series of cash flows.
///
/// Formula: `sum(cash_flow_t / (1 + rate)^t)` for t = 0 to n
///
/// The first cash flow (index 0) is typically the initial investment (negative).
///
/// - `rate`: Discount rate per period as decimal
/// - `cash_flows`: Iterator of cash flows, starting at period 0
pub fn net_present_value<I>(rate: Decimal, cash_flows: I) -> Result<Decimal, ArithmeticError>
where
    I: IntoIterator<Item = Decimal>,
{
    let mut npv = Decimal::ZERO;
    let mut period = 0u32;

    for cf in cash_flows {
        let pv = present_value(cf, rate, period)?;
        npv = npv.try_add(pv)?;
        period = period.saturating_add(1);
    }

    Ok(npv)
}

/// Calculates the compound factor (1 + rate)^periods.
fn compound_factor(rate: Decimal, periods: u32) -> Result<Decimal, ArithmeticError> {
    if periods == 0 {
        return Ok(Decimal::ONE);
    }

    let base = Decimal::ONE.try_add(rate)?;
    pow_checked(base, periods)
}

/// Integer exponentiation with overflow checking.
fn pow_checked(base: Decimal, exp: u32) -> Result<Decimal, ArithmeticError> {
    if exp == 0 {
        return Ok(Decimal::ONE);
    }

    let mut result = Decimal::ONE;
    let mut current_base = base;
    let mut remaining = exp;

    while remaining > 0 {
        if remaining & 1 == 1 {
            result = result.try_mul(current_base)?;
        }
        remaining >>= 1;
        if remaining > 0 {
            current_base = current_base.try_mul(current_base)?;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use precision_core::RoundingMode;

    #[test]
    fn future_value_basic() {
        let pv = Decimal::from(1000i64);
        let rate = Decimal::new(10, 2); // 10%
        let periods = 1;

        let fv = future_value(pv, rate, periods).unwrap();
        assert_eq!(fv, Decimal::from(1100i64));
    }

    #[test]
    fn future_value_multiple_periods() {
        let pv = Decimal::from(1000i64);
        let rate = Decimal::new(10, 2); // 10%
        let periods = 3;

        let fv = future_value(pv, rate, periods).unwrap();
        // 1000 * 1.1^3 = 1331
        assert_eq!(fv, Decimal::from(1331i64));
    }

    #[test]
    fn future_value_zero_periods() {
        let pv = Decimal::from(1000i64);
        let rate = Decimal::new(10, 2);

        let fv = future_value(pv, rate, 0).unwrap();
        assert_eq!(fv, pv);
    }

    #[test]
    fn present_value_basic() {
        let fv = Decimal::from(1100i64);
        let rate = Decimal::new(10, 2); // 10%
        let periods = 1;

        let pv = present_value(fv, rate, periods).unwrap();
        assert_eq!(pv, Decimal::from(1000i64));
    }

    #[test]
    fn present_value_multiple_periods() {
        let fv = Decimal::from(1331i64);
        let rate = Decimal::new(10, 2); // 10%
        let periods = 3;

        let pv = present_value(fv, rate, periods).unwrap();
        assert_eq!(pv, Decimal::from(1000i64));
    }

    #[test]
    fn pv_fv_inverse() {
        let original = Decimal::from(5000i64);
        let rate = Decimal::new(8, 2); // 8%
        let periods = 5;

        let fv = future_value(original, rate, periods).unwrap();
        let recovered = present_value(fv, rate, periods).unwrap();

        // Should recover original value (may have small precision loss)
        let diff = (recovered - original).abs();
        assert!(diff < Decimal::new(1, 10));
    }

    #[test]
    fn npv_positive_project() {
        let rate = Decimal::new(10, 2); // 10%

        // Initial investment of -1000, then cash flows of 400 for 4 years
        let cash_flows = [
            Decimal::from(-1000i64),
            Decimal::from(400i64),
            Decimal::from(400i64),
            Decimal::from(400i64),
            Decimal::from(400i64),
        ];

        let npv = net_present_value(rate, cash_flows).unwrap();
        let rounded = npv.round(2, RoundingMode::HalfEven);

        // NPV should be approximately 267.95
        assert!(npv > Decimal::ZERO);
        assert_eq!(rounded, Decimal::new(26795, 2));
    }

    #[test]
    fn npv_negative_project() {
        let rate = Decimal::new(10, 2); // 10%

        // Initial investment of -1000, then cash flows of 200 for 4 years
        let cash_flows = [
            Decimal::from(-1000i64),
            Decimal::from(200i64),
            Decimal::from(200i64),
            Decimal::from(200i64),
            Decimal::from(200i64),
        ];

        let npv = net_present_value(rate, cash_flows).unwrap();

        // NPV should be negative (bad investment)
        assert!(npv < Decimal::ZERO);
    }

    #[test]
    fn npv_zero_rate() {
        let rate = Decimal::ZERO;

        let cash_flows = [
            Decimal::from(-100i64),
            Decimal::from(50i64),
            Decimal::from(50i64),
            Decimal::from(50i64),
        ];

        let npv = net_present_value(rate, cash_flows).unwrap();
        // With 0% discount rate, NPV = sum of cash flows = 50
        assert_eq!(npv, Decimal::from(50i64));
    }
}
