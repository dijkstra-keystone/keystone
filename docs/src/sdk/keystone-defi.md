# keystone-defi SDK

Unified DeFi computation SDK combining all Keystone modules.

## Installation

```toml
[dependencies]
keystone-defi = "0.1.0-alpha.3"
```

## Overview

`keystone-defi` provides a single integration point for DeFi protocols:

| Module | Use Case |
|--------|----------|
| `precision` | 128-bit decimal arithmetic |
| `lending` | Health factor, liquidation, collateral |
| `amm` | Swaps, liquidity, price impact |
| `vault` | ERC4626 shares, compounding, APY |
| `derivatives` | Perpetuals, funding, margin |
| `options` | Black-Scholes, Greeks |

## Quick Start

Use the prelude for common imports:

```rust
use keystone_defi::prelude::*;
use core::str::FromStr;

// Lending: Health factor
let hf = health_factor(
    Decimal::from_str("10000")?,  // collateral
    Decimal::from_str("5000")?,   // debt
    Decimal::from_str("0.8")?,    // threshold
)?;

// AMM: Swap output
let output = calculate_swap_output(
    Decimal::from(1_000_000i64),
    Decimal::from(1_000_000i64),
    Decimal::from(1000i64),
    Decimal::from(30i64),
)?;

// Vault: Share price
let price = calculate_share_price(
    Decimal::from(1_050_000i64),  // total assets
    Decimal::from(1_000_000i64),  // total supply
)?;

// Derivatives: PnL
let position = PerpPosition {
    size: Decimal::from_str("1.5")?,
    entry_price: Decimal::from(2000i64),
    is_long: true,
    leverage: Decimal::from(10i64),
    collateral: Decimal::from(300i64),
};
let pnl = calculate_pnl(&position, Decimal::from(2200i64))?;
```

## Modules

### precision

Core decimal type and arithmetic:

```rust
use keystone_defi::precision::{Decimal, RoundingMode, ArithmeticError};

let a = Decimal::from_str("123.456")?;
let b = Decimal::from_str("789.012")?;
let sum = a.try_add(b)?;
```

### lending

Risk metrics for lending protocols:

```rust
use keystone_defi::lending::*;

let hf = health_factor(collateral, debt, threshold)?;
let liq = liquidation_price(collateral, debt, price, threshold)?;
let max = max_borrowable(collateral, threshold, current_debt)?;
let healthy = is_healthy(collateral, debt, threshold)?;
```

### amm

DEX and AMM calculations:

```rust
use keystone_defi::amm::*;

// Constant product
let out = calculate_swap_output(reserve_in, reserve_out, amount_in, fee_bps)?;
let impact = calculate_price_impact(reserve_in, reserve_out, amount_in)?;

// Concentrated liquidity
let sqrt_price = tick_to_sqrt_price(tick)?;
let liquidity = calculate_liquidity_from_amounts(...)?;
let il = calculate_impermanent_loss(...)?;
```

### vault

Yield vault calculations:

```rust
use keystone_defi::vault::*;

let shares = calculate_shares_for_deposit(assets, total_assets, total_supply)?;
let assets = calculate_assets_for_redeem(shares, total_assets, total_supply)?;
let price = calculate_share_price(total_assets, total_supply)?;
let apy = calculate_apy_from_apr(apr, 365)?; // Daily compounding
let fee = calculate_performance_fee(gains, fee_bps)?;
```

### derivatives

Perpetual futures:

```rust
use keystone_defi::derivatives::*;

let pnl = calculate_pnl(&position, current_price)?;
let liq = calculate_liquidation_price(&position, maintenance_rate)?;
let funding = calculate_funding_rate(&funding_params)?;
let leverage = calculate_effective_leverage(&position, current_price)?;
```

### options

Options pricing:

```rust
use keystone_defi::options::*;

let call = black_scholes_call(&params)?;
let put = black_scholes_put(&params)?;
let greeks = call_greeks(&params)?;
let iv = implied_volatility(market_price, &params, true, None, None)?;
```

## Stylus Integration

All types are `no_std` compatible:

```rust
#![cfg_attr(not(feature = "export-abi"), no_main, no_std)]
use keystone_defi::prelude::*;
use stylus_sdk::prelude::*;

#[public]
impl MyContract {
    pub fn health_check(&self, collateral: U256, debt: U256) -> Result<U256, Vec<u8>> {
        let c = u256_to_decimal(collateral);
        let d = u256_to_decimal(debt);
        let threshold = Decimal::from_str("0.8").unwrap();

        let hf = health_factor(c, d, threshold)
            .map_err(|_| b"calc error".to_vec())?;

        Ok(decimal_to_u256(hf))
    }
}
```

## Feature Flags

```toml
[dependencies]
keystone-defi = { version = "0.1.0-alpha.3", default-features = false }

# Enable std library
keystone-defi = { version = "0.1.0-alpha.3", features = ["std"] }
```
