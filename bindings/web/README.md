# @rusted-geom/bindings-web

TypeScript and WASM bindings for the RustedGeom CAD kernel.

## Alpha API Shape (`0.1.0-alpha.1`)

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
import {
  createKernelRuntime,
  type CurvePresetInput,
} from "@rusted-geom/bindings-web";
import wasmUrl from "@rusted-geom/bindings-web/wasm/rusted_geom.wasm";

const runtime = await createKernelRuntime(wasmUrl);
const session = runtime.createSession();

const preset: CurvePresetInput = {
  degree: 2,
  closed: false,
  points: [
    { x: 0, y: 0, z: 0 },
    { x: 1, y: 0.2, z: 0 },
    { x: 2, y: 1.0, z: 0 },
    { x: 3, y: 1.1, z: 0 },
  ],
  tolerance: { abs_tol: 1e-9, rel_tol: 1e-9, angle_tol: 1e-9 },
};

const curve = session.curve.buildCurveFromPreset(preset);
const point = session.curve.curvePointAt(curve, 0.5);
const length = session.curve.curveLength(curve);

console.log(point, length);

session.kernel.releaseObject(curve);
session.destroy();
runtime.destroy();
```

## Flat-to-domain migration quick map

If your code previously called methods directly on `session`, move them into domain clients:

| Flat session call | Domain-scoped call |
| --- | --- |
| `session.createMeshBox(...)` | `session.mesh.createMeshBox(...)` |
| `session.curvePointAt(...)` | `session.curve.curvePointAt(...)` |
| `session.createNurbsSurface(...)` | `session.surface.createNurbsSurface(...)` |
| `session.createFaceFromSurface(...)` | `session.face.createFaceFromSurface(...)` |
| `session.intersectSurfacePlane(...)` | `session.intersection.intersectSurfacePlane(...)` |
| `session.releaseObject(...)` | `session.kernel.releaseObject(...)` |
