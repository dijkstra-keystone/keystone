//! Term Structure (Yield Curve) primitives for fixed income calculations.
//!
//! A yield curve relates time to risk-free rates, discount factors, and forward rates.
//! This module provides the foundational traits and implementations for building
//! and querying yield curves in a `no_std` environment.
//!
//! # Overview
//!
//! - [`TermStructure`] trait: Core interface for all yield curve implementations
//! - [`FlatTermStructure`]: Constant rate across all maturities
//! - [`PiecewiseTermStructure`]: Multiple rate points with interpolation

use crate::day_count::YearFraction;
use precision_core::{ArithmeticError, Decimal};

/// Core trait for term structure (yield curve) implementations.
///
/// All yield curves must implement these fundamental methods that relate
/// time to discount factors, zero rates, and forward rates.
pub trait TermStructure {
    /// Returns the discount factor for a given time.
    ///
    /// The discount factor D(t) is the present value of $1 to be received at time t.
    /// D(t) = exp(-r(t) * t) where r(t) is the continuously compounded zero rate.
    ///
    /// # Arguments
    /// * `t` - Time to maturity as a year fraction
    ///
    /// # Returns
    /// Discount factor in (0, 1] range
    fn discount_factor(&self, t: YearFraction) -> Result<Decimal, ArithmeticError>;

    /// Returns the continuously compounded zero rate for a given time.
    ///
    /// The zero rate r(t) is the rate that, when continuously compounded,
    /// gives the discount factor: D(t) = exp(-r(t) * t)
    ///
    /// # Arguments
    /// * `t` - Time to maturity as a year fraction
    ///
    /// # Returns
    /// Zero rate as a decimal (e.g., 0.05 for 5%)
    fn zero_rate(&self, t: YearFraction) -> Result<Decimal, ArithmeticError>;

    /// Returns the forward rate between two times.
    ///
    /// The forward rate f(t1, t2) is the rate applicable from time t1 to t2,
    /// implied by the zero rates at t1 and t2.
    ///
    /// Formula: f(t1,t2) = (r(t2)*t2 - r(t1)*t1) / (t2 - t1)
    ///
    /// # Arguments
    /// * `t1` - Start time as year fraction
    /// * `t2` - End time as year fraction (must be > t1)
    ///
    /// # Returns
    /// Forward rate as a decimal
    fn forward_rate(&self, t1: YearFraction, t2: YearFraction) -> Result<Decimal, ArithmeticError> {
        if t2 <= t1 {
            return Err(ArithmeticError::DivisionByZero);
        }

        let r1 = self.zero_rate(t1)?;
        let r2 = self.zero_rate(t2)?;

        let r1_t1 = r1.try_mul(t1)?;
        let r2_t2 = r2.try_mul(t2)?;
        let delta_t = t2.try_sub(t1)?;

        r2_t2.try_sub(r1_t1)?.try_div(delta_t)
    }

    /// Returns the instantaneous forward rate at time t.
    ///
    /// This is the limit of forward_rate(t, t+dt) as dt -> 0.
    /// Approximated using a small time step.
    fn instantaneous_forward(&self, t: YearFraction) -> Result<Decimal, ArithmeticError> {
        let dt = Decimal::new(1, 4); // 0.0001 years (~0.9 hours)
        let t2 = t.try_add(dt)?;
        self.forward_rate(t, t2)
    }
}

/// A flat (constant) term structure where the rate is the same for all maturities.
///
/// This is the simplest yield curve implementation, useful for testing
/// and as a baseline comparison.
#[derive(Debug, Clone)]
pub struct FlatTermStructure {
    /// The constant continuously compounded rate
    rate: Decimal,
}

impl FlatTermStructure {
    /// Creates a new flat term structure with the given rate.
    ///
    /// # Arguments
    /// * `rate` - Continuously compounded rate (e.g., 0.05 for 5%)
    pub fn new(rate: Decimal) -> Self {
        Self { rate }
    }

