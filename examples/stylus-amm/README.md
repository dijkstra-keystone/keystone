# Stylus AMM Example

Constant product AMM (x×y=k) calculations using Keystone precision arithmetic on Arbitrum Stylus.

## Functions

| Function | Description |
|----------|-------------|
| `calculateSwapOutput` | Output amount for given input (with fee) |
| `calculateSwapInput` | Required input for desired output |
| `calculatePriceImpact` | Slippage percentage for a trade |
| `calculateSpotPrice` | Current marginal price |
| `calculateLiquidityMint` | LP shares for a deposit |
| `calculateLiquidityBurn` | Assets returned for LP share redemption |

## Deployed

- **Arbitrum One**: `0x9615cc2f65d8bbe4cdc80343db75a6ec32da93cd`

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

Uniswap V2-style AMMs, Camelot DEX, SushiSwap — constant product pools requiring precision swap math and fee calculation.
