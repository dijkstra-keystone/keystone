//! Oracle decimal conversion utilities.
//!
//! Different oracle providers use different decimal precisions:
//! - Chainlink: 8 decimals for most feeds, 18 for some
//! - Pyth: Variable precision (exponent-based)
//! - RedStone: 8 decimals
//! - Band Protocol: 18 decimals
//!
//! This module provides utilities for normalizing and converting between
//! different oracle decimal formats.

use crate::{ArithmeticError, Decimal, RoundingMode};

/// Standard oracle decimal formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleDecimals {
    /// USDC, USDT on many chains (6 decimals)
    Six,
    /// Chainlink default (8 decimals)
    Eight,
    /// ETH, most ERC-20 tokens (18 decimals)
    Eighteen,
    /// Custom decimal count
    Custom(u8),
}

impl OracleDecimals {
    /// Get the decimal count.
    pub const fn value(self) -> u8 {
        match self {
            Self::Six => 6,
            Self::Eight => 8,
            Self::Eighteen => 18,
            Self::Custom(n) => n,
        }
    }

    /// Get the scale factor (10^decimals).
    pub fn scale_factor(self) -> Decimal {
        let decimals = self.value();
        Decimal::from(10i64)
            .powi(decimals as i32)
            .unwrap_or(Decimal::MAX)
    }
}

impl From<u8> for OracleDecimals {
    fn from(n: u8) -> Self {
        match n {
            6 => Self::Six,
            8 => Self::Eight,
            18 => Self::Eighteen,
            _ => Self::Custom(n),
        }
    }
}

/// Normalize a raw oracle value to a Decimal.
///
/// Takes a raw integer value from an oracle and converts it to a Decimal
/// using the specified decimal precision.
///
/// # Example
///
/// ```
/// use precision_core::oracle::{normalize_oracle_price, OracleDecimals};
///
/// // Chainlink ETH/USD price: $2500.12345678 (8 decimals)
/// let raw_price = 250012345678i64;
/// let price = normalize_oracle_price(raw_price, OracleDecimals::Eight).unwrap();
/// assert_eq!(price.to_string(), "2500.12345678");
/// ```
pub fn normalize_oracle_price(
    raw_value: i64,
    decimals: OracleDecimals,
) -> Result<Decimal, ArithmeticError> {
    let scale = decimals.scale_factor();
    Decimal::from(raw_value)
        .checked_div(scale)
        .ok_or(ArithmeticError::DivisionByZero)
}

/// Normalize a large raw oracle value to a Decimal.
///
/// Similar to [`normalize_oracle_price`] but accepts i128 for values
/// that exceed i64 range (common with 18-decimal token amounts).
pub fn normalize_oracle_price_i128(
    raw_value: i128,
    decimals: OracleDecimals,
) -> Result<Decimal, ArithmeticError> {
    let scale = decimals.scale_factor();
    Decimal::try_from_i128(raw_value)?
        .checked_div(scale)
        .ok_or(ArithmeticError::DivisionByZero)
}

/// Convert a Decimal to a raw oracle integer value.
///
/// Converts a Decimal to the raw integer format expected by an oracle
/// with the specified decimal precision.
///
/// # Example
///
/// ```
/// use precision_core::oracle::{denormalize_oracle_price, OracleDecimals};
/// use precision_core::Decimal;
/// use core::str::FromStr;
///
/// let price = Decimal::from_str("2500.12345678").unwrap();
/// let raw = denormalize_oracle_price(price, OracleDecimals::Eight).unwrap();
/// assert_eq!(raw, 250012345678);
/// ```
pub fn denormalize_oracle_price(
    value: Decimal,
    decimals: OracleDecimals,
) -> Result<i64, ArithmeticError> {
    let scale = decimals.scale_factor();
    let scaled = value
        .checked_mul(scale)
        .ok_or(ArithmeticError::Overflow)?
        .round(0, RoundingMode::TowardZero);
    let (mantissa, _) = scaled.to_parts();
    i64::try_from(mantissa).map_err(|_| ArithmeticError::Overflow)
}

