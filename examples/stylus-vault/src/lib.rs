//! Stylus Vault Example
//!
//! Demonstrates using Keystone precision arithmetic in a Stylus smart contract
//! for ERC4626-style vault calculations with deterministic results.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::vec::Vec;
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
