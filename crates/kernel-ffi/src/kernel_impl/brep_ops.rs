use crate::elements::brep::bridge::{
    add_face_to_brep, add_surface_face_to_brep, add_uv_loop_to_face, face_from_brep,
};
use crate::elements::brep::ids::{FaceId, ShellId};
use crate::elements::brep::io::{decode_native_brep, encode_native_brep};
use crate::elements::brep::types::{BrepData, BrepShell, BrepSolid};
use crate::elements::brep::validate::{heal_brep_data, report_has_errors, validate_brep_data};
use crate::session::objects::{find_face, find_surface, GeometryObject, MeshData, SessionState, SurfaceData};
use crate::session::store::{
    insert_brep, insert_face, insert_mesh, with_session_mut,
};
use smallvec::SmallVec;

// P1: finish() no longer calls clear_error; with_session_mut clears it on success.
fn finish(session: RgmKernelHandle, result: Result<(), RgmStatus>, message: &str) -> RgmStatus {
    match result {
        Ok(()) => RgmStatus::Ok,
        Err(status) => map_err_with_session(session, status, message),
    }
}

// S1: with_brep_in_progress_mut now checks finalized flag instead of enum variant.
fn with_brep_in_progress_mut<'a>(
    state: &'a mut SessionState,
    brep: RgmObjectHandle,
) -> Result<&'a mut BrepData, RgmStatus> {
    match state.objects.get_mut(&brep.0) {
        Some(GeometryObject::Brep(data)) if !data.finalized => Ok(data),
        Some(GeometryObject::Brep(_)) => Err(RgmStatus::InvalidInput),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

fn with_brep_any_ref<'a>(state: &'a SessionState, brep: RgmObjectHandle) -> Result<&'a BrepData, RgmStatus> {
    match state.objects.get(&brep.0) {
        Some(GeometryObject::Brep(data)) => Ok(data),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

fn with_brep_any_mut<'a>(
    state: &'a mut SessionState,
    brep: RgmObjectHandle,
) -> Result<&'a mut BrepData, RgmStatus> {
    match state.objects.get_mut(&brep.0) {
        Some(GeometryObject::Brep(data)) => Ok(data),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

fn ensure_shells(brep: &mut BrepData) -> u32 {
    if brep.shells.is_empty() {
        let faces = brep.faces.indices().collect::<Vec<FaceId>>();
        let shell_id = brep.shells.push(BrepShell {
            faces,
            closed: false,
        });
        shell_id.raw()
    } else {
        0
    }
}

fn ensure_primary_solid(brep: &mut BrepData) -> u32 {
    if brep.shells.is_empty() {
        let _ = ensure_shells(brep);
    }
    let shells = brep.shells.indices().collect::<Vec<ShellId>>();
    if brep.solids.is_empty() {
        let solid_id = brep.solids.push(BrepSolid { shells });
        solid_id.raw()
    } else {
        // C2: Use first_mut() rather than SolidId::from_raw(0) to avoid index assumptions.
        if let Some(solid) = brep.solids.as_raw_slice_mut().first_mut() {
            solid.shells = shells;
        }
        0
    }
}

fn update_closed_state(brep: &mut BrepData) {
    let closed = !brep.edges.is_empty() && brep.edges.iter().all(|edge| edge.trims.len() == 2);
    for shell in brep.shells.iter_mut() {
        shell.closed = closed;
    }
}

// P4: Pre-allocate by summing sizes before the merge loop.
fn merge_meshes(meshes: &[MeshData]) -> Result<MeshData, RgmStatus> {
    let total_verts: usize = meshes.iter().map(|m| m.vertices.len()).sum();
    let total_tris: usize = meshes.iter().map(|m| m.triangles.len()).sum();
    let mut vertices = Vec::with_capacity(total_verts);
    let mut triangles = Vec::with_capacity(total_tris);

    for mesh in meshes {
        let base = u32::try_from(vertices.len()).map_err(|_| RgmStatus::OutOfRange)?;
        vertices.extend(mesh.vertices.iter().copied());
        for tri in &mesh.triangles {
            triangles.push([
                tri[0].checked_add(base).ok_or(RgmStatus::OutOfRange)?,
                tri[1].checked_add(base).ok_or(RgmStatus::OutOfRange)?,
                tri[2].checked_add(base).ok_or(RgmStatus::OutOfRange)?,
            ]);
        }
    }

    Ok(MeshData {
        vertices,
        triangles,
        transform: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    })
}

fn tessellate_brep(
    state: &SessionState,
    brep: &BrepData,
    options: Option<RgmSurfaceTessellationOptions>,
) -> Result<MeshData, RgmStatus> {
    if brep.faces.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }

    let mut jobs: Vec<(SurfaceData, crate::session::objects::FaceData)> =
        Vec::with_capacity(brep.faces.len());
    for (face_id, _) in brep.faces.iter_enumerated() {
        let face = face_from_brep(brep, face_id.raw())?;
        let surface = find_surface(state, face.surface)?.clone();
        jobs.push((surface, face));
    }

    #[cfg(not(target_arch = "wasm32"))]
    let meshes: Vec<MeshData> = {
        if jobs.len() >= 6 {
            use rayon::prelude::*;
            let meshes: Result<Vec<MeshData>, RgmStatus> = jobs
                .par_iter()
                .map(|(surface, face)| {
                    let samples = tessellate_surface_samples(surface, Some(face), options)?;
                    Ok(build_mesh_from_tessellation(&samples))
                })
                .collect();
            meshes?
        } else {
            let mut meshes = Vec::with_capacity(jobs.len());
            for (surface, face) in &jobs {
                let samples = tessellate_surface_samples(surface, Some(face), options)?;
                meshes.push(build_mesh_from_tessellation(&samples));
            }
            meshes
        }
    };

    #[cfg(target_arch = "wasm32")]
    let meshes: Vec<MeshData> = {
        let mut meshes = Vec::with_capacity(jobs.len());
        for (surface, face) in &jobs {
            let samples = tessellate_surface_samples(surface, Some(face), options)?;
            meshes.push(build_mesh_from_tessellation(&samples));
        }
        meshes
    };

    merge_meshes(&meshes)
}

