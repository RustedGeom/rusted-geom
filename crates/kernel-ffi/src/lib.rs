mod math;

use boolmesh::{
    compute_boolean,
    prelude::{Manifold, OpType as BoolOpType},
};
use kernel_abi_meta::{rgm_export, rgm_ffi_type};
use math::arc_length::{build_arc_length_cache, length_from_u, u_from_length, ArcLengthCache};
use math::frame::{
    normal as frame_normal, orthonormalize_plane_axes, plane as frame_plane, point_from_frame,
    tangent as frame_tangent,
};
use math::intersections::{intersect_curve_curve_points, intersect_curve_plane_points};
use math::nurbs_curve_eval::{
    eval_nurbs_normalized, eval_nurbs_u, map_normalized_to_u, validate_curve, CurveEvalResult,
    NurbsCurveCore,
};
use once_cell::sync::Lazy;
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_2, PI};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

#[rgm_ffi_type]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgmStatus {
    Ok = 0,
    InvalidInput = 1,
    NotFound = 2,
    OutOfRange = 3,
    DegenerateGeometry = 4,
    NoConvergence = 5,
    NumericalFailure = 6,
    NotImplemented = 7,
    InternalError = 255,
}

#[rgm_ffi_type]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RgmKernelHandle(pub u64);

#[rgm_ffi_type]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RgmObjectHandle(pub u64);

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmPoint3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmPlane {
    pub origin: RgmPoint3,
    pub x_axis: RgmVec3,
    pub y_axis: RgmVec3,
    pub z_axis: RgmVec3,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmToleranceContext {
    pub abs_tol: f64,
    pub rel_tol: f64,
    pub angle_tol: f64,
}

#[rgm_ffi_type]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgmAlignmentCoordinateSystem {
    EastingNorthing = 0,
    NorthingEasting = 1,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmLine3 {
    pub start: RgmPoint3,
    pub end: RgmPoint3,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmCircle3 {
    pub plane: RgmPlane,
    pub radius: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmArc3 {
    pub plane: RgmPlane,
    pub radius: f64,
    pub start_angle: f64,
    pub sweep_angle: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmPolycurveSegment {
    pub curve: RgmObjectHandle,
    pub reversed: bool,
}

#[derive(Clone, Debug)]
struct NurbsCurveData {
    core: NurbsCurveCore,
    closed: bool,
    fit_points: Vec<RgmPoint3>,
    arc_length: ArcLengthCache,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct LineData {
    line: RgmLine3,
    canonical_nurbs: NurbsCurveData,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ArcData {
    arc: RgmArc3,
    canonical_nurbs: NurbsCurveData,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct CircleData {
    circle: RgmCircle3,
    canonical_nurbs: NurbsCurveData,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct PolylineData {
    closed: bool,
    points: Vec<RgmPoint3>,
    canonical_nurbs: NurbsCurveData,
}

#[derive(Clone, Debug)]
struct PolycurveSegmentData {
    curve: RgmObjectHandle,
    reversed: bool,
    length: f64,
}

#[derive(Clone, Debug)]
struct PolycurveData {
    segments: Vec<PolycurveSegmentData>,
    cumulative_lengths: Vec<f64>,
    total_length: f64,
}

#[derive(Clone, Debug)]
struct MeshData {
    vertices: Vec<RgmPoint3>,
    triangles: Vec<[u32; 3]>,
    transform: [[f64; 4]; 4],
}

#[derive(Clone, Debug)]
enum CurveData {
    NurbsCurve(NurbsCurveData),
    Line(LineData),
    Arc(ArcData),
    Circle(CircleData),
    Polyline(PolylineData),
    Polycurve(PolycurveData),
}

#[derive(Clone, Debug)]
enum GeometryObject {
    Curve(CurveData),
    Mesh(MeshData),
}

#[derive(Default)]
struct SessionState {
    objects: HashMap<u64, GeometryObject>,
    mesh_accels: HashMap<u64, MeshAccelCache>,
    last_error_code: RgmStatus,
    last_error_message: String,
}

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_OBJECT_ID: AtomicU64 = AtomicU64::new(1);
static SESSIONS: Lazy<Mutex<HashMap<u64, SessionState>>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl Default for RgmStatus {
    fn default() -> Self {
        Self::Ok
    }
}

fn set_error(session_id: u64, status: RgmStatus, message: impl Into<String>) {
    if let Ok(mut sessions) = SESSIONS.lock() {
        if let Some(session) = sessions.get_mut(&session_id) {
            session.last_error_code = status;
            session.last_error_message = message.into();
        }
    }
}

fn clear_error(session_id: u64) {
    if let Ok(mut sessions) = SESSIONS.lock() {
        if let Some(session) = sessions.get_mut(&session_id) {
            session.last_error_code = RgmStatus::Ok;
            session.last_error_message.clear();
        }
    }
}

fn with_session_mut<T>(
    session: RgmKernelHandle,
    f: impl FnOnce(&mut SessionState) -> Result<T, RgmStatus>,
) -> Result<T, RgmStatus> {
    let mut sessions = SESSIONS.lock().map_err(|_| RgmStatus::InternalError)?;
    let state = sessions.get_mut(&session.0).ok_or(RgmStatus::NotFound)?;
    f(state)
}

fn write_point(out: *mut RgmPoint3, value: RgmPoint3) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }

    // SAFETY: pointer nullability checked above.
    unsafe {
        *out = value;
    }

    Ok(())
}

fn write_vec(out: *mut RgmVec3, value: RgmVec3) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }

    // SAFETY: pointer nullability checked above.
    unsafe {
        *out = value;
    }

    Ok(())
}

fn write_plane(out: *mut RgmPlane, value: RgmPlane) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }

    // SAFETY: pointer nullability checked above.
    unsafe {
        *out = value;
    }

    Ok(())
}

fn write_object_handle(out: *mut RgmObjectHandle, value: RgmObjectHandle) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }

    // SAFETY: pointer nullability checked above.
    unsafe {
        *out = value;
    }

    Ok(())
}

fn write_f64(out: *mut f64, value: f64) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }

    // SAFETY: pointer nullability checked above.
    unsafe {
        *out = value;
    }

    Ok(())
}

fn write_u32(out: *mut u32, value: u32) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }

    // SAFETY: pointer nullability checked above.
    unsafe {
        *out = value;
    }

    Ok(())
}

fn write_intersection_points(
    out_points: *mut RgmPoint3,
    point_capacity: u32,
    points: &[RgmPoint3],
    out_count: *mut u32,
) -> Result<(), RgmStatus> {
    if out_count.is_null() {
        return Err(RgmStatus::InvalidInput);
    }

    // SAFETY: out_count validated above.
    unsafe {
        *out_count = points.len().try_into().unwrap_or(u32::MAX);
    }

    if point_capacity == 0 {
        return Ok(());
    }
    if out_points.is_null() {
        return Err(RgmStatus::InvalidInput);
    }

    let copy_count = points.len().min(point_capacity as usize);
    for (idx, point) in points.iter().take(copy_count).enumerate() {
        // SAFETY: caller provided output buffer with at least `point_capacity` elements.
        unsafe {
            *out_points.add(idx) = *point;
        }
    }

    Ok(())
}

fn map_err_with_session(session: RgmKernelHandle, status: RgmStatus, message: &str) -> RgmStatus {
    set_error(session.0, status, message);
    status
}

