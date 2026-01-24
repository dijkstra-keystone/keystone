# Thread: Health Factor — The Number That Saves Your Collateral

## Tweet 1 (Hook)
Liquidated? Probably didn't watch your health factor.

Here's the single most important number in DeFi lending, explained in 2 minutes. 🧵

## Tweet 2 (Definition)
Health Factor = (Collateral × Liquidation Threshold) / Debt

HF > 1.0: You're safe
HF = 1.0: You're at the edge
HF < 1.0: Liquidators take your collateral

## Tweet 3 (Example)
You deposit $10,000 ETH (80% threshold)
You borrow $5,000 USDC

HF = (10,000 × 0.80) / 5,000 = 1.6

Comfortable. For now.

## Tweet 4 (Price Drops)
ETH drops 20%. Your collateral is now $8,000.

HF = (8,000 × 0.80) / 5,000 = 1.28

Still safe, but getting nervous.

## Tweet 5 (Danger Zone)
ETH drops 37.5%. Collateral: $6,250.

HF = (6,250 × 0.80) / 5,000 = 1.0

One more tick down and liquidators come for your ETH at a discount.

## Tweet 6 (Liquidation Price)
Know your liquidation price:

Liq Price = Debt / (Collateral Amount × Threshold)
         = 5,000 / (5 ETH × 0.80)
         = $1,250 per ETH

If ETH hits $1,250, you get liquidated.

## Tweet 7 (Code)
```rust
use risk_metrics::{health_factor, liquidation_price};

let hf = health_factor(collateral, debt, threshold)?;
let liq = liquidation_price(eth_amount, debt, threshold)?;

if hf < Decimal::new(12, 1) {  // < 1.2
    alert_user("Add collateral!");
}
```

## Tweet 8 (Safe Targets)
Target health factors:
• Conservative: 2.0+ (50% buffer)
• Moderate: 1.5 (33% buffer)
• Aggressive: 1.2 (17% buffer)

Volatility matters. ETH? Stay above 1.5.

## Tweet 9 (CTA)
Build position monitoring with deterministic math.

Keystone risk-metrics crate:
• health_factor()
• liquidation_price()
• max_borrowable()

Docs: docs.dijkstrakeystone.com/risk/health-factor

---
*Character counts verified. Ready for scheduling.*
