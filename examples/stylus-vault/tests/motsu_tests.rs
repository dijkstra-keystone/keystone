use alloy_primitives::{Address, U256};
use motsu::prelude::*;
use stylus_vault_example::Vault;

const ONE_ETH: u128 = 1_000_000_000_000_000_000;

#[motsu::test]
fn test_shares_for_deposit_empty_vault(contract: Contract<Vault>) {
    let alice = Address::random();

    let assets = U256::from(1_000u64) * U256::from(ONE_ETH);

    let shares = contract
        .sender(alice)
        .calculate_shares_for_deposit(assets, U256::ZERO, U256::ZERO)
        .expect("should handle empty vault");

    assert_eq!(shares, assets);
}

#[motsu::test]
fn test_shares_for_deposit_existing_vault(contract: Contract<Vault>) {
    let alice = Address::random();

    let assets = U256::from(100u64) * U256::from(ONE_ETH);
    let total_assets = U256::from(1_000u64) * U256::from(ONE_ETH);
    let total_supply = U256::from(500u64) * U256::from(ONE_ETH);

    let shares = contract
        .sender(alice)
        .calculate_shares_for_deposit(assets, total_assets, total_supply)
        .expect("should calculate shares");

    let expected = U256::from(50u64) * U256::from(ONE_ETH);
    assert_eq!(shares, expected);
}

#[motsu::test]
fn test_assets_for_redeem(contract: Contract<Vault>) {
    let alice = Address::random();

    let shares = U256::from(50u64) * U256::from(ONE_ETH);
    let total_assets = U256::from(1_000u64) * U256::from(ONE_ETH);
    let total_supply = U256::from(500u64) * U256::from(ONE_ETH);

    let assets = contract
        .sender(alice)
        .calculate_assets_for_redeem(shares, total_assets, total_supply)
        .expect("should calculate assets");

    let expected = U256::from(100u64) * U256::from(ONE_ETH);
    assert_eq!(assets, expected);
}

#[motsu::test]
fn test_share_price(contract: Contract<Vault>) {
    let alice = Address::random();

    let total_assets = U256::from(2_000u64) * U256::from(ONE_ETH);
    let total_supply = U256::from(1_000u64) * U256::from(ONE_ETH);

    let price = contract
        .sender(alice)
        .calculate_share_price(total_assets, total_supply)
        .expect("should calculate price");

    let expected = U256::from(2u64) * U256::from(ONE_ETH);
    assert_eq!(price, expected);
}

#[motsu::test]
fn test_share_price_empty_vault(contract: Contract<Vault>) {
    let alice = Address::random();

    let price = contract
        .sender(alice)
        .calculate_share_price(U256::ZERO, U256::ZERO)
        .expect("should return 1e18 for empty vault");

    let expected = U256::from(ONE_ETH);
    assert_eq!(price, expected);
}

#[motsu::test]
fn test_compound_yield(contract: Contract<Vault>) {
    let alice = Address::random();

    let principal = U256::from(1_000u64) * U256::from(ONE_ETH);
    let rate_bps = U256::from(100u64);
    let periods = U256::from(3u64);

    let final_value = contract
        .sender(alice)
        .calculate_compound_yield(principal, rate_bps, periods)
        .expect("should calculate compound yield");

    let min_expected = U256::from(1030u64) * U256::from(ONE_ETH);
    let max_expected = U256::from(1031u64) * U256::from(ONE_ETH);

    assert!(final_value > min_expected);
    assert!(final_value < max_expected);
}

#[motsu::test]
fn test_apy_from_apr(contract: Contract<Vault>) {
    let alice = Address::random();

    let apr_bps = U256::from(1000u64);
    let compounds = U256::from(12u64);

    let apy_bps = contract
        .sender(alice)
        .calculate_apy_from_apr(apr_bps, compounds)
        .expect("should calculate APY");

    let min_expected = U256::from(1040u64) * U256::from(ONE_ETH) / U256::from(100u64);
    let max_expected = U256::from(1050u64) * U256::from(ONE_ETH) / U256::from(100u64);

    assert!(apy_bps > min_expected);
    assert!(apy_bps < max_expected);
}

#[motsu::test]
fn test_performance_fee(contract: Contract<Vault>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_performance_fee(U256::from(2000u64));

    let gains = U256::from(1_000u64) * U256::from(ONE_ETH);

    let fee = contract
        .sender(alice)
        .calculate_performance_fee(gains)
        .expect("should calculate performance fee");

    let expected = U256::from(200u64) * U256::from(ONE_ETH);
    assert_eq!(fee, expected);
}

#[motsu::test]
fn test_performance_fee_zero_gains(contract: Contract<Vault>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_performance_fee(U256::from(2000u64));

    let fee = contract
        .sender(alice)
        .calculate_performance_fee(U256::ZERO)
        .expect("should handle zero gains");

    assert_eq!(fee, U256::ZERO);
}

#[motsu::test]
fn test_management_fee(contract: Contract<Vault>) {
    let alice = Address::random();

    contract
        .sender(alice)
        .set_management_fee(U256::from(200u64));

    let total_assets = U256::from(1_000_000u64) * U256::from(ONE_ETH);
    let thirty_days = U256::from(30u64 * 24 * 60 * 60);

    let fee = contract
        .sender(alice)
        .calculate_management_fee(total_assets, thirty_days)
        .expect("should calculate management fee");

    let min_expected = U256::from(1_600u64) * U256::from(ONE_ETH);
    let max_expected = U256::from(1_700u64) * U256::from(ONE_ETH);

    assert!(fee > min_expected);
    assert!(fee < max_expected);
}

#[motsu::test]
fn test_net_asset_value(contract: Contract<Vault>) {
    let alice = Address::random();

    let balance = U256::from(500_000u64) * U256::from(ONE_ETH);
    let strategy = U256::from(450_000u64) * U256::from(ONE_ETH);
    let rewards = U256::from(50_000u64) * U256::from(ONE_ETH);
    let supply = U256::from(1_000_000u64) * U256::from(ONE_ETH);

    let nav = contract
        .sender(alice)
        .calculate_net_asset_value(balance, strategy, rewards, supply)
        .expect("should calculate NAV");

    let expected = U256::from(ONE_ETH);
    assert_eq!(nav, expected);
}

#[motsu::test]
fn test_deposit_redeem_symmetry(contract: Contract<Vault>) {
    let alice = Address::random();

    let assets = U256::from(100u64) * U256::from(ONE_ETH);
    let total_assets = U256::from(1_000u64) * U256::from(ONE_ETH);
    let total_supply = U256::from(1_000u64) * U256::from(ONE_ETH);

    let shares = contract
        .sender(alice)
        .calculate_shares_for_deposit(assets, total_assets, total_supply)
        .expect("should calculate shares");

    let new_total_assets = total_assets + assets;
    let new_total_supply = total_supply + shares;

    let redeemed = contract
        .sender(alice)
        .calculate_assets_for_redeem(shares, new_total_assets, new_total_supply)
        .expect("should calculate redeemed assets");

    let diff = if redeemed > assets {
        redeemed - assets
    } else {
        assets - redeemed
    };

    assert!(diff < U256::from(ONE_ETH));
}
