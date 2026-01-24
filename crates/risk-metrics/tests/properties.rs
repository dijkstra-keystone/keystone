//! Property-based tests for risk metrics.

use proptest::prelude::*;
use risk_metrics::{
    available_liquidity, collateral_ratio, health_factor, is_healthy, liquidation_price,
    loan_to_value, max_borrowable, utilization_rate, Decimal,
};

fn positive_amount() -> impl Strategy<Value = Decimal> {
    (1i64..=1_000_000, 0u32..=2).prop_map(|(m, s)| Decimal::new(m, s))
}

fn threshold() -> impl Strategy<Value = Decimal> {
    (50i64..=95).prop_map(|m| Decimal::new(m, 2))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn health_factor_healthy_position(
        collateral in positive_amount(),
        threshold in threshold()
    ) {
        // Debt is half of liquidation threshold
        let max_debt = collateral.checked_mul(threshold).unwrap();
        let debt = max_debt.checked_div(Decimal::from(2i64)).unwrap();

        if !debt.is_zero() {
            let hf = health_factor(collateral, debt, threshold).unwrap();
            prop_assert!(hf > Decimal::ONE, "hf {} should > 1", hf);
        }
    }

    #[test]
    fn health_factor_at_liquidation(collateral in positive_amount(), threshold in threshold()) {
        let debt = collateral.checked_mul(threshold).unwrap();

        if !debt.is_zero() {
            let hf = health_factor(collateral, debt, threshold).unwrap();
            let diff = (hf - Decimal::ONE).abs();
            prop_assert!(diff < Decimal::new(1, 10), "hf {} should ≈ 1", hf);
        }
    }

    #[test]
    fn is_healthy_consistent_with_health_factor(
        collateral in positive_amount(),
        threshold in threshold()
    ) {
        let debt = collateral.checked_div(Decimal::from(2i64)).unwrap();

        if !debt.is_zero() {
            let hf = health_factor(collateral, debt, threshold).unwrap();
            let healthy = is_healthy(collateral, debt, threshold, Decimal::ONE).unwrap();

            if hf >= Decimal::ONE {
                prop_assert!(healthy);
            }
        }
    }

    #[test]
    fn collateral_ratio_inverse_of_ltv(collateral in positive_amount(), debt in positive_amount()) {
        let ltv = loan_to_value(debt, collateral).unwrap();
        let cr = collateral_ratio(collateral, debt).unwrap();

        let product = ltv.checked_mul(cr).unwrap();
        let diff = (product - Decimal::ONE).abs();
        prop_assert!(diff < Decimal::new(1, 10), "ltv * cr should ≈ 1");
    }

    #[test]
    fn ltv_bounded_zero_to_one_for_healthy(
        collateral in positive_amount()
    ) {
        // Debt less than collateral
        let debt = collateral.checked_div(Decimal::from(2i64)).unwrap();
        let ltv = loan_to_value(debt, collateral).unwrap();
        prop_assert!(ltv >= Decimal::ZERO);
        prop_assert!(ltv <= Decimal::ONE);
    }

    #[test]
    fn utilization_rate_bounded(
        supply in positive_amount()
    ) {
        // Borrows less than supply
        let borrows = supply.checked_div(Decimal::from(2i64)).unwrap();
        let util = utilization_rate(borrows, supply).unwrap();
        prop_assert!(util >= Decimal::ZERO);
        prop_assert!(util <= Decimal::ONE);
    }

    #[test]
    fn available_liquidity_sum(supply in positive_amount()) {
        let borrows = supply.checked_div(Decimal::from(3i64)).unwrap();
        let liquidity = available_liquidity(supply, borrows).unwrap();
        let total = liquidity.checked_add(borrows).unwrap();
        prop_assert_eq!(total, supply);
    }

    #[test]
    fn max_borrowable_respects_ltv(
        collateral in positive_amount(),
        max_ltv in threshold()
    ) {
        let current_debt = Decimal::ZERO;
        let max_borrow = max_borrowable(collateral, max_ltv, current_debt).unwrap();
        let expected = collateral.checked_mul(max_ltv).unwrap();
        prop_assert_eq!(max_borrow, expected);
    }

    #[test]
    fn max_borrowable_decreases_with_debt(
        collateral in positive_amount(),
        max_ltv in threshold()
    ) {
        let max_total = collateral.checked_mul(max_ltv).unwrap();
        let debt1 = max_total.checked_div(Decimal::from(4i64)).unwrap();
        let debt2 = max_total.checked_div(Decimal::from(2i64)).unwrap();

        let available1 = max_borrowable(collateral, max_ltv, debt1).unwrap();
        let available2 = max_borrowable(collateral, max_ltv, debt2).unwrap();

        prop_assert!(available1 > available2);
    }

    #[test]
    fn liquidation_price_below_current_for_healthy(
        collateral_amount in positive_amount(),
        current_price in positive_amount(),
        threshold in threshold()
    ) {
        let collateral_value = collateral_amount.checked_mul(current_price).unwrap();
        // Debt is 50% of max
        let max_debt = collateral_value.checked_mul(threshold).unwrap();
        let debt = max_debt.checked_div(Decimal::from(2i64)).unwrap();

        if !debt.is_zero() && !collateral_amount.is_zero() {
            let liq_price = liquidation_price(collateral_amount, debt, threshold).unwrap();
            // Liquidation price should be below current price for healthy position
            prop_assert!(liq_price < current_price, "liq {} should < current {}", liq_price, current_price);
        }
    }
}
