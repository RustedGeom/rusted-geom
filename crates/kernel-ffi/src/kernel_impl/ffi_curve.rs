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

    // SAFETY: pointers are non-null by guard above.
    let line = unsafe { *line };
    let tol = unsafe { *tol };
    rgm_curve_create_line_impl(session, line, tol, out_object)
}

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

    // SAFETY: pointers are non-null by guard above.
    let circle = unsafe { *circle };
    let tol = unsafe { *tol };
    rgm_curve_create_circle_impl(session, circle, tol, out_object)
}

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

    // SAFETY: pointers are non-null by guard above.
    let arc = unsafe { *arc };
    let tol = unsafe { *tol };
    rgm_curve_create_arc_impl(session, arc, tol, out_object)
}

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

    // SAFETY: pointers are non-null by guard above.
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

    // SAFETY: pointers are non-null by guard above.
    let start = unsafe { *start };
    let mid = unsafe { *mid };
    let end = unsafe { *end };
    let tol = unsafe { *tol };
    rgm_curve_create_arc_by_3_points_impl(session, start, mid, end, tol, out_object)
}

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

    // SAFETY: tol is non-null by guard above.
    let tol = unsafe { *tol };
    rgm_curve_create_polyline_impl(session, points, point_count, closed, tol, out_object)
}

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
