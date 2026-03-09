// Conversion between USD prim types and internal kernel math types.
// This is the single ingestion boundary: USD types are unwrapped here into
// the plain Rust primitives that the math layer expects.

use rusted_usd::foundation::GfVec3f;
use rusted_usd::foundation::GfVec2d;
use rusted_usd::foundation::SdfPath;
use rusted_usd::stage::UsdStage;
use std::collections::HashSet;

/// Default tolerance for conversions when no context is available.
fn default_tol() -> RgmToleranceContext {
    RgmToleranceContext {
        abs_tol: 1e-10,
        rel_tol: 1e-10,
        angle_tol: 1e-6,
    }
}

/// Extract a `NurbsCurveCore` for curve `index` from a `UsdGeomNurbsCurves` prim.
///
/// Widens `point3f` (f32) to f64 at the boundary.
pub(crate) fn nurbs_core_from_curves_prim(
    curves: &rusted_usd::schema::generated::UsdGeomNurbsCurves,
    index: usize,
) -> Result<NurbsCurveCore, RgmStatus> {
    if index >= curves.curve_vertex_counts.len() {
        return Err(RgmStatus::InvalidInput);
    }

    let n_verts = curves.curve_vertex_counts[index] as usize;
    let order = if index < curves.order.len() {
        curves.order[index] as usize
    } else {
        return Err(RgmStatus::InvalidInput);
    };
    let degree = order.checked_sub(1).ok_or(RgmStatus::InvalidInput)?;

    let ctrl_offset: usize = curves
        .curve_vertex_counts
        .iter()
        .take(index)
        .map(|c| *c as usize)
        .sum();

    let control_points: Vec<RgmPoint3> = curves.points[ctrl_offset..ctrl_offset + n_verts]
        .iter()
        .map(|p| RgmPoint3 {
            x: p.x as f64,
            y: p.y as f64,
            z: p.z as f64,
        })
        .collect();

    let weights = if curves.point_weights.is_empty() {
        vec![1.0; n_verts]
    } else {
        curves.point_weights[ctrl_offset..ctrl_offset + n_verts].to_vec()
    };

    let knot_len = n_verts + order;
    let knot_offset: usize = curves
        .curve_vertex_counts
        .iter()
        .take(index)
        .zip(curves.order.iter())
        .map(|(c, o)| *c as usize + *o as usize)
        .sum();
    let knots = curves.knots[knot_offset..knot_offset + knot_len].to_vec();

    let (u_start, u_end) = if index < curves.ranges.len() {
        let r = &curves.ranges[index];
        (r.x, r.y)
    } else {
        (knots[degree], knots[n_verts])
    };

    let periodic = false; // TODO: detect from form attribute if added

    Ok(NurbsCurveCore {
        degree,
        periodic,
        control_points,
        weights,
        knots,
        u_start,
        u_end,
        tol: default_tol(),
    })
}

/// Extract a `NurbsSurfaceCore` from a `UsdGeomNurbsPatch` prim.
///
/// Widens `point3f` (f32) to f64 at the boundary.
pub(crate) fn nurbs_core_from_patch_prim(
    patch: &rusted_usd::schema::generated::UsdGeomNurbsPatch,
) -> Result<NurbsSurfaceCore, RgmStatus> {
    let u_order = patch.u_order.ok_or(RgmStatus::InvalidInput)? as usize;
    let v_order = patch.v_order.ok_or(RgmStatus::InvalidInput)? as usize;
    let u_count = patch.u_vertex_count.ok_or(RgmStatus::InvalidInput)? as usize;
    let v_count = patch.v_vertex_count.ok_or(RgmStatus::InvalidInput)? as usize;

    let degree_u = u_order.checked_sub(1).ok_or(RgmStatus::InvalidInput)?;
    let degree_v = v_order.checked_sub(1).ok_or(RgmStatus::InvalidInput)?;

    let control_points: Vec<RgmPoint3> = patch
        .points
        .iter()
        .map(|p| RgmPoint3 {
            x: p.x as f64,
            y: p.y as f64,
            z: p.z as f64,
        })
        .collect();

    let weights = if patch.point_weights.is_empty() {
        vec![1.0; control_points.len()]
    } else {
        patch.point_weights.clone()
    };

    let knots_u = patch.u_knots.clone();
    let knots_v = patch.v_knots.clone();

    let (u_start, u_end) = if let Some(ref r) = patch.u_range {
        (r.x, r.y)
    } else {
        (knots_u[degree_u], knots_u[u_count])
    };

    let (v_start, v_end) = if let Some(ref r) = patch.v_range {
        (r.x, r.y)
    } else {
        (knots_v[degree_v], knots_v[v_count])
    };

    let periodic_u = patch.u_form.as_str() == "periodic";
    let periodic_v = patch.v_form.as_str() == "periodic";

    Ok(NurbsSurfaceCore {
        degree_u,
        degree_v,
        periodic_u,
        periodic_v,
        control_u_count: u_count,
        control_v_count: v_count,
        control_points,
        weights,
        knots_u,
        knots_v,
        u_start,
        u_end,
        v_start,
        v_end,
        tol: default_tol(),
    })
}

