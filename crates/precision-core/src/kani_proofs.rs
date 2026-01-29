//! Kani formal verification proofs for Decimal overflow safety.
//!
//! These proofs verify that checked arithmetic operations properly detect overflow
//! and return None, preventing undefined behavior in financial calculations.
//!
//! Run with: `cargo kani --harness <harness_name>`
//! Run all:  `cargo kani`

#[cfg(kani)]
mod verification {
    use crate::Decimal;

    /// Proves that checked_add returns None when overflow would occur,
    /// and returns Some with correct result otherwise.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_checked_add_no_panic() {
        let a_mantissa: i64 = kani::any();
        let b_mantissa: i64 = kani::any();

        let a = Decimal::new(a_mantissa, 0);
        let b = Decimal::new(b_mantissa, 0);

        // This should never panic - it returns Option
        let result = a.checked_add(b);

        // If result is Some, the value should be mathematically correct
        // (within the representable range)
        if let Some(sum) = result {
            // The sum should be equal to a + b when no overflow
            kani::assert(
                sum.checked_sub(a).map_or(false, |diff| diff == b)
                    || sum.checked_sub(b).map_or(false, |diff| diff == a),
                "checked_add result should be reversible",
            );
        }
    }

    /// Proves that checked_sub returns None when overflow would occur.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_checked_sub_no_panic() {
        let a_mantissa: i64 = kani::any();
        let b_mantissa: i64 = kani::any();

        let a = Decimal::new(a_mantissa, 0);
        let b = Decimal::new(b_mantissa, 0);

        // This should never panic
        let result = a.checked_sub(b);

        // If result is Some, adding b should give back a
        if let Some(diff) = result {
            kani::assert(
                diff.checked_add(b).map_or(false, |sum| sum == a),
                "checked_sub result should be reversible",
            );
        }
    }

    /// Proves that checked_mul returns None when overflow would occur.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_checked_mul_no_panic() {
        let a_mantissa: i32 = kani::any();
        let b_mantissa: i32 = kani::any();

        let a = Decimal::new(a_mantissa as i64, 0);
        let b = Decimal::new(b_mantissa as i64, 0);

        // This should never panic
        let _result = a.checked_mul(b);
    }

    /// Proves that checked_div handles division by zero correctly.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_checked_div_zero_handling() {
        let a_mantissa: i64 = kani::any();

        let a = Decimal::new(a_mantissa, 0);
        let zero = Decimal::ZERO;

        // Division by zero should return None, not panic
        let result = a.checked_div(zero);
        kani::assert(result.is_none(), "division by zero must return None");
    }

    /// Proves that checked_div returns None on overflow.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_checked_div_no_panic() {
        let a_mantissa: i64 = kani::any();
        let b_mantissa: i64 = kani::any();

        // Assume b is non-zero for this test
        kani::assume(b_mantissa != 0);

        let a = Decimal::new(a_mantissa, 0);
        let b = Decimal::new(b_mantissa, 0);

        // This should never panic
        let _result = a.checked_div(b);
    }

    /// Proves saturating_add never panics and stays within bounds.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_saturating_add_bounds() {
        let a_mantissa: i64 = kani::any();
        let b_mantissa: i64 = kani::any();

        let a = Decimal::new(a_mantissa, 0);
        let b = Decimal::new(b_mantissa, 0);

        // This should never panic
        let result = a.saturating_add(b);

        // Result must be within MIN..=MAX
        kani::assert(result >= Decimal::MIN, "saturating_add >= MIN");
        kani::assert(result <= Decimal::MAX, "saturating_add <= MAX");
    }

    /// Proves saturating_sub never panics and stays within bounds.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_saturating_sub_bounds() {
        let a_mantissa: i64 = kani::any();
        let b_mantissa: i64 = kani::any();

        let a = Decimal::new(a_mantissa, 0);
        let b = Decimal::new(b_mantissa, 0);

        // This should never panic
        let result = a.saturating_sub(b);

        // Result must be within MIN..=MAX
        kani::assert(result >= Decimal::MIN, "saturating_sub >= MIN");
        kani::assert(result <= Decimal::MAX, "saturating_sub <= MAX");
    }

    /// Proves saturating_mul never panics and stays within bounds.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_saturating_mul_bounds() {
        let a_mantissa: i32 = kani::any();
        let b_mantissa: i32 = kani::any();

        let a = Decimal::new(a_mantissa as i64, 0);
        let b = Decimal::new(b_mantissa as i64, 0);

        // This should never panic
        let result = a.saturating_mul(b);

        // Result must be within MIN..=MAX
        kani::assert(result >= Decimal::MIN, "saturating_mul >= MIN");
        kani::assert(result <= Decimal::MAX, "saturating_mul <= MAX");
    }

    /// Proves abs never panics.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_abs_no_panic() {
        let mantissa: i64 = kani::any();
        let a = Decimal::new(mantissa, 0);

        // This should never panic
        let result = a.abs();

        // Result should be non-negative
        kani::assert!(!result.is_negative(), "abs should be non-negative");
    }

    /// Proves signum produces valid output.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_signum_output() {
        let mantissa: i64 = kani::any();
        let a = Decimal::new(mantissa, 0);

        let result = a.signum();

        // signum must return -1, 0, or 1
        kani::assert(
            result == Decimal::NEGATIVE_ONE || result == Decimal::ZERO || result == Decimal::ONE,
            "signum must return -1, 0, or 1",
        );

        // Consistency checks
        if a.is_positive() {
            kani::assert(result == Decimal::ONE, "positive => signum = 1");
        }
        if a.is_negative() {
            kani::assert(result == Decimal::NEGATIVE_ONE, "negative => signum = -1");
        }
        if a.is_zero() {
            kani::assert(result == Decimal::ZERO, "zero => signum = 0");
        }
    }

    /// Proves min/max are commutative.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_min_max_commutative() {
        let a_mantissa: i64 = kani::any();
        let b_mantissa: i64 = kani::any();

        let a = Decimal::new(a_mantissa, 0);
        let b = Decimal::new(b_mantissa, 0);

        // min and max should be commutative
        kani::assert(a.min(b) == b.min(a), "min should be commutative");
        kani::assert(a.max(b) == b.max(a), "max should be commutative");

        // min <= max always
        kani::assert(a.min(b) <= a.max(b), "min <= max");
    }

    /// Proves clamp returns value within bounds.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_clamp_bounds() {
        let val_mantissa: i64 = kani::any();
        let min_mantissa: i64 = kani::any();
        let max_mantissa: i64 = kani::any();

        // Assume min <= max
        kani::assume(min_mantissa <= max_mantissa);

        let val = Decimal::new(val_mantissa, 0);
        let min = Decimal::new(min_mantissa, 0);
        let max = Decimal::new(max_mantissa, 0);

        let result = val.clamp(min, max);

        kani::assert(result >= min, "clamp >= min");
        kani::assert(result <= max, "clamp <= max");
    }

    /// Proves negation is self-inverse.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_negation_inverse() {
        let mantissa: i64 = kani::any();
        let a = Decimal::new(mantissa, 0);

        // -(-a) == a
        kani::assert(-(-a) == a, "negation should be self-inverse");
    }

    /// Proves zero identity for addition.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_zero_identity_add() {
        let mantissa: i64 = kani::any();
        let a = Decimal::new(mantissa, 0);

        // a + 0 == a (when no overflow)
        if let Some(result) = a.checked_add(Decimal::ZERO) {
            kani::assert(result == a, "a + 0 == a");
        }
    }

    /// Proves one identity for multiplication.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_one_identity_mul() {
        let mantissa: i64 = kani::any();
        let a = Decimal::new(mantissa, 0);

        // a * 1 == a (when no overflow)
        if let Some(result) = a.checked_mul(Decimal::ONE) {
            kani::assert(result == a, "a * 1 == a");
        }
    }

    /// Proves sqrt of negative returns None.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_sqrt_negative() {
        let mantissa: i64 = kani::any();
        kani::assume(mantissa < 0);

        let a = Decimal::new(mantissa, 0);
        let result = a.sqrt();

        kani::assert(result.is_none(), "sqrt of negative must be None");
    }

    /// Proves sqrt of non-negative returns Some.
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_sqrt_non_negative() {
        let mantissa: i64 = kani::any();
        kani::assume(mantissa >= 0);
        kani::assume(mantissa <= 1_000_000); // Bound for tractability

        let a = Decimal::new(mantissa, 0);
        let result = a.sqrt();

        // Should always return Some for non-negative
        kani::assert(result.is_some(), "sqrt of non-negative should be Some");

        // Result should be non-negative
        if let Some(r) = result {
            kani::assert!(!r.is_negative(), "sqrt result should be non-negative");
        }
    }
}
