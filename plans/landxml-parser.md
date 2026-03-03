# Plan: LandXML Parser Integration

## Context

A production-grade LandXML 1.2 parser exists at `/Users/cesarecaoduro/GitHub/geom-wasm/geom-forge/src/landxml/`. It handles horizontal alignments (Line/Arc/15 spiral types), vertical profiles (parabolic/circular curves, sampled), TIN terrain surfaces, and station equations. The goal is to port it into `crates/kernel/`, expose it via WASM bindings, and add 3 showcase examples. LandXML files use UTM/survey coordinates (hundreds of thousands of units) which breaks the current camera setup (`far=1200`, fog `near=34`/`far=138`) and causes Float32 GPU precision loss at this scale.

---

## Phase 1 — Rust: Port parser into kernel

### 1.1 Add dependency
**File:** `crates/kernel/Cargo.toml`
```toml
roxmltree = "0.20"
```

### 1.2 Declare module
**File:** `crates/kernel/src/lib.rs` — add `mod landxml;` alongside existing module declarations.

### 1.3 Create `crates/kernel/src/landxml/` module tree

Port all 10 source files. Use a proper `mod` (not `include!`) — the landxml code has its own internal `use` graph and must stay isolated from the flat `kernel_impl` scope.

**Mechanical changes for every file:**
- Replace `use crate::math::Vec3;` → `use crate::RgmPoint3 as Vec3;`
- Replace `Vec3::new(x, y, z)` → `RgmPoint3 { x, y, z }` (no `new()` constructor exists)
- Strip all `use serde::{Deserialize, Serialize};`, `#[derive(Serialize, Deserialize)]`, `#[serde(...)]`

**File-specific changes:**

| File | Changes beyond mechanical |
|---|---|
| `error.rs` | Verbatim copy — no Vec3, no serde in error types |
| `types.rs` | Strip serde; replace `Vec3` field types with `RgmPoint3`; add `UnsymParaCurve` improvement (see below); add `make_point3(x,y,z) -> RgmPoint3` shim |
| `parser.rs` | Remove `use crate::mesh::MeshData` (terrain.rs handles this); apply `UnsymParaCurve` fallback |
| `terrain.rs` | Rewrite: remove `MeshData` dep (conversion moves to WASM layer); keep query helpers `terrain_vertex_count`, `terrain_triangle_count` |
| `horizontal.rs`, `spiral.rs`, `vertical.rs`, `alignment3d.rs`, `station.rs` | Mechanical changes only |

**`UnsymParaCurve` fix** — implement the full asymmetric parabolic formula:

Add to `types.rs`:
```rust
// In VerticalControlCurve enum:
AsymmetricParabola { length_in_m: f64, length_out_m: f64 },

// New VerticalCurveInterval variant:
AsymmetricParabola(AsymmetricParabolaVerticalCurve),

pub struct AsymmetricParabolaVerticalCurve {
    pub s_bvc: f64,   // BVC station = s_pvi - length_in
    pub s_pvi: f64,   // PVI station
    pub s_evc: f64,   // EVC station = s_pvi + length_out
    pub z_bvc: f64,   // elevation at BVC
    pub z_pvi: f64,   // elevation at PVI
    pub g0: f64,      // incoming grade
    pub g_mid: f64,   // grade at PVI: 2*(z_pvi - z_bvc)/L_in - g0
    pub g1: f64,      // outgoing grade
}
```

In `vertical.rs`, evaluate with piecewise quadratic:
```
// First half [s_bvc, s_pvi]:  r1 = (g_mid - g0) / L_in
//   z(s) = z_bvc + g0*(s-s_bvc) + 0.5*r1*(s-s_bvc)^2
// Second half [s_pvi, s_evc]: r2 = (g1 - g_mid) / L_out
//   z(s) = z_pvi + g_mid*(s-s_pvi) + 0.5*r2*(s-s_pvi)^2
```

