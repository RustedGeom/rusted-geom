# Kernel WASM API Reference

This document describes the `wasm-bindgen` API exposed from `src/wasm`.

## Entry Point

- `KernelSession` is the primary runtime object.
- Construct with `new KernelSession()`.
- Handles are object wrappers (curve/surface/mesh/face/intersection/brep/landxml).

## Session Behavior

- Session owns all created objects.
- Handles auto-release on `Drop` and support explicit `.free()` from JS.
- Session stores the latest error message retrievable through `last_error()`.

## API Families

- Curves: create/evaluate/length/frame queries
- Surfaces: create/evaluate/frame/tessellation
- Meshes: create/query/transform/boolean/intersections
- Faces: trimmed-face creation, validation, tessellation
- Intersections: curve/surface/mesh intersection queries
- B-rep: creation/assembly/validation/healing/tessellation/native IO
- Bounds: object bounds and OBB/AABB extraction
- LandXML: parse, surface/alignment sampling, surface mesh extraction

## Data Shapes

- Point arrays are flat `[x, y, z, ...]`.
- UV arrays are flat `[u, v, ...]`.
- Plane arrays are `[ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]`.
- Most failures surface as thrown JS errors (`JsValue`).