fn compute_face_neighbors(brep: &BrepData) -> Vec<SmallVec<[FaceId; 6]>> {
    let mut out = vec![SmallVec::new(); brep.faces.len()];
    for edge in brep.edges.iter() {
        let mut touched = SmallVec::<[FaceId; 4]>::new();
        for &trim_id in &edge.trims {
            if trim_id.index() < brep.trims.len() {
                let face = brep.trims[trim_id].face;
                if !touched.contains(&face) {
                    touched.push(face);
                }
            }
        }
        for &a in &touched {
            for &b in &touched {
                if a != b && a.index() < out.len() && !out[a.index()].contains(&b) {
                    out[a.index()].push(b);
                }
            }
        }
    }
    out
}

// S3: adjacency_dirty flag removed; is_none() is the single dirty indicator.
fn face_adjacency_cached(brep: &mut BrepData, face_id: FaceId) -> Vec<u32> {
    if brep.cache.face_neighbors.is_none() {
        brep.cache.face_neighbors = Some(compute_face_neighbors(brep));
    }
    if let Some(neighbors) = &brep.cache.face_neighbors {
        if face_id.index() < neighbors.len() {
            let mut out = neighbors[face_id.index()]
                .iter()
                .map(|id| id.raw())
                .collect::<Vec<u32>>();
            out.sort_unstable();
            out.dedup();
            return out;
        }
    }
    Vec::new()
}

fn read_faces<'a>(faces: *const RgmObjectHandle, face_count: usize) -> Result<&'a [RgmObjectHandle], RgmStatus> {
    if face_count > 0 && faces.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    Ok(unsafe { std::slice::from_raw_parts(faces, face_count) })
}

