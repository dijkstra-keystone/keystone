//! Interest calculations.

use precision_core::{ArithmeticError, Decimal};

/// Calculates simple interest.
///
/// Formula: `principal * rate * time`
///
/// - `principal`: Initial amount
/// - `rate`: Interest rate as decimal (e.g., 0.05 for 5%)
/// - `periods`: Number of time periods
pub fn simple_interest(
    principal: Decimal,
    rate: Decimal,
    periods: Decimal,
) -> Result<Decimal, ArithmeticError> {
    principal.try_mul(rate)?.try_mul(periods)
}

/// Calculates compound interest (final amount minus principal).
///
/// Formula: `principal * ((1 + rate/n)^(n*t) - 1)`
///
/// - `principal`: Initial amount
/// - `rate`: Annual interest rate as decimal
/// - `compounds_per_period`: Number of compounding periods per year (n)
/// - `periods`: Number of years (t)
///
/// Uses iterative multiplication for integer exponentiation to maintain precision.
pub fn compound_interest(
    principal: Decimal,
    rate: Decimal,
    compounds_per_period: u32,
    periods: u32,
) -> Result<Decimal, ArithmeticError> {
    if compounds_per_period == 0 {
        return Err(ArithmeticError::DivisionByZero);
    }

    let n = Decimal::from(compounds_per_period as i64);
    let rate_per_compound = rate.try_div(n)?;
    let base = Decimal::ONE.try_add(rate_per_compound)?;

    let total_compounds = compounds_per_period.saturating_mul(periods);
    let final_factor = pow_checked(base, total_compounds)?;

    let final_amount = principal.try_mul(final_factor)?;
    final_amount.try_sub(principal)
}

/// Calculates the effective annual rate from a nominal rate.
///
/// Formula: `(1 + nominal_rate/n)^n - 1`
///
/// - `nominal_rate`: Stated annual rate as decimal
/// - `compounds_per_year`: Number of compounding periods per year
pub fn effective_annual_rate(
    nominal_rate: Decimal,
    compounds_per_year: u32,
) -> Result<Decimal, ArithmeticError> {
    if compounds_per_year == 0 {
        return Err(ArithmeticError::DivisionByZero);
    }

    let n = Decimal::from(compounds_per_year as i64);
    let rate_per_period = nominal_rate.try_div(n)?;
    let base = Decimal::ONE.try_add(rate_per_period)?;
    let factor = pow_checked(base, compounds_per_year)?;
    factor.try_sub(Decimal::ONE)
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
    fn simple_interest_basic() {
        let principal = Decimal::from(1000i64);
        let rate = Decimal::new(5, 2); // 0.05 = 5%
        let periods = Decimal::from(2i64);

        let interest = simple_interest(principal, rate, periods).unwrap();
        assert_eq!(interest, Decimal::from(100i64));
    }

    #[test]
    fn simple_interest_fractional() {
        let principal = Decimal::from(10000i64);
        let rate = Decimal::new(75, 3); // 0.075 = 7.5%
        let periods = Decimal::new(5, 1); // 0.5 years

        let interest = simple_interest(principal, rate, periods).unwrap();
        assert_eq!(interest, Decimal::from(375i64));
    }

    #[test]
    fn compound_interest_annual() {
        let principal = Decimal::from(1000i64);
        let rate = Decimal::new(10, 2); // 10%
        let compounds = 1; // annually
        let years = 2;

        let interest = compound_interest(principal, rate, compounds, years).unwrap();
        // 1000 * (1.1)^2 - 1000 = 1210 - 1000 = 210
        assert_eq!(interest, Decimal::from(210i64));
    }

    #[test]
    fn compound_interest_monthly() {
        let principal = Decimal::from(1000i64);
        let rate = Decimal::new(12, 2); // 12%
        let compounds = 12; // monthly
        let years = 1;

        let interest = compound_interest(principal, rate, compounds, years).unwrap();
        // Should be approximately 126.83 (more than simple interest of 120)
        let rounded = interest.round(2, RoundingMode::HalfEven);
        assert_eq!(rounded, Decimal::new(12683, 2));
    }

    #[test]
    fn compound_interest_zero_compounds() {
        let principal = Decimal::from(1000i64);
        let rate = Decimal::new(10, 2);

        assert!(matches!(
            compound_interest(principal, rate, 0, 1),
            Err(ArithmeticError::DivisionByZero)
        ));
    }

    #[test]
    fn effective_annual_rate_monthly() {
        let nominal = Decimal::new(12, 2); // 12% nominal
        let ear = effective_annual_rate(nominal, 12).unwrap();

        // EAR should be approximately 12.68%
        let rounded = ear.round(4, RoundingMode::HalfEven);
        assert_eq!(rounded, Decimal::new(1268, 4));
    }

    #[test]
    fn effective_annual_rate_continuous_approximation() {
        let nominal = Decimal::new(10, 2); // 10%

        // With very high compounding frequency, EAR approaches e^r - 1
        let ear_daily = effective_annual_rate(nominal, 365).unwrap();
        let ear_monthly = effective_annual_rate(nominal, 12).unwrap();

        // Daily compounding should give higher EAR than monthly
        assert!(ear_daily > ear_monthly);
    }

    #[test]
    fn pow_checked_basic() {
        assert_eq!(pow_checked(Decimal::from(2i64), 0).unwrap(), Decimal::ONE);
        assert_eq!(
            pow_checked(Decimal::from(2i64), 1).unwrap(),
            Decimal::from(2i64)
        );
        assert_eq!(
            pow_checked(Decimal::from(2i64), 10).unwrap(),
            Decimal::from(1024i64)
        );
    }

    #[test]
    fn pow_checked_decimal() {
        let base = Decimal::new(11, 1); // 1.1
        let result = pow_checked(base, 2).unwrap();
        assert_eq!(result, Decimal::new(121, 2)); // 1.21
    }
}
