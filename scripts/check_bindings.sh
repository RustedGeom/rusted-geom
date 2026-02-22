#!/usr/bin/env bash
# ---
# script: check_bindings.sh
# description: Regenerate ABI artifacts in check mode to validate bindings are up to date.
# usage: ./scripts/check_bindings.sh
# prerequisites:
#   - Rust toolchain with cargo available
#   - abi-gen crate builds successfully
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

cargo run -p abi-gen -- --workspace "$repo_root" --check
