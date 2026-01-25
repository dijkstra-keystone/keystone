//! Core decimal type implementation.

use crate::error::{ArithmeticError, ParseError};
use crate::rounding::RoundingMode;
use core::cmp::Ordering;
use core::fmt;
use core::ops::{Add, Div, Mul, Neg, Sub};
use core::str::FromStr;
use num_traits::Signed;
use rust_decimal::prelude::MathematicalOps;
use rust_decimal::Decimal as RustDecimal;
use serde::{Deserialize, Serialize};

/// Maximum scale (decimal places) supported.
pub const MAX_SCALE: u32 = 28;

/// A 128-bit decimal number with deterministic arithmetic.
///
/// This type wraps `rust_decimal::Decimal` and provides checked arithmetic
/// operations that explicitly handle overflow, underflow, and division by zero.
/// All operations are deterministic across platforms.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Decimal(RustDecimal);

impl Decimal {
    /// Zero.
    pub const ZERO: Self = Self(RustDecimal::ZERO);

    /// One.
    pub const ONE: Self = Self(RustDecimal::ONE);

    /// Negative one.
    pub const NEGATIVE_ONE: Self = Self(RustDecimal::NEGATIVE_ONE);

    /// Ten.
    pub const TEN: Self = Self(RustDecimal::TEN);

    /// One hundred.
    pub const ONE_HUNDRED: Self = Self(RustDecimal::ONE_HUNDRED);

    /// One thousand.
    pub const ONE_THOUSAND: Self = Self(RustDecimal::ONE_THOUSAND);

    /// Maximum representable value.
    pub const MAX: Self = Self(RustDecimal::MAX);

    /// Minimum representable value.
    pub const MIN: Self = Self(RustDecimal::MIN);

    /// Creates a new decimal from integer mantissa and scale.
    ///
    /// The value is `mantissa * 10^(-scale)`.
    ///
    /// # Panics
    ///
    /// Panics if scale exceeds 28.
    #[must_use]
    pub fn new(mantissa: i64, scale: u32) -> Self {
        Self(RustDecimal::new(mantissa, scale))
    }

    /// Creates a decimal from raw parts.
    ///
    /// The 96-bit mantissa is stored as three 32-bit words in little-endian order.
    /// The sign is `true` for negative values.
    #[must_use]
    pub const fn from_parts(lo: u32, mid: u32, hi: u32, negative: bool, scale: u32) -> Self {
        Self(RustDecimal::from_parts(lo, mid, hi, negative, scale))
    }

    /// Creates a decimal from a 128-bit integer.
    ///
    /// Returns an error if the value is too large to represent.
    pub fn try_from_i128(value: i128) -> Result<Self, ArithmeticError> {
        RustDecimal::try_from_i128_with_scale(value, 0)
            .map(Self)
            .map_err(|_| ArithmeticError::Overflow)
    }

    /// Returns the mantissa as a 128-bit integer and the scale.
    #[must_use]
    pub fn to_parts(self) -> (i128, u32) {
        let unpacked = self.0.unpack();
        let mantissa = i128::from(unpacked.lo)
            | (i128::from(unpacked.mid) << 32)
            | (i128::from(unpacked.hi) << 64);
        let signed = if unpacked.negative {
            -mantissa
        } else {
            mantissa
        };
        (signed, unpacked.scale)
    }

    /// Returns the scale (number of decimal places).
    #[must_use]
    pub fn scale(self) -> u32 {
        self.0.scale()
    }

