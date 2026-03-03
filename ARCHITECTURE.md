# Architecture Overview

## Repository Layout

```
rusted-geom/
  crates/
    kernel/              — Rust NURBS geometry kernel + wasm-bindgen API
  bindings/
    web/                 — Thin TypeScript wrapper around wasm-pack output
  showcase/              — Next.js + Three.js demo app + developer test lab
  scripts/               — Build and staging scripts
  docs/
    algorithms/          — Algorithm RFCs
    architecture/        — Architecture docs
```

---

## Rust Kernel (`crates/kernel`)

### Source layout

```
src/
  lib.rs                 — Crate root; declares all top-level modules
  kernel_impl.rs         — Flat module: pulls in all kernel_impl/*.rs via include!
  kernel_impl/           — Per-domain kernel operations (include! flat module)
    curve_*.rs           — NURBS curve operations
    surface_*.rs         — NURBS surface operations
    mesh_*.rs            — Mesh operations and booleans
    face_*.rs            — Trimmed face operations
    intersection_*.rs    — Curve/surface/mesh intersection algorithms
    brep_*.rs            — B-rep solid assembly and validation
    bounds_ops.rs        — Bounding box computation (AABB/OBB) with caching
    ffi_*.rs             — C ABI export functions (extern "C", #[no_mangle])
  math/
    basis.rs             — find_span, ders_basis_funs (stack-alloc fast path ≤ degree 5)
    nurbs_surface_eval.rs — eval_nurbs_surface_uv_unchecked (hot-path, no validation)
    bounds.rs            — PCA via Jacobi eigendecomposition for OBB computation
    *.rs                 — Other NURBS math utilities
  elements/
    brep/                — B-rep data structures (Face, Loop, Edge, Vertex)
  session/
    store.rs             — Global session registry (DashMap keyed by session ID)
    objects.rs           — Per-session object store
  wasm/                  — wasm-bindgen public API (the web-facing layer)
    mod.rs               — KernelSession struct + handle macros
    curve.rs             — Curve constructors and evaluation
    surface.rs           — Surface constructors and evaluation
    mesh.rs              — Mesh operations
    face.rs              — Trimmed face operations
    intersection.rs      — Intersection operations
    brep.rs              — B-rep assembly
    bounds.rs            — Bounding box computation
    error.rs             — Result<T, JsValue> helpers
```

### The `include!` flat module pattern

`kernel_impl.rs` looks like this:

```rust
include!("kernel_impl/curve_ops.rs");
include!("kernel_impl/surface_ops.rs");
// … 50+ more files
```

This means all `use` statements and type definitions in an earlier-included file are
automatically visible in later files — there is no need to `use super::*` or re-import.
This is intentional for performance: it allows `#[inline(always)]` functions to be
inlined across "module" boundaries without any runtime cost.

**Rule:** When adding a new operation, create a new file in `kernel_impl/` and add a
single `include!` line to `kernel_impl.rs`. Place it after any files whose items you need.

### Rayon / WASM threading gate

Rayon runs sequentially in WASM (single-threaded) while still paying scheduler
overhead. Every parallel operation is gated:

```rust
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

// Usage:
#[cfg(not(target_arch = "wasm32"))]
let result = items.par_iter().map(f).collect();
#[cfg(target_arch = "wasm32")]
let result = items.iter().map(f).collect();
```

Also gate the `use rayon::prelude::*` import itself — leaving it in under WASM
causes an unused-import warning that breaks `--deny warnings` builds.

### Session and object ownership

- `KernelSession` owns a session in the global `SESSIONS` store.
- Every geometry object (curve, surface, mesh, …) is stored in the session's
  object registry, keyed by a `u64` object ID.
- `CurveHandle`, `SurfaceHandle`, etc. hold `(session_id, object_id)` pairs.
- `Drop` on a handle calls `rgm_object_release`, which decrements the ref count
  and frees the object when it reaches zero.
- `Drop` on `KernelSession` calls `rgm_kernel_destroy`, which releases all
  remaining objects and removes the session from the store.
- On the JS side, wasm-bindgen registers a `FinalizationRegistry` callback
  so that GC-collected handles also release their objects. Explicit `.free()`
  calls are preferred for deterministic cleanup.

### wasm-bindgen API conventions

- All `Vec<f64>` parameters become `Float64Array` in the generated TypeScript.
  Callers must pass `new Float64Array([...])`, not plain `number[]`.
- Points are passed flat: `[x, y, z, x, y, z, …]`.
- UV coordinates are passed flat: `[u, v, u, v, …]`.
- Planes are passed flat: `[ox, oy, oz, xx, xy, xz, yx, yy, yz, zx, zy, zz]`.
- Fallible operations return `Result<T, JsValue>` — they throw in JavaScript.
- `Bounds3` is a value type with flat public fields (`aabb_min_x`, `obb_center_x`,
  `obb_ax_x`, `local_aabb_min_x`, etc.) — call `.free()` when done.

---

## Performance Hot Paths

### Basis function evaluation (`math/basis.rs`)

`ders_basis_funs` uses a stack-allocated array for degree ≤ 5, avoiding heap
allocation in the hot path. For degree > 5 it falls back to a heap `Vec`.

### Surface evaluation (`math/nurbs_surface_eval.rs`)

`eval_nurbs_surface_uv_unchecked` skips parameter validation and knot span
search validation for internal callers that already hold valid `(u, v)` parameters.
Do not call this from user-facing code without first validating inputs.

### SSI stagnation early-exit (`kernel_impl/surface_face_intersections_a.rs`)

The Levenberg-Marquardt inner loop breaks on stagnation after 3 non-improving
iterations:

```rust
if stagnation >= 3 { break; }
```

The `best` tracker stores the lowest-residual result seen across all iterations,
so early exit is always safe — the final result is the optimum found regardless
of which iteration we stopped at. **Do not add a residual threshold guard here**
(e.g., `&& residual_norm <= tol * 8.0`) — that guard silently disables early
exit for grid cells far from the intersection, forcing full 24-iteration runs on
the vast majority of the seed grid and causing a ~5× slowdown.

### Bounding box caching (`kernel_impl/bounds_ops.rs`)

`compute_bounds_for_object` caches results in `SessionState.bounds_cache` keyed
by `(object_id, BoundsOptions)`. The cache is invalidated when an object is
modified. The `BoundsMode::Fast` path (control-point hull) is ~10× faster than
`BoundsMode::Optimal` (PCA + OBB refinement) — use `Fast` for visibility culling.

---

## TypeScript / WASM Layer

### Build pipeline

```
cargo → wasm-pack build → crates/kernel/pkg/
                               ├── rusted_geom_bg.wasm
                               ├── rusted_geom.js
                               └── rusted_geom.d.ts
                          ↓ scripts/stage_web_wasm.sh
                          showcase/public/wasm/rusted_geom_bg.wasm
                          bindings/web/dist/
```

`scripts/build_kernel_wasm.sh` runs `wasm-pack build --target web --release` and
stages the output. Run it after any Rust change.

### `bindings/web`

The package re-exports `KernelSession` and all handle types from the wasm-pack
output, plus a `loadKernel(url)` helper that calls wasm-pack's `init()`:

```ts
import { loadKernel } from "@rusted-geom/bindings-web";
await loadKernel(wasmUrl);          // call once before any KernelSession
const session = new KernelSession();
```

### Showcase test lab (`showcase/src/app/tests/`)

The `/tests` route runs the kernel contract suite in the browser against the live
WASM build. It is developer infrastructure — run it to verify WASM correctness
after a kernel change without needing to write a new Playwright test.
