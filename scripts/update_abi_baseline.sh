#!/usr/bin/env bash
# ---
# script: update_abi_baseline.sh
# description: Generate ABI artifacts and update the committed ABI baseline file.
# usage: ./scripts/update_abi_baseline.sh
# prerequisites:
#   - Rust toolchain with cargo available
#   - abi-gen crate builds successfully
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

cargo run -p abi-gen -- --workspace "$repo_root"
mkdir -p "$repo_root/abi/baseline"
cp "$repo_root/target/abi/rgm_abi.json" "$repo_root/abi/baseline/rgm_abi.json"
