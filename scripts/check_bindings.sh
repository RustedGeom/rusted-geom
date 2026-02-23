#!/usr/bin/env bash
# ---
# script: check_bindings.sh
# description: Regenerate ABI artifacts and verify committed bindings are up to date.
# usage: ./scripts/check_bindings.sh
# prerequisites:
#   - Rust toolchain with cargo available
#   - abi-gen crate builds successfully
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

# Generate all outputs (writes target/abi/rgm_abi.json + bindings/web/src/generated/*)
cargo run -p abi-gen -- --workspace "$repo_root"

# Verify that committed generated files were not stale
if ! git diff --exit-code bindings/web/src/generated/ > /dev/null 2>&1; then
  echo "Error: committed bindings are stale. Run: cargo run -p abi-gen -- --workspace ." >&2
  git diff --stat bindings/web/src/generated/ >&2
  exit 1
fi
