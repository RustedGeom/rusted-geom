use kernel_abi_meta::{rgm_export, rgm_ffi_type};
use once_cell::sync::Lazy;
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
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

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct NurbsCurveData {
    degree: u32,
    closed: bool,
    fit_points: Vec<RgmPoint3>,
    weights: Vec<f64>,
    knots: Vec<f64>,
    params: Vec<f64>,
    cumulative_lengths: Vec<f64>,
    total_length: f64,
}

#[derive(Clone, Debug)]
enum GeometryObject {
    NurbsCurve(NurbsCurveData),
}

#[derive(Default)]
struct SessionState {
    objects: HashMap<u64, GeometryObject>,
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

fn distance(a: RgmPoint3, b: RgmPoint3) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dz = b.z - a.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn subtract(a: RgmPoint3, b: RgmPoint3) -> RgmVec3 {
    RgmVec3 {
        x: a.x - b.x,
        y: a.y - b.y,
        z: a.z - b.z,
    }
}

fn normalize(v: RgmVec3) -> Option<RgmVec3> {
    let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    if len <= f64::EPSILON {
        return None;
    }

    Some(RgmVec3 {
        x: v.x / len,
        y: v.y / len,
        z: v.z / len,
    })
}

fn cross(a: RgmVec3, b: RgmVec3) -> RgmVec3 {
    RgmVec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

fn interpolate_point(a: RgmPoint3, b: RgmPoint3, t: f64) -> RgmPoint3 {
    RgmPoint3 {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        z: a.z + (b.z - a.z) * t,
    }
}

fn cumulative_lengths(points: &[RgmPoint3]) -> (Vec<f64>, f64) {
    let mut cumulative = vec![0.0; points.len()];
    let mut total = 0.0;

    for i in 1..points.len() {
        total += distance(points[i - 1], points[i]);
        cumulative[i] = total;
    }

    (cumulative, total)
}

fn chord_length_params(cumulative: &[f64], total_length: f64) -> Vec<f64> {
    let count = cumulative.len();
    if count <= 1 {
        return vec![0.0; count];
    }

    if total_length <= f64::EPSILON {
        return (0..count)
            .map(|idx| idx as f64 / (count - 1) as f64)
            .collect();
    }

    cumulative.iter().map(|v| v / total_length).collect()
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

fn periodic_knots(point_count: usize, degree: usize) -> Vec<f64> {
    let knot_count = point_count + degree + 1;
    if knot_count <= 1 {
        return vec![0.0; knot_count];
    }

    (0..knot_count)
        .map(|idx| idx as f64 / (knot_count - 1) as f64)
        .collect()
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

    // SAFETY: Pointer nullability checked above.
    unsafe {
        *out = value;
    }

    Ok(())
}

fn write_vec(out: *mut RgmVec3, value: RgmVec3) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }

    // SAFETY: Pointer nullability checked above.
    unsafe {
        *out = value;
    }

    Ok(())
}

fn write_plane(out: *mut RgmPlane, value: RgmPlane) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }

    // SAFETY: Pointer nullability checked above.
    unsafe {
        *out = value;
    }

    Ok(())
}

fn evaluate_curve_at_normalized(
    curve: &NurbsCurveData,
    t_norm: f64,
) -> Result<RgmPoint3, RgmStatus> {
    if !(0.0..=1.0).contains(&t_norm) {
        return Err(RgmStatus::OutOfRange);
    }

    if curve.fit_points.is_empty() {
        return Err(RgmStatus::DegenerateGeometry);
    }

    if curve.fit_points.len() == 1 {
        return Ok(curve.fit_points[0]);
    }

    for i in 0..(curve.params.len() - 1) {
        let a = curve.params[i];
        let b = curve.params[i + 1];
        let inside =
            (t_norm >= a && t_norm <= b) || (i + 1 == curve.params.len() - 1 && t_norm == 1.0);
        if inside {
            let segment_t = if (b - a).abs() <= f64::EPSILON {
                0.0
            } else {
                (t_norm - a) / (b - a)
            };
            return Ok(interpolate_point(
                curve.fit_points[i],
                curve.fit_points[i + 1],
                segment_t,
            ));
        }
    }

    Ok(*curve.fit_points.last().unwrap_or(&curve.fit_points[0]))
}

