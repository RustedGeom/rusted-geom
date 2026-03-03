# Mesh Integration Plan (Kernel + ABI + Web Viewer)

## Enhancement Summary

- Deepened on: 2026-02-22
- Scope: add first-class mesh support end-to-end in kernel, ABI generation, TypeScript bindings, and showcase viewer.
- Existing baseline preserved: current curve stack (`NURBS`, line/arc/circle/polyline/polycurve), curve intersections, WASM runtime pipeline.
- Primary outcome: mesh objects become peers of curves in session state with kernel-driven examples for representation, transforms, intersections, and booleans.

### Key Improvements

1. Introduce a scalable mesh core with lazy topology + BVH caches to handle large meshes.
2. Add transform operations (translate/rotate/scale) that are O(1) when transform-only and optional bake-to-geometry.
3. Add robust mesh intersections (mesh-mesh and mesh-plane) with explicit result objects.
4. Add mesh booleans (union/intersection/difference) with robust manifold constraints and deterministic output.
5. Extend the viewer with non-trivial, kernel-generated mesh demos for each requested capability.

### New Considerations Discovered

- Robust booleans require strict topology contracts (closed/manifold/oriented meshes) or a repair path before boolean execution.
- For WASM, transform-as-instance and lazy caches are mandatory to avoid copying 1M+ triangle buffers per operation.
- Section/boolean result types must be explicit in ABI (not overloaded into current curve-only APIs) to keep bindings clean and safe.

---

## Section Manifest

1. Baseline and integration points in current repo.
2. Mesh object model and optimized representation for large meshes.
3. Mesh operation APIs (translation, rotation, scaling).
4. Mesh intersections (mesh-plane, mesh-mesh).
5. Mesh booleans.
6. Missing capabilities to include now (validation, repair, provenance, attributes).
7. Viewer example matrix (non-trivial, kernel-driven).
8. Phased implementation plan and acceptance criteria.
9. Risks, mitigations, and references.

---

## 1) Baseline Integration Map

### Existing architecture to preserve

- Kernel object/session model:
  - `crates/kernel/src/lib.rs`
- Generated ABI + bindings:
  - `tools/abi-gen/src/main.rs`
  - `bindings/web/src/generated/native.ts`
  - `bindings/web/src/generated/types.ts`
- Runtime bridge:
  - `bindings/web/src/runtime/kernel-session.ts`
  - `bindings/web/src/runtime/memory.ts`
- Viewer and examples:
  - `showcase/src/components/kernel-viewer.tsx`
  - `showcase/src/lib/preset-schema.ts`

### Architectural constraints

- Keep session-scoped handle model.
- Keep pointer-safe FFI style (native + wasm ptr variants).
- Keep ABI-generated TypeScript as source of truth.
- Keep viewer examples kernel-driven (construct in kernel, render in viewer).

---

## 2) Mesh Representation (Optimized for Large Meshes)

### 2.1 Proposed object model

Add mesh variants next to existing curve variants.

```rust
enum GeometryObject {
  Curve(CurveData),
  Mesh(MeshData),
  // optional future: Surface(SurfaceData)
}

struct MeshData {
  core: MeshCore,
  xform: RgmTransform3,      // instance transform, default identity
  caches: MeshCaches,
  metadata: MeshMetadata,
}
```

### 2.2 MeshCore layout (large-mesh oriented)

Use indexed triangles with contiguous vectors and no per-face heap allocations.

- `positions: Vec<RgmPoint3>`
- `indices: Vec<[u32; 3]>`
- Optional attributes (phase 2+):
  - `vertex_normals: Option<Vec<RgmVec3>>`
  - `face_groups: Option<Vec<u32>>`
- Keep index type `u32` in v1 (covers up to ~4B vertices, practical limits much lower).

### 2.3 Lazy overlays and caches

- `topology` (halfedge/adjacency overlay) is built lazily only for operations that need it.
- `bvh` is built lazily and invalidated only on topology-changing ops.
- `aabb` and `bounding_sphere` cached.
- `meshlets`/chunk partitions added in v2 for very large interactive workloads.

### 2.4 Why this representation

- Indexed mesh + cache-local contiguous storage is standard for memory/perf.
- Indexed halfedge structures can reduce memory versus pointer-heavy approaches while preserving adjacency semantics.
- Triangle ordering and quantization pipelines improve cache locality and memory footprint for large meshes.

### Research Insights

Best practices:

- Use contiguous index-based mesh structures for lower memory overhead and cache-friendly traversal.
- Run index/vertex reorder passes for large meshes before building acceleration structures.
- Build static AABB/BVH once and query many times; avoid per-query rebuilds.

Performance targets:

- 1M triangle mesh ingest + BVH build should be one-time cost per topology revision.
- Transform-only operations should avoid touching vertex buffers.