    /// Returns `true` if the value is zero.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0.is_zero()
    }

    /// Returns `true` if the value is negative.
    #[must_use]
    pub fn is_negative(self) -> bool {
        self.0.is_sign_negative()
    }

    /// Returns `true` if the value is positive.
    #[must_use]
    pub fn is_positive(self) -> bool {
        self.0.is_sign_positive() && !self.0.is_zero()
    }

    /// Returns the absolute value.
    #[must_use]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Returns the sign of the value: -1, 0, or 1.
    #[must_use]
    pub fn signum(self) -> Self {
        Self(self.0.signum())
    }

    /// Checked addition. Returns `None` on overflow.
    #[must_use]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Checked subtraction. Returns `None` on overflow.
    #[must_use]
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    /// Checked multiplication. Returns `None` on overflow.
    #[must_use]
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        self.0.checked_mul(other.0).map(Self)
    }

    /// Checked division. Returns `None` on division by zero or overflow.
    #[must_use]
    pub fn checked_div(self, other: Self) -> Option<Self> {
        self.0.checked_div(other.0).map(Self)
    }

    /// Checked remainder. Returns `None` on division by zero.
    #[must_use]
    pub fn checked_rem(self, other: Self) -> Option<Self> {
        self.0.checked_rem(other.0).map(Self)
    }

    /// Saturating addition. Returns `MAX` or `MIN` on overflow.
    #[must_use]
    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Saturating subtraction. Returns `MAX` or `MIN` on overflow.
    #[must_use]
    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Saturating multiplication. Returns `MAX` or `MIN` on overflow.
    #[must_use]
    pub fn saturating_mul(self, other: Self) -> Self {
        Self(self.0.saturating_mul(other.0))
    }

    /// Addition with explicit error on overflow.
    pub fn try_add(self, other: Self) -> Result<Self, ArithmeticError> {
        self.checked_add(other).ok_or(ArithmeticError::Overflow)
    }

    /// Subtraction with explicit error on overflow.
    pub fn try_sub(self, other: Self) -> Result<Self, ArithmeticError> {
        self.checked_sub(other).ok_or(ArithmeticError::Overflow)
    }

    /// Multiplication with explicit error on overflow.
    pub fn try_mul(self, other: Self) -> Result<Self, ArithmeticError> {
        self.checked_mul(other).ok_or(ArithmeticError::Overflow)
    }

    /// Division with explicit error handling.
    pub fn try_div(self, other: Self) -> Result<Self, ArithmeticError> {
        if other.is_zero() {
            return Err(ArithmeticError::DivisionByZero);
        }
        self.checked_div(other).ok_or(ArithmeticError::Overflow)
    }

    /// Rounds to the specified number of decimal places using the given mode.
    #[must_use]
    pub fn round(self, dp: u32, mode: RoundingMode) -> Self {
        Self(self.0.round_dp_with_strategy(dp, mode.to_rust_decimal()))
    }

    /// Rounds to the specified number of decimal places using banker's rounding.
    #[must_use]
    pub fn round_dp(self, dp: u32) -> Self {
        self.round(dp, RoundingMode::HalfEven)
    }

    /// Truncates to the specified number of decimal places.
    #[must_use]
    pub fn trunc(self, dp: u32) -> Self {
        self.round(dp, RoundingMode::TowardZero)
    }

    /// Returns the floor (round toward negative infinity).
    #[must_use]
    pub fn floor(self) -> Self {
        Self(self.0.floor())
    }

    /// Returns the ceiling (round toward positive infinity).
    #[must_use]
    pub fn ceil(self) -> Self {
        Self(self.0.ceil())
    }

    /// Normalizes the scale by removing trailing zeros.
    #[must_use]
    pub fn normalize(self) -> Self {
        Self(self.0.normalize())
    }

    /// Rescales to the specified number of decimal places.
    ///
    /// Returns an error if the scale would exceed `MAX_SCALE`.
    pub fn rescale(&mut self, scale: u32) -> Result<(), ArithmeticError> {
        if scale > MAX_SCALE {
            return Err(ArithmeticError::ScaleExceeded);
        }
        self.0.rescale(scale);
        Ok(())
    }

    /// Returns the minimum of two values.
    #[must_use]
    pub fn min(self, other: Self) -> Self {
        if self <= other {
            self
        } else {
            other
        }
    }

    /// Returns the maximum of two values.
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        if self >= other {
            self
        } else {
            other
        }
    }

    /// Clamps the value to the specified range.
    #[must_use]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }

    /// Returns the internal representation for interop.
    #[must_use]
    pub fn into_inner(self) -> RustDecimal {
        self.0
    }

    /// Creates from the internal representation.
    #[must_use]
    pub fn from_inner(inner: RustDecimal) -> Self {
        Self(inner)
    }

    // ========================================================================
    // Transcendental Functions (for Black-Scholes and advanced financial math)
    // ========================================================================

    /// Computes the square root.
    ///
    /// Returns `None` if the value is negative.
    ///
    /// # Example
    ///
    /// ```
    /// use precision_core::Decimal;
    ///
    /// let x = Decimal::from(4i64);
    /// assert_eq!(x.sqrt(), Some(Decimal::from(2i64)));
    ///
    /// let neg = Decimal::from(-1i64);
    /// assert_eq!(neg.sqrt(), None);
    /// ```
    #[must_use]
    pub fn sqrt(self) -> Option<Self> {
        if self.is_negative() {
            return None;
        }
        self.0.sqrt().map(Self)
    }

    /// Computes the square root, returning an error for negative inputs.
    pub fn try_sqrt(self) -> Result<Self, ArithmeticError> {
        if self.is_negative() {
            return Err(ArithmeticError::NegativeSqrt);
        }
        self.sqrt().ok_or(ArithmeticError::Overflow)
    }

    /// Computes e^self (the exponential function).
    ///
    /// Returns `None` on overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use precision_core::Decimal;
    ///
    /// let x = Decimal::ZERO;
    /// assert_eq!(x.exp(), Some(Decimal::ONE));
    /// ```
    #[must_use]
    pub fn exp(self) -> Option<Self> {
        // rust_decimal's exp() can overflow, so we need to catch panics
        // or check bounds. For safety, we use checked_exp if available.
        // Since rust_decimal 1.x exp() returns Decimal directly, we wrap in Option.

        // Check for extreme values that would overflow
        // e^710 is approximately the max for f64, our Decimal has similar limits
        if self > Self::from(100i64) {
            return None; // Would overflow
        }
        if self < Self::from(-100i64) {
            return Some(Self::ZERO); // Underflows to effectively zero
        }

        Some(Self(self.0.exp()))
    }

    /// Computes e^self, returning an error on overflow.
    pub fn try_exp(self) -> Result<Self, ArithmeticError> {
        self.exp().ok_or(ArithmeticError::Overflow)
    }

    /// Computes the natural logarithm (ln).
    ///
    /// Returns `None` if the value is not positive.
    ///
    /// # Example
    ///
    /// ```
    /// use precision_core::Decimal;
    /// use core::str::FromStr;
    ///
    /// let e = Decimal::from_str("2.718281828459045").unwrap();
    /// let ln_e = e.ln().unwrap();
    /// // ln(e) ≈ 1
    /// assert!((ln_e - Decimal::ONE).abs() < Decimal::from_str("0.0001").unwrap());
    /// ```
    #[must_use]
    pub fn ln(self) -> Option<Self> {
        if !self.is_positive() {
            return None;
        }
        Some(Self(self.0.ln()))
    }

    /// Computes the natural logarithm, returning an error for non-positive inputs.
    pub fn try_ln(self) -> Result<Self, ArithmeticError> {
        if self.is_zero() {
            return Err(ArithmeticError::LogOfZero);
        }
        if self.is_negative() {
            return Err(ArithmeticError::LogOfNegative);
        }
        self.ln().ok_or(ArithmeticError::Overflow)
    }

    /// Computes the base-10 logarithm.
    ///
    /// Returns `None` if the value is not positive.
    #[must_use]
    pub fn log10(self) -> Option<Self> {
        if !self.is_positive() {
            return None;
        }
        Some(Self(self.0.log10()))
    }

    /// Computes self^exponent using the formula: x^y = e^(y * ln(x)).
    ///
    /// Returns `None` if the computation would fail (e.g., negative base with
    /// non-integer exponent, or overflow).
    ///
    /// Note: For integer exponents, use [`powi`](Self::powi) for exact results.
    ///
    /// # Example
    ///
    /// ```
    /// use precision_core::Decimal;
    /// use core::str::FromStr;
    ///
    /// let base = Decimal::from(2i64);
    /// let exp = Decimal::from(3i64);
    /// let result = base.pow(exp).unwrap();
    /// // Note: small precision loss due to exp/ln computation
    /// let diff = (result - Decimal::from(8i64)).abs();
    /// assert!(diff < Decimal::from_str("0.001").unwrap());
    /// ```
    #[must_use]
    pub fn pow(self, exponent: Self) -> Option<Self> {
        // Special cases
        if exponent.is_zero() {
            return Some(Self::ONE);
        }
        if self.is_zero() {
            return if exponent.is_positive() {
                Some(Self::ZERO)
            } else {
                None // 0^negative is undefined
            };
        }
        if self == Self::ONE {
            return Some(Self::ONE);
        }

        // For negative bases, only integer exponents are supported
        if self.is_negative() {
            // Check if exponent is an integer
            if exponent.floor() != exponent {
                return None; // Complex result
            }
            let abs_base = self.abs();
            let result = abs_base.ln()?.checked_mul(exponent)?;
            let exp_result = result.exp()?;

            // Determine sign based on whether exponent is odd
            let exp_int = exponent.floor();
            let is_odd = (exp_int / Self::from(2i64)).floor() * Self::from(2i64) != exp_int;

            return Some(if is_odd { -exp_result } else { exp_result });
        }

        // x^y = e^(y * ln(x))
        let ln_x = self.ln()?;
        let product = ln_x.checked_mul(exponent)?;
        product.exp()
    }

    /// Computes self^exponent, returning an error on failure.
    pub fn try_pow(self, exponent: Self) -> Result<Self, ArithmeticError> {
        self.pow(exponent).ok_or(ArithmeticError::Overflow)
    }

    /// Computes self^n for integer exponent using repeated squaring.
    ///
    /// This is more accurate than `pow()` for integer exponents as it avoids
    /// the exp/ln computation. Returns `None` on overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use precision_core::Decimal;
    ///
    /// let base = Decimal::from(2i64);
    /// assert_eq!(base.powi(10), Some(Decimal::from(1024i64)));
    /// ```
    #[must_use]
    pub fn powi(self, n: i32) -> Option<Self> {
        if n == 0 {
            return Some(Self::ONE);
        }

        let (mut base, mut exp) = if n < 0 {
            (Self::ONE.checked_div(self)?, (-n) as u32)
        } else {
            (self, n as u32)
        };

        let mut result = Self::ONE;

        while exp > 0 {
            if exp & 1 == 1 {
                result = result.checked_mul(base)?;
            }
            base = base.checked_mul(base)?;
            exp >>= 1;
        }

        Some(result)
    }

    /// Computes self^n for integer exponent, returning error on failure.
    pub fn try_powi(self, n: i32) -> Result<Self, ArithmeticError> {
        if n < 0 && self.is_zero() {
            return Err(ArithmeticError::DivisionByZero);
        }
        self.powi(n).ok_or(ArithmeticError::Overflow)
    }

    /// Euler's number e ≈ 2.718281828459045.
    pub fn e() -> Self {
        Self::from_str("2.7182818284590452353602874713527")
            .expect("E constant is valid")
    }

    /// Pi ≈ 3.141592653589793.
    pub fn pi() -> Self {
        Self::from_str("3.1415926535897932384626433832795")
            .expect("PI constant is valid")
    }
}

