# precision-core

[![Crates.io](https://img.shields.io/crates/v/precision-core.svg)](https://crates.io/crates/precision-core)
[![Documentation](https://docs.rs/precision-core/badge.svg)](https://docs.rs/precision-core)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../../LICENSE-MIT)

Deterministic fixed-point arithmetic for financial computation.

## Features

- 128-bit decimal arithmetic with up to 28 significant digits
- `no_std` compatible for embedded and WASM targets
- 7 rounding modes including banker's rounding
- Transcendental functions (exp, ln, sqrt, pow)
- Oracle integration utilities (Chainlink, Pyth)
- Deterministic results across all platforms
- Zero unsafe code

## Installation

```toml
[dependencies]
precision-core = "0.1"
```

## Quick Start

```rust
use precision_core::{Decimal, RoundingMode};

// From integers
let a = Decimal::from(100i64);

// From mantissa and scale: value = mantissa * 10^(-scale)
let b = Decimal::new(12345, 2);  // 123.45

// Checked arithmetic
let sum = a.checked_add(b).unwrap();
let product = a.checked_mul(b).unwrap();

// Rounding
let rounded = b.round(1, RoundingMode::HalfUp);  // 123.5

// Transcendental functions
let sqrt = Decimal::from(2i64).try_sqrt().unwrap();  // ~1.414...
let exp = Decimal::ONE.try_exp().unwrap();           // ~2.718...
let ln = Decimal::from(10i64).try_ln().unwrap();     // ~2.302...
```

## Oracle Integration

```rust
use precision_core::oracle::{normalize_oracle_price, OracleDecimals};

// Chainlink BTC/USD (8 decimals)
let btc_raw = 5000012345678i64;  // $50,000.12345678
let btc_price = normalize_oracle_price(btc_raw, OracleDecimals::Eight)?;

// Convert between decimal formats
use precision_core::oracle::convert_decimals;
let usdc_amount = convert_decimals(1000000, OracleDecimals::Six, OracleDecimals::Eighteen)?;
```

## Rounding Modes

| Mode | Description |
|------|-------------|
| `HalfEven` | Banker's rounding (default) |
| `HalfUp` | Traditional rounding |
| `HalfDown` | Ties toward zero |
| `Up` | Toward +infinity |
| `Down` | Toward -infinity |
| `TowardZero` | Truncation |
| `AwayFromZero` | Away from zero |

## no_std Usage

The crate is `no_std` by default. Enable the `std` feature for standard library support:

```toml
[dependencies]
precision-core = { version = "0.1", features = ["std"] }
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