fn distance(a: RgmPoint3, b: RgmPoint3) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dz = b.z - a.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
fn vec_dot(a: RgmVec3, b: RgmVec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[allow(dead_code)]
fn vec_norm(v: RgmVec3) -> f64 {
    vec_dot(v, v).sqrt()
}

#[allow(dead_code)]
fn vec_normalize(v: RgmVec3) -> Option<RgmVec3> {
    let n = vec_norm(v);
    if n <= f64::EPSILON {
        return None;
    }
    Some(RgmVec3 {
        x: v.x / n,
        y: v.y / n,
        z: v.z / n,
    })
}

fn vec_neg(v: RgmVec3) -> RgmVec3 {
    RgmVec3 {
        x: -v.x,
        y: -v.y,
        z: -v.z,
    }
}

fn vec_add(a: RgmVec3, b: RgmVec3) -> RgmVec3 {
    RgmVec3 {
        x: a.x + b.x,
        y: a.y + b.y,
        z: a.z + b.z,
    }
}

fn vec_scale(v: RgmVec3, s: f64) -> RgmVec3 {
    RgmVec3 {
        x: v.x * s,
        y: v.y * s,
        z: v.z * s,
    }
}

fn vec_cross(a: RgmVec3, b: RgmVec3) -> RgmVec3 {
    RgmVec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

fn point_sub(a: RgmPoint3, b: RgmPoint3) -> RgmVec3 {
    RgmVec3 {
        x: a.x - b.x,
        y: a.y - b.y,
        z: a.z - b.z,
    }
}

fn point_add_vec(p: RgmPoint3, v: RgmVec3) -> RgmPoint3 {
    RgmPoint3 {
        x: p.x + v.x,
        y: p.y + v.y,
        z: p.z + v.z,
    }
}

fn matrix_identity() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn matrix_mul(a: [[f64; 4]; 4], b: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut result = [[0.0; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            result[r][c] =
                a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c] + a[r][3] * b[3][c];
        }
    }
    result
}

fn matrix_translation(delta: RgmVec3) -> [[f64; 4]; 4] {
    let mut m = matrix_identity();
    m[0][3] = delta.x;
    m[1][3] = delta.y;
    m[2][3] = delta.z;
    m
}

fn matrix_scale(scale: RgmVec3) -> [[f64; 4]; 4] {
    [
        [scale.x, 0.0, 0.0, 0.0],
        [0.0, scale.y, 0.0, 0.0],
        [0.0, 0.0, scale.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn matrix_rotation(axis: RgmVec3, angle_rad: f64) -> Result<[[f64; 4]; 4], RgmStatus> {
    let unit = vec_normalize(axis).ok_or(RgmStatus::InvalidInput)?;
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let t = 1.0 - c;
    let x = unit.x;
    let y = unit.y;
    let z = unit.z;
    Ok([
        [t * x * x + c, t * x * y - s * z, t * x * z + s * y, 0.0],
        [t * x * y + s * z, t * y * y + c, t * y * z - s * x, 0.0],
        [t * x * z - s * y, t * y * z + s * x, t * z * z + c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

fn matrix_about_pivot(transform: [[f64; 4]; 4], pivot: RgmPoint3) -> [[f64; 4]; 4] {
    let to_pivot = matrix_translation(RgmVec3 {
        x: pivot.x,
        y: pivot.y,
        z: pivot.z,
    });
    let from_pivot = matrix_translation(RgmVec3 {
        x: -pivot.x,
        y: -pivot.y,
        z: -pivot.z,
    });
    matrix_mul(to_pivot, matrix_mul(transform, from_pivot))
}

fn matrix_apply_point(matrix: [[f64; 4]; 4], point: RgmPoint3) -> RgmPoint3 {
    RgmPoint3 {
        x: matrix[0][0] * point.x + matrix[0][1] * point.y + matrix[0][2] * point.z + matrix[0][3],
        y: matrix[1][0] * point.x + matrix[1][1] * point.y + matrix[1][2] * point.z + matrix[1][3],
        z: matrix[2][0] * point.x + matrix[2][1] * point.y + matrix[2][2] * point.z + matrix[2][3],
    }
}

fn wrap_angle_0_2pi(angle: f64) -> f64 {
    let two_pi = 2.0 * PI;
    let mut wrapped = angle % two_pi;
    if wrapped < 0.0 {
        wrapped += two_pi;
    }
    wrapped
}

fn parse_coordinate_system(value: i32) -> Result<RgmAlignmentCoordinateSystem, RgmStatus> {
    match value {
        0 => Ok(RgmAlignmentCoordinateSystem::EastingNorthing),
        1 => Ok(RgmAlignmentCoordinateSystem::NorthingEasting),
        _ => Err(RgmStatus::InvalidInput),
    }
}

fn convert_point_coordinate_system(
    point: RgmPoint3,
    source: RgmAlignmentCoordinateSystem,
    target: RgmAlignmentCoordinateSystem,
) -> RgmPoint3 {
    if source == target {
        return point;
    }

    RgmPoint3 {
        x: point.y,
        y: point.x,
        z: point.z,
    }
}

fn arc_sweep_from_start_mid_end(
    start_angle: f64,
    mid_angle: f64,
    end_angle: f64,
    angle_tol: f64,
) -> Result<f64, RgmStatus> {
    let end_ccw = wrap_angle_0_2pi(end_angle - start_angle);
    let mid_ccw = wrap_angle_0_2pi(mid_angle - start_angle);
    let eps = angle_tol.max(1e-12);
    if end_ccw <= eps {
        return Err(RgmStatus::InvalidInput);
    }

    let sweep = if mid_ccw <= end_ccw + eps {
        end_ccw
    } else {
        end_ccw - 2.0 * PI
    };
    if sweep.abs() <= eps {
        return Err(RgmStatus::InvalidInput);
    }
    Ok(sweep)
}

fn build_arc_from_three_points(
    start: RgmPoint3,
    mid: RgmPoint3,
    end: RgmPoint3,
    tol: RgmToleranceContext,
) -> Result<RgmArc3, RgmStatus> {
    let eps = tol.abs_tol.max(1e-12);
    if distance(start, mid) <= eps || distance(mid, end) <= eps || distance(start, end) <= eps {
        return Err(RgmStatus::InvalidInput);
    }

    let ab = point_sub(mid, start);
    let ac = point_sub(end, start);
    let normal = vec_cross(ab, ac);
    let normal_len2 = vec_dot(normal, normal);
    if normal_len2 <= eps * eps {
        return Err(RgmStatus::InvalidInput);
    }

    let term1 = vec_scale(vec_cross(ac, normal), vec_dot(ab, ab));
    let term2 = vec_scale(vec_cross(normal, ab), vec_dot(ac, ac));
    let center_offset = vec_scale(vec_add(term1, term2), 1.0 / (2.0 * normal_len2));
    let center = point_add_vec(start, center_offset);
    let radius = distance(center, start);
    if radius <= eps {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let z_axis = vec_normalize(normal).ok_or(RgmStatus::DegenerateGeometry)?;
    let x_axis = vec_normalize(point_sub(start, center)).ok_or(RgmStatus::DegenerateGeometry)?;
    let y_axis = vec_normalize(vec_cross(z_axis, x_axis)).ok_or(RgmStatus::DegenerateGeometry)?;

    let mid_vec = point_sub(mid, center);
    let end_vec = point_sub(end, center);
    let mid_angle = vec_dot(mid_vec, y_axis).atan2(vec_dot(mid_vec, x_axis));
    let end_angle = vec_dot(end_vec, y_axis).atan2(vec_dot(end_vec, x_axis));
    let sweep = arc_sweep_from_start_mid_end(0.0, mid_angle, end_angle, tol.angle_tol)?;

    Ok(RgmArc3 {
        plane: RgmPlane {
            origin: center,
            x_axis,
            y_axis,
            z_axis,
        },
        radius,
        start_angle: 0.0,
        sweep_angle: sweep,
    })
}

fn build_arc_from_start_end_angles(
    plane: RgmPlane,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    tol: RgmToleranceContext,
) -> Result<RgmArc3, RgmStatus> {
    let sweep = end_angle - start_angle;
    if sweep.abs() <= tol.angle_tol.max(1e-12) {
        return Err(RgmStatus::InvalidInput);
    }

    Ok(RgmArc3 {
        plane,
        radius,
        start_angle,
        sweep_angle: sweep,
    })
}

fn dedup_closed_points(mut points: Vec<RgmPoint3>, tol: f64) -> Vec<RgmPoint3> {
    if points.len() > 1 {
        let first = points[0];
        let last = points[points.len() - 1];
        if distance(first, last) <= tol {
            points.pop();
        }
    }
    points
}

fn chord_length_params(points: &[RgmPoint3]) -> Vec<f64> {
    if points.len() <= 1 {
        return vec![0.0; points.len()];
    }

    let mut cumulative = vec![0.0; points.len()];
    let mut total = 0.0;

    for i in 1..points.len() {
        total += distance(points[i - 1], points[i]);
        cumulative[i] = total;
    }

    if total <= f64::EPSILON {
        return (0..points.len())
            .map(|idx| idx as f64 / (points.len() - 1) as f64)
            .collect();
    }

    cumulative.into_iter().map(|v| v / total).collect()
}

fn clamped_open_knots(point_count: usize, degree: usize, params: &[f64]) -> Vec<f64> {
    let knot_count = point_count + degree + 1;
    let mut knots = vec![0.0; knot_count];

    for k in 0..=degree {
        knots[k] = 0.0;
        knots[knot_count - 1 - k] = 1.0;
    }

    if point_count > degree + 1 {
        let n = point_count - 1;
        let interior_count = n - degree;

        for j in 1..=interior_count {
            let mut sum = 0.0;
            for i in j..(j + degree) {
                sum += params[i];
            }
            knots[j + degree] = sum / degree as f64;
        }
    }

    knots
}

fn uniform_periodic_knots(control_count: usize, degree: usize) -> Vec<f64> {
    let knot_count = control_count + degree + 1;
    (0..knot_count).map(|idx| idx as f64).collect()
}

fn build_nurbs_from_core(
    degree: usize,
    periodic: bool,
    closed: bool,
    control_points: Vec<RgmPoint3>,
    weights: Vec<f64>,
    knots: Vec<f64>,
    tol: RgmToleranceContext,
    fit_points: Vec<RgmPoint3>,
) -> Result<NurbsCurveData, RgmStatus> {
    let core = NurbsCurveCore {
        degree,
        periodic,
        control_points,
        weights,
        knots,
        u_start: 0.0,
        u_end: 0.0,
        tol,
    };

    build_nurbs_from_core_auto_domain(core, closed, fit_points)
}

fn build_nurbs_from_core_auto_domain(
    mut core: NurbsCurveCore,
    closed: bool,
    fit_points: Vec<RgmPoint3>,
) -> Result<NurbsCurveData, RgmStatus> {
    let ctrl_count = core.control_points.len();
    if ctrl_count == 0 || ctrl_count <= core.degree {
        return Err(RgmStatus::InvalidInput);
    }

    core.u_start = core.knots[core.degree];
    core.u_end = core.knots[ctrl_count];

    validate_curve(&core)?;
    let arc_length = build_arc_length_cache(&core)?;

    Ok(NurbsCurveData {
        core,
        closed,
        fit_points,
        arc_length,
    })
}

fn build_periodic_nurbs_from_base(
    base_points: &[RgmPoint3],
    base_weights: &[f64],
    degree: usize,
    tol: RgmToleranceContext,
    closed: bool,
    fit_points: Vec<RgmPoint3>,
) -> Result<NurbsCurveData, RgmStatus> {
    if base_points.len() != base_weights.len() || base_points.len() <= degree {
        return Err(RgmStatus::InvalidInput);
    }

    let mut control_points = base_points.to_vec();
    let mut weights = base_weights.to_vec();

    for idx in 0..degree {
        control_points.push(base_points[idx]);
        weights.push(base_weights[idx]);
    }

    let knots = uniform_periodic_knots(control_points.len(), degree);
    build_nurbs_from_core(
        degree,
        true,
        closed,
        control_points,
        weights,
        knots,
        tol,
        fit_points,
    )
}

fn build_open_nurbs_from_points(
    points: &[RgmPoint3],
    degree: usize,
    tol: RgmToleranceContext,
    fit_points: Vec<RgmPoint3>,
) -> Result<NurbsCurveData, RgmStatus> {
    if points.len() <= degree {
        return Err(RgmStatus::InvalidInput);
    }

    let params = chord_length_params(points);
    let knots = clamped_open_knots(points.len(), degree, &params);
    let weights = vec![1.0; points.len()];

    build_nurbs_from_core(
        degree,
        false,
        false,
        points.to_vec(),
        weights,
        knots,
        tol,
        fit_points,
    )
}

fn build_nurbs_from_fit_points(
    points: &[RgmPoint3],
    degree: u32,
    closed: bool,
    tol: RgmToleranceContext,
) -> Result<NurbsCurveData, RgmStatus> {
    if points.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }
    if degree == 0 {
        return Err(RgmStatus::InvalidInput);
    }

    let mut fit_points = points.to_vec();
    if closed {
        fit_points = dedup_closed_points(fit_points, tol.abs_tol.max(0.0));
    }

    let degree = degree as usize;
    if fit_points.len() <= degree {
        return Err(RgmStatus::InvalidInput);
    }

    if closed {
        let weights = vec![1.0; fit_points.len()];
        return build_periodic_nurbs_from_base(
            &fit_points,
            &weights,
            degree,
            tol,
            true,
            fit_points.clone(),
        );
    }

    build_open_nurbs_from_points(&fit_points, degree, tol, fit_points.clone())
}

fn build_line_nurbs(line: RgmLine3, tol: RgmToleranceContext) -> Result<NurbsCurveData, RgmStatus> {
    let points = vec![line.start, line.end];
    build_open_nurbs_from_points(&points, 1, tol, points.clone())
}

fn build_polyline_nurbs(
    points: &[RgmPoint3],
    closed: bool,
    tol: RgmToleranceContext,
) -> Result<NurbsCurveData, RgmStatus> {
    if points.len() < 2 {
        return Err(RgmStatus::InvalidInput);
    }

    let mut data = points.to_vec();
    if closed {
        data = dedup_closed_points(data, tol.abs_tol.max(0.0));
        if data.len() < 2 {
            return Err(RgmStatus::InvalidInput);
        }
        let weights = vec![1.0; data.len()];
        return build_periodic_nurbs_from_base(&data, &weights, 1, tol, true, data.clone());
    }

    build_open_nurbs_from_points(&data, 1, tol, data.clone())
}

fn build_arc_nurbs(arc: RgmArc3, tol: RgmToleranceContext) -> Result<NurbsCurveData, RgmStatus> {
    if arc.radius <= tol.abs_tol.max(1e-12) {
        return Err(RgmStatus::InvalidInput);
    }
    if arc.sweep_angle.abs() <= tol.angle_tol.max(1e-12) {
        return Err(RgmStatus::InvalidInput);
    }

    let (x_axis, y_axis) = orthonormalize_plane_axes(arc.plane)?;
    let center = arc.plane.origin;

    let segments = (arc.sweep_angle.abs() / FRAC_PI_2).ceil().max(1.0) as usize;
    let delta = arc.sweep_angle / segments as f64;

    let mut points = Vec::with_capacity(2 * segments + 1);
    let mut weights = Vec::with_capacity(2 * segments + 1);

    for seg in 0..segments {
        let a0 = arc.start_angle + seg as f64 * delta;
        let a1 = a0 + delta;
        let am = 0.5 * (a0 + a1);
        let w_mid = (0.5 * delta).cos();
        if w_mid.abs() <= 1e-12 {
            return Err(RgmStatus::NumericalFailure);
        }

        let p0 = point_from_frame(
            center,
            x_axis,
            y_axis,
            arc.radius * a0.cos(),
            arc.radius * a0.sin(),
        );
        let p1 = point_from_frame(
            center,
            x_axis,
            y_axis,
            (arc.radius / w_mid) * am.cos(),
            (arc.radius / w_mid) * am.sin(),
        );
        let p2 = point_from_frame(
            center,
            x_axis,
            y_axis,
            arc.radius * a1.cos(),
            arc.radius * a1.sin(),
        );

        if seg == 0 {
            points.push(p0);
            weights.push(1.0);
        }

        points.push(p1);
        weights.push(w_mid);
        points.push(p2);
        weights.push(1.0);
    }

    let degree = 2;
    let mut knots = vec![0.0; points.len() + degree + 1];
    let mut cursor = 0_usize;
    for _ in 0..=degree {
        knots[cursor] = 0.0;
        cursor += 1;
    }
    for idx in 1..segments {
        knots[cursor] = idx as f64;
        cursor += 1;
        knots[cursor] = idx as f64;
        cursor += 1;
    }
    for _ in 0..=degree {
        knots[cursor] = segments as f64;
        cursor += 1;
    }

    for knot in &mut knots {
        *knot /= segments as f64;
    }

    build_nurbs_from_core(
        degree,
        false,
        false,
        points,
        weights,
        knots,
        tol,
        Vec::new(),
    )
}

fn build_circle_nurbs(
    circle: RgmCircle3,
    tol: RgmToleranceContext,
) -> Result<NurbsCurveData, RgmStatus> {
    let arc = RgmArc3 {
        plane: circle.plane,
        radius: circle.radius,
        start_angle: 0.0,
        sweep_angle: 2.0 * PI,
    };
    let mut nurbs = build_arc_nurbs(arc, tol)?;
    nurbs.closed = true;
    Ok(nurbs)
}

fn reverse_open_nurbs(curve: &NurbsCurveData) -> Result<NurbsCurveData, RgmStatus> {
    if curve.core.periodic {
        return Err(RgmStatus::InvalidInput);
    }

    let mut control_points = curve.core.control_points.clone();
    control_points.reverse();

    let mut weights = curve.core.weights.clone();
    weights.reverse();

    let mut knots = vec![0.0; curve.core.knots.len()];
    for (idx, value) in curve.core.knots.iter().enumerate() {
        knots[curve.core.knots.len() - 1 - idx] = curve.core.u_start + curve.core.u_end - value;
    }

    build_nurbs_from_core(
        curve.core.degree,
        false,
        curve.closed,
        control_points,
        weights,
        knots,
        curve.core.tol,
        curve.fit_points.clone(),
    )
}

fn periodic_to_open_nurbs(curve: &NurbsCurveData) -> Result<NurbsCurveData, RgmStatus> {
    if !curve.core.periodic {
        return Ok(curve.clone());
    }

    let control_count = curve.core.control_points.len();
    let degree = curve.core.degree.max(1);
    let sample_count = (control_count * 12).max(degree * 18 + 16);
    let span = curve.core.u_end - curve.core.u_start;
    if span <= curve.core.tol.abs_tol.max(1e-12) {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let mut points = Vec::with_capacity(sample_count + 1);
    for idx in 0..=sample_count {
        let t = idx as f64 / sample_count as f64;
        let mut u = curve.core.u_start + t * span;
        if idx == sample_count {
            u = curve.core.u_end - span * 1e-9;
        }
        let eval = eval_nurbs_u(&curve.core, u)?;
        points.push(eval.point);
    }

    build_open_nurbs_from_points(
        &points,
        curve.core.degree,
        curve.core.tol,
        curve.fit_points.clone(),
    )
}

#[derive(Clone, Copy, Debug)]
struct HomogeneousPoint {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl HomogeneousPoint {
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }

    fn scale(self, value: f64) -> Self {
        Self {
            x: self.x * value,
            y: self.y * value,
            z: self.z * value,
            w: self.w * value,
        }
    }

    fn blend(left: Self, right: Self, right_weight: f64) -> Self {
        left.scale(1.0 - right_weight)
            .add(right.scale(right_weight))
    }
}

fn to_homogeneous(point: RgmPoint3, weight: f64) -> HomogeneousPoint {
    HomogeneousPoint {
        x: point.x * weight,
        y: point.y * weight,
        z: point.z * weight,
        w: weight,
    }
}

fn from_homogeneous(value: HomogeneousPoint, eps: f64) -> Result<(RgmPoint3, f64), RgmStatus> {
    if value.w.abs() <= eps {
        return Err(RgmStatus::NumericalFailure);
    }

    Ok((
        RgmPoint3 {
            x: value.x / value.w,
            y: value.y / value.w,
            z: value.z / value.w,
        },
        value.w,
    ))
}

fn knot_multiplicity(knots: &[f64], knot: f64, eps: f64) -> usize {
    knots
        .iter()
        .filter(|value| (**value - knot).abs() <= eps)
        .count()
}

fn insert_knot_once_homogeneous(
    degree: usize,
    knots: &[f64],
    control: &[HomogeneousPoint],
    knot: f64,
    eps: f64,
) -> Result<(Vec<f64>, Vec<HomogeneousPoint>), RgmStatus> {
    if control.is_empty() || degree == 0 {
        return Err(RgmStatus::InvalidInput);
    }

    let n = control.len() - 1;
    let expected_knot_count = n + degree + 2;
    if knots.len() != expected_knot_count {
        return Err(RgmStatus::InvalidInput);
    }

    let m = expected_knot_count - 1;
    let span = math::basis::find_span(n, degree, knot, knots)?;
    let multiplicity = knot_multiplicity(knots, knot, eps);
    if multiplicity >= degree + 1 {
        return Err(RgmStatus::InvalidInput);
    }

    let mut next_knots = vec![0.0; knots.len() + 1];
    let mut next_control = vec![
        HomogeneousPoint {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
        control.len() + 1
    ];

    next_knots[..=span].copy_from_slice(&knots[..=span]);
    next_knots[span + 1] = knot;
    next_knots[span + 2..=m + 1].copy_from_slice(&knots[span + 1..=m]);

    let left_static_end = span.saturating_sub(degree);
    next_control[..=left_static_end].copy_from_slice(&control[..=left_static_end]);

    let right_start = span.saturating_sub(multiplicity);
    next_control[right_start + 1..=n + 1].copy_from_slice(&control[right_start..=n]);

    let blend_start = span.saturating_sub(degree) + 1;
    let blend_end = span.saturating_sub(multiplicity);
    if blend_start <= blend_end {
        for i in blend_start..=blend_end {
            let denom = knots[i + degree] - knots[i];
            let alpha = if denom.abs() <= eps {
                0.0
            } else {
                (knot - knots[i]) / denom
            };
            next_control[i] = HomogeneousPoint::blend(control[i - 1], control[i], alpha);
        }
    }

    Ok((next_knots, next_control))
}

fn elevate_bezier_homogeneous(
    control: &[HomogeneousPoint],
    target_degree: usize,
) -> Result<Vec<HomogeneousPoint>, RgmStatus> {
    if control.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }

    let mut current = control.to_vec();
    let mut degree = current.len() - 1;
    if target_degree < degree {
        return Err(RgmStatus::InvalidInput);
    }

    while degree < target_degree {
        let mut elevated = vec![
            HomogeneousPoint {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            };
            degree + 2
        ];
        elevated[0] = current[0];
        elevated[degree + 1] = current[degree];
        for i in 1..=degree {
            let alpha = i as f64 / (degree + 1) as f64;
            elevated[i] = current[i - 1]
                .scale(alpha)
                .add(current[i].scale(1.0 - alpha));
        }

        current = elevated;
        degree += 1;
    }

    Ok(current)
}

fn elevate_open_nurbs_to_degree(
    curve: &NurbsCurveData,
    target_degree: usize,
) -> Result<NurbsCurveData, RgmStatus> {
    if curve.core.periodic || target_degree < curve.core.degree {
        return Err(RgmStatus::InvalidInput);
    }
    if target_degree == curve.core.degree {
        return Ok(curve.clone());
    }

    let degree = curve.core.degree;
    let control_count = curve.core.control_points.len();
    if control_count <= degree {
        return Err(RgmStatus::InvalidInput);
    }

    let expected_knot_count = control_count + degree + 1;
    if curve.core.knots.len() != expected_knot_count {
        return Err(RgmStatus::InvalidInput);
    }

    let mut knots = curve.core.knots.clone();
    let mut control: Vec<HomogeneousPoint> = curve
        .core
        .control_points
        .iter()
        .copied()
        .zip(curve.core.weights.iter().copied())
        .map(|(point, weight)| to_homogeneous(point, weight))
        .collect();

    let eps = curve.core.tol.abs_tol.max(1e-12);
    let n = control_count - 1;
    let u_start = curve.core.knots[degree];
    let u_end = curve.core.knots[n + 1];

    let mut internal_knots = Vec::new();
    let mut idx = degree + 1;
    while idx <= n {
        let knot = curve.core.knots[idx];
        if knot > u_start + eps && knot < u_end - eps {
            internal_knots.push(knot);
        }
        idx += 1;
        while idx <= n && (curve.core.knots[idx] - knot).abs() <= eps {
            idx += 1;
        }
    }

    for knot in internal_knots {
        loop {
            let multiplicity = knot_multiplicity(&knots, knot, eps);
            if multiplicity >= degree {
                break;
            }
            let (next_knots, next_control) =
                insert_knot_once_homogeneous(degree, &knots, &control, knot, eps)?;
            knots = next_knots;
            control = next_control;
        }
    }

    let n_after = control.len() - 1;
    let mut span_indices = Vec::new();
    for i in degree..=n_after {
        if knots[i + 1] - knots[i] > eps {
            span_indices.push(i);
        }
    }
    if span_indices.is_empty() {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let mut boundaries = Vec::with_capacity(span_indices.len() + 1);
    let mut boundary_shared = Vec::with_capacity(span_indices.len().saturating_sub(1));
    boundaries.push(knots[span_indices[0]]);

    let mut elevated_control = Vec::new();
    let mut prev_end_idx: Option<usize> = None;
    for span_idx in span_indices {
        let segment_start = span_idx - degree;
        let segment_end = span_idx;
        let segment = &control[segment_start..=segment_end];
        let elevated_segment = elevate_bezier_homogeneous(segment, target_degree)?;

        if let Some(prev_end) = prev_end_idx {
            let shared = segment_start == prev_end;
            boundary_shared.push(shared);
            if shared {
                elevated_control.extend_from_slice(&elevated_segment[1..]);
            } else {
                elevated_control.extend_from_slice(&elevated_segment);
            }
        } else {
            elevated_control.extend_from_slice(&elevated_segment);
        }

        prev_end_idx = Some(segment_end);
        boundaries.push(knots[span_idx + 1]);
    }

    let mut elevated_knots = Vec::with_capacity(elevated_control.len() + target_degree + 1);
    for _ in 0..=target_degree {
        elevated_knots.push(boundaries[0]);
    }
    for (idx, boundary) in boundaries
        .iter()
        .take(boundaries.len().saturating_sub(1))
        .skip(1)
        .enumerate()
    {
        let mult = if boundary_shared[idx] {
            target_degree
        } else {
            target_degree + 1
        };
        for _ in 0..mult {
            elevated_knots.push(*boundary);
        }
    }
    for _ in 0..=target_degree {
        elevated_knots.push(boundaries[boundaries.len() - 1]);
    }

    let denom_eps = curve.core.tol.abs_tol.max(1e-14);
    let mut elevated_points = Vec::with_capacity(elevated_control.len());
    let mut elevated_weights = Vec::with_capacity(elevated_control.len());
    for value in elevated_control {
        let (point, weight) = from_homogeneous(value, denom_eps)?;
        elevated_points.push(point);
        elevated_weights.push(weight);
    }

    build_nurbs_from_core(
        target_degree,
        false,
        curve.closed,
        elevated_points,
        elevated_weights,
        elevated_knots,
        curve.core.tol,
        curve.fit_points.clone(),
    )
}

fn reparameterize_open_nurbs(
    curve: &NurbsCurveData,
    new_start: f64,
    new_end: f64,
) -> Result<NurbsCurveData, RgmStatus> {
    if curve.core.periodic {
        return Err(RgmStatus::InvalidInput);
    }

    let old_start = curve.core.u_start;
    let old_end = curve.core.u_end;
    let old_span = old_end - old_start;
    let new_span = new_end - new_start;
    let eps = curve.core.tol.abs_tol.max(1e-12);

    if old_span <= eps || new_span <= eps {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let scale = new_span / old_span;
    let offset = new_start - scale * old_start;
    let knots: Vec<f64> = curve
        .core
        .knots
        .iter()
        .map(|value| scale * *value + offset)
        .collect();

    build_nurbs_from_core(
        curve.core.degree,
        false,
        curve.closed,
        curve.core.control_points.clone(),
        curve.core.weights.clone(),
        knots,
        curve.core.tol,
        curve.fit_points.clone(),
    )
}

fn curve_canonical_nurbs(curve: &CurveData) -> Option<&NurbsCurveData> {
    match curve {
        CurveData::NurbsCurve(data) => Some(data),
        CurveData::Line(data) => Some(&data.canonical_nurbs),
        CurveData::Arc(data) => Some(&data.canonical_nurbs),
        CurveData::Circle(data) => Some(&data.canonical_nurbs),
        CurveData::Polyline(data) => Some(&data.canonical_nurbs),
        CurveData::Polycurve(_) => None,
    }
}

fn find_curve<'a>(
    state: &'a SessionState,
    object: RgmObjectHandle,
) -> Result<&'a CurveData, RgmStatus> {
    match state.objects.get(&object.0) {
        Some(GeometryObject::Curve(curve)) => Ok(curve),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

fn find_mesh<'a>(
    state: &'a SessionState,
    object: RgmObjectHandle,
) -> Result<&'a MeshData, RgmStatus> {
    match state.objects.get(&object.0) {
        Some(GeometryObject::Mesh(mesh)) => Ok(mesh),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

fn curve_total_length(_state: &SessionState, curve: &CurveData) -> Result<f64, RgmStatus> {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        return Ok(nurbs.arc_length.total_length);
    }

    match curve {
        CurveData::Polycurve(poly) => Ok(poly.total_length),
        _ => Err(RgmStatus::InternalError),
    }
}

fn curve_length_at_normalized_data(
    _state: &SessionState,
    curve: &CurveData,
    t_norm: f64,
) -> Result<f64, RgmStatus> {
    if !(0.0..=1.0).contains(&t_norm) {
        return Err(RgmStatus::OutOfRange);
    }

    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        if nurbs.core.periodic && (t_norm - 1.0).abs() <= f64::EPSILON {
            return Ok(nurbs.arc_length.total_length);
        }

        let u = map_normalized_to_u(&nurbs.core, t_norm)?;
        return length_from_u(&nurbs.core, &nurbs.arc_length, u);
    }

    match curve {
        CurveData::Polycurve(poly) => {
            Ok((t_norm * poly.total_length).clamp(0.0, poly.total_length))
        }
        _ => Err(RgmStatus::InternalError),
    }
}

fn evaluate_curve_at_length_data(
    state: &SessionState,
    curve: &CurveData,
    length: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        let u = u_from_length(&nurbs.core, &nurbs.arc_length, length)?;
        return eval_nurbs_u(&nurbs.core, u);
    }

    match curve {
        CurveData::Polycurve(poly) => evaluate_polycurve_at_length(state, poly, length),
        _ => Err(RgmStatus::InternalError),
    }
}

fn evaluate_curve_at_normalized_data(
    state: &SessionState,
    curve: &CurveData,
    t_norm: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        return eval_nurbs_normalized(&nurbs.core, t_norm);
    }

    match curve {
        CurveData::Polycurve(poly) => evaluate_polycurve_at_normalized(state, poly, t_norm),
        _ => Err(RgmStatus::InternalError),
    }
}

fn evaluate_curve_by_handle_at_length(
    state: &SessionState,
    curve: RgmObjectHandle,
    length: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    let curve_data = find_curve(state, curve)?;
    evaluate_curve_at_length_data(state, curve_data, length)
}

fn curve_point_at_normalized_data(
    state: &SessionState,
    curve: &CurveData,
    t_norm: f64,
) -> Result<RgmPoint3, RgmStatus> {
    let eval = evaluate_curve_at_normalized_data(state, curve, t_norm)?;
    Ok(eval.point)
}

fn curve_abs_tol(state: &SessionState, curve: &CurveData) -> Result<f64, RgmStatus> {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        return Ok(nurbs.core.tol.abs_tol.max(1e-9));
    }

    match curve {
        CurveData::Polycurve(poly) => {
            let mut tol = 1e-9_f64;
            for segment in &poly.segments {
                let segment_curve = find_curve(state, segment.curve)?;
                if let Some(nurbs) = curve_canonical_nurbs(segment_curve) {
                    tol = tol.max(nurbs.core.tol.abs_tol.max(1e-9));
                }
            }
            Ok(tol)
        }
        _ => Ok(1e-9),
    }
}

fn intersect_curve_plane_points_data(
    state: &SessionState,
    curve: &CurveData,
    plane: RgmPlane,
) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let abs_tol = curve_abs_tol(state, curve)?;
    intersect_curve_plane_points(
        |t_norm| curve_point_at_normalized_data(state, curve, t_norm),
        plane,
        abs_tol,
    )
}

fn intersect_curve_curve_points_data(
    state: &SessionState,
    curve_a: &CurveData,
    curve_b: &CurveData,
) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let abs_tol = curve_abs_tol(state, curve_a)?.max(curve_abs_tol(state, curve_b)?);
    intersect_curve_curve_points(
        |t_norm| curve_point_at_normalized_data(state, curve_a, t_norm),
        |t_norm| curve_point_at_normalized_data(state, curve_b, t_norm),
        abs_tol,
    )
}

fn evaluate_polycurve_at_normalized(
    state: &SessionState,
    poly: &PolycurveData,
    t_norm: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    if !(0.0..=1.0).contains(&t_norm) {
        return Err(RgmStatus::OutOfRange);
    }

    if poly.segments.is_empty() {
        return Err(RgmStatus::DegenerateGeometry);
    }

    if poly.total_length <= f64::EPSILON {
        return evaluate_curve_by_handle_at_length(state, poly.segments[0].curve, 0.0);
    }

    let length = t_norm * poly.total_length;
    evaluate_polycurve_at_length(state, poly, length)
}

fn evaluate_polycurve_at_length(
    state: &SessionState,
    poly: &PolycurveData,
    length: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    if length < 0.0 || length > poly.total_length + 1e-10 {
        return Err(RgmStatus::OutOfRange);
    }

    if poly.segments.is_empty() {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let target = length.clamp(0.0, poly.total_length);

    let idx = poly
        .cumulative_lengths
        .iter()
        .position(|v| target <= *v + 1e-10)
        .unwrap_or(poly.cumulative_lengths.len().saturating_sub(1));

    let seg = &poly.segments[idx];
    let seg_start = if idx == 0 {
        0.0
    } else {
        poly.cumulative_lengths[idx - 1]
    };
    let mut local = target - seg_start;

    if seg.reversed {
        local = seg.length - local;
    }

    let mut eval = evaluate_curve_by_handle_at_length(state, seg.curve, local)?;
    if seg.reversed {
        eval.d1 = vec_neg(eval.d1);
    }

    Ok(eval)
}

fn insert_curve(state: &mut SessionState, curve: CurveData) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
    state
        .objects
        .insert(object_id, GeometryObject::Curve(curve));
    RgmObjectHandle(object_id)
}

fn insert_mesh(state: &mut SessionState, mesh: MeshData) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
    state.objects.insert(object_id, GeometryObject::Mesh(mesh));
    RgmObjectHandle(object_id)
}

fn mesh_world_vertices(mesh: &MeshData) -> Vec<RgmPoint3> {
    mesh.vertices
        .iter()
        .copied()
        .map(|point| matrix_apply_point(mesh.transform, point))
        .collect()
}

fn ensure_mesh_accel(state: &mut SessionState, handle: RgmObjectHandle) -> Result<(), RgmStatus> {
    if state.mesh_accels.contains_key(&handle.0) {
        return Ok(());
    }

    let mesh = find_mesh(state, handle)?.clone();
    let world_vertices = mesh_world_vertices(&mesh);
    let triangles = mesh
        .triangles
        .iter()
        .map(|tri| TriangleRecord::from_mesh(&world_vertices, *tri))
        .collect::<Vec<_>>();
    let bvh = MeshBvh::build(&triangles);
    state
        .mesh_accels
        .insert(handle.0, MeshAccelCache { triangles, bvh });
    Ok(())
}

fn build_mesh_from_indexed(
    vertices: &[RgmPoint3],
    flat_indices: &[u32],
) -> Result<MeshData, RgmStatus> {
    if vertices.len() < 3 || flat_indices.len() < 3 || flat_indices.len() % 3 != 0 {
        return Err(RgmStatus::InvalidInput);
    }

    let mut triangles = Vec::with_capacity(flat_indices.len() / 3);
    for tri in flat_indices.chunks_exact(3) {
        if tri[0] as usize >= vertices.len()
            || tri[1] as usize >= vertices.len()
            || tri[2] as usize >= vertices.len()
        {
            return Err(RgmStatus::OutOfRange);
        }
        triangles.push([tri[0], tri[1], tri[2]]);
    }

    if triangles.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }

    Ok(MeshData {
        vertices: vertices.to_vec(),
        triangles,
        transform: matrix_identity(),
    })
}

fn build_box_mesh(center: RgmPoint3, size: RgmVec3) -> Result<MeshData, RgmStatus> {
    if size.x <= 0.0 || size.y <= 0.0 || size.z <= 0.0 {
        return Err(RgmStatus::InvalidInput);
    }

    let hx = size.x * 0.5;
    let hy = size.y * 0.5;
    let hz = size.z * 0.5;
    let vertices = vec![
        RgmPoint3 {
            x: center.x - hx,
            y: center.y - hy,
            z: center.z - hz,
        },
        RgmPoint3 {
            x: center.x + hx,
            y: center.y - hy,
            z: center.z - hz,
        },
        RgmPoint3 {
            x: center.x + hx,
            y: center.y + hy,
            z: center.z - hz,
        },
        RgmPoint3 {
            x: center.x - hx,
            y: center.y + hy,
            z: center.z - hz,
        },
        RgmPoint3 {
            x: center.x - hx,
            y: center.y - hy,
            z: center.z + hz,
        },
        RgmPoint3 {
            x: center.x + hx,
            y: center.y - hy,
            z: center.z + hz,
        },
        RgmPoint3 {
            x: center.x + hx,
            y: center.y + hy,
            z: center.z + hz,
        },
        RgmPoint3 {
            x: center.x - hx,
            y: center.y + hy,
            z: center.z + hz,
        },
    ];
    let flat_indices: [u32; 36] = [
        0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2, 7, 6,
        3, 0, 4, 3, 4, 7,
    ];
    build_mesh_from_indexed(&vertices, &flat_indices)
}

fn build_uv_sphere_mesh(
    center: RgmPoint3,
    radius: f64,
    u_steps: u32,
    v_steps: u32,
) -> Result<MeshData, RgmStatus> {
    if radius <= 0.0 || u_steps < 8 || v_steps < 4 {
        return Err(RgmStatus::InvalidInput);
    }
    let u_steps = u_steps as usize;
    let v_steps = v_steps as usize;
    let mut vertices = Vec::with_capacity((u_steps + 1) * (v_steps + 1));
    for v in 0..=v_steps {
        let vv = v as f64 / v_steps as f64;
        let phi = std::f64::consts::PI * vv;
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        for u in 0..=u_steps {
            let uu = u as f64 / u_steps as f64;
            let theta = 2.0 * std::f64::consts::PI * uu;
            let x = radius * theta.cos() * sin_phi;
            let y = radius * theta.sin() * sin_phi;
            let z = radius * cos_phi;
            vertices.push(RgmPoint3 {
                x: center.x + x,
                y: center.y + y,
                z: center.z + z,
            });
        }
    }

    let ring = u_steps + 1;
    let mut indices = Vec::with_capacity(u_steps * v_steps * 6);
    for v in 0..v_steps {
        for u in 0..u_steps {
            let a = (v * ring + u) as u32;
            let b = a + 1;
            let c = ((v + 1) * ring + u) as u32;
            let d = c + 1;
            if v != 0 {
                indices.extend_from_slice(&[a, c, b]);
            }
            if v != v_steps - 1 {
                indices.extend_from_slice(&[b, c, d]);
            }
        }
    }

    build_mesh_from_indexed(&vertices, &indices)
}

fn build_torus_mesh(
    center: RgmPoint3,
    major_radius: f64,
    minor_radius: f64,
    major_steps: u32,
    minor_steps: u32,
) -> Result<MeshData, RgmStatus> {
    if major_radius <= 0.0 || minor_radius <= 0.0 || major_steps < 8 || minor_steps < 6 {
        return Err(RgmStatus::InvalidInput);
    }

    let major_steps = major_steps as usize;
    let minor_steps = minor_steps as usize;
    let mut vertices = Vec::with_capacity(major_steps * minor_steps);
    for i in 0..major_steps {
        let u = i as f64 / major_steps as f64;
        let theta = 2.0 * std::f64::consts::PI * u;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        for j in 0..minor_steps {
            let v = j as f64 / minor_steps as f64;
            let phi = 2.0 * std::f64::consts::PI * v;
            let cos_p = phi.cos();
            let sin_p = phi.sin();
            let r = major_radius + minor_radius * cos_p;
            vertices.push(RgmPoint3 {
                x: center.x + r * cos_t,
                y: center.y + r * sin_t,
                z: center.z + minor_radius * sin_p,
            });
        }
    }

    let idx = |i: usize, j: usize| -> u32 {
        ((i % major_steps) * minor_steps + (j % minor_steps)) as u32
    };
    let mut indices = Vec::with_capacity(major_steps * minor_steps * 6);
    for i in 0..major_steps {
        for j in 0..minor_steps {
            let a = idx(i, j);
            let b = idx(i + 1, j);
            let c = idx(i, j + 1);
            let d = idx(i + 1, j + 1);
            indices.extend_from_slice(&[a, b, c, c, b, d]);
        }
    }
    build_mesh_from_indexed(&vertices, &indices)
}

fn mesh_copy_vertices_world(
    mesh: &MeshData,
    out_vertices: *mut RgmPoint3,
    vertex_capacity: u32,
    out_count: *mut u32,
) -> Result<(), RgmStatus> {
    let points = mesh_world_vertices(mesh);
    write_intersection_points(out_vertices, vertex_capacity, &points, out_count)
}

fn mesh_copy_indices(
    mesh: &MeshData,
    out_indices: *mut u32,
    index_capacity: u32,
    out_count: *mut u32,
) -> Result<(), RgmStatus> {
    if out_count.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    let flat_count = mesh.triangles.len().saturating_mul(3);
    // SAFETY: out_count validated above.
    unsafe {
        *out_count = flat_count.try_into().unwrap_or(u32::MAX);
    }

    if index_capacity == 0 {
        return Ok(());
    }
    if out_indices.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    let copy_count = flat_count.min(index_capacity as usize);
    for idx in 0..copy_count {
        let tri = idx / 3;
        let lane = idx % 3;
        // SAFETY: out_indices has capacity guaranteed by caller.
        unsafe {
            *out_indices.add(idx) = mesh.triangles[tri][lane];
        }
    }
    Ok(())
}

fn plane_unit_normal(plane: RgmPlane) -> Option<RgmVec3> {
    vec_normalize(plane.z_axis).or_else(|| vec_normalize(vec_cross(plane.x_axis, plane.y_axis)))
}

fn push_unique_point(points: &mut Vec<RgmPoint3>, candidate: RgmPoint3, tol: f64) {
    if points
        .iter()
        .any(|point| distance(*point, candidate) <= tol)
    {
        return;
    }
    points.push(candidate);
}

fn intersect_triangle_plane_segment(
    a: RgmPoint3,
    b: RgmPoint3,
    c: RgmPoint3,
    plane_origin: RgmPoint3,
    plane_normal: RgmVec3,
    tol: f64,
) -> Option<(RgmPoint3, RgmPoint3)> {
    let d0 = vec_dot(point_sub(a, plane_origin), plane_normal);
    let d1 = vec_dot(point_sub(b, plane_origin), plane_normal);
    let d2 = vec_dot(point_sub(c, plane_origin), plane_normal);
    if d0.abs() <= tol && d1.abs() <= tol && d2.abs() <= tol {
        return None;
    }

    let mut points = Vec::new();
    let mut edge_hit = |p0: RgmPoint3, p1: RgmPoint3, s0: f64, s1: f64| {
        if s0.abs() <= tol {
            push_unique_point(&mut points, p0, tol);
        }
        if s1.abs() <= tol {
            push_unique_point(&mut points, p1, tol);
        }
        if (s0 > tol && s1 < -tol) || (s0 < -tol && s1 > tol) {
            let t = s0 / (s0 - s1);
            let segment = point_sub(p1, p0);
            let hit = point_add_vec(p0, vec_scale(segment, t));
            push_unique_point(&mut points, hit, tol);
        }
    };

    edge_hit(a, b, d0, d1);
    edge_hit(b, c, d1, d2);
    edge_hit(c, a, d2, d0);

    if points.len() < 2 {
        return None;
    }

    let mut best = (points[0], points[1]);
    let mut best_len = distance(points[0], points[1]);
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            let len = distance(points[i], points[j]);
            if len > best_len {
                best_len = len;
                best = (points[i], points[j]);
            }
        }
    }
    if best_len <= tol {
        None
    } else {
        Some(best)
    }
}