impl Default for Decimal {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Decimal({})", self.0)
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for Decimal {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::Empty);
        }
        RustDecimal::from_str(s)
            .map(Self)
            .map_err(|_| ParseError::InvalidCharacter)
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Neg for Decimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        self.checked_add(other).expect("decimal overflow")
    }
}

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        self.checked_sub(other).expect("decimal overflow")
    }
}

impl Mul for Decimal {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        self.checked_mul(other).expect("decimal overflow")
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        self.checked_div(other).expect("decimal division error")
    }
}

macro_rules! impl_from_int {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Decimal {
                fn from(n: $t) -> Self {
                    Self(RustDecimal::from(n))
                }
            }
        )*
    };
}

impl_from_int!(i8, i16, i32, i64, u8, u16, u32, u64);

impl From<i128> for Decimal {
    fn from(n: i128) -> Self {
        Self(RustDecimal::from(n))
    }
}

impl From<u128> for Decimal {
    fn from(n: u128) -> Self {
        Self(RustDecimal::from(n))
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn zero_identity() {
        let a = Decimal::from(42i64);
        assert_eq!(a + Decimal::ZERO, a);
        assert_eq!(a - Decimal::ZERO, a);
        assert_eq!(a * Decimal::ZERO, Decimal::ZERO);
    }

    #[test]
    fn one_identity() {
        let a = Decimal::from(42i64);
        assert_eq!(a * Decimal::ONE, a);
        assert_eq!(a / Decimal::ONE, a);
    }

    #[test]
    fn negation() {
        let a = Decimal::from(42i64);
        assert_eq!(-(-a), a);
        assert_eq!(a + (-a), Decimal::ZERO);
    }

    #[test]
    fn basic_arithmetic() {
        let a = Decimal::new(100, 2);
        let b = Decimal::new(200, 2);
        assert_eq!(a + b, Decimal::new(300, 2));
        assert_eq!(b - a, Decimal::new(100, 2));
        assert_eq!(a * Decimal::from(2i64), b);
        assert_eq!(b / Decimal::from(2i64), a);
    }

    #[test]
    fn division_precision() {
        let a = Decimal::from(1i64);
        let b = Decimal::from(3i64);
        let result = a / b;
        assert_eq!(result.round_dp(6), Decimal::from_str("0.333333").unwrap());
    }

    #[test]
    fn rounding_modes() {
        let a = Decimal::from_str("2.5").unwrap();
        assert_eq!(a.round(0, RoundingMode::HalfEven), Decimal::from(2i64));
        assert_eq!(a.round(0, RoundingMode::HalfUp), Decimal::from(3i64));
        assert_eq!(a.round(0, RoundingMode::Down), Decimal::from(2i64));
        assert_eq!(a.round(0, RoundingMode::Up), Decimal::from(3i64));

        let b = Decimal::from_str("3.5").unwrap();
        assert_eq!(b.round(0, RoundingMode::HalfEven), Decimal::from(4i64));
    }

    #[test]
    fn checked_operations() {
        assert!(Decimal::MAX.checked_add(Decimal::ONE).is_none());
        assert!(Decimal::MIN.checked_sub(Decimal::ONE).is_none());
        assert!(Decimal::ZERO.checked_div(Decimal::ZERO).is_none());
    }

    #[test]
    fn try_operations() {
        assert!(matches!(
            Decimal::MAX.try_add(Decimal::ONE),
            Err(ArithmeticError::Overflow)
        ));
        assert!(matches!(
            Decimal::ONE.try_div(Decimal::ZERO),
            Err(ArithmeticError::DivisionByZero)
        ));
    }

    #[test]
    fn parse_and_display() {
        let a: Decimal = "123.456".parse().unwrap();
        assert_eq!(a.to_string(), "123.456");

        let b: Decimal = "-0.001".parse().unwrap();
        assert_eq!(b.to_string(), "-0.001");
    }

    #[test]
    fn ordering() {
        let a = Decimal::from(1i64);
        let b = Decimal::from(2i64);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a.min(b), a);
        assert_eq!(a.max(b), b);
    }

