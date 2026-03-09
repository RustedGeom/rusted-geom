#!/usr/bin/env bash
# ---
# script: check_wasm_size.sh
# description: Fail if the WASM binary exceeds the size budget.
# usage: ./scripts/check_wasm_size.sh
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
WASM_PATH="$repo_root/showcase/public/wasm/rusted_geom.wasm"
LIMIT=1500000  # 1.5 MB budget

if [ ! -f "$WASM_PATH" ]; then
  echo "WASM file not found: $WASM_PATH"
  exit 1
fi

SIZE=$(wc -c < "$WASM_PATH")

echo "WASM size: ${SIZE} bytes (budget: ${LIMIT} bytes)"

if [ "$SIZE" -gt "$LIMIT" ]; then
  echo "ERROR: WASM binary exceeds size budget (${SIZE} > ${LIMIT})"
  exit 1
fi

echo "OK: WASM binary within budget."
