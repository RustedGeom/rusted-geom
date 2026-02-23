---
title: "perf: Optimize surface intersection algorithms to sub-50ms"
type: perf
date: 2026-02-23
---

# perf: Optimize Surface Intersection Algorithms to Sub-50ms

## Context

The geometry kernel's intersection algorithms — surface-surface, surface-curve, and surface-plane — are too slow for interactive CAD use. The target is consistent execution under 50ms per intersection call on representative inputs while maintaining full mathematical precision. The performance budget is being consumed by: (1) a nearest-seed lookup that is O(n log n) per grid point and called in tight inner loops; (2) fixed 24-iteration Levenberg-Marquardt that doesn't exit early on stagnation; (3) an O(n²) brute-force loop in curve-curve; (4) heap allocations inside the innermost NURBS evaluator; and (5) no parallelism anywhere. All algorithms are single-threaded with no external math dependencies.

---

## Critical Files

| File | Role |
|---|---|
| `crates/kernel-ffi/src/kernel_impl/surface_face_intersections_a.rs` | LM refinement loops (SSI, surface-curve, point projection) |
| `crates/kernel-ffi/src/kernel_impl/surface_face_intersections_b.rs` | Seed generation, marching, surface-curve candidates |
| `crates/kernel-ffi/src/kernel_impl/surface_face_intersections_c.rs` | Surface-plane grid, BranchSpatialDeduper |
| `crates/kernel-ffi/src/math/intersections.rs` | Curve-plane (384 samples), curve-curve (160×160 grid) |
| `crates/kernel-ffi/src/math/nurbs_surface_eval.rs` | NURBS evaluation hotspot, `validate_surface` |
| `crates/kernel-ffi/src/math/basis.rs` | `find_span`, `ders_basis_funs` — heap-allocates inside LM inner loop |
| `crates/kernel-ffi/Cargo.toml` | Add `criterion` dev-dep + `rayon` dep |
| `Cargo.toml` (workspace) | Add `rayon.workspace = true` |

---

## Phase 0 — Benchmark Infrastructure (prerequisite for everything else)

**Why first:** Without measurements every claim is speculation. Baselines must be captured before any code change.

### 0.1 — Add criterion

