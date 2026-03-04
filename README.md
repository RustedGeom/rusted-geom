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

**[Live Showcase](https://rusted-geom-kernel-showcase.vercel.app)** — interactive 3D viewer with NURBS curves, surfaces, mesh booleans, and more.

### Key Capabilities

- **NURBS Curves & Surfaces** — construction, evaluation, tessellation, fit-point interpolation
- **Mesh Operations** — box/torus primitives, boolean operations (union, intersection, difference)
- **B-Rep Modeling** — trimmed faces, loops, edges, vertex topology
- **Intersections** — surface–surface and curve–surface intersection with branch tracking
- **Bounding Volumes** — axis-aligned (AABB) and oriented (OBB) bounding boxes
- **LandXML** — parsing, sampling, and surface reconstruction from survey data
- **CAD Export** — IGES 5.3 and ACIS SAT file generation for curves, surfaces, and B-rep
- **Benchmarked** — Criterion-based benchmarks with regression detection

### Project Status

> **Alpha** (`0.1.0-alpha.4`) — APIs and package layout are evolving. Contributions and feedback welcome.

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
| `crates/kernel` | Core NURBS math, B-Rep structures, session management, WASM bindings |
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

## Quickstart

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
| Run Rust unit tests | `cargo test -p kernel` |
| TypeScript type check | `npm --prefix ./bindings/web run typecheck` |
| Web runtime tests | `npm --prefix ./bindings/web run test` |
| Showcase unit tests | `pnpm --dir showcase test:unit` |
| E2E tests (Playwright) | `pnpm --dir showcase test:e2e` |
| Full benchmarks | `cargo bench -p kernel` |

---

## Architecture

```
crates/kernel (Rust)
  ├── math/            — NURBS basis functions, evaluation, fitting
  ├── elements/brep/   — B-rep topology (Face, Loop, Edge, Vertex)
  ├── session/         — object registry and session lifecycle
  ├── kernel_impl/     — C ABI kernel operations
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
