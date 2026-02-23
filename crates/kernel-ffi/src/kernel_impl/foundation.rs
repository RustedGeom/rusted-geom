
use boolmesh::{
    compute_boolean,
    prelude::{Manifold, OpType as BoolOpType},
};
use kernel_abi_meta::{rgm_export, rgm_ffi_type};
use crate::math;
use crate::math::arc_length::{build_arc_length_cache, length_from_u, u_from_length, ArcLengthCache};
use crate::math::frame::{
    normal as frame_normal, orthonormalize_plane_axes, plane as frame_plane, point_from_frame,
    tangent as frame_tangent,
};
use crate::math::intersections::{intersect_curve_curve_points, intersect_curve_plane_points};
use crate::math::nurbs_curve_eval::{
    eval_nurbs_normalized, eval_nurbs_u, map_normalized_to_u, validate_curve, CurveEvalResult,
    NurbsCurveCore,
};
use crate::math::nurbs_surface_eval::{
    eval_nurbs_surface_normalized, eval_nurbs_surface_uv, validate_surface, NurbsSurfaceCore,
    SurfaceEvalResult,
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

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmPoint2 {
    pub x: f64,
    pub y: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmVec2 {
    pub x: f64,
    pub y: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmUv2 {
    pub u: f64,
    pub v: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmNurbsSurfaceDesc {
    pub degree_u: u32,
    pub degree_v: u32,
    pub periodic_u: bool,
    pub periodic_v: bool,
    pub control_u_count: u32,
    pub control_v_count: u32,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmSurfaceEvalFrame {
    pub point: RgmPoint3,
    pub du: RgmVec3,
    pub dv: RgmVec3,
    pub normal: RgmVec3,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmTrimEdgeInput {
    pub start_uv: RgmUv2,
    pub end_uv: RgmUv2,
    pub curve_3d: RgmObjectHandle,
    pub has_curve_3d: bool,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmTrimLoopInput {
    pub edge_count: u32,
    pub is_outer: bool,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmSurfaceTessellationOptions {
    pub min_u_segments: u32,
    pub min_v_segments: u32,
    pub max_u_segments: u32,
    pub max_v_segments: u32,
    pub chord_tol: f64,
    pub normal_tol_rad: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmIntersectionBranchSummary {
    pub point_count: u32,
    pub uv_a_count: u32,
    pub uv_b_count: u32,
    pub curve_t_count: u32,
    pub closed: bool,
    pub flags: u32,
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
struct SurfaceData {
    core: NurbsSurfaceCore,
    transform: [[f64; 4]; 4],
}

#[derive(Clone, Debug)]
struct TrimEdgeData {
    start_uv: RgmUv2,
    end_uv: RgmUv2,
    curve_3d: Option<RgmObjectHandle>,
    uv_samples: Vec<RgmUv2>,
}

#[derive(Clone, Debug)]
struct TrimLoopData {
    edges: Vec<TrimEdgeData>,
    is_outer: bool,
}

#[derive(Clone, Debug)]
struct FaceData {
    surface: RgmObjectHandle,
    loops: Vec<TrimLoopData>,
}

#[derive(Clone, Debug)]
struct IntersectionBranchData {
    points: Vec<RgmPoint3>,
    uv_a: Vec<RgmUv2>,
    uv_b: Vec<RgmUv2>,
    curve_t: Vec<f64>,
    closed: bool,
    flags: u32,
}

#[derive(Clone, Debug)]
struct IntersectionData {
    branches: Vec<IntersectionBranchData>,
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
    Surface(SurfaceData),
    Face(FaceData),
    Intersection(IntersectionData),
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

fn write_surface_eval_frame(
    out: *mut RgmSurfaceEvalFrame,
    value: RgmSurfaceEvalFrame,
) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    unsafe {
        *out = value;
    }
    Ok(())
}

fn write_branch_summary(
    out: *mut RgmIntersectionBranchSummary,
    value: RgmIntersectionBranchSummary,
) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
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

fn write_uv_points(
    out_points: *mut RgmUv2,
    point_capacity: u32,
    points: &[RgmUv2],
    out_count: *mut u32,
) -> Result<(), RgmStatus> {
    if out_count.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
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
        unsafe {
            *out_points.add(idx) = *point;
        }
    }
    Ok(())
}

fn write_f64_array(
    out_values: *mut f64,
    value_capacity: u32,
    values: &[f64],
    out_count: *mut u32,
) -> Result<(), RgmStatus> {
    if out_count.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    unsafe {
        *out_count = values.len().try_into().unwrap_or(u32::MAX);
    }
    if value_capacity == 0 {
        return Ok(());
    }
    if out_values.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    let copy_count = values.len().min(value_capacity as usize);
    for (idx, value) in values.iter().take(copy_count).enumerate() {
        unsafe {
            *out_values.add(idx) = *value;
        }
    }
    Ok(())
}

fn map_err_with_session(session: RgmKernelHandle, status: RgmStatus, message: &str) -> RgmStatus {
    set_error(session.0, status, message);
    status
}