/// Convert a Decimal to a raw oracle i128 value.
///
/// Similar to [`denormalize_oracle_price`] but returns i128 for values
/// that exceed i64 range.
pub fn denormalize_oracle_price_i128(
    value: Decimal,
    decimals: OracleDecimals,
) -> Result<i128, ArithmeticError> {
    let scale = decimals.scale_factor();
    let scaled = value
        .checked_mul(scale)
        .ok_or(ArithmeticError::Overflow)?
        .round(0, RoundingMode::TowardZero);
    let (mantissa, _) = scaled.to_parts();
    Ok(mantissa)
}

/// Convert a price between two different decimal precisions.
///
/// # Example
///
/// ```
/// use precision_core::oracle::{convert_decimals, OracleDecimals};
///
/// // Convert from 8 decimals (Chainlink) to 6 decimals (USDC)
/// let chainlink_price = 250012345678i64;  // $2500.12345678
/// let usdc_price = convert_decimals(
///     chainlink_price,
///     OracleDecimals::Eight,
///     OracleDecimals::Six
/// ).unwrap();
/// assert_eq!(usdc_price, 2500123456);  // $2500.123456
/// ```
pub fn convert_decimals(
    value: i64,
    from: OracleDecimals,
    to: OracleDecimals,
) -> Result<i64, ArithmeticError> {
    let from_decimals = from.value() as i32;
    let to_decimals = to.value() as i32;
    let diff = to_decimals - from_decimals;

    if diff == 0 {
        return Ok(value);
    }

    let factor = 10i64
        .checked_pow(diff.unsigned_abs())
        .ok_or(ArithmeticError::Overflow)?;

    if diff > 0 {
        value.checked_mul(factor).ok_or(ArithmeticError::Overflow)
    } else {
        Ok(value / factor)
    }
}

/// Convert a price between decimal precisions, returning i128.
///
/// Use this when converting to higher decimals where the result may exceed i64.
///
/// # Example
///
/// ```
/// use precision_core::oracle::{convert_decimals_i128, OracleDecimals};
///
/// // Convert from 8 decimals (Chainlink) to 18 decimals (on-chain)
/// let chainlink_price = 250012345678i64;  // $2500.12345678
/// let onchain_price = convert_decimals_i128(
///     chainlink_price,
///     OracleDecimals::Eight,
///     OracleDecimals::Eighteen
/// ).unwrap();
/// assert_eq!(onchain_price, 2500123456780000000000i128);
/// ```
pub fn convert_decimals_i128(
    value: i64,
    from: OracleDecimals,
    to: OracleDecimals,
) -> Result<i128, ArithmeticError> {
    let from_decimals = from.value() as i32;
    let to_decimals = to.value() as i32;
    let diff = to_decimals - from_decimals;

    if diff == 0 {
        return Ok(value as i128);
    }

    let factor = 10i128
        .checked_pow(diff.unsigned_abs())
        .ok_or(ArithmeticError::Overflow)?;

    if diff > 0 {
        (value as i128)
            .checked_mul(factor)
            .ok_or(ArithmeticError::Overflow)
    } else {
        Ok((value as i128) / factor)
    }
}

/// Scale a token amount between different decimal precisions.
///
/// Useful for converting between tokens with different decimals
/// (e.g., USDC with 6 decimals to DAI with 18 decimals).
///
/// # Example
///
/// ```
/// use precision_core::oracle::{scale_token_amount, OracleDecimals};
///
/// // Convert 1000 USDC (6 decimals) representation to 8 decimals
/// let usdc_amount = 1_000_000_000i64;  // 1000 USDC (6 decimals)
/// let scaled = scale_token_amount(
///     usdc_amount,
///     OracleDecimals::Six,
///     OracleDecimals::Eight
/// ).unwrap();
/// assert_eq!(scaled, 100_000_000_000);  // 1000 * 10^8
/// ```
pub fn scale_token_amount(
    amount: i64,
    from_decimals: OracleDecimals,
    to_decimals: OracleDecimals,
) -> Result<i64, ArithmeticError> {
    convert_decimals(amount, from_decimals, to_decimals)
}

