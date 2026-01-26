# Stylus Integration

Keystone's `no_std` core makes it compatible with Arbitrum Stylus smart contracts. This guide covers building and deploying Stylus contracts that use Keystone for precision arithmetic.

## Prerequisites

### Install Rust and WASM target

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

### Install cargo-stylus

```bash
cargo install cargo-stylus
```

### Get testnet ETH

For deployment, you'll need Arbitrum Sepolia ETH from a faucet.

## Project Setup

### Cargo.toml

```toml
[package]
name = "my-stylus-contract"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
precision-core = { version = "0.1", default-features = false }
stylus-sdk = "=0.9.0"
alloy-primitives = "=0.8.20"
ruint = ">=1.12.3, <1.17"

[features]
default = ["mini-alloc"]
mini-alloc = ["stylus-sdk/mini-alloc"]
export-abi = ["stylus-sdk/export-abi"]

[lib]
crate-type = ["lib", "cdylib"]

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### Contract Structure

```rust
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use alloy_primitives::U256;
use precision_core::{Decimal, RoundingMode};
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct MyContract {
        uint256 some_value;
    }
}

#[public]
impl MyContract {
    pub fn my_function(&self, input: U256) -> Result<U256, Vec<u8>> {
        // Use Keystone precision arithmetic
        let decimal = u256_to_decimal(input);
        let result = decimal.checked_mul(Decimal::from(2i64))
            .ok_or_else(|| b"overflow".to_vec())?;
        Ok(decimal_to_u256(result))
    }
}
```

## U256 Conversion

Stylus uses `U256` for numbers, while Keystone uses `Decimal`. Convert between them:

```rust
const SCALE: u64 = 1_000_000_000_000_000_000; // 1e18

fn u256_to_decimal(value: U256) -> Decimal {
    let lo: u128 = value.as_limbs()[0] as u128
        | ((value.as_limbs()[1] as u128) << 64);
    let raw = Decimal::from(lo);
    raw.checked_div(Decimal::from(SCALE))
        .unwrap_or(Decimal::MAX)
}

fn decimal_to_u256(value: Decimal) -> U256 {
    let scaled = value
        .checked_mul(Decimal::from(SCALE))
        .unwrap_or(Decimal::MAX)
        .round(0, RoundingMode::TowardZero);
    let (mantissa, _scale) = scaled.to_parts();
    U256::from(mantissa.unsigned_abs())
}
```

## Building

### Check compilation

```bash
cargo stylus check
```

### Build for release

```bash
cargo build --release --target wasm32-unknown-unknown
```

### Export ABI

```bash
cargo stylus export-abi
```

This generates Solidity-compatible ABI for your contract.

## Deployment

### Deploy to Arbitrum Sepolia

```bash
cargo stylus deploy \
  --private-key-path=./key.txt \
  --endpoint=https://sepolia-rollup.arbitrum.io/rpc
```

The CLI will output your contract address after deployment.

### Deploy to Arbitrum One (mainnet)

```bash
cargo stylus deploy \
  --private-key-path=./key.txt \
  --endpoint=https://arb1.arbitrum.io/rpc
```

## Contract Interaction

### Using Foundry Cast

Read-only call:

```bash
cast call <address> "myFunction(uint256)(uint256)" 1000000000000000000 \
  --rpc-url https://sepolia-rollup.arbitrum.io/rpc
```

Transaction:

```bash
cast send <address> "myFunction(uint256)" 1000000000000000000 \
  --private-key <key> \
  --rpc-url https://sepolia-rollup.arbitrum.io/rpc
```

## Example Contracts

Keystone provides three reference implementations:

### stylus-lending

Health factor, liquidation price, and max borrow calculations for lending protocols.

```bash
cd examples/stylus-lending
cargo stylus check
```

### stylus-amm

Constant product AMM calculations: swap output, price impact, liquidity math.

```bash
cd examples/stylus-amm
cargo stylus check
```

### stylus-vault

ERC4626-style vault calculations: share price, compound yield, APY conversion.

```bash
cd examples/stylus-vault
cargo stylus check
```

## Best Practices

### Error Handling

Use checked arithmetic and return errors as `Vec<u8>`:

```rust
let result = a.checked_mul(b)
    .ok_or_else(|| b"multiplication overflow".to_vec())?;
```

### Precision

Keystone provides 28 significant digits. For most DeFi use cases, this exceeds requirements.

### Gas Optimization

- Use `opt-level = "z"` for smallest WASM size
- Enable LTO and strip debug info
- The 24KB compressed size limit requires efficient code

### Rounding

Choose rounding modes appropriate for financial context:

- `HalfUp`: Traditional rounding (0.5 rounds up)
- `HalfEven`: Banker's rounding (0.5 rounds to nearest even)
- `TowardZero`: Truncation (always toward zero)
- `Down`: Floor (always toward negative infinity)

## Troubleshooting

### Contract too large

If your contract exceeds 24KB:

1. Remove unused dependencies
2. Use `#[inline(never)]` on rarely-called functions
3. Consider splitting into multiple contracts

### Build failures

Ensure all dependencies support `no_std`:

```toml
precision-core = { version = "0.1", default-features = false }
```

### Deployment fails

1. Verify you have sufficient ETH for gas
2. Check RPC endpoint is accessible
3. Ensure private key file has correct format