Implementation details:

- Add a `MeshCaches` struct with `Option` members and clear invalidation paths.
- Keep transforms separate from raw geometry until explicit bake.

Edge cases:

- Degenerate triangles (area near zero).
- Repeated vertices and non-finite values.
- Non-manifold edges at ingest.

---

## 3) Mesh Operations: Translation, Rotation, Scaling

### 3.1 API contract

Add immutable-return and in-place variants; default runtime path should use in-place transform for speed.

- `rgm_mesh_translate(session, mesh, delta, out_mesh)` (immutable variant)
- `rgm_mesh_rotate(session, mesh, axis, angle_rad, pivot, out_mesh)`
- `rgm_mesh_scale(session, mesh, scale_xyz, pivot, out_mesh)`
- `rgm_mesh_transform_in_place(session, mesh, transform)`
- `rgm_mesh_bake_transform(session, mesh, out_mesh)`

### 3.2 Numerical semantics

- Maintain transform as matrix/isometry in `MeshData.xform`.
- For intersections/booleans, choose one policy and enforce consistently:
  - Option A: always evaluate in world-space by applying xform lazily during queries.
  - Option B: bake before topology-changing ops.
- Recommended for v1: Option A for intersections, Option B for booleans (for simpler robust topology handling).

### Research Insights

Best practices:

- Separate geometric payload from instance transform to keep transforms O(1).
- Bake only when topology mutation or serialization requires final coordinates.

Edge cases:

- Non-uniform scale affects normals and orientation; detect determinant sign flips.
- Zero/near-zero scales should return `DegenerateGeometry`.

---

## 4) Mesh Intersections

## 4.1 Mesh-plane

### Output model

Add explicit polyline result object:

```rust
struct MeshSectionData {
  loops: Vec<Vec<RgmPoint3>>,   // multi-loop section result
  open_chains: Vec<Vec<RgmPoint3>>,
  plane: RgmPlane,
}
```

### Algorithm

1. Traverse mesh BVH for candidate triangles against plane AABB slab test.
2. Intersect candidate triangles with plane.
3. Merge segment fragments via tolerance-aware endpoint welding.
4. Build ordered loops/chains.
5. Return section object handle and optional summary (loop count, total length).

### ABI additions

- `rgm_intersect_mesh_plane(... out_section_handle)`
- `rgm_section_get_loop_count(...)`
- `rgm_section_copy_loop_points(...)`

### 4.2 Mesh-mesh

### Output model

- Intersection graph as polyline components (`MeshIntersectionData`), with optional witness triangle IDs.

### Algorithm

1. Dual-BVH traversal to collect candidate triangle pairs.
2. Robust triangle-triangle intersection per pair.
3. Collect unique segments with tolerant hashing.
4. Stitch segments into connected polylines/loops.
5. Return intersection object.

### Research Insights

Best practices:

- Use BVH broad-phase before exact primitive tests.
- Separate predicate robustness from construction robustness; predicates should be robust first.
- Mesh-plane and mesh-mesh should return topology-aware section results, not only point clouds.

Performance considerations:

- Dual-BVH traversal avoids O(n*m) brute-force pair checks.
- Intersection stitching must be linearithmic, not quadratic, in segment count.

Edge cases:

- Coplanar triangle overlaps.
- Near-parallel plane slices creating tiny fragments.
- Duplicate/near-duplicate segments.

---

## 5) Mesh Booleans

### 5.1 Supported operations (v1)

- Union
- Intersection
- Difference (A-B)
- Symmetric difference (optional v1.1)

### 5.2 Input contract

- Triangulated closed meshes.
- Manifold topology (or repaired before boolean).
- Consistent orientation.

Fail fast with clear status + `last_error` when preconditions fail.

### 5.3 Boolean engine strategy

Use backend abstraction from day one:

```rust
enum MeshBooleanBackend {
  NativeKernel,
  Boolmesh, // optional feature-gated backend
}
```

Recommended delivery strategy:

- v1 ship with backend abstraction + one production backend.
- Prefer backend that works in both native and wasm build pipeline.
- Keep deterministic output tests independent of backend.

### 5.4 Native boolean pipeline (if implemented in-tree)

1. Validate and normalize inputs.
2. Dual-BVH candidate triangle intersection.
3. Robust splitting / arrangement construction.
4. Cell/face classification by winding rules.
5. Boundary extraction and orientation.
6. Triangulation of output polygons.
7. Post-clean (weld, remove slivers, compact indices).

### Research Insights

Best practices:

- Treat robustness as primary requirement, not speed-first.
- Use exact/robust predicates where signs decide topology.
- Keep boolean output manifold guarantees explicit in API docs.

Performance:

