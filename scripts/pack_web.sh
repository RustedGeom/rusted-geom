#!/usr/bin/env bash
# ---
# script: pack_web.sh
# description: Build the web bindings package (JS + d.ts + wasm) and produce an npm tarball.
# usage: ./scripts/pack_web.sh
# prerequisites:
#   - Node.js + npm
#   - Rust toolchain with cargo available
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

npm --prefix "$repo_root/bindings/web" run build
npm --prefix "$repo_root/bindings/web" run pack:local
