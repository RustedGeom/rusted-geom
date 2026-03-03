#[no_mangle]
pub extern "C" fn rgm_face_create_from_surface(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    out_face: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_face_create_from_surface_impl(session, surface, out_face)
}

#[no_mangle]
pub extern "C" fn rgm_face_add_loop(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    points: *const RgmUv2,
    point_count: usize,
    is_outer: bool,
) -> RgmStatus {
    rgm_face_add_loop_impl(session, face, points, point_count, is_outer)
}

#[no_mangle]
pub extern "C" fn rgm_face_add_loop_edges(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    loop_input: *const RgmTrimLoopInput,
    edges: *const RgmTrimEdgeInput,
    edge_count: usize,
) -> RgmStatus {
    if loop_input.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null trim loop input");
    }

    // SAFETY: loop_input is non-null by guard above.
    let loop_input = unsafe { *loop_input };
    rgm_face_add_loop_edges_impl(session, face, loop_input, edges, edge_count)
}

#[no_mangle]
pub extern "C" fn rgm_face_remove_loop(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    loop_index: u32,
) -> RgmStatus {
    rgm_face_remove_loop_impl(session, face, loop_index)
}

#[no_mangle]
pub extern "C" fn rgm_face_split_trim_edge(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    loop_index: u32,
    edge_index: u32,
    split_t: f64,
) -> RgmStatus {
    rgm_face_split_trim_edge_impl(session, face, loop_index, edge_index, split_t)
}

#[no_mangle]
pub extern "C" fn rgm_face_reverse_loop(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    loop_index: u32,
) -> RgmStatus {
    rgm_face_reverse_loop_impl(session, face, loop_index)
}

#[no_mangle]
pub extern "C" fn rgm_face_validate(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    out_valid: *mut bool,
) -> RgmStatus {
    rgm_face_validate_impl(session, face, out_valid)
}

#[no_mangle]
pub extern "C" fn rgm_face_heal(session: RgmKernelHandle, face: RgmObjectHandle) -> RgmStatus {
    rgm_face_heal_impl(session, face)
}

#[no_mangle]
pub extern "C" fn rgm_face_tessellate_to_mesh(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    options: *const RgmSurfaceTessellationOptions,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    let options = if options.is_null() {
        None
    } else {
        // SAFETY: options is non-null in this branch.
        Some(unsafe { *options })
    };

    rgm_face_tessellate_to_mesh_impl(session, face, options, out_mesh)
}
