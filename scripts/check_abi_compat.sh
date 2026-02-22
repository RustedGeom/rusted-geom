#!/usr/bin/env bash
# ---
# script: check_abi_compat.sh
# description: Verify the current generated ABI against the baseline with semver-major enforcement.
# usage: ./scripts/check_abi_compat.sh
# prerequisites:
#   - Rust toolchain with cargo available
#   - abi-gen crate builds successfully
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

cargo run -p abi-gen -- \
  --workspace "$repo_root" \
  --check \
  --enforce-semver-major \
  --baseline "$repo_root/abi/baseline/rgm_abi.json"
