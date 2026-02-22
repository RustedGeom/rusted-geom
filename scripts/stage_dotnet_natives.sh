#!/usr/bin/env bash
# ---
# script: stage_dotnet_natives.sh
# description: Build native kernel libraries and stage them under bindings/dotnet/runtimes.
# usage: ./scripts/stage_dotnet_natives.sh [rid ...]
# prerequisites:
#   - Rust toolchain with cargo available
#   - target toolchains installed for requested RIDs
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

if [ "$#" -eq 0 ]; then
  set -- osx-arm64
fi

build_and_stage() {
  local rid="$1"
  local target=""
  local src=""
  local dst=""

  case "$rid" in
    osx-arm64)
      target="aarch64-apple-darwin"
      src="$repo_root/target/$target/release/libkernel_ffi.dylib"
      dst="$repo_root/bindings/dotnet/runtimes/$rid/native/librusted_geom.dylib"
      ;;
    win-x64)
      target="x86_64-pc-windows-msvc"
      src="$repo_root/target/$target/release/kernel_ffi.dll"
      dst="$repo_root/bindings/dotnet/runtimes/$rid/native/rusted_geom.dll"
      ;;
    linux-x64)
      target="x86_64-unknown-linux-gnu"
      src="$repo_root/target/$target/release/libkernel_ffi.so"
      dst="$repo_root/bindings/dotnet/runtimes/$rid/native/librusted_geom.so"
      ;;
    *)
      echo "Unsupported RID: $rid" >&2
      exit 1
      ;;
  esac

  rustup target add "$target" >/dev/null
  cargo build -p kernel-ffi --target "$target" --release

  mkdir -p "$(dirname "$dst")"
  cp "$src" "$dst"
  echo "staged $rid native: $dst"
}

for rid in "$@"; do
  build_and_stage "$rid"
done
