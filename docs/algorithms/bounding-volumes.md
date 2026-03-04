# Bounding Volume Computation

Implementation reference for AABB and OBB computation in
`crates/kernel/src/math/bounds.rs` and `crates/kernel/src/kernel_impl/bounds_ops.rs`.

## Overview

rusted-geom supports two bounding-volume modes accessible via `compute_bounds`:

| Mode | Enum | Description |
|------|------|-------------|
| Fast | `RgmBoundsMode::Fast` (0) | Control-point hull. Uses the convex hull of control points (curves/surfaces) or mesh vertices. No surface evaluation needed. |
| Optimal | `RgmBoundsMode::Optimal` (1) | PCA-based OBB with surface sampling. Evaluates the geometry at `sample_budget` points and computes a tighter oriented bounding box. |

## Fast Mode

1. Collect the defining points of the object:
   - Curves: NURBS control points (or polyline vertices).
   - Surfaces: control point grid.
   - Meshes: vertex positions (after transform).
   - B-rep: union of face tessellation vertices.

2. Compute the axis-aligned bounding box (AABB) from min/max of each coordinate.

3. The OBB is set to the AABB (axis-aligned) since no PCA is performed.

4. Apply uniform `padding` to all box dimensions.

Results are cached per `(object_id, mode, sample_budget)` to avoid recomputation.

## Optimal Mode

1. Sample the geometry at `sample_budget` uniformly distributed points:
   - Curves: uniform parameter samples.
   - Surfaces: uniform (u,v) grid samples.
   - Meshes: vertex positions.

2. Compute the AABB from the sampled points.

3. Compute the oriented bounding box (OBB) using PCA:
   a. Calculate the centroid of all sample points.
   b. Build the 3x3 covariance matrix from centered points.
   c. Perform eigenvalue decomposition to extract principal axes.
   d. The three eigenvectors become the OBB local axes (x, y, z).
   e. Project all points onto the local axes to determine half-extents.

4. The OBB center is the midpoint of the projected min/max along each axis.

5. A local-frame AABB is also provided (the AABB in OBB coordinates), which is
   useful for containment testing without world-space rotation.

## Output (`Bounds3`)

| Field Group | Description |
|-------------|-------------|
| `aabb_min/max` | World-space axis-aligned bounding box |
| `obb_center` | OBB center in world space |
| `obb_half` | OBB half-extents along local axes |
| `obb_ax/ay/az` | OBB local axis directions (unit vectors) |
| `local_aabb_min/max` | AABB in the OBB coordinate frame |

## Usage

```ts
// Fast mode -- no sampling, uses control points
const fast = session.compute_bounds(handle.object_id(), 0, 0, 0.0);

// Optimal mode -- 2048 samples, 0.1 padding
const optimal = session.compute_bounds(handle.object_id(), 1, 2048, 0.1);
```