/// Scale a token amount using i128 for large values.
///
/// # Example
///
/// ```
/// use precision_core::oracle::{scale_token_amount_i128, OracleDecimals};
///
/// // Convert 1000 USDC (6 decimals) to 18 decimal representation
/// let usdc_amount = 1_000_000_000i64;  // 1000 USDC
/// let scaled = scale_token_amount_i128(
///     usdc_amount,
///     OracleDecimals::Six,
///     OracleDecimals::Eighteen
/// ).unwrap();
/// assert_eq!(scaled, 1_000_000_000_000_000_000_000i128);  // 1000 * 10^18
/// ```
pub fn scale_token_amount_i128(
    amount: i64,
    from_decimals: OracleDecimals,
    to_decimals: OracleDecimals,
) -> Result<i128, ArithmeticError> {
    convert_decimals_i128(amount, from_decimals, to_decimals)
}

/// Calculate the value of tokens in a quote currency.
///
/// Computes: amount * price, handling decimal conversions.
/// Uses Decimal internally for precision, returns result in specified decimals.
///
/// # Arguments
///
/// * `amount` - Token amount in its native decimals
/// * `amount_decimals` - Decimal precision of the token
/// * `price` - Price per token in quote currency
/// * `price_decimals` - Decimal precision of the price feed
/// * `result_decimals` - Desired decimal precision for the result
///
/// # Example
///
/// ```
/// use precision_core::oracle::{calculate_value, OracleDecimals};
///
/// // Calculate value of 1000 USDC at $1.00 per USDC
/// let usdc_amount = 1_000_000_000i64;  // 1000 USDC (6 decimals)
/// let usdc_price = 100000000i64;  // $1.00 (8 decimals from Chainlink)
///
/// let value = calculate_value(
///     usdc_amount,
///     OracleDecimals::Six,
///     usdc_price,
///     OracleDecimals::Eight,
///     OracleDecimals::Six  // Result in 6 decimals
/// ).unwrap();
///
/// assert_eq!(value, 1_000_000_000);  // $1000 in 6 decimals
/// ```
pub fn calculate_value(
    amount: i64,
    amount_decimals: OracleDecimals,
    price: i64,
    price_decimals: OracleDecimals,
    result_decimals: OracleDecimals,
) -> Result<i64, ArithmeticError> {
    let amount_dec = normalize_oracle_price(amount, amount_decimals)?;
    let price_dec = normalize_oracle_price(price, price_decimals)?;

    let value = amount_dec
        .checked_mul(price_dec)
        .ok_or(ArithmeticError::Overflow)?;

    denormalize_oracle_price(value, result_decimals)
}

/// Calculate value and return as i128 for large results.
pub fn calculate_value_i128(
    amount: i64,
    amount_decimals: OracleDecimals,
    price: i64,
    price_decimals: OracleDecimals,
    result_decimals: OracleDecimals,
) -> Result<i128, ArithmeticError> {
    let amount_dec = normalize_oracle_price(amount, amount_decimals)?;
    let price_dec = normalize_oracle_price(price, price_decimals)?;

    let value = amount_dec
        .checked_mul(price_dec)
        .ok_or(ArithmeticError::Overflow)?;

    denormalize_oracle_price_i128(value, result_decimals)
}

