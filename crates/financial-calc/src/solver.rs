//! Numerical root-finding solvers for financial calculations.
//!
//! These solvers are essential for yield curve bootstrapping, implied volatility
//! calculation, and other inverse problems in quantitative finance.
//!
//! # Available Methods
//!
//! - [`newton_raphson`]: Fast convergence with derivative, best for smooth functions
//! - [`brent`]: Guaranteed convergence without derivatives, robust fallback
//! - [`bisection`]: Simple bracketing method, always converges

use precision_core::{ArithmeticError, Decimal};

/// Default maximum iterations for solvers.
pub const DEFAULT_MAX_ITER: u32 = 100;

/// Default tolerance for convergence.
pub fn default_tolerance() -> Decimal {
    Decimal::new(1, 12) // 1e-12
}

/// Result of a solver iteration.
#[derive(Debug, Clone)]
pub struct SolverResult {
    /// The solution found.
    pub root: Decimal,
    /// Number of iterations used.
    pub iterations: u32,
    /// Final function value at the root.
    pub residual: Decimal,
    /// Whether convergence was achieved.
    pub converged: bool,
}

/// Newton-Raphson method for finding roots.
///
/// Finds x such that f(x) = 0 using the iteration:
/// x_{n+1} = x_n - f(x_n) / f'(x_n)
///
/// # Arguments
/// * `f` - The function to find the root of
/// * `df` - The derivative of f
/// * `x0` - Initial guess
/// * `tolerance` - Convergence tolerance (|f(x)| < tolerance)
/// * `max_iter` - Maximum number of iterations
///
/// # Example
///
/// ```
/// use financial_calc::solver::newton_raphson;
/// use precision_core::Decimal;
///
/// // Find sqrt(2) by solving x^2 - 2 = 0
/// let f = |x: Decimal| x.try_mul(x).and_then(|x2| x2.try_sub(Decimal::from(2i64)));
/// let df = |x: Decimal| x.try_mul(Decimal::from(2i64));
///
/// let result = newton_raphson(f, df, Decimal::ONE, None, None).unwrap();
/// assert!(result.converged);
/// // result.root ≈ 1.414...
/// ```
pub fn newton_raphson<F, DF>(
    f: F,
    df: DF,
    x0: Decimal,
    tolerance: Option<Decimal>,
    max_iter: Option<u32>,
) -> Result<SolverResult, ArithmeticError>
where
    F: Fn(Decimal) -> Result<Decimal, ArithmeticError>,
    DF: Fn(Decimal) -> Result<Decimal, ArithmeticError>,
{
    let tol = tolerance.unwrap_or_else(default_tolerance);
    let max = max_iter.unwrap_or(DEFAULT_MAX_ITER);

    let mut x = x0;
    let mut iterations = 0;

    loop {
        let fx = f(x)?;

        if fx.abs() < tol {
            return Ok(SolverResult {
                root: x,
                iterations,
                residual: fx,
                converged: true,
            });
        }

        if iterations >= max {
            return Ok(SolverResult {
                root: x,
                iterations,
                residual: fx,
                converged: false,
            });
        }

        let dfx = df(x)?;

        if dfx.abs() < Decimal::new(1, 20) {
            // Derivative too small, can't continue
            return Ok(SolverResult {
                root: x,
                iterations,
                residual: fx,
                converged: false,
            });
        }

        let step = fx.try_div(dfx)?;
        x = x.try_sub(step)?;
        iterations += 1;
    }
}

/// Newton-Raphson with numerical derivative approximation.
///
/// Uses finite differences to approximate the derivative, avoiding the need
/// to provide an analytical derivative function.
///
/// # Arguments
/// * `f` - The function to find the root of
/// * `x0` - Initial guess
/// * `h` - Step size for finite difference (default: 1e-8)
/// * `tolerance` - Convergence tolerance
/// * `max_iter` - Maximum iterations
pub fn newton_raphson_numerical<F>(
    f: F,
    x0: Decimal,
    h: Option<Decimal>,
    tolerance: Option<Decimal>,
    max_iter: Option<u32>,
) -> Result<SolverResult, ArithmeticError>
where
    F: Fn(Decimal) -> Result<Decimal, ArithmeticError>,
{
    let step = h.unwrap_or_else(|| Decimal::new(1, 8)); // 1e-8

    let df = |x: Decimal| -> Result<Decimal, ArithmeticError> {
        let x_plus = x.try_add(step)?;
        let x_minus = x.try_sub(step)?;
        let f_plus = f(x_plus)?;
        let f_minus = f(x_minus)?;
        let two_h = step.try_mul(Decimal::from(2i64))?;
        f_plus.try_sub(f_minus)?.try_div(two_h)
    };

    newton_raphson(&f, df, x0, tolerance, max_iter)
}

