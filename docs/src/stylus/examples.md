# Example Contracts

Keystone includes five Stylus example contracts demonstrating precision arithmetic for different DeFi use cases.

## Deployed Contracts (Arbitrum One)

| Contract | Address | Arbiscan |
|----------|---------|----------|
| stylus-lending | `0x4dff9348275ac3c24e2d3abf54af61d3ebee1585` | [View](https://arbiscan.io/address/0x4dff9348275ac3c24e2d3abf54af61d3ebee1585) |
| stylus-amm | `0x9615cc2f65d8bbe4cdc80343db75a6ec32da93cd` | [View](https://arbiscan.io/address/0x9615cc2f65d8bbe4cdc80343db75a6ec32da93cd) |
| stylus-vault | `0xdaf8f1a5f8025210f07665d4ccf2d2c0622a41fa` | [View](https://arbiscan.io/address/0xdaf8f1a5f8025210f07665d4ccf2d2c0622a41fa) |

---

## stylus-lending

**Location:** `examples/stylus-lending/`

Lending protocol calculations using Keystone for deterministic risk assessment.

### Functions

| Function | Description |
|----------|-------------|
| `calculate_health_factor` | (Collateral × Threshold) / Debt |
| `calculate_liquidation_price` | Debt / (Collateral × Threshold) |
| `calculate_max_borrow` | Maximum borrowable given collateral |
| `is_liquidatable` | Check if health factor < 1 |
| `calculate_liquidation_amounts` | Debt coverage with liquidation bonus |

### Example Usage

```rust
// Health factor calculation
// Collateral: $10,000, Debt: $5,000, Threshold: 80%
// HF = (10000 * 0.8) / 5000 = 1.6

let hf = contract.calculate_health_factor(
    U256::from(10000) * U256::from(10).pow(U256::from(18)), // collateral
    U256::from(5000) * U256::from(10).pow(U256::from(18)),  // debt
)?;
// Returns 1.6e18
```

### Target Protocols

- Aave-style lending markets
- Radiant, Lodestar on Arbitrum
- Any protocol using health factor for liquidation

---

## stylus-amm

**Location:** `examples/stylus-amm/`

Constant product AMM (x*y=k) calculations for decentralized exchanges.

### Functions

| Function | Description |
|----------|-------------|
| `calculate_swap_output` | Output amount for given input |
| `calculate_price_impact` | Price impact as percentage |
| `calculate_swap_input` | Required input for desired output |
| `calculate_spot_price` | Current pool price |
| `calculate_liquidity_mint` | LP tokens for deposit |
| `calculate_liquidity_burn` | Assets returned for LP redemption |

### Example Usage

```rust
// Swap calculation with 0.3% fee
// Pool: 1000 ETH / 2,000,000 USDC
// Swap 10 ETH for USDC

contract.set_fee(U256::from(30)); // 30 bps = 0.3%

let output = contract.calculate_swap_output(
    U256::from(1000) * SCALE,     // reserve_in (ETH)
    U256::from(2000000) * SCALE,  // reserve_out (USDC)
    U256::from(10) * SCALE,       // amount_in
)?;
// Returns ~19,841 USDC (with fee and slippage)
```

### Target Protocols

- Uniswap V2-style AMMs
- Camelot, Trader Joe on Arbitrum
- Any constant product DEX

---

## stylus-vault

**Location:** `examples/stylus-vault/`

ERC4626-compatible vault calculations for yield aggregators.

### Functions

| Function | Description |
|----------|-------------|
| `calculate_shares_for_deposit` | Shares to mint for deposit |
| `calculate_assets_for_redeem` | Assets returned for share redemption |
| `calculate_share_price` | Current price per share |
| `calculate_compound_yield` | Compounded return over periods |
| `calculate_apy_from_apr` | Convert APR to APY |
| `calculate_performance_fee` | Fee on gains |
| `calculate_management_fee` | Time-based fee |
| `calculate_net_asset_value` | NAV per share |

### Example Usage

```rust
// Calculate shares for $1000 deposit
// Vault has $100,000 total assets, 95,000 shares outstanding

let shares = contract.calculate_shares_for_deposit(
    U256::from(1000) * SCALE,    // deposit amount
    U256::from(100000) * SCALE,  // total assets
    U256::from(95000) * SCALE,   // total supply
)?;
// Returns 950 shares (share price = 100000/95000 ≈ 1.0526)
```

```rust
// Calculate APY from 5% APR with daily compounding

let apy_bps = contract.calculate_apy_from_apr(
    U256::from(500) * SCALE,  // 500 bps = 5% APR
    U256::from(365),          // daily compounding
)?;
// Returns ~512.67 bps = 5.1267% APY
```

### Target Protocols

- ERC4626 vaults
- Yearn-style yield aggregators
- Pendle, Jones DAO on Arbitrum

---

## stylus-options

**Location:** `examples/stylus-options/`

On-chain Black-Scholes options pricing with 128-bit decimal precision.

### Functions

| Function | Description |
|----------|-------------|
| `price_call` | European call option price (Black-Scholes) |
| `price_put` | European put option price |
| `call_option_greeks` | Delta, Gamma, Theta, Vega, Rho for calls |
| `put_option_greeks` | Greeks for puts |
| `calculate_iv` | Implied volatility from market price |
| `put_call_parity_check` | Verify C - P = S - Ke^(-rT) |

### Target Protocols

- Dopex (Arbitrum options)
- Rysk Finance (options AMM)
- Any protocol requiring on-chain `exp()`, `ln()`, `sqrt()` for BSM pricing

---

## stylus-oracle

**Location:** `examples/stylus-oracle/`

RedStone oracle integration for price-aware DeFi calculations.

### Functions

| Function | Description |
|----------|-------------|
| `calculate_health_factor_with_prices` | Health factor using live oracle prices |
| `calculate_liquidation_price_with_oracle` | Oracle-based liquidation trigger |
| `calculate_max_borrow_with_prices` | Max borrow with real-time pricing |
| `is_liquidatable_with_prices` | Liquidation status with live data |
| `calculate_twap` | Time-weighted average price |
| `calculate_price_deviation` | Anomaly detection vs median |

### Target Protocols

- Any DeFi protocol using RedStone pull-based oracles on Arbitrum
- Cross-VM interoperability between WASM and EVM contracts

---

## Building Examples

```bash
# Build all examples
cd examples/stylus-lending && cargo build --release
cd ../stylus-amm && cargo build --release
cd ../stylus-vault && cargo build --release
```

## Deploying Examples

```bash
cd examples/stylus-lending
cargo stylus deploy \
  --private-key-path=../../key.txt \
  --endpoint=https://sepolia-rollup.arbitrum.io/rpc
```

Replace with your private key path and desired network.
