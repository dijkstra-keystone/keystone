use alloy_primitives::{Address, U256};
use motsu::prelude::*;
use stylus_oracle_example::OraclePricedLending;

const ONE_ETH: u128 = 1_000_000_000_000_000_000;
const ETH_PRICE: u64 = 200_000_000_000; // $2000 with 8 decimals
const USDC_PRICE: u64 = 100_000_000;    // $1 with 8 decimals

#[motsu::test]
fn test_health_factor_with_prices(contract: Contract<OraclePricedLending>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    // 10 ETH at $2000 = $20,000 collateral
    // 10,000 USDC at $1 = $10,000 debt
    // HF = (20,000 * 0.8) / 10,000 = 1.6

    let collateral_amount = U256::from(10u64) * U256::from(ONE_ETH);
    let collateral_price = U256::from(ETH_PRICE);
    let debt_amount = U256::from(10_000u64) * U256::from(ONE_ETH);
    let debt_price = U256::from(USDC_PRICE);

    let hf = contract
        .sender(alice)
        .calculate_health_factor_with_prices(
            collateral_amount,
            collateral_price,
            debt_amount,
            debt_price,
        )
        .expect("should calculate health factor");

    let expected = U256::from(16u64) * U256::from(ONE_ETH) / U256::from(10u64);
    let diff = if hf > expected { hf - expected } else { expected - hf };
    assert!(diff < U256::from(ONE_ETH / 100)); // 1% tolerance
}

#[motsu::test]
fn test_health_factor_zero_debt(contract: Contract<OraclePricedLending>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    let hf = contract
        .sender(alice)
        .calculate_health_factor_with_prices(
            U256::from(10u64) * U256::from(ONE_ETH),
            U256::from(ETH_PRICE),
            U256::ZERO,
            U256::from(USDC_PRICE),
        )
        .expect("should handle zero debt");

    assert_eq!(hf, U256::MAX);
}

#[motsu::test]
fn test_liquidation_price(contract: Contract<OraclePricedLending>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    // 10 ETH collateral, $10,000 USDC debt
    // Liq price = 10,000 / (10 * 0.8) = $1,250

    let collateral_amount = U256::from(10u64) * U256::from(ONE_ETH);
    let debt_amount = U256::from(10_000u64) * U256::from(ONE_ETH);
    let debt_price = U256::from(USDC_PRICE);

    let liq_price = contract
        .sender(alice)
        .calculate_liquidation_price_with_oracle(collateral_amount, debt_amount, debt_price)
        .expect("should calculate liquidation price");

    // Expected: $1,250 in 8 decimals = 125_000_000_000
    let expected = U256::from(125_000_000_000u128);
    let diff = if liq_price > expected {
        liq_price - expected
    } else {
        expected - liq_price
    };
    assert!(diff < U256::from(1_000_000u64)); // Small tolerance
}

#[motsu::test]
fn test_max_borrow_with_prices(contract: Contract<OraclePricedLending>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    // 10 ETH at $2000, borrow USDC at $1, target HF 1.5
    // Max debt value = (20,000 * 0.8) / 1.5 = 10,666.67
    // Max borrow = 10,666.67 USDC

    let collateral_amount = U256::from(10u64) * U256::from(ONE_ETH);
    let collateral_price = U256::from(ETH_PRICE);
    let debt_price = U256::from(USDC_PRICE);
    let target_hf = U256::from(15u64) * U256::from(ONE_ETH) / U256::from(10u64);

    let max_borrow = contract
        .sender(alice)
        .calculate_max_borrow_with_prices(
            collateral_amount,
            collateral_price,
            debt_price,
            target_hf,
        )
        .expect("should calculate max borrow");

    let expected = U256::from(10_666u64) * U256::from(ONE_ETH);
    let diff = if max_borrow > expected {
        max_borrow - expected
    } else {
        expected - max_borrow
    };
    assert!(diff < U256::from(100u64) * U256::from(ONE_ETH)); // 100 token tolerance
}

#[motsu::test]
fn test_is_liquidatable(contract: Contract<OraclePricedLending>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_threshold(U256::from(8000u64));

    // Healthy position: HF = 1.6
    let healthy = contract
        .sender(alice)
        .is_liquidatable_with_prices(
            U256::from(10u64) * U256::from(ONE_ETH),
            U256::from(ETH_PRICE),
            U256::from(10_000u64) * U256::from(ONE_ETH),
            U256::from(USDC_PRICE),
        )
        .expect("should check liquidation");
    assert!(!healthy);

    // Unhealthy position: 5 ETH collateral, same debt
    // HF = (10,000 * 0.8) / 10,000 = 0.8
    let unhealthy = contract
        .sender(alice)
        .is_liquidatable_with_prices(
            U256::from(5u64) * U256::from(ONE_ETH),
            U256::from(ETH_PRICE),
            U256::from(10_000u64) * U256::from(ONE_ETH),
            U256::from(USDC_PRICE),
        )
        .expect("should check liquidation");
    assert!(unhealthy);
}

#[motsu::test]
fn test_liquidation_with_bonus(contract: Contract<OraclePricedLending>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_liquidation_bonus(U256::from(500u64)); // 5% bonus

    // Cover $1000 USDC debt, ETH at $2000
    // Base collateral = 1000 / 2000 = 0.5 ETH
    // With 5% bonus = 0.525 ETH

    let debt_to_cover = U256::from(1_000u64) * U256::from(ONE_ETH);
    let collateral_price = U256::from(ETH_PRICE);
    let debt_price = U256::from(USDC_PRICE);

    let (returned_debt, collateral_received) = contract
        .sender(alice)
        .calculate_liquidation_with_prices(debt_to_cover, collateral_price, debt_price)
        .expect("should calculate liquidation");

    assert_eq!(returned_debt, debt_to_cover);

    let expected_collateral = U256::from(525u64) * U256::from(ONE_ETH) / U256::from(1000u64);
    let diff = if collateral_received > expected_collateral {
        collateral_received - expected_collateral
    } else {
        expected_collateral - collateral_received
    };
    assert!(diff < U256::from(ONE_ETH / 100)); // 1% tolerance
}

#[motsu::test]
fn test_price_deviation(contract: Contract<OraclePricedLending>) {
    let alice = Address::random();

    // 5% deviation: price = 2100, median = 2000
    let price = U256::from(210_000_000_000u128); // $2100
    let median = U256::from(200_000_000_000u128); // $2000

    let deviation_bps = contract
        .sender(alice)
        .calculate_price_deviation(price, median)
        .expect("should calculate deviation");

    // 5% = 500 bps
    assert_eq!(deviation_bps, U256::from(500u64));
}

#[motsu::test]
fn test_trusted_signer(contract: Contract<OraclePricedLending>) {
    let alice = Address::random();
    let signer = Address::random();

    assert!(!contract.sender(alice).is_trusted_signer(signer));

    contract.sender(alice).set_trusted_signer(signer, true);

    assert!(contract.sender(alice).is_trusted_signer(signer));
}
