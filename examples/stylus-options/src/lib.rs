#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use alloy_primitives::U256;
use financial_calc::options::{
    black_scholes_call, black_scholes_put, call_greeks, implied_volatility, put_greeks,
    OptionParams,
};
use precision_core::{Decimal, RoundingMode};
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct OptionsEngine {
        /// Default risk-free rate in basis points (e.g., 500 = 5%)
        uint256 risk_free_rate_bps;
    }
}

const SCALE: u64 = 1_000_000_000_000_000_000;
const BPS_DIVISOR: u64 = 10_000;

fn u256_to_decimal(value: U256) -> Decimal {
    let lo: u128 = value.as_limbs()[0] as u128 | ((value.as_limbs()[1] as u128) << 64);
    let raw = Decimal::from(lo);
    raw.checked_div(Decimal::from(SCALE))
        .unwrap_or(Decimal::MAX)
}

fn decimal_to_u256(value: Decimal) -> U256 {
    let scaled = value
        .checked_mul(Decimal::from(SCALE))
        .unwrap_or(Decimal::MAX)
        .round(0, RoundingMode::TowardZero);
    let (mantissa, _scale) = scaled.to_parts();
    U256::from(mantissa.unsigned_abs())
}

fn build_params(
    spot: U256,
    strike: U256,
    volatility: U256,
    time_to_expiry: U256,
    rate: Decimal,
) -> OptionParams {
    OptionParams {
        spot: u256_to_decimal(spot),
        strike: u256_to_decimal(strike),
        volatility: u256_to_decimal(volatility),
        time: u256_to_decimal(time_to_expiry),
        rate,
    }
}

#[public]
impl OptionsEngine {
    /// Price a European call option using Black-Scholes.
    ///
    /// spot: underlying price (1e18 scaled)
    /// strike: strike price (1e18 scaled)
    /// volatility: annualized vol (1e18 scaled, e.g., 0.2e18 = 20%)
    /// time_to_expiry: years to expiry (1e18 scaled, e.g., 0.25e18 = 3 months)
    ///
    /// Returns: call option price (1e18 scaled)
    pub fn price_call(
        &self,
        spot: U256,
        strike: U256,
        volatility: U256,
        time_to_expiry: U256,
    ) -> Result<U256, Vec<u8>> {
        let rate = self.get_rate()?;
        let params = build_params(spot, strike, volatility, time_to_expiry, rate);

        let price = black_scholes_call(&params).map_err(|_| b"bs calc error".to_vec())?;

        Ok(decimal_to_u256(price))
    }

    /// Price a European put option using Black-Scholes.
    ///
    /// Returns: put option price (1e18 scaled)
    pub fn price_put(
        &self,
        spot: U256,
        strike: U256,
        volatility: U256,
        time_to_expiry: U256,
    ) -> Result<U256, Vec<u8>> {
        let rate = self.get_rate()?;
        let params = build_params(spot, strike, volatility, time_to_expiry, rate);

        let price = black_scholes_put(&params).map_err(|_| b"bs calc error".to_vec())?;

        Ok(decimal_to_u256(price))
    }

    /// Calculate Greeks for a call option.
    ///
    /// Returns: (delta, gamma, theta, vega, rho) all scaled by 1e18
    pub fn call_option_greeks(
        &self,
        spot: U256,
        strike: U256,
        volatility: U256,
        time_to_expiry: U256,
    ) -> Result<(U256, U256, U256, U256, U256), Vec<u8>> {
        let rate = self.get_rate()?;
        let params = build_params(spot, strike, volatility, time_to_expiry, rate);

        let greeks = call_greeks(&params).map_err(|_| b"greeks calc error".to_vec())?;

        Ok((
            decimal_to_u256(greeks.delta),
            decimal_to_u256(greeks.gamma),
            decimal_to_u256(greeks.theta.abs()),
            decimal_to_u256(greeks.vega),
            decimal_to_u256(greeks.rho),
        ))
    }

    /// Calculate Greeks for a put option.
    ///
    /// Returns: (delta, gamma, theta, vega, rho) all scaled by 1e18
    pub fn put_option_greeks(
        &self,
        spot: U256,
        strike: U256,
        volatility: U256,
        time_to_expiry: U256,
    ) -> Result<(U256, U256, U256, U256, U256), Vec<u8>> {
        let rate = self.get_rate()?;
        let params = build_params(spot, strike, volatility, time_to_expiry, rate);

        let greeks = put_greeks(&params).map_err(|_| b"greeks calc error".to_vec())?;

        Ok((
            decimal_to_u256(greeks.delta.abs()),
            decimal_to_u256(greeks.gamma),
            decimal_to_u256(greeks.theta.abs()),
            decimal_to_u256(greeks.vega),
            decimal_to_u256(greeks.rho.abs()),
        ))
    }

    /// Calculate implied volatility from market price.
    ///
    /// market_price: observed option price (1e18 scaled)
    /// is_call: true for call, false for put
    ///
    /// Returns: implied volatility (1e18 scaled, e.g., 0.2e18 = 20%)
    pub fn calculate_iv(
        &self,
        spot: U256,
        strike: U256,
        time_to_expiry: U256,
        market_price: U256,
        is_call: bool,
    ) -> Result<U256, Vec<u8>> {
        let rate = self.get_rate()?;
        let params = build_params(
            spot,
            strike,
            U256::from(SCALE / 5), // initial guess: 20% vol
            time_to_expiry,
            rate,
        );

        let mp = u256_to_decimal(market_price);

        let tolerance = Decimal::new(1, 6); // 0.000001
        let iv = implied_volatility(mp, &params, is_call, 100, tolerance)
            .map_err(|_| b"iv calc error".to_vec())?;

        Ok(decimal_to_u256(iv))
    }

