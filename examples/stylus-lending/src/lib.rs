//! Stylus Lending Example
//!
//! Demonstrates using Keystone precision arithmetic in a Stylus smart contract
//! for DeFi lending calculations with deterministic results.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::vec::Vec;
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
