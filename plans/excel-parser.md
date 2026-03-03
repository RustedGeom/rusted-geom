# Plan: Bridge Data Panel — Excel-like UI integrated into the showcase viewer

## Context

The BAE PCG v1.8.10 workbook (`BAE_PCG_v1_8_10_EX_13_Valhalla.xlsm`) is a bridge design input system that structures geometry/material data for export to CSI Bridge, PGSuper, MIDAS, and VBent. The goal is to surface this business logic inside the rusted-geom showcase so engineers can edit bridge parameters in an Excel-like panel and immediately see the geometry update in the 3D viewer.

The user confirmed: **live formula recomputation** on edit + **actual .xlsm file import**.

---

## Excel File Structure (source of truth)

| Sheet | Type | Rows × Cols | Key content |
|---|---|---|---|
| `Model_Setup` | Input | 120 × 38 | Bridge name, units, start coords, bearing |
| `Substructure` | Input | 55 × 65 | Pier stations, skew, columns, section IDs |
| `Superstructure` | Input | 64 × 55 | Span girder count, deck offsets, overhangs |
| `Loading` | Input | 64 × 84 | Barrier weights, dead loads |
| `User-Defined_Rdwy_Geometry` | Input | 99 × 45 | Horizontal + vertical alignment curves |
| `REFs` | Reference | 100 × 40 | Unit conversions, rebar specs |
| `SUB_OUT`, `SUPER_OUT`, `RDWY_OUT`, `LOAD_OUT`, `SETUP_OUT` | Output | — | JSON-exportable arrays |

Data rows start at row 14 in Substructure/Superstructure; rows 10–13 are merged multi-row headers.

---

## Architecture

### New files

```
showcase/src/
├── lib/
│   ├── bridge-types.ts              # TypeScript data model for all bridge data
│   └── bridge-xlsx-parser.ts        # FileReader + DOMParser xlsm parser
├── components/viewer/bridge/
│   ├── BridgeDataPanel.tsx          # Outer shell: tabs, resize, open/close animation
│   ├── BridgeSheetGrid.tsx          # Virtualized scrollable grid
│   ├── BridgeCellEditor.tsx         # Inline <input> for editable cells
│   ├── BridgeSheetTabs.tsx          # Tab strip (sheet names)
│   ├── BridgeResizeHandle.tsx       # Drag-to-resize top strip
│   └── bridge-geometry.ts           # Pure TS: bridge data → WASM geometry calls
```

### Modified files

- `src/components/kernel-viewer.tsx` — add bridge state + effect + JSX
- `src/components/viewer/toolbar/ViewerToolbar.tsx` — add bridge panel toggle button
- `src/app/globals.css` — add bridge panel + cell CSS classes

---

## Data Model (`bridge-types.ts`)

```typescript
type BridgeUnitSystem = "imperial" | "metric";
type BridgeCellKind = "input" | "optional" | "computed" | "readonly";

interface BridgeModelSetup { bridgeName, unitSystem, startCoord, bearingDeg, ... }
interface BridgePier { rowIndex, station, skewDeg, columnCount, sectionId, ... }
interface BridgeSpan { rowIndex, girderCount, leftOverhang, rightOverhang, deckThickness, ... }
interface BridgeHCurve { station, radius, length, direction: "L"|"R"|"T" }
interface BridgeVCurve { pviStation, pviElevation, gradeIn, gradeOut, length }

interface SheetCell { row, col, value, kind: BridgeCellKind, format? }
interface BridgeSheet { name, displayName, cells: Map<"R{r}C{c}", SheetCell>, headerRows, dataStartRow }
interface BridgeWorkbook { setup, piers, spans, hcurves, vcurves, sheets, isDirty }
```

Cell coordinates-to-float conversion happens at the geometry layer, not at the model layer.

---

## Panel UI Design

The panel is a **full-width bottom drawer** (position: absolute, bottom) that opens upward — same animation pattern as `KernelConsole` (`translateY`). Default height 320px, draggable 180–70vh.

