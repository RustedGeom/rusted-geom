# rusted-geom

Metadata-driven geometry kernel binding pipeline in Rust.

## Workspace
- `crates/kernel-abi-meta`: ABI annotation proc macros.
- `crates/kernel-ffi`: session-scoped C ABI surface.
- `tools/abi-gen`: metadata extractor and binding generator.
- `bindings/dotnet`: generated .NET 8 interop layer.
- `bindings/web`: generated TypeScript facade + typed WASM runtime bridge.
- `showcase`: Next.js full-page Three.js + Tweakpane kernel viewer.
- `include/rusted_geom.h`: generated C header.

## Scripts

All scripts in `scripts/` include front matter (`script`, `description`, `usage`, `prerequisites`) and should stay synchronized with this table.

| Script | Purpose | Command |
| --- | --- | --- |
| `generate_bindings.sh` | Generate ABI artifacts for the workspace. | `./scripts/generate_bindings.sh` |
| `check_bindings.sh` | Verify generated artifacts are current (`--check`). | `./scripts/check_bindings.sh` |
| `check_abi_compat.sh` | Enforce ABI compatibility against baseline with semver-major checks. | `./scripts/check_abi_compat.sh` |
| `update_abi_baseline.sh` | Regenerate ABI and update `abi/baseline/rgm_abi.json`. | `./scripts/update_abi_baseline.sh` |
| `build_kernel_wasm.sh` | Build wasm kernel artifact and copy to showcase public assets. | `./scripts/build_kernel_wasm.sh` |
| `stage_dotnet_natives.sh` | Build and stage native libs under `bindings/dotnet/runtimes/<rid>/native`. | `./scripts/stage_dotnet_natives.sh [rid ...]` |
| `pack_dotnet.sh` | Build staged native libs and produce NuGet artifacts in `dist/nuget`. | `./scripts/pack_dotnet.sh [rid ...]` |
| `stage_web_wasm.sh` | Build wasm kernel artifact and stage it into `bindings/web/dist/wasm`. | `./scripts/stage_web_wasm.sh` |
| `pack_web.sh` | Build web bindings (JS + `.d.ts` + wasm) and pack tarball into `dist/npm`. | `./scripts/pack_web.sh` |

## Binding Pipeline

```mermaid
flowchart TD
  A["Rust ABI metadata (`crates/*`)"] --> B["`abi-gen` (`tools/abi-gen`)"]
  B --> C["Generated C header (`include/rusted_geom.h`)"]
  B --> D["Generated .NET bindings (`bindings/dotnet`)"]
  B --> E["Generated TS bindings (`bindings/web`)"]
  B --> F["ABI snapshot (`target/abi/rgm_abi.json`)"]
  F --> G["Baseline (`abi/baseline/rgm_abi.json`)"]
  G --> H["Compatibility gate (`check_abi_compat.sh`)"]
```

## ABI Compatibility Rule

`check_abi_compat.sh` validates current ABI against baseline with semver-major enforcement:

$$
\text{breaking\_changes}(ABI_{current}, ABI_{baseline}) = 0
$$

If breaking changes are intentional, regenerate and commit a new baseline with:

```bash
./scripts/update_abi_baseline.sh
```

## CAD Naming Migration (1.0.0 Hard Break)

Legacy curve evaluator names were replaced with CAD-kernel names across C/.NET/TS.

| Old | New |
| --- | --- |
| `rgm_curve_point_at_normalized_length` | `rgm_curve_point_at` |
| `rgm_curve_point_at_distance_length` | `rgm_curve_point_at_length` |
| `rgm_curve_derivative1_at_normalized_length` | `rgm_curve_d1_at` |
| `rgm_curve_derivative1_at_distance_length` | `rgm_curve_d1_at_length` |
| `rgm_curve_derivative2_at_normalized_length` | `rgm_curve_d2_at` |
| `rgm_curve_derivative2_at_distance_length` | `rgm_curve_d2_at_length` |
| `rgm_curve_plane_at_normalized_length` | `rgm_curve_plane_at` |
| `rgm_curve_plane_at_distance_length` | `rgm_curve_plane_at_length` |

New additions:
- `rgm_curve_tangent_at`, `rgm_curve_tangent_at_length`
- `rgm_curve_normal_at`, `rgm_curve_normal_at_length`
- `rgm_curve_d0_at`, `rgm_curve_d0_at_length`

.NET and TS generated APIs now expose `CurveHandle` instance methods:
- `PointAt`, `PointAtLength`, `TangentAt`, `TangentAtLength`, `NormalAt`, `NormalAtLength`, `PlaneAt`, `PlaneAtLength`
- `D0`, `D0AtLength`, `D1`, `D1AtLength`, `D2`, `D2AtLength`

## Test
```bash
cargo test --workspace
```

## Packaging

Build release packages with generated types and staged native assets:

```bash
./scripts/pack_dotnet.sh osx-arm64
./scripts/pack_web.sh
```

Outputs:
- NuGet: `dist/nuget/*.nupkg`
- npm tarball: `dist/npm/*.tgz`

## Showcase Quickstart
```bash
pnpm install
./scripts/build_kernel_wasm.sh
pnpm --dir showcase dev
```

Open [http://localhost:3000](http://localhost:3000) for the kernel-driven viewer.
