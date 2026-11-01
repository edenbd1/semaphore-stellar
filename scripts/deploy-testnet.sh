#!/usr/bin/env bash
# Build and deploy the Semaphore verifier to Stellar testnet.
# Usage: scripts/deploy-testnet.sh [identity]
set -euo pipefail
IDENTITY="${1:-semaphore-dev}"
cd "$(dirname "$0")/.."

# One-time setup (safe to re-run):
stellar keys generate "$IDENTITY" --network testnet 2>/dev/null || true
stellar keys fund "$IDENTITY" --network testnet 2>/dev/null || true

stellar contract build
WASM=target/wasm32v1-none/release/semaphore_verifier.wasm
stellar contract deploy --wasm "$WASM" --source "$IDENTITY" --network testnet
