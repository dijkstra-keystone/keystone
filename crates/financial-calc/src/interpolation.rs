//! Interpolation methods for yield curves and financial data.
//!
//! This module provides deterministic interpolation algorithms suitable for
//! `no_std` environments. These are essential for constructing smooth yield
//! curves between observed market points.
//!
//! # Available Methods
//!
//! - [`Linear`]: Simple linear interpolation between points
//! - [`LogLinear`]: Interpolation in log space (preserves positive values)
//! - [`CubicSpline`]: Smooth cubic spline with natural boundary conditions

use precision_core::{ArithmeticError, Decimal};

/// Maximum number of points for interpolation (for no_std fixed allocation).
pub const MAX_INTERP_POINTS: usize = 32;

/// A data point for interpolation.
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    /// X coordinate (typically time)
    pub x: Decimal,
    /// Y coordinate (typically rate or discount factor)
    pub y: Decimal,
}

impl DataPoint {
    /// Creates a new data point.
    pub fn new(x: Decimal, y: Decimal) -> Self {
        Self { x, y }
    }
}

/// Trait for interpolation methods.
pub trait Interpolator {
    /// Interpolates a value at the given x coordinate.
    ///
    /// Returns error if x is outside the data range or if interpolation fails.
    fn interpolate(&self, x: Decimal) -> Result<Decimal, ArithmeticError>;

    /// Returns true if extrapolation beyond data bounds is supported.
    fn supports_extrapolation(&self) -> bool {
        false
    }
}

/// Linear interpolation between data points.
///
/// For a point x between x_i and x_{i+1}, the interpolated value is:
/// y = y_i + (y_{i+1} - y_i) * (x - x_i) / (x_{i+1} - x_i)
#[derive(Debug, Clone)]
pub struct Linear {
    points: [Option<DataPoint>; MAX_INTERP_POINTS],
    count: usize,
}

impl Linear {
    /// Creates a new empty linear interpolator.
    pub fn new() -> Self {
        Self {
            points: [None; MAX_INTERP_POINTS],
            count: 0,
        }
    }

    /// Adds a data point, keeping points sorted by x.
    pub fn add_point(&mut self, point: DataPoint) -> Result<(), ArithmeticError> {
        if self.count >= MAX_INTERP_POINTS {
            return Err(ArithmeticError::Overflow);
        }

        // Find insertion point
        let mut idx = self.count;
        for i in 0..self.count {
            if let Some(p) = &self.points[i] {
                if point.x < p.x {
                    idx = i;
                    break;
                }
            }
        }

        // Shift points
        for i in (idx..self.count).rev() {
            self.points[i + 1] = self.points[i];
        }

        self.points[idx] = Some(point);
        self.count += 1;
        Ok(())
    }

    /// Returns the number of data points.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if no data points are stored.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn find_bracket(&self, x: Decimal) -> (Option<&DataPoint>, Option<&DataPoint>) {
        let mut lower: Option<&DataPoint> = None;
        let mut upper: Option<&DataPoint> = None;

        for i in 0..self.count {
            if let Some(p) = &self.points[i] {
                if p.x <= x {
                    lower = Some(p);
                }
                if p.x >= x && upper.is_none() {
                    upper = Some(p);
                }
            }
        }

        (lower, upper)
    }
}

impl Default for Linear {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpolator for Linear {
    fn interpolate(&self, x: Decimal) -> Result<Decimal, ArithmeticError> {
        if self.count == 0 {
            return Err(ArithmeticError::DivisionByZero);
        }

        let (lower, upper) = self.find_bracket(x);

        match (lower, upper) {
            (Some(l), Some(u)) if l.x == u.x => Ok(l.y),
            (Some(l), Some(u)) => {
                // Linear interpolation
                let x_range = u.x.try_sub(l.x)?;
                let y_range = u.y.try_sub(l.y)?;
                let x_offset = x.try_sub(l.x)?;
                let slope = y_range.try_div(x_range)?;
                l.y.try_add(slope.try_mul(x_offset)?)
            }
            (Some(l), None) => Ok(l.y), // Flat extrapolation
            (None, Some(u)) => Ok(u.y), // Flat extrapolation
            (None, None) => Err(ArithmeticError::DivisionByZero),
        }
    }

