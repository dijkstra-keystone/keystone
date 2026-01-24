//! Rounding modes for decimal arithmetic.

use serde::{Deserialize, Serialize};

/// Rounding mode for decimal operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum RoundingMode {
    /// Round toward negative infinity.
    Down,
    /// Round toward positive infinity.
    Up,
    /// Round toward zero (truncate).
    TowardZero,
    /// Round away from zero.
    AwayFromZero,
    /// Round to nearest, ties go to even (banker's rounding).
    #[default]
    HalfEven,
    /// Round to nearest, ties round up.
    HalfUp,
    /// Round to nearest, ties round down.
    HalfDown,
}

impl RoundingMode {
    pub(crate) fn to_rust_decimal(self) -> rust_decimal::RoundingStrategy {
        match self {
            Self::Down => rust_decimal::RoundingStrategy::ToNegativeInfinity,
            Self::Up => rust_decimal::RoundingStrategy::ToPositiveInfinity,
            Self::TowardZero => rust_decimal::RoundingStrategy::ToZero,
            Self::AwayFromZero => rust_decimal::RoundingStrategy::AwayFromZero,
            Self::HalfEven => rust_decimal::RoundingStrategy::MidpointNearestEven,
            Self::HalfUp => rust_decimal::RoundingStrategy::MidpointAwayFromZero,
            Self::HalfDown => rust_decimal::RoundingStrategy::MidpointTowardZero,
        }
    }
}
