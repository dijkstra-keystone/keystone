use alloy_primitives::{Address, U256};
use motsu::prelude::*;
use stylus_options_example::OptionsEngine;

const ONE_ETH: u128 = 1_000_000_000_000_000_000;

fn scaled(val: u64) -> U256 {
    U256::from(val) * U256::from(ONE_ETH)
}

fn scaled_fraction(num: u64, den: u64) -> U256 {
    U256::from(num) * U256::from(ONE_ETH) / U256::from(den)
}

#[motsu::test]
fn test_atm_call_price(contract: Contract<OptionsEngine>) {
    let alice = Address::random();

    // 5% risk-free rate
    contract.sender(alice).set_risk_free_rate(U256::from(500u64));

    let spot = scaled(100);
    let strike = scaled(100);
    let vol = scaled_fraction(2, 10); // 0.2 = 20%
    let time = scaled_fraction(1, 4); // 0.25 = 3 months

    let price = contract
        .sender(alice)
        .price_call(spot, strike, vol, time)
        .expect("should price call");

    // ATM call ≈ $3.80-$4.20
    let min = scaled(3);
    let max = scaled(5);
    assert!(price > min, "call price too low");
    assert!(price < max, "call price too high");
}

#[motsu::test]
fn test_atm_put_price(contract: Contract<OptionsEngine>) {
    let alice = Address::random();

    contract.sender(alice).set_risk_free_rate(U256::from(500u64));

    let spot = scaled(100);
    let strike = scaled(100);
    let vol = scaled_fraction(2, 10);
    let time = scaled_fraction(1, 4);

    let price = contract
        .sender(alice)
        .price_put(spot, strike, vol, time)
        .expect("should price put");

    // ATM put ≈ $2.50-$4.00
    let min = scaled(2);
    let max = scaled(5);
    assert!(price > min, "put price too low");
    assert!(price < max, "put price too high");
}

#[motsu::test]
fn test_put_call_parity(contract: Contract<OptionsEngine>) {
    let alice = Address::random();

    contract.sender(alice).set_risk_free_rate(U256::from(500u64));

    let spot = scaled(100);
    let strike = scaled(100);
    let vol = scaled_fraction(2, 10);
    let time = scaled_fraction(1, 4);

    let parity_diff = contract
        .sender(alice)
        .put_call_parity_check(spot, strike, vol, time)
        .expect("should check parity");

    // Parity difference should be near zero
    let tolerance = U256::from(ONE_ETH / 100); // 0.01
    assert!(parity_diff < tolerance, "put-call parity violated");
}

#[motsu::test]
fn test_call_greeks(contract: Contract<OptionsEngine>) {
    let alice = Address::random();

    contract.sender(alice).set_risk_free_rate(U256::from(500u64));

    let spot = scaled(100);
    let strike = scaled(100);
    let vol = scaled_fraction(2, 10);
    let time = scaled_fraction(1, 4);

    let (delta, gamma, theta, vega, rho) = contract
        .sender(alice)
        .call_option_greeks(spot, strike, vol, time)
        .expect("should calculate greeks");

    // Delta: 0 < delta < 1 for call
    assert!(delta > U256::ZERO, "delta should be positive");
    assert!(delta < U256::from(ONE_ETH), "delta should be < 1");

    // Gamma: should be positive
    assert!(gamma > U256::ZERO, "gamma should be positive");

    // Theta: should be positive (we take abs)
    assert!(theta > U256::ZERO, "theta should be positive");

    // Vega: should be positive
    assert!(vega > U256::ZERO, "vega should be positive");
}

#[motsu::test]
fn test_otm_put_cheap(contract: Contract<OptionsEngine>) {
    let alice = Address::random();

    contract.sender(alice).set_risk_free_rate(U256::from(500u64));

    let spot = scaled(100);
    let strike = scaled(80); // 20% OTM
    let vol = scaled_fraction(2, 10);
    let time = scaled_fraction(1, 4);

    let price = contract
        .sender(alice)
        .price_put(spot, strike, vol, time)
        .expect("should price OTM put");

    // Deep OTM put should be very cheap
    assert!(price < U256::from(ONE_ETH), "OTM put should be < 1");
}

#[motsu::test]
fn test_itm_call_exceeds_intrinsic(contract: Contract<OptionsEngine>) {
    let alice = Address::random();

    contract.sender(alice).set_risk_free_rate(U256::from(500u64));

    let spot = scaled(120);
    let strike = scaled(100);
    let vol = scaled_fraction(2, 10);
    let time = scaled_fraction(1, 4);

    let price = contract
        .sender(alice)
        .price_call(spot, strike, vol, time)
        .expect("should price ITM call");

    // ITM call must exceed intrinsic value (S - K = 20)
    let intrinsic = scaled(20);
    assert!(price >= intrinsic, "call should exceed intrinsic value");
}

#[motsu::test]
fn test_higher_vol_means_higher_price(contract: Contract<OptionsEngine>) {
    let alice = Address::random();

    contract.sender(alice).set_risk_free_rate(U256::from(500u64));

    let spot = scaled(100);
    let strike = scaled(100);
    let time = scaled_fraction(1, 4);

    let low_vol = scaled_fraction(1, 10); // 10%
    let high_vol = scaled_fraction(4, 10); // 40%

    let low_price = contract
        .sender(alice)
        .price_call(spot, strike, low_vol, time)
        .expect("low vol price");

    let high_price = contract
        .sender(alice)
        .price_call(spot, strike, high_vol, time)
        .expect("high vol price");

    assert!(high_price > low_price, "higher vol should mean higher price");
}