fn triangle_aabb(points: [RgmPoint3; 3]) -> (RgmPoint3, RgmPoint3) {
    let mut min = points[0];
    let mut max = points[0];
    for point in points.iter().skip(1) {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        min.z = min.z.min(point.z);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        max.z = max.z.max(point.z);
    }
    (min, max)
}

#[derive(Clone, Copy, Debug)]
struct TriangleRecord {
    points: [RgmPoint3; 3],
    min: RgmPoint3,
    max: RgmPoint3,
}

impl TriangleRecord {
    fn from_mesh(vertices: &[RgmPoint3], tri: [u32; 3]) -> Self {
        let points = [
            vertices[tri[0] as usize],
            vertices[tri[1] as usize],
            vertices[tri[2] as usize],
        ];
        let (min, max) = triangle_aabb(points);
        Self { points, min, max }
    }
}

struct MeshAccelCache {
    triangles: Vec<TriangleRecord>,
    bvh: Option<MeshBvh>,
}

#[derive(Clone, Copy)]
struct BvhNode {
    min: RgmPoint3,
    max: RgmPoint3,
    left: Option<usize>,
    right: Option<usize>,
    start: usize,
    count: usize,
}

impl BvhNode {
    fn leaf(min: RgmPoint3, max: RgmPoint3, start: usize, count: usize) -> Self {
        Self {
            min,
            max,
            left: None,
            right: None,
            start,
            count,
        }
    }

