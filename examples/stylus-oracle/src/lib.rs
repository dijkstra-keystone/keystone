//! Stylus Oracle Integration Example
//!
//! Demonstrates integrating RedStone oracle price feeds with Keystone precision
//! arithmetic for accurate DeFi calculations on Arbitrum Stylus.
//!
//! RedStone uses a pull-based model where price data is passed in transaction
//! calldata and verified cryptographically on-chain (~3K gas per feed per signer).

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use alloy_primitives::{Address, U256};
use precision_core::{Decimal, RoundingMode};
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct OraclePricedLending {
        /// Liquidation threshold in basis points
        uint256 liquidation_threshold_bps;
        /// Liquidation bonus in basis points
        uint256 liquidation_bonus_bps;
        /// Trusted price feed signers (RedStone authorized signers)
        mapping(address => bool) trusted_signers;
        /// Minimum required signers for price validity
        uint256 min_signers;
        /// Maximum price staleness in seconds
        uint256 max_staleness;
    }
}

const SCALE: u64 = 1_000_000_000_000_000_000;
const BPS_DIVISOR: u64 = 10_000;

/// Price feed data structure (RedStone format)
#[derive(Clone, Copy)]
pub struct PriceFeed {
    /// Asset identifier (e.g., keccak256("ETH"))
    pub asset_id: [u8; 32],
    /// Price value (8 decimals, e.g., 200000000000 = $2000.00)
    pub value: u128,
    /// Timestamp in seconds
    pub timestamp: u64,
}

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

/// Convert oracle price (8 decimals) to internal decimal
fn oracle_price_to_decimal(price_8dec: u128) -> Decimal {
    let raw = Decimal::from(price_8dec);
    raw.checked_div(Decimal::from(100_000_000i64))
        .unwrap_or(Decimal::ZERO)
}

#[public]
impl OraclePricedLending {
    // ========================================================================
    // Oracle-Integrated Lending Functions
    // ========================================================================

