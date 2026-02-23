# @rusted-geom/bindings-web

TypeScript and WASM bindings for the RustedGeom CAD kernel.

## v5 API Shape

The session API is domain-scoped:
- `session.kernel.*`
- `session.curve.*`
- `session.mesh.*`
- `session.surface.*`
- `session.face.*`
- `session.intersection.*`

## Build

```bash
npm install
npm run build
```

Or from repo root:

```bash
./scripts/pack_web.sh
```

This produces:
- `dist/**/*.js`
- `dist/**/*.d.ts`
- `dist/wasm/rusted_geom.wasm`

## Runtime loading

You can load the packaged wasm from:
- `@rusted-geom/bindings-web/wasm/rusted_geom.wasm`

Example:

```ts
import { createKernelRuntime } from "@rusted-geom/bindings-web";
import wasmUrl from "@rusted-geom/bindings-web/wasm/rusted_geom.wasm";

const runtime = await createKernelRuntime(wasmUrl);
const session = runtime.createSession();

const curve = session.curve.buildCurveFromPreset({
  degree: 3,
  closed: false,
  points: [
    { x: 0, y: 0, z: 0 },
    { x: 1, y: 0.2, z: 0 },
    { x: 2, y: 1.0, z: 0 },
    { x: 3, y: 1.1, z: 0 },
  ],
  tolerance: { abs_tol: 1e-9, rel_tol: 1e-9, angle_tol: 1e-9 },
});

const p = session.curve.pointAt(curve, 0.5);
const len = session.curve.curveLength(curve);
console.log(p, len);
```

## v4 -> v5 migration quick map

| v4 | v5 |
| --- | --- |
| `session.createLine(...)` | `session.curve.createLine(...)` |
| `session.createMeshBox(...)` | `session.mesh.createMeshBox(...)` |
| `session.createNurbsSurface(...)` | `session.surface.createNurbsSurface(...)` |
| `session.createFaceFromSurface(...)` | `session.face.createFaceFromSurface(...)` |
| `session.intersectCurveCurve(...)` | `session.intersection.intersectCurveCurve(...)` |
| `session.releaseObject(...)` | `session.kernel.releaseObject(...)` |
