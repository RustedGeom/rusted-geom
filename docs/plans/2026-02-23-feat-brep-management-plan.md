---
title: "feat: BREP management (create, operations, NURBS integration, I/O)"
type: feat
date: 2026-02-23
---

# feat: BREP Management (Performance-First Kernel Plan)

## Enhancement Summary

**Deepened on:** 2026-02-24 (pass 1) → 2026-02-24 (pass 2)
**Sections enhanced:** 10
**Research inputs used:** Live codebase exploration (actual file/struct contents),
  Rust ecosystem research (index_vec, bon, slotmap, ahash, bitvec, dashmap, postcard,
  bincode 3.0, rkyv WASM hazard, bumpalo, rstar, criterion, codspeed),
  STEP/IGES ecosystem audit (truck-stepio 0.3.0 entity coverage, ruststep warning,
  AP214/AP242 entity sets, STEP round-trip fidelity hazards, IGES viability analysis),
  FFI/ABI analysis (WASM usize hazard, validation report design, orientation semantics),
  BREP algorithm research (sewing phases, canonical validation order, healing pipeline,
  watertight tessellation, transaction protocol), performance architecture research.

### Key Improvements
1. **Tiered arena strategy**: `index_vec` typed u32 indices for primary topology arrays
   (zero overhead, compile-time safety); `slotmap::SecondaryMap` only for derived caches