/// Compute the axis-aligned bounding box of a slice of control points,
/// returned as `[min, max]` GfVec3f pair suitable for the `extent` attribute.
pub(crate) fn extent_of_points(pts: &[RgmPoint3]) -> Option<[GfVec3f; 2]> {
    if pts.is_empty() {
        return None;
    }
    let mut min = (f64::INFINITY, f64::INFINITY, f64::INFINITY);
    let mut max = (f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in pts {
        if p.x < min.0 { min.0 = p.x; }
        if p.y < min.1 { min.1 = p.y; }
        if p.z < min.2 { min.2 = p.z; }
        if p.x > max.0 { max.0 = p.x; }
        if p.y > max.1 { max.1 = p.y; }
        if p.z > max.2 { max.2 = p.z; }
    }
    Some([
        GfVec3f::new(min.0 as f32, min.1 as f32, min.2 as f32),
        GfVec3f::new(max.0 as f32, max.1 as f32, max.2 as f32),
    ])
}

/// Convert kernel `NurbsCurveCore` back to a `UsdGeomNurbsCurves` prim
/// (single curve, index 0).
pub(crate) fn curves_prim_from_nurbs_core(core: &NurbsCurveCore) -> rusted_usd::schema::generated::UsdGeomNurbsCurves {
    let mut prim = rusted_usd::schema::generated::UsdGeomNurbsCurves::default();
    prim.curve_vertex_counts = vec![core.control_points.len() as i32];
    prim.order = vec![(core.degree + 1) as i32];
    prim.knots = core.knots.clone();
    prim.ranges = vec![GfVec2d::new(core.u_start, core.u_end)];
    prim.points = core
        .control_points
        .iter()
        .map(|p| GfVec3f::new(p.x as f32, p.y as f32, p.z as f32))
        .collect();
    if core.weights.iter().any(|w| (*w - 1.0).abs() > 1e-15) {
        prim.point_weights = core.weights.clone();
    }
    prim.widths = vec![0.01f32; core.control_points.len()];
    if let Some([lo, hi]) = extent_of_points(&core.control_points) {
        prim.extent = vec![lo, hi];
    }
    prim
}

/// Convert kernel `NurbsSurfaceCore` back to a `UsdGeomNurbsPatch` prim.
pub(crate) fn patch_prim_from_nurbs_core(core: &NurbsSurfaceCore) -> rusted_usd::schema::generated::UsdGeomNurbsPatch {
    let mut prim = rusted_usd::schema::generated::UsdGeomNurbsPatch::default();
    prim.u_vertex_count = Some(core.control_u_count as i32);
    prim.v_vertex_count = Some(core.control_v_count as i32);
    prim.u_order = Some((core.degree_u + 1) as i32);
    prim.v_order = Some((core.degree_v + 1) as i32);
    prim.u_knots = core.knots_u.clone();
    prim.v_knots = core.knots_v.clone();
    prim.u_range = Some(GfVec2d::new(core.u_start, core.u_end));
    prim.v_range = Some(GfVec2d::new(core.v_start, core.v_end));
    prim.points = core
        .control_points
        .iter()
        .map(|p| GfVec3f::new(p.x as f32, p.y as f32, p.z as f32))
        .collect();
    if core.weights.iter().any(|w| (*w - 1.0).abs() > 1e-15) {
        prim.point_weights = core.weights.clone();
    }
    if core.periodic_u {
        prim.u_form = rusted_usd::foundation::TfToken::new("periodic");
    }
    if core.periodic_v {
        prim.v_form = rusted_usd::foundation::TfToken::new("periodic");
    }
    if let Some([lo, hi]) = extent_of_points(&core.control_points) {
        prim.extent = vec![lo, hi];
    }
    prim
}

/// Convert a `UsdGeomMesh` to the internal `MeshData` representation.
pub(crate) fn mesh_data_from_prim(mesh: &rusted_usd::schema::generated::UsdGeomMesh) -> MeshData {
    let vertices: Vec<RgmPoint3> = mesh
        .points
        .iter()
        .map(|p| RgmPoint3 {
            x: p.x as f64,
            y: p.y as f64,
            z: p.z as f64,
        })
        .collect();

    let mut triangles = Vec::new();
    let mut idx_offset = 0usize;
    for &fvc in &mesh.face_vertex_counts {
        let n = fvc as usize;
        if n >= 3 {
            let base = mesh.face_vertex_indices[idx_offset] as u32;
            for j in 1..n - 1 {
                triangles.push([
                    base,
                    mesh.face_vertex_indices[idx_offset + j] as u32,
                    mesh.face_vertex_indices[idx_offset + j + 1] as u32,
                ]);
            }
        }
        idx_offset += n;
    }

    MeshData {
        vertices,
        triangles,
        transform: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

fn matrix_is_identity(matrix: [[f64; 4]; 4]) -> bool {
    matrix == matrix_identity()
}

pub(crate) fn xform_prim_from_matrix(
    matrix: [[f64; 4]; 4],
) -> rusted_usd::schema::generated::UsdGeomXform {
    let mut prim = rusted_usd::schema::generated::UsdGeomXform::default();
    if !matrix_is_identity(matrix) {
        prim.xform_op_transform = Some(matrix);
        prim.xform_op_order = vec![rusted_usd::foundation::TfToken::new("xformOp:transform")];
    }
    prim
}

pub(crate) fn local_transform_for_path(stage: &UsdStage, path: &SdfPath) -> [[f64; 4]; 4] {
    stage
        .get::<rusted_usd::schema::generated::UsdGeomXform>(path)
        .and_then(|x| x.xform_op_transform)
        .unwrap_or_else(matrix_identity)
}

pub(crate) fn world_transform_for_path(stage: &UsdStage, path: &SdfPath) -> [[f64; 4]; 4] {
    let mut chain = Vec::new();
    let mut current = Some(path.clone());
    while let Some(node) = current {
        chain.push(node.clone());
        current = node.parent().filter(|parent| parent.as_str() != "/");
    }

    let mut world = matrix_identity();
    for node in chain.into_iter().rev() {
        world = matrix_mul(world, local_transform_for_path(stage, &node));
    }
    world
}

pub(crate) fn transform_curve_core(
    core: &NurbsCurveCore,
    transform: [[f64; 4]; 4],
) -> NurbsCurveCore {
    if matrix_is_identity(transform) {
        return core.clone();
    }
    let mut next = core.clone();
    for point in &mut next.control_points {
        *point = matrix_apply_point(transform, *point);
    }
    next
}

/// Convert internal `MeshData` back to a local-space `UsdGeomMesh` prim.
pub(crate) fn mesh_prim_from_data(data: &MeshData) -> rusted_usd::schema::generated::UsdGeomMesh {
    let mut prim = rusted_usd::schema::generated::UsdGeomMesh::default();
    prim.points = data
        .vertices
        .iter()
        .map(|p| GfVec3f::new(p.x as f32, p.y as f32, p.z as f32))
        .collect();
    prim.subdivision_scheme = rusted_usd::foundation::TfToken::new("none");

    prim.face_vertex_counts = vec![3i32; data.triangles.len()];
    prim.face_vertex_indices = data
        .triangles
        .iter()
        .flat_map(|t| t.iter().map(|&i| i as i32))
        .collect();
    if let Some([lo, hi]) = extent_of_points(&data.vertices) {
        prim.extent = vec![lo, hi];
    }

    prim
}

/// Convert a `SurfaceData` to a local-space `UsdGeomNurbsPatch` prim.
pub(crate) fn patch_prim_from_surface_data(data: &SurfaceData) -> rusted_usd::schema::generated::UsdGeomNurbsPatch {
    patch_prim_from_nurbs_core(&data.core)
}

/// Fit a NURBS curve through an ordered sequence of 3D points using global
/// interpolation with chord-length parameterisation. Returns the NurbsCurveCore
/// or None if the input is degenerate.
pub(crate) fn fit_nurbs_through_points(pts: &[RgmPoint3], degree: usize) -> Option<NurbsCurveCore> {
    let n = pts.len();
    if n < degree + 1 {
        return None;
    }

    let mut chord_lengths = Vec::with_capacity(n);
    chord_lengths.push(0.0_f64);
    for i in 1..n {
        let dx = pts[i].x - pts[i - 1].x;
        let dy = pts[i].y - pts[i - 1].y;
        let dz = pts[i].z - pts[i - 1].z;
        chord_lengths.push(chord_lengths[i - 1] + (dx * dx + dy * dy + dz * dz).sqrt());
    }
    let total_len = *chord_lengths.last().unwrap();
    if total_len < 1e-14 {
        return None;
    }
    let params: Vec<f64> = chord_lengths.iter().map(|&c| c / total_len).collect();

    let p = degree;
    let nk = n + p + 1;
    let mut knots = vec![0.0_f64; nk];
    for i in 0..=p {
        knots[i] = 0.0;
        knots[nk - 1 - i] = 1.0;
    }
    for j in 1..n - p {
        let sum: f64 = (j..j + p).map(|idx| params[idx]).sum();
        knots[j + p] = sum / p as f64;
    }

    let weights = vec![1.0; n];

    Some(NurbsCurveCore {
        degree,
        knots,
        weights,
        control_points: pts.to_vec(),
        periodic: false,
        u_start: 0.0,
        u_end: 1.0,
        tol: RgmToleranceContext {
            abs_tol: 1e-9,
            rel_tol: 1e-9,
            angle_tol: 1e-9,
        },
    })
}

/// Export the USD stage (or a subset of prims) to USDA text.
///
/// If `object_ids` is empty, the full stage is serialised.  Otherwise only the
/// prims corresponding to the given IDs are included.
pub(crate) fn export_usda_text(
    session: RgmKernelHandle,
    object_ids: &[u64],
) -> Result<String, String> {
    let entry = SESSIONS
        .get(&session.0)
        .ok_or_else(|| "Session not found".to_string())?;
    let state = entry.value().read();
    let canonical_stage = canonical_stage_for_usd_export(&state.stage);
    if object_ids.is_empty() {
        Ok(rusted_usd::usda::writer::write_stage(&canonical_stage))
    } else {
        let paths = collect_usd_export_root_paths(&state, object_ids);
        Ok(rusted_usd::usda::writer::write_prims(&canonical_stage, &paths))
    }
}

fn is_usd_display_helper_path(path: &SdfPath) -> bool {
    let name = path.name();
    name == "displayMesh"
}

pub(crate) fn canonical_stage_for_usd_export(stage: &UsdStage) -> UsdStage {
    let mut canonical = UsdStage::new();

    let mut prims: Vec<_> = stage.all_prims().collect();
    prims.sort_by_key(|prim| prim.path.as_str().len());

    for prim in prims {
        if is_usd_display_helper_path(&prim.path) {
            continue;
        }
        let schema = match &prim.schema {
            rusted_usd::schema::generated::SchemaData::Mesh(mesh) => {
                let mut next = mesh.clone();
                let world = world_transform_for_path(stage, &prim.path);
                next.points = mesh
                    .points
                    .iter()
                    .map(|p| {
                        let wp = matrix_apply_point(world, RgmPoint3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 });
                        GfVec3f::new(wp.x as f32, wp.y as f32, wp.z as f32)
                    })
                    .collect();
                if let Some([lo, hi]) = extent_of_points(
                    &next
                        .points
                        .iter()
                        .map(|p| RgmPoint3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 })
                        .collect::<Vec<_>>(),
                ) {
                    next.extent = vec![lo, hi];
                }
                rusted_usd::schema::generated::SchemaData::Mesh(next)
            }
            rusted_usd::schema::generated::SchemaData::NurbsPatch(patch) => {
                let mut next = patch.clone();
                let world = world_transform_for_path(stage, &prim.path);
                next.points = patch
                    .points
                    .iter()
                    .map(|p| {
                        let wp = matrix_apply_point(world, RgmPoint3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 });
                        GfVec3f::new(wp.x as f32, wp.y as f32, wp.z as f32)
                    })
                    .collect();
                if let Some([lo, hi]) = extent_of_points(
                    &next
                        .points
                        .iter()
                        .map(|p| RgmPoint3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 })
                        .collect::<Vec<_>>(),
                ) {
                    next.extent = vec![lo, hi];
                }
                rusted_usd::schema::generated::SchemaData::NurbsPatch(next)
            }
            rusted_usd::schema::generated::SchemaData::NurbsCurves(curves) => {
                let mut next = curves.clone();
                let world = world_transform_for_path(stage, &prim.path);
                next.points = curves
                    .points
                    .iter()
                    .map(|p| {
                        let wp = matrix_apply_point(world, RgmPoint3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 });
                        GfVec3f::new(wp.x as f32, wp.y as f32, wp.z as f32)
                    })
                    .collect();
                if let Some([lo, hi]) = extent_of_points(
                    &next
                        .points
                        .iter()
                        .map(|p| RgmPoint3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 })
                        .collect::<Vec<_>>(),
                ) {
                    next.extent = vec![lo, hi];
                }
                rusted_usd::schema::generated::SchemaData::NurbsCurves(next)
            }
            rusted_usd::schema::generated::SchemaData::BasisCurves(curves) => {
                let mut next = curves.clone();
                let world = world_transform_for_path(stage, &prim.path);
                next.points = curves
                    .points
                    .iter()
                    .map(|p| {
                        let wp = matrix_apply_point(world, RgmPoint3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 });
                        GfVec3f::new(wp.x as f32, wp.y as f32, wp.z as f32)
                    })
                    .collect();
                if let Some([lo, hi]) = extent_of_points(
                    &next
                        .points
                        .iter()
                        .map(|p| RgmPoint3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 })
                        .collect::<Vec<_>>(),
                ) {
                    next.extent = vec![lo, hi];
                }
                rusted_usd::schema::generated::SchemaData::BasisCurves(next)
            }
            rusted_usd::schema::generated::SchemaData::Xform(_) => {
                rusted_usd::schema::generated::SchemaData::Xform(
                    rusted_usd::schema::generated::UsdGeomXform::default(),
                )
            }
            other => other.clone(),
        };
        canonical.define_prim(prim.path.clone(), schema);
    }
    canonical
}