    fn branch(min: RgmPoint3, max: RgmPoint3, left: usize, right: usize) -> Self {
        Self {
            min,
            max,
            left: Some(left),
            right: Some(right),
            start: 0,
            count: 0,
        }
    }

    fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

struct MeshBvh {
    root: usize,
    nodes: Vec<BvhNode>,
    tri_indices: Vec<usize>,
}

impl MeshBvh {
    fn build(records: &[TriangleRecord]) -> Option<Self> {
        if records.is_empty() {
            return None;
        }

        let mut tri_indices = (0..records.len()).collect::<Vec<_>>();
        let tri_count = tri_indices.len();
        let mut nodes = Vec::new();
        let root = Self::build_node(records, &mut tri_indices, &mut nodes, 0, tri_count);
        Some(Self {
            root,
            nodes,
            tri_indices,
        })
    }

    fn build_node(
        records: &[TriangleRecord],
        tri_indices: &mut Vec<usize>,
        nodes: &mut Vec<BvhNode>,
        start: usize,
        end: usize,
    ) -> usize {
        let node_idx = nodes.len();
        nodes.push(BvhNode::leaf(
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            start,
            end.saturating_sub(start),
        ));

        let (min, max) = Self::range_bounds(records, &tri_indices[start..end]);
        let count = end.saturating_sub(start);
        const LEAF_TRIANGLES: usize = 8;
        if count <= LEAF_TRIANGLES {
            nodes[node_idx] = BvhNode::leaf(min, max, start, count);
            return node_idx;
        }

        let axis = Self::split_axis(records, &tri_indices[start..end]);
        tri_indices[start..end].sort_by(|a, b| {
            let ca = Self::triangle_centroid_axis(records[*a], axis);
            let cb = Self::triangle_centroid_axis(records[*b], axis);
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = start + count / 2;
        if mid == start || mid == end {
            nodes[node_idx] = BvhNode::leaf(min, max, start, count);
            return node_idx;
        }

        let left = Self::build_node(records, tri_indices, nodes, start, mid);
        let right = Self::build_node(records, tri_indices, nodes, mid, end);
        nodes[node_idx] = BvhNode::branch(min, max, left, right);
        node_idx
    }

    fn range_bounds(records: &[TriangleRecord], indices: &[usize]) -> (RgmPoint3, RgmPoint3) {
        let mut min = records[indices[0]].min;
        let mut max = records[indices[0]].max;
        for &idx in indices.iter().skip(1) {
            let tri = records[idx];
            min.x = min.x.min(tri.min.x);
            min.y = min.y.min(tri.min.y);
            min.z = min.z.min(tri.min.z);
            max.x = max.x.max(tri.max.x);
            max.y = max.y.max(tri.max.y);
            max.z = max.z.max(tri.max.z);
        }
        (min, max)
    }

    fn split_axis(records: &[TriangleRecord], indices: &[usize]) -> usize {
        let mut cmin = Self::triangle_centroid(records[indices[0]]);
        let mut cmax = cmin;
        for &idx in indices.iter().skip(1) {
            let c = Self::triangle_centroid(records[idx]);
            cmin.x = cmin.x.min(c.x);
            cmin.y = cmin.y.min(c.y);
            cmin.z = cmin.z.min(c.z);
            cmax.x = cmax.x.max(c.x);
            cmax.y = cmax.y.max(c.y);
            cmax.z = cmax.z.max(c.z);
        }
        let ex = cmax.x - cmin.x;
        let ey = cmax.y - cmin.y;
        let ez = cmax.z - cmin.z;
        if ex >= ey && ex >= ez {
            0
        } else if ey >= ez {
            1
        } else {
            2
        }
    }

    fn triangle_centroid(record: TriangleRecord) -> RgmPoint3 {
        RgmPoint3 {
            x: (record.min.x + record.max.x) * 0.5,
            y: (record.min.y + record.max.y) * 0.5,
            z: (record.min.z + record.max.z) * 0.5,
        }
    }

    fn triangle_centroid_axis(record: TriangleRecord, axis: usize) -> f64 {
        match axis {
            0 => (record.min.x + record.max.x) * 0.5,
            1 => (record.min.y + record.max.y) * 0.5,
            _ => (record.min.z + record.max.z) * 0.5,
        }
    }
}

fn aabb_node_plane_overlap(
    min: RgmPoint3,
    max: RgmPoint3,
    plane_origin: RgmPoint3,
    plane_normal: RgmVec3,
    tol: f64,
) -> bool {
    let center = RgmPoint3 {
        x: (min.x + max.x) * 0.5,
        y: (min.y + max.y) * 0.5,
        z: (min.z + max.z) * 0.5,
    };
    let half = RgmVec3 {
        x: (max.x - min.x) * 0.5,
        y: (max.y - min.y) * 0.5,
        z: (max.z - min.z) * 0.5,
    };
    let dist = vec_dot(point_sub(center, plane_origin), plane_normal);
    let radius = half.x * plane_normal.x.abs()
        + half.y * plane_normal.y.abs()
        + half.z * plane_normal.z.abs();
    dist.abs() <= radius + tol
}

fn node_span(node: BvhNode) -> f64 {
    (node.max.x - node.min.x).abs()
        + (node.max.y - node.min.y).abs()
        + (node.max.z - node.min.z).abs()
}

fn aabb_overlap(
    a_min: RgmPoint3,
    a_max: RgmPoint3,
    b_min: RgmPoint3,
    b_max: RgmPoint3,
    tol: f64,
) -> bool {
    !(a_max.x < b_min.x - tol
        || b_max.x < a_min.x - tol
        || a_max.y < b_min.y - tol
        || b_max.y < a_min.y - tol
        || a_max.z < b_min.z - tol
        || b_max.z < a_min.z - tol)
}

fn segment_triangle_intersection(
    p0: RgmPoint3,
    p1: RgmPoint3,
    t0: RgmPoint3,
    t1: RgmPoint3,
    t2: RgmPoint3,
    tol: f64,
) -> Option<RgmPoint3> {
    let dir = point_sub(p1, p0);
    let edge1 = point_sub(t1, t0);
    let edge2 = point_sub(t2, t0);
    let pvec = vec_cross(dir, edge2);
    let det = vec_dot(edge1, pvec);
    if det.abs() <= tol {
        return None;
    }
    let inv_det = 1.0 / det;
    let tvec = point_sub(p0, t0);
    let u = vec_dot(tvec, pvec) * inv_det;
    if u < -tol || u > 1.0 + tol {
        return None;
    }
    let qvec = vec_cross(tvec, edge1);
    let v = vec_dot(dir, qvec) * inv_det;
    if v < -tol || u + v > 1.0 + tol {
        return None;
    }
    let t = vec_dot(edge2, qvec) * inv_det;
    if t < -tol || t > 1.0 + tol {
        return None;
    }

    Some(point_add_vec(p0, vec_scale(dir, t)))
}

fn tri_tri_intersection_segment(
    a0: RgmPoint3,
    a1: RgmPoint3,
    a2: RgmPoint3,
    b0: RgmPoint3,
    b1: RgmPoint3,
    b2: RgmPoint3,
    tol: f64,
) -> Option<(RgmPoint3, RgmPoint3)> {
    let mut points = Vec::new();
    let mut collect = |hit: Option<RgmPoint3>| {
        if let Some(point) = hit {
            push_unique_point(&mut points, point, tol * 4.0);
        }
    };

    collect(segment_triangle_intersection(a0, a1, b0, b1, b2, tol));
    collect(segment_triangle_intersection(a1, a2, b0, b1, b2, tol));
    collect(segment_triangle_intersection(a2, a0, b0, b1, b2, tol));
    collect(segment_triangle_intersection(b0, b1, a0, a1, a2, tol));
    collect(segment_triangle_intersection(b1, b2, a0, a1, a2, tol));
    collect(segment_triangle_intersection(b2, b0, a0, a1, a2, tol));

    if points.len() < 2 {
        return None;
    }
    let mut best = (points[0], points[1]);
    let mut best_len = distance(points[0], points[1]);
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            let len = distance(points[i], points[j]);
            if len > best_len {
                best_len = len;
                best = (points[i], points[j]);
            }
        }
    }
    if best_len <= tol {
        None
    } else {
        Some(best)
    }
}

fn rgm_nurbs_interpolate_fit_points_impl(
    session: RgmKernelHandle,
    points: *const RgmPoint3,
    point_count: usize,
    degree: u32,
    closed: bool,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if points.is_null() || out_object.is_null() {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Null pointer passed to constructor",
        );
    }

    if point_count == 0 {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "At least one fit point is required",
        );
    }

    // SAFETY: points is non-null and point_count comes from caller.
    let fit_points = unsafe { std::slice::from_raw_parts(points, point_count) };
    let curve = match build_nurbs_from_fit_points(fit_points, degree, closed, tol) {
        Ok(curve) => curve,
        Err(status) => {
            return map_err_with_session(
                session,
                status,
                "Failed to build NURBS from fit points: validate degree and fit point count",
            )
        }
    };

    let result = with_session_mut(session, |state| {
        let handle = insert_curve(state, CurveData::NurbsCurve(curve));
        write_object_handle(out_object, handle)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Session not found"),
    }
}

fn create_curve_object(
    session: RgmKernelHandle,
    out_object: *mut RgmObjectHandle,
    build: impl FnOnce(&SessionState) -> Result<CurveData, RgmStatus>,
    message: &str,
) -> RgmStatus {
    if out_object.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_object pointer");
    }

    let result = with_session_mut(session, |state| {
        let curve = build(state)?;
        let handle = insert_curve(state, curve);
        write_object_handle(out_object, handle)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, message),
    }
}

fn create_mesh_object(
    session: RgmKernelHandle,
    out_object: *mut RgmObjectHandle,
    build: impl FnOnce(&SessionState) -> Result<MeshData, RgmStatus>,
    message: &str,
) -> RgmStatus {
    if out_object.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_object pointer");
    }

    let result = with_session_mut(session, |state| {
        let mesh = build(state)?;
        let handle = insert_mesh(state, mesh);
        write_object_handle(out_object, handle)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, message),
    }
}

fn rgm_mesh_create_indexed_impl(
    session: RgmKernelHandle,
    vertices: *const RgmPoint3,
    vertex_count: usize,
    indices: *const u32,
    index_count: usize,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if vertices.is_null() || indices.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null mesh buffer pointer");
    }

    // SAFETY: pointers and lengths come from caller.
    let vertices = unsafe { std::slice::from_raw_parts(vertices, vertex_count) };
    // SAFETY: pointers and lengths come from caller.
    let indices = unsafe { std::slice::from_raw_parts(indices, index_count) };

    create_mesh_object(
        session,
        out_object,
        |_| build_mesh_from_indexed(vertices, indices),
        "Mesh construction failed",
    )
}

fn rgm_mesh_create_box_impl(
    session: RgmKernelHandle,
    center: RgmPoint3,
    size: RgmVec3,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_object,
        |_| build_box_mesh(center, size),
        "Mesh box construction failed",
    )
}

fn rgm_mesh_create_uv_sphere_impl(
    session: RgmKernelHandle,
    center: RgmPoint3,
    radius: f64,
    u_steps: u32,
    v_steps: u32,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_object,
        |_| build_uv_sphere_mesh(center, radius, u_steps, v_steps),
        "Mesh UV sphere construction failed",
    )
}

fn rgm_mesh_create_torus_impl(
    session: RgmKernelHandle,
    center: RgmPoint3,
    major_radius: f64,
    minor_radius: f64,
    major_steps: u32,
    minor_steps: u32,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_object,
        |_| build_torus_mesh(center, major_radius, minor_radius, major_steps, minor_steps),
        "Mesh torus construction failed",
    )
}

fn rgm_mesh_transform_impl(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    transform: [[f64; 4]; 4],
    out_object: *mut RgmObjectHandle,
    message: &str,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_object,
        |state| {
            let source = find_mesh(state, mesh)?;
            let mut next = source.clone();
            next.transform = matrix_mul(transform, source.transform);
            Ok(next)
        },
        message,
    )
}

fn rgm_mesh_bake_transform_impl(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_object,
        |state| {
            let source = find_mesh(state, mesh)?;
            let vertices = mesh_world_vertices(source);
            Ok(MeshData {
                vertices,
                triangles: source.triangles.clone(),
                transform: matrix_identity(),
            })
        },
        "Mesh bake transform failed",
    )
}

