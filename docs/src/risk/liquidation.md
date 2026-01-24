# Liquidation Calculations

Functions for calculating liquidation thresholds and prices in DeFi lending.

## Liquidation Price

Calculate the price at which a position becomes liquidatable:

```rust
use risk_metrics::{liquidation_price, Decimal};

let collateral_amount = Decimal::from(5i64);  // 5 ETH
let debt = Decimal::from(10000i64);            // $10,000 debt
let threshold = Decimal::new(80, 2);           // 80% threshold

let liq_price = liquidation_price(collateral_amount, debt, threshold)?;
// liq_price = $2,500 per ETH
```

Formula: `Liquidation Price = Debt / (Collateral Amount × Threshold)`

When ETH drops to $2,500, the position reaches HF = 1.0 and becomes liquidatable.

## Liquidation Threshold

Determine the threshold at which liquidation occurs:

```rust
use risk_metrics::{liquidation_threshold, Decimal};

let collateral_value = Decimal::from(10000i64);
let debt = Decimal::from(8000i64);
let current_hf = Decimal::new(125, 2);  // 1.25

let threshold = liquidation_threshold(collateral_value, debt, current_hf)?;
// Calculates the effective liquidation threshold
```

## Maximum Borrowable

Calculate maximum debt for a given health factor target:

```rust
use risk_metrics::{max_borrowable, Decimal};

let collateral = Decimal::from(100000i64);
let threshold = Decimal::new(80, 2);
let min_health_factor = Decimal::new(15, 1);  // Target HF 1.5

let max_debt = max_borrowable(collateral, threshold, min_health_factor)?;
// max_debt = $53,333.33
```

Formula: `Max Debt = (Collateral × Threshold) / Min Health Factor`

## Practical Examples

### Position Monitoring Dashboard

```rust
use risk_metrics::{liquidation_price, health_factor, Decimal};

struct Position {
    eth_amount: Decimal,
    eth_price: Decimal,
    debt_usd: Decimal,
    threshold: Decimal,
}

fn analyze_position(pos: &Position) -> Result<(), ArithmeticError> {
    let collateral_usd = pos.eth_amount.checked_mul(pos.eth_price)?;

    let hf = health_factor(collateral_usd, pos.debt_usd, pos.threshold)?;
    let liq_price = liquidation_price(pos.eth_amount, pos.debt_usd, pos.threshold)?;

    let buffer = pos.eth_price
        .checked_sub(liq_price)?
        .checked_div(pos.eth_price)?
        .checked_mul(Decimal::ONE_HUNDRED)?;

    println!("Health Factor: {}", hf);
    println!("Liquidation Price: ${}", liq_price);
    println!("Price Buffer: {}%", buffer);

    Ok(())
}
```

### Liquidation Alert System

```rust
use risk_metrics::{liquidation_price, Decimal};

fn check_liquidation_risk(
    collateral_amount: Decimal,
    debt: Decimal,
    threshold: Decimal,
    current_price: Decimal,
) -> Result<RiskLevel, ArithmeticError> {
    let liq_price = liquidation_price(collateral_amount, debt, threshold)?;

    let distance = current_price
        .checked_sub(liq_price)?
        .checked_div(current_price)?;

    Ok(match distance {
        d if d >= Decimal::new(30, 2) => RiskLevel::Safe,
        d if d >= Decimal::new(15, 2) => RiskLevel::Moderate,
        d if d >= Decimal::new(5, 2) => RiskLevel::High,
        _ => RiskLevel::Critical,
    })
}

enum RiskLevel {
    Safe,
    Moderate,
    High,
    Critical,
}
```

### Multi-Collateral Liquidation

```rust
// Calculate weighted liquidation threshold for multiple assets
fn weighted_liquidation_threshold(
    assets: &[(Decimal, Decimal)],  // (value, threshold) pairs
) -> Result<Decimal, ArithmeticError> {
    let mut weighted_sum = Decimal::ZERO;
    let mut total_value = Decimal::ZERO;

    for (value, threshold) in assets {
        weighted_sum = weighted_sum
            .checked_add(value.checked_mul(*threshold)?)?;
        total_value = total_value.checked_add(*value)?;
    }

    weighted_sum.checked_div(total_value)
}
```

## Liquidation Mechanics

### Typical Liquidation Process

1. Health factor drops below 1.0
2. Liquidator repays portion of debt
3. Liquidator receives collateral + bonus
4. Position health factor improves

### Liquidation Bonus

```rust
// Calculate liquidator profit
let debt_to_repay = Decimal::from(1000i64);
let liquidation_bonus = Decimal::new(5, 2);  // 5% bonus

let collateral_received = debt_to_repay
    .checked_mul(Decimal::ONE.checked_add(liquidation_bonus)?)?;
// Liquidator receives $1,050 worth of collateral for repaying $1,000
```

## Edge Cases

```rust
// Zero collateral amount
let result = liquidation_price(
    Decimal::ZERO,
    Decimal::from(1000i64),
    Decimal::new(80, 2),
);
// Returns error or MAX (no liquidation price exists)

// Zero debt
let result = liquidation_price(
    Decimal::from(10i64),
    Decimal::ZERO,
    Decimal::new(80, 2),
);
// Returns ZERO (no liquidation risk)
```
