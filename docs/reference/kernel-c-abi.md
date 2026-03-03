# Kernel C ABI Reference

This is the stable interop reference for native/managed consumers.

## Core Lifecycle

- `rgm_kernel_create(out_session)`
- `rgm_kernel_destroy(session)`
- `rgm_object_release(session, object)`
- `rgm_last_error_code(session, out_code)`
- `rgm_last_error_message(session, out_buf, buf_capacity, out_written)`

## Conventions

- All functions return `RgmStatus`.
- Objects are opaque handles (`RgmObjectHandle`).
- Inputs/outputs crossing boundaries are `#[repr(C)]` types.
- Variable-length output uses two-pass flow:
  - call with null/zero capacity to get required count
  - allocate and call again to copy.

## Domain Groups

- Curve: `rgm_curve_*`, `rgm_nurbs_interpolate_fit_points`
- Surface: `rgm_surface_*`
- Mesh: `rgm_mesh_*`
- Face: `rgm_face_*`
- Intersection: `rgm_intersect_*`, `rgm_intersection_*`
- B-rep: `rgm_brep_*`
- Bounds: `rgm_object_compute_bounds`
- Memory helpers: `rgm_alloc`, `rgm_dealloc`

## Breaking-Change Rules

Treat these as ABI-breaking:

- function name/signature changes
- enum discriminant changes
- `#[repr(C)]` field changes/order changes
- ownership contract changes
