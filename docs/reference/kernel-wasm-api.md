# Kernel WASM API Reference

Complete reference for the `wasm-bindgen` API exposed by the `@rustedgeom/kernel` package.

## Entry Point

- `KernelSession` is the primary runtime object.
- Construct with `new KernelSession()` after calling `loadKernel(wasmUrl)`.
- Handles are lightweight object wrappers that auto-release on GC or explicit `.free()`.

## Session

| Method | Returns | Description |
|--------|---------|-------------|
| `new KernelSession()` | `KernelSession` | Create session with default tolerances (abs=1e-6, rel=1e-4, angle=1e-6 rad) |
| `abs_tol()` | `f64` | Absolute distance tolerance |
| `rel_tol()` | `f64` | Relative distance tolerance |
| `angle_tol()` | `f64` | Angular tolerance (radians) |
| `set_abs_tol(v)` | `void` | Set absolute tolerance |
| `set_rel_tol(v)` | `void` | Set relative tolerance |
| `set_angle_tol(v)` | `void` | Set angular tolerance |
| `last_error()` | `String` | Last kernel error message |

## Handle Types

All handles expose `.object_id() -> f64` and `.free()`.

| Handle | Description |
|--------|-------------|
| `CurveHandle` | Curve (line, circle, arc, polyline, polycurve, NURBS) |
| `SurfaceHandle` | NURBS surface |
| `MeshHandle` | Triangle mesh |
| `FaceHandle` | Trimmed face (surface + trim loops) |
| `IntersectionHandle` | Intersection result with branches |
| `BrepHandle` | B-rep solid (faces, shells, solids) |
| `LandXmlDocHandle` | Parsed LandXML document |

## Curves

### Constructors

| Method | Parameters | Returns |
|--------|-----------|---------|
| `create_line` | `x0, y0, z0, x1, y1, z1` | `CurveHandle` |
| `create_circle` | `origin_xyz, x_axis_xyz, y_axis_xyz, z_axis_xyz, radius` (13 args) | `CurveHandle` |
| `create_arc` | `origin_xyz, x_axis_xyz, y_axis_xyz, z_axis_xyz, radius, start_angle, sweep_angle` (15 args) | `CurveHandle` |
| `create_arc_by_angles` | `origin_xyz, axes_xyz, radius, start_angle, end_angle` (15 args) | `CurveHandle` |
| `create_arc_by_3_points` | `x0,y0,z0, x1,y1,z1, x2,y2,z2` | `CurveHandle` |
| `create_polyline` | `points: Vec<f64>` (flat xyz), `closed: bool` | `CurveHandle` |
| `create_polycurve` | `segments: Vec<f64>` (flat `[id, reversed, ...]`) | `CurveHandle` |
| `interpolate_nurbs_fit_points` | `points: Vec<f64>` (flat xyz), `degree: u32`, `closed: bool` | `CurveHandle` |
| `curve_to_nurbs` | `curve: &CurveHandle` | `CurveHandle` |

### Evaluation (by normalised parameter t in [0,1])

| Method | Parameters | Returns |
|--------|-----------|---------|
| `curve_point_at` | `curve, t` | `[x, y, z]` |
| `curve_d0_at` | `curve, t` | `[x, y, z]` (alias for point_at) |
| `curve_d1_at` | `curve, t` | `[dx, dy, dz]` (first derivative) |
| `curve_d2_at` | `curve, t` | `[dx, dy, dz]` (second derivative) |
| `curve_tangent_at` | `curve, t` | `[tx, ty, tz]` (unit tangent) |
| `curve_normal_at` | `curve, t` | `[nx, ny, nz]` (unit normal) |
| `curve_plane_at` | `curve, t` | `[ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]` (Frenet frame) |
| `curve_length` | `curve` | `f64` (total arc length) |
| `curve_length_at` | `curve, t` | `f64` (arc length to parameter) |

### Evaluation (by arc-length distance)

| Method | Parameters | Returns |
|--------|-----------|---------|
| `curve_point_at_length` | `curve, distance` | `[x, y, z]` |
| `curve_d0_at_length` | `curve, distance` | `[x, y, z]` |
| `curve_d1_at_length` | `curve, distance` | `[dx, dy, dz]` |
| `curve_d2_at_length` | `curve, distance` | `[dx, dy, dz]` |
| `curve_tangent_at_length` | `curve, distance` | `[tx, ty, tz]` |
| `curve_normal_at_length` | `curve, distance` | `[nx, ny, nz]` |
| `curve_plane_at_length` | `curve, distance` | `[ox,oy,oz, ...]` (Frenet frame) |