    fn supports_extrapolation(&self) -> bool {
        true // Flat extrapolation
    }
}

/// Log-linear interpolation.
///
/// Interpolates in log space, which ensures the result is always positive.
/// Useful for discount factors which must be in (0, 1].
///
/// y = exp(ln(y_i) + (ln(y_{i+1}) - ln(y_i)) * (x - x_i) / (x_{i+1} - x_i))
#[derive(Debug, Clone)]
pub struct LogLinear {
    points: [Option<DataPoint>; MAX_INTERP_POINTS],
    count: usize,
}

impl LogLinear {
    /// Creates a new empty log-linear interpolator.
    pub fn new() -> Self {
        Self {
            points: [None; MAX_INTERP_POINTS],
            count: 0,
        }
    }

    /// Adds a data point.
    ///
    /// Y values must be positive for log-linear interpolation.
    pub fn add_point(&mut self, point: DataPoint) -> Result<(), ArithmeticError> {
        if !point.y.is_positive() {
            return Err(ArithmeticError::LogOfNegative);
        }
        if self.count >= MAX_INTERP_POINTS {
            return Err(ArithmeticError::Overflow);
        }

        // Find insertion point
        let mut idx = self.count;
        for i in 0..self.count {
            if let Some(p) = &self.points[i] {
                if point.x < p.x {
                    idx = i;
                    break;
                }
            }
        }

        // Shift points
        for i in (idx..self.count).rev() {
            self.points[i + 1] = self.points[i];
        }

        self.points[idx] = Some(point);
        self.count += 1;
        Ok(())
    }

    /// Returns the number of data points.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if no data points are stored.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn find_bracket(&self, x: Decimal) -> (Option<&DataPoint>, Option<&DataPoint>) {
        let mut lower: Option<&DataPoint> = None;
        let mut upper: Option<&DataPoint> = None;

        for i in 0..self.count {
            if let Some(p) = &self.points[i] {
                if p.x <= x {
                    lower = Some(p);
                }
                if p.x >= x && upper.is_none() {
                    upper = Some(p);
                }
            }
        }

        (lower, upper)
    }
}

impl Default for LogLinear {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpolator for LogLinear {
    fn interpolate(&self, x: Decimal) -> Result<Decimal, ArithmeticError> {
        if self.count == 0 {
            return Err(ArithmeticError::DivisionByZero);
        }

        let (lower, upper) = self.find_bracket(x);

        match (lower, upper) {
            (Some(l), Some(u)) if l.x == u.x => Ok(l.y),
            (Some(l), Some(u)) => {
                // Interpolate in log space
                let ln_l = l.y.try_ln()?;
                let ln_u = u.y.try_ln()?;

                let x_range = u.x.try_sub(l.x)?;
                let ln_range = ln_u.try_sub(ln_l)?;
                let x_offset = x.try_sub(l.x)?;
                let slope = ln_range.try_div(x_range)?;
                let ln_result = ln_l.try_add(slope.try_mul(x_offset)?)?;

                ln_result.try_exp()
            }
            (Some(l), None) => Ok(l.y),
            (None, Some(u)) => Ok(u.y),
            (None, None) => Err(ArithmeticError::DivisionByZero),
        }
    }

    fn supports_extrapolation(&self) -> bool {
        true
    }
}

/// Natural cubic spline interpolation.
///
/// Provides a smooth C2 interpolation with continuous first and second derivatives.
/// Uses the Thomas Algorithm (TDMA) for solving the tridiagonal system in no_std.
///
/// This is the gold standard for yield curve interpolation as it produces
/// smooth forward rate curves without artificial kinks.
#[derive(Debug, Clone)]
pub struct CubicSpline {
    points: [Option<DataPoint>; MAX_INTERP_POINTS],
    /// Second derivatives at each point (computed after all points are added)
    second_derivs: [Decimal; MAX_INTERP_POINTS],
    count: usize,
    /// Whether the spline coefficients have been computed
    computed: bool,
}

impl CubicSpline {
    /// Creates a new empty cubic spline interpolator.
    pub fn new() -> Self {
        Self {
            points: [None; MAX_INTERP_POINTS],
            second_derivs: [Decimal::ZERO; MAX_INTERP_POINTS],
            count: 0,
            computed: false,
        }
    }

    /// Adds a data point.
    ///
    /// Note: After adding all points, call `compute()` to calculate spline coefficients.
    pub fn add_point(&mut self, point: DataPoint) -> Result<(), ArithmeticError> {
        if self.count >= MAX_INTERP_POINTS {
            return Err(ArithmeticError::Overflow);
        }

        // Find insertion point
        let mut idx = self.count;
        for i in 0..self.count {
            if let Some(p) = &self.points[i] {
                if point.x < p.x {
                    idx = i;
                    break;
                }
            }
        }

        // Shift points
        for i in (idx..self.count).rev() {
            self.points[i + 1] = self.points[i];
        }

        self.points[idx] = Some(point);
        self.count += 1;
        self.computed = false; // Need to recompute
        Ok(())
    }

