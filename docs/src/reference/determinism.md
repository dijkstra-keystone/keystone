# Determinism

Keystone guarantees identical results across all platforms and execution environments.

## Why Determinism Matters

### Financial Applications

- Audit trails require reproducible calculations
- Reconciliation between systems
- Regulatory compliance

### Zero-Knowledge Proofs

- Prover and verifier must compute identical results
- Any divergence breaks proof validity
- Cross-platform proof generation

### Distributed Systems

- Consensus requires identical state transitions
- Smart contract execution
- Multi-party computation

## Guarantees

### Bit-Exact Results

The same inputs always produce the same outputs, regardless of:

- Operating system (Linux, macOS, Windows)
- CPU architecture (x86_64, ARM64, WASM)
- Compiler version
- Optimization level

### No Floating-Point

Keystone uses fixed-point decimal arithmetic:

```rust
// Floating-point: non-deterministic
let a: f64 = 0.1;
let b: f64 = 0.2;
let c = a + b;  // May vary by platform

// Keystone: deterministic
let a = Decimal::new(1, 1);  // 0.1
let b = Decimal::new(2, 1);  // 0.2
let c = a.checked_add(b);    // Always 0.3
```

### no_std Core

The core library has no OS dependencies:

```rust
#![no_std]  // No standard library
#![forbid(unsafe_code)]  // No undefined behavior
```

## Verification

### Test Vectors

Pre-computed test vectors verify determinism:

```rust
#[test]
fn test_determinism_vectors() {
    let vectors = [
        ("100", "3", "div", "33.333333333333333333333333333"),
        ("0.1", "0.2", "add", "0.3"),
        ("999999999999999999999999999", "2", "mul",
         "1999999999999999999999999998"),
    ];

    for (a, b, op, expected) in vectors {
        let a: Decimal = a.parse().unwrap();
        let b: Decimal = b.parse().unwrap();
        let result = match op {
            "add" => a.checked_add(b),
            "mul" => a.checked_mul(b),
            "div" => a.checked_div(b),
            _ => panic!("unknown op"),
        };
        assert_eq!(result.unwrap().to_string(), expected);
    }
}
```

### Cross-Platform CI

CI runs on multiple platforms:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    target:
      - x86_64-unknown-linux-gnu
      - x86_64-pc-windows-msvc
      - x86_64-apple-darwin
      - aarch64-apple-darwin
      - wasm32-unknown-unknown
```

### Property-Based Testing

Random inputs verify consistency:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn add_commutative(a: i64, b: i64) {
        let da = Decimal::from(a);
        let db = Decimal::from(b);

        let ab = da.checked_add(db);
        let ba = db.checked_add(da);

        assert_eq!(ab, ba);
    }
}
```

## ZK Compatibility

### SP1 Integration

Keystone is designed for use in SP1 zkVM:

```rust
// In SP1 program
#![no_main]
sp1_zkvm::entrypoint!(main);

use precision_core::Decimal;

pub fn main() {
    let a: Decimal = sp1_zkvm::io::read();
    let b: Decimal = sp1_zkvm::io::read();

    let result = a.checked_mul(b).expect("overflow");

    sp1_zkvm::io::commit(&result);
}
```

### Proof Generation

```rust
// Generate proof
let (pk, vk) = client.setup(ELF);
let mut stdin = SP1Stdin::new();
stdin.write(&a);
stdin.write(&b);

let proof = client.prove(&pk, stdin)?;

// Verify anywhere
client.verify(&proof, &vk)?;
```

## Implementation Details

### Decimal Representation

128-bit fixed-point with explicit scale:

```rust
struct Decimal {
    // 96-bit mantissa + 1-bit sign
    mantissa: i128,
    // Scale: number of decimal places (0-28)
    scale: u32,
}
```

### Normalization

Canonical form ensures comparison stability:

```rust
let a = Decimal::new(100, 2);   // 1.00
let b = Decimal::new(1, 0);     // 1

// Internal comparison normalizes
assert!(a == b);

// Explicit normalization
let normalized = a.normalize(); // 1 (scale = 0)
```

### Rounding Specification

Each rounding mode has precise semantics:

| Mode | Tie-Breaking Rule |
|------|-------------------|
| HalfEven | To nearest even digit |
| HalfUp | Away from zero |
| HalfDown | Toward zero |

## Edge Cases

### Maximum Precision

28 significant digits:

```rust
let max_precision = Decimal::new(
    9999999999999999999999999999i128,
    0
);
```

### Minimum Value

```rust
let min_scale = Decimal::new(1, 28);
// 0.0000000000000000000000000001
```

### Overflow Behavior

Explicit error handling, never silent wraparound:

```rust
let max = Decimal::MAX;
let result = max.checked_add(Decimal::ONE);
assert!(result.is_none());  // Never silently wraps
```
