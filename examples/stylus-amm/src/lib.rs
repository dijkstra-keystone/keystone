//! Stylus AMM Example
//!
//! Demonstrates using Keystone precision arithmetic in a Stylus smart contract
//! for constant product AMM calculations with deterministic results.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use alloy_primitives::U256;
use precision_core::{Decimal, RoundingMode};
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct AmmPool {
        /// Fee in basis points (e.g., 30 = 0.3%)
        uint256 fee_bps;
    }
}

const SCALE: u64 = 1_000_000_000_000_000_000;
const BPS_DIVISOR: u64 = 10_000;

/// Integer square root using Newton-Raphson (no floating point)
fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

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
impl AmmPool {
    /// Calculate output amount for constant product AMM (x*y=k)
    ///
    /// Formula: amount_out = (reserve_out * amount_in * (1 - fee)) / (reserve_in + amount_in * (1 - fee))
    ///
    /// All values scaled by 1e18
    pub fn calculate_swap_output(
        &self,
        reserve_in: U256,
        reserve_out: U256,
        amount_in: U256,
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
        let fee_bps = u256_to_decimal(self.fee_bps.get());

        let fee_multiplier = Decimal::ONE
            .checked_sub(fee_bps.checked_div(Decimal::from(BPS_DIVISOR)).ok_or_else(|| b"division error".to_vec())?)
            .ok_or_else(|| b"underflow".to_vec())?;

        let amount_in_with_fee = amt_in
            .checked_mul(fee_multiplier)
            .ok_or_else(|| b"overflow".to_vec())?;

        let numerator = r_out
            .checked_mul(amount_in_with_fee)
            .ok_or_else(|| b"overflow".to_vec())?;

        let denominator = r_in
            .checked_add(amount_in_with_fee)
            .ok_or_else(|| b"overflow".to_vec())?;

        let amount_out = numerator
            .checked_div(denominator)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(amount_out))
    }

    /// Calculate price impact percentage (scaled by 1e18, e.g., 1e16 = 1%)
    ///
    /// Price impact = 1 - (new_price / spot_price)
    pub fn calculate_price_impact(
        &self,
        reserve_in: U256,
        reserve_out: U256,
        amount_in: U256,
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

        let spot_price = r_out
            .checked_div(r_in)
            .ok_or_else(|| b"division error".to_vec())?;

        let amount_out = u256_to_decimal(self.calculate_swap_output(reserve_in, reserve_out, amount_in)?);

        let effective_price = amount_out
            .checked_div(amt_in)
            .ok_or_else(|| b"division error".to_vec())?;

        let impact = Decimal::ONE
            .checked_sub(effective_price.checked_div(spot_price).ok_or_else(|| b"division error".to_vec())?)
            .unwrap_or(Decimal::ZERO);

        Ok(decimal_to_u256(impact.max(Decimal::ZERO)))
    }

    /// Calculate required input amount for desired output
    ///
    /// Formula: amount_in = (reserve_in * amount_out) / ((reserve_out - amount_out) * (1 - fee))
    pub fn calculate_swap_input(
        &self,
        reserve_in: U256,
        reserve_out: U256,
        amount_out: U256,
    ) -> Result<U256, Vec<u8>> {
        if reserve_in == U256::ZERO || reserve_out == U256::ZERO {
            return Err(b"zero reserve".to_vec());
        }
        if amount_out == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let r_in = u256_to_decimal(reserve_in);
        let r_out = u256_to_decimal(reserve_out);
        let amt_out = u256_to_decimal(amount_out);

        if amt_out >= r_out {
            return Err(b"insufficient liquidity".to_vec());
        }

        let fee_bps = u256_to_decimal(self.fee_bps.get());
        let fee_multiplier = Decimal::ONE
            .checked_sub(fee_bps.checked_div(Decimal::from(BPS_DIVISOR)).ok_or_else(|| b"division error".to_vec())?)
            .ok_or_else(|| b"underflow".to_vec())?;

        let numerator = r_in
            .checked_mul(amt_out)
            .ok_or_else(|| b"overflow".to_vec())?;

        let denominator = r_out
            .checked_sub(amt_out)
            .ok_or_else(|| b"underflow".to_vec())?
            .checked_mul(fee_multiplier)
            .ok_or_else(|| b"overflow".to_vec())?;

        let amount_in = numerator
            .checked_div(denominator)
            .ok_or_else(|| b"division error".to_vec())?
            .checked_add(Decimal::ONE.checked_div(Decimal::from(SCALE)).unwrap_or(Decimal::ZERO))
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok(decimal_to_u256(amount_in))
    }

    /// Calculate spot price (reserve_b / reserve_a, scaled by 1e18)
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

    /// Calculate liquidity shares to mint for a deposit
    ///
    /// For first deposit: shares = sqrt(amount_a * amount_b)
    /// For subsequent: shares = min(amount_a / reserve_a, amount_b / reserve_b) * total_supply
    pub fn calculate_liquidity_mint(
        &self,
        amount_a: U256,
        amount_b: U256,
        reserve_a: U256,
        reserve_b: U256,
        total_supply: U256,
    ) -> Result<U256, Vec<u8>> {
        let amt_a = u256_to_decimal(amount_a);
        let amt_b = u256_to_decimal(amount_b);

        if total_supply == U256::ZERO {
            let product = amt_a
                .checked_mul(amt_b)
                .ok_or_else(|| b"overflow".to_vec())?;
            let (mantissa, scale) = product.to_parts();
            let sqrt_mantissa = isqrt(mantissa.unsigned_abs());
            let sqrt_scale = scale / 2;
            let shares = Decimal::new(sqrt_mantissa as i64, sqrt_scale);
            return Ok(decimal_to_u256(shares));
        }

        let r_a = u256_to_decimal(reserve_a);
        let r_b = u256_to_decimal(reserve_b);
        let supply = u256_to_decimal(total_supply);

        if r_a == Decimal::ZERO || r_b == Decimal::ZERO {
            return Err(b"zero reserve".to_vec());
        }

        let ratio_a = amt_a
            .checked_div(r_a)
            .ok_or_else(|| b"division error".to_vec())?;
        let ratio_b = amt_b
            .checked_div(r_b)
            .ok_or_else(|| b"division error".to_vec())?;

        let min_ratio = ratio_a.min(ratio_b);
        let shares = min_ratio
            .checked_mul(supply)
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok(decimal_to_u256(shares))
    }

    /// Calculate amounts to return when burning liquidity shares
    ///
    /// amount_a = (shares / total_supply) * reserve_a
    /// amount_b = (shares / total_supply) * reserve_b
    pub fn calculate_liquidity_burn(
        &self,
        shares: U256,
        reserve_a: U256,
        reserve_b: U256,
        total_supply: U256,
    ) -> Result<(U256, U256), Vec<u8>> {
        if total_supply == U256::ZERO {
            return Err(b"zero supply".to_vec());
        }

        let s = u256_to_decimal(shares);
        let r_a = u256_to_decimal(reserve_a);
        let r_b = u256_to_decimal(reserve_b);
        let supply = u256_to_decimal(total_supply);

        let ratio = s
            .checked_div(supply)
            .ok_or_else(|| b"division error".to_vec())?;

        let amount_a = ratio
            .checked_mul(r_a)
            .ok_or_else(|| b"overflow".to_vec())?;
        let amount_b = ratio
            .checked_mul(r_b)
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok((decimal_to_u256(amount_a), decimal_to_u256(amount_b)))
    }

    /// Set swap fee (admin only in production)
    pub fn set_fee(&mut self, fee_bps: U256) {
        self.fee_bps.set(fee_bps);
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
    fn test_constant_product_invariant() {
        // For x*y=k AMM, after a swap: (x + dx) * (y - dy) = x * y
        let r_in = Decimal::from(1_000_000i64); // 1M tokens
        let r_out = Decimal::from(1_000_000i64); // 1M tokens
        let amt_in = Decimal::from(1_000i64); // 1K tokens
        let fee = Decimal::ZERO; // No fee for this test

        let k_before = r_in.checked_mul(r_out).unwrap();

        // amount_out = (r_out * amt_in) / (r_in + amt_in)
        let numerator = r_out.checked_mul(amt_in).unwrap();
        let denominator = r_in.checked_add(amt_in).unwrap();
        let amt_out = numerator.checked_div(denominator).unwrap();

        let new_r_in = r_in.checked_add(amt_in).unwrap();
        let new_r_out = r_out.checked_sub(amt_out).unwrap();
        let k_after = new_r_in.checked_mul(new_r_out).unwrap();

        // k should be approximately equal (may have slight rounding)
        let diff = (k_after - k_before).abs();
        let tolerance = Decimal::from(1i64); // 1 token tolerance
        assert!(diff < tolerance);
    }

    #[test]
    fn test_swap_output_formula() {
        // With 0.3% fee (30 bps):
        // amount_out = (r_out * amt_in * 0.997) / (r_in + amt_in * 0.997)
        let r_in = Decimal::from(100_000i64);
        let r_out = Decimal::from(100_000i64);
        let amt_in = Decimal::from(1_000i64);
        let fee_multiplier = Decimal::from(997i64).checked_div(Decimal::from(1_000i64)).unwrap();

        let amt_in_with_fee = amt_in.checked_mul(fee_multiplier).unwrap();
        let numerator = r_out.checked_mul(amt_in_with_fee).unwrap();
        let denominator = r_in.checked_add(amt_in_with_fee).unwrap();
        let amt_out = numerator.checked_div(denominator).unwrap();

        // Approximate expected: 1000 * 0.997 * 100000 / (100000 + 997) ≈ 987
        assert!(amt_out > Decimal::from(985i64));
        assert!(amt_out < Decimal::from(990i64));
    }

    #[test]
    fn test_price_impact_increases_with_size() {
        let r_in = Decimal::from(1_000_000i64);
        let r_out = Decimal::from(1_000_000i64);

        // Small trade
        let small_amt = Decimal::from(1_000i64);
        let small_out = r_out.checked_mul(small_amt).unwrap()
            .checked_div(r_in.checked_add(small_amt).unwrap()).unwrap();
        let small_impact = Decimal::ONE
            .checked_sub(small_out.checked_div(small_amt).unwrap()).unwrap();

        // Large trade
        let large_amt = Decimal::from(100_000i64);
        let large_out = r_out.checked_mul(large_amt).unwrap()
            .checked_div(r_in.checked_add(large_amt).unwrap()).unwrap();
        let large_impact = Decimal::ONE
            .checked_sub(large_out.checked_div(large_amt).unwrap()).unwrap();

        assert!(large_impact > small_impact);
    }

    #[test]
    fn test_spot_price_calculation() {
        let r_a = Decimal::from(2_000i64);
        let r_b = Decimal::from(4_000i64);

        let price = r_b.checked_div(r_a).unwrap();
        assert_eq!(price, Decimal::from(2i64)); // 1 A = 2 B
    }

    #[test]
    fn test_liquidity_mint_first_deposit() {
        let amt_a = Decimal::from(1_000_000i64);
        let amt_b = Decimal::from(1_000_000i64);

        let product = amt_a.checked_mul(amt_b).unwrap();
        // sqrt(1e6 * 1e6) = 1e6
        assert_eq!(product, Decimal::from(1_000_000_000_000i64));
    }

    #[test]
    fn test_liquidity_burn_returns_proportional() {
        let shares = Decimal::from(100i64);
        let total_supply = Decimal::from(1_000i64);
        let r_a = Decimal::from(10_000i64);
        let r_b = Decimal::from(20_000i64);

        let ratio = shares.checked_div(total_supply).unwrap(); // 0.1
        let amt_a = ratio.checked_mul(r_a).unwrap(); // 1000
        let amt_b = ratio.checked_mul(r_b).unwrap(); // 2000

        assert_eq!(amt_a, Decimal::from(1_000i64));
        assert_eq!(amt_b, Decimal::from(2_000i64));
    }

    #[test]
    fn test_swap_input_formula() {
        // If we want exactly 1000 out, how much in do we need?
        // amount_in = (r_in * amt_out) / ((r_out - amt_out) * (1 - fee))
        let r_in = Decimal::from(100_000i64);
        let r_out = Decimal::from(100_000i64);
        let amt_out = Decimal::from(1_000i64);
        let fee_multiplier = Decimal::from(997i64).checked_div(Decimal::from(1_000i64)).unwrap();

        let numerator = r_in.checked_mul(amt_out).unwrap();
        let denominator = r_out.checked_sub(amt_out).unwrap()
            .checked_mul(fee_multiplier).unwrap();
        let amt_in = numerator.checked_div(denominator).unwrap();

        // Should be slightly more than 1000 due to fee
        assert!(amt_in > Decimal::from(1_010i64));
        assert!(amt_in < Decimal::from(1_020i64));
    }
}
