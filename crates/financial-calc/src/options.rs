//! Black-Scholes options pricing and Greeks.
//!
//! This module provides deterministic options pricing calculations using
//! the Black-Scholes-Merton model. All calculations use fixed-point decimal
//! arithmetic for cross-platform consistency.
//!
//! # Example
//!
//! ```
//! use financial_calc::options::{black_scholes_call, OptionParams};
//! use precision_core::Decimal;
//! use core::str::FromStr;
//!
//! let params = OptionParams {
//!     spot: Decimal::from(100i64),
//!     strike: Decimal::from(100i64),
//!     rate: Decimal::from_str("0.05").unwrap(),  // 5% risk-free rate
//!     time: Decimal::from_str("0.25").unwrap(),  // 3 months
//!     volatility: Decimal::from_str("0.2").unwrap(), // 20% vol
//! };
//!
//! let call_price = black_scholes_call(&params).unwrap();
//! ```

use precision_core::{ArithmeticError, Decimal};

/// Parameters for Black-Scholes option pricing.
#[derive(Debug, Clone, Copy)]
pub struct OptionParams {
    /// Current price of the underlying asset.
    pub spot: Decimal,
    /// Strike price of the option.
    pub strike: Decimal,
    /// Risk-free interest rate (annualized, as decimal e.g., 0.05 for 5%).
    pub rate: Decimal,
    /// Time to expiration in years (e.g., 0.25 for 3 months).
    pub time: Decimal,
    /// Volatility (annualized, as decimal e.g., 0.2 for 20%).
    pub volatility: Decimal,
}

/// Greeks for an option position.
#[derive(Debug, Clone, Copy)]
pub struct Greeks {
    /// Rate of change of option price with respect to underlying price.
    pub delta: Decimal,
    /// Rate of change of delta with respect to underlying price.
    pub gamma: Decimal,
    /// Rate of change of option price with respect to time (per day).
    pub theta: Decimal,
    /// Rate of change of option price with respect to volatility.
    pub vega: Decimal,
    /// Rate of change of option price with respect to interest rate.
    pub rho: Decimal,
}

/// Standard normal cumulative distribution function.
///
/// Uses Hart approximation (1968) which provides high accuracy across the full range.
/// Maximum error: approximately 7.5×10⁻⁸.
pub fn normal_cdf(x: Decimal) -> Result<Decimal, ArithmeticError> {
    let zero = Decimal::ZERO;
    let one = Decimal::ONE;

    // For very large |x|, return boundary values
    if x > parse_const("8.0") {
        return Ok(one);
    }
    if x < parse_const("-8.0") {
        return Ok(zero);
    }

    // Use symmetry: for x < 0, N(x) = 1 - N(-x)
    let (abs_x, negate) = if x < zero { (-x, true) } else { (x, false) };

    // Horner's method coefficients for rational approximation
    // Based on Cody's rational Chebyshev approximation
    let a1 = parse_const("0.319381530");
    let a2 = parse_const("-0.356563782");
    let a3 = parse_const("1.781477937");
    let a4 = parse_const("-1.821255978");
    let a5 = parse_const("1.330274429");
    let p = parse_const("0.2316419");

    let k = one.try_div(one.try_add(p.try_mul(abs_x)?)?)?;

    // Polynomial: a1*k + a2*k² + a3*k³ + a4*k⁴ + a5*k⁵
    let poly = k.try_mul(a1.try_add(k.try_mul(
        a2.try_add(k.try_mul(a3.try_add(k.try_mul(a4.try_add(k.try_mul(a5)?)?)?)?)?)?,
    )?)?)?;

    // Standard normal PDF at x: (1/√(2π)) * exp(-x²/2)
    let two = Decimal::from(2i64);
    let neg_half_x_sq = abs_x
        .try_mul(abs_x)?
        .try_div(two)?
        .try_mul(Decimal::NEGATIVE_ONE)?;
    let exp_term = neg_half_x_sq.try_exp()?;

    let sqrt_two_pi = parse_const("2.5066282746310002"); // √(2π)
    let pdf = exp_term.try_div(sqrt_two_pi)?;

    // N(x) ≈ 1 - pdf * poly for x >= 0
    let n_x = one.try_sub(pdf.try_mul(poly)?)?;

    if negate {
        one.try_sub(n_x)
    } else {
        Ok(n_x)
    }
}

