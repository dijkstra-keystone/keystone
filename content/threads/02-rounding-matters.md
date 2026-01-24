# Thread: Rounding Matters More Than You Think

## Tweet 1 (Hook)
Your rounding mode choice can shift billions of dollars.

Banks use "banker's rounding" for a reason. Most developers have never heard of it. 🧵

## Tweet 2 (The Problem)
Traditional rounding (HalfUp):
2.5 → 3
3.5 → 4
4.5 → 5

Always rounds .5 up. Seems fair.

But sum a million transactions: systematic bias toward higher values.

## Tweet 3 (Banker's Rounding)
Banker's rounding (HalfEven):
2.5 → 2 (to even)
3.5 → 4 (to even)
4.5 → 4 (to even)

Ties round to nearest EVEN digit.

Statistically unbiased over large datasets.

## Tweet 4 (Why It Matters)
Interest calculations on $1B daily:
• HalfUp: systematically overpays interest
• HalfEven: errors cancel out over time

The difference? Millions of dollars annually.

That's why banks, exchanges, and financial regulators mandate it.

## Tweet 5 (DeFi Reality)
Most DeFi protocols: Math.round() (HalfUp bias)

Lending pools, yield aggregators, AMMs—all accumulating systematic errors.

Small per-transaction. Massive at scale.

## Tweet 6 (Full Control)
Keystone provides 7 rounding modes:

• HalfEven (banker's) ← default
• HalfUp (traditional)
• HalfDown
• Up (toward +∞)
• Down (toward -∞)
• TowardZero (truncate)
• AwayFromZero

## Tweet 7 (Usage)
```rust
use precision_core::{Decimal, RoundingMode};

let value = Decimal::new(12345, 3);  // 12.345

value.round(2, RoundingMode::HalfEven);  // 12.34
value.round(2, RoundingMode::HalfUp);    // 12.35
value.round(2, RoundingMode::Down);      // 12.34
```

## Tweet 8 (Tax Example)
Tax calculations: round toward government

```rust
let tax = amount.checked_mul(rate)?;
let rounded = tax.round(2, RoundingMode::Up);
// Government always gets at least what's owed
```

## Tweet 9 (CTA)
Stop leaving money on the table with the wrong rounding mode.

Keystone: precision-core crate
Docs: docs.dijkstrakeystone.com/concepts/rounding

MIT/Apache-2.0 licensed.

---
*Character counts verified. Ready for scheduling.*
