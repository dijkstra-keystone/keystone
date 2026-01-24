# Percentage Operations

Precise percentage calculations for financial applications.

## Basic Percentage

Calculate a percentage of a value:

```rust
use financial_calc::{percentage_of, Decimal};

let total = Decimal::from(1500i64);
let rate = Decimal::new(15, 2);  // 15%

let amount = percentage_of(total, rate)?;
// amount = 225 (15% of 1500)
```

## Percentage Change

Calculate the percentage change between two values:

```rust
use financial_calc::{percentage_change, Decimal};

let old_price = Decimal::from(100i64);
let new_price = Decimal::from(125i64);

let change = percentage_change(old_price, new_price)?;
// change = 25% (0.25)
```

Formula: `(new - old) / old × 100`

## Add Percentage

Add a percentage to a base value:

```rust
use financial_calc::{add_percentage, Decimal};

let price = Decimal::from(100i64);
let tax_rate = Decimal::new(825, 4);  // 8.25%

let total = add_percentage(price, tax_rate)?;
// total = 108.25
```

## Subtract Percentage

Remove a percentage from a value:

```rust
use financial_calc::{subtract_percentage, Decimal};

let original = Decimal::from(100i64);
let discount = Decimal::new(20, 2);  // 20%

let final_price = subtract_percentage(original, discount)?;
// final_price = 80
```

## Reverse Percentage

Find the original value before a percentage was added:

```rust
use financial_calc::{reverse_percentage, Decimal};

let total_with_tax = Decimal::new(10825, 2);  // 108.25
let tax_rate = Decimal::new(825, 4);  // 8.25%

let original = reverse_percentage(total_with_tax, tax_rate)?;
// original = 100.00
```

## Practical Examples

### Sales Tax Calculation

```rust
let subtotal = Decimal::new(15999, 2);  // $159.99
let state_tax = Decimal::new(6, 2);      // 6%
let local_tax = Decimal::new(225, 4);    // 2.25%

let state_amount = percentage_of(subtotal, state_tax)?;
let local_amount = percentage_of(subtotal, local_tax)?;
let total = subtotal
    .checked_add(state_amount)?
    .checked_add(local_amount)?;
```

### Profit Margin

```rust
let revenue = Decimal::from(500000i64);
let costs = Decimal::from(350000i64);
let profit = revenue.checked_sub(costs)?;

// Profit margin = profit / revenue
let margin = profit
    .checked_mul(Decimal::ONE_HUNDRED)?
    .checked_div(revenue)?;
// margin = 30%
```

### Compound Discounts

```rust
// Apply 20% off, then additional 10% off
let original = Decimal::from(100i64);
let first_discount = Decimal::new(20, 2);
let second_discount = Decimal::new(10, 2);

let after_first = subtract_percentage(original, first_discount)?;  // 80
let final_price = subtract_percentage(after_first, second_discount)?;  // 72

// Note: Not the same as 30% off (which would be 70)
```

### Tip Calculator

```rust
let bill = Decimal::new(8567, 2);  // $85.67

let tip_15 = percentage_of(bill, Decimal::new(15, 2))?;  // $12.85
let tip_18 = percentage_of(bill, Decimal::new(18, 2))?;  // $15.42
let tip_20 = percentage_of(bill, Decimal::new(20, 2))?;  // $17.13
```

## Basis Points

For financial applications, use basis points (1 bp = 0.01%):

```rust
use precision_core::{Decimal, within_basis_points};

let quoted_rate = Decimal::new(525, 4);   // 5.25%
let actual_rate = Decimal::new(5251, 5);  // 5.251%

// Check if within 5 basis points
within_basis_points(quoted_rate, actual_rate, Decimal::from(5i64));  // true
```

| Basis Points | Percentage |
|--------------|------------|
| 1 bp | 0.01% |
| 10 bp | 0.10% |
| 25 bp | 0.25% |
| 50 bp | 0.50% |
| 100 bp | 1.00% |
