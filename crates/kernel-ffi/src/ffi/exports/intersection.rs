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
        write_out(out_curve, handle)
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
        write_out(out_point, eval.point)
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
        write_out(out_length, length)
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
        write_out(out_length, length)
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
        write_out(out_point, eval.point)
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
        write_out(out_d1, eval.d1)
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
        write_out(out_d1, eval.d1)
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
        write_out(out_derivative, eval.d2)
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
        write_out(out_derivative, eval.d2)
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
        write_out(out_tangent, tangent)
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
        write_out(out_tangent, tangent)
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
        write_out(out_normal, normal)
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
        write_out(out_normal, normal)
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
        write_out(out_plane, plane)
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
        write_out(out_plane, plane)
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
        write_out(out_point, converted)
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
        write_slice(out_points, point_capacity, &points, out_count)
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
        write_slice(out_points, point_capacity, &points, out_count)
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
pub extern "C" fn rgm_intersect_surface_surface(
    session: RgmKernelHandle,
    surface_a: RgmObjectHandle,
    surface_b: RgmObjectHandle,
    out_intersection: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_intersect_surface_surface_impl(session, surface_a, surface_b, out_intersection)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersect_surface_plane(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    plane: *const RgmPlane,
    out_intersection: *mut RgmObjectHandle,
) -> RgmStatus {
    if plane.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null plane pointer");
    }
    // SAFETY: pointer is non-null by guard above.
    let plane = unsafe { *plane };
    rgm_intersect_surface_plane_impl(session, surface, plane, out_intersection)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersect_surface_curve(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    curve: RgmObjectHandle,
    out_intersection: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_intersect_surface_curve_impl(session, surface, curve, out_intersection)
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersection_branch_count(
    session: RgmKernelHandle,
    intersection: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let intersection_data = find_intersection(state, intersection)?;
        write_out(
            out_count,
            intersection_data
                .branches
                .len()
                .try_into()
                .unwrap_or(u32::MAX),
        )
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Intersection branch count failed"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersection_branch_summary(
    session: RgmKernelHandle,
    intersection: RgmObjectHandle,
    branch_index: u32,
    out_summary: *mut RgmIntersectionBranchSummary,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let intersection_data = find_intersection(state, intersection)?;
        let idx = branch_index as usize;
        if idx >= intersection_data.branches.len() {
            return Err(RgmStatus::OutOfRange);
        }
        let branch = &intersection_data.branches[idx];
        let summary = RgmIntersectionBranchSummary {
            point_count: branch.points.len().try_into().unwrap_or(u32::MAX),
            uv_a_count: branch.uv_a.len().try_into().unwrap_or(u32::MAX),
            uv_b_count: branch.uv_b.len().try_into().unwrap_or(u32::MAX),
            curve_t_count: branch.curve_t.len().try_into().unwrap_or(u32::MAX),
            closed: branch.closed,
            flags: branch.flags,
        };
        write_out(out_summary, summary)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Intersection summary failed"),
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersection_copy_branch_points(
    session: RgmKernelHandle,
    intersection: RgmObjectHandle,
    branch_index: u32,
    out_points: *mut RgmPoint3,
    point_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let intersection_data = find_intersection(state, intersection)?;
        let idx = branch_index as usize;
        if idx >= intersection_data.branches.len() {
            return Err(RgmStatus::OutOfRange);
        }
        write_slice(
            out_points,
            point_capacity,
            &intersection_data.branches[idx].points,
            out_count,
        )
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Intersection branch points copy failed")
        }
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersection_copy_branch_uv_on_surface_a(
    session: RgmKernelHandle,
    intersection: RgmObjectHandle,
    branch_index: u32,
    out_points: *mut RgmUv2,
    point_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let intersection_data = find_intersection(state, intersection)?;
        let idx = branch_index as usize;
        if idx >= intersection_data.branches.len() {
            return Err(RgmStatus::OutOfRange);
        }
        write_slice(
            out_points,
            point_capacity,
            &intersection_data.branches[idx].uv_a,
            out_count,
        )
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Intersection branch uv-a copy failed")
        }
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersection_copy_branch_uv_on_surface_b(
    session: RgmKernelHandle,
    intersection: RgmObjectHandle,
    branch_index: u32,
    out_points: *mut RgmUv2,
    point_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let intersection_data = find_intersection(state, intersection)?;
        let idx = branch_index as usize;
        if idx >= intersection_data.branches.len() {
            return Err(RgmStatus::OutOfRange);
        }
        write_slice(
            out_points,
            point_capacity,
            &intersection_data.branches[idx].uv_b,
            out_count,
        )
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Intersection branch uv-b copy failed")
        }
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersection_copy_branch_curve_t(
    session: RgmKernelHandle,
    intersection: RgmObjectHandle,
    branch_index: u32,
    out_values: *mut f64,
    value_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let intersection_data = find_intersection(state, intersection)?;
        let idx = branch_index as usize;
        if idx >= intersection_data.branches.len() {
            return Err(RgmStatus::OutOfRange);
        }
        write_slice(
            out_values,
            value_capacity,
            &intersection_data.branches[idx].curve_t,
            out_count,
        )
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => {
            map_err_with_session(session, status, "Intersection branch curve-t copy failed")
        }
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_intersection_branch_to_nurbs(
    session: RgmKernelHandle,
    intersection: RgmObjectHandle,
    branch_index: u32,
    tol: *const RgmToleranceContext,
    out_curve: *mut RgmObjectHandle,
) -> RgmStatus {
    if tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null tolerance pointer");
    }
    // SAFETY: pointer is non-null by guard above.
    let tol = unsafe { *tol };
    if out_curve.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_curve pointer");
    }

    let result = with_session_mut(session, |state| {
        let intersection_data = find_intersection(state, intersection)?;
        let idx = branch_index as usize;
        if idx >= intersection_data.branches.len() {
            return Err(RgmStatus::OutOfRange);
        }
        let branch = &intersection_data.branches[idx];
        if branch.points.len() < 2 {
            return Err(RgmStatus::DegenerateGeometry);
        }
        let nurbs = build_open_nurbs_from_points(&branch.points, 1, tol, branch.points.clone())?;
        let handle = insert_curve(state, CurveData::NurbsCurve(nurbs));
        write_out(out_curve, handle)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Intersection branch to nurbs failed"),
    }
}