### Utilities

| Method | Parameters | Returns |
|--------|-----------|---------|
| `convert_coordinate_system` | `x, y, z, source: i32, target: i32` | `[x, y, z]` |

Source/target: `0` = EastingNorthing, `1` = NorthingEasting.

## Surfaces

### Constructors

| Method | Parameters | Returns |
|--------|-----------|---------|
| `create_nurbs_surface` | `degree_u, degree_v, control_u_count, control_v_count, periodic_u, periodic_v, control_points: Vec<f64>, weights: Vec<f64>, knots_u: Vec<f64>, knots_v: Vec<f64>` | `SurfaceHandle` |

### Evaluation (normalised u,v in [0,1])

| Method | Parameters | Returns |
|--------|-----------|---------|
| `surface_point_at` | `surface, u, v` | `[x, y, z]` |
| `surface_d1_at` | `surface, u, v` | `[du_x,du_y,du_z, dv_x,dv_y,dv_z]` |
| `surface_d2_at` | `surface, u, v` | `[duu_xyz, duv_xyz, dvv_xyz]` (9 values) |
| `surface_normal_at` | `surface, u, v` | `[nx, ny, nz]` |
| `surface_frame_at` | `surface, u, v` | `SurfaceEvalResult { px,py,pz, du_xyz, dv_xyz, nx,ny,nz }` |

### Transforms

| Method | Parameters | Returns |
|--------|-----------|---------|
| `surface_translate` | `surface, dx, dy, dz` | `SurfaceHandle` |
| `surface_rotate` | `surface, axis_xyz, angle_rad, pivot_xyz` (7 args) | `SurfaceHandle` |
| `surface_scale` | `surface, sx, sy, sz, pivot_xyz` (6 args) | `SurfaceHandle` |
| `surface_bake_transform` | `surface` | `SurfaceHandle` |

### Tessellation

| Method | Parameters | Returns |
|--------|-----------|---------|
| `surface_tessellate_to_mesh` | `surface, options: Vec<f64>` | `MeshHandle` |

Options: `[min_u, min_v, max_u, max_v, chord_tol, normal_tol_rad]` (6 values). Empty for defaults.

## Meshes

### Constructors

| Method | Parameters | Returns |
|--------|-----------|---------|
| `create_indexed_mesh` | `vertices: Vec<f64>` (flat xyz), `indices: Vec<u32>` | `MeshHandle` |
| `create_box_mesh` | `cx, cy, cz, sx, sy, sz` | `MeshHandle` |
| `create_uv_sphere_mesh` | `cx, cy, cz, radius, u_steps, v_steps` | `MeshHandle` |
| `create_torus_mesh` | `cx, cy, cz, major_r, minor_r, major_steps, minor_steps` | `MeshHandle` |

### Transforms

| Method | Parameters | Returns |
|--------|-----------|---------|
| `mesh_translate` | `mesh, dx, dy, dz` | `MeshHandle` |
| `mesh_rotate` | `mesh, axis_xyz, angle_rad, pivot_xyz` | `MeshHandle` |
| `mesh_scale` | `mesh, sx, sy, sz, pivot_xyz` | `MeshHandle` |
| `mesh_bake_transform` | `mesh` | `MeshHandle` |

### Queries

| Method | Parameters | Returns |
|--------|-----------|---------|
| `mesh_vertex_count` | `mesh` | `u32` |
| `mesh_triangle_count` | `mesh` | `u32` |
| `mesh_copy_vertices` | `mesh` | `Vec<f64>` (flat xyz) |
| `mesh_copy_indices` | `mesh` | `Vec<u32>` (flat i0,i1,i2) |

### Boolean & Intersection

| Method | Parameters | Returns |
|--------|-----------|---------|
| `mesh_boolean` | `mesh_a, mesh_b, op: i32` | `MeshHandle` |
| `intersect_mesh_plane` | `mesh, plane: Vec<f64>` (12 values) | `Vec<f64>` (flat xyz hits) |
| `intersect_mesh_mesh` | `mesh_a, mesh_b` | `Vec<f64>` (flat xyz hits) |

Boolean ops: `0` = Union, `1` = Intersection, `2` = Difference.

