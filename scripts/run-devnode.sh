#!/bin/bash
# Lightweight Nitro Dev Node for Quick Stylus Testing
# Docs: https://docs.arbitrum.io/run-arbitrum-node/run-nitro-dev-node

set -e

DEVNODE_DIR="$HOME/.arbitrum/nitro-devnode"

echo "=== Nitro Dev Node (Lightweight) ==="
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is required."
    exit 1
fi

# Clone or update
if [ -d "$DEVNODE_DIR" ]; then
    echo "Updating nitro-devnode..."
    cd "$DEVNODE_DIR"
    git pull origin main
else
    echo "Cloning nitro-devnode..."
    mkdir -p "$(dirname $DEVNODE_DIR)"
    git clone https://github.com/OffchainLabs/nitro-devnode.git "$DEVNODE_DIR"
    cd "$DEVNODE_DIR"
fi

echo ""
echo "Starting dev node..."

# Run the dev node
./run-dev-node.sh

echo ""
echo "Dev node running at http://localhost:8547"