/// Bisection method for finding roots.
///
/// Requires that f(a) and f(b) have opposite signs (bracket the root).
/// Converges linearly but is guaranteed to find a root if one exists in [a, b].
///
/// # Arguments
/// * `f` - The function to find the root of
/// * `a` - Lower bound of the bracket
/// * `b` - Upper bound of the bracket
/// * `tolerance` - Convergence tolerance (|b - a| < tolerance)
/// * `max_iter` - Maximum iterations
pub fn bisection<F>(
    f: F,
    mut a: Decimal,
    mut b: Decimal,
    tolerance: Option<Decimal>,
    max_iter: Option<u32>,
) -> Result<SolverResult, ArithmeticError>
where
    F: Fn(Decimal) -> Result<Decimal, ArithmeticError>,
{
    let tol = tolerance.unwrap_or_else(default_tolerance);
    let max = max_iter.unwrap_or(DEFAULT_MAX_ITER);

    let mut fa = f(a)?;
    let fb = f(b)?;

    // Check that we have a bracket
    if fa.is_positive() == fb.is_positive() && !fa.is_zero() && !fb.is_zero() {
        return Err(ArithmeticError::DivisionByZero); // No bracket
    }

    let mut iterations = 0;

    while iterations < max {
        let mid = a.try_add(b)?.try_div(Decimal::from(2i64))?;
        let fmid = f(mid)?;

        if fmid.abs() < tol || b.try_sub(a)?.abs() < tol {
            return Ok(SolverResult {
                root: mid,
                iterations,
                residual: fmid,
                converged: true,
            });
        }

        if fa.is_positive() == fmid.is_positive() {
            a = mid;
            fa = fmid;
        } else {
            b = mid;
        }

        iterations += 1;
    }

    let mid = a.try_add(b)?.try_div(Decimal::from(2i64))?;
    let fmid = f(mid)?;

    Ok(SolverResult {
        root: mid,
        iterations,
        residual: fmid,
        converged: false,
    })
}

