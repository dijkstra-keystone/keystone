//! Property-based testing support.

use crate::Decimal;
use proptest::prelude::*;

impl Arbitrary for Decimal {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (any::<i64>(), 0u32..=18)
            .prop_map(|(mantissa, scale)| Decimal::new(mantissa, scale))
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RoundingMode;

    fn small_decimal() -> impl Strategy<Value = Decimal> {
        (-1_000_000i64..=1_000_000, 0u32..=6).prop_map(|(m, s)| Decimal::new(m, s))
    }

    fn non_zero_decimal() -> impl Strategy<Value = Decimal> {
        any::<i64>()
            .prop_filter("non-zero", |&m| m != 0)
            .prop_flat_map(|m| (Just(m), 0u32..=18))
            .prop_map(|(m, s)| Decimal::new(m, s))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn addition_is_commutative(a in small_decimal(), b in small_decimal()) {
            if let (Some(ab), Some(ba)) = (a.checked_add(b), b.checked_add(a)) {
                prop_assert_eq!(ab, ba);
            }
        }

        #[test]
        fn addition_is_associative(
            a in small_decimal(),
            b in small_decimal(),
            c in small_decimal()
        ) {
            if let (Some(ab), Some(bc)) = (a.checked_add(b), b.checked_add(c)) {
                if let (Some(ab_c), Some(a_bc)) = (ab.checked_add(c), a.checked_add(bc)) {
                    prop_assert_eq!(ab_c, a_bc);
                }
            }
        }

        #[test]
        fn multiplication_is_commutative(a in small_decimal(), b in small_decimal()) {
            if let (Some(ab), Some(ba)) = (a.checked_mul(b), b.checked_mul(a)) {
                prop_assert_eq!(ab, ba);
            }
        }

        #[test]
        fn multiplication_identity(a in small_decimal()) {
            prop_assert_eq!(a.checked_mul(Decimal::ONE), Some(a));
        }

        #[test]
        fn addition_identity(a in small_decimal()) {
            prop_assert_eq!(a.checked_add(Decimal::ZERO), Some(a));
        }

        #[test]
        fn subtraction_identity(a in small_decimal()) {
            prop_assert_eq!(a.checked_sub(Decimal::ZERO), Some(a));
        }

        #[test]
        fn multiplication_by_zero(a in small_decimal()) {
            prop_assert_eq!(a.checked_mul(Decimal::ZERO), Some(Decimal::ZERO));
        }

        #[test]
        fn negation_involution(a in small_decimal()) {
            prop_assert_eq!(-(-a), a);
        }

        #[test]
        fn additive_inverse(a in small_decimal()) {
            prop_assert_eq!(a.checked_add(-a), Some(Decimal::ZERO));
        }

        #[test]
        fn division_by_self(a in non_zero_decimal()) {
            if let Some(result) = a.checked_div(a) {
                let diff = (result - Decimal::ONE).abs();
                prop_assert!(diff < Decimal::new(1, 20), "a/a should equal 1, got {}", result);
            }
        }

        #[test]
        fn abs_is_non_negative(a in small_decimal()) {
            prop_assert!(!a.abs().is_negative() || a.abs().is_zero());
        }

        #[test]
        fn abs_of_negation(a in small_decimal()) {
            prop_assert_eq!(a.abs(), (-a).abs());
        }

        #[test]
        fn ordering_consistency(a in small_decimal(), b in small_decimal()) {
            let cmp = a.cmp(&b);
            let reverse = b.cmp(&a);
            prop_assert_eq!(cmp, reverse.reverse());
        }

        #[test]
        fn min_max_relationship(a in small_decimal(), b in small_decimal()) {
            let min = a.min(b);
            let max = a.max(b);
            prop_assert!(min <= max);
            prop_assert!(min == a || min == b);
            prop_assert!(max == a || max == b);
        }

        #[test]
        fn clamp_bounds(
            a in small_decimal(),
            min in small_decimal(),
            max in small_decimal()
        ) {
            if min <= max {
                let clamped = a.clamp(min, max);
                prop_assert!(clamped >= min);
                prop_assert!(clamped <= max);
            }
        }

        #[test]
        fn round_preserves_value_within_precision(a in small_decimal()) {
            let rounded = a.round_dp(18);
            let diff = (rounded - a).abs();
            prop_assert!(diff < Decimal::new(1, 18));
        }

        #[test]
        fn floor_is_lte_original(a in small_decimal()) {
            prop_assert!(a.floor() <= a);
        }

        #[test]
        fn ceil_is_gte_original(a in small_decimal()) {
            prop_assert!(a.ceil() >= a);
        }

        #[test]
        fn trunc_toward_zero(a in small_decimal()) {
            let t = a.trunc(0);
            if a.is_positive() {
                prop_assert!(t <= a);
            } else if a.is_negative() {
                prop_assert!(t >= a);
            }
        }

        #[test]
        fn saturating_add_no_panic(a in any::<Decimal>(), b in any::<Decimal>()) {
            let _ = a.saturating_add(b);
        }

        #[test]
        fn saturating_sub_no_panic(a in any::<Decimal>(), b in any::<Decimal>()) {
            let _ = a.saturating_sub(b);
        }

        #[test]
        fn saturating_mul_no_panic(a in any::<Decimal>(), b in any::<Decimal>()) {
            let _ = a.saturating_mul(b);
        }

        #[test]
        fn distributive_property(
            a in small_decimal(),
            b in small_decimal(),
            c in small_decimal()
        ) {
            if let Some(bc) = b.checked_add(c) {
                if let (Some(a_bc), Some(ab), Some(ac)) = (
                    a.checked_mul(bc),
                    a.checked_mul(b),
                    a.checked_mul(c),
                ) {
                    if let Some(ab_ac) = ab.checked_add(ac) {
                        let diff = (a_bc - ab_ac).abs();
                        prop_assert!(
                            diff < Decimal::new(1, 10),
                            "distributive: {} vs {}, diff = {}",
                            a_bc, ab_ac, diff
                        );
                    }
                }
            }
        }

        #[test]
        fn rounding_half_up_basic(mantissa in -999i64..=999, scale in 0u32..=3) {
            let a = Decimal::new(mantissa, scale);
            let rounded = a.round(0, RoundingMode::HalfUp);
            let diff = (a - rounded).abs();
            prop_assert!(diff <= Decimal::new(5, 1));
        }
    }
}
