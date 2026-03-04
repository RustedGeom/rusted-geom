# Surface Tessellation

Implementation reference for the adaptive surface tessellation strategy in
`crates/kernel/src/kernel_impl/`.

## Overview

Surface tessellation converts a NURBS surface (or trimmed face) into a triangle mesh
suitable for rendering or downstream mesh operations. The tessellation respects both
geometric accuracy (chord tolerance) and visual smoothness (normal angle tolerance).

## Parameters

The tessellation is controlled by `RgmSurfaceTessellationOptions`:

| Parameter | Description |
|-----------|-------------|
| `min_u_segments` | Minimum subdivisions in the u direction |
| `min_v_segments` | Minimum subdivisions in the v direction |
| `max_u_segments` | Maximum subdivisions in the u direction |
| `max_v_segments` | Maximum subdivisions in the v direction |
| `chord_tol` | Maximum allowed chord-to-surface distance |
| `normal_tol_rad` | Maximum allowed angle between adjacent face normals (radians) |

## Algorithm

1. **Initial grid**: The surface is sampled on a uniform (u,v) grid with at least
   `min_u_segments x min_v_segments` cells. Each cell is split into two triangles.

2. **Chord refinement**: For each triangle, the midpoints of its edges are evaluated on
   the surface. If the distance between the triangle edge midpoint and the true surface
   point exceeds `chord_tol`, the edge is subdivided. Refinement continues recursively
   up to `max_u_segments x max_v_segments`.

3. **Normal refinement**: Surface normals at triangle vertices are compared. If the angle
   between adjacent normals exceeds `normal_tol_rad`, the shared edge is subdivided to
   improve shading quality.

4. **Trimmed faces**: When tessellating a `FaceHandle`, the trim loops are projected into
   the (u,v) parameter space. Triangles that fall outside the outer loop or inside inner
   (hole) loops are discarded. Triangles that straddle a trim boundary are clipped and
   re-triangulated.

## Scale-Aware Tolerance

For surfaces with large world-space extent, the chord tolerance is automatically scaled
relative to the characteristic surface size (`surface_world_scale`). This prevents
over-tessellation of large, nearly flat regions while maintaining detail on curved areas.

## Output

The result is an indexed triangle mesh (`MeshHandle`) with:

- Vertices: 3D world-space positions evaluated from the surface.
- Indices: triangle connectivity (counter-clockwise winding).