    #[test]
    fn abs_and_signum() {
        let pos = Decimal::from(5i64);
        let neg = Decimal::from(-5i64);

        assert_eq!(pos.abs(), pos);
        assert_eq!(neg.abs(), pos);
        assert_eq!(pos.signum(), Decimal::ONE);
        assert_eq!(neg.signum(), Decimal::NEGATIVE_ONE);
        assert_eq!(Decimal::ZERO.signum(), Decimal::ZERO);
    }

    #[test]
    fn clamp() {
        let min = Decimal::from(0i64);
        let max = Decimal::from(100i64);

        assert_eq!(Decimal::from(50i64).clamp(min, max), Decimal::from(50i64));
        assert_eq!(Decimal::from(-10i64).clamp(min, max), min);
        assert_eq!(Decimal::from(150i64).clamp(min, max), max);
    }

    // ========================================================================
    // Transcendental Function Tests
    // ========================================================================

    #[test]
    fn sqrt_perfect_squares() {
        assert_eq!(Decimal::from(4i64).sqrt(), Some(Decimal::from(2i64)));
        assert_eq!(Decimal::from(9i64).sqrt(), Some(Decimal::from(3i64)));
        assert_eq!(Decimal::from(16i64).sqrt(), Some(Decimal::from(4i64)));
        assert_eq!(Decimal::from(100i64).sqrt(), Some(Decimal::from(10i64)));
        assert_eq!(Decimal::ZERO.sqrt(), Some(Decimal::ZERO));
        assert_eq!(Decimal::ONE.sqrt(), Some(Decimal::ONE));
    }

