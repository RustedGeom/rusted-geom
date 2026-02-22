#!/usr/bin/env bash
# ---
# script: build_kernel_wasm.sh
# description: Build the kernel wasm artifact and copy it into showcase public assets.
# usage: ./scripts/build_kernel_wasm.sh
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

cargo build -p kernel-ffi --target wasm32-unknown-unknown --release

mkdir -p "$repo_root/showcase/public/wasm"
cp "$repo_root/target/wasm32-unknown-unknown/release/rusted_geom.wasm" \
  "$repo_root/showcase/public/wasm/rusted_geom.wasm"