/// Normalize a Pyth-style price with exponent.
///
/// Pyth prices are returned as (price, exponent) where the actual price
/// is price * 10^exponent.
///
/// # Example
///
/// ```
/// use precision_core::oracle::normalize_pyth_price;
///
/// // Pyth ETH/USD price: 250012345678 with exponent -8
/// let price = 250012345678i64;
/// let exponent = -8i32;
/// let normalized = normalize_pyth_price(price, exponent).unwrap();
/// assert_eq!(normalized.to_string(), "2500.12345678");
/// ```
pub fn normalize_pyth_price(price: i64, exponent: i32) -> Result<Decimal, ArithmeticError> {
    let price_dec = Decimal::from(price);

    if exponent == 0 {
        return Ok(price_dec);
    }

    let scale = Decimal::from(10i64)
        .powi(exponent.abs())
        .ok_or(ArithmeticError::Overflow)?;

    if exponent > 0 {
        price_dec
            .checked_mul(scale)
            .ok_or(ArithmeticError::Overflow)
    } else {
        price_dec
            .checked_div(scale)
            .ok_or(ArithmeticError::DivisionByZero)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::string::ToString;
    use core::str::FromStr;

    #[test]
    fn test_normalize_chainlink_price() {
        let raw = 250012345678i64;
        let price = normalize_oracle_price(raw, OracleDecimals::Eight).unwrap();
        assert_eq!(price.to_string(), "2500.12345678");
    }

    #[test]
    fn test_denormalize_price() {
        let price = Decimal::from_str("2500.12345678").unwrap();
        let raw = denormalize_oracle_price(price, OracleDecimals::Eight).unwrap();
        assert_eq!(raw, 250012345678);
    }

    #[test]
    fn test_convert_8_to_6_decimals() {
        let chainlink = 250012345678i64;
        let usdc = convert_decimals(chainlink, OracleDecimals::Eight, OracleDecimals::Six).unwrap();
        assert_eq!(usdc, 2500123456);
    }

    #[test]
    fn test_convert_8_to_18_decimals_i128() {
        let chainlink = 250012345678i64;
        let onchain =
            convert_decimals_i128(chainlink, OracleDecimals::Eight, OracleDecimals::Eighteen)
                .unwrap();
        assert_eq!(onchain, 2500123456780000000000i128);
    }

    #[test]
    fn test_convert_18_to_8_decimals_via_normalize() {
        // Test round-trip: normalize a Chainlink price, then convert back
        let original = 250012345678i64;
        let normalized = normalize_oracle_price(original, OracleDecimals::Eight).unwrap();
        let recovered = denormalize_oracle_price(normalized, OracleDecimals::Eight).unwrap();
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_scale_usdc_to_8_decimals() {
        let usdc = 1_000_000_000i64; // 1000 USDC (6 decimals)
        let scaled = scale_token_amount(usdc, OracleDecimals::Six, OracleDecimals::Eight).unwrap();
        assert_eq!(scaled, 100_000_000_000);
    }

    #[test]
    fn test_scale_usdc_to_18_decimals_i128() {
        let usdc = 1_000_000_000i64; // 1000 USDC
        let scaled =
            scale_token_amount_i128(usdc, OracleDecimals::Six, OracleDecimals::Eighteen).unwrap();
        assert_eq!(scaled, 1_000_000_000_000_000_000_000i128);
    }

    #[test]
    fn test_pyth_positive_exponent() {
        let price = normalize_pyth_price(25, 2).unwrap();
        assert_eq!(price.to_string(), "2500");
    }

    #[test]
    fn test_pyth_negative_exponent() {
        let price = normalize_pyth_price(250012345678, -8).unwrap();
        assert_eq!(price.to_string(), "2500.12345678");
    }

    #[test]
    fn test_pyth_zero_exponent() {
        let price = normalize_pyth_price(2500, 0).unwrap();
        assert_eq!(price.to_string(), "2500");
    }

    #[test]
    fn test_calculate_usdc_value() {
        // Calculate value of 1000 USDC at $1.00 per USDC
        let usdc_amount = 1_000_000_000i64; // 1000 USDC (6 decimals)
        let usdc_price = 100000000i64; // $1.00 (8 decimals from Chainlink)

        let value = calculate_value(
            usdc_amount,
            OracleDecimals::Six,
            usdc_price,
            OracleDecimals::Eight,
            OracleDecimals::Six,
        )
        .unwrap();

        assert_eq!(value, 1_000_000_000); // $1000 in 6 decimals
    }

    #[test]
    fn test_calculate_btc_value() {
        // Calculate value of 0.1 BTC at $50000 per BTC
        // Using 8 decimal representation for BTC amount
        let btc_amount = 10_000_000i64; // 0.1 BTC (8 decimals)
        let btc_price = 5000000000000i64; // $50000 (8 decimals)

        let value = calculate_value(
            btc_amount,
            OracleDecimals::Eight,
            btc_price,
            OracleDecimals::Eight,
            OracleDecimals::Six,
        )
        .unwrap();

        assert_eq!(value, 5_000_000_000); // $5000 in 6 decimals
    }

    #[test]
    fn test_oracle_decimals_from_u8() {
        assert_eq!(OracleDecimals::from(6), OracleDecimals::Six);
        assert_eq!(OracleDecimals::from(8), OracleDecimals::Eight);
        assert_eq!(OracleDecimals::from(18), OracleDecimals::Eighteen);
        assert_eq!(OracleDecimals::from(12), OracleDecimals::Custom(12));
    }
}