    /// Returns the underlying constant rate.
    pub fn rate(&self) -> Decimal {
        self.rate
    }
}

impl TermStructure for FlatTermStructure {
    fn discount_factor(&self, t: YearFraction) -> Result<Decimal, ArithmeticError> {
        // D(t) = exp(-r * t)
        let rt = self.rate.try_mul(t)?;
        exp_approx(-rt)
    }

    fn zero_rate(&self, _t: YearFraction) -> Result<Decimal, ArithmeticError> {
        Ok(self.rate)
    }

    fn forward_rate(
        &self,
        _t1: YearFraction,
        _t2: YearFraction,
    ) -> Result<Decimal, ArithmeticError> {
        // For flat curve, forward rate equals spot rate
        Ok(self.rate)
    }
}

/// A node in a piecewise yield curve, representing a rate at a specific time.
#[derive(Debug, Clone, Copy)]
pub struct CurveNode {
    /// Time to maturity as year fraction
    pub time: YearFraction,
    /// Zero rate at this time
    pub rate: Decimal,
}

impl CurveNode {
    /// Creates a new curve node.
    pub fn new(time: YearFraction, rate: Decimal) -> Self {
        Self { time, rate }
    }
}

/// Maximum number of nodes in a piecewise curve (for no_std fixed allocation).
pub const MAX_CURVE_NODES: usize = 32;

/// A piecewise linear term structure built from discrete rate points.
///
/// Rates between nodes are linearly interpolated in rate space.
/// This is the foundation for bootstrapped yield curves.
#[derive(Debug, Clone)]
pub struct PiecewiseTermStructure {
    /// Curve nodes sorted by time
    nodes: [Option<CurveNode>; MAX_CURVE_NODES],
    /// Number of active nodes
    count: usize,
}

impl PiecewiseTermStructure {
    /// Creates an empty piecewise term structure.
    pub fn new() -> Self {
        Self {
            nodes: [None; MAX_CURVE_NODES],
            count: 0,
        }
    }

    /// Adds a node to the curve.
    ///
    /// Nodes are kept sorted by time. Returns error if curve is full.
    pub fn add_node(&mut self, node: CurveNode) -> Result<(), ArithmeticError> {
        if self.count >= MAX_CURVE_NODES {
            return Err(ArithmeticError::Overflow);
        }

        // Find insertion point (keep sorted)
        let mut insert_idx = self.count;
        for i in 0..self.count {
            if let Some(existing) = &self.nodes[i] {
                if node.time < existing.time {
                    insert_idx = i;
                    break;
                }
            }
        }

        // Shift nodes to make room
        for i in (insert_idx..self.count).rev() {
            self.nodes[i + 1] = self.nodes[i];
        }

        self.nodes[insert_idx] = Some(node);
        self.count += 1;
        Ok(())
    }

    /// Returns the number of nodes in the curve.
    pub fn node_count(&self) -> usize {
        self.count
    }

    /// Finds the bracketing nodes for a given time.
    fn find_bracket(&self, t: YearFraction) -> (Option<&CurveNode>, Option<&CurveNode>) {
        if self.count == 0 {
            return (None, None);
        }

        let mut lower: Option<&CurveNode> = None;
        let mut upper: Option<&CurveNode> = None;

        for i in 0..self.count {
            if let Some(node) = &self.nodes[i] {
                if node.time <= t {
                    lower = Some(node);
                }
                if node.time >= t && upper.is_none() {
                    upper = Some(node);
                }
            }
        }

        (lower, upper)
    }
}

impl Default for PiecewiseTermStructure {
    fn default() -> Self {
        Self::new()
    }
}

impl TermStructure for PiecewiseTermStructure {
    fn discount_factor(&self, t: YearFraction) -> Result<Decimal, ArithmeticError> {
        let rate = self.zero_rate(t)?;
        let rt = rate.try_mul(t)?;
        exp_approx(-rt)
    }