In `build_designed_model`, when the control curve is `AsymmetricParabola { length_in_m, length_out_m }`:
- Compute z_bvc = z_pvi - g_incoming * length_in_m
- Compute g_mid = 2*(z_pvi - z_bvc)/length_in_m - g_incoming
- Emit `VerticalCurveInterval::AsymmetricParabola(...)` covering [s_bvc, s_evc]

In `parser.rs`, parse `UnsymParaCurve` in both Strict and Lenient modes (hard rejection removed). Push a warning in Lenient mode only.

**`landxml/mod.rs`** — re-export with `pub(crate)`:
```rust
pub(crate) use types::*;
pub(crate) use error::LandXmlError;
pub(crate) use parser::parse_landxml;
pub(crate) use alignment3d::{sample_alignment_3d};
pub(crate) use horizontal::evaluate_alignment_2d;
pub(crate) use station::{display_to_internal_station, internal_to_display_station};
pub(crate) use vertical::{evaluate_vertical_model, sample_vertical_model};
```

---

## Phase 2 — Rust: Session object extension

**File:** `crates/kernel/src/session/objects.rs`

Add `LandXmlDocData` struct and extend `GeometryObject`:
```rust
pub(crate) struct LandXmlDocData {
    pub(crate) doc: crate::landxml::LandXmlDocument,
}

pub(crate) enum GeometryObject {
    // ... existing variants ...
    LandXmlDoc(LandXmlDocData),  // NEW
}
```

Add `find_landxml_doc` helper (same pattern as `find_mesh`).

**File:** `crates/kernel/src/session/store.rs` — add `insert_landxml_doc`.

---

## Phase 3 — Rust: WASM bindings

### 3.1 Handle definition
**File:** `crates/kernel/src/wasm/mod.rs`
- Add `mod landxml;`
- Add `define_handle!(LandXmlDocHandle, "Handle to a parsed LandXML document.");`
- Add `pub use landxml::LandXmlDocHandle;`

### 3.2 Error helper
**File:** `crates/kernel/src/wasm/error.rs` — add `pub(crate) fn js_err(s: RgmStatus) -> JsValue`.

### 3.3 New file `crates/kernel/src/wasm/landxml.rs`

Methods on `KernelSession` (all follow the existing mesh.rs pattern):

| Method | Signature | Notes |
|---|---|---|
| `landxml_parse` | `(xml: &str, mode: u32, point_order: u32, units_policy: u32) -> Result<LandXmlDocHandle, JsValue>` | mode 0=Strict/1=Lenient; point_order 0=NEZ/1=ENZ/2=EZN; policy 0=normalize/1=preserve |
| `landxml_surface_count` | `(doc: &LandXmlDocHandle) -> Result<u32, JsValue>` | |
| `landxml_surface_name` | `(doc: &LandXmlDocHandle, index: u32) -> Result<String, JsValue>` | |
| `landxml_surface_copy_vertices` | `(doc: &LandXmlDocHandle, index: u32) -> Result<Vec<f64>, JsValue>` | flat [x,y,z,...] raw UTM f64 — TS does centroid subtract |
| `landxml_surface_copy_indices` | `(doc: &LandXmlDocHandle, index: u32) -> Result<Vec<u32>, JsValue>` | flat [i0,i1,i2,...] |
| `landxml_alignment_count` | `(doc: &LandXmlDocHandle) -> Result<u32, JsValue>` | |
| `landxml_alignment_name` | `(doc: &LandXmlDocHandle, index: u32) -> Result<String, JsValue>` | |
| `landxml_sample_alignment` | `(doc: &LandXmlDocHandle, alignment_index: u32, n_steps: u32) -> Result<Vec<f64>, JsValue>` | flat [x,y,z,...]; uses first profile if available, falls back to 2D horizontal |
| `landxml_sample_all_alignments` | `(doc: &LandXmlDocHandle, n_steps: u32) -> Result<Vec<f64>, JsValue>` | packed: `[count, n0, ...pts0, n1, ...pts1, ...]` — single call for all alignments |
| `landxml_warning_count` | `(doc: &LandXmlDocHandle) -> Result<u32, JsValue>` | |
| `landxml_linear_unit` | `(doc: &LandXmlDocHandle) -> Result<String, JsValue>` | e.g. "meter" or "USSurveyFoot" |

