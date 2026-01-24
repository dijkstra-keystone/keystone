# Tolerance Comparisons

Financial calculations often require approximate equality checks due to rounding. Keystone provides tolerance-based comparison functions.

## Absolute Tolerance

Check if two values differ by at most a fixed amount:

```rust
use precision_core::{Decimal, approx_eq};

let a = Decimal::new(10000, 2);  // 100.00
let b = Decimal::new(10001, 2);  // 100.01
let tolerance = Decimal::new(1, 2);  // 0.01

approx_eq(a, b, tolerance);  // true (difference <= 0.01)
```

## Relative Tolerance

Check if two values differ by at most a percentage of their magnitude:

```rust
use precision_core::{Decimal, approx_eq_relative};

let a = Decimal::from(1_000_000i64);
let b = Decimal::from(1_000_100i64);
let tolerance = Decimal::new(1, 3);  // 0.1%

approx_eq_relative(a, b, tolerance);  // true
```

## Combined Tolerance

Use both absolute and relative tolerances (passes if either succeeds):

```rust
use precision_core::{Decimal, approx_eq_ulps};

let a = Decimal::from(100i64);
let b = Decimal::new(10001, 2);  // 100.01

approx_eq_ulps(
    a, b,
    Decimal::new(1, 2),   // absolute: 0.01
    Decimal::new(1, 3),   // relative: 0.1%
);
```

## Percentage Tolerance

Check if values are within a percentage of each other:

```rust
use precision_core::{Decimal, within_percentage};

let actual = Decimal::from(102i64);
let expected = Decimal::from(100i64);

within_percentage(actual, expected, Decimal::from(5i64));  // true (within 5%)
within_percentage(actual, expected, Decimal::from(1i64));  // false (not within 1%)
```

## Basis Points Tolerance

For financial applications using basis points (1 bp = 0.01%):

```rust
use precision_core::{Decimal, within_basis_points};

let a = Decimal::new(10010, 2);  // 100.10
let b = Decimal::from(100i64);

within_basis_points(a, b, Decimal::from(100i64));  // true (within 100 bps = 1%)
within_basis_points(a, b, Decimal::from(5i64));    // false (not within 5 bps)
```

## Use Cases

| Scenario | Recommended Function |
|----------|---------------------|
| Comparing prices | `within_basis_points` |
| Verifying calculations | `approx_eq` with small absolute tolerance |
| Comparing large values | `approx_eq_relative` |
| Test assertions | `approx_eq_ulps` (handles edge cases) |
| Rate comparisons | `within_percentage` |