    fn zero_rate(&self, t: YearFraction) -> Result<Decimal, ArithmeticError> {
        if self.count == 0 {
            return Err(ArithmeticError::DivisionByZero);
        }

        let (lower, upper) = self.find_bracket(t);

        match (lower, upper) {
            (Some(l), Some(u)) if l.time == u.time => {
                // Exact match
                Ok(l.rate)
            }
            (Some(l), Some(u)) => {
                // Linear interpolation
                let t_range = u.time.try_sub(l.time)?;
                let r_range = u.rate.try_sub(l.rate)?;
                let t_offset = t.try_sub(l.time)?;
                let slope = r_range.try_div(t_range)?;
                l.rate.try_add(slope.try_mul(t_offset)?)
            }
            (Some(l), None) => {
                // Extrapolate flat from last node
                Ok(l.rate)
            }
            (None, Some(u)) => {
                // Extrapolate flat from first node
                Ok(u.rate)
            }
            (None, None) => Err(ArithmeticError::DivisionByZero),
        }
    }
}

/// Approximates exp(x) using Taylor series.
///
/// This is a no_std compatible implementation for small to moderate values of x.
/// For |x| < 2, uses 12 terms for good precision.
fn exp_approx(x: Decimal) -> Result<Decimal, ArithmeticError> {
    // For very small x, return 1 + x
    if x.abs() < Decimal::new(1, 10) {
        return Decimal::ONE.try_add(x);
    }

    // Taylor series: exp(x) = 1 + x + x^2/2! + x^3/3! + ...
    let mut sum = Decimal::ONE;
    let mut term = Decimal::ONE;

    for n in 1..=16 {
        term = term.try_mul(x)?.try_div(Decimal::from(n as i64))?;
        sum = sum.try_add(term)?;

        // Early termination if term is negligible
        if term.abs() < Decimal::new(1, 20) {
            break;
        }
    }

    Ok(sum)
}

