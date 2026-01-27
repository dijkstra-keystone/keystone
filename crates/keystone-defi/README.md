# keystone-defi

Unified DeFi computation SDK for Arbitrum Stylus.

## Overview

Single integration point combining precision arithmetic, financial calculations, and risk metrics for DeFi protocols on Arbitrum.

## Modules

- **precision** - 128-bit decimal arithmetic with 28-digit precision
- **lending** - Health factor, liquidation, collateral calculations
- **amm** - Swap output, concentrated liquidity, impermanent loss
- **vault** - ERC4626 share/asset math, compounding, APY
- **derivatives** - Perpetual futures, funding rates, margin
- **options** - Black-Scholes pricing, Greeks

## Usage

```rust
use keystone_defi::prelude::*;
use core::str::FromStr;

// Lending
let health = health_factor(
    Decimal::from_str("10000").unwrap(),
    Decimal::from_str("5000").unwrap(),
    Decimal::from_str("0.8").unwrap(),
).unwrap();

// AMM
let output = calculate_swap_output(
    Decimal::from(1000000i64),
    Decimal::from(1000000i64),
    Decimal::from(1000i64),
    Decimal::from(30i64),
).unwrap();

// Derivatives
let position = PerpPosition {
    size: Decimal::from_str("1.5").unwrap(),
    entry_price: Decimal::from(2000i64),
    is_long: true,
    leverage: Decimal::from(10i64),
    collateral: Decimal::from(300i64),
};
let pnl = calculate_pnl(&position, Decimal::from(2200i64)).unwrap();
```

## Stylus Integration

All types are `no_std` compatible for Arbitrum Stylus smart contracts.

## License

MIT OR Apache-2.0
