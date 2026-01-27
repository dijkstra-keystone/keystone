# Keystone

Deterministic precision arithmetic for financial computation and verifiable systems.

[![CI](https://github.com/dijkstra-keystone/keystone/actions/workflows/ci.yml/badge.svg)](https://github.com/dijkstra-keystone/keystone/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/precision-core.svg)](https://crates.io/crates/precision-core)
[![docs.rs](https://docs.rs/precision-core/badge.svg)](https://docs.rs/precision-core)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

## Why Keystone?

DeFi protocols make critical decisions—liquidations, pricing, risk assessments—based on mathematical calculations. When your laptop, a validator node, and a WASM frontend compute the same formula and get *different* results due to floating-point inconsistencies, bad things happen.

Keystone guarantees **bit-identical results** across x86, ARM, and WASM. No floating-point surprises. No platform-dependent rounding. The same input produces the same output, every time, everywhere.

## Overview

Keystone provides a suite of libraries for financial calculations with guaranteed determinism across platforms. Built for DeFi applications, verifiable computation, and any system requiring bit-exact reproducibility.

## Crates

| Crate | Description |
|-------|-------------|
| [precision-core](crates/precision-core) | 128-bit decimal arithmetic with 7 rounding modes |
| [financial-calc](crates/financial-calc) | Interest, time value, options pricing, percentages |
| [risk-metrics](crates/risk-metrics) | Health factor, liquidation, and position metrics |
| [keystone-wasm](crates/wasm-bindings) | WebAssembly bindings for browser usage |

## Stylus Examples

Ready-to-deploy examples for Arbitrum Stylus:

| Example | Description |
|---------|-------------|
| [stylus-lending](examples/stylus-lending) | Health factor and liquidation calculations |
| [stylus-amm](examples/stylus-amm) | Constant product AMM math |
| [stylus-vault](examples/stylus-vault) | ERC4626-style vault calculations |

### Deployed on Arbitrum One

| Contract | Address |
|----------|---------|
| stylus-lending | [`0x4dff9348275ac3c24e2d3abf54af61d3ebee1585`](https://arbiscan.io/address/0x4dff9348275ac3c24e2d3abf54af61d3ebee1585) |
| stylus-amm | [`0x9615cc2f65d8bbe4cdc80343db75a6ec32da93cd`](https://arbiscan.io/address/0x9615cc2f65d8bbe4cdc80343db75a6ec32da93cd) |
| stylus-vault | [`0xdaf8f1a5f8025210f07665d4ccf2d2c0622a41fa`](https://arbiscan.io/address/0xdaf8f1a5f8025210f07665d4ccf2d2c0622a41fa) |

## Features

- **Deterministic**: Identical results on all platforms (x86, ARM, WASM)
- **no_std**: Core library works in embedded and WASM environments
- **Precise**: 128-bit decimals with up to 28 significant digits
- **Safe**: `#![forbid(unsafe_code)]` throughout
- **Financial**: Banker's rounding and 6 other rounding modes
- **Fast**: Nanosecond-level operations (~8ns add, ~36ns divide)

## Quick Start

```rust
use precision_core::{Decimal, RoundingMode};

// Create decimals
let price = Decimal::new(9999, 2);     // 99.99
let quantity = Decimal::from(5i64);
let tax_rate = Decimal::new(825, 4);   // 8.25%

// Calculate
let subtotal = price.checked_mul(quantity)?;
let tax = subtotal.checked_mul(tax_rate)?;
let total = subtotal.checked_add(tax)?;

// Round for display
let display = total.round(2, RoundingMode::HalfUp);
```

## WASM Usage

```javascript
import * as keystone from '@dijkstra-keystone/wasm';

const subtotal = keystone.multiply("99.99", "5");
const tax = keystone.multiply(subtotal, "0.0825");
const total = keystone.add(subtotal, tax);
const display = keystone.round(total, 2, "half_up");
```

## Documentation

- [Online Documentation](https://docs.dijkstrakeystone.com)
- [API Reference](https://docs.rs/precision-core)

## Building

```bash
# Run tests
cargo test --all

# Build WASM
cd crates/wasm-bindings
wasm-pack build --target web --release

# Build documentation
cd docs && mdbook build
```

## Benchmarks

| Operation | Time |
|-----------|------|
| Addition | ~8 ns |
| Multiplication | ~8 ns |
| Division | ~36 ns |
| compound_interest | ~850 ns |

```bash
cargo bench
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