## Faces (Trimmed Surfaces)

| Method | Parameters | Returns |
|--------|-----------|---------|
| `create_face_from_surface` | `surface` | `FaceHandle` |
| `face_add_loop` | `face, points_uv: Vec<f64>` (flat uv), `is_outer: bool` | `void` |
| `face_add_loop_edges` | `face, loop_is_outer: bool, edges_flat: Vec<f64>` | `void` |
| `face_remove_loop` | `face, loop_index: u32` | `void` |
| `face_reverse_loop` | `face, loop_index: u32` | `void` |
| `face_split_trim_edge` | `face, loop_idx, edge_idx, split_t` | `void` |
| `face_validate` | `face` | `bool` |
| `face_heal` | `face` | `void` |
| `face_tessellate_to_mesh` | `face, options: Vec<f64>` | `MeshHandle` |

`edges_flat` format: `[u0,v0, u1,v1, obj_id, has_curve]` per edge (6 values each).

## B-rep

### Assembly

| Method | Parameters | Returns |
|--------|-----------|---------|
| `brep_create_empty` | (none) | `BrepHandle` |
| `brep_create_from_faces` | `face_ids: Vec<f64>` | `BrepHandle` |
| `brep_create_from_surface` | `surface` | `BrepHandle` |
| `brep_add_face` | `brep, face` | `u32` (face ID) |
| `brep_add_face_from_surface` | `brep, surface` | `u32` (face ID) |
| `brep_add_loop_uv` | `brep, face_id: u32, uvs: Vec<f64>, is_outer: bool` | `u32` (loop ID) |
| `brep_finalize_shell` | `brep` | `u32` (shell ID) |
| `brep_finalize_solid` | `brep` | `u32` (solid ID) |

### Queries

| Method | Parameters | Returns |
|--------|-----------|---------|
| `brep_face_count` | `brep` | `u32` |
| `brep_shell_count` | `brep` | `u32` |
| `brep_solid_count` | `brep` | `u32` |
| `brep_is_solid` | `brep` | `bool` |
| `brep_state` | `brep` | `u32` (0=empty, 1=faces, 2=shell, 3=solid) |
| `brep_estimate_area` | `brep` | `f64` |
| `brep_face_adjacency` | `brep, face_id: u32` | `Vec<u32>` |

### Validation & Repair

| Method | Parameters | Returns |
|--------|-----------|---------|
| `brep_validate` | `brep` | `BrepValidationResult { valid, face_count, shell_count, solid_count, issues }` |
| `brep_heal` | `brep` | `u32` (issues fixed) |

### Tessellation

| Method | Parameters | Returns |
|--------|-----------|---------|
| `brep_tessellate_to_mesh` | `brep, options: Vec<f64>` | `MeshHandle` |

### Serialization

| Method | Parameters | Returns |
|--------|-----------|---------|
| `brep_save_native` | `brep` | `Vec<u8>` (binary) |
| `brep_load_native` | `bytes: Vec<u8>` | `BrepHandle` |

### Other

| Method | Parameters | Returns |
|--------|-----------|---------|
| `brep_clone` | `brep` | `BrepHandle` |
| `brep_from_face_object` | `face` | `BrepHandle` |
| `brep_extract_face_object` | `brep, face_id: u32` | `FaceHandle` |

## Intersections

### Curve-level

| Method | Parameters | Returns |
|--------|-----------|---------|
| `intersect_curve_curve` | `curve_a, curve_b` | `Vec<f64>` (flat xyz hits) |
| `intersect_curve_plane` | `curve, plane_flat: Vec<f64>` | `Vec<f64>` (flat xyz hits) |

### Surface-level (returns IntersectionHandle)

| Method | Parameters | Returns |
|--------|-----------|---------|
| `intersect_surface_surface` | `surface_a, surface_b` | `IntersectionHandle` |
| `intersect_surface_plane` | `surface, plane_flat: Vec<f64>` | `IntersectionHandle` |
| `intersect_surface_curve` | `surface, curve` | `IntersectionHandle` |

### Branch access

