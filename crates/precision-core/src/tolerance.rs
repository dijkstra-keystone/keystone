//! Tolerance-based comparison operations.

use crate::decimal::Decimal;

/// Compares two decimals with an absolute tolerance.
///
/// Returns `true` if `|a - b| <= tolerance`.
#[must_use]
pub fn approx_eq(a: Decimal, b: Decimal, tolerance: Decimal) -> bool {
    let diff = if a >= b { a - b } else { b - a };
    diff <= tolerance
}

/// Compares two decimals with a relative tolerance.
///
/// Returns `true` if `|a - b| <= max(|a|, |b|) * relative_tolerance`.
///
/// For comparing values near zero, use `approx_eq` with an absolute tolerance instead.
#[must_use]
pub fn approx_eq_relative(a: Decimal, b: Decimal, relative_tolerance: Decimal) -> bool {
    if a == b {
        return true;
    }

    let diff = (a - b).abs();
    let max_abs = a.abs().max(b.abs());

    if max_abs.is_zero() {
        return diff.is_zero();
    }

    if let Some(threshold) = max_abs.checked_mul(relative_tolerance) {
        diff <= threshold
    } else {
        false
    }
}

/// Compares two decimals with both absolute and relative tolerances.
///
/// Returns `true` if either tolerance check passes.
/// This handles both small values (where absolute tolerance matters)
/// and large values (where relative tolerance matters).
#[must_use]
pub fn approx_eq_ulps(
    a: Decimal,
    b: Decimal,
    absolute_tolerance: Decimal,
    relative_tolerance: Decimal,
) -> bool {
    approx_eq(a, b, absolute_tolerance) || approx_eq_relative(a, b, relative_tolerance)
}

/// Checks if a value is within a percentage of another value.
///
/// Returns `true` if `|a - b| <= |b| * (percentage / 100)`.
#[must_use]
pub fn within_percentage(a: Decimal, b: Decimal, percentage: Decimal) -> bool {
    if b.is_zero() {
        return a.is_zero();
    }

    let diff = (a - b).abs();
    let threshold = b
        .abs()
        .checked_mul(percentage)
        .and_then(|v| v.checked_div(Decimal::ONE_HUNDRED));

    match threshold {
        Some(t) => diff <= t,
        None => false,
    }
}

/// Checks if a value is within a basis point tolerance of another value.
///
/// One basis point = 0.01% = 0.0001.
#[must_use]
pub fn within_basis_points(a: Decimal, b: Decimal, bps: Decimal) -> bool {
    if b.is_zero() {
        return a.is_zero();
    }

    let diff = (a - b).abs();
    let threshold = b
        .abs()
        .checked_mul(bps)
        .and_then(|v| v.checked_div(Decimal::new(10000, 0)));

    match threshold {
        Some(t) => diff <= t,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approx_eq_exact() {
        let a = Decimal::new(100, 2);
        let tolerance = Decimal::new(1, 4);
        assert!(approx_eq(a, a, tolerance));
    }

    #[test]
    fn approx_eq_within_tolerance() {
        let a = Decimal::new(1000, 3);
        let b = Decimal::new(1001, 3);
        let tolerance = Decimal::new(1, 3);
        assert!(approx_eq(a, b, tolerance));
    }

    #[test]
    fn approx_eq_outside_tolerance() {
        let a = Decimal::new(1000, 3);
        let b = Decimal::new(1002, 3);
        let tolerance = Decimal::new(1, 3);
        assert!(!approx_eq(a, b, tolerance));
    }

    #[test]
    fn approx_eq_relative_basic() {
        let a = Decimal::from(100i64);
        let b = Decimal::from(101i64);
        let tolerance = Decimal::new(2, 2); // 2%
        assert!(approx_eq_relative(a, b, tolerance));
    }

    #[test]
    fn approx_eq_relative_large_values() {
        let a = Decimal::from(1_000_000i64);
        let b = Decimal::from(1_000_100i64);
        let tolerance = Decimal::new(1, 3); // 0.1%
        assert!(approx_eq_relative(a, b, tolerance));
    }

    #[test]
    fn within_percentage_basic() {
        let a = Decimal::from(102i64);
        let b = Decimal::from(100i64);
        assert!(within_percentage(a, b, Decimal::from(5i64))); // within 5%
        assert!(!within_percentage(a, b, Decimal::from(1i64))); // not within 1%
    }

    #[test]
    fn within_basis_points_basic() {
        let a = Decimal::new(10010, 2); // 100.10
        let b = Decimal::from(100i64);
        // 100 bps of 100 = 1.0, difference is 0.10, so within 100 bps
        assert!(within_basis_points(a, b, Decimal::from(100i64))); // within 100 bps (1%)
        // 5 bps of 100 = 0.05, difference is 0.10, so NOT within 5 bps
        assert!(!within_basis_points(a, b, Decimal::from(5i64))); // not within 5 bps
    }

    #[test]
    fn within_basis_points_zero() {
        let a = Decimal::from(100i64);
        let b = Decimal::ZERO;
        assert!(!within_basis_points(a, b, Decimal::from(100i64)));
        assert!(within_basis_points(Decimal::ZERO, b, Decimal::from(100i64)));
    }
}
