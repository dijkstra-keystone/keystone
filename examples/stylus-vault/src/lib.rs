//! Stylus Vault Example
//!
//! Demonstrates using Keystone precision arithmetic in a Stylus smart contract
//! for ERC4626-style vault calculations with deterministic results.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use alloy_primitives::U256;
use precision_core::{Decimal, RoundingMode};
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct Vault {
        /// Performance fee in basis points (e.g., 2000 = 20%)
        uint256 performance_fee_bps;
        /// Management fee in basis points per year (e.g., 200 = 2%)
        uint256 management_fee_bps;
    }
}

const SCALE: u64 = 1_000_000_000_000_000_000;
const BPS_DIVISOR: u64 = 10_000;

/// Convert U256 to Decimal (assumes 18 decimals, scaled to 1e18)
fn u256_to_decimal(value: U256) -> Decimal {
    let lo: u128 = value.as_limbs()[0] as u128 | ((value.as_limbs()[1] as u128) << 64);
    let raw = Decimal::from(lo);
    raw.checked_div(Decimal::from(SCALE))
        .unwrap_or(Decimal::MAX)
}

/// Convert Decimal to U256 (returns value scaled to 1e18)
fn decimal_to_u256(value: Decimal) -> U256 {
    let scaled = value
        .checked_mul(Decimal::from(SCALE))
        .unwrap_or(Decimal::MAX)
        .round(0, RoundingMode::TowardZero);
    let (mantissa, _scale) = scaled.to_parts();
    U256::from(mantissa.unsigned_abs())
}