| Method | Parameters | Returns |
|--------|-----------|---------|
| `intersection_branch_count` | `intersection` | `u32` |
| `intersection_branch_summary` | `intersection, branch_index` | `BranchSummary { point_count, uv_a_count, uv_b_count, curve_t_count, closed, flags }` |
| `intersection_branch_copy_points` | `intersection, branch_index` | `Vec<f64>` (flat xyz) |
| `intersection_branch_copy_uv_a` | `intersection, branch_index` | `Vec<f64>` (flat uv) |
| `intersection_branch_copy_uv_b` | `intersection, branch_index` | `Vec<f64>` (flat uv) |
| `intersection_branch_copy_curve_t` | `intersection, branch_index` | `Vec<f64>` |
| `intersection_branch_to_nurbs` | `intersection, branch_index` | `CurveHandle` |

## Bounding Volumes

| Method | Parameters | Returns |
|--------|-----------|---------|
| `compute_bounds` | `object_id: f64, mode: u32, sample_budget: u32, padding: f64` | `Bounds3` |

Mode: `0` = Fast (control-point hull), `1` = Optimal (PCA + OBB refinement).

`Bounds3` fields: `aabb_min_xyz`, `aabb_max_xyz`, `obb_center_xyz`, `obb_half_xyz`, `obb_ax_xyz`, `obb_ay_xyz`, `obb_az_xyz`, `local_aabb_min_xyz`, `local_aabb_max_xyz`.

## CAD Export

| Method | Parameters | Returns |
|--------|-----------|---------|
| `export_iges` | `object_ids: Vec<f64>` | `String` (IGES 5.3 text) |
| `export_sat` | `object_ids: Vec<f64>` | `String` (ACIS SAT text) |

## LandXML

### Parsing

| Method | Parameters | Returns |
|--------|-----------|---------|
| `landxml_parse` | `xml: &str, mode: u32, point_order: u32, units_policy: u32` | `LandXmlDocHandle` |

Mode: `0`=Strict, `1`=Lenient. Point order: `0`=NEZ, `1`=ENZ, `2`=EZN. Units: `0`=NormalizeToMeters, `1`=PreserveSource.

### Surfaces

| Method | Parameters | Returns |
|--------|-----------|---------|
| `landxml_surface_count` | `doc` | `u32` |
| `landxml_surface_name` | `doc, index` | `String` |
| `landxml_surface_copy_vertices` | `doc, index` | `Vec<f64>` (flat xyz, UTM) |
| `landxml_surface_copy_indices` | `doc, index` | `Vec<u32>` (triangle indices) |
| `landxml_extract_surface_mesh` | `doc, index` | `MeshHandle` |

### Alignments

| Method | Parameters | Returns |
|--------|-----------|---------|
| `landxml_alignment_count` | `doc` | `u32` |
| `landxml_alignment_name` | `doc, index` | `String` |
| `landxml_alignment_station_range` | `doc, alignment_index` | `[sta_start, sta_end]` |
| `landxml_alignment_profile_count` | `doc, alignment_index` | `u32` |
| `landxml_alignment_profile_name` | `doc, alignment_index, profile_index` | `String` |

### Sampling

| Method | Parameters | Returns |
|--------|-----------|---------|
| `landxml_sample_horiz_2d_segments` | `doc, alignment_index` | `Vec<f64>` packed segments |
| `landxml_sample_alignment_3d` | `doc, alignment_index, profile_index, n_steps` | `Vec<f64>` packed `[npts, x,y,z, ...]` |
| `landxml_probe_alignment` | `doc, align_idx, profile_idx, display_station` | `[px,py,pz, tx,ty,tz, grade]` |

### Metadata

| Method | Parameters | Returns |
|--------|-----------|---------|
| `landxml_warning_count` | `doc` | `u32` |
| `landxml_linear_unit` | `doc` | `String` |

### Plan Linears (FeatureLines & Breaklines)

| Method | Parameters | Returns |
|--------|-----------|---------|
| `landxml_plan_linear_count` | `doc` | `u32` |
| `landxml_plan_linear_name` | `doc, index` | `String` |
| `landxml_plan_linear_kind` | `doc, index` | `u32` (0=FeatureLine, 1=Breakline) |
| `landxml_plan_linear_copy_points` | `doc, index` | `Vec<f64>` (flat xyz) |

## Data Shapes

- **Point arrays**: flat `[x, y, z, x, y, z, ...]`.
- **UV arrays**: flat `[u, v, u, v, ...]`.
- **Plane arrays**: `[ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]` (origin + 3 axes).
- **Tessellation options**: `[min_u, min_v, max_u, max_v, chord_tol, normal_tol_rad]`.
- Failures surface as thrown JS errors (`JsValue`).