fn rgm_mesh_boolean_impl(
    session: RgmKernelHandle,
    mesh_a: RgmObjectHandle,
    mesh_b: RgmObjectHandle,
    op: i32,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_object,
        |state| {
            let a = find_mesh(state, mesh_a)?;
            let b = find_mesh(state, mesh_b)?;
            let a_vertices = mesh_world_vertices(a);
            let b_vertices = mesh_world_vertices(b);
            let mut a_pos = Vec::with_capacity(a_vertices.len() * 3);
            for vertex in &a_vertices {
                a_pos.push(vertex.x);
                a_pos.push(vertex.y);
                a_pos.push(vertex.z);
            }
            let mut b_pos = Vec::with_capacity(b_vertices.len() * 3);
            for vertex in &b_vertices {
                b_pos.push(vertex.x);
                b_pos.push(vertex.y);
                b_pos.push(vertex.z);
            }
            let mut a_indices = Vec::with_capacity(a.triangles.len() * 3);
            for tri in &a.triangles {
                a_indices.push(tri[0] as usize);
                a_indices.push(tri[1] as usize);
                a_indices.push(tri[2] as usize);
            }
            let mut b_indices = Vec::with_capacity(b.triangles.len() * 3);
            for tri in &b.triangles {
                b_indices.push(tri[0] as usize);
                b_indices.push(tri[1] as usize);
                b_indices.push(tri[2] as usize);
            }

            let manifold_a =
                Manifold::new(&a_pos, &a_indices).map_err(|_| RgmStatus::DegenerateGeometry)?;
            let manifold_b =
                Manifold::new(&b_pos, &b_indices).map_err(|_| RgmStatus::DegenerateGeometry)?;
            let op = match op {
                0 => BoolOpType::Add,
                1 => BoolOpType::Intersect,
                2 => BoolOpType::Subtract,
                _ => return Err(RgmStatus::InvalidInput),
            };
            let result = compute_boolean(&manifold_a, &manifold_b, op)
                .map_err(|_| RgmStatus::NumericalFailure)?;

            let out_vertices = result
                .ps
                .iter()
                .map(|vertex| RgmPoint3 {
                    x: vertex.x as f64,
                    y: vertex.y as f64,
                    z: vertex.z as f64,
                })
                .collect::<Vec<_>>();
            let out_triangles = result
                .get_indices()
                .iter()
                .map(|tri| {
                    Ok([
                        u32::try_from(tri.x).map_err(|_| RgmStatus::OutOfRange)?,
                        u32::try_from(tri.y).map_err(|_| RgmStatus::OutOfRange)?,
                        u32::try_from(tri.z).map_err(|_| RgmStatus::OutOfRange)?,
                    ])
                })
                .collect::<Result<Vec<[u32; 3]>, RgmStatus>>()?;

            Ok(MeshData {
                vertices: out_vertices,
                triangles: out_triangles,
                transform: matrix_identity(),
            })
        },
        "Mesh boolean failed",
    )
}

fn rgm_curve_create_line_impl(
    session: RgmKernelHandle,
    line: RgmLine3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_curve_object(
        session,
        out_object,
        |_| {
            let canonical_nurbs = build_line_nurbs(line, tol)?;
            Ok(CurveData::Line(LineData {
                line,
                canonical_nurbs,
            }))
        },
        "Line constructor failed",
    )
}

fn rgm_curve_create_circle_impl(
    session: RgmKernelHandle,
    circle: RgmCircle3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_curve_object(
        session,
        out_object,
        |_| {
            let canonical_nurbs = build_circle_nurbs(circle, tol)?;
            Ok(CurveData::Circle(CircleData {
                circle,
                canonical_nurbs,
            }))
        },
        "Circle constructor failed",
    )
}

fn rgm_curve_create_arc_impl(
    session: RgmKernelHandle,
    arc: RgmArc3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_curve_object(
        session,
        out_object,
        |_| {
            let canonical_nurbs = build_arc_nurbs(arc, tol)?;
            Ok(CurveData::Arc(ArcData {
                arc,
                canonical_nurbs,
            }))
        },
        "Arc constructor failed",
    )
}

fn rgm_curve_create_arc_by_angles_impl(
    session: RgmKernelHandle,
    plane: RgmPlane,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    let arc = match build_arc_from_start_end_angles(plane, radius, start_angle, end_angle, tol) {
        Ok(value) => value,
        Err(status) => {
            return map_err_with_session(session, status, "Arc-by-angles constructor failed");
        }
    };

    rgm_curve_create_arc_impl(session, arc, tol, out_object)
}

fn rgm_curve_create_arc_by_3_points_impl(
    session: RgmKernelHandle,
    start: RgmPoint3,
    mid: RgmPoint3,
    end: RgmPoint3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    let arc = match build_arc_from_three_points(start, mid, end, tol) {
        Ok(value) => value,
        Err(status) => {
            return map_err_with_session(session, status, "Arc-by-3-points constructor failed");
        }
    };

    rgm_curve_create_arc_impl(session, arc, tol, out_object)
}

fn rgm_curve_create_polyline_impl(
    session: RgmKernelHandle,
    points: *const RgmPoint3,
    point_count: usize,
    closed: bool,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if points.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null points pointer");
    }

    // SAFETY: pointer and count validated by caller contract.
    let points = unsafe { std::slice::from_raw_parts(points, point_count) };

    create_curve_object(
        session,
        out_object,
        |_| {
            let canonical_nurbs = build_polyline_nurbs(points, closed, tol)?;
            Ok(CurveData::Polyline(PolylineData {
                closed,
                points: points.to_vec(),
                canonical_nurbs,
            }))
        },
        "Polyline constructor failed",
    )
}

fn rgm_curve_create_polycurve_impl(
    session: RgmKernelHandle,
    segments: *const RgmPolycurveSegment,
    segment_count: usize,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if segments.is_null() || segment_count == 0 {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Invalid polycurve segments",
        );
    }

    // SAFETY: pointer/count validated above.
    let segments = unsafe { std::slice::from_raw_parts(segments, segment_count) };

    create_curve_object(
        session,
        out_object,
        |state| {
            let mut segment_data = Vec::with_capacity(segments.len());
            let mut cumulative = Vec::with_capacity(segments.len());
            let mut total = 0.0;

            for seg in segments {
                let curve = find_curve(state, seg.curve)?;
                if matches!(curve, CurveData::Polycurve(_)) {
                    return Err(RgmStatus::InvalidInput);
                }
                let len = curve_total_length(state, curve)?;
                total += len;
                cumulative.push(total);
                segment_data.push(PolycurveSegmentData {
                    curve: seg.curve,
                    reversed: seg.reversed,
                    length: len,
                });
            }

            Ok(CurveData::Polycurve(PolycurveData {
                segments: segment_data,
                cumulative_lengths: cumulative,
                total_length: total,
            }))
        },
        "Polycurve constructor failed",
    )
}

#[rgm_export(ts = "create", receiver = "kernel_static")]
#[no_mangle]
pub extern "C" fn rgm_kernel_create(out_session: *mut RgmKernelHandle) -> RgmStatus {
    if out_session.is_null() {
        return RgmStatus::InvalidInput;
    }

    let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);

    let Ok(mut sessions) = SESSIONS.lock() else {
        return RgmStatus::InternalError;
    };

    sessions.insert(session_id, SessionState::default());

    // SAFETY: out_session is non-null by guard above.
    unsafe {
        *out_session = RgmKernelHandle(session_id);
    }

    RgmStatus::Ok
}

#[rgm_export(ts = "dispose", receiver = "kernel")]
#[no_mangle]
pub extern "C" fn rgm_kernel_destroy(session: RgmKernelHandle) -> RgmStatus {
    let Ok(mut sessions) = SESSIONS.lock() else {
        return RgmStatus::InternalError;
    };

    match sessions.remove(&session.0) {
        Some(_) => RgmStatus::Ok,
        None => RgmStatus::NotFound,
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_alloc(byte_len: usize, align: usize, out_ptr: *mut *mut u8) -> RgmStatus {
    if out_ptr.is_null() {
        return RgmStatus::InvalidInput;
    }

    if byte_len == 0 || align == 0 || !align.is_power_of_two() {
        return RgmStatus::InvalidInput;
    }

    let Ok(layout) = Layout::from_size_align(byte_len, align) else {
        return RgmStatus::InvalidInput;
    };

    // SAFETY: layout is validated above.
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return RgmStatus::InternalError;
    }

    // SAFETY: out_ptr is non-null by guard above.
    unsafe {
        *out_ptr = ptr;
    }

    RgmStatus::Ok
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_alloc_addr(byte_len: usize, align: usize) -> usize {
    if byte_len == 0 || align == 0 || !align.is_power_of_two() {
        return 0;
    }

    let Ok(layout) = Layout::from_size_align(byte_len, align) else {
        return 0;
    };

    // SAFETY: layout is validated above.
    let ptr = unsafe { alloc(layout) };
    ptr as usize
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_dealloc(ptr: *mut u8, byte_len: usize, align: usize) -> RgmStatus {
    if ptr.is_null() {
        return RgmStatus::InvalidInput;
    }

    if byte_len == 0 || align == 0 || !align.is_power_of_two() {
        return RgmStatus::InvalidInput;
    }

    let Ok(layout) = Layout::from_size_align(byte_len, align) else {
        return RgmStatus::InvalidInput;
    };

    // SAFETY: ptr and layout originate from rgm_alloc contract.
    unsafe {
        dealloc(ptr, layout);
    }

    RgmStatus::Ok
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_object_release(
    session: RgmKernelHandle,
    object: RgmObjectHandle,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        if state.objects.remove(&object.0).is_some() {
            state.mesh_accels.remove(&object.0);
            Ok(())
        } else {
            Err(RgmStatus::NotFound)
        }
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Object not found in this session"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_last_error_code(session: RgmKernelHandle, out_code: *mut i32) -> RgmStatus {
    if out_code.is_null() {
        return RgmStatus::InvalidInput;
    }

    let Ok(sessions) = SESSIONS.lock() else {
        return RgmStatus::InternalError;
    };

    let Some(state) = sessions.get(&session.0) else {
        return RgmStatus::NotFound;
    };

    // SAFETY: out_code is non-null by guard above.
    unsafe {
        *out_code = state.last_error_code as i32;
    }

    RgmStatus::Ok
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_last_error_message(
    session: RgmKernelHandle,
    buffer: *mut u8,
    buffer_len: usize,
    out_written: *mut usize,
) -> RgmStatus {
    if out_written.is_null() {
        return RgmStatus::InvalidInput;
    }

    let Ok(sessions) = SESSIONS.lock() else {
        return RgmStatus::InternalError;
    };

    let Some(state) = sessions.get(&session.0) else {
        return RgmStatus::NotFound;
    };

    let message_bytes = state.last_error_message.as_bytes();
    let bytes_to_copy = if buffer.is_null() || buffer_len == 0 {
        0
    } else {
        message_bytes.len().min(buffer_len.saturating_sub(1))
    };

    // SAFETY: out_written is non-null by guard. buffer writes guarded by null and length checks.
    unsafe {
        *out_written = bytes_to_copy;
        if !buffer.is_null() && buffer_len > 0 {
            std::ptr::copy_nonoverlapping(message_bytes.as_ptr(), buffer, bytes_to_copy);
            *buffer.add(bytes_to_copy) = 0;
        }
    }

    RgmStatus::Ok
}

#[rgm_export(ts = "interpolateNurbsFitPoints", receiver = "kernel")]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_nurbs_interpolate_fit_points(
    session: RgmKernelHandle,
    points: *const RgmPoint3,
    point_count: usize,
    degree: u32,
    closed: bool,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_nurbs_interpolate_fit_points_impl(
        session,
        points,
        point_count,
        degree,
        closed,
        tol,
        out_object,
    )
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_nurbs_interpolate_fit_points(
    session: RgmKernelHandle,
    points: *const RgmPoint3,
    point_count: usize,
    degree: u32,
    closed: bool,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if tol.is_null() {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Null tolerance pointer passed to constructor",
        );
    }

    // SAFETY: tol is non-null by guard above.
    let tol = unsafe { *tol };
    rgm_nurbs_interpolate_fit_points_impl(
        session,
        points,
        point_count,
        degree,
        closed,
        tol,
        out_object,
    )
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_nurbs_interpolate_fit_points_ptr_tol(
    session: RgmKernelHandle,
    points: *const RgmPoint3,
    point_count: usize,
    degree: u32,
    closed: bool,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if tol.is_null() {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Null tolerance pointer passed to constructor",
        );
    }

    // SAFETY: tol is non-null by guard above.
    let tol = unsafe { *tol };
    rgm_nurbs_interpolate_fit_points_impl(
        session,
        points,
        point_count,
        degree,
        closed,
        tol,
        out_object,
    )
}

#[rgm_export(ts = "createLine", receiver = "kernel")]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_curve_create_line(
    session: RgmKernelHandle,
    line: RgmLine3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_curve_create_line_impl(session, line, tol, out_object)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_curve_create_line(
    session: RgmKernelHandle,
    line: *const RgmLine3,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if line.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null constructor pointer");
    }
    let line = unsafe { *line };
    let tol = unsafe { *tol };
    rgm_curve_create_line_impl(session, line, tol, out_object)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_curve_create_line_ptr_tol(
    session: RgmKernelHandle,
    line: *const RgmLine3,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if line.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null constructor pointer");
    }
    let line = unsafe { *line };
    let tol = unsafe { *tol };
    rgm_curve_create_line_impl(session, line, tol, out_object)
}

#[rgm_export(ts = "createCircle", receiver = "kernel")]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_curve_create_circle(
    session: RgmKernelHandle,
    circle: RgmCircle3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_curve_create_circle_impl(session, circle, tol, out_object)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_curve_create_circle(
    session: RgmKernelHandle,
    circle: *const RgmCircle3,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if circle.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null constructor pointer");
    }
    let circle = unsafe { *circle };
    let tol = unsafe { *tol };
    rgm_curve_create_circle_impl(session, circle, tol, out_object)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_curve_create_circle_ptr_tol(
    session: RgmKernelHandle,
    circle: *const RgmCircle3,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if circle.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null constructor pointer");
    }
    let circle = unsafe { *circle };
    let tol = unsafe { *tol };
    rgm_curve_create_circle_impl(session, circle, tol, out_object)
}

#[rgm_export(ts = "createArc", receiver = "kernel")]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_curve_create_arc(
    session: RgmKernelHandle,
    arc: RgmArc3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_curve_create_arc_impl(session, arc, tol, out_object)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_curve_create_arc(
    session: RgmKernelHandle,
    arc: *const RgmArc3,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if arc.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null constructor pointer");
    }
    let arc = unsafe { *arc };
    let tol = unsafe { *tol };
    rgm_curve_create_arc_impl(session, arc, tol, out_object)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_curve_create_arc_ptr_tol(
    session: RgmKernelHandle,
    arc: *const RgmArc3,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if arc.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null constructor pointer");
    }
    let arc = unsafe { *arc };
    let tol = unsafe { *tol };
    rgm_curve_create_arc_impl(session, arc, tol, out_object)
}

#[rgm_export(ts = "createArcByAngles", receiver = "kernel")]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_curve_create_arc_by_angles(
    session: RgmKernelHandle,
    plane: RgmPlane,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_curve_create_arc_by_angles_impl(
        session,
        plane,
        radius,
        start_angle,
        end_angle,
        tol,
        out_object,
    )
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_curve_create_arc_by_angles(
    session: RgmKernelHandle,
    plane: *const RgmPlane,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if plane.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null constructor pointer");
    }
    let plane = unsafe { *plane };
    let tol = unsafe { *tol };
    rgm_curve_create_arc_by_angles_impl(
        session,
        plane,
        radius,
        start_angle,
        end_angle,
        tol,
        out_object,
    )
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_curve_create_arc_by_angles_ptr_tol(
    session: RgmKernelHandle,
    plane: *const RgmPlane,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if plane.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null constructor pointer");
    }
    let plane = unsafe { *plane };
    let tol = unsafe { *tol };
    rgm_curve_create_arc_by_angles_impl(
        session,
        plane,
        radius,
        start_angle,
        end_angle,
        tol,
        out_object,
    )
}

#[rgm_export(ts = "createArcBy3Points", receiver = "kernel")]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_curve_create_arc_by_3_points(
    session: RgmKernelHandle,
    start: RgmPoint3,
    mid: RgmPoint3,
    end: RgmPoint3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_curve_create_arc_by_3_points_impl(session, start, mid, end, tol, out_object)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_curve_create_arc_by_3_points(
    session: RgmKernelHandle,
    start: *const RgmPoint3,
    mid: *const RgmPoint3,
    end: *const RgmPoint3,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if start.is_null() || mid.is_null() || end.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null constructor pointer");
    }
    let start = unsafe { *start };
    let mid = unsafe { *mid };
    let end = unsafe { *end };
    let tol = unsafe { *tol };
    rgm_curve_create_arc_by_3_points_impl(session, start, mid, end, tol, out_object)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_curve_create_arc_by_3_points_ptr_tol(
    session: RgmKernelHandle,
    start: *const RgmPoint3,
    mid: *const RgmPoint3,
    end: *const RgmPoint3,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if start.is_null() || mid.is_null() || end.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null constructor pointer");
    }
    let start = unsafe { *start };
    let mid = unsafe { *mid };
    let end = unsafe { *end };
    let tol = unsafe { *tol };
    rgm_curve_create_arc_by_3_points_impl(session, start, mid, end, tol, out_object)
}

