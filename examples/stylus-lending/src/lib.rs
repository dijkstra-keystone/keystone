//! Stylus Lending Example
//!
//! Demonstrates using Keystone precision arithmetic in a Stylus smart contract
//! for DeFi lending calculations with deterministic results.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use precision_core::{Decimal, RoundingMode};
use alloy_primitives::U256;
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct LendingPool {
        /// Liquidation threshold in basis points (e.g., 8000 = 80%)
        uint256 liquidation_threshold_bps;
        /// Liquidation bonus in basis points (e.g., 500 = 5%)
        uint256 liquidation_bonus_bps;
    }
}

/// Convert U256 to Decimal (assumes 18 decimals, scaled to 1e18)
fn u256_to_decimal(value: U256) -> Decimal {
    // Extract lower 128 bits (sufficient for most DeFi values)
    let lo: u128 = value.as_limbs()[0] as u128 | ((value.as_limbs()[1] as u128) << 64);
    // Create decimal and apply 18 decimal scaling
    let raw = Decimal::from(lo);
    raw.checked_div(Decimal::from(1_000_000_000_000_000_000u64))
        .unwrap_or(Decimal::MAX)
}

/// Convert Decimal to U256 (returns value scaled to 1e18)
fn decimal_to_u256(value: Decimal) -> U256 {
    // Scale up by 1e18 and round
    let scaled = value
        .checked_mul(Decimal::from(1_000_000_000_000_000_000u64))
        .unwrap_or(Decimal::MAX)
        .round(0, RoundingMode::TowardZero);
    let (mantissa, _scale) = scaled.to_parts();
    U256::from(mantissa.unsigned_abs())
}