/// Standard normal probability density function.
pub fn normal_pdf(x: Decimal) -> Result<Decimal, ArithmeticError> {
    let two = Decimal::from(2i64);
    let two_pi = Decimal::pi().try_mul(two)?;
    let sqrt_two_pi = two_pi.try_sqrt()?;

    // exp(-x²/2) / sqrt(2π)
    let neg_half_x_sq = x.try_mul(x)?.try_div(two)?.try_mul(Decimal::NEGATIVE_ONE)?;
    let exp_term = neg_half_x_sq.try_exp()?;

    exp_term.try_div(sqrt_two_pi)
}

/// Calculates d1 and d2 parameters for Black-Scholes.
fn calculate_d1_d2(params: &OptionParams) -> Result<(Decimal, Decimal), ArithmeticError> {
    let two = Decimal::from(2i64);

    // sqrt(T)
    let sqrt_t = params.time.try_sqrt()?;

    // σ√T
    let vol_sqrt_t = params.volatility.try_mul(sqrt_t)?;

    // ln(S/K)
    let ln_s_k = params.spot.try_div(params.strike)?.try_ln()?;

    // (r + σ²/2)
    let vol_sq = params.volatility.try_mul(params.volatility)?;
    let vol_sq_half = vol_sq.try_div(two)?;
    let r_plus_vol = params.rate.try_add(vol_sq_half)?;

    // d1 = (ln(S/K) + (r + σ²/2)T) / (σ√T)
    let numerator = ln_s_k.try_add(r_plus_vol.try_mul(params.time)?)?;
    let d1 = numerator.try_div(vol_sqrt_t)?;

    // d2 = d1 - σ√T
    let d2 = d1.try_sub(vol_sqrt_t)?;

    Ok((d1, d2))
}

/// Calculates the Black-Scholes price for a European call option.
///
/// # Arguments
///
/// * `params` - Option parameters (spot, strike, rate, time, volatility)
///
/// # Returns
///
/// The theoretical call option price.
pub fn black_scholes_call(params: &OptionParams) -> Result<Decimal, ArithmeticError> {
    validate_params(params)?;

    let (d1, d2) = calculate_d1_d2(params)?;

    let n_d1 = normal_cdf(d1)?;
    let n_d2 = normal_cdf(d2)?;

    // Discount factor: e^(-rT)
    let neg_rt = params
        .rate
        .try_mul(params.time)?
        .try_mul(Decimal::NEGATIVE_ONE)?;
    let discount = neg_rt.try_exp()?;

    // C = S * N(d1) - K * e^(-rT) * N(d2)
    let term1 = params.spot.try_mul(n_d1)?;
    let term2 = params.strike.try_mul(discount)?.try_mul(n_d2)?;

    term1.try_sub(term2)
}

/// Calculates the Black-Scholes price for a European put option.
///
/// # Arguments
///
/// * `params` - Option parameters (spot, strike, rate, time, volatility)
///
/// # Returns
///
/// The theoretical put option price.
pub fn black_scholes_put(params: &OptionParams) -> Result<Decimal, ArithmeticError> {
    validate_params(params)?;

    let (d1, d2) = calculate_d1_d2(params)?;

    let n_neg_d1 = normal_cdf(-d1)?;
    let n_neg_d2 = normal_cdf(-d2)?;

    // Discount factor: e^(-rT)
    let neg_rt = params
        .rate
        .try_mul(params.time)?
        .try_mul(Decimal::NEGATIVE_ONE)?;
    let discount = neg_rt.try_exp()?;

    // P = K * e^(-rT) * N(-d2) - S * N(-d1)
    let term1 = params.strike.try_mul(discount)?.try_mul(n_neg_d2)?;
    let term2 = params.spot.try_mul(n_neg_d1)?;

    term1.try_sub(term2)
}

