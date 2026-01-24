# risk-metrics

Risk metrics and calculations for DeFi applications.

## Features

- Health factor calculations
- Liquidation price and threshold
- Position sizing and LTV
- Pool utilization metrics
- `no_std` compatible
- Deterministic results

## Usage

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
- `health_factor(collateral, debt, threshold)`
- `is_healthy(collateral, debt, threshold)`
- `collateral_ratio(collateral, debt)`

### Liquidation
- `liquidation_price(collateral_amount, debt, threshold)`
- `liquidation_threshold(collateral, debt, health_factor)`
- `max_borrowable(collateral, threshold, min_health_factor)`

### Position
- `loan_to_value(debt, collateral)`
- `utilization_rate(borrows, liquidity)`
- `available_liquidity(total_liquidity, borrows)`

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
