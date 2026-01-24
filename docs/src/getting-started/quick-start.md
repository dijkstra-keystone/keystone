# Quick Start

## Basic Arithmetic

```rust
use precision_core::Decimal;

// From integers
let a = Decimal::from(100i64);

// From mantissa and scale: value = mantissa * 10^(-scale)
let b = Decimal::new(12345, 2);  // 123.45

// From strings
let c: Decimal = "99.99".parse().unwrap();

// Arithmetic with checked operations
let sum = a.try_add(b)?;
let product = a.try_mul(c)?;
let quotient = a.try_div(b)?;
```

## Rounding

```rust
use precision_core::{Decimal, RoundingMode};

let value = Decimal::new(12345, 3);  // 12.345

// Banker's rounding (half-even) - default
let rounded = value.round_dp(2);  // 12.34

// Other modes
let up = value.round(2, RoundingMode::Up);        // 12.35
let down = value.round(2, RoundingMode::Down);    // 12.34
let half_up = value.round(2, RoundingMode::HalfUp); // 12.35
```

## Financial Calculations

```rust
use financial_calc::{compound_interest, future_value, Decimal};

let principal = Decimal::from(10000i64);
let rate = Decimal::new(5, 2);  // 5%

// Compound interest: monthly for 5 years
let interest = compound_interest(principal, rate, 12, 5)?;

// Future value
let fv = future_value(principal, rate, 10)?;
```

## Risk Metrics

```rust
use risk_metrics::{health_factor, liquidation_price, Decimal};

let collateral = Decimal::from(10000i64);
let debt = Decimal::from(5000i64);
let threshold = Decimal::new(80, 2);  // 80%

// Health factor: (collateral * threshold) / debt
let hf = health_factor(collateral, debt, threshold)?;  // 1.6

// Liquidation price
let liq = liquidation_price(
    Decimal::from(5i64),  // 5 ETH collateral
    debt,
    threshold,
)?;  // $1,250 per ETH
```

## Error Handling

```rust
use precision_core::{Decimal, ArithmeticError};

let result = Decimal::MAX.try_add(Decimal::ONE);
match result {
    Ok(value) => println!("Result: {}", value),
    Err(ArithmeticError::Overflow) => println!("Overflow"),
    Err(ArithmeticError::DivisionByZero) => println!("Division by zero"),
    Err(e) => println!("Error: {}", e),
}
```
