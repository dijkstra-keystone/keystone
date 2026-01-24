# Position Management

Functions for managing and analyzing trading and lending positions.

## Position Size

Calculate appropriate position size based on risk parameters:

```rust
use risk_metrics::{position_size, Decimal};

let account_balance = Decimal::from(100000i64);
let risk_per_trade = Decimal::new(2, 2);  // 2% risk
let stop_loss_pct = Decimal::new(5, 2);   // 5% stop loss

let size = position_size(account_balance, risk_per_trade, stop_loss_pct)?;
// size = $40,000 (2% risk / 5% stop = 40% of account)
```

## Leverage Calculation

Determine effective leverage of a position:

```rust
use risk_metrics::{effective_leverage, Decimal};

let position_value = Decimal::from(50000i64);
let collateral = Decimal::from(10000i64);

let leverage = effective_leverage(position_value, collateral)?;
// leverage = 5x
```

## Margin Requirements

Calculate required margin for a leveraged position:

```rust
use risk_metrics::{required_margin, Decimal};

let position_size = Decimal::from(100000i64);
let leverage = Decimal::from(10i64);

let margin = required_margin(position_size, leverage)?;
// margin = $10,000
```

## Profit and Loss

### Unrealized PnL

```rust
use risk_metrics::{unrealized_pnl, Decimal};

let entry_price = Decimal::from(2000i64);
let current_price = Decimal::from(2150i64);
let position_size = Decimal::from(5i64);  // 5 ETH
let is_long = true;

let pnl = unrealized_pnl(entry_price, current_price, position_size, is_long)?;
// pnl = +$750
```

### Return on Investment

```rust
use risk_metrics::{position_roi, Decimal};

let entry_value = Decimal::from(10000i64);
let current_value = Decimal::from(12500i64);

let roi = position_roi(entry_value, current_value)?;
// roi = 25%
```

## Risk-Adjusted Returns

### Sharpe Ratio Components

```rust
use risk_metrics::{excess_return, Decimal};

let portfolio_return = Decimal::new(12, 2);  // 12%
let risk_free_rate = Decimal::new(4, 2);     // 4%

let excess = excess_return(portfolio_return, risk_free_rate)?;
// excess = 8%
```

## Practical Examples

### Position Sizing with Kelly Criterion

```rust
use risk_metrics::Decimal;

fn kelly_fraction(
    win_probability: Decimal,
    win_loss_ratio: Decimal,
) -> Result<Decimal, ArithmeticError> {
    // Kelly % = W - (1-W)/R
    // where W = win probability, R = win/loss ratio
    let loss_probability = Decimal::ONE.checked_sub(win_probability)?;
    let second_term = loss_probability.checked_div(win_loss_ratio)?;
    win_probability.checked_sub(second_term)
}

let win_rate = Decimal::new(55, 2);    // 55% win rate
let win_loss = Decimal::new(15, 1);    // 1.5:1 win/loss ratio

let kelly = kelly_fraction(win_rate, win_loss)?;
// Use half-Kelly for safety: kelly / 2
```

### Portfolio Position Limits

```rust
fn validate_position(
    new_position: Decimal,
    portfolio_value: Decimal,
    max_concentration: Decimal,  // e.g., 10%
) -> bool {
    let concentration = new_position
        .checked_div(portfolio_value)
        .unwrap_or(Decimal::MAX);

    concentration <= max_concentration
}
```

### Liquidation-Safe Position

```rust
use risk_metrics::{max_borrowable, health_factor, Decimal};

fn safe_position_size(
    available_collateral: Decimal,
    threshold: Decimal,
    target_hf: Decimal,
    price_buffer: Decimal,  // e.g., 20% buffer for volatility
) -> Result<Decimal, ArithmeticError> {
    // Reduce effective collateral by price buffer
    let buffered_collateral = available_collateral
        .checked_mul(Decimal::ONE.checked_sub(price_buffer)?)?;

    max_borrowable(buffered_collateral, threshold, target_hf)
}
```

### Break-Even Analysis

```rust
fn break_even_price(
    entry_price: Decimal,
    position_size: Decimal,
    fees: Decimal,
    is_long: bool,
) -> Result<Decimal, ArithmeticError> {
    let fee_per_unit = fees.checked_div(position_size)?;

    if is_long {
        entry_price.checked_add(fee_per_unit)
    } else {
        entry_price.checked_sub(fee_per_unit)
    }
}
```

## Position Tracking

```rust
struct Position {
    asset: String,
    entry_price: Decimal,
    size: Decimal,
    is_long: bool,
    stop_loss: Option<Decimal>,
    take_profit: Option<Decimal>,
}

impl Position {
    fn value_at(&self, price: Decimal) -> Result<Decimal, ArithmeticError> {
        self.size.checked_mul(price)
    }

    fn pnl_at(&self, price: Decimal) -> Result<Decimal, ArithmeticError> {
        let diff = if self.is_long {
            price.checked_sub(self.entry_price)?
        } else {
            self.entry_price.checked_sub(price)?
        };
        diff.checked_mul(self.size)
    }

    fn should_close(&self, price: Decimal) -> bool {
        if let Some(sl) = self.stop_loss {
            if (self.is_long && price <= sl) || (!self.is_long && price >= sl) {
                return true;
            }
        }
        if let Some(tp) = self.take_profit {
            if (self.is_long && price >= tp) || (!self.is_long && price <= tp) {
                return true;
            }
        }
        false
    }
}
```
