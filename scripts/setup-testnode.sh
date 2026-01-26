#!/bin/bash
# Arbitrum Nitro Testnode Setup for Stylus Development
# Docs: https://docs.arbitrum.io/stylus/how-tos/local-stylus-dev-node

set -e

TESTNODE_DIR="$HOME/.arbitrum/nitro-testnode"

echo "=== Arbitrum Nitro Testnode Setup ==="
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is required. Install from https://docs.docker.com/get-docker/"
    exit 1
fi

if ! command -v docker compose &> /dev/null; then
    echo "Error: Docker Compose is required."
    exit 1
fi

echo "Docker found: $(docker --version)"
echo ""

# Clone or update nitro-testnode
if [ -d "$TESTNODE_DIR" ]; then
    echo "Updating existing nitro-testnode..."
    cd "$TESTNODE_DIR"
    git pull origin stylus
else
    echo "Cloning nitro-testnode (stylus branch)..."
    mkdir -p "$(dirname $TESTNODE_DIR)"
    git clone -b stylus --recurse-submodules https://github.com/OffchainLabs/nitro-testnode.git "$TESTNODE_DIR"
    cd "$TESTNODE_DIR"
fi

echo ""
echo "=== Starting Testnode ==="
echo ""

# Start the testnode
./test-node.bash --detach

echo ""
echo "=== Testnode Running ==="
echo ""
echo "RPC Endpoint: http://localhost:8547"
echo "Chain ID: 412346"
echo ""
echo "Pre-funded account:"
echo "  Address: 0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E"
echo "  Private Key: 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659"
echo ""
echo "To deploy Stylus contracts:"
echo "  cargo stylus deploy --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659 --endpoint http://localhost:8547"
echo ""
echo "To stop: cd $TESTNODE_DIR && docker compose down"
echo "To view logs: cd $TESTNODE_DIR && docker compose logs -f"
