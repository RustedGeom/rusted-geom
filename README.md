# rusted-geom

NURBS geometry kernel compiled to WASM, with `wasm-bindgen` TypeScript bindings.

## Project Status

`rusted-geom` is currently in alpha release stage (`0.1.0-alpha.1`).
APIs and package shape are expected to evolve quickly.

## Workspace Layout

- `crates/kernel-ffi`: NURBS geometry kernel + `wasm-bindgen` public API.
- `bindings/web`: Thin TypeScript wrapper around the wasm-pack output.
- `showcase`: Next.js full-page Three.js kernel viewer.

## Prerequisites

- Rust stable toolchain
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/) ≥ 0.13
- Node.js and `pnpm`

```bash
npm install -g pnpm
cargo install wasm-pack
```

## Quickstart (Local Run)

From a clean checkout, run these commands from repo root:

```bash
pnpm install
./scripts/build_kernel_wasm.sh    # compile Rust → WASM, stage to showcase/public/wasm/
./scripts/stage_web_wasm.sh       # copy WASM binary into bindings/web dist
cd bindings/web && npm run build  # generate dist/ for TypeScript module resolution
pnpm --dir showcase dev
```

Open [http://localhost:3000](http://localhost:3000).

## Common Workflows

| Workflow | Command |
| --- | --- |
| Build kernel WASM for showcase | `./scripts/build_kernel_wasm.sh` |
| Stage WASM into web bindings | `./scripts/stage_web_wasm.sh` |
| Build + pack `@rusted-geom/bindings-web` | `./scripts/pack_web.sh` |
| Run all Rust unit tests | `cargo test -p kernel-ffi` |
| TypeScript typecheck | `npm --prefix ./bindings/web run typecheck` |
| Web runtime tests | `npm --prefix ./bindings/web run test` |
| E2E tests | `npx --prefix showcase playwright test` |

## TypeScript Usage

### 1. Session + curve evaluation

```ts
import { loadKernel, KernelSession } from "@rusted-geom/bindings-web";
import wasmUrl from "@rusted-geom/bindings-web/wasm/rusted_geom_bg.wasm";

await loadKernel(wasmUrl);          // one-time WASM initialisation
const session = new KernelSession();

const curve = session.interpolate_nurbs_fit_points(
  new Float64Array([0, 0, 0, 1, 0.25, 0, 2, 1, 0, 3, 1.25, 0]),
  /*degree*/ 2,
  /*closed*/ false,
);
const [x, y, z] = session.curve_point_at(curve, 0.35);
const length = session.curve_length(curve);

console.log({ x, y, z }, length);

curve.free();
session.free();
```

### 2. Mesh boolean

```ts
await loadKernel(wasmUrl);
const session = new KernelSession();

const box   = session.create_mesh_box(0, 0, 0, 8, 8, 8);
const torus = session.create_mesh_torus(2, 0, 0, 2.5, 0.8, 64, 48);

// Boolean op: 0 = union, 1 = intersection, 2 = difference
const result = session.mesh_boolean(box, torus, 2);
console.log("triangles:", session.mesh_triangle_count(result));

result.free();
torus.free();
box.free();
session.free();
```

### 3. Surface + bounding box

```ts
await loadKernel(wasmUrl);
const session = new KernelSession();

const surface = session.create_nurbs_surface(
  1, 1,                              // degree_u, degree_v
  false, false,                      // periodic_u, periodic_v
  2, 2,                              // control_u_count, control_v_count
  new Float64Array([0,0,0, 0,1,0, 1,0,0.1, 1,1,0.1]), // control points [x,y,z,…]
  new Float64Array([1, 1, 1, 1]),    // weights
  new Float64Array([0, 0, 1, 1]),    // knots_u
  new Float64Array([0, 0, 1, 1]),    // knots_v
);

// mode 0 = Fast (control-point hull), 1 = Optimal (PCA + OBB)
const b = session.compute_bounds(surface.object_id(), 1, 0, 0.0);
console.log("AABB min:", b.aabb_min_x, b.aabb_min_y, b.aabb_min_z);
console.log("AABB max:", b.aabb_max_x, b.aabb_max_y, b.aabb_max_z);

b.free();
surface.free();
session.free();
```

### 4. Surface–surface intersection

```ts
await loadKernel(wasmUrl);
const session = new KernelSession();

// … create surfaceA, surfaceB …

const result = session.intersect_surface_surface(surfaceA, surfaceB);
const branchCount = session.intersection_branch_count(result);

for (let i = 0; i < branchCount; i++) {
  const summary = session.intersection_branch_summary(result, i);
  const pts     = session.intersection_branch_points(result, i);
  console.log(`branch ${i}: ${summary.point_count} points, closed=${summary.closed}`);
}

result.free();
```

## High-Level Architecture

```
crates/kernel-ffi (Rust)
  ├── kernel_impl/*.rs    — NURBS math + kernel operations (include! flat module)
  ├── math/*.rs           — basis functions, surface evaluation
  ├── elements/brep/      — B-rep data structures
  ├── session/            — session store + object registry
  └── wasm/               — #[wasm_bindgen] public API
         ↓  wasm-pack build
crates/kernel-ffi/pkg/
  ├── rusted_geom_bg.wasm
  ├── rusted_geom.js      — ESM glue (auto-generated)
  └── rusted_geom.d.ts    — TypeScript declarations (auto-generated)
         ↓
bindings/web/             — thin re-export + loader
         ↓
showcase/                 — Next.js + Three.js viewer + developer test lab
```

## Algorithm Documents

- [NURBS Fit-Point Constructor RFC (M1)](docs/algorithms/nurbs-fit-point-interpolation-rfc.md)

## Additional Documentation

- [Architecture Overview](ARCHITECTURE.md)
- [Kernel Module Map](docs/architecture/kernel-module-map.md)
