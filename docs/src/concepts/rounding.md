# Rounding Modes

Keystone supports 7 rounding modes for precise control over decimal rounding.

## Available Modes

| Mode | Description | 2.5 → | 3.5 → | -2.5 → |
|------|-------------|-------|-------|--------|
| `HalfEven` | Banker's rounding (ties to even) | 2 | 4 | -2 |
| `HalfUp` | Ties round away from zero | 3 | 4 | -3 |
| `HalfDown` | Ties round toward zero | 2 | 3 | -2 |
| `Up` | Always round toward +∞ | 3 | 4 | -2 |
| `Down` | Always round toward -∞ | 2 | 3 | -3 |
| `TowardZero` | Truncate (round toward zero) | 2 | 3 | -2 |
| `AwayFromZero` | Round away from zero | 3 | 4 | -3 |

## Usage

```rust
use precision_core::{Decimal, RoundingMode};

let value = Decimal::new(12345, 3);  // 12.345

// Round to 2 decimal places
value.round(2, RoundingMode::HalfEven);   // 12.34
value.round(2, RoundingMode::HalfUp);     // 12.35
value.round(2, RoundingMode::Down);       // 12.34
value.round(2, RoundingMode::Up);         // 12.35

// Convenience methods
value.round_dp(2);  // 12.34 (uses HalfEven)
value.trunc(2);     // 12.34 (uses TowardZero)
value.floor();      // 12 (round toward -∞)
value.ceil();       // 13 (round toward +∞)
```

## Banker's Rounding (HalfEven)

The default mode. Ties round to the nearest even digit. This minimizes cumulative rounding error over many operations.

```rust
let a = Decimal::new(25, 1);  // 2.5
let b = Decimal::new(35, 1);  // 3.5

a.round(0, RoundingMode::HalfEven);  // 2 (rounds to even)
b.round(0, RoundingMode::HalfEven);  // 4 (rounds to even)
```

## Financial Applications

| Use Case | Recommended Mode |
|----------|------------------|
| Invoice totals | `HalfUp` |
| Tax calculations | `HalfUp` or `Up` (toward government) |
| Interest accumulation | `HalfEven` |
| Currency display | `HalfUp` |
| Truncation (e.g., satoshis) | `TowardZero` |

## WASM

```javascript
import { round } from '@dijkstra-keystone/wasm';

round("2.345", 2, "half_even");  // "2.34"
round("2.345", 2, "half_up");    // "2.35"
round("2.345", 2, "truncate");   // "2.34"
```

Available mode strings: `"down"`, `"up"`, `"toward_zero"`, `"truncate"`, `"away_from_zero"`, `"half_even"`, `"bankers"`, `"half_up"`, `"half_down"`.