    #[test]
    fn sqrt_negative_returns_none() {
        assert_eq!(Decimal::from(-1i64).sqrt(), None);
        assert_eq!(Decimal::from(-100i64).sqrt(), None);
    }

    #[test]
    fn sqrt_non_perfect() {
        let sqrt2 = Decimal::from(2i64).sqrt().unwrap();
        // sqrt(2) ≈ 1.414213...
        let expected = Decimal::from_str("1.4142135623730951").unwrap();
        let diff = (sqrt2 - expected).abs();
        assert!(diff < Decimal::from_str("0.0001").unwrap());
    }

    #[test]
    fn exp_basic() {
        // e^0 = 1
        assert_eq!(Decimal::ZERO.exp(), Some(Decimal::ONE));

        // e^1 ≈ 2.718...
        let e = Decimal::ONE.exp().unwrap();
        let expected = Decimal::e();
        let diff = (e - expected).abs();
        assert!(diff < Decimal::from_str("0.0001").unwrap());
    }

    #[test]
    fn exp_overflow_protection() {
        // Very large exponent should return None (overflow)
        assert_eq!(Decimal::from(200i64).exp(), None);

        // Very negative exponent should return zero (underflow to zero)
        let result = Decimal::from(-200i64).exp();
        assert_eq!(result, Some(Decimal::ZERO));
    }