- Broad-phase + parallel pair processing is mandatory for large meshes.
- Repair/cleanup should be proportional to changed regions where possible.

Edge cases:

- Coplanar overlapping faces.
- Tangential touches.
- Near-coincident triangles.

---

## 6) Missing Capabilities You Should Add Now

These are the key “you are missing this” items to avoid repainting later:

1. Mesh validation + repair API
- `rgm_mesh_validate` (closed, manifold, oriented, self-intersections)
- `rgm_mesh_repair` (weld, remove degenerates, orient consistently)

2. Provenance and groups
- Preserve source face/group IDs through intersections and booleans.
- Critical for debugging and viewer explainability.

3. Attribute policy
- Decide v1: geometry-only or geometry+normals/groups.
- If geometry-only, expose this explicitly to avoid surprise.

4. Determinism contract
- Stable output ordering for same input/tolerance/build.
- Required for reproducible tests and ABI consumers.

5. Tessellation bridge from current kernel
- Add kernel-generated mesh primitives so viewer demos are not static files:
  - box/cylinder/sphere/torus
  - optional implicit surface mesher (gyroid or signed-distance primitive) in v1.1

6. Performance instrumentation
- API endpoints to query triangle count, vertex count, BVH node count, memory estimate, build timings.

---

## 7) Viewer Examples (Non-Trivial, Kernel-Driven)

Add new `ExampleKey` entries and build each example entirely from kernel constructors + mesh ops.

### Example A: Large Mesh Representation

- Name: `Large Mesh (Gyroid Block + Stats)`
- Build path:
  - Generate implicit gyroid shell in kernel (or high-resolution procedural lattice if implicit mesher deferred).
  - Run weld + index compaction + BVH build.
- Viewer output:
  - shaded mesh + wireframe toggle
  - kernel stats panel: triangles, vertices, memory estimate, BVH nodes, build time.
- Non-trivial requirement: >500k triangles target in desktop mode.

### Example B: Transform Operations

- Name: `Transform Chain (Bracket + Rotor)`
- Build path:
  - Create two parametric meshes in kernel.
  - Apply chained translate/rotate/non-uniform scale around non-origin pivots.
- Viewer output:
  - original ghost mesh + transformed mesh
  - matrix readout + determinant/orientation warning.
- Kernel-driven checks:
  - AABB/volume delta logs before and after bake.

### Example C: Mesh-Mesh Intersection

- Name: `Mesh-Mesh Intersection (Offset Torus vs Ribbed Shell)`
- Build path:
  - Generate two non-trivial closed meshes in kernel.
  - Intersect and return polylines.
- Viewer output:
  - both meshes (semi-transparent) + highlighted intersection loops.
  - component count and loop lengths.

### Example D: Mesh-Plane Intersection

- Name: `Mesh-Plane Section Stack (Oblique Slicer)`
- Build path:
  - Kernel-generated complex mesh.
  - Intersect with 1..N planes (parallel + oblique).
- Viewer output:
  - section loops with consistent coloring per plane.
  - navigation slider across planes.

### Example E: Mesh Boolean

- Name: `Boolean CSG (Lattice Enclosure)`
- Build path:
  - Union of shell + ribs, then subtract angled channels, then intersect with clip volume.
- Viewer output:
  - input meshes ghosted, result solid.
  - manifold/validation badge from kernel.

### Example F: Validation/Repair (the extra capability)

- Name: `Repair Then Boolean (Broken Input Recovery)`
- Build path:
  - Start from intentionally flawed mesh (duplicate verts, open seam).
  - Run `validate`, `repair`, then boolean.
- Viewer output:
  - before/after diagnostics with exact kernel messages.

---

## 8) ABI + Bindings + Runtime Integration Plan

### 8.1 FFI type additions

- `RgmTriangle3u32 { a, b, c }`
- `RgmTransform3` (4x4 or 3x4 affine)
- `RgmMeshBooleanOp`
- `RgmMeshValidationReport` (counts + flags)

### 8.2 New exports (minimum set)

- mesh creation/import:
  - `rgm_mesh_create_indexed`
  - primitive constructors (`rgm_mesh_create_box`, etc.)
- transforms:
  - `rgm_mesh_translate`, `rgm_mesh_rotate`, `rgm_mesh_scale`, `rgm_mesh_bake_transform`
- intersections:
  - `rgm_intersect_mesh_plane`, `rgm_intersect_mesh_mesh`
- booleans:
  - `rgm_mesh_boolean`
- validation/repair:
  - `rgm_mesh_validate`, `rgm_mesh_repair`
- sampling/export:
  - `rgm_mesh_copy_vertices`, `rgm_mesh_copy_indices`

### 8.3 TypeScript runtime updates