pub(crate) fn collect_usd_export_root_paths(
    state: &SessionState,
    object_ids: &[u64],
) -> Vec<SdfPath> {
    let mut roots = Vec::new();
    let mut seen = HashSet::new();
    for &id in object_ids {
        let Some(path) = state.path_index.get(&id) else {
            continue;
        };
        if !is_usd_display_helper_path(path) && seen.insert(path.clone()) {
            roots.push(path.clone());
        }
        if let Some(parent) = path.parent() {
            let display = parent.child(&format!("{}_display", path.name()));
            if state.stage.prim(&display).is_some() && seen.insert(display.clone()) {
                roots.push(display);
            }
        }
    }
    roots
}

pub(crate) fn collect_export_root_paths(
    state: &SessionState,
    object_ids: &[u64],
) -> Vec<SdfPath> {
    let mut roots = Vec::new();
    let mut seen = HashSet::new();

    for &id in object_ids {
        let Some(path) = state.path_index.get(&id) else {
            continue;
        };
        if seen.insert(path.clone()) {
            roots.push(path.clone());
        }

        if let Some(parent) = path.parent() {
            for suffix in ["_display", "_mesh"] {
                let sibling = parent.child(&format!("{}{}", path.name(), suffix));
                if state.stage.prim(&sibling).is_some() && seen.insert(sibling.clone()) {
                    roots.push(sibling);
                }
            }
        }
    }

    roots
}

pub(crate) fn collect_stage_subtree_paths(stage: &UsdStage, roots: &[SdfPath]) -> Vec<SdfPath> {
    let mut paths = Vec::new();
    let mut stack = roots.to_vec();
    let mut seen = HashSet::new();

    while let Some(path) = stack.pop() {
        if !seen.insert(path.clone()) {
            continue;
        }
        if stage.prim(&path).is_none() {
            continue;
        }
        paths.push(path.clone());
        for child in stage.children(&path).iter().rev() {
            stack.push(child.clone());
        }
    }

    paths
}
