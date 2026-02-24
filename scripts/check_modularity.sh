#!/usr/bin/env bash
# ---
# script: check_modularity.sh
# description: Enforce facade-size thresholds for key entry files to prevent monolith regression.
# usage: ./scripts/check_modularity.sh
# prerequisites:
#   - bash
#   - wc
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

check_max_lines() {
  local file="$1"
  local max_lines="$2"
  local actual
  actual="$(wc -l < "$file" | tr -d ' ')"
  if [[ "$actual" -gt "$max_lines" ]]; then
    echo "ERROR: $file has $actual lines (max $max_lines)"
    exit 1
  fi
}

# Thin facade guardrails.
check_max_lines "crates/kernel-ffi/src/lib.rs" 200
check_max_lines "crates/kernel-ffi/src/kernel_impl.rs" 200
check_max_lines "bindings/web/src/index.ts" 20
check_max_lines "bindings/web/src/loader.ts" 30
check_max_lines "bindings/web/src/types.ts" 150

# Keep handwritten kernel internals below the maintainability threshold.
while IFS= read -r file; do
  check_max_lines "$file" 1200
done < <(find "crates/kernel-ffi/src" -type f -name "*.rs" ! -path "*/tests/*")

echo "Modularity checks passed."
