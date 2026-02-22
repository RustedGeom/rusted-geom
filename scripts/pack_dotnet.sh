#!/usr/bin/env bash
# ---
# script: pack_dotnet.sh
# description: Stage native libraries and produce NuGet package artifacts.
# usage: ./scripts/pack_dotnet.sh [rid ...]
# prerequisites:
#   - dotnet SDK 8+
#   - Rust toolchain with cargo available
# ---
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

if [ "$#" -eq 0 ]; then
  set -- osx-arm64
fi

./scripts/stage_dotnet_natives.sh "$@"

mkdir -p "$repo_root/dist/nuget"

dotnet pack "$repo_root/bindings/dotnet/RustedGeom.Bindings.csproj" \
  -c Release \
  -o "$repo_root/dist/nuget" \
  /p:ContinuousIntegrationBuild=true