fn read_points<'a>(points: *const RgmUv2, point_count: usize) -> Result<&'a [RgmUv2], RgmStatus> {
    if point_count > 0 && points.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    Ok(unsafe { std::slice::from_raw_parts(points, point_count) })
}

fn read_bytes<'a>(bytes: *const u8, byte_count: usize) -> Result<&'a [u8], RgmStatus> {
    if byte_count > 0 && bytes.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    Ok(unsafe { std::slice::from_raw_parts(bytes, byte_count) })
}

pub(crate) fn rgm_brep_create_empty_impl(
    session: RgmKernelHandle,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        // S1: insert_brep with finalized=false (was insert_brep_in_progress)
        let handle = insert_brep(state, BrepData::new());
        write_out(out_brep, handle)
    });
    finish(session, result, "BREP create empty failed")
}

pub(crate) fn rgm_brep_create_from_faces_impl(
    session: RgmKernelHandle,
    faces: *const RgmObjectHandle,
    face_count: usize,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    let faces = match read_faces(faces, face_count) {
        Ok(value) => value,
        Err(status) => return map_err_with_session(session, status, "Invalid BREP face array"),
    };

    let result = with_session_mut(session, |state| {
        let mut brep = BrepData::new();
        for &face_handle in faces {
            let face = find_face(state, face_handle)?.clone();
            add_face_to_brep(&mut brep, &face)?;
        }
        // S1: finalized=false (was insert_brep_in_progress)
        let handle = insert_brep(state, brep);
        write_out(out_brep, handle)
    });

    finish(session, result, "BREP create from faces failed")
}

pub(crate) fn rgm_brep_create_from_surface_impl(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let _ = find_surface(state, surface)?;
        let mut brep = BrepData::new();
        add_surface_face_to_brep(&mut brep, surface);
        // S1: finalized=false
        let handle = insert_brep(state, brep);
        write_out(out_brep, handle)
    });

    finish(session, result, "BREP create from surface failed")
}

pub(crate) fn rgm_brep_add_face_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    face: RgmObjectHandle,
    out_face_id: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let face_data = find_face(state, face)?.clone();
        let brep_data = with_brep_in_progress_mut(state, brep)?;
        let face_id = add_face_to_brep(brep_data, &face_data)?;
        write_out(out_face_id, face_id.raw())
    });

    finish(session, result, "BREP add face failed")
}

pub(crate) fn rgm_brep_add_face_from_surface_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    surface: RgmObjectHandle,
    out_face_id: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let _ = find_surface(state, surface)?;
        let brep_data = with_brep_in_progress_mut(state, brep)?;
        let face_id = add_surface_face_to_brep(brep_data, surface);
        write_out(out_face_id, face_id.raw())
    });

    finish(session, result, "BREP add face from surface failed")
}

pub(crate) fn rgm_brep_add_loop_uv_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    face_id: u32,
    points: *const RgmUv2,
    point_count: usize,
    is_outer: bool,
    out_loop_id: *mut u32,
) -> RgmStatus {
    let points = match read_points(points, point_count) {
        Ok(value) => value,
        Err(status) => return map_err_with_session(session, status, "Invalid BREP loop points"),
    };

    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_in_progress_mut(state, brep)?;
        let fid = FaceId::from_raw(face_id);
        if fid.index() >= brep_data.faces.len() {
            return Err(RgmStatus::OutOfRange);
        }
        let loop_id = add_uv_loop_to_face(brep_data, fid, points, is_outer)?;
        write_out(out_loop_id, loop_id.raw())
    });

    finish(session, result, "BREP add loop failed")
}

pub(crate) fn rgm_brep_finalize_shell_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_shell_id: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        // S1: check finalized flag instead of matching on BrepInProgress/Brep enum variant
        let brep_data = with_brep_any_mut(state, brep)?;
        if brep_data.finalized {
            return Err(RgmStatus::InvalidInput);
        }
        let shell_id = ensure_shells(brep_data);
        update_closed_state(brep_data);
        brep_data.finalized = true;
        write_out(out_shell_id, shell_id)
    });

    finish(session, result, "BREP finalize shell failed")
}