In `crates/kernel-ffi/Cargo.toml`:
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "intersection_bench"
harness = false
```

### 0.2 — Create `crates/kernel-ffi/benches/intersection_bench.rs`

Six benchmark groups, each calling FFI directly through the session handle (following the pattern in `tests/kernel_ffi.rs`):

| Group | What it stresses |
|---|---|
| `bench_surface_surface_cylinders` | Two degree-3 NURBS cylinders at 45°; long SSI curve |
| `bench_surface_surface_near_tangent` | Sphere vs grazing plane; maximises LM iterations |
| `bench_surface_plane_sphere` | NURBS sphere sliced; adaptive grid doubling path |
| `bench_surface_curve_multi_hit` | Sinusoidal degree-5 curve crossing a flat surface 4× |
| `bench_curve_curve_crossing` | Two degree-3 arcs; exercises 160×160 segment loop |
| `bench_curve_plane_dense` | 384-sample baseline for curve-plane |

Run before any changes: `cargo bench --bench intersection_bench -- --save-baseline before`

---

## Phase 1 — Algorithmic Wins (target: −55% vs baseline, zero precision risk)

### 1.1 — LM stagnation early exit (highest bang-per-line-of-code)

**Files:** `surface_face_intersections_a.rs` — all five LM loops:
- `refine_surface_surface_uv_pair` (line 182)
- `project_surface_surface_curve_point` (line 294)
- `refine_surface_curve_hit` (line 536)
- `project_point_to_surface` (line 665)
- `project_point_to_curve` in `surface_face_intersections_b.rs` (line 11)

**Change:** Add two stack variables before each `for _ in 0..24` loop:
```rust
let mut stagnation: u8 = 0;
let mut prev_norm = f64::INFINITY;
```
After each `best` update and before the convergence check, add:
```rust
let improvement = (prev_norm - residual_norm) / prev_norm.max(1e-30);
if improvement < 0.01 { stagnation += 1; } else { stagnation = 0; }
prev_norm = prev_norm.min(residual_norm);
if stagnation >= 3 && residual_norm <= tol * 8.0 { break; }
```
The `best` fallback already guarantees returning the best-seen result if we exit early — no accuracy risk. Cuts average iterations from 24 to 8–12 for well-behaved surfaces.

**Estimated impact:** −30% on all LM-heavy paths (marching, seeding, refinement).

### 1.2 — Curve-curve AABB pre-reject

**File:** `math/intersections.rs`, function `intersect_curve_curve_points` (line 301)

**Change:** Before the 160×160 loop, compute per-segment AABBs inflated by `pair_tol` for both curves (O(n) precompute). In the inner loop, add a 6-comparison AABB overlap test before calling `closest_segment_parameters`. Reject the pair immediately if bounding boxes don't overlap.

```rust
struct SegBBox { min_x: f64, max_x: f64, min_y: f64, max_y: f64, min_z: f64, max_z: f64 }
// build bbox_a[0..samples_a] and bbox_b[0..samples_b] once
// in inner loop:
if bbox_a[ia].min_x > bbox_b[ib].max_x || ... { continue; }
```

Inflation by `pair_tol` guarantees no genuine near-misses are skipped. For non-intersecting curves, reduces 25,600 expensive calls to ~200 cheap AABB rejections.

**Estimated impact:** −90% on curve-curve for non-intersecting cases; negligible overhead for intersecting cases.

### 1.3 — Spatial seed grid replacing `nearest_surface_seed_uvs`

**File:** `surface_face_intersections_b.rs`

**Problem:** `nearest_surface_seed_uvs` (line 114) does O(n) linear scan + full sort on every call. Called at line 228 and line 549 inside tight source-grid loops. For a 30×30 source grid against a 12×12=144-point seed grid: ~130k comparisons and ~1,800 allocating sorts per doubling iteration.

**Change:** Add a `SurfaceSeedGrid` struct backed by a 3D spatial cell index (same pattern as `BranchSpatialDeduper` in `surface_face_intersections_c.rs` lines 397–448):

```rust
struct SurfaceSeedGrid {
    seeds: Vec<SurfaceProjectionSeed>,
    cells: HashMap<(i32, i32, i32), Vec<usize>>,
    inv_cell: f64,
}

impl SurfaceSeedGrid {
    fn from_seeds(seeds: Vec<SurfaceProjectionSeed>, cell_size: f64) -> Self { ... }
    fn nearest_k(&self, point: RgmPoint3, k: usize) -> Vec<RgmUv2> {
        // scan 3×3×3 = 27 neighbor cells, partial-sort by distance
    }
}
```

Build the grid once before the outer loop in `generate_surface_surface_seeds` (line 310). Replace both `nearest_surface_seed_uvs` calls with `grid.nearest_k(point, 6)`.

**Estimated impact:** −3–5× on seeding phase; seeding is often the dominant cost.

### 1.4 — Surface-plane AABB pre-check

**File:** `surface_face_intersections_c.rs`, function `intersect_surface_plane_uv_segments` (line 356)

**Change:** Before the adaptive grid loop, compute the AABB of the NURBS control polygon (strictly contains the surface — fundamental NURBS convex hull property) and test whether all 8 corners are on the same side of the plane:

```rust
let (aabb_min, aabb_max) = surface_control_aabb(&surface_data);
// 8 corners → signed distances
// if all > tol or all < -tol → guaranteed miss → return Ok(vec![])
```

Factor `surface_control_aabb` into `math/mod.rs` as a shared helper.

**Estimated impact:** Near-zero cost for definite misses; high value for scene-level dispatch.

---

## Phase 2 — Structural Optimizations (target: additional −15%)

### 2.1 — Remove `validate_surface` from hot NURBS evaluation path

**File:** `math/nurbs_surface_eval.rs`

Extract validation into public entry point only. Add `pub(crate) fn eval_nurbs_surface_uv_unchecked` that skips `validate_surface`. Update `eval_surface_data_uv` in `curve_surface_ops_a.rs` (line 522) to call the unchecked variant — surfaces are validated at construction time in `build_surface_from_desc`.

### 2.2 — Stack-allocate `ders_basis_funs` temporaries

**File:** `math/basis.rs`, function `ders_basis_funs` (line 76)

Replace heap-allocated `ndu: Vec<Vec<f64>>` (line 85) and `a: Vec<Vec<f64>>` (line 115) with stack-allocated `[[f64; 6]; 6]` for degree ≤ 5 (covers all practical NURBS). Add runtime branch: `if degree > 5 { /* original heap path */ }`. Eliminates 2 heap allocations per NURBS surface evaluation.

### 2.3 — `#[inline(always)]` on critical inner-loop functions

