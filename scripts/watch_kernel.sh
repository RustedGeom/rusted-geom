#!/usr/bin/env bash
# ---
# script: watch_kernel.sh
# description: Watch Rust kernel sources and rebuild WASM on changes.
#              Run alongside `pnpm dev:fast` in a second terminal.
# usage: ./scripts/watch_kernel.sh
# requires: cargo-watch (cargo install cargo-watch)
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v cargo-watch &>/dev/null && ! cargo watch --version &>/dev/null 2>&1; then
  echo "cargo-watch not found. Install with: cargo install cargo-watch"
  exit 1
fi

echo "Watching kernel for changes..."
cd "$repo_root"

cargo watch \
  --watch crates/kernel/src \
  --watch crates/kernel/Cargo.toml \
  --shell "wasm-pack build --target web --release --out-dir $repo_root/crates/kernel/pkg $repo_root/crates/kernel && \
    $([ -x \"$(command -v wasm-opt)\" ] && echo \"wasm-opt -O3 --enable-simd $repo_root/crates/kernel/pkg/rusted_geom_bg.wasm -o $repo_root/crates/kernel/pkg/rusted_geom_bg.wasm &&\") \
    mkdir -p $repo_root/showcase/public/wasm && \
    cp $repo_root/crates/kernel/pkg/rusted_geom_bg.wasm $repo_root/showcase/public/wasm/rusted_geom.wasm && \
    echo 'WASM staged.'"