Also add `landxml_extract_surface_mesh(doc, index) -> Result<MeshHandle, JsValue>` — creates a proper `MeshData` in the session from the TIN, enabling future kernel-side operations (mesh-plane section, boolean, bounds) on LandXML terrain. The raw `copy_vertices` / `copy_indices` methods remain for TypeScript-side centroid localization before GPU upload.

---

## Phase 4 — Showcase: File staging

Copy **all 21** files from `docs/landxml-test-files/` to `showcase/public/landxml/`. No Next.js config changes needed.

---

## Phase 5 — Showcase: Viewer changes

**Single ExampleKey** — one entry `"landxmlViewer"` (not 3 hardcoded files). The user picks which file to load from a dropdown rendered inside the inspector panel.

**File:** `showcase/src/lib/viewer-types.ts`
- Extend `ExampleKey` union with `"landxmlViewer"`
- Export `type LandXmlExampleKey = "landxmlViewer"`
- Export `LANDXML_FILE_LIST: readonly string[]` — all 21 filenames (e.g. `"12DExample.xml"`)

**File:** `showcase/src/lib/examples.ts`
- Add 1 entry to `EXAMPLE_OPTIONS`, `EXAMPLE_SUMMARIES`, `parseExampleSelection`
- Add new category `{ label: "LandXML", key: "landxml", items: [{ key: "landxmlViewer", label: "LandXML File Viewer" }] }`

**Component state addition** in `kernel-viewer.tsx`:
```typescript
const [activeLandXmlFile, setActiveLandXmlFile] = useState<string>("12DExample.xml");
```
Whenever `activeLandXmlFile` changes while `activeExample === "landxmlViewer"`, re-trigger `updateExampleAsync("landxmlViewer")`.

**Inspector panel** — when `activeExample === "landxmlViewer"`, render a `<select>` dropdown in the inspector (alongside existing sections) populated from `LANDXML_FILE_LIST`. Changing the selection calls `setActiveLandXmlFile(value)` which triggers a reload.

**File:** `showcase/src/components/kernel-viewer.tsx`

#### A. `fitViewToLargeScene` (new top-level function alongside `fitViewToPoints`)
```typescript
function fitViewToLargeScene(
  camera: THREE.PerspectiveCamera,
  controls: OrbitControls,
  fog: THREE.Fog | null,
  points: THREE.Vector3[],
): void {
  if (points.length === 0) return;
  const bounds = new THREE.Box3();
  for (const p of points) bounds.expandByPoint(p);
  const sphere = bounds.getBoundingSphere(new THREE.Sphere());
  const r = Math.max(0.1, sphere.radius);

  const distance = Math.max(4, r * 2.8);
  camera.position.set(
    sphere.center.x + distance,
    sphere.center.y + distance * 0.55,
    sphere.center.z + distance,
  );
  controls.target.copy(sphere.center);

  // Dynamic near/far — scales from small kernel objects to UTM terrain
  camera.near = Math.max(0.001, r / 1000);
  camera.far  = Math.max(1200, r * 20);
  camera.updateProjectionMatrix();

  // Scale fog to scene — disable for very large scenes (r > 5000)
  if (fog) {
    fog.near = r > 5000 ? 1e9 : Math.max(34, r * 0.5);
    fog.far  = r > 5000 ? 1e9 : Math.max(138, r * 3.5);
  }

  controls.update();
}
```

#### B. Async loading strategy

`buildExampleCurve` is a synchronous `useCallback`. LandXML examples need `fetch()`. **Strategy: minimal parallel async path** — no refactor to the existing sync path.