- `bindings/web/src/runtime/memory.ts`
  - add mesh struct sizes + array read/write helpers.
- `bindings/web/src/runtime/kernel-session.ts`
  - add mesh APIs mirroring curve API style.
- keep error pathway via `KernelRuntimeError` unchanged.

### 8.4 Viewer schema updates

- Extend schema to support mesh session payloads while preserving existing curve preset compatibility.
- Recommend bump session format to `version: 2` with backward parser for version `1`.

---

## 9) Phased Execution Plan

## Phase 0: Foundations

- Add `MeshData` object variant and ingest/export APIs.
- Add mesh primitive constructors for kernel-driven demos.
- Add mesh rendering path in viewer.

Exit criteria:

- Create/release mesh handles.
- Render kernel-generated meshes in showcase.

## Phase 1: Optimized Representation + Transforms

- Add lazy BVH cache and mesh stats endpoints.
- Implement translate/rotate/scale + bake.
- Ship Example A + Example B.

Exit criteria:

- Large mesh demo runs with stable memory behavior.
- Transform chain demonstrates no vertex-copy churn before bake.

## Phase 2: Intersections

- Implement mesh-plane and mesh-mesh intersections + result object APIs.
- Ship Example C + Example D.

Exit criteria:

- Multi-loop sectioning works and is stable under oblique planes.
- Mesh-mesh intersection returns connected components correctly.

## Phase 3: Booleans + Validation/Repair

- Implement boolean backend abstraction and selected backend.
- Implement `validate` + minimal `repair` needed for boolean preconditions.
- Ship Example E + Example F.

Exit criteria:

- Union/intersection/difference pass manifold checks on curated corpus.
- Repair->boolean flow works for representative broken inputs.

## Phase 4: Hardening and Performance

- Add fuzz/property tests for robustness.
- Add native + wasm performance baselines.
- Freeze ABI and regenerate baseline artifacts.

Exit criteria:

- Deterministic outputs across repeated runs.
- No ABI regressions.

---

## 10) Acceptance Criteria

### Kernel

- Mesh objects are first-class and independently releasable.
- Transform APIs preserve precision and reject degenerate scales.
- Intersections return structured result objects (not raw ad-hoc buffers).
- Booleans produce validated manifold outputs or explicit failure status.

### Bindings

- New mesh APIs available in generated TS with strict types.
- WASM pointer marshalling supports large buffers safely.

### Viewer

- Six mesh examples available and selectable.
- All examples are kernel-generated and non-trivial.
- Intersection/boolean examples show explicit overlays + diagnostics.

### Quality

- Unit tests for kernel geometry operations.
- Runtime tests in `bindings/web/tests/runtime.test.ts` for mesh APIs.
- E2E viewer smoke updated for new examples.

---

## 11) Risks and Mitigations

1. Boolean robustness risk
- Mitigation: strict precondition validation + repair path + backend abstraction.

2. WASM memory pressure on large meshes
- Mitigation: transform-as-instance, lazy caches, chunked copy APIs.

3. Performance regressions from eager topology building
- Mitigation: build topology/BVH lazily and cache with explicit invalidation.

4. API surface bloat
- Mitigation: start with minimum stable primitives and extend only after example coverage.

5. Dependency risk if external boolean backend is used
- Mitigation: keep backend trait and golden test corpus to allow backend swap.

---

## 12) References

- libigl boolean workflow and arrangement+winding strategy:
  - https://libigl.github.io/tutorial/
- Fast and Robust Mesh Arrangements using Floating-Point Arithmetic (cited in libigl tutorial):
  - https://www.cs.columbia.edu/cg/mesh-arrangements/
- CGAL corefinement and boolean operations:
  - https://doc.cgal.org/latest/Polygon_mesh_processing/group__PMP__corefinement__grp.html
- CGAL AABB tree design and query model:
  - https://doc.cgal.org/latest/AABB_tree/index.html
- CGAL indexed halfedge `Surface_mesh` notes:
  - https://doc.cgal.org/latest/Surface_mesh/index.html
- Shewchuk robust predicates:
  - https://www.cs.cmu.edu/~quake/robust.html
- Fast winding numbers (inside/outside robustness context):
  - https://www.dgp.toronto.edu/projects/fast-winding-numbers/
- meshoptimizer (indexing/cache/fetch/quantization/meshlet pipeline):
  - https://github.com/zeux/meshoptimizer
- parry3d `TriMesh` capabilities (plane intersection + BVH):
  - https://docs.rs/parry3d/latest/parry3d/shape/struct.TriMesh.html
- boolmesh (pure Rust boolean backend candidate):
  - https://docs.rs/crate/boolmesh/0.1.7
- Manifold robustness concepts:
  - https://manifoldcad.org/docs/html/

