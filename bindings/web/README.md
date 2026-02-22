# @rusted-geom/bindings-web

TypeScript and WASM bindings for the RustedGeom CAD kernel.

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
```
