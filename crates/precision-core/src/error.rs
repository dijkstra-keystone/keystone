//! Error types for decimal operations.

use core::fmt;
use serde::{Deserialize, Serialize};

/// Error returned when an arithmetic operation fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArithmeticError {
    /// Result exceeds maximum representable value.
    Overflow,
    /// Result is smaller than minimum representable value.
    Underflow,
    /// Division by zero attempted.
    DivisionByZero,
    /// Scale exceeds maximum precision.
    ScaleExceeded,
}

impl fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => write!(f, "arithmetic overflow"),
            Self::Underflow => write!(f, "arithmetic underflow"),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::ScaleExceeded => write!(f, "scale exceeds maximum precision"),
        }
    }
}

/// Error returned when parsing a decimal from a string fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseError {
    /// Input string is empty.
    Empty,
    /// Invalid character in input.
    InvalidCharacter,
    /// Multiple decimal points in input.
    MultipleDecimalPoints,
    /// Value exceeds representable range.
    OutOfRange,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty string"),
            Self::InvalidCharacter => write!(f, "invalid character"),
            Self::MultipleDecimalPoints => write!(f, "multiple decimal points"),
            Self::OutOfRange => write!(f, "value out of range"),
        }
    }
}
