# Stylus Lending Example

DeFi lending protocol calculations using Keystone precision arithmetic on Arbitrum Stylus.

## Functions

| Function | Description |
|----------|-------------|
| `calculateHealthFactor` | (collateral × threshold) / debt |
| `calculateLiquidationPrice` | debt / (collateral × threshold) |
| `calculateMaxBorrow` | (collateral × threshold) / target_hf |
| `isLiquidatable` | health factor < 1.0 check |
| `calculateLiquidationAmounts` | debt coverage and bonus collateral |

## Deployed

- **Arbitrum One**: `0x4dff9348275ac3c24e2d3abf54af61d3ebee1585`

## Building

```bash
cargo stylus check
cargo build --release --target wasm32-unknown-unknown
```

## Testing

```bash
cargo test
```

Integration tests use the [Motsu](https://crates.io/crates/motsu) framework for contract-level testing without a blockchain runtime.

## Target Protocols

Aave, Compound, Morpho — any lending protocol requiring deterministic health factor and liquidation calculations.
