use alloy_primitives::{Address, U256};
use motsu::prelude::*;
use stylus_lending_example::LendingPool;

const ONE_ETH: u128 = 1_000_000_000_000_000_000;

#[motsu::test]
fn test_health_factor_healthy_position(contract: Contract<LendingPool>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    let collateral = U256::from(10_000u64) * U256::from(ONE_ETH);
    let debt = U256::from(5_000u64) * U256::from(ONE_ETH);

    let hf = contract
        .sender(alice)
        .calculate_health_factor(collateral, debt)
        .expect("should calculate health factor");

    let one_point_six = U256::from(16u64) * U256::from(ONE_ETH) / U256::from(10u64);
    assert_eq!(hf, one_point_six);
}

#[motsu::test]
fn test_health_factor_unhealthy_position(contract: Contract<LendingPool>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    let collateral = U256::from(1_000u64) * U256::from(ONE_ETH);
    let debt = U256::from(1_000u64) * U256::from(ONE_ETH);

    let hf = contract
        .sender(alice)
        .calculate_health_factor(collateral, debt)
        .expect("should calculate health factor");

    let zero_point_eight = U256::from(8u64) * U256::from(ONE_ETH) / U256::from(10u64);
    assert_eq!(hf, zero_point_eight);
}

#[motsu::test]
fn test_health_factor_zero_debt_returns_max(contract: Contract<LendingPool>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    let collateral = U256::from(10_000u64) * U256::from(ONE_ETH);
    let debt = U256::ZERO;

    let hf = contract
        .sender(alice)
        .calculate_health_factor(collateral, debt)
        .expect("should return max for zero debt");

    assert_eq!(hf, U256::MAX);
}

#[motsu::test]
fn test_is_liquidatable_healthy(contract: Contract<LendingPool>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    let collateral = U256::from(10_000u64) * U256::from(ONE_ETH);
    let debt = U256::from(5_000u64) * U256::from(ONE_ETH);

    let liquidatable = contract
        .sender(alice)
        .is_liquidatable(collateral, debt)
        .expect("should check liquidation status");

    assert!(!liquidatable);
}

#[motsu::test]
fn test_is_liquidatable_unhealthy(contract: Contract<LendingPool>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    let collateral = U256::from(1_000u64) * U256::from(ONE_ETH);
    let debt = U256::from(1_000u64) * U256::from(ONE_ETH);

    let liquidatable = contract
        .sender(alice)
        .is_liquidatable(collateral, debt)
        .expect("should check liquidation status");

    assert!(liquidatable);
}

#[motsu::test]
fn test_liquidation_price(contract: Contract<LendingPool>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    let collateral_amount = U256::from(10u64) * U256::from(ONE_ETH);
    let debt = U256::from(8_000u64) * U256::from(ONE_ETH);

    let liq_price = contract
        .sender(alice)
        .calculate_liquidation_price(collateral_amount, debt)
        .expect("should calculate liquidation price");

    let expected = U256::from(1_000u64) * U256::from(ONE_ETH);
    assert_eq!(liq_price, expected);
}

#[motsu::test]
fn test_max_borrow(contract: Contract<LendingPool>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    let collateral = U256::from(10_000u64) * U256::from(ONE_ETH);
    let target_hf = U256::from(15u64) * U256::from(ONE_ETH) / U256::from(10u64);

    let max_borrow = contract
        .sender(alice)
        .calculate_max_borrow(collateral, target_hf)
        .expect("should calculate max borrow");

    let expected_approx = U256::from(5333u64) * U256::from(ONE_ETH);
    let diff = if max_borrow > expected_approx {
        max_borrow - expected_approx
    } else {
        expected_approx - max_borrow
    };

    assert!(diff < U256::from(ONE_ETH));
}

#[motsu::test]
fn test_liquidation_amounts(contract: Contract<LendingPool>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_bonus(U256::from(500u64));

    let debt_to_cover = U256::from(1_000u64) * U256::from(ONE_ETH);
    let collateral_price = U256::from(2_000u64) * U256::from(ONE_ETH);

    let (returned_debt, collateral_received) = contract
        .sender(alice)
        .calculate_liquidation_amounts(debt_to_cover, collateral_price)
        .expect("should calculate liquidation amounts");

    assert_eq!(returned_debt, debt_to_cover);

    let expected_collateral = U256::from(525u64) * U256::from(ONE_ETH) / U256::from(1000u64);
    assert_eq!(collateral_received, expected_collateral);
}

#[motsu::test]
fn test_threshold_update(contract: Contract<LendingPool>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(7500u64));

    let collateral = U256::from(10_000u64) * U256::from(ONE_ETH);
    let debt = U256::from(5_000u64) * U256::from(ONE_ETH);

    let hf = contract
        .sender(alice)
        .calculate_health_factor(collateral, debt)
        .expect("should use new threshold");

    let expected = U256::from(15u64) * U256::from(ONE_ETH) / U256::from(10u64);
    assert_eq!(hf, expected);
}