/// Brent's method for finding roots.
///
/// Combines bisection, secant method, and inverse quadratic interpolation
/// for robust and fast convergence. This is the recommended general-purpose
/// root-finding method.
///
/// # Arguments
/// * `f` - The function to find the root of
/// * `a` - Lower bound of the bracket
/// * `b` - Upper bound of the bracket
/// * `tolerance` - Convergence tolerance
/// * `max_iter` - Maximum iterations
///
/// # Example
///
/// ```
/// use financial_calc::solver::brent;
/// use precision_core::Decimal;
///
/// // Find cube root of 8 by solving x^3 - 8 = 0
/// let f = |x: Decimal| {
///     x.try_mul(x).and_then(|x2| x2.try_mul(x)).and_then(|x3| x3.try_sub(Decimal::from(8i64)))
/// };
///
/// let result = brent(f, Decimal::ONE, Decimal::from(3i64), None, None).unwrap();
/// assert!(result.converged);
/// // result.root ≈ 2.0
/// ```
pub fn brent<F>(
    f: F,
    mut a: Decimal,
    mut b: Decimal,
    tolerance: Option<Decimal>,
    max_iter: Option<u32>,
) -> Result<SolverResult, ArithmeticError>
where
    F: Fn(Decimal) -> Result<Decimal, ArithmeticError>,
{
    let tol = tolerance.unwrap_or_else(default_tolerance);
    let max = max_iter.unwrap_or(DEFAULT_MAX_ITER);

    let mut fa = f(a)?;
    let mut fb = f(b)?;

    // Check bracket
    if fa.is_positive() == fb.is_positive() && !fa.is_zero() && !fb.is_zero() {
        return Err(ArithmeticError::DivisionByZero);
    }

    // Ensure |f(a)| >= |f(b)|
    if fa.abs() < fb.abs() {
        core::mem::swap(&mut a, &mut b);
        core::mem::swap(&mut fa, &mut fb);
    }

    let mut c = a;
    let mut fc = fa;
    let mut d = b.try_sub(a)?;
    let mut e = d;

    let mut iterations = 0;

    while iterations < max {
        if fb.abs() < tol {
            return Ok(SolverResult {
                root: b,
                iterations,
                residual: fb,
                converged: true,
            });
        }

        if fa.abs() < fb.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }

        let tol1 = Decimal::from(2i64)
            .try_mul(Decimal::new(1, 15))?
            .try_mul(b.abs())?
            .try_add(tol.try_div(Decimal::from(2i64))?)?;
        let xm = c.try_sub(b)?.try_div(Decimal::from(2i64))?;

        if xm.abs() <= tol1 || fb.abs() < tol {
            return Ok(SolverResult {
                root: b,
                iterations,
                residual: fb,
                converged: true,
            });
        }

        // Attempt inverse quadratic interpolation
        let mut use_bisection = true;

        if e.abs() >= tol1 && fa.abs() > fb.abs() {
            let s;
            if (a.try_sub(c)?).abs() < Decimal::new(1, 20) {
                // Linear interpolation (secant method)
                s = fb.try_mul(b.try_sub(a)?)?.try_div(fa.try_sub(fb)?)?;
            } else {
                // Inverse quadratic interpolation
                let r = fb.try_div(fc)?;
                let q = fa.try_div(fc)?;
                let p = fb.try_div(fa)?;

                let num = p
                    .try_mul(Decimal::from(2i64).try_mul(xm)?.try_mul(q.try_sub(Decimal::ONE)?)?)?
                    .try_sub(b.try_sub(a)?.try_mul(q.try_sub(Decimal::ONE)?)?)?;
                let den = q
                    .try_sub(Decimal::ONE)?
                    .try_mul(r.try_sub(Decimal::ONE)?)?
                    .try_mul(p.try_sub(Decimal::ONE)?)?;

                if den.abs() > Decimal::new(1, 20) {
                    s = num.try_div(den)?;
                } else {
                    s = xm; // Fall back to bisection step
                }
            }

            // Check if interpolation step is acceptable
            let bound1 = Decimal::from(3i64)
                .try_mul(xm)?
                .try_div(Decimal::from(4i64))?
                .try_sub(tol1.try_div(Decimal::from(2i64))?)?
                .abs();
            let bound2 = e.abs().try_div(Decimal::from(2i64))?;

            if s.abs() < bound1.min(bound2) {
                e = d;
                d = s;
                use_bisection = false;
            }
        }

        if use_bisection {
            d = xm;
            e = d;
        }

        a = b;
        fa = fb;

        if d.abs() > tol1 {
            b = b.try_add(d)?;
        } else {
            // Move by at least tol1
            let sign = if xm.is_positive() {
                Decimal::ONE
            } else {
                Decimal::NEGATIVE_ONE
            };
            b = b.try_add(sign.try_mul(tol1)?)?;
        }

        fb = f(b)?;

        // Ensure bracket is maintained
        if (fb.is_positive() && fc.is_positive()) || (fb.is_negative() && fc.is_negative()) {
            c = a;
            fc = fa;
            d = b.try_sub(a)?;
            e = d;
        }

        iterations += 1;
    }

    Ok(SolverResult {
        root: b,
        iterations,
        residual: fb,
        converged: false,
    })
}