**Files:** `math/basis.rs`, `curve_surface_ops_a.rs`

Add `#[inline(always)]` to:
- `find_span` (binary search, ~15 instructions)
- `eval_surface_data_uv` and `surface_eval_result_to_frame`

### 2.4 — Marching evaluation reuse

**Files:** `surface_face_intersections_a.rs`, `surface_face_intersections_b.rs`

Extend `project_surface_surface_curve_point` return type to include the final evaluated frames `(RgmSurfaceEvalFrame, RgmSurfaceEvalFrame)`. In `march_surface_surface_direction`, cache these frames and skip the re-evaluation at lines 381–382 of the next step's `surface_surface_tangent_dir`. Saves 2 NURBS evaluations per step (up to 480 for long curves at `max_steps`).

---

## Phase 3 — Parallelism via Rayon (target: additional −40–70% on N-core)

### 3.1 — Add rayon dependency

`Cargo.toml` workspace: `rayon = "1.10"`
`crates/kernel-ffi/Cargo.toml`: `rayon.workspace = true`

Verify `SurfaceData` and `SurfaceProjectionSeed` are `Send + Sync` (both are plain data structs — auto-derived).

### 3.2 — Parallel seed projection

**File:** `surface_face_intersections_b.rs`, `collect_surface_surface_projection_seeds`

Convert the grid iteration to a `par_iter()` map returning `Option<(RgmPoint3, RgmUv2, RgmUv2)>` per cell, then sequential deduplication pass. Same pattern for `generate_surface_curve_candidates` sample loop (lines 537–583). The second `BranchSpatialDeduper` pass in `generate_surface_surface_seeds` handles any ordering differences.

### 3.3 — Parallel forward + backward marching

**File:** `surface_face_intersections_b.rs`, `build_surface_surface_branch_from_seed` (line 450)

```rust
let (forward_result, backward_result) = rayon::join(
    || march_surface_surface_direction(..., 1.0, ...),
    || march_surface_surface_direction(..., -1.0, ...),
);
```

Both directions are fully independent (read-only surface data, no shared mutable state).

---

## Verification

### Run full test suite after every phase
```bash
cargo test --package kernel-ffi -- --nocapture
```

Key regression tests per phase:
- **Phase 1.1–1.3:** `viewer_surface_surface_example_returns_non_empty_branches`
- **Phase 1.4:** `surface_plane_and_surface_surface_intersections_are_trim_clipped`
- **Phase 1.2:** `intersect_curve_curve_counts_expected_hits`
- **Phase 2.1–2.2:** `can_evaluate_point_derivatives_and_plane`
- **Phase 3:** Add parallel stress test — 8 concurrent sessions performing the same SSI, assert results identical to sequential

### Benchmark gates
```bash
cargo bench --bench intersection_bench -- --baseline before
```

| After phase | Target vs baseline |
|---|---|
| Phase 1 complete | ≤ −55% |
| Phase 2 complete | ≤ −65% |
| Phase 3 complete (4 cores) | ≤ −80%, ≤ 50ms on all fixtures |

### Precision guard for Phase 1.1
In debug builds, assert that residual_norm at stagnation exit is within `tol * 2.0` of the 24-iteration path result on the same seed.

---

## Estimated Total Impact

| Optimization | Mechanism | Est. Speedup |
|---|---|---|
| 1.1 LM stagnation exit | Fewer iterations per convergence | −30% overall |
| 1.2 Curve-curve AABB | Skip 95%+ of O(n²) pairs | −90% curve-curve |
| 1.3 Spatial seed grid | O(1) vs O(n log n) nearest lookup | −3–5× seeding |
| 1.4 Surface-plane AABB | Skip definite misses before any eval | Case-dependent |
| 2.x Micro-opts | Inlining + stack alloc + eval reuse | −15% residual |
| 3.x Rayon | N-core parallelism on seed+march | −40–70% on 4+ cores |

**Combined target: sub-50ms on representative inputs, full mathematical precision preserved.**
