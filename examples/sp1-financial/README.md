# SP1 Financial Proofs

ZK-provable financial calculations using Keystone precision arithmetic and Succinct's SP1 zkVM.

## Prerequisites

1. Install the Succinct toolchain:
   ```bash
   curl -L https://sp1.succinct.xyz | bash
   sp1up
   ```

2. Verify installation:
   ```bash
   cargo +succinct --version
   ```

## Building

Build the zkVM program:
```bash
cd script
cargo build --release
```

The build process compiles the program crate to the `riscv32im-succinct-zkvm-elf` target.

## Usage

### Simulation (no proof)

```bash
# Health factor calculation
cargo run --release -- health-factor --collateral 10000000000000000000000 --debt 5000000000000000000000

# Compound interest
cargo run --release -- compound-interest --principal 1000000000000000000000 --rate-bps 500 --periods 12

# AMM swap output
cargo run --release -- swap-output --reserve-in 1000000000000000000000000 --reserve-out 500000000000000000000000 --amount-in 10000000000000000000000
```

### With ZK Proof

Add `--prove` to generate and verify a Groth16 proof:

```bash
cargo run --release -- health-factor --collateral 10000000000000000000000 --debt 5000000000000000000000 --prove
```

## Operations

| Operation | Description |
|-----------|-------------|
| `health-factor` | Lending protocol health factor (collateral × threshold / debt) |
| `compound-interest` | Compound interest over N periods |
| `swap-output` | AMM constant product swap output amount |
| `share-price` | ERC4626 vault share price calculation |

## On-chain Verification

The Groth16 proof can be verified on-chain using the `sp1-verifier` crate at ~200K gas.

```solidity
interface ISP1Verifier {
    function verifyProof(
        bytes32 vkey,
        bytes calldata publicValues,
        bytes calldata proof
    ) external view;
}
```

## Architecture

```
sp1-financial/
├── program/           # zkVM program (compiles to RISC-V)
│   ├── Cargo.toml
│   └── src/main.rs    # Financial calculation logic
└── script/            # Host/prover
    ├── Cargo.toml
    ├── build.rs       # Builds program ELF
    └── src/main.rs    # CLI and proof generation
```

The `program` crate uses `precision-core` for deterministic 128-bit decimal arithmetic.
All calculations produce identical results regardless of where they execute.