/// Calculates the Greeks for a call option.
pub fn call_greeks(params: &OptionParams) -> Result<Greeks, ArithmeticError> {
    validate_params(params)?;

    let (d1, d2) = calculate_d1_d2(params)?;
    let sqrt_t = params.time.try_sqrt()?;

    let n_d1 = normal_cdf(d1)?;
    let n_d2 = normal_cdf(d2)?;
    let n_prime_d1 = normal_pdf(d1)?;

    // Delta = N(d1)
    let delta = n_d1;

    // Gamma = N'(d1) / (S * σ * √T)
    let gamma_denom = params.spot.try_mul(params.volatility)?.try_mul(sqrt_t)?;
    let gamma = n_prime_d1.try_div(gamma_denom)?;

    // Vega = S * √T * N'(d1)
    let vega = params.spot.try_mul(sqrt_t)?.try_mul(n_prime_d1)?;
    // Convert to per 1% move (standard convention)
    let vega = vega.try_div(Decimal::from(100i64))?;

    // Discount factor
    let neg_rt = params
        .rate
        .try_mul(params.time)?
        .try_mul(Decimal::NEGATIVE_ONE)?;
    let discount = neg_rt.try_exp()?;

    // Theta = -(S * N'(d1) * σ) / (2√T) - r * K * e^(-rT) * N(d2)
    let two = Decimal::from(2i64);
    let theta_term1 = params
        .spot
        .try_mul(n_prime_d1)?
        .try_mul(params.volatility)?
        .try_div(two.try_mul(sqrt_t)?)?;
    let theta_term2 = params
        .rate
        .try_mul(params.strike)?
        .try_mul(discount)?
        .try_mul(n_d2)?;
    let theta = theta_term1
        .try_add(theta_term2)?
        .try_mul(Decimal::NEGATIVE_ONE)?;
    // Convert to per-day (divide by 365)
    let theta = theta.try_div(Decimal::from(365i64))?;

    // Rho = K * T * e^(-rT) * N(d2)
    let rho = params
        .strike
        .try_mul(params.time)?
        .try_mul(discount)?
        .try_mul(n_d2)?;
    // Convert to per 1% move
    let rho = rho.try_div(Decimal::from(100i64))?;

    Ok(Greeks {
        delta,
        gamma,
        theta,
        vega,
        rho,
    })
}

/// Calculates the Greeks for a put option.
pub fn put_greeks(params: &OptionParams) -> Result<Greeks, ArithmeticError> {
    validate_params(params)?;

    let (d1, d2) = calculate_d1_d2(params)?;
    let sqrt_t = params.time.try_sqrt()?;

    let n_neg_d1 = normal_cdf(-d1)?;
    let n_neg_d2 = normal_cdf(-d2)?;
    let n_prime_d1 = normal_pdf(d1)?;

    // Delta = N(d1) - 1 = -N(-d1)
    let delta = n_neg_d1.try_mul(Decimal::NEGATIVE_ONE)?;

    // Gamma is same for call and put
    let gamma_denom = params.spot.try_mul(params.volatility)?.try_mul(sqrt_t)?;
    let gamma = n_prime_d1.try_div(gamma_denom)?;

    // Vega is same for call and put
    let vega = params.spot.try_mul(sqrt_t)?.try_mul(n_prime_d1)?;
    let vega = vega.try_div(Decimal::from(100i64))?;

    // Discount factor
    let neg_rt = params
        .rate
        .try_mul(params.time)?
        .try_mul(Decimal::NEGATIVE_ONE)?;
    let discount = neg_rt.try_exp()?;

    // Theta = -(S * N'(d1) * σ) / (2√T) + r * K * e^(-rT) * N(-d2)
    let two = Decimal::from(2i64);
    let theta_term1 = params
        .spot
        .try_mul(n_prime_d1)?
        .try_mul(params.volatility)?
        .try_div(two.try_mul(sqrt_t)?)?;
    let theta_term2 = params
        .rate
        .try_mul(params.strike)?
        .try_mul(discount)?
        .try_mul(n_neg_d2)?;
    let theta = theta_term2.try_sub(theta_term1)?;
    let theta = theta.try_div(Decimal::from(365i64))?;

    // Rho = -K * T * e^(-rT) * N(-d2)
    let rho = params
        .strike
        .try_mul(params.time)?
        .try_mul(discount)?
        .try_mul(n_neg_d2)?
        .try_mul(Decimal::NEGATIVE_ONE)?;
    let rho = rho.try_div(Decimal::from(100i64))?;

    Ok(Greeks {
        delta,
        gamma,
        theta,
        vega,
        rho,
    })
}