/// Approximates ln(x) using series expansion.
///
/// Uses the identity: ln(x) = 2 * arctanh((x-1)/(x+1)) for x > 0
#[allow(dead_code)]
fn ln_approx(x: Decimal) -> Result<Decimal, ArithmeticError> {
    if x <= Decimal::ZERO {
        return Err(ArithmeticError::DivisionByZero);
    }

    // For x close to 1, use ln(1+y) series where y = x - 1
    let y = x.try_sub(Decimal::ONE)?;
    if y.abs() < Decimal::new(5, 1) {
        // |x - 1| < 0.5, use arctanh formula
        let num = x.try_sub(Decimal::ONE)?;
        let den = x.try_add(Decimal::ONE)?;
        let z = num.try_div(den)?;

        // arctanh(z) = z + z^3/3 + z^5/5 + ...
        let mut sum = z;
        let mut z_pow = z;
        let z_sq = z.try_mul(z)?;

        for n in (3..=15).step_by(2) {
            z_pow = z_pow.try_mul(z_sq)?;
            let term = z_pow.try_div(Decimal::from(n as i64))?;
            sum = sum.try_add(term)?;
        }

        // ln(x) = 2 * arctanh((x-1)/(x+1))
        sum.try_mul(Decimal::from(2i64))
    } else {
        // For larger values, use reduction: ln(x) = ln(x/e) + 1
        // This is simplified; in production, use range reduction
        let e_approx = Decimal::new(2718281828, 9); // ~e
        let reduced = x.try_div(e_approx)?;
        let ln_reduced = ln_approx(reduced)?;
        ln_reduced.try_add(Decimal::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use precision_core::RoundingMode;

    #[test]
    fn test_flat_structure_discount() {
        let curve = FlatTermStructure::new(Decimal::new(5, 2)); // 5%
        let t = Decimal::ONE; // 1 year

        let df = curve.discount_factor(t).unwrap();
        // exp(-0.05) ≈ 0.9512
        let rounded = df.round(4, RoundingMode::HalfEven);
        assert!(rounded > Decimal::new(95, 2));
        assert!(rounded < Decimal::new(96, 2));
    }

    #[test]
    fn test_flat_structure_zero_rate() {
        let rate = Decimal::new(5, 2);
        let curve = FlatTermStructure::new(rate);

        assert_eq!(curve.zero_rate(Decimal::ONE).unwrap(), rate);
        assert_eq!(curve.zero_rate(Decimal::from(5i64)).unwrap(), rate);
    }

    #[test]
    fn test_flat_structure_forward_rate() {
        let rate = Decimal::new(5, 2);
        let curve = FlatTermStructure::new(rate);

        let fwd = curve
            .forward_rate(Decimal::ONE, Decimal::from(2i64))
            .unwrap();
        assert_eq!(fwd, rate);
    }

    #[test]
    fn test_piecewise_interpolation() {
        let mut curve = PiecewiseTermStructure::new();
        curve
            .add_node(CurveNode::new(Decimal::ONE, Decimal::new(3, 2)))
            .unwrap(); // 3% at 1Y
        curve
            .add_node(CurveNode::new(Decimal::from(2i64), Decimal::new(4, 2)))
            .unwrap(); // 4% at 2Y

        // At 1.5Y, should interpolate to 3.5%
        let rate = curve.zero_rate(Decimal::new(15, 1)).unwrap();
        let rounded = rate.round(4, RoundingMode::HalfEven);
        assert_eq!(rounded, Decimal::new(35, 3));
    }

    #[test]
    fn test_piecewise_extrapolation() {
        let mut curve = PiecewiseTermStructure::new();
        curve
            .add_node(CurveNode::new(Decimal::ONE, Decimal::new(3, 2)))
            .unwrap();
        curve
            .add_node(CurveNode::new(Decimal::from(2i64), Decimal::new(4, 2)))
            .unwrap();

        // Beyond 2Y, should extrapolate flat at 4%
        let rate = curve.zero_rate(Decimal::from(5i64)).unwrap();
        assert_eq!(rate, Decimal::new(4, 2));

        // Before 1Y, should extrapolate flat at 3%
        let rate = curve.zero_rate(Decimal::new(5, 1)).unwrap();
        assert_eq!(rate, Decimal::new(3, 2));
    }

    #[test]
    fn test_exp_approx() {
        // exp(0) = 1
        let e0 = exp_approx(Decimal::ZERO).unwrap();
        assert_eq!(e0, Decimal::ONE);

        // exp(1) ≈ 2.718
        let e1 = exp_approx(Decimal::ONE).unwrap();
        let rounded = e1.round(3, RoundingMode::HalfEven);
        assert_eq!(rounded, Decimal::new(2718, 3));

        // exp(-1) ≈ 0.368
        let e_neg1 = exp_approx(-Decimal::ONE).unwrap();
        let rounded = e_neg1.round(3, RoundingMode::HalfEven);
        assert_eq!(rounded, Decimal::new(368, 3));
    }

    #[test]
    fn test_forward_rate_calculation() {
        let mut curve = PiecewiseTermStructure::new();
        curve
            .add_node(CurveNode::new(Decimal::ONE, Decimal::new(3, 2)))
            .unwrap(); // 3% at 1Y
        curve
            .add_node(CurveNode::new(Decimal::from(2i64), Decimal::new(4, 2)))
            .unwrap(); // 4% at 2Y

        // Forward rate from 1Y to 2Y
        // f(1,2) = (r2*t2 - r1*t1) / (t2 - t1) = (0.04*2 - 0.03*1) / 1 = 0.05
        let fwd = curve
            .forward_rate(Decimal::ONE, Decimal::from(2i64))
            .unwrap();
        let rounded = fwd.round(4, RoundingMode::HalfEven);
        assert_eq!(rounded, Decimal::new(5, 2));
    }
}
