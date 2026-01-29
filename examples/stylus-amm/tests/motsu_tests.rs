use alloy_primitives::{Address, U256};
use motsu::prelude::*;
use stylus_amm_example::AmmPool;

const ONE_ETH: u128 = 1_000_000_000_000_000_000;

#[motsu::test]
fn test_swap_output_basic(contract: Contract<AmmPool>) {
    let alice = Address::random();

    contract.sender(alice).set_fee(U256::from(30u64));

    let reserve_in = U256::from(100_000u64) * U256::from(ONE_ETH);
    let reserve_out = U256::from(100_000u64) * U256::from(ONE_ETH);
    let amount_in = U256::from(1_000u64) * U256::from(ONE_ETH);

    let amount_out = contract
        .sender(alice)
        .calculate_swap_output(reserve_in, reserve_out, amount_in)
        .expect("should calculate swap output");

    let min_expected = U256::from(985u64) * U256::from(ONE_ETH);
    let max_expected = U256::from(990u64) * U256::from(ONE_ETH);

    assert!(amount_out > min_expected);
    assert!(amount_out < max_expected);
}

#[motsu::test]
fn test_swap_output_zero_fee(contract: Contract<AmmPool>) {
    let alice = Address::random();

    contract.sender(alice).set_fee(U256::ZERO);

    let reserve_in = U256::from(100_000u64) * U256::from(ONE_ETH);
    let reserve_out = U256::from(100_000u64) * U256::from(ONE_ETH);
    let amount_in = U256::from(1_000u64) * U256::from(ONE_ETH);

    let amount_out = contract
        .sender(alice)
        .calculate_swap_output(reserve_in, reserve_out, amount_in)
        .expect("should calculate swap output");

    let expected_approx = U256::from(990u64) * U256::from(ONE_ETH);
    let diff = if amount_out > expected_approx {
        amount_out - expected_approx
    } else {
        expected_approx - amount_out
    };

    assert!(diff < U256::from(10u64) * U256::from(ONE_ETH));
}

#[motsu::test]
fn test_swap_output_zero_input(contract: Contract<AmmPool>) {
    let alice = Address::random();

    contract.sender(alice).set_fee(U256::from(30u64));

    let reserve_in = U256::from(100_000u64) * U256::from(ONE_ETH);
    let reserve_out = U256::from(100_000u64) * U256::from(ONE_ETH);

    let amount_out = contract
        .sender(alice)
        .calculate_swap_output(reserve_in, reserve_out, U256::ZERO)
        .expect("should handle zero input");

    assert_eq!(amount_out, U256::ZERO);
}

#[motsu::test]
fn test_swap_output_zero_reserve_fails(contract: Contract<AmmPool>) {
    let alice = Address::random();

    contract.sender(alice).set_fee(U256::from(30u64));

    let amount_in = U256::from(1_000u64) * U256::from(ONE_ETH);

    let result = contract
        .sender(alice)
        .calculate_swap_output(U256::ZERO, U256::from(ONE_ETH), amount_in);

    assert!(result.is_err());
}

#[motsu::test]
fn test_price_impact_increases_with_size(contract: Contract<AmmPool>) {
    let alice = Address::random();

    contract.sender(alice).set_fee(U256::from(30u64));

    let reserve_in = U256::from(1_000_000u64) * U256::from(ONE_ETH);
    let reserve_out = U256::from(1_000_000u64) * U256::from(ONE_ETH);

    let small_amount = U256::from(1_000u64) * U256::from(ONE_ETH);
    let large_amount = U256::from(100_000u64) * U256::from(ONE_ETH);

    let small_impact = contract
        .sender(alice)
        .calculate_price_impact(reserve_in, reserve_out, small_amount)
        .expect("should calculate small impact");

    let large_impact = contract
        .sender(alice)
        .calculate_price_impact(reserve_in, reserve_out, large_amount)
        .expect("should calculate large impact");

    assert!(large_impact > small_impact);
}

#[motsu::test]
fn test_spot_price(contract: Contract<AmmPool>) {
    let alice = Address::random();

    let reserve_a = U256::from(2_000u64) * U256::from(ONE_ETH);
    let reserve_b = U256::from(4_000u64) * U256::from(ONE_ETH);

    let price = contract
        .sender(alice)
        .calculate_spot_price(reserve_a, reserve_b)
        .expect("should calculate spot price");

    let expected = U256::from(2u64) * U256::from(ONE_ETH);
    assert_eq!(price, expected);
}

