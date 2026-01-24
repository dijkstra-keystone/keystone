# financial-calc

Financial calculation functions built on precision-core.

## Features

- Interest calculations (simple, compound, effective annual rate)
- Time value of money (future value, present value, NPV)
- Percentage operations
- `no_std` compatible
- Deterministic results

## Usage

```rust
use financial_calc::{compound_interest, future_value, Decimal};

let principal = Decimal::from(10000i64);
let rate = Decimal::new(5, 2);  // 5%

// Compound interest: monthly for 5 years
let interest = compound_interest(principal, rate, 12, 5)?;

// Future value
let fv = future_value(principal, rate, 10)?;
```

## Functions

### Interest
- `simple_interest(principal, rate, time)`
- `compound_interest(principal, rate, periods_per_year, years)`
- `effective_annual_rate(nominal_rate, periods)`

### Time Value
- `future_value(present_value, rate, periods)`
- `present_value(future_value, rate, periods)`
- `net_present_value(rate, cash_flows)`

### Percentages
- `percentage_of(value, percent)`
- `percentage_change(old, new)`
- `basis_points_to_decimal(bps)`

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