fn evaluate_d1_at_normalized(curve: &NurbsCurveData, t_norm: f64) -> Result<RgmVec3, RgmStatus> {
    if !(0.0..=1.0).contains(&t_norm) {
        return Err(RgmStatus::OutOfRange);
    }

    if curve.fit_points.len() < 2 {
        return Err(RgmStatus::DegenerateGeometry);
    }

    for i in 0..(curve.params.len() - 1) {
        let a = curve.params[i];
        let b = curve.params[i + 1];
        let inside =
            (t_norm >= a && t_norm <= b) || (i + 1 == curve.params.len() - 1 && t_norm == 1.0);
        if inside {
            return Ok(subtract(curve.fit_points[i + 1], curve.fit_points[i]));
        }
    }

    Err(RgmStatus::DegenerateGeometry)
}

fn evaluate_tangent_at_normalized(
    curve: &NurbsCurveData,
    t_norm: f64,
) -> Result<RgmVec3, RgmStatus> {
    let d1 = evaluate_d1_at_normalized(curve, t_norm)?;
    normalize(d1).ok_or(RgmStatus::DegenerateGeometry)
}

fn evaluate_plane_at_normalized(
    curve: &NurbsCurveData,
    t_norm: f64,
) -> Result<RgmPlane, RgmStatus> {
    let origin = evaluate_curve_at_normalized(curve, t_norm)?;
    let x_axis = evaluate_tangent_at_normalized(curve, t_norm)?;

    let world_up = RgmVec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    let fallback_up = RgmVec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    let mut z_axis = cross(x_axis, world_up);
    if normalize(z_axis).is_none() {
        z_axis = cross(x_axis, fallback_up);
    }
    let z_axis = normalize(z_axis).ok_or(RgmStatus::DegenerateGeometry)?;
    let y_axis = normalize(cross(z_axis, x_axis)).ok_or(RgmStatus::DegenerateGeometry)?;

    Ok(RgmPlane {
        origin,
        x_axis,
        y_axis,
        z_axis,
    })
}

fn evaluate_normal_at_normalized(
    curve: &NurbsCurveData,
    t_norm: f64,
) -> Result<RgmVec3, RgmStatus> {
    let plane = evaluate_plane_at_normalized(curve, t_norm)?;
    Ok(plane.z_axis)
}

fn map_length_to_normalized(curve: &NurbsCurveData, length: f64) -> Result<f64, RgmStatus> {
    if length < 0.0 || length > curve.total_length {
        return Err(RgmStatus::OutOfRange);
    }

    Ok(if curve.total_length <= f64::EPSILON {
        0.0
    } else {
        length / curve.total_length
    })
}

fn find_curve<'a>(
    state: &'a SessionState,
    object: RgmObjectHandle,
) -> Result<&'a NurbsCurveData, RgmStatus> {
    match state.objects.get(&object.0) {
        Some(GeometryObject::NurbsCurve(curve)) => Ok(curve),
        None => Err(RgmStatus::NotFound),
    }
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

    if fit_points.len() <= degree as usize {
        return Err(RgmStatus::InvalidInput);
    }

    let (cumulative, total_length) = cumulative_lengths(&fit_points);
    let params = chord_length_params(&cumulative, total_length);
    let weights = vec![1.0; fit_points.len()];
    let knots = if closed {
        periodic_knots(fit_points.len(), degree as usize)
    } else {
        clamped_open_knots(fit_points.len(), degree as usize, &params)
    };

    Ok(NurbsCurveData {
        degree,
        closed,
        fit_points,
        weights,
        knots,
        params,
        cumulative_lengths: cumulative,
        total_length,
    })
}

