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

mkdir -p "$repo_root/showcase/public/wasm"
cp "$repo_root/crates/kernel/pkg/rusted_geom_bg.wasm" \
   "$repo_root/showcase/public/wasm/rusted_geom.wasm"
