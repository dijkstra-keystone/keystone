# AMM & DEX Calculations

Automated Market Maker calculations including concentrated liquidity (Uniswap V3-style).

## Overview

The `financial-calc::amm` module provides:

- Constant product (x*y=k) swap calculations
- Concentrated liquidity math
- Tick and sqrt price conversions
- Impermanent loss calculations
- Liquidity provision math

## Constant Product Swaps

Calculate output amount for a swap:

```rust
use financial_calc::amm::calculate_swap_output;
use precision_core::Decimal;

let output = calculate_swap_output(
    Decimal::from(1_000_000i64),  // reserve_in
    Decimal::from(1_000_000i64),  // reserve_out
    Decimal::from(1_000i64),      // amount_in
    Decimal::from(30i64),         // fee_bps (0.3%)
)?;
```

Calculate required input for desired output:

```rust
use financial_calc::amm::calculate_swap_input;

let input = calculate_swap_input(
    Decimal::from(1_000_000i64),  // reserve_in
    Decimal::from(1_000_000i64),  // reserve_out
    Decimal::from(1_000i64),      // amount_out
    Decimal::from(30i64),         // fee_bps
)?;
```

## Price Impact

Calculate price impact of a trade:

```rust
use financial_calc::amm::calculate_price_impact;

let impact = calculate_price_impact(
    Decimal::from(1_000_000i64),  // reserve_in
    Decimal::from(1_000_000i64),  // reserve_out
    Decimal::from(10_000i64),     // amount_in (1% of reserves)
)?;
// Returns ~0.01 (1% impact)
```

## Concentrated Liquidity

### Tick Math

Convert between ticks and sqrt prices:

```rust
use financial_calc::amm::{tick_to_sqrt_price, sqrt_price_to_tick};

// Tick to sqrt price
let sqrt_price = tick_to_sqrt_price(1000)?;

// Sqrt price to tick
let tick = sqrt_price_to_tick(sqrt_price)?;
```

Tick spacing constants:

```rust
use financial_calc::amm::{TICK_SPACING_LOW, TICK_SPACING_MEDIUM, TICK_SPACING_HIGH};

// 0.05% fee tier: spacing = 10
// 0.30% fee tier: spacing = 60
// 1.00% fee tier: spacing = 200
```

### Liquidity Calculations

Calculate liquidity from token amounts:

```rust
use financial_calc::amm::calculate_liquidity_from_amounts;

let liquidity = calculate_liquidity_from_amounts(
    sqrt_price_current,
    sqrt_price_lower,
    sqrt_price_upper,
    amount_0,
    amount_1,
)?;
```

Calculate token amounts from liquidity:

```rust
use financial_calc::amm::calculate_amounts_from_liquidity;

let (amount_0, amount_1) = calculate_amounts_from_liquidity(
    sqrt_price_current,
    sqrt_price_lower,
    sqrt_price_upper,
    liquidity,
)?;
```

### Position Value

Calculate current value of a concentrated position:

```rust
use financial_calc::amm::calculate_position_value;

let value = calculate_position_value(
    sqrt_price_current,
    sqrt_price_lower,
    sqrt_price_upper,
    liquidity,
)?;
```

## Impermanent Loss

Calculate IL for a concentrated position:

```rust
use financial_calc::amm::calculate_impermanent_loss;

let il = calculate_impermanent_loss(
    entry_sqrt_price,
    current_sqrt_price,
    sqrt_price_lower,
    sqrt_price_upper,
    liquidity,
)?;
// Returns negative value (e.g., -0.05 for 5% loss vs HODL)
```

## Full-Range Liquidity

For Uniswap V2-style pools:

```rust
use financial_calc::amm::{calculate_liquidity_mint, calculate_liquidity_burn};

// Mint LP tokens
let shares = calculate_liquidity_mint(
    amount_0,
    amount_1,
    reserve_0,
    reserve_1,
    total_supply,
)?;

// Burn LP tokens
let (out_0, out_1) = calculate_liquidity_burn(
    shares,
    reserve_0,
    reserve_1,
    total_supply,
)?;
```