    #[test]
    fn ln_basic() {
        // ln(1) = 0
        assert_eq!(Decimal::ONE.ln(), Some(Decimal::ZERO));

        // ln(e) ≈ 1
        let e = Decimal::e();
        let ln_e = e.ln().unwrap();
        let diff = (ln_e - Decimal::ONE).abs();
        assert!(diff < Decimal::from_str("0.0001").unwrap());
    }

    #[test]
    fn ln_invalid_inputs() {
        // ln(0) is undefined
        assert_eq!(Decimal::ZERO.ln(), None);

        // ln(negative) is undefined (in reals)
        assert_eq!(Decimal::from(-1i64).ln(), None);
    }

    #[test]
    fn exp_ln_inverse() {
        // exp(ln(x)) = x
        let x = Decimal::from(5i64);
        let result = x.ln().unwrap().exp().unwrap();
        let diff = (result - x).abs();
        assert!(diff < Decimal::from_str("0.0001").unwrap());

        // ln(exp(x)) = x
        let y = Decimal::from(2i64);
        let result2 = y.exp().unwrap().ln().unwrap();
        let diff2 = (result2 - y).abs();
        assert!(diff2 < Decimal::from_str("0.0001").unwrap());
    }

    #[test]
    fn pow_basic() {
        // 2^3 ≈ 8 (small precision loss due to exp/ln)
        let result = Decimal::from(2i64).pow(Decimal::from(3i64)).unwrap();
        let diff = (result - Decimal::from(8i64)).abs();
        assert!(diff < Decimal::from_str("0.0001").unwrap());

        // x^0 = 1
        assert_eq!(
            Decimal::from(100i64).pow(Decimal::ZERO),
            Some(Decimal::ONE)
        );

        // x^1 ≈ x
        let result2 = Decimal::from(42i64).pow(Decimal::ONE).unwrap();
        let diff2 = (result2 - Decimal::from(42i64)).abs();
        assert!(diff2 < Decimal::from_str("0.0001").unwrap());
    }

    #[test]
    fn pow_fractional_exponent() {
        // 4^0.5 = 2 (square root)
        let result = Decimal::from(4i64)
            .pow(Decimal::from_str("0.5").unwrap())
            .unwrap();
        let diff = (result - Decimal::from(2i64)).abs();
        assert!(diff < Decimal::from_str("0.0001").unwrap());
    }

    #[test]
    fn constants() {
        // Verify constants are reasonable
        let e = Decimal::e();
        assert!(e > Decimal::from(2i64));
        assert!(e < Decimal::from(3i64));

        let pi = Decimal::pi();
        assert!(pi > Decimal::from(3i64));
        assert!(pi < Decimal::from(4i64));
    }

    #[test]
    fn powi_exact() {
        // Integer powers should be exact
        assert_eq!(Decimal::from(2i64).powi(0), Some(Decimal::ONE));
        assert_eq!(Decimal::from(2i64).powi(1), Some(Decimal::from(2i64)));
        assert_eq!(Decimal::from(2i64).powi(2), Some(Decimal::from(4i64)));
        assert_eq!(Decimal::from(2i64).powi(3), Some(Decimal::from(8i64)));
        assert_eq!(Decimal::from(2i64).powi(10), Some(Decimal::from(1024i64)));

        // Negative exponents
        let half = Decimal::from(2i64).powi(-1).unwrap();
        assert_eq!(half, Decimal::from_str("0.5").unwrap());

        let quarter = Decimal::from(2i64).powi(-2).unwrap();
        assert_eq!(quarter, Decimal::from_str("0.25").unwrap());
    }
}
