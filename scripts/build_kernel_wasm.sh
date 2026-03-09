#!/usr/bin/env bash
# ---
# script: build_kernel_wasm.sh
# description: Build the kernel WASM artifact via wasm-pack and stage it into
#              showcase public assets.
# usage: ./scripts/build_kernel_wasm.sh
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

wasm-pack build --target web --release \
  --out-dir "$repo_root/crates/kernel/pkg" \
  "$repo_root/crates/kernel"

if command -v wasm-opt &>/dev/null; then
  echo "Running wasm-opt -O3..."
  wasm-opt -O3 --enable-simd \
    "$repo_root/crates/kernel/pkg/rusted_geom_bg.wasm" \
    -o "$repo_root/crates/kernel/pkg/rusted_geom_bg.wasm"
fi

mkdir -p "$repo_root/showcase/public/wasm"
cp "$repo_root/crates/kernel/pkg/rusted_geom_bg.wasm" \
   "$repo_root/showcase/public/wasm/rusted_geom.wasm"
cp "$repo_root/crates/kernel/pkg/rusted_geom.js" \
   "$repo_root/showcase/public/wasm/rusted_geom.js"
cp "$repo_root/crates/kernel/pkg/rusted_geom.d.ts" \
   "$repo_root/showcase/public/wasm/rusted_geom.d.ts"
