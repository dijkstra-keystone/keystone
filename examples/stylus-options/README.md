# Stylus Options Pricing Example

On-chain Black-Scholes options pricing using Keystone precision arithmetic on Arbitrum Stylus.

## Features

- **European Call/Put Pricing** — Black-Scholes-Merton model with 128-bit decimal precision
- **Greeks Calculation** — Delta, Gamma, Theta, Vega, Rho for calls and puts
- **Implied Volatility** — Newton-Raphson IV solver from market prices
- **Put-Call Parity Check** — Verify pricing consistency (C - P = S - Ke^(-rT))

## Why On-chain Options Math?

DeFi options protocols (Dopex, Rysk, Lyra) require accurate pricing for:
- Fair value calculation for AMM-based options
- Greeks computation for risk management
- IV surface construction from market data
- Put-call parity enforcement

Solidity lacks native support for `exp()`, `ln()`, and `sqrt()` — the core functions in Black-Scholes. Stylus executes these natively in WASM.

## Usage

### Building

```bash
cargo build --release --target wasm32-unknown-unknown
```

### Export ABI

```bash
cargo stylus export-abi
```

### Price a Call Option

```bash
# ETH at $2000, strike $2000, 20% vol, 3 months, 5% rate
cast call <address> \
  "priceCall(uint256,uint256,uint256,uint256)(uint256)" \
  2000000000000000000000 \
  2000000000000000000000 \
  200000000000000000 \
  250000000000000000 \
  --rpc-url https://arb1.arbitrum.io/rpc
```

### Calculate Greeks

```bash
cast call <address> \
  "callOptionGreeks(uint256,uint256,uint256,uint256)(uint256,uint256,uint256,uint256,uint256)" \
  2000000000000000000000 \
  2000000000000000000000 \
  200000000000000000 \
  250000000000000000 \
  --rpc-url https://arb1.arbitrum.io/rpc
```

Returns: (delta, gamma, theta, vega, rho) all scaled by 1e18.

## Parameters

All values use 1e18 scaling:

| Parameter | Example | Meaning |
|-----------|---------|---------|
| spot | 2000e18 | $2,000 underlying price |
| strike | 2000e18 | $2,000 strike price |
| volatility | 0.2e18 | 20% annualized volatility |
| time_to_expiry | 0.25e18 | 3 months (0.25 years) |
| risk_free_rate | 500 bps | 5% stored as basis points |

## Implementation

The contract wraps `financial-calc::options` which implements:

- **Normal CDF**: Hart approximation (1968), 7.5×10⁻⁸ accuracy
- **Black-Scholes**: Standard BSM with `exp()`, `ln()`, `sqrt()` from precision-core
- **Greeks**: Analytical closed-form solutions
- **IV Solver**: Newton-Raphson with vega as derivative, configurable tolerance

## Testing

```bash
cargo test
```

## References

- Black, F. & Scholes, M. (1973). "The Pricing of Options and Corporate Liabilities"
- Hull, J.C. (2018). "Options, Futures, and Other Derivatives"
- [Dopex](https://www.dopex.io/) — Arbitrum options protocol
- [Rysk Finance](https://www.rysk.finance/) — Options AMM requiring Greeks calculation