#[rgm_export(ts = "createPolyline", receiver = "kernel")]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_curve_create_polyline(
    session: RgmKernelHandle,
    points: *const RgmPoint3,
    point_count: usize,
    closed: bool,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_curve_create_polyline_impl(session, points, point_count, closed, tol, out_object)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_curve_create_polyline(
    session: RgmKernelHandle,
    points: *const RgmPoint3,
    point_count: usize,
    closed: bool,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null tolerance pointer");
    }
    let tol = unsafe { *tol };
    rgm_curve_create_polyline_impl(session, points, point_count, closed, tol, out_object)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_curve_create_polyline_ptr_tol(
    session: RgmKernelHandle,
    points: *const RgmPoint3,
    point_count: usize,
    closed: bool,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null tolerance pointer");
    }
    let tol = unsafe { *tol };
    rgm_curve_create_polyline_impl(session, points, point_count, closed, tol, out_object)
}

#[rgm_export(ts = "createPolycurve", receiver = "kernel")]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_curve_create_polycurve(
    session: RgmKernelHandle,
    segments: *const RgmPolycurveSegment,
    segment_count: usize,
    _tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_curve_create_polycurve_impl(session, segments, segment_count, out_object)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_curve_create_polycurve(
    session: RgmKernelHandle,
    segments: *const RgmPolycurveSegment,
    segment_count: usize,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null tolerance pointer");
    }
    rgm_curve_create_polycurve_impl(session, segments, segment_count, out_object)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_curve_create_polycurve_ptr_tol(
    session: RgmKernelHandle,
    segments: *const RgmPolycurveSegment,
    segment_count: usize,
    tol: *const RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null tolerance pointer");
    }
    rgm_curve_create_polycurve_impl(session, segments, segment_count, out_object)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_create_indexed(
    session: RgmKernelHandle,
    vertices: *const RgmPoint3,
    vertex_count: usize,
    indices: *const u32,
    index_count: usize,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_mesh_create_indexed_impl(
        session,
        vertices,
        vertex_count,
        indices,
        index_count,
        out_object,
    )
}

#[rgm_export]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_mesh_create_box(
    session: RgmKernelHandle,
    center: RgmPoint3,
    size: RgmVec3,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_mesh_create_box_impl(session, center, size, out_object)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_mesh_create_box(
    session: RgmKernelHandle,
    center: *const RgmPoint3,
    size: *const RgmVec3,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if center.is_null() || size.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null mesh box pointer");
    }
    // SAFETY: pointers validated above.
    rgm_mesh_create_box_impl(session, unsafe { *center }, unsafe { *size }, out_object)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_create_box_ptr(
    session: RgmKernelHandle,
    center: *const RgmPoint3,
    size: *const RgmVec3,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if center.is_null() || size.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null mesh box pointer");
    }
    // SAFETY: pointers validated above.
    rgm_mesh_create_box_impl(session, unsafe { *center }, unsafe { *size }, out_object)
}

#[rgm_export]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_mesh_create_uv_sphere(
    session: RgmKernelHandle,
    center: RgmPoint3,
    radius: f64,
    u_steps: u32,
    v_steps: u32,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_mesh_create_uv_sphere_impl(session, center, radius, u_steps, v_steps, out_object)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_mesh_create_uv_sphere(
    session: RgmKernelHandle,
    center: *const RgmPoint3,
    radius: f64,
    u_steps: u32,
    v_steps: u32,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if center.is_null() {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Null mesh UV sphere center pointer",
        );
    }
    // SAFETY: pointer validated above.
    rgm_mesh_create_uv_sphere_impl(
        session,
        unsafe { *center },
        radius,
        u_steps,
        v_steps,
        out_object,
    )
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_create_uv_sphere_ptr(
    session: RgmKernelHandle,
    center: *const RgmPoint3,
    radius: f64,
    u_steps: u32,
    v_steps: u32,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if center.is_null() {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Null mesh UV sphere center pointer",
        );
    }
    // SAFETY: pointer validated above.
    rgm_mesh_create_uv_sphere_impl(
        session,
        unsafe { *center },
        radius,
        u_steps,
        v_steps,
        out_object,
    )
}

#[rgm_export]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_mesh_create_torus(
    session: RgmKernelHandle,
    center: RgmPoint3,
    major_radius: f64,
    minor_radius: f64,
    major_steps: u32,
    minor_steps: u32,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_mesh_create_torus_impl(
        session,
        center,
        major_radius,
        minor_radius,
        major_steps,
        minor_steps,
        out_object,
    )
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_mesh_create_torus(
    session: RgmKernelHandle,
    center: *const RgmPoint3,
    major_radius: f64,
    minor_radius: f64,
    major_steps: u32,
    minor_steps: u32,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if center.is_null() {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Null mesh torus center pointer",
        );
    }
    // SAFETY: pointer validated above.
    rgm_mesh_create_torus_impl(
        session,
        unsafe { *center },
        major_radius,
        minor_radius,
        major_steps,
        minor_steps,
        out_object,
    )
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_create_torus_ptr(
    session: RgmKernelHandle,
    center: *const RgmPoint3,
    major_radius: f64,
    minor_radius: f64,
    major_steps: u32,
    minor_steps: u32,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if center.is_null() {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Null mesh torus center pointer",
        );
    }
    // SAFETY: pointer validated above.
    rgm_mesh_create_torus_impl(
        session,
        unsafe { *center },
        major_radius,
        minor_radius,
        major_steps,
        minor_steps,
        out_object,
    )
}

#[rgm_export]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_mesh_translate(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    delta: RgmVec3,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    let transform = matrix_translation(delta);
    rgm_mesh_transform_impl(
        session,
        mesh,
        transform,
        out_mesh,
        "Mesh translation failed",
    )
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_mesh_translate(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    delta: *const RgmVec3,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    if delta.is_null() {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Null mesh translation pointer",
        );
    }
    // SAFETY: pointer validated above.
    let transform = matrix_translation(unsafe { *delta });
    rgm_mesh_transform_impl(
        session,
        mesh,
        transform,
        out_mesh,
        "Mesh translation failed",
    )
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_translate_ptr(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    delta: *const RgmVec3,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    if delta.is_null() {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Null mesh translation pointer",
        );
    }
    // SAFETY: pointer validated above.
    let transform = matrix_translation(unsafe { *delta });
    rgm_mesh_transform_impl(
        session,
        mesh,
        transform,
        out_mesh,
        "Mesh translation failed",
    )
}

#[rgm_export]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_mesh_rotate(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    axis: RgmVec3,
    angle_rad: f64,
    pivot: RgmPoint3,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    let Ok(rotation) = matrix_rotation(axis, angle_rad) else {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Mesh rotation failed");
    };
    let transform = matrix_about_pivot(rotation, pivot);
    rgm_mesh_transform_impl(session, mesh, transform, out_mesh, "Mesh rotation failed")
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_mesh_rotate(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    axis: *const RgmVec3,
    angle_rad: f64,
    pivot: *const RgmPoint3,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    if axis.is_null() || pivot.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null mesh rotate pointer");
    }
    // SAFETY: pointers validated above.
    let Ok(rotation) = matrix_rotation(unsafe { *axis }, angle_rad) else {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Mesh rotation failed");
    };
    let transform = matrix_about_pivot(rotation, unsafe { *pivot });
    rgm_mesh_transform_impl(session, mesh, transform, out_mesh, "Mesh rotation failed")
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_rotate_ptr(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    axis: *const RgmVec3,
    angle_rad: f64,
    pivot: *const RgmPoint3,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    if axis.is_null() || pivot.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null mesh rotate pointer");
    }
    // SAFETY: pointers validated above.
    let Ok(rotation) = matrix_rotation(unsafe { *axis }, angle_rad) else {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Mesh rotation failed");
    };
    let transform = matrix_about_pivot(rotation, unsafe { *pivot });
    rgm_mesh_transform_impl(session, mesh, transform, out_mesh, "Mesh rotation failed")
}

#[rgm_export]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_mesh_scale(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    scale: RgmVec3,
    pivot: RgmPoint3,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    if scale.x.abs() <= 1e-12 || scale.y.abs() <= 1e-12 || scale.z.abs() <= 1e-12 {
        return map_err_with_session(session, RgmStatus::DegenerateGeometry, "Zero mesh scale");
    }
    let transform = matrix_about_pivot(matrix_scale(scale), pivot);
    rgm_mesh_transform_impl(session, mesh, transform, out_mesh, "Mesh scale failed")
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_mesh_scale(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    scale: *const RgmVec3,
    pivot: *const RgmPoint3,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    if scale.is_null() || pivot.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null mesh scale pointer");
    }
    // SAFETY: pointers validated above.
    let scale_value = unsafe { *scale };
    if scale_value.x.abs() <= 1e-12 || scale_value.y.abs() <= 1e-12 || scale_value.z.abs() <= 1e-12
    {
        return map_err_with_session(session, RgmStatus::DegenerateGeometry, "Zero mesh scale");
    }
    let transform = matrix_about_pivot(matrix_scale(scale_value), unsafe { *pivot });
    rgm_mesh_transform_impl(session, mesh, transform, out_mesh, "Mesh scale failed")
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_scale_ptr(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    scale: *const RgmVec3,
    pivot: *const RgmPoint3,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    if scale.is_null() || pivot.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null mesh scale pointer");
    }
    // SAFETY: pointers validated above.
    let scale_value = unsafe { *scale };
    if scale_value.x.abs() <= 1e-12 || scale_value.y.abs() <= 1e-12 || scale_value.z.abs() <= 1e-12
    {
        return map_err_with_session(session, RgmStatus::DegenerateGeometry, "Zero mesh scale");
    }
    let transform = matrix_about_pivot(matrix_scale(scale_value), unsafe { *pivot });
    rgm_mesh_transform_impl(session, mesh, transform, out_mesh, "Mesh scale failed")
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_bake_transform(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_mesh_bake_transform_impl(session, mesh, out_mesh)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_vertex_count(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let mesh = find_mesh(state, mesh)?;
        write_u32(
            out_count,
            mesh.vertices.len().try_into().unwrap_or(u32::MAX),
        )
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Mesh vertex count failed"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_triangle_count(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let mesh = find_mesh(state, mesh)?;
        write_u32(
            out_count,
            mesh.triangles.len().try_into().unwrap_or(u32::MAX),
        )
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Mesh triangle count failed"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_copy_vertices(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_vertices: *mut RgmPoint3,
    vertex_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let mesh = find_mesh(state, mesh)?;
        mesh_copy_vertices_world(mesh, out_vertices, vertex_capacity, out_count)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Mesh vertex copy failed"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_copy_indices(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_indices: *mut u32,
    index_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let mesh = find_mesh(state, mesh)?;
        mesh_copy_indices(mesh, out_indices, index_capacity, out_count)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Mesh index copy failed"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersect_mesh_plane(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    plane: *const RgmPlane,
    out_points: *mut RgmPoint3,
    point_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    if plane.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null plane pointer");
    }
    if out_count.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_count pointer");
    }
    // SAFETY: plane is non-null by guard above.
    let plane = unsafe { *plane };
    let Some(normal) = plane_unit_normal(plane) else {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Invalid plane normal");
    };

    let result = with_session_mut(session, |state| {
        ensure_mesh_accel(state, mesh)?;
        let accel = state
            .mesh_accels
            .get(&mesh.0)
            .ok_or(RgmStatus::InternalError)?;
        let mut segments = Vec::new();
        let tol = 1e-7;
        if let Some(bvh) = &accel.bvh {
            let mut stack = vec![bvh.root];
            while let Some(node_idx) = stack.pop() {
                let node = bvh.nodes[node_idx];
                if !aabb_node_plane_overlap(node.min, node.max, plane.origin, normal, tol) {
                    continue;
                }
                if node.is_leaf() {
                    for &tri_idx in &bvh.tri_indices[node.start..(node.start + node.count)] {
                        let tri = accel.triangles[tri_idx];
                        if let Some((p0, p1)) = intersect_triangle_plane_segment(
                            tri.points[0],
                            tri.points[1],
                            tri.points[2],
                            plane.origin,
                            normal,
                            tol,
                        ) {
                            segments.push(p0);
                            segments.push(p1);
                        }
                    }
                } else {
                    if let Some(left) = node.left {
                        stack.push(left);
                    }
                    if let Some(right) = node.right {
                        stack.push(right);
                    }
                }
            }
        } else {
            for tri in &accel.triangles {
                if let Some((p0, p1)) = intersect_triangle_plane_segment(
                    tri.points[0],
                    tri.points[1],
                    tri.points[2],
                    plane.origin,
                    normal,
                    tol,
                ) {
                    segments.push(p0);
                    segments.push(p1);
                }
            }
        }
        write_intersection_points(out_points, point_capacity, &segments, out_count)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Mesh-plane intersection failed"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersect_mesh_mesh(
    session: RgmKernelHandle,
    mesh_a: RgmObjectHandle,
    mesh_b: RgmObjectHandle,
    out_points: *mut RgmPoint3,
    point_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    if out_count.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_count pointer");
    }
    let result = with_session_mut(session, |state| {
        ensure_mesh_accel(state, mesh_a)?;
        ensure_mesh_accel(state, mesh_b)?;
        let accel_a = state
            .mesh_accels
            .get(&mesh_a.0)
            .ok_or(RgmStatus::InternalError)?;
        let accel_b = state
            .mesh_accels
            .get(&mesh_b.0)
            .ok_or(RgmStatus::InternalError)?;
        let tol = 1e-7;
        let mut segments = Vec::new();

        if let (Some(bvh_a), Some(bvh_b)) = (&accel_a.bvh, &accel_b.bvh) {
            let mut stack = vec![(bvh_a.root, bvh_b.root)];
            while let Some((node_a_idx, node_b_idx)) = stack.pop() {
                let node_a = bvh_a.nodes[node_a_idx];
                let node_b = bvh_b.nodes[node_b_idx];
                if !aabb_overlap(node_a.min, node_a.max, node_b.min, node_b.max, tol) {
                    continue;
                }

                if node_a.is_leaf() && node_b.is_leaf() {
                    for &tri_a_idx in
                        &bvh_a.tri_indices[node_a.start..(node_a.start + node_a.count)]
                    {
                        let tri_a = accel_a.triangles[tri_a_idx];
                        for &tri_b_idx in
                            &bvh_b.tri_indices[node_b.start..(node_b.start + node_b.count)]
                        {
                            let tri_b = accel_b.triangles[tri_b_idx];
                            if !aabb_overlap(tri_a.min, tri_a.max, tri_b.min, tri_b.max, tol) {
                                continue;
                            }
                            if let Some((p0, p1)) = tri_tri_intersection_segment(
                                tri_a.points[0],
                                tri_a.points[1],
                                tri_a.points[2],
                                tri_b.points[0],
                                tri_b.points[1],
                                tri_b.points[2],
                                tol,
                            ) {
                                segments.push(p0);
                                segments.push(p1);
                            }
                        }
                    }
                    continue;
                }

                if node_a.is_leaf() {
                    if let Some(left) = node_b.left {
                        stack.push((node_a_idx, left));
                    }
                    if let Some(right) = node_b.right {
                        stack.push((node_a_idx, right));
                    }
                    continue;
                }

                if node_b.is_leaf() {
                    if let Some(left) = node_a.left {
                        stack.push((left, node_b_idx));
                    }
                    if let Some(right) = node_a.right {
                        stack.push((right, node_b_idx));
                    }
                    continue;
                }

                if node_span(node_a) >= node_span(node_b) {
                    if let Some(left) = node_a.left {
                        stack.push((left, node_b_idx));
                    }
                    if let Some(right) = node_a.right {
                        stack.push((right, node_b_idx));
                    }
                } else {
                    if let Some(left) = node_b.left {
                        stack.push((node_a_idx, left));
                    }
                    if let Some(right) = node_b.right {
                        stack.push((node_a_idx, right));
                    }
                }
            }
        } else {
            for tri_a in &accel_a.triangles {
                for tri_b in &accel_b.triangles {
                    if !aabb_overlap(tri_a.min, tri_a.max, tri_b.min, tri_b.max, tol) {
                        continue;
                    }
                    if let Some((p0, p1)) = tri_tri_intersection_segment(
                        tri_a.points[0],
                        tri_a.points[1],
                        tri_a.points[2],
                        tri_b.points[0],
                        tri_b.points[1],
                        tri_b.points[2],
                        tol,
                    ) {
                        segments.push(p0);
                        segments.push(p1);
                    }
                }
            }
        }
        write_intersection_points(out_points, point_capacity, &segments, out_count)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Mesh-mesh intersection failed"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_mesh_boolean(
    session: RgmKernelHandle,
    mesh_a: RgmObjectHandle,
    mesh_b: RgmObjectHandle,
    op: i32,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_mesh_boolean_impl(session, mesh_a, mesh_b, op, out_mesh)
}

fn polycurve_to_nurbs(
    state: &SessionState,
    poly: &PolycurveData,
) -> Result<NurbsCurveData, RgmStatus> {
    if poly.segments.is_empty() {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let mut segment_nurbs = Vec::with_capacity(poly.segments.len());
    for seg in &poly.segments {
        let curve = find_curve(state, seg.curve)?;
        let Some(base) = curve_canonical_nurbs(curve) else {
            return Err(RgmStatus::InvalidInput);
        };
        let open = if base.core.periodic {
            periodic_to_open_nurbs(base)?
        } else {
            base.clone()
        };
        let nurbs = if seg.reversed {
            reverse_open_nurbs(&open)?
        } else {
            open
        };
        segment_nurbs.push(nurbs);
    }

    let target_degree = segment_nurbs
        .iter()
        .map(|curve| curve.core.degree)
        .max()
        .unwrap_or(1);
    let tol = segment_nurbs[0].core.tol;

    let mut normalized = Vec::with_capacity(segment_nurbs.len());
    for curve in segment_nurbs {
        if curve.core.degree == target_degree {
            normalized.push(curve);
        } else {
            normalized.push(elevate_open_nurbs_to_degree(&curve, target_degree)?);
        }
    }

    let mut control_points = Vec::new();
    let mut weights = Vec::new();
    let mut knots = Vec::new();
    let mut cursor = 0.0_f64;

    for (idx, segment) in normalized.iter().enumerate() {
        let span = segment.core.u_end - segment.core.u_start;
        let mapped = reparameterize_open_nurbs(segment, cursor, cursor + span)?;
        cursor += span;

        control_points.extend_from_slice(&mapped.core.control_points);
        weights.extend_from_slice(&mapped.core.weights);
        if idx == 0 {
            knots.extend_from_slice(&mapped.core.knots);
        } else {
            knots.extend(mapped.core.knots.iter().skip(target_degree + 1).copied());
        }
    }

    build_nurbs_from_core(
        target_degree,
        false,
        false,
        control_points,
        weights,
        knots,
        tol,
        Vec::new(),
    )
}

#[rgm_export(ts = "toNurbs", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_to_nurbs(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    out_curve: *mut RgmObjectHandle,
) -> RgmStatus {
    if out_curve.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_curve pointer");
    }

    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let nurbs = if let Some(existing) = curve_canonical_nurbs(curve_data) {
            existing.clone()
        } else {
            match curve_data {
                CurveData::Polycurve(poly) => polycurve_to_nurbs(state, poly)?,
                _ => return Err(RgmStatus::InternalError),
            }
        };

        let handle = insert_curve(state, CurveData::NurbsCurve(nurbs));
        write_object_handle(out_curve, handle)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve to NURBS conversion failed"),
    }
}

