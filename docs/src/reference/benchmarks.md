# Benchmarks

Performance characteristics of Keystone operations measured with Criterion.

## Arithmetic Operations

| Operation | Time (ns) | Throughput |
|-----------|-----------|------------|
| Addition | ~8 | 125M ops/sec |
| Subtraction | ~8 | 125M ops/sec |
| Multiplication | ~8 | 125M ops/sec |
| Division | ~36 | 28M ops/sec |
| Remainder | ~40 | 25M ops/sec |

## Rounding Operations

| Operation | Time (ns) |
|-----------|-----------|
| round_dp (2 places) | ~15 |
| floor | ~12 |
| ceil | ~12 |
| trunc | ~10 |

## Financial Calculations

| Operation | Time (ns) |
|-----------|-----------|
| simple_interest | ~25 |
| compound_interest (12 periods, 5 years) | ~850 |
| future_value (10 periods) | ~200 |
| present_value (10 periods) | ~220 |
| effective_annual_rate | ~180 |

## Risk Metrics

| Operation | Time (ns) |
|-----------|-----------|
| health_factor | ~45 |
| liquidation_price | ~50 |
| collateral_ratio | ~40 |
| max_borrowable | ~55 |

## precision-core vs rust_decimal

Measured with Criterion.rs on identical hardware. precision-core wraps rust_decimal
with checked arithmetic, overflow detection, and formal verification boundaries.

| Operation | precision-core (ns) | rust_decimal (ns) | Overhead |
|-----------|--------------------:|-------------------:|---------:|
| Addition | 7.3 | 7.0 | ~4% |
| Subtraction | 8.0 | 7.5 | ~7% |
| Multiplication | 8.7 | 7.7 | ~13% |
| Division | 32.8 | 32.4 | ~1% |
| mul_div (compound) | 22.7 | 20.4 | ~11% |
| Compound interest (12×) | 127.0 | 105.0 | ~21% |
| Health factor (mul+div) | 23.4 | 21.2 | ~10% |
| Swap output (3 ops) | 73.8 | 69.9 | ~6% |
| Large value mul | 16.7 | 16.1 | ~4% |
| Large value div | 70.7 | 69.4 | ~2% |

The wrapper overhead ranges from 1-13% for individual operations and 6-21% for
compound operations. This cost buys:

- Checked arithmetic that returns `Option` instead of panicking
- Formal verification via Kani proof harnesses
- Consistent error types (`ArithmeticError`)
- DeFi-specific operations (health factor, swap math)

Run the comparison benchmark:

```bash
cargo bench --package precision-core --bench comparison
```

## Running Benchmarks

```bash
cd keystone
cargo bench
```

### Specific Benchmark

```bash
cargo bench --bench arithmetic
cargo bench --bench financial
cargo bench --bench risk
```

### With Baseline Comparison

```bash
# Save baseline
cargo bench -- --save-baseline main

# Compare to baseline
cargo bench -- --baseline main
```

## Benchmark Code

Located in `crates/precision-core/benches/`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use precision_core::Decimal;

fn bench_addition(c: &mut Criterion) {
    let a = Decimal::from(12345i64);
    let b = Decimal::from(67890i64);

    c.bench_function("decimal_add", |bencher| {
        bencher.iter(|| black_box(a).checked_add(black_box(b)))
    });
}