pub(crate) fn rgm_brep_finalize_solid_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_solid_id: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        // S1: check finalized flag; only do shell/closed-state setup if not yet finalized
        let brep_data = with_brep_any_mut(state, brep)?;
        if !brep_data.finalized {
            let _ = ensure_shells(brep_data);
            update_closed_state(brep_data);
        }
        let solid_id = ensure_primary_solid(brep_data);
        brep_data.invalidate_topology();
        brep_data.finalized = true;
        write_out(out_solid_id, solid_id)
    });

    finish(session, result, "BREP finalize solid failed")
}

pub(crate) fn rgm_brep_validate_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_report: *mut RgmBrepValidationReport,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_any_ref(state, brep)?;
        let report = validate_brep_data(brep_data);
        write_out(out_report, report)
    });

    finish(session, result, "BREP validate failed")
}

pub(crate) fn rgm_brep_heal_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_fixed_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_any_mut(state, brep)?;
        let fixed = heal_brep_data(brep_data);
        write_out(out_fixed_count, fixed)
    });

    finish(session, result, "BREP heal failed")
}

pub(crate) fn rgm_brep_clone_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        // S1: single variant, no BrepInProgress arm needed
        let cloned = with_brep_any_ref(state, brep)?.clone();
        let handle = insert_brep(state, cloned);
        write_out(out_brep, handle)
    });

    finish(session, result, "BREP clone failed")
}

pub(crate) fn rgm_brep_face_count_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_any_ref(state, brep)?;
        write_out(out_count, u32::try_from(brep_data.faces.len()).unwrap_or(u32::MAX))
    });

    finish(session, result, "BREP face count failed")
}

pub(crate) fn rgm_brep_shell_count_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_any_ref(state, brep)?;
        write_out(out_count, u32::try_from(brep_data.shells.len()).unwrap_or(u32::MAX))
    });

    finish(session, result, "BREP shell count failed")
}

pub(crate) fn rgm_brep_solid_count_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_any_ref(state, brep)?;
        write_out(out_count, u32::try_from(brep_data.solids.len()).unwrap_or(u32::MAX))
    });

    finish(session, result, "BREP solid count failed")
}

pub(crate) fn rgm_brep_is_solid_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_is_solid: *mut bool,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_any_ref(state, brep)?;
        write_out(out_is_solid, !brep_data.solids.is_empty())
    });

    finish(session, result, "BREP solid query failed")
}

pub(crate) fn rgm_brep_face_adjacency_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    face_id: u32,
    out_face_ids: *mut u32,
    face_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_any_mut(state, brep)?;
        let fid = FaceId::from_raw(face_id);
        if fid.index() >= brep_data.faces.len() {
            return Err(RgmStatus::OutOfRange);
        }
        let adjacent = face_adjacency_cached(brep_data, fid);
        write_slice(out_face_ids, face_capacity, &adjacent, out_count)
    });

    finish(session, result, "BREP face adjacency failed")
}

pub(crate) fn rgm_brep_tessellate_to_mesh_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    options: Option<RgmSurfaceTessellationOptions>,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let mesh = {
            let brep_data = with_brep_any_ref(state, brep)?;
            tessellate_brep(state, brep_data, options)?
        };
        let handle = insert_mesh(state, mesh);
        write_out(out_mesh, handle)
    });

    finish(session, result, "BREP tessellation failed")
}

pub(crate) fn rgm_brep_from_face_object_impl(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let face_data = find_face(state, face)?.clone();
        let mut brep = BrepData::new();
        add_face_to_brep(&mut brep, &face_data)?;
        let handle = insert_brep(state, brep);
        write_out(out_brep, handle)
    });

    finish(session, result, "BREP from face object failed")
}

