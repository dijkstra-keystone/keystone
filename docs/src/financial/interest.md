# Interest Calculations

The `financial-calc` crate provides precise interest calculation functions.

## Simple Interest

Interest calculated only on the principal:

```rust
use financial_calc::{simple_interest, Decimal};

let principal = Decimal::from(10000i64);
let rate = Decimal::new(5, 2);  // 5% annual rate
let years = Decimal::from(3i64);

let interest = simple_interest(principal, rate, years)?;
// interest = 10000 * 0.05 * 3 = 1500
```

Formula: `I = P × r × t`

## Compound Interest

Interest calculated on principal plus accumulated interest:

```rust
use financial_calc::{compound_interest, Decimal};

let principal = Decimal::from(10000i64);
let rate = Decimal::new(5, 2);  // 5% annual rate
let periods_per_year = 12;       // monthly compounding
let years = 5;

let total_interest = compound_interest(principal, rate, periods_per_year, years)?;
// Returns total interest earned over the period
```

Formula: `A = P(1 + r/n)^(nt)` where interest = `A - P`

### Compounding Frequencies

| Frequency | Periods per Year |
|-----------|------------------|
| Annual | 1 |
| Semi-annual | 2 |
| Quarterly | 4 |
| Monthly | 12 |
| Daily | 365 |

## Effective Annual Rate (EAR)

Convert a nominal rate to its effective annual equivalent:

```rust
use financial_calc::{effective_annual_rate, Decimal};

let nominal_rate = Decimal::new(5, 2);  // 5% nominal
let periods = 12;  // monthly compounding

let ear = effective_annual_rate(nominal_rate, periods)?;
// ear ≈ 5.116% (effective annual rate)
```

Formula: `EAR = (1 + r/n)^n - 1`

## Continuous Compounding

For theoretical continuous compounding, use a high number of periods:

```rust
let rate = Decimal::new(5, 2);
let periods = 365 * 24;  // hourly approximation

let ear = effective_annual_rate(rate, periods)?;
// Approaches e^r - 1
```

## Error Handling

All interest functions return `Result<Decimal, ArithmeticError>`:

```rust
use financial_calc::{compound_interest, Decimal};
use precision_core::ArithmeticError;

let result = compound_interest(
    Decimal::MAX,
    Decimal::from(100i64),
    12,
    100,
);

match result {
    Ok(interest) => println!("Interest: {}", interest),
    Err(ArithmeticError::Overflow) => println!("Calculation overflow"),
    Err(e) => println!("Error: {:?}", e),
}
```

## Practical Examples

### Savings Account

```rust
let deposit = Decimal::from(5000i64);
let apy = Decimal::new(425, 4);  // 4.25% APY
let months = 18;

// APY is already effective rate, so use simple calculation
let years = Decimal::new(months as i64, 0)
    .checked_div(Decimal::from(12i64))
    .unwrap();
let earnings = simple_interest(deposit, apy, years)?;
```

### Loan Interest

```rust
let loan = Decimal::from(250000i64);
let apr = Decimal::new(675, 4);  // 6.75% APR
let years = 30;

// Total interest over loan lifetime (monthly compounding)
let total_interest = compound_interest(loan, apr, 12, years)?;
```
