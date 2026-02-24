fn default_bounds_options() -> RgmBoundsOptions {
    RgmBoundsOptions {
        mode: RgmBoundsMode::Fast,
        sample_budget: 0,
        padding: 0.0,
    }
}

fn sanitize_bounds_options(options: Option<RgmBoundsOptions>) -> RgmBoundsOptions {
    let mut value = options.unwrap_or_else(default_bounds_options);
    if !value.padding.is_finite() || value.padding < 0.0 {
        value.padding = 0.0;
    }
    value.sample_budget = value.sample_budget.min(1_000_000);
    value
}

fn quantize_padding(padding: f64) -> i64 {
    if !padding.is_finite() {
        return 0;
    }
    (padding.max(0.0) * 1_000_000_000.0).round() as i64
}

fn sample_bucket(sample_budget: u32) -> u32 {
    if sample_budget == 0 {
        return 0;
    }
    let mut bucket = 1_u32;
    while bucket < sample_budget {
        let next = bucket.saturating_mul(2);
        if next <= bucket {
            return u32::MAX;
        }
        bucket = next;
    }
    bucket
}

fn bounds_cache_key(options: RgmBoundsOptions) -> BoundsCacheKey {
    BoundsCacheKey {
        mode: options.mode,
        sample_bucket: sample_bucket(options.sample_budget),
        padding_quantized: quantize_padding(options.padding),
    }
}

fn curve_fast_points_recursive(
    state: &SessionState,
    curve: &CurveData,
    visited: &mut std::collections::HashSet<u64>,
) -> Result<Vec<RgmPoint3>, RgmStatus> {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        return Ok(nurbs.core.control_points.clone());
    }

    match curve {
        CurveData::Polycurve(poly) => {
            let mut out = Vec::new();
            for segment in &poly.segments {
                if !visited.insert(segment.curve.0) {
                    continue;
                }
                let child = find_curve(state, segment.curve)?;
                out.extend(curve_fast_points_recursive(state, child, visited)?);
                visited.remove(&segment.curve.0);
            }
            if out.is_empty() {
                out.push(curve_point_at_normalized_data(state, curve, 0.0)?);
                out.push(curve_point_at_normalized_data(state, curve, 1.0)?);
            }
            Ok(out)
        }
        _ => Err(RgmStatus::InternalError),
    }
}

fn curve_fast_points(state: &SessionState, curve: &CurveData) -> Result<Vec<RgmPoint3>, RgmStatus> {
    curve_fast_points_recursive(state, curve, &mut std::collections::HashSet::new())
}

fn curve_optimal_points(
    state: &SessionState,
    curve: &CurveData,
    sample_budget: u32,
) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let sample_count = if sample_budget == 0 {
        320
    } else {
        sample_budget.clamp(32, 4096) as usize
    };
    let mut points = Vec::with_capacity(sample_count + 1);
    for idx in 0..=sample_count {
        let t = (idx as f64) / (sample_count as f64);
        points.push(curve_point_at_normalized_data(state, curve, t)?);
    }
    points.extend(curve_fast_points(state, curve)?);
    Ok(points)
}

fn surface_fast_points(surface: &SurfaceData) -> Result<Vec<RgmPoint3>, RgmStatus> {
    if surface.core.control_points.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }
    Ok(surface
        .core
        .control_points
        .iter()
        .map(|point| matrix_apply_point(surface.transform, *point))
        .collect())
}

fn surface_optimal_points(surface: &SurfaceData, sample_budget: u32) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let mut points = surface_fast_points(surface)?;
    let budget = if sample_budget == 0 {
        1600
    } else {
        sample_budget.clamp(64, 16_384)
    };
    let grid = ((budget as f64).sqrt().round() as usize).clamp(8, 128);
    for iu in 0..=grid {
        let u = (iu as f64) / (grid as f64);
        for iv in 0..=grid {
            let v = (iv as f64) / (grid as f64);
            let frame = eval_surface_data_normalized(surface, RgmUv2 { u, v })?;
            points.push(frame.point);
        }
    }
    Ok(points)
}

