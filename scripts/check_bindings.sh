#!/usr/bin/env bash
# Verify binding-layer references are aligned with the kernel crate layout.
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

required_files=(
  "bindings/web/src/index.ts"
  "bindings/web/src/loader.ts"
  "bindings/web/tests/runtime.test.ts"
)

for f in "${required_files[@]}"; do
  if [[ ! -f "$f" ]]; then
    echo "ERROR: missing expected binding file: $f"
    exit 1
  fi
done

if grep -R -n -E "kernel[-]ffi" bindings/web >/dev/null 2>&1; then
  echo "ERROR: legacy kernel naming reference found in bindings/web"
  grep -R -n -E "kernel[-]ffi" bindings/web
  exit 1
fi

if ! grep -n "crates/kernel/pkg/rusted_geom\\.js" bindings/web/src/index.ts >/dev/null 2>&1; then
  echo "ERROR: bindings/web/src/index.ts does not point to crates/kernel/pkg/rusted_geom.js"
  exit 1
fi

if ! grep -n "crates/kernel/pkg/rusted_geom\\.js" bindings/web/src/loader.ts >/dev/null 2>&1; then
  echo "ERROR: bindings/web/src/loader.ts does not point to crates/kernel/pkg/rusted_geom.js"
  exit 1
fi

echo "Bindings checks passed."