/// Calculates implied volatility using Newton-Raphson iteration.
///
/// # Arguments
///
/// * `market_price` - The observed market price of the option
/// * `params` - Option parameters (volatility field is ignored as initial guess)
/// * `is_call` - True for call option, false for put
/// * `max_iterations` - Maximum number of iterations (default: 100)
/// * `tolerance` - Convergence tolerance (default: 0.0001)
///
/// # Returns
///
/// The implied volatility as a decimal (e.g., 0.20 for 20%).
pub fn implied_volatility(
    market_price: Decimal,
    params: &OptionParams,
    is_call: bool,
    max_iterations: Option<u32>,
    tolerance: Option<Decimal>,
) -> Result<Decimal, ArithmeticError> {
    let max_iter = max_iterations.unwrap_or(100);
    let tol = tolerance.unwrap_or_else(|| parse_const("0.0001"));

    // Initial guess using Brenner-Subrahmanyam approximation
    // σ ≈ √(2π/T) * (C/S)
    let two_pi = Decimal::pi().try_mul(Decimal::from(2i64))?;
    let sqrt_two_pi_over_t = two_pi.try_div(params.time)?.try_sqrt()?;
    let mut sigma = sqrt_two_pi_over_t.try_mul(market_price.try_div(params.spot)?)?;

    // Clamp initial guess to reasonable range
    let min_vol = parse_const("0.01");
    let max_vol = parse_const("5.0");
    sigma = sigma.max(min_vol).min(max_vol);

    // Newton-Raphson iteration
    for _ in 0..max_iter {
        let mut iter_params = *params;
        iter_params.volatility = sigma;

        let price = if is_call {
            black_scholes_call(&iter_params)?
        } else {
            black_scholes_put(&iter_params)?
        };

        let diff = price.try_sub(market_price)?;

        // Check convergence
        if diff.abs() < tol {
            return Ok(sigma);
        }

        // Vega = ∂C/∂σ = S * √T * N'(d1)
        let (d1, _) = calculate_d1_d2(&iter_params)?;
        let sqrt_t = params.time.try_sqrt()?;
        let n_prime_d1 = normal_pdf(d1)?;
        let vega = params.spot.try_mul(sqrt_t)?.try_mul(n_prime_d1)?;

        // Avoid division by zero
        if vega.abs() < parse_const("0.00000001") {
            break;
        }

        // Newton-Raphson update: σ_new = σ - (C(σ) - C_market) / vega
        let adjustment = diff.try_div(vega)?;
        sigma = sigma.try_sub(adjustment)?;

        // Keep sigma in valid range
        sigma = sigma.max(min_vol).min(max_vol);
    }

    Ok(sigma)
}

fn validate_params(params: &OptionParams) -> Result<(), ArithmeticError> {
    if params.spot <= Decimal::ZERO {
        return Err(ArithmeticError::LogOfNegative);
    }
    if params.strike <= Decimal::ZERO {
        return Err(ArithmeticError::LogOfNegative);
    }
    if params.time <= Decimal::ZERO {
        return Err(ArithmeticError::NegativeSqrt);
    }
    if params.volatility <= Decimal::ZERO {
        return Err(ArithmeticError::LogOfNegative);
    }
    Ok(())
}