#[motsu::test]
fn test_swap_input_calculation(contract: Contract<AmmPool>) {
    let alice = Address::random();

    contract.sender(alice).set_fee(U256::from(30u64));

    let reserve_in = U256::from(100_000u64) * U256::from(ONE_ETH);
    let reserve_out = U256::from(100_000u64) * U256::from(ONE_ETH);
    let amount_out = U256::from(1_000u64) * U256::from(ONE_ETH);

    let amount_in = contract
        .sender(alice)
        .calculate_swap_input(reserve_in, reserve_out, amount_out)
        .expect("should calculate swap input");

    let min_expected = U256::from(1_010u64) * U256::from(ONE_ETH);
    let max_expected = U256::from(1_020u64) * U256::from(ONE_ETH);

    assert!(amount_in > min_expected);
    assert!(amount_in < max_expected);
}

#[motsu::test]
fn test_liquidity_mint_first_deposit(contract: Contract<AmmPool>) {
    let alice = Address::random();

    let amount_a = U256::from(1_000_000u64) * U256::from(ONE_ETH);
    let amount_b = U256::from(1_000_000u64) * U256::from(ONE_ETH);

    let shares = contract
        .sender(alice)
        .calculate_liquidity_mint(amount_a, amount_b, U256::ZERO, U256::ZERO, U256::ZERO)
        .expect("should calculate initial liquidity");

    assert!(shares > U256::ZERO);
}

#[motsu::test]
fn test_liquidity_mint_proportional(contract: Contract<AmmPool>) {
    let alice = Address::random();

    let reserve_a = U256::from(10_000u64) * U256::from(ONE_ETH);
    let reserve_b = U256::from(20_000u64) * U256::from(ONE_ETH);
    let total_supply = U256::from(1_000u64) * U256::from(ONE_ETH);

    let amount_a = U256::from(1_000u64) * U256::from(ONE_ETH);
    let amount_b = U256::from(2_000u64) * U256::from(ONE_ETH);

    let shares = contract
        .sender(alice)
        .calculate_liquidity_mint(amount_a, amount_b, reserve_a, reserve_b, total_supply)
        .expect("should calculate proportional mint");

    let expected = U256::from(100u64) * U256::from(ONE_ETH);
    let diff = if shares > expected {
        shares - expected
    } else {
        expected - shares
    };

    assert!(diff < U256::from(ONE_ETH));
}

#[motsu::test]
fn test_liquidity_burn(contract: Contract<AmmPool>) {
    let alice = Address::random();

    let shares = U256::from(100u64) * U256::from(ONE_ETH);
    let reserve_a = U256::from(10_000u64) * U256::from(ONE_ETH);
    let reserve_b = U256::from(20_000u64) * U256::from(ONE_ETH);
    let total_supply = U256::from(1_000u64) * U256::from(ONE_ETH);

    let (amount_a, amount_b) = contract
        .sender(alice)
        .calculate_liquidity_burn(shares, reserve_a, reserve_b, total_supply)
        .expect("should calculate burn amounts");

    let expected_a = U256::from(1_000u64) * U256::from(ONE_ETH);
    let expected_b = U256::from(2_000u64) * U256::from(ONE_ETH);

    assert_eq!(amount_a, expected_a);
    assert_eq!(amount_b, expected_b);
}

#[motsu::test]
fn test_constant_product_invariant(contract: Contract<AmmPool>) {
    let alice = Address::random();

    contract.sender(alice).set_fee(U256::ZERO);

    let reserve_in = U256::from(1_000_000u64) * U256::from(ONE_ETH);
    let reserve_out = U256::from(1_000_000u64) * U256::from(ONE_ETH);
    let amount_in = U256::from(10_000u64) * U256::from(ONE_ETH);

    let k_before = reserve_in * reserve_out;

    let amount_out = contract
        .sender(alice)
        .calculate_swap_output(reserve_in, reserve_out, amount_in)
        .expect("should calculate output");

    let new_reserve_in = reserve_in + amount_in;
    let new_reserve_out = reserve_out - amount_out;
    let k_after = new_reserve_in * new_reserve_out;

    assert!(k_after >= k_before);
}
