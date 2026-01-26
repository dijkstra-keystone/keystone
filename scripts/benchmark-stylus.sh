#!/bin/bash
# Stylus Gas Benchmark Script
# Requires: foundry (cast)

set -e

export PATH="$HOME/.foundry/bin:$PATH"

RPC_URL="https://arb1.arbitrum.io/rpc"

# Contract addresses (Arbitrum One mainnet)
LENDING="0x4dff9348275ac3c24e2d3abf54af61d3ebee1585"
AMM="0x9615cc2f65d8bbe4cdc80343db75a6ec32da93cd"
VAULT="0xdaf8f1a5f8025210f07665d4ccf2d2c0622a41fa"

# 1e18 scale
SCALE="1000000000000000000"

echo "=== Keystone Stylus Gas Benchmarks ==="
echo ""

# Check if cast is available
if ! command -v cast &> /dev/null; then
    echo "Error: 'cast' not found. Install Foundry first:"
    echo "  curl -L https://foundry.paradigm.xyz | bash"
    echo "  foundryup"
    exit 1
fi

echo "--- stylus-lending ---"
echo "Contract: $LENDING"

# Note: These calls may revert if storage isn't initialized
# In production, you'd initialize the contract first

echo ""
echo "--- stylus-amm ---"
echo "Contract: $AMM"

echo ""
echo "Estimating swap output calculation..."
# calculate_swap_output(reserve_in, reserve_out, amount_in)
# 1000 ETH, 2M USDC, swap 10 ETH
cast estimate $AMM \
  "calculateSwapOutput(uint256,uint256,uint256)(uint256)" \
  "1000$SCALE" "2000000$SCALE" "10$SCALE" \
  --rpc-url $RPC_URL 2>/dev/null || echo "  (requires fee_bps to be set)"

echo ""
echo "Estimating spot price calculation..."
cast estimate $AMM \
  "calculateSpotPrice(uint256,uint256)(uint256)" \
  "1000$SCALE" "2000$SCALE" \
  --rpc-url $RPC_URL 2>/dev/null || echo "  Call failed"

echo ""
echo "--- stylus-vault ---"
echo "Contract: $VAULT"

echo ""
echo "Estimating share price calculation..."
# calculate_share_price(total_assets, total_supply)
cast estimate $VAULT \
  "calculateSharePrice(uint256,uint256)(uint256)" \
  "1000000$SCALE" "950000$SCALE" \
  --rpc-url $RPC_URL 2>/dev/null || echo "  Call failed"

echo ""
echo "Estimating shares for deposit..."
# calculate_shares_for_deposit(assets, total_assets, total_supply)
cast estimate $VAULT \
  "calculateSharesForDeposit(uint256,uint256,uint256)(uint256)" \
  "1000$SCALE" "1000000$SCALE" "950000$SCALE" \
  --rpc-url $RPC_URL 2>/dev/null || echo "  Call failed"

echo ""
echo "=== Benchmark Complete ==="
