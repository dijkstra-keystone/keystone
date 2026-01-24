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
