//! Session lifecycle: global session registry, accessor helpers, and
//! object-insertion utilities.
//!
//! `SESSIONS` is a sharded global map from session ID to `SessionState`
//! protected by per-session read/write locks. All mutable access goes through
//! [`with_session_mut`], which acquires a write lock for the target session.

use crate::session::objects::{
    CurveData, GeometryObject, IntersectionData, LandXmlDocData, MeshData, SessionState,
    SurfaceData,
};
use crate::{RgmKernelHandle, RgmObjectHandle, RgmStatus};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
pub(crate) static NEXT_OBJECT_ID: AtomicU64 = AtomicU64::new(1);
pub(crate) static SESSIONS: Lazy<DashMap<u64, RwLock<SessionState>>> = Lazy::new(DashMap::new);

pub(crate) fn set_error(session_id: u64, status: RgmStatus, message: impl Into<String>) {
    if let Some(entry) = SESSIONS.get(&session_id) {
        let mut session = entry.value().write();
        session.last_error_code = status;
        session.last_error_message = message.into();
    }
}

fn clear_error_state(state: &mut SessionState) {
    state.last_error_code = RgmStatus::Ok;
    state.last_error_message.clear();
}

// P1: This function is kept for compatibility with existing export callsites outside brep_ops.
// In brep_ops.rs, finish() no longer calls clear_error; with_session_mut clears on success.
pub(crate) fn clear_error(session_id: u64) {
    if let Some(entry) = SESSIONS.get(&session_id) {
        let mut session = entry.value().write();
        clear_error_state(&mut session);
    }
}

// P1: Error is cleared inside the write lock on success — no second lock acquisition.
pub(crate) fn with_session_mut<T>(
    session: RgmKernelHandle,
    f: impl FnOnce(&mut SessionState) -> Result<T, RgmStatus>,
) -> Result<T, RgmStatus> {
    let entry = SESSIONS.get(&session.0).ok_or(RgmStatus::NotFound)?;
    let mut state = entry.value().write();
    let result = f(&mut state);
    if result.is_ok() {
        clear_error_state(&mut state);
    }
    result
}

pub(crate) fn map_err_with_session(
    session: RgmKernelHandle,
    status: RgmStatus,
    message: &str,
) -> RgmStatus {
    set_error(session.0, status, message);
    status
}

// ─── Object Insert Helpers ────────────────────────────────────────────────────
//
// Each insert writes to the legacy `objects` map AND defines a prim on the
// `UsdStage`.  The `path_index` maps object_id → SdfPath for handle lookups.

pub(crate) fn insert_curve(state: &mut SessionState, curve: CurveData) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);

    let path = rusted_usd::foundation::SdfPath::new(&format!("/Curves/Curve_{object_id}"));
    match &curve {
        CurveData::Polycurve(poly) => {
            state.stage.define_prim(
                path.clone(),
                rusted_usd::schema::generated::SchemaData::Scope(
                    rusted_usd::schema::generated::UsdGeomScope::default(),
                ),
            );
            let seg_data: Vec<_> = poly.segments.iter().enumerate().filter_map(|(i, seg)| {
                let seg_path = state.path_index.get(&seg.curve.0)?;
                let seg_curves = state.stage.get::<rusted_usd::schema::generated::UsdGeomNurbsCurves>(seg_path)?;
                let child_prim = seg_curves.clone();
                let core = crate::nurbs_core_from_curves_prim(seg_curves, 0).ok();
                Some((i, child_prim, core))
            }).collect();
            for (i, child_prim, core) in seg_data {
                let child_path = path.child(&format!("Seg_{i}"));
                state.stage.define_prim(
                    child_path.clone(),
                    rusted_usd::schema::generated::SchemaData::NurbsCurves(child_prim),
                );
                if let Some(ref core) = core {
                    add_curve_display_prim(state, &child_path, core);
                }
            }
        }
        _ => {
            if let Some(nurbs) = crate::session::objects::curve_canonical_nurbs(&curve) {
                let prim = crate::curves_prim_from_nurbs_core(&nurbs.core);
                state.stage.define_prim(
                    path.clone(),
                    rusted_usd::schema::generated::SchemaData::NurbsCurves(prim),
                );
                add_curve_display_prim(state, &path, &nurbs.core);
            }
        }
    }
    state.path_index.insert(object_id, path);

    state
        .objects
        .insert(object_id, GeometryObject::Curve(curve));
    RgmObjectHandle(object_id)
}

