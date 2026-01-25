# risk-metrics

[![Crates.io](https://img.shields.io/crates/v/risk-metrics.svg)](https://crates.io/crates/risk-metrics)
[![Documentation](https://docs.rs/risk-metrics/badge.svg)](https://docs.rs/risk-metrics)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../../LICENSE-MIT)

Risk metrics and calculations for DeFi applications.

## Features

- Health factor calculations
- Liquidation price and threshold
- Position sizing and LTV
- Pool utilization metrics
- `no_std` compatible
- Deterministic results

## Installation

```toml
[dependencies]
risk-metrics = "0.1"
```

## Quick Start

```rust
use risk_metrics::{health_factor, liquidation_price, Decimal};

let collateral = Decimal::from(10000i64);
let debt = Decimal::from(5000i64);
let threshold = Decimal::new(80, 2);  // 80%

// Health factor: (collateral * threshold) / debt
let hf = health_factor(collateral, debt, threshold)?;  // 1.6

// Liquidation price
let liq = liquidation_price(
    Decimal::from(5i64),  // 5 ETH collateral
    debt,
    threshold,
)?;  // $1,250 per ETH
```

## Functions

### Health
- `health_factor(collateral, debt, threshold)` - Calculate position health
- `is_healthy(collateral, debt, threshold)` - Check if position is safe
- `collateral_ratio(collateral, debt)` - Raw collateralization ratio

### Liquidation
- `liquidation_price(collateral_amount, debt, threshold)` - Price at which liquidation occurs
- `liquidation_threshold(collateral, debt, health_factor)` - Threshold for given health factor
- `max_borrowable(collateral, threshold, min_health_factor)` - Maximum safe debt

### Position
- `loan_to_value(debt, collateral)` - LTV ratio
- `utilization_rate(borrows, liquidity)` - Pool utilization
- `available_liquidity(total_liquidity, borrows)` - Remaining liquidity

## DeFi Protocol Compatibility

Designed for integration with lending protocols:
- Aave-style health factor calculations
- Compound-style collateral ratios
- MakerDAO-style liquidation thresholds

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
