# precision-core

Deterministic fixed-point arithmetic for financial computation.

## Features

- 128-bit decimal arithmetic with up to 28 significant digits
- `no_std` compatible for embedded and WASM targets
- 7 rounding modes including banker's rounding
- Deterministic results across all platforms
- Zero unsafe code

## Usage

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

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
