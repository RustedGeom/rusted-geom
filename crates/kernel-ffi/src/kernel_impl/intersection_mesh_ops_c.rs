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

fn create_surface_object(
    session: RgmKernelHandle,
    out_object: *mut RgmObjectHandle,
    build: impl FnOnce(&SessionState) -> Result<SurfaceData, RgmStatus>,
    message: &str,
) -> RgmStatus {
    if out_object.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_object pointer");
    }

    let result = with_session_mut(session, |state| {
        let surface = build(state)?;
        let handle = insert_surface(state, surface);
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

fn create_face_object(
    session: RgmKernelHandle,
    out_object: *mut RgmObjectHandle,
    build: impl FnOnce(&SessionState) -> Result<FaceData, RgmStatus>,
    message: &str,
) -> RgmStatus {
    if out_object.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_object pointer");
    }

    let result = with_session_mut(session, |state| {
        let face = build(state)?;
        let handle = insert_face(state, face);
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

fn create_intersection_object(
    session: RgmKernelHandle,
    out_object: *mut RgmObjectHandle,
    build: impl FnOnce(&SessionState) -> Result<IntersectionData, RgmStatus>,
    message: &str,
) -> RgmStatus {
    if out_object.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_object pointer");
    }

    let result = with_session_mut(session, |state| {
        let intersection = build(state)?;
        let handle = insert_intersection(state, intersection);
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

