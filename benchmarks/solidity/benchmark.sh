#!/bin/bash
# Deploy and benchmark Solidity VaultCalc for comparison with Stylus
# Usage: ./benchmark.sh <private-key>

set -e

export PATH="$HOME/.foundry/bin:$PATH"

if [ -z "$1" ]; then
    echo "Usage: ./benchmark.sh <private-key>"
    echo "Example: ./benchmark.sh 0x..."
    exit 1
fi

PRIVATE_KEY=$1
RPC="https://arb1.arbitrum.io/rpc"

cd "$(dirname "$0")"

echo "=== Deploying Solidity VaultCalc ==="
DEPLOY_OUTPUT=$(forge create VaultCalc --private-key $PRIVATE_KEY --rpc-url $RPC --json 2>&1)
SOLIDITY_ADDR=$(echo $DEPLOY_OUTPUT | jq -r '.deployedTo')

if [ "$SOLIDITY_ADDR" == "null" ] || [ -z "$SOLIDITY_ADDR" ]; then
    echo "Deployment failed:"
    echo $DEPLOY_OUTPUT
    exit 1
fi

echo "Deployed to: $SOLIDITY_ADDR"
echo ""

# Stylus contract for comparison
STYLUS_VAULT="0xdaf8f1a5f8025210f07665d4ccf2d2c0622a41fa"

echo "=== Gas Comparison: Stylus vs Solidity ==="
echo ""

echo "calculateSharePrice(1000000e18, 950000e18):"
echo -n "  Stylus:   "
cast estimate $STYLUS_VAULT "calculateSharePrice(uint256,uint256)" 1000000000000000000000000 950000000000000000000000 --rpc-url $RPC
echo -n "  Solidity: "
cast estimate $SOLIDITY_ADDR "calculateSharePrice(uint256,uint256)" 1000000000000000000000000 950000000000000000000000 --rpc-url $RPC
echo ""

echo "calculateSharesForDeposit(1000e18, 1000000e18, 950000e18):"
echo -n "  Stylus:   "
cast estimate $STYLUS_VAULT "calculateSharesForDeposit(uint256,uint256,uint256)" 1000000000000000000000 1000000000000000000000000 950000000000000000000000 --rpc-url $RPC
echo -n "  Solidity: "
cast estimate $SOLIDITY_ADDR "calculateSharesForDeposit(uint256,uint256,uint256)" 1000000000000000000000 1000000000000000000000000 950000000000000000000000 --rpc-url $RPC
echo ""

echo "calculateCompoundYield(1000e18, 500bps, 30 periods):"
echo -n "  Stylus:   "
cast estimate $STYLUS_VAULT "calculateCompoundYield(uint256,uint256,uint256)" 1000000000000000000000 500000000000000000000 30 --rpc-url $RPC
echo -n "  Solidity: "
cast estimate $SOLIDITY_ADDR "calculateCompoundYield(uint256,uint256,uint256)" 1000000000000000000000 500000000000000000000 30 --rpc-url $RPC
echo ""

echo "calculateApyFromApr(500bps, 365 compounds):"
echo -n "  Stylus:   "
cast estimate $STYLUS_VAULT "calculateApyFromApr(uint256,uint256)" 500000000000000000000 365 --rpc-url $RPC
echo -n "  Solidity: "
cast estimate $SOLIDITY_ADDR "calculateApyFromApr(uint256,uint256)" 500000000000000000000 365 --rpc-url $RPC
echo ""

echo "=== Benchmark Complete ==="
echo "Solidity contract deployed at: $SOLIDITY_ADDR"
