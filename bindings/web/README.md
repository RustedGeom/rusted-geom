# @rustedgeom/kernel

TypeScript and WASM bindings for the RustedGeom geometry kernel (`0.1.0`).

## Install

The package is published to [GitHub Packages](https://github.com/RustedGeom/rusted-geom/packages). Configure your project to use the `@rustedgeom` scope from the GitHub registry:

**1. Create or update `.npmrc` in your project root:**

```
@rustedgeom:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}
```

> You need a GitHub Personal Access Token with `read:packages` scope. Set it as the `GITHUB_TOKEN` environment variable, or replace `${GITHUB_TOKEN}` with the token directly.

**2. Install the package:**

```bash
npm install @rustedgeom/kernel
# or
pnpm add @rustedgeom/kernel
```

## Build from source

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

## Quick start

```ts
import { loadKernel, KernelSession } from "@rustedgeom/kernel";

await loadKernel("/wasm/rusted_geom.wasm");
const session = new KernelSession();

// NURBS curve via fit-point interpolation
const curve = session.interpolate_nurbs_fit_points(
  new Float64Array([0, 0, 0, 1, 0.25, 0, 2, 1, 0, 3, 1.25, 0]),
  2,     // degree
  false, // closed
);

const [x, y, z] = session.curve_point_at(curve, 0.5);
const length = session.curve_length(curve);

curve.free();
session.free();
```

## API overview

All methods live directly on `KernelSession`. Handles are lightweight wrappers
returned from creation methods and freed automatically on `Drop` or explicitly
via `.free()`.

### Curves

```ts
const line = session.create_line(0, 0, 0, 1, 0, 0);
const circle = session.create_circle(0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 5.0);
const arc = session.create_arc(0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 3.0, 0, Math.PI);
const polyline = session.create_polyline(new Float64Array([0,0,0, 1,1,0, 2,0,0]), false);
const nurbs = session.interpolate_nurbs_fit_points(pts, 3, false);

session.curve_point_at(nurbs, 0.5);    // [x, y, z]
session.curve_tangent_at(nurbs, 0.5);  // [tx, ty, tz]
session.curve_length(nurbs);           // total arc length
session.curve_length_at(nurbs, 0.5);   // arc length to parameter
```

### Surfaces

```ts
const surface = session.create_nurbs_surface(
  2, 2, 3, 3, false, false,
  controlPoints, weights, knotsU, knotsV,
);

session.surface_point_at(surface, 0.5, 0.5);  // [x, y, z]
session.surface_frame_at(surface, 0.5, 0.5);  // { px, py, pz, du_x, ... nx, ny, nz }
const mesh = session.surface_tessellate_to_mesh(surface, tessOptions);
```

### Meshes

```ts
const box = session.create_box_mesh(0, 0, 0, 4, 3, 2);
const torus = session.create_torus_mesh(0, 0, 0, 3.0, 1.0, 64, 48);
const sphere = session.create_uv_sphere_mesh(0, 0, 0, 5.0, 32, 24);

session.mesh_vertex_count(box);
session.mesh_triangle_count(box);
session.mesh_copy_vertices(box);   // Float64Array [x,y,z,...]
session.mesh_copy_indices(box);    // Uint32Array [i0,i1,i2,...]

const diff = session.mesh_boolean(box, torus, 2); // 0=union, 1=intersect, 2=difference
```

### B-rep

```ts
const face = session.create_face_from_surface(surface);
session.face_add_loop(face, new Float64Array(uvs), true);
session.face_validate(face);
const faceMesh = session.face_tessellate_to_mesh(face, tessParams);

const brep = session.brep_create_empty();
session.brep_add_face(brep, face);
session.brep_finalize_shell(brep);
session.brep_validate(brep);
const brepMesh = session.brep_tessellate_to_mesh(brep, tessParams);
```

### Intersections

```ts
session.intersect_curve_curve(lineA, lineB);       // Float64Array [x,y,z,...]
session.intersect_curve_plane(curve, planeFlat);    // Float64Array
session.intersect_surface_plane(surface, plane);    // IntersectionHandle
session.intersect_surface_surface(surfA, surfB);    // IntersectionHandle

const count = session.intersection_branch_count(handle);
const summary = session.intersection_branch_summary(handle, 0);
const points = session.intersection_branch_copy_points(handle, 0);
```

### Bounding volumes

```ts
// mode: 0=Fast (control-point hull), 1=Optimal (PCA + OBB)
const bounds = session.compute_bounds(handle.object_id(), 0, 0, 0.0);
// bounds.aabb_min_x, bounds.aabb_max_x, ...
// bounds.obb_cx, bounds.obb_hx, ... (Optimal mode)
```

### LandXML

```ts
const doc = session.landxml_parse(xmlText, 1, 0, 0); // mode, order, normalize
session.landxml_surface_count(doc);
session.landxml_alignment_count(doc);
const verts = session.landxml_surface_copy_vertices(doc, 0);
const indices = session.landxml_surface_copy_indices(doc, 0);
```

### CAD export (IGES / SAT — curves, surfaces, B-reps)

```ts
const iges = session.export_iges(new Float64Array([curve.object_id(), surface.object_id()]));
const sat = session.export_sat(new Float64Array([brep.object_id()]));
```

### Mesh export (STL / glTF)

```ts
const stl = session.export_stl(new Float64Array([mesh.object_id()]));
const gltf = session.export_gltf(new Float64Array([mesh.object_id()]));
```

## Session lifecycle

- `new KernelSession()` creates an isolated session with its own object store.
- `session.abs_tol()`, `session.rel_tol()`, `session.angle_tol()` query tolerances.
- `session.set_abs_tol(v)`, etc. to configure.
- `session.last_error()` retrieves the last kernel error string.
- `session.free()` tears down the session and all owned objects.

## License

[MIT](../../LICENSE)
