#!/usr/bin/env bash
# End-to-end Semaphore lib flow on testnet: deploy -> register vkey -> create group ->
# set root -> validate a real proof -> (replay fails). Usage: scripts/semaphore-flow.sh [identity]
set -euo pipefail
IDENTITY="${1:-semaphore-dev}"
cd "$(dirname "$0")/.."
stellar keys generate "$IDENTITY" --network testnet 2>/dev/null || true
stellar keys fund "$IDENTITY" --network testnet 2>/dev/null || true
ADMIN=$(stellar keys address "$IDENTITY")

stellar contract build
WASM=target/wasm32v1-none/release/semaphore.wasm
SID=$(stellar contract deploy --wasm "$WASM" --source "$IDENTITY" --network testnet -- --admin "$ADMIN" | grep -oE 'C[A-Z0-9]{55}' | tail -1)
echo "SEMAPHORE_ID=$SID"

A=$(node tools/emit-semaphore-args.mjs)
VK=$(echo "$A" | node -e 'let d="";process.stdin.on("data",c=>d+=c).on("end",()=>console.log(JSON.stringify(JSON.parse(d).vk)))')
val(){ echo "$A" | node -e "let d='';process.stdin.on('data',c=>d+=c).on('end',()=>console.log(JSON.parse(d).$1))"; }

inv(){ stellar contract invoke --id "$SID" --source "$IDENTITY" --network testnet --send=yes -- "$@"; }
inv register_vkey --depth 2 --vk "$VK"
GID=$(inv create_group --admin "$ADMIN" | tail -1)
inv update_group_root --group_id "$GID" --new_root "$(val root)" --depth 2
inv validate_proof --group_id "$GID" --merkle_root "$(val root)" --nullifier "$(val nullifier)" \
  --message "$(val message)" --scope "$(val scope)" --a "$(val a)" --b "$(val b)" --c "$(val c)"
echo ">>> replaying the same nullifier (expected to fail with Error #7):"
inv validate_proof --group_id "$GID" --merkle_root "$(val root)" --nullifier "$(val nullifier)" \
  --message "$(val message)" --scope "$(val scope)" --a "$(val a)" --b "$(val b)" --c "$(val c)" || echo "rejected as expected"