fn map_err_with_session(session: RgmKernelHandle, status: RgmStatus, message: &str) -> RgmStatus {
    set_error(session.0, status, message);
    status
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

    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
    let insert_result = with_session_mut(session, |state| {
        state
            .objects
            .insert(object_id, GeometryObject::NurbsCurve(curve));
        Ok(())
    });

    match insert_result {
        Ok(()) => {
            clear_error(session.0);
            // SAFETY: out_object non-null guarded above.
            unsafe {
                *out_object = RgmObjectHandle(object_id);
            }
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Session not found"),
    }
}

#[rgm_export(dotnet = "Create", ts = "create", receiver = "kernel_static")]
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

    // SAFETY: out_session is non-null by the guard above.
    unsafe {
        *out_session = RgmKernelHandle(session_id);
    }

    RgmStatus::Ok
}

#[rgm_export(dotnet = "Dispose", ts = "dispose", receiver = "kernel")]
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

    // SAFETY: out_written is non-null by guard. buffer writes are guarded by null and length checks.
    unsafe {
        *out_written = bytes_to_copy;
        if !buffer.is_null() && buffer_len > 0 {
            std::ptr::copy_nonoverlapping(message_bytes.as_ptr(), buffer, bytes_to_copy);
            *buffer.add(bytes_to_copy) = 0;
        }
    }

    RgmStatus::Ok
}

#[rgm_export(
    dotnet = "InterpolateNurbsFitPoints",
    ts = "interpolateNurbsFitPoints",
    receiver = "kernel"
)]
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

#[rgm_export(dotnet = "PointAt", ts = "pointAt", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_point_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_point: *mut RgmPoint3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let point = evaluate_curve_at_normalized(curve_data, t_norm)?;
        write_point(out_point, point)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Curve point evaluation failed"),
    }
}

#[rgm_export(dotnet = "PointAtLength", ts = "pointAtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_point_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_point: *mut RgmPoint3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let t_norm = map_length_to_normalized(curve_data, distance_length)?;

        let point = evaluate_curve_at_normalized(curve_data, t_norm)?;
        write_point(out_point, point)
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

#[rgm_export(dotnet = "D0", ts = "d0", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d0_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_point: *mut RgmPoint3,
) -> RgmStatus {
    rgm_curve_point_at(session, curve, t_norm, out_point)
}

#[rgm_export(dotnet = "D0AtLength", ts = "d0AtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d0_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_point: *mut RgmPoint3,
) -> RgmStatus {
    rgm_curve_point_at_length(session, curve, distance_length, out_point)
}

#[rgm_export(dotnet = "TangentAt", ts = "tangentAt", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_tangent_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_tangent: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let tangent = evaluate_tangent_at_normalized(curve_data, t_norm)?;
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

#[rgm_export(dotnet = "TangentAtLength", ts = "tangentAtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_tangent_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_tangent: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let t_norm = map_length_to_normalized(curve_data, distance_length)?;
        let tangent = evaluate_tangent_at_normalized(curve_data, t_norm)?;
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

#[rgm_export(dotnet = "NormalAt", ts = "normalAt", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_normal_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_normal: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let normal = evaluate_normal_at_normalized(curve_data, t_norm)?;
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

#[rgm_export(dotnet = "NormalAtLength", ts = "normalAtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_normal_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_normal: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let t_norm = map_length_to_normalized(curve_data, distance_length)?;
        let normal = evaluate_normal_at_normalized(curve_data, t_norm)?;
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

#[rgm_export(dotnet = "D1", ts = "d1", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d1_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_d1: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let d1 = evaluate_d1_at_normalized(curve_data, t_norm)?;
        write_vec(out_d1, d1)
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

#[rgm_export(dotnet = "D1AtLength", ts = "d1AtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d1_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_d1: *mut RgmVec3,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let t_norm = map_length_to_normalized(curve_data, distance_length)?;
        let d1 = evaluate_d1_at_normalized(curve_data, t_norm)?;
        write_vec(out_d1, d1)
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

