#!/usr/bin/env bash
# ---
# script: generate_bindings.sh
# description: Generate ABI artifacts for the workspace.
# usage: ./scripts/generate_bindings.sh
# prerequisites:
#   - Rust toolchain with cargo available
#   - abi-gen crate builds successfully
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

cargo run -p abi-gen -- --workspace "$repo_root"