pub(crate) fn insert_mesh(state: &mut SessionState, mesh: MeshData) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);

    let path = rusted_usd::foundation::SdfPath::new(&format!("/Meshes/Mesh_{object_id}"));
    let xform = crate::xform_prim_from_matrix(mesh.transform);
    state.stage.define_prim(
        path.clone(),
        rusted_usd::schema::generated::SchemaData::Xform(xform),
    );
    let geom_path = path.child("Geom");
    let prim = crate::mesh_prim_from_data(&mesh);
    state.stage.define_prim(
        geom_path,
        rusted_usd::schema::generated::SchemaData::Mesh(prim),
    );
    state.path_index.insert(object_id, path);

    state.objects.insert(object_id, GeometryObject::Mesh(mesh));
    RgmObjectHandle(object_id)
}

pub(crate) fn insert_surface(state: &mut SessionState, surface: SurfaceData) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);

    let path = rusted_usd::foundation::SdfPath::new(&format!("/Surfaces/Surface_{object_id}"));
    let xform = crate::xform_prim_from_matrix(surface.transform);
    state.stage.define_prim(
        path.clone(),
        rusted_usd::schema::generated::SchemaData::Xform(xform),
    );
    let patch_path = path.child("Patch");
    let prim = crate::patch_prim_from_surface_data(&surface);
    state.stage.define_prim(
        patch_path.clone(),
        rusted_usd::schema::generated::SchemaData::NurbsPatch(prim),
    );
    add_surface_display_mesh_prim(state, &path, &surface);
    state.path_index.insert(object_id, path);

    state
        .objects
        .insert(object_id, GeometryObject::Surface(surface));
    RgmObjectHandle(object_id)
}

fn add_surface_display_mesh_prim(
    state: &mut SessionState,
    surface_path: &rusted_usd::foundation::SdfPath,
    surface: &SurfaceData,
) {
    let Ok(samples) = crate::tessellate_surface_samples(surface, None) else {
        return;
    };
    if samples.vertices.is_empty() || samples.triangles.is_empty() {
        return;
    }
    let mesh = crate::mesh_prim_from_data(&crate::build_mesh_from_tessellation(
        &samples,
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    ));
    let mesh_path = surface_path.child("displayMesh");
    state.stage.define_prim(
        mesh_path,
        rusted_usd::schema::generated::SchemaData::Mesh(mesh),
    );
}

pub(crate) fn insert_intersection(
    state: &mut SessionState,
    intersection: IntersectionData,
) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);

    let scope_path = rusted_usd::foundation::SdfPath::new(&format!("/Results/Intersect_{object_id}"));
    state.stage.define_prim(
        scope_path.clone(),
        rusted_usd::schema::generated::SchemaData::Scope(
            rusted_usd::schema::generated::UsdGeomScope::default(),
        ),
    );
    for (i, branch) in intersection.branches.iter().enumerate() {
        if branch.points.len() >= 2 {
            let child_path = scope_path.child(&format!("Branch_{i}"));
            let mut curves = rusted_usd::schema::generated::UsdGeomNurbsCurves::default();
            curves.curve_vertex_counts = vec![branch.points.len() as i32];
            curves.widths = vec![0.01f32; branch.points.len()];
            curves.order = vec![2]; // linear polyline
            let n = branch.points.len();
            let mut knots = vec![0.0; n + 2];
            for j in 0..n + 2 {
                knots[j] = if j == 0 { 0.0 } else if j >= n + 1 { 1.0 } else { (j - 1) as f64 / (n - 1) as f64 };
            }
            curves.knots = knots;
            curves.points = branch.points.iter()
                .map(|p| rusted_usd::foundation::GfVec3f::new(p.x as f32, p.y as f32, p.z as f32))
                .collect();
            if let Some([lo, hi]) = crate::kernel_impl::extent_of_points(&branch.points) {
                curves.extent = vec![lo, hi];
            }
            state.stage.define_prim(
                child_path.clone(),
                rusted_usd::schema::generated::SchemaData::NurbsCurves(curves),
            );
            add_curve_mesh_prim(state, &child_path, &branch.points);
        }
    }
    state.path_index.insert(object_id, scope_path);

    state
        .objects
        .insert(object_id, GeometryObject::Intersection(intersection));
    RgmObjectHandle(object_id)
}

/// Evaluate a curve at N uniform parametric steps, returning 3D points.
fn evaluate_curve_points(
    core: &crate::math::nurbs_curve_eval::NurbsCurveCore,
    n: usize,
) -> Vec<crate::RgmPoint3> {
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;
        if let Ok(res) = crate::math::nurbs_curve_eval::eval_nurbs_normalized(core, t) {
            pts.push(res.point);
        }
    }
    pts
}