    /// Returns the number of data points.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if no data points are stored.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Computes the spline coefficients using the Thomas Algorithm.
    ///
    /// Must be called after all points are added and before interpolation.
    /// Uses natural spline boundary conditions (second derivative = 0 at endpoints).
    pub fn compute(&mut self) -> Result<(), ArithmeticError> {
        if self.count < 2 {
            self.computed = true;
            return Ok(());
        }

        let n = self.count;

        // Working arrays for Thomas Algorithm
        let mut a = [Decimal::ZERO; MAX_INTERP_POINTS]; // Sub-diagonal
        let mut b = [Decimal::ZERO; MAX_INTERP_POINTS]; // Main diagonal
        let mut c = [Decimal::ZERO; MAX_INTERP_POINTS]; // Super-diagonal
        let mut d = [Decimal::ZERO; MAX_INTERP_POINTS]; // Right-hand side

        // Build the tridiagonal system for natural spline
        // Natural boundary: M_0 = M_{n-1} = 0
        self.second_derivs[0] = Decimal::ZERO;
        self.second_derivs[n - 1] = Decimal::ZERO;

        // Set up equations for interior points
        for i in 1..n - 1 {
            let p0 = self.points[i - 1].as_ref().unwrap();
            let p1 = self.points[i].as_ref().unwrap();
            let p2 = self.points[i + 1].as_ref().unwrap();

            let h0 = p1.x.try_sub(p0.x)?;
            let h1 = p2.x.try_sub(p1.x)?;

            a[i] = h0;
            b[i] = Decimal::from(2i64).try_mul(h0.try_add(h1)?)?;
            c[i] = h1;

            let dy0 = p1.y.try_sub(p0.y)?.try_div(h0)?;
            let dy1 = p2.y.try_sub(p1.y)?.try_div(h1)?;
            d[i] = Decimal::from(6i64).try_mul(dy1.try_sub(dy0)?)?;
        }

        // Thomas Algorithm (forward elimination)
        for i in 2..n - 1 {
            let m = a[i].try_div(b[i - 1])?;
            b[i] = b[i].try_sub(m.try_mul(c[i - 1])?)?;
            d[i] = d[i].try_sub(m.try_mul(d[i - 1])?)?;
        }

        // Back substitution
        if n > 2 {
            self.second_derivs[n - 2] = d[n - 2].try_div(b[n - 2])?;
        }
        for i in (1..n - 2).rev() {
            self.second_derivs[i] = d[i]
                .try_sub(c[i].try_mul(self.second_derivs[i + 1])?)?
                .try_div(b[i])?;
        }

        self.computed = true;
        Ok(())
    }

    fn find_segment(&self, x: Decimal) -> Option<(usize, &DataPoint, &DataPoint)> {
        for i in 0..self.count - 1 {
            let p0 = self.points[i].as_ref()?;
            let p1 = self.points[i + 1].as_ref()?;
            if x >= p0.x && x <= p1.x {
                return Some((i, p0, p1));
            }
        }
        None
    }
}

impl Default for CubicSpline {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpolator for CubicSpline {
    fn interpolate(&self, x: Decimal) -> Result<Decimal, ArithmeticError> {
        if !self.computed {
            return Err(ArithmeticError::DivisionByZero); // Not computed
        }
        if self.count == 0 {
            return Err(ArithmeticError::DivisionByZero);
        }
        if self.count == 1 {
            return Ok(self.points[0].as_ref().unwrap().y);
        }

        // Handle extrapolation with flat extension
        let first = self.points[0].as_ref().unwrap();
        let last = self.points[self.count - 1].as_ref().unwrap();

        if x <= first.x {
            return Ok(first.y);
        }
        if x >= last.x {
            return Ok(last.y);
        }

        // Find the segment containing x
        let (i, p0, p1) = self.find_segment(x).ok_or(ArithmeticError::DivisionByZero)?;

        let h = p1.x.try_sub(p0.x)?;
        let a = p1.x.try_sub(x)?.try_div(h)?;
        let b = x.try_sub(p0.x)?.try_div(h)?;

        let m0 = self.second_derivs[i];
        let m1 = self.second_derivs[i + 1];

        // Cubic spline formula:
        // S(x) = a*y0 + b*y1 + ((a^3 - a)*M0 + (b^3 - b)*M1) * h^2 / 6
        let a3 = a.try_mul(a)?.try_mul(a)?;
        let b3 = b.try_mul(b)?.try_mul(b)?;
        let h2_6 = h.try_mul(h)?.try_div(Decimal::from(6i64))?;

        let term1 = a.try_mul(p0.y)?;
        let term2 = b.try_mul(p1.y)?;
        let term3 = a3.try_sub(a)?.try_mul(m0)?.try_mul(h2_6)?;
        let term4 = b3.try_sub(b)?.try_mul(m1)?.try_mul(h2_6)?;

        term1.try_add(term2)?.try_add(term3)?.try_add(term4)
    }

