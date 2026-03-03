#[no_mangle]
pub extern "C" fn rgm_surface_create_nurbs(
    session: RgmKernelHandle,
    desc: *const RgmNurbsSurfaceDesc,
    control_points: *const RgmPoint3,
    control_point_count: usize,
    weights: *const f64,
    weight_count: usize,
    knots_u: *const f64,
    knot_u_count: usize,
    knots_v: *const f64,
    knot_v_count: usize,
    tol: *const RgmToleranceContext,
    out_surface: *mut RgmObjectHandle,
) -> RgmStatus {
    if desc.is_null() || tol.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null surface descriptor");
    }
    // SAFETY: pointer is non-null by guard above.
    let desc = unsafe { *desc };
    // SAFETY: pointer is non-null by guard above.
    let tol = unsafe { *tol };
    rgm_surface_create_nurbs_impl(
        session,
        desc,
        control_points,
        control_point_count,
        weights,
        weight_count,
        knots_u,
        knot_u_count,
        knots_v,
        knot_v_count,
        tol,
        out_surface,
    )
}

#[no_mangle]
pub extern "C" fn rgm_surface_point_at(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    uv_norm: *const RgmUv2,
    out_point: *mut RgmPoint3,
) -> RgmStatus {
    if uv_norm.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null uv pointer");
    }
    // SAFETY: pointer is non-null by guard above.
    let uv_norm = unsafe { *uv_norm };
    let result = with_session_mut(session, |state| {
        let surface = find_surface(state, surface)?;
        let frame = eval_surface_data_normalized(surface, uv_norm)?;
        write_out(out_point, frame.point)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Surface point evaluation failed"),
    }
}

#[no_mangle]
pub extern "C" fn rgm_surface_d1_at(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    uv_norm: *const RgmUv2,
    out_du: *mut RgmVec3,
    out_dv: *mut RgmVec3,
) -> RgmStatus {
    if uv_norm.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null uv pointer");
    }
    // SAFETY: pointer is non-null by guard above.
    let uv_norm = unsafe { *uv_norm };
    let result = with_session_mut(session, |state| {
        let surface = find_surface(state, surface)?;
        let frame = eval_surface_data_normalized(surface, uv_norm)?;
        write_out(out_du, frame.du)?;
        write_out(out_dv, frame.dv)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Surface first derivatives failed"),
    }
}

#[no_mangle]
pub extern "C" fn rgm_surface_d2_at(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    uv_norm: *const RgmUv2,
    out_duu: *mut RgmVec3,
    out_duv: *mut RgmVec3,
    out_dvv: *mut RgmVec3,
) -> RgmStatus {
    if uv_norm.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null uv pointer");
    }
    // SAFETY: pointer is non-null by guard above.
    let uv_norm = unsafe { *uv_norm };
    let result = with_session_mut(session, |state| {
        let surface = find_surface(state, surface)?;
        let eval = eval_nurbs_surface_normalized(&surface.core, uv_norm)?;
        let duu = matrix_apply_vec(surface.transform, eval.duu);
        let duv = matrix_apply_vec(surface.transform, eval.duv);
        let dvv = matrix_apply_vec(surface.transform, eval.dvv);
        write_out(out_duu, duu)?;
        write_out(out_duv, duv)?;
        write_out(out_dvv, dvv)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Surface second derivatives failed"),
    }
}

#[no_mangle]
pub extern "C" fn rgm_surface_normal_at(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    uv_norm: *const RgmUv2,
    out_normal: *mut RgmVec3,
) -> RgmStatus {
    if uv_norm.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null uv pointer");
    }
    // SAFETY: pointer is non-null by guard above.
    let uv_norm = unsafe { *uv_norm };
    let result = with_session_mut(session, |state| {
        let surface = find_surface(state, surface)?;
        let frame = eval_surface_data_normalized(surface, uv_norm)?;
        write_out(out_normal, frame.normal)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Surface normal evaluation failed"),
    }
}

#[no_mangle]
pub extern "C" fn rgm_surface_frame_at(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    uv_norm: *const RgmUv2,
    out_frame: *mut RgmSurfaceEvalFrame,
) -> RgmStatus {
    if uv_norm.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null uv pointer");
    }
    // SAFETY: pointer is non-null by guard above.
    let uv_norm = unsafe { *uv_norm };
    let result = with_session_mut(session, |state| {
        let surface = find_surface(state, surface)?;
        let frame = eval_surface_data_normalized(surface, uv_norm)?;
        write_out(out_frame, frame)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Surface frame evaluation failed"),
    }
}

#[no_mangle]
pub extern "C" fn rgm_surface_translate(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    delta: *const RgmVec3,
    out_surface: *mut RgmObjectHandle,
) -> RgmStatus {
    if delta.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null translation vector");
    }
    // SAFETY: pointer is non-null by guard above.
    let delta = unsafe { *delta };
    let transform = matrix_translation(delta);
    rgm_surface_transform_impl(
        session,
        surface,
        transform,
        out_surface,
        "Surface translation failed",
    )
}

#[no_mangle]
pub extern "C" fn rgm_surface_rotate(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    axis: *const RgmVec3,
    angle_rad: f64,
    pivot: *const RgmPoint3,
    out_surface: *mut RgmObjectHandle,
) -> RgmStatus {
    if axis.is_null() || pivot.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null rotation pointer");
    }
    // SAFETY: pointer is non-null by guard above.
    let axis = unsafe { *axis };
    // SAFETY: pointer is non-null by guard above.
    let pivot = unsafe { *pivot };
    let rotation = match matrix_rotation(axis, angle_rad) {
        Ok(value) => value,
        Err(status) => return map_err_with_session(session, status, "Surface rotation failed"),
    };
    let transform = matrix_about_pivot(rotation, pivot);
    rgm_surface_transform_impl(
        session,
        surface,
        transform,
        out_surface,
        "Surface rotation failed",
    )
}

#[no_mangle]
pub extern "C" fn rgm_surface_scale(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    scale: *const RgmVec3,
    pivot: *const RgmPoint3,
    out_surface: *mut RgmObjectHandle,
) -> RgmStatus {
    if scale.is_null() || pivot.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null scale pointer");
    }
    // SAFETY: pointer is non-null by guard above.
    let scale = unsafe { *scale };
    // SAFETY: pointer is non-null by guard above.
    let pivot = unsafe { *pivot };
    let transform = matrix_about_pivot(matrix_scale(scale), pivot);
    rgm_surface_transform_impl(
        session,
        surface,
        transform,
        out_surface,
        "Surface scale failed",
    )
}

#[no_mangle]
pub extern "C" fn rgm_surface_bake_transform(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    out_surface: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_surface_bake_transform_impl(session, surface, out_surface)
}

#[no_mangle]
pub extern "C" fn rgm_surface_tessellate_to_mesh(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    options: *const RgmSurfaceTessellationOptions,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    let options = if options.is_null() {
        None
    } else {
        // SAFETY: pointer is non-null by guard above.
        Some(unsafe { *options })
    };
    rgm_surface_tessellate_to_mesh_impl(session, surface, options, out_mesh)
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
