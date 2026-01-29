//! Stylus Gas Benchmark
//!
//! Unified benchmark contract for comparing Keystone precision arithmetic
//! against Solidity implementations.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use alloy_primitives::U256;
use precision_core::{Decimal, RoundingMode};
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct GasBenchmark {}
}

const SCALE: u64 = 1_000_000_000_000_000_000;
const BPS_DIVISOR: u64 = 10_000;

fn u256_to_decimal(value: U256) -> Decimal {
    let lo: u128 = value.as_limbs()[0] as u128 | ((value.as_limbs()[1] as u128) << 64);
    let raw = Decimal::from(lo);
    raw.checked_div(Decimal::from(SCALE))
        .unwrap_or(Decimal::MAX)
}

fn decimal_to_u256(value: Decimal) -> U256 {
    let scaled = value
        .checked_mul(Decimal::from(SCALE))
        .unwrap_or(Decimal::MAX)
        .round(0, RoundingMode::TowardZero);
    let (mantissa, _scale) = scaled.to_parts();
    U256::from(mantissa.unsigned_abs())
}

#[public]
impl GasBenchmark {
    // ========================================================================
    // Lending Calculations
    // ========================================================================

    pub fn calculate_health_factor(
        &self,
        collateral_value: U256,
        debt_value: U256,
        threshold_bps: U256,
    ) -> Result<U256, Vec<u8>> {
        if debt_value == U256::ZERO {
            return Ok(U256::MAX);
        }

        let collateral = u256_to_decimal(collateral_value);
        let debt = u256_to_decimal(debt_value);
        let threshold = u256_to_decimal(threshold_bps)
            .checked_div(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let weighted = collateral
            .checked_mul(threshold)
            .ok_or_else(|| b"overflow".to_vec())?;

        let hf = weighted
            .checked_div(debt)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(hf))
    }

    pub fn calculate_liquidation_price(
        &self,
        collateral_amount: U256,
        debt_value: U256,
        threshold_bps: U256,
    ) -> Result<U256, Vec<u8>> {
        if collateral_amount == U256::ZERO {
            return Err(b"zero collateral".to_vec());
        }

        let amount = u256_to_decimal(collateral_amount);
        let debt = u256_to_decimal(debt_value);
        let threshold = u256_to_decimal(threshold_bps)
            .checked_div(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let denom = amount
            .checked_mul(threshold)
            .ok_or_else(|| b"overflow".to_vec())?;

        let price = debt
            .checked_div(denom)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(price))
    }

    pub fn calculate_max_borrow(
        &self,
        collateral_value: U256,
        target_health_factor: U256,
        threshold_bps: U256,
    ) -> Result<U256, Vec<u8>> {
        let collateral = u256_to_decimal(collateral_value);
        let target_hf = u256_to_decimal(target_health_factor);
        let threshold = u256_to_decimal(threshold_bps)
            .checked_div(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let weighted = collateral
            .checked_mul(threshold)
            .ok_or_else(|| b"overflow".to_vec())?;

        let max_borrow = weighted
            .checked_div(target_hf)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(max_borrow))
    }

    // ========================================================================
    // AMM Calculations
    // ========================================================================

    pub fn calculate_swap_output(
        &self,
        reserve_in: U256,
        reserve_out: U256,
        amount_in: U256,
        fee_bps: U256,
    ) -> Result<U256, Vec<u8>> {
        if reserve_in == U256::ZERO || reserve_out == U256::ZERO {
            return Err(b"zero reserve".to_vec());
        }
        if amount_in == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let r_in = u256_to_decimal(reserve_in);
        let r_out = u256_to_decimal(reserve_out);
        let amt_in = u256_to_decimal(amount_in);
        let fee = u256_to_decimal(fee_bps)
            .checked_div(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let fee_mult = Decimal::ONE
            .checked_sub(fee)
            .ok_or_else(|| b"underflow".to_vec())?;

        let amt_in_with_fee = amt_in
            .checked_mul(fee_mult)
            .ok_or_else(|| b"overflow".to_vec())?;

        let num = r_out
            .checked_mul(amt_in_with_fee)
            .ok_or_else(|| b"overflow".to_vec())?;

        let denom = r_in
            .checked_add(amt_in_with_fee)
            .ok_or_else(|| b"overflow".to_vec())?;

        let amt_out = num
            .checked_div(denom)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(amt_out))
    }

    pub fn calculate_price_impact(
        &self,
        reserve_in: U256,
        reserve_out: U256,
        amount_in: U256,
        fee_bps: U256,
    ) -> Result<U256, Vec<u8>> {
        if amount_in == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let r_in = u256_to_decimal(reserve_in);
        let r_out = u256_to_decimal(reserve_out);
        let amt_in = u256_to_decimal(amount_in);

        let spot_price = r_out
            .checked_div(r_in)
            .ok_or_else(|| b"division error".to_vec())?;

        let amt_out = u256_to_decimal(
            self.calculate_swap_output(reserve_in, reserve_out, amount_in, fee_bps)?
        );

        let eff_price = amt_out
            .checked_div(amt_in)
            .ok_or_else(|| b"division error".to_vec())?;

        let impact = Decimal::ONE
            .checked_sub(
                eff_price
                    .checked_div(spot_price)
                    .ok_or_else(|| b"division error".to_vec())?
            )
            .unwrap_or(Decimal::ZERO)
            .max(Decimal::ZERO);

        Ok(decimal_to_u256(impact))
    }

    pub fn calculate_spot_price(
        &self,
        reserve_a: U256,
        reserve_b: U256,
    ) -> Result<U256, Vec<u8>> {
        if reserve_a == U256::ZERO {
            return Err(b"zero reserve".to_vec());
        }

        let r_a = u256_to_decimal(reserve_a);
        let r_b = u256_to_decimal(reserve_b);

        let price = r_b
            .checked_div(r_a)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(price))
    }

    // ========================================================================
    // Vault Calculations
    // ========================================================================

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

    pub fn calculate_compound_yield(
        &self,
        principal: U256,
        rate_bps: U256,
        periods: U256,
    ) -> Result<U256, Vec<u8>> {
        let p = u256_to_decimal(principal);
        let rate = u256_to_decimal(rate_bps)
            .checked_div(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let one_plus_rate = Decimal::ONE
            .checked_add(rate)
            .ok_or_else(|| b"overflow".to_vec())?;

        let n: u32 = periods.as_limbs()[0].min(365) as u32;

        let mut mult = Decimal::ONE;
        for _ in 0..n {
            mult = mult
                .checked_mul(one_plus_rate)
                .ok_or_else(|| b"overflow".to_vec())?;
        }

        let final_val = p
            .checked_mul(mult)
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok(decimal_to_u256(final_val))
    }

    pub fn calculate_apy_from_apr(
        &self,
        apr_bps: U256,
        compounds_per_year: U256,
    ) -> Result<U256, Vec<u8>> {
        let apr = u256_to_decimal(apr_bps)
            .checked_div(Decimal::from(BPS_DIVISOR as i64))
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

        let mut mult = Decimal::ONE;
        for _ in 0..n {
            mult = mult
                .checked_mul(one_plus_rate)
                .ok_or_else(|| b"overflow".to_vec())?;
        }

        let apy = mult
            .checked_sub(Decimal::ONE)
            .ok_or_else(|| b"underflow".to_vec())?;

        let apy_bps = apy
            .checked_mul(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok(decimal_to_u256(apy_bps))
    }
}