/// Write a `BasisCurves` sibling prim with evaluated points alongside a `NurbsCurves` prim.
/// This gives smooth curve display in viewers that don't evaluate NURBS natively
/// (usdview, Omniverse, Blender).
fn add_curve_display_prim(
    state: &mut SessionState,
    nurbs_path: &rusted_usd::foundation::SdfPath,
    core: &crate::math::nurbs_curve_eval::NurbsCurveCore,
) {
    let rgm_pts = evaluate_curve_points(core, 64);
    if rgm_pts.len() < 2 {
        return;
    }

    let pts: Vec<rusted_usd::foundation::GfVec3f> = rgm_pts
        .iter()
        .map(|p| rusted_usd::foundation::GfVec3f::new(p.x as f32, p.y as f32, p.z as f32))
        .collect();

    let mut basis = rusted_usd::schema::generated::UsdGeomBasisCurves::default();
    basis.type_ = rusted_usd::foundation::TfToken::new("linear");
    basis.curve_vertex_counts = vec![pts.len() as i32];
    basis.widths = vec![0.005f32; pts.len()];
    if let Some([lo, hi]) = crate::kernel_impl::extent_of_points(&rgm_pts) {
        basis.extent = vec![lo, hi];
    }
    basis.points = pts;

    if let Some(parent) = nurbs_path.parent() {
        let display_path = parent.child(&format!("{}_display", nurbs_path.name()));
        state.stage.define_prim(
            display_path,
            rusted_usd::schema::generated::SchemaData::BasisCurves(basis),
        );
    }

    add_curve_mesh_prim(state, nurbs_path, &rgm_pts);
}

/// Write a thin tube `Mesh` sibling prim for a curve so that viewers which only
/// support mesh geometry (e.g. Rhino) can display it.
///
/// Creates a 4-sided tube with configurable radius around the evaluated polyline.
fn add_curve_mesh_prim(
    state: &mut SessionState,
    nurbs_path: &rusted_usd::foundation::SdfPath,
    polyline: &[crate::RgmPoint3],
) {
    use crate::math::vec3 as v3;
    use crate::{RgmPoint3, RgmVec3};

    if polyline.len() < 2 {
        return;
    }

    let extent = crate::kernel_impl::extent_of_points(polyline);
    let diag = if let Some([lo, hi]) = extent {
        let dx = (hi.x - lo.x) as f64;
        let dy = (hi.y - lo.y) as f64;
        let dz = (hi.z - lo.z) as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    } else {
        1.0
    };
    let radius = (diag * 0.002).max(1e-4);

    const SIDES: usize = 4;
    let n = polyline.len();

    let mut vertices: Vec<RgmPoint3> = Vec::with_capacity(n * SIDES);
    let mut face_vertex_counts: Vec<i32> = Vec::with_capacity((n - 1) * SIDES);
    let mut face_vertex_indices: Vec<i32> = Vec::with_capacity((n - 1) * SIDES * 4);

    let seed_up = RgmVec3 { x: 0.0, y: 0.0, z: 1.0 };
    let alt_up = RgmVec3 { x: 0.0, y: 1.0, z: 0.0 };

    for i in 0..n {
        let tangent = if i == 0 {
            v3::sub(polyline[1], polyline[0])
        } else if i == n - 1 {
            v3::sub(polyline[n - 1], polyline[n - 2])
        } else {
            v3::sub(polyline[i + 1], polyline[i - 1])
        };

        let t = v3::normalize(tangent).unwrap_or(RgmVec3 { x: 1.0, y: 0.0, z: 0.0 });
        let up = if v3::norm(v3::cross(t, seed_up)) > 0.1 { seed_up } else { alt_up };
        let n1 = v3::normalize(v3::cross(t, up)).unwrap_or(RgmVec3 { x: 1.0, y: 0.0, z: 0.0 });
        let n2 = v3::cross(t, n1);

        for s in 0..SIDES {
            let angle = std::f64::consts::TAU * s as f64 / SIDES as f64;
            let (sin_a, cos_a) = angle.sin_cos();
            let offset = v3::add(v3::scale(n1, cos_a * radius), v3::scale(n2, sin_a * radius));
            vertices.push(v3::add_vec(polyline[i], offset));
        }
    }

    for i in 0..(n - 1) {
        let base = (i * SIDES) as i32;
        let next = ((i + 1) * SIDES) as i32;
        for s in 0..SIDES as i32 {
            let s_next = (s + 1) % SIDES as i32;
            face_vertex_counts.push(4);
            face_vertex_indices.push(base + s);
            face_vertex_indices.push(base + s_next);
            face_vertex_indices.push(next + s_next);
            face_vertex_indices.push(next + s);
        }
    }

    let gf_pts: Vec<rusted_usd::foundation::GfVec3f> = vertices
        .iter()
        .map(|p| rusted_usd::foundation::GfVec3f::new(p.x as f32, p.y as f32, p.z as f32))
        .collect();

    let mut mesh = rusted_usd::schema::generated::UsdGeomMesh::default();
    mesh.points = gf_pts;
    mesh.face_vertex_counts = face_vertex_counts;
    mesh.face_vertex_indices = face_vertex_indices;
    mesh.subdivision_scheme = rusted_usd::foundation::TfToken::new("none");
    if let Some([lo, hi]) = crate::kernel_impl::extent_of_points(&vertices) {
        mesh.extent = vec![lo, hi];
    }

    if let Some(parent) = nurbs_path.parent() {
        let mesh_path = parent.child(&format!("{}_mesh", nurbs_path.name()));
        state.stage.define_prim(
            mesh_path,
            rusted_usd::schema::generated::SchemaData::Mesh(mesh),
        );
    }
}