fn mesh_aabb_world_corners(mesh: &MeshData) -> Result<[RgmPoint3; 8], RgmStatus> {
    let local = crate::math::bounds::aabb_from_points(&mesh.vertices)?;
    let local_corners = crate::math::bounds::aabb_corners(local);
    Ok(local_corners.map(|corner| matrix_apply_point(mesh.transform, corner)))
}

fn mesh_sampled_points(mesh: &MeshData, max_samples: usize) -> Vec<RgmPoint3> {
    if mesh.vertices.is_empty() {
        return Vec::new();
    }
    if mesh.vertices.len() <= max_samples.max(1) {
        return mesh
            .vertices
            .iter()
            .map(|point| matrix_apply_point(mesh.transform, *point))
            .collect();
    }
    let step = (mesh.vertices.len() as f64 / max_samples.max(1) as f64).ceil() as usize;
    let mut out = Vec::with_capacity(max_samples.max(1));
    for idx in (0..mesh.vertices.len()).step_by(step.max(1)) {
        out.push(matrix_apply_point(mesh.transform, mesh.vertices[idx]));
    }
    out
}

fn mesh_fast_points(mesh: &MeshData, sample_budget: u32) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let max_samples = if sample_budget == 0 {
        768
    } else {
        sample_budget.clamp(32, 8_192) as usize
    };
    let corners = mesh_aabb_world_corners(mesh)?;
    let mut points = corners.to_vec();
    points.extend(mesh_sampled_points(mesh, max_samples));
    Ok(points)
}

fn mesh_optimal_points(mesh: &MeshData, sample_budget: u32) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let max_samples = if sample_budget == 0 {
        4096
    } else {
        sample_budget.clamp(128, 32_768) as usize
    };
    let corners = mesh_aabb_world_corners(mesh)?;
    let mut points = corners.to_vec();
    points.extend(mesh_sampled_points(mesh, max_samples));
    Ok(points)
}

fn surface_points_aabb(surface: &SurfaceData) -> Result<RgmAabb3, RgmStatus> {
    let points = surface_fast_points(surface)?;
    crate::math::bounds::aabb_from_points(&points)
}

fn brep_fast_points(state: &mut SessionState, brep_handle: RgmObjectHandle) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let (faces, mut updates): (Vec<(usize, RgmObjectHandle, Option<RgmAabb3>)>, Vec<(usize, RgmAabb3)>) = {
        let brep = find_brep(state, brep_handle)?;
        if brep.faces.is_empty() {
            return Err(RgmStatus::InvalidInput);
        }
        (
            brep.faces
                .iter_enumerated()
                .map(|(face_id, face)| {
                    (
                        face_id.index(),
                        face.surface,
                        face.bbox.map(|bbox| RgmAabb3 {
                            min: bbox.min,
                            max: bbox.max,
                        }),
                    )
                })
                .collect(),
            Vec::new(),
        )
    };

    let mut points = Vec::with_capacity(faces.len() * 8);
    for (face_idx, surface_handle, cached) in faces {
        let aabb = if let Some(value) = cached {
            value
        } else {
            let surface = find_surface(state, surface_handle)?;
            let value = surface_points_aabb(surface)?;
            updates.push((face_idx, value));
            value
        };
        points.extend(crate::math::bounds::aabb_corners(aabb));
    }

    if !updates.is_empty() {
        let brep = find_brep_mut(state, brep_handle)?;
        for (face_idx, aabb) in updates {
            if let Some(face) = brep
                .faces
                .as_raw_slice_mut()
                .get_mut(face_idx)
            {
                face.bbox = Some(crate::elements::brep::types::BrepAabb3 {
                    min: aabb.min,
                    max: aabb.max,
                });
            }
        }
    }

    Ok(points)
}