criterion_group!(benches, bench_addition);
criterion_main!(benches);
```

## Stylus Gas Benchmarks

Keystone Stylus contracts deployed on Arbitrum One.

### Deployed Contracts

| Contract | Address | Size |
|----------|---------|------|
| stylus-lending | `0x4dff9348275ac3c24e2d3abf54af61d3ebee1585` | 12.2 KB |
| stylus-amm | `0x9615cc2f65d8bbe4cdc80343db75a6ec32da93cd` | 16.9 KB |
| stylus-vault | `0xdaf8f1a5f8025210f07665d4ccf2d2c0622a41fa` | 14.4 KB |

### Measured Gas Usage (Arbitrum One)

| Contract | Function | Gas |
|----------|----------|-----|
| stylus-lending | `calculateHealthFactor` | 59,853 |
| stylus-lending | `isLiquidatable` | 59,851 |
| stylus-amm | `calculateSpotPrice` | 60,026 |
| stylus-amm | `calculateSwapOutput` | 62,651 |
| stylus-amm | `calculateLiquidityMint` | 61,142 |
| stylus-vault | `calculateSharePrice` | 59,082 |
| stylus-vault | `calculateSharesForDeposit` | 59,396 |
| stylus-vault | `calculateAssetsForRedeem` | 59,452 |
| stylus-vault | `calculateCompoundYield` (30 periods) | 59,904 |
| stylus-vault | `calculateApyFromApr` (365 compounds) | 75,656 |
| stylus-vault | `calculatePerformanceFee` | 60,799 |
| stylus-vault | `calculateManagementFee` | 61,122 |

### Gas Comparison: Stylus vs Solidity (Measured)

Solidity benchmark contract: `0x41d4f095Da18Fd25c28CDbE0532a6fb730bbB9CF`

| Operation | Stylus (gas) | Solidity (gas) | Winner |
|-----------|--------------|----------------|--------|
| Share price (1 div) | 58,742 | 22,606 | Solidity 62% cheaper |
| Shares for deposit (2 ops) | 59,005 | 22,898 | Solidity 61% cheaper |
| Compound yield (30 loops) | 59,513 | 33,205 | Solidity 44% cheaper |
| APY from APR (365 loops) | 75,316 | 148,881 | **Stylus 49% cheaper** |

### Key Insight

**Simple arithmetic**: Solidity wins due to lower base overhead.

**Loop-heavy computation**: Stylus wins significantly. As iterations increase, Stylus becomes more efficient because WASM opcodes cost ~100x less than EVM opcodes.

### When to Use Stylus

Use Keystone + Stylus when:
- Calculations involve many iterations (compound interest, Monte Carlo)
- 28-digit precision is required (vs Solidity's 18-digit practical limit)
- Cross-platform determinism matters (same results everywhere)

Use Solidity when:
- Simple arithmetic only
- Gas optimization is critical for single operations
- Existing Solidity codebase

### Break-Even Analysis

Based on measured data, Stylus becomes cheaper at approximately **100+ loop iterations** per call.

### Why Stylus is Cheaper

1. **WASM Execution**: WASM opcodes cost ~1/100th of EVM opcodes
2. **No Storage Overhead**: Pure computation functions avoid SLOAD/SSTORE
3. **Efficient Loops**: Iteration-heavy calculations (compound interest) benefit most
4. **Native Integer Ops**: 128-bit arithmetic is native in WASM

### Running Gas Benchmarks

Install Foundry:
```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

Estimate gas for a call:
```bash
cast estimate 0x4dff9348275ac3c24e2d3abf54af61d3ebee1585 \
  "calculateHealthFactor(uint256,uint256)(uint256)" \
  10000000000000000000000 5000000000000000000000 \
  --rpc-url https://arb1.arbitrum.io/rpc
```

### Activation Costs

One-time costs paid at deployment:

| Contract | Activation Fee |
|----------|---------------|
| stylus-lending | 0.000090 ETH |
| stylus-amm | 0.000103 ETH |
| stylus-vault | 0.000099 ETH |

After activation, calls use standard Arbitrum gas pricing.

---

## WASM Performance

WASM operations include JS-WASM boundary overhead:

| Operation | Native (ns) | WASM (ns) | Overhead |
|-----------|-------------|-----------|----------|
| Addition | 8 | ~50 | ~6x |
| Division | 36 | ~100 | ~3x |
| compound_interest | 850 | ~1200 | ~1.4x |

### WASM Binary Size

| Build | Size |
|-------|------|
| Debug | ~450 KB |
| Release | ~97 KB |
| Release + wasm-opt | ~85 KB |

## Optimization Settings

From `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = "fat"          # Full link-time optimization
codegen-units = 1    # Better optimization
panic = "abort"      # Smaller binary
strip = true         # Remove symbols
overflow-checks = true  # Keep overflow checking
```

## Memory Usage

| Type | Size |
|------|------|
| Decimal | 16 bytes |
| RoundingMode | 1 byte |
| ArithmeticError | 1 byte |

Stack-only allocation for all operations.

## Comparison with Alternatives

### vs JavaScript Number

```javascript
// JavaScript floating-point
0.1 + 0.2  // 0.30000000000000004

// Keystone
keystone.add("0.1", "0.2")  // "0.3"
```

Keystone is slower but guarantees correctness for financial calculations.

### vs BigInt

JavaScript BigInt handles integers only. Keystone provides:
- Decimal precision up to 28 digits
- Native rounding modes
- Financial functions

### vs decimal.js

Similar precision, but Keystone provides:
- Deterministic cross-platform results
- Smaller WASM bundle
- ZK-proof compatibility
- Rust native performance

## Profiling

### CPU Profiling

```bash
cargo bench --bench arithmetic -- --profile-time 10
```

### Memory Profiling

```bash
cargo bench --bench arithmetic -- --plotting-backend disabled
```

## CI Benchmarks

Benchmarks run on every PR to detect regressions:

```yaml
- name: Run benchmarks
  run: cargo bench -- --noplot

- name: Compare with main
  run: |
    cargo bench -- --save-baseline pr
    git checkout main
    cargo bench -- --baseline pr
```