pub(crate) fn rgm_brep_extract_face_object_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    face_id: u32,
    out_face: *mut RgmObjectHandle,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_any_ref(state, brep)?;
        let face = face_from_brep(brep_data, face_id)?;
        let handle = insert_face(state, face);
        write_out(out_face, handle)
    });

    finish(session, result, "BREP extract face object failed")
}

// S1: rgm_brep_state now returns brep.finalized as u32 (1 = finalized, 0 = in-progress).
pub(crate) fn rgm_brep_state_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_state: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_any_ref(state, brep)?;
        write_out(out_state, brep_data.finalized as u32)
    });

    finish(session, result, "BREP state query failed")
}

pub(crate) fn rgm_brep_estimate_area_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_area: *mut f64,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        // S3: area_dirty flag removed; is_none() is the single cache indicator.
        {
            let brep_data = with_brep_any_ref(state, brep)?;
            if let Some(area) = brep_data.cache.area_estimate {
                return write_out(out_area, area);
            }
        }

        // C1: Collect surface handles first (releases borrow), then compute, then write cache.
        let face_surfaces = {
            let brep_data = with_brep_any_ref(state, brep)?;
            brep_data.faces.iter().map(|face| face.surface).collect::<Vec<_>>()
        };

        let mut area = 0.0;
        for surface_handle in face_surfaces {
            if let Ok(surface) = find_surface(state, surface_handle) {
                // B2: 4×4 midpoint-rule integration of ||∂S/∂u × ∂S/∂v|| over the UV domain.
                // More accurate than the two-corner parallelogram estimate.
                // TODO(v2): account for trim loop clipping when computing trimmed area.
                let n = 4usize;
                let du = (surface.core.u_end - surface.core.u_start) / n as f64;
                let dv = (surface.core.v_end - surface.core.v_start) / n as f64;
                for i in 0..n {
                    for j in 0..n {
                        let u = surface.core.u_start + (i as f64 + 0.5) * du;
                        let v = surface.core.v_start + (j as f64 + 0.5) * dv;
                        if let Ok(eval) = eval_nurbs_surface_uv_unchecked(
                            &surface.core,
                            RgmUv2 { u, v },
                        ) {
                            area += v3::norm(v3::cross(eval.du, eval.dv)) * du * dv;
                        }
                    }
                }
            }
        }

        // S3: no area_dirty flag to reset
        let brep_data = with_brep_any_mut(state, brep)?;
        brep_data.cache.area_estimate = Some(area);
        write_out(out_area, area)
    });

    finish(session, result, "BREP area estimate failed")
}

pub(crate) fn rgm_brep_save_native_impl(
    session: RgmKernelHandle,
    brep: RgmObjectHandle,
    out_bytes: *mut u8,
    byte_capacity: u32,
    out_count: *mut u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let brep_data = with_brep_any_ref(state, brep)?;
        let bytes = encode_native_brep(brep_data)?;
        write_slice(out_bytes, byte_capacity, &bytes, out_count)
    });

    finish(session, result, "BREP native save failed")
}

pub(crate) fn rgm_brep_load_native_impl(
    session: RgmKernelHandle,
    bytes: *const u8,
    byte_count: usize,
    out_brep: *mut RgmObjectHandle,
) -> RgmStatus {
    let bytes = match read_bytes(bytes, byte_count) {
        Ok(value) => value,
        Err(status) => return map_err_with_session(session, status, "Invalid BREP native bytes"),
    };

    let result = with_session_mut(session, |state| {
        let brep = decode_native_brep(bytes)?;
        for face in brep.faces.iter() {
            let _ = find_surface(state, face.surface)?;
        }
        // B3: Validate after decoding; reject BREPs with Error-severity issues.
        let report = validate_brep_data(&brep);
        if report_has_errors(&report) {
            return Err(RgmStatus::InvalidInput);
        }
        let handle = insert_brep(state, brep);
        write_out(out_brep, handle)
    });

    finish(session, result, "BREP native load failed")
}