/// Secant method for finding roots.
///
/// Similar to Newton's method but approximates the derivative using the
/// secant line between two points. Requires two initial guesses.
///
/// # Arguments
/// * `f` - The function to find the root of
/// * `x0` - First initial guess
/// * `x1` - Second initial guess (should differ from x0)
/// * `tolerance` - Convergence tolerance
/// * `max_iter` - Maximum iterations
pub fn secant<F>(
    f: F,
    mut x0: Decimal,
    mut x1: Decimal,
    tolerance: Option<Decimal>,
    max_iter: Option<u32>,
) -> Result<SolverResult, ArithmeticError>
where
    F: Fn(Decimal) -> Result<Decimal, ArithmeticError>,
{
    let tol = tolerance.unwrap_or_else(default_tolerance);
    let max = max_iter.unwrap_or(DEFAULT_MAX_ITER);

    let mut f0 = f(x0)?;
    let mut f1 = f(x1)?;

    let mut iterations = 0;

    while iterations < max {
        if f1.abs() < tol {
            return Ok(SolverResult {
                root: x1,
                iterations,
                residual: f1,
                converged: true,
            });
        }

        let df = f1.try_sub(f0)?;
        if df.abs() < Decimal::new(1, 20) {
            // Secant line is too flat
            return Ok(SolverResult {
                root: x1,
                iterations,
                residual: f1,
                converged: false,
            });
        }

        let dx = x1.try_sub(x0)?;
        let x2 = x1.try_sub(f1.try_mul(dx)?.try_div(df)?)?;

        x0 = x1;
        f0 = f1;
        x1 = x2;
        f1 = f(x1)?;

        iterations += 1;
    }

    Ok(SolverResult {
        root: x1,
        iterations,
        residual: f1,
        converged: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newton_sqrt2() {
        // Find sqrt(2) by solving x^2 - 2 = 0
        let f = |x: Decimal| x.try_mul(x).and_then(|x2| x2.try_sub(Decimal::from(2i64)));
        let df = |x: Decimal| x.try_mul(Decimal::from(2i64));

        let result = newton_raphson(f, df, Decimal::ONE, None, None).unwrap();

        assert!(result.converged);
        let sqrt2 = Decimal::from(2i64).sqrt().unwrap();
        let diff = (result.root - sqrt2).abs();
        assert!(diff < Decimal::new(1, 10));
    }

    #[test]
    fn test_newton_numerical() {
        // Find cube root of 8
        let f = |x: Decimal| {
            x.try_mul(x)
                .and_then(|x2| x2.try_mul(x))
                .and_then(|x3| x3.try_sub(Decimal::from(8i64)))
        };

        let result =
            newton_raphson_numerical(f, Decimal::from(2i64), None, None, None).unwrap();

        assert!(result.converged);
        let diff = (result.root - Decimal::from(2i64)).abs();
        assert!(diff < Decimal::new(1, 8));
    }

    #[test]
    fn test_bisection() {
        // Find root of x^2 - 2 = 0 in [1, 2]
        let f = |x: Decimal| x.try_mul(x).and_then(|x2| x2.try_sub(Decimal::from(2i64)));

        let result = bisection(f, Decimal::ONE, Decimal::from(2i64), None, None).unwrap();

        assert!(result.converged);
        let sqrt2 = Decimal::from(2i64).sqrt().unwrap();
        let diff = (result.root - sqrt2).abs();
        assert!(diff < Decimal::new(1, 10));
    }

    #[test]
    fn test_brent() {
        // Find cube root of 8
        let f = |x: Decimal| {
            x.try_mul(x)
                .and_then(|x2| x2.try_mul(x))
                .and_then(|x3| x3.try_sub(Decimal::from(8i64)))
        };

        let result = brent(f, Decimal::ONE, Decimal::from(3i64), None, None).unwrap();

        assert!(result.converged);
        let diff = (result.root - Decimal::from(2i64)).abs();
        assert!(diff < Decimal::new(1, 10));
    }

    #[test]
    fn test_secant() {
        // Find sqrt(2)
        let f = |x: Decimal| x.try_mul(x).and_then(|x2| x2.try_sub(Decimal::from(2i64)));

        let result = secant(
            f,
            Decimal::ONE,
            Decimal::from(2i64),
            None,
            None,
        )
        .unwrap();

        assert!(result.converged);
        let sqrt2 = Decimal::from(2i64).sqrt().unwrap();
        let diff = (result.root - sqrt2).abs();
        assert!(diff < Decimal::new(1, 10));
    }

    #[test]
    fn test_brent_vs_bisection_efficiency() {
        // Brent should converge faster than bisection
        let f = |x: Decimal| x.try_mul(x).and_then(|x2| x2.try_sub(Decimal::from(2i64)));

        let brent_result = brent(f, Decimal::ONE, Decimal::from(2i64), Some(Decimal::new(1, 10)), None).unwrap();
        let bisect_result = bisection(f, Decimal::ONE, Decimal::from(2i64), Some(Decimal::new(1, 10)), None).unwrap();

        // Brent should use fewer iterations
        assert!(brent_result.iterations <= bisect_result.iterations);
    }

    #[test]
    fn test_implied_rate_from_discount() {
        // Given discount factor D = exp(-r*t), find r
        // D = 0.95, t = 1 year -> r = -ln(0.95) ≈ 0.0513
        let d = Decimal::new(95, 2); // 0.95
        let t = Decimal::ONE;

        let f = |r: Decimal| {
            // f(r) = exp(-r*t) - D = 0
            let neg_rt = r.try_mul(t).map(|x| -x)?;
            neg_rt.try_exp().and_then(|exp_val| exp_val.try_sub(d))
        };

        let result = brent(f, Decimal::ZERO, Decimal::new(1, 1), None, None).unwrap();

        assert!(result.converged);
        // Expected: r ≈ 0.0513
        let expected = Decimal::new(513, 4); // 0.0513
        let diff = (result.root - expected).abs();
        assert!(diff < Decimal::new(1, 3)); // Within 0.001
    }
}
