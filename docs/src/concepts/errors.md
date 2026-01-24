# Error Handling

Keystone uses explicit error types for all fallible operations.

## Arithmetic Errors

```rust
use precision_core::ArithmeticError;

pub enum ArithmeticError {
    Overflow,       // Result exceeds MAX
    Underflow,      // Result below MIN
    DivisionByZero, // Division by zero
    ScaleExceeded,  // Scale > 28
}
```

## Parse Errors

```rust
use precision_core::ParseError;

pub enum ParseError {
    Empty,                  // Empty string
    InvalidCharacter,       // Non-numeric character
    MultipleDecimalPoints,  // "1.2.3"
    OutOfRange,            // Value too large
}
```

## Handling Patterns

### Option-based (checked operations)

```rust
let a = Decimal::from(100i64);
let b = Decimal::ZERO;

match a.checked_div(b) {
    Some(result) => use_result(result),
    None => handle_error(),
}
```

### Result-based (try operations)

```rust
let result = a.try_div(b);
match result {
    Ok(value) => use_value(value),
    Err(ArithmeticError::DivisionByZero) => handle_div_zero(),
    Err(e) => handle_other(e),
}
```

### With the `?` operator

```rust
fn calculate(a: Decimal, b: Decimal) -> Result<Decimal, ArithmeticError> {
    let sum = a.try_add(b)?;
    let product = sum.try_mul(Decimal::from(2i64))?;
    Ok(product)
}
```

## Panicking Operations

Standard operators panic on error. Use only when errors are impossible:

```rust
// These panic on overflow or division by zero
let sum = a + b;
let diff = a - b;
let prod = a * b;
let quot = a / b;  // panics if b == 0
```

## WASM Error Handling

JavaScript functions throw on error:

```javascript
try {
  const result = keystone.divide("1", "0");
} catch (e) {
  console.error(e.message);  // "division by zero"
}
```

Or use optional chaining:

```javascript
const result = (() => {
  try { return keystone.divide(a, b); }
  catch { return null; }
})();
```
