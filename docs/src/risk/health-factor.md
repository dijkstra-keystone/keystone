# Health Factor

The health factor is a key metric for collateralized lending positions in DeFi protocols.

## Definition

Health factor measures the safety of a collateralized position:

```
Health Factor = (Collateral Value × Liquidation Threshold) / Debt Value
```

| Health Factor | Status |
|---------------|--------|
| > 1.0 | Safe |
| = 1.0 | At liquidation threshold |
| < 1.0 | Liquidatable |

## Basic Usage

```rust
use risk_metrics::{health_factor, Decimal};

let collateral = Decimal::from(10000i64);  // $10,000 collateral
let debt = Decimal::from(5000i64);          // $5,000 debt
let threshold = Decimal::new(80, 2);        // 80% liquidation threshold

let hf = health_factor(collateral, debt, threshold)?;
// hf = (10000 × 0.80) / 5000 = 1.6
```

## Health Check

Quick boolean check for position safety:

```rust
use risk_metrics::{is_healthy, Decimal};

let collateral = Decimal::from(10000i64);
let debt = Decimal::from(8500i64);
let threshold = Decimal::new(80, 2);

is_healthy(collateral, debt, threshold)?;  // false (HF < 1.0)
```

## Collateral Ratio

Calculate the ratio of collateral to debt:

```rust
use risk_metrics::{collateral_ratio, Decimal};

let collateral = Decimal::from(15000i64);
let debt = Decimal::from(10000i64);

let ratio = collateral_ratio(collateral, debt)?;
// ratio = 1.5 (150% collateralization)
```

## Practical Examples

### Monitor Position Safety

```rust
use risk_metrics::{health_factor, Decimal};

let collateral = Decimal::from(50000i64);
let debt = Decimal::from(30000i64);
let threshold = Decimal::new(825, 3);  // 82.5%

let hf = health_factor(collateral, debt, threshold)?;

match hf {
    hf if hf >= Decimal::new(15, 1) => println!("Position is safe"),
    hf if hf >= Decimal::new(12, 1) => println!("Consider adding collateral"),
    hf if hf >= Decimal::ONE => println!("Warning: Near liquidation"),
    _ => println!("DANGER: Position liquidatable"),
}
```

### Calculate Safe Borrow Amount

```rust
use risk_metrics::{health_factor, Decimal};

let collateral = Decimal::from(100000i64);
let threshold = Decimal::new(80, 2);
let target_hf = Decimal::new(15, 1);  // Target 1.5 health factor

// max_debt = (collateral × threshold) / target_hf
let max_safe_debt = collateral
    .checked_mul(threshold)?
    .checked_div(target_hf)?;
// max_safe_debt = $53,333.33
```

### Multi-Asset Position

```rust
// Weighted average for multiple collateral types
let eth_value = Decimal::from(50000i64);
let eth_threshold = Decimal::new(80, 2);

let btc_value = Decimal::from(30000i64);
let btc_threshold = Decimal::new(75, 2);

let weighted_collateral = eth_value
    .checked_mul(eth_threshold)?
    .checked_add(btc_value.checked_mul(btc_threshold)?)?;

let debt = Decimal::from(40000i64);
let hf = weighted_collateral.checked_div(debt)?;
```

## Protocol-Specific Thresholds

Different protocols use different liquidation thresholds:

| Protocol | Typical Threshold |
|----------|-------------------|
| Aave (ETH) | 82.5% |
| Compound (ETH) | 82.5% |
| MakerDAO (ETH-A) | 83% |
| Aave (Stablecoins) | 90% |

## Edge Cases

```rust
use risk_metrics::{health_factor, Decimal};
use precision_core::ArithmeticError;

// Zero debt = infinite health (returns MAX)
let hf = health_factor(
    Decimal::from(1000i64),
    Decimal::ZERO,
    Decimal::new(80, 2),
)?;
// hf = Decimal::MAX

// Zero collateral with debt
let hf = health_factor(
    Decimal::ZERO,
    Decimal::from(1000i64),
    Decimal::new(80, 2),
)?;
// hf = 0
```