#[rgm_export(dotnet = "D2", ts = "d2", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d2_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    _t_norm: f64,
    out_derivative: *mut RgmVec3,
) -> RgmStatus {
    let _ = curve;
    let result = with_session_mut(session, |_state| {
        write_vec(
            out_derivative,
            RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        )
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

#[rgm_export(dotnet = "D2AtLength", ts = "d2AtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_d2_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    _distance_length: f64,
    out_derivative: *mut RgmVec3,
) -> RgmStatus {
    rgm_curve_d2_at(session, curve, 0.0, out_derivative)
}

#[rgm_export(dotnet = "PlaneAt", ts = "planeAt", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_plane_at(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    t_norm: f64,
    out_plane: *mut RgmPlane,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let plane = evaluate_plane_at_normalized(curve_data, t_norm)?;
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

#[rgm_export(dotnet = "PlaneAtLength", ts = "planeAtLength", receiver = "curve")]
#[no_mangle]
pub extern "C" fn rgm_curve_plane_at_length(
    session: RgmKernelHandle,
    curve: RgmObjectHandle,
    distance_length: f64,
    out_plane: *mut RgmPlane,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let curve_data = find_curve(state, curve)?;
        let t_norm = map_length_to_normalized(curve_data, distance_length)?;

        let plane = evaluate_plane_at_normalized(curve_data, t_norm)?;
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

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersect_curve_curve(
    session: RgmKernelHandle,
    _curve_a: RgmObjectHandle,
    _curve_b: RgmObjectHandle,
    out_count: *mut usize,
) -> RgmStatus {
    if out_count.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_count pointer");
    }

    // SAFETY: out_count is non-null by guard above.
    unsafe {
        *out_count = 0;
    }

    map_err_with_session(
        session,
        RgmStatus::NotImplemented,
        "Curve-curve intersection is stubbed in this milestone",
    )
}

#[rgm_export]
#[no_mangle]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn rgm_intersect_curve_plane(
    session: RgmKernelHandle,
    _curve: RgmObjectHandle,
    _plane: RgmPlane,
    out_count: *mut usize,
) -> RgmStatus {
    if out_count.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_count pointer");
    }

    // SAFETY: out_count is non-null by guard above.
    unsafe {
        *out_count = 0;
    }

    map_err_with_session(
        session,
        RgmStatus::NotImplemented,
        "Curve-plane intersection is stubbed in this milestone",
    )
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn rgm_intersect_curve_plane(
    session: RgmKernelHandle,
    _curve: RgmObjectHandle,
    _plane: *const RgmPlane,
    out_count: *mut usize,
) -> RgmStatus {
    if _plane.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null plane pointer");
    }

    if out_count.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_count pointer");
    }

    // SAFETY: out_count is non-null by guard above.
    unsafe {
        *out_count = 0;
    }

    map_err_with_session(
        session,
        RgmStatus::NotImplemented,
        "Curve-plane intersection is stubbed in this milestone",
    )
}

