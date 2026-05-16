#!/usr/bin/env bash
# deploy.sh — deploy StarInk contract to Stellar testnet or mainnet
# Usage: ./scripts/deploy.sh [testnet|mainnet]
#
# TODO: implement full deploy flow
#   1. cargo build --target wasm32-unknown-unknown --release
#   2. stellar contract optimize
#   3. stellar contract deploy
#   4. stellar contract invoke initialize

set -euo pipefail

NETWORK="${1:-testnet}"
echo "Deploy target: $NETWORK"
echo "Not yet implemented — see docs/deployment.md"