1. Add `pendingAsyncExampleRef = useRef<AbortController | null>(null)` to the component.
2. Add top-level helper:
```typescript
function isAsyncExample(key: ExampleKey): key is LandXmlExampleKey {
  return key === "landxmlViewer";
}
```
3. Add top-level async builder `buildLandXmlExample(session, filename, signal)`:
   - `fetch("/landxml/" + filename, { signal })`
   - `session.landxml_parse(xmlText, 1 /*Lenient*/, 0 /*NEZ*/, 0 /*normalize*/)`
   - Compute a shared centroid from the first available surface (or first alignment point cloud) in f64
   - If surfaces present: call `landxml_surface_copy_vertices/copy_indices`, subtract centroid → `MeshVisual`
   - If alignments present: call `landxml_sample_all_alignments(doc, 200)`, parse packed format, subtract same centroid → `OverlayCurveVisual[]` (one per alignment, cycling colors)
   - Log parse stats: surface count, alignment count, vertex count, warnings, parse time
   - Return a `BuiltExample`
4. Add `updateLandXmlFile(filename: string)` useCallback — sets `kernelStatus="computing"`, calls `buildLandXmlExample(session, filename, signal)`, applies result to state setters, calls `fitViewToLargeScene` in a rAF
5. In `onExampleSelectionChange` / `onExampleBrowserSelect`: when `isAsyncExample(next)` → call `updateLandXmlFile(activeLandXmlFile)` instead of `updateCurveForExample`
6. When `activeLandXmlFile` changes and `activeExample === "landxmlViewer"` → call `updateLandXmlFile(newFile)` in a `useEffect`

#### C. Coordinate localization (UTM → local origin)
For all LandXML geometry: compute centroid as mean of all vertex positions in f64, subtract before writing to `MeshVisual.vertices` or `OverlayCurveVisual.points`. This keeps values in the ±5000 range, safe for Float32 GPU buffers. The centroid is computed in TypeScript after extracting raw f64 data from the kernel.

#### D. LandXML inspector section
When `activeExample === "landxmlViewer"`, render a `LandXmlSection` sub-component in the inspector (after `ExampleSection`, before `GizmoSection`):
```
┌─ LandXML File ─────────────────────────────────────┐
│  [dropdown: 12DExample.xml ▾]                      │
│  Surfaces: 1 · Alignments: 3 · Vertices: 12,847    │
│  Units: meter · Warnings: 0 · Parsed in 42ms       │
└─────────────────────────────────────────────────────┘
```
State: `activeLandXmlFile`, `landXmlStats: { surfCount, alignCount, vertCount, unit, warnCount, parseMs } | null`.

---

## Implementation Orchestration

Work is decomposed into 4 parallel agents launched together, followed by a build + verify pass:

**Wave 1 — launch all 4 in parallel:**

| Agent | Scope | Key files |
|---|---|---|
| **A: Rust parser port** | Create `crates/kernel/src/landxml/` (all 10 .rs files). Adapt Vec3→RgmPoint3, strip serde, implement full UnsymParaCurve formula. | `landxml/mod.rs`, `types.rs`, `parser.rs`, `horizontal.rs`, `spiral.rs`, `station.rs`, `vertical.rs`, `alignment3d.rs`, `terrain.rs`, `error.rs` |
| **B: Rust kernel integration** | Wire landxml module into the kernel: Cargo.toml dep, lib.rs declaration, session/objects.rs extension, session/store.rs, wasm/mod.rs handle + module, wasm/error.rs helper, new wasm/landxml.rs bindings | `Cargo.toml`, `lib.rs`, `session/objects.rs`, `session/store.rs`, `wasm/mod.rs`, `wasm/error.rs`, `wasm/landxml.rs` |
| **C: Showcase types + registry** | Stage all 21 XML files, update type/example registry files (no viewer logic) | `showcase/public/landxml/*.xml`, `viewer-types.ts`, `examples.ts` |
| **D: Showcase viewer** | Implement `fitViewToLargeScene`, `buildLandXmlExample`, `updateLandXmlFile`, `LandXmlSection` panel, async routing in `kernel-viewer.tsx`; add `activeLandXmlFile`/`landXmlStats` state | `kernel-viewer.tsx` |

Agents A and B must agree on the `crate::landxml::*` public API surface before writing. The agreed interface: `parse_landxml`, `LandXmlDocument`, `LandXmlParseOptions`, `sample_alignment_3d`, `evaluate_alignment_2d`, `RgmPoint3 as Vec3`. Agent B writes imports against this contract; Agent A delivers the matching implementations.