#[cfg(test)]
fn debug_get_curve(session: RgmKernelHandle, object: RgmObjectHandle) -> Option<NurbsCurveData> {
    let sessions = SESSIONS.lock().ok()?;
    let state = sessions.get(&session.0)?;
    match state.objects.get(&object.0)? {
        GeometryObject::NurbsCurve(curve) => Some(curve.clone()),
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
    fn alloc_addr_and_dealloc_roundtrip() {
        let ptr = rgm_alloc_addr(32, 8);
        assert_ne!(ptr, 0);
        assert_eq!(rgm_dealloc(ptr as *mut u8, 32, 8), RgmStatus::Ok);
    }

    #[test]
    fn interpolate_with_pointer_tolerance() {
        let session = create_session();
        let points = sample_points();
        let tolerance = RgmToleranceContext {
            abs_tol: 1e-9,
            rel_tol: 1e-9,
            angle_tol: 1e-9,
        };
        let mut object = RgmObjectHandle(0);
        let status = rgm_nurbs_interpolate_fit_points_ptr_tol(
            session,
            points.as_ptr(),
            points.len(),
            2,
            false,
            &tolerance as *const _,
            &mut object as *mut _,
        );

        assert_eq!(status, RgmStatus::Ok);
        assert_ne!(object.0, 0);
        assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn object_release_is_session_scoped() {
        let session_a = create_session();
        let session_b = create_session();

        let points = sample_points();
        let mut object = RgmObjectHandle(0);
        let status = rgm_nurbs_interpolate_fit_points(
            session_a,
            points.as_ptr(),
            points.len(),
            2,
            false,
            RgmToleranceContext {
                abs_tol: 1e-9,
                rel_tol: 1e-9,
                angle_tol: 1e-9,
            },
            &mut object as *mut _,
        );
        assert_eq!(status, RgmStatus::Ok);

        assert_eq!(rgm_object_release(session_b, object), RgmStatus::NotFound);
        assert_eq!(rgm_object_release(session_a, object), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session_a, object), RgmStatus::NotFound);

        assert_eq!(rgm_kernel_destroy(session_a), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session_b), RgmStatus::Ok);
    }

    #[test]
    fn interpolate_open_curve_creates_clamped_knots() {
        let session = create_session();
        let points = sample_points();
        let mut object = RgmObjectHandle(0);

        let status = rgm_nurbs_interpolate_fit_points(
            session,
            points.as_ptr(),
            points.len(),
            2,
            false,
            RgmToleranceContext {
                abs_tol: 1e-9,
                rel_tol: 1e-9,
                angle_tol: 1e-9,
            },
            &mut object as *mut _,
        );
        assert_eq!(status, RgmStatus::Ok);

        let curve = debug_get_curve(session, object).expect("curve exists");
        assert_eq!(curve.weights, vec![1.0; points.len()]);
        assert!((curve.knots[0] - 0.0).abs() < 1e-12);
        assert!((curve.knots[1] - 0.0).abs() < 1e-12);
        assert!((curve.knots[curve.knots.len() - 1] - 1.0).abs() < 1e-12);
        assert!((curve.knots[curve.knots.len() - 2] - 1.0).abs() < 1e-12);

        assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn interpolate_closed_curve_dedups_seam_point() {
        let session = create_session();
        let mut points = sample_points();
        points.push(points[0]);

        let mut object = RgmObjectHandle(0);
        let status = rgm_nurbs_interpolate_fit_points(
            session,
            points.as_ptr(),
            points.len(),
            2,
            true,
            RgmToleranceContext {
                abs_tol: 1e-6,
                rel_tol: 1e-9,
                angle_tol: 1e-9,
            },
            &mut object as *mut _,
        );
        assert_eq!(status, RgmStatus::Ok);

        let curve = debug_get_curve(session, object).expect("curve exists");
        assert_eq!(curve.fit_points.len(), points.len() - 1);
        assert_ne!(curve.knots[0], curve.knots[curve.knots.len() - 1]);

        assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn can_evaluate_point_and_plane() {
        let session = create_session();
        let points = sample_points();
        let mut object = RgmObjectHandle(0);

        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                points.as_ptr(),
                points.len(),
                1,
                false,
                RgmToleranceContext {
                    abs_tol: 1e-9,
                    rel_tol: 1e-9,
                    angle_tol: 1e-9
                },
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
        assert!(point.x >= 1.0 && point.x <= 2.5);

        let mut plane = RgmPlane {
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
        assert_eq!(
            rgm_curve_plane_at(session, object, 0.2, &mut plane as *mut _),
            RgmStatus::Ok
        );

        assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
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
                        RgmToleranceContext {
                            abs_tol: 1e-9,
                            rel_tol: 1e-9,
                            angle_tol: 1e-9,
                        },
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
