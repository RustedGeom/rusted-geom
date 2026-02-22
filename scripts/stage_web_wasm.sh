#!/usr/bin/env bash
# ---
# script: stage_web_wasm.sh
# description: Build the wasm kernel artifact and stage it into bindings/web/dist for npm packaging.
# usage: ./scripts/stage_web_wasm.sh
# prerequisites:
#   - Rust toolchain with cargo available
#   - wasm32-unknown-unknown target installed
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

rustup target add wasm32-unknown-unknown >/dev/null
cargo build -p kernel-ffi --target wasm32-unknown-unknown --release

mkdir -p "$repo_root/bindings/web/dist/wasm"
cp "$repo_root/target/wasm32-unknown-unknown/release/rusted_geom.wasm" \
  "$repo_root/bindings/web/dist/wasm/rusted_geom.wasm"

echo "staged web wasm: $repo_root/bindings/web/dist/wasm/rusted_geom.wasm"
