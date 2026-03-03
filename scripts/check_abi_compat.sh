#!/usr/bin/env bash
# Lightweight ABI compatibility sanity checks for release workflow.
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

if [[ ! -f "docs/architecture/abi-stability.md" ]]; then
  echo "ERROR: missing docs/architecture/abi-stability.md"
  exit 1
fi

if [[ ! -f "docs/reference/kernel-c-abi.md" ]]; then
  echo "ERROR: missing docs/reference/kernel-c-abi.md"
  exit 1
fi

if ! grep -R -n 'pub extern "C" fn rgm_' crates/kernel/src/kernel_impl >/dev/null 2>&1; then
  echo "ERROR: no exported rgm_ C ABI symbols found in crates/kernel/src/kernel_impl"
  exit 1
fi

non_archive_legacy_refs="$(
  grep -R -n -E "kernel[-]ffi" . \
    --exclude-dir=archive \
    --exclude-dir=target \
    --exclude-dir=.git \
    --exclude-dir=node_modules \
    --exclude-dir=.next \
    --exclude-dir=dist \
    --exclude=*.tsbuildinfo \
    --exclude=check_bindings.sh \
    --exclude=check_abi_compat.sh || true
)"
if [[ -n "$non_archive_legacy_refs" ]]; then
  echo "ERROR: non-archive legacy kernel naming references still exist:"
  echo "$non_archive_legacy_refs"
  exit 1
fi

echo "ABI compatibility checks passed."
