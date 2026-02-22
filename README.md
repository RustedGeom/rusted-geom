# rusted-geom

Metadata-driven geometry kernel binding pipeline in Rust.

## Workspace
- `crates/kernel-abi-meta`: ABI annotation proc macros.
- `crates/kernel-ffi`: session-scoped C ABI surface.
- `tools/abi-gen`: metadata extractor and binding generator.
- `bindings/web`: generated TypeScript facade + typed WASM runtime bridge.
- `showcase`: Next.js full-page Three.js + Tweakpane kernel viewer.

## Scripts

All scripts in `scripts/` include front matter (`script`, `description`, `usage`, `prerequisites`) and should stay synchronized with this table.

| Script | Purpose | Command |
| --- | --- | --- |
| `generate_bindings.sh` | Generate ABI artifacts for the workspace. | `./scripts/generate_bindings.sh` |
| `check_bindings.sh` | Verify generated artifacts are current (`--check`). | `./scripts/check_bindings.sh` |
| `check_abi_compat.sh` | Enforce ABI compatibility against baseline with semver-major checks. | `./scripts/check_abi_compat.sh` |
| `update_abi_baseline.sh` | Regenerate ABI and update `abi/baseline/rgm_abi.json`. | `./scripts/update_abi_baseline.sh` |
| `build_kernel_wasm.sh` | Build wasm kernel artifact and copy to showcase public assets. | `./scripts/build_kernel_wasm.sh` |
| `stage_web_wasm.sh` | Build wasm kernel artifact and stage it into `bindings/web/dist/wasm`. | `./scripts/stage_web_wasm.sh` |
| `pack_web.sh` | Build web bindings (JS + `.d.ts` + wasm) and pack tarball into `dist/npm`. | `./scripts/pack_web.sh` |

## Binding Pipeline

```mermaid
flowchart TD
  A["Rust ABI metadata (`crates/*`)"] --> B["`abi-gen` (`tools/abi-gen`)"]
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

Legacy curve evaluator names were replaced with CAD-kernel names across C/TS.

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

TS generated APIs expose `CurveHandle` instance methods:
- `PointAt`, `PointAtLength`, `TangentAt`, `TangentAtLength`, `NormalAt`, `NormalAtLength`, `PlaneAt`, `PlaneAtLength`
- `D0`, `D0AtLength`, `D1`, `D1AtLength`, `D2`, `D2AtLength`

## Kernel Usage Examples

### WASM + TypeScript (`@rusted-geom/bindings-web`)

Build and pack the web bindings (includes JS, types, and `rusted_geom.wasm`):

```bash
./scripts/pack_web.sh
```

Then install the tarball in your app:

```bash
npm install /absolute/path/to/rusted-geom/dist/npm/*.tgz
```

End-to-end kernel session example:

```ts
import {
  createKernelRuntime,
  KernelRuntimeError,
  statusToName,
  type CurvePresetInput,
  type KernelSession,
} from "@rusted-geom/bindings-web";
import wasmUrl from "@rusted-geom/bindings-web/wasm/rusted_geom.wasm";

const preset: CurvePresetInput = {
  name: "demo-spline",
  degree: 3,
  closed: false,
  points: [
    { x: 0, y: 0, z: 0 },
    { x: 1, y: 0.25, z: 0 },
    { x: 2, y: 1.0, z: 0 },
    { x: 3, y: 1.25, z: 0 },
    { x: 4, y: 1.0, z: 0 },
  ],
  tolerance: {
    abs_tol: 1e-9,
    rel_tol: 1e-9,
    angle_tol: 1e-9,
  },
};

async function runKernelDemo(): Promise<void> {
  const runtime = await createKernelRuntime(wasmUrl);
  let session: KernelSession | null = null;
  let curveHandle: bigint | null = null;

  try {
    session = runtime.createSession();
    curveHandle = session.buildCurveFromPreset(preset);

    const point = session.pointAt(curveHandle, 0.35);
    const totalLength = session.curveLength(curveHandle);
    const lengthAt35 = session.curveLengthAt(curveHandle, 0.35);
    const sampled = session.sampleCurvePolyline(curveHandle, 64);

    console.log("Kernel capabilities:", runtime.capabilities);
    console.log("Point @ t=0.35:", point);
    console.log("Total length:", totalLength);
    console.log("Length @ t=0.35:", lengthAt35);
    console.log("Polyline sample count:", sampled.length);
  } catch (error) {
    if (error instanceof KernelRuntimeError) {
      console.error(
        `KernelRuntimeError (${statusToName(error.status)}):`,
        error.details ?? error.message,
      );
    } else {
      console.error("Unexpected error:", error);
    }

    if (session) {
      console.error("Last kernel error:", session.lastError());
    }
  } finally {
    if (session && curveHandle !== null) {
      session.releaseObject(curveHandle);
    }
    if (session) {
      session.destroy();
    }
    runtime.destroy();
  }
}

void runKernelDemo();
```

If you prefer loading wasm bytes directly (e.g., Node.js/SSR), pass `Uint8Array` or `ArrayBuffer` instead of a URL:

```ts
import { readFile } from "node:fs/promises";
import { createKernelRuntime } from "@rusted-geom/bindings-web";

const wasmBytes = await readFile("/absolute/path/to/rusted_geom.wasm");
const runtime = await createKernelRuntime(wasmBytes);
```

## Test
```bash
cargo test --workspace
```

## Packaging

Build release packages with generated types and staged native assets:

```bash
./scripts/pack_web.sh
```

Outputs:
- npm tarball: `dist/npm/*.tgz`

## Showcase Quickstart
```bash
pnpm install
./scripts/build_kernel_wasm.sh
pnpm --dir showcase dev
```

Open [http://localhost:3000](http://localhost:3000) for the kernel-driven viewer.
