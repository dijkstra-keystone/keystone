# Derivatives & Perpetuals

Perpetual futures calculations for protocols like GMX, Vertex, and Vela.

## Overview

The `financial-calc::derivatives` module provides:

- PnL calculations for perpetual positions
- Liquidation price and distance
- Funding rate calculations
- Margin and leverage metrics
- Breakeven price accounting for fees

## Position Structure

```rust
use financial_calc::derivatives::PerpPosition;
use precision_core::Decimal;

let position = PerpPosition {
    size: Decimal::from_str("1.5")?,        // 1.5 ETH
    entry_price: Decimal::from(2000i64),    // $2000
    is_long: true,
    leverage: Decimal::from(10i64),         // 10x
    collateral: Decimal::from(300i64),      // $300
};
```

## PnL Calculations

Calculate profit/loss:

```rust
use financial_calc::derivatives::{calculate_pnl, calculate_pnl_percentage, calculate_roe};

let current_price = Decimal::from(2200i64);

// Absolute PnL
let pnl = calculate_pnl(&position, current_price)?;
// Long: (2200 - 2000) * 1.5 = $300 profit

// PnL as percentage of collateral
let pnl_pct = calculate_pnl_percentage(&position, current_price)?;
// 300 / 300 = 100%

// Return on Equity
let roe = calculate_roe(&position, current_price)?;
// 100%
```

## Liquidation

Calculate liquidation price:

```rust
use financial_calc::derivatives::{calculate_liquidation_price, calculate_liquidation_distance};

let maintenance_margin_rate = Decimal::from_str("0.01")?; // 1%

let liq_price = calculate_liquidation_price(&position, maintenance_margin_rate)?;
// For long: entry - (collateral - maintenance) / size

let distance = calculate_liquidation_distance(
    &position,
    current_price,
    maintenance_margin_rate,
)?;
// Returns percentage distance to liquidation
```

## Funding Rate

Calculate funding rate using mark-index premium model:

```rust
use financial_calc::derivatives::{calculate_funding_rate, FundingParams};

let params = FundingParams {
    mark_price: Decimal::from(2020i64),   // Mark above index
    index_price: Decimal::from(2000i64),
    interest_rate: Decimal::ZERO,
    premium_cap: Decimal::from_str("0.01")?, // 1% cap
    funding_interval_hours: Decimal::from(8i64),
};

let rate = calculate_funding_rate(&params)?;
// Premium = (2020 - 2000) / 2000 = 1%
```

Calculate funding payment:

```rust
use financial_calc::derivatives::calculate_funding_payment;

let payment = calculate_funding_payment(
    &position,
    mark_price,
    funding_rate,
)?;
// Positive = receive, Negative = pay
// Longs pay positive funding, shorts receive
```

## Leverage Metrics

Track effective leverage as price moves:

```rust
use financial_calc::derivatives::{calculate_effective_leverage, calculate_margin_ratio};

let eff_leverage = calculate_effective_leverage(&position, current_price)?;
// Notional / (Collateral + Unrealized PnL)

let margin_ratio = calculate_margin_ratio(&position, current_price)?;
// Inverse of effective leverage
```

## Position Sizing

Calculate max position or required collateral:

```rust
use financial_calc::derivatives::{calculate_max_position_size, calculate_required_collateral};

let max_size = calculate_max_position_size(
    collateral,
    leverage,
    entry_price,
)?;

let required = calculate_required_collateral(
    size,
    entry_price,
    leverage,
)?;
```

## Breakeven Price

Account for trading fees:

```rust
use financial_calc::derivatives::calculate_breakeven_price;

let breakeven = calculate_breakeven_price(
    &position,
    Decimal::from_str("0.001")?, // 0.1% open fee
    Decimal::from_str("0.001")?, // 0.1% close fee
)?;
```

## Average Entry Price

When adding to an existing position:

```rust
use financial_calc::derivatives::calculate_average_entry_price;

let avg = calculate_average_entry_price(
    existing_size,
    existing_avg_price,
    additional_size,
    additional_price,
)?;
```
