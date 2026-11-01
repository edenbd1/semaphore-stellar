#!/usr/bin/env bash
# Invoke verify() on a deployed contract with the ground-truth fixture.
# Usage: scripts/invoke-verify.sh <CONTRACT_ID> [identity] [--send]
set -euo pipefail
CID="$1"; IDENTITY="${2:-semaphore-dev}"; SEND="${3:-}"
cd "$(dirname "$0")/.."
ARGS=$(node tools/emit-invoke-args.mjs)
get(){ echo "$ARGS" | node -e "process.stdin.on('data',d=>console.log(JSON.parse(d).$1))"; }
SENDFLAG=""; [ "$SEND" = "--send" ] && SENDFLAG="--send=yes"
stellar contract invoke --id "$CID" --source "$IDENTITY" --network testnet $SENDFLAG -- verify \
  --pub_signals "$(get pub_signals)" --a "$(get a)" --b "$(get b)" --c "$(get c)" \
  --vk_alpha "$(get vk_alpha)" --vk_beta "$(get vk_beta)" --vk_gamma "$(get vk_gamma)" \
  --vk_delta "$(get vk_delta)" --vk_ic "$(get vk_ic)"
