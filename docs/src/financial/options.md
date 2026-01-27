# Options Pricing

Black-Scholes options pricing and Greeks calculations.

## Overview

The `financial-calc` crate provides complete Black-Scholes-Merton model implementation including:

- European call/put pricing
- All five Greeks (delta, gamma, theta, vega, rho)
- Implied volatility calculation
- Standard normal CDF/PDF

## Basic Usage

```rust
use financial_calc::options::{black_scholes_call, OptionParams};
use precision_core::Decimal;
use core::str::FromStr;

let params = OptionParams {
    spot: Decimal::from(100i64),           // Current price
    strike: Decimal::from(100i64),         // Strike price
    rate: Decimal::from_str("0.05")?,      // 5% risk-free rate
    time: Decimal::from_str("0.25")?,      // 3 months to expiry
    volatility: Decimal::from_str("0.2")?, // 20% annualized vol
};

let call_price = black_scholes_call(&params)?;
```

## Greeks

Calculate all Greeks for risk management:

```rust
use financial_calc::options::{call_greeks, put_greeks};

let greeks = call_greeks(&params)?;

println!("Delta: {}", greeks.delta);  // Price sensitivity
println!("Gamma: {}", greeks.gamma);  // Delta sensitivity
println!("Theta: {}", greeks.theta);  // Time decay (per day)
println!("Vega: {}", greeks.vega);    // Vol sensitivity (per 1%)
println!("Rho: {}", greeks.rho);      // Rate sensitivity (per 1%)
```

## Implied Volatility

Recover implied volatility from market prices using Newton-Raphson:

```rust
use financial_calc::options::implied_volatility;

let market_price = Decimal::from_str("10.5")?;
let iv = implied_volatility(
    market_price,
    &params,
    true,  // is_call
    None,  // max_iterations (default: 100)
    None,  // tolerance (default: 0.0001)
)?;
```

## Put-Call Parity

The implementation satisfies put-call parity:

```
C - P = S - K * e^(-rT)
```

Both call and put prices are internally consistent.

## Accuracy

- Uses Hart approximation (1968) for normal CDF
- Maximum CDF error: ~7.5×10⁻⁸
- All calculations use deterministic decimal arithmetic