fn brep_optimal_points(
    state: &mut SessionState,
    brep_handle: RgmObjectHandle,
    sample_budget: u32,
) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let mut points = brep_fast_points(state, brep_handle)?;
    let surfaces: Vec<RgmObjectHandle> = {
        let brep = find_brep(state, brep_handle)?;
        brep.faces.iter().map(|face| face.surface).collect()
    };
    if surfaces.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }

    let total_budget = if sample_budget == 0 {
        2048
    } else {
        sample_budget.clamp(128, 65_536)
    };
    let per_face_budget = (total_budget / (surfaces.len() as u32)).max(16);
    let grid = ((per_face_budget as f64).sqrt().round() as usize).clamp(4, 64);

    for surface_handle in surfaces {
        let surface = find_surface(state, surface_handle)?;
        for iu in 0..=grid {
            let u = (iu as f64) / (grid as f64);
            for iv in 0..=grid {
                let v = (iv as f64) / (grid as f64);
                let frame = eval_surface_data_normalized(surface, RgmUv2 { u, v })?;
                points.push(frame.point);
            }
        }
    }

    Ok(points)
}

fn compute_bounds_for_object(
    state: &mut SessionState,
    object: RgmObjectHandle,
    options: RgmBoundsOptions,
) -> Result<RgmBounds3, RgmStatus> {
    let mode = options.mode;
    let kind = match state.objects.get(&object.0) {
        Some(GeometryObject::Curve(_)) => 0_u8,
        Some(GeometryObject::Surface(_)) => 1_u8,
        Some(GeometryObject::Mesh(_)) => 2_u8,
        Some(GeometryObject::Brep(_)) => 3_u8,
        Some(GeometryObject::Face(_)) | Some(GeometryObject::Intersection(_)) => 4_u8,
        None => 255_u8,
    };

    match kind {
        0 => {
            let curve = find_curve(state, object)?;
            let points = if mode == RgmBoundsMode::Fast {
                curve_fast_points(state, curve)?
            } else {
                curve_optimal_points(state, curve, options.sample_budget)?
            };
            if points.is_empty() {
                return Err(RgmStatus::InvalidInput);
            }
            let abs_tol = curve_abs_tol(state, curve).unwrap_or(1e-9).max(1e-9);
            crate::math::bounds::compute_bounds_from_points(&points, mode, options.padding + abs_tol)
        }
        1 => {
            let surface = find_surface(state, object)?;
            let points = if mode == RgmBoundsMode::Fast {
                surface_fast_points(surface)?
            } else {
                surface_optimal_points(surface, options.sample_budget)?
            };
            crate::math::bounds::compute_bounds_from_points(&points, mode, options.padding)
        }
        2 => {
            let mesh = find_mesh(state, object)?;
            let points = if mode == RgmBoundsMode::Fast {
                mesh_fast_points(mesh, options.sample_budget)?
            } else {
                mesh_optimal_points(mesh, options.sample_budget)?
            };
            crate::math::bounds::compute_bounds_from_points(&points, mode, options.padding)
        }
        3 => {
            let points = if mode == RgmBoundsMode::Fast {
                brep_fast_points(state, object)?
            } else {
                brep_optimal_points(state, object, options.sample_budget)?
            };
            crate::math::bounds::compute_bounds_from_points(&points, mode, options.padding)
        }
        4 => Err(RgmStatus::InvalidInput),
        _ => Err(RgmStatus::NotFound),
    }
}

fn rgm_object_compute_bounds_impl(
    session: RgmKernelHandle,
    object: RgmObjectHandle,
    options: Option<RgmBoundsOptions>,
    out_bounds: *mut RgmBounds3,
) -> RgmStatus {
    let options = sanitize_bounds_options(options);
    let key = bounds_cache_key(options);
    let result = with_session_mut(session, |state| {
        if let Some(cached) = state.bounds_cache.get(&object.0) {
            if cached.key == key {
                return write_out(out_bounds, cached.bounds);
            }
        }

        let bounds = compute_bounds_for_object(state, object, options)?;
        state
            .bounds_cache
            .insert(object.0, BoundsCacheEntry { key, bounds });
        write_out(out_bounds, bounds)
    });

    match result {
        Ok(()) => RgmStatus::Ok,
        Err(status) => map_err_with_session(session, status, "Object bounds computation failed"),
    }
}
