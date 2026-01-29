# Stylus Vault Example

ERC4626-style vault calculations using Keystone precision arithmetic on Arbitrum Stylus.

## Functions

| Function | Description |
|----------|-------------|
| `calculateSharesForDeposit` | Shares to mint (ERC4626 convertToShares) |
| `calculateAssetsForRedeem` | Assets to return (ERC4626 convertToAssets) |
| `calculateSharePrice` | Current price per share |
| `calculateCompoundYield` | principal × (1 + rate)^periods |
| `calculateApyFromApr` | (1 + APR/n)^n - 1 |
| `calculatePerformanceFee` | Fee on gains |
| `calculateManagementFee` | Time-proportional annual fee |
| `calculateNetAssetValue` | (balance + strategy + rewards) / supply |

## Deployed

- **Arbitrum One**: `0xdaf8f1a5f8025210f07665d4ccf2d2c0622a41fa`

## Building

```bash
cargo stylus check
cargo build --release --target wasm32-unknown-unknown
```

## Testing

```bash
cargo test
```

Integration tests use the [Motsu](https://crates.io/crates/motsu) framework.

## Target Protocols

ERC4626 vaults, Yearn-style yield aggregators, Morpho vault curators — any protocol requiring deterministic share price and yield calculations.
