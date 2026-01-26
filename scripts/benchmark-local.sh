#!/bin/bash
# Local Benchmark Script - Deploy and measure gas on local testnode
# Requires: nitro-testnode running, foundry installed

set -e

export PATH="$HOME/.foundry/bin:$PATH"

RPC="http://localhost:8547"
PRIVATE_KEY="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659"

echo "=== Local Stylus Benchmark ==="
echo ""

# Check if node is running
if ! curl -s "$RPC" -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' > /dev/null 2>&1; then
    echo "Error: Local node not running at $RPC"
    echo "Start with: ./scripts/setup-testnode.sh"
    exit 1
fi

echo "Node running. Deploying contracts..."
echo ""

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
KEYSTONE_DIR="$(dirname "$SCRIPT_DIR")"

# Deploy stylus-lending
echo "--- Deploying stylus-lending ---"
cd "$KEYSTONE_DIR/examples/stylus-lending"
LENDING_OUTPUT=$(cargo stylus deploy --private-key $PRIVATE_KEY --endpoint $RPC 2>&1)
LENDING_ADDR=$(echo "$LENDING_OUTPUT" | grep "deployed code at address" | awk '{print $NF}')
echo "Deployed: $LENDING_ADDR"

# Deploy stylus-amm
echo ""
echo "--- Deploying stylus-amm ---"
cd "$KEYSTONE_DIR/examples/stylus-amm"
AMM_OUTPUT=$(cargo stylus deploy --private-key $PRIVATE_KEY --endpoint $RPC 2>&1)
AMM_ADDR=$(echo "$AMM_OUTPUT" | grep "deployed code at address" | awk '{print $NF}')
echo "Deployed: $AMM_ADDR"

# Deploy stylus-vault
echo ""
echo "--- Deploying stylus-vault ---"
cd "$KEYSTONE_DIR/examples/stylus-vault"
VAULT_OUTPUT=$(cargo stylus deploy --private-key $PRIVATE_KEY --endpoint $RPC 2>&1)
VAULT_ADDR=$(echo "$VAULT_OUTPUT" | grep "deployed code at address" | awk '{print $NF}')
echo "Deployed: $VAULT_ADDR"

# Deploy Solidity comparison
echo ""
echo "--- Deploying Solidity VaultCalc ---"
cd "$KEYSTONE_DIR/benchmarks/solidity"
SOLIDITY_OUTPUT=$(forge create VaultCalc --private-key $PRIVATE_KEY --rpc-url $RPC --broadcast 2>&1)
SOLIDITY_ADDR=$(echo "$SOLIDITY_OUTPUT" | grep "Deployed to:" | awk '{print $3}')
echo "Deployed: $SOLIDITY_ADDR"

echo ""
echo "=== Gas Benchmarks ==="
echo ""

echo "calculateSharePrice:"
echo -n "  Stylus:   "
cast estimate $VAULT_ADDR "calculateSharePrice(uint256,uint256)" 1000000000000000000000000 950000000000000000000000 --rpc-url $RPC
echo -n "  Solidity: "
cast estimate $SOLIDITY_ADDR "calculateSharePrice(uint256,uint256)" 1000000000000000000000000 950000000000000000000000 --rpc-url $RPC

echo ""
echo "calculateApyFromApr (365 loops):"
echo -n "  Stylus:   "
cast estimate $VAULT_ADDR "calculateApyFromApr(uint256,uint256)" 500000000000000000000 365 --rpc-url $RPC
echo -n "  Solidity: "
cast estimate $SOLIDITY_ADDR "calculateApyFromApr(uint256,uint256)" 500 365 --rpc-url $RPC

echo ""
echo "=== Benchmark Complete ==="
echo ""
echo "Contract Addresses:"
echo "  stylus-lending: $LENDING_ADDR"
echo "  stylus-amm:     $AMM_ADDR"
echo "  stylus-vault:   $VAULT_ADDR"
echo "  VaultCalc:      $SOLIDITY_ADDR"