2. Fixes critical STEP I/O risk: ruststep is "DO NOT USE FOR PRODUCT"; truck-stepio has
   open boolean gap (issue #91); revises I/O to bincode-3.0 native + import-only STEP
3. **`rkyv` WASM alignment hazard fixed**: `f64` is 4-byte aligned on wasm32 vs 8-byte on
   x86_64 — rkyv archived bytes are not portable across targets; replaced with bincode 3.0
4. **IGES dropped entirely**: no Rust library exists, dying standard, poor BREP quality
5. Designs concrete `RgmBrepValidationReport` (16-issue inline, stack-allocated, no-heap)
6. Specifies `BrepFaceId`/`BrepEdgeId` as branded `number` (u32), not `bigint` (u64)
7. Adds per-session DashMap locking to eliminate cross-session Mutex bottleneck
8. Adds concrete algorithm pseudocode: 6-phase sewing, 10-check validation order,
   8-pass healing pipeline, watertight tessellation, transaction protocol

### New Considerations Discovered
- `elements/face/` stubs already exist — Phase 3 fills existing files, not new ones
- `ops/` directory already exists — BREP ops go in `ops/brep/` following existing pattern
- No `benches/` directory exists — Phase 0 must create it with Cargo.toml bench config
- Rayon threshold for geometry: ~10,000 ns per task; per-face AABB only pays off for N>64
- `boolmesh` crate (already imported) is for mesh booleans only — NURBS BREP booleans
  require a separate implementation using the surface-surface intersection pipeline
- STEP orientation has THREE layered flags (`EDGE_CURVE.same_sense`,
  `ORIENTED_EDGE.orientation`, `ADVANCED_FACE.same_sense`) — most common source of
  normal-direction bugs on import; must validate all three on every edge
- truck-stepio 0.3.0 lacks `TRIMMED_CURVE` and `COMPOSITE_CURVE` — blocks reading any
  STEP file exported by CATIA, NX, or CREO; add in second STEP pass

---

## Context

You need to implement BREP management with four outcomes:
1. Create BREP objects.
2. Run operations on BREP.
3. Manage existing NURBS curves/surfaces as BREP.
4. Add BREP I/O.

Performance is a hard requirement for kernel paths.

---

## Section Manifest

Section 1: Baseline and constraints
Research focus: what already exists in object model/ABI/runtime and what cannot regress.

Section 2: BREP data model
Research focus: topology/geometry split, orientation semantics, entity sharing, stable IDs.

Section 3: Creation APIs
Research focus: minimal ABI, construction correctness, low-copy ingestion from existing objects.

Section 4: BREP operations
Research focus: staged operations with robust validation/healing and predictable complexity.

Section 5: NURBS integration
Research focus: wrapping existing curve/surface handles without duplication.

Section 6: I/O strategy
Research focus: pragmatic internal format first, standards-based exchange second.

Section 7: Performance architecture
Research focus: data layout, locking model, caching, parallelism, benchmark gates.

Section 8: Implementation phases and file map
Research focus: concrete sequencing and touchpoints in this repo.

Section 9: Acceptance criteria
Research focus: correctness, interoperability, and latency/throughput budgets.

Section 10: Risks and mitigations
Research focus: algorithmic, ABI, interoperability, and memory risks.

---

## 1) Baseline and Constraints

Current repository state:
- `GeometryObject` supports `Curve`, `Surface`, `Face`, `Mesh`, `Intersection` but no BREP aggregate object.
- `FaceData` already models trimmed surface topology (`surface + loops/edges`) and is the closest precursor to a BREP face.
- TS runtime advertises `igesImport: false`, `igesExport: false`; no current BREP import/export ABI.
- ABI compatibility is enforced by `abi/baseline/rgm_abi.json`; BREP APIs must be additive and versioned.

Design constraints:
- Keep session-scoped opaque handles and error semantics (`RgmStatus`) consistent.
- Preserve existing curve/surface/face behavior and do not force BREP adoption for existing clients.
- Make BREP ops deterministic under tolerance context and robust to near-degenerate geometry.

### Research Insights

**Codebase Reality — do NOT recreate these (already exist):**
- `crates/kernel-ffi/src/elements/face/` — has stubs: `heal.rs`, `loops.rs`, `tessellate.rs`, `types.rs`, `validate.rs`
- `crates/kernel-ffi/src/elements/` — dirs: `curve/`, `face/`, `intersection/`, `mesh/`, `surface/`, `intersections/`, `transform.rs`
- `crates/kernel-ffi/src/ops/` — exists already (has `transform.rs`); BREP ops go in `ops/brep/`
- `FaceData` already has: `surface: RgmObjectHandle` + `loops: Vec<TrimLoopData>` where `TrimLoopData` has `Vec<TrimEdgeData>`
- `TrimEdgeData` already has: `start_uv`, `end_uv`, `curve_3d: Option<RgmObjectHandle>`, `uv_samples: Vec<RgmUv2>`
- `boolmesh` crate already imported — mesh booleans only; NURBS BREP booleans need separate implementation
- `write_slice` helper in `ffi/ptr.rs` — use for all array output (count probe pattern)
- `#[rgm_export(ts = "...", receiver = "...")]` + `#[no_mangle]` — all exports follow this macro pattern
- **No `benches/` directory exists yet** — Phase 0 must create it

**Session storage (existing enums to extend):**
```rust
// session/store.rs
pub(crate) static SESSIONS: Lazy<Mutex<HashMap<u64, SessionState>>>;
pub(crate) static NEXT_SESSION_ID: AtomicU64;
pub(crate) static NEXT_OBJECT_ID: AtomicU64;

// session/objects.rs
pub(crate) enum GeometryObject {
    Curve(CurveData), Mesh(MeshData), Surface(SurfaceData),
    Face(FaceData), Intersection(IntersectionData),
    // Add: Brep(BrepData), BrepInProgress(BrepData)
}
```

**TS handles (existing branded pattern to follow for BREP):**
```typescript
// handles.ts — existing branded pattern
type FaceHandle = bigint & { readonly [_faceBrand]: void };
// Add BrepHandle (bigint), BrepFaceId (number/u32), BrepEdgeId, BrepLoopId, BrepShellId
```

**Best Practices:**
- Keep geometry and topology separated, with topology entities reusing geometry and carrying orientation metadata.
- Support shared sub-shapes (same vertex/edge used by multiple parents) instead of deep copies.

**References:**
- openNURBS `ON_Brep` class docs: [developer.rhino3d.com](https://developer.rhino3d.com/api/cpp/class_o_n___brep.html)
- Open CASCADE Topology guide: [dev.opencascade.org](https://dev.opencascade.org/doc/overview/html/occt_user_guides__modeling_data.html)

---

## 2) BREP Data Model (Kernel Internal)

Introduce two new session object variants:
- `GeometryObject::Brep(BrepData)` — finalized, validated BREP
- `GeometryObject::BrepInProgress(BrepData)` — under construction via C ABI; `rgm_brep_finalize_shell` transitions to `Brep`

Proposed topology entities (arena-indexed):
- `BrepVertex { point: RgmPoint3, tol: f64, incident_edges: SmallVec<[EdgeId; 4]> }`
- `BrepEdge { curve_3d: Option<RgmObjectHandle>, v_start: VertexId, v_end: VertexId, trims: SmallVec<[TrimId; 2]> }`
- `BrepTrim { edge: EdgeId, face: FaceId, loop_id: LoopId, uv_curve: Trim2dRep, reversed: bool }`
- `BrepLoop { trims: SmallVec<[TrimId; 8]>, is_outer: bool }`
- `BrepFace { surface: RgmObjectHandle, loops: SmallVec<[LoopId; 4]>, orientation: i8, bbox: Option<Aabb3> }`
- `BrepShell { faces: Vec<FaceId>, closed: bool }`
- `BrepSolid { shells: Vec<ShellId> }`
- `BrepData { vertices, edges, trims, loops, faces, shells, solids, cache }`

Key invariants:
- Edge orientation is per-use (trim level), not per-edge global.
- One geometric edge can be referenced by multiple trims/faces.
- Faces reference surfaces by handle; no control-net duplication by default.
- All topology arrays use stable typed indices for O(1) lookup and cache-friendly traversal.

### Research Insights

**Tiered arena strategy — `index_vec` primary, `slotmap` secondary:**

For a CAD kernel with distinct build/query phases and rare per-element deletion, `slotmap` as the primary topology storage adds generational overhead (4 bytes per slot, version check per access) that is unnecessary. Plain `IndexVec` (from `index_vec`) gives the same compile-time type safety at zero runtime cost:

```toml
# Cargo.toml additions
index_vec = "0.1.4"   # typed u32 indices over plain Vec, zero runtime cost
smallvec = { version = "1.15", features = ["union"] }  # inline edge/trim/loop lists
bon = "3.9"           # typestate builders for entity construction
ahash = "0.8"         # faster HashMap for adjacency caches, WASM-safe
# Optional — only if you need safe deletion during incremental edits:
# slotmap = { version = "1.1", features = ["serde"] }
```

```rust
use index_vec::{define_index_type, IndexVec};

define_index_type! { pub struct VertexId = u32; }
define_index_type! { pub struct EdgeId   = u32; }
define_index_type! { pub struct TrimId   = u32; }
define_index_type! { pub struct LoopId   = u32; }
define_index_type! { pub struct FaceId   = u32; }
define_index_type! { pub struct ShellId  = u32; }
define_index_type! { pub struct SolidId  = u32; }

pub(crate) struct BrepData {
    pub(crate) vertices: IndexVec<VertexId, BrepVertex>,
    pub(crate) edges:    IndexVec<EdgeId,   BrepEdge>,
    pub(crate) trims:    IndexVec<TrimId,   BrepTrim>,
    pub(crate) loops:    IndexVec<LoopId,   BrepLoop>,
    pub(crate) faces:    IndexVec<FaceId,   BrepFace>,
    pub(crate) shells:   IndexVec<ShellId,  BrepShell>,
    pub(crate) solids:   IndexVec<SolidId,  BrepSolid>,
    pub(crate) cache:    BrepCache,
}
```

`IndexVec` is a newtype over `Vec` — iteration is as fast as a plain `Vec`, indexing with the wrong type is a compile error. IDs are `u32` (WASM-safe), serializable directly as sub-entity indices in FFI.

**Use `slotmap::SecondaryMap` only for derived adjacency caches:**

When you need edge-to-faces or vertex-to-edges maps keyed by the same IDs but built lazily:

```rust
// For deletion-heavy paths (V2 boolean ops), upgrade primary to slotmap:
// use slotmap::{new_key_type, SlotMap, SecondaryMap};
// new_key_type! { pub struct BrepEdgeKey; }
// let mut edge_faces: SecondaryMap<BrepEdgeKey, SmallVec<[FaceId; 2]>>;
```

For the initial BREP build phases, plain `Vec<SmallVec<[FaceId; 2]>>` indexed by `EdgeId.index()` is more cache-friendly than a hash-based secondary map.

**SmallVec sizing rules (based on real CAD model statistics):**

| Entity | Field | Inline N | Rationale |
|---|---|---|---|
| `BrepVertex` | `incident_edges` | 4 | Manifold valence almost always ≤ 4 |
| `BrepEdge` | `trims` | 2 | Always exactly 2 for closed manifold |
| `BrepLoop` | `trims` | 8 | Rectangular/cylindrical patches inline |
| `BrepFace` | `loops` | 4 | 1 outer + 0–3 holes |
| `BrepShell` | `faces` | `Vec` | Never small |

Always enable `features = ["union"]` — removes one machine-word discriminant per SmallVec (8 bytes saved per instance).

**Struct layout — `f64` fields first, `bool` last (no padding holes):**
```rust
// Good: 56 bytes, all 8-byte aligned, no padding
pub(crate) struct BrepVertex {
    pub(crate) point: RgmPoint3,                         // 3×f64 = 24 bytes
    pub(crate) tol: f64,                                 // 8 bytes
    pub(crate) incident_edges: SmallVec<[EdgeId; 4]>,   // inline = 4×4+8 = 24 bytes
}

// Good: u32 indices packed together, bool last (3 bytes padding at end, acceptable)
pub(crate) struct BrepTrim {
    pub(crate) edge:    EdgeId,   // 4 bytes
    pub(crate) face:    FaceId,   // 4 bytes
    pub(crate) loop_id: LoopId,   // 4 bytes
    pub(crate) reversed: bool,    // 1 byte (+3 pad)
    pub(crate) uv_curve: Trim2dRep, // depends on enum size
}
```

Add `#[cfg(test)] assert_eq!(std::mem::size_of::<BrepTrim>(), EXPECTED)` tests to prevent accidental layout growth.

**Dirty flag cache invalidation pattern:**
```rust
pub(crate) struct BrepCache {
    bbox_dirty: bool,
    adjacency_dirty: bool,
    pub(crate) shell_bbox: Option<Aabb3>,
    pub(crate) edge_to_faces: Option<Vec<SmallVec<[FaceId; 2]>>>,
    pub(crate) vertex_to_edges: Option<Vec<SmallVec<[EdgeId; 8]>>>,
    pub(crate) face_neighbors: Option<Vec<SmallVec<[FaceId; 6]>>>,
}
impl BrepCache {
    pub(crate) fn invalidate_topology(&mut self) {
        self.bbox_dirty = true;
        self.adjacency_dirty = true;
        self.shell_bbox = None;
        self.edge_to_faces = None;
        self.vertex_to_edges = None;
        self.face_neighbors = None;
    }
    pub(crate) fn invalidate_geometry(&mut self) {
        // Surface geometry changed; topology (adjacency) unchanged
        self.bbox_dirty = true;
        self.shell_bbox = None;
    }
}
```

Any `pub(crate)` mutation method on `BrepData` must call `self.cache.invalidate_topology()` or `self.cache.invalidate_geometry()` at its end.

**References:**
- [developer.rhino3d.com](https://developer.rhino3d.com/api/cpp/class_o_n___brep.html)
- [dev.opencascade.org](https://dev.opencascade.org/doc/overview/html/occt_user_guides__modeling_data.html)
- `index_vec` docs: [docs.rs/index_vec](https://docs.rs/index_vec/latest/index_vec/)
- `bon` typestate builders: [docs.rs/bon](https://docs.rs/bon/latest/bon/)

---

## 3) Creation APIs (C ABI + TS Session)

Add additive exports (new `ffi/exports/brep.rs`):
- `rgm_brep_create_empty(session, out_brep)`
- `rgm_brep_create_from_faces(session, faces_ptr, face_count, out_brep)`
- `rgm_brep_create_from_surface(session, surface, out_brep_face_or_shell)`
- `rgm_brep_add_face(session, brep, surface, out_face_id)`
- `rgm_brep_add_loop_uv(session, brep, face_id, uv_ptr, uv_count, is_outer)`
- `rgm_brep_finalize_shell(session, brep, out_shell_id)`
- `rgm_brep_validate(session, brep, out_report)`

TS runtime additions:
- `session.brep.*` client mirroring current `curve/surface/face` client pattern.
- Keep current `session.face.*` APIs intact; add bridge helpers to convert face handles to BREP faces.

### Research Insights

**`BrepInProgress` for staged C ABI construction:**

```rust
// session/objects.rs — add alongside existing variants
pub(crate) enum GeometryObject {
    Curve(CurveData),
    Surface(SurfaceData),
    Face(FaceData),
    Mesh(MeshData),
    Intersection(IntersectionData),
    Brep(BrepData),                  // finalized — validate was called, shell is closed
    BrepInProgress(BrepData),        // under construction via rgm_brep_add_face et al.
}
```

`rgm_brep_finalize_shell` transitions `BrepInProgress` → `Brep`. Calling any mutation API on a `Brep` (already finalized) returns `InvalidInput`. This prevents accidental structural edits after validation.

**Typestate builder for internal Rust construction** (`bon` crate):
```rust
use bon::Builder;

#[derive(Debug, Builder)]
pub(crate) struct BrepEdge {
    pub(crate) v_start: VertexId,
    pub(crate) v_end: VertexId,
    #[builder(default)]
    pub(crate) curve_3d: Option<RgmObjectHandle>,
    #[builder(default)]
    pub(crate) trims: SmallVec<[TrimId; 2]>,
}
// Compiler error if v_start or v_end missing at call site.
let edge = BrepEdge::builder().v_start(v0).v_end(v1).build();
```

**Concrete validation report — fixed-size inline struct (no heap allocation):**

```rust
// Add to foundation.rs
#[rgm_ffi_type]
#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum RgmValidationSeverity { Info = 0, Warning = 1, Error = 2 }

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RgmValidationIssue {
    pub severity:    RgmValidationSeverity,
    pub code:        u32,    // machine-readable issue code
    pub entity_kind: u32,    // 0=vertex, 1=edge, 2=trim, 3=loop, 4=face
    pub entity_id:   u32,    // arena index as u32
    pub param_u:     f64,    // UV location of issue; NaN if unknown
    pub param_v:     f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RgmBrepValidationReport {
    pub issue_count:  u32,
    pub max_severity: RgmValidationSeverity,
    pub overflow:     bool,                      // true if >16 issues exist
    pub issues:       [RgmValidationIssue; 16],  // inline, stack-allocated
}
```

**Sub-entity ID convention — use `u32` (branded `number`), NOT `u64`/`bigint`:**

BREP topology sub-entities are addressed by `u32` arena indices, not `u64` session handles. Expose in TS as branded `number` types (not `bigint`) — WASM is 32-bit address space, `u32` is sufficient for all entity counts.

```typescript
// handles.ts additions
declare const _brepBrand:       unique symbol;
declare const _brepFaceIdBrand: unique symbol;
declare const _brepEdgeIdBrand: unique symbol;
declare const _brepLoopIdBrand: unique symbol;
declare const _brepShellIdBrand: unique symbol;

export type BrepHandle  = bigint & { readonly [_brepBrand]: void };
export type BrepFaceId  = number & { readonly [_brepFaceIdBrand]: void };
export type BrepEdgeId  = number & { readonly [_brepEdgeIdBrand]: void };
export type BrepLoopId  = number & { readonly [_brepLoopIdBrand]: void };
export type BrepShellId = number & { readonly [_brepShellIdBrand]: void };
```

**WASM usize hazard:** Never use `usize` for BREP counts or entity indices in FFI-exposed structs. Use `u32` explicitly — WASM is 32-bit and `usize` = 4 bytes there vs 8 bytes on native; mismatches cause silent ABI breaks.

**Best Practices:**
- Introduce APIs in narrow slices and keep them additive for ABI stability.
- Separate mutation and validation entrypoints; avoid implicit expensive validation on each call.

**Edge Cases:**
- Input faces referencing surfaces from different sessions must return `InvalidInput`.
- Empty/degenerate loops or non-closed trim chains must fail fast with structured validation codes.
- `write_slice` from `ffi/ptr.rs` handles null-probe / count-probe pattern — use it for all array output.

---

## 4) Operations on BREP

### V1 (must-have, low risk)
- `brep_clone`
- `brep_transform` (topology preserved, geometry handles transformed/wrapped)
- `brep_sew_faces` (merge coincident edges/vertices within tolerance)
- `brep_split_edge`
- `brep_merge_collinear_edges`
- `brep_face_adjacency`
- `brep_tessellate_to_mesh` (per-face tessellation with shared boundary welding)
- `brep_validate` and `brep_heal`

### V2 (follow-up)
- `brep_boolean` (solid/shell boolean; staged via robust surface/face intersection pipeline — NOT via `boolmesh` which is mesh-only)
- `brep_offset_shell`
- `brep_slice_by_plane`

### Research Insights

**File placement:** `ops/` already exists with `transform.rs`. BREP ops go in `ops/brep/`, following the existing pattern. Do not create a new top-level directory.

**`boolmesh` is mesh-only:** The `boolmesh` crate already imported handles mesh boolean operations only. NURBS BREP booleans require a separate implementation on top of the surface-surface intersection pipeline (`surface_face_intersections_*.rs`). Do not attempt to route BREP booleans through `boolmesh`.

**Implementation priority order** (from the adjacency dependency graph):

| Priority | Step | Dependency |
|---|---|---|
| 1 | Adjacency cache rebuild | None |
| 2 | Structural validation (Checks 1–7) | Adjacency |
| 3 | Sewing | Adjacency |
| 4 | Healing passes 1–4 | Structural validation |
| 5 | Constrained tessellation | Sewing |
| 6 | Transaction protocol | All mutations |
| 7 | Collinear edge merge | Sewing |
| 8 | Full geometric validation (Checks 8–10) | Healing |
| 9 | Boolean operations (V2) | SSI pipeline |

**Sewing algorithm (6 phases):**

```
fn brep_sew_faces(faces: &mut [BrepFace], tol: f64):

  Phase 1 — Collect free edge endpoints (3D)
    free_edges: Vec<(face_idx, edge_idx, p_start, p_end, edge_tol)>

  Phase 2 — Spatial bucketing
    Grid or k-d tree over all free-edge midpoints, cell size = 2*tol
    Candidate pairs: edges in same or adjacent cells

  Phase 3 — Pair matching
    for each candidate pair (e1, e2):
      eff_tol = tol + e1.edge_tol + e2.edge_tol  // local tolerance mode
      if dist(e1.p_start, e2.p_start) < eff_tol
         && dist(e1.p_end, e2.p_end) < eff_tol:
        record_pair(e1, e2, same_dir=true)
      elif dist(e1.p_start, e2.p_end) < eff_tol
           && dist(e1.p_end, e2.p_start) < eff_tol:
        record_pair(e1, e2, same_dir=false)  // reversed = normal for adjacent faces

  Phase 4 — Conflict resolution (manifold enforcement)
    Edge with > 1 match: keep best (min gap); flag rest as non-manifold warnings

  Phase 5 — Vertex merging
    For each matched pair: v_merged = midpoint; set BrepVertex.tol = max(tol, gap/2)
    Reassign all trims incident to old vertices

  Phase 6 — Emit shared BrepEdge
    For each pair: create BrepEdge { v_start, v_end, trims: [trim1, trim2] }
    // trim1.reversed XOR trim2.reversed must be true for outward-normal manifold

  Rebuild adjacency cache after all pairs processed.
```

**Canonical validation order (10 checks, cheap → expensive):**

```
// === STRUCTURAL (O(n), no geometry) ===
Check 1: Index bounds — all BrepTrim/Edge/Loop/Face references are valid indices
Check 2: Loop completeness — trims form a closed UV chain; last.end ≈ first.start
Check 3: Loop-face containment — each loop references a face that lists it;
          each face has exactly one outer loop
Check 4: Edge-trim consistency — each edge referenced by 1–2 trims; 0 trims = dangling

// === TOPOLOGICAL (O(n), adjacency traversal) ===
Check 5: Orientation consistency — for each shared edge:
          trim_a.reversed XOR trim_b.reversed must be true
Check 6: Closed-shell check — free edge count must be 0 for closed shells
Check 7: Non-manifold vertex — face star around each vertex must form a single cycle

// === GEOMETRIC (expensive, O(n × eval_cost)) — only on explicit validate call ===
Check 8: Edge-vertex gap — eval curve at t=0/t=1; dist to vertex.point ≤ vertex.tol
Check 9: Trim-edge 3D consistency — sample UV curve → eval surface; compare to curve_3d
Check 10: Outward-normal orientation — ray-cast from sample point; odd crossings = inverted
```

Transaction commit runs only Checks 1–7. Full geometric Checks 8–10 run only on explicit `brep_validate`.

**Healing pipeline (8 passes, run in order — later passes require earlier ones):**

```
Pass 1: Remove short edges (length < tol) — merge v_start/v_end, re-chain loops
Pass 2: Reorder loop trims — ensure consecutive trim connectivity; required before gap closure
Pass 3: Close vertex gaps — snap trim endpoint UVs to common midpoint within heal_tol
Pass 4: Normalize orientation — BFS from seed face; flip adjacent faces to match outward normal
Pass 5: Fix degenerate poles — insert zero-length edge at surface singularities
Pass 6: Fix self-intersections (optional) — detect and shorten overlapping UV trim curves
Pass 7: Remove small faces (optional) — area < tol², or sliver with one dimension < tol
Pass 8: Rebuild caches — adjacency + face AABBs
```

**Gap closure must precede orientation normalization** — you cannot propagate consistent orientation across broken topology.

**Watertight tessellation (constrained boundary pre-discretization):**

```
Phase 1 — Edge pre-discretization (shared boundary skeleton)
  For each BrepEdge:
    if curve_3d exists: adaptive_discretize(curve, linear_tol, angular_tol)
    else: project UV trim samples → eval surface
  Store polyline keyed by EdgeId — same polyline used by BOTH adjacent faces

Phase 2 — Per-face constrained Delaunay in UV
  For each BrepFace:
    Map edge polylines → UV space via surface closest-point or trim parameterization
    Constrained Delaunay triangulation with boundary constraints
    Adaptive interior refinement based on surface curvature

Phase 3 — Assembly with shared vertex welding
  Boundary vertices (from edge polylines): look up global index — shared across faces
  Interior vertices: assign new global index
  Result: watertight triangle soup with no T-junctions at shared edges
```

**Transaction protocol (rollback on validation failure):**

```rust
// elements/brep/transaction.rs
struct BrepTransaction {
    modified_vertices: Vec<(VertexId, BrepVertex)>,  // (index, old value)
    modified_edges:    Vec<(EdgeId,   BrepEdge)>,
    // ... etc for each entity type
    original_vertex_count: usize,
    original_edge_count:   usize,
    // ... etc — for truncating appended entries on rollback
    cache_was_valid: bool,
}

fn apply_atomic<F>(brep: &mut BrepData, op: F) -> Result<(), BrepError>
where F: FnOnce(&mut BrepData) -> Result<(), BrepError>
{
    let txn = begin_transaction(brep);
    match op(brep) {
        Ok(()) => match validate_structural(brep) {
            Ok(()) => { rebuild_adjacency_cache(brep); drop(txn); Ok(()) }
            Err(e) => { rollback(brep, txn); Err(BrepError::ValidationFailed(e)) }
        }
        Err(e) => { rollback(brep, txn); Err(e) }
    }
}
// Rollback: restore modified entries; truncate appended entries; invalidate cache.
// Copy-on-first-write: only save old value on first write during a transaction.
```

**Performance Considerations:**
- Run adjacency-dependent ops using cached edge->trim->face maps from `BrepCache`.
- Rayon threshold for geometry is ~10,000 ns per task: per-face AABB only pays off for N > 64 faces.
- Gate all Rayon parallelism behind `#[cfg(not(target_arch = "wasm32"))]` — see memory notes.

**Edge Cases:**
- Non-manifold vertices (3+ shells incident) should be representable but flagged by validator.
- Nearly coincident edges must use tolerance buckets to avoid flip-flop merges in `brep_sew_faces`.
- `brep_split_edge` must update all trim references to the original edge atomically.

---

## 5) Managing Existing NURBS as BREP

Goal: make existing `CurveData` / `SurfaceData` first-class BREP geometry providers.

Rules:
- Wrap existing handles; do not duplicate NURBS control points unless explicitly requested.
- For trim edges, allow:
  - `curve_3d` handle only,
  - UV polyline only,
  - both (for consistency checking/healing).
- Store per-face surface handle and trim references so existing evaluators remain reusable.

Bridge APIs:
- `rgm_brep_from_face_object(session, face_handle, out_brep)`
- `rgm_brep_extract_face_object(session, brep, face_id, out_face_handle)`
- `rgm_brep_edge_to_nurbs(session, brep, edge_id, tol, out_curve_handle)`

### Research Insights

**Near-zero-copy bridge from `FaceData` to `BrepFace`:**

`TrimEdgeData` already stores `uv_samples: Vec<RgmUv2>` — `BrepTrim` can directly reference or clone this. `FaceData.surface: RgmObjectHandle` maps directly to `BrepFace.surface: RgmObjectHandle`. The bridge is structurally near-zero-copy:

```rust
// elements/brep/bridge.rs
pub(crate) fn brep_from_face(face: &FaceData) -> BrepData {
    let mut brep = BrepData::default();
    let face_id = FaceId::from_raw(0);
    brep.faces.push(BrepFace {
        surface: face.surface,
        loops: SmallVec::new(),
        orientation: 1,
        bbox: None,
    });
    for loop_data in &face.loops {
        let mut trim_ids = SmallVec::<[TrimId; 8]>::new();
        for edge in &loop_data.edges {
            let trim_id = TrimId::from_raw(brep.trims.len() as u32);
            brep.trims.push(BrepTrim {
                edge: EdgeId::from_raw(u32::MAX), // resolved in sew pass
                face: face_id,
                loop_id: LoopId::from_raw(u32::MAX), // filled below
                uv_curve: Trim2dRep::Polyline(edge.uv_samples.clone()),
                reversed: false,
            });
            trim_ids.push(trim_id);
        }
        let loop_id = LoopId::from_raw(brep.loops.len() as u32);
        brep.loops.push(BrepLoop { trims: trim_ids, is_outer: loop_data.is_outer });
        brep.faces[face_id].loops.push(loop_id);
    }
    brep
}
```

**Basis row caching for tessellation** (halves basis function calls on regular UV grids):
```rust
// In elements/face/tessellate.rs (fill existing stub)
for &u in u_samples {
    let u_span  = find_span(&surface.u_knots, u);
    let u_basis = ders_basis_funs(&surface.u_knots, u, u_span, surface.degree_u, 0);
    for &v in v_samples {
        // Only recompute v-direction basis; u_basis reused across entire row
        let v_span  = find_span(&surface.v_knots, v);
        let v_basis = ders_basis_funs(&surface.v_knots, v, v_span, surface.degree_v, 0);
        out.push(eval_with_precomputed(surface, u_span, &u_basis, v_span, &v_basis));
    }
}
```

**Best Practices:**
- Geometry/topology decoupling allows BREP edits while reusing mature NURBS evaluators.
- Keep orientation at trim/use level; same 3D curve may appear with opposite direction in adjacent faces.

**Performance Considerations:**
- Reusing existing curve/surface handles avoids large memory copies and revalidation cost.
- Cache UV-to-3D correspondence per trim for repeated validation/tessellation passes.

---

## 6) I/O Strategy for BREP

### Phase A: Native fast I/O (first)
- Use `bincode 3.0` (serde-compatible, compact binary encoding) as the native binary format.
- Store topology arrays (IndexVec IDs serialized as u32) and geometry handle mapping.
- Separate topology section from geometry section in the binary layout — allows topology traversal (boolean ops, render setup) without touching geometry memory.
- Include magic bytes + version block for forward compatibility.
- **Do NOT use `rkyv`** for cross-platform payloads — see hazard note below.

### Phase B: STEP exchange (second, AP214 import-only initially)
- Target AP214 (`FILE_SCHEMA('automotive_design')`) — the de facto format from all major CAD systems.
- Use `truck-stepio 0.3.0` for import.
- **Do NOT attempt boolean ops on truck-stepio-imported geometry** (open issue #91: `ShapeOpsCurve` trait not implemented for truck-stepio `Curve3D`/`Surface` types).
- In second STEP pass, add `TRIMMED_CURVE` and `COMPOSITE_CURVE` — required for any STEP file from CATIA, NX, or CREO.

### Phase C: AP242 (gated, no IGES)
- **Skip IGES entirely** — no Rust library exists, dying standard, BREP quality poor in practice.
- AP242 must pass conformance fixtures before being advertised as supported.
- For pure BREP geometry, AP242 topology entities are identical to AP214; only the FILE_SCHEMA header changes.

### Research Insights

**`rkyv 0.8` WASM alignment hazard — do NOT use for cross-target exchange:**

`f64` is **4-byte aligned on wasm32** but **8-byte aligned on x86_64**. Archived bytes written by the native kernel have a different binary layout than on WASM. Cross-target rkyv exchange is silently broken. Only use rkyv if you ever need zero-copy access in a **single-platform, native-only** context.

**`bincode 3.0` for native binary format (API changed from 2.x):**
```toml
bincode = { version = "3.0", features = ["serde"] }
```
bincode 3.0 has a **breaking API change** from 2.x: use `encode_to_vec` / `decode_from_slice` (not the old `serialize`/`deserialize`). Compact binary, WASM-safe, forward-compatible via serde `skip`/`default`.

**Versioned envelope pattern:**
```rust
const MAGIC: u32 = 0x52474D42; // "RGMB" = RustedGeom Manifest Binary
const CURRENT_VERSION: u32 = 1;

#[derive(bincode::Encode, bincode::Decode)]
struct BrepFileHeader {
    magic: u32,
    version: u32,
    topology_byte_offset: u64,
    geometry_byte_offset: u64,
}
```

**`truck-stepio 0.3.0` current state:**
- Targets AP214 (`'automotive_design'`), active development through Feb 2026
- Assembly reading added Dec 2025 (`truck-assembly` crate, PR #211)
- Parses: `MANIFOLD_SOLID_BREP`, `ADVANCED_FACE`, `EDGE_LOOP`, `EDGE_CURVE`, `VERTEX_POINT`, `B_SPLINE_SURFACE_WITH_KNOTS`, `B_SPLINE_CURVE_WITH_KNOTS`, `SURFACE_CURVE`, `PCURVE`, all analytical surfaces/curves
- **Missing (add in STEP pass 2):** `TRIMMED_CURVE`, `COMPOSITE_CURVE`, `OFFSET_CURVE_3D`, `OFFSET_SURFACE`, color/material entities

**Minimum viable AP214 STEP entity set (35 entities):**
```
-- Header boilerplate --
PRODUCT, PRODUCT_DEFINITION_FORMATION, PRODUCT_DEFINITION
PRODUCT_DEFINITION_SHAPE, SHAPE_DEFINITION_REPRESENTATION
ADVANCED_BREP_SHAPE_REPRESENTATION, REPRESENTATION_CONTEXT

-- Solid topology --
MANIFOLD_SOLID_BREP, BREP_WITH_VOIDS
CLOSED_SHELL, OPEN_SHELL
ADVANCED_FACE, FACE_OUTER_BOUND, FACE_BOUND
EDGE_LOOP, ORIENTED_EDGE, EDGE_CURVE, VERTEX_POINT

-- Geometry (minimum) --
CARTESIAN_POINT, DIRECTION, AXIS2_PLACEMENT_3D
LINE, CIRCLE
B_SPLINE_CURVE_WITH_KNOTS + RATIONAL_B_SPLINE_CURVE (complex entity)
SURFACE_CURVE, PCURVE, DEFINITIONAL_REPRESENTATION
PLANE, CYLINDRICAL_SURFACE, CONICAL_SURFACE, SPHERICAL_SURFACE, TOROIDAL_SURFACE
B_SPLINE_SURFACE_WITH_KNOTS + RATIONAL_B_SPLINE_SURFACE (complex entity)

-- Add in STEP pass 2 (CATIA/NX/CREO required) --
TRIMMED_CURVE, COMPOSITE_CURVE, SEAM_CURVE
SURFACE_OF_LINEAR_EXTRUSION, SURFACE_OF_REVOLUTION
```

**STEP orientation semantics — three layered flags, most common source of bugs:**

```
EDGE_CURVE.same_sense    = whether the 3D curve parameter runs start→end (.T.) or reversed (.F.)
ORIENTED_EDGE.orientation = whether this edge use goes start→end (.T.) or end→start (.F.)
ADVANCED_FACE.same_sense  = whether the surface normal agrees with the required face normal
```

Validate all three flags on every imported edge. OCCT issue trackers and truck issue #77 both document that `same_sense` inversion is the #1 source of incorrect face normals on import.

**STEP round-trip fidelity hazards:**
1. **Knot normalization** — normalize knot vectors to [0,1] on export; import stores as-is but normalize before computation
2. **Rational weight handling** — STEP stores Euclidean control points + separate weight list; some CATIA versions write homogeneous (weight-multiplied) points with all weights=1.0 (wrong on import)
3. **Periodic seam edges** — closed NURBS curves require `SEAM_CURVE`; truck's handling of periodic boundary conditions is noted as fragile
4. **Vertex gap healing** — STEP files from different systems have vertices differing by 1e-4 to 1e-3; a vertex-snap healing pass is required after import to produce manifold topology
5. **Non-manifold assembly edges** — some assemblies share interface faces across solids producing non-manifold edges; truck's `Solid::try_new` rejects these; must detect and handle
6. **Same-sense orientation inversion** — see above; validate all three flags

**Critical: `ruststep` is NOT production-ready:**
The README states verbatim: _"This project is still in experimental stage. DO NOT USE FOR PRODUCT."_ Last substantive commit: 2025-03-19. Do not use. truck-stepio uses ruststep internally as its parser — you should use truck-stepio as the interface, not ruststep directly.

**Alternative to study:** `cadk 0.1.0` (2025-07-11) — CAD kernel in pure Rust with B-Rep, CSG, tessellation, AP203 support. Worth watching for STEP parsing approach.

**Revised I/O dependency graph:**
```
Phase A: bincode 3.0 native binary (topology + handle mapping, versioned envelope)
Phase B: truck-stepio import-only AP214 (no booleans; add TRIMMED_CURVE in pass 2)
Phase C: AP242 gated — not for production until conformance fixtures pass; no IGES
```

**References:**
- STEP AP242 overview: [nist.gov](https://www.nist.gov/services-resources/software/step-file-analyzer-and-viewer)
- OCCT STEP translator docs: [dev.opencascade.org](https://dev.opencascade.org/doc/occt-7.0.0/overview/html/occt_user_guides__step.html)
- `truck-stepio` crate docs: [docs.rs](https://docs.rs/truck-stepio/latest/truck_stepio/)
- `ruststep` crate docs (experimental, do not use for prod): [docs.rs](https://docs.rs/ruststep/latest/ruststep/)
- OCCT ShapeFix/ShapeHealing guide: [dev.opencascade.org](https://dev.opencascade.org/doc/overview/html/occt_user_guides__shape_healing.html)
- OCCT Boolean Operations specification: [dev.opencascade.org](https://dev.opencascade.org/doc/overview/html/specification__boolean_operations.html)

---

## 7) Performance Architecture and Targets

### Data/Locking
- Keep `SessionState` object lookup for compatibility, but store `BrepData` internals in contiguous `IndexVec` arrays.
- Replace global session mutex with per-session RW lock sharded by `DashMap`.

### Caching
- Mandatory caches:
  - face AABB (stored on `BrepFace.bbox: Option<Aabb3>`, rebuilt lazily),
  - edge bbox and length,
  - vertex weld buckets,
  - adjacency (vertex->edges, edge->trims, face->neighbors) via `Vec<SmallVec>` indexed by entity ID.
- Rebuild incrementally after local edits when possible; `cache.invalidate_topology()` on every structural mutation.

### Parallelism
- Use Rayon for:
  - per-face bbox/tess prep (N > 64 faces),
  - validation passes over independent faces/loops,
  - import parsing post-processing.
- All Rayon usage gated behind `#[cfg(not(target_arch = "wasm32"))]` per project convention.

### Benchmark Plan
Add `crates/kernel-ffi/benches/brep_bench.rs` with groups:
- `create_brep_from_100_trimmed_faces`
- `sew_5k_edges`
- `validate_1k_face_shell`
- `tessellate_500_face_brep`
- `load_save_native_brep_50mb`
- `step_import_roundtrip_fixture`

### Performance Gates (initial)
- BREP creation (100 trimmed faces): p50 < 20 ms, p95 < 35 ms.
- Sew 5k edges: p50 < 60 ms.
- Validate 1k-face shell: p50 < 40 ms.
- Native load/save 50 MB: each < 150 ms.
- Existing surface/curve intersection bench regressions: <= 5%.

### Research Insights

**Session locking — highest-impact, lowest-effort improvement:**

```toml
dashmap = "6.0"
parking_lot = "0.12"
```

```rust
// Replace in session/store.rs:
// Before: pub(crate) static SESSIONS: Lazy<Mutex<HashMap<u64, SessionState>>>;
// After:
pub(crate) static SESSIONS: Lazy<DashMap<u64, parking_lot::RwLock<SessionState>>> =
    Lazy::new(DashMap::new);
```

`DashMap` shards by default into 64 buckets — two threads on different sessions no longer block each other. This eliminates cross-session lock contention under parallel workloads. Use `parking_lot::RwLock` for fine-grained per-session read/write separation.

**Dedicated kernel Rayon pool (WASM-gated):**
```rust
#[cfg(not(target_arch = "wasm32"))]
static KERNEL_POOL: Lazy<rayon::ThreadPool> = Lazy::new(|| {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get().min(8))
        .thread_name(|i| format!("rusted-geom-{}", i))
        .build()
        .expect("kernel thread pool init failed")
});
```

**Criterion benchmark setup (Phase 0):**
```toml
# crates/kernel-ffi/Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
codspeed-criterion-compat = "2.7"

[[bench]]
name = "brep_bench"
harness = false
```

Use `iter_batched` (not `iter`) for mutation-consuming operations to avoid measuring setup cost:
```rust
b.iter_batched(
    || fixture_brep_100_trimmed_faces(),   // setup, not timed
    |brep| brep_sew_faces(brep, 1e-6),    // measured
    BatchSize::LargeInput,
)
```

**CI regression gating:** Use CodSpeed for deterministic instruction-count measurement — eliminates CI noise false positives. Free for open-source. Add to GitHub Actions after first benchmark baseline is established.

**Rayon task granularity:** ~10,000 ns per task is the crossover. Per-face AABB recompute only pays off for N > 64 faces. Below that, sequential iteration is faster. Profile before parallelizing.

---

## 8) Implementation Phases and File Map

### Phase 0: Scaffolding and benchmarks
- Add benchmark harness and fixtures.
- **No `benches/` directory exists** — must create from scratch.
- Files:
  - `crates/kernel-ffi/benches/brep_bench.rs` (new)
  - `crates/kernel-ffi/Cargo.toml` (add `[[bench]]` section, `dev-dependencies`, new `[dependencies]`: `index_vec`, `smallvec`, `bon`, `ahash`, `dashmap`, `parking_lot`, `bincode`)

### Phase 1: Core BREP object model
- Add `BrepData` and topology types with `index_vec` typed IDs.
- Add `BrepInProgress` and `Brep` variants to `GeometryObject`.
- Files:
  - `crates/kernel-ffi/src/session/objects.rs` (add `Brep(BrepData)` and `BrepInProgress(BrepData)` variants)
  - `crates/kernel-ffi/src/session/store.rs` (add DashMap locking)
  - `crates/kernel-ffi/src/kernel_impl/foundation.rs` (new FFI types: `RgmBrepValidationReport`, `RgmValidationIssue`, `RgmValidationSeverity`)
  - `crates/kernel-ffi/src/elements/brep/ids.rs` (new — `define_index_type!` declarations)
  - `crates/kernel-ffi/src/elements/brep/types.rs` (new — `BrepData`, `BrepVertex`, `BrepEdge`, `BrepTrim`, `BrepLoop`, `BrepFace`, `BrepShell`, `BrepSolid`, `BrepCache`)

### Phase 2: ABI and runtime wiring
- Add exports and impl stubs.
- Add generated binding updates.
- Files:
  - `crates/kernel-ffi/src/ffi/exports/brep.rs` (new)
  - `crates/kernel-ffi/src/ffi/exports/mod.rs`
  - `crates/kernel-ffi/src/kernel_impl/ffi_impl.rs`
  - `bindings/web/src/runtime/session/brep.ts` (new)
  - `bindings/web/src/runtime/session/index.ts`
  - `bindings/web/src/runtime/session/core.ts`
  - `bindings/web/src/runtime/session/handles.ts` (add BrepHandle, BrepFaceId, BrepEdgeId, BrepLoopId, BrepShellId)

### Phase 3: Creation + validation/healing
- Implement constructors, loop/edge ingest, validator reports, heal passes.
- Run validation checks in canonical order (Checks 1–7 on commit, Checks 8–10 on explicit call).
- **Fill existing stubs** in `elements/face/` — do NOT create new files for these:
  - `crates/kernel-ffi/src/elements/face/heal.rs` (fill: passes 1–4 of healing pipeline)
  - `crates/kernel-ffi/src/elements/face/loops.rs` (fill: loop reorder, chain validation)
  - `crates/kernel-ffi/src/elements/face/tessellate.rs` (fill: constrained tessellation + basis row cache)
  - `crates/kernel-ffi/src/elements/face/validate.rs` (fill: Checks 1–10)
  - `crates/kernel-ffi/src/elements/brep/validator.rs` (new — `BrepData::validate()`)
  - `crates/kernel-ffi/src/elements/brep/transaction.rs` (new — `apply_atomic`, `rollback`)
  - `crates/kernel-ffi/src/elements/brep/cache.rs` (new — `BrepCache` rebuild methods)
  - `crates/kernel-ffi/src/kernel_impl/ffi_impl.rs`
  - `crates/kernel-ffi/src/tests/kernel_ffi.rs` (BREP integration tests)

### Phase 4: BREP operations
- Implement sew/split/merge/adjacency/tessellate following the algorithm pseudocode in Section 4.
- Files:
  - `crates/kernel-ffi/src/ops/brep/sew.rs` (new — 6-phase sewing)
  - `crates/kernel-ffi/src/ops/brep/heal.rs` (new — 8-pass healing pipeline)
  - `crates/kernel-ffi/src/ops/brep/tessellate.rs` (new — watertight tessellation)
  - `crates/kernel-ffi/src/ops/brep/adjacency.rs` (new — adjacency cache build)
  - `crates/kernel-ffi/src/ops/brep/mod.rs` (new)
  - `crates/kernel-ffi/src/elements/brep/`
  - `crates/kernel-ffi/src/tests/kernel_ffi.rs`

### Phase 5: NURBS bridges
- Wrap existing surface/curve/face handles into BREP and back.
- Files:
  - `crates/kernel-ffi/src/elements/brep/bridge.rs` (new)
  - `crates/kernel-ffi/src/kernel_impl/ffi_impl.rs`
  - `bindings/web/src/runtime/session/brep.ts`

### Phase 6: I/O
- Native binary format first (`bincode 3.0` with versioned envelope), STEP import-only next.
- Files:
  - `crates/kernel-ffi/src/io/brep_native.rs` (new — bincode 3.0 encode/decode)
  - `crates/kernel-ffi/src/io/step.rs` (new — truck-stepio AP214 import only, no boolean ops; document TRIMMED_CURVE gap)
  - `crates/kernel-ffi/src/ffi/exports/brep.rs`
  - `bindings/web/src/runtime/session/brep.ts`

### Phase 7: ABI baseline and docs
- Regenerate bindings and ABI baseline.
- Update architecture docs and examples.
- Files:
  - `abi/baseline/rgm_abi.json`
  - `bindings/web/src/generated/*`
  - `docs/architecture/kernel-module-map.md`

---

## 9) Acceptance Criteria

Correctness:
- Can create BREP from:
  - empty,
  - existing face handles,
  - existing surfaces + loops.
- Validator runs checks in canonical order (1–7 structural before 8–10 geometric).
- Validator catches: open loops, dangling trims, non-matching orientations, invalid handle/session references.
- Heal pipeline runs passes in dependency order (reorder before gap closure before orientation normalization).
- Heal can fix at least: tiny gaps within tolerance, reversed loop winding, duplicate coincident vertices, short degenerate edges.
- `RgmBrepValidationReport` returns up to 16 issues inline (stack-allocated); `overflow=true` when more exist.

NURBS integration:
- Existing curve/surface evaluation APIs remain unchanged.
- BREP round-trip (`face -> brep -> face`) preserves geometry and trim topology within tolerance.
- `FaceData` -> `BrepData` bridge is near-zero-copy (no control point duplication).

I/O:
- `bincode 3.0` native format round-trip is lossless for topology and tolerance data.
- STEP AP214 import (`truck-stepio`) passes fixture comparisons for the 35-entity minimum viable set.
- Orientation flags validated on all imported edges (`EDGE_CURVE.same_sense`, `ORIENTED_EDGE.orientation`, `ADVANCED_FACE.same_sense`).
- Knot vectors normalized to [0,1] on export.
- AP242, IGES, and TRIMMED_CURVE/COMPOSITE_CURVE remain explicitly gated — not advertised until second STEP pass passes conformance fixtures.

Performance:
- Meets gates in Section 7.
- No >5% regression in existing intersection benchmarks.
- Rayon parallelism only enabled for N > 64 faces; sequential path for smaller BREPs.

ABI/runtime:
- All new APIs are additive and ABI baseline updated intentionally.
- TS runtime exposes BREP client with typed handles (`BrepHandle` as `bigint`; sub-entity IDs as branded `number`/u32) and error propagation parity.
- No `usize` in any FFI-exposed BREP struct (use `u32` throughout).

---

## 10) Risks and Mitigations

Risk: Topology mutation complexity leads to invalid states.
Mitigation: enforce `apply_atomic` transactions with structural validation (Checks 1–7) on commit and rollback on failure; `index_vec` typed IDs prevent mixing VertexId/EdgeId/etc. at compile time.

Risk: Memory growth from adjacency/caches on large models.
Mitigation: lazy cache materialization via `Option<Vec<...>>`; explicit `cache.invalidate_topology()` API; `BrepFace.bbox: Option<Aabb3>` rebuilt only before spatial queries.

Risk: STEP interoperability mismatch across CAD vendors.
Mitigation: start with 35-entity minimum viable set + golden fixtures from multiple exporters; `truck-stepio` import-only until boolean gap (issue #91) resolved; AP242 gated behind conformance test suite.

Risk: STEP orientation flag inversion on import.
Mitigation: validate all three flags (`EDGE_CURVE.same_sense`, `ORIENTED_EDGE.orientation`, `ADVANCED_FACE.same_sense`) on every imported edge; run orientation-consistency check (Check 5) immediately after STEP import before any operations; add fixture from CATIA/NX/CREO export.

Risk: TRIMMED_CURVE/COMPOSITE_CURVE gaps blocking real-world STEP files.
Mitigation: document the gap prominently in `io/step.rs`; return structured error `RgmStatus::NotImplemented` with entity type code for unsupported entities rather than silently skipping; add to STEP pass 2 scope.

Risk: Lock contention under parallel workloads.
Mitigation: replace global `Mutex<HashMap>` with `DashMap<u64, RwLock<SessionState>>`; per-session fine-grained locks; benchmark lock hold-time explicitly; Rayon pool isolated from global pool.

Risk: Hidden tolerance drift between curve/surface/trim pipelines.
Mitigation: centralize tolerance policy and include `param_u`/`param_v` location in `RgmValidationIssue`; run vertex-gap healing pass after STEP import (vertex positions may differ by 1e-4 to 1e-3 across vendor files).

Risk: `ruststep` chosen for STEP I/O despite "DO NOT USE FOR PRODUCT" warning.
Mitigation: do NOT use `ruststep` directly; use `truck-stepio` (which wraps ruststep internally); document the decision in `io/step.rs`; re-evaluate `cadk` as an alternative when its AP203 support matures.

Risk: `rkyv` used for cross-target binary payloads causing silent data corruption.
Mitigation: do NOT use `rkyv` for any format that crosses the native/wasm32 boundary; f64 alignment difference (8-byte native vs 4-byte wasm32) makes rkyv archived bytes non-portable; use `bincode 3.0` for all persistence and cross-target exchange.

Risk: WASM ABI breakage from `usize` mismatches.
Mitigation: audit all FFI-exposed structs at Phase 2; enforce `u32` for counts and entity indices; add CI check via `cbindgen` or ABI diff tooling.

Risk: `index_vec` IDs become stale after removals in incremental edit operations.
Mitigation: for V1 (append-only BREP construction), IDs are always valid; for V2 (boolean ops with deletion), upgrade to `slotmap` primary storage at that point — migration path is mechanical since `IndexVec`/`SlotMap` share the typed-key API shape.

---

## References

- openNURBS BREP class and topology conventions: [https://developer.rhino3d.com/api/cpp/class_o_n___brep.html](https://developer.rhino3d.com/api/cpp/class_o_n___brep.html)
- Open CASCADE modeling data/topology guide: [https://dev.opencascade.org/doc/overview/html/occt_user_guides__modeling_data.html](https://dev.opencascade.org/doc/overview/html/occt_user_guides__modeling_data.html)
- Open CASCADE STEP translator guide: [https://dev.opencascade.org/doc/occt-7.0.0/overview/html/occt_user_guides__step.html](https://dev.opencascade.org/doc/occt-7.0.0/overview/html/occt_user_guides__step.html)
- Open CASCADE ShapeHealing guide: [https://dev.opencascade.org/doc/overview/html/occt_user_guides__shape_healing.html](https://dev.opencascade.org/doc/overview/html/occt_user_guides__shape_healing.html)
- Open CASCADE Boolean Operations specification: [https://dev.opencascade.org/doc/overview/html/specification__boolean_operations.html](https://dev.opencascade.org/doc/overview/html/specification__boolean_operations.html)
- NIST STEP analyzer (AP242 context): [https://www.nist.gov/services-resources/software/step-file-analyzer-and-viewer](https://www.nist.gov/services-resources/software/step-file-analyzer-and-viewer)
- `truck-stepio` crate docs: [https://docs.rs/truck-stepio/latest/truck_stepio/](https://docs.rs/truck-stepio/latest/truck_stepio/)
- `ruststep` crate docs (experimental — DO NOT USE FOR PRODUCT): [https://docs.rs/ruststep/latest/ruststep/](https://docs.rs/ruststep/latest/ruststep/)
- `index_vec` crate docs: [https://docs.rs/index_vec/latest/index_vec/](https://docs.rs/index_vec/latest/index_vec/)
- `bon` typestate builders: [https://docs.rs/bon/latest/bon/](https://docs.rs/bon/latest/bon/)
- `slotmap` crate docs: [https://docs.rs/slotmap/latest/slotmap/](https://docs.rs/slotmap/latest/slotmap/)
- `bincode` 3.0 crate docs: [https://docs.rs/bincode/latest/bincode/](https://docs.rs/bincode/latest/bincode/)
- `postcard` crate docs: [https://docs.rs/postcard/latest/postcard/](https://docs.rs/postcard/latest/postcard/)
- `dashmap` crate docs: [https://docs.rs/dashmap/latest/dashmap/](https://docs.rs/dashmap/latest/dashmap/)
- `cadk` crate (emerging alternative): [https://github.com/davefol/cadk](https://github.com/davefol/cadk)
- truck-topology Arc model: [https://docs.rs/truck-topology/latest/truck_topology/](https://docs.rs/truck-topology/latest/truck_topology/)
