# Stylus Oracle Integration Example

Demonstrates integrating [RedStone](https://redstone.finance) oracle price feeds with Keystone precision arithmetic for accurate DeFi calculations on Arbitrum Stylus.

## Overview

RedStone uses a pull-based oracle model where price data is passed in transaction calldata and verified cryptographically on-chain. This provides:
- **30%+ gas savings** vs traditional push oracles
- **~3K gas** per feed per signer for cryptographic verification
- **Fresh prices** on every transaction (no staleness issues)

## Features

### Oracle-Integrated Lending
- `calculate_health_factor_with_prices` - Health factor using live oracle prices
- `calculate_liquidation_price_with_oracle` - Liquidation trigger price
- `calculate_max_borrow_with_prices` - Maximum borrowable amount
- `is_liquidatable_with_prices` - Liquidation status check
- `calculate_liquidation_with_prices` - Liquidation amounts with bonus

### Price Aggregation
- `calculate_twap` - Time-weighted average price from multiple updates
- `calculate_price_deviation` - Anomaly detection vs median price

## Usage

### Building

```bash
cargo build --release --target wasm32-unknown-unknown
```

### With cargo-stylus

```bash
cargo stylus check
cargo stylus export-abi
```

### Deploying

```bash
cargo stylus deploy \
  --private-key-path=./key.txt \
  --endpoint=https://sepolia-rollup.arbitrum.io/rpc
```

## Price Format

RedStone prices use 8 decimal precision:
- `$2,000.00` = `200_000_000_000` (2000 * 10^8)
- `$1.00` = `100_000_000` (1 * 10^8)

Internal calculations use Keystone's 128-bit Decimal type for maximum precision.

## Example: Health Factor Calculation

```rust
// 10 ETH at $2000 = $20,000 collateral
// 10,000 USDC at $1 = $10,000 debt
// Threshold 80%

let collateral_amount = U256::from(10u64) * U256::from(1e18);
let collateral_price = U256::from(200_000_000_000u128); // $2000
let debt_amount = U256::from(10_000u64) * U256::from(1e18);
let debt_price = U256::from(100_000_000u128); // $1

let health_factor = contract.calculate_health_factor_with_prices(
    collateral_amount,
    collateral_price,
    debt_amount,
    debt_price,
)?;

// health_factor = 1.6e18 (160%)
```

## RedStone Integration

When the full RedStone Stylus SDK is available, add to `Cargo.toml`:

```toml
[dependencies]
redstone-stylus = "0.1"
```

Then verify signatures:

```rust
use redstone_stylus::{verify_prices, RedStonePayload};

// Extract RedStone payload from calldata
let payload = RedStonePayload::from_calldata()?;

// Verify signatures (uses openzeppelin_stylus ECDSA)
let prices = verify_prices(
    &payload,
    &trusted_signers,
    min_signers,
    max_staleness,
)?;

// Use verified prices with Keystone calculations
let hf = self.calculate_health_factor_with_prices(
    collateral_amount,
    prices.get("ETH")?,
    debt_amount,
    prices.get("USDC")?,
)?;
```

## Gas Comparison

| Operation | EVM (Chainlink) | Stylus (RedStone) | Savings |
|-----------|-----------------|-------------------|---------|
| Get price | ~2,500 gas | ~3,000 gas* | -20% |
| Health factor calc | ~800 gas | ~150 gas | 81% |
| Combined | ~3,300 gas | ~3,150 gas | **5%** |

*RedStone's pull model verifies signatures on-chain, but computation savings offset this cost.

For compute-heavy operations (multiple prices, TWAP), Stylus savings increase significantly.

## References

- [RedStone Stylus Blog Post](https://blog.redstone.finance/2025/11/04/arbitrum-stylus-wasm-superior-performance-beyond-evm-limitations/)
- [Arbitrum RedStone Integration](https://blog.arbitrum.io/how-redstone-is-advancing-oracle-capabilities-with-stylus/)
- [RedStone Documentation](https://docs.redstone.finance/)