**Wave 2 — sequential after Wave 1 reports back:**
1. `cargo build -p kernel` — fix any cross-agent integration errors
2. `cargo test -p kernel`
3. `./scripts/build_kernel_wasm.sh`
4. `pnpm --dir showcase dev` — smoke-test all verification items

---

## Phase 6 — Build & Verify

```bash
# 1. Rust compile check (fast, no WASM)
cargo build -p kernel

# 2. Rust tests
cargo test -p kernel

# 3. WASM build
./scripts/build_kernel_wasm.sh

# 4. Dev server
pnpm --dir showcase dev
```

**Manual verification checklist:**
- [ ] Existing NURBS example still loads (regression check)
- [ ] New "LandXML" category appears in example browser with one item "LandXML File Viewer"
- [ ] Selecting it loads `12DExample.xml` by default; spinner shows while fetching/parsing
- [ ] Inspector shows LandXML panel with dropdown listing all 21 filenames and parse stats
- [ ] Switching to `OpenRoadTin.xml` (1.3 MB, imperial): TIN mesh renders without z-fighting; camera auto-fits; no fog clipping
- [ ] Switching to `C3DDesignExample.xml`: both terrain mesh and alignment overlays appear with shared centroid
- [ ] Files with alignments only (no TIN) render polylines; files with TIN only render mesh
- [ ] `camera.near`/`camera.far` adjust proportionally to scene scale (verify in console)
- [ ] Switching back to a non-LandXML example works cleanly (no leftover handles/state)

---

## Files to Create / Modify

### New files
| Path | Purpose |
|---|---|
| `crates/kernel/src/landxml/mod.rs` | Module root |
| `crates/kernel/src/landxml/error.rs` | Verbatim from source |
| `crates/kernel/src/landxml/types.rs` | Adapted: no serde, RgmPoint3, UnsymParaCurve fix |
| `crates/kernel/src/landxml/parser.rs` | Vec3→RgmPoint3, UnsymParaCurve fallback |
| `crates/kernel/src/landxml/horizontal.rs` | Vec3→RgmPoint3 |
| `crates/kernel/src/landxml/spiral.rs` | Vec3→RgmPoint3 |
| `crates/kernel/src/landxml/station.rs` | Verbatim |
| `crates/kernel/src/landxml/vertical.rs` | Vec3→RgmPoint3 |
| `crates/kernel/src/landxml/alignment3d.rs` | Vec3→RgmPoint3 |
| `crates/kernel/src/landxml/terrain.rs` | Rewritten (no MeshData dep) |
| `crates/kernel/src/wasm/landxml.rs` | WASM bindings |
| `showcase/public/landxml/*.xml` | All 21 test files staged from `docs/landxml-test-files/` |

### Modified files
| Path | Change |
|---|---|
| `crates/kernel/Cargo.toml` | Add `roxmltree = "0.20"` |
| `crates/kernel/src/lib.rs` | Add `mod landxml;` |
| `crates/kernel/src/session/objects.rs` | Add `LandXmlDocData`, `GeometryObject::LandXmlDoc`, `find_landxml_doc` |
| `crates/kernel/src/session/store.rs` | Add `insert_landxml_doc` |
| `crates/kernel/src/wasm/mod.rs` | Add `mod landxml`, `define_handle!(LandXmlDocHandle, ...)`, `pub use landxml::LandXmlDocHandle` |
| `crates/kernel/src/wasm/error.rs` | Add `pub(crate) fn js_err` |
| `showcase/src/lib/viewer-types.ts` | Add `"landxmlViewer"` to `ExampleKey`, export `LandXmlExampleKey`, `LANDXML_FILE_LIST` |
| `showcase/src/lib/examples.ts` | Add 1 option/summary/category entry, update parseExampleSelection |
| `showcase/src/components/kernel-viewer.tsx` | Add `fitViewToLargeScene`, `buildLandXmlExample`, `updateLandXmlFile`, `LandXmlSection` inspector panel, `activeLandXmlFile`/`landXmlStats` state, async routing |
