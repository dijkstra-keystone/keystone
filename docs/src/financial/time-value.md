# Time Value of Money

Functions for present and future value calculations.

## Future Value

Calculate the future value of a present amount:

```rust
use financial_calc::{future_value, Decimal};

let present = Decimal::from(10000i64);
let rate = Decimal::new(7, 2);  // 7% annual return
let periods = 10;

let fv = future_value(present, rate, periods)?;
// fv ≈ 19,671.51
```

Formula: `FV = PV × (1 + r)^n`

## Present Value

Calculate the present value of a future amount:

```rust
use financial_calc::{present_value, Decimal};

let future = Decimal::from(100000i64);
let rate = Decimal::new(5, 2);  // 5% discount rate
let periods = 20;

let pv = present_value(future, rate, periods)?;
// pv ≈ 37,688.95
```

Formula: `PV = FV / (1 + r)^n`

## Net Present Value (NPV)

Evaluate investment profitability by discounting future cash flows:

```rust
use financial_calc::{net_present_value, Decimal};

let rate = Decimal::new(10, 2);  // 10% discount rate
let cash_flows = [
    Decimal::from(-100000i64),  // Initial investment (negative)
    Decimal::from(30000i64),    // Year 1
    Decimal::from(40000i64),    // Year 2
    Decimal::from(50000i64),    // Year 3
    Decimal::from(60000i64),    // Year 4
];

let npv = net_present_value(rate, &cash_flows)?;
// npv > 0 indicates profitable investment
```

### NPV Decision Rule

| NPV | Decision |
|-----|----------|
| > 0 | Accept (creates value) |
| = 0 | Indifferent |
| < 0 | Reject (destroys value) |

## Practical Examples

### Retirement Planning

```rust
// How much do I need to save monthly to reach $1M in 30 years?
let target = Decimal::from(1_000_000i64);
let annual_return = Decimal::new(7, 2);
let years = 30;

// First, find present value
let pv_target = present_value(target, annual_return, years)?;
// pv_target ≈ $131,367

// Then calculate monthly savings needed (simplified)
let months = years * 12;
let monthly = pv_target.checked_div(Decimal::from(months as i64)).unwrap();
```

### Investment Comparison

```rust
// Compare two investments with different cash flow patterns

// Investment A: $50K now, returns $80K in 5 years
let rate = Decimal::new(8, 2);
let fv_a = future_value(Decimal::from(50000i64), rate, 5)?;
let profit_a = Decimal::from(80000i64).checked_sub(fv_a);

// Investment B: $50K now, returns $15K annually for 5 years
let cash_flows_b = [
    Decimal::from(-50000i64),
    Decimal::from(15000i64),
    Decimal::from(15000i64),
    Decimal::from(15000i64),
    Decimal::from(15000i64),
    Decimal::from(15000i64),
];
let npv_b = net_present_value(rate, &cash_flows_b)?;
```

### Inflation Adjustment

```rust
// What will $100 be worth in 10 years with 3% inflation?
let amount = Decimal::from(100i64);
let inflation = Decimal::new(3, 2);
let years = 10;

// Future purchasing power (inverse of future value)
let future_purchasing_power = present_value(amount, inflation, years)?;
// ≈ $74.41 in today's dollars
```

## Error Conditions

- **Overflow**: Large values or many periods can overflow
- **DivisionByZero**: Rate of exactly -100% in present_value

```rust
// Handle potential errors
match future_value(principal, rate, periods) {
    Ok(fv) => use_value(fv),
    Err(ArithmeticError::Overflow) => handle_overflow(),
    Err(e) => handle_error(e),
}
```