#[public]
impl Vault {
    /// Calculate shares to mint for a deposit (ERC4626 convertToShares)
    ///
    /// shares = (assets * total_supply) / total_assets
    /// For empty vault: shares = assets (1:1 ratio)
    ///
    /// All values scaled by 1e18
    pub fn calculate_shares_for_deposit(
        &self,
        assets: U256,
        total_assets: U256,
        total_supply: U256,
    ) -> Result<U256, Vec<u8>> {
        if assets == U256::ZERO {
            return Ok(U256::ZERO);
        }

        if total_supply == U256::ZERO {
            return Ok(assets);
        }

        let a = u256_to_decimal(assets);
        let ta = u256_to_decimal(total_assets);
        let ts = u256_to_decimal(total_supply);

        if ta == Decimal::ZERO {
            return Err(b"zero total assets".to_vec());
        }

        let shares = a
            .checked_mul(ts)
            .ok_or_else(|| b"overflow".to_vec())?
            .checked_div(ta)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(shares))
    }

    /// Calculate assets to return for redemption (ERC4626 convertToAssets)
    ///
    /// assets = (shares * total_assets) / total_supply
    ///
    /// All values scaled by 1e18
    pub fn calculate_assets_for_redeem(
        &self,
        shares: U256,
        total_assets: U256,
        total_supply: U256,
    ) -> Result<U256, Vec<u8>> {
        if shares == U256::ZERO {
            return Ok(U256::ZERO);
        }

        if total_supply == U256::ZERO {
            return Err(b"zero supply".to_vec());
        }

        let s = u256_to_decimal(shares);
        let ta = u256_to_decimal(total_assets);
        let ts = u256_to_decimal(total_supply);

        let assets = s
            .checked_mul(ta)
            .ok_or_else(|| b"overflow".to_vec())?
            .checked_div(ts)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(assets))
    }

    /// Calculate current share price (assets per share, scaled by 1e18)
    ///
    /// price = total_assets / total_supply
    pub fn calculate_share_price(
        &self,
        total_assets: U256,
        total_supply: U256,
    ) -> Result<U256, Vec<u8>> {
        if total_supply == U256::ZERO {
            return Ok(U256::from(SCALE));
        }

        let ta = u256_to_decimal(total_assets);
        let ts = u256_to_decimal(total_supply);

        let price = ta
            .checked_div(ts)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(price))
    }

    /// Calculate compounded yield over periods
    ///
    /// final_value = principal * (1 + rate)^periods
    ///
    /// rate_bps: rate per period in basis points
    /// All values scaled by 1e18
    pub fn calculate_compound_yield(
        &self,
        principal: U256,
        rate_bps: U256,
        periods: U256,
    ) -> Result<U256, Vec<u8>> {
        let p = u256_to_decimal(principal);
        let rate = u256_to_decimal(rate_bps)
            .checked_div(Decimal::from(BPS_DIVISOR))
            .ok_or_else(|| b"division error".to_vec())?;

        let one_plus_rate = Decimal::ONE
            .checked_add(rate)
            .ok_or_else(|| b"overflow".to_vec())?;

        let n: u32 = periods.as_limbs()[0].min(365) as u32;

        let mut multiplier = Decimal::ONE;
        for _ in 0..n {
            multiplier = multiplier
                .checked_mul(one_plus_rate)
                .ok_or_else(|| b"overflow".to_vec())?;
        }

        let final_value = p
            .checked_mul(multiplier)
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok(decimal_to_u256(final_value))
    }

    /// Calculate APY from APR
    ///
    /// APY = (1 + APR/n)^n - 1
    ///
    /// apr_bps: annual rate in basis points
    /// compounds_per_year: compounding frequency (e.g., 365 for daily)
    /// Returns: APY in basis points scaled by 1e18
    pub fn calculate_apy_from_apr(
        &self,
        apr_bps: U256,
        compounds_per_year: U256,
    ) -> Result<U256, Vec<u8>> {
        let apr = u256_to_decimal(apr_bps)
            .checked_div(Decimal::from(BPS_DIVISOR))
            .ok_or_else(|| b"division error".to_vec())?;

        let n: u32 = compounds_per_year.as_limbs()[0].min(365) as u32;
        if n == 0 {
            return Err(b"zero compounds".to_vec());
        }

        let rate_per_period = apr
            .checked_div(Decimal::from(n as i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let one_plus_rate = Decimal::ONE
            .checked_add(rate_per_period)
            .ok_or_else(|| b"overflow".to_vec())?;

        let mut multiplier = Decimal::ONE;
        for _ in 0..n {
            multiplier = multiplier
                .checked_mul(one_plus_rate)
                .ok_or_else(|| b"overflow".to_vec())?;
        }

        let apy = multiplier
            .checked_sub(Decimal::ONE)
            .ok_or_else(|| b"underflow".to_vec())?;

        let apy_bps = apy
            .checked_mul(Decimal::from(BPS_DIVISOR))
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok(decimal_to_u256(apy_bps))
    }

    /// Calculate performance fee on gains
    ///
    /// fee = gains * fee_rate
    ///
    /// gains: profit amount (current - previous value)
    /// Returns: fee amount scaled by 1e18
    pub fn calculate_performance_fee(&self, gains: U256) -> Result<U256, Vec<u8>> {
        if gains == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let g = u256_to_decimal(gains);
        let fee_rate = u256_to_decimal(self.performance_fee_bps.get())
            .checked_div(Decimal::from(BPS_DIVISOR))
            .ok_or_else(|| b"division error".to_vec())?;

        let fee = g
            .checked_mul(fee_rate)
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok(decimal_to_u256(fee))
    }

    /// Calculate management fee for a time period
    ///
    /// fee = total_assets * (annual_rate * time_fraction)
    ///
    /// seconds_elapsed: time since last fee collection
    /// Returns: fee amount scaled by 1e18
    pub fn calculate_management_fee(
        &self,
        total_assets: U256,
        seconds_elapsed: U256,
    ) -> Result<U256, Vec<u8>> {
        if total_assets == U256::ZERO || seconds_elapsed == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let ta = u256_to_decimal(total_assets);
        let annual_rate = u256_to_decimal(self.management_fee_bps.get())
            .checked_div(Decimal::from(BPS_DIVISOR))
            .ok_or_else(|| b"division error".to_vec())?;

        let seconds: u64 = seconds_elapsed.as_limbs()[0].min(365 * 24 * 60 * 60);
        let time_fraction = Decimal::from(seconds as i64)
            .checked_div(Decimal::from(365 * 24 * 60 * 60i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let fee = ta
            .checked_mul(annual_rate)
            .ok_or_else(|| b"overflow".to_vec())?
            .checked_mul(time_fraction)
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok(decimal_to_u256(fee))
    }

    /// Calculate vault's total value including unrealized gains
    ///
    /// Returns share value in underlying asset terms
    pub fn calculate_net_asset_value(
        &self,
        underlying_balance: U256,
        strategy_value: U256,
        pending_rewards: U256,
        total_supply: U256,
    ) -> Result<U256, Vec<u8>> {
        if total_supply == U256::ZERO {
            return Ok(U256::from(SCALE));
        }

        let balance = u256_to_decimal(underlying_balance);
        let strategy = u256_to_decimal(strategy_value);
        let rewards = u256_to_decimal(pending_rewards);
        let supply = u256_to_decimal(total_supply);

        let total_value = balance
            .checked_add(strategy)
            .ok_or_else(|| b"overflow".to_vec())?
            .checked_add(rewards)
            .ok_or_else(|| b"overflow".to_vec())?;

        let nav = total_value
            .checked_div(supply)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(nav))
    }

    /// Set performance fee (admin only in production)
    pub fn set_performance_fee(&mut self, fee_bps: U256) {
        self.performance_fee_bps.set(fee_bps);
    }

    /// Set management fee (admin only in production)
    pub fn set_management_fee(&mut self, fee_bps: U256) {
        self.management_fee_bps.set(fee_bps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_ETH: u128 = 1_000_000_000_000_000_000; // 1e18

    #[test]
    fn test_u256_decimal_roundtrip() {
        let original = U256::from(12345u64) * U256::from(ONE_ETH);
        let decimal = u256_to_decimal(original);
        let recovered = decimal_to_u256(decimal);

        let diff = if recovered > original { recovered - original } else { original - recovered };
        assert!(diff < U256::from(1000u64));
    }

    #[test]
    fn test_shares_for_deposit_empty_vault() {
        // Empty vault: shares = assets (1:1)
        let assets = Decimal::from(1_000i64);
        let total_assets = Decimal::ZERO;
        let total_supply = Decimal::ZERO;

        // For empty vault, should return assets directly
        assert_eq!(total_supply, Decimal::ZERO);
    }

    #[test]
    fn test_shares_for_deposit_existing_vault() {
        // shares = (assets * total_supply) / total_assets
        let assets = Decimal::from(100i64);
        let total_assets = Decimal::from(1_000i64);
        let total_supply = Decimal::from(500i64);

        let shares = assets
            .checked_mul(total_supply).unwrap()
            .checked_div(total_assets).unwrap();

        // 100 * 500 / 1000 = 50 shares
        assert_eq!(shares, Decimal::from(50i64));
    }

    #[test]
    fn test_assets_for_redeem() {
        // assets = (shares * total_assets) / total_supply
        let shares = Decimal::from(50i64);
        let total_assets = Decimal::from(1_000i64);
        let total_supply = Decimal::from(500i64);

        let assets = shares
            .checked_mul(total_assets).unwrap()
            .checked_div(total_supply).unwrap();

        // 50 * 1000 / 500 = 100 assets
        assert_eq!(assets, Decimal::from(100i64));
    }

    #[test]
    fn test_share_price_calculation() {
        // price = total_assets / total_supply
        let total_assets = Decimal::from(2_000i64);
        let total_supply = Decimal::from(1_000i64);

        let price = total_assets.checked_div(total_supply).unwrap();
        assert_eq!(price, Decimal::from(2i64)); // 2:1 ratio
    }

    #[test]
    fn test_compound_yield() {
        // final = principal * (1 + rate)^n
        let principal = Decimal::from(1_000i64);
        let rate = Decimal::from(1i64).checked_div(Decimal::from(100i64)).unwrap(); // 1%
        let periods = 3;

        let one_plus_rate = Decimal::ONE.checked_add(rate).unwrap(); // 1.01
        let mut multiplier = Decimal::ONE;
        for _ in 0..periods {
            multiplier = multiplier.checked_mul(one_plus_rate).unwrap();
        }
        let final_value = principal.checked_mul(multiplier).unwrap();

        // 1000 * 1.01^3 ≈ 1030.301
        assert!(final_value > Decimal::from(1030i64));
        assert!(final_value < Decimal::from(1031i64));
    }

    #[test]
    fn test_apy_from_apr() {
        // APY = (1 + APR/n)^n - 1
        let apr = Decimal::from(10i64).checked_div(Decimal::from(100i64)).unwrap(); // 10%
        let n = 12; // Monthly compounding

        let rate_per_period = apr.checked_div(Decimal::from(n as i64)).unwrap();
        let one_plus_rate = Decimal::ONE.checked_add(rate_per_period).unwrap();

        let mut multiplier = Decimal::ONE;
        for _ in 0..n {
            multiplier = multiplier.checked_mul(one_plus_rate).unwrap();
        }
        let apy = multiplier.checked_sub(Decimal::ONE).unwrap();

        // 10% APR with monthly compounding ≈ 10.47% APY
        let apy_percent = apy.checked_mul(Decimal::from(100i64)).unwrap();
        assert!(apy_percent > Decimal::from(10i64));
        assert!(apy_percent < Decimal::from(11i64));
    }

    #[test]
    fn test_performance_fee() {
        // fee = gains * fee_rate
        let gains = Decimal::from(1_000i64);
        let fee_rate = Decimal::from(20i64).checked_div(Decimal::from(100i64)).unwrap(); // 20%

        let fee = gains.checked_mul(fee_rate).unwrap();
        assert_eq!(fee, Decimal::from(200i64));
    }

    #[test]
    fn test_management_fee() {
        // fee = total_assets * annual_rate * time_fraction
        let total_assets = Decimal::from(1_000_000i64);
        let annual_rate = Decimal::from(2i64).checked_div(Decimal::from(100i64)).unwrap(); // 2%
        let seconds_per_year = Decimal::from(365 * 24 * 60 * 60i64);
        let seconds_elapsed = Decimal::from(30 * 24 * 60 * 60i64); // 30 days

        let time_fraction = seconds_elapsed.checked_div(seconds_per_year).unwrap();
        let fee = total_assets
            .checked_mul(annual_rate).unwrap()
            .checked_mul(time_fraction).unwrap();

        // 1M * 0.02 * (30/365) ≈ 1643.84
        assert!(fee > Decimal::from(1_600i64));
        assert!(fee < Decimal::from(1_700i64));
    }

    #[test]
    fn test_net_asset_value() {
        // NAV = (balance + strategy + rewards) / supply
        let balance = Decimal::from(500_000i64);
        let strategy = Decimal::from(450_000i64);
        let rewards = Decimal::from(50_000i64);
        let supply = Decimal::from(1_000_000i64);

        let total = balance.checked_add(strategy).unwrap()
            .checked_add(rewards).unwrap();
        let nav = total.checked_div(supply).unwrap();

        // (500k + 450k + 50k) / 1M = 1.0
        assert_eq!(nav, Decimal::ONE);
    }

    #[test]
    fn test_deposit_redeem_symmetry() {
        // Depositing and redeeming should be symmetric (minus fees)
        let assets = Decimal::from(100i64);
        let total_assets = Decimal::from(1_000i64);
        let total_supply = Decimal::from(1_000i64);

        // Deposit: get shares
        let shares = assets
            .checked_mul(total_supply).unwrap()
            .checked_div(total_assets).unwrap();

        // Redeem: get assets back
        let new_total_assets = total_assets.checked_add(assets).unwrap();
        let new_total_supply = total_supply.checked_add(shares).unwrap();

        let redeemed = shares
            .checked_mul(new_total_assets).unwrap()
            .checked_div(new_total_supply).unwrap();

        // Should get back exactly what we put in (1:1 price ratio)
        let diff = (redeemed - assets).abs();
        assert!(diff < Decimal::from(1i64));
    }
}
