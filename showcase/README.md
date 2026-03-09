# Rusted Geom Showcase

Next.js 16 app that visualizes Rusted Geom kernel output through wasm using a full-page Three.js viewer with custom toolbar, inspector, and console panels.

## Dev Workflow

```bash
pnpm install
pnpm --dir showcase wasm:build
pnpm --dir showcase dev
```

### Fast iteration scripts

| Script | What it does |
|--------|-------------|
| `pnpm dev` | Full rebuild — WASM + bindings + Next.js dev server |
| `pnpm dev:fast` | Skip WASM rebuild — just `next dev` (use when editing UI only) |
| `pnpm dev:bindings` | Rebuild TypeScript bindings then start dev server |

For WASM hot-reload during Rust iteration, run in a second terminal:

```bash
./scripts/watch_kernel.sh   # cargo watch → wasm-pack → stage
```

Then use `pnpm dev:fast` in your main terminal to avoid redundant rebuilds.

## Shareable Example URLs

Every active example is reflected in the URL via `?example=<key>`:

```
http://localhost:3000/?example=surfaceIntersectSurface
```

Navigating to a URL with a valid `?example=` param loads that example automatically. Share links to any specific example directly.

## Runtime Contract

- The viewer does not generate geometry in UI components.
- Curves are built by kernel calls (`nurbsInterpolateFitPointsPtrTol`) and sampled by kernel calls (`curvePointAt`).
- Presets are external JSON data files in `showcase/public/showcases/`.
- Session save/load serializes preset + camera/view state.

## WASM Handle Lifecycle

WASM object handles are tracked via `HandleRegistry` (`src/lib/handle-registry.ts`). Every `session.*_create_*` call should go through `registry.track(handle)`. Cleanup on example switch calls `registry.release()`, which calls `.free()` on all tracked handles, preventing WASM memory leaks.

## Example Notes

- `Mesh (CSG difference: box - torus)` demonstrates constructive solid geometry.
- CSG operation shown: `result = A - B`, where `A` is a box and `B` is an offset torus that intersects the box wall.
- Select either source solid in the Gizmo target dropdown (`A` or `B`) and drag to move/rotate/scale it.
- On drag commit, the viewer recomputes the boolean result from kernel state and refreshes the result mesh.

## IGES Status

IGES toolbar actions are visible but disabled in v1.
They are gated on kernel capabilities and will be enabled once kernel IGES APIs are implemented.

## Test

```bash
pnpm --dir showcase test:unit
pnpm --dir showcase test:e2e
```
