#[no_mangle]
pub extern "C" fn rgm_brep_create_empty(
    session: RgmKernelHandle,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_brep_create_empty_impl(session, out_brep)
}

#[no_mangle]
pub extern "C" fn rgm_brep_create_from_faces(
    session: RgmKernelHandle,
    faces: *const RgmObjectHandle,
    face_count: usize,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_brep_create_from_faces_impl(session, faces, face_count, out_brep)
}

#[no_mangle]
pub extern "C" fn rgm_brep_create_from_surface(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_brep_create_from_surface_impl(session, surface, out_brep)
}

#[no_mangle]
pub extern "C" fn rgm_brep_add_face(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    face: RgmObjectHandle,
    out_face_id: *mut u32,
) -> RgmStatus {
    rgm_brep_add_face_impl(session, brep, face, out_face_id)
}

#[no_mangle]
pub extern "C" fn rgm_brep_add_face_from_surface(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    surface: RgmObjectHandle,
    out_face_id: *mut u32,
) -> RgmStatus {
    rgm_brep_add_face_from_surface_impl(session, brep, surface, out_face_id)
}

#[no_mangle]
pub extern "C" fn rgm_brep_add_loop_uv(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    face_id: u32,
    points: *const RgmUv2,
    point_count: usize,
    is_outer: bool,
    out_loop_id: *mut u32,
) -> RgmStatus {
    rgm_brep_add_loop_uv_impl(
        session,
        brep,
        face_id,
        points,
        point_count,
        is_outer,
        out_loop_id,
    )
}

#[no_mangle]
pub extern "C" fn rgm_brep_finalize_shell(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_shell_id: *mut u32,
) -> RgmStatus {
    rgm_brep_finalize_shell_impl(session, brep, out_shell_id)
}

#[no_mangle]
pub extern "C" fn rgm_brep_finalize_solid(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_solid_id: *mut u32,
) -> RgmStatus {
    rgm_brep_finalize_solid_impl(session, brep, out_solid_id)
}

#[no_mangle]
pub extern "C" fn rgm_brep_validate(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_report: *mut RgmBrepValidationReport,
) -> RgmStatus {
    rgm_brep_validate_impl(session, brep, out_report)
}

#[no_mangle]
pub extern "C" fn rgm_brep_heal(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_fixed_count: *mut u32,
) -> RgmStatus {
    rgm_brep_heal_impl(session, brep, out_fixed_count)
}

#[no_mangle]
pub extern "C" fn rgm_brep_clone(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_brep_clone_impl(session, brep, out_brep)
}

#[no_mangle]
pub extern "C" fn rgm_brep_face_count(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    rgm_brep_face_count_impl(session, brep, out_count)
}

#[no_mangle]
pub extern "C" fn rgm_brep_shell_count(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    rgm_brep_shell_count_impl(session, brep, out_count)
}

#[no_mangle]
pub extern "C" fn rgm_brep_solid_count(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    rgm_brep_solid_count_impl(session, brep, out_count)
}

#[no_mangle]
pub extern "C" fn rgm_brep_is_solid(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_is_solid: *mut bool,
) -> RgmStatus {
    rgm_brep_is_solid_impl(session, brep, out_is_solid)
}

#[no_mangle]
pub extern "C" fn rgm_brep_face_adjacency(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    face_id: u32,
    out_face_ids: *mut u32,
    face_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    rgm_brep_face_adjacency_impl(
        session,
        brep,
        face_id,
        out_face_ids,
        face_capacity,
        out_count,
    )
}

#[no_mangle]
pub extern "C" fn rgm_brep_tessellate_to_mesh(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    options: *const RgmSurfaceTessellationOptions,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    let options = if options.is_null() {
        None
    } else {
        Some(unsafe { *options })
    };
    rgm_brep_tessellate_to_mesh_impl(session, brep, options, out_mesh)
}

#[no_mangle]
pub extern "C" fn rgm_brep_from_face_object(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_brep_from_face_object_impl(session, face, out_brep)
}

#[no_mangle]
pub extern "C" fn rgm_brep_extract_face_object(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    face_id: u32,
    out_face: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_brep_extract_face_object_impl(session, brep, face_id, out_face)
}

#[no_mangle]
pub extern "C" fn rgm_brep_state(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_state: *mut u32,
) -> RgmStatus {
    rgm_brep_state_impl(session, brep, out_state)
}

#[no_mangle]
pub extern "C" fn rgm_brep_estimate_area(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_area: *mut f64,
) -> RgmStatus {
    rgm_brep_estimate_area_impl(session, brep, out_area)
}

#[no_mangle]
pub extern "C" fn rgm_brep_save_native(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_bytes: *mut u8,
    byte_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    rgm_brep_save_native_impl(session, brep, out_bytes, byte_capacity, out_count)
}

#[no_mangle]
pub extern "C" fn rgm_brep_load_native(
    session: RgmKernelHandle,
    bytes: *const u8,
    byte_count: usize,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    rgm_brep_load_native_impl(session, bytes, byte_count, out_brep)
}