#[public]
impl LendingPool {
    /// Calculate health factor for a position
    ///
    /// Health Factor = (Collateral Value × Liquidation Threshold) / Debt Value
    ///
    /// Returns health factor scaled by 1e18 (e.g., 1.5e18 = 150%)
    pub fn calculate_health_factor(
        &self,
        collateral_value: U256,
        debt_value: U256,
    ) -> Result<U256, Vec<u8>> {
        if debt_value == U256::ZERO {
            return Ok(U256::MAX);
        }

        let collateral = u256_to_decimal(collateral_value);
        let debt = u256_to_decimal(debt_value);
        let threshold_bps = u256_to_decimal(self.liquidation_threshold_bps.get());
        let threshold = threshold_bps
            .checked_div(Decimal::from(10000i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let weighted_collateral = collateral
            .checked_mul(threshold)
            .ok_or_else(|| b"overflow".to_vec())?;

        let health_factor = weighted_collateral
            .checked_div(debt)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(health_factor))
    }

    /// Calculate liquidation price for single-collateral position
    ///
    /// Liquidation Price = Debt / (Collateral Amount × Threshold)
    pub fn calculate_liquidation_price(
        &self,
        collateral_amount: U256,
        debt_value: U256,
    ) -> Result<U256, Vec<u8>> {
        if collateral_amount == U256::ZERO {
            return Err(b"zero collateral".to_vec());
        }

        let amount = u256_to_decimal(collateral_amount);
        let debt = u256_to_decimal(debt_value);
        let threshold_bps = u256_to_decimal(self.liquidation_threshold_bps.get());
        let threshold = threshold_bps
            .checked_div(Decimal::from(10000i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let denominator = amount
            .checked_mul(threshold)
            .ok_or_else(|| b"overflow".to_vec())?;

        let liquidation_price = debt
            .checked_div(denominator)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(liquidation_price))
    }

    /// Calculate maximum borrowable amount given collateral
    ///
    /// Max Borrow = (Collateral × Threshold) / Target Health Factor
    pub fn calculate_max_borrow(
        &self,
        collateral_value: U256,
        target_health_factor: U256,
    ) -> Result<U256, Vec<u8>> {
        let collateral = u256_to_decimal(collateral_value);
        let target_hf = u256_to_decimal(target_health_factor);
        let threshold_bps = u256_to_decimal(self.liquidation_threshold_bps.get());
        let threshold = threshold_bps
            .checked_div(Decimal::from(10000i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let weighted = collateral
            .checked_mul(threshold)
            .ok_or_else(|| b"overflow".to_vec())?;

        let max_borrow = weighted
            .checked_div(target_hf)
            .ok_or_else(|| b"division error".to_vec())?;

        Ok(decimal_to_u256(max_borrow))
    }

    /// Check if position is liquidatable
    pub fn is_liquidatable(&self, collateral_value: U256, debt_value: U256) -> Result<bool, Vec<u8>> {
        let hf = self.calculate_health_factor(collateral_value, debt_value)?;
        let one = U256::from(10u128.pow(18));
        Ok(hf < one)
    }

    /// Calculate liquidation amount and bonus
    ///
    /// Returns (debt_to_cover, collateral_to_receive)
    pub fn calculate_liquidation_amounts(
        &self,
        debt_to_cover: U256,
        collateral_price: U256,
    ) -> Result<(U256, U256), Vec<u8>> {
        let debt = u256_to_decimal(debt_to_cover);
        let price = u256_to_decimal(collateral_price);
        let bonus_bps = u256_to_decimal(self.liquidation_bonus_bps.get());
        let bonus = bonus_bps
            .checked_div(Decimal::from(10000i64))
            .ok_or_else(|| b"division error".to_vec())?;

        let one_plus_bonus = Decimal::ONE
            .checked_add(bonus)
            .ok_or_else(|| b"overflow".to_vec())?;

        let base_collateral = debt
            .checked_div(price)
            .ok_or_else(|| b"division error".to_vec())?;

        let total_collateral = base_collateral
            .checked_mul(one_plus_bonus)
            .ok_or_else(|| b"overflow".to_vec())?;

        Ok((debt_to_cover, decimal_to_u256(total_collateral)))
    }

    /// Set liquidation threshold (admin only in production)
    pub fn set_liquidation_threshold(&mut self, threshold_bps: U256) {
        self.liquidation_threshold_bps.set(threshold_bps);
    }

    /// Set liquidation bonus (admin only in production)
    pub fn set_liquidation_bonus(&mut self, bonus_bps: U256) {
        self.liquidation_bonus_bps.set(bonus_bps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_ETH: u128 = 1_000_000_000_000_000_000; // 1e18

    #[test]
    fn test_u256_to_decimal_conversion() {
        let one_eth = U256::from(ONE_ETH);
        let decimal = u256_to_decimal(one_eth);
        assert_eq!(decimal, Decimal::ONE);

        let half_eth = U256::from(ONE_ETH / 2);
        let decimal = u256_to_decimal(half_eth);
        let expected = Decimal::from(5i64).checked_div(Decimal::from(10i64)).unwrap();
        assert_eq!(decimal, expected);
    }

    #[test]
    fn test_decimal_to_u256_conversion() {
        let one = Decimal::ONE;
        let u256_val = decimal_to_u256(one);
        assert_eq!(u256_val, U256::from(ONE_ETH));

        let half = Decimal::from(5i64).checked_div(Decimal::from(10i64)).unwrap();
        let u256_val = decimal_to_u256(half);
        assert_eq!(u256_val, U256::from(ONE_ETH / 2));
    }

    #[test]
    fn test_u256_decimal_roundtrip() {
        let original = U256::from(12345u64) * U256::from(ONE_ETH);
        let decimal = u256_to_decimal(original);
        let recovered = decimal_to_u256(decimal);

        let diff = if recovered > original { recovered - original } else { original - recovered };
        assert!(diff < U256::from(1000u64));
    }

    #[test]
    fn test_health_factor_computation() {
        // Test the pure computation: HF = (collateral * threshold) / debt
        let collateral = Decimal::from(10_000i64);
        let debt = Decimal::from(5_000i64);
        let threshold = Decimal::from(8i64).checked_div(Decimal::from(10i64)).unwrap(); // 0.8

        let weighted = collateral.checked_mul(threshold).unwrap();
        let hf = weighted.checked_div(debt).unwrap();

        // HF = (10000 * 0.8) / 5000 = 1.6
        let expected = Decimal::from(16i64).checked_div(Decimal::from(10i64)).unwrap();
        assert_eq!(hf, expected);
    }

    #[test]
    fn test_health_factor_unhealthy() {
        let collateral = Decimal::from(1_000i64);
        let debt = Decimal::from(1_000i64);
        let threshold = Decimal::from(8i64).checked_div(Decimal::from(10i64)).unwrap();

        let weighted = collateral.checked_mul(threshold).unwrap();
        let hf = weighted.checked_div(debt).unwrap();

        // HF = (1000 * 0.8) / 1000 = 0.8
        let expected = Decimal::from(8i64).checked_div(Decimal::from(10i64)).unwrap();
        assert_eq!(hf, expected);
        assert!(hf < Decimal::ONE); // Position is liquidatable
    }

    #[test]
    fn test_liquidation_price_computation() {
        // Liquidation price = debt / (amount * threshold)
        let debt = Decimal::from(8_000i64);
        let amount = Decimal::from(10i64);
        let threshold = Decimal::from(8i64).checked_div(Decimal::from(10i64)).unwrap();

        let denominator = amount.checked_mul(threshold).unwrap();
        let liq_price = debt.checked_div(denominator).unwrap();

        // LP = 8000 / (10 * 0.8) = 1000
        assert_eq!(liq_price, Decimal::from(1_000i64));
    }

    #[test]
    fn test_max_borrow_computation() {
        // Max borrow = (collateral * threshold) / target_hf
        let collateral = Decimal::from(10_000i64);
        let threshold = Decimal::from(8i64).checked_div(Decimal::from(10i64)).unwrap();
        let target_hf = Decimal::from(15i64).checked_div(Decimal::from(10i64)).unwrap(); // 1.5

        let weighted = collateral.checked_mul(threshold).unwrap();
        let max_borrow = weighted.checked_div(target_hf).unwrap();

        // Max = (10000 * 0.8) / 1.5 = 5333.33...
        let expected_approx = Decimal::from(5333i64);
        let diff = (max_borrow - expected_approx).abs();
        assert!(diff < Decimal::ONE);
    }

    #[test]
    fn test_liquidation_amounts_computation() {
        // Collateral to receive = (debt / price) * (1 + bonus)
        let debt = Decimal::from(1_000i64);
        let price = Decimal::from(2_000i64);
        let bonus = Decimal::from(5i64).checked_div(Decimal::from(100i64)).unwrap(); // 5%

        let one_plus_bonus = Decimal::ONE.checked_add(bonus).unwrap();
        let base_collateral = debt.checked_div(price).unwrap();
        let total_collateral = base_collateral.checked_mul(one_plus_bonus).unwrap();

        // Collateral = (1000 / 2000) * 1.05 = 0.525
        let expected = Decimal::from(525i64).checked_div(Decimal::from(1_000i64)).unwrap();
        assert_eq!(total_collateral, expected);
    }

    #[test]
    fn test_precision_core_decimal_ops() {
        // Verify precision-core Decimal works correctly for DeFi precision
        let large = Decimal::from(1_000_000_000_000i64);
        let small = Decimal::from(1i64).checked_div(Decimal::from(1_000_000i64)).unwrap();

        let product = large.checked_mul(small).unwrap();
        assert_eq!(product, Decimal::from(1_000_000i64));
    }

    #[test]
    fn test_basis_points_conversion() {
        // 8000 bps = 80%
        let bps = Decimal::from(8000i64);
        let percentage = bps.checked_div(Decimal::from(10_000i64)).unwrap();

        let expected = Decimal::from(8i64).checked_div(Decimal::from(10i64)).unwrap();
        assert_eq!(percentage, expected);
    }
}