#[rgm_export(ts = "pointAt", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_point_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_point: *mut RgmPoint3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_normalized_data(state, curve_data, t_norm)?;
        write_point(out_point, eval.point)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve point evaluation failed"),
    }
}

#[rgm_export(ts = "length", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    out_length: *mut f64,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let length = curve_total_length(state, curve_data)?;
        write_f64(out_length, length)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve length evaluation failed"),
    }
}

#[rgm_export(ts = "lengthAt", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_length_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_length: *mut f64,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let length = curve_length_at_normalized_data(state, curve_data, t_norm)?;
        write_f64(out_length, length)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve length-at-parameter failed"),
    }
}

#[rgm_export(ts = "pointAtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_point_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_point: *mut RgmPoint3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_length_data(state, curve_data, distance_length)?;
        write_point(out_point, eval.point)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Curve point-at-distance evaluation failed")
        }
    }
}

#[rgm_export(ts = "d0", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d0_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_point: *mut RgmPoint3,
) -> RgmStatus {
    rgm_curve_point_at(session, curve, t_norm, out_point)
}

#[rgm_export(ts = "d0AtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d0_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_point: *mut RgmPoint3,
) -> RgmStatus {
    rgm_curve_point_at_length(session, curve, distance_length, out_point)
}

#[rgm_export(ts = "d1", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d1_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_d1: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_normalized_data(state, curve_data, t_norm)?;
        write_vec(out_d1, eval.d1)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Curve first derivative evaluation failed")
        }
    }
}

#[rgm_export(ts = "d1AtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d1_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_d1: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_length_data(state, curve_data, distance_length)?;
        write_vec(out_d1, eval.d1)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Curve first derivative-at-distance failed")
        }
    }
}

#[rgm_export(ts = "d2", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d2_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_derivative: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_normalized_data(state, curve_data, t_norm)?;
        write_vec(out_derivative, eval.d2)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Curve second derivative evaluation failed")
        }
    }
}

#[rgm_export(ts = "d2AtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d2_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_derivative: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_length_data(state, curve_data, distance_length)?;
        write_vec(out_derivative, eval.d2)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(
            session,
            status,
            "Curve second derivative-at-distance failed",
        ),
    }
}

#[rgm_export(ts = "tangentAt", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_tangent_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_tangent: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_normalized_data(state, curve_data, t_norm)?;
        let tangent = frame_tangent(eval)?;
        write_vec(out_tangent, tangent)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve tangent evaluation failed"),
    }
}

#[rgm_export(ts = "tangentAtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_tangent_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_tangent: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_length_data(state, curve_data, distance_length)?;
        let tangent = frame_tangent(eval)?;
        write_vec(out_tangent, tangent)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Curve tangent-at-length evaluation failed")
        }
    }
}

#[rgm_export(ts = "normalAt", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_normal_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_normal: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_normalized_data(state, curve_data, t_norm)?;
        let abs_tol = if let Some(nurbs) = curve_canonical_nurbs(curve_data) {
            nurbs.core.tol.abs_tol
        } else {
            1e-9
        };
        let normal = frame_normal(eval, abs_tol)?;
        write_vec(out_normal, normal)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve normal evaluation failed"),
    }
}

#[rgm_export(ts = "normalAtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_normal_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_normal: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_length_data(state, curve_data, distance_length)?;
        let abs_tol = if let Some(nurbs) = curve_canonical_nurbs(curve_data) {
            nurbs.core.tol.abs_tol
        } else {
            1e-9
        };
        let normal = frame_normal(eval, abs_tol)?;
        write_vec(out_normal, normal)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Curve normal-at-length evaluation failed")
        }
    }
}

#[rgm_export(ts = "planeAt", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_plane_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_plane: *mut RgmPlane,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_normalized_data(state, curve_data, t_norm)?;
        let abs_tol = if let Some(nurbs) = curve_canonical_nurbs(curve_data) {
            nurbs.core.tol.abs_tol
        } else {
            1e-9
        };
        let plane = frame_plane(eval, abs_tol)?;
        write_plane(out_plane, plane)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve plane evaluation failed"),
    }
}

#[rgm_export(ts = "planeAtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_plane_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_plane: *mut RgmPlane,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let eval = evaluate_curve_at_length_data(state, curve_data, distance_length)?;
        let abs_tol = if let Some(nurbs) = curve_canonical_nurbs(curve_data) {
            nurbs.core.tol.abs_tol
        } else {
            1e-9
        };
        let plane = frame_plane(eval, abs_tol)?;
        write_plane(out_plane, plane)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Curve plane-at-distance evaluation failed")
        }
    }
}

#[rgm_export(ts = "convertPointCoordinateSystem", receiver = "kernel")]
#[no_mangle]
pub extern "C" fn rgm_point_convert_coordinate_system(
    session: RgmKernelHandle,
    x: f64,
    y: f64,
    z: f64,
    source_coordinate_system: i32,
    target_coordinate_system: i32,
    out_point: *mut RgmPoint3,
) -> RgmStatus {
    let result = with_session_mut(session, |_| {
        let source = parse_coordinate_system(source_coordinate_system)?;
        let target = parse_coordinate_system(target_coordinate_system)?;
        let point = RgmPoint3 { x, y, z };
        let converted = convert_point_coordinate_system(point, source, target);
        write_point(out_point, converted)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Coordinate system conversion failed"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersect_curve_curve(
    session: RgmKernelHandle,
    curve_a: RgmObjectHandle,
    curve_b: RgmObjectHandle,
    out_points: *mut RgmPoint3,
    point_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    if out_count.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_count pointer");
    }

    let result = with_session_mut(session, |state| {
        let curve_data_a = find_curve(state, curve_a)?;
        let curve_data_b = find_curve(state, curve_b)?;
        let points = intersect_curve_curve_points_data(state, curve_data_a, curve_data_b)?;
        write_intersection_points(out_points, point_capacity, &points, out_count)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve-curve intersection failed"),
    }
}

#[rgm_export]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_intersect_curve_plane(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    plane: RgmPlane,
    out_points: *mut RgmPoint3,
    point_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    if out_count.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_count pointer");
    }

    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let points = intersect_curve_plane_points_data(state, curve_data, plane)?;
        write_intersection_points(out_points, point_capacity, &points, out_count)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve-plane intersection failed"),
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_intersect_curve_plane(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    plane: *const RgmPlane,
    out_points: *mut RgmPoint3,
    point_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    if plane.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null plane pointer");
    }

    if out_count.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_count pointer");
    }

    // SAFETY: plane is non-null by guard above.
    let plane = unsafe { *plane };
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let points = intersect_curve_plane_points_data(state, curve_data, plane)?;
        write_intersection_points(out_points, point_capacity, &points, out_count)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve-plane intersection failed"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersect_curve_plane_ptr(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    plane: *const RgmPlane,
    out_points: *mut RgmPoint3,
    point_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    if plane.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null plane pointer");
    }
    if out_count.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_count pointer");
    }

    // SAFETY: plane is non-null by guard above.
    let plane = unsafe { *plane };
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let points = intersect_curve_plane_points_data(state, curve_data, plane)?;
        write_intersection_points(out_points, point_capacity, &points, out_count)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve-plane intersection failed"),
    }
}