    /// Calculate health factor using oracle prices
    ///
    /// # Arguments
    /// * `collateral_amount` - Amount of collateral tokens (18 decimals)
    /// * `collateral_price` - Collateral price from oracle (8 decimals)
    /// * `debt_amount` - Amount of debt tokens (18 decimals)
    /// * `debt_price` - Debt price from oracle (8 decimals)
    ///
    /// # Returns
    /// Health factor scaled by 1e18 (1.5e18 = 150%)
    pub fn calculate_health_factor_with_prices(
        &self,
        collateral_amount: U256,
        collateral_price: U256,
        debt_amount: U256,
        debt_price: U256,
    ) -> Result<U256, Vec<u8>> {
        if debt_amount == U256::ZERO {
            return Ok(U256::MAX);
        }

        let coll_amt = u256_to_decimal(collateral_amount);
        let coll_price = oracle_price_to_decimal(collateral_price.as_limbs()[0] as u128);
        let debt_amt = u256_to_decimal(debt_amount);
        let debt_pr = oracle_price_to_decimal(debt_price.as_limbs()[0] as u128);

        let threshold = u256_to_decimal(self.liquidation_threshold_bps.get())
            .checked_div(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"division error".to_vec())?;

        // Collateral value = amount * price
        let collateral_value = coll_amt
            .checked_mul(coll_price)
            .ok_or_else(|| b"overflow".to_vec())?;

        // Debt value = amount * price
        let debt_value = debt_amt
            .checked_mul(debt_pr)
            .ok_or_else(|| b"overflow".to_vec())?;

        if debt_value == Decimal::ZERO {
            return Ok(U256::MAX);
        }

        // Health factor = (collateral_value * threshold) / debt_value
        let weighted = collateral_value
            .checked_mul(threshold)
            .ok_or_else(|| b"overflow".to_vec())?;

        let hf = weighted
            .checked_div(debt_value)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(hf))
    }

    /// Calculate liquidation price using oracle data
    ///
    /// Returns the collateral price at which the position becomes liquidatable
    pub fn calculate_liquidation_price_with_oracle(
        &self,
        collateral_amount: U256,
        debt_amount: U256,
        debt_price: U256,
    ) -> Result<U256, Vec<u8>> {
        if collateral_amount == U256::ZERO {
            return Err(b"zero collateral".to_vec());
        }

        let coll_amt = u256_to_decimal(collateral_amount);
        let debt_amt = u256_to_decimal(debt_amount);
        let debt_pr = oracle_price_to_decimal(debt_price.as_limbs()[0] as u128);

        let threshold = u256_to_decimal(self.liquidation_threshold_bps.get())
            .checked_div(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"division error".to_vec())?;

        // Debt value in USD
        let debt_value = debt_amt
            .checked_mul(debt_pr)
            .ok_or_else(|| b"overflow".to_vec())?;

        // Liquidation price = debt_value / (collateral_amount * threshold)
        let denom = coll_amt
            .checked_mul(threshold)
            .ok_or_else(|| b"overflow".to_vec())?;

        let liq_price = debt_value
            .checked_div(denom)
            .ok_or_else(|| b"division error".to_vec())?;

        // Convert back to 8 decimals for oracle format
        let liq_price_8dec = liq_price
            .checked_mul(Decimal::from(100_000_000i64))
            .ok_or_else(|| b"overflow".to_vec())?
            .round(0, RoundingMode::TowardZero);

        let (mantissa, _) = liq_price_8dec.to_parts();
        Ok(U256::from(mantissa.unsigned_abs()))
    }

    /// Calculate maximum borrowable amount at target health factor
    pub fn calculate_max_borrow_with_prices(
        &self,
        collateral_amount: U256,
        collateral_price: U256,
        debt_price: U256,
        target_health_factor: U256,
    ) -> Result<U256, Vec<u8>> {
        let coll_amt = u256_to_decimal(collateral_amount);
        let coll_price = oracle_price_to_decimal(collateral_price.as_limbs()[0] as u128);
        let debt_pr = oracle_price_to_decimal(debt_price.as_limbs()[0] as u128);
        let target_hf = u256_to_decimal(target_health_factor);

        let threshold = u256_to_decimal(self.liquidation_threshold_bps.get())
            .checked_div(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"division error".to_vec())?;

        // Collateral value
        let coll_value = coll_amt
            .checked_mul(coll_price)
            .ok_or_else(|| b"overflow".to_vec())?;

        // Max debt value = (collateral_value * threshold) / target_hf
        let weighted = coll_value
            .checked_mul(threshold)
            .ok_or_else(|| b"overflow".to_vec())?;

        let max_debt_value = weighted
            .checked_div(target_hf)
            .ok_or_else(|| b"division error".to_vec())?;

        // Max borrow amount = max_debt_value / debt_price
        let max_borrow = max_debt_value
            .checked_div(debt_pr)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(max_borrow))
    }

    /// Check if position is liquidatable at current oracle prices
    pub fn is_liquidatable_with_prices(
        &self,
        collateral_amount: U256,
        collateral_price: U256,
        debt_amount: U256,
        debt_price: U256,
    ) -> Result<bool, Vec<u8>> {
        let hf = self.calculate_health_factor_with_prices(
            collateral_amount,
            collateral_price,
            debt_amount,
            debt_price,
        )?;

        let one = U256::from(SCALE);
        Ok(hf < one)
    }

    /// Calculate liquidation amounts with bonus
    pub fn calculate_liquidation_with_prices(
        &self,
        debt_to_cover: U256,
        collateral_price: U256,
        debt_price: U256,
    ) -> Result<(U256, U256), Vec<u8>> {
        let debt_amt = u256_to_decimal(debt_to_cover);
        let coll_price = oracle_price_to_decimal(collateral_price.as_limbs()[0] as u128);
        let debt_pr = oracle_price_to_decimal(debt_price.as_limbs()[0] as u128);

        let bonus = u256_to_decimal(self.liquidation_bonus_bps.get())
            .checked_div(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let one_plus_bonus = Decimal::ONE
            .checked_add(bonus)
            .ok_or_else(|| b"overflow".to_vec())?;

        // Debt value to cover
        let debt_value = debt_amt
            .checked_mul(debt_pr)
            .ok_or_else(|| b"overflow".to_vec())?;

        // Collateral to receive = (debt_value / collateral_price) * (1 + bonus)
        let base_collateral = debt_value
            .checked_div(coll_price)
            .ok_or_else(|| b"division error".to_vec())?;

        let total_collateral = base_collateral
            .checked_mul(one_plus_bonus)
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok((debt_to_cover, decimal_to_u256(total_collateral)))
    }

    // ========================================================================
    // Price Aggregation
    // ========================================================================

    /// Calculate TWAP (Time-Weighted Average Price) from multiple price points
    ///
    /// Useful for manipulation resistance when using multiple oracle updates
    pub fn calculate_twap(
        &self,
        prices: Vec<U256>,
        timestamps: Vec<U256>,
    ) -> Result<U256, Vec<u8>> {
        if prices.len() != timestamps.len() || prices.is_empty() {
            return Err(b"invalid input".to_vec());
        }

        if prices.len() == 1 {
            return Ok(prices[0]);
        }

        let mut weighted_sum = Decimal::ZERO;
        let mut total_time = Decimal::ZERO;

        for i in 1..prices.len() {
            let price = oracle_price_to_decimal(prices[i].as_limbs()[0] as u128);
            let prev_price = oracle_price_to_decimal(prices[i - 1].as_limbs()[0] as u128);
            let avg_price = price
                .checked_add(prev_price)
                .ok_or_else(|| b"overflow".to_vec())?
                .checked_div(Decimal::from(2i64))
                .ok_or_else(|| b"division error".to_vec())?;

            let time_delta = Decimal::from(
                (timestamps[i].as_limbs()[0] - timestamps[i - 1].as_limbs()[0]) as i64,
            );

            weighted_sum = weighted_sum
                .checked_add(avg_price.checked_mul(time_delta).ok_or_else(|| b"overflow".to_vec())?)
                .ok_or_else(|| b"overflow".to_vec())?;

            total_time = total_time
                .checked_add(time_delta)
                .ok_or_else(|| b"overflow".to_vec())?;
        }

        if total_time == Decimal::ZERO {
            return Ok(prices[0]);
        }

        let twap = weighted_sum
            .checked_div(total_time)
            .ok_or_else(|| b"division error".to_vec())?;

        // Convert back to 8 decimals
        let twap_8dec = twap
            .checked_mul(Decimal::from(100_000_000i64))
            .ok_or_else(|| b"overflow".to_vec())?
            .round(0, RoundingMode::HalfEven);

        let (mantissa, _) = twap_8dec.to_parts();
        Ok(U256::from(mantissa.unsigned_abs()))
    }

    /// Calculate price deviation from median (for anomaly detection)
    pub fn calculate_price_deviation(
        &self,
        price: U256,
        median_price: U256,
    ) -> Result<U256, Vec<u8>> {
        let p = oracle_price_to_decimal(price.as_limbs()[0] as u128);
        let median = oracle_price_to_decimal(median_price.as_limbs()[0] as u128);

        if median == Decimal::ZERO {
            return Err(b"zero median".to_vec());
        }

        let diff = if p > median { p - median } else { median - p };
        let deviation = diff
            .checked_div(median)
            .ok_or_else(|| b"division error".to_vec())?;

        // Return as basis points (1e4 scale)
        let deviation_bps = deviation
            .checked_mul(Decimal::from(BPS_DIVISOR as i64))
            .ok_or_else(|| b"overflow".to_vec())?
            .round(0, RoundingMode::TowardZero);

        let (mantissa, _) = deviation_bps.to_parts();
        Ok(U256::from(mantissa.unsigned_abs()))
    }

    // ========================================================================
    // Admin Functions
    // ========================================================================

    pub fn set_liquidation_threshold(&mut self, threshold_bps: U256) {
        self.liquidation_threshold_bps.set(threshold_bps);
    }

    pub fn set_liquidation_bonus(&mut self, bonus_bps: U256) {
        self.liquidation_bonus_bps.set(bonus_bps);
    }

    pub fn set_trusted_signer(&mut self, signer: Address, trusted: bool) {
        self.trusted_signers.setter(signer).set(trusted);
    }

    pub fn set_min_signers(&mut self, min: U256) {
        self.min_signers.set(min);
    }

    pub fn set_max_staleness(&mut self, seconds: U256) {
        self.max_staleness.set(seconds);
    }

    pub fn is_trusted_signer(&self, signer: Address) -> bool {
        self.trusted_signers.get(signer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_ETH: u128 = 1_000_000_000_000_000_000;
    const ETH_PRICE_8DEC: u128 = 200_000_000_000; // $2000.00

    #[test]
    fn test_oracle_price_conversion() {
        let price = oracle_price_to_decimal(ETH_PRICE_8DEC);
        assert_eq!(price, Decimal::from(2000i64));
    }

    #[test]
    fn test_health_factor_with_oracle_prices() {
        // 10 ETH collateral at $2000 = $20,000 value
        // 10,000 USDC debt at $1 = $10,000 value
        // Threshold 80%
        // HF = (20,000 * 0.8) / 10,000 = 1.6

        let coll_amt = Decimal::from(10i64);
        let coll_price = Decimal::from(2000i64);
        let debt_amt = Decimal::from(10_000i64);
        let debt_price = Decimal::ONE;
        let threshold = Decimal::from(8i64)
            .checked_div(Decimal::from(10i64))
            .unwrap();

        let coll_value = coll_amt.checked_mul(coll_price).unwrap();
        let debt_value = debt_amt.checked_mul(debt_price).unwrap();
        let weighted = coll_value.checked_mul(threshold).unwrap();
        let hf = weighted.checked_div(debt_value).unwrap();

        let expected = Decimal::from(16i64)
            .checked_div(Decimal::from(10i64))
            .unwrap();
        assert_eq!(hf, expected);
    }

    #[test]
    fn test_liquidation_price_calculation() {
        // 10 ETH collateral, $10,000 USDC debt
        // Threshold 80%
        // Liquidation price = 10,000 / (10 * 0.8) = $1,250

        let debt_value = Decimal::from(10_000i64);
        let coll_amt = Decimal::from(10i64);
        let threshold = Decimal::from(8i64)
            .checked_div(Decimal::from(10i64))
            .unwrap();

        let denom = coll_amt.checked_mul(threshold).unwrap();
        let liq_price = debt_value.checked_div(denom).unwrap();

        assert_eq!(liq_price, Decimal::from(1_250i64));
    }

    #[test]
    fn test_twap_calculation() {
        // Simple average of two prices
        let p1 = Decimal::from(2000i64);
        let p2 = Decimal::from(2100i64);

        let avg = p1.checked_add(p2).unwrap()
            .checked_div(Decimal::from(2i64)).unwrap();

        assert_eq!(avg, Decimal::from(2050i64));
    }

    #[test]
    fn test_price_deviation() {
        // 5% deviation
        let price = Decimal::from(2100i64);
        let median = Decimal::from(2000i64);

        let diff = price - median;
        let deviation = diff.checked_div(median).unwrap();
        let deviation_bps = deviation
            .checked_mul(Decimal::from(10_000i64))
            .unwrap();

        assert_eq!(deviation_bps, Decimal::from(500i64)); // 5% = 500 bps
    }
}
