//! Property-based tests for financial calculations.

use financial_calc::{
    basis_points_to_decimal, compound_interest, effective_annual_rate, future_value,
    percentage_change, percentage_of, present_value, simple_interest, Decimal,
};
use proptest::prelude::*;

fn positive_amount() -> impl Strategy<Value = Decimal> {
    (1i64..=1_000_000, 0u32..=4).prop_map(|(m, s)| Decimal::new(m, s))
}

fn rate() -> impl Strategy<Value = Decimal> {
    (1i64..=500, 2u32..=4).prop_map(|(m, s)| Decimal::new(m, s))
}

fn small_periods() -> impl Strategy<Value = u32> {
    1u32..=30
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn simple_interest_zero_rate(principal in positive_amount(), periods in 1i64..=100) {
        let periods_dec = Decimal::from(periods);
        let interest = simple_interest(principal, Decimal::ZERO, periods_dec).unwrap();
        prop_assert_eq!(interest, Decimal::ZERO);
    }

    #[test]
    fn simple_interest_zero_periods(principal in positive_amount(), r in rate()) {
        let interest = simple_interest(principal, r, Decimal::ZERO).unwrap();
        prop_assert_eq!(interest, Decimal::ZERO);
    }

    #[test]
    fn simple_interest_positive(principal in positive_amount(), r in rate(), periods in 1i64..=10) {
        let periods_dec = Decimal::from(periods);
        let interest = simple_interest(principal, r, periods_dec).unwrap();
        prop_assert!(interest > Decimal::ZERO);
    }

    #[test]
    fn compound_interest_zero_rate(principal in positive_amount(), periods in small_periods()) {
        let interest = compound_interest(principal, Decimal::ZERO, 12, periods).unwrap();
        prop_assert_eq!(interest, Decimal::ZERO);
    }

    #[test]
    fn compound_exceeds_simple(principal in positive_amount(), r in rate(), periods in 2u32..=10) {
        let simple = simple_interest(principal, r, Decimal::from(periods as i64)).unwrap();
        let compound = compound_interest(principal, r, 12, periods).unwrap();
        // Compound interest should exceed simple interest for periods > 1
        prop_assert!(compound >= simple, "compound {} should >= simple {}", compound, simple);
    }

    #[test]
    fn ear_exceeds_nominal(nominal in rate(), compounds in 2u32..=12) {
        let ear = effective_annual_rate(nominal, compounds).unwrap();
        // EAR should always be >= nominal rate when compounding > 1
        prop_assert!(ear >= nominal, "EAR {} should >= nominal {}", ear, nominal);
    }

    #[test]
    fn ear_increases_with_compounding(nominal in rate()) {
        let ear_monthly = effective_annual_rate(nominal, 12).unwrap();
        let ear_daily = effective_annual_rate(nominal, 365).unwrap();
        prop_assert!(ear_daily >= ear_monthly);
    }

    #[test]
    fn percentage_of_zero(value in positive_amount()) {
        let result = percentage_of(value, Decimal::ZERO).unwrap();
        prop_assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    fn percentage_of_hundred(value in positive_amount()) {
        let result = percentage_of(value, Decimal::ONE_HUNDRED).unwrap();
        prop_assert_eq!(result, value);
    }

    #[test]
    fn percentage_change_same_value(value in positive_amount()) {
        let change = percentage_change(value, value).unwrap();
        prop_assert_eq!(change, Decimal::ZERO);
    }

    #[test]
    fn percentage_change_double(value in positive_amount()) {
        let doubled = value.checked_mul(Decimal::from(2i64)).unwrap();
        let change = percentage_change(value, doubled).unwrap();
        prop_assert_eq!(change, Decimal::ONE_HUNDRED);
    }

    #[test]
    fn basis_points_100_is_one_percent(bps in 1i64..=10000) {
        let decimal = basis_points_to_decimal(Decimal::from(bps)).unwrap();
        let as_percentage = decimal.checked_mul(Decimal::ONE_HUNDRED).unwrap();
        let expected = Decimal::new(bps, 2);
        prop_assert_eq!(as_percentage, expected);
    }

    #[test]
    fn fv_pv_inverse(pv in positive_amount(), r in rate(), periods in small_periods()) {
        let fv = future_value(pv, r, periods).unwrap();
        let recovered = present_value(fv, r, periods).unwrap();
        let diff = (pv - recovered).abs();
        prop_assert!(diff < Decimal::new(1, 10), "PV {} != recovered {}", pv, recovered);
    }

    #[test]
    fn fv_increases_with_time(pv in positive_amount(), r in rate()) {
        let fv1 = future_value(pv, r, 1).unwrap();
        let fv5 = future_value(pv, r, 5).unwrap();
        prop_assert!(fv5 > fv1);
    }

    #[test]
    fn fv_increases_with_rate(pv in positive_amount(), periods in small_periods()) {
        let r1 = Decimal::new(5, 2);  // 5%
        let r2 = Decimal::new(10, 2); // 10%
        let fv1 = future_value(pv, r1, periods).unwrap();
        let fv2 = future_value(pv, r2, periods).unwrap();
        prop_assert!(fv2 > fv1);
    }

    #[test]
    fn pv_decreases_with_rate(fv in positive_amount(), periods in small_periods()) {
        let r1 = Decimal::new(5, 2);
        let r2 = Decimal::new(10, 2);
        let pv1 = present_value(fv, r1, periods).unwrap();
        let pv2 = present_value(fv, r2, periods).unwrap();
        prop_assert!(pv2 < pv1);
    }

    #[test]
    fn zero_rate_preserves_value(pv in positive_amount(), periods in small_periods()) {
        let fv = future_value(pv, Decimal::ZERO, periods).unwrap();
        prop_assert_eq!(fv, pv);
    }
}
