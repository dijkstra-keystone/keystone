# Decimal Type

The `Decimal` type provides 128-bit decimal arithmetic with up to 28 significant digits.

## Construction

```rust
use precision_core::Decimal;

// Constants
let zero = Decimal::ZERO;
let one = Decimal::ONE;
let hundred = Decimal::ONE_HUNDRED;

// From integers (infallible)
let a = Decimal::from(42i64);
let b = Decimal::from(1_000_000u64);

// From mantissa and scale
// Value = mantissa * 10^(-scale)
let c = Decimal::new(12345, 2);    // 123.45
let d = Decimal::new(-500, 3);     // -0.500

// From strings (fallible)
let e: Decimal = "123.456".parse()?;
let f: Decimal = "-0.001".parse()?;

// From 128-bit integers (infallible)
let g = Decimal::from(i128::MAX);
```

## Checked Arithmetic

All arithmetic operations have checked variants that return `Option`:

```rust
let a = Decimal::from(100i64);
let b = Decimal::from(3i64);

// Returns None on overflow/underflow/division-by-zero
let sum = a.checked_add(b);      // Some(103)
let diff = a.checked_sub(b);     // Some(97)
let prod = a.checked_mul(b);     // Some(300)
let quot = a.checked_div(b);     // Some(33.333...)
let rem = a.checked_rem(b);      // Some(1)

// Overflow example
let overflow = Decimal::MAX.checked_add(Decimal::ONE);  // None
```

## Try Arithmetic

For explicit error handling:

```rust
use precision_core::{Decimal, ArithmeticError};

let a = Decimal::from(100i64);
let b = Decimal::ZERO;

match a.try_div(b) {
    Ok(result) => println!("{}", result),
    Err(ArithmeticError::DivisionByZero) => println!("Cannot divide by zero"),
    Err(ArithmeticError::Overflow) => println!("Result too large"),
    Err(e) => println!("Error: {}", e),
}
```

## Saturating Arithmetic

Operations that clamp to `MAX` or `MIN` on overflow:

```rust
let max = Decimal::MAX;
let result = max.saturating_add(Decimal::ONE);  // MAX (no panic)
```

## Properties

```rust
let a = Decimal::new(-12345, 3);  // -12.345

a.is_zero();        // false
a.is_negative();    // true
a.is_positive();    // false
a.scale();          // 3
a.abs();            // 12.345
a.signum();         // -1
```

## Comparison

```rust
let a = Decimal::from(100i64);
let b = Decimal::from(200i64);

a < b;              // true
a.min(b);           // 100
a.max(b);           // 200
a.clamp(Decimal::ZERO, Decimal::from(150i64));  // 100
```

## Normalization

Remove trailing zeros:

```rust
let a = Decimal::new(1000, 2);  // 10.00
let normalized = a.normalize();  // 10 (scale = 0)
```

## Internal Representation

Access the underlying components:

```rust
let a = Decimal::new(12345, 3);
let (mantissa, scale) = a.to_parts();  // (12345, 3)

// Access rust_decimal directly if needed
let inner = a.into_inner();
```