```
┌────────────────────── drag resize handle (4px) ──────────────────────────┐
│ [Model Setup] [Substructure] [Superstructure] [Loading] [Roadway]  [...] [×]│
│               ──────────────  active tab underline (--accent)             │
├───────────────────────────────────────────────────────────────────────────┤
│  # │ Station(ft) │ Skew(°) │ Col cnt │ Section ID │ Cap ID │ …            │
│    │             │         │         │            │        │   ← sticky   │
│ 14 │  [1200.0]   │ [24.4]  │   [2]   │ C2         │ CAP2   │  (blue=input)│
│ 15 │  [1480.5]   │ [24.4]  │   [2]   │ C2         │ CAP2   │              │
│ 16 │   1680.2    │  24.4   │    2    │ C2         │ CAP2   │  (grey=comp) │
│    └─────────── overflow: auto, virtualised rows ─────────────────────────┘
```

**Sticky rows**: column header row `position: sticky; top: 0`.
**Sticky col**: row-number column `position: sticky; left: 0`.
**No external grid library** — CSS flexbox with absolute-positioned virtual rows.

**Cell color coding** (matching Excel convention):
- Blue (`--accent-soft` bg, `--accent` border on focus): required inputs
- Yellow (`rgba(220,165,20,0.08)` bg): optional inputs
- Grey (`--panel-soft` bg, `--ink-soft` text): computed/derived values (read-only)

---

## Formula Reimplementation (minimal set, `bridge-geometry.ts`)

Only the 8 functions needed to build 3D geometry:

1. `bearingToDir(deg)` → `{dx, dy}` — bearing angle to world direction
2. `stationToXY(sta, startE, startN, bearingDeg, hcurves)` → `{x, y}` — walks hcurve array, accumulates bearing through arcs (`L/R` radians per arc)
3. `elevationAtStation(sta, vcurves)` → `z` — vertical profile parabola
4. `stationToXYZ(sta, workbook)` → `RgmPoint3` — composes 2+3
5. `hcurveToArcPoints(hc, startPt, entryBearing)` → `{start, mid, end, exitBearing}` — 3-point arc for WASM
6. `pierCapPoints(pier, workbook)` → `{center, leftEnd, rightEnd}` — cap beam endpoints from skew angle
7. `deckEdgeAtPier(span, pier, workbook)` → `{left, right}` — deck edge control points for NURBS surface
8. `resolveSectionGeometry(sectionId, setupSheet)` → `{diam, capWidth, height}` — section lookup

**Coordinate origin offset**: Valhalla E/N coordinates are in State Plane feet (~2.35M, ~6.93M). Subtract the bridge start point before passing to Three.js to avoid float precision jitter.

---

## 3D Geometry Layers (WASM calls)

| Layer | WASM calls | Visual |
|---|---|---|
| Roadway alignment | `create_line` + `create_arc_by_3_points` → `create_polycurve` | Blue overlay curve |
| Pier/abutment sticks | `create_polyline` per pier (cap beam + column lines) | White lines |
| Deck ribbon | `create_nurbs_surface` (degree 1×1, 2 rows × N cols control grid) → `brep_tessellate_to_mesh` | Light blue mesh, 45% opacity |
| Column boxes | `create_box_mesh` per column | Dark grey meshes |

Bridge geometry lives in its own `THREE.Group` (`bridgeGroupRef`) — cleared and rebuilt on workbook change. Independent from the active example, so switching examples does not clear the bridge overlay.

---

## `.xlsm` Parser (`bridge-xlsx-parser.ts`)

The `.xlsm` is a ZIP containing `xl/worksheets/sheet*.xml`. Strategy:

1. `FileReader.readAsArrayBuffer` → hand off to a minimal ZIP reader (use `DecompressionStream` API, available in all modern browsers — no library)
2. Parse the XML of each relevant sheet with `DOMParser`
3. Resolve shared strings from `xl/sharedStrings.xml`
4. Map each `<c r="A1" t="s">` cell to `SheetCell` with `kind` inferred from cell address (rows 1–13 = header/readonly; rows 14+ = input unless formula present)
5. Formulas (`<f>` tag present) → `kind = "computed"`, value = cached `<v>` from the file
6. Return `BridgeWorkbook` populated from the five input sheets

