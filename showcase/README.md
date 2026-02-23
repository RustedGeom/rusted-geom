# Rusted Geom Showcase

Next.js 16 app that visualizes Rusted Geom kernel output through wasm using a full-page Three.js viewer with custom toolbar, inspector, and console panels.

## Dev Workflow

```bash
pnpm install
pnpm --dir showcase wasm:build
pnpm --dir showcase dev
```

## Runtime Contract

- The viewer does not generate geometry in UI components.
- Curves are built by kernel calls (`nurbsInterpolateFitPointsPtrTol`) and sampled by kernel calls (`curvePointAt`).
- Presets are external JSON data files in `showcase/public/showcases/`.
- Session save/load serializes preset + camera/view state.

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
