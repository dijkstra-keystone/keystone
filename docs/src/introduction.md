# Keystone

Deterministic financial computation infrastructure for DeFi and fintech.

## What is Keystone?

Keystone provides precision arithmetic and financial calculations that produce identical results across all platforms. Built in Rust with `no_std` support, it compiles to native code, WebAssembly, and ZK-proving targets.

## Key Features

- **128-bit decimal arithmetic** with 28 digits of precision
- **7 rounding modes** including banker's rounding (half-even)
- **Checked operations** that explicitly handle overflow
- **Cross-platform determinism** verified in CI
- **DeFi risk metrics** for health factors, liquidation, and position analysis
- **Financial calculations** for interest, time value of money, and percentages

## Crates

| Crate | Description |
|-------|-------------|
| `precision-core` | Decimal type with deterministic arithmetic |
| `financial-calc` | Interest, TVM, and percentage calculations |
| `risk-metrics` | DeFi health factors and liquidation logic |
| `wasm-bindings` | JavaScript/TypeScript bindings |

## Quick Example

```rust
use precision_core::{Decimal, RoundingMode};

let price = Decimal::new(123456, 2);  // 1234.56
let quantity = Decimal::new(15, 1);   // 1.5
let total = price.try_mul(quantity)?; // 1851.84

// Round to cents using banker's rounding
let rounded = total.round(2, RoundingMode::HalfEven);
```

## Performance

Operations complete in nanoseconds:

| Operation | Time |
|-----------|------|
| Addition | ~8 ns |
| Multiplication | ~8 ns |
| Division | ~36 ns |
| Health factor | ~23 ns |

## License

Dual-licensed under MIT and Apache 2.0.