A "Load .xlsm" button in the panel header triggers a hidden `<input type="file" accept=".xlsm,.xlsx">`.

---

## Integration into `kernel-viewer.tsx`

### New state (add after `isConsoleOpen`)
```typescript
const [isBridgePanelOpen, setIsBridgePanelOpen] = useState(false);
const [bridgeWorkbook, setBridgeWorkbook] = useState<BridgeWorkbook | null>(null);
const [bridgePanelHeight, setBridgePanelHeight] = useState(320);
const ownedBridgeHandlesRef = useRef<AnyHandle[]>([]);
const bridgeGroupRef = useRef<THREE.Group | null>(null);
```

### New effect
```typescript
useEffect(() => {
  // free old handles, clear bridgeGroupRef from scene
  // if workbook !== null: buildBridgeGeometry(session, workbook) → add to bridgeGroupRef
  // camera.zoomExtents includes bridge bounds
}, [bridgeWorkbook]);
```

### New JSX (after `<KernelConsole>`)
```tsx
<BridgeDataPanel
  isOpen={isBridgePanelOpen}
  height={bridgePanelHeight}
  onHeightChange={setBridgePanelHeight}
  workbook={bridgeWorkbook}
  onWorkbookChange={setBridgeWorkbook}
  onClose={() => setIsBridgePanelOpen(false)}
/>
```

### Keyboard shortcut: `B` → toggle bridge panel

### Toolbar: new button in "Panels" group (grid-icon SVG), between inspector and console toggles

---

## CSS additions to `globals.css`

```css
/* Bridge panel */
--cell-input-bg: var(--accent-soft);
--cell-optional-bg: rgba(220, 165, 20, 0.08);
--cell-computed-bg: var(--panel-soft);
--grid-cell-height: 22px;
--grid-row-num-width: 44px;

.bridge-panel { position: absolute; bottom: ...; z-index: 30; ... }
.bridge-panel.is-collapsed { transform: translateY(calc(100% + 1rem)); }
.bridge-tab.is-active { border-bottom: 2px solid var(--accent); }
.bridge-cell--input { background: var(--cell-input-bg); }
.bridge-cell--optional { background: var(--cell-optional-bg); }
.bridge-cell--computed { background: var(--cell-computed-bg); color: var(--ink-soft); }
/* ... etc */
```

Z-index: bridge panel = 30, console = 32 (console floats above).

---

## Implementation Order

1. **`bridge-types.ts`** — data model + Valhalla fixture constant
2. **`bridge-geometry.ts`** — 8 formula functions (pure TS, no WASM yet); verify Valhalla pier positions match expected coordinates
3. **`bridge-geometry.ts`** (extend) — `buildBridgeGeometry(session, workbook)` with all WASM calls
4. **`BridgeDataPanel.tsx` + CSS** — shell only: resize handle, header, tabs (no grid), open/close animation
5. **`ViewerToolbar.tsx` + `kernel-viewer.tsx`** — toolbar button, state, toggle, keyboard shortcut `B`
6. **`BridgeSheetGrid.tsx` + `BridgeCellEditor.tsx`** — virtualised grid renderer; cell edits → `onWorkbookChange`
7. **`kernel-viewer.tsx` effect** — wire geometry rebuild on workbook change; bridge THREE.Group management
8. **`bridge-xlsx-parser.ts`** — FileReader + DOMParser xlsm import; "Load .xlsm" button in panel header

---

## Verification

- **Unit tests**: `bridge-geometry.ts` functions — `stationToXYZ` on Valhalla start point should return (0, 0, 733.85) in local coords
- **Visual check**: Load Valhalla fixture → alignment curve traces SSE bearing; pier sticks appear at correct station intervals
- **Edit test**: Change pier 1 station from 1200 to 1300 ft → stick moves in 3D view within one render frame
- **Import test**: Load `BAE_PCG_v1_8_10_EX_13_Valhalla.xlsm` → panel populates all 5 tabs with correct data, 3D geometry appears
- **Theme test**: Toggle dark/light — all cell colors update via CSS variables
- **Resize test**: Drag panel height from 180px to 500px; grid scrolls correctly; Three.js canvas not affected