#[cfg(test)]
fn debug_get_curve(session: RgmKernelHandle, object: RgmObjectHandle) -> Option<CurveData> {
    let sessions = SESSIONS.lock().ok()?;
    let state = sessions.get(&session.0)?;
    match state.objects.get(&object.0)? {
        GeometryObject::Curve(curve) => Some(curve.clone()),
        GeometryObject::Mesh(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    fn create_session() -> RgmKernelHandle {
        let mut session = RgmKernelHandle(0);
        let status = rgm_kernel_create(&mut session as *mut _);
        assert_eq!(status, RgmStatus::Ok);
        session
    }

    fn tol() -> RgmToleranceContext {
        RgmToleranceContext {
            abs_tol: 1e-9,
            rel_tol: 1e-9,
            angle_tol: 1e-9,
        }
    }

    fn sample_points() -> Vec<RgmPoint3> {
        vec![
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 2.0,
                y: 1.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 3.0,
                y: 1.0,
                z: 0.0,
            },
        ]
    }

    #[test]
    fn session_create_and_destroy() {
        let mut session = RgmKernelHandle(0);
        assert_eq!(rgm_kernel_create(&mut session as *mut _), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::NotFound);
    }

    #[test]
    fn alloc_and_dealloc_roundtrip() {
        let mut ptr: *mut u8 = ptr::null_mut();
        assert_eq!(rgm_alloc(64, 8, &mut ptr as *mut _), RgmStatus::Ok);
        assert!(!ptr.is_null());

        // SAFETY: ptr is allocated for 64 bytes above.
        unsafe {
            for idx in 0..64 {
                *ptr.add(idx) = idx as u8;
            }
        }

        assert_eq!(rgm_dealloc(ptr, 64, 8), RgmStatus::Ok);
    }

    #[test]
    fn interpolate_open_curve_creates_nurbs() {
        let session = create_session();
        let points = sample_points();
        let mut object = RgmObjectHandle(0);

        let status = rgm_nurbs_interpolate_fit_points(
            session,
            points.as_ptr(),
            points.len(),
            2,
            false,
            tol(),
            &mut object as *mut _,
        );
        assert_eq!(status, RgmStatus::Ok);

        let curve = debug_get_curve(session, object).expect("curve exists");
        let CurveData::NurbsCurve(curve) = curve else {
            panic!("expected NURBS curve");
        };
        assert_eq!(curve.core.weights, vec![1.0; points.len()]);
        assert!((curve.core.knots[0] - 0.0).abs() < 1e-12);
        assert!((curve.core.knots[curve.core.knots.len() - 1] - 1.0).abs() < 1e-12);
        assert!(curve.arc_length.total_length > 0.0);

        assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn can_evaluate_point_derivatives_and_plane() {
        let session = create_session();
        let points = sample_points();
        let mut object = RgmObjectHandle(0);

        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                points.as_ptr(),
                points.len(),
                2,
                false,
                tol(),
                &mut object as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut point = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_point_at(session, object, 0.5, &mut point as *mut _),
            RgmStatus::Ok
        );

        let mut d1 = RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_d1_at(session, object, 0.5, &mut d1 as *mut _),
            RgmStatus::Ok
        );

        let mut d2 = RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_d2_at(session, object, 0.5, &mut d2 as *mut _),
            RgmStatus::Ok
        );
        assert!(vec_norm(d2) > 0.0);

        let mut plane = RgmPlane {
            origin: point,
            x_axis: RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            y_axis: RgmVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            z_axis: RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        };

        assert_eq!(
            rgm_curve_plane_at(session, object, 0.5, &mut plane as *mut _),
            RgmStatus::Ok
        );

        assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn length_queries_are_available() {
        let session = create_session();
        let points = sample_points();
        let mut object = RgmObjectHandle(0);

        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                points.as_ptr(),
                points.len(),
                2,
                false,
                tol(),
                &mut object as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut total: f64 = 0.0;
        assert_eq!(
            rgm_curve_length(session, object, &mut total as *mut _),
            RgmStatus::Ok
        );
        assert!(total > 0.0);

        let mut s0: f64 = -1.0;
        let mut s1: f64 = -1.0;
        let mut smid: f64 = -1.0;
        assert_eq!(
            rgm_curve_length_at(session, object, 0.0, &mut s0 as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_length_at(session, object, 0.5, &mut smid as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_length_at(session, object, 1.0, &mut s1 as *mut _),
            RgmStatus::Ok
        );

        assert!(s0.abs() < 1e-8);
        assert!(smid > s0);
        assert!(smid < s1);
        assert!((s1 - total).abs() < 1e-7);

        assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn line_constructor_is_exact_and_linear() {
        let session = create_session();
        let mut line = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 10.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut p = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_point_at(session, line, 0.25, &mut p as *mut _),
            RgmStatus::Ok
        );
        assert!((p.x - 2.5).abs() < 1e-9);

        let mut d2 = RgmVec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        assert_eq!(
            rgm_curve_d2_at(session, line, 0.5, &mut d2 as *mut _),
            RgmStatus::Ok
        );
        assert!(vec_norm(d2) < 1e-7);

        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn line_length_matches_euclidean_distance() {
        let session = create_session();
        let mut line = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 3.0,
                        y: 4.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, line, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 5.0).abs() < 1e-9);

        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn zero_length_line_is_supported() {
        let session = create_session();
        let mut line = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                    end: RgmPoint3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = -1.0_f64;
        assert_eq!(
            rgm_curve_length(session, line, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!(length.abs() < 1e-12);

        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn arc_by_angles_length_matches_radius_times_sweep() {
        let session = create_session();
        let mut arc = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_arc_by_angles(
                session,
                RgmPlane {
                    origin: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    x_axis: RgmVec3 {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    y_axis: RgmVec3 {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                    },
                    z_axis: RgmVec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                },
                10.0,
                0.0,
                PI,
                tol(),
                &mut arc as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, arc, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 10.0 * PI).abs() < 1e-8);

        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn arc_by_angles_length_is_positive_when_angles_are_reversed() {
        let session = create_session();
        let mut arc = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_arc_by_angles(
                session,
                RgmPlane {
                    origin: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    x_axis: RgmVec3 {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    y_axis: RgmVec3 {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                    },
                    z_axis: RgmVec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                },
                5.0,
                PI,
                0.0,
                tol(),
                &mut arc as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, arc, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 5.0 * PI).abs() < 1e-8);

        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn arc_by_3_points_preserves_radius_and_endpoints() {
        let session = create_session();
        let mut arc = RgmObjectHandle(0);
        let start = RgmPoint3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let mid = RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };
        let end = RgmPoint3 {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        };

        assert_eq!(
            rgm_curve_create_arc_by_3_points(session, start, mid, end, tol(), &mut arc as *mut _),
            RgmStatus::Ok
        );

        let mut p0 = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut p1 = p0;
        assert_eq!(
            rgm_curve_point_at(session, arc, 0.0, &mut p0),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, arc, 1.0, &mut p1),
            RgmStatus::Ok
        );
        assert!(distance(start, p0) < 1e-7);
        assert!(distance(end, p1) < 1e-7);

        let center = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let mut p = RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            assert_eq!(rgm_curve_point_at(session, arc, t, &mut p), RgmStatus::Ok);
            assert!((distance(center, p) - 1.0).abs() < 1e-6);
        }

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, arc, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - PI).abs() < 1e-7);

        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polyline_length_sums_open_segments() {
        let session = create_session();
        let mut polyline = RgmObjectHandle(0);
        let points = [
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 10.0,
                y: 10.0,
                z: 0.0,
            },
        ];

        assert_eq!(
            rgm_curve_create_polyline(
                session,
                points.as_ptr(),
                points.len(),
                false,
                tol(),
                &mut polyline as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, polyline, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 20.0).abs() < 1e-9);

        assert_eq!(rgm_object_release(session, polyline), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polyline_length_includes_closing_segment_when_closed() {
        let session = create_session();
        let mut polyline = RgmObjectHandle(0);
        let points = [
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 10.0,
                y: 10.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            },
        ];

        assert_eq!(
            rgm_curve_create_polyline(
                session,
                points.as_ptr(),
                points.len(),
                true,
                tol(),
                &mut polyline as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, polyline, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 40.0).abs() < 1e-7);

        assert_eq!(rgm_object_release(session, polyline), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polycurve_length_sums_children() {
        let session = create_session();
        let mut line1 = RgmObjectHandle(0);
        let mut line2 = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 10.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line1 as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 10.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 20.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line2 as *mut _,
            ),
            RgmStatus::Ok
        );

        let segments = [
            RgmPolycurveSegment {
                curve: line1,
                reversed: false,
            },
            RgmPolycurveSegment {
                curve: line2,
                reversed: false,
            },
        ];
        let mut polycurve = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polycurve(
                session,
                segments.as_ptr(),
                segments.len(),
                tol(),
                &mut polycurve as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, polycurve, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 20.0).abs() < 1e-9);

        assert_eq!(rgm_object_release(session, polycurve), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line2), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line1), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn point_coordinate_system_conversion_swaps_axes() {
        let session = create_session();
        let mut out = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        assert_eq!(
            rgm_point_convert_coordinate_system(
                session,
                10.0,
                20.0,
                30.0,
                RgmAlignmentCoordinateSystem::EastingNorthing as i32,
                RgmAlignmentCoordinateSystem::NorthingEasting as i32,
                &mut out as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!((out.x - 20.0).abs() < 1e-12);
        assert!((out.y - 10.0).abs() < 1e-12);
        assert!((out.z - 30.0).abs() < 1e-12);

        assert_eq!(
            rgm_point_convert_coordinate_system(
                session,
                out.x,
                out.y,
                out.z,
                RgmAlignmentCoordinateSystem::NorthingEasting as i32,
                RgmAlignmentCoordinateSystem::EastingNorthing as i32,
                &mut out as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!((out.x - 10.0).abs() < 1e-12);
        assert!((out.y - 20.0).abs() < 1e-12);
        assert!((out.z - 30.0).abs() < 1e-12);

        assert_eq!(
            rgm_point_convert_coordinate_system(session, 1.0, 2.0, 3.0, 42, 0, &mut out as *mut _),
            RgmStatus::InvalidInput
        );

        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn circle_constructor_is_periodic() {
        let session = create_session();
        let mut circle = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_circle(
                session,
                RgmCircle3 {
                    plane: RgmPlane {
                        origin: RgmPoint3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        x_axis: RgmVec3 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        y_axis: RgmVec3 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
                        },
                        z_axis: RgmVec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 1.0,
                        },
                    },
                    radius: 5.0,
                },
                tol(),
                &mut circle as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut p0 = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut p1 = p0;
        assert_eq!(
            rgm_curve_point_at(session, circle, 0.0, &mut p0),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, circle, 1.0, &mut p1),
            RgmStatus::Ok
        );
        assert!(distance(p0, p1) < 1e-6);
        for t in [0.0, 0.13, 0.27, 0.51, 0.79, 1.0] {
            let mut p = RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            assert_eq!(
                rgm_curve_point_at(session, circle, t, &mut p),
                RgmStatus::Ok
            );
            let r = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
            assert!((r - 5.0).abs() < 1e-5);
        }

        assert_eq!(rgm_object_release(session, circle), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn arc_constructor_preserves_radius_and_endpoints() {
        let session = create_session();
        let mut arc = RgmObjectHandle(0);

        let start = -0.4_f64;
        let sweep = 1.2_f64;
        let radius = 3.25_f64;
        let center = RgmPoint3 {
            x: 1.2,
            y: -0.7,
            z: 0.5,
        };

        assert_eq!(
            rgm_curve_create_arc(
                session,
                RgmArc3 {
                    plane: RgmPlane {
                        origin: center,
                        x_axis: RgmVec3 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        y_axis: RgmVec3 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
                        },
                        z_axis: RgmVec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 1.0,
                        },
                    },
                    radius,
                    start_angle: start,
                    sweep_angle: sweep,
                },
                tol(),
                &mut arc as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut p0 = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut p1 = p0;
        let mut pm = p0;

        assert_eq!(
            rgm_curve_point_at(session, arc, 0.0, &mut p0),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, arc, 0.5, &mut pm),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, arc, 1.0, &mut p1),
            RgmStatus::Ok
        );

        let expected_start = RgmPoint3 {
            x: center.x + radius * start.cos(),
            y: center.y + radius * start.sin(),
            z: center.z,
        };
        let expected_end = RgmPoint3 {
            x: center.x + radius * (start + sweep).cos(),
            y: center.y + radius * (start + sweep).sin(),
            z: center.z,
        };

        assert!(distance(p0, expected_start) < 1e-6);
        assert!(distance(p1, expected_end) < 1e-6);

        for p in [p0, pm, p1] {
            let r = distance(center, p);
            assert!((r - radius).abs() < 1e-5);
        }

        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polycurve_is_global_curve() {
        let session = create_session();
        let mut line = RgmObjectHandle(0);
        let mut arc = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        assert_eq!(
            rgm_curve_create_arc(
                session,
                RgmArc3 {
                    plane: RgmPlane {
                        origin: RgmPoint3 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        x_axis: RgmVec3 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        y_axis: RgmVec3 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
                        },
                        z_axis: RgmVec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 1.0,
                        },
                    },
                    radius: 1.0,
                    start_angle: 0.0,
                    sweep_angle: FRAC_PI_2,
                },
                tol(),
                &mut arc as *mut _,
            ),
            RgmStatus::Ok
        );

        let segments = [
            RgmPolycurveSegment {
                curve: line,
                reversed: false,
            },
            RgmPolycurveSegment {
                curve: arc,
                reversed: false,
            },
        ];

        let mut poly = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polycurve(
                session,
                segments.as_ptr(),
                segments.len(),
                tol(),
                &mut poly as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut p = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_point_at(session, poly, 0.75, &mut p as *mut _),
            RgmStatus::Ok
        );

        let mut nurbs = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_to_nurbs(session, poly, &mut nurbs as *mut _),
            RgmStatus::Ok
        );
        let converted = debug_get_curve(session, nurbs).expect("converted curve exists");
        let CurveData::NurbsCurve(converted) = converted else {
            panic!("expected converted NURBS curve");
        };
        assert_eq!(converted.core.degree, 2);
        assert_eq!(converted.core.control_points.len(), 6);

        let total = 1.0 + FRAC_PI_2;
        for dist in [0.0, 0.25, 0.75, 1.25, total] {
            let mut from_poly = RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            let mut from_nurbs = from_poly;
            assert_eq!(
                rgm_curve_point_at_length(session, poly, dist, &mut from_poly as *mut _),
                RgmStatus::Ok
            );
            assert_eq!(
                rgm_curve_point_at_length(session, nurbs, dist, &mut from_nurbs as *mut _),
                RgmStatus::Ok
            );
            assert!(distance(from_poly, from_nurbs) < 1e-6);
        }

        assert_eq!(rgm_object_release(session, nurbs), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, poly), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polycurve_to_nurbs_supports_mixed_degrees_exactly() {
        let session = create_session();
        let fit_points = [
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 1.0,
                y: 1.5,
                z: 0.0,
            },
            RgmPoint3 {
                x: 2.0,
                y: -1.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 3.5,
                y: 0.8,
                z: 0.0,
            },
            RgmPoint3 {
                x: 4.5,
                y: -0.3,
                z: 0.0,
            },
            RgmPoint3 {
                x: 5.5,
                y: 0.4,
                z: 0.0,
            },
        ];

        let mut cubic = RgmObjectHandle(0);
        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                fit_points.as_ptr(),
                fit_points.len(),
                3,
                false,
                tol(),
                &mut cubic as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut arc = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_arc(
                session,
                RgmArc3 {
                    plane: RgmPlane {
                        origin: RgmPoint3 {
                            x: 7.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        x_axis: RgmVec3 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        y_axis: RgmVec3 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
                        },
                        z_axis: RgmVec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 1.0,
                        },
                    },
                    radius: 1.2,
                    start_angle: PI,
                    sweep_angle: FRAC_PI_2,
                },
                tol(),
                &mut arc as *mut _,
            ),
            RgmStatus::Ok
        );

        let segments = [
            RgmPolycurveSegment {
                curve: cubic,
                reversed: false,
            },
            RgmPolycurveSegment {
                curve: arc,
                reversed: false,
            },
        ];
        let mut poly = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polycurve(
                session,
                segments.as_ptr(),
                segments.len(),
                tol(),
                &mut poly as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut nurbs = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_to_nurbs(session, poly, &mut nurbs as *mut _),
            RgmStatus::Ok
        );

        let converted = debug_get_curve(session, nurbs).expect("converted curve exists");
        let CurveData::NurbsCurve(converted) = converted else {
            panic!("expected converted NURBS curve");
        };
        assert_eq!(converted.core.degree, 3);
        assert_eq!(converted.core.control_points.len(), 10);

        let mut total = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, poly, &mut total as *mut _),
            RgmStatus::Ok
        );
        for fraction in [0.0, 0.13, 0.27, 0.51, 0.79, 1.0] {
            let s = total * fraction;
            let mut from_poly = RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            let mut from_nurbs = from_poly;
            assert_eq!(
                rgm_curve_point_at_length(session, poly, s, &mut from_poly as *mut _),
                RgmStatus::Ok
            );
            assert_eq!(
                rgm_curve_point_at_length(session, nurbs, s, &mut from_nurbs as *mut _),
                RgmStatus::Ok
            );
            assert!(distance(from_poly, from_nurbs) < 1e-6);
        }

        assert_eq!(rgm_object_release(session, nurbs), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, poly), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, cubic), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polycurve_to_nurbs_supports_periodic_segments() {
        let session = create_session();
        let fit_points = [
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 1.4,
                y: 0.8,
                z: 0.0,
            },
            RgmPoint3 {
                x: 0.4,
                y: 1.2,
                z: 0.0,
            },
            RgmPoint3 {
                x: -0.4,
                y: 0.6,
                z: 0.0,
            },
        ];

        let mut periodic = RgmObjectHandle(0);
        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                fit_points.as_ptr(),
                fit_points.len(),
                3,
                true,
                tol(),
                &mut periodic as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut line = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 2.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 3.0,
                        y: 0.4,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let segments = [
            RgmPolycurveSegment {
                curve: periodic,
                reversed: false,
            },
            RgmPolycurveSegment {
                curve: line,
                reversed: false,
            },
        ];
        let mut poly = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polycurve(
                session,
                segments.as_ptr(),
                segments.len(),
                tol(),
                &mut poly as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut nurbs = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_to_nurbs(session, poly, &mut nurbs as *mut _),
            RgmStatus::Ok
        );

        let mut poly_total = 0.0_f64;
        let mut nurbs_total = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, poly, &mut poly_total as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_length(session, nurbs, &mut nurbs_total as *mut _),
            RgmStatus::Ok
        );
        assert!(poly_total > 0.0);
        assert!(nurbs_total > 0.0);
        assert!((poly_total - nurbs_total).abs() / poly_total.max(1e-9) < 0.12);

        let mut poly_start = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut nurbs_start = poly_start;
        let mut poly_end = poly_start;
        let mut nurbs_end = poly_start;
        assert_eq!(
            rgm_curve_point_at(session, poly, 0.0, &mut poly_start as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, nurbs, 0.0, &mut nurbs_start as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, poly, 1.0, &mut poly_end as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, nurbs, 1.0, &mut nurbs_end as *mut _),
            RgmStatus::Ok
        );
        assert!(distance(poly_start, nurbs_start) < 0.15);
        assert!(distance(poly_end, nurbs_end) < 0.15);

        assert_eq!(rgm_object_release(session, nurbs), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, poly), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, periodic), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn intersect_curve_plane_counts_expected_hits() {
        let session = create_session();
        let mut line_crossing = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    end: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                },
                tol(),
                &mut line_crossing as *mut _,
            ),
            RgmStatus::Ok
        );

        let plane_xy = RgmPlane {
            origin: RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            x_axis: RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            y_axis: RgmVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            z_axis: RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        };

        let mut count = 0_u32;
        let mut points = [RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 8];
        assert_eq!(
            rgm_intersect_curve_plane(
                session,
                line_crossing,
                plane_xy,
                points.as_mut_ptr(),
                points.len() as u32,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 1);
        assert!(
            distance(
                points[0],
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
            ) < 1e-8
        );

        let mut line_parallel = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: -1.0,
                        y: 0.0,
                        z: 2.0,
                    },
                    end: RgmPoint3 {
                        x: 1.0,
                        y: 0.0,
                        z: 2.0,
                    },
                },
                tol(),
                &mut line_parallel as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_intersect_curve_plane(
                session,
                line_parallel,
                plane_xy,
                ptr::null_mut(),
                0,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 0);

        let mut circle = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_circle(
                session,
                RgmCircle3 {
                    plane: RgmPlane {
                        origin: RgmPoint3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        x_axis: RgmVec3 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        y_axis: RgmVec3 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
                        },
                        z_axis: RgmVec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 1.0,
                        },
                    },
                    radius: 1.0,
                },
                tol(),
                &mut circle as *mut _,
            ),
            RgmStatus::Ok
        );

        let tangent_plane = RgmPlane {
            origin: RgmPoint3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            x_axis: RgmVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            y_axis: RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            z_axis: RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        };
        assert_eq!(
            rgm_intersect_curve_plane(
                session,
                circle,
                tangent_plane,
                points.as_mut_ptr(),
                points.len() as u32,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 1);
        assert!((points[0].x - 1.0).abs() < 5e-2);

        let secant_plane = RgmPlane {
            origin: RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            x_axis: RgmVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            y_axis: RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            z_axis: RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        };
        assert_eq!(
            rgm_intersect_curve_plane(
                session,
                circle,
                secant_plane,
                points.as_mut_ptr(),
                points.len() as u32,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 2);
        assert!(points[0].x.abs() < 5e-2 && points[1].x.abs() < 5e-2);

        assert_eq!(rgm_object_release(session, circle), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line_parallel), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line_crossing), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn intersect_curve_curve_counts_expected_hits() {
        let session = create_session();
        let mut line_x = RgmObjectHandle(0);
        let mut line_y = RgmObjectHandle(0);
        let mut line_skew = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: -1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line_x as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line_y as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: -1.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    end: RgmPoint3 {
                        x: 1.0,
                        y: 0.0,
                        z: 1.0,
                    },
                },
                tol(),
                &mut line_skew as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut count = 0_u32;
        let mut points = [RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 8];
        assert_eq!(
            rgm_intersect_curve_curve(
                session,
                line_x,
                line_y,
                points.as_mut_ptr(),
                points.len() as u32,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 1);
        assert!(
            distance(
                points[0],
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
            ) < 1e-8
        );

        assert_eq!(
            rgm_intersect_curve_curve(
                session,
                line_y,
                line_skew,
                ptr::null_mut(),
                0,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 0);

        assert_eq!(rgm_object_release(session, line_skew), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line_y), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line_x), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn threaded_sessions_are_isolated() {
        let threads: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(|| {
                    let session = create_session();
                    let points = sample_points();
                    let mut object = RgmObjectHandle(0);
                    let status = rgm_nurbs_interpolate_fit_points(
                        session,
                        points.as_ptr(),
                        points.len(),
                        2,
                        false,
                        tol(),
                        &mut object as *mut _,
                    );
                    assert_eq!(status, RgmStatus::Ok);
                    assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
                    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
                })
            })
            .collect();

        for thread in threads {
            thread.join().expect("thread should complete");
        }
    }
}