pub(crate) fn insert_landxml_doc(
    state: &mut SessionState,
    doc: LandXmlDocData,
) -> RgmObjectHandle {
    use crate::landxml::evaluate_alignment_3d;

    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);

    let path = rusted_usd::foundation::SdfPath::new(&format!("/LandXML/Doc_{object_id}"));
    state.stage.define_prim(
        path.clone(),
        rusted_usd::schema::generated::SchemaData::Scope(
            rusted_usd::schema::generated::UsdGeomScope::default(),
        ),
    );

    let n_steps: usize = 200;
    for (surface_index, surface) in doc.doc.surfaces.iter().enumerate() {
        let vertices = surface.vertices_m.clone();
        let triangles: Vec<[u32; 3]> = surface
            .triangles
            .chunks_exact(3)
            .map(|tri| [tri[0], tri[1], tri[2]])
            .collect();
        let mesh = crate::mesh_prim_from_data(&MeshData {
            vertices,
            triangles,
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        });
        let child_path = path.child(&format!("Surface_{surface_index}"));
        state.stage.define_prim(
            child_path,
            rusted_usd::schema::generated::SchemaData::Mesh(mesh),
        );
    }

    for (a_idx, alignment) in doc.doc.alignments.iter().enumerate() {
        let sta_start = alignment.sta_start_m;
        let sta_end = sta_start + alignment.length_m;
        if sta_end <= sta_start {
            continue;
        }
        let step = (sta_end - sta_start) / n_steps as f64;
        for (p_idx, profile) in alignment.profiles.iter().enumerate() {
            let mut pts = Vec::new();
            let mut s = sta_start;
            while s <= sta_end + 1e-9 {
                if let Ok(sample) = evaluate_alignment_3d(alignment, profile, s.min(sta_end)) {
                    pts.push(sample.point);
                }
                s += step;
            }
            if pts.len() < 4 {
                continue;
            }
            if let Some(core) = crate::fit_nurbs_through_points(&pts, 3) {
                let prim = crate::curves_prim_from_nurbs_core(&core);
                let child_path = path.child(&format!("Alignment_{a_idx}_Profile_{p_idx}"));
                state.stage.define_prim(
                    child_path.clone(),
                    rusted_usd::schema::generated::SchemaData::NurbsCurves(prim),
                );
                add_curve_display_prim(state, &child_path, &core);
            }
        }
    }

    for (linear_index, linear) in doc.doc.plan_linears.iter().enumerate() {
        if linear.points.len() < 2 {
            continue;
        }
        if let Some(core) = crate::fit_nurbs_through_points(&linear.points, 1) {
            let prim = crate::curves_prim_from_nurbs_core(&core);
            let child_path = path.child(&format!("PlanLinear_{linear_index}"));
            state.stage.define_prim(
                child_path.clone(),
                rusted_usd::schema::generated::SchemaData::NurbsCurves(prim),
            );
            add_curve_display_prim(state, &child_path, &core);
        }
    }

    state.path_index.insert(object_id, path);

    state
        .objects
        .insert(object_id, GeometryObject::LandXmlDoc(doc));
    RgmObjectHandle(object_id)
}
