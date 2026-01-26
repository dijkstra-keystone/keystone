#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Financial calculation functions built on precision-core.
//!
//! This crate provides common financial calculations with deterministic results,
//! including:
//!
//! - Interest calculations (simple, compound, continuous)
//! - Time value of money (present value, future value, NPV)
//! - Percentage operations and basis points
//! - **Options pricing** (Black-Scholes model, Greeks, implied volatility)
//! - **Term structures** (yield curves, discount factors, forward rates)
//! - **Day count conventions** (Actual/360, 30/360, etc.)

pub mod day_count;
pub mod interpolation;
mod interest;
pub mod options;
mod percentage;
pub mod solver;
pub mod term_structure;
mod time_value;

pub use day_count::{Date, DayCountConvention, YearFraction};
pub use interpolation::{CubicSpline, DataPoint, Interpolator, Linear, LogLinear};
pub use interest::{compound_interest, effective_annual_rate, simple_interest};
pub use options::{
    black_scholes_call, black_scholes_put, call_greeks, implied_volatility, normal_cdf, normal_pdf,
    put_greeks, Greeks, OptionParams,
};
pub use percentage::{basis_points_to_decimal, percentage_change, percentage_of};
pub use precision_core::{ArithmeticError, Decimal, RoundingMode};
pub use term_structure::{
    CurveNode, FlatTermStructure, PiecewiseTermStructure, TermStructure, MAX_CURVE_NODES,
};
pub use solver::{
    bisection, brent, default_tolerance, newton_raphson, newton_raphson_numerical, secant,
    SolverResult, DEFAULT_MAX_ITER,
};
pub use time_value::{future_value, net_present_value, present_value};
