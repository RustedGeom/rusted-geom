#!/usr/bin/env bash
# ---
# script: stage_web_wasm.sh
# description: Build the kernel WASM via wasm-pack and stage artifacts for both
#              the bindings package and the showcase.
# usage: ./scripts/stage_web_wasm.sh
# prerequisites: wasm-pack installed (https://rustwasm.github.io/wasm-pack/)
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

wasm-pack build --target web --release \
  --out-dir "$repo_root/crates/kernel/pkg" \
  "$repo_root/crates/kernel"

# Stage the WASM binary for showcase public assets.
mkdir -p "$repo_root/showcase/public/wasm"
cp "$repo_root/crates/kernel/pkg/rusted_geom_bg.wasm" \
   "$repo_root/showcase/public/wasm/rusted_geom.wasm"

echo "Staged wasm to showcase/public/wasm/rusted_geom.wasm"
echo "Bindings pkg available at crates/kernel/pkg/"
