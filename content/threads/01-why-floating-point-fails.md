# Thread: Why Floating-Point Arithmetic Fails in Finance

## Tweet 1 (Hook)
0.1 + 0.2 = 0.30000000000000004

This bug has cost DeFi protocols millions. Here's why floating-point math is dangerous for financial applications, and how we fixed it. 🧵

## Tweet 2 (Problem)
JavaScript, Python, and most languages use IEEE 754 floating-point.

Great for games and graphics. Catastrophic for money.

The issue: 0.1 cannot be exactly represented in binary. Every calculation introduces tiny errors that compound.

## Tweet 3 (Real Impact)
These rounding errors have caused:
• Failed balance checks in lending protocols
• Arbitrage exploits from price discrepancies
• Failed ZK proof verification (prover ≠ verifier)
• Audit nightmares tracing calculation differences

## Tweet 4 (Determinism Problem)
Worse: floating-point isn't even deterministic.

The same calculation can give different results on:
• Different CPUs (x86 vs ARM)
• Different compilers
• Different optimization levels

Consensus systems require bit-exact reproducibility.

## Tweet 5 (Solution Intro)
We built Keystone: deterministic 128-bit decimal arithmetic.

• 28 significant digits
• 7 rounding modes (including banker's rounding)
• Identical results on ALL platforms
• no_std for WASM and embedded

## Tweet 6 (Code Example)
```rust
use precision_core::Decimal;

let a = Decimal::new(1, 1);  // 0.1
let b = Decimal::new(2, 1);  // 0.2
let c = a.checked_add(b)?;   // 0.3 exactly

// Same result on x86, ARM, WASM. Always.
```

## Tweet 7 (ZK Compatible)
Keystone is ZK-forward.

Running the same calculation in SP1 zkVM produces identical outputs to the verifier.

Proofs that actually verify. Novel concept.

## Tweet 8 (CTA)
Now open source under MIT/Apache-2.0.

GitHub: github.com/dijkstra-keystone/keystone
Docs: docs.dijkstrakeystone.com

Star if you've been burned by floating-point math.

---
*Character counts verified. Ready for scheduling.*