    fn supports_extrapolation(&self) -> bool {
        true // Flat extrapolation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use precision_core::RoundingMode;

    #[test]
    fn test_linear_interpolation() {
        let mut interp = Linear::new();
        interp
            .add_point(DataPoint::new(Decimal::ZERO, Decimal::ZERO))
            .unwrap();
        interp
            .add_point(DataPoint::new(Decimal::from(10i64), Decimal::from(100i64)))
            .unwrap();

        // Midpoint
        let result = interp.interpolate(Decimal::from(5i64)).unwrap();
        assert_eq!(result, Decimal::from(50i64));

        // At known point
        let result = interp.interpolate(Decimal::from(10i64)).unwrap();
        assert_eq!(result, Decimal::from(100i64));
    }

    #[test]
    fn test_linear_extrapolation() {
        let mut interp = Linear::new();
        interp
            .add_point(DataPoint::new(Decimal::ONE, Decimal::from(10i64)))
            .unwrap();
        interp
            .add_point(DataPoint::new(Decimal::from(2i64), Decimal::from(20i64)))
            .unwrap();

        // Extrapolate left (flat)
        let result = interp.interpolate(Decimal::ZERO).unwrap();
        assert_eq!(result, Decimal::from(10i64));

        // Extrapolate right (flat)
        let result = interp.interpolate(Decimal::from(5i64)).unwrap();
        assert_eq!(result, Decimal::from(20i64));
    }

    #[test]
    fn test_loglinear_preserves_positivity() {
        let mut interp = LogLinear::new();
        interp
            .add_point(DataPoint::new(Decimal::ZERO, Decimal::ONE))
            .unwrap();
        interp
            .add_point(DataPoint::new(
                Decimal::from(10i64),
                Decimal::new(1, 1),
            )) // 0.1
            .unwrap();

        // All interpolated values should be positive
        for i in 0..=10 {
            let x = Decimal::from(i as i64);
            let result = interp.interpolate(x).unwrap();
            assert!(result.is_positive());
        }
    }

    #[test]
    fn test_loglinear_rejects_negative() {
        let mut interp = LogLinear::new();
        let result = interp.add_point(DataPoint::new(Decimal::ZERO, -Decimal::ONE));
        assert!(result.is_err());
    }

    #[test]
    fn test_cubic_spline_smooth() {
        let mut spline = CubicSpline::new();
        spline
            .add_point(DataPoint::new(Decimal::ZERO, Decimal::ZERO))
            .unwrap();
        spline
            .add_point(DataPoint::new(Decimal::ONE, Decimal::ONE))
            .unwrap();
        spline
            .add_point(DataPoint::new(Decimal::from(2i64), Decimal::from(4i64)))
            .unwrap();
        spline
            .add_point(DataPoint::new(Decimal::from(3i64), Decimal::from(9i64)))
            .unwrap();

        spline.compute().unwrap();

        // Check interpolation at known points
        let y0 = spline.interpolate(Decimal::ZERO).unwrap();
        assert_eq!(y0, Decimal::ZERO);

        let y3 = spline.interpolate(Decimal::from(3i64)).unwrap();
        assert_eq!(y3, Decimal::from(9i64));

        // Interpolated values should be smooth
        let y_mid = spline.interpolate(Decimal::new(15, 1)).unwrap(); // x = 1.5
        let rounded = y_mid.round(1, RoundingMode::HalfEven);
        // For y = x^2 data, at x=1.5, y should be around 2.25
        assert!(rounded >= Decimal::from(2i64));
        assert!(rounded <= Decimal::from(3i64));
    }

    #[test]
    fn test_cubic_spline_requires_compute() {
        let mut spline = CubicSpline::new();
        spline
            .add_point(DataPoint::new(Decimal::ZERO, Decimal::ZERO))
            .unwrap();
        spline
            .add_point(DataPoint::new(Decimal::ONE, Decimal::ONE))
            .unwrap();

        // Should fail before compute()
        let result = spline.interpolate(Decimal::new(5, 1));
        assert!(result.is_err());

        spline.compute().unwrap();

        // Should work after compute()
        let result = spline.interpolate(Decimal::new(5, 1));
        assert!(result.is_ok());
    }
}