    /// Put-call parity check: C - P = S - K * e^(-rT)
    ///
    /// Returns the parity difference (should be near zero for fair prices).
    pub fn put_call_parity_check(
        &self,
        spot: U256,
        strike: U256,
        volatility: U256,
        time_to_expiry: U256,
    ) -> Result<U256, Vec<u8>> {
        let rate = self.get_rate()?;
        let params = build_params(spot, strike, volatility, time_to_expiry, rate);

        let call = black_scholes_call(&params).map_err(|_| b"call error".to_vec())?;
        let put = black_scholes_put(&params).map_err(|_| b"put error".to_vec())?;

        let s = u256_to_decimal(spot);
        let k = u256_to_decimal(strike);
        let t = u256_to_decimal(time_to_expiry);

        let neg_rt = (-rate)
            .checked_mul(t)
            .ok_or_else(|| b"overflow".to_vec())?;
        let discount = neg_rt.exp().ok_or_else(|| b"exp error".to_vec())?;
        let pv_strike = k
            .checked_mul(discount)
            .ok_or_else(|| b"overflow".to_vec())?;

        // C - P should equal S - K*e^(-rT)
        let lhs = call
            .checked_sub(put)
            .ok_or_else(|| b"underflow".to_vec())?;
        let rhs = s
            .checked_sub(pv_strike)
            .ok_or_else(|| b"underflow".to_vec())?;

        let diff = (lhs - rhs).abs();
        Ok(decimal_to_u256(diff))
    }

    /// Set the risk-free rate (admin only in production).
    pub fn set_risk_free_rate(&mut self, rate_bps: U256) {
        self.risk_free_rate_bps.set(rate_bps);
    }
}

impl OptionsEngine {
    fn get_rate(&self) -> Result<Decimal, Vec<u8>> {
        let rate_bps = u256_to_decimal(self.risk_free_rate_bps.get());
        rate_bps
            .checked_div(Decimal::from(BPS_DIVISOR))
            .ok_or_else(|| b"rate error".to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    const ONE_ETH: u128 = 1_000_000_000_000_000_000;

    #[test]
    fn test_atm_call_pricing() {
        let params = OptionParams {
            spot: Decimal::from(100i64),
            strike: Decimal::from(100i64),
            rate: Decimal::from_str("0.05").unwrap(),
            time: Decimal::from_str("0.25").unwrap(),
            volatility: Decimal::from_str("0.2").unwrap(),
        };

        let price = black_scholes_call(&params).unwrap();
        // ATM call with 20% vol, 3 months, 5% rate ≈ $3.80-$4.20
        assert!(price > Decimal::from(3i64));
        assert!(price < Decimal::from(5i64));
    }

    #[test]
    fn test_put_call_parity() {
        let params = OptionParams {
            spot: Decimal::from(100i64),
            strike: Decimal::from(100i64),
            rate: Decimal::from_str("0.05").unwrap(),
            time: Decimal::from_str("0.25").unwrap(),
            volatility: Decimal::from_str("0.2").unwrap(),
        };

        let call = black_scholes_call(&params).unwrap();
        let put = black_scholes_put(&params).unwrap();

        let neg_rt = (-params.rate)
            .checked_mul(params.time)
            .unwrap();
        let discount = neg_rt.exp().unwrap();
        let pv_strike = params.strike.checked_mul(discount).unwrap();

        // C - P = S - K*e^(-rT)
        let lhs = call.checked_sub(put).unwrap();
        let rhs = params.spot.checked_sub(pv_strike).unwrap();
        let diff = (lhs - rhs).abs();

        assert!(diff < Decimal::from_str("0.01").unwrap());
    }

    #[test]
    fn test_call_greeks_delta_bounds() {
        let params = OptionParams {
            spot: Decimal::from(100i64),
            strike: Decimal::from(100i64),
            rate: Decimal::from_str("0.05").unwrap(),
            time: Decimal::from_str("0.25").unwrap(),
            volatility: Decimal::from_str("0.2").unwrap(),
        };

        let greeks = call_greeks(&params).unwrap();

        // Call delta should be between 0 and 1
        assert!(greeks.delta > Decimal::ZERO);
        assert!(greeks.delta < Decimal::ONE);

        // ATM delta should be near 0.5
        let half = Decimal::from_str("0.5").unwrap();
        let diff = (greeks.delta - half).abs();
        assert!(diff < Decimal::from_str("0.1").unwrap());
    }

    #[test]
    fn test_otm_put_cheap() {
        let params = OptionParams {
            spot: Decimal::from(100i64),
            strike: Decimal::from(80i64), // 20% OTM
            rate: Decimal::from_str("0.05").unwrap(),
            time: Decimal::from_str("0.25").unwrap(),
            volatility: Decimal::from_str("0.2").unwrap(),
        };

        let put = black_scholes_put(&params).unwrap();
        // Deep OTM put should be very cheap
        assert!(put < Decimal::from(1i64));
    }

    #[test]
    fn test_itm_call_intrinsic_value() {
        let params = OptionParams {
            spot: Decimal::from(120i64),
            strike: Decimal::from(100i64), // 20% ITM
            rate: Decimal::from_str("0.05").unwrap(),
            time: Decimal::from_str("0.25").unwrap(),
            volatility: Decimal::from_str("0.2").unwrap(),
        };

        let call = black_scholes_call(&params).unwrap();
        let intrinsic = params.spot - params.strike;

        // ITM call must be worth at least intrinsic value
        assert!(call >= intrinsic);
    }
}
