<p align="center">
  <strong>rusted-geom</strong>
</p>

<p align="center">
  A high-performance NURBS geometry kernel written in Rust, compiled to WebAssembly.
</p>

<p align="center">
  <a href="https://github.com/RustedGeom/rusted-geom/actions/workflows/ci.yml">
    <img src="https://github.com/RustedGeom/rusted-geom/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
  <a href="https://github.com/RustedGeom/rusted-geom/releases">
    <img src="https://img.shields.io/github/v/release/RustedGeom/rusted-geom?include_prereleases&label=release" alt="Release">
  </a>
  <a href="https://rusted-geom-kernel-showcase.vercel.app">
    <img src="https://img.shields.io/badge/showcase-live-blue" alt="Showcase">
  </a>
  <a href="https://github.com/RustedGeom/rusted-geom/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/RustedGeom/rusted-geom" alt="License: MIT">
  </a>
</p>

---

## Overview

**rusted-geom** is a NURBS (Non-Uniform Rational B-Spline) geometry kernel built in Rust and compiled to WebAssembly via `wasm-pack`. It exposes a fully typed TypeScript API through `wasm-bindgen`, enabling browser and Node.js applications to perform computational geometry operations at near-native speed.

**[Live Showcase](https://rusted-geom-kernel-showcase.vercel.app)** — interactive 3D viewer with NURBS curves, surfaces, sweep/loft, CSG booleans, and more.

### Key Capabilities

- **NURBS Curves & Surfaces** — construction, evaluation, tessellation, fit-point interpolation
- **Sweep & Loft** — sweep a profile along a path or loft through cross-sections to generate surfaces
- **CSG Booleans** — union, intersection, and difference via direct mesh CSG (boolmesh)
- **Volume Computation** — closed-mesh volume via divergence theorem (signed tetrahedron sum)
- **Mesh Operations** — box/torus/sphere primitives, transforms, mesh export
- **Intersections** — surface–surface and curve–surface intersection with branch tracking
- **Bounding Volumes** — axis-aligned (AABB) and oriented (OBB) bounding boxes
- **LandXML** — parsing, sampling, and surface reconstruction from survey data
- **CAD Export** — IGES 5.3 and ACIS SAT file generation for curves and surfaces
- **Mesh Export** — STL and glTF 2.0 export for triangulated meshes
- **Benchmarked** — Criterion-based benchmarks with regression detection

### Project Status

> **v0.3.0** — Removed B-Rep/Face layer; NURBS + Mesh only. Sweep and loft produce surfaces directly. APIs may still evolve. Contributions and feedback welcome.

---

## Workspace Layout

```
rusted-geom/
├── crates/kernel/        # Rust geometry kernel + wasm-bindgen API
├── bindings/web/         # TypeScript wrapper (@rustedgeom/kernel)
├── showcase/             # Next.js + Three.js interactive viewer
├── scripts/              # Build, staging, and CI helper scripts
└── docs/                 # Architecture, algorithms, and API reference
```

| Package | Description |
|---------|-------------|
| `crates/kernel` | Core NURBS math, session management, WASM bindings |
| `bindings/web` | Thin ESM re-export + WASM loader for `@rustedgeom/kernel` |
| `showcase` | Next.js 16 + Three.js full-page viewer and developer test lab |

---

## Prerequisites

| Tool | Version |
|------|---------|
| Rust | stable toolchain |
| [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) | ≥ 0.13 |
| Node.js | ≥ 18 |
| pnpm | ≥ 9 |

```bash
cargo install wasm-pack
npm install -g pnpm
```

---

## Install the npm package

The `@rustedgeom/kernel` package is published to [GitHub Packages](https://github.com/RustedGeom/rusted-geom/packages).

**1. Add `.npmrc` to your project root:**

```
@rustedgeom:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}
```

> `GITHUB_TOKEN` must be a GitHub Personal Access Token with `read:packages` scope.

**2. Install:**

```bash
npm install @rustedgeom/kernel
```

---

## Quickstart (from source)

```bash
# 1. Install JS dependencies
pnpm install

# 2. Build Rust kernel to WASM and stage into showcase
./scripts/build_kernel_wasm.sh
./scripts/stage_web_wasm.sh

# 3. Build the TypeScript bindings
cd bindings/web && npm run build && cd ../..

# 4. Launch the showcase viewer
pnpm --dir showcase dev
```

Open [http://localhost:3000](http://localhost:3000) to explore the interactive viewer.

---

## Usage (TypeScript)

### Curve evaluation

```ts
import { loadKernel, KernelSession } from "@rustedgeom/kernel";
import wasmUrl from "@rustedgeom/kernel/wasm/rusted_geom_bg.wasm";

await loadKernel(wasmUrl);
const session = new KernelSession();

const curve = session.interpolate_nurbs_fit_points(
  new Float64Array([0, 0, 0, 1, 0.25, 0, 2, 1, 0, 3, 1.25, 0]),
  2,     // degree
  false, // closed
);

const [x, y, z] = session.curve_point_at(curve, 0.35);
const length = session.curve_length(curve);

curve.free();
session.free();
```

### Mesh boolean

```ts
const box   = session.create_box_mesh(0, 0, 0, 8, 8, 8);
const torus = session.create_torus_mesh(2, 0, 0, 2.5, 0.8, 64, 48);
const result = session.mesh_boolean(box, torus, 2); // 0=union, 1=intersect, 2=difference

console.log("triangles:", session.mesh_triangle_count(result));
```

### Sweep & Loft (surface generation)

```ts
// Sweep a closed profile along a 3D path → surface → mesh
const path = session.interpolate_nurbs_fit_points(
  new Float64Array([0,0,0, 5,0,2, 10,0,0]), 2, false
);
const profile = session.create_polyline(
  new Float64Array([0,-1,0, 0,1,0, 0,1,1, 0,-1,1]), true
);
const surface = session.sweep(path, profile, 16);
const mesh = session.surface_tessellate_to_mesh(surface, new Float64Array([]));
console.log("triangles:", session.mesh_triangle_count(mesh));

// Loft through cross-section curves
const s0 = session.create_polyline(new Float64Array([0,-1,0, 0,1,0, 0,1,1, 0,-1,1]), true);
const s1 = session.create_polyline(new Float64Array([10,-2,0, 10,2,0, 10,2,1.5, 10,-2,1.5]), true);
const loftSurf = session.loft(new Float64Array([s0.object_id(), s1.object_id()]), 12);
const loftMesh = session.surface_tessellate_to_mesh(loftSurf, new Float64Array([]));
```

### CSG boolean (union / intersection / difference)

```ts
const boxA = session.create_box_mesh(0, 0, 0, 4, 4, 4);
const sphere = session.create_uv_sphere_mesh(1.5, 1.5, 0, 2.0, 32, 24);

// 0 = union, 1 = intersection, 2 = difference
const union = session.mesh_boolean(boxA, sphere, 0);
const diff  = session.mesh_boolean(boxA, sphere, 2);

console.log("union triangles:", session.mesh_triangle_count(union));
console.log("volume:", session.mesh_volume(diff));
```

### Volume computation

```ts
const box = session.create_box_mesh(0, 0, 0, 3, 5, 7);
const volume = session.mesh_volume(box);
console.log("volume:", volume); // ~105.0
```

### Surface–surface intersection

```ts
const result = session.intersect_surface_surface(surfaceA, surfaceB);
const branchCount = session.intersection_branch_count(result);

for (let i = 0; i < branchCount; i++) {
  const summary = session.intersection_branch_summary(result, i);
  console.log(`branch ${i}: ${summary.point_count} pts, closed=${summary.closed}`);
}
```

> See the [Kernel WASM API Reference](docs/reference/kernel-wasm-api.md) for the full API surface.

---

## Common Workflows

| Task | Command |
|------|---------|
| Build kernel WASM for showcase | `./scripts/build_kernel_wasm.sh` |
| Stage WASM into web bindings | `./scripts/stage_web_wasm.sh` |
| Build + pack `@rustedgeom/kernel` | `./scripts/pack_web.sh` |
| Launch showcase (full rebuild) | `pnpm --dir showcase dev` |
| Launch showcase (UI-only, skip WASM) | `pnpm --dir showcase dev:fast` |
| WASM hot-reload during Rust iteration | `./scripts/watch_kernel.sh` |
| Check WASM binary size against budget | `./scripts/check_wasm_size.sh` |
| Run Rust unit tests | `cargo test -p kernel` |
| TypeScript type check | `npm --prefix ./bindings/web run typecheck` |
| Web runtime tests | `npm --prefix ./bindings/web run test` |
| Showcase unit tests | `pnpm --dir showcase test:unit` |
| E2E tests (Playwright) | `pnpm --dir showcase test:e2e` |
| Full benchmarks | `cargo bench -p kernel` |

---

## Architecture

### Geometry pipeline

The kernel is organized as a three-layer pipeline. Each layer has a single responsibility and feeds the next:

```
NURBS             create geometry (curves, surfaces, evaluation, fitting, sweep, loft)
  │
  ▼
Mesh              CSG booleans, volume computation, visualization, STL / glTF export
```

**NURBS** is the creation layer. Curves and surfaces are built here via interpolation, sweep, loft, or direct control-point specification. All geometry is rational B-spline.

**Mesh** is the output and boolean layer. Triangle meshes support CSG boolean operations (union, intersection, difference) via direct mesh intersection, volume computation (divergence theorem), 3D visualization, and mesh-format export (STL, glTF). Surfaces can be tessellated to meshes for booleans and visualization.

### Source layout

```
crates/kernel (Rust)
  ├── math/            — NURBS basis functions, evaluation, fitting
  ├── session/         — object registry and session lifecycle
  ├── kernel_impl/     — C ABI kernel operations
  │     ├── curve_*          — NURBS curve creation and evaluation
  │     ├── surface_*        — NURBS surface creation, tessellation, intersection
  │     ├── sweep_loft_ops   — sweep / loft surface construction
  │     ├── volume_ops       — mesh volume computation (divergence theorem)
  │     └── ffi_*            — C ABI + WASM FFI exports
  ├── landxml/         — LandXML parsing and sampling
  └── wasm/            — #[wasm_bindgen] public API
         ↓  wasm-pack build
bindings/web/          — ESM loader + TypeScript re-exports
         ↓
showcase/              — Next.js + Three.js interactive viewer
```

For a deeper dive, see:

- [Architecture Overview](ARCHITECTURE.md)
- [Kernel Module Map](docs/architecture/kernel-module-map.md)
- [ABI Stability](docs/architecture/abi-stability.md)
- [.NET Binding Readiness](docs/architecture/dotnet-binding-readiness.md)

---

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | High-level architecture, hot paths, session model |
| [Kernel C ABI Reference](docs/reference/kernel-c-abi.md) | C foreign-function interface |
| [Kernel WASM API Reference](docs/reference/kernel-wasm-api.md) | Full WASM/TypeScript API |
| [LandXML Support Matrix](docs/reference/landxml-support-matrix.md) | Supported LandXML elements |
| [NURBS Fit-Point RFC](docs/algorithms/nurbs-fit-point-interpolation-rfc.md) | Fit-point interpolation algorithm |
| [Cox–de Boor Evaluation](docs/algorithms/cox-de-boor-evaluation.md) | B-spline basis function algorithm |
| [Surface Tessellation](docs/algorithms/surface-tessellation.md) | Adaptive surface tessellation strategy |
| [Bounding Volumes](docs/algorithms/bounding-volumes.md) | AABB and OBB computation via PCA |
| [LandXML Alignment Evaluation](docs/algorithms/landxml-alignment-evaluation.md) | Horizontal and vertical alignment sampling |

---

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

---

## License

This project is licensed under the [MIT License](LICENSE).