fn parse_const(s: &str) -> Decimal {
    s.parse().expect("Invalid constant")
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    fn decimal(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_normal_cdf_standard_values() {
        // N(0) = 0.5
        let n_0 = normal_cdf(Decimal::ZERO).unwrap();
        assert!((n_0 - decimal("0.5")).abs() < decimal("0.0001"));

        // N(-∞) → 0, N(∞) → 1
        // Test with large values
        let n_neg_3 = normal_cdf(decimal("-3")).unwrap();
        assert!(n_neg_3 < decimal("0.01"));

        let n_pos_3 = normal_cdf(decimal("3")).unwrap();
        assert!(n_pos_3 > decimal("0.99"));
    }

    #[test]
    fn test_normal_cdf_symmetry() {
        // N(x) + N(-x) = 1
        let x = decimal("1.5");
        let n_x = normal_cdf(x).unwrap();
        let n_neg_x = normal_cdf(-x).unwrap();
        let sum = n_x + n_neg_x;
        assert!((sum - Decimal::ONE).abs() < decimal("0.0001"));
    }

    #[test]
    fn test_black_scholes_atm_call() {
        // At-the-money call with standard parameters
        let params = OptionParams {
            spot: Decimal::from(100i64),
            strike: Decimal::from(100i64),
            rate: decimal("0.05"),
            time: decimal("1.0"), // 1 year
            volatility: decimal("0.2"),
        };

        let price = black_scholes_call(&params).unwrap();

        // Expected price is approximately 10.45 for these parameters
        assert!(price > decimal("9"));
        assert!(price < decimal("12"));
    }

    #[test]
    fn test_put_call_parity() {
        // C - P = S - K * e^(-rT)
        let params = OptionParams {
            spot: Decimal::from(100i64),
            strike: Decimal::from(95i64),
            rate: decimal("0.05"),
            time: decimal("0.5"),
            volatility: decimal("0.25"),
        };

        let call = black_scholes_call(&params).unwrap();
        let put = black_scholes_put(&params).unwrap();

        let neg_rt = params.rate * params.time * Decimal::NEGATIVE_ONE;
        let discount = neg_rt.exp().unwrap();
        let pv_strike = params.strike * discount;

        let lhs = call - put;
        let rhs = params.spot - pv_strike;

        assert!((lhs - rhs).abs() < decimal("0.01"));
    }

    #[test]
    fn test_call_delta_bounds() {
        let params = OptionParams {
            spot: Decimal::from(100i64),
            strike: Decimal::from(100i64),
            rate: decimal("0.05"),
            time: decimal("0.25"),
            volatility: decimal("0.2"),
        };

        let greeks = call_greeks(&params).unwrap();

        // Call delta should be between 0 and 1
        assert!(greeks.delta >= Decimal::ZERO);
        assert!(greeks.delta <= Decimal::ONE);

        // ATM delta should be around 0.5 (slightly higher due to drift)
        assert!(greeks.delta > decimal("0.4"));
        assert!(greeks.delta < decimal("0.7"));
    }

    #[test]
    fn test_gamma_positive() {
        let params = OptionParams {
            spot: Decimal::from(100i64),
            strike: Decimal::from(100i64),
            rate: decimal("0.05"),
            time: decimal("0.25"),
            volatility: decimal("0.2"),
        };

        let call_g = call_greeks(&params).unwrap();
        let put_g = put_greeks(&params).unwrap();

        // Gamma should be positive and equal for calls and puts
        assert!(call_g.gamma > Decimal::ZERO);
        assert!((call_g.gamma - put_g.gamma).abs() < decimal("0.0001"));
    }

    #[test]
    fn test_implied_volatility_recovery() {
        let true_vol = decimal("0.25");
        let params = OptionParams {
            spot: Decimal::from(100i64),
            strike: Decimal::from(105i64),
            rate: decimal("0.05"),
            time: decimal("0.5"),
            volatility: true_vol,
        };

        let price = black_scholes_call(&params).unwrap();

        // Recover implied volatility from price
        let iv = implied_volatility(price, &params, true, None, None).unwrap();

        assert!((iv - true_vol).abs() < decimal("0.001"));
    }
}
