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

#[no_mangle]
pub extern "C" fn rgm_mesh_bake_transform(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_mesh_bake_transform_impl(session, mesh, out_mesh)
}

#[no_mangle]
pub extern "C" fn rgm_mesh_vertex_count(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let mesh = find_mesh(state, mesh)?;
        write_out(
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

#[no_mangle]
pub extern "C" fn rgm_mesh_triangle_count(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let mesh = find_mesh(state, mesh)?;
        write_out(
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
        write_slice(out_points, point_capacity, &segments, out_count)
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Mesh-plane intersection failed"),
    }
}

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
        write_slice(out_points, point_capacity, &segments, out_count)
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Mesh-mesh intersection failed"),
    }
}

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
