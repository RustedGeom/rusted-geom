// ─── FFI: Volume ─────────────────────────────────────────────────────────────
//
// C-callable function for mesh volume computation.
// This file is include!-ed from ffi_impl.rs.

#[no_mangle]
pub extern "C" fn rgm_mesh_volume(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_volume: *mut f64,
) -> RgmStatus {
    rgm_mesh_volume_impl(session, mesh, out_volume)
}

fn rgm_mesh_volume_impl(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_volume: *mut f64,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let mesh_data = find_mesh(state, mesh)?;
        let world_verts = mesh_world_vertices(mesh_data);
        let vol = mesh_volume_compute(&world_verts, &mesh_data.triangles);
        write_out(out_volume, vol)
    });
    match result {
        Ok(()) => RgmStatus::Ok,
        Err(status) => map_err_with_session(session, status, "Mesh volume failed"),
    }
}
