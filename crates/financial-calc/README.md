# financial-calc

[![Crates.io](https://img.shields.io/crates/v/financial-calc.svg)](https://crates.io/crates/financial-calc)
[![Documentation](https://docs.rs/financial-calc/badge.svg)](https://docs.rs/financial-calc)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../../LICENSE-MIT)

Financial calculation functions built on precision-core.

## Features

- Interest calculations (simple, compound, effective annual rate)
- Time value of money (future value, present value, NPV)
- Black-Scholes options pricing with Greeks
- Percentage operations
- `no_std` compatible
- Deterministic results

## Installation

```toml
[dependencies]
financial-calc = "0.1"
```

## Quick Start

```rust
use financial_calc::{compound_interest, future_value, Decimal};

let principal = Decimal::from(10000i64);
let rate = Decimal::new(5, 2);  // 5%

// Compound interest: monthly for 5 years
let interest = compound_interest(principal, rate, 12, 5)?;

// Future value
let fv = future_value(principal, rate, 10)?;
```

## Options Pricing

```rust
use financial_calc::options::{OptionParams, OptionType, black_scholes_price, calculate_greeks};
use precision_core::Decimal;

let params = OptionParams {
    spot: Decimal::from(100i64),
    strike: Decimal::from(100i64),
    rate: Decimal::new(5, 2),         // 5% risk-free rate
    volatility: Decimal::new(20, 2),  // 20% volatility
    time: Decimal::new(25, 2),        // 0.25 years (3 months)
};

// Calculate option price
let call_price = black_scholes_price(&params, OptionType::Call)?;
let put_price = black_scholes_price(&params, OptionType::Put)?;

// Calculate Greeks
let greeks = calculate_greeks(&params, OptionType::Call)?;
println!("Delta: {}", greeks.delta);
println!("Gamma: {}", greeks.gamma);
println!("Theta: {}", greeks.theta);
println!("Vega: {}", greeks.vega);
```

## Functions

### Interest
- `simple_interest(principal, rate, time)`
- `compound_interest(principal, rate, periods_per_year, years)`
- `effective_annual_rate(nominal_rate, periods)`

### Time Value
- `future_value(present_value, rate, periods)`
- `present_value(future_value, rate, periods)`
- `net_present_value(rate, cash_flows)`

### Options (Black-Scholes)
- `black_scholes_price(params, option_type)` - Call/Put price
- `calculate_greeks(params, option_type)` - Delta, Gamma, Theta, Vega, Rho
- `implied_volatility(params, market_price, option_type)` - Newton-Raphson IV solver

### Percentages
- `percentage_of(value, percent)`
- `percentage_change(old, new)`
- `basis_points_to_decimal(bps)`

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
